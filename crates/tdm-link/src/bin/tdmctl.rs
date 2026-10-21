//! `tdmctl`: talk to the tdm-dash co-processor from a PC or from the Pi's
//! development image.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use tdm_link::{SerialSource, Source, find_ports, scenario, scenarios, sim::SimMcu};
use tdm_logic::{Dash, Language, LinkEvent, Profile, calib, conv, fmt};
use tdm_proto::{HostMsg, McuMsg};

/// The command-line tool always speaks English.
const EN: Language = Language::En;

const USAGE: &str = "\
tdmctl — tool for the tdm-dash sensor module

USAGE:
  tdmctl list                         list the modules connected over USB
  tdmctl monitor [--port DEV] [-n S]  show the telemetry (for S seconds, default forever)
  tdmctl record FILE [--port DEV]     record the traffic for --source replay:FILE
  tdmctl reset-trip a|b [--port DEV]  reset a trip
  tdmctl init-odo KM [--port DEV]     set the odometer (blank FRAM only)
  tdmctl bootsel [--port DEV]         restart the Pico into its USB bootloader
  tdmctl sim [SCENARIO] [-n S]        simulate without hardware, print speed and rpm
  tdmctl scenarios                    list the simulator scenarios

CALIBRATION (manual, chapter 10; --profile FILE = the profile in use):
  tdmctl calibrate wheel KM_GPS KM_TRIP           wheel circumference
  tdmctl calibrate tach TRUE_RPM SHOWN_RPM        tachometer pulses per revolution
  tdmctl calibrate battery V_METER V_SHOWN        battery divider ratio
  tdmctl calibrate coolant FILE                   coolant sender curve (\"°C ohm\" per line)
";

struct Args {
    cmd: String,
    pos: Vec<String>,
    port: Option<PathBuf>,
    seconds: Option<f64>,
    profile: Option<PathBuf>,
}

fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().ok_or("missing command")?;
    let mut a = Args {
        cmd,
        pos: vec![],
        port: None,
        seconds: None,
        profile: None,
    };
    while let Some(x) = it.next() {
        match x.as_str() {
            "--port" => a.port = Some(it.next().ok_or("--port needs a device")?.into()),
            "-n" | "--seconds" => {
                a.seconds = Some(
                    it.next()
                        .ok_or("-n needs a number")?
                        .parse()
                        .map_err(|_| "-n needs a number")?,
                )
            }
            "--profile" => a.profile = Some(it.next().ok_or("--profile needs a file")?.into()),
            "-h" | "--help" => return Err(String::new()),
            _ => a.pos.push(x),
        }
    }
    Ok(a)
}

fn main() -> ExitCode {
    let a = match parse() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("error: {e}\n");
            }
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let profile = match &a.profile {
        Some(p) => match Profile::load(p) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => Profile::builtin(),
    };
    let r = match a.cmd.as_str() {
        "list" => {
            let ports = find_ports();
            if ports.is_empty() {
                println!(
                    "no module found (VID:PID {:04x}:{:04x})",
                    tdm_proto::USB_VID,
                    tdm_proto::USB_PID
                );
            }
            for p in ports {
                println!("{}", p.display());
            }
            Ok(())
        }
        "monitor" => monitor(&a, &profile, None),
        "record" => match a.pos.first() {
            Some(f) => monitor(&a, &profile, Some(f.into())),
            None => Err("record needs a file name".into()),
        },
        "reset-trip" => match a.pos.first().map(String::as_str) {
            Some("a" | "A") => command(&a, HostMsg::ResetTrip { idx: 0 }),
            Some("b" | "B") => command(&a, HostMsg::ResetTrip { idx: 1 }),
            _ => Err("reset-trip needs a or b".into()),
        },
        "init-odo" => match a
            .pos
            .first()
            .and_then(|k| k.replace(',', ".").parse::<f64>().ok())
        {
            Some(km) if km >= 0.0 => command(
                &a,
                HostMsg::InitOdometer {
                    odo_m: (km * 1000.0).round() as u32,
                },
            ),
            _ => Err("init-odo needs the km, for example 48213".into()),
        },
        "bootsel" => command(&a, HostMsg::EnterBootloader),
        "scenarios" => {
            for s in scenarios() {
                println!("{:<10} {} — {}", s.id, s.name, s.note);
            }
            Ok(())
        }
        "sim" => simulate(&a, profile),
        "calibrate" => calibrate(&a, &profile),
        other => Err(format!("unknown command '{other}'")),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn wait_hello(src: &mut SerialSource, t0: Instant) -> Result<(), String> {
    let mut next = 0.0;
    while t0.elapsed() < Duration::from_secs(5) {
        let now = t0.elapsed().as_secs_f64();
        if now >= next {
            src.send(HostMsg::GetInfo);
            next = now + 0.3;
        }
        for e in src.poll(now) {
            if let LinkEvent::Msg(McuMsg::Hello(h)) = e {
                println!(
                    "module: firmware {}.{}.{}, protocol {}, FRAM {:?}, up {} s",
                    h.fw[0],
                    h.fw[1],
                    h.fw[2],
                    h.proto,
                    h.odo_status,
                    h.uptime_us / 1_000_000
                );
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err("the module does not answer (try `tdmctl list`)".into())
}

fn command(a: &Args, msg: HostMsg) -> Result<(), String> {
    let mut src = SerialSource::spawn(a.port.clone(), None);
    let t0 = Instant::now();
    wait_hello(&mut src, t0)?;
    src.send(msg);
    if matches!(msg, HostMsg::EnterBootloader) {
        println!("restarting into the bootloader: the Pico will show up as the RPI-RP2 drive");
        std::thread::sleep(Duration::from_millis(300));
        return Ok(());
    }
    let deadline = t0.elapsed() + Duration::from_secs(2);
    while t0.elapsed() < deadline {
        for e in src.poll(0.0) {
            if let LinkEvent::Msg(McuMsg::Ack { status, .. }) = e {
                if status != tdm_proto::status::OK {
                    return Err("the module refused the command".into());
                }
                println!("done");
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err("no acknowledgement from the module".into())
}

fn monitor(a: &Args, p: &Profile, record: Option<PathBuf>) -> Result<(), String> {
    let mut src = SerialSource::spawn(a.port.clone(), record);
    let t0 = Instant::now();
    wait_hello(&mut src, t0)?;
    src.send(HostMsg::SetConfig(p.mcu_config()));
    let mut last_print = 0.0;
    let mut frames = 0u32;
    loop {
        let now = t0.elapsed().as_secs_f64();
        if a.seconds.is_some_and(|s| now > s) {
            return Ok(());
        }
        for e in src.poll(now) {
            match e {
                LinkEvent::Msg(McuMsg::Telemetry(t)) => {
                    frames += 1;
                    if now - last_print >= 1.0 {
                        let v = conv::speed_kmh(p, &t.wheel).unwrap_or(f64::NAN);
                        let r = conv::rpm(p, &t.ign).unwrap_or(f64::NAN);
                        let (c, ohm) = conv::coolant(p, t.coolant_raw);
                        println!(
                            "{:>7.1} s  {frames:>3} frame/s  {:>5.1} km/h  {:>5.0} rpm  {:>5.2} V  coolant {:?} ({} Ω)  light {}  inputs {:#06x}",
                            now,
                            v,
                            r,
                            conv::vbat_volts(p, t.vbat_raw),
                            c,
                            ohm.map_or("--".into(), |o| fmt::dec(EN, o, 1)),
                            t.light_raw,
                            t.inputs
                        );
                        frames = 0;
                        last_print = now;
                    }
                }
                LinkEvent::Msg(McuMsg::Odometer(o)) if now - last_print < 0.02 => {
                    println!(
                        "           km {} · A {} · B {}",
                        fmt::int(EN, o.odo_m as u64 / 1000),
                        fmt::dec(EN, o.trip_m[0] as f64 / 1000.0, 1),
                        fmt::dec(EN, o.trip_m[1] as f64 / 1000.0, 1)
                    );
                }
                LinkEvent::Down(r) => println!("link lost: {r}"),
                LinkEvent::Up(d) => {
                    println!("connected: {d}");
                    src.send(HostMsg::GetInfo);
                }
                _ => {}
            }
        }
        // keep the backlight alive while monitoring a bench setup
        src.send(HostMsg::Heartbeat { ui_frame: 0 });
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn number(s: Option<&String>, what: &str) -> Result<f64, String> {
    s.and_then(|v| v.replace(',', ".").parse().ok())
        .ok_or_else(|| format!("missing {what}"))
}

/// Prints the corrected value as a TOML fragment for the profile.
fn calibrate(a: &Args, p: &Profile) -> Result<(), String> {
    let arg = |i: usize| a.pos.get(i);
    match a.pos.first().map(String::as_str) {
        Some("wheel") => {
            let gps = number(arg(1), "the distance measured with the GPS, in km")?;
            let trip = number(arg(2), "the trip shown by the dashboard, in km")?;
            let old = p.speed.wheel_circumference_mm;
            let new = calib::wheel_circumference(old, gps, trip)?;
            println!(
                "circumference: {old:.1} → {new:.1} mm ({:+.2} %)\n\n[speed]\nwheel_circumference_mm = {new:.1}",
                (new / old - 1.0) * 100.0
            );
        }
        Some("tach") => {
            let reference = number(arg(1), "the true rpm (reference tachometer)")?;
            let shown = number(arg(2), "the rpm shown by the dashboard")?;
            let old = p.tach.pulses_per_rev;
            let (exact, snapped) = calib::tach_pulses_per_rev(old, reference, shown)?;
            println!("pulses per revolution: {old} → {exact:.3}, rounded to {snapped}");
            if (exact - snapped).abs() > 0.05 * snapped {
                println!(
                    "warning: far from a multiple of 0.5, repeat the measurement at steady rpm"
                );
            }
            println!("\n[tach]\npulses_per_rev = {snapped}");
        }
        Some("battery") => {
            let actual = number(arg(1), "the voltage measured with the multimeter")?;
            let shown = number(arg(2), "the voltage shown by the dashboard")?;
            let old = p.board.vbat_divider;
            let new = calib::vbat_divider(old, actual, shown)?;
            println!(
                "divider: {old:.4} → {new:.4} ({:+.2} %)\n\n[board]\nvbat_divider = {new:.4}",
                (new / old - 1.0) * 100.0
            );
        }
        Some("coolant") => {
            let file = arg(1).ok_or("missing the file with the measurements")?;
            let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
            let curve = calib::coolant_curve(&calib::parse_measurements(&text)?)?;
            println!(
                "{} points\n\n[coolant]\n{}",
                curve.len(),
                calib::curve_toml(&curve)
            );
        }
        _ => return Err("calibrate needs wheel, tach, battery or coolant".into()),
    }
    Ok(())
}

fn simulate(a: &Args, p: Profile) -> Result<(), String> {
    let id = a.pos.first().map_or("city", String::as_str);
    let sc = scenario(id).ok_or_else(|| format!("unknown scenario '{id}' (tdmctl scenarios)"))?;
    let secs = a.seconds.unwrap_or(30.0);
    let mut sim = SimMcu::new(&p, sc, 1);
    let mut d = Dash::new(p, None);
    d.set_language(EN);
    d.on_event(LinkEvent::Up("sim".into()), 0.0);
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    let mut next_print = 1.0;
    println!("    time   true     shown   rpm  coolant   battery  warning");
    while t < secs {
        t += dt;
        for m in sim.step(dt) {
            d.on_event(LinkEvent::Msg(m), t);
        }
        let vm = d.tick(t);
        d.first_frame_shown(t);
        for c in d.take_commands() {
            if let Some(r) = sim.send(c) {
                d.on_event(LinkEvent::Msg(r), t);
            }
        }
        if t >= next_print {
            next_print += 1.0;
            println!(
                "{:>6.0} s  {:>5.1}  {:>8}  {:>5}  {:>5}°  {:>6} V  {}",
                t,
                sim.true_kmh(),
                vm.speed_text,
                vm.rpm_text,
                vm.coolant_text,
                vm.vbat_text,
                vm.warning.map_or(String::new(), |w| w.title)
            );
        }
    }
    Ok(())
}
