//! SimConnect acquisition shared by the application and its optional diagnostics.
//!
//! Only explicit SimConnect selection loads native code. HTTP integration is #6.
mod event;
mod journal;
mod lifecycle;
mod native;
mod runtime;

pub use journal::DiagnosticStatus;
pub use runtime::{RunReport, SimConnectConfig, SimConnectError, resolve_dll, run};
