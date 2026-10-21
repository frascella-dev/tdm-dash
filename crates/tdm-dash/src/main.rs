//! tdm-dash: the instrument cluster UI.
//!
//! Platforms: `desktop` (a window on a PC, for development), `kms` (direct
//! DRM output on the Raspberry Pi) and `headless` (screenshots, benchmark).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tdm_link::SourceSpec;
use tdm_logic::{Dash, Layout, Profile, ThemeMode};

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
tdm-dash — quadro strumenti

USO: tdm-dash [opzioni]
  --source SPEC        sim[:scenario] | serial[:/dev/ttyACMx] | replay:FILE  (default: sim:city sul PC, serial sul Pi)
  --profile FILE       profilo moto (default: /boot/tdm-dash.toml, /etc/tdm-dash/default.toml, integrato)
  --platform NOME      desktop | kms | headless
  --layout NOME        classic | sport | touring (sostituisce il profilo)
  --theme NOME         auto | day | night
  --record FILE        registra il traffico USB per --source replay:FILE
  --screenshot DIR     salva le schermate di riferimento in DIR ed esce
  --bench N            misura il tempo di disegno su N frame ed esce
";

#[derive(Debug)]
pub struct Options {
    source: Option<SourceSpec>,
    profile: Option<PathBuf>,
    platform: Option<String>,
    layout: Option<Layout>,
    theme: Option<ThemeMode>,
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
        record: None,
        screenshot: None,
        bench: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or(format!("{a} vuole un valore"));
        match a.as_str() {
            "--source" => o.source = Some(val()?.parse()?),
            "--profile" => o.profile = Some(val()?.into()),
            "--platform" => o.platform = Some(val()?),
            "--layout" => {
                o.layout = Some(match val()?.as_str() {
                    "classic" => Layout::Classic,
                    "sport" => Layout::Sport,
                    "touring" => Layout::Touring,
                    x => return Err(format!("layout sconosciuto '{x}'")),
                })
            }
            "--theme" => {
                o.theme = Some(match val()?.as_str() {
                    "auto" => ThemeMode::Auto,
                    "day" => ThemeMode::Day,
                    "night" => ThemeMode::Night,
                    x => return Err(format!("tema sconosciuto '{x}'")),
                })
            }
            "--record" => o.record = Some(val()?.into()),
            "--screenshot" => o.screenshot = Some(val()?.into()),
            "--bench" => o.bench = Some(val()?.parse().map_err(|_| "--bench vuole un numero")?),
            "-h" | "--help" => return Err(String::new()),
            x => return Err(format!("opzione sconosciuta '{x}'")),
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
        eprintln!("tdm-dash: {e}; uso il profilo integrato");
    }
    let mut d = Dash::new(profile, err);
    if let Some(l) = o.layout {
        d.set_layout(l);
    }
    if let Some(t) = o.theme {
        d.set_theme_mode(t);
    }
    d
}

fn main() -> ExitCode {
    let o = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("errore: {e}\n");
            }
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    if let Some(dir) = &o.screenshot {
        return match shots::screenshots(dir) {
            Ok(n) => {
                println!("{n} schermate salvate in {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("errore: {e}");
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
        p => Err(format!("piattaforma '{p}' non compilata in questo binario")),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("errore: {e}");
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
        "tdm-dash: M = MODE, S = SET (tieni premuto per il lungo), ←/→ frecce, H abbaglianti, L/K luce, U scollega USB, 1/2/3 layout"
    );
    ui.run().map_err(|e| e.to_string())
}
