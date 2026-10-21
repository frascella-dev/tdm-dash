//! Dashboard texts in English and Italian (docs/adr/0012).
//!
//! The language comes from `[display] language` in the profile. Every text
//! is a [`Msg`] variant and both [`en`] and [`it`] match on all of them, so
//! a new text does not compile until it has both translations. Texts with
//! values are the functions at the end of the file.

use serde::Deserialize;

use crate::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    It,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    // fixed labels of the Slint screens
    RpmUnit,
    KRpmUnit,
    FillShort,
    EstimatedRange,
    RangeCaps,
    RidingCaps,
    AverageCaps,
    TripTitle,
    TripA,
    SinceFill,
    RidingTime,
    Average,
    TopSpeed,
    EstConsumption,
    FuelUsed,
    Range,
    TripKeys,
    DiagTitle,
    SettingsTitle,
    OnlyStopped,
    SettingsKeys,
    // settings
    SetLayout,
    SetTheme,
    SetBrightness,
    SetDiagScreen,
    SetSpeedCal,
    SetTachCal,
    SetExit,
    ThemeAuto,
    ThemeDay,
    ThemeNight,
    Shown,
    Hidden,
    // toasts
    OdoAlreadySet,
    TripResetFailed,
    CommandRefused,
    WarningSnoozed,
    TripAReset,
    FillRecorded,
    SettingsOnlyStopped,
    EditInProfile,
    // info strip and diagnostics
    TurnSignal,
    DiagNote,
    DiagWheel,
    DiagIgnition,
    DiagBattery,
    DiagCoolant,
    DiagLight,
    DiagInputs,
    DiagLink,
    DiagBoot,
    DiagFirmware,
    DiagProfile,
    SensorFault,
    LinkOk,
    LinkStale,
    LinkLost,
    WaitingModule,
    FramOk,
    FramBlank,
    FramCorrupt,
    FramMissing,
    // warnings (title, detail)
    WLinkLost,
    WLinkLostDetail,
    WCoolantCritical,
    WCoolantCriticalDetail,
    WOvervoltage,
    WOvervoltageDetail,
    WCoolantHigh,
    WHighVoltage,
    WNotCharging,
    WNotChargingDetail,
    WBatteryLow,
    WCoolantSensorFault,
    WFwMismatch,
    WFwMismatchDetail,
    WConfigError,
    WConfigErrorDetail,
    WOdometerFault,
    WOdometerFaultDetail,
    WTurnSignalOn,
    WFuelLow,
    WSnoozeDetail,
}

pub fn tr(lang: Language, m: Msg) -> &'static str {
    match lang {
        Language::En => en(m),
        Language::It => it(m),
    }
}

pub fn en(m: Msg) -> &'static str {
    use Msg::*;
    match m {
        RpmUnit => "rpm",
        KRpmUnit => "×1000 rpm",
        FillShort => "FILL",
        EstimatedRange => "estimated range",
        RangeCaps => "RANGE",
        RidingCaps => "RIDING",
        AverageCaps => "AVERAGE",
        TripTitle => "TRIP",
        TripA => "TRIP A",
        SinceFill => "SINCE FILL-UP",
        RidingTime => "Riding time",
        Average => "Average",
        TopSpeed => "Top speed",
        EstConsumption => "Est. consumption",
        FuelUsed => "Fuel used",
        Range => "Range",
        TripKeys => "SET: select · hold SET: reset · MODE: next screen",
        DiagTitle => "DIAGNOSTICS",
        SettingsTitle => "SETTINGS",
        OnlyStopped => "only when stopped",
        SettingsKeys => "MODE: next item · SET: change · hold SET: exit",
        SetLayout => "Riding layout",
        SetTheme => "Theme",
        SetBrightness => "Brightness",
        SetDiagScreen => "Diagnostics screen",
        SetSpeedCal => "Speed calibration",
        SetTachCal => "Tachometer calibration",
        SetExit => "Exit",
        ThemeAuto => "Auto",
        ThemeDay => "Day",
        ThemeNight => "Night",
        Shown => "Shown",
        Hidden => "Hidden",
        OdoAlreadySet => "Odometer already set",
        TripResetFailed => "Trip reset failed",
        CommandRefused => "Command refused by the module",
        WarningSnoozed => "Warning snoozed for 5 minutes",
        TripAReset => "Trip A reset",
        FillRecorded => "Fill-up recorded",
        SettingsOnlyStopped => "Settings only when stopped",
        EditInProfile => "Edit in the profile /boot/tdm-dash.toml",
        TurnSignal => "Turn signal",
        DiagNote => "Front tyre changed? Recalibrate the speed (manual, ch. 10)",
        DiagWheel => "Wheel",
        DiagIgnition => "Ignition",
        DiagBattery => "Battery",
        DiagCoolant => "Coolant",
        DiagLight => "Light",
        DiagInputs => "Inputs",
        DiagLink => "Link",
        DiagBoot => "Boot",
        DiagFirmware => "Firmware",
        DiagProfile => "Profile",
        SensorFault => "sensor open or shorted",
        LinkOk => "ok",
        LinkStale => "late",
        LinkLost => "lost",
        WaitingModule => "waiting for the module",
        FramOk => "FRAM ok",
        FramBlank => "FRAM blank",
        FramCorrupt => "FRAM corrupt",
        FramMissing => "FRAM missing",
        WLinkLost => "Sensor module link lost",
        WLinkLostDetail => "Speed and rpm not available",
        WCoolantCritical => "Coolant temperature critical",
        WCoolantCriticalDetail => "Stop as soon as you can and switch off the engine",
        WOvervoltage => "Overvoltage",
        WOvervoltageDetail => "Faulty voltage regulator: risk for battery and lights",
        WCoolantHigh => "Coolant temperature high",
        WHighVoltage => "High voltage",
        WNotCharging => "Battery not charging",
        WNotChargingDetail => "Check stator and regulator · SET: snooze",
        WBatteryLow => "Battery low",
        WCoolantSensorFault => "Coolant sender disconnected",
        WFwMismatch => "Module firmware not compatible",
        WFwMismatchDetail => "Update the Pico firmware · SET: snooze",
        WConfigError => "Profile error: using the built-in profile",
        WConfigErrorDetail => "Check /boot/tdm-dash.toml · SET: snooze",
        WOdometerFault => "Odometer memory fault",
        WOdometerFaultDetail => "See Diagnostics · SET: snooze",
        WTurnSignalOn => "Turn signal on",
        WFuelLow => "Estimated reserve",
        WSnoozeDetail => "SET: snooze for 5 minutes",
    }
}

pub fn it(m: Msg) -> &'static str {
    use Msg::*;
    match m {
        RpmUnit => "giri/min",
        KRpmUnit => "×1000 giri/min",
        FillShort => "PIENO",
        EstimatedRange => "autonomia stimata",
        RangeCaps => "AUTONOMIA",
        RidingCaps => "IN SELLA",
        AverageCaps => "MEDIA",
        TripTitle => "VIAGGIO",
        TripA => "PARZIALE A",
        SinceFill => "DAL PIENO",
        RidingTime => "In sella",
        Average => "Media",
        TopSpeed => "Massima",
        EstConsumption => "Consumo stimato",
        FuelUsed => "Benzina usata",
        Range => "Autonomia",
        TripKeys => "SET: scegli · SET lungo: azzera · MODE: schermata successiva",
        DiagTitle => "DIAGNOSTICA",
        SettingsTitle => "IMPOSTAZIONI",
        OnlyStopped => "solo da fermi",
        SettingsKeys => "MODE: voce successiva · SET: cambia · SET lungo: esci",
        SetLayout => "Layout di guida",
        SetTheme => "Tema",
        SetBrightness => "Luminosità",
        SetDiagScreen => "Schermata diagnostica",
        SetSpeedCal => "Calibrazione velocità",
        SetTachCal => "Calibrazione contagiri",
        SetExit => "Esci",
        ThemeAuto => "Automatico",
        ThemeDay => "Giorno",
        ThemeNight => "Notte",
        Shown => "Attiva",
        Hidden => "Nascosta",
        OdoAlreadySet => "Contachilometri già impostato",
        TripResetFailed => "Azzeramento non riuscito",
        CommandRefused => "Comando rifiutato dal modulo",
        WarningSnoozed => "Avviso silenziato per 5 minuti",
        TripAReset => "Parziale A azzerato",
        FillRecorded => "Pieno registrato",
        SettingsOnlyStopped => "Impostazioni disponibili solo da fermi",
        EditInProfile => "Si modifica nel profilo /boot/tdm-dash.toml",
        TurnSignal => "Freccia",
        DiagNote => "Gomma anteriore cambiata? Ricalibra la velocità (manuale, cap. 10)",
        DiagWheel => "Ruota",
        DiagIgnition => "Accensione",
        DiagBattery => "Batteria",
        DiagCoolant => "Acqua",
        DiagLight => "Luce",
        DiagInputs => "Ingressi",
        DiagLink => "Collegamento",
        DiagBoot => "Avvio",
        DiagFirmware => "Firmware",
        DiagProfile => "Profilo",
        SensorFault => "sensore aperto o in corto",
        LinkOk => "attivo",
        LinkStale => "in ritardo",
        LinkLost => "perso",
        WaitingModule => "in attesa del modulo",
        FramOk => "FRAM ok",
        FramBlank => "FRAM vuota",
        FramCorrupt => "FRAM corrotta",
        FramMissing => "FRAM assente",
        WLinkLost => "Collegamento con il modulo sensori perso",
        WLinkLostDetail => "Velocità e giri non disponibili",
        WCoolantCritical => "Temperatura acqua critica",
        WCoolantCriticalDetail => "Fermati appena puoi e spegni il motore",
        WOvervoltage => "Sovratensione",
        WOvervoltageDetail => "Regolatore di tensione guasto: rischio per batteria e luci",
        WCoolantHigh => "Temperatura acqua alta",
        WHighVoltage => "Tensione alta",
        WNotCharging => "Batteria non in carica",
        WNotChargingDetail => "Controlla statore e regolatore · SET: silenzia",
        WBatteryLow => "Batteria bassa",
        WCoolantSensorFault => "Sensore temperatura scollegato",
        WFwMismatch => "Firmware del modulo non compatibile",
        WFwMismatchDetail => "Aggiorna il firmware del Pico · SET: silenzia",
        WConfigError => "Errore nel profilo: uso quello integrato",
        WConfigErrorDetail => "Controlla /boot/tdm-dash.toml · SET: silenzia",
        WOdometerFault => "Memoria del contachilometri guasta",
        WOdometerFaultDetail => "Vedi Diagnostica · SET: silenzia",
        WTurnSignalOn => "Freccia accesa",
        WFuelLow => "Riserva stimata",
        WSnoozeDetail => "SET: silenzia per 5 minuti",
    }
}

// ---------------------------------------------------------------- texts with values

/// Speed calibration value in Settings.
pub fn speed_cal(lang: Language, circumference_mm: f64, ppr: u8) -> String {
    let mm = fmt::int(lang, circumference_mm as u64);
    match lang {
        Language::En => format!("{mm} mm · {ppr} pulses/rev"),
        Language::It => format!("{mm} mm · {ppr} impulsi/giro"),
    }
}

/// Tachometer calibration value in Settings.
pub fn tach_cal(lang: Language, ppr: f64, redline: u32) -> String {
    let (p, r) = (fmt::dec(lang, ppr, 1), fmt::int(lang, redline as u64));
    match lang {
        Language::En => format!("{p} pulses/rev · red {r}"),
        Language::It => format!("{p} impulsi/giro · rosso {r}"),
    }
}

/// Diagnostics, wheel row.
pub fn diag_wheel(lang: Language, hz: f64, pulses: u16, span_ms: f64) -> String {
    let (h, s) = (fmt::dec(lang, hz, 1), fmt::dec(lang, span_ms, 1));
    match lang {
        Language::En => format!("{h} Hz · {pulses} pulses in {s} ms"),
        Language::It => format!("{h} Hz · {pulses} impulsi in {s} ms"),
    }
}

/// Diagnostics, ignition row.
pub fn diag_ignition(lang: Language, hz: f64, rpm: f64) -> String {
    format!(
        "{} Hz · {} {}",
        fmt::dec(lang, hz, 1),
        fmt::int(lang, rpm as u64),
        tr(lang, Msg::RpmUnit)
    )
}

/// Diagnostics, coolant row when colder than the first point of the curve.
pub fn diag_below_curve(lang: Language, ohm: f64) -> String {
    let o = fmt::dec(lang, ohm, 1);
    match lang {
        Language::En => format!("{o} Ω → below the curve"),
        Language::It => format!("{o} Ω → sotto la curva"),
    }
}

/// Diagnostics, light row.
pub fn diag_light(lang: Language, raw: u16, pct: f64, backlight_pct: f64) -> String {
    match lang {
        Language::En => format!("ADC {raw} · {pct:.0} % · backlight {backlight_pct:.0} %"),
        Language::It => format!("ADC {raw} · {pct:.0} % · retroill. {backlight_pct:.0} %"),
    }
}

/// Diagnostics, inputs row: 0/1 for neutral, high beam, left, right, lights, MODE, SET.
pub fn diag_inputs(lang: Language, v: [u8; 7]) -> String {
    let [n, hb, l, r, lights, mode, set] = v;
    match lang {
        Language::En => {
            format!("N {n} · HB {hb} · L {l} · R {r} · LIGHTS {lights} · MODE {mode} · SET {set}")
        }
        Language::It => {
            format!("N {n} · ABB {hb} · SX {l} · DX {r} · LUCI {lights} · MODE {mode} · SET {set}")
        }
    }
}

/// Diagnostics, link row.
pub fn diag_link(lang: Language, state: &str, frames: u64, lost: u32, reconnects: u32) -> String {
    match lang {
        Language::En => {
            format!("{state} · {frames} frames · {lost} lost · reconnections {reconnects}")
        }
        Language::It => {
            format!("{state} · {frames} frame · {lost} persi · riconnessioni {reconnects}")
        }
    }
}

/// Diagnostics, boot row.
pub fn diag_boot(lang: Language, secs: f64) -> String {
    let s = fmt::dec(lang, secs, 2);
    match lang {
        Language::En => format!("{s} s from key to first frame"),
        Language::It => format!("{s} s dalla chiave al primo frame"),
    }
}

/// Diagnostics, firmware row.
pub fn diag_firmware(lang: Language, fw: [u8; 3], proto: u16, fram: &str) -> String {
    let [a, b, c] = fw;
    match lang {
        Language::En => format!("Pico {a}.{b}.{c} · protocol {proto} · {fram}"),
        Language::It => format!("Pico {a}.{b}.{c} · protocollo {proto} · {fram}"),
    }
}

/// Splash screen footer.
pub fn splash_profile(lang: Language, bike: &str, version: &str) -> String {
    match lang {
        Language::En => format!("profile {bike} · v{version}"),
        Language::It => format!("profilo {bike} · v{version}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_parse_from_profile_values() {
        #[derive(Deserialize)]
        struct T {
            l: Language,
        }
        let p = |s: &str| toml::from_str::<T>(&format!("l = \"{s}\"")).map(|t| t.l);
        assert_eq!(p("en").unwrap(), Language::En);
        assert_eq!(p("it").unwrap(), Language::It);
        assert!(p("de").is_err());
    }

    #[test]
    fn texts_with_values() {
        assert_eq!(
            speed_cal(Language::En, 1975.0, 6),
            "1,975 mm · 6 pulses/rev"
        );
        assert_eq!(
            speed_cal(Language::It, 1975.0, 6),
            "1 975 mm · 6 impulsi/giro"
        );
        assert_eq!(
            diag_ignition(Language::En, 52.5, 3150.0),
            "52.5 Hz · 3,150 rpm"
        );
        assert_eq!(
            diag_ignition(Language::It, 52.5, 3150.0),
            "52,5 Hz · 3 150 giri/min"
        );
    }
}
