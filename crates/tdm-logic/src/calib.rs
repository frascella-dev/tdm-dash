//! Calibration from measurements on the bike (manual, chapter 10).
//!
//! Each function takes the value currently in the profile and what was
//! measured, and returns the corrected value; `tdmctl calibrate` prints it
//! as TOML to paste into `/boot/tdm-dash.toml`. Corrections far from 1 are
//! refused: they mean a wrong measurement or a wrong setting (pulses per
//! revolution), not a calibration.

/// Largest correction accepted for wheel and battery (±10 %).
pub const MAX_CORRECTION: f64 = 0.10;
/// Shortest GPS distance for a wheel calibration, in km.
pub const MIN_DISTANCE_KM: f64 = 5.0;

fn ratio_ok(r: f64, what: &str) -> Result<f64, String> {
    if !r.is_finite() || r <= 0.0 {
        return Err(format!("{what}: valori non validi"));
    }
    if (r - 1.0).abs() > MAX_CORRECTION {
        return Err(format!(
            "{what}: correzione del {:+.1} %, oltre il ±{:.0} %: controlla le misure",
            (r - 1.0) * 100.0,
            MAX_CORRECTION * 100.0
        ));
    }
    Ok(r)
}

/// Rolling circumference from a GPS distance and the trip counted by the
/// dashboard over the same ride. The trip uses only the circumference (not
/// the display gain), so the correction is the plain ratio.
pub fn wheel_circumference(current_mm: f64, gps_km: f64, trip_km: f64) -> Result<f64, String> {
    if gps_km < MIN_DISTANCE_KM {
        return Err(format!(
            "servono almeno {MIN_DISTANCE_KM:.0} km misurati col GPS"
        ));
    }
    let r = ratio_ok(gps_km / trip_km, "circonferenza")?;
    Ok(current_mm * r)
}

/// Ignition pulses per crankshaft revolution from a reference reading (the
/// original rev counter, or a workshop tachometer) and the one shown.
/// Returns the exact value and the nearest half-integer, which is what an
/// ignition really gives (1 per revolution on the TDM, 0.5 on a single).
pub fn tach_pulses_per_rev(
    current: f64,
    reference_rpm: f64,
    shown_rpm: f64,
) -> Result<(f64, f64), String> {
    if reference_rpm <= 0.0 || shown_rpm <= 0.0 {
        return Err("giri: valori non validi".into());
    }
    // shown = f * 60 / current, reference = f * 60 / exact
    let exact = current * shown_rpm / reference_rpm;
    let snapped = (exact * 2.0).round() / 2.0;
    if snapped <= 0.0 {
        return Err("giri: il risultato è zero, controlla le misure".into());
    }
    Ok((exact, snapped))
}

/// Battery divider ratio from the voltage shown and the one measured with a
/// multimeter on the battery terminals at the same moment.
pub fn vbat_divider(current: f64, actual_v: f64, shown_v: f64) -> Result<f64, String> {
    let r = ratio_ok(actual_v / shown_v, "tensione")?;
    Ok(current * r)
}

/// Coolant curve from bath measurements given as (°C, ohm), in any order and
/// with repeated temperatures averaged. Returns the profile format: pairs
/// (ohm, °C) with resistance decreasing.
pub fn coolant_curve(points: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, String> {
    let mut by_temp: Vec<(f64, f64, u32)> = Vec::new();
    for &(t, ohm) in points {
        if !(t.is_finite() && ohm.is_finite()) || ohm <= 0.0 {
            return Err(format!("misura non valida: {t} °C, {ohm} Ω"));
        }
        match by_temp.iter_mut().find(|e| (e.0 - t).abs() < 0.05) {
            Some(e) => {
                e.1 += ohm;
                e.2 += 1;
            }
            None => by_temp.push((t, ohm, 1)),
        }
    }
    if by_temp.len() < 3 {
        return Err("servono almeno 3 temperature diverse".into());
    }
    by_temp.sort_by(|a, b| a.0.total_cmp(&b.0));
    let curve: Vec<(f64, f64)> = by_temp
        .iter()
        .map(|&(t, sum, n)| (sum / n as f64, t))
        .collect();
    if let Some(w) = curve.windows(2).find(|w| w[1].0 >= w[0].0) {
        return Err(format!(
            "la resistenza deve scendere al salire della temperatura: {:.1} Ω a {} °C, {:.1} Ω a {} °C",
            w[0].0, w[0].1, w[1].0, w[1].1
        ));
    }
    Ok(curve)
}

/// Reads bath measurements: one "°C ohm" pair per line, separated by
/// spaces, tabs or ';' (or a single ','), decimal comma accepted, '#'
/// starts a comment.
pub fn parse_measurements(text: &str) -> Result<Vec<(f64, f64)>, String> {
    let mut out = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = if line.contains([';', '\t', ' ']) {
            line.split([';', '\t', ' '])
                .filter(|f| !f.is_empty())
                .collect()
        } else {
            line.split(',').collect()
        };
        let num = |s: &str| s.trim().replace(',', ".").parse::<f64>();
        match fields.as_slice() {
            [t, r] => match (num(t), num(r)) {
                (Ok(t), Ok(r)) => out.push((t, r)),
                _ => return Err(format!("riga {}: numeri non validi: {raw}", n + 1)),
            },
            _ => {
                return Err(format!(
                    "riga {}: servono due valori (°C e ohm): {raw}",
                    n + 1
                ));
            }
        }
    }
    Ok(out)
}

/// The coolant curve as a TOML line for the profile.
pub fn curve_toml(curve: &[(f64, f64)]) -> String {
    let pts: Vec<String> = curve
        .iter()
        .map(|(ohm, t)| format!("[{ohm:.1}, {t:.0}]"))
        .collect();
    format!("curve = [{}]", pts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;

    #[test]
    fn wheel_ratio() {
        let c = wheel_circumference(1960.0, 10.20, 10.00).unwrap();
        assert!((c - 1999.2).abs() < 0.01);
        assert!(wheel_circumference(1960.0, 3.0, 3.0).is_err(), "too short");
        assert!(
            wheel_circumference(1960.0, 12.0, 10.0).is_err(),
            "20 % is not a calibration"
        );
    }

    /// Replaces the line starting with `key` in the built-in profile.
    fn with_line(key: &str, line: &str) -> String {
        let old = crate::profile::BUILTIN_TOML
            .lines()
            .find(|l| l.starts_with(key))
            .unwrap();
        crate::profile::BUILTIN_TOML.replace(old, line)
    }

    #[test]
    fn calibrated_wheel_is_accepted_by_the_profile() {
        let p = Profile::builtin();
        let c = wheel_circumference(p.speed.wheel_circumference_mm, 10.1, 10.0).unwrap();
        let q = Profile::parse(&with_line(
            "wheel_circumference_mm",
            &format!("wheel_circumference_mm = {c:.1}"),
        ))
        .unwrap();
        assert!((q.speed.wheel_circumference_mm - c).abs() < 0.05);
    }

    #[test]
    fn tach_snaps_to_half() {
        // a single shows twice the real revs with 1 pulse per revolution
        let (exact, snapped) = tach_pulses_per_rev(1.0, 1500.0, 3000.0).unwrap();
        assert!((exact - 2.0).abs() < 1e-9);
        assert_eq!(snapped, 2.0);
        let (_, snapped) = tach_pulses_per_rev(1.0, 1200.0, 610.0).unwrap();
        assert_eq!(snapped, 0.5);
    }

    #[test]
    fn battery_ratio() {
        let k = vbat_divider(122.0 / 22.0, 12.62, 12.48).unwrap();
        assert!((k - 122.0 / 22.0 * 12.62 / 12.48).abs() < 1e-9);
        assert!(vbat_divider(5.5, 12.0, 6.0).is_err());
    }

    #[test]
    fn coolant_from_bath() {
        let m = parse_measurements(
            "# bagnomaria\n20;1050\n40 ; 452\n60\t198,5\n80;51\n80;49\n100,27.5\n",
        )
        .unwrap();
        let c = coolant_curve(&m).unwrap();
        assert_eq!(c.len(), 5);
        assert_eq!(c[0], (1050.0, 20.0));
        assert_eq!(c[3], (50.0, 80.0), "repeated temperatures are averaged");
        assert_eq!(
            curve_toml(&c),
            "curve = [[1050.0, 20], [452.0, 40], [198.5, 60], [50.0, 80], [27.5, 100]]"
        );
        // and the profile accepts it
        let p = Profile::builtin();
        let q = Profile::parse(&with_line("curve =", &curve_toml(&c))).unwrap();
        assert_eq!(q.coolant.curve.len(), 5);
        assert_eq!(p.coolant.warn_c, q.coolant.warn_c);
    }

    #[test]
    fn coolant_rejects_bad_data() {
        assert!(
            coolant_curve(&[(20.0, 1000.0), (40.0, 450.0)]).is_err(),
            "too few"
        );
        assert!(
            coolant_curve(&[(20.0, 100.0), (40.0, 450.0), (60.0, 30.0)]).is_err(),
            "not monotonic"
        );
        assert!(parse_measurements("20 1000 7").is_err());
        assert!(parse_measurements("venti mille").is_err());
    }
}
