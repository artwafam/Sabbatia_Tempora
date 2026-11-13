//! The card figures as paper puppets: cut-out parts from the 1910 cards,
//! pinned at their joints.
//!
//! Motion is deliberately puppet-like: stiff limbs rotate about pins with
//! eased, slightly overshooting curves, and hanging things (the Hermit's
//! lantern, the Fool's bag) swing as damped pendulums. Between gestures the
//! figure holds still and nothing repaints.

use std::f32::consts::{PI, TAU};
use std::time::Duration;

use eframe::egui::{self, Color32, Mesh, Painter, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};
use sabbatia::companion::{Character, Mood};
use sabbatia::dialogue::Cue;
use serde::Deserialize;

use super::assets;

/// Frame interval while a gesture plays: 20 fps, a stop-motion rate that
/// suits a paper puppet and keeps CPU use low.
const FRAME: Duration = Duration::from_millis(50);
const MOOD_TRANSITION: f32 = 0.9;
const MAX_PARTS: usize = 8;

struct Art {
    rig: &'static str,
    parts: &'static [(&'static str, &'static [u8])],
}

const HERMIT: Art = Art {
    rig: include_str!("../../assets/hermit/rig.toml"),
    parts: &[
        ("body", include_bytes!("../../assets/hermit/body.png")),
        ("head", include_bytes!("../../assets/hermit/head.png")),
        ("arm", include_bytes!("../../assets/hermit/arm.png")),
        ("staff", include_bytes!("../../assets/hermit/staff.png")),
        ("lantern", include_bytes!("../../assets/hermit/lantern.png")),
    ],
};

const FOOL: Art = Art {
    rig: include_str!("../../assets/fool/rig.toml"),
    parts: &[
        ("staff_arm", include_bytes!("../../assets/fool/staff_arm.png")),
        ("leg", include_bytes!("../../assets/fool/leg.png")),
        ("body", include_bytes!("../../assets/fool/body.png")),
        ("head", include_bytes!("../../assets/fool/head.png")),
        ("rose_arm", include_bytes!("../../assets/fool/rose_arm.png")),
        ("bag", include_bytes!("../../assets/fool/bag.png")),
        ("dog", include_bytes!("../../assets/fool/dog.png")),
    ],
};

fn art(who: Character) -> &'static Art {
    match who {
        Character::Hermit => &HERMIT,
        Character::Fool => &FOOL,
    }
}

#[derive(Deserialize)]
struct RigFile {
    width: f32,
    height: f32,
    light: Option<[f32; 2]>,
    light_part: Option<String>,
    part: Vec<RigPart>,
}

#[derive(Deserialize)]
struct RigPart {
    name: String,
    x: f32,
    y: f32,
    pivot: Option<[f32; 2]>,
    parent: Option<String>,
    #[serde(default)]
    hang: bool,
}

fn parse(who: Character) -> RigFile {
    toml::from_str(art(who).rig).expect("embedded rig is valid TOML")
}

/// Width ÷ height of a character's figure, without loading textures.
pub fn aspect(who: Character) -> f32 {
    let r = parse(who);
    r.width / r.height
}

struct Part {
    name: String,
    tex: TextureHandle,
    offset: Vec2,
    size: Vec2,
    pivot: Option<Pos2>,
    parent: Option<usize>,
    hang: bool,
}

/// Per-part angle (degrees, positive = clockwise on screen) and lift (rig
/// pixels upward), plus effects. For a hanging part the angle is its world
/// angle, independent of its parent (gravity).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pose {
    angle: [f32; MAX_PARTS],
    lift: [f32; MAX_PARTS],
    glow: f32,
    bob: f32,
}

impl Pose {
    fn lerp(a: &Pose, b: &Pose, t: f32) -> Pose {
        let l = |x: f32, y: f32| x + (y - x) * t;
        let mut p = Pose { glow: l(a.glow, b.glow), bob: l(a.bob, b.bob), ..Pose::default() };
        for i in 0..MAX_PARTS {
            p.angle[i] = l(a.angle[i], b.angle[i]);
            p.lift[i] = l(a.lift[i], b.lift[i]);
        }
        p
    }

    fn add(&mut self, o: &Pose) {
        for i in 0..MAX_PARTS {
            self.angle[i] += o.angle[i];
            self.lift[i] += o.lift[i];
        }
        self.glow = (self.glow + o.glow).min(1.0);
        self.bob += o.bob;
    }

    fn scale(&mut self, k: f32) {
        for i in 0..MAX_PARTS {
            self.angle[i] *= k;
            self.lift[i] *= k;
        }
        self.glow *= k;
        self.bob *= k;
    }
}

/// A move described by part names: (name, angle, lift).
type Moves<'a> = &'a [(&'a str, f32, f32)];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Gesture {
    // The Hermit
    Swing,
    Nod,
    Lift,
    // The Fool
    Step,
    LookUp,
    Offer,
    DogHop,
    BagSwing,
    /// Head lowered in thought; the dog lowers its head too.
    Ponder,
}

impl Gesture {
    #[cfg(test)]
    const HERMIT: [Gesture; 3] = [Gesture::Swing, Gesture::Nod, Gesture::Lift];
    #[cfg(test)]
    const FOOL: [Gesture; 6] =
        [Gesture::Step, Gesture::LookUp, Gesture::Offer, Gesture::DogHop, Gesture::BagSwing, Gesture::Ponder];

    fn duration(self) -> f32 {
        match self {
            Gesture::Swing => 3.4,
            Gesture::Nod => 1.8,
            Gesture::Lift => 3.0,
            Gesture::Step => 1.6,
            Gesture::LookUp => 2.2,
            Gesture::Offer => 2.4,
            Gesture::DogHop => 1.4,
            Gesture::BagSwing => 3.2,
            Gesture::Ponder => 2.6,
        }
    }
}

/// Damped pendulum, starting at its maximum (degrees).
fn pendulum(t: f32, amplitude: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    amplitude * (-t / 1.1).exp() * (TAU * t / 1.25).sin()
}

/// Rise-and-return with a small overshoot, like a limb pulled by a string.
fn puppet_bell(u: f32) -> f32 {
    let s = (PI * u).sin();
    let overshoot = 0.08 * (2.0 * PI * u).sin().max(0.0) * (1.0 - u);
    (s * s + overshoot).max(0.0)
}

fn bump(u: f32) -> f32 {
    (PI * u).sin().max(0.0)
}

/// Two little hops of decreasing height.
fn hops(u: f32) -> f32 {
    (TAU * u).sin().abs() * (1.0 - 0.35 * u)
}

fn smoothstep(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Tiny xorshift PRNG for gesture timing (no external crate).
struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Developer hook: `SABBATIA_PREVIEW=happy|sleepy|idle` forces a mood.
fn preview_mood() -> Option<Mood> {
    match std::env::var("SABBATIA_PREVIEW").ok()?.as_str() {
        "happy" => Some(Mood::Happy),
        "sleepy" => Some(Mood::Sleepy),
        "idle" => Some(Mood::Idle),
        _ => None,
    }
}

pub struct Puppet {
    who: Character,
    size: Vec2,
    parts: Vec<Part>,
    light: Option<(usize, Pos2)>,
    mood: Mood,
    from: Pose,
    mood_since: f64,
    gesture: Option<(Gesture, f64)>,
    next_gesture_at: f64,
    rng: Rng,
}

impl Puppet {
    pub fn load(ctx: &egui::Context, who: Character, mood: Mood) -> Result<Puppet, String> {
        let rig = parse(who);
        let names: Vec<&str> = rig.part.iter().map(|p| p.name.as_str()).collect();
        if names.len() > MAX_PARTS {
            return Err(format!("rig has {} parts; at most {MAX_PARTS} supported", names.len()));
        }
        let mut parts = Vec::with_capacity(rig.part.len());
        for (i, p) in rig.part.iter().enumerate() {
            let bytes = art(who).parts.iter().find(|(n, _)| *n == p.name).map(|(_, b)| *b).ok_or("missing image")?;
            let tex = assets::texture(ctx, &format!("{}-{}", who.config_value(), p.name), bytes);
            let [w, h] = tex.size();
            let parent = match &p.parent {
                Some(name) => Some(
                    names[..i].iter().position(|n| n == name).ok_or(format!("{} must follow its parent", p.name))?,
                ),
                None => None,
            };
            parts.push(Part {
                name: p.name.clone(),
                tex,
                offset: vec2(p.x, p.y),
                size: vec2(w as f32, h as f32),
                pivot: p.pivot.map(|[x, y]| pos2(x, y)),
                parent,
                hang: p.hang,
            });
        }
        let light = match (&rig.light_part, rig.light) {
            (Some(name), Some([x, y])) => names.iter().position(|n| n == name).map(|i| (i, pos2(x, y))),
            _ => None,
        };
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1;
        let mood = preview_mood().unwrap_or(mood);
        let mut p = Puppet {
            who,
            size: vec2(rig.width, rig.height),
            parts,
            light,
            mood,
            from: Pose::default(),
            mood_since: f64::NEG_INFINITY,
            gesture: None,
            next_gesture_at: 5.0,
            rng: Rng(seed),
        };
        p.from = p.mood_pose(mood);
        Ok(p)
    }

    fn pose(&self, moves: Moves<'_>, glow: f32, bob: f32) -> Pose {
        let mut p = Pose { glow, bob, ..Pose::default() };
        for &(name, angle, lift) in moves {
            if let Some(i) = self.parts.iter().position(|q| q.name == name) {
                p.angle[i] = angle;
                p.lift[i] = lift;
            }
        }
        p
    }

    fn mood_pose(&self, m: Mood) -> Pose {
        match (self.who, m) {
            (Character::Hermit, Mood::Idle) => self.pose(&[], 0.55, 0.0),
            // Lantern raised high, head lifted.
            (Character::Hermit, Mood::Happy) => self.pose(&[("head", 3.0, 0.0), ("arm", 8.0, 0.0)], 1.0, 0.0),
            // Head bowed, lantern lowered and dim.
            (Character::Hermit, Mood::Sleepy) => self.pose(&[("head", -4.0, 0.0), ("arm", -5.0, 0.0)], 0.22, 0.0),
            (Character::Fool, Mood::Idle) => self.pose(&[], 0.0, 0.0),
            // Rose held up, face to the sky, the dog up on its toes.
            (Character::Fool, Mood::Happy) => {
                self.pose(&[("rose_arm", -7.0, 0.0), ("head", 3.0, 0.0), ("dog", 3.0, 2.0)], 0.0, 0.0)
            }
            // Head down, bindle and rose lowered.
            (Character::Fool, Mood::Sleepy) => {
                self.pose(&[("head", -6.0, 0.0), ("staff_arm", 3.0, 0.0), ("rose_arm", 6.0, 0.0)], 0.0, 0.0)
            }
        }
    }

    /// Offsets added to the base pose `t` seconds into a gesture. Every
    /// gesture eases fully back to rest over its last 15%, so ending one
    /// never makes the figure snap.
    fn gesture_pose(&self, g: Gesture, t: f32) -> Pose {
        let d = g.duration();
        let u = (t / d).clamp(0.0, 1.0);
        let b = puppet_bell(u);
        let mut p = match g {
            Gesture::Swing => self.pose(&[("lantern", pendulum(t, 9.0), 0.0)], 0.0, bump(u) * 0.8),
            Gesture::Nod => self.pose(&[("head", -5.0 * b, 0.0)], 0.0, b * 1.2),
            Gesture::Lift => {
                // The lantern lags behind the hand, then swings on the way down.
                let swing = if u > 0.45 { pendulum(t - 0.45 * d, 6.0) } else { -3.0 * b };
                self.pose(&[("arm", 9.0 * b, 0.0), ("head", 2.5 * b, 0.0), ("lantern", swing, 0.0)], 0.25 * b, 0.0)
            }
            Gesture::Step => {
                self.pose(&[("leg", -9.0 * b, 0.0), ("staff_arm", -1.5 * b, 0.0), ("bag", pendulum(t - 0.3, 5.0), 0.0)], 0.0, 1.5 * b)
            }
            Gesture::LookUp => self.pose(&[("head", 5.0 * b, 0.0), ("rose_arm", -3.0 * b, 0.0)], 0.0, 0.0),
            Gesture::Offer => self.pose(&[("rose_arm", -10.0 * b, 0.0), ("head", 2.0 * b, 0.0)], 0.0, 0.0),
            Gesture::DogHop => self.pose(&[("dog", 4.0 * hops(u), 9.0 * hops(u))], 0.0, 0.0),
            Gesture::Ponder => self.pose(&[("head", -6.0 * b, 0.0), ("dog", -3.0 * b, 0.0), ("rose_arm", 3.0 * b, 0.0)], 0.0, 0.0),
            Gesture::BagSwing => {
                let heave = puppet_bell((u * 3.0).min(1.0));
                self.pose(&[("bag", pendulum(t, 11.0), 0.0), ("staff_arm", -2.0 * heave, 0.0)], 0.0, 0.0)
            }
        };
        p.scale(1.0 - smoothstep((u - 0.85) / 0.15));
        p
    }

    fn pick_gesture(&mut self) -> Gesture {
        let r = self.rng.next_f32();
        match (self.who, self.mood) {
            (Character::Hermit, Mood::Sleepy) if r < 0.6 => Gesture::Nod,
            (Character::Hermit, _) if r < 0.4 => Gesture::Swing,
            (Character::Hermit, _) if r < 0.7 => Gesture::Nod,
            (Character::Hermit, _) => Gesture::Lift,
            (Character::Fool, Mood::Happy) if r < 0.35 => Gesture::DogHop,
            (Character::Fool, _) if r < 0.2 => Gesture::DogHop,
            (Character::Fool, _) if r < 0.4 => Gesture::Step,
            (Character::Fool, _) if r < 0.6 => Gesture::LookUp,
            (Character::Fool, _) if r < 0.8 => Gesture::BagSwing,
            (Character::Fool, _) => Gesture::Offer,
        }
    }

    /// Change mood with a short eased transition.
    pub fn set_mood(&mut self, mood: Mood, now: f64) {
        let mood = preview_mood().unwrap_or(mood);
        if mood != self.mood {
            self.from = self.base_pose(now);
            self.mood = mood;
            self.mood_since = now;
        }
    }

    /// Act out a line of dialogue. `Still` stops any gesture: a
    /// contemplative pause. The next idle gesture waits a little longer.
    pub fn cue(&mut self, cue: Cue, now: f64) {
        let g = match (self.who, cue) {
            (_, Cue::Still) => None,
            (Character::Hermit, Cue::Raise | Cue::Hop | Cue::LookUp) => Some(Gesture::Lift),
            (Character::Hermit, Cue::Bow | Cue::Ponder) => Some(Gesture::Nod),
            (Character::Hermit, Cue::Swing | Cue::Step) => Some(Gesture::Swing),
            (Character::Fool, Cue::Raise) => Some(Gesture::Offer),
            (Character::Fool, Cue::Bow | Cue::Ponder) => Some(Gesture::Ponder),
            (Character::Fool, Cue::Swing) => Some(Gesture::BagSwing),
            (Character::Fool, Cue::Step) => Some(Gesture::Step),
            (Character::Fool, Cue::Hop) => Some(Gesture::DogHop),
            (Character::Fool, Cue::LookUp) => Some(Gesture::LookUp),
        };
        self.gesture = g.map(|g| (g, now));
        self.next_gesture_at = self.next_gesture_at.max(now + 20.0);
    }

    fn base_pose(&self, now: f64) -> Pose {
        let t = ((now - self.mood_since) as f32 / MOOD_TRANSITION).clamp(0.0, 1.0);
        Pose::lerp(&self.from, &self.mood_pose(self.mood), smoothstep(t))
    }

    /// The pose at `now` and how long until the next frame is needed.
    pub fn update(&mut self, now: f64, animate: bool) -> (Pose, Option<Duration>) {
        let mut pose = self.base_pose(now);
        let transitioning = (now - self.mood_since) < f64::from(MOOD_TRANSITION);
        if !animate {
            self.gesture = None;
            return (pose, transitioning.then_some(FRAME));
        }
        if self.gesture.is_none() && now >= self.next_gesture_at {
            let g = self.pick_gesture();
            self.gesture = Some((g, now));
        }
        if let Some((g, start)) = self.gesture {
            let t = (now - start) as f32;
            if t >= g.duration() {
                self.gesture = None;
                let (lo, hi) = if self.mood == Mood::Sleepy { (35.0, 70.0) } else { (20.0, 45.0) };
                self.next_gesture_at = now + f64::from(lo + (hi - lo) * self.rng.next_f32());
            } else {
                pose.add(&self.gesture_pose(g, t));
                if self.light.is_some() {
                    // The flame flickers while he moves.
                    pose.glow *= 0.92 + 0.08 * (now as f32 * 23.0).sin();
                }
                return (pose, Some(FRAME));
            }
        }
        if transitioning {
            return (pose, Some(FRAME));
        }
        (pose, Some(Duration::from_secs_f64((self.next_gesture_at - now).max(0.0))))
    }

    /// Draw into `rect`, which should have this puppet's aspect ratio.
    pub fn paint(&self, painter: &Painter, rect: Rect, pose: &Pose) {
        let s = rect.height() / self.size.y;
        let origin = rect.min + vec2(0.0, -pose.bob * s);
        let to_screen = |p: Pos2| origin + p.to_vec2() * s;

        // World transform of every part, parents first.
        let mut world = [Affine::IDENTITY; MAX_PARTS];
        let mut total = [0.0f32; MAX_PARTS];
        for (i, part) in self.parts.iter().enumerate() {
            let (parent, parent_angle) = part.parent.map_or((Affine::IDENTITY, 0.0), |j| (world[j], total[j]));
            let angle = if part.hang { pose.angle[i] - parent_angle } else { pose.angle[i] };
            let local = match part.pivot {
                Some(pv) => Affine::translate(vec2(0.0, -pose.lift[i])).then_after(Affine::rotate_about(pv, angle)),
                None => Affine::IDENTITY,
            };
            world[i] = parent.then_after(local);
            total[i] = parent_angle + angle;
        }

        // Ground shadow under the feet.
        let feet = to_screen(pos2(self.size.x * 0.55, self.size.y - 2.0));
        soft_ellipse(painter, feet, vec2(self.size.x * 0.36 * s, 4.0 * s), Color32::from_black_alpha(40));

        // Lantern light, behind everything.
        if let Some((i, at)) = self.light {
            let a = (pose.glow.clamp(0.0, 1.0) * 120.0) as u8;
            let r = 34.0 * s;
            soft_ellipse(painter, to_screen(world[i].apply(at)), vec2(r, r), Color32::from_rgba_unmultiplied(255, 214, 90, a));
        }

        for (i, part) in self.parts.iter().enumerate() {
            let corners = [
                part.offset.to_pos2(),
                (part.offset + vec2(part.size.x, 0.0)).to_pos2(),
                (part.offset + part.size).to_pos2(),
                (part.offset + vec2(0.0, part.size.y)).to_pos2(),
            ];
            let uv = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)];
            let mut mesh = Mesh::with_texture(part.tex.id());
            for (c, t) in corners.iter().zip(uv) {
                mesh.vertices.push(egui::epaint::Vertex { pos: to_screen(world[i].apply(*c)), uv: t, color: Color32::WHITE });
            }
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);
            painter.add(mesh);
        }
    }
}

/// 2-D affine map: p ↦ M·p + t.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Affine {
    m: [f32; 4],
    t: Vec2,
}

impl Affine {
    const IDENTITY: Affine = Affine { m: [1.0, 0.0, 0.0, 1.0], t: Vec2::ZERO };

    fn translate(d: Vec2) -> Affine {
        Affine { t: d, ..Affine::IDENTITY }
    }

    /// Rotation by `deg` (clockwise on screen) about `c`.
    fn rotate_about(c: Pos2, deg: f32) -> Affine {
        let (s, k) = deg.to_radians().sin_cos();
        let m = [k, -s, s, k];
        // p ↦ R(p − c) + c
        let t = c.to_vec2() - vec2(m[0] * c.x + m[1] * c.y, m[2] * c.x + m[3] * c.y);
        Affine { m, t }
    }

    fn apply(&self, p: Pos2) -> Pos2 {
        pos2(self.m[0] * p.x + self.m[1] * p.y + self.t.x, self.m[2] * p.x + self.m[3] * p.y + self.t.y)
    }

    /// `self ∘ other`: apply `other` first.
    fn then_after(&self, other: Affine) -> Affine {
        let a = &self.m;
        let b = &other.m;
        Affine {
            m: [a[0] * b[0] + a[1] * b[2], a[0] * b[1] + a[1] * b[3], a[2] * b[0] + a[3] * b[2], a[2] * b[1] + a[3] * b[3]],
            t: vec2(
                a[0] * other.t.x + a[1] * other.t.y + self.t.x,
                a[2] * other.t.x + a[3] * other.t.y + self.t.y,
            ),
        }
    }
}

/// A radial gradient ellipse fading to transparent at its edge.
fn soft_ellipse(painter: &Painter, center: Pos2, radius: Vec2, color: Color32) {
    const N: u32 = 32;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for i in 0..N {
        let a = TAU * i as f32 / N as f32;
        mesh.colored_vertex(center + vec2(a.cos() * radius.x, a.sin() * radius.y), Color32::TRANSPARENT);
    }
    for i in 0..N {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % N);
    }
    painter.add(mesh);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rigs_parse_and_name_every_part() {
        for who in Character::ALL {
            let rig = parse(who);
            assert!(rig.part.len() <= MAX_PARTS);
            assert_eq!(rig.part.len(), art(who).parts.len());
            for (i, p) in rig.part.iter().enumerate() {
                let (_, bytes) = art(who).parts.iter().find(|(n, _)| *n == p.name).unwrap();
                let img = assets::decode_png(bytes).unwrap();
                assert!(p.x + img.size[0] as f32 <= rig.width + 1.0, "{who:?} {} overflows", p.name);
                assert!(p.y + img.size[1] as f32 <= rig.height + 1.0, "{who:?} {} overflows", p.name);
                if let Some(parent) = &p.parent {
                    assert!(rig.part[..i].iter().any(|q| &q.name == parent), "{} precedes its parent", p.name);
                }
                if p.hang {
                    assert!(p.pivot.is_some() && p.parent.is_some());
                }
            }
        }
    }

    #[test]
    fn affine_rotation_about_pivot() {
        let r = Affine::rotate_about(pos2(10.0, 10.0), 90.0);
        let p = r.apply(pos2(20.0, 10.0));
        assert!((p.x - 10.0).abs() < 1e-4 && (p.y - 20.0).abs() < 1e-4, "{p:?}"); // clockwise on screen
        assert_eq!(r.apply(pos2(10.0, 10.0)), pos2(10.0, 10.0));
        let moved = Affine::translate(vec2(5.0, 0.0)).then_after(r);
        assert!((moved.apply(pos2(20.0, 10.0)).x - 15.0).abs() < 1e-4);
    }

    #[test]
    fn pendulum_settles() {
        assert!(pendulum(6.0, 9.0).abs() < 0.1);
        assert_eq!(pendulum(-1.0, 9.0), 0.0);
    }

    /// Gesture curves (independent of textures): every gesture moves
    /// mid-way and is fully at rest at its end.
    #[test]
    fn gestures_start_and_end_at_rest() {
        for g in Gesture::HERMIT.iter().chain(Gesture::FOOL.iter()) {
            let d = g.duration();
            let at = |t: f32| {
                let u = (t / d).clamp(0.0, 1.0);
                let fade = 1.0 - smoothstep((u - 0.85) / 0.15);
                (puppet_bell(u) + pendulum(t, 1.0).abs() + hops(u)) * fade
            };
            assert!(at(d) < 1e-4, "{g:?} does not end at rest");
            assert!(at(0.3 * d) > 0.05, "{g:?} barely moves");
        }
    }
}
