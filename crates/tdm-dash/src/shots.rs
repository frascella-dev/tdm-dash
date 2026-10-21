//! Reference screenshots and the render benchmark, both on the headless
//! platform with simulated time.

use std::path::Path;
use std::time::Duration;

use slint::ComponentHandle;
use slint::platform::software_renderer::RepaintBufferType;
use tdm_link::{SimSource, scenario};
use tdm_logic::{Dash, Language, Layout, Profile, Screen, ThemeMode};

use crate::app::App;
use crate::headless::{self, Headless};
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

/// Runs a shot's scenario up to its time and returns the rendered frame.
fn render_shot(h: &Headless, ui: &AppWindow, s: &Shot) -> Vec<slint::Rgb8Pixel> {
    h.rebase();
    let p = Profile::builtin();
    let mut dash = Dash::new(p.clone(), None);
    dash.set_layout(s.layout);
    dash.set_theme_mode(s.theme);
    dash.set_language(s.language);
    let src = SimSource::new(&p, scenario(s.scenario).expect("scenario"), 11);
    let mut app = App::new(ui.clone_strong(), dash, Box::new(src));
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
    let mut buf = headless::rgb_buffer();
    h.redraw_all();
    h.draw(&mut buf);
    buf
}

pub fn screenshots(dir: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    let h = Headless::install(RepaintBufferType::NewBuffer);
    let ui = AppWindow::new().map_err(std::io::Error::other)?;
    ui.show().map_err(std::io::Error::other)?;
    let shots = catalogue();
    for s in &shots {
        let buf = render_shot(&h, &ui, s);
        headless::save_png(&dir.join(format!("{}.png", s.name)), &buf)?;
    }
    Ok(shots.len())
}

/// Draw time per frame while riding: partial rendering into an RGB565
/// buffer, as on the Pi.
pub fn bench(o: &Options, frames: usize) {
    let h = Headless::install(RepaintBufferType::ReusedBuffer);
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
    let mut app = App::new(ui.clone_strong(), dash, Box::new(src));
    let mut buf = headless::rgb565_buffer();
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
        "scenario {sc}, {} frames drawn out of {frames}: p50 {:.2} ms · p95 {:.2} ms · max {:.2} ms",
        times.len(),
        pct(0.5).as_secs_f64() * 1e3,
        pct(0.95).as_secs_f64() * 1e3,
        pct(1.0).as_secs_f64() * 1e3
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders every reference screen (one test: Slint allows one platform
    /// per process) and checks a few pixels that prove the screen is right.
    #[test]
    fn every_screen_renders() {
        let h = Headless::install(RepaintBufferType::NewBuffer);
        let ui = AppWindow::new().unwrap();
        ui.show().unwrap();
        for s in catalogue() {
            let buf = render_shot(&h, &ui, &s);
            let distinct: std::collections::HashSet<_> =
                buf.iter().step_by(97).map(|p| (p.r, p.g, p.b)).collect();
            assert!(distinct.len() > 8, "{} looks blank", s.name);
            if s.name == "13-warning-critical" {
                // critical band: red across the full width under the telltales
                let p = buf[90 * headless::W + 700];
                assert!(p.r > 120 && p.g < 60 && p.b < 60, "no critical band: {p:?}");
            }
            if s.name == "04-classic-night" {
                let p = buf[5 * headless::W + 5];
                assert!(p.r < 20 && p.g < 20, "night background expected");
            }
        }
    }
}
