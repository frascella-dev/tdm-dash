//! Integrating debouncer for up to 16 digital inputs.

#[derive(Debug, Clone)]
pub struct Debouncer {
    stable: u16,
    elapsed_ms: [u16; 16],
    threshold_ms: [u16; 16],
}

impl Debouncer {
    pub fn new(default_ms: u16) -> Self {
        Self {
            stable: 0,
            elapsed_ms: [0; 16],
            threshold_ms: [default_ms; 16],
        }
    }

    pub fn set_threshold(&mut self, bit: usize, ms: u16) {
        self.threshold_ms[bit] = ms;
    }

    pub fn set_all(&mut self, ms: u16) {
        self.threshold_ms = [ms; 16];
    }

    /// Feeds the raw input word sampled `dt_ms` after the previous one.
    pub fn update(&mut self, raw: u16, dt_ms: u16) -> u16 {
        for bit in 0..16 {
            let mask = 1u16 << bit;
            if (raw ^ self.stable) & mask == 0 {
                self.elapsed_ms[bit] = 0;
                continue;
            }
            self.elapsed_ms[bit] = self.elapsed_ms[bit].saturating_add(dt_ms);
            if self.elapsed_ms[bit] >= self.threshold_ms[bit] {
                self.stable ^= mask;
                self.elapsed_ms[bit] = 0;
            }
        }
        self.stable
    }

    pub fn stable(&self) -> u16 {
        self.stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounces_are_ignored_and_steady_changes_pass() {
        let mut d = Debouncer::new(20);
        d.set_threshold(0, 50);
        for i in 0..30 {
            d.update(if i % 2 == 0 { 0b11 } else { 0 }, 1);
        }
        assert_eq!(d.stable(), 0);
        for _ in 0..19 {
            d.update(0b11, 1);
        }
        assert_eq!(d.stable(), 0);
        d.update(0b11, 1);
        assert_eq!(d.stable(), 0b10, "bit 1 after 20 ms, bit 0 still waiting");
        for _ in 0..30 {
            d.update(0b11, 1);
        }
        assert_eq!(d.stable(), 0b11);
    }
}
