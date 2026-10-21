//! `tdmctl`: talk to the tdm-dash co-processor from a PC or from the Pi's
//! development image.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use tdm_link::{SerialSource, Source, find_ports, scenario, scenarios, sim::SimMcu};
use tdm_logic::{Dash, LinkEvent, Profile, calib, conv, fmt};
use tdm_proto::{HostMsg, McuMsg};

const USAGE: &str = "\
tdmctl — strumento per il modulo sensori di tdm-dash

USO:
  tdmctl list                         elenca i moduli collegati via USB
  tdmctl monitor [--port DEV] [-n S]  mostra la telemetria (S secondi, default infinito)
  tdmctl record FILE [--port DEV]     registra il traffico per --source replay:FILE
  tdmctl reset-trip a|b [--port DEV]  azzera un parziale
  tdmctl init-odo KM [--port DEV]     imposta il contachilometri (solo FRAM vuota)
  tdmctl bootsel [--port DEV]         riavvia il Pico nel bootloader USB
  tdmctl sim [SCENARIO] [-n S]        simula senza hardware e stampa velocità e giri
  tdmctl scenarios                    elenca gli scenari del simulatore

CALIBRAZIONE (capitolo 10 del manuale; --profile FILE = profilo in uso):
  tdmctl calibrate wheel KM_GPS KM_PARZIALE      circonferenza della ruota
  tdmctl calibrate tach GIRI_VERI GIRI_MOSTRATI  impulsi per giro del contagiri
  tdmctl calibrate battery V_TESTER V_MOSTRATI   rapporto del partitore della batteria
  tdmctl calibrate coolant FILE                  curva del sensore acqua (\"°C ohm\" per riga)
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
    let cmd = it.next().ok_or("manca il comando")?;
    let mut a = Args {
        cmd,
        pos: vec![],
        port: None,
        seconds: None,
        profile: None,
    };
    while let Some(x) = it.next() {
        match x.as_str() {
            "--port" => a.port = Some(it.next().ok_or("--port vuole un dispositivo")?.into()),
            "-n" | "--seconds" => {
                a.seconds = Some(
                    it.next()
                        .ok_or("-n vuole un numero")?
                        .parse()
                        .map_err(|_| "-n vuole un numero")?,
                )
            }
            "--profile" => a.profile = Some(it.next().ok_or("--profile vuole un file")?.into()),
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
                eprintln!("errore: {e}\n");
            }
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let profile = match &a.profile {
        Some(p) => match Profile::load(p) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("errore: {e}");
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
                    "nessun modulo trovato (VID:PID {:04x}:{:04x})",
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
            None => Err("record vuole il nome del file".into()),
        },
        "reset-trip" => match a.pos.first().map(String::as_str) {
            Some("a" | "A") => command(&a, HostMsg::ResetTrip { idx: 0 }),
            Some("b" | "B") => command(&a, HostMsg::ResetTrip { idx: 1 }),
            _ => Err("reset-trip vuole a oppure b".into()),
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
            _ => Err("init-odo vuole i km, per esempio 48213".into()),
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
        other => Err(format!("comando sconosciuto '{other}'")),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("errore: {e}");
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
                    "modulo: firmware {}.{}.{}, protocollo {}, FRAM {:?}, acceso da {} s",
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
    Err("il modulo non risponde (prova `tdmctl list`)".into())
}

fn command(a: &Args, msg: HostMsg) -> Result<(), String> {
    let mut src = SerialSource::spawn(a.port.clone(), None);
    let t0 = Instant::now();
    wait_hello(&mut src, t0)?;
    src.send(msg);
    if matches!(msg, HostMsg::EnterBootloader) {
        println!("riavvio nel bootloader: il Pico comparirà come disco RPI-RP2");
        std::thread::sleep(Duration::from_millis(300));
        return Ok(());
    }
    let deadline = t0.elapsed() + Duration::from_secs(2);
    while t0.elapsed() < deadline {
        for e in src.poll(0.0) {
            if let LinkEvent::Msg(McuMsg::Ack { status, .. }) = e {
                if status != tdm_proto::status::OK {
                    return Err("il modulo ha rifiutato il comando".into());
                }
                println!("fatto");
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err("nessuna conferma dal modulo".into())
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
                            "{:>7.1} s  {frames:>3} frame/s  {:>5.1} km/h  {:>5.0} giri  {:>5.2} V  acqua {:?} ({} Ω)  luce {}  ingressi {:#06x}",
                            now,
                            v,
                            r,
                            conv::vbat_volts(p, t.vbat_raw),
                            c,
                            ohm.map_or("--".into(), |o| fmt::dec(o, 1)),
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
                        fmt::int(o.odo_m as u64 / 1000),
                        fmt::dec(o.trip_m[0] as f64 / 1000.0, 1),
                        fmt::dec(o.trip_m[1] as f64 / 1000.0, 1)
                    );
                }
                LinkEvent::Down(r) => println!("collegamento perso: {r}"),
                LinkEvent::Up(d) => {
                    println!("collegato: {d}");
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
        .ok_or_else(|| format!("manca {what}"))
}

/// Prints the corrected value as a TOML fragment for the profile.
fn calibrate(a: &Args, p: &Profile) -> Result<(), String> {
    let arg = |i: usize| a.pos.get(i);
    match a.pos.first().map(String::as_str) {
        Some("wheel") => {
            let gps = number(arg(1), "la distanza misurata col GPS, in km")?;
            let trip = number(arg(2), "il parziale segnato dal quadro, in km")?;
            let old = p.speed.wheel_circumference_mm;
            let new = calib::wheel_circumference(old, gps, trip)?;
            println!(
                "circonferenza: {old:.1} → {new:.1} mm ({:+.2} %)\n\n[speed]\nwheel_circumference_mm = {new:.1}",
                (new / old - 1.0) * 100.0
            );
        }
        Some("tach") => {
            let reference = number(arg(1), "i giri veri (contagiri di riferimento)")?;
            let shown = number(arg(2), "i giri mostrati dal quadro")?;
            let old = p.tach.pulses_per_rev;
            let (exact, snapped) = calib::tach_pulses_per_rev(old, reference, shown)?;
            println!("impulsi per giro: {old} → {exact:.3}, arrotondato a {snapped}");
            if (exact - snapped).abs() > 0.05 * snapped {
                println!(
                    "attenzione: lontano da un multiplo di 0,5, ripeti la misura a regime stabile"
                );
            }
            println!("\n[tach]\npulses_per_rev = {snapped}");
        }
        Some("battery") => {
            let actual = number(arg(1), "la tensione misurata col tester")?;
            let shown = number(arg(2), "la tensione mostrata dal quadro")?;
            let old = p.board.vbat_divider;
            let new = calib::vbat_divider(old, actual, shown)?;
            println!(
                "partitore: {old:.4} → {new:.4} ({:+.2} %)\n\n[board]\nvbat_divider = {new:.4}",
                (new / old - 1.0) * 100.0
            );
        }
        Some("coolant") => {
            let file = arg(1).ok_or("manca il file con le misure")?;
            let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
            let curve = calib::coolant_curve(&calib::parse_measurements(&text)?)?;
            println!(
                "{} punti\n\n[coolant]\n{}",
                curve.len(),
                calib::curve_toml(&curve)
            );
        }
        _ => return Err("calibrate vuole wheel, tach, battery oppure coolant".into()),
    }
    Ok(())
}

fn simulate(a: &Args, p: Profile) -> Result<(), String> {
    let id = a.pos.first().map_or("city", String::as_str);
    let sc =
        scenario(id).ok_or_else(|| format!("scenario sconosciuto '{id}' (tdmctl scenarios)"))?;
    let secs = a.seconds.unwrap_or(30.0);
    let mut sim = SimMcu::new(&p, sc, 1);
    let mut d = Dash::new(p, None);
    d.on_event(LinkEvent::Up("sim".into()), 0.0);
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    let mut next_print = 1.0;
    println!("   tempo  reale  mostrata  giri    acqua  batteria  avviso");
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
