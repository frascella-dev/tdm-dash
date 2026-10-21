//! Conversions from MCU readings to physical values.

use tdm_proto::PulseStats;

use crate::profile::Profile;

/// Speed in km/h from wheel statistics; `None` for an implausible reading.
///
/// While no new edge arrives the estimate is capped by "one pulse distance
/// over the time since the last edge", so braking to a stop shows a falling
/// speed instead of the last measured one.
pub fn speed_kmh(p: &Profile, s: &PulseStats) -> Option<f64> {
    if s.span_edges == 0 || s.age_us > p.speed.timeout_ms * 1000 {
        return Some(0.0);
    }
    let mm = p.mm_per_pulse();
    let mut v = s.span_edges as f64 * mm / s.span_us as f64; // mm/µs
    let avg_period = s.span_us as f64 / s.span_edges as f64;
    if s.age_us as f64 > avg_period {
        v = v.min(mm / s.age_us as f64);
    }
    let kmh = v * 3600.0;
    (kmh <= p.speed.max_kmh).then_some(kmh)
}

pub fn rpm(p: &Profile, s: &PulseStats) -> Option<f64> {
    if s.span_edges == 0 || s.age_us > p.tach.timeout_ms * 1000 {
        return Some(0.0);
    }
    let mut hz = s.span_edges as f64 / s.span_us as f64 * 1e6;
    let avg_period = s.span_us as f64 / s.span_edges as f64;
    if s.age_us as f64 > avg_period {
        hz = hz.min(1e6 / s.age_us as f64);
    }
    let r = hz * 60.0 / p.tach.pulses_per_rev;
    (r <= p.tach.max_rpm as f64 * 1.1).then_some(r)
}

/// Displayed speed: never below the true speed (UNECE R39).
pub fn displayed_kmh(p: &Profile, measured: f64) -> f64 {
    if measured < p.speed.zero_below_kmh {
        0.0
    } else {
        measured * p.speed.display_gain + p.speed.display_offset_kmh
    }
}

pub fn vbat_volts(p: &Profile, raw: u16) -> f64 {
    raw as f64 / 4095.0 * p.board.adc_vref * p.board.vbat_divider
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coolant {
    Celsius(f64),
    /// Colder than the first point of the curve.
    Low,
    /// Sender open or shorted.
    Fault,
}

/// Coolant sender read through a pull-up from the ADC reference.
pub fn coolant(p: &Profile, raw: u16) -> (Coolant, Option<f64>) {
    let r = raw as f64 / 4095.0;
    if !(0.02..=0.98).contains(&r) {
        return (Coolant::Fault, None);
    }
    let ohm = p.board.coolant_pullup_ohm * r / (1.0 - r);
    (temp_from_ohm(&p.coolant.curve, ohm), Some(ohm))
}

pub fn temp_from_ohm(curve: &[(f64, f64)], ohm: f64) -> Coolant {
    let (r0, _) = curve[0];
    if ohm > r0 {
        return Coolant::Low;
    }
    let last = curve[curve.len() - 1];
    if ohm <= last.0 {
        return Coolant::Celsius(last.1);
    }
    for w in curve.windows(2) {
        let ((ra, ta), (rb, tb)) = (w[0], w[1]);
        if ohm <= ra && ohm >= rb {
            let k = (ra.ln() - ohm.ln()) / (ra.ln() - rb.ln());
            return Coolant::Celsius(ta + (tb - ta) * k);
        }
    }
    Coolant::Fault
}

/// Inverse of [`temp_from_ohm`], used by the simulator.
pub fn ohm_from_temp(curve: &[(f64, f64)], t: f64) -> f64 {
    let first = curve[0];
    if t <= first.1 {
        return first.0 * ((first.1 - t) * 0.04).exp();
    }
    for w in curve.windows(2) {
        let ((ra, ta), (rb, tb)) = (w[0], w[1]);
        if t >= ta && t <= tb {
            let k = (t - ta) / (tb - ta);
            return (ra.ln() + (rb.ln() - ra.ln()) * k).exp();
        }
    }
    let last = curve[curve.len() - 1];
    last.0 * (-(t - last.1) * 0.02).exp()
}

/// Ambient light on a 0–1 log scale between the profile's anchors.
pub fn light_level(p: &Profile, raw: u16) -> f64 {
    let d = &p.display;
    ((raw.max(1) as f64 / d.light_raw_dark).ln() / (d.light_raw_bright / d.light_raw_dark).ln())
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_for(kmh: f64, p: &Profile) -> PulseStats {
        let mm = p.mm_per_pulse();
        let period_us = mm / (kmh / 3.6) * 1000.0;
        let k = p.speed.pulses_per_rev as u16 * 2;
        PulseStats {
            total: 0,
            span_edges: k,
            span_us: (period_us * k as f64) as u32,
            age_us: 100,
        }
    }

    #[test]
    fn displayed_speed_never_below_true_speed() {
        // UNECE R39: 0 ≤ V_shown − V_true ≤ 0.1 V_true + 4 km/h
        let p = Profile::builtin();
        let mut v = 5.0;
        while v <= 200.0 {
            let measured = speed_kmh(&p, &stats_for(v, &p)).unwrap();
            let shown = displayed_kmh(&p, measured);
            let digits = shown.round(); // worst case of the quantiser
            assert!(digits >= v, "{v} km/h shown as {digits}");
            assert!(shown - v <= 0.1 * v + 4.0, "{v} km/h shown as {shown}");
            v += 0.5;
        }
    }

    #[test]
    fn braking_decays_without_new_edges() {
        let p = Profile::builtin();
        let mut s = stats_for(60.0, &p);
        let v0 = speed_kmh(&p, &s).unwrap();
        s.age_us = 200_000;
        let v1 = speed_kmh(&p, &s).unwrap();
        assert!(v1 < v0 * 0.2, "{v0} → {v1}");
        s.age_us = 2_000_000;
        assert_eq!(speed_kmh(&p, &s), Some(0.0));
    }

    #[test]
    fn coolant_curve_roundtrip_and_faults() {
        let p = Profile::builtin();
        for t in [45.0, 80.0, 95.0, 110.0] {
            let ohm = ohm_from_temp(&p.coolant.curve, t);
            let raw = (4095.0 * ohm / (ohm + p.board.coolant_pullup_ohm)).round() as u16;
            match coolant(&p, raw).0 {
                Coolant::Celsius(x) => assert!((x - t).abs() < 0.6, "{t} → {x}"),
                other => panic!("{t}: {other:?}"),
            }
        }
        assert_eq!(coolant(&p, 4090).0, Coolant::Fault);
        assert_eq!(coolant(&p, 3).0, Coolant::Fault);
        assert_eq!(temp_from_ohm(&p.coolant.curve, 5000.0), Coolant::Low);
    }

    #[test]
    fn battery_scale() {
        let p = Profile::builtin();
        let raw = (13.9 / p.board.vbat_divider / 3.3 * 4095.0).round() as u16;
        assert!((vbat_volts(&p, raw) - 13.9).abs() < 0.01);
    }
}
