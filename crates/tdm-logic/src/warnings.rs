//! Warnings with on-delays, clear conditions and snooze (manual, cap. 11).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info = 1,
    Caution = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningId {
    LinkLost,
    CoolantCritical,
    Overvoltage,
    CoolantHigh,
    HighVoltage,
    NotCharging,
    BatteryLow,
    CoolantSensorFault,
    FwMismatch,
    ConfigError,
    OdometerFault,
    TurnSignalOn,
    FuelLow,
}

/// Which icon slot a warning lights up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Link,
    Coolant,
    Battery,
    System,
    Turn,
    Fuel,
}

impl WarningId {
    pub const ALL: [WarningId; 13] = [
        WarningId::LinkLost,
        WarningId::CoolantCritical,
        WarningId::Overvoltage,
        WarningId::CoolantHigh,
        WarningId::HighVoltage,
        WarningId::NotCharging,
        WarningId::BatteryLow,
        WarningId::CoolantSensorFault,
        WarningId::FwMismatch,
        WarningId::ConfigError,
        WarningId::OdometerFault,
        WarningId::TurnSignalOn,
        WarningId::FuelLow,
    ];

    pub fn severity(self) -> Severity {
        use WarningId::*;
        match self {
            LinkLost | CoolantCritical | Overvoltage => Severity::Critical,
            TurnSignalOn | FuelLow => Severity::Info,
            _ => Severity::Caution,
        }
    }

    pub fn icon(self) -> Icon {
        use WarningId::*;
        match self {
            LinkLost => Icon::Link,
            CoolantCritical | CoolantHigh | CoolantSensorFault => Icon::Coolant,
            Overvoltage | HighVoltage | NotCharging | BatteryLow => Icon::Battery,
            FwMismatch | ConfigError | OdometerFault => Icon::System,
            TurnSignalOn => Icon::Turn,
            FuelLow => Icon::Fuel,
        }
    }

    pub fn title(self) -> &'static str {
        use WarningId::*;
        match self {
            LinkLost => "Collegamento con il modulo sensori perso",
            CoolantCritical => "Temperatura acqua critica",
            Overvoltage => "Sovratensione",
            CoolantHigh => "Temperatura acqua alta",
            HighVoltage => "Tensione alta",
            NotCharging => "Batteria non in carica",
            BatteryLow => "Batteria bassa",
            CoolantSensorFault => "Sensore temperatura scollegato",
            FwMismatch => "Firmware del modulo non compatibile",
            ConfigError => "Errore nel profilo: uso quello integrato",
            OdometerFault => "Memoria del contachilometri guasta",
            TurnSignalOn => "Freccia accesa",
            FuelLow => "Riserva stimata",
        }
    }

    pub fn detail(self) -> &'static str {
        use WarningId::*;
        match self {
            LinkLost => "Velocità e giri non disponibili",
            CoolantCritical => "Fermati appena puoi e spegni il motore",
            Overvoltage => "Regolatore di tensione guasto: rischio per batteria e luci",
            NotCharging => "Controlla statore e regolatore · SET: silenzia",
            FwMismatch => "Aggiorna il firmware del Pico · SET: silenzia",
            ConfigError => "Controlla /boot/tdm-dash.toml · SET: silenzia",
            OdometerFault => "Vedi Diagnostica · SET: silenzia",
            _ => "SET: silenzia per 5 minuti",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    on: bool,
    raise_t: f64,
    clear_t: f64,
    since: f64,
    snooze_until: f64,
}

/// One evaluation of a warning's conditions.
#[derive(Debug, Clone, Copy)]
pub struct Cond {
    pub raise: bool,
    pub on_delay: f64,
    pub clear: bool,
    pub off_delay: f64,
}

impl Cond {
    pub fn new(raise: bool, on_delay: f64, clear: bool, off_delay: f64) -> Self {
        Self {
            raise,
            on_delay,
            clear,
            off_delay,
        }
    }

    /// Raised immediately, cleared as soon as the condition goes away.
    pub fn flag(active: bool) -> Self {
        Self::new(active, 0.0, !active, 0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Warnings {
    states: std::collections::HashMap<WarningId, State>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Active {
    pub id: WarningId,
    pub since: f64,
}

impl Warnings {
    pub fn update(&mut self, id: WarningId, c: Cond, now: f64, dt: f64) {
        let s = self.states.entry(id).or_default();
        if !s.on {
            s.raise_t = if c.raise { s.raise_t + dt } else { 0.0 };
            if c.raise && s.raise_t >= c.on_delay {
                s.on = true;
                s.since = now;
                s.clear_t = 0.0;
            }
        } else {
            s.clear_t = if c.clear { s.clear_t + dt } else { 0.0 };
            if c.clear && s.clear_t >= c.off_delay {
                s.on = false;
                s.raise_t = 0.0;
            }
        }
    }

    pub fn force_off(&mut self, id: WarningId) {
        if let Some(s) = self.states.get_mut(&id) {
            s.on = false;
        }
    }

    pub fn is_on(&self, id: WarningId) -> bool {
        self.states.get(&id).is_some_and(|s| s.on)
    }

    /// Hides a caution for 5 minutes; it comes back if it is raised again
    /// after clearing.
    pub fn snooze(&mut self, id: WarningId, now: f64) {
        if id.severity() == Severity::Caution
            && let Some(s) = self.states.get_mut(&id)
        {
            s.snooze_until = now + 300.0;
        }
    }

    /// Active warnings, most severe first, then most recent.
    pub fn active(&self, now: f64, include_snoozed: bool) -> Vec<Active> {
        let mut v: Vec<Active> = WarningId::ALL
            .iter()
            .filter_map(|&id| {
                let s = self.states.get(&id)?;
                let snoozed = id.severity() == Severity::Caution && s.snooze_until > now;
                (s.on && (include_snoozed || !snoozed)).then_some(Active { id, since: s.since })
            })
            .collect();
        v.sort_by(|a, b| {
            b.id.severity()
                .cmp(&a.id.severity())
                .then(b.since.total_cmp(&a.since))
        });
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_delay_and_hysteresis() {
        let mut w = Warnings::default();
        let mut t = 0.0;
        let step = |w: &mut Warnings, t: &mut f64, temp: f64| {
            *t += 0.1;
            w.update(
                WarningId::CoolantHigh,
                Cond::new(temp >= 105.0, 3.0, temp < 102.0, 0.0),
                *t,
                0.1,
            );
        };
        for _ in 0..25 {
            step(&mut w, &mut t, 106.0);
        }
        assert!(!w.is_on(WarningId::CoolantHigh), "needs 3 s");
        for _ in 0..10 {
            step(&mut w, &mut t, 106.0);
        }
        assert!(w.is_on(WarningId::CoolantHigh));
        step(&mut w, &mut t, 103.0);
        assert!(w.is_on(WarningId::CoolantHigh), "hysteresis");
        step(&mut w, &mut t, 101.0);
        assert!(!w.is_on(WarningId::CoolantHigh));
    }

    #[test]
    fn ordering_and_snooze() {
        let mut w = Warnings::default();
        w.update(WarningId::FuelLow, Cond::flag(true), 1.0, 0.1);
        w.update(WarningId::BatteryLow, Cond::flag(true), 2.0, 0.1);
        w.update(WarningId::LinkLost, Cond::flag(true), 3.0, 0.1);
        let ids: Vec<_> = w.active(3.0, false).iter().map(|a| a.id).collect();
        assert_eq!(
            ids,
            vec![
                WarningId::LinkLost,
                WarningId::BatteryLow,
                WarningId::FuelLow
            ]
        );
        w.snooze(WarningId::BatteryLow, 3.0);
        w.snooze(WarningId::LinkLost, 3.0); // critical: cannot be snoozed
        let ids: Vec<_> = w.active(4.0, false).iter().map(|a| a.id).collect();
        assert_eq!(ids, vec![WarningId::LinkLost, WarningId::FuelLow]);
        assert_eq!(w.active(400.0, false).len(), 3);
    }
}
