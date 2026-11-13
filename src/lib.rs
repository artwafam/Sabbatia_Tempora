//! Sabbatia Tempora | core library.
//!
//! Layering (each layer depends only on those above it):
//!
//! * [`astro`]     | pure astronomical algorithms (Meeus), in Julian Days.
//! * [`calendar`]  | seasons, Wheel of the Year rules, lunar phase naming,
//!                   lore and the computed [`calendar::almanac::Almanac`].
//! * [`i18n`]      | interface language, names and date formatting.
//! * [`companion`] | the two characters and their mood, from the almanac.
//! * [`dialogue`]  | what they say, when, and how often.
//! * [`config`]    | user configuration (TOML) and desktop integration files.
//!
//! The GUI lives in the binary crate and only reads these.

pub mod astro;
pub mod calendar;
pub mod companion;
pub mod config;
pub mod dialogue;
pub mod i18n;
