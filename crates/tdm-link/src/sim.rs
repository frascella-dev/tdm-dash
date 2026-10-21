//! Simulated motorcycle and co-processor. The vehicle model produces wheel
//! and ignition edges, inputs and ADC readings; they go through the same
//! `tdm_core::McuCore` that runs on the RP2040, so the dashboard sees exactly
//! the messages real hardware would send. Scenarios mirror the ones in the
//! design page (`design/index.html`).

use tdm_core::{McuCore, MemNvm};
use tdm_logic::Profile;
use tdm_logic::conv::ohm_from_temp;
use tdm_proto::{HostMsg, McuMsg, input};

/// Approximate 3VD gearbox: primary × final with the 150/70-17 rear tyre.
const GEARS: [f64; 5] = [2.571, 1.778, 1.381, 1.125, 0.962];
const RPM_PER_MS: f64 = (60.0 / 1.98) * 1.708 * 2.8125;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    pub kmh: f64,
    pub accel: f64,
    pub decel: f64,
    pub hold: f64,
    pub turn: Option<Turn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Battery {
    Ok,
    /// Stator not charging: voltage sinks with the engine running.
    NotCharging,
    /// Shunt regulator failed open: voltage climbs.
    Overcharging,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    pub upshift_rpm: f64,
    pub downshift_rpm: f64,
    pub coolant_moving: f64,
    pub coolant_stopped: f64,
    pub coolant_start: Option<f64>,
    pub coolant_rate: f64,
    pub battery: Battery,
    /// Trip B (since refuelling) at start, km.
    pub trip_b_km: Option<f64>,
    /// USB drop: every `.0` seconds for `.1` seconds.
    pub link_drop: Option<(f64, f64)>,
    pub legs: Vec<Leg>,
}

const fn leg(kmh: f64, accel: f64, decel: f64, hold: f64, turn: Option<Turn>) -> Leg {
    Leg {
        kmh,
        accel,
        decel,
        hold,
        turn,
    }
}

pub fn scenarios() -> Vec<Scenario> {
    use Turn::*;
    let base = Scenario {
        id: "",
        name: "",
        note: "",
        upshift_rpm: 4300.0,
        downshift_rpm: 2300.0,
        coolant_moving: 88.0,
        coolant_stopped: 97.0,
        coolant_start: None,
        coolant_rate: 0.08,
        battery: Battery::Ok,
        trip_b_km: None,
        link_drop: None,
        legs: vec![],
    };
    vec![
        Scenario {
            id: "city",
            name: "City",
            note: "Traffic lights, neutral when stopped, turn signals at corners.",
            legs: vec![
                leg(0.0, 2.0, 3.0, 5.0, None),
                leg(48.0, 2.2, 3.0, 7.0, None),
                leg(26.0, 2.0, 3.0, 2.0, Some(Right)),
                leg(46.0, 2.0, 3.0, 6.0, None),
                leg(0.0, 2.0, 3.2, 6.0, None),
                leg(38.0, 2.4, 3.0, 3.0, Some(Left)),
                leg(52.0, 1.8, 3.0, 8.0, None),
                leg(0.0, 2.0, 3.0, 4.0, None),
            ],
            ..base.clone()
        },
        Scenario {
            id: "mountain",
            name: "Mountain pass",
            note: "Pulls up to the red zone: shift light above 7500.",
            upshift_rpm: 7900.0,
            downshift_rpm: 4800.0,
            coolant_moving: 92.0,
            coolant_stopped: 98.0,
            legs: vec![
                leg(0.0, 2.0, 3.0, 2.0, None),
                leg(95.0, 4.2, 3.0, 2.0, None),
                leg(42.0, 2.0, 6.0, 2.0, None),
                leg(105.0, 4.4, 3.0, 1.5, None),
                leg(38.0, 2.0, 6.5, 2.0, Some(Left)),
                leg(88.0, 4.2, 3.0, 2.0, None),
                leg(50.0, 2.0, 5.5, 1.5, None),
                leg(112.0, 4.2, 3.0, 2.0, None),
                leg(0.0, 2.0, 5.0, 3.0, None),
            ],
            ..base.clone()
        },
        Scenario {
            id: "motorway",
            name: "Motorway",
            note: "Cruising at 130 with overtakes; the estimated range drops below the reserve.",
            upshift_rpm: 5500.0,
            downshift_rpm: 3000.0,
            coolant_moving: 84.0,
            coolant_stopped: 92.0,
            trip_b_km: Some(240.5),
            legs: vec![
                leg(0.0, 2.0, 3.0, 2.0, None),
                leg(130.0, 2.8, 3.0, 14.0, None),
                leg(148.0, 1.6, 3.0, 5.0, Some(Left)),
                leg(130.0, 2.0, 1.2, 16.0, Some(Right)),
            ],
            ..base.clone()
        },
        Scenario {
            id: "warmup",
            name: "Cold warm-up",
            note: "Idling after a night outdoors: the coolant starts at 14 °C.",
            coolant_moving: 82.0,
            coolant_stopped: 82.0,
            coolant_start: Some(14.0),
            coolant_rate: 0.9,
            legs: vec![leg(0.0, 2.0, 3.0, 1e9, None)],
            ..base.clone()
        },
        Scenario {
            id: "nocharge",
            name: "Charging fault",
            note: "The stator does not charge: first \"not charging\", then \"battery low\".",
            upshift_rpm: 5200.0,
            downshift_rpm: 3100.0,
            battery: Battery::NotCharging,
            legs: vec![
                leg(0.0, 2.0, 3.0, 2.0, None),
                leg(80.0, 2.5, 3.0, 20.0, None),
                leg(60.0, 2.0, 2.0, 10.0, None),
                leg(85.0, 2.0, 3.0, 20.0, None),
            ],
            ..base.clone()
        },
        Scenario {
            id: "regulator",
            name: "Faulty regulator",
            note: "The regulator no longer limits: above 15 V, then 15.5 V.",
            upshift_rpm: 5200.0,
            downshift_rpm: 3000.0,
            battery: Battery::Overcharging,
            legs: vec![
                leg(0.0, 2.0, 3.0, 2.0, None),
                leg(90.0, 2.5, 3.0, 30.0, None),
                leg(70.0, 2.0, 2.0, 15.0, None),
            ],
            ..base.clone()
        },
        Scenario {
            id: "overheat",
            name: "Summer traffic jam",
            note: "Queuing in the sun: the coolant passes 105 °C, then 112 °C.",
            upshift_rpm: 3800.0,
            downshift_rpm: 2100.0,
            coolant_moving: 99.0,
            coolant_stopped: 117.0,
            coolant_rate: 0.45,
            legs: vec![
                leg(0.0, 2.0, 3.0, 8.0, None),
                leg(12.0, 1.2, 3.0, 3.0, None),
                leg(0.0, 2.0, 2.0, 9.0, None),
                leg(9.0, 1.0, 3.0, 2.0, None),
                leg(0.0, 2.0, 2.0, 10.0, None),
            ],
            ..base.clone()
        },
        Scenario {
            id: "linkdrop",
            name: "Unstable link",
            note: "Every 20 s the USB cable drops out for 4 s.",
            upshift_rpm: 4500.0,
            downshift_rpm: 2400.0,
            link_drop: Some((20.0, 4.0)),
            legs: vec![
                leg(0.0, 2.0, 3.0, 3.0, None),
                leg(60.0, 2.4, 3.0, 12.0, None),
                leg(30.0, 2.0, 3.0, 4.0, None),
                leg(70.0, 2.2, 3.0, 10.0, None),
                leg(0.0, 2.0, 3.0, 4.0, None),
            ],
            ..base
        },
    ]
}

pub fn scenario(id: &str) -> Option<Scenario> {
    scenarios().into_iter().find(|s| s.id == id)
}

/// Small deterministic RNG so simulations and tests are reproducible.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    pub fn unit(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn gauss(&mut self) -> f64 {
        let u = self.unit().max(1e-12);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

/// Inputs the user can drive by hand in the desktop simulator.
#[derive(Debug, Clone, Copy, Default)]
pub struct Manual {
    pub mode: bool,
    pub set: bool,
    pub high_beam: bool,
    /// Turn signal forced until this sim time.
    pub turn: Option<(Turn, f64)>,
    /// Ambient light in lux.
    pub lux: f64,
}

#[derive(Debug)]
pub struct SimMcu {
    pub scenario: Scenario,
    core: McuCore<MemNvm<4096>>,
    rng: Rng,
    t: f64,
    scn_t: f64,
    leg: usize,
    leg_hold: f64,
    v: f64,
    gear: usize,
    stop_t: f64,
    rpm: f64,
    shift_t: f64,
    coolant: f64,
    vbat: f64,
    drift: f64,
    dist_mm: f64,
    next_edge_mm: f64,
    bolt: usize,
    bolt_err: Vec<f64>,
    ign_phase: f64,
    wheel_mm: f64,
    ppr: usize,
    curve: Vec<(f64, f64)>,
    pullup: f64,
    divider: f64,
    pub manual: Manual,
    link_cut_until: f64,
}

impl SimMcu {
    pub fn new(profile: &Profile, scenario: Scenario, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let ppr = profile.speed.pulses_per_rev as usize;
        let bolt_err = (0..ppr).map(|_| (rng.unit() - 0.5) * 0.04).collect();
        // start from a realistic odometer, written through the core itself
        let mut core = McuCore::new(MemNvm::<4096>::new(), 0);
        core.on_host(HostMsg::InitOdometer { odo_m: 48_213_400 }, 0);
        core.set_trips_for_simulation([
            212_400_000_000,
            (scenario.trip_b_km.unwrap_or(214.0) * 1e9) as u64,
        ]);
        Self {
            core,
            rng,
            t: 0.0,
            scn_t: 0.0,
            leg: 0,
            leg_hold: 0.0,
            v: 0.0,
            gear: 0,
            stop_t: 2.0,
            rpm: 1200.0,
            shift_t: 0.0,
            coolant: scenario.coolant_start.unwrap_or(scenario.coolant_moving),
            vbat: 12.7,
            drift: 0.0,
            dist_mm: 0.0,
            next_edge_mm: 0.0,
            bolt: 0,
            bolt_err,
            ign_phase: 0.0,
            wheel_mm: profile.speed.wheel_circumference_mm,
            ppr,
            curve: profile.coolant.curve.clone(),
            pullup: profile.board.coolant_pullup_ohm,
            divider: profile.board.vbat_divider,
            manual: Manual {
                lux: 3000.0,
                ..Manual::default()
            },
            link_cut_until: 0.0,
            scenario,
        }
    }

    pub fn set_scenario(&mut self, sc: Scenario) {
        self.leg = 0;
        self.leg_hold = 0.0;
        self.scn_t = 0.0;
        if let Some(c) = sc.coolant_start {
            self.coolant = c;
        }
        self.drift = 0.0;
        if let Some(b) = sc.trip_b_km {
            let mut t = self.core.odometer().trip_um;
            t[1] = (b * 1e9) as u64;
            self.core.set_trips_for_simulation(t);
        }
        self.scenario = sc;
    }

    pub fn cut_link(&mut self, seconds: f64) {
        self.link_cut_until = self.t + seconds;
    }

    pub fn now(&self) -> f64 {
        self.t
    }

    pub fn true_kmh(&self) -> f64 {
        self.v * 3.6
    }

    pub fn backlight_permille(&mut self) -> u16 {
        self.core.poll(self.now_us()).backlight_permille
    }

    fn now_us(&self) -> u64 {
        (self.t * 1e6) as u64
    }

    /// True when the scenario or the user has cut the USB link.
    pub fn link_down(&self) -> bool {
        let scripted = self
            .scenario
            .link_drop
            .is_some_and(|(every, dur)| self.scn_t % every > every - dur);
        scripted || self.t < self.link_cut_until
    }

    /// Delivers a host message; returns the reply if any (none while the
    /// link is down).
    pub fn send(&mut self, msg: HostMsg) -> Option<McuMsg> {
        if self.link_down() {
            return None;
        }
        let now = self.now_us();
        self.core.on_host(msg, now)
    }

    /// Advances the simulation by `dt` seconds and returns the messages the
    /// MCU sent in that time.
    pub fn step(&mut self, dt: f64) -> Vec<McuMsg> {
        let mut out = Vec::new();
        let steps = (dt / 0.002).ceil().max(1.0) as usize;
        let h = dt / steps as f64;
        for _ in 0..steps {
            self.step_vehicle(h);
            self.t += h;
            self.emit_edges(h);
            self.update_inputs(h);
            let now = self.now_us();
            let p = self.core.poll(now);
            if !self.link_down() {
                out.extend(p.telemetry.map(McuMsg::Telemetry));
                out.extend(p.odometer.map(McuMsg::Odometer));
            }
        }
        out
    }

    fn step_vehicle(&mut self, dt: f64) {
        let sc = &self.scenario;
        self.scn_t += dt;
        let leg = sc.legs[self.leg];
        let target = leg.kmh / 3.6;
        let accel = leg.accel * (1.0 - self.v / 80.0);
        if self.v < target - 0.05 {
            self.v = (self.v + accel * dt).min(target);
        } else if self.v > target + 0.05 {
            self.v = (self.v - leg.decel * dt).max(target);
        }
        if (self.v - target).abs() < 0.15 {
            self.leg_hold += dt;
            if self.leg_hold >= leg.hold {
                self.leg = (self.leg + 1) % sc.legs.len();
                self.leg_hold = 0.0;
            }
        }
        let kmh = self.v * 3.6;
        if self.v < 0.3 {
            self.stop_t += dt;
            if self.stop_t > 1.5 {
                self.gear = 0;
            }
        } else {
            self.stop_t = 0.0;
            if self.gear == 0 {
                self.gear = 1;
            }
        }
        if self.gear > 0 {
            let r = self.v * RPM_PER_MS * GEARS[self.gear - 1];
            if r > sc.upshift_rpm && self.gear < 5 {
                self.gear += 1;
                self.shift_t = 0.25;
            } else if r < sc.downshift_rpm && self.gear > 1 && kmh > 8.0 {
                self.gear -= 1;
                self.shift_t = 0.2;
            }
        }
        let rpm = if self.gear == 0 {
            1200.0 + 40.0 * (self.scn_t * 2.1).sin()
        } else {
            let mut r = self.v * RPM_PER_MS * GEARS[self.gear - 1];
            if self.gear == 1 && kmh < 14.0 {
                r = r.max(if self.v < target - 0.2 {
                    2600.0
                } else {
                    1250.0
                });
            }
            r.max(1150.0)
        };
        self.shift_t = (self.shift_t - dt).max(0.0);
        let tau = if self.shift_t > 0.0 { 0.09 } else { 0.05 };
        self.rpm += (rpm - self.rpm) * (1.0 - (-dt / tau).exp());

        let ct = if kmh < 5.0 {
            sc.coolant_stopped
        } else {
            sc.coolant_moving
        };
        self.coolant += (ct - self.coolant).clamp(-sc.coolant_rate * 2.0, sc.coolant_rate) * dt;

        let charging = if self.rpm > 1800.0 { 14.2 } else { 12.85 };
        let vt = match sc.battery {
            Battery::Ok => charging,
            Battery::NotCharging => {
                if self.rpm > 2200.0 {
                    self.drift += 0.022 * dt;
                }
                12.7 - self.drift
            }
            Battery::Overcharging => {
                self.drift = (self.drift + 0.05 * dt).min(2.2);
                charging + self.drift
            }
        };
        self.vbat += (vt - self.vbat) * (1.0 - (-dt / 1.2).exp());
    }

    fn emit_edges(&mut self, dt: f64) {
        let t0 = self.t - dt;
        let d0 = self.dist_mm;
        let d1 = d0 + self.v * dt * 1000.0;
        let mm_per_pulse = self.wheel_mm / self.ppr as f64;
        while self.next_edge_mm <= d1 {
            let frac = if d1 > d0 {
                ((self.next_edge_mm - d0) / (d1 - d0)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let te = (t0 + frac * dt) * 1e6 + self.rng.gauss() * 30.0;
            self.core.on_wheel_edge(te.max(0.0) as u64);
            self.bolt = (self.bolt + 1) % self.ppr;
            self.next_edge_mm += mm_per_pulse * (1.0 + self.bolt_err[self.bolt]);
        }
        self.dist_mm = d1;
        let p0 = self.ign_phase;
        let p1 = p0 + self.rpm / 60.0 * dt;
        let mut k = p0.floor() + 1.0;
        while k <= p1 {
            let te = (t0 + (k - p0) / (p1 - p0) * dt) * 1e6;
            self.core.on_ign_edge(te as u64);
            if self.rng.unit() < 0.3 {
                self.core.on_ign_edge(te as u64 + 400); // coil ringing
            }
            k += 1.0;
        }
        self.ign_phase = p1;
    }

    fn update_inputs(&mut self, dt: f64) {
        let leg = self.scenario.legs[self.leg];
        let flash = (self.t % 0.7) < 0.36; // ≈ 85 flashes per minute
        let manual_turn = self
            .manual
            .turn
            .filter(|(_, until)| *until > self.t)
            .map(|(d, _)| d);
        let turn = manual_turn.or(leg.turn);
        let mut w = input::ILLUMINATION;
        if self.gear == 0 {
            w |= input::NEUTRAL;
        }
        if self.manual.high_beam {
            w |= input::HIGH_BEAM;
        }
        match turn {
            Some(Turn::Left) if flash => w |= input::TURN_LEFT,
            Some(Turn::Right) if flash => w |= input::TURN_RIGHT,
            _ => {}
        }
        if self.manual.mode {
            w |= input::BTN_MODE;
        }
        if self.manual.set {
            w |= input::BTN_SET;
        }
        self.core
            .on_inputs(w, (dt * 1000.0).round().max(1.0) as u16);
        let r = ohm_from_temp(&self.curve, self.coolant);
        let coolant =
            (4095.0 * r / (r + self.pullup) + self.rng.gauss() * 3.0).clamp(0.0, 4095.0) as u16;
        let vbat = ((self.vbat + self.rng.gauss() * 0.02) / self.divider / 3.3 * 4095.0)
            .clamp(0.0, 4095.0) as u16;
        let light = (self.manual.lux * 0.8 + 6.0).clamp(0.0, 4095.0) as u16;
        self.core.on_adc(vbat, coolant, light);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdm_logic::{Dash, LinkEvent};

    fn run_dash(sc: &str, secs: f64) -> (Dash, SimMcu, tdm_logic::ViewModel) {
        let p = Profile::builtin();
        let mut sim = SimMcu::new(&p, scenario(sc).unwrap(), 7);
        let mut d = Dash::new(p, None);
        d.set_language(tdm_logic::Language::En);
        d.on_event(LinkEvent::Up("sim".into()), 0.0);
        let mut vm = d.tick(0.0);
        let dt = 1.0 / 60.0;
        let mut t = 0.0;
        while t < secs {
            t += dt;
            for m in sim.step(dt) {
                d.on_event(LinkEvent::Msg(m), t);
            }
            vm = d.tick(t);
            d.first_frame_shown(t);
            for c in d.take_commands() {
                if let Some(r) = sim.send(c) {
                    d.on_event(LinkEvent::Msg(r), t);
                }
            }
        }
        (d, sim, vm)
    }

    #[test]
    fn city_ride_end_to_end() {
        let (_d, mut sim, vm) = run_dash("city", 20.0);
        assert_eq!(vm.state, tdm_logic::UiState::Ride);
        let shown: f64 = vm.speed_text.parse().unwrap();
        let tru = sim.true_kmh();
        assert!(
            shown >= tru - 1.0 && shown <= tru * 1.1 + 5.0,
            "shown {shown} true {tru}"
        );
        assert!(
            sim.backlight_permille() > 0,
            "UI heartbeat keeps the backlight on"
        );
        assert!(vm.odo_text.starts_with("48,2"));
    }

    #[test]
    fn overheat_raises_warnings_in_order() {
        let (d, _sim, vm) = run_dash("overheat", 60.0);
        let _ = d;
        let w = vm.warning.expect("a warning");
        assert!(matches!(
            w.id,
            tdm_logic::WarningId::CoolantCritical | tdm_logic::WarningId::CoolantHigh
        ));
    }

    #[test]
    fn regulator_failure_is_critical() {
        let (_d, _sim, vm) = run_dash("regulator", 45.0);
        assert_eq!(
            vm.warning.map(|w| w.id),
            Some(tdm_logic::WarningId::Overvoltage)
        );
    }

    #[test]
    fn simulated_distance_matches_odometer() {
        let p = Profile::builtin();
        let mut sim = SimMcu::new(&p, scenario("motorway").unwrap(), 1);
        let start = sim.core.odometer().odo_um;
        let mut true_mm = 0.0;
        for _ in 0..(60 * 30) {
            sim.step(1.0 / 60.0);
            true_mm += sim.v / 60.0 * 1000.0;
        }
        let measured_mm = (sim.core.odometer().odo_um - start) as f64 / 1000.0;
        assert!(
            (measured_mm - true_mm).abs() / true_mm < 0.01,
            "{measured_mm} vs {true_mm}"
        );
    }
}
