//! The dashboard state machine. It consumes link events and a monotonic
//! clock (seconds) and produces a [`ViewModel`] plus the commands for the
//! co-processor. It has no I/O, so the whole behaviour is unit-tested.

use tdm_proto::{
    Hello, HostMsg, McuMsg, OdoStatus, Odometer, PROTO_VERSION, Telemetry, UserSettings, cmd, flag,
    status,
};

use crate::conv::{self, Coolant};
use crate::filters::{Ema, HystQuantizer, Median3, Spring};
use crate::fmt;
use crate::i18n::{self, Language, Msg, tr};
use crate::profile::{Layout, NightSource, Profile, ThemeMode};
use crate::warnings::{Cond, Icon, Severity, WarningId, Warnings};

pub const SPLASH_S: f64 = 0.9;
pub const SELFTEST_S: f64 = 1.2;
pub const STALE_S: f64 = 0.3;
pub const LOST_S: f64 = 1.0;
pub const HEARTBEAT_S: f64 = 0.25;
pub const GETINFO_RETRY_S: f64 = 0.3;
const SHORT_PRESS_S: f64 = 0.6;
const LONG_PRESS_S: f64 = 1.0;
const SETTINGS_IDLE_S: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkEvent {
    Up(String),
    Msg(McuMsg),
    Down(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Splash,
    SelfTest,
    Ride,
    Settings,
    PowerOff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Trip,
    Diag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Day,
    Night,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Ok,
    Stale,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Level {
    #[default]
    Off,
    Normal,
    Caution,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Mode,
    Set,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarningView {
    pub id: WarningId,
    pub severity: Severity,
    pub icon: Icon,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewModel {
    pub state: UiState,
    pub screen: Screen,
    pub layout: Layout,
    pub theme: Theme,
    pub link: LinkState,
    /// 0–1 of the time spent in splash / self-test, for their animations.
    pub phase: f64,
    pub speed_text: String,
    /// Needle position, 0–1 of the dial.
    pub speed_needle: f64,
    pub dial_max_kmh: u32,
    pub rpm_text: String,
    pub rpm_needle: f64,
    pub max_rpm: u32,
    pub redline_frac: f64,
    pub shift_frac: f64,
    /// Shift light on (already blinking at 4 Hz).
    pub shift: bool,
    pub tt_left: bool,
    pub tt_right: bool,
    pub tt_neutral: bool,
    pub tt_high_beam: bool,
    pub coolant_text: String,
    pub coolant_frac: f64,
    pub coolant_level: Level,
    pub coolant_warn_frac: f64,
    pub vbat_text: String,
    pub vbat_level: Level,
    pub odo_text: String,
    pub trip_a_text: String,
    pub trip_b_text: String,
    pub range_text: String,
    pub range_low: bool,
    pub ride_time_text: String,
    pub avg_text: String,
    pub max_text: String,
    pub fuel_used_text: String,
    pub consumption_text: String,
    pub warning: Option<WarningView>,
    pub icon_coolant: Level,
    pub icon_battery: Level,
    pub icon_link: bool,
    pub icon_fuel: bool,
    pub info: Option<(Icon, String)>,
    pub toast: Option<String>,
    pub settings: Vec<(String, String)>,
    pub settings_sel: usize,
    pub trip_sel: usize,
    pub diag: Vec<(String, String)>,
    pub diag_note: String,
    /// Backlight the rider sees, 0–1 (used by the desktop simulator).
    pub backlight: f64,
}

#[derive(Debug, Clone, Copy, Default)]
struct ButtonState {
    down_since: Option<f64>,
    long_fired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    Short,
    Long,
}

impl ButtonState {
    fn update(&mut self, pressed: bool, now: f64) -> Option<Press> {
        match (self.down_since, pressed) {
            (None, true) => {
                self.down_since = Some(now);
                self.long_fired = false;
                None
            }
            (Some(t0), true) if !self.long_fired && now - t0 >= LONG_PRESS_S => {
                self.long_fired = true;
                Some(Press::Long)
            }
            (Some(t0), false) => {
                self.down_since = None;
                (!self.long_fired && now - t0 < SHORT_PRESS_S).then_some(Press::Short)
            }
            _ => None,
        }
    }
}

const SETTINGS_ITEMS: [Msg; 7] = [
    Msg::SetLayout,
    Msg::SetTheme,
    Msg::SetBrightness,
    Msg::SetDiagScreen,
    Msg::SetSpeedCal,
    Msg::SetTachCal,
    Msg::SetExit,
];

#[derive(Debug)]
pub struct Dash {
    p: Profile,
    lang: Language,
    config_error: Option<String>,
    state: UiState,
    state_t: f64,
    screen: Screen,
    layout: Layout,
    theme_mode: ThemeMode,
    brightness_offset: i8,
    diag_enabled: bool,
    started: Option<f64>,
    last_t: f64,

    link_up: bool,
    last_tel_t: Option<f64>,
    hello: Option<Hello>,
    fw_mismatch: bool,
    getinfo_next: f64,
    reconnects: u32,
    frames: u64,
    seq_gaps: u32,
    last_seq: Option<u16>,
    last_uptime: Option<u64>,
    tel: Option<Telemetry>,
    odo: Option<Odometer>,
    init_sent: bool,

    speed_med: Median3,
    rpm_med: Median3,
    speed_raw: f64,
    rpm_raw: f64,
    speed_ema: Ema,
    speed_disp: f64,
    digits: HystQuantizer,
    needle_speed: Spring,
    needle_rpm: Spring,
    coolant: Option<Coolant>,
    coolant_ohm: Option<f64>,
    cool_ema: Ema,
    vbat_ema: Ema,
    light_n: Option<f64>,
    inputs: u16,

    theme: Theme,
    theme_pending: f64,
    theme_countdown: Option<f64>,
    bright: f64,
    bl_sent: Option<u16>,
    bl_next: f64,
    first_frame: bool,
    hb_next: f64,
    frame_no: u32,

    buttons: [ButtonState; 2],
    warnings: Warnings,
    turn_ride_t: f64,

    set_idx: usize,
    set_idle: f64,
    trip_sel: usize,
    toast: Option<(String, f64)>,
    ride_t: f64,
    max_kmh: i64,
    dist_start_m: Option<u32>,

    cmds: Vec<HostMsg>,
}

impl Dash {
    pub fn new(profile: Profile, config_error: Option<String>) -> Self {
        let layout = profile.display.layout;
        let theme_mode = profile.display.theme;
        Self {
            theme: if theme_mode == ThemeMode::Night {
                Theme::Night
            } else {
                Theme::Day
            },
            lang: profile.display.language,
            p: profile,
            config_error,
            state: UiState::Splash,
            state_t: 0.0,
            screen: Screen::Main,
            layout,
            theme_mode,
            brightness_offset: 0,
            diag_enabled: true,
            started: None,
            last_t: 0.0,
            link_up: false,
            last_tel_t: None,
            hello: None,
            fw_mismatch: false,
            getinfo_next: 0.0,
            reconnects: 0,
            frames: 0,
            seq_gaps: 0,
            last_seq: None,
            last_uptime: None,
            tel: None,
            odo: None,
            init_sent: false,
            speed_med: Median3::default(),
            rpm_med: Median3::default(),
            speed_raw: 0.0,
            rpm_raw: 0.0,
            speed_ema: Ema::new(0.3),
            speed_disp: 0.0,
            digits: HystQuantizer::new(0.6),
            needle_speed: Spring::default(),
            needle_rpm: Spring::default(),
            coolant: None,
            coolant_ohm: None,
            cool_ema: Ema::new(3.0),
            vbat_ema: Ema::new(1.0),
            light_n: None,
            inputs: 0,
            theme_pending: 0.0,
            theme_countdown: None,
            bright: 60.0,
            bl_sent: None,
            bl_next: 0.0,
            first_frame: false,
            hb_next: 0.0,
            frame_no: 0,
            buttons: [ButtonState::default(); 2],
            warnings: Warnings::default(),
            turn_ride_t: 0.0,
            set_idx: 0,
            set_idle: 0.0,
            trip_sel: 0,
            toast: None,
            ride_t: 0.0,
            max_kmh: 0,
            dist_start_m: None,
            cmds: Vec::new(),
        }
    }

    pub fn profile(&self) -> &Profile {
        &self.p
    }

    pub fn state(&self) -> UiState {
        self.state
    }

    /// Overrides for the simulator, screenshots and command-line options.
    pub fn set_layout(&mut self, l: Layout) {
        self.layout = l;
    }

    /// Language of the dashboard texts, overriding the profile.
    pub fn set_language(&mut self, l: Language) {
        self.lang = l;
    }

    pub fn language(&self) -> Language {
        self.lang
    }

    pub fn set_theme_mode(&mut self, m: ThemeMode) {
        self.theme_mode = m;
    }

    pub fn set_screen(&mut self, s: Screen) {
        self.screen = s;
        if self.state == UiState::Settings && s != Screen::Main {
            self.set_state(UiState::Ride);
        }
    }

    pub fn open_settings(&mut self) {
        self.set_idx = 0;
        self.set_idle = 0.0;
        self.set_state(UiState::Settings);
    }

    /// Commands for the co-processor produced since the last call.
    pub fn take_commands(&mut self) -> Vec<HostMsg> {
        std::mem::take(&mut self.cmds)
    }

    /// Call once the first frame is on screen: only then the backlight is
    /// switched on, so the rider never sees an undefined picture.
    pub fn first_frame_shown(&mut self, now: f64) {
        if !self.first_frame {
            self.first_frame = true;
            self.bl_next = now;
        }
    }

    fn set_state(&mut self, s: UiState) {
        self.state = s;
        self.state_t = 0.0;
    }

    fn toast(&mut self, msg: Msg, now: f64) {
        self.toast = Some((tr(self.lang, msg).to_string(), now + 2.2));
    }

    pub fn on_event(&mut self, ev: LinkEvent, now: f64) {
        match ev {
            LinkEvent::Up(_) => {
                self.link_up = true;
                self.hello = None;
                self.getinfo_next = now;
                self.last_seq = None;
            }
            LinkEvent::Down(_) => {
                self.link_up = false;
                self.hello = None;
            }
            LinkEvent::Msg(m) => self.on_msg(m, now),
        }
    }

    fn on_msg(&mut self, m: McuMsg, now: f64) {
        match m {
            McuMsg::Hello(h) => {
                if self.last_tel_t.is_some_and(|t| now - t > LOST_S) {
                    self.reconnects += 1;
                }
                if self.last_uptime.is_some_and(|u| h.uptime_us < u) {
                    // the MCU rebooted: old filter state is meaningless
                    self.reset_filters();
                }
                self.last_uptime = Some(h.uptime_us);
                self.fw_mismatch = h.proto != PROTO_VERSION;
                if !self.fw_mismatch {
                    let want = self.p.mcu_config();
                    if h.config != want {
                        self.cmds.push(HostMsg::SetConfig(want));
                    }
                    match h.settings {
                        Some(s) => self.apply_settings(s),
                        None => self.cmds.push(HostMsg::StoreSettings(self.user_settings())),
                    }
                    if h.odo_status == OdoStatus::Blank
                        && self.p.odometer.initial_km > 0
                        && !self.init_sent
                    {
                        self.init_sent = true;
                        self.cmds.push(HostMsg::InitOdometer {
                            odo_m: self.p.odometer.initial_km * 1000,
                        });
                    }
                }
                self.hello = Some(h);
            }
            McuMsg::Telemetry(t) => {
                if self.fw_mismatch || self.hello.is_none() {
                    return;
                }
                if let Some(s) = self.last_seq
                    && t.seq != s.wrapping_add(1)
                {
                    self.seq_gaps += 1;
                }
                self.last_seq = Some(t.seq);
                self.last_uptime = Some(t.uptime_us);
                self.last_tel_t = Some(now);
                self.frames += 1;
                if let Some(v) = conv::speed_kmh(&self.p, &t.wheel) {
                    self.speed_raw = self.speed_med.update(v);
                }
                if let Some(r) = conv::rpm(&self.p, &t.ign) {
                    self.rpm_raw = self.rpm_med.update(r);
                }
                let (c, ohm) = conv::coolant(&self.p, t.coolant_raw);
                self.coolant = Some(c);
                self.coolant_ohm = ohm;
                self.inputs = t.inputs;
                let n = conv::light_level(&self.p, t.light_raw);
                if self.light_n.is_none() && self.theme_mode == ThemeMode::Auto {
                    // first reading decides the theme without waiting
                    self.theme = if n < self.p.display.night_below {
                        Theme::Night
                    } else {
                        Theme::Day
                    };
                    self.bright = self.target_brightness(n);
                }
                self.light_n = Some(n);
                self.tel = Some(t);
            }
            McuMsg::Odometer(o) => self.odo = Some(o),
            McuMsg::Ack { cmd: c, status: s } => {
                if s != status::OK {
                    let msg = match c {
                        cmd::INIT_ODOMETER => Msg::OdoAlreadySet,
                        cmd::RESET_TRIP => Msg::TripResetFailed,
                        _ => Msg::CommandRefused,
                    };
                    self.toast(msg, now);
                }
            }
            McuMsg::PowerFail { .. } => self.set_state(UiState::PowerOff),
        }
    }

    fn reset_filters(&mut self) {
        self.speed_med.reset();
        self.rpm_med.reset();
        self.speed_ema.reset(None);
        self.cool_ema.reset(None);
        self.vbat_ema.reset(None);
        self.last_seq = None;
    }

    fn user_settings(&self) -> UserSettings {
        UserSettings {
            theme: match self.theme_mode {
                ThemeMode::Auto => 0,
                ThemeMode::Day => 1,
                ThemeMode::Night => 2,
            },
            layout: match self.layout {
                Layout::Classic => 0,
                Layout::Sport => 1,
                Layout::Touring => 2,
            },
            brightness_offset: self.brightness_offset,
            diag: self.diag_enabled,
        }
    }

    fn apply_settings(&mut self, s: UserSettings) {
        self.theme_mode = match s.theme {
            1 => ThemeMode::Day,
            2 => ThemeMode::Night,
            _ => ThemeMode::Auto,
        };
        self.layout = match s.layout {
            1 => Layout::Sport,
            2 => Layout::Touring,
            _ => Layout::Classic,
        };
        self.brightness_offset = s.brightness_offset.clamp(-30, 30);
        self.diag_enabled = s.diag;
    }

    fn store_settings(&mut self) {
        self.cmds.push(HostMsg::StoreSettings(self.user_settings()));
    }

    pub fn link_state(&self, now: f64) -> LinkState {
        match self.last_tel_t {
            Some(t) if now - t < STALE_S => LinkState::Ok,
            Some(t) if now - t < LOST_S => LinkState::Stale,
            _ => LinkState::Lost,
        }
    }

    fn target_brightness(&self, n: f64) -> f64 {
        let d = &self.p.display;
        (d.brightness_min_pct
            + (d.brightness_max_pct - d.brightness_min_pct) * n
            + self.brightness_offset as f64)
            .clamp(d.brightness_min_pct, 100.0)
    }

    fn vbat(&self) -> Option<f64> {
        self.vbat_ema.value()
    }

    fn coolant_c(&self) -> Option<f64> {
        match self.coolant {
            Some(Coolant::Celsius(_)) => self.cool_ema.value(),
            _ => None,
        }
    }

    fn range_km(&self) -> Option<f64> {
        let f = self.p.fuel.as_ref()?;
        let since_fill = self.odo?.trip_m[1] as f64 / 1000.0;
        Some(((f.tank_l - since_fill * f.l_per_100km / 100.0) / f.l_per_100km * 100.0).max(0.0))
    }

    /// Advances the dashboard to `now` (seconds, monotonic).
    pub fn tick(&mut self, now: f64) -> ViewModel {
        let started = *self.started.get_or_insert(now);
        let dt = (now - self.last_t).clamp(0.0, 0.25);
        self.last_t = now;
        self.state_t += dt;
        self.frame_no = self.frame_no.wrapping_add(1);
        let link = self.link_state(now);

        if self.link_up && self.hello.is_none() && now >= self.getinfo_next {
            self.getinfo_next = now + GETINFO_RETRY_S;
            self.cmds.push(HostMsg::GetInfo);
        }

        self.update_measurements(link, dt);
        if link == LinkState::Ok {
            self.update_buttons(now);
        }
        self.update_warnings(link, now, dt, now - started);
        self.update_states(now, dt);
        self.update_brightness(dt);
        self.emit_periodic(now);
        self.view(now, link)
    }

    fn update_measurements(&mut self, link: LinkState, dt: f64) {
        if link == LinkState::Ok {
            let e = self.speed_ema.update(self.speed_raw, dt);
            self.speed_disp = conv::displayed_kmh(&self.p, e);
            if self.speed_disp == 0.0 {
                self.digits.set(0);
            } else {
                self.digits.update(self.speed_disp);
            }
            if let Some(Coolant::Celsius(c)) = self.coolant {
                self.cool_ema.update(c, dt);
            }
            if let Some(t) = self.tel {
                self.vbat_ema
                    .update(conv::vbat_volts(&self.p, t.vbat_raw), dt);
            }
            if self.digits.update(self.speed_disp) > 2 {
                self.ride_t += dt;
            }
            self.max_kmh = self.max_kmh.max(self.digits.update(self.speed_disp));
            if self.dist_start_m.is_none() {
                self.dist_start_m = self.odo.map(|o| o.trip_m[0]);
            }
        }
        // stale: needles freeze; lost: needles park at zero
        match link {
            LinkState::Ok => {
                self.needle_speed.update(self.speed_disp, 10.0, dt);
                self.needle_rpm.update(self.rpm_raw, 25.0, dt);
            }
            LinkState::Lost => {
                self.needle_speed.update(0.0, 10.0, dt);
                self.needle_rpm.update(0.0, 25.0, dt);
            }
            LinkState::Stale => {}
        }
    }

    fn update_buttons(&mut self, now: f64) {
        let i = &self.p.inputs;
        let (m, s) = (
            i.btn_mode.active(self.inputs),
            i.btn_set.active(self.inputs),
        );
        if let Some(p) = self.buttons[0].update(m, now) {
            self.on_button(Button::Mode, p == Press::Long, now);
        }
        if let Some(p) = self.buttons[1].update(s, now) {
            self.on_button(Button::Set, p == Press::Long, now);
        }
    }

    fn speed_shown(&self) -> i64 {
        let mut q = self.digits;
        q.update(self.speed_disp)
    }

    fn on_button(&mut self, b: Button, long: bool, now: f64) {
        match self.state {
            UiState::Settings => {
                self.set_idle = 0.0;
                match (b, long) {
                    (Button::Mode, false) => {
                        self.set_idx = (self.set_idx + 1) % SETTINGS_ITEMS.len()
                    }
                    (Button::Set, true) => self.set_state(UiState::Ride),
                    (Button::Set, false) => self.change_setting(now),
                    _ => {}
                }
            }
            UiState::Ride => {
                let top = self.warnings.active(now, false).first().copied();
                if b == Button::Set
                    && !long
                    && let Some(w) = top
                    && w.id.severity() == Severity::Caution
                {
                    self.warnings.snooze(w.id, now);
                    self.toast(Msg::WarningSnoozed, now);
                    return;
                }
                match (b, long) {
                    (Button::Mode, false) => {
                        self.screen = match (self.screen, self.diag_enabled) {
                            (Screen::Main, _) => Screen::Trip,
                            (Screen::Trip, true) => Screen::Diag,
                            _ => Screen::Main,
                        };
                    }
                    (Button::Set, true) if self.screen == Screen::Trip => {
                        let idx = self.trip_sel as u8;
                        self.cmds.push(HostMsg::ResetTrip { idx });
                        if idx == 0 {
                            self.ride_t = 0.0;
                            self.max_kmh = 0;
                            self.dist_start_m = Some(0);
                        }
                        self.toast(
                            if idx == 0 {
                                Msg::TripAReset
                            } else {
                                Msg::FillRecorded
                            },
                            now,
                        );
                    }
                    (Button::Set, true) => {
                        if self.speed_shown() < 3 {
                            self.set_idx = 0;
                            self.set_idle = 0.0;
                            self.set_state(UiState::Settings);
                        } else {
                            self.toast(Msg::SettingsOnlyStopped, now);
                        }
                    }
                    (Button::Set, false) if self.screen == Screen::Trip => {
                        self.trip_sel = 1 - self.trip_sel
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn change_setting(&mut self, now: f64) {
        match self.set_idx {
            0 => {
                self.layout = match self.layout {
                    Layout::Classic => Layout::Sport,
                    Layout::Sport => Layout::Touring,
                    Layout::Touring => Layout::Classic,
                };
                self.store_settings();
            }
            1 => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Auto => ThemeMode::Day,
                    ThemeMode::Day => ThemeMode::Night,
                    ThemeMode::Night => ThemeMode::Auto,
                };
                self.store_settings();
            }
            2 => {
                self.brightness_offset = if self.brightness_offset >= 20 {
                    -20
                } else {
                    self.brightness_offset + 10
                };
                self.store_settings();
            }
            3 => {
                self.diag_enabled = !self.diag_enabled;
                self.store_settings();
            }
            4 | 5 => self.toast(Msg::EditInProfile, now),
            _ => self.set_state(UiState::Ride),
        }
    }

    fn update_warnings(&mut self, link: LinkState, now: f64, dt: f64, uptime: f64) {
        let live = link != LinkState::Lost;
        let p = &self.p;
        let t = self.coolant_c();
        let v = self.vbat().unwrap_or(13.0);
        let rpm = self.rpm_raw;
        let w = &mut self.warnings;
        // give the link a moment after boot before calling it lost
        w.update(
            WarningId::LinkLost,
            Cond::flag(!live && uptime > 3.0),
            now,
            dt,
        );
        let tc = |f: &dyn Fn(f64) -> bool| t.is_some_and(f);
        let c = &p.coolant;
        w.update(
            WarningId::CoolantCritical,
            Cond::new(
                live && tc(&|x| x >= c.critical_c),
                2.0,
                tc(&|x| x < c.critical_c - c.hysteresis_c) || !live,
                0.0,
            ),
            now,
            dt,
        );
        w.update(
            WarningId::CoolantHigh,
            Cond::new(
                live && tc(&|x| x >= c.warn_c),
                3.0,
                tc(&|x| x < c.warn_c - c.hysteresis_c) || !live,
                0.0,
            ),
            now,
            dt,
        );
        let b = &p.battery;
        w.update(
            WarningId::Overvoltage,
            Cond::new(live && v > b.critical_high_v, 1.0, v < b.high_v - 0.3, 0.0),
            now,
            dt,
        );
        w.update(
            WarningId::HighVoltage,
            Cond::new(live && v > b.high_v, 2.0, v < b.high_v - 0.3, 0.0),
            now,
            dt,
        );
        w.update(
            WarningId::NotCharging,
            Cond::new(
                live && v < b.not_charging_v && rpm > b.not_charging_min_rpm,
                30.0,
                v > b.not_charging_v + 0.3,
                10.0,
            ),
            now,
            dt,
        );
        w.update(
            WarningId::BatteryLow,
            Cond::new(
                live && v < b.low_v && rpm > 500.0,
                10.0,
                v > b.low_v + 0.4,
                0.0,
            ),
            now,
            dt,
        );
        let fault = matches!(self.coolant, Some(Coolant::Fault));
        w.update(
            WarningId::CoolantSensorFault,
            Cond::new(live && fault, 5.0, !fault, 5.0),
            now,
            dt,
        );
        w.update(WarningId::FwMismatch, Cond::flag(self.fw_mismatch), now, dt);
        w.update(
            WarningId::ConfigError,
            Cond::flag(self.config_error.is_some()),
            now,
            dt,
        );
        let odo_bad = self
            .hello
            .is_some_and(|h| matches!(h.odo_status, OdoStatus::Corrupt | OdoStatus::Missing))
            || self.tel.is_some_and(|t| t.flags & flag::FRAM_OK == 0);
        w.update(WarningId::OdometerFault, Cond::flag(odo_bad), now, dt);
        let turn = self.p.inputs.turn_left.active(self.inputs)
            || self.p.inputs.turn_right.active(self.inputs);
        // the flasher blinks: keep the timer while the input toggles
        self.turn_ride_t = if turn || (self.turn_ride_t > 0.0 && self.speed_disp > 20.0) {
            if self.speed_disp > 20.0 {
                self.turn_ride_t + dt
            } else {
                self.turn_ride_t
            }
        } else {
            0.0
        };
        let range = self.range_km();
        let w = &mut self.warnings;
        w.update(
            WarningId::TurnSignalOn,
            Cond::new(self.turn_ride_t > 30.0, 0.0, self.speed_disp < 5.0, 2.0),
            now,
            dt,
        );
        if let (Some(f), Some(r)) = (self.p.fuel.as_ref(), range) {
            let reserve_km = f.reserve_l / f.l_per_100km * 100.0;
            w.update(
                WarningId::FuelLow,
                Cond::new(r < reserve_km, 0.0, r > reserve_km + 5.0, 0.0),
                now,
                dt,
            );
        }
        if w.is_on(WarningId::CoolantCritical) {
            w.force_off(WarningId::CoolantHigh);
        }
        if w.is_on(WarningId::Overvoltage) {
            w.force_off(WarningId::HighVoltage);
        }
    }

    fn update_states(&mut self, now: f64, dt: f64) {
        let moving = self.speed_shown() >= 5;
        match self.state {
            UiState::Splash if self.state_t >= SPLASH_S => {
                let next = if self.p.display.selftest && !moving {
                    UiState::SelfTest
                } else {
                    UiState::Ride
                };
                self.set_state(next);
            }
            UiState::SelfTest if self.state_t >= SELFTEST_S || moving => {
                self.set_state(UiState::Ride)
            }
            UiState::Settings => {
                self.set_idle += dt;
                if self.set_idle > SETTINGS_IDLE_S || moving {
                    self.set_state(UiState::Ride);
                }
            }
            _ => {}
        }
        if self.toast.as_ref().is_some_and(|(_, until)| *until < now) {
            self.toast = None;
        }
    }

    fn update_brightness(&mut self, dt: f64) {
        let n = match (self.p.display.night_source, self.light_n) {
            (NightSource::Illumination, _) => {
                if self.p.inputs.illumination.active(self.inputs) {
                    0.0
                } else {
                    1.0
                }
            }
            (_, Some(n)) => n,
            (_, None) => 0.6,
        };
        match self.theme_mode {
            ThemeMode::Day => {
                self.theme = Theme::Day;
                self.theme_countdown = None;
            }
            ThemeMode::Night => {
                self.theme = Theme::Night;
                self.theme_countdown = None;
            }
            ThemeMode::Auto => {
                let d = &self.p.display;
                let want = if n < d.night_below {
                    Theme::Night
                } else if n > d.day_above {
                    Theme::Day
                } else {
                    self.theme
                };
                if want != self.theme {
                    self.theme_pending += dt;
                    let need = if want == Theme::Night { 10.0 } else { 5.0 };
                    self.theme_countdown = Some(need - self.theme_pending);
                    if self.theme_pending >= need {
                        self.theme = want;
                        self.theme_pending = 0.0;
                    }
                } else {
                    self.theme_pending = 0.0;
                    self.theme_countdown = None;
                }
            }
        }
        let target = self.target_brightness(n);
        let tau = if target > self.bright { 0.7 } else { 1.5 };
        self.bright += (target - self.bright) * (1.0 - (-dt / tau).exp());
    }

    /// Backlight PWM duty for a perceived brightness percentage (gamma 2.2).
    pub fn permille_for(pct: f64) -> u16 {
        ((pct / 100.0).clamp(0.0, 1.0).powf(2.2) * 1000.0)
            .round()
            .max(8.0) as u16
    }

    fn emit_periodic(&mut self, now: f64) {
        if !self.first_frame || self.state == UiState::PowerOff {
            return;
        }
        if now >= self.hb_next {
            self.hb_next = now + HEARTBEAT_S;
            self.cmds.push(HostMsg::Heartbeat {
                ui_frame: self.frame_no,
            });
        }
        if now >= self.bl_next {
            let pm = Self::permille_for(self.bright);
            let changed = self
                .bl_sent
                .is_none_or(|s| (s as i32 - pm as i32).abs() > 10);
            if changed {
                let fade_ms = if self.bl_sent.is_none() { 300 } else { 100 };
                self.cmds.push(HostMsg::SetBacklight {
                    permille: pm,
                    fade_ms,
                });
                self.bl_sent = Some(pm);
                self.bl_next = now + 0.1;
            }
        }
    }

    // ------------------------------------------------------------ view

    fn view(&self, now: f64, link: LinkState) -> ViewModel {
        let p = &self.p;
        let lang = self.lang;
        let selftest = self.state == UiState::SelfTest;
        let phase = match self.state {
            UiState::Splash => (self.state_t / SPLASH_S).min(1.0),
            UiState::SelfTest => (self.state_t / SELFTEST_S).min(1.0),
            _ => 0.0,
        };
        let sweep = if selftest {
            (phase * std::f64::consts::PI).sin()
        } else {
            0.0
        };
        let lost = link == LinkState::Lost;
        let digits = self.speed_shown();
        let speed_text = if selftest {
            format!("{}", (sweep * p.speed.dial_max_kmh as f64).round())
        } else if lost {
            "---".into()
        } else {
            digits.to_string()
        };
        let rpm_val = if selftest {
            sweep * p.tach.max_rpm as f64
        } else {
            self.rpm_raw
        };
        let rpm_text = if lost && !selftest {
            "----".into()
        } else {
            fmt::int(lang, ((rpm_val / 50.0).round() * 50.0) as u64)
        };
        let flash = (now * 4.0).fract() < 0.5;
        let tt = |m: &crate::profile::InputMap| selftest || (!lost && m.active(self.inputs));

        let cool = self.coolant_c();
        let coolant_level = match cool {
            Some(c) if c >= p.coolant.critical_c => Level::Critical,
            Some(c) if c >= p.coolant.warn_c => Level::Caution,
            _ => Level::Normal,
        };
        let span = p.coolant.gauge_max_c - p.coolant.gauge_min_c;
        let coolant_frac = if selftest {
            1.0
        } else {
            cool.map_or(0.0, |c| {
                ((c - p.coolant.gauge_min_c) / span).clamp(0.0, 1.0)
            })
        };
        let coolant_text = match self.coolant {
            _ if lost => "--".into(),
            Some(Coolant::Fault) => "---".into(),
            Some(Coolant::Low) => "LO".into(),
            Some(Coolant::Celsius(_)) => cool.map_or("--".into(), |c| format!("{}", c.round())),
            None => "--".into(),
        };
        let v = self.vbat();
        let vbat_level = match v {
            Some(x) if x > p.battery.critical_high_v => Level::Critical,
            Some(x) if x > p.battery.high_v || x < p.battery.low_v => Level::Caution,
            _ => Level::Normal,
        };
        let km = |m: Option<u32>, d: usize| {
            m.map_or_else(
                || {
                    if d > 0 {
                        fmt::blank(lang, 3, 1)
                    } else {
                        "-----".into()
                    }
                },
                |m| {
                    if d == 0 {
                        fmt::int(lang, m as u64 / 1000)
                    } else {
                        fmt::dec(lang, m as f64 / 1000.0, d)
                    }
                },
            )
        };
        let odo = self.odo;
        let range = self.range_km();
        let active = self.warnings.active(now, false);
        let all = self.warnings.active(now, true);
        let level_for = |icon: Icon| {
            all.iter()
                .filter(|a| a.id.icon() == icon && a.id.severity() >= Severity::Caution)
                .map(|a| match a.id.severity() {
                    Severity::Critical => Level::Critical,
                    _ => Level::Caution,
                })
                .max()
                .unwrap_or(Level::Off)
        };
        let warning = active
            .iter()
            .find(|a| a.id.severity() >= Severity::Caution)
            .map(|a| WarningView {
                id: a.id,
                severity: a.id.severity(),
                icon: a.id.icon(),
                title: a.id.title(lang).into(),
                detail: a.id.detail(lang).into(),
            });
        let info = active
            .iter()
            .find(|a| a.id.severity() == Severity::Info)
            .map(|a| {
                let text = match a.id {
                    WarningId::FuelLow => {
                        format!("~{} km", fmt::int(lang, range.unwrap_or(0.0) as u64))
                    }
                    _ => tr(lang, Msg::TurnSignal).into(),
                };
                (a.id.icon(), text)
            });
        let dist_km = match (odo, self.dist_start_m) {
            (Some(o), Some(s)) => o.trip_m[0].saturating_sub(s) as f64 / 1000.0,
            _ => 0.0,
        };
        let avg_text = if self.ride_t > 5.0 {
            format!(
                "{} km/h",
                fmt::int(lang, (dist_km / (self.ride_t / 3600.0)) as u64)
            )
        } else {
            "--".into()
        };
        let fuel_used = odo
            .zip(p.fuel.as_ref())
            .map(|(o, f)| o.trip_m[1] as f64 / 1000.0 * f.l_per_100km / 100.0);

        ViewModel {
            state: self.state,
            screen: self.screen,
            layout: self.layout,
            theme: self.theme,
            link,
            phase,
            speed_text,
            speed_needle: if selftest {
                sweep
            } else {
                (self.needle_speed.x / p.speed.dial_max_kmh as f64).clamp(-0.01, 1.01)
            },
            dial_max_kmh: p.speed.dial_max_kmh,
            rpm_text,
            rpm_needle: if selftest {
                sweep
            } else {
                (self.needle_rpm.x / p.tach.max_rpm as f64).clamp(-0.01, 1.01)
            },
            max_rpm: p.tach.max_rpm,
            redline_frac: p.tach.redline_rpm as f64 / p.tach.max_rpm as f64,
            shift_frac: p.tach.shift_rpm as f64 / p.tach.max_rpm as f64,
            shift: !selftest && !lost && self.rpm_raw >= p.tach.shift_rpm as f64 && flash,
            tt_left: tt(&p.inputs.turn_left),
            tt_right: tt(&p.inputs.turn_right),
            tt_neutral: tt(&p.inputs.neutral),
            tt_high_beam: tt(&p.inputs.high_beam),
            coolant_text,
            coolant_frac,
            coolant_level,
            coolant_warn_frac: ((p.coolant.warn_c - p.coolant.gauge_min_c) / span).clamp(0.0, 1.0),
            vbat_text: if lost {
                fmt::blank(lang, 2, 1)
            } else {
                v.map_or_else(|| fmt::blank(lang, 2, 1), |x| fmt::dec(lang, x, 1))
            },
            vbat_level,
            odo_text: km(odo.map(|o| o.odo_m), 0),
            trip_a_text: km(odo.map(|o| o.trip_m[0]), 1),
            trip_b_text: km(odo.map(|o| o.trip_m[1]), 1),
            range_text: range.map_or("--".into(), |r| format!("~{}", fmt::int(lang, r as u64))),
            range_low: active.iter().any(|a| a.id == WarningId::FuelLow),
            ride_time_text: fmt::duration(self.ride_t),
            avg_text,
            max_text: format!("{} km/h", self.max_kmh),
            fuel_used_text: fuel_used
                .map_or("--".into(), |l| format!("~{} l", fmt::dec(lang, l, 1))),
            consumption_text: p.fuel.as_ref().map_or("--".into(), |f| {
                format!("{} l/100 km", fmt::dec(lang, f.l_per_100km, 1))
            }),
            warning: if selftest { None } else { warning },
            icon_coolant: if selftest {
                Level::Caution
            } else {
                level_for(Icon::Coolant)
            },
            icon_battery: if selftest {
                Level::Caution
            } else {
                level_for(Icon::Battery)
            },
            icon_link: !selftest && self.warnings.is_on(WarningId::LinkLost),
            icon_fuel: selftest,
            info: if selftest { None } else { info },
            toast: self.toast.as_ref().map(|(m, _)| m.clone()),
            settings: self.settings_view(),
            settings_sel: self.set_idx,
            trip_sel: self.trip_sel,
            diag: self.diag_view(now, link),
            diag_note: tr(lang, Msg::DiagNote).into(),
            backlight: if self.first_frame {
                self.bright / 100.0
            } else {
                0.0
            },
        }
    }

    fn settings_view(&self) -> Vec<(String, String)> {
        let p = &self.p;
        let lang = self.lang;
        SETTINGS_ITEMS
            .iter()
            .enumerate()
            .map(|(i, &label)| {
                let v = match i {
                    0 => match self.layout {
                        Layout::Classic => "Classic",
                        Layout::Sport => "Sport",
                        Layout::Touring => "Touring",
                    }
                    .to_string(),
                    1 => tr(
                        lang,
                        match self.theme_mode {
                            ThemeMode::Auto => Msg::ThemeAuto,
                            ThemeMode::Day => Msg::ThemeDay,
                            ThemeMode::Night => Msg::ThemeNight,
                        },
                    )
                    .to_string(),
                    2 => format!("{:+} %", self.brightness_offset),
                    3 => tr(
                        lang,
                        if self.diag_enabled {
                            Msg::Shown
                        } else {
                            Msg::Hidden
                        },
                    )
                    .to_string(),
                    4 => i18n::speed_cal(
                        lang,
                        p.speed.wheel_circumference_mm,
                        p.speed.pulses_per_rev,
                    ),
                    5 => i18n::tach_cal(lang, p.tach.pulses_per_rev, p.tach.redline_rpm),
                    _ => String::new(),
                };
                (tr(lang, label).to_string(), v)
            })
            .collect()
    }

    fn diag_view(&self, now: f64, link: LinkState) -> Vec<(String, String)> {
        let lang = self.lang;
        let t = self.tel;
        let mut v = Vec::new();
        let hz = |s: &tdm_proto::PulseStats| {
            if s.span_us > 0 {
                s.span_edges as f64 / s.span_us as f64 * 1e6
            } else {
                0.0
            }
        };
        let label = |m: Msg| tr(lang, m).to_string();
        v.push((
            label(Msg::DiagWheel),
            t.map_or("--".into(), |t| {
                i18n::diag_wheel(
                    lang,
                    hz(&t.wheel),
                    t.wheel.span_edges,
                    t.wheel.span_us as f64 / 1000.0,
                )
            }),
        ));
        v.push((
            label(Msg::DiagIgnition),
            t.map_or("--".into(), |t| {
                i18n::diag_ignition(lang, hz(&t.ign), self.rpm_raw)
            }),
        ));
        v.push((
            label(Msg::DiagBattery),
            t.map_or("--".into(), |t| {
                format!(
                    "{} V · ADC {}",
                    self.vbat().map_or("--".into(), |x| fmt::dec(lang, x, 2)),
                    t.vbat_raw
                )
            }),
        ));
        v.push((
            label(Msg::DiagCoolant),
            match (self.coolant, self.coolant_ohm) {
                (Some(Coolant::Fault), _) => label(Msg::SensorFault),
                (Some(Coolant::Low), Some(o)) => i18n::diag_below_curve(lang, o),
                (Some(Coolant::Celsius(_)), Some(o)) => {
                    format!(
                        "{} Ω → {} °C",
                        fmt::dec(lang, o, 1),
                        self.coolant_c()
                            .map_or("--".into(), |c| fmt::dec(lang, c, 1))
                    )
                }
                _ => "--".into(),
            },
        ));
        v.push((
            label(Msg::DiagLight),
            t.map_or("--".into(), |t| {
                i18n::diag_light(
                    lang,
                    t.light_raw,
                    (self.light_n.unwrap_or(0.0) * 100.0).round(),
                    self.bright.round(),
                )
            }),
        ));
        let bit = |m: &crate::profile::InputMap| m.active(self.inputs) as u8;
        let i = &self.p.inputs;
        v.push((
            label(Msg::DiagInputs),
            i18n::diag_inputs(
                lang,
                [
                    bit(&i.neutral),
                    bit(&i.high_beam),
                    bit(&i.turn_left),
                    bit(&i.turn_right),
                    bit(&i.illumination),
                    bit(&i.btn_mode),
                    bit(&i.btn_set),
                ],
            ),
        ));
        let ls = tr(
            lang,
            match link {
                LinkState::Ok => Msg::LinkOk,
                LinkState::Stale => Msg::LinkStale,
                LinkState::Lost => Msg::LinkLost,
            },
        );
        v.push((
            label(Msg::DiagLink),
            i18n::diag_link(lang, ls, self.frames, self.seq_gaps, self.reconnects),
        ));
        v.push((
            label(Msg::DiagBoot),
            self.hello
                .and_then(|h| h.first_backlight_ms)
                .map_or("--".into(), |ms| i18n::diag_boot(lang, ms as f64 / 1000.0)),
        ));
        v.push((
            label(Msg::DiagFirmware),
            self.hello.map_or(label(Msg::WaitingModule), |h| {
                let fram = tr(
                    lang,
                    match h.odo_status {
                        OdoStatus::Ok => Msg::FramOk,
                        OdoStatus::Blank => Msg::FramBlank,
                        OdoStatus::Corrupt => Msg::FramCorrupt,
                        OdoStatus::Missing => Msg::FramMissing,
                    },
                );
                i18n::diag_firmware(lang, h.fw, h.proto, fram)
            }),
        ));
        if let Some(e) = &self.config_error {
            v.push((label(Msg::DiagProfile), e.clone()));
        }
        let _ = now;
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdm_proto::{PulseStats, input};

    fn hello(p: &Profile) -> McuMsg {
        McuMsg::Hello(Hello {
            proto: PROTO_VERSION,
            fw: [0, 1, 0],
            board_rev: 1,
            uptime_us: 1_000_000,
            reset_reason: 0,
            config: p.mcu_config(),
            settings: Some(UserSettings {
                diag: true,
                ..UserSettings::default()
            }),
            odo_status: OdoStatus::Ok,
            first_backlight_ms: None,
        })
    }

    fn tel(p: &Profile, kmh: f64, inputs: u16) -> McuMsg {
        let mm = p.mm_per_pulse();
        let span_edges = if kmh > 0.0 { 12 } else { 0 };
        let span_us = if kmh > 0.0 {
            (mm / (kmh / 3.6) * 1000.0 * 12.0) as u32
        } else {
            0
        };
        let vbat_raw = (13.9 / p.board.vbat_divider / 3.3 * 4095.0) as u16;
        McuMsg::Telemetry(Telemetry {
            seq: 0,
            uptime_us: 2_000_000,
            wheel: PulseStats {
                total: 0,
                span_edges,
                span_us,
                age_us: 1000,
            },
            ign: PulseStats {
                total: 0,
                span_edges: 4,
                span_us: 4 * 50_000,
                age_us: 1000,
            },
            vbat_raw,
            coolant_raw: 1300,
            light_raw: 3000,
            inputs,
            flags: flag::FRAM_OK,
        })
    }

    /// Runs the dash at 50 Hz for `secs`, feeding the same telemetry.
    fn run(d: &mut Dash, t: &mut f64, secs: f64, kmh: f64, inputs: u16) -> ViewModel {
        let p = d.profile().clone();
        let mut vm = d.tick(*t);
        let end = *t + secs;
        while *t < end {
            *t += 0.02;
            let mut m = tel(&p, kmh, inputs);
            if let McuMsg::Telemetry(x) = &mut m {
                x.seq = (*t * 50.0) as u16;
            }
            d.on_event(LinkEvent::Msg(m), *t);
            vm = d.tick(*t);
        }
        vm
    }

    fn connected() -> (Dash, f64) {
        connected_in(Language::En)
    }

    fn connected_in(lang: Language) -> (Dash, f64) {
        let p = Profile::builtin();
        let mut d = Dash::new(p.clone(), None);
        d.set_language(lang);
        d.tick(0.0);
        d.on_event(LinkEvent::Up("sim".into()), 0.0);
        assert_eq!(d.tick(0.01).state, UiState::Splash);
        assert!(d.take_commands().contains(&HostMsg::GetInfo));
        d.on_event(LinkEvent::Msg(hello(&p)), 0.02);
        d.first_frame_shown(0.02);
        (d, 0.02)
    }

    #[test]
    fn boot_sequence_reaches_ride_and_lights_backlight() {
        let (mut d, mut t) = connected();
        let vm = run(&mut d, &mut t, 1.0, 0.0, 0);
        assert_eq!(vm.state, UiState::SelfTest);
        let cmds = d.take_commands();
        assert!(
            cmds.iter()
                .any(|c| matches!(c, HostMsg::SetBacklight { fade_ms: 300, .. }))
        );
        assert!(
            cmds.iter()
                .filter(|c| matches!(c, HostMsg::Heartbeat { .. }))
                .count()
                >= 3
        );
        let vm = run(&mut d, &mut t, 1.5, 0.0, 0);
        assert_eq!(vm.state, UiState::Ride);
    }

    #[test]
    fn speed_digits_settle_above_true_speed() {
        let (mut d, mut t) = connected();
        let vm = run(&mut d, &mut t, 5.0, 100.0, 0);
        let shown: f64 = vm.speed_text.parse().unwrap();
        assert!((104.0..=105.0).contains(&shown), "shown {shown}");
        assert!((vm.speed_needle * 220.0 - 104.0).abs() < 1.5);
    }

    #[test]
    fn link_loss_shows_dashes_and_critical_warning() {
        let (mut d, mut t) = connected();
        run(&mut d, &mut t, 4.0, 60.0, 0);
        t += 0.5;
        assert_eq!(d.tick(t).link, LinkState::Stale);
        t += 0.6;
        let vm = d.tick(t);
        assert_eq!(vm.link, LinkState::Lost);
        assert_eq!(vm.speed_text, "---");
        assert_eq!(vm.warning.unwrap().id, WarningId::LinkLost);
    }

    #[test]
    fn texts_follow_the_language() {
        for (lang, title, odo) in [
            (
                Language::En,
                "Sensor module link lost",
                "Odometer already set",
            ),
            (
                Language::It,
                "Collegamento con il modulo sensori perso",
                "Contachilometri già impostato",
            ),
        ] {
            let (mut d, mut t) = connected_in(lang);
            run(&mut d, &mut t, 4.0, 60.0, 0);
            d.on_event(
                LinkEvent::Msg(McuMsg::Ack {
                    cmd: cmd::INIT_ODOMETER,
                    status: status::REFUSED,
                }),
                t,
            );
            assert_eq!(d.tick(t).toast.as_deref(), Some(odo));
            t += 1.2;
            let vm = d.tick(t);
            assert_eq!(vm.warning.unwrap().title, title);
            assert_eq!(vm.diag[0].0, tr(lang, Msg::DiagWheel));
        }
    }

    #[test]
    fn settings_only_when_stationary() {
        let (mut d, mut t) = connected();
        run(&mut d, &mut t, 3.0, 50.0, 0);
        run(&mut d, &mut t, 1.2, 50.0, input::BTN_SET);
        let vm = run(&mut d, &mut t, 0.1, 50.0, 0);
        assert_eq!(vm.state, UiState::Ride);
        assert_eq!(vm.toast.as_deref(), Some("Settings only when stopped"));
        run(&mut d, &mut t, 3.0, 0.0, 0);
        let vm = run(&mut d, &mut t, 1.2, 0.0, input::BTN_SET);
        assert_eq!(vm.state, UiState::Settings);
        // MODE short moves, SET short changes the layout and stores it
        run(&mut d, &mut t, 0.2, 0.0, 0);
        d.take_commands();
        run(&mut d, &mut t, 0.2, 0.0, input::BTN_SET);
        let vm = run(&mut d, &mut t, 0.2, 0.0, 0);
        assert_eq!(vm.layout, Layout::Sport);
        assert!(
            d.take_commands()
                .iter()
                .any(|c| matches!(c, HostMsg::StoreSettings(s) if s.layout == 1))
        );
        // riding off closes the settings
        let vm = run(&mut d, &mut t, 2.0, 30.0, 0);
        assert_eq!(vm.state, UiState::Ride);
    }

    #[test]
    fn mode_cycles_screens_and_trip_reset() {
        let (mut d, mut t) = connected();
        run(&mut d, &mut t, 3.0, 0.0, 0);
        run(&mut d, &mut t, 0.2, 0.0, input::BTN_MODE);
        let vm = run(&mut d, &mut t, 0.2, 0.0, 0);
        assert_eq!(vm.screen, Screen::Trip);
        d.take_commands();
        run(&mut d, &mut t, 1.2, 0.0, input::BTN_SET);
        assert!(d.take_commands().contains(&HostMsg::ResetTrip { idx: 0 }));
        run(&mut d, &mut t, 0.2, 0.0, 0);
        run(&mut d, &mut t, 0.2, 0.0, input::BTN_MODE);
        let vm = run(&mut d, &mut t, 0.2, 0.0, 0);
        assert_eq!(vm.screen, Screen::Diag);
    }

    #[test]
    fn hello_with_other_config_gets_set_config() {
        let p = Profile::builtin();
        let mut d = Dash::new(p.clone(), None);
        d.on_event(LinkEvent::Up("x".into()), 0.0);
        let mut h = hello(&p);
        if let McuMsg::Hello(x) = &mut h {
            x.config.um_per_pulse = 1;
            x.settings = None;
        }
        d.on_event(LinkEvent::Msg(h), 0.1);
        let cmds = d.take_commands();
        assert!(cmds.contains(&HostMsg::SetConfig(p.mcu_config())));
        assert!(cmds.iter().any(|c| matches!(c, HostMsg::StoreSettings(_))));
    }

    #[test]
    fn protocol_mismatch_ignores_telemetry() {
        let p = Profile::builtin();
        let mut d = Dash::new(p.clone(), None);
        d.on_event(LinkEvent::Up("x".into()), 0.0);
        let mut h = hello(&p);
        if let McuMsg::Hello(x) = &mut h {
            x.proto = PROTO_VERSION + 1;
        }
        d.on_event(LinkEvent::Msg(h), 0.1);
        d.on_event(LinkEvent::Msg(tel(&p, 50.0, 0)), 0.2);
        let vm = d.tick(0.3);
        assert_eq!(vm.link, LinkState::Lost);
        assert!(d.warnings.is_on(WarningId::FwMismatch));
    }

    #[test]
    fn power_fail_stops_everything() {
        let (mut d, mut t) = connected();
        run(&mut d, &mut t, 1.0, 0.0, 0);
        d.take_commands();
        d.on_event(LinkEvent::Msg(McuMsg::PowerFail { vbat_raw: 100 }), t);
        let vm = d.tick(t + 0.5);
        assert_eq!(vm.state, UiState::PowerOff);
        assert!(d.take_commands().is_empty());
    }

    #[test]
    fn telltales_follow_inputs() {
        let (mut d, mut t) = connected();
        run(&mut d, &mut t, 3.0, 0.0, 0);
        let vm = run(&mut d, &mut t, 0.1, 0.0, input::NEUTRAL | input::TURN_LEFT);
        assert!(vm.tt_neutral && vm.tt_left && !vm.tt_right && !vm.tt_high_beam);
    }

    #[test]
    fn backlight_gamma() {
        assert_eq!(Dash::permille_for(100.0), 1000);
        assert_eq!(Dash::permille_for(50.0), 218);
        assert_eq!(Dash::permille_for(0.0), 8);
    }
}
