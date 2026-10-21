//! Dashboard logic: bike profile, conversions, filters, warnings and the UI
//! state machine. No I/O: link events and time go in, a view model and the
//! co-processor commands come out.

pub mod conv;
pub mod dash;
pub mod filters;
pub mod fmt;
pub mod profile;
pub mod warnings;

pub use dash::{Dash, Level, LinkEvent, LinkState, Screen, Theme, UiState, ViewModel};
pub use profile::{Layout, Profile, ThemeMode};
pub use warnings::{Icon, Severity, WarningId};
