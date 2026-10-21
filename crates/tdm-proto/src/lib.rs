//! Wire protocol between the RP2040 co-processor and the Linux UI.
//!
//! A frame is `postcard(message) ‖ CRC-16`, COBS-encoded and terminated by
//! `0x00`. The CRC also covers a direction tag that is never transmitted, so a
//! frame echoed back by a tty that is still in cooked mode fails the check on
//! the other side instead of being taken for a command (see `docs/adr/0002`).
//!
//! Compatibility rule: enum variants and struct fields are append-only.
//! Any other change bumps [`PROTO_VERSION`].

#![cfg_attr(not(feature = "std"), no_std)]

use crc::{CRC_16_IBM_SDLC, Crc};
use serde::{Deserialize, Serialize};

/// Bumped on every breaking change.
pub const PROTO_VERSION: u16 = 1;

/// Largest encoded frame, delimiter included.
pub const MAX_FRAME: usize = 128;

/// Largest payload before COBS encoding (postcard bytes plus the CRC).
pub const MAX_RAW: usize = MAX_FRAME - 1 - (MAX_FRAME / 254 + 1);

const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

/// USB identity of the co-processor (pid.codes test PID until a real one is
/// allocated).
pub const USB_VID: u16 = 0x1209;
pub const USB_PID: u16 = 0x0001;
pub const USB_PRODUCT: &str = "tdm-dash io";

/// Frame direction; mixed into the CRC but not sent on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    McuToHost,
    HostToMcu,
}

impl Dir {
    pub const fn tag(self) -> u8 {
        match self {
            Dir::McuToHost => 0xA5,
            Dir::HostToMcu => 0x5A,
        }
    }
}

// ---------------------------------------------------------------- messages

/// Edge statistics computed by the MCU over whole groups of pulses (whole
/// wheel revolutions for the speed sensor), so uneven bolt spacing cancels.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PulseStats {
    /// Accepted edges since boot (wraps).
    pub total: u32,
    /// Edges spanned by the measurement window; 0 means "no valid window".
    pub span_edges: u16,
    /// Duration of the window, from first to last edge.
    pub span_us: u32,
    /// Time since the last accepted edge.
    pub age_us: u32,
}

/// Bits of [`Telemetry::inputs`] (debounced on the MCU).
pub mod input {
    pub const NEUTRAL: u16 = 1 << 0;
    pub const HIGH_BEAM: u16 = 1 << 1;
    pub const TURN_LEFT: u16 = 1 << 2;
    pub const TURN_RIGHT: u16 = 1 << 3;
    pub const ILLUMINATION: u16 = 1 << 4;
    pub const BTN_MODE: u16 = 1 << 8;
    pub const BTN_SET: u16 = 1 << 9;
}

/// Bits of [`Telemetry::flags`].
pub mod flag {
    pub const FRAM_OK: u8 = 1 << 0;
    pub const POWER_LOW: u8 = 1 << 1;
    pub const ODO_INITIALISED: u8 = 1 << 2;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Telemetry {
    pub seq: u16,
    pub uptime_us: u64,
    pub wheel: PulseStats,
    pub ign: PulseStats,
    /// 12-bit ADC readings, already oversampled.
    pub vbat_raw: u16,
    pub coolant_raw: u16,
    pub light_raw: u16,
    pub inputs: u16,
    pub flags: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Odometer {
    pub odo_m: u32,
    pub trip_m: [u32; 2],
}

/// Measurement settings the host derives from the bike profile.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct McuConfig {
    /// Distance per accepted wheel edge, in micrometres.
    pub um_per_pulse: u32,
    pub wheel_ppr: u8,
    pub wheel_min_gap_us: u32,
    pub wheel_min_window_us: u32,
    pub ign_min_gap_us: u32,
    pub ign_blank_pct: u8,
    pub ign_min_window_us: u32,
    pub debounce_ms: u8,
}

/// Rider preferences, persisted in FRAM by the MCU.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UserSettings {
    /// 0 = auto, 1 = day, 2 = night.
    pub theme: u8,
    /// 0 = classic, 1 = sport, 2 = touring.
    pub layout: u8,
    pub brightness_offset: i8,
    pub diag: bool,
}

/// State of the odometer record in FRAM.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OdoStatus {
    /// Valid record loaded.
    #[default]
    Ok,
    /// FRAM blank: counting from zero, `InitOdometer` accepted once.
    Blank,
    /// Both slots corrupt: never overwritten, only `InitOdometer` accepted.
    Corrupt,
    /// FRAM not answering on I2C.
    Missing,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hello {
    pub proto: u16,
    pub fw: [u8; 3],
    pub board_rev: u8,
    pub uptime_us: u64,
    pub reset_reason: u8,
    pub config: McuConfig,
    pub settings: Option<UserSettings>,
    pub odo_status: OdoStatus,
    /// Key-on to first `SetBacklight`, measured by the MCU.
    pub first_backlight_ms: Option<u32>,
}

/// Command identifiers echoed in [`McuMsg::Ack`].
pub mod cmd {
    pub const SET_CONFIG: u8 = 1;
    pub const RESET_TRIP: u8 = 2;
    pub const INIT_ODOMETER: u8 = 3;
    pub const STORE_SETTINGS: u8 = 4;
}

/// Status codes in [`McuMsg::Ack`].
pub mod status {
    pub const OK: u8 = 0;
    pub const REFUSED: u8 = 1;
    pub const FRAM_ERROR: u8 = 2;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum McuMsg {
    Hello(Hello),
    Telemetry(Telemetry),
    Odometer(Odometer),
    Ack { cmd: u8, status: u8 },
    PowerFail { vbat_raw: u16 },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostMsg {
    GetInfo,
    Heartbeat { ui_frame: u32 },
    SetConfig(McuConfig),
    SetBacklight { permille: u16, fade_ms: u16 },
    ResetTrip { idx: u8 },
    InitOdometer { odo_m: u32 },
    StoreSettings(UserSettings),
    EnterBootloader,
}

// ---------------------------------------------------------------- framing

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// Message does not fit in [`MAX_FRAME`].
    TooLarge,
    /// More than [`MAX_FRAME`] bytes without a delimiter.
    Overflow,
    Cobs,
    Crc,
    Postcard,
}

fn crc(dir: Dir, payload: &[u8]) -> u16 {
    let mut d = CRC16.digest();
    d.update(&[dir.tag()]);
    d.update(payload);
    d.finalize()
}

/// Encodes `msg` into `out`, delimiter included; returns the frame length.
pub fn encode<T: Serialize>(msg: &T, dir: Dir, out: &mut [u8]) -> Result<usize, FrameError> {
    let mut raw = [0u8; MAX_RAW];
    let n = postcard::to_slice(msg, &mut raw[..MAX_RAW - 2])
        .map_err(|_| FrameError::TooLarge)?
        .len();
    let c = crc(dir, &raw[..n]);
    raw[n..n + 2].copy_from_slice(&c.to_le_bytes());
    let m = n + 2;
    if out.len() < cobs::max_encoding_length(m) + 1 {
        return Err(FrameError::TooLarge);
    }
    let k = cobs::encode(&raw[..m], out);
    out[k] = 0;
    Ok(k + 1)
}

/// Checks the CRC of a COBS-decoded frame and returns its postcard payload.
pub fn verify(dir: Dir, frame: &[u8]) -> Result<&[u8], FrameError> {
    if frame.len() < 3 {
        return Err(FrameError::Crc);
    }
    let (payload, tail) = frame.split_at(frame.len() - 2);
    if crc(dir, payload).to_le_bytes() != [tail[0], tail[1]] {
        return Err(FrameError::Crc);
    }
    Ok(payload)
}

/// Deserialises a payload returned by [`FrameDecoder`].
pub fn decode<'a, T: Deserialize<'a>>(payload: &'a [u8]) -> Result<T, FrameError> {
    postcard::from_bytes(payload).map_err(|_| FrameError::Postcard)
}

/// Splits a byte stream into verified payloads. Resynchronises on the next
/// delimiter after any error, so garbage on the line costs at most one frame.
#[derive(Debug)]
pub struct FrameDecoder<const N: usize = MAX_FRAME> {
    dir: Dir,
    buf: [u8; N],
    len: usize,
    overflow: bool,
}

impl<const N: usize> FrameDecoder<N> {
    /// `dir` is the direction of the frames this side *receives*.
    pub const fn new(dir: Dir) -> Self {
        Self {
            dir,
            buf: [0; N],
            len: 0,
            overflow: false,
        }
    }

    pub fn reset(&mut self) {
        self.len = 0;
        self.overflow = false;
    }

    /// Feeds received bytes; `f` is called once per complete frame.
    pub fn feed(&mut self, data: &[u8], mut f: impl FnMut(Result<&[u8], FrameError>)) {
        for &b in data {
            if b != 0 {
                if self.len < N {
                    self.buf[self.len] = b;
                    self.len += 1;
                } else {
                    self.overflow = true;
                }
                continue;
            }
            if self.overflow {
                f(Err(FrameError::Overflow));
            } else if self.len > 0 {
                match cobs::decode_in_place(&mut self.buf[..self.len]) {
                    Ok(n) => f(verify(self.dir, &self.buf[..n])),
                    Err(_) => f(Err(FrameError::Cobs)),
                }
            }
            self.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::vec::Vec;

    fn sample_telemetry() -> McuMsg {
        McuMsg::Telemetry(Telemetry {
            seq: 0xFFFF,
            uptime_us: u64::MAX / 3,
            wheel: PulseStats {
                total: u32::MAX,
                span_edges: 24,
                span_us: 412_345,
                age_us: 1_234,
            },
            ign: PulseStats {
                total: 99_999,
                span_edges: 8,
                span_us: 60_000,
                age_us: 2_000,
            },
            vbat_raw: 4095,
            coolant_raw: 1212,
            light_raw: 4000,
            inputs: 0x030F,
            flags: 0xFF,
        })
    }

    fn largest_messages() -> Vec<McuMsg> {
        let cfg = McuConfig {
            um_per_pulse: u32::MAX,
            wheel_ppr: 255,
            wheel_min_gap_us: u32::MAX,
            wheel_min_window_us: u32::MAX,
            ign_min_gap_us: u32::MAX,
            ign_blank_pct: 255,
            ign_min_window_us: u32::MAX,
            debounce_ms: 255,
        };
        std::vec![
            sample_telemetry(),
            McuMsg::Hello(Hello {
                proto: u16::MAX,
                fw: [255; 3],
                board_rev: 255,
                uptime_us: u64::MAX,
                reset_reason: 255,
                config: cfg,
                settings: Some(UserSettings {
                    theme: 255,
                    layout: 255,
                    brightness_offset: -128,
                    diag: true
                }),
                odo_status: OdoStatus::Missing,
                first_backlight_ms: Some(u32::MAX),
            }),
            McuMsg::Odometer(Odometer {
                odo_m: u32::MAX,
                trip_m: [u32::MAX; 2]
            }),
        ]
    }

    fn roundtrip(msg: &McuMsg) -> McuMsg {
        let mut out = [0u8; MAX_FRAME];
        let n = encode(msg, Dir::McuToHost, &mut out).unwrap();
        assert_eq!(out[n - 1], 0);
        assert!(!out[..n - 1].contains(&0));
        let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::McuToHost);
        let mut got = None;
        dec.feed(&out[..n], |r| {
            got = Some(decode::<McuMsg>(r.unwrap()).unwrap())
        });
        got.unwrap()
    }

    #[test]
    fn every_message_fits_and_roundtrips() {
        for m in largest_messages() {
            assert_eq!(roundtrip(&m), m);
        }
        let mut out = [0u8; MAX_FRAME];
        let n = encode(
            &HostMsg::SetConfig(McuConfig::default()),
            Dir::HostToMcu,
            &mut out,
        )
        .unwrap();
        assert!(n <= MAX_FRAME);
    }

    #[test]
    fn echoed_frame_is_rejected() {
        // The host must never accept its own (or an echoed MCU) frame as a
        // message from the other direction.
        let mut out = [0u8; MAX_FRAME];
        let n = encode(&HostMsg::ResetTrip { idx: 0 }, Dir::HostToMcu, &mut out).unwrap();
        let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::McuToHost);
        let mut results = Vec::new();
        dec.feed(&out[..n], |r| results.push(r.map(|p| p.len())));
        assert_eq!(results, std::vec![Err(FrameError::Crc)]);
    }

    #[test]
    fn resyncs_after_garbage_and_corruption() {
        let mut out = [0u8; MAX_FRAME];
        let n = encode(&sample_telemetry(), Dir::McuToHost, &mut out).unwrap();
        let mut stream = Vec::new();
        stream.extend_from_slice(&[0x13, 0x37, 0x99, 0x00]); // garbage frame
        let mut corrupt = out[..n].to_vec();
        corrupt[3] ^= 0x40;
        stream.extend_from_slice(&corrupt);
        stream.extend_from_slice(&[0xAB; 300]); // overflow, no delimiter
        stream.push(0);
        stream.extend_from_slice(&out[..n]);
        let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::McuToHost);
        let mut ok = 0;
        let mut errors = 0;
        // Feed in odd-sized chunks, as a serial port would deliver them.
        for chunk in stream.chunks(7) {
            dec.feed(chunk, |r| match r {
                Ok(p) => {
                    assert_eq!(decode::<McuMsg>(p).unwrap(), sample_telemetry());
                    ok += 1;
                }
                Err(_) => errors += 1,
            });
        }
        assert_eq!(ok, 1);
        assert_eq!(errors, 3);
    }

    #[test]
    fn single_bit_flips_never_decode() {
        let mut out = [0u8; MAX_FRAME];
        let n = encode(&sample_telemetry(), Dir::McuToHost, &mut out).unwrap();
        for i in 0..n - 1 {
            for bit in 0..8 {
                let mut f = out[..n].to_vec();
                f[i] ^= 1 << bit;
                if f[i] == 0 {
                    continue; // becomes a delimiter: a different, shorter frame
                }
                let mut dec = FrameDecoder::<MAX_FRAME>::new(Dir::McuToHost);
                dec.feed(&f, |r| {
                    if let Ok(p) = r {
                        // CRC-16 catches all single-bit errors.
                        panic!("bit flip at {i}:{bit} decoded to {:?}", decode::<McuMsg>(p));
                    }
                });
            }
        }
    }
}
