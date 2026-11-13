//! The desktop companion: a card figure standing on the desktop. Click him
//! and he talks in a speech bubble; from the bubble open the almanac (four
//! tiles with expandable details) or the settings. Drag him to move him;
//! right-click for the menu.
//!
//! The window is transparent and exactly as large as what is shown. Its
//! anchor is the figure's feet, so the bubble opens upwards without moving
//! him.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, Galley, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, Ui, UiBuilder, Vec2, ViewportCommand, WindowLevel, pos2, vec2,
};
use sabbatia::calendar::almanac::Almanac;
use sabbatia::calendar::{CrossQuarterRule, Hemisphere, lore};
use sabbatia::companion::{self, Character};
use sabbatia::config::{self, Config, WindowState};
use sabbatia::i18n::{Lang, LanguagePref};

use super::assets::Icons;
use super::desktop::Desktop;
use super::glyphs;
use super::puppet::{self, Puppet};
use super::text::Texts;
use super::theme::{self, Tint};

/// Upper bound between recomputations; also bounds staleness after suspend.
const MAX_REFRESH_INTERVAL: chrono::TimeDelta = chrono::TimeDelta::minutes(10);
/// Transparent margin around the content (room for the bubble's shadow).
pub const MARGIN: f32 = 10.0;
/// Space under the feet for the ground shadow.
pub const BELOW_FEET: f32 = 8.0;
const TAIL: f32 = 12.0;
const TALK_W: f32 = 250.0;
const PAD: f32 = 14.0;
const GAP: f32 = 8.0;
const DETAIL_MAX_HEIGHT: f32 = 300.0;
/// Tallest bubble the window makes room for (the settings or the almanac
/// with a tile open).
const BUBBLE_MAX: f32 = 600.0;
/// A talk bubble left alone closes by itself.
const TALK_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    None,
    Talk,
    Almanac,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Season,
    Moon,
    Wheel,
    Sabbat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Talk,
    Almanac,
    Settings,
    Close,
    SwitchCharacter,
    SwitchHemisphere,
    Quit,
}

pub struct SabbatiaApp {
    cfg: Config,
    lang: Lang,
    almanac: Almanac,
    text: Texts,
    lines: Vec<String>,
    line_idx: usize,
    next_refresh: DateTime<Utc>,
    panel: Panel,
    expanded: Option<Section>,
    notice: Option<String>,
    icons: Icons,
    puppet: Option<Puppet>,
    /// Bubble content height measured on the previous frame.
    bubble_h: f32,
    /// Screen position of the figure's bottom-left corner.
    foot: Option<Pos2>,
    saved_foot: Option<Pos2>,
    /// X11 connection for desktop hints, the click-through region and the
    /// pointer's screen position; `None` on Wayland.
    desktop: Option<Desktop>,
    desktop_tried: bool,
    /// Window's top-left corner on screen, as last placed.
    win_pos: Option<Pos2>,
    /// Window size last applied. It changes only with the settings, never
    /// while interacting, so nothing is resized mid-gesture.
    applied_size: Vec2,
    /// Pointer position relative to the window's corner while dragging.
    drag_offset: Option<Vec2>,
    talk_since: Instant,
    /// Desktop hints last applied for this `always_on_top` value, and how
    /// many times (they are repeated shortly after start, because requests
    /// sent before the window manager has mapped the window are ignored).
    pinned: Option<(bool, u8)>,
    started: Instant,
}

impl SabbatiaApp {
    pub fn new(cc: &eframe::CreationContext<'_>, cfg: Config, window: WindowState, notice: Option<String>) -> Self {
        theme::apply(&cc.egui_ctx);
        let lang = cfg.language.resolve();
        let now = Utc::now();
        let almanac = Almanac::compute(now, &Local, cfg.calendar_settings());
        let puppet = load_puppet(&cc.egui_ctx, cfg.character, &almanac);
        let (panel, expanded) = preview();
        let applied_size = window_size(&cfg, figure_size(&cfg));
        SabbatiaApp {
            text: Texts::build(&almanac, lang),
            lines: companion::lines(&almanac, lang, cfg.character),
            line_idx: 0,
            next_refresh: next_refresh_after(now, &almanac),
            almanac,
            lang,
            cfg,
            panel,
            expanded,
            notice,
            icons: Icons::load(&cc.egui_ctx),
            puppet,
            bubble_h: 80.0,
            foot: window.foot_x.zip(window.foot_y).map(|(x, y)| pos2(x, y)),
            saved_foot: window.foot_x.zip(window.foot_y).map(|(x, y)| pos2(x, y)),
            desktop: None,
            desktop_tried: false,
            win_pos: None,
            applied_size,
            drag_offset: None,
            talk_since: Instant::now(),
            pinned: None,
            started: Instant::now(),
        }
    }

    fn recompute(&mut self, now: DateTime<Utc>, t: f64) {
        self.almanac = Almanac::compute(now, &Local, self.cfg.calendar_settings());
        self.text = Texts::build(&self.almanac, self.lang);
        let lines = companion::lines(&self.almanac, self.lang, self.cfg.character);
        if lines != self.lines {
            self.lines = lines;
            self.line_idx = 0;
        }
        if let Some(p) = &mut self.puppet {
            p.set_mood(companion::mood(&self.almanac), t);
        }
        self.next_refresh = next_refresh_after(now, &self.almanac);
    }

    /// Persist a settings change and apply its side effects.
    fn settings_changed(&mut self, ctx: &egui::Context, before: &Config, t: f64) {
        if self.cfg == *before {
            return;
        }
        if self.cfg.always_on_top != before.always_on_top {
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(window_level(&self.cfg)));
            self.pinned = None; // re-apply the desktop hints
        }
        if self.cfg.character != before.character {
            self.puppet = load_puppet(ctx, self.cfg.character, &self.almanac);
        }
        self.notice = None;
        if self.cfg.autostart != before.autostart
            && let Err(e) = config::sync_autostart(self.cfg.autostart)
        {
            self.notice = Some(format!("Autostart: {e}"));
        }
        if let Err(e) = config::save(&self.cfg) {
            self.notice = Some(format!("{e}"));
        }
        self.lang = self.cfg.language.resolve();
        if self.cfg.calendar_settings() != before.calendar_settings()
            || self.cfg.language != before.language
            || self.cfg.character != before.character
        {
            self.recompute(Utc::now(), t);
        }
    }

    fn apply(&mut self, ctx: &egui::Context, action: Action, t: f64) {
        let before = self.cfg.clone();
        match action {
            Action::Talk => self.open_talk(),
            Action::Almanac => self.panel = Panel::Almanac,
            Action::Settings => self.panel = Panel::Settings,
            Action::Close => {
                self.panel = Panel::None;
                self.expanded = None;
            }
            Action::SwitchCharacter => {
                self.cfg.character = match self.cfg.character {
                    Character::Hermit => Character::Fool,
                    Character::Fool => Character::Hermit,
                }
            }
            Action::SwitchHemisphere => self.cfg.hemisphere = Some(self.cfg.effective_hemisphere().toggled()),
            Action::Quit => ctx.send_viewport_cmd(ViewportCommand::Close),
        }
        self.settings_changed(ctx, &before, t);
    }

    fn open_talk(&mut self) {
        if self.panel == Panel::Talk {
            self.line_idx = (self.line_idx + 1) % self.lines.len().max(1);
        }
        self.panel = Panel::Talk;
        self.talk_since = Instant::now();
    }


    fn bubble_width(&self) -> f32 {
        match self.panel {
            Panel::None => 0.0,
            Panel::Talk => TALK_W,
            Panel::Almanac | Panel::Settings => self.cfg.width,
        }
    }

    // ── bubble contents ───────────────────────────────────────────────────

    fn talk(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let u = self.lang.ui();
        let w = ui.available_width();
        let line = self.lines.get(self.line_idx).map_or("", String::as_str);
        let g = ui.painter().layout_job(glyphs::wrapped(line, theme::semibold(12.5), theme::INK, w));
        let (r, resp) = ui.allocate_exact_size(g.size(), Sense::click());
        ui.painter().galley(r.min, g, theme::INK);
        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            self.open_talk();
        }
        ui.label(RichText::new(&self.text.date_long).font(theme::semibold(11.0)).color(theme::DIM));
        ui.horizontal(|ui| {
            if link(ui, u.almanac).clicked() {
                *action = Some(Action::Almanac);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if close_button(ui, u.close).clicked() {
                    *action = Some(Action::Close);
                }
            });
        });
    }

    fn almanac_view(&mut self, ui: &mut Ui, action: &mut Option<Action>, detail_max: f32) {
        let u = self.lang.ui();
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.text.date_long).font(theme::bold(13.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if close_button(ui, u.close).clicked() {
                    *action = Some(Action::Close);
                }
            });
        });
        self.tiles(ui, action);
        self.detail(ui, detail_max);
    }

    fn tiles(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let u = self.lang.ui();
        let t = &self.text;
        let w = ui.available_width();
        let tw = ((w - GAP) / 2.0).floor();
        let painter = ui.painter().clone();
        let value_w = tw - 2.0 * TILE_PAD;

        let mk = |label: &str, tint: Tint, value: &str, sub: egui::text::LayoutJob| TileGalleys {
            label: painter.layout_job(glyphs::label_job(label, 9.5, tint.label)),
            value: painter.layout_job(glyphs::wrapped(value, theme::bold(15.0), theme::INK, value_w)),
            sub: painter.layout_job(sub),
        };
        let dim = |s: &str| glyphs::wrapped(s, theme::semibold(11.0), theme::DIM, value_w);
        let mut sabbat_sub = egui::text::LayoutJob::default();
        sabbat_sub.wrap.max_width = value_w;
        sabbat_sub.append(
            &t.sabbat_countdown,
            0.0,
            egui::TextFormat { font_id: theme::bold(11.0), color: theme::INK, ..Default::default() },
        );
        sabbat_sub.append(
            &format!(" · {}", t.sabbat_when),
            0.0,
            egui::TextFormat { font_id: theme::semibold(11.0), color: theme::DIM, ..Default::default() },
        );

        let progress = Some(self.almanac.season.progress as f32);
        let specs = [
            (Section::Season, theme::SEASON, &self.icons.season, mk(u.season, theme::SEASON, t.season_title, dim(&t.season_range)), progress),
            (Section::Moon, theme::MOON, &self.icons.moon, mk(u.moon, theme::MOON, t.moon_title, dim(&t.moon_sub)), None),
            (Section::Wheel, theme::WHEEL, &self.icons.wheel, mk(u.wheel, theme::WHEEL, &t.wheel_title, dim(&t.wheel_sub)), None),
            (Section::Sabbat, theme::SABBAT, &self.icons.sabbat, mk(u.next_sabbat, theme::SABBAT, t.sabbat_name, sabbat_sub), None),
        ];

        let mut clicked = None;
        for row in specs.chunks(2) {
            let h = row.iter().map(|(_, _, _, g, p)| tile_height(g, p.is_some())).fold(0.0, f32::max);
            let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
            for (i, (sec, tint, icon, g, progress)) in row.iter().enumerate() {
                let r = Rect::from_min_size(rect.min + vec2(i as f32 * (tw + GAP), 0.0), vec2(tw, h));
                let resp = tile(ui, r, *sec, *tint, icon, g, *progress, self.expanded == Some(*sec));
                menu_on(&resp, self.lang, action);
                if resp.clicked() {
                    clicked = Some(*sec);
                }
            }
        }
        if let Some(s) = clicked {
            self.expanded = if self.expanded == Some(s) { None } else { Some(s) };
        }
    }

    fn detail(&self, ui: &mut Ui, max_height: f32) {
        let Some(sec) = self.expanded else { return };
        let tint = match sec {
            Section::Season => theme::SEASON,
            Section::Moon => theme::MOON,
            Section::Wheel => theme::WHEEL,
            Section::Sabbat => theme::SABBAT,
        };
        let (t, u, lang, a) = (&self.text, self.lang.ui(), self.lang, &self.almanac);
        egui::Frame::new().fill(tint.fill).corner_radius(theme::TILE_RADIUS).inner_margin(12).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .id_salt("detail")
                .max_height(max_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 5.0;
                    match sec {
                        Section::Season => {
                            title(ui, &format!("{} · {}", t.season_title, lang.hemisphere(a.season.hemisphere)));
                            para(ui, t.season_description);
                            kv(ui, u.began, &t.season_began);
                            kv(ui, u.ends, &t.season_ends);
                            kv(ui, u.progress, &t.season_progress);
                            note(ui, lore::season_note(lang));
                        }
                        Section::Moon => {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
                                let lit_right = a.moon.waxing == (a.settings.hemisphere == Hemisphere::Northern);
                                glyphs::moon(ui.painter(), r.center(), 25.0, a.moon.illumination as f32, lit_right);
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 1.0;
                                    title(ui, t.moon_title);
                                    small(ui, &format!("{}: {}", u.illumination, t.moon_illumination));
                                    small(ui, &format!("{}: {}", u.lunar_age, t.moon_age));
                                });
                            });
                            kv(ui, u.new_moon_on, &t.moon_lunation);
                            phase_cards(ui, &t.moon_upcoming);
                            note(ui, t.moon_note);
                        }
                        Section::Wheel => {
                            title(ui, &t.wheel_title);
                            para(ui, t.wheel_description);
                            kv(ui, u.since, &t.wheel_since);
                            kv(ui, u.until, &t.wheel_until);
                            kv(ui, u.sun, &t.wheel_sun);
                            note(ui, lore::wheel_note(lang));
                            source(ui, lore::WHEEL_SOURCE);
                        }
                        Section::Sabbat => {
                            let s = a.next_sabbat.occurrence.sabbat;
                            let l = lore::sabbat(s, lang);
                            title(ui, t.sabbat_name);
                            ui.label(RichText::new(l.epithet).font(theme::semibold(12.0)).color(tint.label));
                            kv(ui, u.date, &t.sabbat_date);
                            kv(ui, u.dating, &t.sabbat_basis);
                            if let Some(alt) = &t.sabbat_alternative {
                                kv(ui, u.alternative, alt);
                            }
                            heading(ui, u.history, tint.label);
                            para(ui, l.history);
                            heading(ui, u.significance, tint.label);
                            para(ui, l.significance);
                            heading(ui, u.symbolism, tint.label);
                            para(ui, l.symbolism);
                            if s.is_cross_quarter() {
                                note(ui, lore::cross_quarter_note(lang));
                            }
                            heading(ui, u.sources, tint.label);
                            for src in l.sources {
                                source(ui, src);
                            }
                        }
                    }
                });
        });
    }

    fn settings(&mut self, ui: &mut Ui, action: &mut Option<Action>, t: f64) {
        let u = self.lang.ui();
        let before = self.cfg.clone();
        ui.horizontal(|ui| {
            ui.label(RichText::new(u.settings).font(theme::bold(16.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if pill(ui, u.done).clicked() {
                    *action = Some(Action::Talk);
                }
            });
        });

        section_label(ui, u.companion);
        segmented(
            ui,
            "who",
            &mut self.cfg.character,
            &[(Character::Hermit, Character::Hermit.name(self.lang)), (Character::Fool, Character::Fool.name(self.lang))],
        );
        section_label(ui, u.hemisphere);
        let mut h = self.cfg.effective_hemisphere();
        if segmented(ui, "hemi", &mut h, &[(Hemisphere::Southern, u.southern), (Hemisphere::Northern, u.northern)]) {
            self.cfg.hemisphere = Some(h);
        }
        section_label(ui, u.cross_quarters);
        segmented(
            ui,
            "cq",
            &mut self.cfg.cross_quarters,
            &[(CrossQuarterRule::Traditional, u.traditional), (CrossQuarterRule::Astronomical, u.astronomical)],
        );
        section_label(ui, u.language);
        segmented(
            ui,
            "lang",
            &mut self.cfg.language,
            &[(LanguagePref::Auto, u.automatic), (LanguagePref::En, "English"), (LanguagePref::PtBr, "Português")],
        );
        ui.add_space(2.0);
        switch(ui, "anim", &mut self.cfg.animations, u.animations);
        switch(ui, "top", &mut self.cfg.always_on_top, u.keep_on_top);
        if !self.cfg.always_on_top {
            note(ui, u.on_desktop_note);
        }
        switch(ui, "auto", &mut self.cfg.autostart, u.autostart);

        let tz = if self.cfg.timezone.is_empty() { u.system_zone } else { self.cfg.timezone.as_str() };
        note(ui, &format!("{}: {tz} ({})", u.time_zone, self.text.utc_offset));
        if let Some(p) = config::config_path() {
            let shown = match std::env::var_os("HOME").map(std::path::PathBuf::from) {
                Some(home) if p.starts_with(&home) => format!("~/{}", p.strip_prefix(&home).unwrap_or(&p).display()),
                _ => p.display().to_string(),
            };
            note(ui, &format!("{}: {shown}", u.config_file));
        }
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter().rect_filled(r, 0.0, theme::RULE);
        if link_colored(ui, u.quit_app, theme::POPPY).clicked() {
            *action = Some(Action::Quit);
        }
        self.settings_changed(ui.ctx(), &before, t);
    }

    // ── window placement ──────────────────────────────────────────────────

    fn foot_for(&self, win_pos: Pos2) -> Pos2 {
        pos2(win_pos.x + MARGIN, win_pos.y + self.applied_size.y - BELOW_FEET)
    }

    fn save_foot(&mut self) {
        let Some(f) = self.foot else { return };
        if self.saved_foot == Some(f) {
            return;
        }
        if config::save_window_state(&WindowState { foot_x: Some(f.x), foot_y: Some(f.y) }).is_ok() {
            self.saved_foot = Some(f);
        }
    }

    /// Dragging follows the pointer's real position on screen (read from
    /// X11), so the window never lags behind or overshoots it. Without X11
    /// the compositor moves the window.
    fn drag(&mut self, ctx: &egui::Context, resp: &Response) {
        let ppp = ctx.pixels_per_point();
        if resp.drag_started() {
            match &self.desktop {
                Some(d) => {
                    self.win_pos = d.window_position(ppp).or(self.win_pos);
                    // Where the button went down, relative to the window's
                    // corner (the drag only starts after a few pixels of
                    // movement, so the current position would lag behind).
                    self.drag_offset = ctx.input(|i| i.pointer.press_origin()).map(|p| p - win_origin(ctx));
                }
                None => ctx.send_viewport_cmd(ViewportCommand::StartDrag),
            }
        }
        if resp.dragged()
            && let (Some(d), Some(offset)) = (&self.desktop, self.drag_offset)
            && let Some(p) = d.pointer(ppp)
        {
            let pos = p - offset;
            if self.win_pos.is_none_or(|w| (w - pos).length() >= 0.5) {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
                self.win_pos = Some(pos);
            }
            // Keep following even if a quick move gets ahead of the window
            // and no pointer event reaches it for a moment.
            ctx.request_repaint();
        }
        if resp.drag_stopped() {
            // Settle exactly where the button was released.
            if let (Some(d), Some(offset)) = (&self.desktop, self.drag_offset)
                && let Some(p) = d.pointer(ppp)
            {
                let pos = p - offset;
                if self.win_pos.is_none_or(|w| (w - pos).length() >= 0.5) {
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
                    self.win_pos = Some(pos);
                }
            }
            self.drag_offset = None;
            if self.desktop.is_none() {
                self.win_pos = ctx.input(|i| i.viewport().outer_rect).map(|r| r.min);
            }
            if let Some(w) = self.win_pos {
                self.foot = Some(self.foot_for(w));
            }
            self.save_foot();
        }
    }
}

impl eframe::App for SabbatiaApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let now = Utc::now();
        let t = ctx.input(|i| i.time);
        if now >= self.next_refresh {
            self.recompute(now, t);
        }
        let mut wait = (self.next_refresh - now).to_std().unwrap_or(Duration::ZERO);
        let mut action = None;

        if !self.desktop_tried {
            self.desktop = Desktop::connect(frame);
            self.desktop_tried = true;
        }
        // Apply the desktop hints at start, again at 0.5 s and 2 s (requests
        // sent before the window manager has mapped the window are ignored),
        // and after changes.
        const PIN_AT: [Duration; 3] = [Duration::ZERO, Duration::from_millis(500), Duration::from_secs(2)];
        let above = self.cfg.always_on_top;
        let done = match self.pinned {
            Some((a, n)) if a == above => n as usize,
            _ => 0,
        };
        if let (Some(d), Some(&due)) = (&self.desktop, PIN_AT.get(done)) {
            let since = if done == 0 { Duration::ZERO } else { self.started.elapsed() };
            if since >= due {
                if let Err(e) = d.pin(above) {
                    eprintln!("sabbatia: desktop hints: {e}");
                }
                self.pinned = Some((above, done as u8 + 1));
            } else {
                wait = wait.min(due - since);
            }
        }

        // The window has a fixed size; it changes only when the figure or
        // bubble size settings do, keeping the feet where they were.
        let fig = figure_size(&self.cfg);
        let size = window_size(&self.cfg, fig);
        if self.win_pos.is_none() {
            self.win_pos = ctx.input(|i| i.viewport().outer_rect).map(|r| r.min);
        }
        if self.foot.is_none() {
            self.foot = self.win_pos.map(|w| self.foot_for(w));
        }
        if (size - self.applied_size).length() > 0.5 {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
            self.applied_size = size;
            if let Some(f) = self.foot {
                let pos = pos2(f.x - MARGIN, f.y - (size.y - BELOW_FEET));
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
                self.win_pos = Some(pos);
            }
        }

        // Lay out from the window's real edges: the figure stands at the
        // bottom-left corner, the bubble opens above him.
        let win = ctx.content_rect();
        let fig_rect = Rect::from_min_size(pos2(win.left() + MARGIN, win.bottom() - BELOW_FEET - fig.y), fig);
        let bw = self.bubble_width();
        let open = self.panel != Panel::None;
        let bubble_bottom = fig_rect.top() - TAIL;
        // Never let the bubble run above the top of the screen.
        let screen_top = win.top() + self.win_pos.map_or(0.0, |w| (-w.y).max(0.0));
        let room = bubble_bottom - (win.top() + MARGIN).max(screen_top + MARGIN);
        let detail_max = (room - 300.0).clamp(100.0, DETAIL_MAX_HEIGHT);
        let mut input = vec![fig_rect.expand(2.0)];

        // ── the bubble ──
        if open {
            let bubble = Rect::from_min_max(pos2(win.left() + MARGIN, bubble_bottom - self.bubble_h), pos2(win.left() + MARGIN + bw, bubble_bottom));
            let painter = ui.painter().clone();
            let shadow_slot = painter.add(Shape::Noop);
            let bubble_slot = painter.add(Shape::Noop);
            let bg = ui.interact(bubble, ui.scope_id().with("bubble"), Sense::click_and_drag());
            self.drag(&ctx, &bg);
            menu_on(&bg, self.lang, &mut action);

            let inner = Rect::from_min_size(bubble.min + vec2(PAD, PAD - 2.0), vec2(bw - 2.0 * PAD, 4000.0));
            let content = ui
                .scope_builder(UiBuilder::new().max_rect(inner), |ui| {
                    ui.spacing_mut().item_spacing = vec2(GAP, GAP);
                    match self.panel {
                        Panel::Talk => self.talk(ui, &mut action),
                        Panel::Almanac => self.almanac_view(ui, &mut action, detail_max),
                        Panel::Settings => self.settings(ui, &mut action, t),
                        Panel::None => {}
                    }
                    if let Some(n) = &self.notice {
                        ui.label(RichText::new(n).font(theme::semibold(11.0)).color(theme::POPPY));
                    }
                })
                .response
                .rect;
            let measured = (content.bottom() - bubble.top() + PAD - 2.0).ceil();
            if (measured - self.bubble_h).abs() > 0.5 {
                // Lay the frame out again at the right height before it is
                // shown, instead of showing one frame at the old size.
                self.bubble_h = measured;
                ctx.request_discard("bubble height changed");
            }
            let fill = if self.panel == Panel::Talk { theme::WHITE } else { theme::CREAM };
            let radius = if self.panel == Panel::Talk { 16.0 } else { f32::from(theme::CARD_RADIUS) };
            let shadow = egui::Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha(45) };
            painter.set(shadow_slot, shadow.as_shape(bubble, radius));
            painter.set(bubble_slot, Shape::rect_filled(bubble, radius, fill));
            painter.rect_stroke(bubble, radius, Stroke::new(1.5, theme::INK), StrokeKind::Inside);
            // Tail down to the figure's head.
            let head_x = (fig_rect.left() + fig.x * 0.5).clamp(bubble.left() + 24.0, bubble.right() - 24.0);
            let (b, tip) = (bubble.bottom() - 0.75, pos2(head_x - 4.0, bubble.bottom() + TAIL - 1.0));
            painter.add(Shape::convex_polygon(
                vec![pos2(head_x - 9.0, b - 1.5), pos2(head_x + 7.0, b - 1.5), tip],
                fill,
                Stroke::NONE,
            ));
            painter.line_segment([pos2(head_x - 9.0, b), tip], Stroke::new(1.5, theme::INK));
            painter.line_segment([pos2(head_x + 7.0, b), tip], Stroke::new(1.5, theme::INK));
            input.push(Rect::from_min_max(bubble.min, pos2(bubble.max.x, bubble.max.y + TAIL)).expand(2.0));

            if self.panel == Panel::Talk {
                let hovered = ctx.input(|i| i.pointer.hover_pos()).is_some();
                if hovered {
                    self.talk_since = Instant::now();
                }
                let left = TALK_TIMEOUT.saturating_sub(self.talk_since.elapsed());
                if left.is_zero() {
                    action = Some(Action::Close);
                } else {
                    wait = wait.min(left);
                }
            }
        }

        // ── the figure ──
        let fresp = ui
            .interact(fig_rect, ui.scope_id().with("figure"), Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::PointingHand);
        self.drag(&ctx, &fresp);
        menu_on(&fresp, self.lang, &mut action);
        let animate = self.cfg.animations;
        if let Some(p) = &mut self.puppet {
            if fresp.clicked() {
                p.poke(t);
            }
            let (pose, w) = p.update(t, animate);
            p.paint(ui.painter(), fig_rect, &pose);
            if let Some(w) = w {
                wait = wait.min(w);
            }
        }
        if fresp.clicked() {
            action = Some(Action::Talk);
        }

        // Only the figure and the bubble take the pointer; the rest of the
        // window lets clicks through to the desktop. Menus and drags need
        // the whole window.
        if ctx.any_popup_open() || self.drag_offset.is_some() {
            input = vec![win];
        }
        if let Some(d) = &mut self.desktop
            && let Err(e) = d.set_input_region(&input, ppp)
        {
            eprintln!("sabbatia: input region: {e}");
        }

        if let Some(a) = action {
            self.apply(&ctx, a, t);
            ctx.request_repaint();
        }
        ctx.request_repaint_after(wait);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_foot();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

/// The window's top-left corner in egui coordinates.
fn win_origin(ctx: &egui::Context) -> Pos2 {
    ctx.content_rect().min
}

/// Height of the figure from the settings; width from the character's art.
pub fn figure_size(cfg: &Config) -> Vec2 {
    let h = cfg.figure_size;
    vec2((h * puppet::aspect(cfg.character)).round(), h)
}

/// The window's fixed size: the figure plus room above for the largest
/// bubble. The empty part is transparent and lets clicks through.
pub fn window_size(cfg: &Config, fig: Vec2) -> Vec2 {
    let w = fig.x.max(TALK_W).max(cfg.width) + 2.0 * MARGIN;
    vec2(w.ceil(), (MARGIN + BUBBLE_MAX + TAIL + fig.y + BELOW_FEET).ceil())
}

fn load_puppet(ctx: &egui::Context, who: Character, a: &Almanac) -> Option<Puppet> {
    match Puppet::load(ctx, who, companion::mood(a)) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("sabbatia: companion: {e}");
            None
        }
    }
}

/// Developer hook for screenshots: `SABBATIA_PREVIEW=talk|almanac|settings`,
/// or `season|moon|wheel|sabbat` for the almanac with that tile open.
fn preview() -> (Panel, Option<Section>) {
    match std::env::var("SABBATIA_PREVIEW").ok().as_deref() {
        Some("talk") => (Panel::Talk, None),
        Some("almanac") => (Panel::Almanac, None),
        Some("settings") => (Panel::Settings, None),
        Some("season") => (Panel::Almanac, Some(Section::Season)),
        Some("moon") => (Panel::Almanac, Some(Section::Moon)),
        Some("wheel") => (Panel::Almanac, Some(Section::Wheel)),
        Some("sabbat") => (Panel::Almanac, Some(Section::Sabbat)),
        _ => (Panel::None, None),
    }
}

pub fn window_level(cfg: &Config) -> WindowLevel {
    if cfg.always_on_top { WindowLevel::AlwaysOnTop } else { WindowLevel::AlwaysOnBottom }
}

/// Next local midnight (day counts change) or the periodic refresh.
fn next_refresh_after(now: DateTime<Utc>, a: &Almanac) -> DateTime<Utc> {
    let periodic = now + MAX_REFRESH_INTERVAL;
    let midnight = a
        .today
        .succ_opt()
        .and_then(|d| Local.from_local_datetime(&d.and_time(NaiveTime::MIN)).earliest())
        .map(|t| t.with_timezone(&Utc) + chrono::TimeDelta::seconds(1));
    match midnight {
        Some(m) if m < periodic => m,
        _ => periodic,
    }
}

// ── menu ──────────────────────────────────────────────────────────────────

fn menu_on(resp: &Response, lang: Lang, action: &mut Option<Action>) {
    resp.context_menu(|ui| menu_items(ui, lang, action));
}

fn menu_items(ui: &mut Ui, lang: Lang, action: &mut Option<Action>) {
    let u = lang.ui();
    ui.set_min_width(180.0);
    let mut item = |ui: &mut Ui, text: RichText, a: Action| {
        if ui.add(egui::Button::new(text).min_size(vec2(170.0, 26.0))).clicked() {
            *action = Some(a);
            ui.close();
        }
    };
    item(ui, RichText::new(u.talk), Action::Talk);
    item(ui, RichText::new(u.almanac), Action::Almanac);
    item(ui, RichText::new(u.settings_menu), Action::Settings);
    ui.separator();
    item(ui, RichText::new(u.switch_character), Action::SwitchCharacter);
    item(ui, RichText::new(u.switch_hemisphere), Action::SwitchHemisphere);
    ui.separator();
    item(ui, RichText::new(u.quit).font(theme::bold(12.5)).color(theme::POPPY), Action::Quit);
}

// ── tiles ─────────────────────────────────────────────────────────────────

struct TileGalleys {
    label: Arc<Galley>,
    value: Arc<Galley>,
    sub: Arc<Galley>,
}

const TILE_PAD: f32 = 10.0;
const TILE_ICON: f32 = 26.0;

fn tile_height(g: &TileGalleys, progress: bool) -> f32 {
    TILE_PAD + TILE_ICON + 6.0 + g.value.size().y + 1.0 + g.sub.size().y + if progress { 10.0 } else { 0.0 } + TILE_PAD
}

#[allow(clippy::too_many_arguments)]
fn tile(
    ui: &mut Ui,
    rect: Rect,
    sec: Section,
    tint: Tint,
    icon: &TextureHandle,
    g: &TileGalleys,
    progress: Option<f32>,
    selected: bool,
) -> Response {
    let resp = ui
        .interact(rect, ui.scope_id().with(("tile", sec as u8)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let r = if resp.is_pointer_button_down_on() { rect.shrink(1.0) } else { rect };
    let p = ui.painter();
    p.rect_filled(r, theme::TILE_RADIUS, if resp.hovered() { tint.hover } else { tint.fill });
    if selected {
        p.rect_stroke(r, theme::TILE_RADIUS, Stroke::new(1.5, theme::INK), StrokeKind::Inside);
    }
    let ic = Rect::from_min_size(r.min + vec2(TILE_PAD, TILE_PAD), vec2(TILE_ICON, TILE_ICON));
    p.image(icon.id(), ic, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    p.circle_stroke(ic.center(), TILE_ICON / 2.0 - 0.4, Stroke::new(1.3, theme::INK));
    p.galley(pos2(ic.right() + 6.0, ic.center().y - g.label.size().y / 2.0), g.label.clone(), theme::INK);
    let mut y = ic.bottom() + 6.0;
    p.galley(pos2(r.left() + TILE_PAD, y), g.value.clone(), theme::INK);
    y += g.value.size().y + 1.0;
    p.galley(pos2(r.left() + TILE_PAD, y), g.sub.clone(), theme::DIM);
    y += g.sub.size().y + 5.0;
    if let Some(f) = progress {
        let bar = Rect::from_min_size(pos2(r.left() + TILE_PAD, y), vec2(r.width() - 2.0 * TILE_PAD, 5.0));
        glyphs::progress(p, bar, f, theme::LEAF, Color32::from_rgb(0xD3, 0xE3, 0xCA));
    }
    resp
}

// ── text helpers ──────────────────────────────────────────────────────────

fn title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(theme::bold(15.0)));
}

fn heading(ui: &mut Ui, text: &str, color: Color32) {
    ui.add_space(3.0);
    ui.label(RichText::new(text).font(theme::bold(12.0)).color(color));
}

fn para(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).font(theme::semibold(12.0))).wrap());
}

fn small(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(theme::semibold(11.5)).color(theme::INK_SOFT));
}

fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).font(theme::semibold(10.5)).color(theme::DIM)).wrap());
}

fn source(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).font(theme::semibold(10.5)).italics().color(theme::DIM)).wrap());
}

fn kv(ui: &mut Ui, key: &str, value: &str) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(92.0, 15.0), Sense::hover());
        ui.painter().text(r.left_top(), Align2::LEFT_TOP, key, theme::semibold(11.0), theme::DIM);
        ui.add(egui::Label::new(RichText::new(value).font(theme::semibold(11.5))).wrap());
    });
}

fn phase_cards(ui: &mut Ui, phases: &[(&'static str, String)]) {
    let w = ui.available_width();
    let cw = ((w - 6.0) / 2.0).floor();
    for pair in phases.chunks(2) {
        let (row, _) = ui.allocate_exact_size(vec2(w, 36.0), Sense::hover());
        for (i, (name, when)) in pair.iter().enumerate() {
            let r = Rect::from_min_size(row.min + vec2(i as f32 * (cw + 6.0), 0.0), vec2(cw, 36.0));
            let p = ui.painter();
            p.rect_filled(r, 10.0, theme::CREAM);
            p.text(r.min + vec2(8.0, 4.0), Align2::LEFT_TOP, name, theme::semibold(10.5), theme::DIM);
            p.text(r.min + vec2(8.0, 18.0), Align2::LEFT_TOP, when, theme::bold(11.5), theme::INK);
        }
    }
}

// ── small controls ────────────────────────────────────────────────────────

fn link(ui: &mut Ui, text: &str) -> Response {
    link_colored(ui, &format!("{text} ›"), theme::NIGHT)
}

fn link_colored(ui: &mut Ui, text: &str, color: Color32) -> Response {
    ui.add(egui::Label::new(RichText::new(text).font(theme::bold(12.5)).color(color)).sense(Sense::click()))
        .on_hover_cursor(CursorIcon::PointingHand)
}

fn close_button(ui: &mut Ui, hint: &str) -> Response {
    let (r, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(hint);
    let p = ui.painter();
    if resp.hovered() {
        p.circle_filled(r.center(), 11.0, theme::TRACK);
    }
    let c = r.center();
    let s = Stroke::new(1.6, theme::INK_SOFT);
    p.line_segment([c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], s);
    p.line_segment([c + vec2(4.5, -4.5), c + vec2(-4.5, 4.5)], s);
    resp
}

fn section_label(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(glyphs::label_job(text, 9.5, theme::DIM));
}

fn pill(ui: &mut Ui, text: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(text.to_owned(), theme::bold(12.0), theme::INK);
    let size = galley.size() + vec2(22.0, 10.0);
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    let p = ui.painter();
    p.rect_filled(r, r.height() / 2.0, if resp.hovered() { theme::TRACK } else { theme::WHITE });
    p.rect_stroke(r, r.height() / 2.0, Stroke::new(1.5, theme::INK), StrokeKind::Inside);
    p.galley(r.center() - galley.size() / 2.0, galley, theme::INK);
    resp
}

/// Pill-shaped segmented control. Returns true when the value changed.
fn segmented<T: Copy + PartialEq>(ui: &mut Ui, salt: &str, value: &mut T, options: &[(T, &str)]) -> bool {
    let w = ui.available_width();
    let h = 30.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    ui.painter().rect_filled(rect, h / 2.0, theme::TRACK);
    let seg_w = (w - 6.0) / options.len() as f32;
    let mut changed = false;
    for (i, (v, label)) in options.iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.left() + 3.0 + i as f32 * seg_w, rect.top() + 3.0), vec2(seg_w, h - 6.0));
        let resp = ui.interact(r, ui.scope_id().with((salt, i)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let selected = *value == *v;
        let p = ui.painter();
        if selected {
            p.rect_filled(r, r.height() / 2.0, theme::WHITE);
            p.rect_stroke(r, r.height() / 2.0, Stroke::new(1.5, theme::INK), StrokeKind::Inside);
        } else if resp.hovered() {
            p.rect_filled(r, r.height() / 2.0, theme::RULE);
        }
        let (font, color) = if selected { (theme::bold(12.0), theme::INK) } else { (theme::semibold(12.0), theme::INK_SOFT) };
        p.text(r.center(), Align2::CENTER_CENTER, label, font, color);
        if resp.clicked() && !selected {
            *value = *v;
            changed = true;
        }
    }
    changed
}

/// Label on the left, switch on the right.
fn switch(ui: &mut Ui, salt: &str, on: &mut bool, label: &str) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 24.0), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);
    if resp.clicked() {
        *on = !*on;
    }
    let k = ui.ctx().animate_bool_with_time(ui.scope_id().with(("switch", salt)), *on, 0.12);
    let p = ui.painter();
    p.text(rect.left_center(), Align2::LEFT_CENTER, label, theme::semibold(12.5), theme::INK);
    let track = Rect::from_min_size(pos2(rect.right() - 36.0, rect.center().y - 10.0), vec2(36.0, 20.0));
    let fill = Color32::from_rgb(
        lerp_u8(theme::TRACK.r(), theme::LEAF.r(), k),
        lerp_u8(theme::TRACK.g(), theme::LEAF.g(), k),
        lerp_u8(theme::TRACK.b(), theme::LEAF.b(), k),
    );
    p.rect_filled(track, 10.0, fill);
    p.rect_stroke(track, 10.0, Stroke::new(1.5, theme::INK), StrokeKind::Inside);
    let x = track.left() + 10.0 + k * 16.0;
    p.circle_filled(pos2(x, track.center().y), 6.5, theme::WHITE);
    p.circle_stroke(pos2(x, track.center().y), 6.5, Stroke::new(1.0, theme::INK));
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8
}
