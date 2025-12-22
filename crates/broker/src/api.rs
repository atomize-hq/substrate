//! Global broker API surface and singleton orchestration.

use crate::handle::BrokerHandle;
use crate::policy::Decision;
use anyhow::Result;
use std::path::Path;
use std::sync::OnceLock;
use substrate_common::WorldFsMode;
use tracing::warn;

static GLOBAL_BROKER: OnceLock<BrokerHandle> = OnceLock::new();

pub fn set_global_broker(broker: BrokerHandle) -> Result<()> {
    if GLOBAL_BROKER.get().is_some() {
        return Ok(());
    }
    GLOBAL_BROKER
        .set(broker)
        .map_err(|_| anyhow::anyhow!("Global broker already initialized"))?;
    Ok(())
}

fn global_broker() -> Result<BrokerHandle> {
    GLOBAL_BROKER
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Broker not initialized; call set_global_broker first"))
}

pub fn init(config_path: Option<&Path>) -> Result<()> {
    let broker = GLOBAL_BROKER.get().cloned().unwrap_or_default();
    broker.initialize(config_path)?;
    set_global_broker(broker)?;
    Ok(())
}

pub fn evaluate(cmd: &str, cwd: &str, world_id: Option<&str>) -> Result<Decision> {
    let broker = global_broker()?;
    broker.evaluate(cmd, cwd, world_id)
}

pub fn quick_check(argv: &[String], cwd: &str) -> Result<Decision> {
    let broker = global_broker()?;
    broker.quick_check(argv, cwd)
}

pub fn detect_profile(cwd: &Path) -> Result<()> {
    let broker = global_broker()?;
    broker.detect_profile(cwd)
}

pub fn reload_policy(path: &Path) -> Result<()> {
    let broker = global_broker()?;
    broker.load_policy(path)
}

pub fn set_observe_only(observe: bool) {
    match global_broker() {
        Ok(broker) => broker.set_observe_only(observe),
        Err(err) => {
            warn!("Failed to set observe_only on global broker: {}", err);
        }
    }
}

pub fn allowed_domains() -> Vec<String> {
    match global_broker() {
        Ok(broker) => broker.allowed_domains(),
        Err(err) => {
            warn!(
                "Allowed domains requested before broker initialization: {}",
                err
            );
            Vec::new()
        }
    }
}

pub fn world_fs_mode() -> WorldFsMode {
    match global_broker() {
        Ok(broker) => broker.world_fs_mode(),
        Err(err) => {
            warn!(
                "world_fs_mode requested before broker initialization: {}",
                err
            );
            WorldFsMode::Writable
        }
    }
}
