#![forbid(unsafe_code)]

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

const AGENT_KIND: &str = "opencode";
const CAP_RUN_V1: &str = "agent_api.run";
const CAP_EVENTS_V1: &str = "agent_api.events";
const CAP_EVENTS_LIVE_V1: &str = "agent_api.events.live";
const CAP_SESSION_RESUME_V1: &str = "agent_api.session.resume.v1";
const CAP_SESSION_FORK_V1: &str = "agent_api.session.fork.v1";

mod backend;
mod harness;
mod mapping;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct OpencodeBackendConfig {
    pub binary: Option<PathBuf>,
    pub default_timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct OpencodeBackend {
    config: OpencodeBackendConfig,
}

impl OpencodeBackend {
    pub fn new(config: OpencodeBackendConfig) -> Self {
        Self { config }
    }
}
