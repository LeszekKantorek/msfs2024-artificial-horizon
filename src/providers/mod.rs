//! Source availability boundary. Acquisition is added in issues #2 and #6.

use crate::config::{ConfigError, Source};

#[cfg(all(target_os = "windows", feature = "simconnect"))]
mod simconnect;

pub(crate) fn validate_source(source: Source) -> Result<(), ConfigError> {
    match source {
        Source::Demo => Ok(()),
        Source::SimConnect => Err(ConfigError::SimConnectUnavailable),
    }
}
