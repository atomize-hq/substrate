#![forbid(unsafe_code)]

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

const AGENT_KIND: &str = "gemini_cli";
const CAP_RUN_V1: &str = "agent_api.run";
const CAP_EVENTS_V1: &str = "agent_api.events";
const CAP_EVENTS_LIVE_V1: &str = "agent_api.events.live";
const CHANNEL_ASSISTANT: &str = "assistant";
const CHANNEL_TOOL: &str = "tool";

mod backend;
mod harness;
mod mapping;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct GeminiCliBackendConfig {
    pub binary: Option<PathBuf>,
    pub default_timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct GeminiCliBackend {
    config: GeminiCliBackendConfig,
}

impl GeminiCliBackend {
    pub fn new(config: GeminiCliBackendConfig) -> Self {
        Self { config }
    }
}
