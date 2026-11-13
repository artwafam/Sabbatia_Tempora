//! Seasons, the Wheel of the Year and lunar phase naming.
//!
//! This layer turns astronomical instants into calendar facts for a chosen
//! hemisphere and time zone. It never draws anything and never reads
//! configuration files.

pub mod almanac;
pub mod lore;

use serde::{Deserialize, Serialize};

use crate::astro::sun::SolarEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Hemisphere {
    Northern,
    #[default]
    Southern,
}

impl Hemisphere {
    pub fn name(self) -> &'static str {
        match self {
            Hemisphere::Northern => "Northern Hemisphere",
            Hemisphere::Southern => "Southern Hemisphere",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Hemisphere::Northern => "North",
            Hemisphere::Southern => "South",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Hemisphere::Northern => Hemisphere::Southern,
            Hemisphere::Southern => Hemisphere::Northern,
        }
    }

    /// Season that begins at a solar event in this hemisphere.
    pub fn season_beginning_at(self, ev: SolarEvent) -> Season {
        use SolarEvent::*;
        match (self, ev) {
            (Hemisphere::Northern, MarchEquinox) | (Hemisphere::Southern, SeptemberEquinox) => {
                Season::Spring
            }
            (Hemisphere::Northern, JuneSolstice) | (Hemisphere::Southern, DecemberSolstice) => {
                Season::Summer
            }
            (Hemisphere::Northern, SeptemberEquinox) | (Hemisphere::Southern, MarchEquinox) => {
                Season::Autumn
            }
            (Hemisphere::Northern, DecemberSolstice) | (Hemisphere::Southern, JuneSolstice) => {
                Season::Winter
            }
        }
    }
}

/// Astronomical seasons, bounded by equinoxes and solstices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

/// How the four Gaelic cross-quarter festivals are dated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CrossQuarterRule {
    /// Fixed calendar days (the 1st of Feb/May/Aug/Nov, celebrated from the
    /// eve before). This is the historical and most common usage.
    #[default]
    Traditional,
    /// The instant the Sun reaches the ecliptic longitude midway between a
    /// solstice and an equinox (45°, 135°, 225°, 315°).
    Astronomical,
}

/// The eight festivals of the modern Pagan Wheel of the Year, in wheel order
/// starting from Samhain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sabbat {
    Samhain,
    Yule,
    Imbolc,
    Ostara,
    Beltane,
    Litha,
    Lughnasadh,
    Mabon,
}

/// How a particular Sabbat's date is determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateBasis {
    /// Fixed by a solstice or equinox.
    SolarEvent(SolarEvent),
    /// Midpoint of the solar longitude between a solstice and an equinox.
    CrossQuarterAstronomical { longitude: u16 },
    /// Fixed traditional calendar day (festival day, with its eve the day before).
    CrossQuarterTraditional { month: u32, day: u32 },
}

impl DateBasis {
    pub fn is_astronomical(self) -> bool {
        !matches!(self, DateBasis::CrossQuarterTraditional { .. })
    }
}

impl Sabbat {
    pub const ALL: [Sabbat; 8] = [
        Sabbat::Samhain,
        Sabbat::Yule,
        Sabbat::Imbolc,
        Sabbat::Ostara,
        Sabbat::Beltane,
        Sabbat::Litha,
        Sabbat::Lughnasadh,
        Sabbat::Mabon,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Sabbat::Samhain => "Samhain",
            Sabbat::Yule => "Yule",
            Sabbat::Imbolc => "Imbolc",
            Sabbat::Ostara => "Ostara",
            Sabbat::Beltane => "Beltane",
            Sabbat::Litha => "Litha",
            Sabbat::Lughnasadh => "Lughnasadh",
            Sabbat::Mabon => "Mabon",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn next(self) -> Sabbat {
        Sabbat::ALL[(self.index() + 1) % 8]
    }

    pub fn previous(self) -> Sabbat {
        Sabbat::ALL[(self.index() + 7) % 8]
    }

    /// Whether this is one of the four Gaelic cross-quarter festivals.
    pub fn is_cross_quarter(self) -> bool {
        matches!(self, Sabbat::Samhain | Sabbat::Imbolc | Sabbat::Beltane | Sabbat::Lughnasadh)
    }

    /// Apparent solar longitude (degrees) that marks this Sabbat in the given
    /// hemisphere. Southern conventions are offset by 180°.
    pub fn solar_longitude(self, hemisphere: Hemisphere) -> u16 {
        // Northern: Samhain 225°, Yule 270°, Imbolc 315°, Ostara 0°, …
        let north = (225 + 45 * self.index() as u16) % 360;
        match hemisphere {
            Hemisphere::Northern => north,
            Hemisphere::Southern => (north + 180) % 360,
        }
    }

    pub fn date_basis(self, hemisphere: Hemisphere, rule: CrossQuarterRule) -> DateBasis {
        let lon = self.solar_longitude(hemisphere);
        if !self.is_cross_quarter() {
            let ev = match lon {
                0 => SolarEvent::MarchEquinox,
                90 => SolarEvent::JuneSolstice,
                180 => SolarEvent::SeptemberEquinox,
                _ => SolarEvent::DecemberSolstice,
            };
            return DateBasis::SolarEvent(ev);
        }
        match rule {
            CrossQuarterRule::Astronomical => DateBasis::CrossQuarterAstronomical { longitude: lon },
            CrossQuarterRule::Traditional => {
                // Festival day on the 1st of the month nearest the solar point.
                let month = match lon {
                    45 => 5,  // May
                    135 => 8, // August
                    225 => 11, // November
                    _ => 2,   // 315°: February
                };
                DateBasis::CrossQuarterTraditional { month, day: 1 }
            }
        }
    }
}

/// Lunar phase as conventionally named, one of eight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    pub fn name(self) -> &'static str {
        match self {
            MoonPhase::New => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::Full => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }

    pub fn is_waxing(self) -> bool {
        matches!(
            self,
            MoonPhase::WaxingCrescent | MoonPhase::FirstQuarter | MoonPhase::WaxingGibbous
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn southern_mapping() {
        let s = Hemisphere::Southern;
        let r = CrossQuarterRule::Traditional;
        assert_eq!(Sabbat::Yule.date_basis(s, r), DateBasis::SolarEvent(SolarEvent::JuneSolstice));
        assert_eq!(Sabbat::Litha.date_basis(s, r), DateBasis::SolarEvent(SolarEvent::DecemberSolstice));
        assert_eq!(Sabbat::Ostara.date_basis(s, r), DateBasis::SolarEvent(SolarEvent::SeptemberEquinox));
        assert_eq!(Sabbat::Mabon.date_basis(s, r), DateBasis::SolarEvent(SolarEvent::MarchEquinox));
        assert_eq!(
            Sabbat::Samhain.date_basis(s, r),
            DateBasis::CrossQuarterTraditional { month: 5, day: 1 }
        );
        assert_eq!(
            Sabbat::Beltane.date_basis(s, r),
            DateBasis::CrossQuarterTraditional { month: 11, day: 1 }
        );
        assert_eq!(
            Sabbat::Imbolc.date_basis(s, CrossQuarterRule::Astronomical),
            DateBasis::CrossQuarterAstronomical { longitude: 135 }
        );
    }

    #[test]
    fn northern_mapping() {
        let n = Hemisphere::Northern;
        let r = CrossQuarterRule::Traditional;
        assert_eq!(Sabbat::Yule.date_basis(n, r), DateBasis::SolarEvent(SolarEvent::DecemberSolstice));
        assert_eq!(Sabbat::Ostara.date_basis(n, r), DateBasis::SolarEvent(SolarEvent::MarchEquinox));
        assert_eq!(
            Sabbat::Samhain.date_basis(n, r),
            DateBasis::CrossQuarterTraditional { month: 11, day: 1 }
        );
        assert_eq!(
            Sabbat::Imbolc.date_basis(n, r),
            DateBasis::CrossQuarterTraditional { month: 2, day: 1 }
        );
    }

    #[test]
    fn seasons_flip_between_hemispheres() {
        for ev in SolarEvent::ALL {
            let n = Hemisphere::Northern.season_beginning_at(ev);
            let s = Hemisphere::Southern.season_beginning_at(ev);
            assert_ne!(n, s);
        }
        assert_eq!(Hemisphere::Southern.season_beginning_at(SolarEvent::DecemberSolstice), Season::Summer);
    }

    #[test]
    fn wheel_order_wraps() {
        assert_eq!(Sabbat::Mabon.next(), Sabbat::Samhain);
        assert_eq!(Sabbat::Samhain.previous(), Sabbat::Mabon);
    }
}
