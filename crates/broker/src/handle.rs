//! Safe handle for interacting with shared broker state.

use crate::broker::Broker;
use crate::mode::PolicyMode;
use crate::policy::Decision;
use crate::policy::WorldFsPolicy;
use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, RwLock};
use substrate_common::WorldFsMode;

#[derive(Clone, Default)]
pub struct BrokerHandle {
    broker: Arc<RwLock<Broker>>,
}

impl BrokerHandle {
    pub fn new() -> Self {
        Self {
            broker: Arc::new(RwLock::new(Broker::new())),
        }
    }

    pub fn initialize(&self, config_path: Option<&Path>) -> Result<()> {
        if let Some(path) = config_path {
            self.load_policy(path)?;
        }
        self.apply_enforcement_env();
        Ok(())
    }

    pub fn load_policy(&self, path: &Path) -> Result<()> {
        let broker = self
            .broker
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire broker write lock: {}", e))?;
        broker.load_policy(path)
    }

    pub fn detect_profile(&self, cwd: &Path) -> Result<()> {
        let broker = self
            .broker
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire broker write lock: {}", e))?;
        broker.detect_and_load_policy(cwd)
    }

    pub fn evaluate(&self, cmd: &str, cwd: &str, world_id: Option<&str>) -> Result<Decision> {
        let broker = self
            .broker
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire broker read lock: {}", e))?;
        broker.evaluate(cmd, cwd, world_id)
    }

    pub fn quick_check(&self, argv: &[String], cwd: &str) -> Result<Decision> {
        let broker = self
            .broker
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire broker read lock: {}", e))?;
        broker.quick_check(argv, cwd)
    }

    pub fn set_observe_only(&self, observe: bool) {
        if let Ok(broker) = self.broker.read() {
            broker.set_observe_only(observe);
        }
    }

    pub fn set_policy_mode(&self, mode: PolicyMode) {
        if let Ok(broker) = self.broker.read() {
            broker.set_policy_mode(mode);
        }
    }

    pub fn policy_mode(&self) -> PolicyMode {
        self.broker
            .read()
            .map(|b| b.policy_mode())
            .unwrap_or_else(|_| PolicyMode::from_env())
    }

    pub fn is_observe_only(&self) -> bool {
        self.broker
            .read()
            .map(|b| b.is_observe_only())
            .unwrap_or(true)
    }

    pub fn allowed_domains(&self) -> Vec<String> {
        self.broker
            .read()
            .map(|b| b.allowed_domains())
            .unwrap_or_default()
    }

    pub fn world_fs_mode(&self) -> WorldFsMode {
        self.broker
            .read()
            .map(|b| b.world_fs_mode())
            .unwrap_or(WorldFsMode::Writable)
    }

    pub fn world_fs_policy(&self) -> WorldFsPolicy {
        self.broker
            .read()
            .map(|b| b.world_fs_policy())
            .unwrap_or_else(|_| crate::Policy::default().world_fs_policy())
    }

    fn apply_enforcement_env(&self) {
        self.set_policy_mode(PolicyMode::from_env());
    }
}
