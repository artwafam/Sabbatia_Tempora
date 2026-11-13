//! Interface language: detection, UI strings, names and date formatting.
//!
//! Dates are formatted here rather than with chrono's `%A`/`%B`, which only
//! produce English without the `unstable-locales` feature and its tables.

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike, Weekday};
use serde::{Deserialize, Serialize};

use crate::astro::moon::PrincipalPhase;
use crate::astro::sun::SolarEvent;
use crate::calendar::{Hemisphere, MoonPhase, Season};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    PtBr,
}

/// The configured preference; `Auto` follows the session locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LanguagePref {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "pt-BR")]
    PtBr,
}

impl LanguagePref {
    pub fn resolve(self) -> Lang {
        match self {
            LanguagePref::En => Lang::En,
            LanguagePref::PtBr => Lang::PtBr,
            LanguagePref::Auto => detect(|k| std::env::var(k).ok()),
        }
    }

    pub fn config_value(self) -> &'static str {
        match self {
            LanguagePref::Auto => "auto",
            LanguagePref::En => "en",
            LanguagePref::PtBr => "pt-BR",
        }
    }
}

/// Pick a language from POSIX locale variables, in their precedence order.
pub fn detect(var: impl Fn(&str) -> Option<String>) -> Lang {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(v) = var(key).filter(|v| !v.is_empty()) {
            return if v.to_ascii_lowercase().starts_with("pt") { Lang::PtBr } else { Lang::En };
        }
    }
    Lang::En
}

/// Fixed interface strings.
pub struct Ui {
    pub season: &'static str,
    pub moon: &'static str,
    pub wheel: &'static str,
    pub next_sabbat: &'static str,
    pub today: &'static str,
    pub tomorrow: &'static str,
    pub waxing: &'static str,
    pub waning: &'static str,
    pub began: &'static str,
    pub ends: &'static str,
    pub progress: &'static str,
    pub illumination: &'static str,
    pub lunar_age: &'static str,
    pub new_moon_on: &'static str,
    pub since: &'static str,
    pub until: &'static str,
    pub sun: &'static str,
    pub date: &'static str,
    pub dating: &'static str,
    pub alternative: &'static str,
    pub history: &'static str,
    pub significance: &'static str,
    pub symbolism: &'static str,
    pub sources: &'static str,
    pub settings: &'static str,
    pub settings_menu: &'static str,
    pub done: &'static str,
    pub hemisphere: &'static str,
    pub southern: &'static str,
    pub northern: &'static str,
    pub cross_quarters: &'static str,
    pub traditional: &'static str,
    pub astronomical: &'static str,
    pub language: &'static str,
    pub automatic: &'static str,
    pub animations: &'static str,
    pub keep_on_top: &'static str,
    pub autostart: &'static str,
    pub quit: &'static str,
    pub quit_app: &'static str,
    pub talk: &'static str,
    pub almanac: &'static str,
    pub close: &'static str,
    pub companion: &'static str,
    pub switch_character: &'static str,
    pub on_desktop_note: &'static str,
    pub switch_hemisphere: &'static str,
    pub config_file: &'static str,
    pub time_zone: &'static str,
    pub system_zone: &'static str,
}

static UI_EN: Ui = Ui {
    season: "SEASON",
    moon: "MOON",
    wheel: "WHEEL",
    next_sabbat: "NEXT SABBAT",
    today: "Today",
    tomorrow: "Tomorrow",
    waxing: "waxing",
    waning: "waning",
    began: "Began",
    ends: "Ends",
    progress: "Progress",
    illumination: "Illumination",
    lunar_age: "Lunar age",
    new_moon_on: "Last new moon",
    since: "Since",
    until: "Until",
    sun: "Sun",
    date: "Date",
    dating: "Dating",
    alternative: "Alternative",
    history: "History",
    significance: "Significance",
    symbolism: "Symbolism",
    sources: "Sources",
    settings: "Settings",
    settings_menu: "Settings…",
    done: "Done",
    hemisphere: "HEMISPHERE",
    southern: "Southern",
    northern: "Northern",
    cross_quarters: "CROSS-QUARTER DATES",
    traditional: "Traditional",
    astronomical: "Astronomical",
    language: "LANGUAGE",
    automatic: "Automatic",
    animations: "Animations",
    keep_on_top: "Keep above other windows",
    autostart: "Start when I log in",
    quit: "Quit",
    quit_app: "Quit Sabbatia",
    talk: "Talk",
    almanac: "Almanac",
    close: "Close",
    companion: "COMPANION",
    switch_character: "Switch companion",
    on_desktop_note: "Off: the figure sits on the desktop, beneath your windows.",
    switch_hemisphere: "Switch hemisphere",
    config_file: "Location and time zone",
    time_zone: "Time zone",
    system_zone: "system",
};

static UI_PT: Ui = Ui {
    season: "ESTAÇÃO",
    moon: "LUA",
    wheel: "RODA",
    next_sabbat: "PRÓXIMO SABÁ",
    today: "Hoje",
    tomorrow: "Amanhã",
    waxing: "crescendo",
    waning: "minguando",
    began: "Começou",
    ends: "Termina",
    progress: "Andamento",
    illumination: "Iluminação",
    lunar_age: "Idade da lua",
    new_moon_on: "Última lua nova",
    since: "Desde",
    until: "Até",
    sun: "Sol",
    date: "Data",
    dating: "Datação",
    alternative: "Alternativa",
    history: "História",
    significance: "Significado",
    symbolism: "Simbolismo",
    sources: "Fontes",
    settings: "Configurações",
    settings_menu: "Configurações…",
    done: "Concluir",
    hemisphere: "HEMISFÉRIO",
    southern: "Sul",
    northern: "Norte",
    cross_quarters: "FESTIVAIS INTERMEDIÁRIOS",
    traditional: "Tradicional",
    astronomical: "Astronômica",
    language: "IDIOMA",
    automatic: "Automático",
    animations: "Animações",
    keep_on_top: "Manter acima das outras janelas",
    autostart: "Iniciar ao entrar na sessão",
    quit: "Sair",
    quit_app: "Sair do Sabbatia",
    talk: "Conversar",
    almanac: "Almanaque",
    close: "Fechar",
    companion: "COMPANHEIRO",
    switch_character: "Trocar companheiro",
    on_desktop_note: "Desligado: a figura fica na área de trabalho, sob as janelas.",
    switch_hemisphere: "Trocar hemisfério",
    config_file: "Localização e fuso horário",
    time_zone: "Fuso horário",
    system_zone: "do sistema",
};

const WEEKDAYS_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const WEEKDAYS_PT: [&str; 7] =
    ["segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado", "domingo"];
const WD_SHORT_EN: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const WD_SHORT_PT: [&str; 7] = ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"];
const MONTHS_EN: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];
const MONTHS_PT: [&str; 12] = [
    "janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro",
    "dezembro",
];
const MON_SHORT_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MON_SHORT_PT: [&str; 12] = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];

fn wd(w: Weekday) -> usize {
    w.num_days_from_monday() as usize
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

impl Lang {
    pub fn ui(self) -> &'static Ui {
        match self {
            Lang::En => &UI_EN,
            Lang::PtBr => &UI_PT,
        }
    }

    pub fn season(self, s: Season) -> &'static str {
        match self {
            Lang::En => s.name(),
            Lang::PtBr => match s {
                Season::Spring => "Primavera",
                Season::Summer => "Verão",
                Season::Autumn => "Outono",
                Season::Winter => "Inverno",
            },
        }
    }

    pub fn hemisphere(self, h: Hemisphere) -> &'static str {
        match (self, h) {
            (Lang::En, _) => h.name(),
            (Lang::PtBr, Hemisphere::Southern) => "Hemisfério Sul",
            (Lang::PtBr, Hemisphere::Northern) => "Hemisfério Norte",
        }
    }

    pub fn moon_phase(self, p: MoonPhase) -> &'static str {
        match self {
            Lang::En => match p {
                MoonPhase::New => "New moon",
                MoonPhase::WaxingCrescent => "Waxing crescent",
                MoonPhase::FirstQuarter => "First quarter",
                MoonPhase::WaxingGibbous => "Waxing gibbous",
                MoonPhase::Full => "Full moon",
                MoonPhase::WaningGibbous => "Waning gibbous",
                MoonPhase::LastQuarter => "Last quarter",
                MoonPhase::WaningCrescent => "Waning crescent",
            },
            Lang::PtBr => match p {
                MoonPhase::New => "Lua nova",
                MoonPhase::WaxingCrescent => "Lua crescente",
                MoonPhase::FirstQuarter => "Quarto crescente",
                MoonPhase::WaxingGibbous => "Crescente gibosa",
                MoonPhase::Full => "Lua cheia",
                MoonPhase::WaningGibbous => "Minguante gibosa",
                MoonPhase::LastQuarter => "Quarto minguante",
                MoonPhase::WaningCrescent => "Lua minguante",
            },
        }
    }

    pub fn principal_phase(self, p: PrincipalPhase) -> &'static str {
        self.moon_phase(match p {
            PrincipalPhase::New => MoonPhase::New,
            PrincipalPhase::FirstQuarter => MoonPhase::FirstQuarter,
            PrincipalPhase::Full => MoonPhase::Full,
            PrincipalPhase::LastQuarter => MoonPhase::LastQuarter,
        })
    }

    pub fn solar_event(self, e: SolarEvent) -> &'static str {
        match self {
            Lang::En => e.name(),
            Lang::PtBr => match e {
                SolarEvent::MarchEquinox => "equinócio de março",
                SolarEvent::JuneSolstice => "solstício de junho",
                SolarEvent::SeptemberEquinox => "equinócio de setembro",
                SolarEvent::DecemberSolstice => "solstício de dezembro",
            },
        }
    }

    /// Short kind of a solar event: "solstice" / "solstício".
    pub fn solar_event_kind(self, e: SolarEvent) -> &'static str {
        let equinox = matches!(e, SolarEvent::MarchEquinox | SolarEvent::SeptemberEquinox);
        match (self, equinox) {
            (Lang::En, true) => "equinox",
            (Lang::En, false) => "solstice",
            (Lang::PtBr, true) => "equinócio",
            (Lang::PtBr, false) => "solstício",
        }
    }

    /// "1 day" / "39 days", "1 dia" / "39 dias".
    pub fn days(self, n: i64) -> String {
        match (self, n.abs() == 1) {
            (Lang::En, true) => format!("{n} day"),
            (Lang::En, false) => format!("{n} days"),
            (Lang::PtBr, true) => format!("{n} dia"),
            (Lang::PtBr, false) => format!("{n} dias"),
        }
    }

    /// Fixed-point number with the language's decimal separator.
    pub fn decimal(self, x: f64, places: usize) -> String {
        let s = format!("{x:.places$}");
        match self {
            Lang::En => s,
            Lang::PtBr => s.replace('.', ","),
        }
    }

    /// "Thursday, 12 November" / "Quinta-feira, 12 de novembro".
    pub fn date_long(self, d: NaiveDate) -> String {
        let (w, m) = (wd(d.weekday()), d.month0() as usize);
        match self {
            Lang::En => format!("{}, {} {}", WEEKDAYS_EN[w], d.day(), MONTHS_EN[m]),
            Lang::PtBr => format!("{}, {} de {}", capitalize(WEEKDAYS_PT[w]), d.day(), MONTHS_PT[m]),
        }
    }

    /// "22 Sep" / "22 set".
    pub fn day_month(self, d: NaiveDate) -> String {
        let m = d.month0() as usize;
        match self {
            Lang::En => format!("{} {}", d.day(), MON_SHORT_EN[m]),
            Lang::PtBr => format!("{} {}", d.day(), MON_SHORT_PT[m]),
        }
    }

    /// "Mon 21 Dec" / "seg 21 dez".
    pub fn date_short(self, d: NaiveDate) -> String {
        let w = wd(d.weekday());
        match self {
            Lang::En => format!("{} {}", WD_SHORT_EN[w], self.day_month(d)),
            Lang::PtBr => format!("{} {}", WD_SHORT_PT[w], self.day_month(d)),
        }
    }

    /// "Mon 21 Dec 2026" / "seg 21 dez 2026".
    pub fn date_short_year(self, d: NaiveDate) -> String {
        format!("{} {}", self.date_short(d), d.year())
    }

    /// "Tue 17 Nov, 08:47" / "ter 17 nov, 08:47".
    pub fn date_time(self, t: DateTime<FixedOffset>) -> String {
        format!("{}, {}", self.date_short(t.date_naive()), self.time(t))
    }

    pub fn time(self, t: DateTime<FixedOffset>) -> String {
        format!("{:02}:{:02}", t.hour(), t.minute())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| pairs.iter().find(|(kk, _)| *kk == k).map(|(_, v)| v.to_string())
    }

    #[test]
    fn detects_portuguese_with_posix_precedence() {
        assert_eq!(detect(env(&[("LANG", "pt_BR.UTF-8")])), Lang::PtBr);
        assert_eq!(detect(env(&[("LANG", "pt_BR.UTF-8"), ("LC_ALL", "en_US.UTF-8")])), Lang::En);
        assert_eq!(detect(env(&[("LC_MESSAGES", "pt_PT.UTF-8"), ("LANG", "en_GB.UTF-8")])), Lang::PtBr);
        assert_eq!(detect(env(&[("LANG", "")])), Lang::En);
        assert_eq!(detect(env(&[])), Lang::En);
    }

    #[test]
    fn formats_dates() {
        let d = NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
        assert_eq!(Lang::En.date_long(d), "Thursday, 12 November");
        assert_eq!(Lang::PtBr.date_long(d), "Quinta-feira, 12 de novembro");
        let d = NaiveDate::from_ymd_opt(2026, 12, 21).unwrap();
        assert_eq!(Lang::En.date_short_year(d), "Mon 21 Dec 2026");
        assert_eq!(Lang::PtBr.date_short_year(d), "seg 21 dez 2026");
        assert_eq!(Lang::PtBr.day_month(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()), "1 mar");
    }

    #[test]
    fn numbers_and_plurals() {
        assert_eq!(Lang::PtBr.decimal(12.44, 1), "12,4");
        assert_eq!(Lang::En.decimal(12.44, 1), "12.4");
        assert_eq!(Lang::PtBr.days(1), "1 dia");
        assert_eq!(Lang::PtBr.days(39), "39 dias");
        assert_eq!(Lang::En.days(1), "1 day");
    }
}
