//! Manager init configuration resolution and detection helpers.

mod config;
mod runtime;

#[cfg(test)]
mod tests;

pub use config::{ManagerInitConfig, ManifestPaths};
pub use runtime::{
    detect_and_generate, telemetry_payload, write_snippet, ManagerInitResult, ManagerState,
};
