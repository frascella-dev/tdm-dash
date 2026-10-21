//! Where the dashboard's data comes from: the real co-processor over USB, the
//! simulator, or a recorded session.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use tdm_logic::{LinkEvent, Profile};
use tdm_proto::{Dir, FrameDecoder, HostMsg, MAX_FRAME, McuMsg};

use crate::sim::{Scenario, SimMcu};

/// A source is polled from the UI loop with the dashboard clock.
pub trait Source: Send {
    fn poll(&mut self, now: f64) -> Vec<LinkEvent>;
    fn send(&mut self, msg: HostMsg);
    fn describe(&self) -> String;
    /// The simulator, when this source is one (for keyboard input).
    fn sim(&mut self) -> Option<&mut SimMcu> {
        None
    }
}

// ---------------------------------------------------------------- simulator

pub struct SimSource {
    pub sim: SimMcu,
    last: Option<f64>,
    was_down: bool,
    pending: Vec<LinkEvent>,
}

impl std::fmt::Debug for SimSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimSource")
            .field("scenario", &self.sim.scenario.id)
            .finish()
    }
}

impl SimSource {
    pub fn new(profile: &Profile, scenario: Scenario, seed: u64) -> Self {
        Self {
            sim: SimMcu::new(profile, scenario, seed),
            last: None,
            was_down: true,
            pending: Vec::new(),
        }
    }
}

impl Source for SimSource {
    fn poll(&mut self, now: f64) -> Vec<LinkEvent> {
        let dt = self.last.map_or(0.0, |l| (now - l).clamp(0.0, 0.25));
        self.last = Some(now);
        let mut ev = std::mem::take(&mut self.pending);
        let down = self.sim.link_down();
        if down != self.was_down {
            ev.push(if down {
                LinkEvent::Down("cavo scollegato".into())
            } else {
                LinkEvent::Up("simulatore".into())
            });
            self.was_down = down;
        }
        if dt > 0.0 {
            ev.extend(self.sim.step(dt).into_iter().map(LinkEvent::Msg));
        }
        ev
    }

    fn send(&mut self, msg: HostMsg) {
        if let Some(r) = self.sim.send(msg) {
            self.pending.push(LinkEvent::Msg(r));
        }
    }

    fn describe(&self) -> String {
        format!("simulatore: {}", self.sim.scenario.name)
    }

    fn sim(&mut self) -> Option<&mut SimMcu> {
        Some(&mut self.sim)
    }
}

// ---------------------------------------------------------------- serial

/// Finds the co-processor among `/sys/class/tty/ttyACM*` by USB VID:PID.
pub fn find_ports() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(dir) = std::fs::read_dir("/sys/class/tty") else {
        return out;
    };
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.starts_with("ttyACM") {
            continue;
        }
        // device -> interface; its parent holds idVendor/idProduct
        let dev = e.path().join("device/..");
        let read = |f: &str| {
            std::fs::read_to_string(dev.join(f))
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };
        let vid = u16::from_str_radix(&read("idVendor"), 16).unwrap_or(0);
        let pid = u16::from_str_radix(&read("idProduct"), 16).unwrap_or(0);
        if vid == tdm_proto::USB_VID && pid == tdm_proto::USB_PID {
            out.push(PathBuf::from("/dev").join(name));
        }
    }
    out.sort();
    out
}

enum Ctl {
    Send(HostMsg),
}

/// USB CDC link handled by a background thread that reconnects forever.
pub struct SerialSource {
    rx: Receiver<LinkEvent>,
    tx: Sender<Ctl>,
    fixed: Option<PathBuf>,
}

impl std::fmt::Debug for SerialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerialSource")
            .field("port", &self.fixed)
            .finish()
    }
}

impl SerialSource {
    /// `port`: fixed device path, or `None` to discover by VID:PID.
    pub fn spawn(port: Option<PathBuf>, record: Option<PathBuf>) -> Self {
        let (ev_tx, rx) = mpsc::channel();
        let (tx, ctl_rx) = mpsc::channel();
        let fixed = port.clone();
        thread::Builder::new()
            .name("tdm-serial".into())
            .spawn(move || serial_thread(port, record, ev_tx, ctl_rx))
            .expect("spawn serial thread");
        Self { rx, tx, fixed }
    }
}

impl Source for SerialSource {
    fn poll(&mut self, _now: f64) -> Vec<LinkEvent> {
        let mut v = Vec::new();
        while let Ok(e) = self.rx.try_recv() {
            v.push(e);
        }
        v
    }

    fn send(&mut self, msg: HostMsg) {
        let _ = self.tx.send(Ctl::Send(msg));
    }

    fn describe(&self) -> String {
        match &self.fixed {
            Some(p) => format!("USB {}", p.display()),
            None => "USB (automatic discovery)".into(),
        }
    }
}

fn serial_thread(
    port: Option<PathBuf>,
    record: Option<PathBuf>,
    ev: Sender<LinkEvent>,
    ctl: Receiver<Ctl>,
) {
    let mut rec = record
        .and_then(|p| std::fs::File::create(p).ok())
        .map(Recorder::new);
    loop {
        let path = port.clone().or_else(|| find_ports().into_iter().next());
        let Some(path) = path else {
            thread::sleep(Duration::from_millis(250));
            drain(&ctl);
            continue;
        };
        let opened = serialport::new(path.to_string_lossy(), 115_200)
            .timeout(Duration::from_millis(100))
            .open_native();
        let mut sp = match opened {
            Ok(p) => p,
            Err(_) => {
                thread::sleep(Duration::from_millis(250));
                continue;
            }
        };
        if ev.send(LinkEvent::Up(path.display().to_string())).is_err() {
            return;
        }
        let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::McuToHost);
        let mut buf = [0u8; 256];
        let reason = 'conn: loop {
            while let Ok(Ctl::Send(m)) = ctl.try_recv() {
                let mut out = [0u8; MAX_FRAME];
                if let Ok(n) = tdm_proto::encode(&m, Dir::HostToMcu, &mut out)
                    && let Err(e) = sp.write_all(&out[..n])
                {
                    break 'conn e.to_string();
                }
            }
            match sp.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    if let Some(r) = rec.as_mut() {
                        r.write(&buf[..n]);
                    }
                    let mut msgs = Vec::new();
                    dec.feed(&buf[..n], |r| {
                        if let Ok(p) = r
                            && let Ok(m) = tdm_proto::decode::<McuMsg>(p)
                        {
                            msgs.push(m);
                        }
                    });
                    for m in msgs {
                        if ev.send(LinkEvent::Msg(m)).is_err() {
                            return;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => break 'conn e.to_string(),
            }
        };
        if ev.send(LinkEvent::Down(reason)).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn drain(ctl: &Receiver<Ctl>) {
    while ctl.try_recv().is_ok() {}
}

// ---------------------------------------------------------------- recording

/// Raw byte stream with timestamps: `[t_us: u64 LE][len: u16 LE][bytes]`.
#[derive(Debug)]
pub struct Recorder<W: Write> {
    out: W,
    t0: Instant,
}

impl<W: Write> Recorder<W> {
    pub fn new(out: W) -> Self {
        Self {
            out,
            t0: Instant::now(),
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        let t = self.t0.elapsed().as_micros() as u64;
        let _ = self.out.write_all(&t.to_le_bytes());
        let _ = self.out.write_all(&(bytes.len() as u16).to_le_bytes());
        let _ = self.out.write_all(bytes);
    }
}

/// Plays back a recording at its original pace.
#[derive(Debug)]
pub struct ReplaySource {
    chunks: Vec<(f64, Vec<u8>)>,
    idx: usize,
    t0: Option<f64>,
    dec: FrameDecoder<MAX_FRAME>,
    name: String,
    started: bool,
}

impl ReplaySource {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let mut data = Vec::new();
        std::fs::File::open(path)?.read_to_end(&mut data)?;
        let mut chunks = Vec::new();
        let mut i = 0;
        while i + 10 <= data.len() {
            let t = u64::from_le_bytes(data[i..i + 8].try_into().unwrap()) as f64 / 1e6;
            let n = u16::from_le_bytes([data[i + 8], data[i + 9]]) as usize;
            i += 10;
            if i + n > data.len() {
                break;
            }
            chunks.push((t, data[i..i + n].to_vec()));
            i += n;
        }
        Ok(Self {
            chunks,
            idx: 0,
            t0: None,
            dec: FrameDecoder::new(Dir::McuToHost),
            name: path.display().to_string(),
            started: false,
        })
    }
}

impl Source for ReplaySource {
    fn poll(&mut self, now: f64) -> Vec<LinkEvent> {
        let t0 = *self.t0.get_or_insert(now);
        let mut ev = Vec::new();
        if !self.started {
            self.started = true;
            ev.push(LinkEvent::Up(format!("registrazione {}", self.name)));
        }
        while self.idx < self.chunks.len() && self.chunks[self.idx].0 <= now - t0 {
            let bytes = std::mem::take(&mut self.chunks[self.idx].1);
            self.dec.feed(&bytes, |r| {
                if let Ok(p) = r
                    && let Ok(m) = tdm_proto::decode::<McuMsg>(p)
                {
                    ev.push(LinkEvent::Msg(m));
                }
            });
            self.idx += 1;
        }
        ev
    }

    fn send(&mut self, _msg: HostMsg) {}

    fn describe(&self) -> String {
        format!("registrazione {}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::scenario;

    #[test]
    fn record_and_replay_roundtrip() {
        let p = Profile::builtin();
        let mut sim = SimMcu::new(&p, scenario("city").unwrap(), 3);
        sim.send(HostMsg::GetInfo);
        let mut file = Vec::new();
        let mut rec = Recorder::new(&mut file);
        let mut sent = Vec::new();
        for _ in 0..30 {
            for m in sim.step(0.02) {
                let mut out = [0u8; MAX_FRAME];
                let n = tdm_proto::encode(&m, Dir::McuToHost, &mut out).unwrap();
                rec.write(&out[..n]);
                sent.push(m);
            }
        }
        let path = std::env::temp_dir().join(format!("tdm-rec-{}.bin", std::process::id()));
        std::fs::write(&path, &file).unwrap();
        let mut rs = ReplaySource::open(&path).unwrap();
        rs.poll(0.0);
        let ev = rs.poll(100.0);
        let got: Vec<McuMsg> = ev
            .into_iter()
            .filter_map(|e| match e {
                LinkEvent::Msg(m) => Some(m),
                _ => None,
            })
            .collect();
        assert_eq!(got, sent);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn sim_source_reports_link_drops() {
        let p = Profile::builtin();
        let mut s = SimSource::new(&p, scenario("linkdrop").unwrap(), 1);
        s.send(HostMsg::GetInfo);
        let mut ups = 0;
        let mut downs = 0;
        let mut t = 0.0;
        while t < 45.0 {
            t += 0.05;
            for e in s.poll(t) {
                match e {
                    LinkEvent::Up(_) => ups += 1,
                    LinkEvent::Down(_) => downs += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(downs, 2);
        assert_eq!(ups, 3);
    }
}
