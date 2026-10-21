//! Power-fail-safe records in FRAM: each record lives in two slots written
//! alternately; the valid slot with the highest sequence number wins.

use crc::{CRC_32_ISO_HDLC, Crc};

pub const RECORD_LEN: usize = 40;
pub const PAYLOAD_LEN: usize = 28;
const MAGIC: [u8; 2] = *b"TD";
const VERSION: u8 = 1;
const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// Byte-addressed non-volatile memory (FRAM on the board, RAM in tests).
pub trait Nvm {
    type Error: core::fmt::Debug;
    fn read(&mut self, addr: u16, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: u16, data: &[u8]) -> Result<(), Self::Error>;
}

/// Where a record kind lives in the 32 KiB FRAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slots {
    pub kind: u8,
    pub a: u16,
    pub b: u16,
}

pub const ODO_SLOTS: Slots = Slots {
    kind: 1,
    a: 0x0040,
    b: 0x0080,
};
pub const CONFIG_SLOTS: Slots = Slots {
    kind: 2,
    a: 0x0100,
    b: 0x0140,
};
pub const SETTINGS_SLOTS: Slots = Slots {
    kind: 3,
    a: 0x0200,
    b: 0x0240,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Blank,
    Corrupt,
    Valid {
        seq: u32,
        payload: [u8; PAYLOAD_LEN],
    },
}

pub fn encode_record(kind: u8, seq: u32, payload: &[u8; PAYLOAD_LEN]) -> [u8; RECORD_LEN] {
    let mut r = [0u8; RECORD_LEN];
    r[0..2].copy_from_slice(&MAGIC);
    r[2] = kind;
    r[3] = VERSION;
    r[4..8].copy_from_slice(&seq.to_le_bytes());
    r[8..8 + PAYLOAD_LEN].copy_from_slice(payload);
    let c = CRC32.checksum(&r[..RECORD_LEN - 4]);
    r[RECORD_LEN - 4..].copy_from_slice(&c.to_le_bytes());
    r
}

pub fn decode_record(kind: u8, r: &[u8; RECORD_LEN]) -> SlotState {
    if r.iter().all(|&b| b == 0x00) || r.iter().all(|&b| b == 0xFF) {
        return SlotState::Blank;
    }
    let c = u32::from_le_bytes([r[36], r[37], r[38], r[39]]);
    if r[0..2] != MAGIC
        || r[2] != kind
        || r[3] != VERSION
        || CRC32.checksum(&r[..RECORD_LEN - 4]) != c
    {
        return SlotState::Corrupt;
    }
    let mut payload = [0u8; PAYLOAD_LEN];
    payload.copy_from_slice(&r[8..8 + PAYLOAD_LEN]);
    SlotState::Valid {
        seq: u32::from_le_bytes([r[4], r[5], r[6], r[7]]),
        payload,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loaded {
    /// Both slots blank: never written.
    Blank,
    /// Something was written but no slot is valid.
    Corrupt,
    Valid {
        seq: u32,
        payload: [u8; PAYLOAD_LEN],
    },
}

pub fn load<N: Nvm>(nvm: &mut N, slots: Slots) -> Result<Loaded, N::Error> {
    let mut a = [0u8; RECORD_LEN];
    let mut b = [0u8; RECORD_LEN];
    nvm.read(slots.a, &mut a)?;
    nvm.read(slots.b, &mut b)?;
    Ok(
        match (decode_record(slots.kind, &a), decode_record(slots.kind, &b)) {
            (
                SlotState::Valid {
                    seq: sa,
                    payload: pa,
                },
                SlotState::Valid {
                    seq: sb,
                    payload: pb,
                },
            ) => {
                // wrapping comparison: the newer one is "ahead" by less than 2^31
                if sb.wrapping_sub(sa) < 0x8000_0000 && sb != sa {
                    Loaded::Valid {
                        seq: sb,
                        payload: pb,
                    }
                } else {
                    Loaded::Valid {
                        seq: sa,
                        payload: pa,
                    }
                }
            }
            (SlotState::Valid { seq, payload }, _) | (_, SlotState::Valid { seq, payload }) => {
                Loaded::Valid { seq, payload }
            }
            (SlotState::Blank, SlotState::Blank) => Loaded::Blank,
            _ => Loaded::Corrupt,
        },
    )
}

/// Writes the record with sequence `seq` into the slot chosen by its parity,
/// so the previous record (seq − 1) stays intact until this one is complete.
pub fn save<N: Nvm>(
    nvm: &mut N,
    slots: Slots,
    seq: u32,
    payload: &[u8; PAYLOAD_LEN],
) -> Result<(), N::Error> {
    let addr = if seq.is_multiple_of(2) {
        slots.a
    } else {
        slots.b
    };
    nvm.write(addr, &encode_record(slots.kind, seq, payload))
}

/// RAM-backed NVM for tests and the simulator, with torn-write injection.
#[derive(Debug, Clone)]
pub struct MemNvm<const N: usize> {
    pub data: [u8; N],
    /// When set, the next write stops after this many bytes.
    pub tear_after: Option<usize>,
    pub writes: u32,
}

impl<const N: usize> MemNvm<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            tear_after: None,
            writes: 0,
        }
    }
}

impl<const N: usize> Default for MemNvm<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Nvm for MemNvm<N> {
    type Error = ();
    fn read(&mut self, addr: u16, buf: &mut [u8]) -> Result<(), ()> {
        let a = addr as usize;
        buf.copy_from_slice(self.data.get(a..a + buf.len()).ok_or(())?);
        Ok(())
    }
    fn write(&mut self, addr: u16, data: &[u8]) -> Result<(), ()> {
        let a = addr as usize;
        let n = self
            .tear_after
            .take()
            .map_or(data.len(), |k| k.min(data.len()));
        self.data
            .get_mut(a..a + n)
            .ok_or(())?
            .copy_from_slice(&data[..n]);
        self.writes += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(v: u8) -> [u8; PAYLOAD_LEN] {
        [v; PAYLOAD_LEN]
    }

    #[test]
    fn blank_then_alternating_slots() {
        let mut m = MemNvm::<1024>::new();
        assert_eq!(load(&mut m, ODO_SLOTS).unwrap(), Loaded::Blank);
        for seq in 1..=5u32 {
            save(&mut m, ODO_SLOTS, seq, &payload(seq as u8)).unwrap();
            assert_eq!(
                load(&mut m, ODO_SLOTS).unwrap(),
                Loaded::Valid {
                    seq,
                    payload: payload(seq as u8)
                }
            );
        }
    }

    #[test]
    fn torn_write_keeps_previous_record() {
        let mut m = MemNvm::<1024>::new();
        save(&mut m, ODO_SLOTS, 7, &payload(7)).unwrap();
        save(&mut m, ODO_SLOTS, 8, &payload(8)).unwrap();
        for cut in 0..RECORD_LEN {
            let mut t = m.clone();
            t.tear_after = Some(cut);
            save(&mut t, ODO_SLOTS, 9, &payload(9)).unwrap();
            // slot of seq 9 is the one that held seq 7: seq 8 must survive
            assert_eq!(
                load(&mut t, ODO_SLOTS).unwrap(),
                Loaded::Valid {
                    seq: 8,
                    payload: payload(8)
                }
            );
        }
    }

    #[test]
    fn garbage_is_corrupt_not_blank() {
        let mut m = MemNvm::<1024>::new();
        m.data[0x40] = 0x12;
        m.data[0x80 + 5] = 0x34;
        assert_eq!(load(&mut m, ODO_SLOTS).unwrap(), Loaded::Corrupt);
    }

    #[test]
    fn sequence_wraparound() {
        let mut m = MemNvm::<1024>::new();
        save(&mut m, ODO_SLOTS, u32::MAX, &payload(1)).unwrap();
        save(&mut m, ODO_SLOTS, 0, &payload(2)).unwrap();
        assert_eq!(
            load(&mut m, ODO_SLOTS).unwrap(),
            Loaded::Valid {
                seq: 0,
                payload: payload(2)
            }
        );
    }
}
