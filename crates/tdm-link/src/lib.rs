//! Links between the dashboard and the co-processor: USB serial with
//! automatic reconnection, the vehicle simulator and recorded sessions.

pub mod sim;
pub mod source;

use std::path::PathBuf;

pub use sim::{Scenario, SimMcu, scenario, scenarios};
pub use source::{ReplaySource, SerialSource, SimSource, Source, find_ports};
use tdm_logic::Profile;

/// Parsed `--source` argument: `sim[:scenario]`, `serial[:/dev/ttyACMx]` or
/// `replay:FILE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSpec {
    Sim(String),
    Serial(Option<PathBuf>),
    Replay(PathBuf),
}

impl std::str::FromStr for SourceSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let (kind, arg) = s.split_once(':').map_or((s, None), |(k, a)| (k, Some(a)));
        match kind {
            "sim" => {
                let id = arg.unwrap_or("city");
                scenario(id)
                    .map(|_| SourceSpec::Sim(id.to_string()))
                    .ok_or_else(|| {
                        let ids: Vec<_> = scenarios().iter().map(|s| s.id).collect();
                        format!("unknown scenario '{id}' (available: {})", ids.join(", "))
                    })
            }
            "serial" => Ok(SourceSpec::Serial(arg.map(PathBuf::from))),
            "replay" => arg
                .map(|a| SourceSpec::Replay(a.into()))
                .ok_or_else(|| "replay needs a file".into()),
            _ => Err(format!(
                "unknown source '{s}' (sim[:scenario], serial[:device], replay:file)"
            )),
        }
    }
}

pub fn open(
    spec: &SourceSpec,
    profile: &Profile,
    record: Option<PathBuf>,
) -> std::io::Result<Box<dyn Source>> {
    Ok(match spec {
        SourceSpec::Sim(id) => Box::new(SimSource::new(
            profile,
            scenario(id).expect("validated"),
            0x7d3,
        )),
        SourceSpec::Serial(p) => Box::new(SerialSource::spawn(p.clone(), record)),
        SourceSpec::Replay(p) => Box::new(ReplaySource::open(p)?),
    })
}
