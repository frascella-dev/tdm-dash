//! Measurement and odometer logic that runs unchanged on the RP2040 and in the
//! desktop simulator: pulse period measurement, input debouncing, odometer
//! state, power-fail-safe FRAM records and the co-processor's message handling.

#![no_std]

pub mod debounce;
pub mod mcu;
pub mod nvm;
pub mod pulse;

pub use debounce::Debouncer;
pub use mcu::{McuCore, Poll};
pub use nvm::{MemNvm, Nvm};
pub use pulse::{PulseConfig, PulseMeter};

/// Odometer and trip counters in micrometres, so per-pulse distance never
/// accumulates rounding error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OdoState {
    pub odo_um: u64,
    pub trip_um: [u64; 2],
}

impl OdoState {
    pub fn add_distance(&mut self, um: u32) {
        let um = u64::from(um);
        self.odo_um += um;
        for trip in &mut self.trip_um {
            *trip += um;
        }
    }

    pub fn odo_m(&self) -> u32 {
        (self.odo_um / 1_000_000) as u32
    }

    pub fn trip_m(&self, idx: usize) -> u32 {
        (self.trip_um[idx] / 1_000_000) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_accumulates_on_all_counters() {
        let mut odo = OdoState::default();
        for _ in 0..1000 {
            odo.add_distance(1_975_000 / 6);
        }
        assert_eq!(odo.odo_m(), 329);
        assert_eq!(odo.trip_um[0], odo.odo_um);
        assert_eq!(odo.trip_m(1), 329);
    }
}
