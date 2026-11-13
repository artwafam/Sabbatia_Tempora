//! User configuration (`~/.config/sabbatia/config.toml`), window state and the
//! XDG autostart entry.
//!
//! The config file is written by hand-formatting so that it stays commented
//! and readable; it is only rewritten when the user changes a setting from
//! the widget.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::calendar::almanac::CalendarSettings;
use crate::calendar::{CrossQuarterRule, Hemisphere};
use crate::companion::Character;
use crate::i18n::LanguagePref;

pub const APP_ID: &str = "sabbatia";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// Unknown keys are ignored so that files written by older versions (which had
// `theme` and `decorations`) still load.
#[serde(default)]
pub struct Config {
    /// Explicit hemisphere. When absent, derived from `latitude`, else southern.
    pub hemisphere: Option<Hemisphere>,
    pub cross_quarters: CrossQuarterRule,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// IANA time zone name (e.g. "America/Sao_Paulo"). Empty: system zone.
    pub timezone: String,
    pub language: LanguagePref,
    /// Float above other windows instead of sitting on the desktop.
    pub always_on_top: bool,
    pub autostart: bool,
    /// The card figure on the desktop.
    pub character: Character,
    /// Height of the figure in logical pixels.
    pub figure_size: f32,
    /// Let the figure move now and then.
    pub animations: bool,
    /// Width of the almanac bubble in logical pixels.
    pub width: f32,
}

pub const WIDTH_RANGE: std::ops::RangeInclusive<f32> = 280.0..=360.0;
pub const FIGURE_RANGE: std::ops::RangeInclusive<f32> = 90.0..=280.0;

impl Default for Config {
    fn default() -> Self {
        Config {
            hemisphere: None,
            cross_quarters: CrossQuarterRule::Traditional,
            latitude: None,
            longitude: None,
            timezone: String::new(),
            language: LanguagePref::Auto,
            always_on_top: false,
            autostart: false,
            character: Character::Hermit,
            figure_size: 150.0,
            animations: true,
            width: 300.0,
        }
    }
}

impl Config {
    pub fn effective_hemisphere(&self) -> Hemisphere {
        match (self.hemisphere, self.latitude) {
            (Some(h), _) => h,
            (None, Some(lat)) if lat >= 0.0 => Hemisphere::Northern,
            _ => Hemisphere::Southern,
        }
    }

    pub fn calendar_settings(&self) -> CalendarSettings {
        CalendarSettings { hemisphere: self.effective_hemisphere(), cross_quarters: self.cross_quarters }
    }

    /// Check values that serde cannot.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(lat) = self.latitude
            && !(-90.0..=90.0).contains(&lat)
        {
            return Err(format!("latitude {lat} is outside −90…90"));
        }
        if let Some(lon) = self.longitude
            && !(-180.0..=180.0).contains(&lon)
        {
            return Err(format!("longitude {lon} is outside −180…180"));
        }
        if !WIDTH_RANGE.contains(&self.width) {
            return Err(format!("width {} is outside 280…360", self.width));
        }
        if !FIGURE_RANGE.contains(&self.figure_size) {
            return Err(format!("figure_size {} is outside 90…280", self.figure_size));
        }
        if !self.timezone.is_empty() && zoneinfo_path(&self.timezone).is_none() {
            return Err(format!("time zone \"{}\" not found in the system zoneinfo database", self.timezone));
        }
        Ok(())
    }

    pub fn parse(text: &str) -> Result<Config, String> {
        let cfg: Config = toml::from_str(text).map_err(|e| e.to_string())?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Commented TOML representation.
    pub fn to_toml(&self) -> String {
        fn opt_f(v: Option<f64>, key: &str, example: &str) -> String {
            match v {
                Some(x) => format!("{key} = {x}\n"),
                None => format!("# {key} = {example}\n"),
            }
        }
        let hemi = match self.hemisphere {
            Some(Hemisphere::Northern) => "hemisphere = \"northern\"\n".to_string(),
            Some(Hemisphere::Southern) => "hemisphere = \"southern\"\n".to_string(),
            None => "# hemisphere = \"southern\"\n".to_string(),
        };
        let cq = match self.cross_quarters {
            CrossQuarterRule::Traditional => "traditional",
            CrossQuarterRule::Astronomical => "astronomical",
        };
        format!(
            "# Sabbatia Tempora configuration\n\
             \n\
             # \"southern\" or \"northern\". If omitted, derived from latitude;\n\
             # if that is also omitted, the Southern Hemisphere is used.\n\
             {hemi}\
             \n\
             # Cross-quarter festivals (Samhain, Imbolc, Beltane, Lughnasadh):\n\
             # \"traditional\"  = fixed days (1 Feb / 1 May / 1 Aug / 1 Nov)\n\
             # \"astronomical\" = Sun at 45°, 135°, 225° or 315° ecliptic longitude\n\
             cross_quarters = \"{cq}\"\n\
             \n\
             # Optional location, in decimal degrees (south and west negative).\n\
             {lat}{lon}\
             \n\
             # IANA time zone, e.g. \"America/Sao_Paulo\". Empty uses the system zone.\n\
             timezone = \"{tz}\"\n\
             \n\
             # \"auto\" (follow the system), \"en\" or \"pt-BR\"\n\
             language = \"{lang}\"\n\
             \n\
             # The figure on your desktop: \"hermit\" or \"fool\" (with his dog),\n\
             # its height in pixels (90–280), and whether it moves now and then.\n\
             character = \"{who}\"\n\
             figure_size = {fig:.1}\n\
             animations = {anim}\n\
             \n\
             # false: sit on the desktop, beneath all windows. true: float above them.\n\
             always_on_top = {top}\n\
             # Start with the desktop session (writes ~/.config/autostart/{APP_ID}.desktop).\n\
             autostart = {auto}\n\
             # Width of the almanac bubble in pixels (280–360).\n\
             width = {width:.1}\n",
            lat = opt_f(self.latitude, "latitude", "-23.55"),
            lon = opt_f(self.longitude, "longitude", "-46.63"),
            tz = self.timezone.replace('\\', "\\\\").replace('"', "\\\""),
            lang = self.language.config_value(),
            top = self.always_on_top,
            auto = self.autostart,
            who = self.character.config_value(),
            fig = self.figure_size,
            anim = self.animations,
            width = self.width,
        )
    }
}

/// Persisted placement, kept apart from the user-edited config: the point
/// on screen where the figure stands (its bottom-left corner), so the speech
/// bubble can open above it without moving the figure.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WindowState {
    pub foot_x: Option<f32>,
    pub foot_y: Option<f32>,
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from)
}

fn xdg_dir(var: &str, fallback: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home().map(|h| h.join(fallback)))
}

pub fn config_dir() -> Option<PathBuf> {
    xdg_dir("XDG_CONFIG_HOME", ".config").map(|d| d.join(APP_ID))
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

fn state_path() -> Option<PathBuf> {
    xdg_dir("XDG_STATE_HOME", ".local/state").map(|d| d.join(APP_ID).join("window.toml"))
}

pub fn autostart_path() -> Option<PathBuf> {
    xdg_dir("XDG_CONFIG_HOME", ".config").map(|d| d.join("autostart").join(format!("{APP_ID}.desktop")))
}

/// Resolve an IANA zone name to a TZif file, rejecting path traversal.
pub fn zoneinfo_path(name: &str) -> Option<PathBuf> {
    if name.is_empty() || name.starts_with('/') || name.split('/').any(|c| c == ".." || c.is_empty()) {
        return None;
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(d) = std::env::var_os("TZDIR") {
        dirs.push(PathBuf::from(d));
    }
    dirs.push(PathBuf::from("/usr/share/zoneinfo"));
    dirs.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

/// Atomic write: temp file in the same directory, then rename.
fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

/// Load the config, creating a commented default file on first run.
/// Returns the config and an optional warning for the user.
pub fn load() -> (Config, Option<String>) {
    let Some(path) = config_path() else {
        return (Config::default(), Some("HOME is not set; using defaults".into()));
    };
    match fs::read_to_string(&path) {
        Ok(text) => match Config::parse(&text) {
            Ok(cfg) => (cfg, None),
            Err(e) => (Config::default(), Some(format!("{}: {e}; using defaults", path.display()))),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            let warn = write_atomic(&path, &cfg.to_toml())
                .err()
                .map(|e| format!("could not create {}: {e}", path.display()));
            (cfg, warn)
        }
        Err(e) => (Config::default(), Some(format!("{}: {e}; using defaults", path.display()))),
    }
}

pub fn save(cfg: &Config) -> io::Result<()> {
    let path = config_path().ok_or_else(|| io::Error::other("HOME is not set"))?;
    write_atomic(&path, &cfg.to_toml())
}

pub fn load_window_state() -> WindowState {
    state_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| toml::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_window_state(state: &WindowState) -> io::Result<()> {
    let path = state_path().ok_or_else(|| io::Error::other("HOME is not set"))?;
    let text = toml::to_string(state).map_err(io::Error::other)?;
    write_atomic(&path, &text)
}

/// Quote an Exec argument per the Desktop Entry Specification.
fn desktop_exec_quote(arg: &str) -> String {
    let needs_quotes = arg.chars().any(|c| " \t\n\"'\\><~|&;$*?#()`".contains(c));
    if !needs_quotes {
        return arg.replace('%', "%%");
    }
    let mut s = String::with_capacity(arg.len() + 2);
    s.push('"');
    for c in arg.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                s.push('\\');
                s.push(c);
            }
            '%' => s.push_str("%%"),
            _ => s.push(c),
        }
    }
    s.push('"');
    // Inside a desktop-file string value, backslashes are escaped once more.
    s.replace('\\', "\\\\")
}

pub fn autostart_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Sabbatia Tempora\n\
         Comment=Seasonal and lunar desktop companion\n\
         Exec={}\n\
         Icon={APP_ID}\n\
         Terminal=false\n\
         NoDisplay=true\n\
         X-GNOME-Autostart-enabled=true\n\
         X-GNOME-Autostart-Delay=3\n",
        desktop_exec_quote(&exe.to_string_lossy())
    )
}

/// Make the autostart entry match `enabled`. Idempotent.
pub fn sync_autostart(enabled: bool) -> io::Result<()> {
    let path = autostart_path().ok_or_else(|| io::Error::other("HOME is not set"))?;
    if enabled {
        let exe = std::env::current_exe()?;
        let entry = autostart_entry(&exe);
        if fs::read_to_string(&path).ok().as_deref() != Some(entry.as_str()) {
            write_atomic(&path, &entry)?;
        }
        Ok(())
    } else {
        match fs::remove_file(&path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrips_through_toml() {
        let c = Config::default();
        assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);
        let c = Config {
            hemisphere: Some(Hemisphere::Northern),
            cross_quarters: CrossQuarterRule::Astronomical,
            latitude: Some(51.5),
            longitude: Some(-0.12),
            timezone: String::new(),
            language: LanguagePref::PtBr,
            always_on_top: true,
            autostart: true,
            character: Character::Fool,
            figure_size: 200.0,
            animations: false,
            width: 320.0,
        };
        assert_eq!(Config::parse(&c.to_toml()).unwrap(), c);
    }

    #[test]
    fn defaults_to_southern() {
        assert_eq!(Config::default().effective_hemisphere(), Hemisphere::Southern);
        let c = Config::parse("").unwrap();
        assert_eq!(c.effective_hemisphere(), Hemisphere::Southern);
    }

    #[test]
    fn hemisphere_from_latitude_and_override() {
        let c = Config::parse("latitude = 52.0").unwrap();
        assert_eq!(c.effective_hemisphere(), Hemisphere::Northern);
        let c = Config::parse("latitude = 52.0\nhemisphere = \"southern\"").unwrap();
        assert_eq!(c.effective_hemisphere(), Hemisphere::Southern);
    }

    #[test]
    fn rejects_bad_values() {
        assert!(Config::parse("latitude = 123.0").is_err());
        assert!(Config::parse("hemisphere = \"eastern\"").is_err());
        assert!(Config::parse("width = 2000.0").is_err());
        assert!(Config::parse("figure_size = 10.0").is_err());
        assert!(Config::parse("character = \"magician\"").is_err());
        assert!(Config::parse("timezone = \"../../etc/passwd\"").is_err());
        assert!(Config::parse("timezone = \"Not/AZone\"").is_err());
    }

    #[test]
    fn old_config_files_still_load() {
        let c = Config::parse("theme = \"charcoal\"\ndecorations = true\ncompanion = true\nhemisphere = \"northern\"").unwrap();
        assert_eq!(c.effective_hemisphere(), Hemisphere::Northern);
        assert!(Config::parse("language = \"klingon\"").is_err());
    }

    #[test]
    fn exec_quoting() {
        assert_eq!(desktop_exec_quote("/usr/bin/sabbatia"), "/usr/bin/sabbatia");
        assert_eq!(desktop_exec_quote("/home/a b/sabbatia"), "\"/home/a b/sabbatia\"");
        assert_eq!(desktop_exec_quote("/x/$y"), "\"/x/\\\\$y\"");
    }
}
