//! Display strings, formatted once per recomputation rather than every frame.

use sabbatia::calendar::almanac::Almanac;
use sabbatia::calendar::{DateBasis, lore};
use sabbatia::i18n::Lang;

pub struct Texts {
    pub date_long: String,
    pub utc_offset: String,

    pub season_title: &'static str,
    pub season_range: String,
    pub season_description: &'static str,
    pub season_began: String,
    pub season_ends: String,
    pub season_progress: String,

    pub moon_title: &'static str,
    pub moon_sub: String,
    pub moon_illumination: String,
    pub moon_age: String,
    pub moon_lunation: String,
    pub moon_upcoming: Vec<(&'static str, String)>,
    pub moon_note: &'static str,

    pub wheel_title: String,
    pub wheel_sub: String,
    pub wheel_description: &'static str,
    pub wheel_since: String,
    pub wheel_until: String,
    pub wheel_sun: String,

    pub sabbat_name: &'static str,
    pub sabbat_countdown: String,
    pub sabbat_when: String,
    pub sabbat_date: String,
    pub sabbat_basis: String,
    pub sabbat_alternative: Option<String>,
}

impl Texts {
    pub fn build(a: &Almanac, lang: Lang) -> Texts {
        let ui = lang.ui();
        let pt = lang == Lang::PtBr;
        let s = &a.season;
        let m = &a.moon;
        let w = &a.wheel;
        let n = &a.next_sabbat.occurrence;

        let season_len = (s.end.date_naive() - s.start.date_naive()).num_days();
        let season_day = (a.today - s.start.date_naive()).num_days() + 1;

        let time = |t: Option<chrono::DateTime<chrono::FixedOffset>>| t.map(|t| lang.time(t)).unwrap_or_default();
        let (sabbat_basis, sabbat_alternative, when_kind) = match n.basis {
            DateBasis::SolarEvent(ev) => (
                if pt {
                    format!("Astronômica: {} às {}", lang.solar_event(ev), time(n.instant))
                } else {
                    format!("Astronomical: {} at {}", lang.solar_event(ev), time(n.instant))
                },
                None,
                lang.solar_event_kind(ev),
            ),
            DateBasis::CrossQuarterAstronomical { longitude } => (
                if pt {
                    format!("Astronômica: Sol a {longitude}° de longitude eclíptica, {}", time(n.instant))
                } else {
                    format!("Astronomical: Sun at {longitude}° ecliptic longitude, {}", time(n.instant))
                },
                n.alternative.map(|d| {
                    if pt {
                        format!("Data tradicional: {}", lang.date_short_year(d))
                    } else {
                        format!("Traditional date: {}", lang.date_short_year(d))
                    }
                }),
                if pt { "ponto médio" } else { "midpoint" },
            ),
            DateBasis::CrossQuarterTraditional { .. } => {
                let eve = lang.day_month(n.date.pred_opt().unwrap_or(n.date));
                (
                    if pt {
                        format!("Data fixa tradicional, celebrada desde a véspera ({eve})")
                    } else {
                        format!("Traditional fixed date, celebrated from the eve ({eve})")
                    },
                    n.alternative.map(|d| {
                        if pt {
                            format!("Ponto médio astronômico: {}", lang.date_short_year(d))
                        } else {
                            format!("Astronomical midpoint: {}", lang.date_short_year(d))
                        }
                    }),
                    if pt { "data fixa" } else { "fixed date" },
                )
            }
        };

        let countdown = match a.next_sabbat.days_until {
            0 => ui.today.to_string(),
            1 => ui.tomorrow.to_string(),
            d => lang.days(d),
        };

        let pct = lang.decimal(m.illumination * 100.0, 0);
        let viewer_note = match (lang, a.settings.hemisphere) {
            (Lang::En, sabbatia::calendar::Hemisphere::Southern) => {
                "Drawn as seen from the Southern Hemisphere. Times are local."
            }
            (Lang::En, _) => "Drawn as seen from the Northern Hemisphere. Times are local.",
            (Lang::PtBr, sabbatia::calendar::Hemisphere::Southern) => {
                "Desenhada como vista do Hemisfério Sul. Horários locais."
            }
            (Lang::PtBr, _) => "Desenhada como vista do Hemisfério Norte. Horários locais.",
        };

        Texts {
            date_long: lang.date_long(a.today),
            utc_offset: a.computed_at.format("UTC%:z").to_string(),

            season_title: lang.season(s.season),
            season_range: format!("{} – {}", lang.day_month(s.start.date_naive()), lang.day_month(s.end.date_naive())),
            season_description: lore::season_description(s.season, lang),
            season_began: format!("{} · {}", lang.solar_event(s.start_event), lang.date_time(s.start)),
            season_ends: format!("{} · {}", lang.solar_event(s.end_event), lang.date_time(s.end)),
            season_progress: if pt {
                format!("Dia {season_day} de {season_len} · faltam {}", lang.days(s.days_remaining))
            } else {
                format!("Day {season_day} of {season_len} · {} left", lang.days(s.days_remaining))
            },

            moon_title: lang.moon_phase(m.phase),
            moon_sub: if pt {
                format!("{pct}% iluminada · {} dias", lang.decimal(m.age_days, 1))
            } else {
                format!("{pct}% lit · {} days", lang.decimal(m.age_days, 1))
            },
            moon_illumination: format!("{}%", lang.decimal(m.illumination * 100.0, 1)),
            moon_age: if pt {
                format!("{} dias desde a lua nova", lang.decimal(m.age_days, 1))
            } else {
                format!("{} days since new moon", lang.decimal(m.age_days, 1))
            },
            moon_lunation: lang.date_time(m.lunation_start),
            moon_upcoming: m.upcoming.iter().map(|&(p, t)| (lang.principal_phase(p), lang.date_time(t))).collect(),
            moon_note: viewer_note,

            wheel_title: if w.on_festival {
                format!("{} · {}", w.previous.sabbat.name(), ui.today.to_lowercase())
            } else {
                lore::period_title(w.previous.sabbat, lang).to_string()
            },
            wheel_sub: format!("{} – {}", w.previous.sabbat.name(), w.next.sabbat.name()),
            wheel_description: lore::period_after(w.previous.sabbat, lang),
            wheel_since: format!("{} · {}", w.previous.sabbat.name(), lang.date_short_year(w.previous.date)),
            wheel_until: format!("{} · {}", w.next.sabbat.name(), lang.date_short_year(w.next.date)),
            wheel_sun: if pt {
                format!("{}° de longitude eclíptica", lang.decimal(w.sun_longitude, 1))
            } else {
                format!("{}° ecliptic longitude", lang.decimal(w.sun_longitude, 1))
            },

            sabbat_name: n.sabbat.name(),
            sabbat_countdown: countdown,
            sabbat_when: format!("{} · {when_kind}", lang.date_short(n.date)),
            sabbat_date: lang.date_short_year(n.date),
            sabbat_basis,
            sabbat_alternative,
        }
    }
}
