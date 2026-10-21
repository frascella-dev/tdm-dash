//! Period measurement for the wheel sensor and the ignition pickup.

use tdm_proto::PulseStats;

/// Edges older than this are never needed: 4 wheel revolutions at walking
/// pace still fit, and a gap this long means the source has stopped.
const STOPPED_US: u64 = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PulseConfig {
    /// Edges per measurement group (wheel: pulses per revolution; ignition: 1).
    pub group: u16,
    /// The window grows by whole groups until it spans at least this long.
    pub min_window_us: u32,
    /// Upper bound on the window, in groups.
    pub max_groups: u16,
    /// Edges closer than this to the previous one are rejected as noise.
    pub min_gap_us: u32,
    /// Blanking after an edge as a percentage of the last period (ignition
    /// ringing); 0 disables it.
    pub blank_pct: u8,
    /// A gap longer than this restarts the window (source stopped).
    pub reset_us: u32,
}

/// Timestamps accepted edges and reports statistics over whole groups.
///
/// `N` must be at least `group * max_groups + 1`.
#[derive(Debug, Clone)]
pub struct PulseMeter<const N: usize> {
    cfg: PulseConfig,
    edges: [u64; N],
    head: usize,
    len: usize,
    total: u32,
    rejected: u32,
    last_period: u64,
}

impl<const N: usize> PulseMeter<N> {
    pub const fn new(cfg: PulseConfig) -> Self {
        Self {
            cfg,
            edges: [0; N],
            head: 0,
            len: 0,
            total: 0,
            rejected: 0,
            last_period: 0,
        }
    }

    pub fn configure(&mut self, cfg: PulseConfig) {
        assert!(
            (cfg.group as usize) * (cfg.max_groups as usize) < N,
            "PulseMeter buffer too small"
        );
        self.cfg = cfg;
        self.len = 0;
        self.last_period = 0;
    }

    pub fn config(&self) -> PulseConfig {
        self.cfg
    }

    /// Edges rejected by the minimum gap or the blanking window.
    pub fn rejected(&self) -> u32 {
        self.rejected
    }

    pub fn total(&self) -> u32 {
        self.total
    }

    fn at(&self, back: usize) -> u64 {
        // back = 0 is the newest edge
        self.edges[(self.head + N - 1 - back) % N]
    }

    /// Records an edge at `t_us`; returns `false` if it was rejected.
    pub fn on_edge(&mut self, t_us: u64) -> bool {
        if self.len > 0 {
            let gap = t_us.saturating_sub(self.at(0));
            let blank = (self.cfg.blank_pct as u64 * self.last_period) / 100;
            if gap < (self.cfg.min_gap_us as u64).max(blank) {
                self.rejected = self.rejected.wrapping_add(1);
                return false;
            }
            if gap > self.cfg.reset_us as u64 {
                self.len = 0;
            }
            self.last_period = if gap > STOPPED_US { 0 } else { gap };
        }
        self.edges[self.head] = t_us;
        self.head = (self.head + 1) % N;
        self.len = (self.len + 1).min(N);
        self.total = self.total.wrapping_add(1);
        true
    }

    pub fn stats(&self, now_us: u64) -> PulseStats {
        let age = if self.len > 0 {
            now_us.saturating_sub(self.at(0))
        } else {
            u32::MAX as u64
        };
        let mut s = PulseStats {
            total: self.total,
            span_edges: 0,
            span_us: 0,
            age_us: clamp_u32(age),
        };
        let g = self.cfg.group.max(1) as usize;
        if self.len < g + 1 || age > self.cfg.reset_us as u64 {
            return s;
        }
        let mut m = 1;
        while m < self.cfg.max_groups as usize
            && g * (m + 1) < self.len
            && self.at(0) - self.at(g * m) < self.cfg.min_window_us as u64
        {
            m += 1;
        }
        let k = g * m;
        s.span_edges = k as u16;
        s.span_us = clamp_u32(self.at(0) - self.at(k));
        s
    }
}

fn clamp_u32(v: u64) -> u32 {
    v.min(u32::MAX as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHEEL: PulseConfig = PulseConfig {
        group: 6,
        min_window_us: 100_000,
        max_groups: 4,
        min_gap_us: 2_500,
        blank_pct: 0,
        reset_us: 1_500_000,
    };
    const IGN: PulseConfig = PulseConfig {
        group: 1,
        min_window_us: 40_000,
        max_groups: 8,
        min_gap_us: 2_400,
        blank_pct: 40,
        reset_us: 500_000,
    };

    /// Deterministic jitter source (xorshift).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % 10_000) as f64 / 10_000.0
        }
    }

    fn speed_kmh(s: &PulseStats, mm_per_pulse: f64) -> f64 {
        s.span_edges as f64 * mm_per_pulse / s.span_us as f64 * 3600.0
    }

    #[test]
    fn constant_speed_with_uneven_bolts_and_jitter() {
        let mut pm = PulseMeter::<32>::new(WHEEL);
        let mm = 1975.0 / 6.0;
        let v_true = 87.0; // km/h
        let mut rng = Rng(42);
        let spacing: [f64; 6] = [1.02, 0.98, 1.015, 0.985, 1.01, 0.99]; // ±2 %, sums to 6
        let mut t = 0.0f64; // µs
        for i in 0..200 {
            let dist = mm * spacing[i % 6];
            t += dist / (v_true / 3.6) * 1000.0;
            assert!(pm.on_edge((t + (rng.next() - 0.5) * 100.0) as u64));
        }
        let s = pm.stats(t as u64 + 1000);
        assert_eq!(s.span_edges % 6, 0, "window must cover whole revolutions");
        let v = speed_kmh(&s, mm);
        assert!((v - v_true).abs() / v_true < 0.002, "measured {v}");
    }

    #[test]
    fn window_grows_at_low_speed_and_stops_at_max() {
        let mut pm = PulseMeter::<32>::new(WHEEL);
        let mut t = 0u64;
        // 5 km/h: 329 mm per edge every ~237 ms; one revolution > 100 ms
        for _ in 0..30 {
            t += 237_000;
            pm.on_edge(t);
        }
        assert_eq!(pm.stats(t).span_edges, 6);
        // 200 km/h: ~5.9 ms per edge, one revolution 35 ms → 3 revolutions
        for _ in 0..30 {
            t += 5_925;
            pm.on_edge(t);
        }
        assert_eq!(pm.stats(t).span_edges, 18);
    }

    #[test]
    fn stopped_source_reports_no_window() {
        let mut pm = PulseMeter::<32>::new(WHEEL);
        let mut t = 0u64;
        for _ in 0..20 {
            t += 10_000;
            pm.on_edge(t);
        }
        assert!(pm.stats(t + 1_000).span_edges > 0);
        assert_eq!(pm.stats(t + 2_000_000).span_edges, 0);
        // restart after a long stop: old edges must not stretch the window
        t += 3_000_000;
        for _ in 0..7 {
            t += 20_000;
            pm.on_edge(t);
        }
        let s = pm.stats(t);
        assert_eq!(s.span_edges, 6);
        assert_eq!(s.span_us, 120_000);
    }

    #[test]
    fn ignition_ringing_is_blanked() {
        let mut pm = PulseMeter::<32>::new(IGN);
        let mut t = 0u64;
        let period = 60_000_000 / 6000; // 6000 rpm, 1 pulse per revolution
        let mut accepted = 0;
        for i in 0..100 {
            t += period;
            if pm.on_edge(t) {
                accepted += 1;
            }
            if i % 3 == 0 {
                // ringing 400 µs after the spark
                assert!(!pm.on_edge(t + 400));
            }
            if i >= 2 && i % 5 == 0 {
                // a late spurious edge at 30 % of the period is also blanked
                assert!(!pm.on_edge(t + period * 3 / 10));
            }
        }
        assert_eq!(accepted, 100);
        let s = pm.stats(t);
        let rpm = s.span_edges as f64 / s.span_us as f64 * 60e6;
        assert!((rpm - 6000.0).abs() < 1.0);
    }

    #[test]
    fn cranking_after_stop_is_measured() {
        let mut pm = PulseMeter::<32>::new(IGN);
        let mut t = 0u64;
        for _ in 0..5 {
            t += 200_000; // 300 rpm cranking: no blanking reference
            pm.on_edge(t);
        }
        let s = pm.stats(t);
        assert!(s.span_edges >= 1);
    }
}
