//! Shared request/response models and error types for the Agent API.

mod ordinary_initial_control;
pub use ordinary_initial_control::{
    OrdinaryInitialBootstrapControlDeliveryV1, OrdinaryInitialBootstrapControlOperationV1,
    OrdinaryInitialBootstrapControlRequestV1, OrdinaryInitialBootstrapControlResponseV1,
    OrdinaryInitialBootstrapProcessCloseoutV1, OrdinaryInitialBootstrapRetirementObservationV1,
    OrdinaryInitialBootstrapRetirementV1,
};

mod ordinary_work_cancel;
pub use ordinary_work_cancel::{
    OrdinaryAcceptedWorkCancelControlV1, OrdinaryAcceptedWorkCancelRequestV1,
    OrdinaryAcceptedWorkCancelResponseV1,
};

mod ordinary_retained_stop;
pub use ordinary_retained_stop::{
    OrdinaryRetainedWorkerStopAcknowledgmentV1, OrdinaryRetainedWorkerStopBusyReasonV1,
    OrdinaryRetainedWorkerStopRequestV1, OrdinaryRetainedWorkerStopResponseV1,
};

mod retained_bootstrap_completion;
pub use retained_bootstrap_completion::{
    NativeProcessTerminationV1, NativeProcessWaitObservationV1, NativeSettlementAcknowledgmentV1,
    OrdinaryForkBootstrapProvenanceV1, OrdinaryGatewayDisposalV1, RetainedBootstrapCompletionV1,
    RetainedBootstrapObservationV1, RetainedForkBootstrapCompletionV1,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use substrate_common::agent_events::AgentEvent;
pub use substrate_common::agent_events::{
    RuntimeEventIdentityV1, RuntimeFrameIdentityV1, RuntimeTerminalIdentityV1,
    RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
};
pub use substrate_common::{
    validate_identity_tuple_and_placement_posture, FsDiff, HostTransitionWorkCorrelationV1,
    IdentityTuple, OpaqueAuthorityCommitmentV1, PlacementExecution, PlacementPosture, ProcessEvent,
    ProcessEventType, ProcessEventsStatus, ProcessTelemetry, WorldFsMode,
};
pub use world_api::{
    SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerAction,
    SharedWorldOwnerSpec, WorldReuseMode,
};

/// Harness identity selected by configuration, independently of an agent's ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentHarnessFamilyV1 {
    Codex,
    ClaudeCode,
}

/// The four inseparable configured selection fields.
///
/// ```
/// use transport_api_types::AgentSelectionTupleV1;
/// let tuple: AgentSelectionTupleV1 = serde_json::from_str(r#"{
///   "harness":"codex", "provider":"openai",
///   "connection_profile":"global:subscription", "model":"gpt-6-astra"
/// }"#).unwrap();
/// tuple.validate().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSelectionTupleV1 {
    pub harness: AgentHarnessFamilyV1,
    pub provider: String,
    pub connection_profile: String,
    /// Exact upstream request ID, never a catalogue-derived suggestion.
    pub model: String,
}

impl AgentSelectionTupleV1 {
    pub fn validate(&self) -> Result<(), String> {
        selection_identifier(&self.provider, "provider")?;
        selection_profile_id(&self.connection_profile)?;
        selection_text(&self.model, "model", 256)
    }
}

/// Inventory placement; execution roles are validated separately by callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSelectionPlacementV1 {
    Host,
    World,
}

impl AgentSelectionPlacementV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::World => "world",
        }
    }
}

/// Source namespace. Its order is also the canonical configuration layer order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSelectionSourceScopeV1 {
    BuiltIn,
    Global,
    Workspace,
}

impl AgentSelectionSourceScopeV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BuiltIn => "built_in",
            Self::Global => "global",
            Self::Workspace => "workspace",
        }
    }
}

/// Nonsecret provenance captured from the source owner.
///
/// A root identity records the publisher's accepted physical identity; this
/// value neither opens that root nor establishes authority over it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSelectionSourceV1 {
    pub scope: AgentSelectionSourceScopeV1,
    pub root_identity: String,
    pub relative_path: String,
    pub revision: String,
    pub sha256: String,
}

impl AgentSelectionSourceV1 {
    pub fn validate(&self) -> Result<(), String> {
        selection_text(&self.root_identity, "source root identity", 8192)?;
        selection_text(&self.relative_path, "source relative path", 2048)?;
        if self.relative_path.starts_with('/')
            || self.relative_path.contains('\\')
            || self
                .relative_path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err("selection source path must be a confined relative path".into());
        }
        selection_text(&self.revision, "source revision", 256)?;
        selection_sha256(&self.sha256, "source hash")
    }
}

/// Nonsecret reference to an existing credential owner; never credential bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "auth_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentConnectionAuthV1 {
    CodexAccount { reference: String },
    ApiKeyEnvironment { name: String },
}

/// Billing semantics committed with the selected connection profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentConnectionBillingV1 {
    Subscription,
    Api,
}

/// Source-qualified connection configuration, containing references only.
/// The upstream URL is distinct from the native client's managed local URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConnectionProfileV1 {
    pub id: String,
    pub revision: String,
    pub provider: String,
    pub billing: AgentConnectionBillingV1,
    pub endpoint_identity: String,
    pub upstream_base_url: String,
    pub auth: AgentConnectionAuthV1,
    pub source: AgentSelectionSourceV1,
}

impl AgentConnectionProfileV1 {
    pub fn validate(&self) -> Result<(), String> {
        let scope = selection_profile_id(&self.id)?;
        self.source.validate()?;
        if scope != self.source.scope.as_str() {
            return Err("connection profile namespace does not match its source".into());
        }
        Self::validate_fields(
            &self.id,
            &self.revision,
            &self.provider,
            self.billing,
            &self.endpoint_identity,
            &self.upstream_base_url,
            &self.auth,
        )
    }

    /// Validate profile values without source acquisition or permission checks.
    /// This does not validate physical provenance or authorize execution.
    ///
    /// ```
    /// use transport_api_types::{AgentConnectionAuthV1, AgentConnectionBillingV1, AgentConnectionProfileV1};
    /// let auth = AgentConnectionAuthV1::CodexAccount { reference: "configured-account".into() };
    /// AgentConnectionProfileV1::validate_fields(
    ///     "global:subscription", "1", "openai", AgentConnectionBillingV1::Subscription,
    ///     "openai-subscription", "https://chatgpt.com/backend-api", &auth,
    /// ).unwrap();
    /// ```
    pub fn validate_fields(
        id: &str,
        revision: &str,
        provider: &str,
        _billing: AgentConnectionBillingV1,
        endpoint_identity: &str,
        upstream_base_url: &str,
        auth: &AgentConnectionAuthV1,
    ) -> Result<(), String> {
        // Billing is a closed enum; supported route/billing permission is a
        // separate admission check, unchanged by structural value validation.
        selection_profile_id(id)?;
        selection_text(revision, "connection profile revision", 256)?;
        selection_identifier(provider, "connection profile provider")?;
        selection_identifier(endpoint_identity, "endpoint identity")?;
        selection_upstream_url(upstream_base_url)?;
        match auth {
            AgentConnectionAuthV1::CodexAccount { reference } => {
                selection_identifier(reference, "account reference")
            }
            AgentConnectionAuthV1::ApiKeyEnvironment { name } => {
                if name.is_empty()
                    || name.len() > 128
                    || name.bytes().enumerate().any(|(index, byte)| {
                        !(byte.is_ascii_alphabetic()
                            || byte == b'_'
                            || (index > 0 && byte.is_ascii_digit()))
                    })
                {
                    Err("invalid connection profile environment reference".into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// The original selection decision, retained unchanged on Resume/recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSelectionOriginV1 {
    ExplicitAgent,
    ExplicitBackend,
    ConfiguredDefault,
}

/// Complete configuration-derived selection shared by ordinary and later
/// protected consumers. Call `validate` after decoding an untrusted value.
///
/// Validation establishes structural support and commitment equality only.
/// Production must separately authenticate Start/current authority and check
/// policy, credential ownership, endpoint support and native capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedAgentSelectionV1 {
    pub version: u32,
    pub logical_agent_id: String,
    pub realized_agent_id: String,
    pub backend_id: String,
    pub placement: AgentSelectionPlacementV1,
    pub harness_contract_revision: String,
    pub selection: AgentSelectionTupleV1,
    pub connection_profile: AgentConnectionProfileV1,
    pub routed_model_id: String,
    pub upstream_model_id: String,
    pub route_revision: String,
    pub selection_origin: AgentSelectionOriginV1,
    pub inventory_source: AgentSelectionSourceV1,
    pub config_sources: Vec<AgentSelectionSourceV1>,
    pub preference_source: Option<AgentSelectionSourceV1>,
    pub selection_digest: String,
}

impl ResolvedAgentSelectionV1 {
    /// Validate structure and the original selection commitment, not permission.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_content()?;
        selection_sha256(&self.selection_digest, "selection digest")?;
        if self.compute_digest()? != self.selection_digest {
            return Err("selection digest does not match committed content".into());
        }
        Ok(())
    }

    /// Seal a new complete selection. This does not authorize execution.
    pub fn seal(mut self) -> Result<Self, String> {
        self.validate_content()?;
        self.selection_digest = self.compute_digest()?;
        Ok(self)
    }

    /// Domain-separated canonical digest with only `selection_digest` omitted.
    pub fn compute_digest(&self) -> Result<String, String> {
        let mut value = serde_json::to_value(self).map_err(|error| error.to_string())?;
        value
            .as_object_mut()
            .ok_or("invalid selection representation")?
            .remove("selection_digest");
        let mut canonical = Vec::new();
        selection_canonical_json(&value, &mut canonical)?;
        let mut hash = Sha256::new();
        hash.update(b"substrate.resolved-agent-selection.v1\0");
        hash.update(canonical);
        Ok(format!("{:x}", hash.finalize()))
    }

    fn validate_content(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported resolved agent selection version".into());
        }
        selection_identifier(&self.logical_agent_id, "logical agent id")?;
        let realized = format!("{}-{}", self.logical_agent_id, self.placement.as_str());
        if self.realized_agent_id != realized || self.backend_id != format!("cli:{realized}") {
            return Err("selection backend does not match logical identity and placement".into());
        }
        self.selection.validate()?;
        self.connection_profile.validate()?;
        if self.selection.connection_profile != self.connection_profile.id
            || self.selection.provider != self.connection_profile.provider
            || self.selection.model != self.upstream_model_id
        {
            return Err("selection tuple conflicts with the connection profile or route".into());
        }
        let profile = &self.connection_profile;
        let supported = match self.selection.harness {
            AgentHarnessFamilyV1::Codex => {
                self.harness_contract_revision == "codex-cli.v1"
                    && self.route_revision == "codex-subscription.v1"
                    && self.selection.provider == "openai"
                    && self.routed_model_id == self.upstream_model_id
                    && profile.billing == AgentConnectionBillingV1::Subscription
                    && matches!(profile.auth, AgentConnectionAuthV1::CodexAccount { .. })
            }
            AgentHarnessFamilyV1::ClaudeCode => {
                self.harness_contract_revision == "claude-code-cli.v1"
                    && self.route_revision == "claude-code-existing.v1"
                    && self.selection.provider == "anthropic"
                    && self.routed_model_id == "claude-sonnet-4.5"
                    && self.upstream_model_id == "claude-sonnet-4-5-20250929"
                    && profile.billing == AgentConnectionBillingV1::Api
                    && matches!(
                        profile.auth,
                        AgentConnectionAuthV1::ApiKeyEnvironment { .. }
                    )
            }
        };
        if !supported {
            return Err("unsupported harness, provider, profile or route combination".into());
        }
        self.inventory_source.validate()?;
        if self.config_sources.is_empty() || self.config_sources.len() > 3 {
            return Err("selection requires a bounded captured configuration source set".into());
        }
        let mut previous = None;
        for source in &self.config_sources {
            source.validate()?;
            if previous.is_some_and(|scope| scope >= source.scope) {
                return Err("configuration sources must have unique ordered scopes".into());
            }
            previous = Some(source.scope);
        }
        if !self.config_sources.contains(&profile.source) {
            return Err(
                "selected profile is not bound to its captured configuration source".into(),
            );
        }
        match (self.selection_origin, &self.preference_source) {
            (AgentSelectionOriginV1::ConfiguredDefault, Some(source))
                if self.config_sources.contains(source) => {}
            (
                AgentSelectionOriginV1::ExplicitAgent | AgentSelectionOriginV1::ExplicitBackend,
                None,
            ) => {}
            _ => {
                return Err("selection origin does not match default preference provenance".into())
            }
        }
        Ok(())
    }
}

fn selection_text(value: &str, field: &str, maximum: usize) -> Result<(), String> {
    if value.is_empty()
        || value.len() > maximum
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(format!("invalid selection {field}"))
    } else {
        Ok(())
    }
}

fn selection_identifier(value: &str, field: &str) -> Result<(), String> {
    selection_text(value, field, 128)?;
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        Ok(())
    } else {
        Err(format!("invalid selection {field}"))
    }
}

fn selection_profile_id(value: &str) -> Result<&str, String> {
    let (scope, id) = value
        .split_once(':')
        .ok_or("connection profile id must be source-qualified")?;
    if !matches!(scope, "built_in" | "global" | "workspace") {
        return Err("unsupported connection profile source namespace".into());
    }
    selection_identifier(id, "connection profile id")?;
    Ok(scope)
}

fn selection_sha256(value: &str, field: &str) -> Result<(), String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(format!("invalid selection {field}"))
    }
}

fn selection_upstream_url(value: &str) -> Result<(), String> {
    selection_text(value, "upstream URL", 2048)?;
    let rest = value
        .strip_prefix("https://")
        .ok_or("selected upstream URL requires HTTPS")?;
    if value.bytes().any(|byte| byte.is_ascii_whitespace()) || value.contains(['@', '?', '#', '\\'])
    {
        return Err("selected upstream URL may not contain credentials, query or fragment".into());
    }
    let authority = rest.split('/').next().unwrap_or_default();
    let (host, port) = authority
        .split_once(':')
        .map_or((authority, None), |(host, port)| (host, Some(port)));
    if host.is_empty()
        || !host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        || port.is_some_and(|port| port.parse::<u16>().ok().is_none_or(|port| port == 0))
    {
        return Err("unsupported selected upstream URL authority".into());
    }
    Ok(())
}

fn selection_canonical_json(value: &serde_json::Value, out: &mut Vec<u8>) -> Result<(), String> {
    match value {
        serde_json::Value::Object(object) => {
            out.push(b'{');
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                serde_json::to_writer(&mut *out, key).map_err(|error| error.to_string())?;
                out.push(b':');
                selection_canonical_json(value, out)?;
            }
            out.push(b'}');
        }
        serde_json::Value::Array(values) => {
            out.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                selection_canonical_json(value, out)?;
            }
            out.push(b']');
        }
        _ => serde_json::to_writer(out, value).map_err(|error| error.to_string())?,
    }
    Ok(())
}

const INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1: &str = "substrate.install_bootstrap_context";

#[cfg(test)]
mod agent_selection_contract_tests {
    use super::*;

    fn source(scope: AgentSelectionSourceScopeV1, path: &str) -> AgentSelectionSourceV1 {
        AgentSelectionSourceV1 {
            scope,
            root_identity: "accepted-root-identity".into(),
            relative_path: path.into(),
            revision: "revision-1".into(),
            sha256: "a".repeat(64),
        }
    }

    fn selection() -> ResolvedAgentSelectionV1 {
        let config = source(AgentSelectionSourceScopeV1::Global, "config.yaml");
        ResolvedAgentSelectionV1 {
            version: 1,
            logical_agent_id: "custom".into(),
            realized_agent_id: "custom-host".into(),
            backend_id: "cli:custom-host".into(),
            placement: AgentSelectionPlacementV1::Host,
            harness_contract_revision: "codex-cli.v1".into(),
            selection: AgentSelectionTupleV1 {
                harness: AgentHarnessFamilyV1::Codex,
                provider: "openai".into(),
                connection_profile: "global:subscription".into(),
                model: "gpt-6-astra".into(),
            },
            connection_profile: AgentConnectionProfileV1 {
                id: "global:subscription".into(),
                revision: "profile-1".into(),
                provider: "openai".into(),
                billing: AgentConnectionBillingV1::Subscription,
                endpoint_identity: "openai-subscription".into(),
                upstream_base_url: "https://chatgpt.com/backend-api".into(),
                auth: AgentConnectionAuthV1::CodexAccount {
                    reference: "configured-account".into(),
                },
                source: config.clone(),
            },
            routed_model_id: "gpt-6-astra".into(),
            upstream_model_id: "gpt-6-astra".into(),
            route_revision: "codex-subscription.v1".into(),
            selection_origin: AgentSelectionOriginV1::ConfiguredDefault,
            inventory_source: source(AgentSelectionSourceScopeV1::Global, "agents/custom.yaml"),
            config_sources: vec![config.clone()],
            preference_source: Some(config),
            selection_digest: String::new(),
        }
        .seal()
        .unwrap()
    }

    #[test]
    fn test_profile_fields_validate_without_source_and_full_profiles_keep_source_checks() {
        let auth = AgentConnectionAuthV1::CodexAccount {
            reference: "configured-account".into(),
        };
        AgentConnectionProfileV1::validate_fields(
            "global:subscription",
            "1",
            "openai",
            AgentConnectionBillingV1::Subscription,
            "openai-subscription",
            "https://chatgpt.com/backend-api",
            &auth,
        )
        .unwrap();
        for (revision, provider, endpoint, url, auth) in [
            (
                "",
                "openai",
                "endpoint",
                "https://example.test/v1",
                auth.clone(),
            ),
            ("1", "", "endpoint", "https://example.test/v1", auth.clone()),
            ("1", "openai", "", "https://example.test/v1", auth.clone()),
            ("1", "openai", "endpoint", "", auth.clone()),
            (
                "1",
                "openai",
                "endpoint",
                "http://example.test/v1",
                auth.clone(),
            ),
            (
                "1",
                "openai",
                "endpoint",
                "https://secret@example.test/v1",
                auth.clone(),
            ),
            (
                "1",
                "openai",
                "endpoint",
                "https://example.test/v1",
                AgentConnectionAuthV1::CodexAccount {
                    reference: String::new(),
                },
            ),
            (
                "1",
                "openai",
                "endpoint",
                "https://example.test/v1",
                AgentConnectionAuthV1::ApiKeyEnvironment {
                    name: String::new(),
                },
            ),
            (
                "1",
                "openai",
                "endpoint",
                "https://example.test/v1",
                AgentConnectionAuthV1::ApiKeyEnvironment {
                    name: "9BAD".into(),
                },
            ),
        ] {
            assert!(AgentConnectionProfileV1::validate_fields(
                "global:subscription",
                revision,
                provider,
                AgentConnectionBillingV1::Subscription,
                endpoint,
                url,
                &auth
            )
            .is_err());
        }
        assert!(AgentConnectionProfileV1::validate_fields(
            "global:",
            "1",
            "openai",
            AgentConnectionBillingV1::Subscription,
            "endpoint",
            "https://example.test/v1",
            &auth
        )
        .is_err());
        let original = selection().connection_profile;
        original.validate().unwrap();
        let mut wrong_source = original.clone();
        wrong_source.source.sha256.clear();
        assert!(wrong_source.validate().is_err());
        wrong_source = original;
        wrong_source.source.scope = AgentSelectionSourceScopeV1::Workspace;
        assert!(wrong_source.validate().is_err());
    }

    #[test]
    fn test_selection_contract_roundtrip_and_canonical_digest() {
        let selected = selection();
        selected.validate().unwrap();
        let value = serde_json::to_value(&selected).unwrap();
        let decoded: ResolvedAgentSelectionV1 = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(decoded, selected);
        assert_eq!(decoded.compute_digest().unwrap(), selected.selection_digest);
        let mut object = value.as_object().unwrap().clone();
        object.insert("unknown".into(), serde_json::json!(true));
        assert!(serde_json::from_value::<ResolvedAgentSelectionV1>(object.into()).is_err());
    }

    #[test]
    fn test_selection_contract_rejects_tuple_route_profile_and_origin_conflicts() {
        let selected = selection();
        let mut changed = selected.clone();
        changed.selection.model = "different".into();
        assert!(changed.seal().is_err());
        changed = selected.clone();
        changed.connection_profile.provider = "other".into();
        assert!(changed.seal().is_err());
        changed = selected.clone();
        changed.connection_profile.id = "workspace:subscription".into();
        assert!(changed.seal().is_err());
        changed = selected.clone();
        changed.preference_source = None;
        assert!(changed.seal().is_err());
        changed = selected.clone();
        changed.selection_origin = AgentSelectionOriginV1::ExplicitAgent;
        assert!(changed.seal().is_err());
        changed = selected.clone();
        changed.realized_agent_id = "other-host".into();
        assert!(changed.seal().is_err());
    }

    #[test]
    fn test_selection_digest_binds_sources_profile_revision_model_and_origin() {
        let selected = selection();
        let mut variants = Vec::new();
        let mut changed = selected.clone();
        changed.inventory_source.sha256 = "b".repeat(64);
        variants.push(changed);
        changed = selected.clone();
        changed.connection_profile.revision = "profile-2".into();
        variants.push(changed);
        changed = selected.clone();
        changed.selection.model = "configured-model-absent-from-catalog".into();
        changed.upstream_model_id = changed.selection.model.clone();
        changed.routed_model_id = changed.selection.model.clone();
        variants.push(changed);
        changed = selected.clone();
        changed.selection_origin = AgentSelectionOriginV1::ExplicitBackend;
        changed.preference_source = None;
        variants.push(changed);
        for changed in variants {
            assert!(
                changed.validate().is_err(),
                "changed input must invalidate commitment"
            );
            let sealed = changed.seal().unwrap();
            assert_ne!(sealed.selection_digest, selected.selection_digest);
            sealed.validate().unwrap();
        }
    }

    #[test]
    fn test_selection_profiles_reject_secret_endpoint_and_source_substitution() {
        let selected = selection();
        for endpoint in [
            "http://example.test/v1",
            "https://secret@example.test/v1",
            "https://example.test/v1?key=secret",
            "https://example.test/v1#secret",
            "https:///missing-host",
            "https://example.test:0/v1",
        ] {
            let mut changed = selected.clone();
            changed.connection_profile.upstream_base_url = endpoint.into();
            assert!(changed.seal().is_err(), "{endpoint}");
        }
        let mut changed = selected;
        changed.connection_profile.source.sha256 = "b".repeat(64);
        assert!(changed.seal().is_err());
    }

    #[test]
    fn test_selection_claude_preserves_exact_existing_route() {
        let mut selected = selection();
        selected.selection.harness = AgentHarnessFamilyV1::ClaudeCode;
        selected.selection.provider = "anthropic".into();
        selected.selection.model = "claude-sonnet-4-5-20250929".into();
        selected.connection_profile.provider = "anthropic".into();
        selected.connection_profile.billing = AgentConnectionBillingV1::Api;
        selected.connection_profile.auth = AgentConnectionAuthV1::ApiKeyEnvironment {
            name: "ANTHROPIC_API_KEY".into(),
        };
        selected.harness_contract_revision = "claude-code-cli.v1".into();
        selected.route_revision = "claude-code-existing.v1".into();
        selected.routed_model_id = "claude-sonnet-4.5".into();
        selected.upstream_model_id = selected.selection.model.clone();
        let selected = selected.seal().unwrap();
        selected.validate().unwrap();
        let mut unsupported = selected;
        unsupported.selection.model = "new-claude-adapter".into();
        unsupported.upstream_model_id = unsupported.selection.model.clone();
        assert!(unsupported.seal().is_err());
    }
}
const PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1: &str = "substrate.platform_bootstrap_mapping";
const WINDOWS_FORWARDER_SCOPE_DOMAIN_V1: &str = "substrate.windows_forwarder_scope";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformPrincipalV1 {
    Unix { account: String, uid: u32 },
    Windows { account: String, sid: String },
}

/// Bounded identity for the concrete world targeted by one narrowing request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldBindingRefV1 {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityObjectKindV1 {
    AgentDescriptor,
    RetainedWorker,
    ResumeHandle,
    Policy,
    HostAttachContract,
    TransitionTransportPayload,
    TransitionInput,
    LeaseToken,
    ApplicationResult,
    InputAcceptance,
    StartupOwnershipResult,
    ObligationSnapshot,
    PostTurnProtocolEvent,
    PostTurnCompletion,
    TerminalHandoff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorityObjectRefV1 {
    pub ref_id: String,
    pub object_kind: AuthorityObjectKindV1,
    pub schema_version: u32,
    pub commitment: OpaqueAuthorityCommitmentV1,
}

pub type PolicyRefV1 = AuthorityObjectRefV1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityObjectRefV1Def {
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: OpaqueAuthorityCommitmentV1,
}

impl<'de> Deserialize<'de> for AuthorityObjectRefV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = AuthorityObjectRefV1Def::deserialize(deserializer)?;
        let reference = Self {
            ref_id: value.ref_id,
            object_kind: value.object_kind,
            schema_version: value.schema_version,
            commitment: value.commitment,
        };
        reference.validate().map_err(serde::de::Error::custom)?;
        Ok(reference)
    }
}

impl AuthorityObjectRefV1 {
    pub fn validate(&self) -> Result<(), String> {
        let suffix = self.ref_id.strip_prefix("ao_").ok_or_else(|| {
            "parent_policy_ref.ref_id must be 'ao_' plus 32 lowercase hexadecimal characters"
                .to_string()
        })?;
        if suffix.len() != 32
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "parent_policy_ref.ref_id must be 'ao_' plus 32 lowercase hexadecimal characters"
                    .to_string(),
            );
        }
        if self.object_kind != AuthorityObjectKindV1::Policy {
            return Err("parent_policy_ref.object_kind must be policy".to_string());
        }
        if self.schema_version == 0 {
            return Err("parent_policy_ref.schema_version must be positive".to_string());
        }
        self.commitment.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DispatchCapabilitySubjectV1 {
    EphemeralTask,
    RetainedWorkerSpawn,
    RetainedWorkerTurn { retained_participant_id: String },
    RetainedWorkerFork { source_participant_id: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedPolicyPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub world_fs: Option<RestrictedWorldFsPatchV1>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub host_visible: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub fail_closed: Option<RestrictedWorldFsFailClosedPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_enforcement: Option<WorldFsDenyEnforcementV3>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub caged_required: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub discover: Option<RestrictedWorldFsDimensionPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub read: Option<RestrictedWorldFsDimensionPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub write: Option<RestrictedWorldFsWritePatchV1>,
}

impl RestrictedWorldFsPatchV1 {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsFailClosedPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub routing: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsDimensionPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub allow_list: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsWritePatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub enabled: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub allow_list: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_list: Option<Vec<String>>,
}

fn deserialize_optional_non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DispatchPolicyNarrowingPatchV1 {
    pub schema_version: u32,
    pub request_id: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub target_backend_id: String,
    pub target_world: WorldBindingRefV1,
    pub applies_to: DispatchCapabilitySubjectV1,
    pub parent_policy_ref: PolicyRefV1,
    pub parent_policy_revision: String,
    pub restricted_policy_patch: RestrictedPolicyPatchV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyNarrowingPatchV1Def {
    schema_version: u32,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    applies_to: DispatchCapabilitySubjectV1,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    restricted_policy_patch: RestrictedPolicyPatchV1,
    reason: Option<String>,
}

impl<'de> Deserialize<'de> for DispatchPolicyNarrowingPatchV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = DispatchPolicyNarrowingPatchV1Def::deserialize(deserializer)?;
        let carrier = Self {
            schema_version: value.schema_version,
            request_id: value.request_id,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            target_backend_id: value.target_backend_id,
            target_world: value.target_world,
            applies_to: value.applies_to,
            parent_policy_ref: value.parent_policy_ref,
            parent_policy_revision: value.parent_policy_revision,
            restricted_policy_patch: value.restricted_policy_patch,
            reason: value.reason,
        };
        carrier.validate().map_err(serde::de::Error::custom)?;
        Ok(carrier)
    }
}

impl DispatchPolicyNarrowingPatchV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported dispatch narrowing schema_version {} (expected 1)",
                self.schema_version
            ));
        }
        for (name, value) in [
            ("request_id", self.request_id.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            ("target_backend_id", self.target_backend_id.as_str()),
            ("target_world.world_id", self.target_world.world_id.as_str()),
            (
                "parent_policy_revision",
                self.parent_policy_revision.as_str(),
            ),
        ] {
            validate_narrowing_identity(name, value)?;
        }
        if self.target_world.world_generation == 0 {
            return Err("target_world.world_generation must be positive".to_string());
        }
        match &self.applies_to {
            DispatchCapabilitySubjectV1::RetainedWorkerTurn {
                retained_participant_id,
            } => validate_narrowing_identity(
                "applies_to.retained_participant_id",
                retained_participant_id,
            )?,
            DispatchCapabilitySubjectV1::RetainedWorkerFork {
                source_participant_id,
            } => validate_narrowing_identity(
                "applies_to.source_participant_id",
                source_participant_id,
            )?,
            DispatchCapabilitySubjectV1::EphemeralTask
            | DispatchCapabilitySubjectV1::RetainedWorkerSpawn => {}
        }
        self.parent_policy_ref.validate()?;
        if let Some(reason) = &self.reason {
            validate_narrowing_identity("reason", reason)?;
        }
        Ok(())
    }
}

fn validate_narrowing_identity(name: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.contains('\0')
        || value.chars().any(char::is_control)
    {
        return Err(format!("{name} must be a nonempty canonical identity"));
    }
    Ok(())
}

#[cfg(test)]
mod e1_dispatch_policy_narrowing_schema_tests {
    use super::*;
    use serde_json::{json, Value};

    fn valid_carrier_json() -> Value {
        json!({
            "schema_version": 1,
            "request_id": "req_e1_0001",
            "orchestration_session_id": "session_e1_0001",
            "caller_participant_id": "participant_e1_caller",
            "target_backend_id": "codex",
            "target_world": {
                "world_id": "world_e1_0001",
                "world_generation": 7
            },
            "applies_to": { "kind": "ephemeral_task" },
            "parent_policy_ref": {
                "ref_id": "ao_0123456789abcdef0123456789abcdef",
                "object_kind": "policy",
                "schema_version": 1,
                "commitment": {
                    "kind": "CanonicalSha256",
                    "value": {
                        "digest_hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    }
                }
            },
            "parent_policy_revision": "policy-revision-e1-0001",
            "restricted_policy_patch": {
                "world_fs": {
                    "host_visible": false,
                    "read": { "allow_list": ["src/lib.rs"] }
                }
            },
            "reason": "restrict delegated read authority"
        })
    }

    #[test]
    fn dispatch_narrowing_v1_decodes_only_the_strict_world_fs_schema() {
        let decoded: DispatchPolicyNarrowingPatchV1 =
            serde_json::from_value(valid_carrier_json()).expect("strict carrier");
        decoded.validate().expect("valid carrier");

        for mutation in [
            ("schema_version", json!(2)),
            ("request_id", json!(" request")),
            ("parent_policy_revision", json!("")),
        ] {
            let mut value = valid_carrier_json();
            value[mutation.0] = mutation.1;
            assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(value).is_err());
        }

        let mut unknown = valid_carrier_json();
        unknown["unexpected"] = json!(true);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(unknown).is_err());

        let mut non_world_fs = valid_carrier_json();
        non_world_fs["restricted_policy_patch"]["net_allowed"] = json!(["example.com"]);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(non_world_fs).is_err());

        let mut nested_unknown = valid_carrier_json();
        nested_unknown["restricted_policy_patch"]["world_fs"]["read"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(nested_unknown).is_err());

        let mut unsupported_glob_material = valid_carrier_json();
        unsupported_glob_material["restricted_policy_patch"]["world_fs"]["read"]["deny_list"] =
            json!(["src/[ab].rs"]);
        let decoded: DispatchPolicyNarrowingPatchV1 =
            serde_json::from_value(unsupported_glob_material).expect("transport is not authority");
        assert_eq!(decoded.schema_version, 1);
    }

    #[test]
    fn dispatch_narrowing_v1_rejects_missing_substituted_or_ambiguous_identity() {
        for key in [
            "request_id",
            "orchestration_session_id",
            "caller_participant_id",
            "target_backend_id",
            "target_world",
            "applies_to",
            "parent_policy_ref",
            "parent_policy_revision",
            "restricted_policy_patch",
        ] {
            let mut value = valid_carrier_json();
            value.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(value).is_err());
        }

        let mut wrong_kind = valid_carrier_json();
        wrong_kind["parent_policy_ref"]["object_kind"] = json!("resume_handle");
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(wrong_kind).is_err());

        let mut null_patch_field = valid_carrier_json();
        null_patch_field["restricted_policy_patch"]["world_fs"]["host_visible"] = Value::Null;
        assert!(
            serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(null_patch_field).is_err()
        );

        let mut zero_generation = valid_carrier_json();
        zero_generation["target_world"]["world_generation"] = json!(0);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(zero_generation).is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallBootstrapContextV1 {
    pub selected_host_prefix: String,
    pub host_substrate_home: String,
    pub host_substrate_root: String,
    pub intended_host_principal: PlatformPrincipalV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallBootstrapContextCarrierV1 {
    pub context: InstallBootstrapContextV1,
    pub host_context_commitment: String,
}

/// Identity observed for one concrete Lima VM or registered WSL distribution.
///
/// ```
/// use transport_api_types::PlatformInstanceIdentityV1;
///
/// let instance = PlatformInstanceIdentityV1::Lima {
///     vm_name: "substrate".to_string(),
///     guest_machine_id: "0123456789abcdef0123456789abcdef".to_string(),
/// };
/// assert!(matches!(instance, PlatformInstanceIdentityV1::Lima { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformInstanceIdentityV1 {
    Lima {
        vm_name: String,
        guest_machine_id: String,
    },
    Wsl {
        distro_name: String,
        guest_machine_id: String,
    },
}

/// Host and guest transport endpoints observed for a platform instance.
///
/// ```
/// use transport_api_types::PlatformTransportIdentityV1;
///
/// let transport = PlatformTransportIdentityV1::Wsl {
///     pipe_path: r"\\.\pipe\substrate-agent".to_string(),
///     guest_socket: "/run/substrate.sock".to_string(),
/// };
/// assert!(matches!(transport, PlatformTransportIdentityV1::Wsl { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformTransportIdentityV1 {
    Lima {
        host_socket: String,
        guest_socket: String,
    },
    Wsl {
        pipe_path: String,
        guest_socket: String,
    },
}

/// Canonical host-to-platform bootstrap mapping bound to one host context.
///
/// ```
/// use transport_api_types::{
///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
///     PlatformBootstrapMappingV1,
/// };
///
/// let host = InstallBootstrapContextCarrierV1::from_context(
///     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
/// )?;
/// let mapping = PlatformBootstrapMappingV1::new_lima(
///     &host,
///     "substrate",
///     "0123456789abcdef0123456789abcdef",
///     "/Users/alice/.lima",
///     "/home/substrate/.substrate",
///     "substrate",
///     1000,
///     "/opt/substrate/sock/agent.sock",
///     "/run/substrate.sock",
/// )?;
/// assert_eq!(mapping.host_context_commitment, host.host_context_commitment);
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformBootstrapMappingV1 {
    pub host_context_commitment: String,
    pub platform_instance: PlatformInstanceIdentityV1,
    pub host_platform_control_root: String,
    pub realized_substrate_home: String,
    pub realized_principal: PlatformPrincipalV1,
    pub realized_transport: PlatformTransportIdentityV1,
}

/// Deterministic digest scoping shared Windows forwarder state.
///
/// ```
/// use transport_api_types::WindowsForwarderScopeV1;
///
/// let scope = WindowsForwarderScopeV1::derive(
///     "S-1-5-21-1000",
///     "Substrate-WSL",
///     "abcdef0123456789abcdef0123456789",
///     r"\\.\pipe\substrate-agent",
/// )?;
/// assert_eq!(scope.0.len(), 64);
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WindowsForwarderScopeV1(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InstallBootstrapContextErrorV1 {
    #[error("invalid install bootstrap path")]
    InvalidPath,
    #[error("invalid install bootstrap principal")]
    InvalidPrincipal,
    #[error("invalid install bootstrap context")]
    InvalidContext,
    #[error("invalid install bootstrap carrier")]
    InvalidCarrier,
    #[error("install bootstrap commitment mismatch")]
    CommitmentMismatch,
}

impl InstallBootstrapContextV1 {
    pub fn new_unix(
        selected_host_prefix: &str,
        account: &str,
        uid: u32,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let selected_host_prefix = normalize_unix_install_bootstrap_path(selected_host_prefix)?;
        let context = Self {
            host_substrate_home: selected_host_prefix.clone(),
            host_substrate_root: selected_host_prefix.clone(),
            selected_host_prefix,
            intended_host_principal: PlatformPrincipalV1::Unix {
                account: account.to_string(),
                uid,
            },
        };
        context.validate()?;
        Ok(context)
    }

    pub fn new_windows(
        selected_host_prefix: &str,
        account: &str,
        sid: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let selected_host_prefix = normalize_windows_install_bootstrap_path(selected_host_prefix)?;
        let context = Self {
            host_substrate_home: selected_host_prefix.clone(),
            host_substrate_root: selected_host_prefix.clone(),
            selected_host_prefix,
            intended_host_principal: PlatformPrincipalV1::Windows {
                account: account.to_string(),
                sid: sid.to_string(),
            },
        };
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), InstallBootstrapContextErrorV1> {
        if self.selected_host_prefix != self.host_substrate_home
            || self.selected_host_prefix != self.host_substrate_root
        {
            return Err(InstallBootstrapContextErrorV1::InvalidContext);
        }

        match &self.intended_host_principal {
            PlatformPrincipalV1::Unix { account, uid } => {
                if normalize_unix_install_bootstrap_path(&self.selected_host_prefix)?
                    != self.selected_host_prefix
                    || !valid_principal_text(account)
                    || *uid == 0
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
                }
            }
            PlatformPrincipalV1::Windows { account, sid } => {
                if normalize_windows_install_bootstrap_path(&self.selected_host_prefix)?
                    != self.selected_host_prefix
                    || !valid_principal_text(account)
                    || !valid_windows_sid(sid)
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
                }
            }
        }
        Ok(())
    }
}

impl InstallBootstrapContextCarrierV1 {
    pub fn from_context(
        context: InstallBootstrapContextV1,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        context.validate()?;
        let host_context_commitment = commitment_for_context(&context)?;
        Ok(Self {
            context,
            host_context_commitment,
        })
    }

    pub fn commitment_input(&self) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
        self.context.validate()?;
        commitment_input_for_context(&self.context)
    }

    pub fn validate(&self) -> Result<(), InstallBootstrapContextErrorV1> {
        self.context.validate()?;
        if !is_lower_hex_digest(&self.host_context_commitment) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        if commitment_for_context(&self.context)? != self.host_context_commitment {
            return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<String, InstallBootstrapContextErrorV1> {
        self.validate()?;
        let mut record = self.commitment_input()?;
        record.extend_from_slice(b"host_context_commitment=");
        record.extend_from_slice(self.host_context_commitment.as_bytes());
        record.push(b'\n');
        Ok(URL_SAFE_NO_PAD.encode(record))
    }

    pub fn decode(encoded: &str) -> Result<Self, InstallBootstrapContextErrorV1> {
        if encoded.is_empty()
            || encoded
                .bytes()
                .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-')
        {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        if URL_SAFE_NO_PAD.encode(&bytes) != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let record = std::str::from_utf8(&bytes)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        parse_install_bootstrap_record(record)
    }
}

impl PlatformBootstrapMappingV1 {
    /// Constructs a canonical Lima mapping from already-observed values.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    ///
    /// let host = InstallBootstrapContextCarrierV1::from_context(
    ///     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// )?;
    /// let mapping = PlatformBootstrapMappingV1::new_lima(
    ///     &host,
    ///     "substrate",
    ///     "0123456789abcdef0123456789abcdef",
    ///     "/Users/alice/.lima",
    ///     "/home/substrate/.substrate",
    ///     "substrate",
    ///     1000,
    ///     "/opt/substrate/sock/agent.sock",
    ///     "/run/substrate.sock",
    /// )?;
    /// assert_eq!(mapping.host_platform_control_root, "/Users/alice/.lima");
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_lima(
        host_carrier: &InstallBootstrapContextCarrierV1,
        vm_name: &str,
        guest_machine_id: &str,
        host_platform_control_root: &str,
        realized_substrate_home: &str,
        realized_principal_account: &str,
        realized_principal_uid: u32,
        transport_host: &str,
        transport_guest_socket: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        let mapping = Self {
            host_context_commitment: host_carrier.host_context_commitment.clone(),
            platform_instance: PlatformInstanceIdentityV1::Lima {
                vm_name: vm_name.to_string(),
                guest_machine_id: guest_machine_id.to_string(),
            },
            host_platform_control_root: normalize_unix_install_bootstrap_path(
                host_platform_control_root,
            )?,
            realized_substrate_home: normalize_unix_install_bootstrap_path(
                realized_substrate_home,
            )?,
            realized_principal: PlatformPrincipalV1::Unix {
                account: realized_principal_account.to_string(),
                uid: realized_principal_uid,
            },
            realized_transport: PlatformTransportIdentityV1::Lima {
                host_socket: normalize_unix_install_bootstrap_path(transport_host)?,
                guest_socket: normalize_unix_install_bootstrap_path(transport_guest_socket)?,
            },
        };
        mapping.validate(host_carrier)?;
        Ok(mapping)
    }

    /// Constructs a canonical WSL mapping from already-observed values.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    ///
    /// let host = InstallBootstrapContextCarrierV1::from_context(
    ///     InstallBootstrapContextV1::new_windows(
    ///         r"C:\Substrate",
    ///         r"ACME\Alice",
    ///         "S-1-5-21-1000",
    ///     )?,
    /// )?;
    /// let mapping = PlatformBootstrapMappingV1::new_wsl(
    ///     &host,
    ///     "Substrate-WSL",
    ///     "abcdef0123456789abcdef0123456789",
    ///     r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope",
    ///     "/home/substrate/.substrate",
    ///     "substrate",
    ///     1000,
    ///     r"\\.\pipe\Substrate-Agent",
    ///     "/run/substrate.sock",
    /// )?;
    /// assert_eq!(
    ///     mapping.host_platform_control_root,
    ///     r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope"
    /// );
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_wsl(
        host_carrier: &InstallBootstrapContextCarrierV1,
        distro_name: &str,
        guest_machine_id: &str,
        host_platform_control_root: &str,
        realized_substrate_home: &str,
        realized_principal_account: &str,
        realized_principal_uid: u32,
        transport_host: &str,
        transport_guest_socket: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        let mapping = Self {
            host_context_commitment: host_carrier.host_context_commitment.clone(),
            platform_instance: PlatformInstanceIdentityV1::Wsl {
                distro_name: distro_name.to_string(),
                guest_machine_id: guest_machine_id.to_string(),
            },
            host_platform_control_root: normalize_windows_install_bootstrap_path(
                host_platform_control_root,
            )?,
            realized_substrate_home: normalize_unix_install_bootstrap_path(
                realized_substrate_home,
            )?,
            realized_principal: PlatformPrincipalV1::Unix {
                account: realized_principal_account.to_string(),
                uid: realized_principal_uid,
            },
            realized_transport: PlatformTransportIdentityV1::Wsl {
                pipe_path: normalize_windows_pipe_path(transport_host)?,
                guest_socket: normalize_unix_install_bootstrap_path(transport_guest_socket)?,
            },
        };
        mapping.validate(host_carrier)?;
        Ok(mapping)
    }

    /// Validates canonical fields and their binding to the supplied host carrier.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// mapping.validate(&host)?;
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn validate(
        &self,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<(), InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        if !is_lower_hex_digest(&self.host_context_commitment) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        if self.host_context_commitment != host_carrier.host_context_commitment {
            return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
        }

        let PlatformPrincipalV1::Unix { account, .. } = &self.realized_principal else {
            return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
        };
        if !valid_principal_text(account) {
            return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
        }

        match (&self.platform_instance, &self.realized_transport) {
            (
                PlatformInstanceIdentityV1::Lima {
                    vm_name,
                    guest_machine_id,
                },
                PlatformTransportIdentityV1::Lima {
                    host_socket,
                    guest_socket,
                },
            ) => {
                if !valid_principal_text(vm_name)
                    || !is_lower_hex_machine_id(guest_machine_id)
                    || normalize_unix_install_bootstrap_path(&self.host_platform_control_root)?
                        != self.host_platform_control_root
                    || normalize_unix_install_bootstrap_path(&self.realized_substrate_home)?
                        != self.realized_substrate_home
                    || normalize_unix_install_bootstrap_path(host_socket)? != *host_socket
                    || normalize_unix_install_bootstrap_path(guest_socket)? != *guest_socket
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidContext);
                }
            }
            (
                PlatformInstanceIdentityV1::Wsl {
                    distro_name,
                    guest_machine_id,
                },
                PlatformTransportIdentityV1::Wsl {
                    pipe_path,
                    guest_socket,
                },
            ) => {
                if !valid_principal_text(distro_name)
                    || !is_lower_hex_machine_id(guest_machine_id)
                    || normalize_windows_install_bootstrap_path(&self.host_platform_control_root)?
                        != self.host_platform_control_root
                    || normalize_unix_install_bootstrap_path(&self.realized_substrate_home)?
                        != self.realized_substrate_home
                    || normalize_windows_pipe_path(pipe_path)? != *pipe_path
                    || normalize_unix_install_bootstrap_path(guest_socket)? != *guest_socket
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidContext);
                }
            }
            _ => return Err(InstallBootstrapContextErrorV1::InvalidContext),
        }
        Ok(())
    }

    /// Encodes the exact canonical thirteen-line mapping record as unpadded base64url.
    ///
    /// ```
    /// # use transport_api_types::{
    /// #     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    /// #     PlatformBootstrapMappingV1,
    /// # };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// let encoded = mapping.encode(&host)?;
    /// assert!(!encoded.contains('='));
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn encode(
        &self,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<String, InstallBootstrapContextErrorV1> {
        self.validate(host_carrier)?;
        Ok(URL_SAFE_NO_PAD.encode(platform_bootstrap_mapping_record(self)?))
    }

    /// Decodes and canonically re-encodes a mapping bound to the supplied host carrier.
    ///
    /// ```
    /// # use transport_api_types::{
    /// #     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    /// #     PlatformBootstrapMappingV1,
    /// # };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// # let encoded = mapping.encode(&host)?;
    /// let decoded = PlatformBootstrapMappingV1::decode(&encoded, &host)?;
    /// assert_eq!(decoded, mapping);
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn decode(
        encoded: &str,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        if !valid_unpadded_base64url(encoded) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        if URL_SAFE_NO_PAD.encode(&bytes) != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let record = std::str::from_utf8(&bytes)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        let mapping = parse_platform_bootstrap_mapping_record(record, host_carrier)?;
        if mapping.encode(host_carrier)? != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        Ok(mapping)
    }
}

impl WindowsForwarderScopeV1 {
    /// Derives the canonical six-line Windows forwarder scope digest.
    ///
    /// ```
    /// use transport_api_types::WindowsForwarderScopeV1;
    ///
    /// let scope = WindowsForwarderScopeV1::derive(
    ///     "S-1-5-21-1000",
    ///     "Substrate-WSL",
    ///     "abcdef0123456789abcdef0123456789",
    ///     r"\\.\pipe\Substrate-Agent",
    /// )?;
    /// assert_eq!(
    ///     scope.0,
    ///     "3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a"
    /// );
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn derive(
        windows_sid: &str,
        distro_name: &str,
        guest_machine_id: &str,
        pipe_path: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let digest = Sha256::digest(windows_forwarder_scope_input(
            windows_sid,
            distro_name,
            guest_machine_id,
            pipe_path,
        )?);
        Ok(Self(format!("{digest:x}")))
    }
}

/// Normalizes the case-insensitive name in a canonical Windows named-pipe path.
///
/// ```
/// use transport_api_types::normalize_windows_pipe_path;
///
/// assert_eq!(
///     normalize_windows_pipe_path(r"\\.\pipe\Substrate-Agent")?,
///     r"\\.\pipe\substrate-agent"
/// );
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
pub fn normalize_windows_pipe_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let Some(name) = raw.strip_prefix(r"\\.\pipe\") else {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    };
    if name.is_empty()
        || name.len() > 128
        || !name.is_ascii()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!(r"\\.\pipe\{}", name.to_ascii_lowercase()))
}

pub fn normalize_unix_install_bootstrap_path(
    raw: &str,
) -> Result<String, InstallBootstrapContextErrorV1> {
    if raw.is_empty()
        || raw == "/"
        || !raw.starts_with('/')
        || raw.starts_with("//")
        || raw.contains('\0')
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }

    let mut components = Vec::new();
    for component in raw[1..].split('/') {
        if component.is_empty() {
            continue;
        }
        if component == "." || component == ".." {
            return Err(InstallBootstrapContextErrorV1::InvalidPath);
        }
        components.push(component);
    }
    if components.is_empty() {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!("/{}", components.join("/")))
}

pub fn normalize_windows_install_bootstrap_path(
    raw: &str,
) -> Result<String, InstallBootstrapContextErrorV1> {
    if raw.is_empty() || raw.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    let normalized_separators = raw.replace('/', "\\");
    let lower = normalized_separators.to_ascii_lowercase();
    if lower.starts_with(r"\\?\") || lower.starts_with(r"\\.\") {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }

    let normalized = if normalized_separators.starts_with(r"\\") {
        normalize_windows_unc_path(&normalized_separators)?
    } else {
        normalize_windows_drive_path(&normalized_separators)?
    };
    Ok(normalized)
}

fn normalize_windows_drive_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let bytes = raw.as_bytes();
    if bytes.len() < 4 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' || bytes[2] != b'\\' {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    let components = windows_components(&raw[3..])?;
    let drive = (bytes[0] as char).to_ascii_uppercase();
    Ok(format!("{drive}:\\{}", components.join("\\")))
}

fn normalize_windows_unc_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let components = windows_components(&raw[2..])?;
    if components.len() < 3 {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!(r"\\{}", components.join("\\")))
}

fn windows_components(raw: &str) -> Result<Vec<&str>, InstallBootstrapContextErrorV1> {
    let components = raw
        .split('\\')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !valid_windows_component(component))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(components)
}

fn valid_windows_component(component: &str) -> bool {
    if component == "."
        || component == ".."
        || component.ends_with('.')
        || component.ends_with(' ')
        || component.chars().any(|ch| {
            ch == '\0'
                || ('\u{1}'..='\u{1f}').contains(&ch)
                || matches!(ch, '<' | '>' | '"' | '|' | '?' | '*' | ':')
        })
    {
        return false;
    }
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !reserved_numbered_windows_name(&stem, "COM")
        && !reserved_numbered_windows_name(&stem, "LPT")
}

fn reserved_numbered_windows_name(stem: &str, prefix: &str) -> bool {
    let Some(suffix) = stem.strip_prefix(prefix) else {
        return false;
    };
    matches!(
        suffix,
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
    )
}

fn valid_principal_text(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r'))
}

fn valid_windows_sid(sid: &str) -> bool {
    let mut parts = sid.split('-');
    if parts.next() != Some("S") {
        return false;
    }
    let numeric = parts.collect::<Vec<_>>();
    numeric.len() >= 2
        && numeric.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (*part == "0" || !part.starts_with('0'))
        })
}

fn commitment_input_for_context(
    context: &InstallBootstrapContextV1,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    context.validate()?;
    let selected = encode_inner_field(&context.selected_host_prefix);
    let home = encode_inner_field(&context.host_substrate_home);
    let root = encode_inner_field(&context.host_substrate_root);
    let principal = match &context.intended_host_principal {
        PlatformPrincipalV1::Unix { account, uid } => format!(
            "principal_kind=unix\nprincipal_account={}\nprincipal_uid={}\n",
            encode_inner_field(account),
            uid
        ),
        PlatformPrincipalV1::Windows { account, sid } => format!(
            "principal_kind=windows\nprincipal_account={}\nprincipal_sid={}\n",
            encode_inner_field(account),
            encode_inner_field(sid)
        ),
    };
    Ok(format!(
        "domain={INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1}\nversion=1\nselected_host_prefix={selected}\nhost_substrate_home={home}\nhost_substrate_root={root}\n{principal}"
    )
    .into_bytes())
}

fn commitment_for_context(
    context: &InstallBootstrapContextV1,
) -> Result<String, InstallBootstrapContextErrorV1> {
    let digest = Sha256::digest(commitment_input_for_context(context)?);
    Ok(format!("{digest:x}"))
}

fn encode_inner_field(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_inner_field(value: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-')
    {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
    if URL_SAFE_NO_PAD.encode(&decoded) != value {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    String::from_utf8(decoded).map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)
}

fn valid_unpadded_base64url(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn platform_bootstrap_mapping_record(
    mapping: &PlatformBootstrapMappingV1,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    let (platform_kind, instance_name, guest_machine_id) = match &mapping.platform_instance {
        PlatformInstanceIdentityV1::Lima {
            vm_name,
            guest_machine_id,
        } => ("lima", vm_name, guest_machine_id),
        PlatformInstanceIdentityV1::Wsl {
            distro_name,
            guest_machine_id,
        } => ("wsl", distro_name, guest_machine_id),
    };
    let (transport_kind, transport_host, transport_guest_socket) = match &mapping.realized_transport
    {
        PlatformTransportIdentityV1::Lima {
            host_socket,
            guest_socket,
        } => ("lima", host_socket, guest_socket),
        PlatformTransportIdentityV1::Wsl {
            pipe_path,
            guest_socket,
        } => ("wsl", pipe_path, guest_socket),
    };
    let PlatformPrincipalV1::Unix { account, uid } = &mapping.realized_principal else {
        return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
    };
    Ok(format!(
        "domain={PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1}\n\
version=1\n\
host_context_commitment={}\n\
platform_kind={platform_kind}\n\
instance_name={}\n\
guest_machine_id={guest_machine_id}\n\
host_platform_control_root={}\n\
realized_substrate_home={}\n\
realized_principal_account={}\n\
realized_principal_uid={uid}\n\
transport_kind={transport_kind}\n\
transport_host={}\n\
transport_guest_socket={}\n",
        mapping.host_context_commitment,
        encode_inner_field(instance_name),
        encode_inner_field(&mapping.host_platform_control_root),
        encode_inner_field(&mapping.realized_substrate_home),
        encode_inner_field(account),
        encode_inner_field(transport_host),
        encode_inner_field(transport_guest_socket),
    )
    .into_bytes())
}

fn parse_platform_bootstrap_mapping_record(
    record: &str,
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Result<PlatformBootstrapMappingV1, InstallBootstrapContextErrorV1> {
    if !record.ends_with('\n') || record.contains('\r') || record.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let lines = record.split_terminator('\n').collect::<Vec<_>>();
    if lines.len() != 13 {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    require_record_value(lines[0], "domain", PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1)?;
    require_record_value(lines[1], "version", "1")?;
    let host_context_commitment = record_value(lines[2], "host_context_commitment")?.to_string();
    if !is_lower_hex_digest(&host_context_commitment) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    if host_context_commitment != host_carrier.host_context_commitment {
        return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
    }

    let platform_kind = record_value(lines[3], "platform_kind")?;
    let instance_name = decode_inner_field(record_value(lines[4], "instance_name")?)?;
    let guest_machine_id = record_value(lines[5], "guest_machine_id")?;
    if !is_lower_hex_machine_id(guest_machine_id) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let host_platform_control_root =
        decode_inner_field(record_value(lines[6], "host_platform_control_root")?)?;
    let realized_substrate_home =
        decode_inner_field(record_value(lines[7], "realized_substrate_home")?)?;
    let realized_principal_account =
        decode_inner_field(record_value(lines[8], "realized_principal_account")?)?;
    let realized_principal_uid =
        parse_canonical_u32(record_value(lines[9], "realized_principal_uid")?)?;
    let transport_kind = record_value(lines[10], "transport_kind")?;
    let transport_host = decode_inner_field(record_value(lines[11], "transport_host")?)?;
    let transport_guest_socket =
        decode_inner_field(record_value(lines[12], "transport_guest_socket")?)?;

    match (platform_kind, transport_kind) {
        ("lima", "lima") => PlatformBootstrapMappingV1::new_lima(
            host_carrier,
            &instance_name,
            guest_machine_id,
            &host_platform_control_root,
            &realized_substrate_home,
            &realized_principal_account,
            realized_principal_uid,
            &transport_host,
            &transport_guest_socket,
        ),
        ("wsl", "wsl") => PlatformBootstrapMappingV1::new_wsl(
            host_carrier,
            &instance_name,
            guest_machine_id,
            &host_platform_control_root,
            &realized_substrate_home,
            &realized_principal_account,
            realized_principal_uid,
            &transport_host,
            &transport_guest_socket,
        ),
        _ => Err(InstallBootstrapContextErrorV1::InvalidCarrier),
    }
}

fn parse_canonical_u32(value: &str) -> Result<u32, InstallBootstrapContextErrorV1> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    value
        .parse()
        .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)
}

fn is_lower_hex_machine_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn windows_forwarder_scope_input(
    windows_sid: &str,
    distro_name: &str,
    guest_machine_id: &str,
    pipe_path: &str,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    if !valid_windows_sid(windows_sid)
        || !valid_principal_text(distro_name)
        || !is_lower_hex_machine_id(guest_machine_id)
    {
        return Err(InstallBootstrapContextErrorV1::InvalidContext);
    }
    let pipe_path = normalize_windows_pipe_path(pipe_path)?;
    Ok(format!(
        "domain={WINDOWS_FORWARDER_SCOPE_DOMAIN_V1}\n\
version=1\n\
windows_sid={}\n\
distro_name={}\n\
guest_machine_id={guest_machine_id}\n\
pipe_path={}\n",
        encode_inner_field(windows_sid),
        encode_inner_field(distro_name),
        encode_inner_field(&pipe_path),
    )
    .into_bytes())
}

fn parse_install_bootstrap_record(
    record: &str,
) -> Result<InstallBootstrapContextCarrierV1, InstallBootstrapContextErrorV1> {
    if !record.ends_with('\n') || record.contains('\r') || record.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let lines = record.split_terminator('\n').collect::<Vec<_>>();
    if lines.len() != 9 {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    require_record_value(lines[0], "domain", INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1)?;
    require_record_value(lines[1], "version", "1")?;
    let selected_host_prefix = decode_inner_field(record_value(lines[2], "selected_host_prefix")?)?;
    let host_substrate_home = decode_inner_field(record_value(lines[3], "host_substrate_home")?)?;
    let host_substrate_root = decode_inner_field(record_value(lines[4], "host_substrate_root")?)?;
    let principal_kind = record_value(lines[5], "principal_kind")?;
    let account = decode_inner_field(record_value(lines[6], "principal_account")?)?;
    let intended_host_principal = match principal_kind {
        "unix" => {
            let raw_uid = record_value(lines[7], "principal_uid")?;
            if raw_uid.is_empty()
                || !raw_uid.bytes().all(|byte| byte.is_ascii_digit())
                || (raw_uid.len() > 1 && raw_uid.starts_with('0'))
            {
                return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
            }
            let uid = raw_uid
                .parse::<u32>()
                .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
            PlatformPrincipalV1::Unix { account, uid }
        }
        "windows" => PlatformPrincipalV1::Windows {
            account,
            sid: decode_inner_field(record_value(lines[7], "principal_sid")?)?,
        },
        _ => return Err(InstallBootstrapContextErrorV1::InvalidCarrier),
    };
    let host_context_commitment = record_value(lines[8], "host_context_commitment")?.to_string();
    if !is_lower_hex_digest(&host_context_commitment) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let carrier = InstallBootstrapContextCarrierV1 {
        context: InstallBootstrapContextV1 {
            selected_host_prefix,
            host_substrate_home,
            host_substrate_root,
            intended_host_principal,
        },
        host_context_commitment,
    };
    carrier.validate()?;
    Ok(carrier)
}

fn record_value<'a>(
    line: &'a str,
    expected_key: &str,
) -> Result<&'a str, InstallBootstrapContextErrorV1> {
    let (key, value) = line
        .split_once('=')
        .ok_or(InstallBootstrapContextErrorV1::InvalidCarrier)?;
    if key != expected_key || value.contains('=') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    Ok(value)
}

fn require_record_value(
    line: &str,
    key: &str,
    expected_value: &str,
) -> Result<(), InstallBootstrapContextErrorV1> {
    if record_value(line, key)? != expected_value {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    Ok(())
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySnapshotWorldFsIsolationV2 {
    Workspace,
    Full,
}

fn default_true() -> bool {
    true
}

fn default_allow_list_dot() -> Vec<String> {
    vec![".".to_string()]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEnforcementV2 {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV2 {
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV2 {
    pub mode: WorldFsMode,
    pub isolation: PolicySnapshotWorldFsIsolationV2,
    pub require_world: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<WorldFsEnforcementV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<PolicySnapshotWorldFsDimensionV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotLimitsV2 {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_runtime_ms: u64,
    pub max_egress_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV2 {
    pub schema_version: u32,
    pub world_fs: PolicySnapshotWorldFsV2,
    pub net_allowed: Vec<String>,
    pub limits: PolicySnapshotLimitsV2,
}

impl PolicySnapshotV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 2)",
                self.schema_version
            ));
        }
        validate_world_fs_snapshot(&self.world_fs)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsDenyEnforcementV3 {
    Strict,
    PreferStrict,
    Weak,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsFailClosedV3 {
    #[serde(default)]
    pub routing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV3 {
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsWriteV3 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

impl Default for PolicySnapshotWorldFsWriteV3 {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV3 {
    #[serde(default = "default_true")]
    pub host_visible: bool,
    #[serde(default)]
    pub fail_closed: PolicySnapshotWorldFsFailClosedV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny_enforcement: Option<WorldFsDenyEnforcementV3>,
    #[serde(default)]
    pub caged_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default)]
    pub write: PolicySnapshotWorldFsWriteV3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV3 {
    pub schema_version: u32,
    #[serde(default)]
    pub net_allowed: Vec<String>,
    pub world_fs: PolicySnapshotWorldFsV3,
}

impl PolicySnapshotV3 {
    pub fn canonicalize(&self) -> Result<Self, String> {
        if self.schema_version != 3 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 3)",
                self.schema_version
            ));
        }

        let mut snapshot = self.clone();
        snapshot.net_allowed = canonicalize_net_allowed(&snapshot.net_allowed);

        if snapshot.world_fs.read.is_none() {
            snapshot.world_fs.read = Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: default_allow_list_dot(),
                deny_list: Vec::new(),
            });
        }

        let read_clone =
            snapshot
                .world_fs
                .read
                .clone()
                .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: default_allow_list_dot(),
                    deny_list: Vec::new(),
                });

        if snapshot.world_fs.discover.is_none() {
            snapshot.world_fs.discover = Some(read_clone.clone());
        }

        if snapshot.world_fs.write.allow_list.is_empty() {
            snapshot.world_fs.write.allow_list = default_allow_list_dot();
        }

        normalize_and_validate_world_fs_snapshot_v3(&mut snapshot.world_fs)?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), String> {
        let _ = self.canonicalize()?;
        Ok(())
    }

    pub fn resolve_world_network_routing(
        &self,
        world_net_filter: bool,
    ) -> Result<WorldNetworkRoutingV1, String> {
        let snapshot = self.canonicalize()?;
        let restrictive = snapshot.net_allowed.as_slice() != ["*"];
        let isolate_network = world_net_filter && restrictive;

        if isolate_network {
            validate_net_allowed_for_enforcement(&snapshot.net_allowed)?;
        }

        Ok(WorldNetworkRoutingV1 {
            isolate_network,
            allowed_domains: if isolate_network {
                snapshot.net_allowed
            } else {
                Vec::new()
            },
        })
    }
}

/// Exact durable E2 commitment reference for the retained worker's immutable cap.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DispatchPolicyCommitmentRefCarrierV1 {
    pub authority_store_id: String,
    pub commitment_id: String,
    pub exact_linkage_hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyCommitmentRefCarrierV1Def {
    authority_store_id: String,
    commitment_id: String,
    exact_linkage_hash: String,
}

impl DispatchPolicyCommitmentRefCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_narrowing_identity(
            "policy_snapshot_carrier.immutable_worker_cap_ref.authority_store_id",
            &self.authority_store_id,
        )?;
        validate_prefixed_uuid_v7(
            "policy_snapshot_carrier.immutable_worker_cap_ref.commitment_id",
            &self.commitment_id,
            "dpc_",
        )?;
        validate_lowercase_sha256_digest(
            "policy_snapshot_carrier.immutable_worker_cap_ref.exact_linkage_hash",
            &self.exact_linkage_hash,
        )
    }
}

impl<'de> Deserialize<'de> for DispatchPolicyCommitmentRefCarrierV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = DispatchPolicyCommitmentRefCarrierV1Def::deserialize(deserializer)?;
        let reference = Self {
            authority_store_id: value.authority_store_id,
            commitment_id: value.commitment_id,
            exact_linkage_hash: value.exact_linkage_hash,
        };
        reference.validate().map_err(serde::de::Error::custom)?;
        Ok(reference)
    }
}

/// Pre-transport identity of the retained turn that will later be exact-linked
/// to its durable B1/B2.1 evidence by E2.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RetainedTurnPolicyCommitmentSubjectV1 {
    pub retained_participant_id: String,
    pub active_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedTurnPolicyCommitmentSubjectV1Def {
    retained_participant_id: String,
    active_run_id: String,
    #[serde(default)]
    message_id: Option<String>,
}

impl RetainedTurnPolicyCommitmentSubjectV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_narrowing_identity(
            "policy_snapshot_carrier.subject.retained_participant_id",
            &self.retained_participant_id,
        )?;
        validate_narrowing_identity(
            "policy_snapshot_carrier.subject.active_run_id",
            &self.active_run_id,
        )?;
        if let Some(message_id) = self.message_id.as_deref() {
            validate_prefixed_uuid_v7(
                "policy_snapshot_carrier.subject.message_id",
                message_id,
                "wwm_",
            )?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RetainedTurnPolicyCommitmentSubjectV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = RetainedTurnPolicyCommitmentSubjectV1Def::deserialize(deserializer)?;
        let subject = Self {
            retained_participant_id: value.retained_participant_id,
            active_run_id: value.active_run_id,
            message_id: value.message_id,
        };
        subject.validate().map_err(serde::de::Error::custom)?;
        Ok(subject)
    }
}

/// Strict pre-transport carrier for the policy material of one retained turn.
///
/// The immutable worker-cap reference is the carrier's durable E2 identity. The
/// current turn's final E2 reference cannot exist until its B1 acceptance and
/// B2.1 execution claim are durable, so the turn subject is carried separately
/// and exact-joined during final E2 publication.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DispatchPolicySnapshotCarrierV1 {
    pub schema_version: u32,
    pub immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub immutable_worker_cap_created_revision: u64,
    pub immutable_worker_cap_application_revision: u64,
    pub subject: RetainedTurnPolicyCommitmentSubjectV1,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub caller_backend_id: String,
    pub target_backend_id: String,
    pub target_world: WorldBindingRefV1,
    pub policy_snapshot_bytes_base64: String,
    pub policy_snapshot_byte_length: u64,
    pub policy_snapshot_ref: PolicyRefV1,
    pub policy_snapshot_hash: String,
    pub policy_snapshot_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicySnapshotCarrierV1Def {
    schema_version: u32,
    immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    immutable_worker_cap_created_revision: u64,
    immutable_worker_cap_application_revision: u64,
    subject: RetainedTurnPolicyCommitmentSubjectV1,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    policy_snapshot_bytes_base64: String,
    policy_snapshot_byte_length: u64,
    policy_snapshot_ref: PolicyRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    #[serde(default)]
    reason: Option<String>,
}

impl DispatchPolicySnapshotCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_representation(true)
    }

    /// Validate ordinary wire representation only. The executing service must
    /// independently authenticate configured B1/E2 authority before any effect.
    pub fn validate_ordinary_representation(&self) -> Result<(), String> {
        self.validate_representation(false)
    }

    fn validate_representation(&self, require_positive_generation: bool) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported policy_snapshot_carrier.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        self.immutable_worker_cap_ref.validate()?;
        if self.immutable_worker_cap_created_revision == 0
            || self.immutable_worker_cap_application_revision == 0
        {
            return Err(
                "policy_snapshot_carrier immutable worker-cap revisions must be positive equality-only facts"
                    .to_string(),
            );
        }
        self.subject.validate()?;
        for (field, value) in [
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            ("caller_backend_id", self.caller_backend_id.as_str()),
            ("target_backend_id", self.target_backend_id.as_str()),
            ("target_world.world_id", self.target_world.world_id.as_str()),
            (
                "policy_snapshot_revision",
                self.policy_snapshot_revision.as_str(),
            ),
        ] {
            validate_narrowing_identity(&format!("policy_snapshot_carrier.{field}"), value)?;
        }
        validate_gateway_backend_id_selector(&self.caller_backend_id).map_err(|_| {
            "policy_snapshot_carrier.caller_backend_id must be an exact <kind>:<name> selector"
                .to_string()
        })?;
        validate_gateway_backend_id_selector(&self.target_backend_id).map_err(|_| {
            "policy_snapshot_carrier.target_backend_id must be an exact <kind>:<name> selector"
                .to_string()
        })?;
        if require_positive_generation && self.target_world.world_generation == 0 {
            return Err(
                "policy_snapshot_carrier.target_world.world_generation must be positive"
                    .to_string(),
            );
        }
        self.policy_snapshot_ref.validate()?;
        if let Some(reason) = self.reason.as_deref() {
            validate_narrowing_identity("policy_snapshot_carrier.reason", reason)?;
        }
        let _ = self.policy_snapshot()?;
        Ok(())
    }

    pub fn policy_snapshot_bytes(&self) -> Result<Vec<u8>, String> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.policy_snapshot_bytes_base64)
            .map_err(|_| {
                "policy_snapshot_carrier.policy_snapshot_bytes_base64 must be canonical padded RFC 4648 base64"
                    .to_string()
            })?;
        if bytes.len() as u64 != self.policy_snapshot_byte_length
            || base64::engine::general_purpose::STANDARD.encode(&bytes)
                != self.policy_snapshot_bytes_base64
        {
            return Err(
                "policy_snapshot_carrier policy snapshot byte length or canonical base64 changed"
                    .to_string(),
            );
        }
        Ok(bytes)
    }

    pub fn policy_snapshot(&self) -> Result<PolicySnapshotV3, String> {
        let bytes = self.policy_snapshot_bytes()?;
        let snapshot: PolicySnapshotV3 = serde_json::from_slice(&bytes)
            .map_err(|error| format!("decode exact carried PolicySnapshotV3 bytes: {error}"))?;
        let reserialized = serde_json::to_vec(&snapshot)
            .map_err(|error| format!("reserialize exact carried PolicySnapshotV3: {error}"))?;
        if reserialized != bytes {
            return Err(
                "policy_snapshot_carrier bytes must equal serde_json::to_vec(decoded PolicySnapshotV3)"
                    .to_string(),
            );
        }
        let canonical = snapshot.canonicalize()?;
        let canonical_bytes = serde_json::to_vec(&canonical)
            .map_err(|error| format!("serialize canonical carried PolicySnapshotV3: {error}"))?;
        if canonical_bytes != bytes {
            return Err(
                "policy_snapshot_carrier bytes must already contain the canonical E1 PolicySnapshotV3"
                    .to_string(),
            );
        }
        validate_lowercase_sha256_digest(
            "policy_snapshot_carrier.policy_snapshot_hash",
            &self.policy_snapshot_hash,
        )?;
        let expected_hash = format!("{:x}", Sha256::digest(&bytes));
        if self.policy_snapshot_hash != expected_hash {
            return Err(
                "policy_snapshot_carrier.policy_snapshot_hash must equal E1 SHA-256 over the exact carried bytes"
                    .to_string(),
            );
        }
        Ok(snapshot)
    }

    /// Rejects a serialized/untrusted copy unless every field exactly matches
    /// a carrier constructed from independently authenticated E2 authority.
    pub fn validate_exact_authenticated_copy(&self, authenticated: &Self) -> Result<(), String> {
        self.validate()?;
        authenticated.validate()?;
        if self != authenticated {
            return Err(
                "policy_snapshot_carrier changed from its authenticated E2 source".to_string(),
            );
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for DispatchPolicySnapshotCarrierV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = DispatchPolicySnapshotCarrierV1Def::deserialize(deserializer)?;
        let carrier = Self {
            schema_version: value.schema_version,
            immutable_worker_cap_ref: value.immutable_worker_cap_ref,
            immutable_worker_cap_created_revision: value.immutable_worker_cap_created_revision,
            immutable_worker_cap_application_revision: value
                .immutable_worker_cap_application_revision,
            subject: value.subject,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            caller_backend_id: value.caller_backend_id,
            target_backend_id: value.target_backend_id,
            target_world: value.target_world,
            policy_snapshot_bytes_base64: value.policy_snapshot_bytes_base64,
            policy_snapshot_byte_length: value.policy_snapshot_byte_length,
            policy_snapshot_ref: value.policy_snapshot_ref,
            policy_snapshot_hash: value.policy_snapshot_hash,
            policy_snapshot_revision: value.policy_snapshot_revision,
            reason: value.reason,
        };
        carrier.validate().map_err(serde::de::Error::custom)?;
        Ok(carrier)
    }
}

/// Ordinary intake representation, never an admission or capability. Its wire
/// fields are exactly the protected carrier's fields; only generation zero is
/// additionally representable. Default protected decoding remains positive-only.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
struct OrdinaryDispatchPolicySnapshotCarrierV1(DispatchPolicySnapshotCarrierV1);

impl<'de> Deserialize<'de> for OrdinaryDispatchPolicySnapshotCarrierV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = DispatchPolicySnapshotCarrierV1Def::deserialize(deserializer)?;
        let carrier = DispatchPolicySnapshotCarrierV1 {
            schema_version: value.schema_version,
            immutable_worker_cap_ref: value.immutable_worker_cap_ref,
            immutable_worker_cap_created_revision: value.immutable_worker_cap_created_revision,
            immutable_worker_cap_application_revision: value
                .immutable_worker_cap_application_revision,
            subject: value.subject,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            caller_backend_id: value.caller_backend_id,
            target_backend_id: value.target_backend_id,
            target_world: value.target_world,
            policy_snapshot_bytes_base64: value.policy_snapshot_bytes_base64,
            policy_snapshot_byte_length: value.policy_snapshot_byte_length,
            policy_snapshot_ref: value.policy_snapshot_ref,
            policy_snapshot_hash: value.policy_snapshot_hash,
            policy_snapshot_revision: value.policy_snapshot_revision,
            reason: value.reason,
        };
        carrier
            .validate_ordinary_representation()
            .map_err(serde::de::Error::custom)?;
        Ok(Self(carrier))
    }
}

pub fn canonicalize_net_allowed(entries: &[String]) -> Vec<String> {
    let mut canonical = Vec::with_capacity(entries.len());

    for raw in entries {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut normalized = trimmed.to_ascii_lowercase();
        normalized.truncate(normalized.trim_end_matches('.').len());
        if normalized.is_empty() {
            continue;
        }

        if normalized.starts_with('[') && normalized.ends_with(']') {
            let inner = &normalized[1..normalized.len() - 1];
            if matches!(inner.parse::<IpAddr>(), Ok(IpAddr::V6(_))) {
                normalized = inner.to_string();
            }
        }

        if normalized == "*" {
            return vec!["*".to_string()];
        }

        if !canonical.contains(&normalized) {
            canonical.push(normalized);
        }
    }

    canonical
}

pub fn validate_net_allowed_for_enforcement(entries: &[String]) -> Result<(), String> {
    let canonical = canonicalize_net_allowed(entries);

    for entry in &canonical {
        validate_net_allowed_entry_for_enforcement(entry)?;
    }

    Ok(())
}

fn validate_net_allowed_entry_for_enforcement(entry: &str) -> Result<(), String> {
    if entry == "*" {
        return Ok(());
    }

    if !entry.is_ascii() {
        return Err(
            "net_allowed entries must be ASCII; use punycode A-labels for IDNs".to_string(),
        );
    }

    if entry.contains("://") {
        return Err("net_allowed entries must not include URL schemes".to_string());
    }
    if entry.contains('/') {
        return Err("net_allowed entries must not include paths".to_string());
    }
    if entry.contains('?') {
        return Err("net_allowed entries must not include query strings".to_string());
    }
    if entry.contains('#') {
        return Err("net_allowed entries must not include URL fragments".to_string());
    }
    if entry.contains(['*', '[', ']']) {
        return Err("net_allowed wildcard forms other than '*' are not supported".to_string());
    }

    if entry.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if entry.contains(':') {
        return Err(
            "net_allowed entries must be hostnames or IP literals without ports".to_string(),
        );
    }

    validate_hostname(entry)
}

fn validate_hostname(entry: &str) -> Result<(), String> {
    if entry.starts_with('.') || entry.ends_with('.') {
        return Err("net_allowed hostnames must not start or end with '.'".to_string());
    }

    for label in entry.split('.') {
        if label.is_empty() {
            return Err("net_allowed hostnames must not contain empty labels".to_string());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("net_allowed hostname labels must not start or end with '-'".to_string());
        }
        if !label
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return Err(
                "net_allowed hostnames may contain only ASCII letters, digits, '-' and '.'"
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn normalize_project_pattern(raw: &str) -> Result<String, String> {
    let mut pattern = raw.trim();
    if pattern.is_empty() {
        return Err("pattern must be non-empty".to_string());
    }
    if pattern.starts_with('/') {
        return Err("absolute paths are not allowed".to_string());
    }

    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped;
    }

    let mut normalized = pattern.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = ".".to_string();
    }

    if normalized.split('/').any(|segment| segment == "..") {
        return Err("path segments must not be '..'".to_string());
    }

    Ok(normalized)
}

fn contains_any_glob_metacharacters(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[') || value.contains(']')
}

fn contains_unsupported_deny_metacharacters(value: &str) -> bool {
    value.contains('?') || value.contains('[') || value.contains(']')
}

fn validate_deny_wildcards(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            continue;
        }
        if run > 0 && run != 1 && run != 2 {
            return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
        }
        run = 0;
    }
    if run > 0 && run != 1 && run != 2 {
        return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
    }
    Ok(())
}

fn validate_dimension(prefix: &str, dim: &PolicySnapshotWorldFsDimensionV2) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if contains_any_glob_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
    }

    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if contains_unsupported_deny_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
    }

    Ok(())
}

fn validate_world_fs_snapshot(world_fs: &PolicySnapshotWorldFsV2) -> Result<(), String> {
    match world_fs.isolation {
        PolicySnapshotWorldFsIsolationV2::Workspace => {
            if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.discover.is_some() {
                return Err(
                    "world_fs.discover must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.read.is_some() {
                return Err(
                    "world_fs.read must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            if world_fs.write.is_some() {
                return Err(
                    "world_fs.write must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            Ok(())
        }
        PolicySnapshotWorldFsIsolationV2::Full => {
            let read = world_fs.read.as_ref().ok_or_else(|| {
                "world_fs.read must be present when world_fs.isolation=full".to_string()
            })?;
            validate_dimension("world_fs.read", read)?;

            if let Some(discover) = world_fs.discover.as_ref() {
                validate_dimension("world_fs.discover", discover)?;
            }

            match world_fs.mode {
                WorldFsMode::ReadOnly => {
                    if world_fs.write.is_some() {
                        return Err(
                            "world_fs.write must be omitted when world_fs.mode=read_only"
                                .to_string(),
                        );
                    }
                }
                WorldFsMode::Writable => {
                    let write = world_fs.write.as_ref().ok_or_else(|| {
                        "world_fs.write must be present when world_fs.mode=writable".to_string()
                    })?;
                    validate_dimension("world_fs.write", write)?;
                }
            }

            let any_deny = world_fs
                .read
                .as_ref()
                .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .discover
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .write
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty());

            if any_deny {
                if world_fs.enforcement.is_none() {
                    return Err(
                        "world_fs.enforcement must be present when any deny_list is non-empty"
                            .to_string(),
                    );
                }
                if !world_fs.require_world {
                    return Err("deny_list requires world_fs.require_world=true".to_string());
                }
            } else if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement is only valid when at least one deny_list is non-empty"
                        .to_string(),
                );
            }

            Ok(())
        }
    }
}

fn normalize_project_pattern_v3(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("pattern must not be empty".to_string());
    }

    if raw.starts_with('/') {
        return Err("absolute patterns are invalid".to_string());
    }

    let mut segments: Vec<&str> = Vec::new();
    for seg in raw.split('/') {
        let seg = seg.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err("pattern must not contain '..' segments".to_string());
        }
        segments.push(seg);
    }

    if segments.is_empty() {
        Ok(".".to_string())
    } else {
        Ok(segments.join("/"))
    }
}

fn validate_deny_wildcards_v3(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            if run > 2 {
                return Err(
                    "deny_list wildcard runs must be '*' or '**' (no '***' or longer)".to_string(),
                );
            }
        } else {
            run = 0;
        }
    }
    Ok(())
}

fn validate_dimension_v3(
    prefix: &str,
    dim: &mut PolicySnapshotWorldFsDimensionV3,
) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    let mut allow_out = Vec::with_capacity(dim.allow_list.len());
    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if normalized.contains(['*', '?', '[', ']']) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
        allow_out.push(normalized);
    }
    dim.allow_list = allow_out;

    let mut deny_out = Vec::with_capacity(dim.deny_list.len());
    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if normalized.contains(['?', '[', ']']) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards_v3(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        deny_out.push(normalized);
    }
    dim.deny_list = deny_out;

    Ok(())
}

fn normalize_and_validate_world_fs_snapshot_v3(
    world_fs: &mut PolicySnapshotWorldFsV3,
) -> Result<(), String> {
    if !world_fs.write.enabled && !world_fs.fail_closed.routing {
        return Err(
            "world_fs.write.enabled=false requires world_fs.fail_closed.routing=true".to_string(),
        );
    }

    let mut read = world_fs
        .read
        .clone()
        .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        });
    validate_dimension_v3("world_fs.read", &mut read)?;
    world_fs.read = Some(read.clone());

    let mut discover = world_fs.discover.clone().unwrap_or_else(|| read.clone());
    validate_dimension_v3("world_fs.discover", &mut discover)?;
    world_fs.discover = Some(discover.clone());

    let mut write_dim = PolicySnapshotWorldFsDimensionV3 {
        allow_list: world_fs.write.allow_list.clone(),
        deny_list: world_fs.write.deny_list.clone(),
    };
    validate_dimension_v3("world_fs.write", &mut write_dim)?;
    world_fs.write.allow_list = write_dim.allow_list;
    world_fs.write.deny_list = write_dim.deny_list;

    let any_deny = !read.deny_list.is_empty()
        || !discover.deny_list.is_empty()
        || !world_fs.write.deny_list.is_empty();

    if world_fs.host_visible && any_deny {
        return Err("deny_list usage requires world_fs.host_visible=false".to_string());
    }

    if any_deny && world_fs.deny_enforcement.is_none() {
        return Err(
            "world_fs.deny_enforcement must be present when any deny_list is non-empty".to_string(),
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_execs: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_egress_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldNetworkRoutingV1 {
    pub isolate_network: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ResolvedMemberRuntimeDescriptorDef")]
pub struct ResolvedMemberRuntimeDescriptorV1 {
    pub backend_kind: MemberRuntimeBackendKindV1,
    pub binary_path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolvedMemberRuntimeDescriptorDef {
    backend_kind: MemberRuntimeBackendKindV1,
    binary_path: String,
}

impl ResolvedMemberRuntimeDescriptorV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_request_field(
            "member_dispatch.resolved_runtime.binary_path",
            &self.binary_path,
        )?;
        if !Path::new(&self.binary_path).is_absolute() {
            return Err(
                "member_dispatch.resolved_runtime.binary_path must be an absolute path".to_string(),
            );
        }
        Ok(())
    }
}

impl TryFrom<ResolvedMemberRuntimeDescriptorDef> for ResolvedMemberRuntimeDescriptorV1 {
    type Error = String;

    fn try_from(value: ResolvedMemberRuntimeDescriptorDef) -> Result<Self, Self::Error> {
        let descriptor = Self {
            backend_kind: value.backend_kind,
            binary_path: value.binary_path,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum RetainedWorkerAuthorityObjectCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

impl RetainedWorkerAuthorityObjectCommitmentV1 {
    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::CanonicalSha256 { digest_hex } => {
                validate_lowercase_sha256_digest(field, digest_hex)
            }
            Self::StoreHmacSha256 {
                key_id,
                domain,
                digest_hex,
            } => {
                validate_non_empty_request_field(&format!("{field}.key_id"), key_id)?;
                validate_non_empty_request_field(&format!("{field}.domain"), domain)?;
                validate_lowercase_sha256_digest(field, digest_hex)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RetainedWorkerAdmissionCommitmentCarrierDef")]
pub struct RetainedWorkerAdmissionCommitmentCarrierV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub key_id: String,
    pub digest_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionCommitmentCarrierDef {
    schema_version: u32,
    algorithm: String,
    key_id: String,
    digest_hex: String,
}

impl RetainedWorkerAdmissionCommitmentCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported retained_worker_launch_authority.canonical_spawn_fingerprint.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        if self.algorithm != "hmac-sha-256" {
            return Err(
                "retained_worker_launch_authority.canonical_spawn_fingerprint.algorithm must be hmac-sha-256"
                    .to_string(),
            );
        }
        validate_non_empty_request_field(
            "retained_worker_launch_authority.canonical_spawn_fingerprint.key_id",
            &self.key_id,
        )?;
        validate_lowercase_sha256_digest(
            "retained_worker_launch_authority.canonical_spawn_fingerprint",
            &self.digest_hex,
        )
    }
}

impl TryFrom<RetainedWorkerAdmissionCommitmentCarrierDef>
    for RetainedWorkerAdmissionCommitmentCarrierV1
{
    type Error = String;

    fn try_from(value: RetainedWorkerAdmissionCommitmentCarrierDef) -> Result<Self, Self::Error> {
        let commitment = Self {
            schema_version: value.schema_version,
            algorithm: value.algorithm,
            key_id: value.key_id,
            digest_hex: value.digest_hex,
        };
        commitment.validate()?;
        Ok(commitment)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RetainedWorkerLaunchWorldBindingV1 {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RetainedWorkerLaunchAuthorityProofDef")]
pub struct RetainedWorkerLaunchAuthorityProofV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub issuer_request_id: String,
    pub canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    pub registration_id: String,
    pub registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
    pub authority_revision_after: u64,
    pub authority_record_commitment_after: RetainedWorkerAuthorityObjectCommitmentV1,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub retained_participant_id: String,
    pub bootstrap_run_id: String,
    pub transport_claim_id: String,
    pub backend_id: String,
    pub protocol: String,
    pub world_binding: RetainedWorkerLaunchWorldBindingV1,
    pub current_policy_ref_id: String,
    pub current_policy_revision: String,
    pub retained_worker_ref_id: String,
    pub retained_worker_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerLaunchAuthorityProofDef {
    schema_version: u32,
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    registration_id: String,
    registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: RetainedWorkerAuthorityObjectCommitmentV1,
    orchestration_session_id: String,
    caller_participant_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    transport_claim_id: String,
    backend_id: String,
    protocol: String,
    world_binding: RetainedWorkerLaunchWorldBindingV1,
    current_policy_ref_id: String,
    current_policy_revision: String,
    retained_worker_ref_id: String,
    retained_worker_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
}

impl RetainedWorkerLaunchAuthorityProofV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported retained_worker_launch_authority.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        for (field, value) in [
            ("authority_store_id", self.authority_store_id.as_str()),
            ("issuer_request_id", self.issuer_request_id.as_str()),
            ("registration_id", self.registration_id.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            (
                "retained_participant_id",
                self.retained_participant_id.as_str(),
            ),
            ("bootstrap_run_id", self.bootstrap_run_id.as_str()),
            ("transport_claim_id", self.transport_claim_id.as_str()),
            ("backend_id", self.backend_id.as_str()),
            ("protocol", self.protocol.as_str()),
            (
                "world_binding.world_id",
                self.world_binding.world_id.as_str(),
            ),
            ("current_policy_ref_id", self.current_policy_ref_id.as_str()),
            (
                "current_policy_revision",
                self.current_policy_revision.as_str(),
            ),
            (
                "retained_worker_ref_id",
                self.retained_worker_ref_id.as_str(),
            ),
        ] {
            validate_non_empty_request_field(
                &format!("retained_worker_launch_authority.{field}"),
                value,
            )?;
        }
        self.canonical_spawn_fingerprint.validate()?;
        self.registration_commitment
            .validate("retained_worker_launch_authority.registration_commitment")?;
        self.authority_record_commitment_after
            .validate("retained_worker_launch_authority.authority_record_commitment_after")?;
        self.retained_worker_commitment
            .validate("retained_worker_launch_authority.retained_worker_commitment")?;
        Ok(())
    }
}

impl TryFrom<RetainedWorkerLaunchAuthorityProofDef> for RetainedWorkerLaunchAuthorityProofV1 {
    type Error = String;

    fn try_from(value: RetainedWorkerLaunchAuthorityProofDef) -> Result<Self, Self::Error> {
        let proof = Self {
            schema_version: value.schema_version,
            authority_store_id: value.authority_store_id,
            issuer_request_id: value.issuer_request_id,
            canonical_spawn_fingerprint: value.canonical_spawn_fingerprint,
            registration_id: value.registration_id,
            registration_commitment: value.registration_commitment,
            authority_revision_after: value.authority_revision_after,
            authority_record_commitment_after: value.authority_record_commitment_after,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            retained_participant_id: value.retained_participant_id,
            bootstrap_run_id: value.bootstrap_run_id,
            transport_claim_id: value.transport_claim_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            world_binding: value.world_binding,
            current_policy_ref_id: value.current_policy_ref_id,
            current_policy_revision: value.current_policy_revision,
            retained_worker_ref_id: value.retained_worker_ref_id,
            retained_worker_commitment: value.retained_worker_commitment,
        };
        proof.validate()?;
        Ok(proof)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum E2MemberLaunchKindV1 {
    FreshSpawn,
    Fork,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct E2DispatchPolicyReservationRefCarrierV1 {
    pub authority_store_id: String,
    pub reservation_id: String,
    pub reservation_hash: String,
}

impl E2DispatchPolicyReservationRefCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("authority_store_id", self.authority_store_id.as_str()),
            ("reservation_id", self.reservation_id.as_str()),
        ] {
            validate_narrowing_identity(
                &format!("e2_launch_activation.reservation_ref.{field}"),
                value,
            )?;
        }
        validate_lowercase_sha256_digest(
            "e2_launch_activation.reservation_ref.reservation_hash",
            &self.reservation_hash,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum E2LaunchRequestCommitmentV1 {
    HmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
    CanonicalSha256 {
        domain: String,
        digest_hex: String,
    },
}

impl E2LaunchRequestCommitmentV1 {
    pub fn validate(&self) -> Result<(), String> {
        let (key_id, domain, digest_hex) = match self {
            Self::HmacSha256 {
                key_id,
                domain,
                digest_hex,
            } => (Some(key_id.as_str()), domain.as_str(), digest_hex.as_str()),
            Self::CanonicalSha256 { domain, digest_hex } => {
                (None, domain.as_str(), digest_hex.as_str())
            }
        };
        if let Some(key_id) = key_id {
            validate_narrowing_identity("e2_launch_activation.request_commitment.key_id", key_id)?;
        }
        validate_narrowing_identity("e2_launch_activation.request_commitment.domain", domain)?;
        validate_lowercase_sha256_digest(
            "e2_launch_activation.request_commitment.digest_hex",
            digest_hex,
        )
    }
}

/// Strict shell-authenticated E2 authority pinned before retained-member launch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "E2MemberLaunchActivationCarrierDef")]
pub struct E2MemberLaunchActivationCarrierV1 {
    pub schema_version: u32,
    pub activation_id: String,
    pub launch_kind: E2MemberLaunchKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation_ref: Option<E2DispatchPolicyReservationRefCarrierV1>,
    pub commitment_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub immutable_worker_cap_created_revision: u64,
    pub immutable_worker_cap_application_revision: u64,
    pub policy_snapshot_bytes_base64: String,
    pub policy_snapshot_byte_length: u64,
    pub policy_snapshot_ref: PolicyRefV1,
    pub policy_snapshot_hash: String,
    pub policy_snapshot_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub request_id: String,
    pub idempotency_key: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub caller_backend_id: String,
    pub target_backend_id: String,
    pub retained_participant_id: String,
    pub bootstrap_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    pub target_world: WorldBindingRefV1,
    pub parent_policy_ref: PolicyRefV1,
    pub parent_policy_revision: String,
    pub request_commitment: E2LaunchRequestCommitmentV1,
    pub registry_publication_revision: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct E2MemberLaunchActivationCarrierDef {
    schema_version: u32,
    activation_id: String,
    launch_kind: E2MemberLaunchKindV1,
    #[serde(default)]
    reservation_ref: Option<E2DispatchPolicyReservationRefCarrierV1>,
    commitment_ref: DispatchPolicyCommitmentRefCarrierV1,
    immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    immutable_worker_cap_created_revision: u64,
    immutable_worker_cap_application_revision: u64,
    policy_snapshot_bytes_base64: String,
    policy_snapshot_byte_length: u64,
    policy_snapshot_ref: PolicyRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    #[serde(default)]
    reason: Option<String>,
    request_id: String,
    idempotency_key: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    #[serde(default)]
    source_participant_id: Option<String>,
    target_world: WorldBindingRefV1,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    request_commitment: E2LaunchRequestCommitmentV1,
    registry_publication_revision: u64,
}

impl E2MemberLaunchActivationCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported e2_launch_activation.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        let activation_suffix = self.activation_id.strip_prefix("e2a_").ok_or_else(|| {
            "e2_launch_activation.activation_id must be e2a_ plus 32 lowercase hexadecimal characters"
                .to_string()
        })?;
        if activation_suffix.len() != 32
            || !activation_suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "e2_launch_activation.activation_id must be e2a_ plus 32 lowercase hexadecimal characters"
                    .to_string(),
            );
        }
        self.commitment_ref.validate()?;
        self.immutable_worker_cap_ref.validate()?;
        if self.immutable_worker_cap_created_revision == 0
            || self.immutable_worker_cap_application_revision == 0
            || self.registry_publication_revision == 0
            || self.registry_publication_revision != self.immutable_worker_cap_application_revision
        {
            return Err(
                "e2_launch_activation revisions must be positive exact publication facts"
                    .to_string(),
            );
        }
        match self.launch_kind {
            E2MemberLaunchKindV1::FreshSpawn => {
                if self.reservation_ref.is_none()
                    || self.source_participant_id.is_some()
                    || !matches!(
                        self.request_commitment,
                        E2LaunchRequestCommitmentV1::HmacSha256 { .. }
                    )
                {
                    return Err("fresh_spawn E2 activation requires reservation/HMAC and forbids a source participant".to_string());
                }
            }
            E2MemberLaunchKindV1::Fork => {
                if self.reservation_ref.is_some()
                    || self.source_participant_id.is_none()
                    || !matches!(
                        self.request_commitment,
                        E2LaunchRequestCommitmentV1::CanonicalSha256 { .. }
                    )
                {
                    return Err("fork E2 activation requires source/SHA-256 and forbids a Fresh Spawn reservation".to_string());
                }
            }
        }
        if let Some(reservation_ref) = self.reservation_ref.as_ref() {
            reservation_ref.validate()?;
            if reservation_ref.authority_store_id != self.commitment_ref.authority_store_id {
                return Err("e2_launch_activation reservation/commitment stores differ".to_string());
            }
        }
        self.request_commitment.validate()?;
        self.policy_snapshot_ref.validate()?;
        self.parent_policy_ref.validate()?;
        for (field, value) in [
            ("request_id", self.request_id.as_str()),
            ("idempotency_key", self.idempotency_key.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            ("caller_backend_id", self.caller_backend_id.as_str()),
            ("target_backend_id", self.target_backend_id.as_str()),
            (
                "retained_participant_id",
                self.retained_participant_id.as_str(),
            ),
            ("bootstrap_run_id", self.bootstrap_run_id.as_str()),
            ("target_world.world_id", self.target_world.world_id.as_str()),
            (
                "policy_snapshot_revision",
                self.policy_snapshot_revision.as_str(),
            ),
            (
                "parent_policy_revision",
                self.parent_policy_revision.as_str(),
            ),
        ] {
            validate_narrowing_identity(&format!("e2_launch_activation.{field}"), value)?;
        }
        if let Some(source) = self.source_participant_id.as_deref() {
            validate_narrowing_identity("e2_launch_activation.source_participant_id", source)?;
        }
        if let Some(reason) = self.reason.as_deref() {
            validate_narrowing_identity("e2_launch_activation.reason", reason)?;
        }
        validate_gateway_backend_id_selector(&self.caller_backend_id).map_err(|_| {
            "e2_launch_activation.caller_backend_id must be an exact selector".to_string()
        })?;
        validate_gateway_backend_id_selector(&self.target_backend_id).map_err(|_| {
            "e2_launch_activation.target_backend_id must be an exact selector".to_string()
        })?;
        let _ = self.policy_snapshot()?;
        Ok(())
    }

    pub fn policy_snapshot(&self) -> Result<PolicySnapshotV3, String> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.policy_snapshot_bytes_base64)
            .map_err(|_| "e2_launch_activation snapshot is not canonical base64".to_string())?;
        if bytes.len() as u64 != self.policy_snapshot_byte_length
            || base64::engine::general_purpose::STANDARD.encode(&bytes)
                != self.policy_snapshot_bytes_base64
        {
            return Err("e2_launch_activation snapshot byte identity changed".to_string());
        }
        let snapshot: PolicySnapshotV3 = serde_json::from_slice(&bytes)
            .map_err(|error| format!("decode E2 launch PolicySnapshotV3: {error}"))?;
        if serde_json::to_vec(&snapshot).map_err(|error| error.to_string())? != bytes
            || serde_json::to_vec(&snapshot.clone().canonicalize()?)
                .map_err(|error| error.to_string())?
                != bytes
            || format!("{:x}", Sha256::digest(&bytes)) != self.policy_snapshot_hash
        {
            return Err("e2_launch_activation exact E1 snapshot bytes/hash changed".to_string());
        }
        validate_lowercase_sha256_digest(
            "e2_launch_activation.policy_snapshot_hash",
            &self.policy_snapshot_hash,
        )?;
        Ok(snapshot)
    }
}

impl TryFrom<E2MemberLaunchActivationCarrierDef> for E2MemberLaunchActivationCarrierV1 {
    type Error = String;

    fn try_from(value: E2MemberLaunchActivationCarrierDef) -> Result<Self, Self::Error> {
        let carrier = Self {
            schema_version: value.schema_version,
            activation_id: value.activation_id,
            launch_kind: value.launch_kind,
            reservation_ref: value.reservation_ref,
            commitment_ref: value.commitment_ref,
            immutable_worker_cap_ref: value.immutable_worker_cap_ref,
            immutable_worker_cap_created_revision: value.immutable_worker_cap_created_revision,
            immutable_worker_cap_application_revision: value
                .immutable_worker_cap_application_revision,
            policy_snapshot_bytes_base64: value.policy_snapshot_bytes_base64,
            policy_snapshot_byte_length: value.policy_snapshot_byte_length,
            policy_snapshot_ref: value.policy_snapshot_ref,
            policy_snapshot_hash: value.policy_snapshot_hash,
            policy_snapshot_revision: value.policy_snapshot_revision,
            reason: value.reason,
            request_id: value.request_id,
            idempotency_key: value.idempotency_key,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            caller_backend_id: value.caller_backend_id,
            target_backend_id: value.target_backend_id,
            retained_participant_id: value.retained_participant_id,
            bootstrap_run_id: value.bootstrap_run_id,
            source_participant_id: value.source_participant_id,
            target_world: value.target_world,
            parent_policy_ref: value.parent_policy_ref,
            parent_policy_revision: value.parent_policy_revision,
            request_commitment: value.request_commitment,
            registry_publication_revision: value.registry_publication_revision,
        };
        carrier.validate()?;
        Ok(carrier)
    }
}

fn validate_lowercase_sha256_digest(field: &str, digest_hex: &str) -> Result<(), String> {
    if digest_hex.len() != 64
        || !digest_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{field}.digest_hex must be exactly 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ConfigProjectionRefV1Def")]
pub struct ConfigProjectionRefV1 {
    pub authority_store_id: String,
    pub series_id: String,
    pub record_id: String,
    pub revision: u64,
    pub record_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigProjectionRefV1Def {
    authority_store_id: String,
    series_id: String,
    record_id: String,
    revision: u64,
    record_hash: String,
}

impl ConfigProjectionRefV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_prefixed_uuid_v7(
            "config_projection.dormant_projection_ref.authority_store_id",
            &self.authority_store_id,
            "cpa_",
        )?;
        validate_prefixed_uuid_v7(
            "config_projection.dormant_projection_ref.series_id",
            &self.series_id,
            "cps_",
        )?;
        validate_prefixed_uuid_v7(
            "config_projection.dormant_projection_ref.record_id",
            &self.record_id,
            "cpr_",
        )?;
        if self.revision == 0 {
            return Err(
                "config_projection.dormant_projection_ref.revision must be positive".to_string(),
            );
        }
        validate_lowercase_sha256_digest(
            "config_projection.dormant_projection_ref.record_hash",
            &self.record_hash,
        )
    }
}

impl TryFrom<ConfigProjectionRefV1Def> for ConfigProjectionRefV1 {
    type Error = String;

    fn try_from(value: ConfigProjectionRefV1Def) -> Result<Self, Self::Error> {
        let reference = Self {
            authority_store_id: value.authority_store_id,
            series_id: value.series_id,
            record_id: value.record_id,
            revision: value.revision,
            record_hash: value.record_hash,
        };
        reference.validate()?;
        Ok(reference)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ConfigProjectionAuthoringInputRefV1Def")]
pub struct ConfigProjectionAuthoringInputRefV1 {
    pub authority_store_id: String,
    pub effective_config_source_hash: String,
    pub agent_inventory_source_hash: String,
    pub runtime_artifact_manifest_id: String,
    pub runtime_artifact_manifest_revision: u64,
    pub runtime_artifact_manifest_hash: String,
    pub input_ref_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigProjectionAuthoringInputRefV1Def {
    authority_store_id: String,
    effective_config_source_hash: String,
    agent_inventory_source_hash: String,
    runtime_artifact_manifest_id: String,
    runtime_artifact_manifest_revision: u64,
    runtime_artifact_manifest_hash: String,
    input_ref_hash: String,
}

#[derive(Serialize)]
struct ConfigProjectionAuthoringInputRefV1HashBody<'a> {
    agent_inventory_source_hash: &'a str,
    authority_store_id: &'a str,
    effective_config_source_hash: &'a str,
    runtime_artifact_manifest_hash: &'a str,
    runtime_artifact_manifest_id: &'a str,
    runtime_artifact_manifest_revision: u64,
}

#[derive(Serialize)]
struct ConfigProjectionAuthoringInputRefV1HashPreimage<'a> {
    domain: &'static str,
    input_ref: ConfigProjectionAuthoringInputRefV1HashBody<'a>,
}

impl ConfigProjectionAuthoringInputRefV1 {
    pub fn canonical_hash(&self) -> Result<String, String> {
        let preimage = ConfigProjectionAuthoringInputRefV1HashPreimage {
            domain: "substrate.e3.config-projection-authoring-input-ref.v1",
            input_ref: ConfigProjectionAuthoringInputRefV1HashBody {
                agent_inventory_source_hash: &self.agent_inventory_source_hash,
                authority_store_id: &self.authority_store_id,
                effective_config_source_hash: &self.effective_config_source_hash,
                runtime_artifact_manifest_hash: &self.runtime_artifact_manifest_hash,
                runtime_artifact_manifest_id: &self.runtime_artifact_manifest_id,
                runtime_artifact_manifest_revision: self.runtime_artifact_manifest_revision,
            },
        };
        let bytes = serde_json::to_vec(&preimage).map_err(|error| error.to_string())?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_prefixed_uuid_v7(
            "config_projection.authoring_input_ref.authority_store_id",
            &self.authority_store_id,
            "cpa_",
        )?;
        validate_lowercase_sha256_digest(
            "config_projection.authoring_input_ref.effective_config_source_hash",
            &self.effective_config_source_hash,
        )?;
        validate_lowercase_sha256_digest(
            "config_projection.authoring_input_ref.agent_inventory_source_hash",
            &self.agent_inventory_source_hash,
        )?;
        validate_prefixed_uuid_v7(
            "config_projection.authoring_input_ref.runtime_artifact_manifest_id",
            &self.runtime_artifact_manifest_id,
            "ram_",
        )?;
        if self.runtime_artifact_manifest_revision != 1 {
            return Err(
                "config_projection.authoring_input_ref.runtime_artifact_manifest_revision must equal 1"
                    .to_string(),
            );
        }
        validate_lowercase_sha256_digest(
            "config_projection.authoring_input_ref.runtime_artifact_manifest_hash",
            &self.runtime_artifact_manifest_hash,
        )?;
        validate_lowercase_sha256_digest(
            "config_projection.authoring_input_ref.input_ref_hash",
            &self.input_ref_hash,
        )?;
        if self.canonical_hash()? != self.input_ref_hash {
            return Err(
                "config_projection.authoring_input_ref.input_ref_hash does not match canonical input"
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl TryFrom<ConfigProjectionAuthoringInputRefV1Def> for ConfigProjectionAuthoringInputRefV1 {
    type Error = String;

    fn try_from(value: ConfigProjectionAuthoringInputRefV1Def) -> Result<Self, Self::Error> {
        let reference = Self {
            authority_store_id: value.authority_store_id,
            effective_config_source_hash: value.effective_config_source_hash,
            agent_inventory_source_hash: value.agent_inventory_source_hash,
            runtime_artifact_manifest_id: value.runtime_artifact_manifest_id,
            runtime_artifact_manifest_revision: value.runtime_artifact_manifest_revision,
            runtime_artifact_manifest_hash: value.runtime_artifact_manifest_hash,
            input_ref_hash: value.input_ref_hash,
        };
        reference.validate()?;
        Ok(reference)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "InWorldGatewayRefV1Def")]
pub struct InWorldGatewayRefV1 {
    pub authority_store_id: String,
    pub gateway_instance_id: String,
    pub gateway_identity_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct InWorldGatewayRefV1Def {
    authority_store_id: String,
    gateway_instance_id: String,
    gateway_identity_hash: String,
}

impl InWorldGatewayRefV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_prefixed_uuid_v7(
            "config_projection.expected_gateway_ref.authority_store_id",
            &self.authority_store_id,
            "cpa_",
        )?;
        validate_prefixed_uuid_v7(
            "config_projection.expected_gateway_ref.gateway_instance_id",
            &self.gateway_instance_id,
            "cgi_",
        )?;
        validate_lowercase_sha256_digest(
            "config_projection.expected_gateway_ref.gateway_identity_hash",
            &self.gateway_identity_hash,
        )
    }
}

impl TryFrom<InWorldGatewayRefV1Def> for InWorldGatewayRefV1 {
    type Error = String;

    fn try_from(value: InWorldGatewayRefV1Def) -> Result<Self, Self::Error> {
        let reference = Self {
            authority_store_id: value.authority_store_id,
            gateway_instance_id: value.gateway_instance_id,
            gateway_identity_hash: value.gateway_identity_hash,
        };
        reference.validate()?;
        Ok(reference)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ManagedGatewayActivationIntentRefV1Def")]
pub struct ManagedGatewayActivationIntentRefV1 {
    pub authority_store_id: String,
    pub activation_intent_id: String,
    pub intent_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayActivationIntentRefV1Def {
    authority_store_id: String,
    activation_intent_id: String,
    intent_hash: String,
}

impl ManagedGatewayActivationIntentRefV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_prefixed_uuid_v7(
            "config_projection.activation_intent_ref.authority_store_id",
            &self.authority_store_id,
            "cpa_",
        )?;
        validate_prefixed_uuid_v7(
            "config_projection.activation_intent_ref.activation_intent_id",
            &self.activation_intent_id,
            "gai_",
        )?;
        validate_lowercase_sha256_digest(
            "config_projection.activation_intent_ref.intent_hash",
            &self.intent_hash,
        )
    }
}

impl TryFrom<ManagedGatewayActivationIntentRefV1Def> for ManagedGatewayActivationIntentRefV1 {
    type Error = String;

    fn try_from(value: ManagedGatewayActivationIntentRefV1Def) -> Result<Self, Self::Error> {
        let reference = Self {
            authority_store_id: value.authority_store_id,
            activation_intent_id: value.activation_intent_id,
            intent_hash: value.intent_hash,
        };
        reference.validate()?;
        Ok(reference)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(try_from = "E3ConfigProjectionPrepareRequestV1Def")]
pub struct E3ConfigProjectionPrepareRequestV1 {
    pub schema_version: u32,
    pub preparation_id: String,
    pub preparation_idempotency_key: String,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    pub retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    pub e2_launch_activation: E2MemberLaunchActivationCarrierV1,
    pub authoring_input_ref: ConfigProjectionAuthoringInputRefV1,
    pub integrated_auth: GatewayIntegratedAuthPayloadV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct E3ConfigProjectionPrepareRequestV1Def {
    schema_version: u32,
    preparation_id: String,
    preparation_idempotency_key: String,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    #[serde(deserialize_with = "Option::deserialize")]
    parent_participant_id: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(deserialize_with = "Option::deserialize")]
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    e2_launch_activation: E2MemberLaunchActivationCarrierV1,
    authoring_input_ref: ConfigProjectionAuthoringInputRefV1,
    integrated_auth: GatewayIntegratedAuthPayloadV1,
}

impl TryFrom<E3ConfigProjectionPrepareRequestV1Def> for E3ConfigProjectionPrepareRequestV1 {
    type Error = String;
    fn try_from(value: E3ConfigProjectionPrepareRequestV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            preparation_id: value.preparation_id,
            preparation_idempotency_key: value.preparation_idempotency_key,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            parent_participant_id: value.parent_participant_id,
            resumed_from_participant_id: value.resumed_from_participant_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            resolved_runtime: value.resolved_runtime,
            retained_worker_launch_authority: value.retained_worker_launch_authority,
            e2_launch_activation: value.e2_launch_activation,
            authoring_input_ref: value.authoring_input_ref,
            integrated_auth: value.integrated_auth,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "E3ConfigProjectionPrepareResponseV1Def")]
pub struct E3ConfigProjectionPrepareResponseV1 {
    pub schema_version: u32,
    pub preparation_id: String,
    pub preparation_idempotency_key: String,
    pub config_projection: ConfigProjectionActivationCarrierV1,
    pub prepared_at: String,
    pub expires_at: String,
    pub response_hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct E3ConfigProjectionPrepareResponseV1Def {
    schema_version: u32,
    preparation_id: String,
    preparation_idempotency_key: String,
    config_projection: ConfigProjectionActivationCarrierV1,
    prepared_at: String,
    expires_at: String,
    response_hash: String,
}

impl TryFrom<E3ConfigProjectionPrepareResponseV1Def> for E3ConfigProjectionPrepareResponseV1 {
    type Error = String;
    fn try_from(value: E3ConfigProjectionPrepareResponseV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            preparation_id: value.preparation_id,
            preparation_idempotency_key: value.preparation_idempotency_key,
            config_projection: value.config_projection,
            prepared_at: value.prepared_at,
            expires_at: value.expires_at,
            response_hash: value.response_hash,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "E3ConfigProjectionCancelRequestV1Def")]
pub struct E3ConfigProjectionCancelRequestV1 {
    pub schema_version: u32,
    pub preparation_id: String,
    pub preparation_idempotency_key: String,
    pub config_projection: ConfigProjectionActivationCarrierV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct E3ConfigProjectionCancelRequestV1Def {
    schema_version: u32,
    preparation_id: String,
    preparation_idempotency_key: String,
    config_projection: ConfigProjectionActivationCarrierV1,
}

impl TryFrom<E3ConfigProjectionCancelRequestV1Def> for E3ConfigProjectionCancelRequestV1 {
    type Error = String;
    fn try_from(value: E3ConfigProjectionCancelRequestV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            preparation_id: value.preparation_id,
            preparation_idempotency_key: value.preparation_idempotency_key,
            config_projection: value.config_projection,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "E3ConfigProjectionCancelResponseV1Def")]
pub struct E3ConfigProjectionCancelResponseV1 {
    pub schema_version: u32,
    pub preparation_id: String,
    pub disposition: E3ConfigProjectionCancelDispositionV1,
    pub cancelled_dormant_projection_ref: ConfigProjectionRefV1,
    pub response_hash: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct E3ConfigProjectionCancelResponseV1Def {
    schema_version: u32,
    preparation_id: String,
    disposition: E3ConfigProjectionCancelDispositionV1,
    cancelled_dormant_projection_ref: ConfigProjectionRefV1,
    response_hash: String,
}

impl TryFrom<E3ConfigProjectionCancelResponseV1Def> for E3ConfigProjectionCancelResponseV1 {
    type Error = String;
    fn try_from(value: E3ConfigProjectionCancelResponseV1Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            preparation_id: value.preparation_id,
            disposition: value.disposition,
            cancelled_dormant_projection_ref: value.cancelled_dormant_projection_ref,
            response_hash: value.response_hash,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum E3ConfigProjectionCancelDispositionV1 {
    Cancelled,
    AlreadyTerminal,
}

impl E3ConfigProjectionPrepareRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported member_dispatch.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_non_empty_request_field(
            "member_dispatch.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field("member_dispatch.participant_id", &self.participant_id)?;
        validate_non_empty_request_field(
            "member_dispatch.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.parent_participant_id",
            self.parent_participant_id.as_deref(),
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.resumed_from_participant_id",
            self.resumed_from_participant_id.as_deref(),
        )?;
        validate_non_empty_request_field("member_dispatch.backend_id", &self.backend_id)?;
        validate_non_empty_request_field("member_dispatch.protocol", &self.protocol)?;
        validate_non_empty_request_field("member_dispatch.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_dispatch.world_id", &self.world_id)?;
        self.resolved_runtime.validate()?;
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            proof.validate()?;
        }
        let activation = &self.e2_launch_activation;
        activation.validate()?;
        self.authoring_input_ref.validate()?;
        validate_prefixed_uuid_v7("preparation_id", &self.preparation_id, "e3p_")?;
        if self.backend_id != "cli:codex-world"
            || self.resolved_runtime.backend_kind != MemberRuntimeBackendKindV1::Codex
            || self.integrated_auth.backend_id != self.backend_id
            || self.integrated_auth.cli_codex.is_none()
            || self.integrated_auth.api_env.is_some()
        {
            return Err("UnsupportedConfiguration".into());
        }
        self.integrated_auth
            .validate()
            .map_err(|_| "Malformed integrated auth".to_string())?;
        if self.preparation_idempotency_key != self.canonical_idempotency_key()? {
            return Err("WrongBinding preparation idempotency key".into());
        }

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_dispatch.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }
        if self.parent_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.parent_participant_id must not point to participant_id"
                    .to_string(),
            );
        }
        if self.resumed_from_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.resumed_from_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        let expected_lineage = match activation.launch_kind {
            E2MemberLaunchKindV1::FreshSpawn => {
                self.retained_worker_launch_authority.is_some()
                    && self.parent_participant_id.is_none()
                    && self.resumed_from_participant_id.is_none()
            }
            E2MemberLaunchKindV1::Fork => {
                self.retained_worker_launch_authority.is_none()
                    && self.parent_participant_id.as_deref()
                        == activation.source_participant_id.as_deref()
                    && self.resumed_from_participant_id.is_none()
            }
        };
        let expected_activation_id = format!(
            "e2a_{}",
            activation
                .commitment_ref
                .exact_linkage_hash
                .get(..32)
                .ok_or_else(|| {
                    "member_dispatch V2 E2 commitment linkage hash is malformed".to_string()
                })?
        );
        if !expected_lineage
            || activation.activation_id != expected_activation_id
            || activation.commitment_ref != activation.immutable_worker_cap_ref
            || activation.orchestration_session_id != self.orchestration_session_id
            || activation.caller_participant_id != self.orchestrator_participant_id
            || activation.target_backend_id != self.backend_id
            || activation.retained_participant_id != self.participant_id
            || activation.bootstrap_run_id != self.run_id
            || activation.target_world.world_id != self.world_id
            || activation.target_world.world_generation != self.world_generation
        {
            return Err(
                "member_dispatch V2 E2 activation conflicts with dispatch identity, lineage, or world binding"
                    .to_string(),
            );
        }
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            if proof.orchestration_session_id != self.orchestration_session_id
                || proof.caller_participant_id != self.orchestrator_participant_id
                || proof.retained_participant_id != self.participant_id
                || proof.bootstrap_run_id != self.run_id
                || proof.backend_id != self.backend_id
                || proof.protocol != self.protocol
                || proof.world_binding.world_id != self.world_id
                || proof.world_binding.world_generation != self.world_generation
            {
                return Err(
                    "member_dispatch V2 launch authority conflicts with dispatch identity or world binding"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
    fn canonical_idempotency_key(&self) -> Result<String, String> {
        let mut names = vec!["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"];
        if self
            .integrated_auth
            .cli_codex
            .as_ref()
            .is_some_and(|auth| auth.account_id.is_some())
        {
            names.push("SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID");
        }
        let preimage = serde_json::json!({
            "domain": "substrate.e3.config-projection-prepare-idempotency.v1",
            "credential_shape": {"backend_id": &self.integrated_auth.backend_id, "ordered_field_names": names},
            "request": {
                "schema_version": &self.schema_version,
                "preparation_id": &self.preparation_id,
                "orchestration_session_id": &self.orchestration_session_id,
                "participant_id": &self.participant_id,
                "orchestrator_participant_id": &self.orchestrator_participant_id,
                "parent_participant_id": &self.parent_participant_id,
                "resumed_from_participant_id": &self.resumed_from_participant_id,
                "backend_id": &self.backend_id,
                "protocol": &self.protocol,
                "run_id": &self.run_id,
                "world_id": &self.world_id,
                "world_generation": &self.world_generation,
                "resolved_runtime": &self.resolved_runtime,
                "retained_worker_launch_authority": &self.retained_worker_launch_authority,
                "e2_launch_activation": &self.e2_launch_activation,
                "authoring_input_ref": &self.authoring_input_ref,
            }
        });
        Ok(format!("e3pik_{}", e3_projection_wire_hash(preimage)?))
    }
}

impl E3ConfigProjectionPrepareResponseV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Malformed preparation schema".into());
        }
        validate_prefixed_uuid_v7("preparation_id", &self.preparation_id, "e3p_")?;
        let key = self
            .preparation_idempotency_key
            .strip_prefix("e3pik_")
            .ok_or("Malformed idempotency key")?;
        validate_lowercase_sha256_digest("preparation_idempotency_key", key)?;
        self.config_projection.validate()?;
        let (prepared, prepared_nanos) = e3_projection_wire_timestamp(&self.prepared_at)?;
        let (expires, expires_nanos) = e3_projection_wire_timestamp(&self.expires_at)?;
        if expires.checked_sub(prepared) != Some(120) || prepared_nanos != expires_nanos {
            return Err("WrongBinding preparation expiry".into());
        }
        let mut response = serde_json::to_value(self).map_err(|_| "Malformed response")?;
        response
            .as_object_mut()
            .ok_or("Malformed response")?
            .remove("response_hash");
        if self.response_hash
            != e3_projection_wire_hash(serde_json::json!({
                "domain": "substrate.e3.config-projection-prepare-response.v1", "response": response
            }))?
        {
            return Err("HashInvalid preparation response".into());
        }
        Ok(())
    }
}
impl E3ConfigProjectionCancelRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Malformed cancellation schema".into());
        }
        validate_prefixed_uuid_v7("preparation_id", &self.preparation_id, "e3p_")?;
        let key = self
            .preparation_idempotency_key
            .strip_prefix("e3pik_")
            .ok_or("Malformed idempotency key")?;
        validate_lowercase_sha256_digest("preparation_idempotency_key", key)?;
        self.config_projection.validate()
    }
}
impl E3ConfigProjectionCancelResponseV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Malformed cancellation schema".into());
        }
        validate_prefixed_uuid_v7("preparation_id", &self.preparation_id, "e3p_")?;
        self.cancelled_dormant_projection_ref.validate()?;
        let mut response = serde_json::to_value(self).map_err(|_| "Malformed response")?;
        response
            .as_object_mut()
            .ok_or("Malformed response")?
            .remove("response_hash");
        if self.response_hash
            != e3_projection_wire_hash(serde_json::json!({
                "domain": "substrate.e3.config-projection-cancel-response.v1", "response": response
            }))?
        {
            return Err("HashInvalid cancellation response".into());
        }
        Ok(())
    }
}

// Transport stays below config-projection. These wire preimages contain no secret values.
fn e3_projection_wire_hash(mut value: serde_json::Value) -> Result<String, String> {
    fn sort(value: &mut serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Object(map) => {
                let entries = std::mem::take(map)
                    .into_iter()
                    .collect::<std::collections::BTreeMap<_, _>>();
                for (key, mut value) in entries {
                    sort(&mut value)?;
                    map.insert(key, value);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    sort(item)?;
                }
            }
            serde_json::Value::Number(n) if !n.is_i64() && !n.is_u64() => {
                return Err("Malformed canonical number".into())
            }
            _ => {}
        }
        Ok(())
    }
    sort(&mut value)?;
    let bytes = serde_json::to_string(&value)
        .map_err(|_| "Malformed canonical object")?
        .replace('\u{7f}', "\\u007f");
    Ok(format!("{:x}", Sha256::digest(bytes.as_bytes())))
}

fn e3_projection_wire_timestamp(value: &str) -> Result<(i64, u32), String> {
    let invalid = || "Malformed canonical UTC timestamp".to_string();
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || !value.is_ascii()
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes.last() != Some(&b'Z')
    {
        return Err(invalid());
    }
    let number = |range: std::ops::Range<usize>| -> Result<i64, String> {
        let part = &value[range];
        if !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        part.parse().map_err(|_| invalid())
    };
    let year = number(0..4)?;
    let month = number(5..7)?;
    let day = number(8..10)?;
    let hour = number(11..13)?;
    let minute = number(14..16)?;
    let second = number(17..19)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month)
        || day < 1
        || day > days[(month - 1) as usize]
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(invalid());
    }
    let nanos = if bytes.len() == 20 {
        0
    } else {
        if bytes[19] != b'.' {
            return Err(invalid());
        }
        let count = bytes.len() - 21;
        if ![3, 6, 9].contains(&count) {
            return Err(invalid());
        }
        let fraction = number(20..bytes.len() - 1)? as u32;
        if fraction == 0 || (count > 3 && fraction % 1000 == 0) {
            return Err(invalid());
        }
        fraction * 10u32.pow((9 - count) as u32)
    };
    let previous = year - 1;
    let mut epoch_days = (year - 1970) * 365 + (previous.div_euclid(4) - 1969 / 4)
        - (previous.div_euclid(100) - 1969 / 100)
        + (previous.div_euclid(400) - 1969 / 400);
    epoch_days += days[..(month - 1) as usize].iter().sum::<i64>() + day - 1;
    Ok((
        epoch_days * 86400 + hour * 3600 + minute * 60 + second,
        nanos,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ConfigProjectionActivationCarrierV1Def")]
pub struct ConfigProjectionActivationCarrierV1 {
    pub authority_store_id: String,
    pub series_id: String,
    pub dormant_projection_ref: ConfigProjectionRefV1,
    pub activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    pub expected_gateway_ref: InWorldGatewayRefV1,
    pub fence_id: String,
    pub consumer_id: String,
    pub consumer_lease_revision: u64,
    pub consumer_lease_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigProjectionActivationCarrierV1Def {
    authority_store_id: String,
    series_id: String,
    dormant_projection_ref: ConfigProjectionRefV1,
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    expected_gateway_ref: InWorldGatewayRefV1,
    fence_id: String,
    consumer_id: String,
    consumer_lease_revision: u64,
    consumer_lease_hash: String,
}

impl ConfigProjectionActivationCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_prefixed_uuid_v7(
            "config_projection.authority_store_id",
            &self.authority_store_id,
            "cpa_",
        )?;
        validate_prefixed_uuid_v7("config_projection.series_id", &self.series_id, "cps_")?;
        self.dormant_projection_ref.validate()?;
        self.activation_intent_ref.validate()?;
        self.expected_gateway_ref.validate()?;
        validate_prefixed_uuid_v7("config_projection.fence_id", &self.fence_id, "cpf_")?;
        validate_prefixed_uuid_v7("config_projection.consumer_id", &self.consumer_id, "cpc_")?;
        if self.consumer_lease_revision != 1 {
            return Err("config_projection.consumer_lease_revision must equal 1".to_string());
        }
        validate_lowercase_sha256_digest(
            "config_projection.consumer_lease_hash",
            &self.consumer_lease_hash,
        )?;
        if self.dormant_projection_ref.authority_store_id != self.authority_store_id
            || self.activation_intent_ref.authority_store_id != self.authority_store_id
            || self.expected_gateway_ref.authority_store_id != self.authority_store_id
            || self.dormant_projection_ref.series_id != self.series_id
        {
            return Err(
                "config_projection store or series references must exactly match the carrier"
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl TryFrom<ConfigProjectionActivationCarrierV1Def> for ConfigProjectionActivationCarrierV1 {
    type Error = String;

    fn try_from(value: ConfigProjectionActivationCarrierV1Def) -> Result<Self, Self::Error> {
        let carrier = Self {
            authority_store_id: value.authority_store_id,
            series_id: value.series_id,
            dormant_projection_ref: value.dormant_projection_ref,
            activation_intent_ref: value.activation_intent_ref,
            expected_gateway_ref: value.expected_gateway_ref,
            fence_id: value.fence_id,
            consumer_id: value.consumer_id,
            consumer_lease_revision: value.consumer_lease_revision,
            consumer_lease_hash: value.consumer_lease_hash,
        };
        carrier.validate()?;
        Ok(carrier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "MemberDispatchRequestDef")]
pub struct MemberDispatchRequestV1 {
    #[serde(default = "member_dispatch_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    pub resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberDispatchRequestDef {
    #[serde(default = "member_dispatch_request_v1_default_schema_version")]
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    #[serde(default)]
    parent_participant_id: Option<String>,
    #[serde(default)]
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    #[serde(default)]
    initial_prompt: Option<String>,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default)]
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    #[serde(default)]
    e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct MemberDispatchCommonFieldsV1<'a> {
    pub orchestration_session_id: &'a str,
    pub participant_id: &'a str,
    pub orchestrator_participant_id: &'a str,
    pub parent_participant_id: Option<&'a str>,
    pub resumed_from_participant_id: Option<&'a str>,
    pub backend_id: &'a str,
    pub protocol: &'a str,
    pub run_id: &'a str,
    pub world_id: &'a str,
    pub world_generation: u64,
    pub initial_prompt: Option<&'a str>,
    pub resolved_runtime: &'a ResolvedMemberRuntimeDescriptorV1,
    pub retained_worker_launch_authority: Option<&'a RetainedWorkerLaunchAuthorityProofV1>,
}

fn member_dispatch_request_v1_default_schema_version() -> u32 {
    1
}

impl MemberDispatchRequestV1 {
    pub fn common(&self) -> MemberDispatchCommonFieldsV1<'_> {
        MemberDispatchCommonFieldsV1 {
            orchestration_session_id: &self.orchestration_session_id,
            participant_id: &self.participant_id,
            orchestrator_participant_id: &self.orchestrator_participant_id,
            parent_participant_id: self.parent_participant_id.as_deref(),
            resumed_from_participant_id: self.resumed_from_participant_id.as_deref(),
            backend_id: &self.backend_id,
            protocol: &self.protocol,
            run_id: &self.run_id,
            world_id: &self.world_id,
            world_generation: self.world_generation,
            initial_prompt: self.initial_prompt.as_deref(),
            resolved_runtime: &self.resolved_runtime,
            retained_worker_launch_authority: self.retained_worker_launch_authority.as_ref(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported member_dispatch.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }

        validate_non_empty_request_field(
            "member_dispatch.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field("member_dispatch.participant_id", &self.participant_id)?;
        validate_non_empty_request_field(
            "member_dispatch.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.parent_participant_id",
            self.parent_participant_id.as_deref(),
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.resumed_from_participant_id",
            self.resumed_from_participant_id.as_deref(),
        )?;
        validate_non_empty_request_field("member_dispatch.backend_id", &self.backend_id)?;
        validate_non_empty_request_field("member_dispatch.protocol", &self.protocol)?;
        validate_non_empty_request_field("member_dispatch.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_dispatch.world_id", &self.world_id)?;
        validate_optional_non_empty_request_field(
            "member_dispatch.initial_prompt",
            self.initial_prompt.as_deref(),
        )?;
        self.resolved_runtime.validate()?;
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            proof.validate()?;
        }
        if let Some(activation) = self.e2_launch_activation.as_ref() {
            activation.validate()?;
        }

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_dispatch.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }

        if self.parent_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.parent_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        if self.resumed_from_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.resumed_from_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        Ok(())
    }
}

impl TryFrom<MemberDispatchRequestDef> for MemberDispatchRequestV1 {
    type Error = String;

    fn try_from(value: MemberDispatchRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            parent_participant_id: value.parent_participant_id,
            resumed_from_participant_id: value.resumed_from_participant_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            initial_prompt: value.initial_prompt,
            resolved_runtime: value.resolved_runtime,
            retained_worker_launch_authority: value.retained_worker_launch_authority,
            e2_launch_activation: value.e2_launch_activation,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MemberDispatchRequestV2 {
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    pub resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
    pub config_projection: ConfigProjectionActivationCarrierV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberDispatchRequestV2Def {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    #[serde(default)]
    parent_participant_id: Option<String>,
    #[serde(default)]
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    #[serde(default)]
    initial_prompt: Option<String>,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default)]
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    #[serde(default)]
    e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
    config_projection: ConfigProjectionActivationCarrierV1,
}

impl MemberDispatchRequestV2 {
    pub fn common(&self) -> MemberDispatchCommonFieldsV1<'_> {
        MemberDispatchCommonFieldsV1 {
            orchestration_session_id: &self.orchestration_session_id,
            participant_id: &self.participant_id,
            orchestrator_participant_id: &self.orchestrator_participant_id,
            parent_participant_id: self.parent_participant_id.as_deref(),
            resumed_from_participant_id: self.resumed_from_participant_id.as_deref(),
            backend_id: &self.backend_id,
            protocol: &self.protocol,
            run_id: &self.run_id,
            world_id: &self.world_id,
            world_generation: self.world_generation,
            initial_prompt: self.initial_prompt.as_deref(),
            resolved_runtime: &self.resolved_runtime,
            retained_worker_launch_authority: self.retained_worker_launch_authority.as_ref(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2 {
            return Err(format!(
                "unsupported member_dispatch.schema_version: {} (expected 2)",
                self.schema_version
            ));
        }
        validate_non_empty_request_field(
            "member_dispatch.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field("member_dispatch.participant_id", &self.participant_id)?;
        validate_non_empty_request_field(
            "member_dispatch.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.parent_participant_id",
            self.parent_participant_id.as_deref(),
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.resumed_from_participant_id",
            self.resumed_from_participant_id.as_deref(),
        )?;
        validate_non_empty_request_field("member_dispatch.backend_id", &self.backend_id)?;
        validate_non_empty_request_field("member_dispatch.protocol", &self.protocol)?;
        validate_non_empty_request_field("member_dispatch.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_dispatch.world_id", &self.world_id)?;
        validate_optional_non_empty_request_field(
            "member_dispatch.initial_prompt",
            self.initial_prompt.as_deref(),
        )?;
        self.resolved_runtime.validate()?;
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            proof.validate()?;
        }
        let activation = self
            .e2_launch_activation
            .as_ref()
            .ok_or_else(|| "member_dispatch V2 requires e2_launch_activation".to_string())?;
        activation.validate()?;
        self.config_projection.validate()?;

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_dispatch.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }
        if self.parent_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.parent_participant_id must not point to participant_id"
                    .to_string(),
            );
        }
        if self.resumed_from_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.resumed_from_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        let expected_lineage = match activation.launch_kind {
            E2MemberLaunchKindV1::FreshSpawn => {
                self.retained_worker_launch_authority.is_some()
                    && self.parent_participant_id.is_none()
                    && self.resumed_from_participant_id.is_none()
            }
            E2MemberLaunchKindV1::Fork => {
                self.retained_worker_launch_authority.is_none()
                    && self.parent_participant_id.as_deref()
                        == activation.source_participant_id.as_deref()
                    && self.resumed_from_participant_id.is_none()
            }
        };
        let expected_activation_id = format!(
            "e2a_{}",
            activation
                .commitment_ref
                .exact_linkage_hash
                .get(..32)
                .ok_or_else(|| {
                    "member_dispatch V2 E2 commitment linkage hash is malformed".to_string()
                })?
        );
        if !expected_lineage
            || activation.activation_id != expected_activation_id
            || activation.commitment_ref != activation.immutable_worker_cap_ref
            || activation.orchestration_session_id != self.orchestration_session_id
            || activation.caller_participant_id != self.orchestrator_participant_id
            || activation.target_backend_id != self.backend_id
            || activation.retained_participant_id != self.participant_id
            || activation.bootstrap_run_id != self.run_id
            || activation.target_world.world_id != self.world_id
            || activation.target_world.world_generation != self.world_generation
        {
            return Err(
                "member_dispatch V2 E2 activation conflicts with dispatch identity, lineage, or world binding"
                    .to_string(),
            );
        }
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            if proof.orchestration_session_id != self.orchestration_session_id
                || proof.caller_participant_id != self.orchestrator_participant_id
                || proof.retained_participant_id != self.participant_id
                || proof.bootstrap_run_id != self.run_id
                || proof.backend_id != self.backend_id
                || proof.protocol != self.protocol
                || proof.world_binding.world_id != self.world_id
                || proof.world_binding.world_generation != self.world_generation
            {
                return Err(
                    "member_dispatch V2 launch authority conflicts with dispatch identity or world binding"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

impl TryFrom<MemberDispatchRequestV2Def> for MemberDispatchRequestV2 {
    type Error = String;

    fn try_from(value: MemberDispatchRequestV2Def) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            parent_participant_id: value.parent_participant_id,
            resumed_from_participant_id: value.resumed_from_participant_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            initial_prompt: value.initial_prompt,
            resolved_runtime: value.resolved_runtime,
            retained_worker_launch_authority: value.retained_worker_launch_authority,
            e2_launch_activation: value.e2_launch_activation,
            config_projection: value.config_projection,
        };
        request.validate()?;
        Ok(request)
    }
}

impl<'de> Deserialize<'de> for MemberDispatchRequestV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = MemberDispatchRequestV2Def::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum MemberDispatchRequest {
    V1(MemberDispatchRequestV1),
    V2(MemberDispatchRequestV2),
}

impl MemberDispatchRequest {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::V1(request) => request.validate(),
            Self::V2(request) => request.validate(),
        }
    }

    pub fn common(&self) -> MemberDispatchCommonFieldsV1<'_> {
        match self {
            Self::V1(request) => request.common(),
            Self::V2(request) => request.common(),
        }
    }

    pub fn as_v1(&self) -> Option<&MemberDispatchRequestV1> {
        match self {
            Self::V1(request) => Some(request),
            Self::V2(_) => None,
        }
    }

    pub fn as_v1_mut(&mut self) -> Option<&mut MemberDispatchRequestV1> {
        match self {
            Self::V1(request) => Some(request),
            Self::V2(_) => None,
        }
    }

    pub fn e2_launch_activation(&self) -> Option<&E2MemberLaunchActivationCarrierV1> {
        match self {
            Self::V1(request) => request.e2_launch_activation.as_ref(),
            Self::V2(request) => request.e2_launch_activation.as_ref(),
        }
    }

    pub fn config_projection(&self) -> Option<&ConfigProjectionActivationCarrierV1> {
        match self {
            Self::V1(_) => None,
            Self::V2(request) => Some(&request.config_projection),
        }
    }
}

struct StrictMemberDispatchValue(serde_json::Value);

impl<'de> Deserialize<'de> for StrictMemberDispatchValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictMemberDispatchValueVisitor;

        impl<'de> serde::de::Visitor<'de> for StrictMemberDispatchValueVisitor {
            type Value = StrictMemberDispatchValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON value without duplicate object fields")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::Bool(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::Number(
                    value.into(),
                )))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::Number(
                    value.into(),
                )))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::Number::from_f64(value)
                    .map(serde_json::Value::Number)
                    .map(StrictMemberDispatchValue)
                    .ok_or_else(|| E::custom("non-finite member_dispatch number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::String(
                    value.to_string(),
                )))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::String(value)))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::Null))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                StrictMemberDispatchValue::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(StrictMemberDispatchValue(serde_json::Value::Null))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
                while let Some(value) = sequence.next_element::<StrictMemberDispatchValue>()? {
                    values.push(value.0);
                }
                Ok(StrictMemberDispatchValue(serde_json::Value::Array(values)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut values = serde_json::Map::new();
                while let Some((key, value)) =
                    map.next_entry::<String, StrictMemberDispatchValue>()?
                {
                    if values.insert(key.clone(), value.0).is_some() {
                        return Err(serde::de::Error::custom(format!("duplicate field `{key}`")));
                    }
                }
                Ok(StrictMemberDispatchValue(serde_json::Value::Object(values)))
            }
        }

        deserializer.deserialize_any(StrictMemberDispatchValueVisitor)
    }
}

impl<'de> Deserialize<'de> for MemberDispatchRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MemberDispatchRequestVisitor;

        impl<'de> serde::de::Visitor<'de> for MemberDispatchRequestVisitor {
            type Value = MemberDispatchRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a strict member-dispatch request object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut fields = serde_json::Map::new();
                while let Some((key, value)) =
                    map.next_entry::<String, StrictMemberDispatchValue>()?
                {
                    if fields.insert(key.clone(), value.0).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate member_dispatch field `{key}`"
                        )));
                    }
                }
                let schema_version = match fields.get("schema_version") {
                    None => 1,
                    Some(serde_json::Value::Number(number)) => {
                        number.as_u64().ok_or_else(|| {
                            serde::de::Error::custom(
                                "member_dispatch.schema_version must be an unsigned integer",
                            )
                        })?
                    }
                    Some(_) => {
                        return Err(serde::de::Error::custom(
                            "member_dispatch.schema_version must be an unsigned integer",
                        ))
                    }
                };
                let object = serde_json::Value::Object(fields);
                match schema_version {
                    1 => serde_json::from_value::<MemberDispatchRequestDef>(object)
                        .map_err(serde::de::Error::custom)
                        .and_then(|request| {
                            MemberDispatchRequestV1::try_from(request)
                                .map(MemberDispatchRequest::V1)
                                .map_err(serde::de::Error::custom)
                        }),
                    2 => serde_json::from_value::<MemberDispatchRequestV2Def>(object)
                        .map_err(serde::de::Error::custom)
                        .and_then(|request| {
                            MemberDispatchRequestV2::try_from(request)
                                .map(MemberDispatchRequest::V2)
                                .map_err(serde::de::Error::custom)
                        }),
                    version => Err(serde::de::Error::custom(format!(
                        "unsupported member_dispatch.schema_version: {version}"
                    ))),
                }
            }
        }

        deserializer.deserialize_map(MemberDispatchRequestVisitor)
    }
}

/// Request-scoped proposal identity retained unchanged until runtime acknowledgement.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorldWorkAcceptanceContextV1 {
    pub schema_version: u32,
    pub proposed_acceptance_record_id: String,
    pub request_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    pub caller_backend_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldWorkAcceptanceContextDef {
    schema_version: u32,
    proposed_acceptance_record_id: String,
    request_id: String,
    #[serde(default)]
    message_id: Option<String>,
    caller_backend_id: String,
    #[serde(default)]
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

impl WorldWorkAcceptanceContextV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported acceptance_context.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_prefixed_uuid_v7(
            "acceptance_context.proposed_acceptance_record_id",
            &self.proposed_acceptance_record_id,
            "wwa_",
        )?;
        validate_non_empty_request_field("acceptance_context.request_id", &self.request_id)?;
        validate_non_empty_request_field(
            "acceptance_context.caller_backend_id",
            &self.caller_backend_id,
        )?;
        if let Some(message_id) = self.message_id.as_deref() {
            validate_prefixed_uuid_v7("acceptance_context.message_id", message_id, "wwm_")?;
        }
        if let Some(correlation) = self.host_transition_correlation.as_ref() {
            correlation.validate()?;
        }
        Ok(())
    }
}

impl TryFrom<WorldWorkAcceptanceContextDef> for WorldWorkAcceptanceContextV1 {
    type Error = String;

    fn try_from(value: WorldWorkAcceptanceContextDef) -> Result<Self, Self::Error> {
        let context = Self {
            schema_version: value.schema_version,
            proposed_acceptance_record_id: value.proposed_acceptance_record_id,
            request_id: value.request_id,
            message_id: value.message_id,
            caller_backend_id: value.caller_backend_id,
            host_transition_correlation: value.host_transition_correlation,
        };
        context.validate()?;
        Ok(context)
    }
}

impl<'de> Deserialize<'de> for WorldWorkAcceptanceContextV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = WorldWorkAcceptanceContextDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

fn validate_prefixed_uuid_v7(field: &str, value: &str, prefix: &str) -> Result<(), String> {
    let Some(uuid) = value.strip_prefix(prefix) else {
        return Err(format!("{field} must start with {prefix}"));
    };
    let bytes = uuid.as_bytes();
    let hyphens = [8usize, 13, 18, 23];
    if bytes.len() != 36
        || hyphens.iter().any(|index| bytes[*index] != b'-')
        || bytes.iter().enumerate().any(|(index, byte)| {
            !(hyphens.contains(&index) || byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        })
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
    {
        return Err(format!("{field} must contain a lowercase UUIDv7"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "MemberTurnSubmitRequestDef")]
pub struct MemberTurnSubmitRequestV1 {
    #[serde(default = "member_turn_submit_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub backend_id: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_snapshot_carrier: Option<DispatchPolicySnapshotCarrierV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnSubmitRequestDef {
    #[serde(default = "member_turn_submit_request_v1_default_schema_version")]
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    backend_id: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    prompt: String,
    #[serde(default)]
    policy_snapshot_carrier: Option<DispatchPolicySnapshotCarrierV1>,
    #[serde(default)]
    acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

/// Strict representation for the dedicated ordinary retained-turn endpoint.
///
/// Serialization is identical to `MemberTurnSubmitRequestV1`. This DTO grants
/// no authority: the service must join its actual held ordinary member and
/// independently authenticate configured current B1/E2 authority before effects.
///
/// ```
/// use transport_api_types::OrdinaryMemberTurnSubmitRequestV1;
/// assert!(serde_json::from_str::<OrdinaryMemberTurnSubmitRequestV1>("{}").is_err());
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct OrdinaryMemberTurnSubmitRequestV1(MemberTurnSubmitRequestV1);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryMemberTurnSubmitRequestDef {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    backend_id: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    prompt: String,
    policy_snapshot_carrier: OrdinaryDispatchPolicySnapshotCarrierV1,
    acceptance_context: WorldWorkAcceptanceContextV1,
}

impl OrdinaryMemberTurnSubmitRequestV1 {
    /// Validate representation without claiming configured admission.
    ///
    /// ```no_run
    /// # fn wrap(request: transport_api_types::MemberTurnSubmitRequestV1) -> Result<(), String> {
    /// let ordinary = transport_api_types::OrdinaryMemberTurnSubmitRequestV1::from_request(request)?;
    /// let _borrowed = ordinary.as_request();
    /// let _original = ordinary.into_request();
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_request(request: MemberTurnSubmitRequestV1) -> Result<Self, String> {
        request.validate_ordinary_representation()?;
        Ok(Self(request))
    }

    pub fn as_request(&self) -> &MemberTurnSubmitRequestV1 {
        &self.0
    }

    pub fn into_request(self) -> MemberTurnSubmitRequestV1 {
        self.0
    }
}

impl<'de> Deserialize<'de> for OrdinaryMemberTurnSubmitRequestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = OrdinaryMemberTurnSubmitRequestDef::deserialize(deserializer)?;
        Self::from_request(MemberTurnSubmitRequestV1 {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            backend_id: value.backend_id,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            prompt: value.prompt,
            policy_snapshot_carrier: Some(value.policy_snapshot_carrier.0),
            acceptance_context: Some(value.acceptance_context),
        })
        .map_err(serde::de::Error::custom)
    }
}

fn member_turn_submit_request_v1_default_schema_version() -> u32 {
    1
}

impl MemberTurnSubmitRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_representation(false)
    }

    /// Ordinary wire validation grants no permission. A genuine configured
    /// admission must authenticate the exact received request before effects.
    pub fn validate_ordinary_representation(&self) -> Result<(), String> {
        self.validate_representation(true)?;
        if self.policy_snapshot_carrier.is_none() || self.acceptance_context.is_none() {
            return Err(
                "ordinary retained turn requires its exact E2 carrier and B1 context".into(),
            );
        }
        Ok(())
    }

    fn validate_representation(&self, ordinary: bool) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported member_turn_submit.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }

        validate_non_empty_request_field(
            "member_turn_submit.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field(
            "member_turn_submit.participant_id",
            &self.participant_id,
        )?;
        validate_non_empty_request_field(
            "member_turn_submit.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_non_empty_request_field("member_turn_submit.backend_id", &self.backend_id)?;
        validate_gateway_backend_id_selector(self.backend_id.trim()).map_err(|_| {
            format!(
                "invalid member_turn_submit.backend_id '{}'; expected <kind>:<name>",
                self.backend_id.trim()
            )
        })?;
        validate_non_empty_request_field("member_turn_submit.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_turn_submit.world_id", &self.world_id)?;
        validate_non_empty_request_field("member_turn_submit.prompt", &self.prompt)?;

        if let Some(carrier) = self.policy_snapshot_carrier.as_ref() {
            if ordinary {
                carrier.validate_ordinary_representation()?;
            } else {
                carrier.validate()?;
            }
            if carrier.orchestration_session_id != self.orchestration_session_id
                || carrier.subject.retained_participant_id != self.participant_id
                || carrier.subject.active_run_id != self.run_id
                || carrier.caller_participant_id != self.orchestrator_participant_id
                || carrier.target_backend_id != self.backend_id
                || carrier.target_world.world_id != self.world_id
                || carrier.target_world.world_generation != self.world_generation
            {
                return Err(
                    "member_turn_submit policy_snapshot_carrier identity or world binding changed"
                        .to_string(),
                );
            }
            let acceptance_message_id = self
                .acceptance_context
                .as_ref()
                .and_then(|context| context.message_id.as_ref());
            if carrier.subject.message_id.as_ref() != acceptance_message_id {
                return Err(
                    "member_turn_submit policy_snapshot_carrier subject.message_id must exactly match acceptance_context.message_id"
                        .to_string(),
                );
            }
            if let Some(context) = self.acceptance_context.as_ref() {
                if carrier.caller_backend_id != context.caller_backend_id {
                    return Err(
                        "member_turn_submit policy_snapshot_carrier.caller_backend_id must exactly match acceptance_context.caller_backend_id"
                            .to_string(),
                    );
                }
            }
        }

        if let Some(context) = self.acceptance_context.as_ref() {
            context.validate()?;
            if context.request_id != self.run_id {
                return Err(
                    "member_turn_submit.acceptance_context.request_id must equal run_id"
                        .to_string(),
                );
            }
            if context.message_id.is_none() {
                return Err(
                    "member_turn_submit.acceptance_context.message_id is required".to_string(),
                );
            }
        }

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_turn_submit.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// Validates both request bindings and byte-for-byte equality with a
    /// carrier obtained from independently authenticated host E2 authority.
    pub fn validate_against_authenticated_policy_carrier(
        &self,
        authenticated: &DispatchPolicySnapshotCarrierV1,
    ) -> Result<(), String> {
        self.validate()?;
        let carried = self.policy_snapshot_carrier.as_ref().ok_or_else(|| {
            "member_turn_submit requires policy_snapshot_carrier for authenticated E2 dispatch"
                .to_string()
        })?;
        carried.validate_exact_authenticated_copy(authenticated)
    }
}

impl TryFrom<MemberTurnSubmitRequestDef> for MemberTurnSubmitRequestV1 {
    type Error = String;

    fn try_from(value: MemberTurnSubmitRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            backend_id: value.backend_id,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            prompt: value.prompt,
            policy_snapshot_carrier: value.policy_snapshot_carrier,
            acceptance_context: value.acceptance_context,
        };
        request.validate()?;
        Ok(request)
    }
}

/// Reference to the immutable, nonsecret record bound by genuine original Start.
///
/// This is untrusted transport data. Neither structural validation nor matching
/// digests authenticate an issuer, configured home, current caller or permission.
/// A service must authenticate it using its independently configured authority.
/// Mutable current revisions are deliberately absent so retries reproduce bytes.
///
/// ```
/// use transport_api_types::OrdinaryStartSelectionCarrierV1;
/// assert!(serde_json::from_str::<OrdinaryStartSelectionCarrierV1>("{}").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryStartSelectionCarrierDef")]
pub struct OrdinaryStartSelectionCarrierV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub original_start_intent_id: String,
    pub original_start_issuer_request_id: String,
    pub original_session_id: String,
    pub context_sha256: String,
    pub native_selection_sha256: String,
    pub requested_selection_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryStartSelectionCarrierDef {
    schema_version: u32,
    authority_store_id: String,
    original_start_intent_id: String,
    original_start_issuer_request_id: String,
    original_session_id: String,
    context_sha256: String,
    native_selection_sha256: String,
    requested_selection_sha256: String,
}

impl OrdinaryStartSelectionCarrierV1 {
    /// Validate only the closed transport representation, never authority.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported ordinary Start selection carrier version".into());
        }
        selection_identifier(&self.authority_store_id, "authority store id")?;
        selection_identifier(&self.original_start_intent_id, "original Start intent id")?;
        selection_identifier(&self.original_session_id, "original session id")?;
        let issuer = self
            .original_start_issuer_request_id
            .strip_prefix("req_")
            .ok_or("invalid original Start issuer request id")?;
        selection_sha256(issuer, "original Start issuer request digest")?;
        selection_sha256(&self.context_sha256, "original Start context digest")?;
        selection_sha256(&self.native_selection_sha256, "native selection digest")?;
        selection_sha256(
            &self.requested_selection_sha256,
            "requested selection digest",
        )
    }
}

impl TryFrom<OrdinaryStartSelectionCarrierDef> for OrdinaryStartSelectionCarrierV1 {
    type Error = String;

    fn try_from(value: OrdinaryStartSelectionCarrierDef) -> Result<Self, Self::Error> {
        let carrier = Self {
            schema_version: value.schema_version,
            authority_store_id: value.authority_store_id,
            original_start_intent_id: value.original_start_intent_id,
            original_start_issuer_request_id: value.original_start_issuer_request_id,
            original_session_id: value.original_session_id,
            context_sha256: value.context_sha256,
            native_selection_sha256: value.native_selection_sha256,
            requested_selection_sha256: value.requested_selection_sha256,
        };
        carrier.validate()?;
        Ok(carrier)
    }
}

/// Actual selected native actor whose gateway route is being requested.
///
/// Actor placement is distinct from where the managed gateway executes. A Host
/// actor has no World member binding; a World worker must name its exact binding.
/// The service authenticates these identifiers against original/current authority.
///
/// ```
/// use transport_api_types::OrdinaryGatewayRuntimeIdentityV1;
/// assert!(serde_json::from_str::<OrdinaryGatewayRuntimeIdentityV1>("{}").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "selection_role", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryGatewayRuntimeIdentityV1 {
    HostActor {
        orchestration_session_id: String,
        participant_id: String,
        backend_id: String,
        run_id: String,
    },
    WorldWorker {
        orchestration_session_id: String,
        caller_participant_id: String,
        participant_id: String,
        backend_id: String,
        run_id: String,
        world_id: String,
        world_generation: u64,
    },
}

impl OrdinaryGatewayRuntimeIdentityV1 {
    /// Exact backend named by this role, checked against the authenticated selection.
    pub fn backend_id(&self) -> &str {
        match self {
            Self::HostActor { backend_id, .. } | Self::WorldWorker { backend_id, .. } => backend_id,
        }
    }

    /// Validate representation only; the service must authenticate the role.
    pub fn validate(&self) -> Result<(), String> {
        let (session, participant, run) = match self {
            Self::HostActor {
                orchestration_session_id,
                participant_id,
                run_id,
                ..
            }
            | Self::WorldWorker {
                orchestration_session_id,
                participant_id,
                run_id,
                ..
            } => (orchestration_session_id, participant_id, run_id),
        };
        for (name, value) in [
            ("orchestration session id", session),
            ("participant id", participant),
            ("run id", run),
        ] {
            selection_identifier(value, name)?;
        }
        let backend = self.backend_id();
        validate_gateway_backend_id_selector(backend)?;
        if backend.trim() != backend {
            return Err("ordinary gateway backend id must be exact".into());
        }
        if let Self::WorldWorker {
            caller_participant_id,
            world_id,
            ..
        } = self
        {
            selection_identifier(caller_participant_id, "caller participant id")?;
            selection_identifier(world_id, "world id")?;
            // Zero is an authentic initial backend generation. This ordinary
            // DTO validates representation; service admission authenticates it.
        }
        Ok(())
    }
}

/// Selected gateway join for the explicit Host-actor or World-worker role.
/// The service authenticates Start, the actual runtime and current permission;
/// this representation alone grants no gateway or credential access.
///
/// ```
/// use transport_api_types::OrdinaryGatewaySelectionContextV1;
/// assert!(serde_json::from_str::<OrdinaryGatewaySelectionContextV1>("{}").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "OrdinaryGatewaySelectionContextDef")]
pub struct OrdinaryGatewaySelectionContextV1 {
    pub schema_version: u32,
    pub start: OrdinaryStartSelectionCarrierV1,
    pub runtime: OrdinaryGatewayRuntimeIdentityV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryGatewaySelectionContextDef {
    schema_version: u32,
    start: OrdinaryStartSelectionCarrierV1,
    runtime: OrdinaryGatewayRuntimeIdentityV1,
}

impl OrdinaryGatewaySelectionContextV1 {
    /// Validate representation and explicit runtime role, not permission.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported ordinary gateway selection context version".into());
        }
        self.start.validate()?;
        self.runtime.validate()
    }
}

impl TryFrom<OrdinaryGatewaySelectionContextDef> for OrdinaryGatewaySelectionContextV1 {
    type Error = String;

    fn try_from(value: OrdinaryGatewaySelectionContextDef) -> Result<Self, Self::Error> {
        let context = Self {
            schema_version: value.schema_version,
            start: value.start,
            runtime: value.runtime,
        };
        context.validate()?;
        Ok(context)
    }
}

fn legacy_request_schema_version() -> u32 {
    1
}

fn is_legacy_request_schema_version(version: &u32) -> bool {
    *version == 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "ExecuteRequestDef")]
pub struct ExecuteRequest {
    /// Version 1 is the unchanged unselected format; version 2 requires the
    /// ordinary Start carrier and MemberDispatch V1. This version grants no authority.
    #[serde(
        default = "legacy_request_schema_version",
        skip_serializing_if = "is_legacy_request_schema_version"
    )]
    pub schema_version: u32,
    pub profile: Option<String>,
    pub cmd: String,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub pty: bool,
    pub agent_id: String,
    pub budget: Option<Budget>,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_world: Option<SharedWorldOwnerSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_fs_mode: Option<WorldFsMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_dispatch: Option<MemberDispatchRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_context: Option<WorldWorkAcceptanceContextV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary_start_selection: Option<OrdinaryStartSelectionCarrierV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteRequestDef {
    #[serde(default = "legacy_request_schema_version")]
    schema_version: u32,
    profile: Option<String>,
    cmd: String,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    pty: bool,
    agent_id: String,
    budget: Option<Budget>,
    policy_snapshot: PolicySnapshotV3,
    #[serde(default)]
    shared_world: Option<SharedWorldOwnerSpec>,
    #[serde(default)]
    world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default)]
    world_fs_mode: Option<WorldFsMode>,
    #[serde(default)]
    member_dispatch: Option<MemberDispatchRequest>,
    #[serde(default)]
    acceptance_context: Option<WorldWorkAcceptanceContextV1>,
    #[serde(default)]
    ordinary_start_selection: Option<OrdinaryStartSelectionCarrierV1>,
}

impl ExecuteRequest {
    pub fn validate(&self) -> Result<(), String> {
        match (self.schema_version, self.ordinary_start_selection.as_ref()) {
            (1, None) => {}
            (2, Some(selection)) => {
                selection.validate()?;
                if !matches!(
                    self.member_dispatch.as_ref(),
                    Some(MemberDispatchRequest::V1(_))
                ) {
                    return Err(
                        "selected execute request requires ordinary MemberDispatch V1".into(),
                    );
                }
            }
            _ => {
                return Err(
                    "unsupported execute request version or ordinary selection context".into(),
                );
            }
        }
        let cmd_is_empty = self.cmd.trim().is_empty();

        match self.member_dispatch.as_ref() {
            Some(member_dispatch) => {
                member_dispatch.validate()?;
                let common = member_dispatch.common();
                if !cmd_is_empty {
                    return Err(
                        "execute request member_dispatch requires cmd.trim().is_empty()"
                            .to_string(),
                    );
                }
                if self.pty {
                    return Err("execute request member_dispatch requires pty=false".to_string());
                }
                if let Some(context) = self.acceptance_context.as_ref() {
                    context.validate()?;
                    if context.request_id != common.run_id {
                        return Err(
                            "execute request acceptance_context.request_id must equal member_dispatch.run_id"
                                .to_string(),
                        );
                    }
                    if context.message_id.is_some() {
                        return Err(
                            "execute request task acceptance_context.message_id must be absent"
                                .to_string(),
                        );
                    }
                }
            }
            None => {
                if cmd_is_empty {
                    return Err(
                        "execute request process exec requires a non-empty cmd when member_dispatch is absent"
                        .to_string(),
                    );
                }
                if self.acceptance_context.is_some() {
                    return Err(
                        "execute request acceptance_context requires member_dispatch".to_string(),
                    );
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<ExecuteRequestDef> for ExecuteRequest {
    type Error = String;

    fn try_from(value: ExecuteRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            ordinary_start_selection: value.ordinary_start_selection,
            profile: value.profile,
            cmd: value.cmd,
            cwd: value.cwd,
            env: value.env,
            pty: value.pty,
            agent_id: value.agent_id,
            budget: value.budget,
            policy_snapshot: value.policy_snapshot,
            shared_world: value.shared_world,
            world_network: value.world_network,
            world_fs_mode: value.world_fs_mode,
            member_dispatch: value.member_dispatch,
            acceptance_context: value.acceptance_context,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit: i32,
    pub span_id: String,
    pub stdout_b64: String,
    pub stderr_b64: String,
    pub scopes_used: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_diff: Option<FsDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_world: Option<SharedWorldBindingSnapshot>,
    #[serde(flatten, default)]
    pub process_telemetry: ProcessTelemetry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelRequestV1 {
    pub span_id: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelResponseV1 {
    #[serde(default = "execute_cancel_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub delivered: bool,
}

fn execute_cancel_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecuteStreamReplayRequestV1 {
    #[serde(default = "execute_stream_replay_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub acceptance_record_id: String,
    pub stream_id: String,
    pub after_frame_sequence: u64,
}

fn execute_stream_replay_request_v1_default_schema_version() -> u32 {
    1
}

impl ExecuteStreamReplayRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported execute stream replay schema version {}",
                self.schema_version
            ));
        }
        for (field, value, prefix) in [
            (
                "acceptance_record_id",
                self.acceptance_record_id.as_str(),
                "wwa_",
            ),
            ("stream_id", self.stream_id.as_str(), "rts_"),
        ] {
            if value.trim() != value || !value.starts_with(prefix) || value.len() == prefix.len() {
                return Err(format!(
                    "execute stream replay {field} must be exact, trimmed, and {prefix}-prefixed"
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingDiffBucketV1 {
    pub writes: Vec<String>,
    pub mods: Vec<String>,
    pub deletes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRecordV1 {
    #[serde(default = "pending_diff_record_v1_default_schema_version")]
    pub schema_version: u32,
    pub session_started_at: String,
    pub diff_id: String,
    pub non_pty: PendingDiffBucketV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pty: Option<PendingDiffBucketV1>,
}

fn pending_diff_record_v1_default_schema_version() -> u32 {
    1
}

fn validate_non_empty_request_field(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_optional_non_empty_request_field(
    field: &str,
    value: Option<&str>,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty_request_field(field, value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
    pub discard_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileResponseV1 {
    #[serde(default = "pending_diff_reconcile_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub reconciled: bool,
    pub discarded: u32,
}

fn pending_diff_reconcile_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearResponseV1 {
    #[serde(default = "pending_diff_clear_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub cleared: bool,
}

fn pending_diff_clear_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEntryTypeV1 {
    RegularFile,
    Directory,
    Symlink,
    Socket,
    Fifo,
    BlockDevice,
    CharDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub path: String,
    #[serde(default)]
    pub include_contents: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadResponseV1 {
    #[serde(default = "world_fs_read_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub path: String,
    pub entry_type: WorldFsEntryTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents_b64: Option<String>,
}

fn world_fs_read_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayCliCodexIntegratedAuthV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayApiEnvIntegratedAuthV1 {
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayIntegratedAuthPayloadV1 {
    pub backend_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_codex: Option<GatewayCliCodexIntegratedAuthV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_env: Option<GatewayApiEnvIntegratedAuthV1>,
}

impl GatewayIntegratedAuthPayloadV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_gateway_integrated_auth_payload(self)
    }

    pub fn validate_for_selected_backend(&self, selected_backend: &str) -> Result<(), String> {
        validate_gateway_integrated_auth_payload_for_selected_backend(self, selected_backend)
    }
}

pub fn validate_gateway_backend_id_selector(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some((kind, name)) = trimmed.split_once(':') else {
        return Err(format!(
            "invalid gateway backend_id '{}'; expected <kind>:<name>",
            trimmed
        ));
    };

    if !matches_backend_kind(kind) || !matches_backend_name(name) || name.contains(':') {
        return Err(format!(
            "invalid gateway backend_id '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
            trimmed
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "GatewayLifecycleRequestDef")]
pub struct GatewayLifecycleRequestV1 {
    /// Version 2 selects the explicit ordinary gateway context; version 1 keeps
    /// the existing unselected lifecycle representation.
    #[serde(
        default = "legacy_request_schema_version",
        skip_serializing_if = "is_legacy_request_schema_version"
    )]
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary_selection: Option<OrdinaryGatewaySelectionContextV1>,
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_tuple: Option<IdentityTuple>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_posture: Option<PlacementPosture>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayLifecycleRequestDef {
    #[serde(default = "legacy_request_schema_version")]
    schema_version: u32,
    #[serde(default)]
    ordinary_selection: Option<OrdinaryGatewaySelectionContextV1>,
    profile: Option<String>,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    agent_id: String,
    policy_snapshot: PolicySnapshotV3,
    #[serde(default)]
    world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default)]
    integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
    #[serde(default)]
    identity_tuple: Option<IdentityTuple>,
    #[serde(default)]
    placement_posture: Option<PlacementPosture>,
}

impl GatewayLifecycleRequestV1 {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        match (self.schema_version, self.ordinary_selection.as_ref()) {
            (1, None) => {}
            (2, Some(context)) => {
                context.validate()?;
                if self
                    .integrated_auth
                    .as_ref()
                    .is_some_and(|auth| auth.backend_id != context.runtime.backend_id())
                {
                    return Err("selected gateway auth backend differs from runtime context".into());
                }
            }
            _ => {
                return Err(
                    "unsupported gateway request version or ordinary selection context".into(),
                );
            }
        }
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }
}

impl TryFrom<GatewayLifecycleRequestDef> for GatewayLifecycleRequestV1 {
    type Error = String;

    fn try_from(value: GatewayLifecycleRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            ordinary_selection: value.ordinary_selection,
            profile: value.profile,
            cwd: value.cwd,
            env: value.env,
            agent_id: value.agent_id,
            policy_snapshot: value.policy_snapshot,
            world_network: value.world_network,
            integrated_auth: value.integrated_auth,
            identity_tuple: value.identity_tuple,
            placement_posture: value.placement_posture,
        };
        request.validate_identity_contract()?;
        Ok(request)
    }
}

pub fn validate_gateway_integrated_auth_payload(
    payload: &GatewayIntegratedAuthPayloadV1,
) -> Result<(), String> {
    let backend_id = payload.backend_id.trim();
    validate_gateway_backend_id_selector(backend_id).map_err(|err| {
        if backend_id.is_empty() {
            "request-provided integrated auth payload is missing backend_id".to_string()
        } else {
            err
        }
    })?;

    let cli_codex = payload.cli_codex.as_ref();
    let api_env = payload.api_env.as_ref();
    let facet_count = usize::from(cli_codex.is_some()) + usize::from(api_env.is_some());

    if facet_count != 1 {
        return Err(format!(
            "request-provided integrated auth payload must contain exactly one auth facet (found {facet_count})"
        ));
    }

    if let Some(cli_codex) = cli_codex {
        if !is_cli_codex_gateway_backend(backend_id) {
            return Err(format!(
                "request-provided integrated auth payload for '{}' uses incompatible auth facet 'cli_codex'",
                backend_id
            ));
        }

        if cli_codex
            .account_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(
                "request-provided integrated auth payload contains empty cli_codex.account_id"
                    .to_string(),
            );
        }

        if cli_codex.access_token.trim().is_empty() {
            return Err(
                "request-provided integrated auth payload contains empty cli_codex.access_token"
                    .to_string(),
            );
        }
    }

    if let Some(api_env) = api_env {
        if !backend_id.starts_with("api:") && !is_cli_claude_code_gateway_backend(backend_id) {
            return Err(format!(
                "request-provided integrated auth payload for '{}' uses incompatible auth facet 'api_env'",
                backend_id
            ));
        }

        if api_env.env.is_empty() {
            return Err(
                "request-provided integrated auth payload contains empty api_env.env".to_string(),
            );
        }

        for (name, value) in &api_env.env {
            let trimmed_name = name.trim();
            if trimmed_name.is_empty() {
                return Err(
                    "request-provided integrated auth payload contains blank api_env env name"
                        .to_string(),
                );
            }
            if trimmed_name != name
                || trimmed_name.contains(char::is_whitespace)
                || trimmed_name.contains('=')
            {
                return Err(format!(
                    "request-provided integrated auth payload contains invalid api_env env name '{}'",
                    name
                ));
            }
            if value.trim().is_empty() {
                return Err(format!(
                    "request-provided integrated auth payload contains empty api_env value for '{}'",
                    name
                ));
            }
        }
    }

    Ok(())
}

fn is_cli_codex_gateway_backend(backend_id: &str) -> bool {
    matches!(
        backend_id,
        "cli:codex" | "cli:codex-host" | "cli:codex-world"
    )
}

fn is_cli_claude_code_gateway_backend(backend_id: &str) -> bool {
    matches!(
        backend_id,
        "cli:claude_code" | "cli:claude_code-host" | "cli:claude_code-world"
    )
}

fn matches_backend_kind(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn matches_backend_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

pub fn validate_gateway_integrated_auth_payload_for_selected_backend(
    payload: &GatewayIntegratedAuthPayloadV1,
    selected_backend: &str,
) -> Result<(), String> {
    validate_gateway_integrated_auth_payload(payload)?;

    let selected_backend = selected_backend.trim();
    if payload.backend_id.trim() != selected_backend {
        return Err(format!(
            "request-provided integrated auth payload for '{}' does not match selected backend '{}'",
            payload.backend_id.trim(),
            selected_backend
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayStatusV1 {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayClientWiringV1 {
    pub openai_base_url: String,
    pub anthropic_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "GatewayLifecycleResponseDef")]
pub struct GatewayLifecycleResponseV1 {
    pub status: GatewayStatusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_wiring: Option<GatewayClientWiringV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_tuple: Option<IdentityTuple>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_posture: Option<PlacementPosture>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct GatewayLifecycleResponseDef {
    status: GatewayStatusV1,
    #[serde(default)]
    client_wiring: Option<GatewayClientWiringV1>,
    #[serde(default)]
    identity_tuple: Option<IdentityTuple>,
    #[serde(default)]
    placement_posture: Option<PlacementPosture>,
}

impl GatewayLifecycleResponseV1 {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }
}

impl TryFrom<GatewayLifecycleResponseDef> for GatewayLifecycleResponseV1 {
    type Error = String;

    fn try_from(value: GatewayLifecycleResponseDef) -> Result<Self, Self::Error> {
        let response = Self {
            status: value.status,
            client_wiring: value.client_wiring,
            identity_tuple: value.identity_tuple,
            placement_posture: value.placement_posture,
        };
        response.validate_identity_contract()?;
        Ok(response)
    }
}

/// Closed, untrusted Host gateway turn request. This representation grants no
/// placement, current-run authority or credential access. Cleanup secrets must
/// be retained before sending and must never enter native configuration or logs.
///
/// ```
/// use transport_api_types::HostGatewayTurnRequestV1;
/// assert!(serde_json::from_str::<HostGatewayTurnRequestV1>("{}").is_err());
/// ```
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HostGatewayTurnRequestDef")]
pub struct HostGatewayTurnRequestV1 {
    pub schema_version: u32,
    pub operation: HostGatewayTurnOperationV1,
}

/// One Host turn operation. Acquire is restricted to the HostActor wire role;
/// the service must independently authenticate that role and its actual placement.
/// No Debug implementation is provided for secret-bearing operations.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostGatewayTurnOperationV1 {
    Acquire {
        acquire_id: String,
        selection: Box<OrdinaryGatewaySelectionContextV1>,
        cleanup_token: String,
    },
    Revalidate {
        acquire_id: String,
        lease_handle: String,
    },
    Release {
        acquire_id: String,
        #[serde(deserialize_with = "host_gateway_turn_release_handle")]
        lease_handle: Option<String>,
        cleanup_token: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostGatewayTurnRequestDef {
    schema_version: u32,
    operation: HostGatewayTurnOperationV1,
}

// A custom deserializer makes a missing key an error while allowing explicit null.
fn host_gateway_turn_release_handle<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

impl HostGatewayTurnRequestV1 {
    /// Derive the stable public acquisition key from an already generated and
    /// retained 32-byte secret encoded as canonical unpadded base64url. This is
    /// only a cleanup commitment, never authenticated execution authority.
    ///
    /// This all-zero token is a synthetic derivation fixture, not a production secret.
    /// ```
    /// use transport_api_types::HostGatewayTurnRequestV1;
    /// let fixture = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    /// assert_eq!(HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(fixture).unwrap(),
    ///     "hgt_2166971add18dd42267a763b95a2ccb412255f8c5d0c8d4614ada8d065b4811a");
    /// ```
    pub fn acquire_id_from_cleanup_token(cleanup_token: &str) -> Result<String, String> {
        host_gateway_turn_acquire_id(cleanup_token)
    }

    /// Validate the closed representation, including the cleanup commitment.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("host_gateway_turn_invalid_request".into());
        }
        let (acquire_id, cleanup_token, handle) = match &self.operation {
            HostGatewayTurnOperationV1::Acquire {
                acquire_id,
                selection,
                cleanup_token,
            } => {
                selection
                    .validate()
                    .map_err(|_| "host_gateway_turn_invalid_request")?;
                if !matches!(
                    selection.runtime,
                    OrdinaryGatewayRuntimeIdentityV1::HostActor { .. }
                ) {
                    return Err("host_gateway_turn_invalid_request".into());
                }
                (acquire_id, Some(cleanup_token), None)
            }
            HostGatewayTurnOperationV1::Revalidate {
                acquire_id,
                lease_handle,
            } => (acquire_id, None, Some(lease_handle)),
            HostGatewayTurnOperationV1::Release {
                acquire_id,
                lease_handle,
                cleanup_token,
            } => (acquire_id, Some(cleanup_token), lease_handle.as_ref()),
        };
        let digest = acquire_id
            .strip_prefix("hgt_")
            .ok_or("host_gateway_turn_invalid_request")?;
        selection_sha256(digest, "host gateway acquisition")
            .map_err(|_| "host_gateway_turn_invalid_request")?;
        if let Some(token) = cleanup_token {
            if host_gateway_turn_acquire_id(token)? != *acquire_id {
                return Err("host_gateway_turn_invalid_request".into());
            }
        }
        if let Some(handle) = handle {
            host_gateway_turn_handle(handle)?;
        }
        Ok(())
    }
}

impl TryFrom<HostGatewayTurnRequestDef> for HostGatewayTurnRequestV1 {
    type Error = String;

    fn try_from(value: HostGatewayTurnRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            operation: value.operation,
        };
        request.validate()?;
        Ok(request)
    }
}

fn host_gateway_turn_acquire_id(cleanup_token: &str) -> Result<String, String> {
    use base64::Engine as _;
    use sha2::{Digest as _, Sha256};
    if cleanup_token.len() != 43 {
        return Err("host_gateway_turn_invalid_request".into());
    }
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let token = engine
        .decode(cleanup_token)
        .map_err(|_| "host_gateway_turn_invalid_request")?;
    if token.len() != 32 || engine.encode(&token) != cleanup_token {
        return Err("host_gateway_turn_invalid_request".into());
    }
    let mut digest = Sha256::new();
    digest.update(b"substrate.host-gateway-turn.cleanup.v1\0");
    digest.update(&token);
    Ok(format!("hgt_{:x}", digest.finalize()))
}

// Opaque locator encoding: independently generated 128-bit service and route IDs.
// Syntax validation does not authenticate issuance or the selected service instance.
fn host_gateway_turn_handle(handle: &str) -> Result<(), String> {
    let (service, route) = handle
        .strip_prefix("hgl_")
        .and_then(|suffix| suffix.split_once('_'))
        .ok_or("host_gateway_turn_invalid_handle")?;
    if ![service, route].iter().all(|part| {
        part.len() == 32
            && part
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err("host_gateway_turn_invalid_handle".into());
    }
    Ok(())
}

/// Closed Host gateway turn reply. Unknown denotes unresolved identity and never
/// proves cleanup. Released is reserved for service-owned terminal cleanup.
///
/// ```
/// use transport_api_types::HostGatewayTurnResponseV1;
/// assert!(serde_json::from_str::<HostGatewayTurnResponseV1>("{}").is_err());
/// ```
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HostGatewayTurnResponseDef")]
pub struct HostGatewayTurnResponseV1 {
    pub schema_version: u32,
    pub result: HostGatewayTurnResultV1,
}

/// A result carries no cleanup secret. Handles are opaque service locators and
/// must not be included in diagnostics. Match the result against the sent request.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostGatewayTurnResultV1 {
    Acquired {
        acquire_id: String,
        lease_handle: String,
        #[serde(deserialize_with = "host_gateway_turn_wiring")]
        client_wiring: GatewayClientWiringV1,
    },
    Revalidated {
        acquire_id: String,
        lease_handle: String,
    },
    Released {
        acquire_id: String,
    },
    Unknown {
        acquire_id: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostGatewayTurnResponseDef {
    schema_version: u32,
    result: HostGatewayTurnResultV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostGatewayTurnWiringDef {
    openai_base_url: String,
    anthropic_base_url: String,
}

fn host_gateway_turn_wiring<'de, D>(deserializer: D) -> Result<GatewayClientWiringV1, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wiring = HostGatewayTurnWiringDef::deserialize(deserializer)?;
    Ok(GatewayClientWiringV1 {
        openai_base_url: wiring.openai_base_url,
        anthropic_base_url: wiring.anthropic_base_url,
    })
}

fn host_gateway_turn_endpoint(endpoint: &str) -> Result<(), String> {
    let port = endpoint
        .strip_prefix("http://127.0.0.1:")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .ok_or("host_gateway_turn_invalid_wiring")?;
    if endpoint != format!("http://127.0.0.1:{port}") {
        return Err("host_gateway_turn_invalid_wiring".into());
    }
    Ok(())
}

impl HostGatewayTurnResponseV1 {
    /// Validate representation only; a decoded response is not admission proof.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("host_gateway_turn_invalid_response".into());
        }
        let acquire_id = match &self.result {
            HostGatewayTurnResultV1::Acquired {
                acquire_id,
                lease_handle,
                client_wiring,
            } => {
                host_gateway_turn_handle(lease_handle)?;
                host_gateway_turn_endpoint(&client_wiring.openai_base_url)?;
                host_gateway_turn_endpoint(&client_wiring.anthropic_base_url)?;
                acquire_id
            }
            HostGatewayTurnResultV1::Revalidated {
                acquire_id,
                lease_handle,
            } => {
                host_gateway_turn_handle(lease_handle)?;
                acquire_id
            }
            HostGatewayTurnResultV1::Released { acquire_id }
            | HostGatewayTurnResultV1::Unknown { acquire_id } => acquire_id,
        };
        let digest = acquire_id
            .strip_prefix("hgt_")
            .ok_or("host_gateway_turn_invalid_response")?;
        selection_sha256(digest, "host gateway acquisition")
            .map_err(|_| "host_gateway_turn_invalid_response".to_owned())
    }

    /// Check operation/result, exact echoed acquisition key and any known handle.
    /// A successful Unknown match still leaves cleanup unresolved.
    pub fn validate_for_request(&self, request: &HostGatewayTurnRequestV1) -> Result<(), String> {
        request.validate()?;
        self.validate()?;
        let matches = match (&request.operation, &self.result) {
            (
                HostGatewayTurnOperationV1::Acquire {
                    acquire_id: sent, ..
                }
                | HostGatewayTurnOperationV1::Revalidate {
                    acquire_id: sent, ..
                }
                | HostGatewayTurnOperationV1::Release {
                    acquire_id: sent, ..
                },
                HostGatewayTurnResultV1::Unknown { acquire_id },
            ) => sent == acquire_id,
            (
                HostGatewayTurnOperationV1::Acquire {
                    acquire_id: sent, ..
                },
                HostGatewayTurnResultV1::Acquired { acquire_id, .. },
            ) => sent == acquire_id,
            (
                HostGatewayTurnOperationV1::Revalidate {
                    acquire_id: sent,
                    lease_handle: held,
                },
                HostGatewayTurnResultV1::Revalidated {
                    acquire_id,
                    lease_handle,
                },
            ) => sent == acquire_id && held == lease_handle,
            (
                HostGatewayTurnOperationV1::Release {
                    acquire_id: sent, ..
                },
                HostGatewayTurnResultV1::Released { acquire_id },
            ) => sent == acquire_id,
            _ => false,
        };
        if !matches {
            return Err("host_gateway_turn_wrong_binding".into());
        }
        Ok(())
    }
}

impl TryFrom<HostGatewayTurnResponseDef> for HostGatewayTurnResponseV1 {
    type Error = String;

    fn try_from(value: HostGatewayTurnResponseDef) -> Result<Self, Self::Error> {
        let response = Self {
            schema_version: value.schema_version,
            result: value.result,
        };
        response.validate()?;
        Ok(response)
    }
}

/// Streaming frame describing incremental execution output.
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecuteStreamFrame {
    /// Initial handshake announcing the span identifier for this execution.
    Start {
        frame_identity: RuntimeFrameIdentityV1,
        span_id: String,
    },
    /// Incremental stdout data (base64 encoded for transport safety).
    Stdout {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    /// Incremental stderr data (base64 encoded for transport safety).
    Stderr {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    /// Optional higher-level agent event forwarded from the world.
    Event {
        frame_identity: RuntimeFrameIdentityV1,
        event: AgentEvent,
    },
    /// Terminal frame with exit metadata and optional filesystem diff.
    Exit {
        frame_identity: RuntimeFrameIdentityV1,
        event_identity: RuntimeEventIdentityV1,
        terminal_identity: RuntimeTerminalIdentityV1,
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fs_diff: Option<FsDiff>,
        #[serde(flatten, default)]
        process_telemetry: ProcessTelemetry,
    },
    /// Error reported while attempting to execute the command.
    Error {
        frame_identity: RuntimeFrameIdentityV1,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ExecuteStreamFrameDef {
    Start {
        frame_identity: RuntimeFrameIdentityV1,
        span_id: String,
    },
    Stdout {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    Stderr {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    Event {
        frame_identity: RuntimeFrameIdentityV1,
        event: AgentEvent,
        #[serde(default)]
        event_identity: Option<RuntimeEventIdentityV1>,
    },
    Exit {
        frame_identity: RuntimeFrameIdentityV1,
        event_identity: RuntimeEventIdentityV1,
        terminal_identity: RuntimeTerminalIdentityV1,
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        #[serde(default)]
        fs_diff: Option<FsDiff>,
        #[serde(flatten, default)]
        process_telemetry: ProcessTelemetry,
    },
    Error {
        frame_identity: RuntimeFrameIdentityV1,
        message: String,
    },
}

impl ExecuteStreamFrame {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        let frame_identity = match self {
            Self::Start { frame_identity, .. }
            | Self::Stdout { frame_identity, .. }
            | Self::Stderr { frame_identity, .. }
            | Self::Event { frame_identity, .. }
            | Self::Exit { frame_identity, .. }
            | Self::Error { frame_identity, .. } => frame_identity,
        };
        frame_identity.validate()?;

        match self {
            Self::Event { event, .. } => event
                .event_identity
                .as_ref()
                .ok_or_else(|| "runtime Event frame requires event.event_identity".to_string())
                .and_then(RuntimeEventIdentityV1::validate),
            Self::Exit {
                event_identity,
                terminal_identity,
                ..
            } => {
                event_identity.validate()?;
                terminal_identity.validate()?;
                if !terminal_identity.matches_event(event_identity) {
                    return Err(
                        "runtime Exit event_identity must equal terminal_identity".to_string()
                    );
                }
                Ok(())
            }
            Self::Start { .. } | Self::Stdout { .. } | Self::Stderr { .. } | Self::Error { .. } => {
                Ok(())
            }
        }
    }

    pub fn terminal_identity(&self) -> Option<&RuntimeTerminalIdentityV1> {
        match self {
            Self::Exit {
                terminal_identity, ..
            } => Some(terminal_identity),
            _ => None,
        }
    }

    /// Serialize one already-identified frame to its canonical replayable NDJSON bytes.
    pub fn canonical_ndjson_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate_identity_contract()?;
        let mut bytes = serde_json::to_vec(self).map_err(|err| err.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

impl TryFrom<ExecuteStreamFrameDef> for ExecuteStreamFrame {
    type Error = String;

    fn try_from(value: ExecuteStreamFrameDef) -> Result<Self, String> {
        let frame = match value {
            ExecuteStreamFrameDef::Start {
                frame_identity,
                span_id,
            } => Self::Start {
                frame_identity,
                span_id,
            },
            ExecuteStreamFrameDef::Stdout {
                frame_identity,
                chunk_b64,
            } => Self::Stdout {
                frame_identity,
                chunk_b64,
            },
            ExecuteStreamFrameDef::Stderr {
                frame_identity,
                chunk_b64,
            } => Self::Stderr {
                frame_identity,
                chunk_b64,
            },
            ExecuteStreamFrameDef::Event {
                frame_identity,
                event,
                event_identity,
            } => {
                if event_identity.is_some() {
                    return Err(
                        "runtime Event frame must not duplicate event_identity outside event"
                            .to_string(),
                    );
                }
                Self::Event {
                    frame_identity,
                    event,
                }
            }
            ExecuteStreamFrameDef::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
            } => Self::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
            },
            ExecuteStreamFrameDef::Error {
                frame_identity,
                message,
            } => Self::Error {
                frame_identity,
                message,
            },
        };
        frame.validate_identity_contract()?;
        Ok(frame)
    }
}

impl<'de> Deserialize<'de> for ExecuteStreamFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = ExecuteStreamFrameDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("bad_request: {0}")]
    BadRequest(String),
    #[error("not_found: {0}")]
    NotFound(String),
    #[error("rate_limited: {0}")]
    RateLimited(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Agent-reported world enforcement readiness (world scope).
///
/// This response is produced by `GET /v1/doctor/world`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorReportV1 {
    pub schema_version: u32,
    pub ok: bool,
    pub collected_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_host_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_context_commitment: Option<String>,
    /// Whether the connected world-service supports ingesting `PolicySnapshotV1` on execution requests.
    #[serde(default)]
    pub policy_snapshot_v1_supported: bool,
    /// The policy resolution mode most recently used by the world-service (when known).
    #[serde(default)]
    pub policy_resolution_mode: Option<PolicyResolutionModeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netfilter_status: Option<WorldDoctorNetfilterStatusV1>,
    pub landlock: WorldDoctorLandlockV1,
    pub world_fs_strategy: WorldDoctorWorldFsStrategyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldDoctorNetfilterStatusV1 {
    pub requested: bool,
    pub enabled: bool,
    pub world_netfilter_enable_present: bool,
    #[serde(default)]
    pub last_failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyResolutionModeV1 {
    SnapshotV3,
    LegacyLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorLandlockV1 {
    pub supported: bool,
    pub abi: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyV1 {
    pub primary: WorldDoctorWorldFsStrategyKindV1,
    pub fallback: WorldDoctorWorldFsStrategyKindV1,
    pub probe: WorldDoctorWorldFsStrategyProbeV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyKindV1 {
    Overlay,
    Fuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyProbeV1 {
    pub id: String,
    pub probe_file: String,
    pub result: WorldDoctorWorldFsStrategyProbeResultV1,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyProbeResultV1 {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::{json, Value};

    const UNIX_IH_FRAME: &str = "domain=substrate.install_bootstrap_context\n\
version=1\n\
selected_host_prefix=L29wdC9zdWJzdHJhdGUtZGV2\n\
host_substrate_home=L29wdC9zdWJzdHJhdGUtZGV2\n\
host_substrate_root=L29wdC9zdWJzdHJhdGUtZGV2\n\
principal_kind=unix\n\
principal_account=YWxpY2U\n\
principal_uid=1000\n";
    const UNIX_IH_COMMITMENT: &str =
        "0320704f788ba8e9f13b2eed7af83ad3f33fc999ab80f21abc83722314c9a79d";
    const UNIX_IH_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5pbnN0YWxsX2Jvb3RzdHJhcF9jb250ZXh0CnZlcnNpb249MQpzZWxlY3RlZF9ob3N0X3ByZWZpeD1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKaG9zdF9zdWJzdHJhdGVfaG9tZT1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKaG9zdF9zdWJzdHJhdGVfcm9vdD1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKcHJpbmNpcGFsX2tpbmQ9dW5peApwcmluY2lwYWxfYWNjb3VudD1ZV3hwWTJVCnByaW5jaXBhbF91aWQ9MTAwMApob3N0X2NvbnRleHRfY29tbWl0bWVudD0wMzIwNzA0Zjc4OGJhOGU5ZjEzYjJlZWQ3YWY4M2FkM2YzM2ZjOTk5YWI4MGYyMWFiYzgzNzIyMzE0YzlhNzlkCg";
    const WINDOWS_IH_COMMITMENT: &str =
        "3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7";
    const WINDOWS_IH_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5pbnN0YWxsX2Jvb3RzdHJhcF9jb250ZXh0CnZlcnNpb249MQpzZWxlY3RlZF9ob3N0X3ByZWZpeD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfaG9tZT1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfcm9vdD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKcHJpbmNpcGFsX2tpbmQ9d2luZG93cwpwcmluY2lwYWxfYWNjb3VudD1RVU5OUlZ4QmJHbGpaUQpwcmluY2lwYWxfc2lkPVV5MHhMVFV0TWpFdE1UQXdNQQpob3N0X2NvbnRleHRfY29tbWl0bWVudD0zZTFlNzFiMzI1ZTkyYjE2ZjViZmMwZjNkODc1ZmQxNWYwNGExNjE1YWM5MGIxNDM5ZWIzN2FmZGY5MGE1YWM3Cg";
    const LIMA_PM_FRAME: &str = "domain=substrate.platform_bootstrap_mapping\n\
version=1\n\
host_context_commitment=0320704f788ba8e9f13b2eed7af83ad3f33fc999ab80f21abc83722314c9a79d\n\
platform_kind=lima\n\
instance_name=c3Vic3RyYXRl\n\
guest_machine_id=0123456789abcdef0123456789abcdef\n\
host_platform_control_root=L1VzZXJzL2FsaWNlLy5saW1h\n\
realized_substrate_home=L2hvbWUvc3Vic3RyYXRlLy5zdWJzdHJhdGU\n\
realized_principal_account=c3Vic3RyYXRl\n\
realized_principal_uid=1000\n\
transport_kind=lima\n\
transport_host=L29wdC9zdWJzdHJhdGUtZGV2L3NvY2svYWdlbnQuc29jaw\n\
transport_guest_socket=L3J1bi9zdWJzdHJhdGUuc29jaw\n";
    const LIMA_PM_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5wbGF0Zm9ybV9ib290c3RyYXBfbWFwcGluZwp2ZXJzaW9uPTEKaG9zdF9jb250ZXh0X2NvbW1pdG1lbnQ9MDMyMDcwNGY3ODhiYThlOWYxM2IyZWVkN2FmODNhZDNmMzNmYzk5OWFiODBmMjFhYmM4MzcyMjMxNGM5YTc5ZApwbGF0Zm9ybV9raW5kPWxpbWEKaW5zdGFuY2VfbmFtZT1jM1ZpYzNSeVlYUmwKZ3Vlc3RfbWFjaGluZV9pZD0wMTIzNDU2Nzg5YWJjZGVmMDEyMzQ1Njc4OWFiY2RlZgpob3N0X3BsYXRmb3JtX2NvbnRyb2xfcm9vdD1MMVZ6WlhKekwyRnNhV05sTHk1c2FXMWgKcmVhbGl6ZWRfc3Vic3RyYXRlX2hvbWU9TDJodmJXVXZjM1ZpYzNSeVlYUmxMeTV6ZFdKemRISmhkR1UKcmVhbGl6ZWRfcHJpbmNpcGFsX2FjY291bnQ9YzNWaWMzUnlZWFJsCnJlYWxpemVkX3ByaW5jaXBhbF91aWQ9MTAwMAp0cmFuc3BvcnRfa2luZD1saW1hCnRyYW5zcG9ydF9ob3N0PUwyOXdkQzl6ZFdKemRISmhkR1V0WkdWMkwzTnZZMnN2WVdkbGJuUXVjMjlqYXcKdHJhbnNwb3J0X2d1ZXN0X3NvY2tldD1MM0oxYmk5emRXSnpkSEpoZEdVdWMyOWphdwo";
    const WSL_PM_FRAME: &str = "domain=substrate.platform_bootstrap_mapping\n\
version=1\n\
host_context_commitment=3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7\n\
platform_kind=wsl\n\
instance_name=U3Vic3RyYXRlLVdTTA\n\
guest_machine_id=abcdef0123456789abcdef0123456789\n\
host_platform_control_root=QzpcVXNlcnNcQWxpY2VcQXBwRGF0YVxMb2NhbFxTdWJzdHJhdGVcZm9yd2FyZGVyXHNjb3Bl\n\
realized_substrate_home=L2hvbWUvYm9iLy5zdWJzdHJhdGU\n\
realized_principal_account=Ym9i\n\
realized_principal_uid=1001\n\
transport_kind=wsl\n\
transport_host=XFwuXHBpcGVcc3Vic3RyYXRlLWFnZW50\n\
transport_guest_socket=L3J1bi9zdWJzdHJhdGUuc29jaw\n";
    const WSL_PM_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5wbGF0Zm9ybV9ib290c3RyYXBfbWFwcGluZwp2ZXJzaW9uPTEKaG9zdF9jb250ZXh0X2NvbW1pdG1lbnQ9M2UxZTcxYjMyNWU5MmIxNmY1YmZjMGYzZDg3NWZkMTVmMDRhMTYxNWFjOTBiMTQzOWViMzdhZmRmOTBhNWFjNwpwbGF0Zm9ybV9raW5kPXdzbAppbnN0YW5jZV9uYW1lPVUzVmljM1J5WVhSbExWZFRUQQpndWVzdF9tYWNoaW5lX2lkPWFiY2RlZjAxMjM0NTY3ODlhYmNkZWYwMTIzNDU2Nzg5Cmhvc3RfcGxhdGZvcm1fY29udHJvbF9yb290PVF6cGNWWE5sY25OY1FXeHBZMlZjUVhCd1JHRjBZVnhNYjJOaGJGeFRkV0p6ZEhKaGRHVmNabTl5ZDJGeVpHVnlYSE5qYjNCbApyZWFsaXplZF9zdWJzdHJhdGVfaG9tZT1MMmh2YldVdlltOWlMeTV6ZFdKemRISmhkR1UKcmVhbGl6ZWRfcHJpbmNpcGFsX2FjY291bnQ9WW05aQpyZWFsaXplZF9wcmluY2lwYWxfdWlkPTEwMDEKdHJhbnNwb3J0X2tpbmQ9d3NsCnRyYW5zcG9ydF9ob3N0PVhGd3VYSEJwY0dWY2MzVmljM1J5WVhSbExXRm5aVzUwCnRyYW5zcG9ydF9ndWVzdF9zb2NrZXQ9TDNKMWJpOXpkV0p6ZEhKaGRHVXVjMjlqYXcK";
    const WINDOWS_FORWARDER_SCOPE_FRAME: &str = "domain=substrate.windows_forwarder_scope\n\
version=1\n\
windows_sid=Uy0xLTUtMjEtMTAwMA\n\
distro_name=U3Vic3RyYXRlLVdTTA\n\
guest_machine_id=abcdef0123456789abcdef0123456789\n\
pipe_path=XFwuXHBpcGVcc3Vic3RyYXRlLWFnZW50\n";
    const WINDOWS_FORWARDER_SCOPE_DIGEST: &str =
        "3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a";

    fn encode_ih_record(record: &str) -> String {
        URL_SAFE_NO_PAD.encode(record.as_bytes())
    }

    fn unix_ih_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::decode(UNIX_IH_CARRIER).unwrap()
    }

    fn windows_ih_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::decode(WINDOWS_IH_CARRIER).unwrap()
    }

    #[test]
    fn install_bootstrap_context_unix_golden_vector_is_exact() {
        let context =
            InstallBootstrapContextV1::new_unix("/opt//substrate-dev/", "alice", 1000).unwrap();
        let carrier = InstallBootstrapContextCarrierV1::from_context(context.clone()).unwrap();

        assert_eq!(context.selected_host_prefix, "/opt/substrate-dev");
        assert_eq!(
            carrier.commitment_input().unwrap(),
            UNIX_IH_FRAME.as_bytes()
        );
        assert_eq!(carrier.host_context_commitment, UNIX_IH_COMMITMENT);
        assert_eq!(carrier.encode().unwrap(), UNIX_IH_CARRIER);
        assert_eq!(
            InstallBootstrapContextCarrierV1::decode(UNIX_IH_CARRIER).unwrap(),
            carrier
        );
    }

    #[test]
    fn install_bootstrap_context_windows_golden_vector_is_exact() {
        let context = InstallBootstrapContextV1::new_windows(
            r"c:/Users//Alice/AppData/Local/Substrate/",
            r"ACME\Alice",
            "S-1-5-21-1000",
        )
        .unwrap();
        let carrier = InstallBootstrapContextCarrierV1::from_context(context.clone()).unwrap();

        assert_eq!(
            context.selected_host_prefix,
            r"C:\Users\Alice\AppData\Local\Substrate"
        );
        assert_eq!(carrier.host_context_commitment, WINDOWS_IH_COMMITMENT);
        assert_eq!(carrier.encode().unwrap(), WINDOWS_IH_CARRIER);
        assert_eq!(
            InstallBootstrapContextCarrierV1::decode(WINDOWS_IH_CARRIER).unwrap(),
            carrier
        );
    }

    #[test]
    fn install_bootstrap_context_rejects_noncanonical_paths() {
        for raw in [
            "",
            "/",
            "//server/path",
            "relative/path",
            "/a/./b",
            "/a/../b",
        ] {
            assert!(InstallBootstrapContextV1::new_unix(raw, "alice", 1000).is_err());
        }

        for raw in [
            r"C:relative",
            r"\root-relative",
            r"C:\",
            r"\\server\share",
            r"\\?\C:\Substrate",
            r"C:\a\..\b",
            r"C:\CON\Substrate",
            r"C:\bad.\Substrate",
            r"C:\bad:name\Substrate",
        ] {
            assert!(
                InstallBootstrapContextV1::new_windows(raw, "ACME\\Alice", "S-1-5-21-1").is_err()
            );
        }
    }

    #[test]
    fn install_bootstrap_context_rejects_record_and_commitment_tampering() {
        let canonical_record =
            format!("{UNIX_IH_FRAME}host_context_commitment={UNIX_IH_COMMITMENT}\n");
        let cases = [
            canonical_record.replacen("version=1\n", "", 1),
            canonical_record.replacen("version=1\n", "version=1\nversion=1\n", 1),
            canonical_record.replacen("version=1", "unknown=1", 1),
            canonical_record.replacen(
                "version=1\nselected_host_prefix=",
                "selected_host_prefix=version=1\n",
                1,
            ),
            canonical_record.replacen("L29wdC9zdWJzdHJhdGUtZGV2", "***", 1),
            canonical_record.replacen("principal_uid=1000", "principal_uid=01000", 1),
            canonical_record.replacen("principal_uid=1000\n", "principal_uid=1000\r\n", 1),
            canonical_record.replacen(UNIX_IH_COMMITMENT, &"0".repeat(64), 1),
            format!("{canonical_record}extra=field\n"),
        ];

        for record in cases {
            assert!(
                InstallBootstrapContextCarrierV1::decode(&encode_ih_record(&record)).is_err(),
                "accepted tampered record: {record:?}"
            );
        }
        assert!(InstallBootstrapContextCarrierV1::decode("not+base64").is_err());
        assert!(InstallBootstrapContextCarrierV1::decode(&format!("{UNIX_IH_CARRIER}=")).is_err());
    }

    #[test]
    fn platform_bootstrap_mapping_lima_golden_vector_is_exact() {
        let carrier = unix_ih_carrier();
        let mapping = PlatformBootstrapMappingV1::new_lima(
            &carrier,
            "substrate",
            "0123456789abcdef0123456789abcdef",
            "/Users//alice/.lima/",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/opt/substrate-dev/sock/agent.sock",
            "/run/substrate.sock",
        )
        .unwrap();

        assert_eq!(mapping.host_platform_control_root, "/Users/alice/.lima");
        assert_eq!(
            URL_SAFE_NO_PAD
                .decode(mapping.encode(&carrier).unwrap())
                .unwrap(),
            LIMA_PM_FRAME.as_bytes()
        );
        assert_eq!(LIMA_PM_FRAME.lines().count(), 13);
        assert!(LIMA_PM_FRAME.ends_with('\n'));
        assert_eq!(mapping.encode(&carrier).unwrap(), LIMA_PM_CARRIER);
        assert_eq!(
            PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &carrier).unwrap(),
            mapping
        );
        assert_ne!(
            mapping.host_platform_control_root,
            mapping.realized_substrate_home
        );
        assert_ne!(
            carrier.context.intended_host_principal,
            mapping.realized_principal
        );
    }

    #[test]
    fn platform_bootstrap_mapping_wsl_golden_vector_is_exact() {
        let carrier = windows_ih_carrier();
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"c:/Users/Alice/AppData/Local/Substrate/forwarder/scope/",
            "/home/bob/.substrate",
            "bob",
            1001,
            r"\\.\pipe\Substrate-Agent",
            "/run/substrate.sock",
        )
        .unwrap();

        assert_eq!(
            mapping.host_platform_control_root,
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope"
        );
        assert_eq!(
            mapping.realized_transport,
            PlatformTransportIdentityV1::Wsl {
                pipe_path: r"\\.\pipe\substrate-agent".to_string(),
                guest_socket: "/run/substrate.sock".to_string(),
            }
        );
        assert_eq!(
            URL_SAFE_NO_PAD
                .decode(mapping.encode(&carrier).unwrap())
                .unwrap(),
            WSL_PM_FRAME.as_bytes()
        );
        assert_eq!(WSL_PM_FRAME.lines().count(), 13);
        assert!(WSL_PM_FRAME.ends_with('\n'));
        assert_eq!(mapping.encode(&carrier).unwrap(), WSL_PM_CARRIER);
        assert_eq!(
            PlatformBootstrapMappingV1::decode(WSL_PM_CARRIER, &carrier).unwrap(),
            mapping
        );
    }

    #[test]
    fn platform_bootstrap_mapping_rejects_noncanonical_or_tampered_records() {
        let carrier = unix_ih_carrier();
        let canonical = LIMA_PM_FRAME;
        let noncanonical_host_path = encode_inner_field("/Users/alice//.lima");
        let noncanonical_guest_path = encode_inner_field("/home/substrate//.substrate");
        let cases = [
            canonical.replacen("version=1\n", "", 1),
            canonical.replacen("version=1\n", "version=1\nversion=1\n", 1),
            canonical.replacen("version=1", "unknown=1", 1),
            canonical.replacen(
                "version=1\nhost_context_commitment=",
                "host_context_commitment=version=1\n",
                1,
            ),
            canonical.replacen("c3Vic3RyYXRl", "***", 1),
            canonical.replacen("c3Vic3RyYXRl", "c3Vic3RyYXRl=", 1),
            canonical.replacen(UNIX_IH_COMMITMENT, &"0".repeat(64), 1),
            canonical.replacen(UNIX_IH_COMMITMENT, "abc", 1),
            canonical.replacen(UNIX_IH_COMMITMENT, &UNIX_IH_COMMITMENT.to_uppercase(), 1),
            canonical.replacen(
                "0123456789abcdef0123456789abcdef",
                "0123456789ABCDEF0123456789ABCDEF",
                1,
            ),
            canonical.replacen("0123456789abcdef0123456789abcdef", "0123456789abcdef", 1),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=01000",
                1,
            ),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=-1",
                1,
            ),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=4294967296",
                1,
            ),
            canonical.replacen(
                "host_platform_control_root=L1VzZXJzL2FsaWNlLy5saW1h",
                &format!("host_platform_control_root={noncanonical_host_path}"),
                1,
            ),
            canonical.replacen(
                "realized_substrate_home=L2hvbWUvc3Vic3RyYXRlLy5zdWJzdHJhdGU",
                &format!("realized_substrate_home={noncanonical_guest_path}"),
                1,
            ),
            canonical.replacen("transport_kind=lima", "transport_kind=wsl", 1),
            canonical.replacen("transport_host=", "extra=field\ntransport_host=", 1),
            canonical.trim_end_matches('\n').to_string(),
            canonical.replace('\n', "\r\n"),
        ];

        for record in cases {
            assert!(
                PlatformBootstrapMappingV1::decode(&encode_ih_record(&record), &carrier).is_err(),
                "accepted tampered record: {record:?}"
            );
        }
        assert!(
            PlatformBootstrapMappingV1::decode(&format!("{LIMA_PM_CARRIER}="), &carrier).is_err()
        );
        let windows_carrier = windows_ih_carrier();
        let noncanonical_windows_path =
            encode_inner_field(r"C:\Users\Alice\..\Alice\AppData\Local\Substrate");
        let noncanonical_windows_record = WSL_PM_FRAME.replacen(
            "host_platform_control_root=QzpcVXNlcnNcQWxpY2VcQXBwRGF0YVxMb2NhbFxTdWJzdHJhdGVcZm9yd2FyZGVyXHNjb3Bl",
            &format!("host_platform_control_root={noncanonical_windows_path}"),
            1,
        );
        assert!(PlatformBootstrapMappingV1::decode(
            &encode_ih_record(&noncanonical_windows_record),
            &windows_carrier,
        )
        .is_err());

        let mut mapping = PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &carrier).unwrap();
        mapping.realized_transport = PlatformTransportIdentityV1::Wsl {
            pipe_path: r"\\.\pipe\substrate-agent".to_string(),
            guest_socket: "/run/substrate.sock".to_string(),
        };
        assert!(mapping.validate(&carrier).is_err());

        let wrong_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/opt/other", "alice", 1000).unwrap(),
        )
        .unwrap();
        assert!(PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &wrong_carrier).is_err());
    }

    #[test]
    fn windows_forwarder_scope_and_pipe_normalization_are_canonical() {
        assert_eq!(
            normalize_windows_pipe_path(r"\\.\pipe\Substrate-Agent").unwrap(),
            r"\\.\pipe\substrate-agent"
        );
        assert_eq!(
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\Substrate-Agent",
            )
            .unwrap()
            .0,
            WINDOWS_FORWARDER_SCOPE_DIGEST
        );
        assert_eq!(
            windows_forwarder_scope_input(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WINDOWS_FORWARDER_SCOPE_FRAME.as_bytes()
        );
        assert_eq!(
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\SUBSTRATE-AGENT",
            )
            .unwrap()
            .0,
            WINDOWS_FORWARDER_SCOPE_DIGEST
        );

        for invalid in [
            "",
            r"\\.\pipe\",
            r"//./pipe/substrate-agent",
            r"\\server\pipe\substrate-agent",
            r"\\.\PIPE\substrate-agent",
            r"\\.\pipe\nested\name",
            r"\\.\pipe\has space",
            "\\\\.\\pipe\\line\nbreak",
            r"\\.\pipe\name$",
        ] {
            assert!(
                normalize_windows_pipe_path(invalid).is_err(),
                "accepted invalid pipe: {invalid:?}"
            );
        }
        assert!(normalize_windows_pipe_path(&format!(r"\\.\pipe\{}", "a".repeat(129))).is_err());

        let canonical = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .unwrap();
        for tampered in [
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1001",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL-2",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456780",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent-2",
            )
            .unwrap(),
        ] {
            assert_ne!(tampered, canonical);
        }

        assert!(WindowsForwarderScopeV1::derive(
            "s-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .is_err());
        assert!(WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "ABCDEF0123456789ABCDEF0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .is_err());
    }

    const LAITDP2_CLIENT: &str = "codex";
    const LAITDP2_ROUTER: &str = "substrate_gateway";
    const LAITDP2_PROVIDER: &str = "openai";
    const LAITDP2_AUTH_AUTHORITY: &str = "codex_subscription";
    const LAITDP2_PROTOCOL: &str = "openai.responses";

    #[derive(Clone, Copy)]
    struct Laitdp2PlatformCase {
        platform: &'static str,
        hidden_transport: &'static str,
    }

    const LAITDP2_PLATFORM_CASES: [Laitdp2PlatformCase; 3] = [
        Laitdp2PlatformCase {
            platform: "linux",
            hidden_transport: "unix_socket",
        },
        Laitdp2PlatformCase {
            platform: "macos",
            hidden_transport: "lima_forwarding",
        },
        Laitdp2PlatformCase {
            platform: "windows",
            hidden_transport: "wsl_named_pipe_or_tcp",
        },
    ];

    fn valid_cli_codex_payload() -> GatewayIntegratedAuthPayloadV1 {
        GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
            api_env: None,
        }
    }

    fn valid_api_openai_payload() -> GatewayIntegratedAuthPayloadV1 {
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());

        GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        }
    }

    fn laitdp2_policy_snapshot_with_restrictive_net_allowed() -> Value {
        json!({
            "schema_version": 3,
            "net_allowed": ["api.openai.com"],
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "write": {
                    "enabled": true,
                    "allow_list": ["."],
                    "deny_list": []
                }
            }
        })
    }

    #[test]
    fn gateway_lifecycle_request_rejects_unknown_fields() {
        let err = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/tmp",
            "env": null,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
                }
            },
            "world_network": null,
            "integrated_auth": null,
            "unexpected": true
        }))
        .expect_err("unknown request field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn gateway_lifecycle_response_round_trips_canonical_identity_objects() {
        let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "client_wiring": {
                "openai_base_url": "http://127.0.0.1:4040",
                "anthropic_base_url": "http://127.0.0.1:4040"
            },
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "provider": "openai",
                "auth_authority": "codex_subscription",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("valid lifecycle response should deserialize");

        let roundtrip = serde_json::to_value(&response).expect("serialize lifecycle response");
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/router")
                .and_then(Value::as_str),
            Some("substrate_gateway")
        );
        assert_eq!(
            roundtrip
                .pointer("/placement_posture/execution")
                .and_then(Value::as_str),
            Some("in_world")
        );
    }

    #[test]
    fn gateway_lifecycle_response_rejects_direct_provider_path_with_bridge_transport() {
        let err = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "identity_tuple": {
                "client": "codex",
                "router": "direct_provider_path",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "host_only",
                "host_to_world_bridge": true
            }
        }))
        .expect_err("invalid routing/posture combination should fail");

        assert!(err.to_string().contains("host_to_world_bridge"));
    }

    #[test]
    fn gateway_lifecycle_response_keeps_tuple_metadata_top_level_when_unavailable() {
        let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "unavailable",
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("unavailable lifecycle response should keep additive tuple metadata");

        let roundtrip = serde_json::to_value(&response).expect("serialize lifecycle response");
        assert_eq!(roundtrip.pointer("/status"), Some(&json!("unavailable")));
        assert_eq!(roundtrip.pointer("/client_wiring"), None);
        assert_eq!(
            roundtrip.pointer("/identity_tuple/client"),
            Some(&json!("codex"))
        );
        assert_eq!(
            roundtrip.pointer("/placement_posture/execution"),
            Some(&json!("in_world"))
        );
    }

    #[test]
    fn gateway_lifecycle_response_rejects_secret_like_tuple_values() {
        let err = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "provider": "https://api.openai.com/v1",
                "auth_authority": "~/.codex/auth.json",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect_err("secret-like tuple metadata should fail validation");

        let error_text = err.to_string();
        assert!(
            error_text.contains("provider") || error_text.contains("auth_authority"),
            "expected validation error to cite the rejected tuple fields, got: {error_text}"
        );
    }

    #[test]
    fn laitdp2_lifecycle_response_publishes_one_tuple_and_posture_meaning_across_platforms() {
        let mut canonical_tuple = None;
        let mut canonical_posture = None;

        for case in LAITDP2_PLATFORM_CASES {
            let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                },
                "identity_tuple": {
                    "client": LAITDP2_CLIENT,
                    "router": LAITDP2_ROUTER,
                    "provider": LAITDP2_PROVIDER,
                    "auth_authority": LAITDP2_AUTH_AUTHORITY,
                    "protocol": LAITDP2_PROTOCOL
                },
                "placement_posture": {
                    "execution": "in_world",
                    "host_to_world_bridge": true
                }
            }))
            .unwrap_or_else(|err| {
                panic!(
                    "{} should accept the shared tuple/posture contract despite hidden {} transport: {err}",
                    case.platform, case.hidden_transport
                )
            });

            let roundtrip = serde_json::to_value(response).expect("serialize lifecycle response");
            let tuple = roundtrip
                .pointer("/identity_tuple")
                .cloned()
                .expect("identity tuple is published");
            let posture = roundtrip
                .pointer("/placement_posture")
                .cloned()
                .expect("placement posture is published");

            assert_eq!(
                tuple.pointer("/client").and_then(Value::as_str),
                Some(LAITDP2_CLIENT)
            );
            assert_eq!(
                tuple.pointer("/router").and_then(Value::as_str),
                Some(LAITDP2_ROUTER)
            );
            assert_eq!(
                tuple.pointer("/provider").and_then(Value::as_str),
                Some(LAITDP2_PROVIDER)
            );
            assert_eq!(
                tuple.pointer("/auth_authority").and_then(Value::as_str),
                Some(LAITDP2_AUTH_AUTHORITY)
            );
            assert_eq!(
                tuple.pointer("/protocol").and_then(Value::as_str),
                Some(LAITDP2_PROTOCOL)
            );
            assert_eq!(
                posture.pointer("/execution").and_then(Value::as_str),
                Some("in_world")
            );
            assert_eq!(
                posture
                    .pointer("/host_to_world_bridge")
                    .and_then(Value::as_bool),
                Some(true)
            );

            if let Some(expected) = canonical_tuple.as_ref() {
                assert_eq!(
                    &tuple, expected,
                    "{} must not give tuple vocabulary platform-specific meaning",
                    case.platform
                );
            } else {
                canonical_tuple = Some(tuple);
            }

            if let Some(expected) = canonical_posture.as_ref() {
                assert_eq!(
                    &posture, expected,
                    "{} must not give placement posture platform-specific meaning",
                    case.platform
                );
            } else {
                canonical_posture = Some(posture);
            }

            assert!(
                roundtrip.pointer("/client_wiring/identity_tuple").is_none(),
                "{} must keep additive tuple metadata outside client_wiring: {roundtrip}",
                case.platform
            );
            assert!(
                roundtrip
                    .pointer("/client_wiring/placement_posture")
                    .is_none(),
                "{} must keep additive placement metadata outside client_wiring: {roundtrip}",
                case.platform
            );
        }
    }

    #[test]
    fn laitdp2_direct_provider_path_posture_invariants_are_platform_independent() {
        for case in LAITDP2_PLATFORM_CASES {
            for invalid_posture in [
                json!({ "execution": "in_world" }),
                json!({ "execution": "host_only", "host_to_world_bridge": true }),
            ] {
                let result = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
                    "status": "available",
                    "identity_tuple": {
                        "client": LAITDP2_CLIENT,
                        "router": "direct_provider_path",
                        "protocol": LAITDP2_PROTOCOL
                    },
                    "placement_posture": invalid_posture
                }));

                assert!(
                    result.is_err(),
                    "{} must reject direct_provider_path unless execution=host_only and bridge transport is absent",
                    case.platform
                );
            }
        }
    }

    #[test]
    fn laitdp2_bridge_transport_does_not_rewrite_router_or_in_world_net_allowed_governance() {
        for case in LAITDP2_PLATFORM_CASES {
            let request = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
                "profile": null,
                "cwd": "/workspace",
                "env": {},
                "agent_id": LAITDP2_CLIENT,
                "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
                "world_network": {
                    "isolate_network": true,
                    "allowed_domains": ["api.openai.com"]
                },
                "integrated_auth": null,
                "identity_tuple": {
                    "client": LAITDP2_CLIENT,
                    "router": LAITDP2_ROUTER,
                    "provider": LAITDP2_PROVIDER,
                    "auth_authority": LAITDP2_AUTH_AUTHORITY,
                    "protocol": LAITDP2_PROTOCOL
                },
                "placement_posture": {
                    "execution": "in_world",
                    "host_to_world_bridge": true
                }
            }))
            .unwrap_or_else(|err| {
                panic!(
                    "{} should allow bridge transport only as transport detail: {err}",
                    case.platform
                )
            });

            let roundtrip = serde_json::to_value(request).expect("serialize lifecycle request");
            assert_eq!(
                roundtrip
                    .pointer("/identity_tuple/router")
                    .and_then(Value::as_str),
                Some(LAITDP2_ROUTER),
                "{} must not convert bridge transport into router identity",
                case.platform
            );
            assert_eq!(
                roundtrip
                    .pointer("/policy_snapshot/net_allowed")
                    .and_then(Value::as_array)
                    .expect("net_allowed array"),
                &[json!("api.openai.com")],
                "{} must preserve policy net_allowed when bridge transport participates",
                case.platform
            );
            assert_eq!(
                roundtrip
                    .pointer("/world_network/allowed_domains")
                    .and_then(Value::as_array)
                    .expect("allowed domains array"),
                &[json!("api.openai.com")],
                "{} must preserve runtime network isolation request when bridge transport participates",
                case.platform
            );
        }
    }

    #[test]
    fn laitdp2_backend_id_remains_adapter_selector_not_tuple_semantics() {
        let request = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/workspace",
            "env": {},
            "agent_id": LAITDP2_CLIENT,
            "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
            "world_network": null,
            "integrated_auth": {
                "backend_id": "cli:codex",
                "cli_codex": {
                    "account_id": "acct_test",
                    "access_token": "test-token"
                }
            },
            "identity_tuple": {
                "client": LAITDP2_CLIENT,
                "router": LAITDP2_ROUTER,
                "provider": LAITDP2_PROVIDER,
                "auth_authority": LAITDP2_AUTH_AUTHORITY,
                "protocol": LAITDP2_PROTOCOL
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("backend_id selector should coexist with separate tuple semantics");

        let roundtrip = serde_json::to_value(request).expect("serialize lifecycle request");
        assert_eq!(
            roundtrip
                .pointer("/integrated_auth/backend_id")
                .and_then(Value::as_str),
            Some("cli:codex")
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/client")
                .and_then(Value::as_str),
            Some(LAITDP2_CLIENT)
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/router")
                .and_then(Value::as_str),
            Some(LAITDP2_ROUTER)
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/auth_authority")
                .and_then(Value::as_str),
            Some(LAITDP2_AUTH_AUTHORITY)
        );
        assert!(
            !roundtrip
                .pointer("/identity_tuple")
                .expect("identity tuple")
                .to_string()
                .contains("cli:codex"),
            "backend_id grammar must not substitute for tuple fields: {roundtrip}"
        );

        let invalid_tuple = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/workspace",
            "env": {},
            "agent_id": LAITDP2_CLIENT,
            "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
            "world_network": null,
            "integrated_auth": null,
            "identity_tuple": {
                "client": "cli:codex",
                "router": "direct_provider_path",
                "protocol": LAITDP2_PROTOCOL
            },
            "placement_posture": {
                "execution": "host_only"
            }
        }));
        assert!(
            invalid_tuple.is_err(),
            "tuple fields must reject backend-id grammar so backend_id cannot become semantic identity"
        );
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_unknown_facet_fields() {
        let err = serde_json::from_value::<GatewayIntegratedAuthPayloadV1>(json!({
            "backend_id": "api:openai",
            "api_env": {
                "env": {
                    "OPENAI_API_KEY": "sk-test"
                },
                "unexpected": true
            }
        }))
        .expect_err("unknown facet field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_multi_facet_payloads() {
        let mut payload = valid_cli_codex_payload();
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        payload.api_env = Some(GatewayApiEnvIntegratedAuthV1 { env });

        let err = payload
            .validate()
            .expect_err("multi-facet payload should fail");
        assert!(err.contains("exactly one auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_facet_backend_mismatch() {
        let mut payload = valid_cli_codex_payload();
        payload.backend_id = "api:openai".to_string();

        let err = payload.validate().expect_err("mismatch should fail");
        assert!(err.contains("incompatible auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_missing_facet_payloads() {
        let payload = GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: None,
        };

        let err = payload
            .validate()
            .expect_err("missing-facet payload should fail");
        assert!(err.contains("exactly one auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_blank_required_values() {
        let mut payload = valid_api_openai_payload();
        payload
            .api_env
            .as_mut()
            .expect("api_env")
            .env
            .insert("OPENAI_API_KEY".to_string(), "   ".to_string());

        let err = payload
            .validate()
            .expect_err("blank required value should fail");
        assert!(err.contains("empty api_env value"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_invalid_api_env_names() {
        let mut payload = valid_api_openai_payload();
        payload
            .api_env
            .as_mut()
            .expect("api_env")
            .env
            .insert("OPENAI API KEY".to_string(), "sk-test".to_string());

        let err = payload
            .validate()
            .expect_err("invalid api env name should fail");
        assert!(err.contains("invalid api_env env name"));
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_valid_cli_codex() {
        valid_cli_codex_payload()
            .validate()
            .expect("valid cli:codex");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_realized_cli_codex_backends() {
        for backend_id in ["cli:codex-host", "cli:codex-world"] {
            GatewayIntegratedAuthPayloadV1 {
                backend_id: backend_id.to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: Some("acct_test".to_string()),
                    access_token: "header.payload.signature".to_string(),
                }),
                api_env: None,
            }
            .validate()
            .unwrap_or_else(|err| panic!("valid {backend_id} cli_codex payload: {err}"));
        }
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_valid_api_openai() {
        valid_api_openai_payload()
            .validate()
            .expect("valid api:openai");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_cli_claude_code_api_env() {
        GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:claude_code".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 {
                env: HashMap::from([("ANTHROPIC_API_KEY".to_string(), "sk-ant-test".to_string())]),
            }),
        }
        .validate()
        .expect("valid cli:claude_code api_env");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_realized_cli_claude_code_backends() {
        for backend_id in ["cli:claude_code-host", "cli:claude_code-world"] {
            GatewayIntegratedAuthPayloadV1 {
                backend_id: backend_id.to_string(),
                cli_codex: None,
                api_env: Some(GatewayApiEnvIntegratedAuthV1 {
                    env: HashMap::from([(
                        "ANTHROPIC_API_KEY".to_string(),
                        "sk-ant-test".to_string(),
                    )]),
                }),
            }
            .validate()
            .unwrap_or_else(|err| panic!("valid {backend_id} api_env payload: {err}"));
        }
    }

    #[test]
    fn runtime_identity_v1_types_are_shared_without_transport_aliases() {
        let frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_transport".to_string(),
            frame_sequence: 1,
        };
        let event = RuntimeEventIdentityV1 {
            event_id: "evt_transport".to_string(),
            event_sequence: 1,
        };
        let terminal = RuntimeTerminalIdentityV1::from(&event);

        assert!(frame.validate().is_ok());
        assert!(event.validate().is_ok());
        assert!(terminal.matches_event(&event));
    }

    fn runtime_frame_identity(sequence: u64) -> RuntimeFrameIdentityV1 {
        RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_transport".to_string(),
            frame_sequence: sequence,
        }
    }

    fn runtime_event_identity(sequence: u64) -> RuntimeEventIdentityV1 {
        RuntimeEventIdentityV1 {
            event_id: format!("evt_transport_{sequence}"),
            event_sequence: sequence,
        }
    }

    fn identified_agent_event(sequence: u64) -> AgentEvent {
        let mut event = AgentEvent::message(
            "agent",
            "session",
            "run",
            substrate_common::agent_events::MessageEventKind::Status,
            "ok",
        );
        event.event_identity = Some(runtime_event_identity(sequence));
        event
    }

    #[test]
    fn execute_stream_frame_v1_serializes_one_canonical_identity_representation() {
        let event = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let event_json = serde_json::to_value(&event).expect("serialize Event frame");
        assert_eq!(event_json["frame_identity"]["schema_version"], json!(1));
        assert_eq!(
            event_json["frame_identity"]["stream_id"],
            json!("rts_transport")
        );
        assert_eq!(event_json["frame_identity"]["frame_sequence"], json!(2));
        assert_eq!(
            event_json["event"]["event_identity"]["event_sequence"],
            json!(1)
        );
        assert!(event_json.get("event_identity").is_none());

        let terminal_event = runtime_event_identity(2);
        let exit = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(3),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: ProcessTelemetry::default(),
        };
        let exit_json = serde_json::to_value(&exit).expect("serialize Exit frame");
        assert_eq!(
            exit_json["event_identity"]["event_id"],
            json!("evt_transport_2")
        );
        assert_eq!(
            exit_json["terminal_identity"]["terminal_event_id"],
            json!("evt_transport_2")
        );

        serde_json::from_value::<ExecuteStreamFrame>(event_json).expect("Event roundtrip");
        serde_json::from_value::<ExecuteStreamFrame>(exit_json).expect("Exit roundtrip");
    }

    #[test]
    fn execute_stream_frame_v1_rejects_missing_malformed_or_conflicting_identity() {
        let valid_event = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let mut missing_event_identity = serde_json::to_value(&valid_event).expect("serialize");
        missing_event_identity["event"]
            .as_object_mut()
            .expect("event object")
            .remove("event_identity");
        assert!(serde_json::from_value::<ExecuteStreamFrame>(missing_event_identity).is_err());

        let mut duplicate_event_identity = serde_json::to_value(&valid_event).expect("serialize");
        duplicate_event_identity
            .as_object_mut()
            .expect("frame object")
            .insert(
                "event_identity".to_string(),
                json!({"event_id": "evt_duplicate", "event_sequence": 1}),
            );
        assert!(serde_json::from_value::<ExecuteStreamFrame>(duplicate_event_identity).is_err());

        let mut zero_frame_sequence = serde_json::to_value(&valid_event).expect("serialize");
        zero_frame_sequence["frame_identity"]["frame_sequence"] = json!(0);
        assert!(serde_json::from_value::<ExecuteStreamFrame>(zero_frame_sequence).is_err());

        let terminal_event = runtime_event_identity(2);
        let valid_exit = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(3),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: ProcessTelemetry::default(),
        };
        let mut mismatched_terminal = serde_json::to_value(valid_exit).expect("serialize");
        mismatched_terminal["terminal_identity"]["terminal_event_id"] = json!("evt_other");
        assert!(serde_json::from_value::<ExecuteStreamFrame>(mismatched_terminal).is_err());
    }

    #[test]
    fn execute_stream_frame_replay_preserves_identity_and_canonical_bytes() {
        let frame = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let original = frame.canonical_ndjson_bytes().expect("canonical bytes");
        let replay = frame
            .clone()
            .canonical_ndjson_bytes()
            .expect("replay bytes");
        assert_eq!(replay, original);

        let transport_error = ExecuteStreamFrame::Error {
            frame_identity: runtime_frame_identity(3),
            message: "transport lost".to_string(),
        };
        assert!(transport_error.terminal_identity().is_none());
    }

    #[test]
    fn serialize_stream_frame_roundtrip() {
        let terminal_event = runtime_event_identity(1);
        let frame = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(1),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".into(),
            scopes_used: vec!["tcp:example.com:443".into()],
            fs_diff: None,
            process_telemetry: ProcessTelemetry {
                process_events: vec![ProcessEvent {
                    event_type: ProcessEventType::WorldProcessStart,
                    ts: "2026-04-01T00:00:00Z".into(),
                    ts_unix_ns: 1_743_465_600_000_000_000,
                    session_id: "ses_test".into(),
                    world_id: "wld_test".into(),
                    pid: 42,
                    ppid: 1,
                    cwd: "/tmp".into(),
                    parent_span: "spn_parent".into(),
                    parent_cmd_id: Some("cmd_test".into()),
                    argv: None,
                    argv_omitted: Some(true),
                    exe: None,
                    exit_code: None,
                    signal: None,
                    duration_ms: None,
                    env: None,
                }],
                process_events_status: ProcessEventsStatus::Truncated,
                process_events_reason: Some("capture_overflow".into()),
                process_events_dropped: Some(3),
                process_events_max: None,
                process_events_backend: None,
                process_events_error: None,
            },
        };

        let json = serde_json::to_string(&frame).expect("serialize");
        let back: ExecuteStreamFrame = serde_json::from_str(&json).expect("deserialize");

        match back {
            ExecuteStreamFrame::Exit {
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
                ..
            } => {
                assert_eq!(exit, 0);
                assert_eq!(span_id, "spn_test");
                assert_eq!(scopes_used, vec!["tcp:example.com:443".to_string()]);
                assert!(fs_diff.is_none());
                assert_eq!(process_telemetry.process_events.len(), 1);
                assert_eq!(
                    process_telemetry.process_events_status,
                    ProcessEventsStatus::Truncated
                );
                assert_eq!(
                    process_telemetry.process_events_reason.as_deref(),
                    Some("capture_overflow")
                );
                assert_eq!(process_telemetry.process_events_dropped, Some(3));
            }
            other => panic!("unexpected frame: {:?}", other),
        }
    }

    #[test]
    fn execute_request_world_fs_mode_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["Github.COM.".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let req = ExecuteRequest {
            schema_version: 1,
            ordinary_start_selection: None,
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: Some(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            world_network: Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            }),
            world_fs_mode: Some(WorldFsMode::ReadOnly),
            member_dispatch: None,
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("read_only"),
            "expected world_fs_mode to serialize"
        );
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(back.world_fs_mode, Some(WorldFsMode::ReadOnly));
        assert_eq!(
            back.shared_world,
            Some(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            })
        );
        assert_eq!(back.policy_snapshot.schema_version, 3);
        assert_eq!(
            back.policy_snapshot.net_allowed,
            vec!["Github.COM.".to_string()]
        );
        assert_eq!(
            back.world_network,
            Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            })
        );
    }

    #[test]
    fn execute_request_policy_snapshot_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["github.com".to_string(), "crates.io".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                },
            },
        };

        let req = ExecuteRequest {
            schema_version: 1,
            ordinary_start_selection: None,
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: None,
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        let snapshot = back.policy_snapshot;
        assert_eq!(snapshot.schema_version, 3);
        assert!(!snapshot.world_fs.host_visible);
        assert_eq!(
            snapshot.world_fs.deny_enforcement,
            Some(WorldFsDenyEnforcementV3::Strict)
        );
        assert_eq!(
            snapshot.world_fs.read.as_ref().expect("read").allow_list,
            vec!["src".to_string()]
        );
        assert_eq!(
            snapshot.net_allowed,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn execute_response_shared_world_round_trip() {
        let response = ExecuteResponse {
            exit: 0,
            span_id: "spn_123".into(),
            stdout_b64: "aGVsbG8=".into(),
            stderr_b64: String::new(),
            scopes_used: vec!["github.com".into()],
            fs_diff: None,
            shared_world: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_123".into(),
                world_id: "wld_123".into(),
                world_generation: 3,
                binding_state: SharedWorldBindingState::Active,
            }),
            process_telemetry: ProcessTelemetry::default(),
        };

        let json = serde_json::to_string(&response).expect("serialize response");
        assert!(
            json.contains("\"shared_world\""),
            "expected shared_world to serialize"
        );
        let back: ExecuteResponse =
            serde_json::from_str(&json).expect("deserialize execute response");
        assert_eq!(
            back.shared_world,
            Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_123".into(),
                world_id: "wld_123".into(),
                world_generation: 3,
                binding_state: SharedWorldBindingState::Active,
            })
        );
    }

    #[test]
    fn execute_cancel_request_round_trip() {
        let req = ExecuteCancelRequestV1 {
            span_id: "spn_cancel".to_string(),
            sig: "INT".to_string(),
        };

        let json = serde_json::to_string(&req).expect("serialize cancel request");
        let back: ExecuteCancelRequestV1 =
            serde_json::from_str(&json).expect("deserialize cancel request");
        assert_eq!(back, req);
    }

    fn test_absolute_binary_path() -> String {
        std::env::current_exe()
            .expect("current_exe")
            .to_string_lossy()
            .into_owned()
    }

    fn sample_retained_worker_launch_authority_proof() -> RetainedWorkerLaunchAuthorityProofV1 {
        RetainedWorkerLaunchAuthorityProofV1 {
            schema_version: 1,
            authority_store_id: "store_123".into(),
            issuer_request_id: "request_123".into(),
            canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                schema_version: 1,
                algorithm: "hmac-sha-256".into(),
                key_id: "admission_key_123".into(),
                digest_hex: "a".repeat(64),
            },
            registration_id: "registration_123".into(),
            registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "b".repeat(64),
            },
            authority_revision_after: 7,
            authority_record_commitment_after:
                RetainedWorkerAuthorityObjectCommitmentV1::StoreHmacSha256 {
                    key_id: "authority_key_123".into(),
                    domain: "substrate.host-session-authority.authority-record.v1".into(),
                    digest_hex: "c".repeat(64),
                },
            orchestration_session_id: "orch_123".into(),
            caller_participant_id: "ash_orch_123".into(),
            retained_participant_id: "ash_member_123".into(),
            bootstrap_run_id: "run_123".into(),
            transport_claim_id: "transport_claim_123".into(),
            backend_id: "cli:codex".into(),
            protocol: "substrate.agent.session".into(),
            world_binding: RetainedWorkerLaunchWorldBindingV1 {
                world_id: "world_123".into(),
                world_generation: 7,
            },
            current_policy_ref_id: "policy_ref_123".into(),
            current_policy_revision: "policy_revision_123".into(),
            retained_worker_ref_id: "retained_worker_ref_123".into(),
            retained_worker_commitment:
                RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "d".repeat(64),
                },
        }
    }

    #[test]
    fn retained_worker_launch_authority_proof_is_closed_and_strictly_validated() {
        let proof = sample_retained_worker_launch_authority_proof();
        proof.validate().expect("valid launch-authority proof");

        let json = serde_json::to_value(&proof).expect("serialize launch-authority proof");
        let decoded: RetainedWorkerLaunchAuthorityProofV1 =
            serde_json::from_value(json.clone()).expect("deserialize launch-authority proof");
        assert_eq!(decoded, proof);

        let mut changed_algorithm = json.clone();
        changed_algorithm["canonical_spawn_fingerprint"]["algorithm"] =
            serde_json::json!("sha-256");
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(changed_algorithm)
                .is_err()
        );

        let mut uppercase_digest = json.clone();
        uppercase_digest["canonical_spawn_fingerprint"]["digest_hex"] =
            serde_json::json!("A".repeat(64));
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(uppercase_digest)
                .is_err()
        );

        let mut unknown_field = json;
        unknown_field["unexpected"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(unknown_field).is_err()
        );
    }

    #[test]
    fn execute_request_member_dispatch_round_trip() {
        let binary_path = test_absolute_binary_path();
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let req = ExecuteRequest {
            schema_version: 1,
            ordinary_start_selection: None,
            profile: None,
            cmd: String::new(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: Some(MemberDispatchRequest::V1(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_123".into(),
                participant_id: "ash_member_123".into(),
                orchestrator_participant_id: "ash_orch_123".into(),
                parent_participant_id: Some("ash_parent_123".into()),
                resumed_from_participant_id: Some("ash_old_123".into()),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                run_id: "run_123".into(),
                world_id: "world_123".into(),
                world_generation: 7,
                initial_prompt: Some("first turn".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: binary_path.clone(),
                },
                retained_worker_launch_authority: None,
                e2_launch_activation: None,
            })),
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        let frozen_v1 = req
            .member_dispatch
            .as_ref()
            .and_then(MemberDispatchRequest::as_v1)
            .expect("fixture remains V1");
        assert_eq!(
            serde_json::to_vec(&MemberDispatchRequest::V1(frozen_v1.clone())).unwrap(),
            serde_json::to_vec(frozen_v1).unwrap(),
            "untagged V1 wrapper must preserve exact member-dispatch bytes"
        );
        assert!(json.contains("\"member_dispatch\""));
        assert!(!json.contains("\"acceptance_context\""));
        assert!(
            !json.contains("retained_worker_launch_authority"),
            "legacy/Run/Fork None compatibility must preserve the pre-carrier wire shape"
        );

        let mut acceptance_context = sample_world_work_acceptance_context();
        acceptance_context.request_id = "run_123".to_string();
        acceptance_context.message_id = None;
        let mut with_acceptance = serde_json::to_value(&req).expect("shape execute request");
        with_acceptance["acceptance_context"] =
            serde_json::to_value(&acceptance_context).expect("shape task acceptance context");
        let accepted: ExecuteRequest = serde_json::from_value(with_acceptance.clone())
            .expect("decode task acceptance context");
        assert_eq!(accepted.acceptance_context, Some(acceptance_context));

        let mut wrong_request = with_acceptance.clone();
        wrong_request["acceptance_context"]["request_id"] =
            serde_json::Value::String("run_other".to_string());
        assert!(serde_json::from_value::<ExecuteRequest>(wrong_request).is_err());

        with_acceptance["acceptance_context"]["message_id"] =
            serde_json::Value::String("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string());
        assert!(serde_json::from_value::<ExecuteRequest>(with_acceptance).is_err());

        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert!(back.cmd.is_empty());
        assert_eq!(
            back.member_dispatch,
            Some(MemberDispatchRequest::V1(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_123".into(),
                participant_id: "ash_member_123".into(),
                orchestrator_participant_id: "ash_orch_123".into(),
                parent_participant_id: Some("ash_parent_123".into()),
                resumed_from_participant_id: Some("ash_old_123".into()),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                run_id: "run_123".into(),
                world_id: "world_123".into(),
                world_generation: 7,
                initial_prompt: Some("first turn".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path,
                },
                retained_worker_launch_authority: None,
                e2_launch_activation: None,
            }))
        );
    }

    #[test]
    fn member_turn_submit_request_round_trip() {
        let req = MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".into(),
            participant_id: "ash_member_123".into(),
            orchestrator_participant_id: "ash_orch_123".into(),
            backend_id: "cli:codex".into(),
            run_id: "run_123".into(),
            world_id: "world_123".into(),
            world_generation: 7,
            prompt: "summarize the failure".into(),
            policy_snapshot_carrier: None,
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize member turn submit request");
        let back: MemberTurnSubmitRequestV1 =
            serde_json::from_str(&json).expect("deserialize member turn submit request");
        assert_eq!(back, req);
    }

    #[test]
    fn dispatch_policy_snapshot_carrier_round_trips_exact_e1_bytes_and_hash() {
        let snapshot = e2_sample_policy_snapshot();
        let carrier = e2_sample_policy_snapshot_carrier();
        let expected_bytes = serde_json::to_vec(&snapshot).expect("serialize exact E1 bytes");
        let expected_hash = format!("{:x}", Sha256::digest(&expected_bytes));

        assert_eq!(carrier.policy_snapshot_hash, expected_hash);
        assert_eq!(carrier.policy_snapshot_bytes().unwrap(), expected_bytes);
        assert_eq!(
            serde_json::to_vec(&carrier.policy_snapshot().unwrap()).unwrap(),
            serde_json::to_vec(&snapshot).unwrap()
        );

        let encoded = serde_json::to_value(&carrier).expect("serialize exact carrier");
        let decoded: DispatchPolicySnapshotCarrierV1 =
            serde_json::from_value(encoded).expect("deserialize exact carrier");
        assert_eq!(decoded, carrier);
    }

    #[test]
    fn dispatch_policy_snapshot_carrier_rejects_hash_mismatch_or_unknown_material() {
        let carrier = e2_sample_policy_snapshot_carrier();
        let mut mismatched = serde_json::to_value(&carrier).expect("serialize carrier");
        mismatched["policy_snapshot_hash"] = serde_json::Value::String("0".repeat(64));
        assert!(serde_json::from_value::<DispatchPolicySnapshotCarrierV1>(mismatched).is_err());

        let mut uppercase = serde_json::to_value(&carrier).expect("serialize carrier");
        uppercase["policy_snapshot_hash"] =
            serde_json::Value::String(carrier.policy_snapshot_hash.to_uppercase());
        assert!(serde_json::from_value::<DispatchPolicySnapshotCarrierV1>(uppercase).is_err());

        let mut unknown = serde_json::to_value(&carrier).expect("serialize carrier");
        unknown["unexpected"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<DispatchPolicySnapshotCarrierV1>(unknown).is_err());

        let exact_bytes = carrier
            .policy_snapshot_bytes()
            .expect("exact carrier bytes");
        let mut noncanonical_bytes = Vec::with_capacity(exact_bytes.len() + 1);
        noncanonical_bytes.push(b' ');
        noncanonical_bytes.extend_from_slice(&exact_bytes);
        let mut noncanonical = carrier.clone();
        noncanonical.policy_snapshot_byte_length = noncanonical_bytes.len() as u64;
        noncanonical.policy_snapshot_bytes_base64 =
            base64::engine::general_purpose::STANDARD.encode(&noncanonical_bytes);
        noncanonical.policy_snapshot_hash = format!("{:x}", Sha256::digest(&noncanonical_bytes));
        assert!(noncanonical.validate().is_err());
    }

    #[test]
    fn dispatch_policy_snapshot_carrier_rejects_substituted_cap_or_stale_revision_copy() {
        let authenticated = e2_sample_policy_snapshot_carrier();

        let mut substituted_cap = authenticated.clone();
        substituted_cap.immutable_worker_cap_ref.commitment_id =
            "dpc_018f0f3a-9b2c-7def-8abc-0123456789ae".to_string();
        substituted_cap
            .validate()
            .expect("substituted ref is well-formed");
        assert!(substituted_cap
            .validate_exact_authenticated_copy(&authenticated)
            .is_err());

        let mut stale_revision = authenticated.clone();
        stale_revision.immutable_worker_cap_application_revision -= 1;
        stale_revision
            .validate()
            .expect("stale revision is well-formed");
        assert!(stale_revision
            .validate_exact_authenticated_copy(&authenticated)
            .is_err());

        let mut substituted_snapshot = authenticated.clone();
        let mut snapshot = substituted_snapshot
            .policy_snapshot()
            .expect("authenticated snapshot");
        snapshot.net_allowed = vec!["substituted.example".to_string()];
        let substituted_bytes = serde_json::to_vec(&snapshot).expect("substituted snapshot bytes");
        substituted_snapshot.policy_snapshot_byte_length = substituted_bytes.len() as u64;
        substituted_snapshot.policy_snapshot_bytes_base64 =
            base64::engine::general_purpose::STANDARD.encode(&substituted_bytes);
        substituted_snapshot.policy_snapshot_hash =
            format!("{:x}", Sha256::digest(&substituted_bytes));
        substituted_snapshot
            .validate()
            .expect("internally valid substituted snapshot");
        assert!(substituted_snapshot
            .validate_exact_authenticated_copy(&authenticated)
            .is_err());

        let mut stale_snapshot_revision = authenticated.clone();
        stale_snapshot_revision.policy_snapshot_revision = "policy-revision-stale".to_string();
        stale_snapshot_revision
            .validate()
            .expect("stale snapshot revision remains structurally valid");
        assert!(stale_snapshot_revision
            .validate_exact_authenticated_copy(&authenticated)
            .is_err());

        let mut unauthenticated_request_copy = authenticated.clone();
        unauthenticated_request_copy.reason = Some("untrusted-request-material".to_string());
        unauthenticated_request_copy
            .validate()
            .expect("untrusted request copy remains structurally valid");
        assert!(unauthenticated_request_copy
            .validate_exact_authenticated_copy(&authenticated)
            .is_err());
    }

    #[test]
    fn member_turn_submit_carries_atomic_policy_snapshot_identity_and_omits_legacy_absence() {
        let carrier = e2_sample_policy_snapshot_carrier();
        let mut request_json = serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "cli:codex",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "prompt": "resume"
        });
        let legacy: MemberTurnSubmitRequestV1 =
            serde_json::from_value(request_json.clone()).expect("legacy request remains valid");
        assert!(legacy.policy_snapshot_carrier.is_none());
        assert!(legacy
            .validate_against_authenticated_policy_carrier(&carrier)
            .is_err());
        assert!(serde_json::to_value(&legacy)
            .expect("serialize legacy request")
            .get("policy_snapshot_carrier")
            .is_none());

        let mut acceptance_context = sample_world_work_acceptance_context();
        acceptance_context.request_id = "run_123".to_string();
        request_json["acceptance_context"] =
            serde_json::to_value(&acceptance_context).expect("serialize acceptance context");
        request_json["policy_snapshot_carrier"] =
            serde_json::to_value(&carrier).expect("serialize carrier");
        let carried: MemberTurnSubmitRequestV1 =
            serde_json::from_value(request_json).expect("carried request validates");
        carried
            .validate_against_authenticated_policy_carrier(&carrier)
            .expect("authenticated carrier exact-joins request bindings");
        assert_eq!(carried.policy_snapshot_carrier, Some(carrier.clone()));

        for (pointer, replacement) in [
            ("/orchestration_session_id", serde_json::json!("orch_other")),
            ("/participant_id", serde_json::json!("ash_member_other")),
            (
                "/orchestrator_participant_id",
                serde_json::json!("ash_orch_other"),
            ),
            ("/backend_id", serde_json::json!("cli:claude-code")),
            ("/run_id", serde_json::json!("run_other")),
            ("/world_id", serde_json::json!("world_other")),
            ("/world_generation", serde_json::json!(8)),
        ] {
            let mut changed = serde_json::to_value(&carried).expect("serialize carried request");
            *changed
                .pointer_mut(pointer)
                .expect("request mutation pointer") = replacement;
            assert!(serde_json::from_value::<MemberTurnSubmitRequestV1>(changed).is_err());
        }

        for (pointer, replacement) in [
            (
                "/policy_snapshot_carrier/orchestration_session_id",
                serde_json::json!("orch_other"),
            ),
            (
                "/policy_snapshot_carrier/subject/retained_participant_id",
                serde_json::json!("ash_member_other"),
            ),
            (
                "/policy_snapshot_carrier/subject/active_run_id",
                serde_json::json!("run_other"),
            ),
            (
                "/policy_snapshot_carrier/caller_participant_id",
                serde_json::json!("ash_orch_other"),
            ),
            (
                "/policy_snapshot_carrier/target_backend_id",
                serde_json::json!("cli:claude-code"),
            ),
            (
                "/policy_snapshot_carrier/target_world/world_id",
                serde_json::json!("world_other"),
            ),
            (
                "/policy_snapshot_carrier/target_world/world_generation",
                serde_json::json!(8),
            ),
        ] {
            let mut changed = serde_json::to_value(&carried).expect("serialize carried request");
            *changed
                .pointer_mut(pointer)
                .expect("carrier mutation pointer") = replacement;
            assert!(serde_json::from_value::<MemberTurnSubmitRequestV1>(changed).is_err());
        }

        let mut changed_message = serde_json::to_value(&carried).expect("serialize request");
        changed_message["policy_snapshot_carrier"]["subject"]["message_id"] =
            serde_json::json!("wwm_018f0f3a-9b2c-7def-8abc-0123456789ad");
        assert!(serde_json::from_value::<MemberTurnSubmitRequestV1>(changed_message).is_err());

        let mut changed_caller_backend = serde_json::to_value(&carried).expect("serialize request");
        changed_caller_backend["policy_snapshot_carrier"]["caller_backend_id"] =
            serde_json::json!("cli:claude-code");
        assert!(
            serde_json::from_value::<MemberTurnSubmitRequestV1>(changed_caller_backend).is_err()
        );
    }

    #[test]
    fn test_ordinary_zero_turn_codec_is_strict_and_transparent() {
        let mut carrier = e2_sample_policy_snapshot_carrier();
        carrier.target_world.world_generation = 0;
        let mut acceptance = sample_world_work_acceptance_context();
        acceptance.request_id = carrier.subject.active_run_id.clone();
        let request = MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: carrier.orchestration_session_id.clone(),
            participant_id: carrier.subject.retained_participant_id.clone(),
            orchestrator_participant_id: carrier.caller_participant_id.clone(),
            backend_id: carrier.target_backend_id.clone(),
            run_id: carrier.subject.active_run_id.clone(),
            world_id: carrier.target_world.world_id.clone(),
            world_generation: 0,
            prompt: "ordinary zero turn".into(),
            policy_snapshot_carrier: Some(carrier.clone()),
            acceptance_context: Some(acceptance),
        };
        let ordinary = OrdinaryMemberTurnSubmitRequestV1::from_request(request.clone()).unwrap();
        let bytes = serde_json::to_vec(&ordinary).unwrap();
        assert_eq!(bytes, serde_json::to_vec(&request).unwrap());
        let decoded: OrdinaryMemberTurnSubmitRequestV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.as_request(), &request);
        assert!(request.validate().is_err());
        assert!(carrier.validate().is_err());
        assert!(carrier.validate_exact_authenticated_copy(&carrier).is_err());
        assert!(serde_json::from_slice::<MemberTurnSubmitRequestV1>(&bytes).is_err());
        assert!(serde_json::from_slice::<DispatchPolicySnapshotCarrierV1>(
            &serde_json::to_vec(&carrier).unwrap(),
        )
        .is_err());
        let raw = String::from_utf8(bytes.clone()).unwrap();
        for needle in [
            "\"schema_version\":1",
            "\"policy_snapshot_hash\":",
            "\"world_generation\":0",
        ] {
            let duplicate = if needle.ends_with(':') {
                raw.replacen(
                    needle,
                    &format!("{needle}\"{}\",{needle}", "a".repeat(64)),
                    1,
                )
            } else {
                raw.replacen(needle, &format!("{needle},{needle}"), 1)
            };
            assert!(serde_json::from_str::<OrdinaryMemberTurnSubmitRequestV1>(&duplicate).is_err());
        }
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        for pointer in [
            "/schema_version",
            "/world_generation",
            "/policy_snapshot_carrier",
            "/acceptance_context",
            "/policy_snapshot_carrier/immutable_worker_cap_application_revision",
        ] {
            let mut value = original.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            value
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(
                serde_json::from_value::<OrdinaryMemberTurnSubmitRequestV1>(value).is_err(),
                "missing {pointer}"
            );
        }
        for (pointer, replacement) in [
            ("/schema_version", json!(2)),
            ("/unexpected", json!(true)),
            ("/policy_snapshot_carrier/schema_version", json!(2)),
            ("/policy_snapshot_carrier/unexpected", json!(true)),
            ("/policy_snapshot_carrier/subject/unexpected", json!(true)),
            (
                "/policy_snapshot_carrier/immutable_worker_cap_application_revision",
                json!(0),
            ),
            (
                "/policy_snapshot_carrier/policy_snapshot_hash",
                json!("f".repeat(64)),
            ),
            (
                "/policy_snapshot_carrier/policy_snapshot_ref/commitment/kind",
                json!("Unknown"),
            ),
            ("/world_generation", json!(1)),
            ("/policy_snapshot_carrier", Value::Null),
        ] {
            let mut value = original.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            value
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert(key.into(), replacement);
            assert!(
                serde_json::from_value::<OrdinaryMemberTurnSubmitRequestV1>(value).is_err(),
                "bad {pointer}"
            );
        }
        let runtime = OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: request.orchestration_session_id,
            caller_participant_id: request.orchestrator_participant_id,
            participant_id: request.participant_id,
            backend_id: request.backend_id,
            run_id: request.run_id,
            world_id: request.world_id,
            world_generation: 0,
        };
        runtime.validate().unwrap(); // Representation only, no service capability.
    }

    fn e2_sample_policy_snapshot_carrier() -> DispatchPolicySnapshotCarrierV1 {
        let snapshot = e2_sample_policy_snapshot();
        let bytes = serde_json::to_vec(&snapshot).expect("serialize E1 fixture snapshot");
        let carrier = DispatchPolicySnapshotCarrierV1 {
            schema_version: 1,
            immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1 {
                authority_store_id: "authority-store-e2".to_string(),
                commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
                exact_linkage_hash: "a".repeat(64),
            },
            immutable_worker_cap_created_revision: 7,
            immutable_worker_cap_application_revision: 9,
            subject: RetainedTurnPolicyCommitmentSubjectV1 {
                retained_participant_id: "ash_member_123".to_string(),
                active_run_id: "run_123".to_string(),
                message_id: Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string()),
            },
            orchestration_session_id: "orch_123".to_string(),
            caller_participant_id: "ash_orch_123".to_string(),
            caller_backend_id: "cli:codex".to_string(),
            target_backend_id: "cli:codex".to_string(),
            target_world: WorldBindingRefV1 {
                world_id: "world_123".to_string(),
                world_generation: 7,
            },
            policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            policy_snapshot_byte_length: bytes.len() as u64,
            policy_snapshot_ref: AuthorityObjectRefV1 {
                ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                object_kind: AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "b".repeat(64),
                },
            },
            policy_snapshot_hash: format!("{:x}", Sha256::digest(&bytes)),
            policy_snapshot_revision: "policy-revision-e2-7".to_string(),
            reason: Some("narrow retained turn".to_string()),
        };
        carrier.validate().expect("valid E2 carrier fixture");
        carrier
    }

    fn e2_sample_policy_snapshot() -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["api.example.com".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: None,
                caged_required: true,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                },
            },
        }
    }

    fn e3a_sample_projection_carrier() -> ConfigProjectionActivationCarrierV1 {
        let authority_store_id = "cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0".to_string();
        let series_id = "cps_018f0f2e-7b4c-7aa1-8c22-123456789ab1".to_string();
        ConfigProjectionActivationCarrierV1 {
            authority_store_id: authority_store_id.clone(),
            series_id: series_id.clone(),
            dormant_projection_ref: ConfigProjectionRefV1 {
                authority_store_id: authority_store_id.clone(),
                series_id,
                record_id: "cpr_018f0f2e-7b4c-7aa1-8c22-123456789ab2".to_string(),
                revision: 1,
                record_hash: "1".repeat(64),
            },
            activation_intent_ref: ManagedGatewayActivationIntentRefV1 {
                authority_store_id: authority_store_id.clone(),
                activation_intent_id: "gai_018f0f2e-7b4c-7aa1-8c22-123456789ab3".to_string(),
                intent_hash: "2".repeat(64),
            },
            expected_gateway_ref: InWorldGatewayRefV1 {
                authority_store_id,
                gateway_instance_id: "cgi_018f0f2e-7b4c-7aa1-8c22-123456789ab4".to_string(),
                gateway_identity_hash: "3".repeat(64),
            },
            fence_id: "cpf_018f0f2e-7b4c-7aa1-8c22-123456789ab5".to_string(),
            consumer_id: "cpc_018f0f2e-7b4c-7aa1-8c22-123456789ab6".to_string(),
            consumer_lease_revision: 1,
            consumer_lease_hash: "4".repeat(64),
        }
    }

    fn e3c_sample_authoring_input_ref() -> ConfigProjectionAuthoringInputRefV1 {
        let mut reference = ConfigProjectionAuthoringInputRefV1 {
            authority_store_id: "cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0".to_string(),
            effective_config_source_hash: "1".repeat(64),
            agent_inventory_source_hash: "2".repeat(64),
            runtime_artifact_manifest_id: "ram_018f0f2e-7b4c-7aa1-8c22-123456789ab7".to_string(),
            runtime_artifact_manifest_revision: 1,
            runtime_artifact_manifest_hash: "3".repeat(64),
            input_ref_hash: String::new(),
        };
        reference.input_ref_hash = reference
            .canonical_hash()
            .expect("hash strict E3-C authoring reference");
        reference
    }

    #[test]
    fn config_projection_authoring_input_ref_v1_is_strict_and_hash_bound() {
        let reference = e3c_sample_authoring_input_ref();
        reference.validate().expect("valid authoring input ref");

        let bytes = serde_json::to_vec(&reference).expect("serialize authoring input ref");
        let decoded: ConfigProjectionAuthoringInputRefV1 =
            serde_json::from_slice(&bytes).expect("strict round trip");
        assert_eq!(decoded, reference);

        let mut unknown = serde_json::to_value(&reference).expect("serialize value");
        unknown["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ConfigProjectionAuthoringInputRefV1>(unknown).is_err());

        let mut substituted = reference.clone();
        substituted.agent_inventory_source_hash = "4".repeat(64);
        assert!(substituted.validate().is_err());

        let mut wrong_revision = reference;
        wrong_revision.runtime_artifact_manifest_revision = 2;
        wrong_revision.input_ref_hash = wrong_revision.canonical_hash().unwrap();
        assert!(wrong_revision.validate().is_err());
    }

    // Codec data only: these identities do not mint HSA, backend or permission evidence.
    fn retained_bootstrap_observation_fixture() -> RetainedBootstrapObservationV1 {
        let activation = e3a_sample_fresh_spawn_activation();
        let event_identity = RuntimeEventIdentityV1 {
            event_id: "native-bootstrap-event".into(),
            event_sequence: 3,
        };
        RetainedBootstrapObservationV1 {
            source_start: OrdinaryStartSelectionCarrierV1 {
                schema_version: 1,
                authority_store_id: activation.commitment_ref.authority_store_id.clone(),
                original_start_intent_id: "original-start-intent".into(),
                original_start_issuer_request_id: format!("req_{}", "d".repeat(64)),
                original_session_id: activation.orchestration_session_id.clone(),
                context_sha256: "a".repeat(64),
                native_selection_sha256: "b".repeat(64),
                requested_selection_sha256: "c".repeat(64),
            },
            runtime: OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                orchestration_session_id: activation.orchestration_session_id.clone(),
                caller_participant_id: activation.caller_participant_id.clone(),
                participant_id: activation.retained_participant_id.clone(),
                backend_id: activation.target_backend_id.clone(),
                run_id: activation.bootstrap_run_id.clone(),
                world_id: activation.target_world.world_id.clone(),
                world_generation: activation.target_world.world_generation,
            },
            activation,
            native_session_id: "actual-native-session-locator".into(),
            span_id: "native-bootstrap-span".into(),
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: 1,
                stream_id: "rts_bootstrap".into(),
                frame_sequence: 5,
            },
            terminal_identity: RuntimeTerminalIdentityV1::from(&event_identity),
            event_identity,
            native_settlement: NativeSettlementAcknowledgmentV1::SdkChecked,
            native_termination: NativeProcessTerminationV1::Exited { exit_code: 0 },
            gateway_disposal: OrdinaryGatewayDisposalV1::ProcessWaited {
                runtime_id: "issued-gateway-instance".into(),
                pid: 12345,
                termination: NativeProcessTerminationV1::Signaled { signal: 15 },
            },
        }
    }

    #[test]
    fn test_retained_bootstrap_completion_round_trip_and_exact_exit_join() {
        let observed = retained_bootstrap_observation_fixture();
        let receipt = RetainedBootstrapCompletionV1::from_observation(observed.clone()).unwrap();
        receipt
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                0,
            )
            .unwrap();
        let encoded = serde_json::to_vec(&receipt).unwrap();
        let round_trip: RetainedBootstrapCompletionV1 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(round_trip, receipt);
        assert_eq!(receipt.observation(), &observed);
        let value = serde_json::to_value(&receipt).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["domain"], RetainedBootstrapCompletionV1::DOMAIN);
        assert_eq!(value["sha256"].as_str().unwrap().len(), 64);
        let mut wrong_frame = observed.frame_identity.clone();
        wrong_frame.frame_sequence += 1;
        assert!(receipt
            .validate_for_exit(
                &wrong_frame,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                0
            )
            .is_err());
        assert!(receipt
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                "other-span",
                0
            )
            .is_err());
        assert!(receipt
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                1
            )
            .is_err());
    }

    #[test]
    fn test_retained_bootstrap_completion_refuses_tampering_and_cross_source() {
        let observed = retained_bootstrap_observation_fixture();
        let receipt = RetainedBootstrapCompletionV1::from_observation(observed.clone()).unwrap();
        let original = serde_json::to_value(&receipt).unwrap();
        for defect in [
            "version", "domain", "digest", "missing", "unknown", "nested",
        ] {
            let mut value = original.clone();
            match defect {
                "version" => value["schema_version"] = 2.into(),
                "domain" => value["domain"] = "other-domain".into(),
                "digest" => value["sha256"] = "0".repeat(64).into(),
                "missing" => {
                    value.as_object_mut().unwrap().remove("sha256");
                }
                "unknown" => value["extra"] = true.into(),
                "nested" => value["observation"]["extra"] = true.into(),
                _ => unreachable!(),
            }
            assert!(
                serde_json::from_value::<RetainedBootstrapCompletionV1>(value).is_err(),
                "{defect}"
            );
        }
        let mut wrong = observed.clone();
        wrong.source_start.original_session_id = "other-session".into();
        assert!(RetainedBootstrapCompletionV1::from_observation(wrong).is_err());
        let mut wrong = observed.clone();
        if let OrdinaryGatewayRuntimeIdentityV1::WorldWorker { participant_id, .. } =
            &mut wrong.runtime
        {
            *participant_id = "other-worker".into();
        }
        assert!(RetainedBootstrapCompletionV1::from_observation(wrong).is_err());
        let mut wrong = observed.clone();
        wrong.native_session_id.clear();
        assert!(RetainedBootstrapCompletionV1::from_observation(wrong).is_err());
        let mut wrong = observed;
        wrong.native_termination = NativeProcessTerminationV1::Exited { exit_code: 1 };
        assert!(RetainedBootstrapCompletionV1::from_observation(wrong).is_err());
    }

    #[test]
    fn test_retained_bootstrap_service_event_boundary() {
        let observed = retained_bootstrap_observation_fixture();
        let report = RetainedBootstrapCompletionV1::from_observation(observed.clone()).unwrap();
        let mut event = AgentEvent::message(
            "telemetry-producer",
            observed.activation.orchestration_session_id.clone(),
            observed.activation.bootstrap_run_id.clone(),
            substrate_common::agent_events::MessageEventKind::Status,
            "completed",
        );
        event.participant_id = Some(observed.activation.retained_participant_id.clone());
        event.backend_id = Some(observed.activation.target_backend_id.clone());
        event.world_id = Some(observed.activation.target_world.world_id.clone());
        event.world_generation = Some(observed.activation.target_world.world_generation);
        event.role = Some("member".into());
        event.span_id = Some(observed.span_id.clone());
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: "announcement".into(),
            event_sequence: 2,
        });
        let frame = RuntimeFrameIdentityV1 {
            schema_version: 1,
            stream_id: observed.frame_identity.stream_id.clone(),
            frame_sequence: 4,
        };
        assert_eq!(
            RetainedBootstrapCompletionV1::from_service_event(&frame, &event).unwrap(),
            None
        );
        event.data = serde_json::json!({(RetainedBootstrapCompletionV1::EVENT_DATA_KEY): report});
        assert!(
            RetainedBootstrapCompletionV1::from_service_event(&frame, &event)
                .unwrap()
                .is_some()
        );
        for defect in [
            "unknown",
            "kind",
            "worker",
            "world",
            "span",
            "sequence",
            "parent",
            "malformed",
        ] {
            let mut wrong = event.clone();
            match defect {
                "unknown" => wrong.data["extra"] = true.into(),
                "kind" => wrong.kind = substrate_common::agent_events::AgentEventKind::Registered,
                "worker" => wrong.participant_id = Some("other-worker".into()),
                "world" => {
                    wrong.world_generation =
                        Some(observed.activation.target_world.world_generation + 1)
                }
                "span" => wrong.span_id = Some("other-span".into()),
                "sequence" => {
                    wrong.event_identity.as_mut().unwrap().event_sequence =
                        observed.event_identity.event_sequence
                }
                "parent" => wrong.parent_participant_id = Some("other-worker".into()),
                "malformed" => {
                    wrong.data[RetainedBootstrapCompletionV1::EVENT_DATA_KEY]["schema_version"] =
                        2.into()
                }
                _ => unreachable!(),
            }
            assert!(
                RetainedBootstrapCompletionV1::from_service_event(&frame, &wrong).is_err(),
                "{defect}"
            );
        }
    }

    // Codec-only Fork provenance; no issued runtime or custody claim.
    fn fork_bootstrap_codec_fixture() -> (
        RetainedBootstrapCompletionV1,
        OrdinaryForkBootstrapProvenanceV1,
    ) {
        let source = retained_bootstrap_observation_fixture();
        let mut child = source.clone();
        child.activation.launch_kind = E2MemberLaunchKindV1::Fork;
        child.activation.reservation_ref = None;
        child.activation.request_commitment = E2LaunchRequestCommitmentV1::CanonicalSha256 {
            domain: "substrate.test.fork.request.v1".into(),
            digest_hex: "f".repeat(64),
        };
        child.activation.source_participant_id =
            Some(source.activation.retained_participant_id.clone());
        child.activation.retained_participant_id = "rwp_child".into();
        child.activation.bootstrap_run_id = "rwr_child".into();
        child.activation.activation_id = format!("e2a_{}", "e".repeat(32));
        child.activation.commitment_ref.commitment_id =
            "dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab8".into();
        child.activation.immutable_worker_cap_ref = child.activation.commitment_ref.clone();
        // E2 revisions belong to each immutable object, not a global sequence.
        assert_eq!(
            child.activation.immutable_worker_cap_created_revision,
            source.activation.immutable_worker_cap_created_revision
        );
        child.runtime = OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: child.activation.orchestration_session_id.clone(),
            caller_participant_id: child.activation.caller_participant_id.clone(),
            participant_id: child.activation.retained_participant_id.clone(),
            backend_id: child.activation.target_backend_id.clone(),
            run_id: child.activation.bootstrap_run_id.clone(),
            world_id: child.activation.target_world.world_id.clone(),
            world_generation: child.activation.target_world.world_generation,
        };
        child.native_session_id = "actual-native-child".into();
        let provenance = OrdinaryForkBootstrapProvenanceV1 {
            source_runtime: source.runtime,
            source_activation: source.activation,
            source_native_session_id: source.native_session_id,
            transferred_child_native_session_id: child.native_session_id.clone(),
        };
        (
            RetainedBootstrapCompletionV1::from_observation(child).unwrap(),
            provenance,
        )
    }

    #[test]
    fn test_fork_protocol_success_preserves_actual_signaled_process_wait() {
        let (child, provenance) = fork_bootstrap_codec_fixture();
        let mut observed = child.observation().clone();
        observed.native_termination = NativeProcessTerminationV1::Signaled { signal: 9 };
        // Synthetic codec data only, not a native service or custody proof.
        let waits = vec![NativeProcessWaitObservationV1 {
            pid: 123,
            termination: observed.native_termination.clone(),
            recovery_termination: None,
        }];
        assert!(RetainedBootstrapCompletionV1::from_observation(observed.clone()).is_err());
        let report = RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            provenance.clone(),
            waits.clone(),
        )
        .unwrap();
        report
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                0,
            )
            .unwrap();
        assert_eq!(
            report.observation().native_termination,
            observed.native_termination
        );
        assert_eq!(report.native_processes(), waits);
        let encoded = serde_json::to_value(&report).unwrap();
        assert_eq!(
            serde_json::from_value::<RetainedForkBootstrapCompletionV1>(encoded).unwrap(),
            report
        );
        assert!(RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            provenance.clone(),
            Vec::new()
        )
        .is_err());
        let mut wrong = waits;
        wrong[0].termination = NativeProcessTerminationV1::Exited { exit_code: 0 };
        assert!(
            RetainedForkBootstrapCompletionV1::from_observation(observed, provenance, wrong)
                .is_err()
        );
    }

    #[test]
    fn test_fork_bootstrap_provenance_closed_codec_and_native_join() {
        let (child, provenance) = fork_bootstrap_codec_fixture();
        let observed = child.observation().clone();
        let waits = vec![NativeProcessWaitObservationV1 {
            pid: 123,
            termination: observed.native_termination.clone(),
            recovery_termination: None,
        }];
        let report = RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            provenance.clone(),
            waits.clone(),
        )
        .unwrap();
        let encoded = serde_json::to_value(&report).unwrap();
        assert_eq!(
            serde_json::from_value::<RetainedForkBootstrapCompletionV1>(encoded.clone()).unwrap(),
            report
        );
        for pointer in [
            "/schema_version",
            "/domain",
            "/sha256",
            "/provenance/source_native_session_id",
            "/provenance/transferred_child_native_session_id",
        ] {
            let mut wrong = encoded.clone();
            *wrong.pointer_mut(pointer).unwrap() = serde_json::json!("changed");
            assert!(
                serde_json::from_value::<RetainedForkBootstrapCompletionV1>(wrong).is_err(),
                "{pointer}"
            );
        }
        let mut unknown = encoded.clone();
        unknown["extra"] = true.into();
        assert!(serde_json::from_value::<RetainedForkBootstrapCompletionV1>(unknown).is_err());
        let mut unknown = encoded;
        unknown["provenance"]["extra"] = true.into();
        assert!(serde_json::from_value::<RetainedForkBootstrapCompletionV1>(unknown).is_err());
        let mut wrong = provenance.clone();
        wrong.transferred_child_native_session_id = wrong.source_native_session_id.clone();
        assert!(RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            wrong,
            waits.clone()
        )
        .is_err());
        let mut wrong = provenance.clone();
        wrong.source_activation.retained_participant_id = "other-source".into();
        assert!(RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            wrong,
            waits.clone()
        )
        .is_err());
        let mut wrong = provenance;
        wrong.source_native_session_id.clear();
        assert!(RetainedForkBootstrapCompletionV1::from_observation(
            observed,
            wrong,
            waits.clone()
        )
        .is_err());
        assert!(RetainedForkBootstrapCompletionV1::from_observation(
            retained_bootstrap_observation_fixture(),
            fork_bootstrap_codec_fixture().1,
            waits
        )
        .is_err());
    }

    #[test]
    fn test_fork_bootstrap_service_event_preserves_parent_and_exit() {
        let (child, provenance) = fork_bootstrap_codec_fixture();
        let report = RetainedForkBootstrapCompletionV1::from_observation(
            child.observation().clone(),
            provenance,
            vec![NativeProcessWaitObservationV1 {
                pid: 123,
                termination: child.observation().native_termination.clone(),
                recovery_termination: None,
            }],
        )
        .unwrap();
        let observed = child.observation();
        let mut event = AgentEvent::message(
            "producer",
            observed.activation.orchestration_session_id.clone(),
            observed.activation.bootstrap_run_id.clone(),
            substrate_common::agent_events::MessageEventKind::Status,
            "completed",
        );
        event.participant_id = Some(observed.activation.retained_participant_id.clone());
        event.backend_id = Some(observed.activation.target_backend_id.clone());
        event.world_id = Some(observed.activation.target_world.world_id.clone());
        event.world_generation = Some(observed.activation.target_world.world_generation);
        event.role = Some("member".into());
        event.parent_participant_id = observed.activation.source_participant_id.clone();
        event.span_id = Some(observed.span_id.clone());
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: "announcement".into(),
            event_sequence: 2,
        });
        let frame = RuntimeFrameIdentityV1 {
            schema_version: 1,
            stream_id: observed.frame_identity.stream_id.clone(),
            frame_sequence: 4,
        };
        event.data =
            serde_json::json!({(RetainedForkBootstrapCompletionV1::EVENT_DATA_KEY): report});
        assert_eq!(
            RetainedForkBootstrapCompletionV1::from_service_event(&frame, &event).unwrap(),
            Some(report.clone())
        );
        assert!(
            RetainedBootstrapCompletionV1::from_service_event(&frame, &event)
                .unwrap()
                .is_none()
        );
        report
            .completion()
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                0,
            )
            .unwrap();
        assert!(report
            .completion()
            .validate_for_exit(
                &observed.frame_identity,
                &observed.event_identity,
                &observed.terminal_identity,
                &observed.span_id,
                1
            )
            .is_err());
        for defect in [
            "parent",
            "span",
            "worker",
            "sequence",
            "extra",
            "dual_marker",
        ] {
            let mut wrong = event.clone();
            match defect {
                "parent" => wrong.parent_participant_id = None,
                "span" => wrong.span_id = Some("other-span".into()),
                "worker" => wrong.participant_id = Some("other-worker".into()),
                "sequence" => {
                    wrong.event_identity.as_mut().unwrap().event_sequence =
                        observed.event_identity.event_sequence
                }
                "extra" => wrong.data["extra"] = true.into(),
                "dual_marker" => {
                    wrong.data[RetainedBootstrapCompletionV1::EVENT_DATA_KEY] =
                        serde_json::to_value(&child).unwrap()
                }
                _ => unreachable!(),
            }
            assert!(
                RetainedForkBootstrapCompletionV1::from_service_event(&frame, &wrong).is_err(),
                "{defect}"
            );
        }
    }

    #[test]
    fn test_unannotated_exit_keeps_original_wire_form() {
        let frame: ExecuteStreamFrame = serde_json::from_value(serde_json::json!({
            "type":"exit", "frame_identity":{"schema_version":1,"stream_id":"rts_old","frame_sequence":2},
            "event_identity":{"event_id":"old-event","event_sequence":1},
            "terminal_identity":{"terminal_event_id":"old-event","terminal_event_sequence":1},
            "exit":0,"span_id":"old-span","scopes_used":[],
            "process_events_status":"unavailable", "process_events":[]
        })).unwrap();
        frame.validate_identity_contract().unwrap();
        let value = serde_json::to_value(&frame).unwrap();
        assert!(value.get("retained_bootstrap_completion").is_none());
    }

    fn e3a_sample_fresh_spawn_activation() -> E2MemberLaunchActivationCarrierV1 {
        let snapshot = e2_sample_policy_snapshot();
        let bytes = serde_json::to_vec(&snapshot).expect("serialize E3-A E2 snapshot");
        let immutable_worker_cap_ref = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e2".to_string(),
            commitment_id: "dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab7".to_string(),
            exact_linkage_hash: "6".repeat(64),
        };
        E2MemberLaunchActivationCarrierV1 {
            schema_version: 1,
            activation_id: format!("e2a_{}", "6".repeat(32)),
            launch_kind: E2MemberLaunchKindV1::FreshSpawn,
            reservation_ref: Some(E2DispatchPolicyReservationRefCarrierV1 {
                authority_store_id: "authority-store-e2".to_string(),
                reservation_id: "reservation-e3a".to_string(),
                reservation_hash: "5".repeat(64),
            }),
            commitment_ref: immutable_worker_cap_ref.clone(),
            immutable_worker_cap_ref,
            immutable_worker_cap_created_revision: 1,
            immutable_worker_cap_application_revision: 2,
            policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            policy_snapshot_byte_length: bytes.len() as u64,
            policy_snapshot_ref: AuthorityObjectRefV1 {
                ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                object_kind: AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "8".repeat(64),
                },
            },
            policy_snapshot_hash: format!("{:x}", Sha256::digest(&bytes)),
            policy_snapshot_revision: "policy-revision-e3a".to_string(),
            reason: Some("strict E3-A transport".to_string()),
            request_id: "request_123".to_string(),
            idempotency_key: "idempotency-e3a".to_string(),
            orchestration_session_id: "orch_123".to_string(),
            caller_participant_id: "ash_orch_123".to_string(),
            caller_backend_id: "cli:codex".to_string(),
            target_backend_id: "cli:codex".to_string(),
            retained_participant_id: "ash_member_123".to_string(),
            bootstrap_run_id: "run_123".to_string(),
            source_participant_id: None,
            target_world: WorldBindingRefV1 {
                world_id: "world_123".to_string(),
                world_generation: 7,
            },
            parent_policy_ref: AuthorityObjectRefV1 {
                ref_id: "ao_fedcba9876543210fedcba9876543210".to_string(),
                object_kind: AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "9".repeat(64),
                },
            },
            parent_policy_revision: "parent-policy-revision-e3a".to_string(),
            request_commitment: E2LaunchRequestCommitmentV1::HmacSha256 {
                key_id: "request-key-e3a".to_string(),
                domain: "substrate.e3a.test".to_string(),
                digest_hex: "a".repeat(64),
            },
            registry_publication_revision: 2,
        }
    }

    fn e3a_sample_member_dispatch_v2() -> MemberDispatchRequestV2 {
        MemberDispatchRequestV2 {
            schema_version: 2,
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member_123".to_string(),
            orchestrator_participant_id: "ash_orch_123".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            initial_prompt: Some("strict V2 launch".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: test_absolute_binary_path(),
            },
            retained_worker_launch_authority: Some(sample_retained_worker_launch_authority_proof()),
            e2_launch_activation: Some(e3a_sample_fresh_spawn_activation()),
            config_projection: e3a_sample_projection_carrier(),
        }
    }

    #[test]
    fn e3a_member_dispatch_v2_round_trips_without_downgrade() {
        let request = e3a_sample_member_dispatch_v2();
        request.validate().expect("valid strict V2 request");
        let wrapped = MemberDispatchRequest::V2(request.clone());
        let bytes = serde_json::to_vec(&wrapped).expect("serialize strict V2 request");
        let decoded: MemberDispatchRequest =
            serde_json::from_slice(&bytes).expect("decode strict V2 request");

        assert_eq!(decoded, wrapped);
        assert!(decoded.as_v1().is_none());
        assert!(decoded.e2_launch_activation().is_some());
        assert_eq!(
            decoded.config_projection(),
            Some(&request.config_projection)
        );
        assert_eq!(decoded.common().participant_id, "ash_member_123");
    }

    #[test]
    fn e3a_member_dispatch_discriminator_is_strict_and_v1_default_is_unchanged() {
        let wrapped = MemberDispatchRequest::V2(e3a_sample_member_dispatch_v2());
        let canonical = serde_json::to_value(&wrapped).expect("serialize V2 value");

        for mutation in [
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!("2"),
        ] {
            let mut changed = canonical.clone();
            changed["schema_version"] = mutation;
            assert!(serde_json::from_value::<MemberDispatchRequest>(changed).is_err());
        }

        let mut newer = canonical.clone();
        newer["schema_version"] = serde_json::json!(3);
        let error = serde_json::from_value::<MemberDispatchRequest>(newer)
            .expect_err("newer member-dispatch schema must be unsupported");
        assert!(error
            .to_string()
            .contains("unsupported member_dispatch.schema_version"));

        let mut unknown = canonical.clone();
        unknown["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<MemberDispatchRequest>(unknown).is_err());

        let mut missing_projection = canonical.clone();
        missing_projection
            .as_object_mut()
            .unwrap()
            .remove("config_projection");
        assert!(serde_json::from_value::<MemberDispatchRequest>(missing_projection).is_err());

        let duplicate = serde_json::to_string(&wrapped)
            .expect("serialize V2")
            .replacen('{', "{\"schema_version\":2,", 1);
        assert!(serde_json::from_str::<MemberDispatchRequest>(&duplicate).is_err());

        let mut v1 = serde_json::to_value(MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_v1".to_string(),
            participant_id: "member_v1".to_string(),
            orchestrator_participant_id: "orchestrator_v1".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run_v1".to_string(),
            world_id: "world_v1".to_string(),
            world_generation: 1,
            initial_prompt: Some("unchanged V1".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: test_absolute_binary_path(),
            },
            retained_worker_launch_authority: None,
            e2_launch_activation: None,
        })
        .expect("serialize V1");
        v1.as_object_mut().unwrap().remove("schema_version");
        let decoded: MemberDispatchRequest =
            serde_json::from_value(v1).expect("absent discriminator stays V1");
        assert!(decoded.as_v1().is_some());
        assert!(decoded.config_projection().is_none());

        let v1_bytes = serde_json::to_string(decoded.as_v1().expect("decoded V1"))
            .expect("serialize nested V1 duplicate fixture");
        let v1_nested_duplicate = v1_bytes.replacen(
            "\"resolved_runtime\":{",
            "\"resolved_runtime\":{\"backend_kind\":\"codex\",",
            1,
        );
        assert_ne!(v1_nested_duplicate, v1_bytes);
        let error = serde_json::from_str::<MemberDispatchRequest>(&v1_nested_duplicate)
            .expect_err("nested duplicate must preserve strict V1 rejection");
        assert!(error.to_string().contains("duplicate field `backend_kind`"));

        let v2_bytes =
            serde_json::to_string(&wrapped).expect("serialize nested V2 duplicate fixture");
        let v2_nested_duplicate = v2_bytes.replacen(
            "\"config_projection\":{",
            "\"config_projection\":{\"consumer_lease_revision\":1,",
            1,
        );
        assert_ne!(v2_nested_duplicate, v2_bytes);
        let error = serde_json::from_str::<MemberDispatchRequest>(&v2_nested_duplicate)
            .expect_err("nested duplicate must preserve strict V2 rejection");
        assert!(error
            .to_string()
            .contains("duplicate field `consumer_lease_revision`"));
    }

    #[test]
    fn e3a_member_dispatch_v2_rejects_malformed_projection_and_e2_binding() {
        let canonical =
            serde_json::to_value(MemberDispatchRequest::V2(e3a_sample_member_dispatch_v2()))
                .expect("serialize V2 value");

        for (pointer, replacement) in [
            (
                "/config_projection/consumer_lease_revision",
                serde_json::json!(0),
            ),
            (
                "/config_projection/consumer_lease_hash",
                serde_json::json!("A".repeat(64)),
            ),
            (
                "/config_projection/dormant_projection_ref/revision",
                serde_json::json!(0),
            ),
            (
                "/config_projection/expected_gateway_ref/authority_store_id",
                serde_json::json!("cpa_018f0f2e-7b4c-7aa1-8c22-123456789aff"),
            ),
            (
                "/e2_launch_activation/retained_participant_id",
                serde_json::json!("another_member"),
            ),
            (
                "/e2_launch_activation/activation_id",
                serde_json::json!(format!("e2a_{}", "7".repeat(32))),
            ),
            (
                "/e2_launch_activation/immutable_worker_cap_ref/commitment_id",
                serde_json::json!("dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab8"),
            ),
            ("/e2_launch_activation", serde_json::Value::Null),
            ("/retained_worker_launch_authority", serde_json::Value::Null),
        ] {
            let mut changed = canonical.clone();
            *changed.pointer_mut(pointer).expect("V2 mutation pointer") = replacement;
            assert!(
                serde_json::from_value::<MemberDispatchRequest>(changed).is_err(),
                "mutation at {pointer} must fail closed"
            );
        }

        let mut nested_unknown = canonical;
        nested_unknown["config_projection"]["expected_gateway_ref"]["unexpected"] =
            serde_json::json!(true);
        assert!(serde_json::from_value::<MemberDispatchRequest>(nested_unknown).is_err());
    }

    fn sample_world_work_acceptance_context() -> WorldWorkAcceptanceContextV1 {
        WorldWorkAcceptanceContextV1 {
            schema_version: 1,
            proposed_acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab".to_string(),
            request_id: "request-123".to_string(),
            message_id: Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string()),
            caller_backend_id: "cli:codex".to_string(),
            host_transition_correlation: None,
        }
    }

    #[test]
    fn world_work_acceptance_context_v1_round_trips_exact_schema() {
        let context = sample_world_work_acceptance_context();
        let json = serde_json::to_value(&context).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["request_id"], "request-123");
        assert_eq!(
            serde_json::from_value::<WorldWorkAcceptanceContextV1>(json).unwrap(),
            context
        );
    }

    #[test]
    fn world_work_acceptance_context_v1_rejects_malformed_or_unknown_fields() {
        let context = sample_world_work_acceptance_context();
        let mut wrong_version = serde_json::to_value(&context).unwrap();
        wrong_version["schema_version"] = serde_json::Value::from(2);
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(wrong_version).is_err());

        let mut malformed_id = serde_json::to_value(&context).unwrap();
        malformed_id["proposed_acceptance_record_id"] =
            serde_json::Value::String("wwa_not-a-uuid".to_string());
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(malformed_id).is_err());

        let mut unknown = serde_json::to_value(&context).unwrap();
        unknown["accepted"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(unknown).is_err());
    }

    #[test]
    fn optional_acceptance_context_is_omitted_for_legacy_requests() {
        let mut turn_json = serde_json::to_value(MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".into(),
            participant_id: "ash_member_123".into(),
            orchestrator_participant_id: "ash_orch_123".into(),
            backend_id: "cli:codex".into(),
            run_id: "request-123".into(),
            world_id: "world_123".into(),
            world_generation: 7,
            prompt: "resume".into(),
            policy_snapshot_carrier: None,
            acceptance_context: None,
        })
        .unwrap();
        assert!(turn_json.get("acceptance_context").is_none());
        turn_json["acceptance_context"] =
            serde_json::to_value(sample_world_work_acceptance_context()).unwrap();
        let decoded: MemberTurnSubmitRequestV1 = serde_json::from_value(turn_json).unwrap();
        assert_eq!(
            decoded.acceptance_context,
            Some(sample_world_work_acceptance_context())
        );
    }

    #[test]
    fn member_turn_submit_rejects_invalid_backend_id() {
        let err = serde_json::from_value::<MemberTurnSubmitRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "codex",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "prompt": "resume"
        }))
        .expect_err("invalid backend id should fail");

        assert!(
            err.to_string()
                .contains("invalid member_turn_submit.backend_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_turn_submit_rejects_empty_prompt() {
        let err = serde_json::from_value::<MemberTurnSubmitRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "cli:codex",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "prompt": "   "
        }))
        .expect_err("blank prompt should fail");

        assert!(
            err.to_string()
                .contains("member_turn_submit.prompt must not be empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_empty_process_exec_cmd_at_boundary() {
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "profile": null,
            "cmd": "   ",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            }
        }))
        .expect_err("empty process cmd should fail");

        assert!(
            err.to_string().contains("non-empty cmd"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_with_non_empty_cmd_at_boundary() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "echo hi",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7,
                "resolved_runtime": {
                    "backend_kind": "codex",
                    "binary_path": binary_path
                }
            }
        }))
        .expect_err("mixed cmd + member dispatch should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch requires cmd.trim().is_empty()"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_with_pty_at_boundary() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "",
            "cwd": "/tmp",
            "pty": true,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7,
                "resolved_runtime": {
                    "backend_kind": "codex",
                    "binary_path": binary_path
                }
            }
        }))
        .expect_err("pty member dispatch should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch requires pty=false"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_dispatch_rejects_self_referential_lineage() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<MemberDispatchRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_member_123",
            "backend_id": "cli:codex",
            "protocol": "substrate.agent.session",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "resolved_runtime": {
                "backend_kind": "codex",
                "binary_path": binary_path
            }
        }))
        .expect_err("self-referential lineage should fail");

        assert!(
            err.to_string()
                .contains("orchestrator_participant_id must not equal participant_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_without_resolved_runtime_at_boundary() {
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7
            }
        }))
        .expect_err("member dispatch without resolved_runtime should fail");

        assert!(
            err.to_string().contains("missing field `resolved_runtime`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_dispatch_rejects_non_absolute_resolved_runtime_binary_path() {
        let err = serde_json::from_value::<MemberDispatchRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "cli:codex",
            "protocol": "substrate.agent.session",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "resolved_runtime": {
                "backend_kind": "codex",
                "binary_path": "codex"
            }
        }))
        .expect_err("relative binary path should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch.resolved_runtime.binary_path must be an absolute path"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_cancel_response_defaults_schema_version() {
        let response: ExecuteCancelResponseV1 = serde_json::from_value(serde_json::json!({
            "delivered": true
        }))
        .expect("deserialize cancel response");

        assert_eq!(response.schema_version, 1);
        assert!(response.delivered);
    }

    #[test]
    fn policy_snapshot_v3_missing_net_allowed_defaults_to_empty() {
        let snapshot: PolicySnapshotV3 = serde_json::from_value(serde_json::json!({
            "schema_version": 3,
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "discover": { "allow_list": ["."], "deny_list": [] },
                "read": { "allow_list": ["."], "deny_list": [] },
                "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
            }
        }))
        .expect("deserialize snapshot");

        assert!(snapshot.net_allowed.is_empty());
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_trim_case_trailing_dot_and_dedupe() {
        let canonical = canonicalize_net_allowed(&[
            " GitHub.COM. ".to_string(),
            "github.com".to_string(),
            "CRATES.IO".to_string(),
            "".to_string(),
            "   ".to_string(),
            "crates.io.".to_string(),
        ]);

        assert_eq!(
            canonical,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_collapses_star_to_singleton() {
        let canonical = canonicalize_net_allowed(&[
            "github.com".to_string(),
            " * ".to_string(),
            "crates.io".to_string(),
        ]);

        assert_eq!(canonical, vec!["*".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_bracketed_ipv6() {
        let canonical = canonicalize_net_allowed(&[" [2001:DB8::1] ".to_string()]);
        assert_eq!(canonical, vec!["2001:db8::1".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_validate_does_not_reject_enforcement_only_net_allowed_shapes() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec![
                "*.example.com".to_string(),
                "https://example.com".to_string(),
            ],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        snapshot.validate().expect("snapshot validates");
    }

    #[test]
    fn policy_snapshot_v3_resolve_world_network_routing_matches_four_case_matrix() {
        let allow_all = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["*".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let restrictive = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec![" Example.COM. ".to_string(), "api.example.com".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let deny_all = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let allow_all_disabled = allow_all
            .resolve_world_network_routing(false)
            .expect("allow-all with filter off");
        assert!(!allow_all_disabled.isolate_network);
        assert!(allow_all_disabled.allowed_domains.is_empty());

        let allow_all_enabled = allow_all
            .resolve_world_network_routing(true)
            .expect("allow-all with filter on");
        assert!(!allow_all_enabled.isolate_network);
        assert!(allow_all_enabled.allowed_domains.is_empty());

        let deny_all_enabled = deny_all
            .resolve_world_network_routing(true)
            .expect("deny-all with filter on");
        assert!(deny_all_enabled.isolate_network);
        assert!(deny_all_enabled.allowed_domains.is_empty());

        let restrictive_enabled = restrictive
            .resolve_world_network_routing(true)
            .expect("restrictive with filter on");
        assert!(restrictive_enabled.isolate_network);
        assert_eq!(
            restrictive_enabled.allowed_domains,
            vec!["example.com".to_string(), "api.example.com".to_string()]
        );
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_rejects_invalid_shapes() {
        for entries in [
            vec!["*.example.com".to_string()],
            vec!["https://example.com".to_string()],
            vec!["example.com:443".to_string()],
            vec!["example.com/path".to_string()],
            vec!["example.com?query".to_string()],
            vec!["example.com#fragment".to_string()],
            vec!["bücher.example".to_string()],
        ] {
            assert!(
                validate_net_allowed_for_enforcement(&entries).is_err(),
                "expected invalid net_allowed entries to fail: {entries:?}"
            );
        }
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_accepts_punycode_and_ips() {
        validate_net_allowed_for_enforcement(&[
            "XN--BCHER-KVA.EXAMPLE.".to_string(),
            "1.2.3.4".to_string(),
            "[2001:db8::1]".to_string(),
        ])
        .expect("valid entries");
    }

    #[test]
    fn world_doctor_report_v1_schema_round_trip() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: Some("/opt/substrate".to_string()),
            host_context_commitment: Some("a".repeat(64)),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: true,
                world_netfilter_enable_present: true,
                last_failure_reason: Some(
                    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
                        .to_string(),
                ),
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let back: super::WorldDoctorReportV1 =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(back.schema_version, report.schema_version);
        assert_eq!(back.ok, report.ok);
        assert_eq!(back.collected_at_utc, report.collected_at_utc);
        assert_eq!(back.selected_host_prefix, report.selected_host_prefix);
        assert_eq!(back.host_context_commitment, report.host_context_commitment);
        assert_eq!(
            back.policy_snapshot_v1_supported,
            report.policy_snapshot_v1_supported
        );
        assert_eq!(back.policy_resolution_mode, report.policy_resolution_mode);
        assert_eq!(back.netfilter_status, report.netfilter_status);
        assert_eq!(back.landlock.supported, report.landlock.supported);
        assert_eq!(back.landlock.abi, report.landlock.abi);
        assert_eq!(back.landlock.reason, report.landlock.reason);
        assert_eq!(
            back.world_fs_strategy.primary,
            report.world_fs_strategy.primary
        );
        assert_eq!(
            back.world_fs_strategy.fallback,
            report.world_fs_strategy.fallback
        );
        assert_eq!(
            back.world_fs_strategy.probe.id,
            report.world_fs_strategy.probe.id
        );
        assert_eq!(
            back.world_fs_strategy.probe.probe_file,
            report.world_fs_strategy.probe.probe_file
        );
        assert_eq!(
            back.world_fs_strategy.probe.result,
            report.world_fs_strategy.probe.result
        );
        assert_eq!(
            back.world_fs_strategy.probe.failure_reason,
            report.world_fs_strategy.probe.failure_reason
        );
    }

    #[test]
    fn world_doctor_report_v1_serializes_null_last_failure_reason_when_absent() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: None,
            host_context_commitment: None,
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let value = serde_json::to_value(&report).expect("serialize report");
        assert_eq!(
            value["netfilter_status"]["last_failure_reason"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn world_doctor_report_v1_serializes_exact_netfilter_status_field_names() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: None,
            host_context_commitment: None,
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let value = serde_json::to_value(&report).expect("serialize report");
        let netfilter_status = value["netfilter_status"]
            .as_object()
            .expect("netfilter_status should serialize as object");
        assert_eq!(netfilter_status.len(), 4);
        assert!(netfilter_status.contains_key("requested"));
        assert!(netfilter_status.contains_key("enabled"));
        assert!(netfilter_status.contains_key("world_netfilter_enable_present"));
        assert!(netfilter_status.contains_key("last_failure_reason"));
    }

    #[test]
    fn world_doctor_report_v1_defaults_snapshot_fields_when_missing() {
        // Legacy world-services may omit snapshot fields; the client schema must default safely.
        let json = r#"{
            "schema_version": 1,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "landlock": { "supported": true, "abi": 3, "reason": null },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        }"#;

        let report: super::WorldDoctorReportV1 = serde_json::from_str(json).expect("deserialize");
        assert!(report.ok);
        assert!(!report.policy_snapshot_v1_supported);
        assert!(report.policy_resolution_mode.is_none());
        assert!(report.netfilter_status.is_none());
        assert!(report.selected_host_prefix.is_none());
        assert!(report.host_context_commitment.is_none());
    }

    #[test]
    fn world_doctor_report_v1_defaults_last_failure_reason_when_missing() {
        let json = r#"{
            "schema_version": 2,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "policy_snapshot_v1_supported": true,
            "policy_resolution_mode": "snapshot_v3",
            "netfilter_status": {
                "requested": true,
                "enabled": false,
                "world_netfilter_enable_present": false
            },
            "landlock": { "supported": true, "abi": 3, "reason": null },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        }"#;

        let report: super::WorldDoctorReportV1 = serde_json::from_str(json).expect("deserialize");
        let status = report.netfilter_status.expect("netfilter status");
        assert!(status.requested);
        assert!(!status.enabled);
        assert!(!status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }

    #[test]
    fn execute_stream_replay_request_round_trips_only_exact_identity() {
        let request = super::ExecuteStreamReplayRequestV1 {
            schema_version: 1,
            acceptance_record_id: "wwa_exact-replay".to_string(),
            stream_id: "rts_exact-replay".to_string(),
            after_frame_sequence: 17,
        };
        request.validate().expect("validate exact replay request");
        let bytes = serde_json::to_vec(&request).expect("serialize exact replay request");
        let decoded: super::ExecuteStreamReplayRequestV1 =
            serde_json::from_slice(&bytes).expect("decode exact replay request");
        assert_eq!(decoded, request);

        for invalid in [
            super::ExecuteStreamReplayRequestV1 {
                schema_version: 2,
                ..request.clone()
            },
            super::ExecuteStreamReplayRequestV1 {
                acceptance_record_id: "session-only".to_string(),
                ..request.clone()
            },
            super::ExecuteStreamReplayRequestV1 {
                stream_id: "rts_other ".to_string(),
                ..request
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }

    fn e3e_preparation_fixture() -> E3ConfigProjectionPrepareRequestV1 {
        let mut dispatch = e3a_sample_member_dispatch_v2();
        dispatch.backend_id = "cli:codex-world".into();
        dispatch
            .e2_launch_activation
            .as_mut()
            .unwrap()
            .target_backend_id = dispatch.backend_id.clone();
        dispatch
            .retained_worker_launch_authority
            .as_mut()
            .unwrap()
            .backend_id = dispatch.backend_id.clone();
        let mut input = ConfigProjectionAuthoringInputRefV1 {
            authority_store_id: dispatch.config_projection.authority_store_id.clone(),
            effective_config_source_hash: "a".repeat(64),
            agent_inventory_source_hash: "b".repeat(64),
            runtime_artifact_manifest_id: "ram_01900000-0000-7000-8000-000000000001".into(),
            runtime_artifact_manifest_revision: 1,
            runtime_artifact_manifest_hash: "c".repeat(64),
            input_ref_hash: String::new(),
        };
        input.input_ref_hash = input.canonical_hash().unwrap();
        let mut request = E3ConfigProjectionPrepareRequestV1 {
            schema_version: 1,
            preparation_id: "e3p_01900000-0000-7000-8000-000000000001".into(),
            preparation_idempotency_key: String::new(),
            orchestration_session_id: dispatch.orchestration_session_id,
            participant_id: dispatch.participant_id,
            orchestrator_participant_id: dispatch.orchestrator_participant_id,
            parent_participant_id: dispatch.parent_participant_id,
            resumed_from_participant_id: dispatch.resumed_from_participant_id,
            backend_id: dispatch.backend_id,
            protocol: dispatch.protocol,
            run_id: dispatch.run_id,
            world_id: dispatch.world_id,
            world_generation: dispatch.world_generation,
            resolved_runtime: dispatch.resolved_runtime,
            retained_worker_launch_authority: dispatch.retained_worker_launch_authority,
            e2_launch_activation: dispatch.e2_launch_activation.unwrap(),
            authoring_input_ref: input,
            integrated_auth: GatewayIntegratedAuthPayloadV1 {
                backend_id: "cli:codex-world".into(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    access_token: "synthetic-first-value".into(),
                    account_id: None,
                }),
                api_env: None,
            },
        };
        request.preparation_idempotency_key = request.canonical_idempotency_key().unwrap();
        request
    }

    #[test]
    fn e3e_prepare_round_trip_binds_e2_without_hashing_secret_values() {
        let mut request = e3e_preparation_fixture();
        request.validate().unwrap();
        let key = request.preparation_idempotency_key.clone();
        request
            .integrated_auth
            .cli_codex
            .as_mut()
            .unwrap()
            .access_token = "synthetic-retry-value".into();
        assert_eq!(request.canonical_idempotency_key().unwrap(), key);
        request.validate().unwrap();
        let encoded = serde_json::to_vec(&request).unwrap();
        let decoded: E3ConfigProjectionPrepareRequestV1 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.preparation_idempotency_key, key);
        request
            .integrated_auth
            .cli_codex
            .as_mut()
            .unwrap()
            .account_id = Some("synthetic-account".into());
        assert_ne!(request.canonical_idempotency_key().unwrap(), key);
        assert!(request.validate().is_err());
    }

    #[test]
    fn e3e_prepare_rejects_unknown_duplicate_and_wrong_lineage() {
        let request = e3e_preparation_fixture();
        let original = serde_json::to_value(&request).unwrap();
        for (field, value) in [
            ("unknown", serde_json::json!(true)),
            ("schema_version", serde_json::json!(2)),
            ("parent_participant_id", serde_json::json!("wrong-parent")),
            ("retained_worker_launch_authority", serde_json::Value::Null),
            ("world_generation", serde_json::json!(999)),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            assert!(serde_json::from_value::<E3ConfigProjectionPrepareRequestV1>(changed).is_err());
        }
        for field in [
            "parent_participant_id",
            "resumed_from_participant_id",
            "retained_worker_launch_authority",
        ] {
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<E3ConfigProjectionPrepareRequestV1>(missing).is_err());
        }
        let encoded = serde_json::to_string(&request).unwrap();
        let duplicate = encoded.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(serde_json::from_str::<E3ConfigProjectionPrepareRequestV1>(&duplicate).is_err());
    }

    #[test]
    fn e3e_prepare_response_requires_exact_expiry_and_integrity() {
        let request = e3e_preparation_fixture();
        let mut response = E3ConfigProjectionPrepareResponseV1 {
            schema_version: 1,
            preparation_id: request.preparation_id,
            preparation_idempotency_key: request.preparation_idempotency_key,
            config_projection: e3a_sample_projection_carrier(),
            prepared_at: "2026-09-12T12:00:00Z".into(),
            expires_at: "2026-09-12T12:02:00Z".into(),
            response_hash: String::new(),
        };
        let mut value = serde_json::to_value(&response).unwrap();
        value.as_object_mut().unwrap().remove("response_hash");
        response.response_hash = e3_projection_wire_hash(serde_json::json!({
            "domain":"substrate.e3.config-projection-prepare-response.v1", "response":value
        }))
        .unwrap();
        response.validate().unwrap();
        let encoded = serde_json::to_vec(&response).unwrap();
        assert_eq!(
            serde_json::from_slice::<E3ConfigProjectionPrepareResponseV1>(&encoded).unwrap(),
            response
        );
        response.expires_at = "2026-09-12T12:02:01Z".into();
        assert!(response.validate().is_err());
        for date in [
            "2026-02-29T12:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:00.000Z",
            "2026-01-01T00:00:00+00:00",
        ] {
            assert!(e3_projection_wire_timestamp(date).is_err());
        }
        assert_eq!(
            e3_projection_wire_timestamp("2024-02-29T00:00:00Z")
                .unwrap()
                .0
                + 86400,
            e3_projection_wire_timestamp("2024-03-01T00:00:00Z")
                .unwrap()
                .0
        );
    }
}

#[cfg(test)]
mod ordinary_selected_transport_tests {
    use super::*;
    use serde_json::{json, Value};

    fn start_reference() -> Value {
        json!({
            "schema_version": 1,
            "authority_store_id": "store_original",
            "original_start_intent_id": "intent_original",
            "original_start_issuer_request_id": format!("req_{}", "a".repeat(64)),
            "original_session_id": "session_original",
            "context_sha256": "b".repeat(64),
            "native_selection_sha256": "c".repeat(64),
            "requested_selection_sha256": "d".repeat(64)
        })
    }

    fn snapshot() -> Value {
        json!({
            "schema_version": 3,
            "net_allowed": [],
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
            }
        })
    }

    fn execute_request() -> Value {
        let binary = if cfg!(windows) {
            r"C:\Windows\System32\cmd.exe"
        } else {
            "/usr/bin/codex"
        };
        json!({
            "schema_version": 2,
            "ordinary_start_selection": start_reference(),
            "profile": null,
            "cmd": "",
            "cwd": "/project",
            "env": null,
            "pty": false,
            "agent_id": "host_actor",
            "budget": null,
            "policy_snapshot": snapshot(),
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "session_original",
                "participant_id": "world_worker",
                "orchestrator_participant_id": "host_actor",
                "backend_id": "cli:worker-world",
                "protocol": "substrate.agent.session",
                "run_id": "run_original",
                "world_id": "world_original",
                "world_generation": 7,
                "initial_prompt": "hello",
                "resolved_runtime": { "backend_kind": "codex", "binary_path": binary }
            }
        })
    }

    fn gateway_request() -> Value {
        json!({
            "schema_version": 2,
            "ordinary_selection": {
                "schema_version": 1,
                "start": start_reference(),
                "runtime": {
                    "selection_role": "world_worker",
                    "orchestration_session_id": "session_original",
                    "caller_participant_id": "host_actor",
                    "participant_id": "world_worker",
                    "backend_id": "cli:worker-world",
                    "run_id": "run_original",
                    "world_id": "world_original",
                    "world_generation": 7
                }
            },
            "profile": null,
            "cwd": "/project",
            "env": null,
            "agent_id": "host_actor",
            "policy_snapshot": snapshot(),
            "identity_tuple": {
                "client": "codex",
                "router": "gateway",
                "protocol": "openai.responses",
                "provider": "openai",
                "auth_authority": "subscription"
            },
            "placement_posture": { "execution": "in_world" }
        })
    }

    fn host_gateway_request() -> Value {
        let mut request = gateway_request();
        request["ordinary_selection"]["runtime"] = json!({
            "selection_role": "host_actor",
            "orchestration_session_id": "session_original",
            "participant_id": "host_actor",
            "backend_id": "cli:orchestrator-host",
            "run_id": "host_run_original"
        });
        request
    }

    #[test]
    fn test_selected_gateway_represents_actual_host_and_world_roles_separately() {
        let host: GatewayLifecycleRequestV1 =
            serde_json::from_value(host_gateway_request()).unwrap();
        let context = host.ordinary_selection.as_ref().unwrap();
        assert!(matches!(
            &context.runtime,
            OrdinaryGatewayRuntimeIdentityV1::HostActor {
                orchestration_session_id,
                participant_id,
                backend_id,
                run_id,
            } if orchestration_session_id == "session_original"
                && participant_id == "host_actor"
                && backend_id == "cli:orchestrator-host"
                && run_id == "host_run_original"
        ));
        let bytes = serde_json::to_vec(&host).unwrap();
        let recovered: GatewayLifecycleRequestV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(host.ordinary_selection, recovered.ordinary_selection);
        let host_shape = serde_json::to_value(&context.runtime).unwrap();
        assert!(host_shape.get("world_id").is_none());
        assert!(host_shape.get("world_generation").is_none());
        let worker: GatewayLifecycleRequestV1 = serde_json::from_value(gateway_request()).unwrap();
        assert!(matches!(
            worker.ordinary_selection.unwrap().runtime,
            OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                world_generation: 7,
                ..
            }
        ));
    }

    #[test]
    fn test_gateway_role_shape_rejects_missing_unknown_duplicate_and_cross_role_fields() {
        for fixture in [host_gateway_request(), gateway_request()] {
            let runtime = fixture["ordinary_selection"]["runtime"].clone();
            for key in runtime.as_object().unwrap().keys() {
                let mut missing = fixture.clone();
                missing["ordinary_selection"]["runtime"]
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
                assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(missing).is_err());
            }
            let mut unknown = fixture.clone();
            unknown["ordinary_selection"]["runtime"]["selection_role"] = json!("future_role");
            assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(unknown).is_err());
            let encoded = serde_json::to_string(&runtime).unwrap();
            let duplicate = format!(
                "{{\"selection_role\":{},{}",
                serde_json::to_string(&runtime["selection_role"]).unwrap(),
                &encoded[1..]
            );
            assert!(serde_json::from_str::<OrdinaryGatewayRuntimeIdentityV1>(&duplicate).is_err());
        }
        let mut invented_host_world = host_gateway_request();
        invented_host_world["ordinary_selection"]["runtime"]["world_id"] = json!("fake_world");
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(invented_host_world).is_err());
        let mut relabelled_worker = gateway_request();
        relabelled_worker["ordinary_selection"]["runtime"]["selection_role"] = json!("host_actor");
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(relabelled_worker).is_err());
        for version in [1, 3] {
            let mut wrong_version = host_gateway_request();
            wrong_version["schema_version"] = json!(version);
            assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(wrong_version).is_err());
        }
        let mut missing_host_selection = host_gateway_request();
        missing_host_selection
            .as_object_mut()
            .unwrap()
            .remove("ordinary_selection");
        assert!(
            serde_json::from_value::<GatewayLifecycleRequestV1>(missing_host_selection).is_err()
        );
    }

    #[test]
    fn test_selected_execute_codec_preserves_both_roles_and_frozen_member_bytes() {
        let input = execute_request();
        let frozen_member =
            serde_json::from_value::<MemberDispatchRequest>(input["member_dispatch"].clone())
                .expect("ordinary V1 member fixture");
        let frozen_bytes = serde_json::to_vec(&frozen_member).unwrap();
        let request: ExecuteRequest = serde_json::from_value(input).unwrap();
        let first = serde_json::to_vec(&request).unwrap();
        let decoded: ExecuteRequest = serde_json::from_slice(&first).unwrap();
        assert_eq!(first, serde_json::to_vec(&decoded).unwrap());
        assert_eq!(
            request.ordinary_start_selection,
            decoded.ordinary_start_selection
        );
        let reference = decoded.ordinary_start_selection.as_ref().unwrap();
        assert_eq!(reference.native_selection_sha256, "c".repeat(64));
        assert_eq!(reference.requested_selection_sha256, "d".repeat(64));
        assert_eq!(
            frozen_bytes,
            serde_json::to_vec(decoded.member_dispatch.as_ref().unwrap()).unwrap()
        );
        assert!(decoded.member_dispatch.unwrap().as_v1().is_some());
    }

    #[test]
    fn test_start_reference_rejects_missing_unknown_duplicate_and_future_fields() {
        let valid = start_reference();
        for key in valid.as_object().unwrap().keys() {
            let mut missing = valid.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<OrdinaryStartSelectionCarrierV1>(missing).is_err());
        }
        let mut unknown = valid.clone();
        unknown["accepted_home"] = json!("/request-chosen-home");
        assert!(serde_json::from_value::<OrdinaryStartSelectionCarrierV1>(unknown).is_err());
        let mut future = valid.clone();
        future["schema_version"] = json!(2);
        assert!(serde_json::from_value::<OrdinaryStartSelectionCarrierV1>(future).is_err());
        let encoded = serde_json::to_string(&valid).unwrap();
        let duplicate = format!("{{\"schema_version\":1,{}", &encoded[1..]);
        assert!(serde_json::from_str::<OrdinaryStartSelectionCarrierV1>(&duplicate).is_err());
    }

    #[test]
    fn test_start_reference_rejects_malformed_issuer_and_role_digests() {
        for key in [
            "original_start_issuer_request_id",
            "context_sha256",
            "native_selection_sha256",
            "requested_selection_sha256",
        ] {
            for invalid in ["", "a", &"A".repeat(64), &"g".repeat(64)] {
                let mut input = start_reference();
                input[key] = json!(invalid);
                assert!(serde_json::from_value::<OrdinaryStartSelectionCarrierV1>(input).is_err());
            }
        }
    }

    #[test]
    fn test_execute_request_version_and_selected_context_are_inseparable() {
        for version in [0, 1, 3] {
            let mut input = execute_request();
            input["schema_version"] = json!(version);
            assert!(serde_json::from_value::<ExecuteRequest>(input).is_err());
        }
        for key in ["ordinary_start_selection", "member_dispatch"] {
            let mut input = execute_request();
            input.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<ExecuteRequest>(input).is_err());
        }
        let mut process = execute_request();
        process["cmd"] = json!("true");
        assert!(serde_json::from_value::<ExecuteRequest>(process).is_err());
        let mut pty = execute_request();
        pty["pty"] = json!(true);
        assert!(serde_json::from_value::<ExecuteRequest>(pty).is_err());
        let encoded = serde_json::to_string(&execute_request()).unwrap();
        let duplicate = format!("{{\"schema_version\":2,{}", &encoded[1..]);
        assert!(serde_json::from_str::<ExecuteRequest>(&duplicate).is_err());
    }

    #[test]
    fn test_unselected_execute_keeps_absent_version_and_carrier_wire_fields() {
        let mut input = execute_request();
        input.as_object_mut().unwrap().remove("schema_version");
        input
            .as_object_mut()
            .unwrap()
            .remove("ordinary_start_selection");
        let request: ExecuteRequest = serde_json::from_value(input).unwrap();
        assert_eq!(request.schema_version, 1);
        let encoded = serde_json::to_value(request).unwrap();
        assert!(encoded.get("schema_version").is_none());
        assert!(encoded.get("ordinary_start_selection").is_none());
        assert_eq!(encoded["member_dispatch"]["schema_version"], 1);
    }

    #[test]
    fn test_selected_gateway_codec_preserves_runtime_join_and_rejects_downgrade() {
        let request: GatewayLifecycleRequestV1 = serde_json::from_value(gateway_request()).unwrap();
        let first = serde_json::to_vec(&request).unwrap();
        let decoded: GatewayLifecycleRequestV1 = serde_json::from_slice(&first).unwrap();
        assert_eq!(first, serde_json::to_vec(&decoded).unwrap());
        assert_eq!(request.ordinary_selection, decoded.ordinary_selection);
        for version in [0, 1, 3] {
            let mut input = gateway_request();
            input["schema_version"] = json!(version);
            assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(input).is_err());
        }
        let mut missing = gateway_request();
        missing
            .as_object_mut()
            .unwrap()
            .remove("ordinary_selection");
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(missing).is_err());
        let mut future = gateway_request();
        future["ordinary_selection"]["schema_version"] = json!(2);
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(future).is_err());
        let mut unknown = gateway_request();
        unknown["ordinary_selection"]["current_root_revision"] = json!(99);
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(unknown).is_err());
        let encoded = serde_json::to_string(&gateway_request()).unwrap();
        let duplicate = format!("{{\"schema_version\":2,{}", &encoded[1..]);
        assert!(serde_json::from_str::<GatewayLifecycleRequestV1>(&duplicate).is_err());
        let context = gateway_request()["ordinary_selection"].clone();
        let encoded = serde_json::to_string(&context).unwrap();
        let duplicate = format!("{{\"schema_version\":1,{}", &encoded[1..]);
        assert!(serde_json::from_str::<OrdinaryGatewaySelectionContextV1>(&duplicate).is_err());
        for key in context.as_object().unwrap().keys() {
            let mut missing = context.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<OrdinaryGatewaySelectionContextV1>(missing).is_err());
        }
        let mut wrong_auth = gateway_request();
        wrong_auth["integrated_auth"] = json!({
            "backend_id": "cli:other-world",
            "cli_codex": { "access_token": "synthetic-test-token" }
        });
        assert!(serde_json::from_value::<GatewayLifecycleRequestV1>(wrong_auth).is_err());
    }

    #[test]
    fn test_unselected_gateway_keeps_absent_version_and_context_wire_fields() {
        let mut input = gateway_request();
        input.as_object_mut().unwrap().remove("schema_version");
        input.as_object_mut().unwrap().remove("ordinary_selection");
        let request: GatewayLifecycleRequestV1 = serde_json::from_value(input).unwrap();
        assert_eq!(request.schema_version, 1);
        let encoded = serde_json::to_value(request).unwrap();
        assert!(encoded.get("schema_version").is_none());
        assert!(encoded.get("ordinary_selection").is_none());
    }
}

#[cfg(test)]
mod host_gateway_turn_tests {
    use super::*;
    use serde_json::{json, Value};

    fn request() -> Value {
        let cleanup_token = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let acquire_id =
            HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(cleanup_token).unwrap();
        json!({"schema_version": 1, "operation": {
            "kind": "acquire", "acquire_id": acquire_id, "cleanup_token": cleanup_token,
            "selection": {"schema_version": 1, "start": {
                "schema_version": 1, "authority_store_id": "store_original",
                "original_start_intent_id": "intent_original",
                "original_start_issuer_request_id": format!("req_{}", "a".repeat(64)),
                "original_session_id": "session_original", "context_sha256": "b".repeat(64),
                "native_selection_sha256": "c".repeat(64), "requested_selection_sha256": "d".repeat(64)
            }, "runtime": {"selection_role": "host_actor", "orchestration_session_id": "session_original",
                "participant_id": "host_actor", "backend_id": "cli:codex-host", "run_id": "run_current"}}
        }})
    }

    fn handle() -> String {
        format!("hgl_{}_{}", "1".repeat(32), "2".repeat(32))
    }

    fn reply() -> Value {
        json!({"schema_version": 1, "result": {"kind": "acquired",
            "acquire_id": request()["operation"]["acquire_id"], "lease_handle": handle(),
            "client_wiring": {"openai_base_url": "http://127.0.0.1:1234", "anthropic_base_url": "http://127.0.0.1:5678"}}})
    }

    #[test]
    fn test_host_gateway_turn_cleanup_commitment_and_required_null() {
        let mut value = request();
        let expected = format!(
            "hgt_{}",
            "2166971add18dd42267a763b95a2ccb412255f8c5d0c8d4614ada8d065b4811a"
        );
        assert_eq!(value["operation"]["acquire_id"], expected);
        let original: HostGatewayTurnRequestV1 = serde_json::from_value(value.clone()).unwrap();
        assert!(serde_json::from_slice::<HostGatewayTurnRequestV1>(
            &serde_json::to_vec(&original).unwrap()
        )
        .is_ok());
        for token in [
            "",
            "A",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB",
        ] {
            value["operation"]["cleanup_token"] = json!(token);
            assert!(serde_json::from_value::<HostGatewayTurnRequestV1>(value.clone()).is_err());
        }
        value = request();
        value["operation"] = json!({"kind": "release", "acquire_id": expected,
            "cleanup_token": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "lease_handle": null});
        let release: HostGatewayTurnRequestV1 = serde_json::from_value(value.clone()).unwrap();
        assert!(serde_json::to_value(release).unwrap()["operation"]
            .get("lease_handle")
            .unwrap()
            .is_null());
        value["operation"]
            .as_object_mut()
            .unwrap()
            .remove("lease_handle");
        assert!(serde_json::from_value::<HostGatewayTurnRequestV1>(value).is_err());
    }

    #[test]
    fn test_host_gateway_turn_nested_request_refusals() {
        let baseline = request();
        for path in [
            "",
            "/operation",
            "/operation/selection",
            "/operation/selection/start",
            "/operation/selection/runtime",
        ] {
            let object = baseline.pointer(path).unwrap().as_object().unwrap();
            for key in object.keys() {
                let mut missing = baseline.clone();
                missing
                    .pointer_mut(path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
                assert!(
                    serde_json::from_value::<HostGatewayTurnRequestV1>(missing).is_err(),
                    "missing {path}/{key}"
                );
            }
            let mut unknown = baseline.clone();
            unknown
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("untrusted_principal".into(), json!(0));
            assert!(
                serde_json::from_value::<HostGatewayTurnRequestV1>(unknown).is_err(),
                "unknown {path}"
            );
            let needle = serde_json::to_string(object).unwrap();
            let duplicate = format!(
                "{{\"{}\":{},{}",
                object.keys().next().unwrap(),
                serde_json::to_string(object.values().next().unwrap()).unwrap(),
                &needle[1..]
            );
            let raw = serde_json::to_string(&baseline)
                .unwrap()
                .replacen(&needle, &duplicate, 1);
            assert!(
                serde_json::from_str::<HostGatewayTurnRequestV1>(&raw).is_err(),
                "duplicate {path}"
            );
        }
        for path in [
            "/schema_version",
            "/operation/selection/schema_version",
            "/operation/selection/start/schema_version",
        ] {
            let mut wrong = baseline.clone();
            *wrong.pointer_mut(path).unwrap() = json!(2);
            assert!(serde_json::from_value::<HostGatewayTurnRequestV1>(wrong).is_err());
        }
        for (path, invalid) in [
            ("/operation/kind", json!("restart")),
            (
                "/operation/acquire_id",
                json!(format!("hgt_{}", "f".repeat(64))),
            ),
            (
                "/operation/selection/runtime/selection_role",
                json!("world_worker"),
            ),
            ("/operation/selection/runtime/run_id", Value::Null),
        ] {
            let mut wrong = baseline.clone();
            *wrong.pointer_mut(path).unwrap() = invalid;
            assert!(serde_json::from_value::<HostGatewayTurnRequestV1>(wrong).is_err());
        }
        let mut world = baseline;
        world["operation"]["selection"]["runtime"]["selection_role"] = json!("world_worker");
        world["operation"]["selection"]["runtime"]["caller_participant_id"] = json!("host_actor");
        world["operation"]["selection"]["runtime"]["world_id"] = json!("world_original");
        world["operation"]["selection"]["runtime"]["world_generation"] = json!(1);
        assert!(serde_json::from_value::<HostGatewayTurnRequestV1>(world).is_err());
    }

    #[test]
    fn test_host_gateway_turn_nested_response_and_loopback_refusals() {
        let baseline = reply();
        assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(baseline.clone()).is_ok());
        for path in ["", "/result", "/result/client_wiring"] {
            let object = baseline.pointer(path).unwrap().as_object().unwrap();
            for key in object.keys() {
                let mut missing = baseline.clone();
                missing
                    .pointer_mut(path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
                assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(missing).is_err());
            }
            let mut unknown = baseline.clone();
            unknown
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("extra".into(), json!(true));
            assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(unknown).is_err());
            let needle = serde_json::to_string(object).unwrap();
            let duplicate = format!(
                "{{\"{}\":{},{}",
                object.keys().next().unwrap(),
                serde_json::to_string(object.values().next().unwrap()).unwrap(),
                &needle[1..]
            );
            assert!(serde_json::from_str::<HostGatewayTurnResponseV1>(
                &serde_json::to_string(&baseline)
                    .unwrap()
                    .replacen(&needle, &duplicate, 1)
            )
            .is_err());
        }
        for bad in [
            "http://localhost:1234",
            "http://127.0.0.1:0",
            "http://127.0.0.1:01234",
            "http://127.0.0.1:1234/",
            "https://127.0.0.1:1234",
            "http://127.0.0.1:65536",
            "http://127.0.0.1:+1234",
        ] {
            for field in ["openai_base_url", "anthropic_base_url"] {
                let mut wrong = baseline.clone();
                wrong["result"]["client_wiring"][field] = json!(bad);
                assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(wrong).is_err());
            }
        }
        for (path, invalid) in [
            ("/schema_version", json!(0)),
            ("/result/kind", json!("available")),
            ("/result/acquire_id", json!("hgt_BAD")),
            ("/result/lease_handle", json!("/tmp/not-a-ticket")),
        ] {
            let mut wrong = baseline.clone();
            *wrong.pointer_mut(path).unwrap() = invalid;
            assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(wrong).is_err());
        }
    }

    #[test]
    fn test_host_gateway_turn_operation_result_binding() {
        let acquire: HostGatewayTurnRequestV1 = serde_json::from_value(request()).unwrap();
        let acquired: HostGatewayTurnResponseV1 = serde_json::from_value(reply()).unwrap();
        assert!(acquired.validate_for_request(&acquire).is_ok());
        let id = request()["operation"]["acquire_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let revalidate = HostGatewayTurnRequestV1 {
            schema_version: 1,
            operation: HostGatewayTurnOperationV1::Revalidate {
                acquire_id: id.clone(),
                lease_handle: handle(),
            },
        };
        let release = HostGatewayTurnRequestV1 {
            schema_version: 1,
            operation: HostGatewayTurnOperationV1::Release {
                acquire_id: id.clone(),
                lease_handle: None,
                cleanup_token: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            },
        };
        for (kind, valid) in [
            ("revalidated", &revalidate),
            ("released", &release),
            ("unknown", &release),
        ] {
            let mut value =
                json!({"schema_version": 1, "result": {"kind": kind, "acquire_id": id}});
            if kind == "revalidated" {
                value["result"]["lease_handle"] = json!(handle());
            }
            let response: HostGatewayTurnResponseV1 =
                serde_json::from_value(value.clone()).unwrap();
            assert!(response.validate_for_request(valid).is_ok());
            assert_eq!(
                response.validate_for_request(&acquire).is_ok(),
                kind == "unknown"
            );
            assert_eq!(
                response.validate_for_request(&revalidate).is_ok(),
                kind == "unknown" || kind == "revalidated"
            );
            assert_eq!(
                response.validate_for_request(&release).is_ok(),
                kind == "unknown" || kind == "released"
            );
            value["result"]["acquire_id"] = json!(format!("hgt_{}", "f".repeat(64)));
            assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(value)
                .unwrap()
                .validate_for_request(valid)
                .is_err());
        }
        let mut wrong = reply();
        wrong["result"]["acquire_id"] = json!(format!("hgt_{}", "f".repeat(64)));
        assert!(serde_json::from_value::<HostGatewayTurnResponseV1>(wrong)
            .unwrap()
            .validate_for_request(&acquire)
            .is_err());
        assert!(acquired.validate_for_request(&release).is_err());
        let swapped = HostGatewayTurnResponseV1 {
            schema_version: 1,
            result: HostGatewayTurnResultV1::Revalidated {
                acquire_id: id,
                lease_handle: format!("hgl_{}_{}", "1".repeat(32), "3".repeat(32)),
            },
        };
        assert!(swapped.validate_for_request(&revalidate).is_err());
    }
}

// Host-only V3 epoch envelope. Frozen V1 and World codecs remain unchanged.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HostGatewayInstanceRequestDef")]
/// Strict instance-bound Host request. Observation conveys no execution authority.
///
/// ```
/// use transport_api_types::HostGatewayInstanceRequestV3;
/// assert!(serde_json::from_str::<HostGatewayInstanceRequestV3>("{}").is_err());
/// ```
pub struct HostGatewayInstanceRequestV3 {
    pub schema_version: u32,
    pub operation: HostGatewayInstanceOperationV3,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
/// Observation or one operation bound to the originally observed service epoch.
pub enum HostGatewayInstanceOperationV3 {
    Observe {},
    Bound {
        service_instance_id: String,
        operation: HostGatewayTurnOperationV1,
        #[serde(deserialize_with = "host_gateway_route_verifier_field_v3")]
        route_verifier: Option<String>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostGatewayInstanceRequestDef {
    schema_version: u32,
    operation: HostGatewayInstanceOperationV3,
}
impl HostGatewayInstanceRequestV3 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 3 {
            return Err("host_gateway_instance_invalid_request".into());
        }
        if let HostGatewayInstanceOperationV3::Bound {
            service_instance_id,
            operation,
            route_verifier,
        } = &self.operation
        {
            validate_host_gateway_instance_v3(service_instance_id)?;
            match (operation, route_verifier) {
                (HostGatewayTurnOperationV1::Acquire { .. }, Some(verifier))
                    if verifier.len() == 64
                        && verifier
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) => {}
                (
                    HostGatewayTurnOperationV1::Revalidate { .. }
                    | HostGatewayTurnOperationV1::Release { .. },
                    None,
                ) => {}
                _ => return Err("host_gateway_route_verifier_invalid".into()),
            }
            HostGatewayTurnRequestV1 {
                schema_version: 1,
                operation: operation.clone(),
            }
            .validate()?;
            // Request locators cannot be rebound to another service instance.
            match operation {
                HostGatewayTurnOperationV1::Revalidate { lease_handle, .. }
                | HostGatewayTurnOperationV1::Release {
                    lease_handle: Some(lease_handle),
                    ..
                } => {
                    if !lease_handle.starts_with(&format!("hgl_{service_instance_id}_")) {
                        return Err("host_gateway_instance_wrong_binding".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
impl TryFrom<HostGatewayInstanceRequestDef> for HostGatewayInstanceRequestV3 {
    type Error = String;
    fn try_from(value: HostGatewayInstanceRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            operation: value.operation,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HostGatewayInstanceResponseDef")]
/// Strict Host instance reply. Match every bound reply against its original request.
///
/// ```
/// use transport_api_types::HostGatewayInstanceResponseV3;
/// assert!(serde_json::from_str::<HostGatewayInstanceResponseV3>("{}").is_err());
/// ```
pub struct HostGatewayInstanceResponseV3 {
    pub schema_version: u32,
    pub result: HostGatewayInstanceResultV3,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
/// Actual observed epoch or the epoch-bound strict Host-turn result.
pub enum HostGatewayInstanceResultV3 {
    Observed {
        service_instance_id: String,
    },
    Bound {
        service_instance_id: String,
        result: HostGatewayTurnResultV1,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostGatewayInstanceResponseDef {
    schema_version: u32,
    result: HostGatewayInstanceResultV3,
}
impl HostGatewayInstanceResponseV3 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 3 {
            return Err("host_gateway_instance_invalid_response".into());
        }
        let instance = match &self.result {
            HostGatewayInstanceResultV3::Observed {
                service_instance_id,
            } => service_instance_id,
            HostGatewayInstanceResultV3::Bound {
                service_instance_id,
                result,
            } => {
                HostGatewayTurnResponseV1 {
                    schema_version: 1,
                    result: result.clone(),
                }
                .validate()?;
                match result {
                    HostGatewayTurnResultV1::Acquired { lease_handle, .. }
                    | HostGatewayTurnResultV1::Revalidated { lease_handle, .. } => {
                        if !lease_handle.starts_with(&format!("hgl_{service_instance_id}_")) {
                            return Err("host_gateway_instance_wrong_binding".into());
                        }
                    }
                    _ => {}
                }
                service_instance_id
            }
        };
        validate_host_gateway_instance_v3(instance)
    }
    pub fn validate_for_request(
        &self,
        request: &HostGatewayInstanceRequestV3,
    ) -> Result<(), String> {
        self.validate()?;
        request.validate()?;
        match (&request.operation, &self.result) {
            (
                HostGatewayInstanceOperationV3::Observe {},
                HostGatewayInstanceResultV3::Observed { .. },
            ) => Ok(()),
            (
                HostGatewayInstanceOperationV3::Bound {
                    service_instance_id: expected,
                    operation,
                    ..
                },
                HostGatewayInstanceResultV3::Bound {
                    service_instance_id: actual,
                    result,
                },
            ) if expected == actual => HostGatewayTurnResponseV1 {
                schema_version: 1,
                result: result.clone(),
            }
            .validate_for_request(&HostGatewayTurnRequestV1 {
                schema_version: 1,
                operation: operation.clone(),
            }),
            _ => Err("host_gateway_instance_wrong_binding".into()),
        }
    }
}
impl TryFrom<HostGatewayInstanceResponseDef> for HostGatewayInstanceResponseV3 {
    type Error = String;
    fn try_from(value: HostGatewayInstanceResponseDef) -> Result<Self, Self::Error> {
        let response = Self {
            schema_version: value.schema_version,
            result: value.result,
        };
        response.validate()?;
        Ok(response)
    }
}
fn validate_host_gateway_instance_v3(value: &str) -> Result<(), String> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("host_gateway_instance_invalid_identity".into());
    }
    Ok(())
}

#[cfg(test)]
mod host_gateway_instance_v3_tests {
    use super::*;
    #[test]
    fn test_observation_is_closed_and_non_admitting() {
        let observe: HostGatewayInstanceRequestV3 =
            serde_json::from_str(r#"{"schema_version":3,"operation":{"kind":"observe"}}"#).unwrap();
        let response: HostGatewayInstanceResponseV3 = serde_json::from_str(
            r#"{"schema_version":3,"result":{"kind":"observed","service_instance_id":"11111111111111111111111111111111"}}"#
        ).unwrap();
        response.validate_for_request(&observe).unwrap();
        for invalid in [
            r#"{"schema_version":1,"operation":{"kind":"observe"}}"#,
            r#"{"schema_version":3,"operation":{"kind":"observe","unexpected":0}}"#,
            r#"{"schema_version":3,"schema_version":3,"operation":{"kind":"observe"}}"#,
            r#"{"schema_version":3,"operation":{"kind":"bound","operation":{"kind":"release"}}}"#,
        ] {
            assert!(serde_json::from_str::<HostGatewayInstanceRequestV3>(invalid).is_err());
        }
    }
}

#[cfg(test)]
mod host_gateway_instance_v3_ingress_tests {
    use super::*;
    use serde_json::{json, Value};
    fn release() -> Value {
        let token = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        json!({"schema_version":3,"operation":{"kind":"bound","service_instance_id":"1".repeat(32),"route_verifier":null,"operation":{
            "kind":"release","acquire_id":HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(token).unwrap(),
            "cleanup_token":token,"lease_handle":null
        }}})
    }
    #[test]
    fn test_instance_bound_request_keeps_strict_nested_v1_ingress() {
        let valid: HostGatewayInstanceRequestV3 = serde_json::from_value(release()).unwrap();
        valid.validate().unwrap();
        for path in ["", "/operation", "/operation/operation"] {
            let mut unknown = release();
            unknown
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(true));
            assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(unknown).is_err());
        }
        for (path, field) in [
            ("", "schema_version"),
            ("", "operation"),
            ("/operation", "kind"),
            ("/operation", "service_instance_id"),
            ("/operation", "operation"),
            ("/operation", "route_verifier"),
            ("/operation/operation", "kind"),
            ("/operation/operation", "acquire_id"),
            ("/operation/operation", "cleanup_token"),
            ("/operation/operation", "lease_handle"),
        ] {
            let mut missing = release();
            missing
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<HostGatewayInstanceRequestV3>(missing).is_err(),
                "{path}/{field}"
            );
        }
        let raw = serde_json::to_string(&release()).unwrap();
        for (needle, duplicate) in [
            (
                "\"schema_version\":3",
                "\"schema_version\":3,\"schema_version\":3",
            ),
            (
                "\"kind\":\"bound\"",
                "\"kind\":\"bound\",\"kind\":\"bound\"",
            ),
            (
                "\"kind\":\"release\"",
                "\"kind\":\"release\",\"kind\":\"release\"",
            ),
            (
                "\"lease_handle\":null",
                "\"lease_handle\":null,\"lease_handle\":null",
            ),
        ] {
            assert!(serde_json::from_str::<HostGatewayInstanceRequestV3>(
                &raw.replacen(needle, duplicate, 1)
            )
            .is_err());
        }
        for version in [0, 1, 2, 4] {
            let mut invalid = release();
            invalid["schema_version"] = json!(version);
            assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(invalid).is_err());
        }
        let mut changed = release();
        changed["operation"]["operation"]["lease_handle"] =
            json!(format!("hgl_{}_{}", "2".repeat(32), "3".repeat(32)));
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(changed).is_err());
    }
    #[test]
    fn test_instance_response_echo_and_operation_matching() {
        let request: HostGatewayInstanceRequestV3 = serde_json::from_value(release()).unwrap();
        let response = json!({"schema_version":3,"result":{"kind":"bound","service_instance_id":"1".repeat(32),
            "result":{"kind":"unknown","acquire_id":release()["operation"]["operation"]["acquire_id"]}}});
        let valid: HostGatewayInstanceResponseV3 =
            serde_json::from_value(response.clone()).unwrap();
        valid.validate_for_request(&request).unwrap();
        for path in ["", "/result", "/result/result"] {
            let mut unknown = response.clone();
            unknown
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(0));
            assert!(serde_json::from_value::<HostGatewayInstanceResponseV3>(unknown).is_err());
        }
        for (path, field) in [
            ("", "schema_version"),
            ("", "result"),
            ("/result", "kind"),
            ("/result", "service_instance_id"),
            ("/result", "result"),
            ("/result/result", "kind"),
            ("/result/result", "acquire_id"),
        ] {
            let mut missing = response.clone();
            missing
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(serde_json::from_value::<HostGatewayInstanceResponseV3>(missing).is_err());
        }
        let mut wrong_epoch = response.clone();
        wrong_epoch["result"]["service_instance_id"] = json!("2".repeat(32));
        let wrong: HostGatewayInstanceResponseV3 = serde_json::from_value(wrong_epoch).unwrap();
        assert!(wrong.validate_for_request(&request).is_err());
        let observed: HostGatewayInstanceResponseV3 =
            serde_json::from_value(json!({"schema_version":3,"result":{
            "kind":"observed","service_instance_id":"1".repeat(32)}}))
            .unwrap();
        assert!(observed.validate_for_request(&request).is_err());
    }
}

fn host_gateway_route_verifier_field_v3<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}
impl HostGatewayInstanceRequestV3 {
    /// Produce a public route verifier from a distinct local capability and its
    /// original epoch/key. This never authenticates a caller or grants permission.
    pub fn route_verifier_from_capability(
        epoch: &str,
        acquisition: &str,
        capability: &str,
    ) -> Result<String, String> {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        validate_host_gateway_instance_v3(epoch)?;
        let acquisition = acquisition
            .strip_prefix("hgt_")
            .ok_or("host_gateway_route_verifier_invalid")?;
        if acquisition.len() != 64
            || !acquisition
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("host_gateway_route_verifier_invalid".into());
        }
        if capability.len() != 43 {
            return Err("host_gateway_route_verifier_invalid".into());
        }
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(capability)
            .map_err(|_| "host_gateway_route_verifier_invalid")?;
        if decoded.len() != 32
            || base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&decoded) != capability
        {
            return Err("host_gateway_route_verifier_invalid".into());
        }
        let mut hash = Sha256::new();
        hash.update(b"substrate.host-gateway.local-route.v1\0");
        for value in [epoch, acquisition] {
            for pair in value.as_bytes().chunks_exact(2) {
                let byte = u8::from_str_radix(
                    std::str::from_utf8(pair).map_err(|_| "host_gateway_route_verifier_invalid")?,
                    16,
                )
                .map_err(|_| "host_gateway_route_verifier_invalid")?;
                hash.update([byte]);
            }
        }
        hash.update(decoded);
        Ok(format!("{:x}", hash.finalize()))
    }
}

#[cfg(test)]
mod host_route_verifier_v3_tests {
    use super::*;
    #[test]
    fn test_route_verifier_independent_vector_and_cross_turn_binding() {
        let capability = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let epoch = "1".repeat(32);
        let key = format!("hgt_{}", "2".repeat(64));
        let value =
            HostGatewayInstanceRequestV3::route_verifier_from_capability(&epoch, &key, capability)
                .unwrap();
        assert_eq!(
            value,
            "a2abb73c94f9c5b5947c2014495f424bcc6b026927dcb35048780fad3dc535c2"
        );
        assert_ne!(
            value,
            HostGatewayInstanceRequestV3::route_verifier_from_capability(
                &"4".repeat(32),
                &key,
                capability
            )
            .unwrap()
        );
        assert_ne!(
            value,
            HostGatewayInstanceRequestV3::route_verifier_from_capability(
                &epoch,
                &format!("hgt_{}", "5".repeat(64)),
                capability
            )
            .unwrap()
        );
    }
}

#[cfg(test)]
mod host_acquire_route_verifier_v3_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_acquire_requires_verifier_other_operations_require_explicit_null() {
        let cleanup = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let acquisition = HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(cleanup).unwrap();
        let mut body = json!({"schema_version":3,"operation":{"kind":"bound","service_instance_id":"1".repeat(32),
            "route_verifier":"a".repeat(64),"operation":{"kind":"acquire","acquire_id":acquisition,"cleanup_token":cleanup,
            "selection":{"schema_version":1,"start":{"schema_version":1,"authority_store_id":"store_original",
                "original_start_intent_id":"intent_original","original_start_issuer_request_id":format!("req_{}","a".repeat(64)),
                "original_session_id":"session_original","context_sha256":"b".repeat(64),"native_selection_sha256":"c".repeat(64),"requested_selection_sha256":"d".repeat(64)},
                "runtime":{"selection_role":"host_actor","orchestration_session_id":"session_original","participant_id":"host_actor","backend_id":"cli:codex-host","run_id":"run_current"}}}}});
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(body.clone()).is_ok());
        body["operation"]["route_verifier"] = json!(null);
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(body.clone()).is_err());
        body["operation"]
            .as_object_mut()
            .unwrap()
            .remove("route_verifier");
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(body.clone()).is_err());
        body["operation"]["route_verifier"] = json!("a".repeat(64));
        body["operation"]["operation"] = json!({"kind":"release","acquire_id":acquisition,"cleanup_token":cleanup,"lease_handle":null});
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(body.clone()).is_err());
        body["operation"]["route_verifier"] = json!(null);
        assert!(serde_json::from_value::<HostGatewayInstanceRequestV3>(body).is_ok());
    }
}

/// Exact selectors for the Linux native owner operation. None is an authority claim.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCurrentResourceV1 {
    pub workspace_path: String,
    pub workspace_ref: String,
    pub workspace_incarnation: String,
    pub orchestration_session_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub backend_id: String,
}

/// Mandatory held HSA observation, distinct from native principal and read permission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceHeldAuthorityObservationV1 {
    pub authority_store_id: String,
    pub home_path: String,
    pub home_device: u64,
    pub home_inode: u64,
    pub root_revision: u64,
    pub authority_revision: u64,
    pub authority_record_sha256: String,
    pub authoritative_lineage_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum WorkspaceCurrentOperationV1 {
    CurrentActorRead {
        lane_ref: String,
        association_incarnation: String,
        association_revision: u64,
        backend_custody_ref: String,
        expected_actor: String,
    },
    /// Calls the actual source-owner publisher; it cannot request a validation grant.
    PublishAssociation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCurrentRequestV1 {
    pub schema_version: u32,
    pub native_context: InstallBootstrapContextCarrierV1,
    pub resource: WorkspaceCurrentResourceV1,
    pub observation: WorkspaceHeldAuthorityObservationV1,
    pub operation: WorkspaceCurrentOperationV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkspaceCurrentRefusalV1 {
    Denied,
    Unsupported,
    Ineligible,
    Changed,
    Integrity,
}

/// Revision observation only; never a send or execution capability.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceSettingsRevisionV1 {
    pub incarnation: String,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceReadRevisionVectorV1 {
    pub settings: Vec<Option<WorkspaceSettingsRevisionV1>>,
    pub workspace_revision: u64,
    pub association_incarnation: String,
    pub association_revision: u64,
    pub authority_store_id: String,
    pub root_revision: u64,
    pub authority_revision: u64,
    pub custody_incarnation: String,
    pub custody_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceEligibleObservationV1 {
    pub workspace_ref: String,
    pub workspace_incarnation: String,
    pub lane_ref: String,
    pub world_id: String,
    pub world_generation: u64,
    pub backend_id: String,
    pub actor_id: String,
    pub revisions: WorkspaceReadRevisionVectorV1,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "state", deny_unknown_fields)]
pub enum WorkspacePreflightResponseV1 {
    Refused { reason: WorkspaceCurrentRefusalV1 },
    EligibleObservation(Box<WorkspaceEligibleObservationV1>),
}

/// Exact independent source-owner association update; not executed by a read.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssociationUpdateV1 {
    pub current: WorkspaceCurrentRequestV1,
    pub active: bool,
}

/// Exact approved YAML patches, parsed by the existing strict owner parsers.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInitialSettingsV1 {
    pub config_yaml: String,
    pub policy_yaml: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInitializationOfferV1 {
    pub schema_version: u32,
    pub native_context: InstallBootstrapContextCarrierV1,
    pub home_device: u64,
    pub home_inode: u64,
    pub selected_location: String,
    pub root_device: u64,
    pub root_inode: u64,
    pub settings: WorkspaceInitialSettingsV1,
    pub effects: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "decision", deny_unknown_fields)]
pub enum WorkspaceInitializationDecisionV1 {
    Approved {
        offer: Box<WorkspaceInitializationOfferV1>,
    },
    Denied,
    Cancelled,
}

/// Bounded nonsecret projection carried by the current-actor reply.
/// Boxing this new reply payload keeps the enum small without changing its JSON map.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCurrentActorV1 {
    pub workspace_ref: String,
    pub workspace_incarnation: String,
    pub orchestration_session_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub backend_id: String,
    pub authority_store_id: String,
    pub root_revision: u64,
    pub authority_revision: u64,
    pub authority_record_sha256: String,
    pub authoritative_lineage_sha256: String,
    pub lane_ref: String,
    pub association_incarnation: String,
    pub association_revision: u64,
    pub backend_custody_ref: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", deny_unknown_fields)]
pub enum WorkspaceCurrentResponseV1 {
    Refused {
        reason: WorkspaceCurrentRefusalV1,
    },
    CurrentActor(Box<WorkspaceCurrentActorV1>),
    AssociationPublished {
        lane_ref: String,
        association_incarnation: String,
        association_revision: u64,
        backend_custody_ref: String,
        effects: WorkspaceAssociationEffectsV1,
    },
    AssociationPublicationFailed {
        reason: WorkspaceCurrentRefusalV1,
        effects: WorkspaceAssociationEffectsV1,
    },
}

/// Effects of the new selected-World publication, reported independently per source.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssociationUpdateEffectsV1 {
    pub association: WorkspaceAssociationEffectsV1,
    pub receipt: WorkspaceAssociationEffectsV1,
}
impl WorkspaceAssociationUpdateEffectsV1 {
    /// Loss after submission leaves both source publications unknown.
    ///
    /// ```
    /// let effects = transport_api_types::WorkspaceAssociationUpdateEffectsV1::unknown();
    /// assert_eq!(effects.receipt.published_visible, None);
    /// ```
    pub fn unknown() -> Self {
        Self {
            association: WorkspaceAssociationEffectsV1::unknown(),
            receipt: WorkspaceAssociationEffectsV1::unknown(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "state", deny_unknown_fields)]
pub enum WorkspaceAssociationUpdateResponseV1 {
    Refused {
        reason: WorkspaceCurrentRefusalV1,
    },
    AssociationPublished {
        lane_ref: String,
        association_incarnation: String,
        association_revision: u64,
        backend_custody_ref: String,
        effects: WorkspaceAssociationUpdateEffectsV1,
    },
    AssociationPublicationFailed {
        reason: WorkspaceCurrentRefusalV1,
        effects: WorkspaceAssociationUpdateEffectsV1,
    },
}

/// Publication effects only. Reads have no effect-bearing operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAssociationEffectsV1 {
    pub staging_created: Option<bool>,
    pub published_visible: Option<bool>,
    pub directory_sync_confirmed: Option<bool>,
}

impl WorkspaceCurrentRequestV1 {
    /// Validate syntax and bounds; callers must separately authenticate all selectors.
    pub fn validate(&self) -> Result<(), &'static str> {
        fn text(value: &str, limit: usize) -> bool {
            !value.is_empty()
                && value.len() <= limit
                && !value.bytes().any(|b| b == 0 || b.is_ascii_control())
        }
        fn path(value: &str) -> bool {
            text(value, 4096)
                && std::path::Path::new(value).is_absolute()
                && std::path::Path::new(value).components().all(|part| {
                    matches!(
                        part,
                        std::path::Component::RootDir | std::path::Component::Normal(_)
                    )
                })
        }
        fn sha(value: &str) -> bool {
            value.len() == 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }
        let r = &self.resource;
        let o = &self.observation;
        if self.schema_version != 1
            || self.native_context.validate().is_err()
            || !path(&r.workspace_path)
            || !path(&o.home_path)
            || ![
                &r.workspace_ref,
                &r.workspace_incarnation,
                &r.orchestration_session_id,
                &r.world_id,
                &r.backend_id,
                &o.authority_store_id,
            ]
            .into_iter()
            .all(|v| text(v, 256))
            || std::path::Path::new(&r.world_id).components().count() != 1
            || !matches!(
                std::path::Path::new(&r.world_id).components().next(),
                Some(std::path::Component::Normal(_))
            )
            || o.home_inode == 0
            || o.root_revision == 0
            || o.authority_revision == 0
            || !sha(&o.authority_record_sha256)
            || !sha(&o.authoritative_lineage_sha256)
        {
            return Err("Malformed");
        }
        if let WorkspaceCurrentOperationV1::CurrentActorRead {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            expected_actor,
        } = &self.operation
        {
            if !text(lane_ref, 256)
                || !text(association_incarnation, 256)
                || *association_revision == 0
                || !sha(backend_custody_ref)
                || !text(expected_actor, 256)
            {
                return Err("Malformed");
            }
        }
        Ok(())
    }
}

impl Default for WorkspaceAssociationEffectsV1 {
    fn default() -> Self {
        Self {
            staging_created: Some(false),
            published_visible: Some(false),
            directory_sync_confirmed: Some(false),
        }
    }
}
impl WorkspaceAssociationEffectsV1 {
    /// Transport loss after submission cannot establish absence of publication effects.
    ///
    /// ```
    /// let effects = transport_api_types::WorkspaceAssociationEffectsV1::unknown();
    /// assert_eq!(effects.published_visible, None);
    /// ```
    pub fn unknown() -> Self {
        Self {
            staging_created: None,
            published_visible: None,
            directory_sync_confirmed: None,
        }
    }
}

#[cfg(test)]
mod workspace_current_wire_tests {
    use super::*;
    fn request() -> WorkspaceCurrentRequestV1 {
        WorkspaceCurrentRequestV1 {
            schema_version: 1,
            native_context: InstallBootstrapContextCarrierV1::from_context(
                InstallBootstrapContextV1::new_unix("/private/home", "fixture", 1001).unwrap(),
            )
            .unwrap(),
            resource: WorkspaceCurrentResourceV1 {
                workspace_path: "/private/work".into(),
                workspace_ref: "ws_exact".into(),
                workspace_incarnation: "wsi_exact".into(),
                orchestration_session_id: "session".into(),
                world_id: "wld_exact".into(),
                world_generation: 0,
                backend_id: "cli:codex".into(),
            },
            observation: WorkspaceHeldAuthorityObservationV1 {
                authority_store_id: "store".into(),
                home_path: "/private/home".into(),
                home_device: 1,
                home_inode: 1,
                root_revision: 1,
                authority_revision: 1,
                authority_record_sha256: "a".repeat(64),
                authoritative_lineage_sha256: "b".repeat(64),
            },
            operation: WorkspaceCurrentOperationV1::PublishAssociation,
        }
    }
    #[test]
    fn test_workspace_current_wire_strict_bounds_and_genuine_generation_zero() {
        let base = request();
        assert!(base.validate().is_ok());
        for mutate in 0..7 {
            let mut wrong = base.clone();
            match mutate {
                0 => wrong.schema_version = 2,
                1 => wrong.resource.world_id = "../other".into(),
                2 => wrong.resource.workspace_path = "/private/../work".into(),
                3 => wrong.observation.root_revision = 0,
                4 => wrong.observation.home_inode = 0,
                5 => wrong.observation.authority_record_sha256 = "z".repeat(64),
                _ => wrong.resource.orchestration_session_id = "s".repeat(257),
            }
            assert!(wrong.validate().is_err(), "mutation {mutate}");
        }
        let mut value = serde_json::to_value(&base).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("authorized".into(), true.into());
        assert!(serde_json::from_value::<WorkspaceCurrentRequestV1>(value).is_err());
        let raw = serde_json::to_string(&base).unwrap().replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(serde_json::from_str::<WorkspaceCurrentRequestV1>(&raw).is_err());
        assert_ne!(
            WorkspaceAssociationEffectsV1::unknown(),
            WorkspaceAssociationEffectsV1::default()
        );
    }
}

#[cfg(test)]
mod workspace_current_response_wire_tests {
    use super::*;

    #[test]
    fn test_workspace_current_response_boxing_preserves_strict_flat_wire() {
        let expected = serde_json::json!({
            "state": "CurrentActor", "workspace_ref": "ws", "workspace_incarnation": "wsi",
            "orchestration_session_id": "session", "world_id": "wld", "world_generation": 0,
            "backend_id": "cli:codex", "authority_store_id": "store", "root_revision": 1,
            "authority_revision": 2, "authority_record_sha256": "a".repeat(64),
            "authoritative_lineage_sha256": "b".repeat(64), "lane_ref": "lane",
            "association_incarnation": "association", "association_revision": 1,
            "backend_custody_ref": "c".repeat(64), "actor_id": "actor"
        });
        let reply: WorkspaceCurrentResponseV1 = serde_json::from_value(expected.clone()).unwrap();
        assert!(
            matches!(&reply, WorkspaceCurrentResponseV1::CurrentActor(current) if current.world_generation == 0)
        );
        assert_eq!(serde_json::to_value(&reply).unwrap(), expected);
        let mut unknown = expected.clone();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("authorized".into(), true.into());
        assert!(serde_json::from_value::<WorkspaceCurrentResponseV1>(unknown).is_err());
        let mut missing = expected.clone();
        missing.as_object_mut().unwrap().remove("actor_id");
        assert!(serde_json::from_value::<WorkspaceCurrentResponseV1>(missing).is_err());
        let raw = serde_json::to_string(&expected).unwrap().replacen(
            "\"actor_id\":\"actor\"",
            "\"actor_id\":\"actor\",\"actor_id\":\"actor\"",
            1,
        );
        assert!(serde_json::from_str::<WorkspaceCurrentResponseV1>(&raw).is_err());
    }
}
