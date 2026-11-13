//! The companions' dialogue: what each one says, when, and how often.
//!
//! * Content lives in [`lines`]: entries with a condition and one to three
//!   steps, in English and Brazilian Portuguese, each step carrying a
//!   gesture cue for the puppet.
//! * A [`Director`] per companion picks entries from the almanac context,
//!   remembers what was said this session, gives fresh astronomical events
//!   priority (once per few hours), runs short conversations across clicks,
//!   and decides the rare moments to speak unprompted.
//!
//! Everything is pure and driven by a session clock in seconds, so it is
//! deterministic under test. No text is generated at run time.

pub mod lines;

use std::collections::VecDeque;

use chrono::Timelike;

use crate::calendar::almanac::Almanac;
use crate::calendar::{MoonPhase, Sabbat, Season};
use crate::companion::Character;
use crate::i18n::Lang;

/// A gesture hint for the puppet. Each companion maps it onto its own moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    /// No movement: a contemplative pause.
    Still,
    /// Lantern / rose raised.
    Raise,
    /// Head bowed.
    Bow,
    /// Lantern / bag swung.
    Swing,
    /// A small step.
    Step,
    /// The dog hops (the Hermit lifts his lantern instead).
    Hop,
    /// Looking up at the sky.
    LookUp,
    /// Head down, thinking; the dog lowers its head.
    Ponder,
}

pub struct Step {
    /// Spoken by the user (a question the companion anticipates) rather
    /// than by the companion.
    pub you: bool,
    pub cue: Cue,
    pub en: &'static str,
    pub pt: &'static str,
}

pub struct Entry {
    pub id: &'static str,
    pub when: When,
    pub steps: &'static [Step],
}

/// When an entry may be said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Always,
    /// The session has lasted a couple of hours.
    LongSession,
    /// No interaction with the companion for a long while (unprompted only).
    Inactive,
    Morning,
    Evening,
    Night,
    InSeason(Season),
    /// Winter or spring: days getting longer.
    Lengthening,
    /// Summer or autumn: days getting shorter.
    Shortening,
    /// A season has just begun or is about to end.
    SeasonTurning,
    FullMoon,
    NewMoon,
    Waxing,
    Waning,
    /// A festival is today.
    SabbatToday,
    /// The next festival is within a week.
    SabbatSoon,
    /// This festival is next and within two weeks.
    Approaching(Sabbat),
}

/// How an entry competes for attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Something meaningful is happening: priority when fresh.
    Event,
    /// True of the present moment: favoured.
    Context,
    General,
    /// Calling out after a long quiet: unprompted only.
    Call,
}

const SESSION_LONG: f64 = 2.0 * 3600.0;
const INACTIVE_AFTER: f64 = 45.0 * 60.0;
/// A conversation continues if the next click comes within this time.
const CONVERSATION_PATIENCE: f64 = 90.0;
/// An event is given priority again only after this long.
const EVENT_COOLDOWN: f64 = 3.0 * 3600.0;
/// Entries said within this many turns are not repeated.
const RECENT: usize = 16;

impl When {
    fn class(self) -> Class {
        match self {
            When::SabbatToday
            | When::SabbatSoon
            | When::Approaching(_)
            | When::FullMoon
            | When::NewMoon
            | When::SeasonTurning => Class::Event,
            When::Always | When::LongSession => Class::General,
            When::Inactive => Class::Call,
            _ => Class::Context,
        }
    }

    /// Among fresh events, the most important is said first.
    fn priority(self) -> u8 {
        match self {
            When::SabbatToday => 5,
            When::FullMoon | When::NewMoon => 4,
            When::Approaching(_) => 3,
            When::SabbatSoon => 2,
            When::SeasonTurning => 1,
            _ => 0,
        }
    }

    fn holds(self, s: &Situation<'_>, idle: f64) -> bool {
        let a = s.almanac;
        let hour = a.computed_at.hour();
        let days = a.next_sabbat.days_until;
        let season = a.season.season;
        match self {
            When::Always => true,
            When::LongSession => s.now >= SESSION_LONG,
            When::Inactive => idle >= INACTIVE_AFTER,
            When::Morning => (5..10).contains(&hour),
            When::Evening => (18..22).contains(&hour),
            When::Night => !(5..22).contains(&hour),
            When::InSeason(x) => season == x,
            When::Lengthening => matches!(season, Season::Winter | Season::Spring),
            When::Shortening => matches!(season, Season::Summer | Season::Autumn),
            When::SeasonTurning => {
                a.season.days_remaining <= 3 || (a.today - a.season.start.date_naive()).num_days() <= 2
            }
            When::FullMoon => a.moon.phase == MoonPhase::Full,
            When::NewMoon => a.moon.phase == MoonPhase::New,
            When::Waxing => a.moon.waxing && !matches!(a.moon.phase, MoonPhase::New | MoonPhase::Full),
            When::Waning => !a.moon.waxing && !matches!(a.moon.phase, MoonPhase::New | MoonPhase::Full),
            When::SabbatToday => days == 0,
            When::SabbatSoon => (1..=7).contains(&days),
            When::Approaching(x) => a.next_sabbat.occurrence.sabbat == x && (1..=14).contains(&days),
        }
    }
}

/// What the director needs to know about the present.
pub struct Situation<'a> {
    pub almanac: &'a Almanac,
    pub lang: Lang,
    /// Seconds since the companion appeared this session.
    pub now: f64,
}

/// One step of dialogue, ready to show.
#[derive(Debug, Clone, PartialEq)]
pub struct Utterance {
    pub id: &'static str,
    pub text: String,
    /// The user's own (anticipated) words rather than the companion's.
    pub you: bool,
    pub cue: Cue,
    /// Another step follows if the user clicks again.
    pub more: bool,
}

pub fn entries(who: Character) -> &'static [Entry] {
    match who {
        Character::Hermit => lines::HERMIT,
        Character::Fool => lines::FOOL,
    }
}

fn fill(text: &str, s: &Situation<'_>) -> String {
    let a = s.almanac;
    if !text.contains('{') {
        return text.to_string();
    }
    text.replace("{sabbat}", a.next_sabbat.occurrence.sabbat.name())
        .replace("{days}", &s.lang.days(a.next_sabbat.days_until))
        .replace("{season}", s.lang.season(a.season.season))
}

/// How talkative a companion is when nobody asks.
struct Temperament {
    /// Minimum time between two things said unprompted.
    min_gap: f64,
    /// Chance, per check, of mentioning a fresh event.
    event_chance: f64,
    /// Chance, per check, of calling out after a long quiet.
    call_chance: f64,
    /// Calling out happens at most this often.
    call_gap: f64,
    /// Chance, per check, of a passing remark.
    remark_chance: f64,
}

fn temperament(who: Character) -> Temperament {
    match who {
        // Long silences are intentional.
        Character::Hermit => Temperament {
            min_gap: 3.0 * 3600.0,
            event_chance: 0.25,
            call_chance: 0.03,
            call_gap: 8.0 * 3600.0,
            remark_chance: 0.0,
        },
        Character::Fool => Temperament {
            min_gap: 40.0 * 60.0,
            event_chance: 0.5,
            call_chance: 0.25,
            call_gap: 2.0 * 3600.0,
            remark_chance: 0.06,
        },
    }
}

pub struct Director {
    who: Character,
    recent: VecDeque<&'static str>,
    conversation: Option<(&'static Entry, usize)>,
    last_spoke: Option<f64>,
    last_touch: f64,
    last_call: Option<f64>,
    /// When each kind of event was last given priority.
    events: Vec<(When, f64)>,
    rng: u64,
}

impl Director {
    pub fn new(who: Character, seed: u64) -> Director {
        Director {
            who,
            recent: VecDeque::with_capacity(RECENT),
            conversation: None,
            last_spoke: None,
            last_touch: 0.0,
            last_call: None,
            events: Vec::new(),
            rng: seed | 1,
        }
    }

    pub fn character(&self) -> Character {
        self.who
    }

    fn random(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 11) as f64 / (1u64 << 53) as f64
    }

    /// The user interacted with the companion (click, drag, almanac…).
    pub fn touch(&mut self, now: f64) {
        self.last_touch = now;
    }

    fn fresh(&self, when: When, now: f64) -> bool {
        self.events.iter().all(|&(w, t)| w != when || now - t >= EVENT_COOLDOWN)
    }

    fn holding(&self, s: &Situation<'_>) -> Vec<&'static Entry> {
        let idle = s.now - self.last_touch;
        entries(self.who).iter().filter(|e| e.when.holds(s, idle)).collect()
    }

    fn utter(&self, e: &'static Entry, i: usize, s: &Situation<'_>) -> Utterance {
        let step = &e.steps[i];
        let text = match s.lang {
            Lang::En => step.en,
            Lang::PtBr => step.pt,
        };
        Utterance { id: e.id, text: fill(text, s), you: step.you, cue: step.cue, more: i + 1 < e.steps.len() }
    }

    fn begin(&mut self, e: &'static Entry, s: &Situation<'_>) -> Utterance {
        if self.recent.len() == RECENT {
            self.recent.pop_front();
        }
        self.recent.push_back(e.id);
        if e.when.class() == Class::Event {
            self.events.retain(|&(w, _)| w != e.when);
            self.events.push((e.when, s.now));
        }
        if e.when.class() == Class::Call {
            self.last_call = Some(s.now);
        }
        self.conversation = Some((e, 0));
        self.last_spoke = Some(s.now);
        self.utter(e, 0, s)
    }

    /// The most important fresh event entry not said recently, if any.
    fn fresh_event(&mut self, pool: &[&'static Entry], now: f64) -> Option<&'static Entry> {
        let fresh: Vec<_> = pool
            .iter()
            .copied()
            .filter(|e| e.when.class() == Class::Event && self.fresh(e.when, now) && !self.recent.contains(&e.id))
            .collect();
        let top = fresh.iter().map(|e| e.when.priority()).max()?;
        let best: Vec<_> = fresh.into_iter().filter(|e| e.when.priority() == top).collect();
        let i = (self.random() * best.len() as f64) as usize;
        best.get(i.min(best.len() - 1)).copied()
    }

    fn weighted(&mut self, pool: &[&'static Entry]) -> Option<&'static Entry> {
        let weight = |e: &Entry| match e.when {
            When::Night => 5.0,
            When::LongSession => 2.5,
            w => match w.class() {
                Class::Context => 3.0,
                Class::Event => 2.0,
                Class::General => 1.5,
                Class::Call => 0.0,
            },
        };
        let total: f64 = pool.iter().map(|e| weight(e)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut r = self.random() * total;
        for e in pool {
            r -= weight(e);
            if r <= 0.0 {
                return Some(e);
            }
        }
        pool.last().copied()
    }

    /// Say a particular entry from a particular step (previews and tests).
    pub fn say(&mut self, id: &str, step: usize, s: &Situation<'_>) -> Option<Utterance> {
        let e = entries(self.who).iter().find(|e| e.id == id)?;
        let i = step.min(e.steps.len() - 1);
        self.conversation = Some((e, i));
        self.last_spoke = Some(s.now);
        Some(self.utter(e, i, s))
    }

    /// The user clicked the companion (or the bubble): continue the
    /// conversation, or start something new.
    pub fn on_click(&mut self, s: &Situation<'_>) -> Utterance {
        self.touch(s.now);
        if let (Some((e, i)), Some(t)) = (self.conversation, self.last_spoke)
            && i + 1 < e.steps.len()
            && s.now - t <= CONVERSATION_PATIENCE
        {
            self.conversation = Some((e, i + 1));
            self.last_spoke = Some(s.now);
            return self.utter(e, i + 1, s);
        }

        let pool: Vec<_> = self.holding(s).into_iter().filter(|e| e.when.class() != Class::Call).collect();
        if let Some(e) = self.fresh_event(&pool, s.now) {
            return self.begin(e, s);
        }
        let unheard: Vec<_> = pool.iter().copied().filter(|e| !self.recent.contains(&e.id)).collect();
        let pick = self.weighted(&unheard).or_else(|| {
            // Everything was said recently: the oldest, but never the last.
            let last = self.recent.back().copied();
            self.recent.iter().find_map(|id| pool.iter().copied().find(|e| e.id == *id && Some(e.id) != last))
        });
        let e = pick.or_else(|| pool.first().copied()).unwrap_or(&entries(self.who)[0]);
        self.begin(e, s)
    }

    /// Called now and then (every few minutes). Returns something to say
    /// unprompted, rarely, or nothing. Never at night.
    pub fn spontaneous(&mut self, s: &Situation<'_>) -> Option<Utterance> {
        let t = temperament(self.who);
        if When::Night.holds(s, 0.0) {
            return None;
        }
        if self.last_spoke.is_some_and(|last| s.now - last < t.min_gap) || s.now < 60.0 {
            return None;
        }
        let pool = self.holding(s);
        if let Some(e) = self.fresh_event(&pool, s.now) {
            return (self.random() < t.event_chance).then(|| self.begin(e, s));
        }
        let idle = s.now - self.last_touch;
        if idle >= INACTIVE_AFTER && self.last_call.is_none_or(|c| s.now - c >= t.call_gap) {
            let calls: Vec<_> =
                pool.iter().copied().filter(|e| e.when.class() == Class::Call && !self.recent.contains(&e.id)).collect();
            if !calls.is_empty() && self.random() < t.call_chance {
                let i = ((self.random() * calls.len() as f64) as usize).min(calls.len() - 1);
                return Some(self.begin(calls[i], s));
            }
            return None;
        }
        if t.remark_chance > 0.0 && self.random() < t.remark_chance {
            let remarks: Vec<_> = pool
                .iter()
                .copied()
                .filter(|e| matches!(e.when.class(), Class::Context | Class::General) && !self.recent.contains(&e.id))
                .collect();
            if let Some(e) = self.weighted(&remarks) {
                return Some(self.begin(e, s));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::almanac::CalendarSettings;
    use crate::calendar::{CrossQuarterRule, Hemisphere};
    use chrono::{FixedOffset, TimeZone, Utc};
    use std::collections::HashSet;

    fn at(y: i32, mo: u32, d: u32, h: u32) -> Almanac {
        let tz = FixedOffset::west_opt(3 * 3600).unwrap();
        let settings = CalendarSettings { hemisphere: Hemisphere::Southern, cross_quarters: CrossQuarterRule::Traditional };
        let utc = Utc.with_ymd_and_hms(y, mo, d, 0, 0, 0).unwrap() + chrono::TimeDelta::hours(i64::from(h) + 3);
        Almanac::compute(utc, &tz, settings)
    }

    fn sit(a: &Almanac, now: f64) -> Situation<'_> {
        Situation { almanac: a, lang: Lang::En, now }
    }

    #[test]
    fn content_is_complete_and_well_formed() {
        for who in Character::ALL {
            let all = entries(who);
            assert!(all.len() >= 40, "{who:?} has only {} entries", all.len());
            let ids: HashSet<_> = all.iter().map(|e| e.id).collect();
            assert_eq!(ids.len(), all.len(), "{who:?} has duplicate ids");
            for e in all {
                assert!(!e.steps.is_empty() && e.steps.len() <= 3, "{}", e.id);
                assert!(!e.steps[0].you, "{} must open with the companion", e.id);
                for st in e.steps {
                    assert!(!st.en.trim().is_empty() && !st.pt.trim().is_empty(), "{}", e.id);
                    assert_ne!(st.en, st.pt, "{} is untranslated", e.id);
                    for text in [st.en, st.pt] {
                        let rest = text.replace("{sabbat}", "").replace("{days}", "").replace("{season}", "");
                        assert!(!rest.contains('{') && !rest.contains('}'), "{}: unknown placeholder", e.id);
                    }
                }
            }
            // Multi-step conversations exist for both.
            assert!(all.iter().filter(|e| e.steps.len() > 1).count() >= 8, "{who:?}");
        }
    }

    #[test]
    fn every_context_has_lines_for_both() {
        for who in Character::ALL {
            let has = |w: When| entries(who).iter().any(|e| e.when == w);
            for w in [When::FullMoon, When::NewMoon, When::Waxing, When::Waning, When::SabbatToday, When::SabbatSoon,
                      When::SeasonTurning, When::Night, When::Lengthening, When::Shortening, When::Inactive] {
                assert!(has(w), "{who:?} has nothing for {w:?}");
            }
            for s in Sabbat::ALL {
                assert!(has(When::Approaching(s)), "{who:?} has nothing for {s:?}");
            }
            for s in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
                assert!(has(When::InSeason(s)), "{who:?} has nothing for {s:?}");
            }
        }
    }

    #[test]
    fn conversations_advance_step_by_step() {
        // 12 Nov 2026, afternoon: waxing crescent. Find a multi-step entry.
        let a = at(2026, 11, 12, 15);
        let mut d = Director::new(Character::Hermit, 7);
        let mut now = 100.0;
        for _ in 0..60 {
            let u = d.on_click(&sit(&a, now));
            if u.more {
                let e = entries(Character::Hermit).iter().find(|e| e.id == u.id).unwrap();
                let mut steps = vec![u];
                for _ in 1..e.steps.len() {
                    now += 5.0;
                    steps.push(d.on_click(&sit(&a, now)));
                }
                assert!(steps.iter().all(|s| s.id == e.id));
                assert!(!steps.last().unwrap().more);
                assert_eq!(steps.last().unwrap().text, e.steps.last().unwrap().en);
                return;
            }
            now += 5.0;
        }
        panic!("never started a conversation");
    }

    #[test]
    fn a_stale_conversation_is_dropped() {
        let a = at(2026, 11, 12, 15);
        let mut d = Director::new(Character::Fool, 3);
        let mut now = 100.0;
        let first = loop {
            let u = d.on_click(&sit(&a, now));
            if u.more {
                break u;
            }
            now += 5.0;
        };
        // Coming back ten minutes later starts something else.
        let later = d.on_click(&sit(&a, now + 600.0));
        assert_ne!(later.id, first.id);
    }

    #[test]
    fn no_immediate_repeats_and_variety() {
        let a = at(2026, 11, 12, 15);
        for who in Character::ALL {
            let mut d = Director::new(who, 42);
            let mut now = 100.0;
            let mut starts = Vec::new();
            while starts.len() < 40 {
                let u = d.on_click(&sit(&a, now));
                // Skip through conversations; count only openings.
                if d.conversation.is_some_and(|(_, i)| i == 0) {
                    starts.push(u.id);
                }
                now += 120.0; // past the conversation patience
            }
            for w in starts.windows(2) {
                assert_ne!(w[0], w[1], "{who:?} repeated {}", w[0]);
            }
            // Within the recent window nothing is repeated.
            for w in starts.windows(RECENT) {
                let set: HashSet<_> = w.iter().collect();
                assert_eq!(set.len(), w.len(), "{who:?} repeated within {RECENT}: {w:?}");
            }
        }
    }

    #[test]
    fn a_fresh_event_comes_first_then_cools_down() {
        // 1 Nov 2026 in the south: Beltane today.
        let a = at(2026, 11, 1, 12);
        for who in Character::ALL {
            let mut d = Director::new(who, 9);
            let u = d.on_click(&sit(&a, 100.0));
            let e = entries(who).iter().find(|e| e.id == u.id).unwrap();
            assert_eq!(e.when, When::SabbatToday, "{who:?} opened with {}", u.id);
            assert!(u.text.contains("Beltane"), "{}", u.text);
            // The event does not monopolise later clicks within its cooldown.
            let mut today = 0;
            for k in 1..20 {
                let u = d.on_click(&sit(&a, 100.0 + k as f64 * 120.0));
                if entries(who).iter().any(|e| e.id == u.id && e.when == When::SabbatToday) {
                    today += 1;
                }
            }
            assert!(today <= 1, "{who:?} kept talking about the festival ({today}×)");
        }
    }

    #[test]
    fn full_moon_is_noticed() {
        let a = at(2026, 11, 24, 15); // full moon in São Paulo
        for who in Character::ALL {
            let mut d = Director::new(who, 1);
            let u = d.on_click(&sit(&a, 100.0));
            let e = entries(who).iter().find(|e| e.id == u.id).unwrap();
            // Litha is not within two weeks; the full moon is the top event.
            assert_eq!(e.when, When::FullMoon, "{who:?}: {}", u.id);
        }
    }

    #[test]
    fn placeholders_are_filled_in_both_languages() {
        let a = at(2026, 11, 12, 15);
        let e = entries(Character::Hermit).iter().find(|e| e.id == "h.countdown").unwrap();
        let d = Director::new(Character::Hermit, 1);
        let en = d.utter(e, 0, &Situation { almanac: &a, lang: Lang::En, now: 0.0 });
        let pt = d.utter(e, 0, &Situation { almanac: &a, lang: Lang::PtBr, now: 0.0 });
        assert_eq!(en.text, "Litha is 39 days away. I am in no hurry. Neither should you be.");
        assert_eq!(pt.text, "Faltam 39 dias para Litha. Não tenho pressa. Você também não deveria ter.");
    }

    /// Simulate a working day of checks every five minutes, with no clicks.
    fn unprompted_over_a_day(who: Character, seed: u64) -> usize {
        let mut d = Director::new(who, seed);
        let mut said = 0;
        let mut now = 0.0;
        while now < 12.0 * 3600.0 {
            // 08:00 to 20:00, an ordinary waxing-moon day with no festival near.
            let hour = 8 + (now / 3600.0) as u32;
            let a = at(2026, 11, 12, hour.min(23));
            if d.spontaneous(&sit(&a, now)).is_some() {
                said += 1;
            }
            now += 300.0;
        }
        said
    }

    #[test]
    fn the_hermit_speaks_less_than_the_fool() {
        let (mut hermit, mut fool) = (0, 0);
        for seed in 1..=20 {
            hermit += unprompted_over_a_day(Character::Hermit, seed);
            fool += unprompted_over_a_day(Character::Fool, seed);
        }
        assert!(hermit < fool, "hermit {hermit} vs fool {fool}");
        // The Fool is lively but not a notification stream: a few times a day.
        assert!(fool / 20 <= 12, "fool spoke {} times a day", fool / 20);
        assert!(hermit / 20 <= 3, "hermit spoke {} times a day", hermit / 20);
    }

    #[test]
    fn nobody_speaks_unprompted_at_night() {
        let a = at(2026, 11, 12, 23);
        for who in Character::ALL {
            let mut d = Director::new(who, 5);
            for k in 0..200 {
                assert!(d.spontaneous(&sit(&a, 4.0 * 3600.0 + k as f64 * 300.0)).is_none());
            }
        }
    }

    #[test]
    fn the_fool_calls_after_a_long_quiet_but_not_soon_after_a_click() {
        let a = at(2026, 11, 12, 15);
        let mut d = Director::new(Character::Fool, 11);
        d.on_click(&sit(&a, 100.0));
        // Shortly after talking: silence.
        for k in 1..8 {
            assert!(d.spontaneous(&sit(&a, 100.0 + k as f64 * 300.0)).is_none());
        }
        // After a long quiet, eventually a call.
        let mut called = false;
        for k in 0..100 {
            if let Some(u) = d.spontaneous(&sit(&a, 3.0 * 3600.0 + k as f64 * 300.0)) {
                let e = entries(Character::Fool).iter().find(|e| e.id == u.id).unwrap();
                called |= e.when == When::Inactive;
                break;
            }
        }
        assert!(called);
    }
}
