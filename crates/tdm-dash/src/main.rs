//! tdm-dash: the instrument cluster UI.
//!
//! Platforms: `desktop` (a window on a PC, for development), `kms` (direct
//! DRM output on the Raspberry Pi) and `headless` (screenshots, benchmark).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tdm_link::SourceSpec;
use tdm_logic::{Dash, Language, Layout, Profile, ThemeMode};

mod app;
mod geometry;
mod headless;
#[cfg(feature = "kms")]
mod kms;
mod shots;
mod theme;

#[allow(missing_debug_implementations)]
mod ui {
    slint::include_modules!();
}
pub use ui::*;

const USAGE: &str = "\
tdm-dash — instrument cluster

USAGE: tdm-dash [options]
  --source SPEC        sim[:scenario] | serial[:/dev/ttyACMx] | replay:FILE  (default: sim:city on a PC, serial on the Pi)
  --profile FILE       bike profile (default: /boot/tdm-dash.toml, /etc/tdm-dash/default.toml, built-in)
  --platform NAME      desktop | kms | headless
  --layout NAME        classic | sport | touring (overrides the profile)
  --theme NAME         auto | day | night
  --language LANG      en | it (overrides the profile)
  --record FILE        record the USB traffic for --source replay:FILE
  --screenshot DIR     save the reference screens in DIR and exit
  --bench N            measure the drawing time over N frames and exit
";

#[derive(Debug)]
pub struct Options {
    source: Option<SourceSpec>,
    profile: Option<PathBuf>,
    platform: Option<String>,
    layout: Option<Layout>,
    theme: Option<ThemeMode>,
    language: Option<Language>,
    record: Option<PathBuf>,
    screenshot: Option<PathBuf>,
    bench: Option<usize>,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options {
        source: None,
        profile: None,
        platform: None,
        layout: None,
        theme: None,
        language: None,
        record: None,
        screenshot: None,
        bench: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--source" => o.source = Some(val()?.parse()?),
            "--profile" => o.profile = Some(val()?.into()),
            "--platform" => o.platform = Some(val()?),
            "--layout" => {
                o.layout = Some(match val()?.as_str() {
                    "classic" => Layout::Classic,
                    "sport" => Layout::Sport,
                    "touring" => Layout::Touring,
                    x => return Err(format!("unknown layout '{x}'")),
                })
            }
            "--theme" => {
                o.theme = Some(match val()?.as_str() {
                    "auto" => ThemeMode::Auto,
                    "day" => ThemeMode::Day,
                    "night" => ThemeMode::Night,
                    x => return Err(format!("unknown theme '{x}'")),
                })
            }
            "--language" => {
                o.language = Some(match val()?.as_str() {
                    "en" => Language::En,
                    "it" => Language::It,
                    x => return Err(format!("unknown language '{x}'")),
                })
            }
            "--record" => o.record = Some(val()?.into()),
            "--screenshot" => o.screenshot = Some(val()?.into()),
            "--bench" => o.bench = Some(val()?.parse().map_err(|_| "--bench needs a number")?),
            "-h" | "--help" => return Err(String::new()),
            x => return Err(format!("unknown option '{x}'")),
        }
    }
    Ok(o)
}

/// Profile layering (docs/adr/0008).
pub fn load_profile(explicit: Option<&Path>) -> (Profile, Option<String>) {
    match explicit {
        Some(p) => match Profile::load(p) {
            Ok(prof) => (prof, None),
            Err(e) => (Profile::builtin(), Some(format!("{}: {e}", p.display()))),
        },
        None => Profile::load_layered(&[
            Path::new("/boot/tdm-dash.toml"),
            Path::new("/etc/tdm-dash/default.toml"),
        ]),
    }
}

pub fn make_dash(o: &Options) -> Dash {
    let (profile, err) = load_profile(o.profile.as_deref());
    if let Some(e) = &err {
        eprintln!("tdm-dash: {e}; using the built-in profile");
    }
    let mut d = Dash::new(profile, err);
    if let Some(l) = o.layout {
        d.set_layout(l);
    }
    if let Some(t) = o.theme {
        d.set_theme_mode(t);
    }
    if let Some(l) = o.language {
        d.set_language(l);
    }
    d
}

fn main() -> ExitCode {
    let o = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("error: {e}\n");
            }
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    if let Some(dir) = &o.screenshot {
        return match shots::screenshots(dir) {
            Ok(n) => {
                println!("{n} screens saved in {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(n) = o.bench {
        shots::bench(&o, n);
        return ExitCode::SUCCESS;
    }
    let default_platform = if cfg!(feature = "kms") && !cfg!(feature = "desktop") {
        "kms"
    } else {
        "desktop"
    };
    let platform = o
        .platform
        .clone()
        .unwrap_or_else(|| default_platform.into());
    let r = match platform.as_str() {
        #[cfg(feature = "desktop")]
        "desktop" => run_desktop(&o),
        #[cfg(feature = "kms")]
        "kms" => kms::run(&o),
        p => Err(format!("platform '{p}' is not built into this binary")),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn open_source(
    o: &Options,
    dash: &Dash,
    default: SourceSpec,
) -> Result<Box<dyn tdm_link::Source>, String> {
    let spec = o.source.clone().unwrap_or(default);
    tdm_link::open(&spec, dash.profile(), o.record.clone()).map_err(|e| e.to_string())
}

#[cfg(feature = "desktop")]
fn run_desktop(o: &Options) -> Result<(), String> {
    use slint::ComponentHandle;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    let dash = make_dash(o);
    let source = open_source(o, &dash, SourceSpec::Sim("city".into()))?;
    let ui = AppWindow::new().map_err(|e| e.to_string())?;
    let app = Rc::new(RefCell::new(app::App::new(ui.clone_strong(), dash, source)));
    let t0 = Instant::now();
    let timer = slint::Timer::default();
    {
        let app = app.clone();
        let mut first = true;
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(16),
            move || {
                let now = t0.elapsed().as_secs_f64();
                let mut a = app.borrow_mut();
                a.frame(now);
                if first {
                    a.dash.first_frame_shown(now);
                    first = false;
                }
            },
        );
    }
    {
        let app = app.clone();
        ui.on_key(move |t, pressed| app.borrow_mut().key(&t, pressed));
    }
    eprintln!(
        "tdm-dash: M = MODE, S = SET (hold for a long press), ←/→ turn signals, H high beam, L/K light, U unplug USB, 1/2/3 layout"
    );
    ui.run().map_err(|e| e.to_string())
}
