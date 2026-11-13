//! The computed snapshot ("almanac") rendered by the UI.
//!
//! Everything here is a pure function of (instant, time zone, settings), so it
//! is deterministic and unit-testable with fixed offsets.

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc};

use super::{CrossQuarterRule, DateBasis, Hemisphere, MoonPhase, Sabbat, Season};
use crate::astro::moon::{self, PrincipalPhase};
use crate::astro::sun::{self, SolarEvent};
use crate::astro::time::{jde_from_utc, utc_from_jde};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CalendarSettings {
    pub hemisphere: Hemisphere,
    pub cross_quarters: CrossQuarterRule,
}

#[derive(Debug, Clone)]
pub struct SeasonInfo {
    pub season: Season,
    pub hemisphere: Hemisphere,
    pub start_event: SolarEvent,
    pub start: DateTime<FixedOffset>,
    pub end_event: SolarEvent,
    pub end: DateTime<FixedOffset>,
    /// Fraction of the season elapsed, 0.0 … 1.0.
    pub progress: f64,
    /// Whole local days until the season ends (0 on the final day).
    pub days_remaining: i64,
}

#[derive(Debug, Clone)]
pub struct SabbatOccurrence {
    pub sabbat: Sabbat,
    pub basis: DateBasis,
    /// Local calendar date of the festival.
    pub date: NaiveDate,
    /// Exact local instant, for astronomically determined dates.
    pub instant: Option<DateTime<FixedOffset>>,
    /// For cross-quarter festivals: the alternative date under the other
    /// rule (astronomical point when traditional is used, and vice versa).
    pub alternative: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct WheelInfo {
    /// Most recent festival on or before today.
    pub previous: SabbatOccurrence,
    /// First festival strictly after today.
    pub next: SabbatOccurrence,
    /// Today is itself a festival day (`previous.date == today`).
    pub on_festival: bool,
    /// Fraction of the interval between the two festivals, 0.0 … 1.0.
    pub progress: f64,
    /// Apparent solar longitude now, degrees.
    pub sun_longitude: f64,
}

#[derive(Debug, Clone)]
pub struct NextSabbat {
    pub occurrence: SabbatOccurrence,
    /// 0 when the festival is today.
    pub days_until: i64,
}

#[derive(Debug, Clone)]
pub struct MoonInfo {
    pub phase: MoonPhase,
    /// Illuminated fraction, 0.0 … 1.0.
    pub illumination: f64,
    /// Days since the most recent new moon.
    pub age_days: f64,
    pub waxing: bool,
    pub lunation_start: DateTime<FixedOffset>,
    /// The next four principal phases after now.
    pub upcoming: [(PrincipalPhase, DateTime<FixedOffset>); 4],
}

#[derive(Debug, Clone)]
pub struct Almanac {
    pub computed_at: DateTime<FixedOffset>,
    pub today: NaiveDate,
    pub settings: CalendarSettings,
    pub season: SeasonInfo,
    pub moon: MoonInfo,
    pub wheel: WheelInfo,
    pub next_sabbat: NextSabbat,
}

fn local<Tz: TimeZone>(t: DateTime<Utc>, tz: &Tz) -> DateTime<FixedOffset> {
    t.with_timezone(tz).fixed_offset()
}

fn solar_event_utc(year: i32, ev: SolarEvent) -> DateTime<Utc> {
    utc_from_jde(sun::solar_event_jde(year, ev))
}

/// UTC instant at which the Sun reaches `longitude` within calendar year `year`.
fn solar_longitude_utc(year: i32, longitude: f64) -> DateTime<Utc> {
    // On 1 January the Sun is near 280°; it advances ≈0.9856°/day.
    let jan1 = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
    let days = (longitude - 280.0).rem_euclid(360.0) / 360.0 * 365.2422;
    let guess = jde_from_utc(jan1) + days;
    utc_from_jde(sun::jde_of_solar_longitude(longitude, guess))
}

/// The occurrence of `sabbat` within calendar year `year`.
pub fn sabbat_in_year<Tz: TimeZone>(
    sabbat: Sabbat,
    year: i32,
    tz: &Tz,
    settings: CalendarSettings,
) -> SabbatOccurrence {
    let basis = sabbat.date_basis(settings.hemisphere, settings.cross_quarters);
    let lon = f64::from(sabbat.solar_longitude(settings.hemisphere));
    match basis {
        DateBasis::SolarEvent(ev) => {
            let t = local(solar_event_utc(year, ev), tz);
            SabbatOccurrence { sabbat, basis, date: t.date_naive(), instant: Some(t), alternative: None }
        }
        DateBasis::CrossQuarterAstronomical { .. } => {
            let t = local(solar_longitude_utc(year, lon), tz);
            let alt = sabbat.date_basis(settings.hemisphere, CrossQuarterRule::Traditional);
            let alternative = match alt {
                DateBasis::CrossQuarterTraditional { month, day } => NaiveDate::from_ymd_opt(year, month, day),
                _ => None,
            };
            SabbatOccurrence { sabbat, basis, date: t.date_naive(), instant: Some(t), alternative }
        }
        DateBasis::CrossQuarterTraditional { month, day } => {
            let date = NaiveDate::from_ymd_opt(year, month, day).expect("valid fixed festival date");
            let astro = local(solar_longitude_utc(year, lon), tz).date_naive();
            SabbatOccurrence { sabbat, basis, date, instant: None, alternative: Some(astro) }
        }
    }
}

/// All festivals in calendar years `year-1 ..= year+1`, sorted by date.
fn occurrences_around<Tz: TimeZone>(year: i32, tz: &Tz, settings: CalendarSettings) -> Vec<SabbatOccurrence> {
    let mut v = Vec::with_capacity(24);
    for y in (year - 1)..=(year + 1) {
        for s in Sabbat::ALL {
            v.push(sabbat_in_year(s, y, tz, settings));
        }
    }
    v.sort_by_key(|o| o.date);
    v
}

fn season_info<Tz: TimeZone>(now: DateTime<Utc>, tz: &Tz, hemisphere: Hemisphere) -> SeasonInfo {
    let year = now.year();
    let mut events: Vec<(SolarEvent, DateTime<Utc>)> = Vec::with_capacity(12);
    for y in (year - 1)..=(year + 1) {
        for ev in SolarEvent::ALL {
            events.push((ev, solar_event_utc(y, ev)));
        }
    }
    events.sort_by_key(|&(_, t)| t);
    let i = events.iter().rposition(|&(_, t)| t <= now).expect("an event in the previous year");
    let (start_event, start) = events[i];
    let (end_event, end) = events[i + 1];
    let progress = (now - start).num_seconds() as f64 / (end - start).num_seconds() as f64;
    let today = local(now, tz).date_naive();
    let end_local = local(end, tz);
    SeasonInfo {
        season: hemisphere.season_beginning_at(start_event),
        hemisphere,
        start_event,
        start: local(start, tz),
        end_event,
        end: end_local,
        progress: progress.clamp(0.0, 1.0),
        days_remaining: (end_local.date_naive() - today).num_days(),
    }
}

fn moon_info<Tz: TimeZone>(now: DateTime<Utc>, tz: &Tz) -> MoonInfo {
    let jde = jde_from_utc(now);
    let k = moon::lunation_at(jde);
    let today = local(now, tz).date_naive();

    // Principal phases of this lunation plus the following new moon.
    let seq: [(PrincipalPhase, i64); 9] = [
        (PrincipalPhase::New, k),
        (PrincipalPhase::FirstQuarter, k),
        (PrincipalPhase::Full, k),
        (PrincipalPhase::LastQuarter, k),
        (PrincipalPhase::New, k + 1),
        (PrincipalPhase::FirstQuarter, k + 1),
        (PrincipalPhase::Full, k + 1),
        (PrincipalPhase::LastQuarter, k + 1),
        (PrincipalPhase::New, k + 2),
    ];
    let times: [(PrincipalPhase, DateTime<Utc>); 9] =
        seq.map(|(p, kk)| (p, utc_from_jde(moon::phase_jde(kk, p))));

    // A principal phase names the whole local calendar day on which it falls
    // (the almanac convention); otherwise the intermediate phase applies.
    let principal_today = times[..5]
        .iter()
        .find(|(_, t)| local(*t, tz).date_naive() == today)
        .map(|&(p, _)| p);
    let phase = match principal_today {
        Some(PrincipalPhase::New) => MoonPhase::New,
        Some(PrincipalPhase::FirstQuarter) => MoonPhase::FirstQuarter,
        Some(PrincipalPhase::Full) => MoonPhase::Full,
        Some(PrincipalPhase::LastQuarter) => MoonPhase::LastQuarter,
        None => {
            if now < times[1].1 {
                MoonPhase::WaxingCrescent
            } else if now < times[2].1 {
                MoonPhase::WaxingGibbous
            } else if now < times[3].1 {
                MoonPhase::WaningGibbous
            } else {
                MoonPhase::WaningCrescent
            }
        }
    };

    let mut upcoming = times.iter().filter(|(_, t)| *t > now).map(|&(p, t)| (p, local(t, tz)));
    let upcoming: [(PrincipalPhase, DateTime<FixedOffset>); 4] =
        std::array::from_fn(|_| upcoming.next().expect("at least four future phases"));

    MoonInfo {
        phase,
        illumination: moon::illuminated_fraction(jde),
        age_days: jde - moon::phase_jde(k, PrincipalPhase::New),
        waxing: now < times[2].1,
        lunation_start: local(times[0].1, tz),
        upcoming,
    }
}

impl Almanac {
    pub fn compute<Tz: TimeZone>(now: DateTime<Utc>, tz: &Tz, settings: CalendarSettings) -> Almanac {
        let computed_at = local(now, tz);
        let today = computed_at.date_naive();

        let occ = occurrences_around(today.year(), tz, settings);
        let prev_i = occ.iter().rposition(|o| o.date <= today).expect("festival in previous year");
        let next_i = occ.iter().position(|o| o.date > today).expect("festival in following year");
        let previous = occ[prev_i].clone();
        let next = occ[next_i].clone();
        let on_festival = previous.date == today;
        let span = (next.date - previous.date).num_days().max(1) as f64;
        let progress = (today - previous.date).num_days() as f64 / span;

        let upcoming = if on_festival { previous.clone() } else { next.clone() };
        let days_until = (upcoming.date - today).num_days();

        Almanac {
            computed_at,
            today,
            settings,
            season: season_info(now, tz, settings.hemisphere),
            moon: moon_info(now, tz),
            wheel: WheelInfo {
                previous,
                next,
                on_festival,
                progress,
                sun_longitude: sun::apparent_longitude(jde_from_utc(now)),
            },
            next_sabbat: NextSabbat { occurrence: upcoming, days_until },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }
    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
    fn south() -> CalendarSettings {
        CalendarSettings { hemisphere: Hemisphere::Southern, cross_quarters: CrossQuarterRule::Traditional }
    }
    fn north() -> CalendarSettings {
        CalendarSettings { hemisphere: Hemisphere::Northern, cross_quarters: CrossQuarterRule::Traditional }
    }
    /// UTC−3 (e.g. São Paulo, no DST since 2019).
    fn brt() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    #[test]
    fn southern_spring_in_november() {
        let a = Almanac::compute(utc(2026, 11, 12, 21, 0), &brt(), south());
        assert_eq!(a.season.season, Season::Spring);
        assert_eq!(a.season.start_event, SolarEvent::SeptemberEquinox);
        assert_eq!(a.season.end_event, SolarEvent::DecemberSolstice);
        // Equinox 2026-09-23 00:05 UTC is 22 Sep 21:05 in UTC−3.
        assert_eq!(a.season.start.date_naive(), date(2026, 9, 22));
        assert_eq!(a.season.end.date_naive(), date(2026, 12, 21));
    }

    #[test]
    fn northern_autumn_in_november() {
        let a = Almanac::compute(utc(2026, 11, 12, 21, 0), &Utc, north());
        assert_eq!(a.season.season, Season::Autumn);
        assert_eq!(a.season.start.date_naive(), date(2026, 9, 23));
    }

    #[test]
    fn time_zone_changes_event_date() {
        // Same instant, different local dates for the 2026 September equinox.
        let s = sabbat_in_year(Sabbat::Ostara, 2026, &brt(), south());
        assert_eq!(s.date, date(2026, 9, 22));
        let s = sabbat_in_year(Sabbat::Ostara, 2026, &Utc, south());
        assert_eq!(s.date, date(2026, 9, 23));
        let tokyo = FixedOffset::east_opt(9 * 3600).unwrap();
        let s = sabbat_in_year(Sabbat::Mabon, 2026, &tokyo, north());
        assert_eq!(s.date, date(2026, 9, 23));
    }

    #[test]
    fn season_boundary_is_instant_exact() {
        // 2026 December solstice: 21 Dec 20:50 UTC.
        let before = Almanac::compute(utc(2026, 12, 21, 20, 40), &Utc, south());
        let after = Almanac::compute(utc(2026, 12, 21, 21, 0), &Utc, south());
        assert_eq!(before.season.season, Season::Spring);
        assert_eq!(after.season.season, Season::Summer);
        assert_eq!(after.season.end_event, SolarEvent::MarchEquinox);
        assert_eq!(after.season.end.year(), 2027);
    }

    #[test]
    fn next_sabbat_countdown_southern() {
        // 12 Nov 2026, Southern: Beltane (1 Nov) has passed; next is Litha on
        // the December solstice, 21 Dec 2026 (17:50 local in UTC−3).
        let a = Almanac::compute(utc(2026, 11, 12, 21, 0), &brt(), south());
        assert_eq!(a.wheel.previous.sabbat, Sabbat::Beltane);
        assert_eq!(a.wheel.previous.date, date(2026, 11, 1));
        assert_eq!(a.next_sabbat.occurrence.sabbat, Sabbat::Litha);
        assert_eq!(a.next_sabbat.occurrence.date, date(2026, 12, 21));
        assert_eq!(a.next_sabbat.days_until, 39);
    }

    #[test]
    fn countdown_crosses_year_boundary() {
        // Northern, 25 Dec 2026: next is Imbolc on 1 Feb 2027.
        let a = Almanac::compute(utc(2026, 12, 25, 12, 0), &Utc, north());
        assert_eq!(a.next_sabbat.occurrence.sabbat, Sabbat::Imbolc);
        assert_eq!(a.next_sabbat.occurrence.date, date(2027, 2, 1));
        assert_eq!(a.next_sabbat.days_until, 38);
        // Southern, 31 Dec 2026: next is Lughnasadh, 1 Feb 2027.
        let a = Almanac::compute(utc(2026, 12, 31, 23, 0), &Utc, south());
        assert_eq!(a.next_sabbat.occurrence.sabbat, Sabbat::Lughnasadh);
        assert_eq!(a.next_sabbat.days_until, 32);
    }

    #[test]
    fn countdown_across_leap_day() {
        // 2028 is a leap year: 1 Feb → 20 Mar 2028 spans 29 Feb.
        let a = Almanac::compute(utc(2028, 2, 2, 12, 0), &Utc, north());
        assert_eq!(a.next_sabbat.occurrence.sabbat, Sabbat::Ostara);
        let eq = a.next_sabbat.occurrence.date;
        assert_eq!(eq, date(2028, 3, 20));
        assert_eq!(a.next_sabbat.days_until, 47); // 27 (Feb, incl. 29th) + 20
    }

    #[test]
    fn festival_day_counts_as_today() {
        let a = Almanac::compute(utc(2026, 11, 1, 15, 0), &brt(), south());
        assert!(a.wheel.on_festival);
        assert_eq!(a.next_sabbat.occurrence.sabbat, Sabbat::Beltane);
        assert_eq!(a.next_sabbat.days_until, 0);
        assert_eq!(a.wheel.next.sabbat, Sabbat::Litha);
    }

    #[test]
    fn wheel_is_complete_and_ordered_each_year() {
        for settings in [north(), south()] {
            for rule in [CrossQuarterRule::Traditional, CrossQuarterRule::Astronomical] {
                let s = CalendarSettings { cross_quarters: rule, ..settings };
                let occ = occurrences_around(2026, &Utc, s);
                assert_eq!(occ.len(), 24);
                // Consecutive festivals follow wheel order with 38–55 day gaps.
                for w in occ.windows(2) {
                    assert_eq!(w[0].sabbat.next(), w[1].sabbat, "{:?}", s);
                    let gap = (w[1].date - w[0].date).num_days();
                    assert!((38..=55).contains(&gap), "{:?} → {:?}: {gap}", w[0].sabbat, w[1].sabbat);
                }
            }
        }
    }

    #[test]
    fn astronomical_cross_quarters_fall_in_early_month() {
        let s = CalendarSettings { hemisphere: Hemisphere::Northern, cross_quarters: CrossQuarterRule::Astronomical };
        for (sab, month) in [(Sabbat::Imbolc, 2), (Sabbat::Beltane, 5), (Sabbat::Lughnasadh, 8), (Sabbat::Samhain, 11)] {
            let o = sabbat_in_year(sab, 2026, &Utc, s);
            assert_eq!(o.date.month(), month, "{sab:?}");
            assert!((3..=9).contains(&o.date.day()), "{sab:?} {}", o.date);
            assert_eq!(o.alternative, Some(date(2026, month, 1)));
        }
    }

    #[test]
    fn moon_phase_names() {
        // Full moon 2025-09-07 18:09 UTC | named Full all that local day.
        let a = Almanac::compute(utc(2025, 9, 7, 2, 0), &Utc, south());
        assert_eq!(a.moon.phase, MoonPhase::Full);
        let a = Almanac::compute(utc(2025, 9, 8, 12, 0), &Utc, south());
        assert_eq!(a.moon.phase, MoonPhase::WaningGibbous);
        assert!(!a.moon.waxing);
        // New moon 2024-04-08 18:21 UTC; two days later: waxing crescent.
        let a = Almanac::compute(utc(2024, 4, 10, 18, 0), &Utc, north());
        assert_eq!(a.moon.phase, MoonPhase::WaxingCrescent);
        assert!((a.moon.age_days - 2.0).abs() < 0.05);
        assert!(a.moon.illumination < 0.1);
    }

    #[test]
    fn principal_phase_respects_local_date() {
        // Full moon 2025-09-07 18:09 UTC = 8 Sep 03:09 in UTC+9.
        let tokyo = FixedOffset::east_opt(9 * 3600).unwrap();
        let a = Almanac::compute(utc(2025, 9, 7, 16, 0), &tokyo, north()); // 8 Sep 01:00 local
        assert_eq!(a.moon.phase, MoonPhase::Full);
        let a = Almanac::compute(utc(2025, 9, 7, 12, 0), &tokyo, north()); // 7 Sep 21:00 local
        assert_eq!(a.moon.phase, MoonPhase::WaxingGibbous);
    }

    #[test]
    fn upcoming_phases_are_future_and_ordered() {
        let now = utc(2026, 11, 12, 21, 0);
        let a = Almanac::compute(now, &Utc, south());
        let mut prev = now.fixed_offset();
        for (_, t) in a.moon.upcoming {
            assert!(t > prev);
            prev = t;
        }
        assert!(a.moon.upcoming[3].0 != a.moon.upcoming[0].0);
    }
}
