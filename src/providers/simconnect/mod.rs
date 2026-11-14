//! SimConnect acquisition shared by the application and its optional diagnostics.
//!
//! Only explicit SimConnect selection loads native code. HTTP integration is #6.
mod event;
mod instruments;
mod journal;
mod lifecycle;
mod mapping;
mod native;
mod pe;
mod runtime;

pub use journal::DiagnosticStatus;
pub use mapping::Definition;
pub use runtime::{
    RunReport, SimConnectConfig, SimConnectError, load_definition, resolve_dll, run,
    run_with_definition, source_kind, validate_library, validate_scenario,
};
