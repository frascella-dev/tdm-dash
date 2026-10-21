//! Bike profile: everything that depends on the motorcycle (see
//! `docs/adr/0008` and `profiles/*.toml`).

use std::path::Path;

use serde::Deserialize;
use tdm_proto::McuConfig;

use crate::i18n::Language;

/// The TDM 850 3VD profile, compiled in as the last fallback.
pub const BUILTIN_TOML: &str = include_str!("../../../profiles/yamaha-tdm850-3vd.toml");

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: u32,
    pub bike: Bike,
    pub speed: Speed,
    pub tach: Tach,
    pub coolant: Coolant,
    pub battery: Battery,
    pub inputs: Inputs,
    pub display: Display,
    #[serde(default)]
    pub fuel: Option<Fuel>,
    #[serde(default)]
    pub board: Board,
    #[serde(default)]
    pub odometer: OdometerInit,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bike {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Speed {
    pub pulses_per_rev: u8,
    pub wheel_circumference_mm: f64,
    pub display_gain: f64,
    pub display_offset_kmh: f64,
    pub zero_below_kmh: f64,
    pub timeout_ms: u32,
    pub max_kmh: f64,
    pub dial_max_kmh: u32,
    pub units: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tach {
    pub pulses_per_rev: f64,
    pub max_rpm: u32,
    pub redline_rpm: u32,
    pub shift_rpm: u32,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Coolant {
    /// `[ohm, °C]` pairs, resistance decreasing.
    pub curve: Vec<(f64, f64)>,
    pub warn_c: f64,
    pub critical_c: f64,
    pub hysteresis_c: f64,
    pub gauge_min_c: f64,
    pub gauge_max_c: f64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Battery {
    pub low_v: f64,
    pub not_charging_v: f64,
    pub not_charging_min_rpm: f64,
    pub high_v: f64,
    pub critical_high_v: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InputMap {
    pub ch: u8,
    #[serde(default)]
    pub invert: bool,
}

impl InputMap {
    pub fn active(&self, word: u16) -> bool {
        ((word >> self.ch) & 1 == 1) != self.invert
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Inputs {
    pub neutral: InputMap,
    pub high_beam: InputMap,
    pub turn_left: InputMap,
    pub turn_right: InputMap,
    pub illumination: InputMap,
    pub btn_mode: InputMap,
    pub btn_set: InputMap,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    #[default]
    Classic,
    Sport,
    Touring,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Day,
    Night,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NightSource {
    LightSensor,
    Illumination,
    Manual,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Display {
    pub layout: Layout,
    pub theme: ThemeMode,
    pub night_source: NightSource,
    pub light_raw_dark: f64,
    pub light_raw_bright: f64,
    pub night_below: f64,
    pub day_above: f64,
    pub brightness_min_pct: f64,
    pub brightness_max_pct: f64,
    pub rotate_180: bool,
    pub selftest: bool,
    /// Language of the dashboard texts; English when absent.
    #[serde(default)]
    pub language: Language,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Fuel {
    pub tank_l: f64,
    pub reserve_l: f64,
    pub l_per_100km: f64,
}

/// Electrical constants of the I/O board.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Board {
    pub coolant_pullup_ohm: f64,
    /// Battery divider ratio (100 k / 22 k → 122/22).
    pub vbat_divider: f64,
    pub adc_vref: f64,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            coolant_pullup_ohm: 100.0,
            vbat_divider: 122.0 / 22.0,
            adc_vref: 3.3,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct OdometerInit {
    #[serde(default)]
    pub initial_km: u32,
}

#[derive(Debug)]
pub enum ProfileError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Invalid(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Io(e) => write!(f, "cannot read profile: {e}"),
            ProfileError::Toml(e) => write!(f, "profile syntax: {e}"),
            ProfileError::Invalid(s) => write!(f, "invalid profile: {s}"),
        }
    }
}

impl std::error::Error for ProfileError {}

impl Profile {
    pub fn parse(text: &str) -> Result<Self, ProfileError> {
        let p: Profile = toml::from_str(text).map_err(ProfileError::Toml)?;
        p.validate()?;
        Ok(p)
    }

    pub fn builtin() -> Self {
        Self::parse(BUILTIN_TOML).expect("built-in profile is valid")
    }

    pub fn load(path: &Path) -> Result<Self, ProfileError> {
        Self::parse(&std::fs::read_to_string(path).map_err(ProfileError::Io)?)
    }

    /// First readable profile among `paths`; on any error falls back to the
    /// built-in profile and returns the error so the UI can show it.
    pub fn load_layered(paths: &[&Path]) -> (Self, Option<String>) {
        for p in paths {
            if !p.exists() {
                continue;
            }
            return match Self::load(p) {
                Ok(prof) => (prof, None),
                Err(e) => (Self::builtin(), Some(format!("{}: {e}", p.display()))),
            };
        }
        (Self::builtin(), None)
    }

    fn validate(&self) -> Result<(), ProfileError> {
        let bad = |m: &str| Err(ProfileError::Invalid(m.to_string()));
        if self.schema != 1 {
            return bad("unsupported schema (expected 1)");
        }
        if self.speed.pulses_per_rev == 0 || self.speed.pulses_per_rev > 15 {
            return bad("speed.pulses_per_rev must be 1–15");
        }
        if !(500.0..4000.0).contains(&self.speed.wheel_circumference_mm) {
            return bad("speed.wheel_circumference_mm out of range");
        }
        // UNECE R39: the displayed speed may never be below the true speed.
        if self.speed.display_gain < 1.0 || self.speed.display_offset_kmh < 0.0 {
            return bad("speed.display_gain must be ≥ 1 and display_offset_kmh ≥ 0");
        }
        if self.speed.units != "metric" {
            return bad("only metric units are supported");
        }
        if self.tach.pulses_per_rev <= 0.0 || self.tach.redline_rpm >= self.tach.max_rpm {
            return bad("tach: pulses_per_rev > 0 and redline < max_rpm required");
        }
        let c = &self.coolant.curve;
        if c.len() < 2
            || c.windows(2).any(|w| w[1].0 >= w[0].0 || w[1].1 <= w[0].1)
            || c.iter().any(|p| p.0 <= 0.0)
        {
            return bad(
                "coolant.curve must have ≥ 2 points, resistance decreasing and temperature increasing",
            );
        }
        if self.coolant.warn_c >= self.coolant.critical_c {
            return bad("coolant.warn_c must be below critical_c");
        }
        if self.battery.high_v >= self.battery.critical_high_v
            || self.battery.low_v >= self.battery.not_charging_v
        {
            return bad("battery thresholds out of order");
        }
        if self.display.light_raw_dark <= 0.0
            || self.display.light_raw_bright <= self.display.light_raw_dark
        {
            return bad("display light anchors out of order");
        }
        Ok(())
    }

    pub fn mm_per_pulse(&self) -> f64 {
        self.speed.wheel_circumference_mm / self.speed.pulses_per_rev as f64
    }

    /// Measurement settings for the co-processor.
    pub fn mcu_config(&self) -> McuConfig {
        let mm = self.mm_per_pulse();
        let vmax_mm_per_us = self.speed.max_kmh / 3.6 / 1000.0;
        let ign_period_min_us = 60e6 / (self.tach.max_rpm as f64 * self.tach.pulses_per_rev);
        McuConfig {
            um_per_pulse: (mm * 1000.0).round() as u32,
            wheel_ppr: self.speed.pulses_per_rev,
            wheel_min_gap_us: (0.5 * mm / vmax_mm_per_us) as u32,
            wheel_min_window_us: 100_000,
            ign_min_gap_us: (0.4 * ign_period_min_us) as u32,
            ign_blank_pct: 40,
            ign_min_window_us: 40_000,
            debounce_ms: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_in_the_repo_is_valid() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "toml") {
                Profile::load(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                n += 1;
            }
        }
        assert!(n >= 2);
    }

    #[test]
    fn language_defaults_to_english() {
        assert_eq!(Profile::builtin().display.language, Language::It);
        let t = BUILTIN_TOML.replace("language = \"it\"", "");
        assert_eq!(Profile::parse(&t).unwrap().display.language, Language::En);
        let t = BUILTIN_TOML.replace("language = \"it\"", "language = \"de\"");
        assert!(Profile::parse(&t).is_err());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let t = BUILTIN_TOML.replace("[speed]", "[speed]\ntypo_field = 1");
        assert!(matches!(Profile::parse(&t), Err(ProfileError::Toml(_))));
    }

    #[test]
    fn gain_below_one_is_rejected() {
        let t = BUILTIN_TOML.replace("display_gain = 1.03", "display_gain = 0.98");
        assert!(matches!(Profile::parse(&t), Err(ProfileError::Invalid(_))));
    }

    #[test]
    fn layered_loading_falls_back_with_error() {
        let dir = std::env::temp_dir().join(format!("tdm-prof-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.toml");
        std::fs::write(&bad, "schema = 1\n[bike]\n").unwrap();
        let missing = dir.join("missing.toml");
        let (p, err) = Profile::load_layered(&[&missing, &bad]);
        assert_eq!(p, Profile::builtin());
        assert!(err.unwrap().contains("bad.toml"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn mcu_config_from_tdm_profile() {
        let c = Profile::builtin().mcu_config();
        assert_eq!(c.um_per_pulse, 329_167);
        assert_eq!(c.wheel_ppr, 6);
        // half a pulse at 240 km/h
        assert!((2400..2500).contains(&c.wheel_min_gap_us));
        assert_eq!(c.ign_min_gap_us, 2400);
    }
}
