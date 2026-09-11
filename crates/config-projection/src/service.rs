use std::sync::Arc;

use crate::{
    AgentInventorySourceMaterialV1, ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1,
    ConfigProjectionRegistryV1, ConfiguredAcceptedHomeAuthorityV1,
    EffectiveSubstrateConfigSourceV1, Timestamp,
};

pub struct AgentConfigProjectionServiceV1 {
    registry: Arc<ConfigProjectionRegistryV1>,
    accepted_home: Arc<ConfiguredAcceptedHomeAuthorityV1>,
}

impl AgentConfigProjectionServiceV1 {
    pub fn new(
        registry: Arc<ConfigProjectionRegistryV1>,
        accepted_home: Arc<ConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self, ConfigProjectionFailureV1> {
        accepted_home.revalidate()?;
        let store = registry.recover()?;
        if store.accepted_home != *accepted_home.accepted_home() {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(Self {
            registry,
            accepted_home,
        })
    }

    pub fn publish_retained_launch_inputs(
        &self,
        effective_config: &EffectiveSubstrateConfigSourceV1,
        agent_inventory: &AgentInventorySourceMaterialV1,
        created_at: Timestamp,
    ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
        self.accepted_home.revalidate()?;
        self.registry
            .import_runtime_artifacts(effective_config, agent_inventory, created_at)
    }

    pub fn publish_retained_fork_inputs(
        &self,
        effective_config: &EffectiveSubstrateConfigSourceV1,
        agent_inventory: &AgentInventorySourceMaterialV1,
        created_at: Timestamp,
    ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
        self.publish_retained_launch_inputs(effective_config, agent_inventory, created_at)
    }
}
