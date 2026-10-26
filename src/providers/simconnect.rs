//! Diagnostic-only SimConnect experiment. Production acquisition remains issue #6.
//!
//! The native library is loaded only by an explicitly started probe worker.
mod event;
mod journal;
mod lifecycle;
mod native;
mod probe;

pub use probe::{ProbeConfig, ProbeError, run_probe};
