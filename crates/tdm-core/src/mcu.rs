//! The co-processor's application logic without any I/O: the firmware feeds
//! it edges, ADC readings, input words and host messages, and the desktop
//! simulator does the same with simulated signals. Both therefore run exactly
//! this code.

use tdm_proto::{
    HostMsg, McuConfig, McuMsg, OdoStatus, Odometer, PROTO_VERSION, Telemetry, UserSettings, cmd,
    flag, status,
};

use crate::OdoState;
use crate::debounce::Debouncer;
use crate::nvm::{self, CONFIG_SLOTS, Loaded, Nvm, ODO_SLOTS, PAYLOAD_LEN, SETTINGS_SLOTS};
use crate::pulse::{PulseConfig, PulseMeter};

/// Persist the odometer after this much distance (10 m).
pub const ODO_SAVE_EVERY_UM: u64 = 10_000_000;
/// Backlight goes dark when the UI heartbeat is missing this long.
pub const HEARTBEAT_TIMEOUT_US: u64 = 2_000_000;
pub const TELEMETRY_PERIOD_US: u64 = 20_000;
pub const ODOMETER_PERIOD_US: u64 = 500_000;

pub const FW_VERSION: [u8; 3] = [0, 1, 0];

/// Settings used until the host sends its profile (TDM 850 3VD defaults), so
/// distance is counted correctly from the first metre even before Linux boots.
pub const DEFAULT_CONFIG: McuConfig = McuConfig {
    um_per_pulse: 1_975_000 / 6,
    wheel_ppr: 6,
    wheel_min_gap_us: 2_500,
    wheel_min_window_us: 100_000,
    ign_min_gap_us: 2_400,
    ign_blank_pct: 40,
    ign_min_window_us: 40_000,
    debounce_ms: 20,
};

pub const PULSE_BUF: usize = 64;

fn wheel_cfg(c: &McuConfig) -> PulseConfig {
    PulseConfig {
        group: c.wheel_ppr.max(1) as u16,
        min_window_us: c.wheel_min_window_us,
        max_groups: 4,
        min_gap_us: c.wheel_min_gap_us,
        blank_pct: 0,
        reset_us: 1_500_000,
    }
}

fn ign_cfg(c: &McuConfig) -> PulseConfig {
    PulseConfig {
        group: 1,
        min_window_us: c.ign_min_window_us,
        max_groups: 16,
        min_gap_us: c.ign_min_gap_us,
        blank_pct: c.ign_blank_pct,
        reset_us: 500_000,
    }
}

fn valid_config(c: &McuConfig) -> bool {
    c.um_per_pulse > 0
        && c.wheel_ppr > 0
        && (c.wheel_ppr as usize) * 4 < PULSE_BUF
        && c.ign_blank_pct < 90
        && c.debounce_ms > 0
}

#[derive(Debug, Clone, Copy, Default)]
struct Fade {
    from: u16,
    to: u16,
    t0: u64,
    dur: u64,
}

impl Fade {
    fn value(&self, now: u64) -> u16 {
        if self.dur == 0 || now >= self.t0 + self.dur {
            return self.to;
        }
        let k = (now - self.t0) as i64;
        let d = self.to as i64 - self.from as i64;
        (self.from as i64 + d * k / self.dur as i64) as u16
    }
}

/// Output of [`McuCore::poll`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Poll {
    pub telemetry: Option<Telemetry>,
    pub odometer: Option<Odometer>,
    /// Backlight PWM duty, 0–1000.
    pub backlight_permille: u16,
}

#[derive(Debug)]
pub struct McuCore<S: Nvm> {
    store: S,
    fram_ok: bool,
    cfg: McuConfig,
    cfg_seq: u32,
    settings: Option<UserSettings>,
    settings_seq: u32,
    wheel: PulseMeter<PULSE_BUF>,
    ign: PulseMeter<PULSE_BUF>,
    odo: OdoState,
    odo_seq: u32,
    odo_saved_um: u64,
    odo_status: OdoStatus,
    odo_initialised: bool,
    deb: Debouncer,
    inputs: u16,
    adc: [u16; 3],
    seq: u16,
    next_telemetry: u64,
    next_odometer: u64,
    host_seen: bool,
    last_heartbeat: Option<u64>,
    fade: Fade,
    first_backlight_ms: Option<u32>,
    bootloader: bool,
    power_low: bool,
    reset_reason: u8,
}

impl<S: Nvm> McuCore<S> {
    /// Loads configuration, settings and odometer from NVM.
    pub fn new(mut store: S, reset_reason: u8) -> Self {
        let mut fram_ok = true;
        let mut cfg = DEFAULT_CONFIG;
        let mut cfg_seq = 0;
        match nvm::load(&mut store, CONFIG_SLOTS) {
            Ok(Loaded::Valid { seq, payload }) => {
                let c = config_from_bytes(&payload);
                if valid_config(&c) {
                    cfg = c;
                    cfg_seq = seq;
                }
            }
            Ok(_) => {}
            Err(_) => fram_ok = false,
        }
        let (mut settings, mut settings_seq) = (None, 0);
        if let Ok(Loaded::Valid { seq, payload }) = nvm::load(&mut store, SETTINGS_SLOTS) {
            settings = Some(settings_from_bytes(&payload));
            settings_seq = seq;
        }
        let (mut odo, mut odo_seq, mut odo_initialised) = (OdoState::default(), 0, false);
        let odo_status = match nvm::load(&mut store, ODO_SLOTS) {
            Ok(Loaded::Valid { seq, payload }) => {
                (odo, odo_initialised) = odo_from_bytes(&payload);
                odo_seq = seq;
                OdoStatus::Ok
            }
            Ok(Loaded::Blank) => OdoStatus::Blank,
            Ok(Loaded::Corrupt) => OdoStatus::Corrupt,
            Err(_) => {
                fram_ok = false;
                OdoStatus::Missing
            }
        };
        let mut deb = Debouncer::new(cfg.debounce_ms as u16);
        deb.set_threshold(0, 50); // neutral switch contacts bounce longer
        Self {
            store,
            fram_ok,
            cfg,
            cfg_seq,
            settings,
            settings_seq,
            wheel: PulseMeter::new(wheel_cfg(&cfg)),
            ign: PulseMeter::new(ign_cfg(&cfg)),
            odo,
            odo_seq,
            odo_saved_um: odo.odo_um,
            odo_status,
            odo_initialised,
            deb,
            inputs: 0,
            adc: [0; 3],
            seq: 0,
            next_telemetry: 0,
            next_odometer: 0,
            host_seen: false,
            last_heartbeat: None,
            fade: Fade::default(),
            first_backlight_ms: None,
            bootloader: false,
            power_low: false,
            reset_reason,
        }
    }

    pub fn config(&self) -> McuConfig {
        self.cfg
    }

    pub fn odometer(&self) -> OdoState {
        self.odo
    }

    pub fn odo_status(&self) -> OdoStatus {
        self.odo_status
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    /// Overwrites the trip counters and persists them. Only the desktop
    /// simulator uses this, to start from a realistic state.
    #[doc(hidden)]
    pub fn set_trips_for_simulation(&mut self, trip_um: [u64; 2]) {
        self.odo.trip_um = trip_um;
        self.save_odo();
    }

    /// True once after the host asked to reboot into the USB bootloader.
    pub fn take_bootloader_request(&mut self) -> bool {
        core::mem::take(&mut self.bootloader)
    }

    pub fn on_wheel_edge(&mut self, t_us: u64) {
        if self.wheel.on_edge(t_us) {
            self.odo.add_distance(self.cfg.um_per_pulse);
            if self.odo.odo_um - self.odo_saved_um >= ODO_SAVE_EVERY_UM {
                self.save_odo();
            }
        }
    }

    pub fn on_ign_edge(&mut self, t_us: u64) {
        self.ign.on_edge(t_us);
    }

    /// Raw input word (see `tdm_proto::input`), sampled every `dt_ms`.
    pub fn on_inputs(&mut self, raw: u16, dt_ms: u16) {
        self.inputs = self.deb.update(raw, dt_ms);
    }

    /// Oversampled 12-bit readings: battery, coolant, ambient light.
    pub fn on_adc(&mut self, vbat: u16, coolant: u16, light: u16) {
        self.adc = [vbat, coolant, light];
    }

    /// Supply collapsing: save the odometer while the hold-up capacitor lasts.
    pub fn on_power_fail(&mut self) -> McuMsg {
        self.power_low = true;
        self.save_odo();
        McuMsg::PowerFail {
            vbat_raw: self.adc[0],
        }
    }

    /// USB detached or the host closed the port: wait for it to speak first.
    pub fn on_host_lost(&mut self) {
        self.host_seen = false;
    }

    pub fn on_host(&mut self, msg: HostMsg, now: u64) -> Option<McuMsg> {
        self.host_seen = true;
        let ack = |c, ok: bool| {
            Some(McuMsg::Ack {
                cmd: c,
                status: if ok { status::OK } else { status::REFUSED },
            })
        };
        match msg {
            HostMsg::GetInfo => Some(McuMsg::Hello(self.hello(now))),
            HostMsg::Heartbeat { .. } => {
                self.last_heartbeat = Some(now);
                None
            }
            HostMsg::SetBacklight { permille, fade_ms } => {
                self.last_heartbeat = Some(now);
                self.set_backlight(permille.min(1000), fade_ms as u64 * 1000, now);
                if self.first_backlight_ms.is_none() && permille > 0 {
                    self.first_backlight_ms = Some((now / 1000) as u32);
                }
                None
            }
            HostMsg::SetConfig(c) => {
                if !valid_config(&c) {
                    return ack(cmd::SET_CONFIG, false);
                }
                if c != self.cfg {
                    self.cfg = c;
                    self.wheel.configure(wheel_cfg(&c));
                    self.ign.configure(ign_cfg(&c));
                    self.deb.set_all(c.debounce_ms as u16);
                    self.deb.set_threshold(0, 50);
                    self.cfg_seq = self.cfg_seq.wrapping_add(1);
                    let ok = nvm::save(
                        &mut self.store,
                        CONFIG_SLOTS,
                        self.cfg_seq,
                        &config_to_bytes(&c),
                    )
                    .is_ok();
                    return ack(cmd::SET_CONFIG, ok);
                }
                ack(cmd::SET_CONFIG, true)
            }
            HostMsg::ResetTrip { idx } => {
                if idx > 1 {
                    return ack(cmd::RESET_TRIP, false);
                }
                self.odo.trip_um[idx as usize] = 0;
                let ok = self.save_odo();
                self.next_odometer = 0;
                ack(cmd::RESET_TRIP, ok)
            }
            HostMsg::InitOdometer { odo_m } => {
                let allowed = matches!(self.odo_status, OdoStatus::Blank | OdoStatus::Corrupt)
                    && !self.odo_initialised;
                if !allowed {
                    return ack(cmd::INIT_ODOMETER, false);
                }
                self.odo.odo_um = odo_m as u64 * 1_000_000;
                self.odo_initialised = true;
                self.odo_status = OdoStatus::Ok;
                let ok = self.save_odo();
                self.next_odometer = 0;
                ack(cmd::INIT_ODOMETER, ok)
            }
            HostMsg::StoreSettings(s) => {
                self.settings = Some(s);
                self.settings_seq = self.settings_seq.wrapping_add(1);
                let ok = nvm::save(
                    &mut self.store,
                    SETTINGS_SLOTS,
                    self.settings_seq,
                    &settings_to_bytes(&s),
                )
                .is_ok();
                Some(McuMsg::Ack {
                    cmd: cmd::STORE_SETTINGS,
                    status: if ok { status::OK } else { status::FRAM_ERROR },
                })
            }
            HostMsg::EnterBootloader => {
                self.save_odo();
                self.bootloader = true;
                None
            }
        }
    }

    pub fn poll(&mut self, now: u64) -> Poll {
        let mut out = Poll::default();
        if let Some(hb) = self.last_heartbeat
            && now.saturating_sub(hb) > HEARTBEAT_TIMEOUT_US
            && self.fade.to != 0
        {
            // fail-dark: a frozen UI must not keep showing stale numbers
            self.set_backlight(0, 0, now);
        }
        if self.host_seen && now >= self.next_telemetry {
            self.next_telemetry = now + TELEMETRY_PERIOD_US;
            out.telemetry = Some(self.telemetry(now));
        }
        if self.host_seen && now >= self.next_odometer {
            self.next_odometer = now + ODOMETER_PERIOD_US;
            out.odometer = Some(self.odometer_msg());
        }
        out.backlight_permille = self.fade.value(now);
        out
    }

    fn set_backlight(&mut self, to: u16, dur: u64, now: u64) {
        let from = self.fade.value(now);
        self.fade = Fade {
            from,
            to,
            t0: now,
            dur,
        };
    }

    fn telemetry(&mut self, now: u64) -> Telemetry {
        self.seq = self.seq.wrapping_add(1);
        let mut flags = 0;
        if self.fram_ok {
            flags |= flag::FRAM_OK;
        }
        if self.power_low {
            flags |= flag::POWER_LOW;
        }
        if self.odo_status == OdoStatus::Ok {
            flags |= flag::ODO_INITIALISED;
        }
        Telemetry {
            seq: self.seq,
            uptime_us: now,
            wheel: self.wheel.stats(now),
            ign: self.ign.stats(now),
            vbat_raw: self.adc[0],
            coolant_raw: self.adc[1],
            light_raw: self.adc[2],
            inputs: self.inputs,
            flags,
        }
    }

    fn odometer_msg(&self) -> Odometer {
        Odometer {
            odo_m: self.odo.odo_m(),
            trip_m: [self.odo.trip_m(0), self.odo.trip_m(1)],
        }
    }

    fn hello(&self, now: u64) -> tdm_proto::Hello {
        tdm_proto::Hello {
            proto: PROTO_VERSION,
            fw: FW_VERSION,
            board_rev: 1,
            uptime_us: now,
            reset_reason: self.reset_reason,
            config: self.cfg,
            settings: self.settings,
            odo_status: self.odo_status,
            first_backlight_ms: self.first_backlight_ms,
        }
    }

    /// Persists the odometer unless the stored record is corrupt (it is then
    /// kept for inspection until an explicit `InitOdometer`).
    fn save_odo(&mut self) -> bool {
        if matches!(self.odo_status, OdoStatus::Corrupt | OdoStatus::Missing) {
            return false;
        }
        self.odo_seq = self.odo_seq.wrapping_add(1);
        let ok = nvm::save(
            &mut self.store,
            ODO_SLOTS,
            self.odo_seq,
            &odo_to_bytes(&self.odo, self.odo_initialised),
        )
        .is_ok();
        if ok {
            self.odo_saved_um = self.odo.odo_um;
            if self.odo_status == OdoStatus::Blank {
                // counting from zero on a fresh FRAM is still a valid record
                self.odo_status = OdoStatus::Ok;
            }
        }
        self.fram_ok &= ok;
        ok
    }
}

// ------------------------------------------------------------ record payloads

fn odo_to_bytes(o: &OdoState, initialised: bool) -> [u8; PAYLOAD_LEN] {
    let mut p = [0u8; PAYLOAD_LEN];
    p[0..8].copy_from_slice(&o.odo_um.to_le_bytes());
    p[8..16].copy_from_slice(&o.trip_um[0].to_le_bytes());
    p[16..24].copy_from_slice(&o.trip_um[1].to_le_bytes());
    p[24] = initialised as u8;
    p
}

fn odo_from_bytes(p: &[u8; PAYLOAD_LEN]) -> (OdoState, bool) {
    let u = |i: usize| u64::from_le_bytes(p[i..i + 8].try_into().unwrap());
    (
        OdoState {
            odo_um: u(0),
            trip_um: [u(8), u(16)],
        },
        p[24] != 0,
    )
}

fn config_to_bytes(c: &McuConfig) -> [u8; PAYLOAD_LEN] {
    let mut p = [0u8; PAYLOAD_LEN];
    p[0..4].copy_from_slice(&c.um_per_pulse.to_le_bytes());
    p[4] = c.wheel_ppr;
    p[5..9].copy_from_slice(&c.wheel_min_gap_us.to_le_bytes());
    p[9..13].copy_from_slice(&c.wheel_min_window_us.to_le_bytes());
    p[13..17].copy_from_slice(&c.ign_min_gap_us.to_le_bytes());
    p[17] = c.ign_blank_pct;
    p[18..22].copy_from_slice(&c.ign_min_window_us.to_le_bytes());
    p[22] = c.debounce_ms;
    p
}

fn config_from_bytes(p: &[u8; PAYLOAD_LEN]) -> McuConfig {
    let u = |i: usize| u32::from_le_bytes(p[i..i + 4].try_into().unwrap());
    McuConfig {
        um_per_pulse: u(0),
        wheel_ppr: p[4],
        wheel_min_gap_us: u(5),
        wheel_min_window_us: u(9),
        ign_min_gap_us: u(13),
        ign_blank_pct: p[17],
        ign_min_window_us: u(18),
        debounce_ms: p[22],
    }
}

fn settings_to_bytes(s: &UserSettings) -> [u8; PAYLOAD_LEN] {
    let mut p = [0u8; PAYLOAD_LEN];
    p[0] = s.theme;
    p[1] = s.layout;
    p[2] = s.brightness_offset as u8;
    p[3] = s.diag as u8;
    p
}

fn settings_from_bytes(p: &[u8; PAYLOAD_LEN]) -> UserSettings {
    UserSettings {
        theme: p[0],
        layout: p[1],
        brightness_offset: p[2] as i8,
        diag: p[3] != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::MemNvm;

    type Core = McuCore<MemNvm<4096>>;

    fn drive_km(core: &mut Core, t: &mut u64, km: f64) {
        let pulses = (km * 1e9 / core.config().um_per_pulse as f64) as u64;
        for _ in 0..pulses {
            *t += 10_000;
            core.on_wheel_edge(*t);
        }
    }

    #[test]
    fn silent_until_host_speaks() {
        let mut c = Core::new(MemNvm::new(), 0);
        assert_eq!(c.poll(1_000_000).telemetry, None);
        let hello = c.on_host(HostMsg::GetInfo, 1_000_000);
        assert!(
            matches!(hello, Some(McuMsg::Hello(h)) if h.proto == PROTO_VERSION && h.odo_status == OdoStatus::Blank)
        );
        assert!(c.poll(1_000_001).telemetry.is_some());
        c.on_host_lost();
        assert!(c.poll(2_000_000).telemetry.is_none());
    }

    #[test]
    fn odometer_survives_power_cycles_and_never_goes_back() {
        let mut t = 0u64;
        let mut c = Core::new(MemNvm::new(), 0);
        drive_km(&mut c, &mut t, 1.234);
        let before = c.odometer().odo_um;
        // power cut without warning: at most 10 m lost
        let nvm = c.store().clone();
        let c2 = Core::new(nvm, 0);
        let after = c2.odometer().odo_um;
        assert!(
            after <= before && before - after <= ODO_SAVE_EVERY_UM + c.config().um_per_pulse as u64
        );
        // power fail warning: nothing lost
        let mut c3 = c2;
        drive_km(&mut c3, &mut t, 0.5);
        c3.on_power_fail();
        let c4 = Core::new(c3.store().clone(), 0);
        assert_eq!(c4.odometer(), c3.odometer());
    }

    #[test]
    fn random_power_cuts_with_torn_writes() {
        // Each cycle drives a random distance, then power fails and the
        // power-fail save itself is torn at a random byte. The odometer must
        // never go back and must lose at most one save interval.
        let mut t = 0u64;
        let mut nvm = MemNvm::<4096>::new();
        let mut prev = 0u64;
        let mut rng = 0x1234_5678u32;
        let pulse = DEFAULT_CONFIG.um_per_pulse as u64;
        for _ in 0..50 {
            let mut c = Core::new(nvm.clone(), 0);
            assert_ne!(c.odo_status(), OdoStatus::Corrupt);
            let start = c.odometer().odo_um;
            assert!(start >= prev, "odometer went back: {start} < {prev}");
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            drive_km(&mut c, &mut t, 0.05 + (rng % 100) as f64 / 1000.0);
            let reached = c.odometer().odo_um;
            c.store_mut().tear_after = Some((rng % 41) as usize);
            c.on_power_fail();
            nvm = c.store().clone();
            nvm.tear_after = None;
            let reloaded = Core::new(nvm.clone(), 0).odometer().odo_um;
            assert!(reloaded <= reached);
            assert!(
                reached - reloaded <= ODO_SAVE_EVERY_UM + pulse,
                "lost {} um",
                reached - reloaded
            );
            prev = reloaded;
        }
    }

    #[test]
    fn init_odometer_only_once_on_blank() {
        let mut c = Core::new(MemNvm::new(), 0);
        let r = c.on_host(HostMsg::InitOdometer { odo_m: 48_213_000 }, 0);
        assert_eq!(
            r,
            Some(McuMsg::Ack {
                cmd: cmd::INIT_ODOMETER,
                status: status::OK
            })
        );
        assert_eq!(c.odometer().odo_m(), 48_213_000);
        let r = c.on_host(HostMsg::InitOdometer { odo_m: 1 }, 0);
        assert_eq!(
            r,
            Some(McuMsg::Ack {
                cmd: cmd::INIT_ODOMETER,
                status: status::REFUSED
            })
        );
        // and not after a reboot either
        let mut c2 = Core::new(c.store().clone(), 0);
        let r = c2.on_host(HostMsg::InitOdometer { odo_m: 1 }, 0);
        assert_eq!(
            r,
            Some(McuMsg::Ack {
                cmd: cmd::INIT_ODOMETER,
                status: status::REFUSED
            })
        );
    }

    #[test]
    fn corrupt_record_is_never_overwritten_by_driving() {
        let mut nvm = MemNvm::<4096>::new();
        nvm.data[0x40..0x48].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let snapshot = nvm.data;
        let mut c = Core::new(nvm, 0);
        assert_eq!(c.odo_status(), OdoStatus::Corrupt);
        let mut t = 0;
        drive_km(&mut c, &mut t, 0.5);
        assert_eq!(c.store().data, snapshot);
    }

    #[test]
    fn heartbeat_loss_turns_backlight_off() {
        let mut c = Core::new(MemNvm::new(), 0);
        c.on_host(
            HostMsg::SetBacklight {
                permille: 800,
                fade_ms: 300,
            },
            1_000_000,
        );
        assert_eq!(c.poll(1_150_000).backlight_permille, 400);
        assert_eq!(c.poll(1_400_000).backlight_permille, 800);
        c.on_host(HostMsg::Heartbeat { ui_frame: 1 }, 2_000_000);
        assert_eq!(c.poll(3_900_000).backlight_permille, 800);
        assert_eq!(c.poll(4_100_000).backlight_permille, 0);
        // UI back: bright again on the next SetBacklight
        c.on_host(
            HostMsg::SetBacklight {
                permille: 800,
                fade_ms: 0,
            },
            5_000_000,
        );
        assert_eq!(c.poll(5_000_001).backlight_permille, 800);
    }

    #[test]
    fn config_persists_and_applies() {
        let mut c = Core::new(MemNvm::new(), 0);
        let mut cfg = DEFAULT_CONFIG;
        cfg.um_per_pulse = 400_000;
        cfg.wheel_ppr = 5;
        assert_eq!(
            c.on_host(HostMsg::SetConfig(cfg), 0),
            Some(McuMsg::Ack {
                cmd: cmd::SET_CONFIG,
                status: status::OK
            })
        );
        let c2 = Core::new(c.store().clone(), 0);
        assert_eq!(c2.config(), cfg);
        let mut bad = cfg;
        bad.wheel_ppr = 0;
        assert_eq!(
            c.on_host(HostMsg::SetConfig(bad), 0),
            Some(McuMsg::Ack {
                cmd: cmd::SET_CONFIG,
                status: status::REFUSED
            })
        );
    }

    #[test]
    fn trip_reset_is_persisted() {
        let mut c = Core::new(MemNvm::new(), 0);
        let mut t = 0;
        drive_km(&mut c, &mut t, 0.3);
        c.on_host(HostMsg::ResetTrip { idx: 1 }, t);
        let c2 = Core::new(c.store().clone(), 0);
        assert_eq!(c2.odometer().trip_um[1], 0);
        assert!(c2.odometer().trip_um[0] > 250_000_000);
    }
}
