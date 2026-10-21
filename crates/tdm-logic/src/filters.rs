//! Display filters. Time is passed in seconds so everything is testable
//! without a clock.

/// First-order low-pass with time constant `tau`.
#[derive(Debug, Clone, Copy)]
pub struct Ema {
    pub tau: f64,
    value: Option<f64>,
}

impl Ema {
    pub const fn new(tau: f64) -> Self {
        Self { tau, value: None }
    }

    pub fn update(&mut self, x: f64, dt: f64) -> f64 {
        let v = match self.value {
            None => x,
            Some(v) => v + (x - v) * (1.0 - (-dt / self.tau).exp()),
        };
        self.value = Some(v);
        v
    }

    pub fn value(&self) -> Option<f64> {
        self.value
    }

    pub fn reset(&mut self, x: Option<f64>) {
        self.value = x;
    }
}

/// Critically damped spring for needles: follows steps without overshoot.
#[derive(Debug, Clone, Copy, Default)]
pub struct Spring {
    pub x: f64,
    pub v: f64,
}

impl Spring {
    pub fn update(&mut self, target: f64, omega: f64, dt: f64) -> f64 {
        let mut left = dt;
        while left > 0.0 {
            let h = left.min(0.016);
            self.v += (omega * omega * (target - self.x) - 2.0 * omega * self.v) * h;
            self.x += self.v * h;
            left -= h;
        }
        self.x
    }

    pub fn reset(&mut self, x: f64) {
        self.x = x;
        self.v = 0.0;
    }
}

/// Integer quantiser with hysteresis, so the digits do not flicker between
/// two values.
#[derive(Debug, Clone, Copy, Default)]
pub struct HystQuantizer {
    pub band: f64,
    shown: i64,
}

impl HystQuantizer {
    pub const fn new(band: f64) -> Self {
        Self { band, shown: 0 }
    }

    pub fn update(&mut self, x: f64) -> i64 {
        if x > self.shown as f64 + self.band || x < self.shown as f64 - self.band {
            self.shown = x.round() as i64;
        }
        self.shown
    }

    pub fn set(&mut self, v: i64) {
        self.shown = v;
    }
}

/// Median of the last three samples, to drop single outliers.
#[derive(Debug, Clone, Copy, Default)]
pub struct Median3 {
    buf: [f64; 3],
    n: usize,
}

impl Median3 {
    pub fn update(&mut self, x: f64) -> f64 {
        self.buf.rotate_left(1);
        self.buf[2] = x;
        self.n = (self.n + 1).min(3);
        let mut s = [0.0; 3];
        s[..self.n].copy_from_slice(&self.buf[3 - self.n..]);
        let s = &mut s[..self.n];
        s.sort_by(|a, b| a.total_cmp(b));
        s[s.len() / 2]
    }

    pub fn reset(&mut self) {
        self.n = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ema_settles() {
        let mut e = Ema::new(0.3);
        e.update(0.0, 0.02);
        let mut v = 0.0;
        for _ in 0..100 {
            v = e.update(100.0, 0.02);
        }
        assert!((v - 100.0).abs() < 0.2);
    }

    #[test]
    fn spring_no_overshoot() {
        let mut s = Spring::default();
        let mut peak: f64 = 0.0;
        for _ in 0..200 {
            peak = peak.max(s.update(1.0, 10.0, 1.0 / 60.0));
        }
        assert!(peak <= 1.0 + 1e-6 && (s.x - 1.0).abs() < 1e-3);
    }

    #[test]
    fn quantizer_hysteresis() {
        let mut q = HystQuantizer::new(0.6);
        assert_eq!(q.update(50.2), 50);
        assert_eq!(q.update(50.55), 50);
        assert_eq!(q.update(49.5), 50);
        assert_eq!(q.update(50.7), 51);
        assert_eq!(q.update(50.45), 51);
    }

    #[test]
    fn median_drops_outlier() {
        let mut m = Median3::default();
        m.update(50.0);
        m.update(51.0);
        assert_eq!(m.update(300.0), 51.0);
        assert_eq!(m.update(52.0), 52.0);
    }
}
