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
        "SimConnect HTTP/SSE integration belongs to issue #6; use the application's --source simconnect mode for acquisition without HTTP, or --source demo for the server"
    )]
    SimConnectUnavailable,
}

pub mod simconnect;

pub(crate) fn validate_source(source: Source) -> Result<(), SourceError> {
    match source {
        Source::Demo => Ok(()),
        Source::SimConnect => Err(SourceError::SimConnectUnavailable),
    }
}
