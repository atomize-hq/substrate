//! Pure complete-selection resolution. Source acquisition and permission checks
//! remain with their existing owners; this module performs neither I/O nor auth.

use std::collections::{BTreeMap, BTreeSet};
use transport_api_types::{
    AgentConnectionProfileV1, AgentHarnessFamilyV1, AgentSelectionOriginV1,
    AgentSelectionPlacementV1, AgentSelectionSourceScopeV1, AgentSelectionSourceV1,
    AgentSelectionTupleV1, ResolvedAgentSelectionV1,
};

#[derive(Clone, Debug)]
pub(crate) struct ConfiguredAgentPlacementV3 {
    pub enabled: bool,
    pub runtime_family: AgentHarnessFamilyV1,
    pub selection: AgentSelectionTupleV1,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfiguredAgentDefinitionV3 {
    pub logical_agent_id: String,
    pub enabled: bool,
    pub source: AgentSelectionSourceV1,
    pub placements: BTreeMap<AgentSelectionPlacementV1, ConfiguredAgentPlacementV3>,
}

pub(crate) struct AgentSelectionConfigV1<'a> {
    pub preference: Option<&'a AgentSelectionTupleV1>,
    pub preference_source: Option<&'a AgentSelectionSourceV1>,
    pub profiles: &'a [AgentConnectionProfileV1],
    pub sources: &'a [AgentSelectionSourceV1],
}

pub(crate) struct AgentSelectionRequestV1<'a> {
    pub agent_id: Option<&'a str>,
    pub backend_id: Option<&'a str>,
    pub placement: AgentSelectionPlacementV1,
}

/// Merge logical definitions before projecting enabled placements. Input order
/// never selects a winner within a root, or makes global override workspace.
pub(crate) fn visible_agent_definitions_v3(
    definitions: &[ConfiguredAgentDefinitionV3],
) -> Result<BTreeMap<&str, &ConfiguredAgentDefinitionV3>, String> {
    let mut seen = BTreeMap::new();
    for definition in definitions {
        definition.source.validate()?;
        if definition.source.scope == AgentSelectionSourceScopeV1::BuiltIn {
            return Err("inventory definitions require a global or workspace source".into());
        }
        let key = (
            definition.source.scope,
            definition.logical_agent_id.as_str(),
        );
        if let Some(previous) = seen.insert(key, &definition.source) {
            return Err(format!(
                "duplicate agent '{}' in {} sources '{}' and '{}'",
                definition.logical_agent_id,
                definition.source.scope.as_str(),
                previous.relative_path,
                definition.source.relative_path,
            ));
        }
        for placement in definition.placements.values() {
            placement.selection.validate()?;
            if placement.runtime_family != placement.selection.harness {
                return Err(format!(
                    "agent '{}' in '{}' has a selection/runtime_family conflict",
                    definition.logical_agent_id, definition.source.relative_path
                ));
            }
        }
    }
    let mut visible = BTreeMap::new();
    for scope in [
        AgentSelectionSourceScopeV1::Global,
        AgentSelectionSourceScopeV1::Workspace,
    ] {
        for definition in definitions
            .iter()
            .filter(|entry| entry.source.scope == scope)
        {
            visible.insert(definition.logical_agent_id.as_str(), definition);
        }
    }
    Ok(visible)
}

pub(crate) fn resolve_agent_selection_v1(
    definitions: &[ConfiguredAgentDefinitionV3],
    config: &AgentSelectionConfigV1<'_>,
    request: &AgentSelectionRequestV1<'_>,
) -> Result<ResolvedAgentSelectionV1, String> {
    let visible = visible_agent_definitions_v3(definitions)?;
    let mut profiles = BTreeMap::new();
    for profile in config.profiles {
        profile.validate()?;
        if profiles.insert(profile.id.as_str(), profile).is_some() {
            return Err(format!("duplicate connection profile '{}'", profile.id));
        }
        if !config.sources.contains(&profile.source) {
            return Err(format!(
                "connection profile '{}' is not bound to its captured source '{}'",
                profile.id, profile.source.relative_path
            ));
        }
    }
    let explicit = request.agent_id.is_some() || request.backend_id.is_some();
    let preference = if explicit {
        None
    } else {
        let preference = config
            .preference
            .ok_or("agents.defaults.selection must be a complete preference")?;
        preference.validate()?;
        let source = config
            .preference_source
            .ok_or("default selection preference has no captured source")?;
        if !config.sources.contains(source) {
            return Err("default selection preference source is not captured".into());
        }
        Some(preference)
    };
    let mut matches = Vec::new();
    for definition in visible.values().filter(|entry| entry.enabled) {
        let Some(placement) = definition.placements.get(&request.placement) else {
            continue;
        };
        if !placement.enabled {
            continue;
        }
        let realized = format!(
            "{}-{}",
            definition.logical_agent_id,
            request.placement.as_str()
        );
        let backend = format!("cli:{realized}");
        if request
            .agent_id
            .is_some_and(|id| id != definition.logical_agent_id && id != realized)
            || request.backend_id.is_some_and(|id| id != backend)
            || preference.is_some_and(|expected| expected != &placement.selection)
        {
            continue;
        }
        matches.push((*definition, placement, realized, backend));
    }
    if matches.len() != 1 {
        let sources = definitions
            .iter()
            .map(|entry| entry.source.relative_path.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "agent selection requires exactly one enabled {} placement; found {} (sources: {})",
            request.placement.as_str(),
            matches.len(),
            sources
        ));
    }
    let (definition, placement, realized, backend) = matches.pop().expect("one selected placement");
    let tuple = &placement.selection;
    let profile = profiles
        .get(tuple.connection_profile.as_str())
        .ok_or_else(|| {
            format!(
                "agent '{}' in '{}' references missing connection profile '{}'",
                definition.logical_agent_id,
                definition.source.relative_path,
                tuple.connection_profile
            )
        })?;
    let (harness_revision, route_revision, routed_model) = match tuple.harness {
        AgentHarnessFamilyV1::Codex => (
            "codex-cli.v1",
            "codex-subscription.v1",
            tuple.model.as_str(),
        ),
        AgentHarnessFamilyV1::ClaudeCode => (
            "claude-code-cli.v1",
            "claude-code-existing.v1",
            "claude-sonnet-4.5",
        ),
    };
    ResolvedAgentSelectionV1 {
        version: 1,
        logical_agent_id: definition.logical_agent_id.clone(),
        realized_agent_id: realized,
        backend_id: backend,
        placement: request.placement,
        harness_contract_revision: harness_revision.into(),
        selection: tuple.clone(),
        connection_profile: (*profile).clone(),
        routed_model_id: routed_model.into(),
        upstream_model_id: tuple.model.clone(),
        route_revision: route_revision.into(),
        selection_origin: if request.backend_id.is_some() {
            AgentSelectionOriginV1::ExplicitBackend
        } else if request.agent_id.is_some() {
            AgentSelectionOriginV1::ExplicitAgent
        } else {
            AgentSelectionOriginV1::ConfiguredDefault
        },
        inventory_source: definition.source.clone(),
        config_sources: config.sources.to_vec(),
        preference_source: if explicit {
            None
        } else {
            config.preference_source.cloned()
        },
        selection_digest: String::new(),
    }
    .seal()
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport_api_types::{AgentConnectionAuthV1, AgentConnectionBillingV1};

    fn source(scope: AgentSelectionSourceScopeV1, path: &str) -> AgentSelectionSourceV1 {
        AgentSelectionSourceV1 {
            scope,
            root_identity: "fixture-root".into(),
            relative_path: path.into(),
            revision: "1".into(),
            sha256: "a".repeat(64),
        }
    }

    fn tuple(model: &str) -> AgentSelectionTupleV1 {
        AgentSelectionTupleV1 {
            harness: AgentHarnessFamilyV1::Codex,
            provider: "openai".into(),
            connection_profile: "global:subscription".into(),
            model: model.into(),
        }
    }

    fn definition(
        id: &str,
        scope: AgentSelectionSourceScopeV1,
        model: &str,
    ) -> ConfiguredAgentDefinitionV3 {
        ConfiguredAgentDefinitionV3 {
            logical_agent_id: id.into(),
            enabled: true,
            source: source(scope, &format!("agents/{id}.yaml")),
            placements: [
                AgentSelectionPlacementV1::Host,
                AgentSelectionPlacementV1::World,
            ]
            .into_iter()
            .map(|placement| {
                (
                    placement,
                    ConfiguredAgentPlacementV3 {
                        enabled: true,
                        runtime_family: AgentHarnessFamilyV1::Codex,
                        selection: tuple(model),
                    },
                )
            })
            .collect(),
        }
    }

    fn resolve(
        definitions: &[ConfiguredAgentDefinitionV3],
        preference: &str,
        agent: Option<&str>,
        backend: Option<&str>,
        placement: AgentSelectionPlacementV1,
    ) -> Result<ResolvedAgentSelectionV1, String> {
        let config_source = source(AgentSelectionSourceScopeV1::Global, "config.yaml");
        let profile = AgentConnectionProfileV1 {
            id: "global:subscription".into(),
            revision: "1".into(),
            provider: "openai".into(),
            billing: AgentConnectionBillingV1::Subscription,
            endpoint_identity: "openai-subscription".into(),
            upstream_base_url: "https://chatgpt.com/backend-api".into(),
            auth: AgentConnectionAuthV1::CodexAccount {
                reference: "configured-account".into(),
            },
            source: config_source.clone(),
        };
        resolve_agent_selection_v1(
            definitions,
            &AgentSelectionConfigV1 {
                preference: Some(&tuple(preference)),
                preference_source: Some(&config_source),
                profiles: &[profile],
                sources: std::slice::from_ref(&config_source),
            },
            &AgentSelectionRequestV1 {
                agent_id: agent,
                backend_id: backend,
                placement,
            },
        )
    }

    #[test]
    fn test_default_matches_full_tuple_without_catalogue() {
        let definitions = [definition(
            "custom",
            AgentSelectionSourceScopeV1::Global,
            "private-configured-id",
        )];
        let selected = resolve(
            &definitions,
            "private-configured-id",
            None,
            None,
            AgentSelectionPlacementV1::World,
        )
        .unwrap();
        assert_eq!(selected.backend_id, "cli:custom-world");
        assert_eq!(selected.upstream_model_id, "private-configured-id");
        assert_eq!(
            selected.selection_origin,
            AgentSelectionOriginV1::ConfiguredDefault
        );
        selected.validate().unwrap();
    }

    #[test]
    fn test_workspace_whole_replacement_mask_and_delete_reveal_global() {
        let global = definition(
            "custom",
            AgentSelectionSourceScopeV1::Global,
            "global-model",
        );
        let other = definition("other", AgentSelectionSourceScopeV1::Global, "other-model");
        let mut workspace = definition(
            "custom",
            AgentSelectionSourceScopeV1::Workspace,
            "local-model",
        );
        workspace
            .placements
            .remove(&AgentSelectionPlacementV1::World);
        let definitions = [workspace.clone(), other.clone(), global.clone()];
        let visible = visible_agent_definitions_v3(&definitions).unwrap();
        assert_eq!(visible.len(), 2);
        assert_eq!(
            visible["custom"].source.scope,
            AgentSelectionSourceScopeV1::Workspace
        );
        assert!(resolve(
            &definitions,
            "global-model",
            None,
            None,
            AgentSelectionPlacementV1::World
        )
        .is_err());
        assert!(resolve(
            &definitions,
            "other-model",
            None,
            None,
            AgentSelectionPlacementV1::World
        )
        .is_ok());
        workspace.enabled = false;
        assert!(resolve(
            &[global.clone(), workspace],
            "global-model",
            None,
            None,
            AgentSelectionPlacementV1::Host
        )
        .is_err());
        assert!(resolve(
            &[global],
            "global-model",
            None,
            None,
            AgentSelectionPlacementV1::Host
        )
        .is_ok());
    }

    #[test]
    fn test_duplicate_ambiguous_and_missing_defaults_refuse() {
        let global = definition("custom", AgentSelectionSourceScopeV1::Global, "model");
        assert!(
            visible_agent_definitions_v3(&[global.clone(), global.clone()])
                .unwrap_err()
                .contains("duplicate")
        );
        let other = definition("other", AgentSelectionSourceScopeV1::Global, "model");
        assert!(resolve(
            &[global.clone(), other],
            "model",
            None,
            None,
            AgentSelectionPlacementV1::Host
        )
        .unwrap_err()
        .contains("found 2"));
        assert!(resolve(
            &[global],
            "absent",
            None,
            None,
            AgentSelectionPlacementV1::Host
        )
        .unwrap_err()
        .contains("found 0"));
    }

    #[test]
    fn test_explicit_selection_keeps_tuple_and_checks_backend_actor() {
        let definitions = [
            definition(
                "host_actor",
                AgentSelectionSourceScopeV1::Global,
                "host-model",
            ),
            definition(
                "worker",
                AgentSelectionSourceScopeV1::Global,
                "worker-model",
            ),
        ];
        let host = resolve(
            &definitions,
            "different-default",
            Some("host_actor"),
            None,
            AgentSelectionPlacementV1::Host,
        )
        .unwrap();
        let worker = resolve(
            &definitions,
            "different-default",
            None,
            Some("cli:worker-world"),
            AgentSelectionPlacementV1::World,
        )
        .unwrap();
        assert_eq!(host.selection.model, "host-model");
        assert_eq!(worker.selection.model, "worker-model");
        assert_ne!(host.logical_agent_id, worker.logical_agent_id);
        assert!(resolve(
            &definitions,
            "host-model",
            Some("host_actor"),
            Some("cli:worker-world"),
            AgentSelectionPlacementV1::World
        )
        .is_err());
    }

    #[test]
    fn test_harness_conflict_and_disabled_placement_refuse() {
        let mut definition = definition("custom", AgentSelectionSourceScopeV1::Global, "model");
        definition
            .placements
            .get_mut(&AgentSelectionPlacementV1::Host)
            .unwrap()
            .runtime_family = AgentHarnessFamilyV1::ClaudeCode;
        assert!(visible_agent_definitions_v3(&[definition.clone()])
            .unwrap_err()
            .contains("runtime_family"));
        definition
            .placements
            .get_mut(&AgentSelectionPlacementV1::Host)
            .unwrap()
            .runtime_family = AgentHarnessFamilyV1::Codex;
        definition
            .placements
            .get_mut(&AgentSelectionPlacementV1::Host)
            .unwrap()
            .enabled = false;
        assert!(resolve(
            &[definition],
            "model",
            None,
            None,
            AgentSelectionPlacementV1::Host
        )
        .is_err());
    }
}
