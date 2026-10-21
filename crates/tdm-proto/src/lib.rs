//! Wire protocol between the RP2040 co-processor and the Linux UI.
//!
//! Frames are `postcard(payload) ‖ CRC-16` COBS-encoded and terminated by
//! `0x00`. The CRC also covers a direction tag that is never transmitted, so a
//! frame echoed back by a tty in cooked mode fails the check on the other side.
//! The message set is implemented in milestone M2 (see `docs/adr/0002`).

#![cfg_attr(not(feature = "std"), no_std)]

/// Bumped on every breaking change. Variants and fields are append-only
/// between bumps.
pub const PROTO_VERSION: u16 = 1;

/// Largest encoded frame, delimiter included.
pub const MAX_FRAME: usize = 128;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_tags_differ() {
        assert_ne!(Dir::McuToHost.tag(), Dir::HostToMcu.tag());
    }
}
