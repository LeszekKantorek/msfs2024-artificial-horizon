//! Reusable configuration and HTTP application, independent of CLI arguments.

#[cfg(not(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc")))]
compile_error!("Only the x86_64-pc-windows-msvc target is supported.");

pub mod config;
pub mod http;
pub mod providers;
pub mod telemetry;

pub use config::{Config, ConfigError};
pub use http::{Server, ServerError};
pub use providers::{Source, SourceError};
