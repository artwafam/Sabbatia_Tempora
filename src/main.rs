//! Sabbatia Tempora — compact seasonal and lunar desktop companion.

mod ui;

use std::process::ExitCode;

use chrono::{Local, Utc};
use eframe::egui;
use sabbatia::calendar::almanac::Almanac;
use sabbatia::companion;
use sabbatia::config::{self, Config};

const HELP: &str = "\
Sabbatia Tempora — seasonal and lunar desktop companion

Usage: sabbatia [OPTION]

  (no option)    open the widget
  -p, --print    print today's almanac to standard output and exit
  -h, --help     show this help
  -V, --version  show the version

Configuration: $XDG_CONFIG_HOME/sabbatia/config.toml (created on first run)";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mode = args.next();
    if let Some(extra) = args.next() {
        eprintln!("sabbatia: unexpected argument '{extra}'\n\n{HELP}");
        return ExitCode::from(2);
    }

    let (cfg, warning) = config::load();
    if let Some(w) = &warning {
        eprintln!("sabbatia: {w}");
    }
    apply_timezone(&cfg);

    match mode.as_deref() {
        None => run_gui(cfg, warning),
        Some("-p" | "--print") => {
            print_almanac(&cfg);
            ExitCode::SUCCESS
        }
        Some("-h" | "--help") => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("sabbatia {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("sabbatia: unknown option '{other}'\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

/// Use the configured IANA zone for all local-time conversions.
///
/// chrono's `Local` honours `TZ` and reads the system zoneinfo database, so no
/// time zone tables need to be bundled.
fn apply_timezone(cfg: &Config) {
    if !cfg.timezone.is_empty() {
        // SAFETY: called at the start of main, before any other thread exists.
        unsafe { std::env::set_var("TZ", format!(":{}", cfg.timezone)) };
    }
}

fn print_almanac(cfg: &Config) {
    let lang = cfg.language.resolve();
    let a = Almanac::compute(Utc::now(), &Local, cfg.calendar_settings());
    let t = ui::text::Texts::build(&a, lang);
    let u = lang.ui();
    let line = |label: &str| format!("{:<14}", label.chars().next().into_iter().chain(label.chars().skip(1).flat_map(char::to_lowercase)).collect::<String>());
    println!("{}  ({})", t.date_long, t.utc_offset);
    println!();
    println!("{}{} — {}", line(u.season), t.season_title, t.season_range);
    println!("{:<14}{}", "", t.season_progress);
    println!("{}{} — {}", line(u.moon), t.moon_title, t.moon_sub);
    println!("{}{} — {}", line(u.wheel), t.wheel_title, t.wheel_sub);
    println!("{}{} — {} ({})", line(u.next_sabbat), t.sabbat_name, t.sabbat_countdown, t.sabbat_date);
    println!("{:<14}{}", "", t.sabbat_basis);
    if let Some(alt) = &t.sabbat_alternative {
        println!("{:<14}{alt}", "");
    }
    println!();
    for (name, when) in &t.moon_upcoming {
        println!("{name:<18}{when}");
    }
    println!();
    for l in companion::lines(&a, lang, cfg.character) {
        println!("  “{l}”");
    }
}

fn run_gui(cfg: Config, warning: Option<String>) -> ExitCode {
    if let Err(e) = config::sync_autostart(cfg.autostart) {
        eprintln!("sabbatia: autostart: {e}");
    }
    let window = config::load_window_state();

    // The window keeps one size: the figure, with transparent room above for
    // the speech bubble. Its empty parts let clicks through to the desktop.
    let size = ui::app::window_size(&cfg, ui::app::figure_size(&cfg));
    let size = [size.x, size.y];
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Sabbatia Tempora")
        .with_app_id(config::APP_ID)
        .with_inner_size(size)
        // No title bar or window buttons: just the figure, on a transparent
        // background, kept on the desktop layer beneath other windows.
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_taskbar(false)
        .with_window_type(egui::X11WindowType::Dock)
        .with_window_level(ui::app::window_level(&cfg))
        .with_icon(ui::assets::window_icon());
    if let (Some(x), Some(y)) = (window.foot_x, window.foot_y) {
        // The saved spot is where his feet stand. Honoured on X11; Wayland
        // compositors choose placement themselves.
        viewport = viewport.with_position([x - ui::app::MARGIN, y - (size[1] - ui::app::BELOW_FEET)]);
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: window.foot_x.is_none(),
        persist_window: false,
        ..Default::default()
    };

    let result = eframe::run_native(
        "Sabbatia Tempora",
        options,
        Box::new(move |cc| Ok(Box::new(ui::app::SabbatiaApp::new(cc, cfg, window, warning)))),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sabbatia: {e}");
            ExitCode::FAILURE
        }
    }
}
