//! Explicit sources; SDK details stay behind this boundary.

pub mod demo;

/// Identifies the selected telemetry source, independently of startup configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Demo,
    SimConnect,
}

pub mod simconnect;
