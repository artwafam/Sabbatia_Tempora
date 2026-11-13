//! Time scales: Julian Day (UT), Julian Ephemeris Day (TT) and ΔT.
//!
//! The astronomical series in this crate (Meeus, *Astronomical Algorithms*,
//! 2nd ed., 1998) are expressed in Dynamical Time (TT/TD). Civil clocks follow
//! UTC, which tracks Earth's rotation (UT). The difference ΔT = TT − UT is
//! about 69–75 s in the 2020s and is applied when converting results back to
//! civil time.

use chrono::{DateTime, TimeZone, Utc};

/// Julian Day of the Unix epoch, 1970-01-01T00:00:00Z.
const JD_UNIX_EPOCH: f64 = 2_440_587.5;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Julian Day (UT) for a UTC instant.
pub fn jd_from_utc(t: DateTime<Utc>) -> f64 {
    let secs = t.timestamp() as f64 + f64::from(t.timestamp_subsec_nanos()) * 1e-9;
    JD_UNIX_EPOCH + secs / SECONDS_PER_DAY
}

/// UTC instant for a Julian Day (UT), rounded to the nearest second.
pub fn utc_from_jd(jd: f64) -> DateTime<Utc> {
    let secs = ((jd - JD_UNIX_EPOCH) * SECONDS_PER_DAY).round() as i64;
    Utc.timestamp_opt(secs, 0)
        .single()
        .expect("Julian Day within chrono's supported range")
}

/// Julian Ephemeris Day (TT) for a UTC instant.
pub fn jde_from_utc(t: DateTime<Utc>) -> f64 {
    let jd = jd_from_utc(t);
    jd + delta_t_seconds(decimal_year_from_jd(jd)) / SECONDS_PER_DAY
}

/// UTC instant for a Julian Ephemeris Day (TT).
pub fn utc_from_jde(jde: f64) -> DateTime<Utc> {
    utc_from_jd(jde - delta_t_seconds(decimal_year_from_jd(jde)) / SECONDS_PER_DAY)
}

/// Approximate decimal year (precision far exceeds what ΔT needs).
pub fn decimal_year_from_jd(jd: f64) -> f64 {
    2000.0 + (jd - 2_451_545.0) / 365.25
}

/// ΔT = TT − UT in seconds.
///
/// Polynomial expressions by Espenak & Meeus, *Five Millennium Canon of Solar
/// Eclipses* (NASA/TP–2006–214141), as published at
/// <https://eclipse.gsfc.nasa.gov/SEhelp/deltatpoly2004.html>.
/// Accuracy is a few seconds for 1900–2025; beyond that ΔT is extrapolated and
/// may drift by tens of seconds, which is far below the one-minute display
/// resolution of this application.
pub fn delta_t_seconds(year: f64) -> f64 {
    let y = year;
    if y < 1900.0 {
        // Long-term parabola (Morrison & Stephenson 2004). Out of the app's
        // practical range but kept continuous and finite.
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + 1.494_119 * t - 0.059_893_9 * t.powi(2) + 0.006_196_6 * t.powi(3)
            - 0.000_197 * t.powi(4)
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + 0.844_93 * t - 0.076_100 * t.powi(2) + 0.002_093_6 * t.powi(3)
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t.powi(2) / 233.0 + t.powi(3) / 2547.0
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t.powi(2) / 260.0 - t.powi(3) / 718.0
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060_374 * t.powi(2)
            + 0.001_727_5 * t.powi(3)
            + 0.000_651_814 * t.powi(4)
            + 0.000_023_735_99 * t.powi(5)
    } else if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + 0.322_17 * t + 0.005_589 * t.powi(2)
    } else if y < 2150.0 {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Reduce an angle in degrees to the range [0, 360).
#[inline]
pub fn normalize_degrees(a: f64) -> f64 {
    let r = a % 360.0;
    if r < 0.0 { r + 360.0 } else { r }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn jd_of_j2000() {
        // J2000.0 = 2000-01-01 12:00 TT; as a UT label the JD is exactly 2451545.0.
        let t = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        assert!((jd_from_utc(t) - 2_451_545.0).abs() < 1e-9);
    }

    #[test]
    fn jd_meeus_example_7a() {
        // Meeus example 7.a: 1957 Oct 4.81 → JD 2436116.31
        let t = Utc.with_ymd_and_hms(1957, 10, 4, 19, 26, 24).unwrap();
        assert!((jd_from_utc(t) - 2_436_116.31).abs() < 1e-5);
    }

    #[test]
    fn jd_roundtrip() {
        let t = Utc.with_ymd_and_hms(2026, 11, 12, 21, 8, 37).unwrap();
        assert_eq!(utc_from_jd(jd_from_utc(t)), t);
        assert_eq!(utc_from_jde(jde_from_utc(t)), t);
    }

    #[test]
    fn delta_t_is_plausible() {
        // Observed values: 2000 ≈ 63.8 s, 2020 ≈ 69.4 s.
        assert!((delta_t_seconds(2000.0) - 63.86).abs() < 0.5);
        assert!((delta_t_seconds(2020.0) - 69.4).abs() < 4.0);
        // Continuity at the 2005 and 2050 seams (within a second).
        assert!((delta_t_seconds(2004.9999) - delta_t_seconds(2005.0)).abs() < 1.0);
        assert!((delta_t_seconds(2049.9999) - delta_t_seconds(2050.0)).abs() < 1.0);
    }

    #[test]
    fn normalize() {
        assert_eq!(normalize_degrees(-30.0), 330.0);
        assert_eq!(normalize_degrees(720.5), 0.5);
    }
}
