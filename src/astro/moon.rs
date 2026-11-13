//! Lunar phases and illumination.
//!
//! * Instants of the principal phases (new, first quarter, full, last
//!   quarter): Meeus, *Astronomical Algorithms* (2nd ed.), chapter 49,
//!   including all periodic and planetary correction terms. Meeus quotes a
//!   mean error of a few seconds and a maximum below about half a minute for
//!   the period 1980–2020.
//! * Illuminated fraction: Meeus chapter 48, phase angle from equation 48.4
//!   with the Moon's mean elements from chapter 47. Accurate to well under
//!   one percent, which is the display resolution used here.

use super::time::normalize_degrees;

/// Mean synodic month in days (Meeus eq. 49.1 linear coefficient).
pub const SYNODIC_MONTH: f64 = 29.530_588_861;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrincipalPhase {
    New,
    FirstQuarter,
    Full,
    LastQuarter,
}

impl PrincipalPhase {
    pub const ALL: [PrincipalPhase; 4] = [
        PrincipalPhase::New,
        PrincipalPhase::FirstQuarter,
        PrincipalPhase::Full,
        PrincipalPhase::LastQuarter,
    ];

    fn k_offset(self) -> f64 {
        match self {
            PrincipalPhase::New => 0.0,
            PrincipalPhase::FirstQuarter => 0.25,
            PrincipalPhase::Full => 0.5,
            PrincipalPhase::LastQuarter => 0.75,
        }
    }
}

/// Instant (JDE) of a principal phase of lunation `lunation`.
///
/// Lunation 0 is the new moon of 2000 January 6 (Meeus' numbering, which
/// differs from the Brown lunation number by 953).
pub fn phase_jde(lunation: i64, phase: PrincipalPhase) -> f64 {
    let k = lunation as f64 + phase.k_offset();
    let t = k / 1236.85;
    let (t2, t3, t4) = (t * t, t * t * t, t * t * t * t);

    let mut jde = 2_451_550.097_66 + SYNODIC_MONTH * k + 0.000_154_37 * t2 - 0.000_000_150 * t3
        + 0.000_000_000_73 * t4;

    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;
    let e2 = e * e;
    let m = (2.5534 + 29.105_356_70 * k - 0.000_001_4 * t2 - 0.000_000_11 * t3).to_radians();
    let mp = (201.5643 + 385.816_935_28 * k + 0.010_758_2 * t2 + 0.000_012_38 * t3
        - 0.000_000_058 * t4)
        .to_radians();
    let f = (160.7108 + 390.670_502_84 * k - 0.001_611_8 * t2 - 0.000_002_27 * t3
        + 0.000_000_011 * t4)
        .to_radians();
    let om = (124.7746 - 1.563_755_88 * k + 0.002_067_2 * t2 + 0.000_002_15 * t3).to_radians();

    let sin = f64::sin;
    let correction = match phase {
        PrincipalPhase::New => {
            -0.407_20 * sin(mp) + 0.172_41 * e * sin(m) + 0.016_08 * sin(2.0 * mp)
                + 0.010_39 * sin(2.0 * f)
                + 0.007_39 * e * sin(mp - m)
                - 0.005_14 * e * sin(mp + m)
                + 0.002_08 * e2 * sin(2.0 * m)
                - 0.001_11 * sin(mp - 2.0 * f)
                - 0.000_57 * sin(mp + 2.0 * f)
                + 0.000_56 * e * sin(2.0 * mp + m)
                - 0.000_42 * sin(3.0 * mp)
                + 0.000_42 * e * sin(m + 2.0 * f)
                + 0.000_38 * e * sin(m - 2.0 * f)
                - 0.000_24 * e * sin(2.0 * mp - m)
                - 0.000_17 * sin(om)
                - 0.000_07 * sin(mp + 2.0 * m)
                + 0.000_04 * sin(2.0 * mp - 2.0 * f)
                + 0.000_04 * sin(3.0 * m)
                + 0.000_03 * sin(mp + m - 2.0 * f)
                + 0.000_03 * sin(2.0 * mp + 2.0 * f)
                - 0.000_03 * sin(mp + m + 2.0 * f)
                + 0.000_03 * sin(mp - m + 2.0 * f)
                - 0.000_02 * sin(mp - m - 2.0 * f)
                - 0.000_02 * sin(3.0 * mp + m)
                + 0.000_02 * sin(4.0 * mp)
        }
        PrincipalPhase::Full => {
            -0.406_14 * sin(mp) + 0.173_02 * e * sin(m) + 0.016_14 * sin(2.0 * mp)
                + 0.010_43 * sin(2.0 * f)
                + 0.007_34 * e * sin(mp - m)
                - 0.005_15 * e * sin(mp + m)
                + 0.002_09 * e2 * sin(2.0 * m)
                - 0.001_11 * sin(mp - 2.0 * f)
                - 0.000_57 * sin(mp + 2.0 * f)
                + 0.000_56 * e * sin(2.0 * mp + m)
                - 0.000_42 * sin(3.0 * mp)
                + 0.000_42 * e * sin(m + 2.0 * f)
                + 0.000_38 * e * sin(m - 2.0 * f)
                - 0.000_24 * e * sin(2.0 * mp - m)
                - 0.000_17 * sin(om)
                - 0.000_07 * sin(mp + 2.0 * m)
                + 0.000_04 * sin(2.0 * mp - 2.0 * f)
                + 0.000_04 * sin(3.0 * m)
                + 0.000_03 * sin(mp + m - 2.0 * f)
                + 0.000_03 * sin(2.0 * mp + 2.0 * f)
                - 0.000_03 * sin(mp + m + 2.0 * f)
                + 0.000_03 * sin(mp - m + 2.0 * f)
                - 0.000_02 * sin(mp - m - 2.0 * f)
                - 0.000_02 * sin(3.0 * mp + m)
                + 0.000_02 * sin(4.0 * mp)
        }
        PrincipalPhase::FirstQuarter | PrincipalPhase::LastQuarter => {
            let base = -0.628_01 * sin(mp) + 0.171_72 * e * sin(m)
                - 0.011_83 * e * sin(mp + m)
                + 0.008_62 * sin(2.0 * mp)
                + 0.008_04 * sin(2.0 * f)
                + 0.004_54 * e * sin(mp - m)
                + 0.002_04 * e2 * sin(2.0 * m)
                - 0.001_80 * sin(mp - 2.0 * f)
                - 0.000_70 * sin(mp + 2.0 * f)
                - 0.000_40 * sin(3.0 * mp)
                - 0.000_34 * e * sin(2.0 * mp - m)
                + 0.000_32 * e * sin(m + 2.0 * f)
                + 0.000_32 * e * sin(m - 2.0 * f)
                - 0.000_28 * e2 * sin(mp + 2.0 * m)
                + 0.000_27 * e * sin(2.0 * mp + m)
                - 0.000_17 * sin(om)
                - 0.000_05 * sin(mp - m - 2.0 * f)
                + 0.000_04 * sin(2.0 * mp + 2.0 * f)
                - 0.000_04 * sin(mp + m + 2.0 * f)
                + 0.000_04 * sin(mp - 2.0 * m)
                + 0.000_03 * sin(mp + m - 2.0 * f)
                + 0.000_03 * sin(3.0 * m)
                + 0.000_02 * sin(2.0 * mp - 2.0 * f)
                + 0.000_02 * sin(mp - m + 2.0 * f)
                - 0.000_02 * sin(3.0 * mp + m);
            let w = 0.003_06 - 0.000_38 * e * m.cos() + 0.000_26 * mp.cos()
                - 0.000_02 * (mp - m).cos()
                + 0.000_02 * (mp + m).cos()
                + 0.000_02 * (2.0 * f).cos();
            if phase == PrincipalPhase::FirstQuarter { base + w } else { base - w }
        }
    };
    jde += correction;

    // Additional corrections for all phases (planetary arguments).
    const A: [(f64, f64, f64); 14] = [
        (0.000_325, 299.77, 0.107_408),
        (0.000_165, 251.88, 0.016_321),
        (0.000_164, 251.83, 26.651_886),
        (0.000_126, 349.42, 36.412_478),
        (0.000_110, 84.66, 18.206_239),
        (0.000_062, 141.74, 53.303_771),
        (0.000_060, 207.14, 2.453_732),
        (0.000_056, 154.84, 7.306_860),
        (0.000_047, 34.52, 27.261_239),
        (0.000_042, 207.19, 0.121_824),
        (0.000_040, 291.34, 1.844_379),
        (0.000_037, 161.72, 24.198_154),
        (0.000_035, 239.56, 25.513_099),
        (0.000_023, 331.55, 3.592_518),
    ];
    for (i, &(coef, a0, a1)) in A.iter().enumerate() {
        let mut arg = a0 + a1 * k;
        if i == 0 {
            arg -= 0.009_173 * t2;
        }
        jde += coef * arg.to_radians().sin();
    }
    jde
}

/// Lunation number of the last new moon at or before `jde`.
pub fn lunation_at(jde: f64) -> i64 {
    let mut k = ((jde - 2_451_550.097_66) / SYNODIC_MONTH).floor() as i64;
    // The true new moon deviates from the mean by at most ~14 h, so one step
    // in either direction is enough; loop for robustness.
    while phase_jde(k, PrincipalPhase::New) > jde {
        k -= 1;
    }
    while phase_jde(k + 1, PrincipalPhase::New) <= jde {
        k += 1;
    }
    k
}

/// Phase angle of the Moon (degrees, 0 = full, 180 = new), Meeus eq. 48.4.
pub fn phase_angle(jde: f64) -> f64 {
    let t = (jde - 2_451_545.0) / 36_525.0;
    let (t2, t3, t4) = (t * t, t * t * t, t * t * t * t);
    let d = normalize_degrees(
        297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t2 + t3 / 545_868.0
            - t4 / 113_065_000.0,
    );
    let m = normalize_degrees(357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t2 + t3 / 24_490_000.0);
    let mp = normalize_degrees(
        134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t2 + t3 / 69_699.0
            - t4 / 14_712_000.0,
    );
    let (dr, mr, mpr) = (d.to_radians(), m.to_radians(), mp.to_radians());
    180.0 - d - 6.289 * mpr.sin() + 2.100 * mr.sin()
        - 1.274 * (2.0 * dr - mpr).sin()
        - 0.658 * (2.0 * dr).sin()
        - 0.214 * (2.0 * mpr).sin()
        - 0.110 * dr.sin()
}

/// Illuminated fraction of the Moon's disk, 0.0 … 1.0 (Meeus eq. 48.1).
pub fn illuminated_fraction(jde: f64) -> f64 {
    (1.0 + phase_angle(jde).to_radians().cos()) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astro::time::{jde_from_utc, utc_from_jde};
    use chrono::{TimeZone, Utc};

    #[test]
    fn meeus_example_49a() {
        // New moon of 1977 February: k = −283, JDE = 2443192.65118.
        let jde = phase_jde(-283, PrincipalPhase::New);
        assert!((jde - 2_443_192.651_18).abs() < 1e-5, "{jde}");
    }

    #[test]
    fn meeus_example_49b() {
        // First last quarter of 2044: k = 544.75, JDE = 2467636.49186.
        let jde = phase_jde(544, PrincipalPhase::LastQuarter);
        assert!((jde - 2_467_636.491_86).abs() < 1e-5, "{jde}");
    }

    fn assert_phase_near(lunation: i64, ph: PrincipalPhase, y: i32, mo: u32, d: u32, h: u32, mi: u32) {
        let got = utc_from_jde(phase_jde(lunation, ph));
        let want = Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap();
        let diff = (got - want).num_minutes().abs();
        assert!(diff <= 2, "{ph:?}: got {got}, want {want}");
    }

    #[test]
    fn known_phases() {
        // Total solar eclipse new moon, 2024-04-08 18:21 UTC.
        let k = lunation_at(jde_from_utc(Utc.with_ymd_and_hms(2024, 4, 9, 0, 0, 0).unwrap()));
        assert_phase_near(k, PrincipalPhase::New, 2024, 4, 8, 18, 21);
        // Total lunar eclipse full moon, 2025-09-07 18:09 UTC.
        let k = lunation_at(jde_from_utc(Utc.with_ymd_and_hms(2025, 9, 1, 0, 0, 0).unwrap()));
        assert_phase_near(k, PrincipalPhase::Full, 2025, 9, 7, 18, 9);
        // Full moon 2024-01-25 17:54 UTC.
        let k = lunation_at(jde_from_utc(Utc.with_ymd_and_hms(2024, 1, 20, 0, 0, 0).unwrap()));
        assert_phase_near(k, PrincipalPhase::Full, 2024, 1, 25, 17, 54);
    }

    #[test]
    fn lunation_brackets_instant() {
        for days in (0..800).step_by(7) {
            let jde = 2_460_000.0 + f64::from(days) + 0.37;
            let k = lunation_at(jde);
            assert!(phase_jde(k, PrincipalPhase::New) <= jde);
            assert!(phase_jde(k + 1, PrincipalPhase::New) > jde);
        }
    }

    #[test]
    fn illumination_extremes() {
        let k = 300;
        let new = phase_jde(k, PrincipalPhase::New);
        let full = phase_jde(k, PrincipalPhase::Full);
        let fq = phase_jde(k, PrincipalPhase::FirstQuarter);
        assert!(illuminated_fraction(new) < 0.01);
        assert!(illuminated_fraction(full) > 0.99);
        assert!((illuminated_fraction(fq) - 0.5).abs() < 0.03);
    }
}
