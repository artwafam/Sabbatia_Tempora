//! Solar events: equinoxes, solstices and the cross-quarter points.
//!
//! * Equinoxes and solstices: Meeus, *Astronomical Algorithms* (2nd ed.),
//!   chapter 27, tables 27.B and 27.C. Valid for years 1000–3000 with errors
//!   below one minute for the current era.
//! * Arbitrary solar longitudes (used for the astronomical cross-quarter
//!   days at 45°, 135°, 225°, 315°): Meeus chapter 25, "low accuracy" solar
//!   coordinates (≈0.01°, i.e. roughly a quarter of an hour), solved by the
//!   iteration given at the end of chapter 27.

use super::time::normalize_degrees;

/// The four cardinal solar events, named by the month in which they occur so
/// that the naming is independent of hemisphere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolarEvent {
    /// Apparent solar longitude 0°.
    MarchEquinox,
    /// Apparent solar longitude 90°.
    JuneSolstice,
    /// Apparent solar longitude 180°.
    SeptemberEquinox,
    /// Apparent solar longitude 270°.
    DecemberSolstice,
}

impl SolarEvent {
    pub const ALL: [SolarEvent; 4] = [
        SolarEvent::MarchEquinox,
        SolarEvent::JuneSolstice,
        SolarEvent::SeptemberEquinox,
        SolarEvent::DecemberSolstice,
    ];

    pub fn longitude(self) -> f64 {
        match self {
            SolarEvent::MarchEquinox => 0.0,
            SolarEvent::JuneSolstice => 90.0,
            SolarEvent::SeptemberEquinox => 180.0,
            SolarEvent::DecemberSolstice => 270.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SolarEvent::MarchEquinox => "March equinox",
            SolarEvent::JuneSolstice => "June solstice",
            SolarEvent::SeptemberEquinox => "September equinox",
            SolarEvent::DecemberSolstice => "December solstice",
        }
    }
}

/// Supported year range for the chapter 27 series.
pub const MIN_YEAR: i32 = 1000;
pub const MAX_YEAR: i32 = 3000;

/// Table 27.C: periodic terms (A, B, C).
const TABLE_27C: [(f64, f64, f64); 24] = [
    (485.0, 324.96, 1934.136),
    (203.0, 337.23, 32964.467),
    (199.0, 342.08, 20.186),
    (182.0, 27.85, 445267.112),
    (156.0, 73.14, 45036.886),
    (136.0, 171.52, 22518.443),
    (77.0, 222.54, 65928.934),
    (74.0, 296.72, 3034.906),
    (70.0, 243.58, 9037.513),
    (58.0, 119.81, 33718.147),
    (52.0, 297.17, 150.678),
    (50.0, 21.02, 2281.226),
    (45.0, 247.54, 29929.562),
    (44.0, 325.15, 31555.956),
    (29.0, 60.93, 4443.417),
    (18.0, 155.12, 67555.328),
    (17.0, 288.79, 4562.452),
    (16.0, 198.04, 62894.029),
    (14.0, 199.76, 31436.921),
    (12.0, 95.39, 14577.848),
    (12.0, 287.11, 31931.756),
    (12.0, 320.81, 34777.259),
    (9.0, 227.73, 1222.114),
    (8.0, 15.45, 16859.074),
];

/// Mean instant (JDE0) of an event, Meeus table 27.B (years +1000 … +3000).
fn mean_event_jde(year: i32, event: SolarEvent) -> f64 {
    let y = (f64::from(year) - 2000.0) / 1000.0;
    let (y2, y3, y4) = (y * y, y * y * y, y * y * y * y);
    match event {
        SolarEvent::MarchEquinox => {
            2_451_623.809_84 + 365_242.374_04 * y + 0.051_69 * y2 - 0.004_11 * y3 - 0.000_57 * y4
        }
        SolarEvent::JuneSolstice => {
            2_451_716.567_67 + 365_241.626_03 * y + 0.003_25 * y2 + 0.008_88 * y3 - 0.000_30 * y4
        }
        SolarEvent::SeptemberEquinox => {
            2_451_810.217_15 + 365_242.017_67 * y - 0.115_75 * y2 + 0.003_37 * y3 + 0.000_78 * y4
        }
        SolarEvent::DecemberSolstice => {
            2_451_900.059_52 + 365_242.740_49 * y - 0.062_23 * y2 - 0.008_23 * y3 + 0.000_32 * y4
        }
    }
}

/// Instant (JDE, Dynamical Time) of an equinox or solstice in `year`.
///
/// `year` is clamped to [`MIN_YEAR`, `MAX_YEAR`].
pub fn solar_event_jde(year: i32, event: SolarEvent) -> f64 {
    let year = year.clamp(MIN_YEAR, MAX_YEAR);
    let jde0 = mean_event_jde(year, event);
    let t = (jde0 - 2_451_545.0) / 36_525.0;
    let w = (35_999.373 * t - 2.47).to_radians();
    let dl = 1.0 + 0.0334 * w.cos() + 0.0007 * (2.0 * w).cos();
    let s: f64 = TABLE_27C
        .iter()
        .map(|&(a, b, c)| a * (b + c * t).to_radians().cos())
        .sum();
    jde0 + 0.000_01 * s / dl
}

/// Apparent geocentric ecliptic longitude of the Sun in degrees, referred to
/// the true equinox of date (Meeus ch. 25, low accuracy, ≈0.01°).
pub fn apparent_longitude(jde: f64) -> f64 {
    let t = (jde - 2_451_545.0) / 36_525.0;
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    let m = (357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t).to_radians();
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    let true_longitude = l0 + c;
    let omega = (125.04 - 1934.136 * t).to_radians();
    normalize_degrees(true_longitude - 0.005_69 - 0.004_78 * omega.sin())
}

/// Instant (JDE) at which the Sun's apparent longitude reaches `target_deg`,
/// searching from `jde_guess` (which should be within a few weeks).
///
/// Uses the correction step from Meeus ch. 27: `+58·sin(target − λ)` days.
pub fn jde_of_solar_longitude(target_deg: f64, jde_guess: f64) -> f64 {
    let mut jde = jde_guess;
    for _ in 0..50 {
        let lambda = apparent_longitude(jde);
        let step = 58.0 * (target_deg - lambda).to_radians().sin();
        jde += step;
        if step.abs() < 1e-7 {
            break;
        }
    }
    jde
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astro::time::utc_from_jde;
    use chrono::{TimeZone, Utc};

    fn assert_utc_near(jde: f64, y: i32, mo: u32, d: u32, h: u32, mi: u32, tol_min: i64) {
        let got = utc_from_jde(jde);
        let want = Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap();
        let diff = (got - want).num_minutes().abs();
        assert!(diff <= tol_min, "got {got}, want {want} (Δ {diff} min)");
    }

    #[test]
    fn known_events_2000() {
        assert_utc_near(solar_event_jde(2000, SolarEvent::MarchEquinox), 2000, 3, 20, 7, 35, 2);
        assert_utc_near(solar_event_jde(2000, SolarEvent::JuneSolstice), 2000, 6, 21, 1, 48, 2);
        assert_utc_near(solar_event_jde(2000, SolarEvent::SeptemberEquinox), 2000, 9, 22, 17, 28, 2);
        assert_utc_near(solar_event_jde(2000, SolarEvent::DecemberSolstice), 2000, 12, 21, 13, 37, 2);
    }

    #[test]
    fn known_events_2024() {
        // Published values (USNO / IMCCE), UTC.
        assert_utc_near(solar_event_jde(2024, SolarEvent::MarchEquinox), 2024, 3, 20, 3, 6, 2);
        assert_utc_near(solar_event_jde(2024, SolarEvent::JuneSolstice), 2024, 6, 20, 20, 51, 2);
        assert_utc_near(solar_event_jde(2024, SolarEvent::SeptemberEquinox), 2024, 9, 22, 12, 44, 2);
        assert_utc_near(solar_event_jde(2024, SolarEvent::DecemberSolstice), 2024, 12, 21, 9, 21, 2);
    }

    #[test]
    fn known_events_2026() {
        assert_utc_near(solar_event_jde(2026, SolarEvent::MarchEquinox), 2026, 3, 20, 14, 46, 2);
        assert_utc_near(solar_event_jde(2026, SolarEvent::JuneSolstice), 2026, 6, 21, 8, 24, 2);
        assert_utc_near(solar_event_jde(2026, SolarEvent::SeptemberEquinox), 2026, 9, 23, 0, 5, 2);
        assert_utc_near(solar_event_jde(2026, SolarEvent::DecemberSolstice), 2026, 12, 21, 20, 50, 2);
    }

    #[test]
    fn longitude_solver_agrees_with_chapter_27() {
        // The low-accuracy longitude must reproduce the ch. 27 instants to
        // within its stated precision (~0.01° ≈ 15 minutes).
        for year in [1990, 2024, 2026, 2050] {
            for ev in SolarEvent::ALL {
                let reference = solar_event_jde(year, ev);
                let solved = jde_of_solar_longitude(ev.longitude(), reference + 5.0);
                let diff_min = (solved - reference).abs() * 1440.0;
                assert!(diff_min < 20.0, "{year} {ev:?}: {diff_min} min");
            }
        }
    }

    #[test]
    fn cross_quarter_order() {
        // Sun reaches 45° (northern Beltane point) in early May.
        let mar = solar_event_jde(2026, SolarEvent::MarchEquinox);
        let jun = solar_event_jde(2026, SolarEvent::JuneSolstice);
        let x = jde_of_solar_longitude(45.0, (mar + jun) / 2.0);
        assert!(x > mar && x < jun);
        let d = utc_from_jde(x);
        assert_eq!(d.format("%m").to_string(), "05");
        assert!((apparent_longitude(x) - 45.0).abs() < 1e-6);
    }
}
