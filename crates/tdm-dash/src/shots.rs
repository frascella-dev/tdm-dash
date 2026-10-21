//! Reference screenshots and the render benchmark, both on the headless
//! platform with simulated time.

use std::path::Path;
use std::time::Duration;

use slint::ComponentHandle;
use slint::platform::software_renderer::RepaintBufferType;
use tdm_link::{SimSource, scenario};
use tdm_logic::{Dash, Form, Language, Layout, Profile, Screen, ThemeMode};

use crate::app::App;
use crate::headless::{self, Headless};
use crate::screen::Fit;
use crate::{AppWindow, Options};

pub struct Shot {
    pub name: &'static str,
    pub scenario: &'static str,
    pub layout: Layout,
    pub theme: ThemeMode,
    pub screen: Option<Screen>,
    pub settings: bool,
    pub at: f64,
    pub language: Language,
}

const fn shot(
    name: &'static str,
    scenario: &'static str,
    layout: Layout,
    theme: ThemeMode,
    at: f64,
) -> Shot {
    Shot {
        name,
        scenario,
        layout,
        theme,
        screen: None,
        settings: false,
        at,
        language: Language::En,
    }
}

pub fn catalogue() -> Vec<Shot> {
    use Layout::*;
    use ThemeMode::*;
    vec![
        shot("01-splash", "warmup", Classic, Day, 0.45),
        shot("02-self-test", "warmup", Classic, Day, 1.5),
        shot("03-classic-day", "city", Classic, Day, 15.5),
        shot("04-classic-night", "city", Classic, Night, 15.5),
        shot("05-sport-day", "mountain", Sport, Day, 9.0),
        shot("06-sport-night", "mountain", Sport, Night, 9.0),
        shot("07-touring-day", "motorway", Touring, Day, 40.0),
        shot("08-touring-night", "motorway", Touring, Night, 40.0),
        Shot {
            screen: Some(Screen::Trip),
            ..shot("09-trip", "city", Classic, Day, 8.0)
        },
        Shot {
            screen: Some(Screen::Diag),
            ..shot("10-diagnostics", "city", Classic, Day, 15.5)
        },
        Shot {
            settings: true,
            ..shot("11-settings", "warmup", Classic, Day, 4.0)
        },
        shot("12-warning-caution", "overheat", Classic, Day, 47.0),
        shot("13-warning-critical", "regulator", Sport, Night, 40.0),
        shot("14-usb-lost", "linkdrop", Classic, Day, 18.5),
        shot("15-warm-up", "warmup", Touring, Day, 6.0),
        // the same screens with the Italian texts, to check they fit
        Shot {
            language: Language::It,
            ..shot("it-07-touring-day", "motorway", Touring, Day, 40.0)
        },
        Shot {
            language: Language::It,
            screen: Some(Screen::Trip),
            ..shot("it-09-trip", "city", Classic, Day, 8.0)
        },
        Shot {
            language: Language::It,
            screen: Some(Screen::Diag),
            ..shot("it-10-diagnostics", "city", Classic, Day, 15.5)
        },
        Shot {
            language: Language::It,
            settings: true,
            ..shot("it-11-settings", "warmup", Classic, Day, 4.0)
        },
        Shot {
            language: Language::It,
            ..shot("it-12-warning-caution", "overheat", Classic, Day, 47.0)
        },
    ]
}

/// A simulated panel; `dir` is the folder of its screens, inside the
/// screenshot folder (empty for the reference 800×480 display).
pub struct Panel {
    pub dir: &'static str,
    pub w: u32,
    pub h: u32,
    pub rotation: u16,
    /// Names of the catalogue shots to take; empty for all of them.
    pub shots: &'static [&'static str],
}

const MAIN: &[&str] = &[
    "01-splash",
    "03-classic-day",
    "04-classic-night",
    "05-sport-day",
    "07-touring-day",
    "09-trip",
    "10-diagnostics",
    "11-settings",
    "12-warning-caution",
    "13-warning-critical",
];

const MAIN_IT: &[&str] = &[
    "01-splash",
    "03-classic-day",
    "04-classic-night",
    "05-sport-day",
    "06-sport-night",
    "07-touring-day",
    "08-touring-night",
    "09-trip",
    "10-diagnostics",
    "11-settings",
    "12-warning-caution",
    "13-warning-critical",
    "14-usb-lost",
    "it-07-touring-day",
    "it-09-trip",
    "it-10-diagnostics",
    "it-11-settings",
    "it-12-warning-caution",
];

pub const PANELS: &[Panel] = &[
    Panel {
        dir: "",
        w: 800,
        h: 480,
        rotation: 0,
        shots: &[],
    },
    Panel {
        dir: "1024x600",
        w: 1024,
        h: 600,
        rotation: 0,
        shots: MAIN,
    },
    Panel {
        dir: "1280x720",
        w: 1280,
        h: 720,
        rotation: 0,
        shots: MAIN,
    },
    Panel {
        dir: "1280x400",
        w: 1280,
        h: 400,
        rotation: 0,
        shots: MAIN_IT,
    },
    Panel {
        dir: "1920x480",
        w: 1920,
        h: 480,
        rotation: 0,
        shots: MAIN,
    },
    Panel {
        dir: "1920x720",
        w: 1920,
        h: 720,
        rotation: 0,
        shots: MAIN,
    },
    // a bar panel that is natively portrait, mounted on its side: the
    // screens are saved as the panel receives them
    Panel {
        dir: "400x1280-r90",
        w: 400,
        h: 1280,
        rotation: 90,
        shots: &["03-classic-day", "13-warning-critical"],
    },
];

impl Panel {
    pub fn wants(&self, shot: &Shot) -> bool {
        self.shots.is_empty() || self.shots.contains(&shot.name)
    }
}

/// Runs a shot's scenario up to its time and returns the rendered frame.
fn render_shot(h: &Headless, ui: &AppWindow, s: &Shot, fit: Fit) -> Vec<slint::Rgb8Pixel> {
    h.rebase();
    let p = Profile::builtin();
    let mut dash = Dash::new(p.clone(), None);
    dash.set_layout(s.layout);
    dash.set_theme_mode(s.theme);
    dash.set_language(s.language);
    let src = SimSource::new(&p, scenario(s.scenario).expect("scenario"), 11);
    let mut app = App::new(ui.clone_strong(), dash, Box::new(src), fit);
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    while t < s.at {
        t += dt;
        app.frame(t);
        app.dash.first_frame_shown(t);
        if let Some(sc) = s.screen
            && t > 3.0
        {
            app.dash.set_screen(sc);
        }
        if s.settings && t > 3.0 && app.dash.state() == tdm_logic::UiState::Ride {
            app.dash.open_settings();
        }
    }
    h.set_time(t);
    let mut buf = h.rgb_buffer();
    h.redraw_all();
    h.draw(&mut buf);
    buf
}

/// Saves the catalogue for every panel of [`PANELS`].
pub fn screenshots(dir: &Path) -> std::io::Result<usize> {
    let h = Headless::install(RepaintBufferType::NewBuffer);
    let ui = AppWindow::new().map_err(std::io::Error::other)?;
    ui.show().map_err(std::io::Error::other)?;
    let shots = catalogue();
    let mut n = 0;
    for panel in PANELS {
        let out = dir.join(panel.dir);
        std::fs::create_dir_all(&out)?;
        let fit = h.set_panel(panel.w, panel.h, panel.rotation, Form::Auto, None);
        let (w, hh) = h.panel();
        for s in shots.iter().filter(|s| panel.wants(s)) {
            let buf = render_shot(&h, &ui, s, fit);
            headless::save_png(&out.join(format!("{}.png", s.name)), &buf, w, hh)?;
            n += 1;
        }
    }
    Ok(n)
}

/// Draw time per frame while riding: partial rendering into an RGB565
/// buffer, as on the Pi.
pub fn bench(o: &Options, frames: usize) {
    let h = Headless::install(RepaintBufferType::ReusedBuffer);
    let (w, hh) = o.size.unwrap_or((800, 480));
    let fit = h.set_panel(w, hh, o.rotation.unwrap_or(0), Form::Auto, None);
    let ui = AppWindow::new().expect("window");
    ui.show().expect("show");
    let p = Profile::builtin();
    let mut dash = crate::make_dash(o);
    if o.layout.is_none() {
        dash.set_layout(Layout::Classic);
    }
    let sc = match &o.source {
        Some(tdm_link::SourceSpec::Sim(id)) => id.clone(),
        _ => "mountain".into(),
    };
    let src = SimSource::new(&p, scenario(&sc).expect("scenario"), 5);
    let mut app = App::new(ui.clone_strong(), dash, Box::new(src), fit);
    let mut buf = h.rgb565_buffer();
    let mut times: Vec<Duration> = Vec::with_capacity(frames);
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    // skip splash and self-test
    while t < 4.0 {
        t += dt;
        app.frame(t);
        app.dash.first_frame_shown(t);
        h.set_time(t);
        h.draw(&mut buf);
    }
    for _ in 0..frames {
        t += dt;
        let t0 = std::time::Instant::now();
        app.frame(t);
        h.set_time(t);
        let draw = h.draw(&mut buf);
        if draw.is_some() {
            times.push(t0.elapsed());
        }
    }
    times.sort();
    let pct = |q: f64| {
        times
            .get(((times.len() as f64 - 1.0) * q) as usize)
            .copied()
            .unwrap_or_default()
    };
    println!(
        "{w}x{hh}, scenario {sc}, {} frames drawn out of {frames}: p50 {:.2} ms · p95 {:.2} ms · max {:.2} ms",
        times.len(),
        pct(0.5).as_secs_f64() * 1e3,
        pct(0.95).as_secs_f64() * 1e3,
        pct(1.0).as_secs_f64() * 1e3
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders every reference screen on every panel (one test: Slint
    /// allows one platform per process) and checks a few pixels that prove
    /// the screen is right.
    #[test]
    fn every_screen_renders() {
        let h = Headless::install(RepaintBufferType::NewBuffer);
        let ui = AppWindow::new().unwrap();
        ui.show().unwrap();
        for panel in PANELS {
            let fit = h.set_panel(panel.w, panel.h, panel.rotation, Form::Auto, None);
            let (w, hh) = h.panel();
            for s in catalogue().iter().filter(|s| panel.wants(s)) {
                let name = format!("{}/{}", panel.dir, s.name);
                let buf = render_shot(&h, &ui, s, fit);
                assert_eq!(buf.len(), w * hh);
                let distinct: std::collections::HashSet<_> =
                    buf.iter().step_by(97).map(|p| (p.r, p.g, p.b)).collect();
                assert!(distinct.len() > 8, "{name} looks blank");
                // logical point → buffer index, through the rotation
                let at = |x: f32, y: f32| {
                    let (px, py) = ((x * fit.scale) as usize, (y * fit.scale) as usize);
                    match panel.rotation {
                        90 => px * w + (w - 1 - py),
                        _ => py * w + px,
                    }
                };
                if s.name == "13-warning-critical" {
                    // critical band: red across the full width under the telltales
                    for x in [0.1, 0.5, 0.9] {
                        let p = buf[at(fit.logical.0 * x, 90.0)];
                        assert!(
                            p.r > 120 && p.g < 60 && p.b < 60,
                            "{name}: no critical band: {p:?}"
                        );
                    }
                }
                if s.name == "04-classic-night" {
                    let p = buf[at(5.0, 5.0)];
                    assert!(p.r < 20 && p.g < 20, "{name}: night background expected");
                }
            }
        }
    }
}
