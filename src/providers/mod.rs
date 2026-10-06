//! Explicit sources; SDK details stay behind this boundary.

pub mod demo;

/// Identifies the selected telemetry source, independently of startup configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Demo,
    SimConnect,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(
        "SimConnect integration is not implemented yet (issues #5 and #6); explicitly select --source demo to start the HTTP skeleton"
    )]
    SimConnectUnavailable,
}

#[cfg(all(target_os = "windows", feature = "simconnect"))]
mod simconnect;

pub(crate) fn validate_source(source: Source) -> Result<(), SourceError> {
    match source {
        Source::Demo => Ok(()),
        Source::SimConnect => Err(SourceError::SimConnectUnavailable),
    }
}
