//! Glue between the dashboard logic, the data source and the Slint window.

use std::rc::Rc;

use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use tdm_link::Source;
use tdm_link::sim::Turn;
use tdm_logic::dash::Level;
use tdm_logic::i18n::{self, Language, Msg, tr};
use tdm_logic::warnings::Icon;
use tdm_logic::{Dash, Layout, LinkState, Screen, Severity, Theme, UiState, ViewModel};

use crate::geometry::{self, DialKind};
use crate::theme;
use crate::{AppWindow, Label, Labels, NeedleGeom, Row, Seg};

struct Faces {
    speed: slint::Image,
    rpm: slint::Image,
}

#[allow(missing_debug_implementations)]
pub struct App {
    pub ui: AppWindow,
    pub dash: Dash,
    pub source: Box<dyn Source>,
    faces: [Option<Faces>; 2],
    theme: Option<Theme>,
    speed_chars: Rc<VecModel<SharedString>>,
    settings: Rc<VecModel<Row>>,
    diag: Rc<VecModel<Row>>,
    last_speed_needle: i32,
    last_rpm_needle: i32,
}

fn level(l: Level) -> i32 {
    match l {
        Level::Off => 0,
        Level::Normal => 1,
        Level::Caution => 2,
        Level::Critical => 3,
    }
}

fn needle(n: geometry::Needle) -> NeedleGeom {
    NeedleGeom {
        cmd: n.cmd.into(),
        x: n.x,
        y: n.y,
        w: n.w,
        h: n.h,
    }
}

fn labels(v: Vec<geometry::DialLabel>) -> ModelRc<Label> {
    ModelRc::new(VecModel::from(
        v.into_iter()
            .map(|l| Label {
                x: l.x,
                y: l.y,
                text: l.text.into(),
                red: l.red,
            })
            .collect::<Vec<_>>(),
    ))
}

/// The fixed texts of the Slint screens, in the dashboard's language.
fn set_labels(ui: &AppWindow, lang: Language) {
    let l = ui.global::<Labels>();
    let t = |m: Msg| SharedString::from(tr(lang, m));
    l.set_rpm(t(Msg::RpmUnit));
    l.set_krpm(t(Msg::KRpmUnit));
    l.set_fill(t(Msg::FillShort));
    l.set_estimated_range(t(Msg::EstimatedRange));
    l.set_range_caps(t(Msg::RangeCaps));
    l.set_riding_caps(t(Msg::RidingCaps));
    l.set_average_caps(t(Msg::AverageCaps));
    l.set_trip_title(t(Msg::TripTitle));
    l.set_trip_a(t(Msg::TripA));
    l.set_since_fill(t(Msg::SinceFill));
    l.set_riding_time(t(Msg::RidingTime));
    l.set_average(t(Msg::Average));
    l.set_top_speed(t(Msg::TopSpeed));
    l.set_est_consumption(t(Msg::EstConsumption));
    l.set_fuel_used(t(Msg::FuelUsed));
    l.set_range(t(Msg::Range));
    l.set_trip_keys(t(Msg::TripKeys));
    l.set_diag_title(t(Msg::DiagTitle));
    l.set_settings_title(t(Msg::SettingsTitle));
    l.set_only_stopped(t(Msg::OnlyStopped));
    l.set_settings_keys(t(Msg::SettingsKeys));
}

impl App {
    pub fn new(ui: AppWindow, dash: Dash, source: Box<dyn Source>) -> Self {
        let p = dash.profile().clone();
        let (segs, arc_labels) = geometry::arc_segments(p.tach.max_rpm, p.tach.redline_rpm);
        ui.set_arc_segs(ModelRc::new(VecModel::from(
            segs.into_iter()
                .map(|s| Seg {
                    cmd: s.cmd.into(),
                    at: s.at,
                    red: s.red,
                    x: s.x,
                    y: s.y,
                    w: s.w,
                    h: s.h,
                })
                .collect::<Vec<_>>(),
        )));
        ui.set_arc_labels(labels(arc_labels));
        ui.set_redline_frac(p.tach.redline_rpm as f32 / p.tach.max_rpm as f32);
        set_labels(&ui, dash.language());
        ui.set_splash_subtitle(
            i18n::splash_profile(dash.language(), &p.bike.name, env!("CARGO_PKG_VERSION")).into(),
        );
        let speed_chars = Rc::new(VecModel::default());
        ui.set_speed_chars(ModelRc::from(speed_chars.clone()));
        let settings = Rc::new(VecModel::default());
        ui.set_settings(ModelRc::from(settings.clone()));
        let diag = Rc::new(VecModel::default());
        ui.set_diag(ModelRc::from(diag.clone()));
        Self {
            ui,
            dash,
            source,
            faces: [None, None],
            theme: None,
            speed_chars,
            settings,
            diag,
            last_speed_needle: i32::MIN,
            last_rpm_needle: i32::MIN,
        }
    }

    /// One UI frame at time `now` (seconds since start).
    pub fn frame(&mut self, now: f64) -> ViewModel {
        for ev in self.source.poll(now) {
            self.dash.on_event(ev, now);
        }
        let vm = self.dash.tick(now);
        for c in self.dash.take_commands() {
            self.source.send(c);
        }
        self.apply(&vm);
        vm
    }

    fn faces(&mut self, t: Theme) -> &Faces {
        let idx = (t == Theme::Night) as usize;
        if self.faces[idx].is_none() {
            let p = self.dash.profile();
            let c = theme::colors(t);
            let (speed, sl) = geometry::dial_face(
                DialKind::Speed {
                    max_kmh: p.speed.dial_max_kmh,
                },
                c,
            );
            let (rpm, rl) = geometry::dial_face(
                DialKind::Rpm {
                    max_rpm: p.tach.max_rpm,
                    redline: p.tach.redline_rpm,
                },
                c,
            );
            self.ui.set_speed_labels(labels(sl));
            self.ui.set_rpm_labels(labels(rl));
            self.faces[idx] = Some(Faces { speed, rpm });
        }
        self.faces[idx].as_ref().expect("faces")
    }

    fn apply(&mut self, vm: &ViewModel) {
        if self.theme != Some(vm.theme) {
            self.theme = Some(vm.theme);
            theme::apply(&self.ui, theme::colors(vm.theme));
            // labels are per theme too: regenerate on switch
            let idx = (vm.theme == Theme::Night) as usize;
            self.faces[idx] = None;
            let f = self.faces(vm.theme);
            let (s, r) = (f.speed.clone(), f.rpm.clone());
            self.ui.set_speed_face(s);
            self.ui.set_rpm_face(r);
        }
        let ui = &self.ui;
        ui.set_ui_state(match vm.state {
            UiState::Splash => 0,
            UiState::SelfTest => 1,
            UiState::Ride => 2,
            UiState::Settings => 3,
            UiState::PowerOff => 4,
        });
        ui.set_screen(match vm.screen {
            Screen::Main => 0,
            Screen::Trip => 1,
            Screen::Diag => 2,
        });
        ui.set_layout(match vm.layout {
            Layout::Classic => 0,
            Layout::Sport => 1,
            Layout::Touring => 2,
        });
        ui.set_phase(vm.phase as f32);
        ui.set_stale(vm.link == LinkState::Stale);

        let chars: Vec<SharedString> = vm
            .speed_text
            .chars()
            .map(|c| c.to_string().into())
            .collect();
        if self.speed_chars.iter().ne(chars.iter().cloned()) {
            self.speed_chars.set_vec(chars);
        }
        // needles: only rebuild the path when they move by ≥ 0.05 % of the dial
        let sn = (vm.speed_needle * 2000.0).round() as i32;
        if sn != self.last_speed_needle {
            self.last_speed_needle = sn;
            ui.set_speed_needle(needle(geometry::needle(vm.speed_needle)));
        }
        let rn = (vm.rpm_needle * 2000.0).round() as i32;
        if rn != self.last_rpm_needle {
            self.last_rpm_needle = rn;
            ui.set_rpm_needle(needle(geometry::needle(vm.rpm_needle)));
        }
        ui.set_rpm_text(vm.rpm_text.clone().into());
        ui.set_rpm_frac(vm.rpm_needle as f32);
        ui.set_shift(vm.shift);
        ui.set_tt_left(vm.tt_left);
        ui.set_tt_right(vm.tt_right);
        ui.set_tt_neutral(vm.tt_neutral);
        ui.set_tt_high_beam(vm.tt_high_beam);
        ui.set_icon_coolant(level(vm.icon_coolant));
        ui.set_icon_battery(level(vm.icon_battery));
        ui.set_icon_link(vm.icon_link);
        ui.set_icon_fuel(vm.icon_fuel);
        ui.set_coolant_text(vm.coolant_text.clone().into());
        ui.set_coolant_frac(vm.coolant_frac as f32);
        ui.set_coolant_warn_frac(vm.coolant_warn_frac as f32);
        ui.set_coolant_level(level(vm.coolant_level));
        ui.set_vbat_text(vm.vbat_text.clone().into());
        ui.set_vbat_level(level(vm.vbat_level));
        ui.set_odo_text(vm.odo_text.clone().into());
        ui.set_trip_a_text(vm.trip_a_text.clone().into());
        ui.set_trip_b_text(vm.trip_b_text.clone().into());
        ui.set_range_text(vm.range_text.clone().into());
        ui.set_range_low(vm.range_low);
        ui.set_ride_time_text(vm.ride_time_text.clone().into());
        ui.set_avg_text(vm.avg_text.clone().into());
        ui.set_max_text(vm.max_text.clone().into());
        ui.set_fuel_used_text(vm.fuel_used_text.clone().into());
        ui.set_consumption_text(vm.consumption_text.clone().into());
        match &vm.warning {
            Some(w) => {
                ui.set_warning_severity(match w.severity {
                    Severity::Critical => 3,
                    Severity::Caution => 2,
                    Severity::Info => 0,
                });
                ui.set_warning_icon(match w.icon {
                    Icon::Link => 1,
                    Icon::Coolant => 2,
                    Icon::Battery => 3,
                    _ => 4,
                });
                ui.set_warning_title(w.title.clone().into());
                ui.set_warning_detail(w.detail.clone().into());
            }
            None => ui.set_warning_severity(0),
        }
        match &vm.info {
            Some((icon, text)) => {
                ui.set_info_icon(if *icon == Icon::Fuel { 1 } else { 2 });
                ui.set_info_text(text.clone().into());
            }
            None => ui.set_info_icon(0),
        }
        ui.set_toast(vm.toast.clone().unwrap_or_default().into());
        let rows = |v: &[(String, String)]| -> Vec<Row> {
            v.iter()
                .map(|(a, b)| Row {
                    label: a.clone().into(),
                    value: b.clone().into(),
                })
                .collect()
        };
        if vm.state == UiState::Settings {
            let r = rows(&vm.settings);
            if self.settings.iter().ne(r.iter().cloned()) {
                self.settings.set_vec(r);
            }
        }
        if vm.screen == Screen::Diag {
            let r = rows(&vm.diag);
            if self.diag.iter().ne(r.iter().cloned()) {
                self.diag.set_vec(r);
            }
        }
        ui.set_settings_sel(vm.settings_sel as i32);
        ui.set_trip_sel(vm.trip_sel as i32);
        ui.set_diag_note(vm.diag_note.clone().into());
    }

    #[cfg_attr(not(feature = "desktop"), allow(dead_code))]
    /// Desktop keys drive the simulator's inputs, so they go through the same
    /// MCU debouncing and gesture decoding as the real buttons.
    pub fn key(&mut self, text: &str, pressed: bool) {
        let now = self.source.sim().map(|s| s.now()).unwrap_or(0.0);
        let Some(sim) = self.source.sim() else {
            return;
        };
        let left = char::from(slint::platform::Key::LeftArrow).to_string();
        let right = char::from(slint::platform::Key::RightArrow).to_string();
        match text {
            "m" | "M" => sim.manual.mode = pressed,
            "s" | "S" => sim.manual.set = pressed,
            "h" | "H" if pressed => sim.manual.high_beam = !sim.manual.high_beam,
            "u" | "U" if pressed => sim.cut_link(4.0),
            "l" if pressed => sim.manual.lux = (sim.manual.lux * 3.0).min(100_000.0),
            "k" if pressed => sim.manual.lux = (sim.manual.lux / 3.0).max(1.0),
            t if pressed && t == left => sim.manual.turn = Some((Turn::Left, now + 5.0)),
            t if pressed && t == right => sim.manual.turn = Some((Turn::Right, now + 5.0)),
            "1" if pressed => self.dash.set_layout(Layout::Classic),
            "2" if pressed => self.dash.set_layout(Layout::Sport),
            "3" if pressed => self.dash.set_layout(Layout::Touring),
            _ => {}
        }
    }
}
