//! What the companion feels and says, derived from the almanac.
//!
//! Pure functions: the UI decides how a mood looks; this module decides which
//! mood applies and which lines are worth saying, in priority order.

use chrono::Timelike;
use serde::{Deserialize, Serialize};

use crate::calendar::MoonPhase;
use crate::calendar::almanac::Almanac;
use crate::calendar::lore;
use crate::i18n::Lang;

/// Which card figure keeps you company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Character {
    /// IX — the Hermit with his lantern.
    #[default]
    Hermit,
    /// 0 — the Fool, with his little dog.
    Fool,
}

impl Character {
    pub const ALL: [Character; 2] = [Character::Hermit, Character::Fool];

    pub fn name(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Character::Hermit, Lang::En) => "The Hermit",
            (Character::Hermit, Lang::PtBr) => "O Eremita",
            (Character::Fool, Lang::En) => "The Fool",
            (Character::Fool, Lang::PtBr) => "O Louco",
        }
    }

    pub fn config_value(self) -> &'static str {
        match self {
            Character::Hermit => "hermit",
            Character::Fool => "fool",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Idle,
    /// A festival is today or close, or the moon is full: lantern raised.
    Happy,
    /// New moon or late night: head bowed, lantern low.
    Sleepy,
}

/// Days before a Sabbat during which the companion is cheerful.
const HAPPY_WINDOW_DAYS: i64 = 3;

fn is_night(a: &Almanac) -> bool {
    let h = a.computed_at.hour();
    !(6..23).contains(&h)
}

pub fn mood(a: &Almanac) -> Mood {
    if a.wheel.on_festival || a.next_sabbat.days_until <= HAPPY_WINDOW_DAYS || a.moon.phase == MoonPhase::Full {
        Mood::Happy
    } else if a.moon.phase == MoonPhase::New || is_night(a) {
        Mood::Sleepy
    } else {
        Mood::Idle
    }
}

/// Lines worth saying now, most relevant first. Never empty.
pub fn lines(a: &Almanac, lang: Lang, who: Character) -> Vec<String> {
    let pt = lang == Lang::PtBr;
    let next = &a.next_sabbat.occurrence;
    let name = next.sabbat.name();
    let epithet = lore::sabbat(next.sabbat, lang).epithet;
    let days = a.next_sabbat.days_until;
    let mut v = Vec::with_capacity(6);

    // Festival and its approach.
    match days {
        0 => v.push(if pt { format!("Hoje é {name}! {epithet}.") } else { format!("Today is {name}! {epithet}.") }),
        1 => v.push(if pt { format!("{name} é amanhã. {epithet}.") } else { format!("{name} is tomorrow. {epithet}.") }),
        _ => {}
    }

    // Notable moon.
    let line = |en: &str, ptbr: &str| if pt { ptbr.to_string() } else { en.to_string() };
    match (a.moon.phase, who) {
        (MoonPhase::Full, Character::Hermit) => {
            v.push(line("Full moon tonight. My lantern can rest.", "Lua cheia esta noite. Minha lanterna pode descansar."))
        }
        (MoonPhase::Full, Character::Fool) => v.push(line(
            "Full moon tonight! A fine night for a walk. Mind the edge.",
            "Lua cheia esta noite! Ótima noite para caminhar. Cuidado com a beira.",
        )),
        (MoonPhase::New, Character::Hermit) => v.push(line(
            "New moon: a dark sky. Good thing I brought my lantern.",
            "Lua nova: céu escuro. Ainda bem que trouxe a lanterna.",
        )),
        (MoonPhase::New, Character::Fool) => v.push(line(
            "New moon. Dark sky, new beginnings. Where shall we go?",
            "Lua nova. Céu escuro, novos começos. Para onde vamos?",
        )),
        _ => {}
    }

    // A season about to turn.
    if (0..=3).contains(&a.season.days_remaining) {
        let season = lang.season(a.season.season);
        let n = lang.days(a.season.days_remaining);
        v.push(match (pt, a.season.days_remaining) {
            (true, 0) => format!("{season}: último dia da estação."),
            (false, 0) => format!("Last day of {}.", season.to_lowercase()),
            (true, _) => format!("{season}: faltam {n} para acabar."),
            (false, _) => format!("{season} ends in {n}."),
        });
    }

    if is_night(a) && !a.wheel.on_festival {
        v.push(match who {
            Character::Hermit => line("It's late. The lantern burns low.", "Já é tarde. A lanterna está fraca."),
            Character::Fool => line("It's late. Even my dog is yawning.", "Já é tarde. Até meu cachorro está bocejando."),
        });
    }

    // Everyday lines.
    if days > 1 {
        let n = lang.days(days);
        v.push(if pt { format!("{name} em {n}. {epithet}.") } else { format!("{name} in {n}. {epithet}.") });
    }
    let pct = (a.moon.illumination * 100.0).round();
    let phase = lang.moon_phase(a.moon.phase);
    v.push(if pt { format!("{phase}, {pct}% iluminada.") } else { format!("{phase}, {pct}% lit.") });
    let period = lore::period_title(a.wheel.previous.sabbat, lang);
    let (from, to) = (a.wheel.previous.sabbat.name(), a.wheel.next.sabbat.name());
    v.push(if pt {
        format!("{period}: entre {from} e {to}.")
    } else {
        format!("{period}: between {from} and {to}.")
    });

    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::almanac::CalendarSettings;
    use crate::calendar::{CrossQuarterRule, Hemisphere};
    use chrono::{FixedOffset, TimeZone, Utc};

    fn at(y: i32, mo: u32, d: u32, h: u32) -> Almanac {
        let tz = FixedOffset::west_opt(3 * 3600).unwrap();
        let settings = CalendarSettings { hemisphere: Hemisphere::Southern, cross_quarters: CrossQuarterRule::Traditional };
        // Local time `h` in UTC−3.
        let utc = Utc.with_ymd_and_hms(y, mo, d, 0, 0, 0).unwrap() + chrono::TimeDelta::hours(i64::from(h) + 3);
        Almanac::compute(utc, &tz, settings)
    }

    #[test]
    fn ordinary_afternoon_is_idle_and_counts_down() {
        let a = at(2026, 11, 12, 15); // waxing crescent, Litha in 39 days
        assert_eq!(mood(&a), Mood::Idle);
        let l = lines(&a, Lang::En, Character::Hermit);
        assert!(l[0].starts_with("Litha in 39 days"), "{l:?}");
        let l = lines(&a, Lang::PtBr, Character::Hermit);
        assert!(l[0].starts_with("Litha em 39 dias"), "{l:?}");
    }

    #[test]
    fn festival_day_is_happy_and_announced_first() {
        let a = at(2026, 11, 1, 12); // Beltane in the south
        assert_eq!(mood(&a), Mood::Happy);
        assert!(lines(&a, Lang::En, Character::Hermit)[0].starts_with("Today is Beltane!"));
        assert!(lines(&a, Lang::PtBr, Character::Hermit)[0].starts_with("Hoje é Beltane!"));
    }

    #[test]
    fn late_night_is_sleepy() {
        let a = at(2026, 11, 12, 23);
        assert_eq!(mood(&a), Mood::Sleepy);
        assert!(lines(&a, Lang::En, Character::Hermit).iter().any(|l| l.contains("late")));
    }

    #[test]
    fn full_moon_is_happy() {
        let a = at(2026, 11, 24, 15); // full moon 24 Nov 2026
        assert_eq!(mood(&a), Mood::Happy);
        assert!(lines(&a, Lang::En, Character::Hermit).iter().any(|l| l.starts_with("Full moon tonight")));
    }

    #[test]
    fn each_character_has_its_own_voice() {
        let a = at(2026, 11, 24, 15); // full moon
        let h = lines(&a, Lang::En, Character::Hermit);
        let f = lines(&a, Lang::En, Character::Fool);
        assert!(h.iter().any(|l| l.contains("lantern")));
        assert!(f.iter().any(|l| l.contains("walk")));
        assert!(!f.iter().any(|l| l.contains("lantern")));
        let a = at(2026, 11, 12, 23);
        assert!(lines(&a, Lang::PtBr, Character::Fool).iter().any(|l| l.contains("cachorro")));
    }

    #[test]
    fn always_has_something_to_say() {
        for d in 1..=28 {
            let a = at(2027, 2, d, 10);
            assert!(!lines(&a, Lang::En, Character::Hermit).is_empty());
            assert!(!lines(&a, Lang::PtBr, Character::Hermit).is_empty());
        }
    }
}
