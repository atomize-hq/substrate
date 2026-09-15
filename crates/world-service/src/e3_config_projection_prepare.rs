//! Service-owned E3 preparation admission. No request can select its HSA authority.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use config_projection::{AgentConfigProjectionServiceV1, ConfigProjectionFailureV1};
use transport_api_types::{
    E3ConfigProjectionPrepareRequestV1, E3ConfigProjectionPrepareResponseV1,
    MemberDispatchCommonFieldsV1,
};

pub(crate) struct E3ConfigProjectionPreparationManagerV1 {
    authority: Arc<substrate_shell::OpenedConfigProjectionHsaAuthorityV1>,
    projection_service: Arc<AgentConfigProjectionServiceV1>,
    registry: Arc<config_projection::ConfigProjectionRegistryV1>,
    exclusion: Arc<crate::e3_child_security::E3PrivilegedChildExclusionV1>,
    preparations: Mutex<BTreeMap<String, SealedE3ConfigProjectionPreparationV1>>,
    transferred_preparations: Mutex<std::collections::BTreeSet<String>>,
    service_instance_id: OnceLock<String>,
}

pub(crate) enum E3PreparationRecoveryV1<'a> {
    Startup { service_instance_id: &'a str },
    Subject(&'a config_projection::ConfigProjectionPreparationMetadataV1),
}

// Inserted before binding or any kernel transaction. A failed attempt stays here until its
// kernel cleanup and durable resolutions succeed; losing an HTTP request never loses the owner.
struct ClockBoottimeDeadline {
    seconds: libc::time_t,
    nanoseconds: libc::c_long,
}

pub(crate) struct SealedE3ConfigProjectionPreparationV1 {
    claimed: bool,
    pub(crate) ready_closed_ref: Option<config_projection::ConfigProjectionRefV1>,
    response: Option<E3ConfigProjectionPrepareResponseV1>,
    pub(crate) projection: Option<config_projection::PublishedConfigProjectionCapabilityV1>,
    pub(crate) publication: Option<config_projection::E3PreparedRetainedLaunchPublicationV1>,
    prepared_boundary: Option<(
        config_projection::GatewayAccessBoundaryV1,
        [config_projection::E3ChildCgroupRegistrationV1; 3],
        config_projection::GatewayRuntimeConfigIdentityV1,
    )>,
    identity: config_projection::ConfigProjectionIdentityV1,
    expected_predecessor: Option<config_projection::ConfigProjectionRefV1>,
    gateway: config_projection::InWorldGatewayIdentityV1,
    logical: config_projection::LogicalAgentConfigProjectionV1,
    fence_id: String,
    expires_deadline: ClockBoottimeDeadline,
    created_at: config_projection::Timestamp,
    credential_ref: config_projection::CredentialSourceRefV1,
    record_id: String,
    launch_input_id: String,
    activation_intent: Option<config_projection::ManagedGatewayActivationIntentV1>,
    native_plan: Option<config_projection::Codex0125ProjectionPlanV1>,
    native_source_manifest: Option<config_projection::NativeProjectionSourceManifestV1>,
    chain_bound: bool,
    // Field order also scrubs credentials before gateway cleanup/exclusion on unwinding.
    credential_source: Option<SealedCredentialSourceCapabilityV1>,
    terminal: Option<(
        config_projection::SecretHandoffStateV1,
        config_projection::Timestamp,
    )>,
    cleanup_complete: bool,
    cleanup_released_consumer: Option<config_projection::ConfigProjectionConsumerLeaseV1>,
    expiry_task: Option<tokio::task::JoinHandle<()>>,
    pub(crate) gateway_authority: Option<crate::gateway_runtime::E3GatewayRuntimeAuthorityV1>,
}

// Owns ingress storage even before it becomes an accepted credential capability. It cannot be
// cloned, formatted or serialized. Keeping the decoded request here also covers authentication
// rejection and unwinding, before any integrated-auth acceptance or projection transaction.
struct SealedCredentialSourceCapabilityV1 {
    wire: Vec<u8>,
    request: Option<E3ConfigProjectionPrepareRequestV1>,
}

impl Drop for SealedCredentialSourceCapabilityV1 {
    fn drop(&mut self) {
        let scrub = |bytes: &mut [u8]| {
            for byte in bytes {
                // SAFETY: the live allocation is exclusively borrowed. Volatile writes and the
                // compiler fence prevent dead-store elimination before its deallocation.
                unsafe { std::ptr::write_volatile(byte, 0) };
            }
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        };
        scrub(&mut self.wire);
        if let Some(request) = self.request.as_mut() {
            if let Some(auth) = request.integrated_auth.cli_codex.as_mut() {
                // SAFETY: replacing UTF-8 bytes with NUL preserves UTF-8, and these strings are
                // immediately dropped without exposing a mutable alias.
                unsafe { scrub(auth.access_token.as_bytes_mut()) };
                if let Some(account) = auth.account_id.as_mut() {
                    unsafe { scrub(account.as_bytes_mut()) };
                }
            }
            if let Some(auth) = request.integrated_auth.api_env.as_mut() {
                for value in auth.env.values_mut() {
                    unsafe { scrub(value.as_bytes_mut()) };
                }
            }
        }
    }
}

impl E3ConfigProjectionPreparationManagerV1 {
    pub(crate) fn new(
        authority: Arc<substrate_shell::OpenedConfigProjectionHsaAuthorityV1>,
        projection_service: Arc<AgentConfigProjectionServiceV1>,
        registry: Arc<config_projection::ConfigProjectionRegistryV1>,
        exclusion: Arc<crate::e3_child_security::E3PrivilegedChildExclusionV1>,
    ) -> Self {
        Self {
            authority,
            projection_service,
            registry,
            exclusion,
            preparations: Mutex::new(BTreeMap::new()),
            transferred_preparations: Mutex::new(std::collections::BTreeSet::new()),
            service_instance_id: OnceLock::new(),
        }
    }

    pub(crate) async fn prepare(
        self: &Arc<Self>,
        mut body: hyper::Body,
    ) -> Result<E3ConfigProjectionPrepareResponseV1, ConfigProjectionFailureV1> {
        use hyper::body::HttpBody;
        self.recover_expired(None, None)?;
        let mut secret = SealedCredentialSourceCapabilityV1 {
            // Reserve the complete bounded buffer once so growth never abandons a secret copy.
            wire: Vec::with_capacity(65_536),
            request: None,
        };
        while let Some(chunk) = body.data().await {
            let chunk = chunk.map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            if chunk.len() > 65_536 - secret.wire.len() {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            secret.wire.extend_from_slice(&chunk);
        }
        secret.request = Some(
            serde_json::from_slice(&secret.wire)
                .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
        );
        let request = secret
            .request
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        // Resolve and drop the complete projection transaction before either HSA read.
        let (_, authoring) = self
            .registry
            .resolve(None, Some(&request.authoring_input_ref))?;
        let (effective, source, manifest) =
            authoring.ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        // Both shell-private reads finish before the service opens any projection transaction.
        // Only the reconstructed authenticated carrier is used from this point onward.
        request.e2_launch_activation = self.authority.authenticate_e3_member_launch_activation_v1(
            MemberDispatchCommonFieldsV1 {
                orchestration_session_id: &request.orchestration_session_id,
                participant_id: &request.participant_id,
                orchestrator_participant_id: &request.orchestrator_participant_id,
                parent_participant_id: request.parent_participant_id.as_deref(),
                resumed_from_participant_id: request.resumed_from_participant_id.as_deref(),
                backend_id: &request.backend_id,
                protocol: &request.protocol,
                run_id: &request.run_id,
                world_id: &request.world_id,
                world_generation: request.world_generation,
                initial_prompt: None,
                resolved_runtime: &request.resolved_runtime,
                retained_worker_launch_authority: request.retained_worker_launch_authority.as_ref(),
            },
            &request.e2_launch_activation,
        )?;
        let logical = self
            .authority
            .read_e3_selected_inventory_projection_v1(&effective, &source)?;
        // Only authenticated, completely joined inputs may reach attempt ownership or effects.
        let expected_sources = vec![
            config_projection::LogicalConfigSourceRefV1::EffectiveSubstrateConfig {
                authority_store_id: effective.authority_store_id.clone(),
                source_revision: effective.source_revision.clone(),
                source_hash: effective.source_hash.clone(),
            },
            config_projection::LogicalConfigSourceRefV1::AgentInventory {
                authority_store_id: effective.authority_store_id.clone(),
                inventory_scope: source.inventory_scope.clone(),
                source_revision: source.source_revision.clone(),
                source_hash: source.source_hash.clone(),
            },
        ];
        let runtime_family = match request.resolved_runtime.backend_kind {
            transport_api_types::MemberRuntimeBackendKindV1::Codex => "codex",
            transport_api_types::MemberRuntimeBackendKindV1::ClaudeCode => "claude_code",
        };
        let runtime_artifact = manifest.entries.iter().find(|entry| {
            entry.authority_role == config_projection::RuntimeArtifactAuthorityRoleV1::Codex0125
        });
        if logical.sources != expected_sources
            || effective.authority_store_id != request.authoring_input_ref.authority_store_id
            || manifest.authority_store_id != effective.authority_store_id
            || logical.backend_id != effective.values.default_backend_id
            || logical.backend_id != request.backend_id
            || logical.protocol != request.protocol
            || logical.placement != "world"
            || logical.execution_scope != "world"
            || logical.runtime_family != runtime_family
            || runtime_artifact.is_none_or(|entry| {
                entry.configured_absolute_path != request.resolved_runtime.binary_path
            })
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let complete = |owned: &mut SealedE3ConfigProjectionPreparationV1| -> Result<
            E3ConfigProjectionPrepareResponseV1,
            ConfigProjectionFailureV1,
        > {
            use config_projection::*;
            use std::os::fd::AsFd;
            use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
            let seal = |domain: &str, key: &str, field: &str, mut value: serde_json::Value| {
                value
                    .as_object_mut()
                    .ok_or(ConfigProjectionFailureV1::Malformed)?
                    .remove(field);
                ConfigProjectionCodecV1::domain_sha256(domain, &serde_json::json!({key:value}))
            };
            let identity = &owned.identity;
            let revision = match &owned.expected_predecessor {
                Some(expected) => expected
                    .revision
                    .checked_add(1)
                    .ok_or(ConfigProjectionFailureV1::StaleRevision)?,
                None => 1,
            };
            let publication = owned
                .publication
                .as_mut()
                .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
            if !owned.chain_bound {
                let (boundary, registrations, gateway_config) = owned
                    .prepared_boundary
                    .as_ref()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                let gateway_ref = InWorldGatewayRefV1 {
                    authority_store_id: identity.authority_store_id.clone(),
                    gateway_instance_id: owned.gateway.gateway_instance_id.clone(),
                    gateway_identity_hash: owned.gateway.gateway_identity_hash.clone(),
                };
                let boundary_ref = GatewayAccessBoundaryRefV1 {
                    authority_store_id: boundary.authority_store_id.clone(),
                    access_boundary_id: boundary.access_boundary_id.clone(),
                    revision: boundary.revision,
                    boundary_hash: boundary.boundary_hash.clone(),
                };
                if owned.activation_intent.is_none() {
                    let mut intent = ManagedGatewayActivationIntentV1 {
                        schema_version: 1,
                        authority_store_id: identity.authority_store_id.clone(),
                        preparation_id: owned.credential_ref.preparation_id.clone(),
                        activation_intent_id: format!("gai_{}", uuid::Uuid::now_v7()),
                        config_projection_identity_hash: identity.identity_hash.clone(),
                        dormant_record_id: owned.record_id.clone(),
                        dormant_revision: revision,
                        expected_gateway_artifact: identity
                            .runtime_artifacts
                            .managed_gateway
                            .clone(),
                        expected_gateway_ref: gateway_ref.clone(),
                        expected_access_boundary_ref: boundary_ref.clone(),
                        secret_handoff_ref: publication.prepared_handoff_ref().clone(),
                        fence_id: owned.fence_id.clone(),
                        readiness_nonce: uuid::Uuid::now_v7().to_string(),
                        created_at: owned.created_at.clone(),
                        intent_hash: String::new(),
                    };
                    intent.intent_hash = seal(
                        "substrate.e3.managed-gateway-activation-intent.v1",
                        "intent",
                        "intent_hash",
                        serde_json::to_value(&intent)
                            .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                    )?;
                    owned.activation_intent = Some(intent);
                }
                let intent = owned
                    .activation_intent
                    .as_ref()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                let intent_ref = ManagedGatewayActivationIntentRefV1 {
                    authority_store_id: identity.authority_store_id.clone(),
                    activation_intent_id: intent.activation_intent_id.clone(),
                    intent_hash: intent.intent_hash.clone(),
                };
                let mut managed_gateway = ManagedGatewayProjectionV1 {
                    projection_hash: String::new(),
                    activation_intent_ref: intent_ref.clone(),
                    expected_gateway_ref: gateway_ref.clone(),
                    codex_base_url: format!(
                        "http://127.0.0.1:{}/v1",
                        boundary.gateway_listener.port
                    ),
                    access_boundary_ref: boundary_ref.clone(),
                    activation_ack_ref: None,
                    posture: ManagedGatewayProjectionPostureV1::Dormant,
                };
                managed_gateway.projection_hash = seal(
                    "substrate.e3.managed-gateway-projection.v1",
                    "projection",
                    "projection_hash",
                    serde_json::to_value(&managed_gateway)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                let home = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(&identity.accepted_home.physical_path)
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                identity
                    .accepted_home
                    .revalidate_linux_from_fd(home.as_fd())?;
                let metadata = home
                    .metadata()
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                let root = NativeProjectionRootV1 {
                    root_id: format!(
                        "cnr_{}",
                        owned
                            .fence_id
                            .strip_prefix("cpf_")
                            .ok_or(ConfigProjectionFailureV1::Malformed)?
                    ),
                    authority_relative_path: format!(
                        "authority-v1/agent-config-projection-v1/native-sources/{}/{}",
                        identity.series_id, owned.fence_id
                    ),
                    guest_absolute_path: format!(
                        "/run/substrate/member-config/{}/{}",
                        identity.series_id, owned.fence_id
                    ),
                    owner_uid: u64::from(metadata.uid()),
                    owner_gid: u64::from(metadata.gid()),
                    directory_mode: 0o700,
                };
                let environment = EffectiveEnvironmentV1 {
                    inherited_names: Vec::new(),
                    set: [
                        ("CODEX_HOME", format!("{}/codex-home", root.guest_absolute_path)),
                        ("CODEX_SQLITE_HOME", format!("{}/state/sqlite", root.guest_absolute_path)),
                        ("HOME", format!("{}/home", root.guest_absolute_path)),
                        ("LANG", "C.UTF-8".to_string()),
                        ("LC_ALL", "C.UTF-8".to_string()),
                        ("PATH", "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()),
                        ("RUST_LOG", "error".to_string()),
                        ("TMPDIR", format!("{}/tmp", root.guest_absolute_path)),
                    ].into_iter().map(|(name, value)| NamedValueV1 { name: name.to_string(), value }).collect(),
                    remove: ["ANTHROPIC_API_KEY", "CODEX_API_KEY", "CODEX_BINARY", "CODEX_OSS_BASE_URL", "CODEX_OSS_PORT", "OPENAI_ACCESS_TOKEN", "OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_ORGANIZATION", "OPENAI_PROJECT", "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD", "SUBSTRATE_E3_NATIVE_REALIZATION_FD", "SUBSTRATE_E3_NATIVE_SOURCE_FD", "SUBSTRATE_E3_SYSTEM_EMPTY_FD", "SUBSTRATE_E3_WORLD_FS_INPUT_FD", "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME", "SUBSTRATE_LLM_AUTH_BUNDLE_FD", "SUBSTRATE_WORLD_ENTRY_BINARY", "SUBSTRATE_WORLD_ENTRY_BINARY_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH", "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD", "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH", "SUBSTRATE_WORLD_ENTRY_ROLE", "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD"].into_iter().map(str::to_string).collect(),
                };
                let mut effective = EffectiveAgentConfigProjectionV1 {
                    projection_hash: String::new(),
                    logical_projection_hash: owned.logical.projection_hash.clone(),
                    accepted_policy: identity.immutable_launch_cap.clone(),
                    capabilities: owned.logical.capabilities.clone(),
                    model: owned.logical.requested_model.clone(),
                    provider: EffectiveManagedProviderV1 {
                        provider_id: "substrate-managed-gateway".into(),
                        wire_api: "responses".into(),
                        requires_openai_auth: false,
                        supports_websockets: false,
                        gateway_intent_ref: intent_ref.clone(),
                    },
                    mcp_servers: owned
                        .logical
                        .requested_mcp_servers
                        .iter()
                        .map(|server| EffectiveMcpServerV1 {
                            server_id: server.server_id.clone(),
                            transport: server.transport.clone(),
                            enabled: server.enabled,
                        })
                        .collect(),
                    features: owned.logical.requested_features.clone(),
                    environment,
                    workspace_overlay: WorkspaceOverlayPostureV1::Disabled,
                };
                effective.projection_hash = seal(
                    "substrate.e3.effective-config-projection.v1",
                    "projection",
                    "projection_hash",
                    serde_json::to_value(&effective)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                if owned.native_plan.is_none() {
                    let workspace = std::fs::OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                        .open(&identity.workspace_root.physical_path)
                        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    identity
                        .workspace_root
                        .revalidate_linux_from_fd(workspace.as_fd())?;
                    // The entire workspace Project layer is disabled by the rendered trust rule;
                    // no content from those locators is read or claimed as loaded.
                    let project_inputs = [".codex/config.toml", ".codex/rules", ".codex/skills"]
                        .into_iter()
                        .map(|relative| CodexLoaderInputAttestationV1 {
                            layer: "Project".into(),
                            locator: format!(
                                "{}/{}",
                                identity.workspace_root.physical_path, relative
                            ),
                            disposition: CodexLoaderInputDispositionV1::DisabledByTrust,
                            directory: identity.workspace_root.clone(),
                            relative_path: relative.into(),
                            device_id: None,
                            inode: None,
                            byte_length: None,
                            sha256: None,
                        })
                        .collect();
                    owned.native_plan = Some(Codex0125ProjectionV1::render(
                        identity,
                        &effective,
                        &managed_gateway,
                        root,
                        &owned.fence_id,
                        project_inputs,
                    )?);
                }
                let (native, manifest) = self.registry.publish_native_source(
                    &identity.series_id,
                    &owned.fence_id,
                    owned
                        .native_plan
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                    owned.created_at.clone(),
                )?;
                if owned
                    .native_source_manifest
                    .as_ref()
                    .is_some_and(|original| original != &manifest)
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                owned.native_source_manifest = Some(manifest);
                let mut handoff = NonsecretHandoffProjectionV1 {
                    projection_hash: String::new(),
                    secret_handoff_ref: publication.prepared_handoff_ref().clone(),
                    credential_source_ref: owned.credential_ref.clone(),
                    receiving_gateway_ref: gateway_ref.clone(),
                    delivery: SecretDeliveryMechanismV1::SecureFd {
                        fd_name: "SUBSTRATE_LLM_AUTH_BUNDLE_FD".into(),
                        one_time: true,
                        gateway_receiver_only: true,
                        deny_child_inheritance: true,
                        close_after_consume: true,
                    },
                    observed_state: SecretHandoffStateV1::Prepared,
                    activation_ack_ref: None,
                };
                handoff.projection_hash = seal(
                    "substrate.e3.nonsecret-handoff-projection.v1",
                    "projection",
                    "projection_hash",
                    serde_json::to_value(&handoff)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                let mut record = AgentConfigProjectionRecordV1 {
                    schema_version: 1,
                    identity: identity.clone(),
                    record_id: owned.record_id.clone(),
                    revision,
                    predecessor_ref: owned.expected_predecessor.clone(),
                    logical: owned.logical.clone(),
                    effective,
                    native,
                    managed_gateway,
                    nonsecret_handoff: handoff,
                    activation: ConfigProjectionActivationV1 {
                        activation_intent_ref: intent_ref.clone(),
                        publication_fence: ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                            fence_id: owned.fence_id.clone(),
                        },
                        gateway_activation_ack_ref: None,
                        released_at: None,
                    },
                    created_at: owned.created_at.clone(),
                    record_hash: String::new(),
                };
                record.record_hash = seal(
                    "substrate.e3.agent-config-projection-record.v1",
                    "record",
                    "record_hash",
                    serde_json::to_value(&record)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                let mut input = ManagedGatewayLaunchInputV1 {
                    schema_version: 1,
                    authority_store_id: identity.authority_store_id.clone(),
                    launch_input_id: owned.launch_input_id.clone(),
                    activation_intent_ref: intent_ref,
                    dormant_projection_ref: ConfigProjectionRefV1 {
                        authority_store_id: identity.authority_store_id.clone(),
                        series_id: identity.series_id.clone(),
                        record_id: record.record_id.clone(),
                        revision: record.revision,
                        record_hash: record.record_hash.clone(),
                    },
                    gateway_ref,
                    config_projection_identity_hash: identity.identity_hash.clone(),
                    orchestration_session_id: identity.orchestration_session_id.clone(),
                    retained_participant_id: identity.retained_participant_id.clone(),
                    backend_id: identity.backend_id.clone(),
                    world_id: identity.world_id.clone(),
                    world_generation: identity.world_generation,
                    listener_identity: boundary.gateway_listener.clone(),
                    gateway_config: gateway_config.clone(),
                    http_surface: GatewayHttpSurfaceV1 {
                        inherited_listener_only: true,
                        readiness_method: "GET".into(),
                        readiness_path: "/health".into(),
                        member_method: "POST".into(),
                        member_path: "/v1/responses".into(),
                        auxiliary_listener_count: 0,
                    },
                    access_boundary_ref: boundary_ref,
                    secret_handoff_prepared_ref: publication.prepared_handoff_ref().clone(),
                    readiness_nonce: intent.readiness_nonce.clone(),
                    launch_input_hash: String::new(),
                };
                input.launch_input_hash = seal(
                    "substrate.e3.managed-gateway-launch-input.v1",
                    "launch_input",
                    "launch_input_hash",
                    serde_json::to_value(&input)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                publication.bind_prepared_chain(
                    record,
                    registrations.clone(),
                    boundary.clone(),
                    intent.clone(),
                    input,
                )?;
                owned.chain_bound = true;
            }
            let request = owned
                .credential_source
                .as_ref()
                .and_then(|source| source.request.as_ref())
                .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?;
            let (response, capability) = self.projection_service.publish_prepared_retained_launch(
                &request.authoring_input_ref,
                &request.e2_launch_activation,
                publication,
            )?;
            owned.projection = Some(capability);
            owned.response = Some(response.clone());
            Ok(response)
        };
        let preparation_id = request.preparation_id.clone();
        let mut preparations = self
            .preparations
            .lock()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if self
            .transferred_preparations
            .lock()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?
            .contains(&preparation_id)
        {
            return Err(ConfigProjectionFailureV1::ExpiredPreparation);
        }
        if preparations
            .values()
            .any(|attempt| attempt.credential_source.is_none() && !attempt.cleanup_complete)
        {
            // Failed cleanup retains authority and closes E3 admission as well as the held
            // non-E3 exclusion. Recovery must finish before admitting another attempt.
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        if preparations.iter().any(|(id, attempt)| {
            id != &preparation_id
                && attempt
                    .credential_source
                    .as_ref()
                    .and_then(|source| source.request.as_ref())
                    .is_some_and(|original| {
                        original.preparation_idempotency_key == request.preparation_idempotency_key
                    })
        }) {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        if let Some(owned) = preparations.get_mut(&preparation_id) {
            if owned.terminal.is_some() || owned.claimed {
                return Err(ConfigProjectionFailureV1::ExpiredPreparation);
            }
            let original = owned
                .credential_source
                .as_ref()
                .and_then(|source| source.request.as_ref())
                .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            if original.preparation_idempotency_key != request.preparation_idempotency_key
                || original.orchestration_session_id != request.orchestration_session_id
                || original.participant_id != request.participant_id
                || original.orchestrator_participant_id != request.orchestrator_participant_id
                || original.parent_participant_id != request.parent_participant_id
                || original.resumed_from_participant_id != request.resumed_from_participant_id
                || original.backend_id != request.backend_id
                || original.protocol != request.protocol
                || original.run_id != request.run_id
                || original.world_id != request.world_id
                || original.world_generation != request.world_generation
                || original.resolved_runtime != request.resolved_runtime
                || original.retained_worker_launch_authority
                    != request.retained_worker_launch_authority
                || original.e2_launch_activation != request.e2_launch_activation
                || original.authoring_input_ref != request.authoring_input_ref
            {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            // A duplicate's secret values are never compared or substituted into the retained owner.
            drop(secret);
            let mut now = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut now) } != 0 {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            if (now.tv_sec, now.tv_nsec)
                >= (
                    owned.expires_deadline.seconds,
                    owned.expires_deadline.nanoseconds,
                )
            {
                return Err(ConfigProjectionFailureV1::ExpiredPreparation);
            }
            if let Some(response) = &owned.response {
                let lease = owned
                    .publication
                    .as_ref()
                    .and_then(|publication| publication.held_consumer_lease())
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                let (resolved, _) = self
                    .registry
                    .resolve(Some((&owned.identity, lease)), None)?;
                match resolved {
                    Some(config_projection::ConfigProjectionResolutionV1::Current {
                        identity,
                        record,
                        projection_ref,
                        ..
                    }) if identity == owned.identity
                        && projection_ref == response.config_projection.dormant_projection_ref
                        && record.managed_gateway.posture
                            == config_projection::ManagedGatewayProjectionPostureV1::Dormant => {}
                    Some(config_projection::ConfigProjectionResolutionV1::Retired { .. }) => {
                        return Err(ConfigProjectionFailureV1::RetiredSeries);
                    }
                    _ => return Err(ConfigProjectionFailureV1::StaleRevision),
                }
                return Ok(response.clone());
            }
            return complete(owned);
        }
        let seal = |domain: &str,
                    key: &str,
                    field: &str,
                    mut value: serde_json::Value|
         -> Result<String, ConfigProjectionFailureV1> {
            value
                .as_object_mut()
                .ok_or(ConfigProjectionFailureV1::Malformed)?
                .remove(field);
            config_projection::ConfigProjectionCodecV1::domain_sha256(
                domain,
                &serde_json::json!({key:value}),
            )
        };
        let artifact = |role: config_projection::RuntimeArtifactAuthorityRoleV1,
                        projection_role: config_projection::ConfigProjectionArtifactRoleV1|
         -> Result<
            config_projection::DescriptorPinnedArtifactV1,
            ConfigProjectionFailureV1,
        > {
            let entry = manifest
                .entries
                .iter()
                .find(|entry| entry.authority_role == role)
                .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
            Ok(config_projection::DescriptorPinnedArtifactV1 {
                role: projection_role,
                configured_absolute_path: entry.configured_absolute_path.clone(),
                device_id: entry.device_id,
                inode: entry.inode,
                file_type: "regular".into(),
                mode: entry.mode,
                owner_uid: entry.owner_uid,
                byte_length: entry.byte_length,
                sha256: entry.sha256.clone(),
                provenance: entry.provenance.clone(),
                runtime_support: entry.runtime_support.clone(),
                authority_ref: config_projection::RuntimeArtifactAuthorityRefV1 {
                    authority_store_id: manifest.authority_store_id.clone(),
                    manifest_id: manifest.manifest_id.clone(),
                    manifest_revision: manifest.revision,
                    manifest_entry_id: entry.manifest_entry_id.clone(),
                    manifest_hash: manifest.manifest_hash.clone(),
                    entry_hash: entry.entry_hash.clone(),
                },
            })
        };
        let e2 = &request.e2_launch_activation;
        let subject = match e2.launch_kind {
            transport_api_types::E2MemberLaunchKindV1::FreshSpawn => {
                config_projection::ConfigProjectionE2SubjectV1::RetainedWorkerLaunch {
                    retained_participant_id: e2.retained_participant_id.clone(),
                    bootstrap_run_id: e2.bootstrap_run_id.clone(),
                }
            }
            transport_api_types::E2MemberLaunchKindV1::Fork => {
                config_projection::ConfigProjectionE2SubjectV1::RetainedWorkerFork {
                    source_participant_id: e2
                        .source_participant_id
                        .clone()
                        .ok_or(ConfigProjectionFailureV1::WrongBinding)?,
                    child_participant_id: e2.retained_participant_id.clone(),
                    bootstrap_run_id: e2.bootstrap_run_id.clone(),
                }
            }
        };
        let cap = config_projection::ConfigProjectionE2CapLinkV1 {
            e2_activation_id: e2.activation_id.clone(),
            e2_launch_kind: e2.launch_kind,
            commitment_ref: e2.commitment_ref.clone(),
            commitment_subject: subject,
            immutable_worker_cap_ref: e2.immutable_worker_cap_ref.clone(),
            immutable_worker_cap_created_revision: e2.immutable_worker_cap_created_revision,
            immutable_worker_cap_application_revision: e2.immutable_worker_cap_application_revision,
            policy_snapshot_ref: e2.policy_snapshot_ref.clone(),
            policy_snapshot_hash: e2.policy_snapshot_hash.clone(),
            policy_snapshot_revision: e2.policy_snapshot_revision.clone(),
            request_id: e2.request_id.clone(),
            idempotency_key: e2.idempotency_key.clone(),
            caller_participant_id: e2.caller_participant_id.clone(),
            caller_backend_id: e2.caller_backend_id.clone(),
            target_backend_id: e2.target_backend_id.clone(),
            target_world: e2.target_world.clone(),
            registry_publication_revision: e2.registry_publication_revision,
        };
        let mut identity = config_projection::ConfigProjectionIdentityV1 {
            schema_version: 1,
            authority_store_id: effective.authority_store_id.clone(),
            series_id: format!("cps_{}", uuid::Uuid::now_v7()),
            accepted_home: effective.accepted_home.clone(),
            workspace_root: effective.workspace_root.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            retained_participant_id: request.participant_id.clone(),
            bootstrap_run_id: request.run_id.clone(),
            backend_id: request.backend_id.clone(),
            runtime_family: logical.runtime_family.clone(),
            world_id: request.world_id.clone(),
            world_generation: request.world_generation,
            immutable_launch_cap: cap,
            runtime_artifacts: config_projection::ConfigProjectionRuntimeArtifactsV1 {
                codex: artifact(
                    config_projection::RuntimeArtifactAuthorityRoleV1::Codex0125,
                    config_projection::ConfigProjectionArtifactRoleV1::Codex0125,
                )?,
                world_entry_wrapper: artifact(
                    config_projection::RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
                    config_projection::ConfigProjectionArtifactRoleV1::WorldEntryWrapper,
                )?,
                managed_gateway: artifact(
                    config_projection::RuntimeArtifactAuthorityRoleV1::ManagedGateway,
                    config_projection::ConfigProjectionArtifactRoleV1::ManagedGateway,
                )?,
            },
            identity_hash: String::new(),
        };
        identity.identity_hash = seal(
            "substrate.e3.config-projection-identity.v1",
            "identity",
            "identity_hash",
            serde_json::to_value(&identity).map_err(|_| ConfigProjectionFailureV1::Malformed)?,
        )?;
        let expected_predecessor = match self
            .projection_service
            .resolve_preparation_subject_v1(&identity)?
        {
            config_projection::ConfigProjectionSubjectReadbackV1::Unbound => None,
            config_projection::ConfigProjectionSubjectReadbackV1::Bound(metadata) => {
                let bound = &metadata.record().identity;
                if metadata
                    .record()
                    .nonsecret_handoff
                    .credential_source_ref
                    .preparation_id
                    == preparation_id
                {
                    return Err(ConfigProjectionFailureV1::ExpiredPreparation);
                }
                if preparations
                    .values()
                    .any(|owner| &owner.identity == bound && !owner.cleanup_complete)
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                // Subject cleanup takes this mutex itself and repeats the exact durable head check.
                drop(preparations);
                self.recover_expired(None, Some(E3PreparationRecoveryV1::Subject(&metadata)))?;
                preparations = self
                    .preparations
                    .lock()
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                if preparations
                    .values()
                    .any(|owner| &owner.identity == bound && !owner.cleanup_complete)
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                let config_projection::ConfigProjectionSubjectReadbackV1::Bound(current) = self
                    .projection_service
                    .resolve_preparation_subject_v1(bound)?
                else {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                };
                if current.record() != metadata.record()
                    || current.projection_ref() != metadata.projection_ref()
                {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                identity = current.record().identity.clone();
                Some(current.projection_ref().clone())
            }
        };
        let fence_id = format!("cpf_{}", uuid::Uuid::now_v7());
        let mut gateway = config_projection::InWorldGatewayIdentityV1 {
            schema_version: 1,
            authority_store_id: identity.authority_store_id.clone(),
            gateway_instance_id: format!("cgi_{}", uuid::Uuid::now_v7()),
            config_projection_identity_hash: identity.identity_hash.clone(),
            orchestration_session_id: identity.orchestration_session_id.clone(),
            retained_participant_id: identity.retained_participant_id.clone(),
            backend_id: identity.backend_id.clone(),
            world_id: identity.world_id.clone(),
            world_generation: identity.world_generation,
            gateway_artifact_sha256: identity.runtime_artifacts.managed_gateway.sha256.clone(),
            access_boundary_id: format!("gab_{}", uuid::Uuid::now_v7()),
            gateway_identity_hash: String::new(),
        };
        gateway.gateway_identity_hash = seal(
            "substrate.e3.in-world-gateway-identity.v1",
            "gateway",
            "gateway_identity_hash",
            serde_json::to_value(&gateway).map_err(|_| ConfigProjectionFailureV1::Malformed)?,
        )?;
        let mut clock = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: the output pointer is valid for this child-free CLOCK_BOOTTIME observation.
        if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut clock) } != 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let expires_deadline = ClockBoottimeDeadline {
            seconds: clock
                .tv_sec
                .checked_add(120)
                .ok_or(ConfigProjectionFailureV1::Malformed)?,
            nanoseconds: clock.tv_nsec,
        };
        let now = chrono::Utc::now();
        let created_at =
            config_projection::Timestamp(now.to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
        let expires_at = config_projection::Timestamp(
            (now + chrono::Duration::seconds(120))
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        );
        let account_present = request
            .integrated_auth
            .cli_codex
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .account_id
            .is_some();
        let mut names = vec!["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string()];
        if account_present {
            names.push("SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID".to_string());
        }
        let mut credential = config_projection::CredentialSourceRefV1 {
            schema_version: 1,
            credential_source_id: format!("crs_{}", uuid::Uuid::now_v7()),
            preparation_id: request.preparation_id.clone(),
            selected_backend_id: request.backend_id.clone(),
            bundle_backend_id: "cli:codex".into(),
            ordered_field_names: names,
            optional_account_id_present: account_present,
            issued_at: created_at.clone(),
            expires_at: expires_at.clone(),
            ref_hash: String::new(),
        };
        credential.ref_hash = seal(
            "substrate.e3.credential-source-ref.v1",
            "credential_source",
            "ref_hash",
            serde_json::to_value(&credential).map_err(|_| ConfigProjectionFailureV1::Malformed)?,
        )?;
        let publication = config_projection::E3PreparedRetainedLaunchPublicationV1::new(
            &identity,
            &gateway,
            &credential,
            format!("gsh_{}", uuid::Uuid::now_v7()),
            request.preparation_idempotency_key.clone(),
            created_at.clone(),
            expires_at,
        )?;
        // Establish the fallible expiry mechanism before acquiring any attempt ownership.
        // CLOCK_BOOTTIME includes suspend; the absolute timer is the same authority as retry
        // validation, and waking only schedules cleanup of the still-owned attempt.
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        let timer = unsafe {
            libc::timerfd_create(libc::CLOCK_BOOTTIME, libc::TFD_CLOEXEC | libc::TFD_NONBLOCK)
        };
        if timer < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let timer = unsafe { OwnedFd::from_raw_fd(timer) };
        let setting = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: expires_deadline.seconds,
                tv_nsec: expires_deadline.nanoseconds,
            },
        };
        if unsafe {
            libc::timerfd_settime(
                timer.as_raw_fd(),
                libc::TFD_TIMER_ABSTIME,
                &setting,
                std::ptr::null_mut(),
            )
        } != 0
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let timer = tokio::io::unix::AsyncFd::new(timer)
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let lease = self
            .exclusion
            .acquire_e3_exclusive(&request.world_id, request.world_generation)
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        preparations.insert(
            preparation_id.clone(),
            SealedE3ConfigProjectionPreparationV1 {
                claimed: false,
                ready_closed_ref: None,
                response: None,
                projection: None,
                publication: Some(publication),
                prepared_boundary: None,
                identity: identity.clone(),
                expected_predecessor,
                gateway: gateway.clone(),
                logical,
                fence_id: fence_id.clone(),
                expires_deadline,
                created_at: created_at.clone(),
                credential_ref: credential,
                record_id: format!("cpr_{}", uuid::Uuid::now_v7()),
                launch_input_id: format!("gal_{}", uuid::Uuid::now_v7()),
                activation_intent: None,
                native_plan: None,
                native_source_manifest: None,
                chain_bound: false,
                credential_source: Some(secret),
                terminal: None,
                cleanup_complete: false,
                cleanup_released_consumer: None,
                expiry_task: None,
                gateway_authority: Some(crate::gateway_runtime::E3GatewayRuntimeAuthorityV1::new(
                    lease,
                )),
            },
        );
        let owned = preparations
            .get_mut(&preparation_id)
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let manager = Arc::downgrade(self);
        owned.expiry_task = Some(tokio::spawn(async move {
            if timer.readable().await.is_ok() {
                if let Some(manager) = manager.upgrade() {
                    if manager.recover_expired(None, None).is_err() {
                        tracing::warn!("E3 expiry cleanup remains owned and unresolved");
                    }
                }
            }
        }));
        // Catch only to scrub the map-owned secret before resuming an unwind. The map lock then
        // poisons, and all nonsecret attempt owners remain retained for process recovery.
        let publication = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            owned
                .gateway_authority
                .as_mut()
                .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?
                .bind_dormant_listener()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            use sha2::{Digest, Sha256};
            use std::os::unix::fs::MetadataExt;
            let mount = std::fs::metadata("/sys/fs/cgroup")
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let parent =
                std::fs::metadata(format!("/sys/fs/cgroup/substrate/{}", identity.world_id))
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let parent = config_projection::CanonicalCgroupIdentityV1 {
                cgroup_v2_mount_device_id: mount.dev(),
                cgroup_v2_mount_inode: mount.ino(),
                cgroup_directory_inode: parent.ino(),
                cgroup_relative_path: format!("substrate/{}", identity.world_id),
            };
            for role in [
                config_projection::E3TerminalProcessRoleV1::ManagedGateway,
                config_projection::E3TerminalProcessRoleV1::ReadinessProbe,
                config_projection::E3TerminalProcessRoleV1::Codex,
            ] {
                let registration_id = format!("ecg_{}", uuid::Uuid::now_v7());
                let component = format!(
                    "substrate-e3-{}",
                    &format!("{:x}", Sha256::digest(registration_id.as_bytes()))[..24]
                );
                let mut intent = config_projection::E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: identity.authority_store_id.clone(),
                    series_id: identity.series_id.clone(),
                    effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                    preparation_id: preparation_id.clone(),
                    fence_id: fence_id.clone(),
                    effect: config_projection::E3KernelEffectKindV1::CreateChildCgroup {
                        cgroup_registration_id: registration_id,
                        role,
                        parent_cgroup: parent.clone(),
                        expected_relative_path: format!(
                            "{}/{}",
                            parent.cgroup_relative_path, component
                        ),
                        child_component: component,
                    },
                    created_at: created_at.clone(),
                    intent_hash: String::new(),
                };
                intent.intent_hash = seal(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    "intent_hash",
                    serde_json::to_value(&intent)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?,
                )?;
                self.registry.publish_kernel_effect_intent(
                    &intent,
                    Some(&mut |reference| {
                        owned
                            .gateway_authority
                            .as_mut()
                            .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?
                            .install_dormant_boundary(
                                &intent,
                                reference,
                                config_projection::GatewayAccessPostureV1::DenyAllDormant,
                                false,
                            )
                            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                    }),
                    None,
                )?;
            }
            let namespace = std::fs::metadata("/proc/self/ns/net")
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let mut intent = config_projection::E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: identity.authority_store_id.clone(),
                series_id: identity.series_id.clone(),
                effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                preparation_id: preparation_id.clone(),
                fence_id: fence_id.clone(),
                effect: config_projection::E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: gateway.access_boundary_id.clone(),
                    network_namespace_inode: namespace.ino(),
                    table_name: format!(
                        "substrate_e3_{}",
                        &format!(
                            "{:x}",
                            Sha256::digest(gateway.access_boundary_id.as_bytes())
                        )[..24]
                    ),
                    chain_name: "gateway_output".into(),
                },
                created_at: created_at.clone(),
                intent_hash: String::new(),
            };
            intent.intent_hash = seal(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                "intent_hash",
                serde_json::to_value(&intent).map_err(|_| ConfigProjectionFailureV1::Malformed)?,
            )?;
            self.registry.publish_kernel_effect_intent(
                &intent,
                Some(&mut |reference| {
                    owned
                        .gateway_authority
                        .as_mut()
                        .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?
                        .install_dormant_boundary(
                            &intent,
                            reference,
                            config_projection::GatewayAccessPostureV1::DenyAllDormant,
                            false,
                        )
                        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                }),
                Some(&gateway),
            )?;
            use std::os::fd::AsFd;
            use std::os::unix::fs::OpenOptionsExt;
            let home = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&identity.accepted_home.physical_path)
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            identity
                .accepted_home
                .revalidate_linux_from_fd(home.as_fd())?;
            let metadata = home
                .metadata()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            owned.prepared_boundary = Some(
                owned
                    .gateway_authority
                    .as_mut()
                    .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?
                    .listen_after_dormant_boundary(
                        &gateway,
                        &identity.series_id,
                        &fence_id,
                        u64::from(metadata.uid()),
                        u64::from(metadata.gid()),
                    )
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?,
            );
            complete(owned)
        }));
        let publication = match publication {
            Ok(publication) => publication,
            Err(panic) => {
                drop(owned.credential_source.take());
                std::panic::resume_unwind(panic);
            }
        };
        match publication {
            Ok(response) => Ok(response),
            // The exact publication progress and original live owners remain reachable for retry
            // and abandonment. No error path releases exclusion over an unresolved effect or lease.
            Err(error) if owned.prepared_boundary.is_some() => Err(error),
            Err(error) => {
                drop(owned.credential_source.take());
                drop(preparations);
                // Preserve the original error while retaining any failed cleanup owner.
                let _ = self.recover_expired(Some(&preparation_id), None);
                Err(error)
            }
        }
    }
    #[allow(dead_code)] // E3-F owns the production caller; E3-E uses the bounded harness.
    pub(crate) fn take_for_v2(
        &self,
        request: &transport_api_types::MemberDispatchRequestV2,
    ) -> Result<SealedE3ConfigProjectionPreparationV1, ConfigProjectionFailureV1> {
        request
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let mut preparations = self
            .preparations
            .lock()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let keys = preparations
            .iter()
            .filter(|(_, owned)| {
                owned
                    .response
                    .as_ref()
                    .is_some_and(|response| response.config_projection == request.config_projection)
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if keys.len() != 1 {
            return Err(ConfigProjectionFailureV1::MissingPreparation);
        }
        let key = &keys[0];
        let owned = preparations
            .get_mut(key)
            .ok_or(ConfigProjectionFailureV1::MissingPreparation)?;
        if owned.claimed || owned.terminal.is_some() || owned.cleanup_complete {
            return Err(ConfigProjectionFailureV1::CancelledPreparation);
        }
        let original = owned
            .credential_source
            .as_ref()
            .and_then(|s| s.request.as_ref())
            .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?;
        if original.preparation_id != *key
            || original.preparation_id != owned.credential_ref.preparation_id
            || original.orchestration_session_id != request.orchestration_session_id
            || original.participant_id != request.participant_id
            || original.orchestrator_participant_id != request.orchestrator_participant_id
            || original.parent_participant_id != request.parent_participant_id
            || original.resumed_from_participant_id != request.resumed_from_participant_id
            || original.backend_id != request.backend_id
            || original.protocol != request.protocol
            || original.run_id != request.run_id
            || original.world_id != request.world_id
            || original.world_generation != request.world_generation
            || original.resolved_runtime != request.resolved_runtime
            || original.retained_worker_launch_authority != request.retained_worker_launch_authority
            || request.e2_launch_activation.as_ref() != Some(&original.e2_launch_activation)
            || owned.response.as_ref().is_none_or(|r| {
                r.preparation_idempotency_key != original.preparation_idempotency_key
            })
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        owned.claimed = true;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.activate_prepared_gateway_v1(owned, request)
        }));
        match outcome {
            Ok(Ok(())) => {
                let mut transferred = self
                    .transferred_preparations
                    .lock()
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                // This is the transfer arbitration point: final validation and waiting for the
                // transfer lock must not let an expired owner escape its preparation deadline.
                let mut now = libc::timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                };
                if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut now) } != 0 {
                    drop(transferred);
                    let _ = self.cleanup_prepared_gateway_v1(owned, false);
                    return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
                }
                if (now.tv_sec, now.tv_nsec)
                    >= (
                        owned.expires_deadline.seconds,
                        owned.expires_deadline.nanoseconds,
                    )
                {
                    drop(transferred);
                    let _ = self.cleanup_prepared_gateway_v1(owned, true);
                    return Err(ConfigProjectionFailureV1::ExpiredPreparation);
                }
                transferred.insert(key.clone());
                if let Some(task) = owned.expiry_task.take() {
                    task.abort();
                }
                preparations
                    .remove(key)
                    .ok_or(ConfigProjectionFailureV1::MissingPreparation)
            }
            Ok(Err(error)) => {
                let _ = self.cleanup_prepared_gateway_v1(owned, false);
                Err(error)
            }
            Err(panic) => {
                drop(owned.credential_source.take());
                let _ = self.cleanup_prepared_gateway_v1(owned, false);
                std::panic::resume_unwind(panic)
            }
        }
    }

    fn activate_prepared_gateway_v1(
        &self,
        owned: &mut SealedE3ConfigProjectionPreparationV1,
        request: &transport_api_types::MemberDispatchRequestV2,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let deadline = libc::timespec {
            tv_sec: owned.expires_deadline.seconds,
            tv_nsec: owned.expires_deadline.nanoseconds,
        };
        let e2 = request
            .e2_launch_activation
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
        let mut authenticate = || -> anyhow::Result<()> {
            let mut now = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut now) } != 0
                || (now.tv_sec, now.tv_nsec) >= (deadline.tv_sec, deadline.tv_nsec)
            {
                return Err(ConfigProjectionFailureV1::ExpiredPreparation.into());
            }
            let current = self.authority.authenticate_e3_member_launch_activation_v1(
                MemberDispatchCommonFieldsV1 {
                    orchestration_session_id: &request.orchestration_session_id,
                    participant_id: &request.participant_id,
                    orchestrator_participant_id: &request.orchestrator_participant_id,
                    parent_participant_id: request.parent_participant_id.as_deref(),
                    resumed_from_participant_id: request.resumed_from_participant_id.as_deref(),
                    backend_id: &request.backend_id,
                    protocol: &request.protocol,
                    run_id: &request.run_id,
                    world_id: &request.world_id,
                    world_generation: request.world_generation,
                    initial_prompt: request.initial_prompt.as_deref(),
                    resolved_runtime: &request.resolved_runtime,
                    retained_worker_launch_authority: request
                        .retained_worker_launch_authority
                        .as_ref(),
                },
                e2,
            )?;
            anyhow::ensure!(current == *e2, "E3 activation authority changed");
            Ok(())
        };
        let security_error =
            |_: anyhow::Error| ConfigProjectionFailureV1::UnsupportedSecurityPosture;
        authenticate().map_err(security_error)?;
        let publication = owned
            .publication
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let input = self
            .projection_service
            .resolve_activation_carrier(publication, &request.config_projection)?;
        let runtime = owned
            .gateway_authority
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        runtime
            .spawn_descriptor_pinned(
                &owned.identity,
                &input,
                e2,
                Arc::clone(&self.exclusion),
                &self.registry,
                self.service_instance_id
                    .get()
                    .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
                deadline,
                &mut authenticate,
            )
            .map_err(security_error)?;
        authenticate().map_err(security_error)?;
        let source = owned
            .credential_source
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?;
        let auth = &mut source
            .request
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?
            .integrated_auth;
        let delivered = runtime
            .deliver_secret_after_privilege_drop(auth)
            .map_err(security_error)?;
        drop(owned.credential_source.take());
        let delivered = config_projection::E3ManagedGatewayActivationObservationV1::Delivered {
            delivered_at: delivered,
        };
        for attempt in 0..3 {
            authenticate().map_err(security_error)?;
            runtime.validate_child_security().map_err(security_error)?;
            match self
                .projection_service
                .activate_managed_gateway(publication, delivered.clone())
            {
                Ok(None) => break,
                Err(ConfigProjectionFailureV1::PartialPublication) if attempt < 2 => continue,
                Err(error) => return Err(error),
                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
            }
        }
        runtime
            .spawn_readiness_probe(&self.registry, &mut authenticate)
            .map_err(security_error)?;
        let ready = runtime
            .probe_readiness(&mut authenticate)
            .map_err(security_error)?;
        for attempt in 0..3 {
            authenticate().map_err(security_error)?;
            runtime.validate_child_security().map_err(security_error)?;
            match self
                .projection_service
                .activate_managed_gateway(publication, ready.clone())
            {
                Ok(Some(reference)) => {
                    owned.ready_closed_ref = Some(reference);
                    break;
                }
                Err(ConfigProjectionFailureV1::PartialPublication) if attempt < 2 => continue,
                Err(error) => return Err(error),
                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
            }
        }
        authenticate().map_err(security_error)?;
        runtime.validate_child_security().map_err(security_error)?;
        let readback = self
            .projection_service
            .activate_managed_gateway(publication, ready)?;
        if readback.is_none() || readback != owned.ready_closed_ref {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        authenticate().map_err(security_error)?;
        runtime.validate_child_security().map_err(security_error)?;
        Ok(())
    }

    fn cleanup_prepared_gateway_v1(
        &self,
        owned: &mut SealedE3ConfigProjectionPreparationV1,
        expired: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if owned.cleanup_complete {
            return Ok(());
        }
        if owned.terminal.is_none() {
            owned.terminal = Some((
                if expired {
                    config_projection::SecretHandoffStateV1::Expired
                } else {
                    config_projection::SecretHandoffStateV1::Failed
                },
                config_projection::Timestamp(
                    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                ),
            ));
        }

        drop(owned.credential_source.take());
        // A readback validates posture and equality without acquiring another capability.
        if let Some(lease) = owned
            .publication
            .as_ref()
            .and_then(|p| p.held_consumer_lease())
        {
            let config_projection::ConfigProjectionSubjectReadbackV1::Bound(current) = self
                .projection_service
                .resolve_preparation_subject_v1(&owned.identity)?
            else {
                return Err(ConfigProjectionFailureV1::StaleRevision);
            };
            if current.record().identity != owned.identity
                || current.record().activation.publication_fence
                    != (config_projection::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                        fence_id: owned.fence_id.clone(),
                    })
                || current.record().nonsecret_handoff.credential_source_ref != owned.credential_ref
                || match current.record().managed_gateway.posture {
                    config_projection::ManagedGatewayProjectionPostureV1::Dormant => {
                        current.projection_ref() != &lease.acquired_projection_ref
                    }
                    config_projection::ManagedGatewayProjectionPostureV1::ReadyClosed => {
                        current.record().predecessor_ref.as_ref()
                            != Some(&lease.acquired_projection_ref)
                    }
                    _ => true,
                }
            {
                return Err(ConfigProjectionFailureV1::StaleRevision);
            }
        }
        if let Some(predecessor) = &owned.expected_predecessor {
            let history: Vec<_> = self
                .registry
                .recover(None)?
                .kernel_effects
                .into_iter()
                .filter(|effect| {
                    effect.projection.as_ref().is_some_and(|record| {
                        record.identity == owned.identity && record.revision <= predecessor.revision
                    })
                })
                .collect();
            crate::gateway_runtime::E3GatewayRuntimeAuthorityV1::revoke(
                crate::gateway_runtime::E3GatewayRevocationTargetV1::Recovered {
                    effects: &history,
                    service_instance_id: self
                        .service_instance_id
                        .get()
                        .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
                    exclusion: &self.exclusion,
                },
                &self.registry,
            )
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        }
        crate::gateway_runtime::E3GatewayRuntimeAuthorityV1::revoke(
            crate::gateway_runtime::E3GatewayRevocationTargetV1::Live(
                owned
                    .gateway_authority
                    .as_mut()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
            ),
            &self.registry,
        )
        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let (state, released_at) = owned
            .terminal
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        self.projection_service.publish_preparation_abandonment(
            config_projection::E3PreparationAbandonmentTargetV1::Live(
                owned
                    .publication
                    .as_mut()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
            ),
            *state,
            released_at.clone(),
            None,
        )?;
        if let Some(original) = owned
            .publication
            .as_ref()
            .and_then(|p| p.held_consumer_lease())
        {
            // Reuses the original Held identity and frozen timestamp. This operation reads back
            // only its exact Released successor, including on a lost abandonment return.
            let released = self
                .registry
                .release_consumer_lease(original, released_at.clone())?;
            if owned
                .cleanup_released_consumer
                .as_ref()
                .is_some_and(|prior| prior != &released)
            {
                return Err(ConfigProjectionFailureV1::StaleRevision);
            }
            owned.cleanup_released_consumer = Some(released);
        }
        // A terminal ref alone cannot reach this release: the exact kernel owner has already
        // proved absence and the service has reconciled and released its original lease.
        drop(owned.projection.take());
        owned
            .gateway_authority
            .as_mut()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?
            .release_exclusion_after_cleanup_v1()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        drop(owned.gateway_authority.take());
        owned.cleanup_complete = true;
        if let Some(task) = owned.expiry_task.take() {
            task.abort();
        }
        Ok(())
    }

    pub(crate) fn cancel(
        &self,
        request: transport_api_types::E3ConfigProjectionCancelRequestV1,
    ) -> Result<transport_api_types::E3ConfigProjectionCancelResponseV1, ConfigProjectionFailureV1>
    {
        request
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        {
            let mut preparations = self
                .preparations
                .lock()
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let owned = preparations
                .get_mut(&request.preparation_id)
                .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?;
            let response = owned
                .response
                .as_ref()
                .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
            if response.preparation_idempotency_key != request.preparation_idempotency_key
                || response.config_projection != request.config_projection
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            self.cleanup_prepared_gateway_v1(owned, false)?;
        }
        let preparations = self
            .preparations
            .lock()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let owned = preparations
            .get(&request.preparation_id)
            .ok_or(ConfigProjectionFailureV1::ExpiredPreparation)?;
        if !owned.cleanup_complete {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let mut response = transport_api_types::E3ConfigProjectionCancelResponseV1 {
            schema_version: 1,
            preparation_id: request.preparation_id,
            disposition: transport_api_types::E3ConfigProjectionCancelDispositionV1::Cancelled,
            cancelled_dormant_projection_ref: request.config_projection.dormant_projection_ref,
            response_hash: String::new(),
        };
        let mut value =
            serde_json::to_value(&response).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .remove("response_hash");
        response.response_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.config-projection-cancel-response.v1",
            &serde_json::json!({"response":value}),
        )?;
        response
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        Ok(response)
    }

    pub(crate) fn recover_expired(
        &self,
        failed_preparation: Option<&str>,
        recovered: Option<E3PreparationRecoveryV1<'_>>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if let Some(recovered) = recovered {
            use crate::gateway_runtime::{
                E3GatewayRevocationTargetV1, E3GatewayRuntimeAuthorityV1,
            };
            match recovered {
                E3PreparationRecoveryV1::Startup {
                    service_instance_id,
                } => {
                    self.exclusion
                        .require_recovering_v1()
                        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    self.service_instance_id
                        .set(service_instance_id.to_owned())
                        .map_err(|_| ConfigProjectionFailureV1::Conflict)?;
                    let readback = self.registry.recover(None)?;
                    E3GatewayRuntimeAuthorityV1::revoke(
                        E3GatewayRevocationTargetV1::Recovered {
                            effects: &readback.kernel_effects,
                            service_instance_id,
                            exclusion: &self.exclusion,
                        },
                        &self.registry,
                    )
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    let readback = self.registry.recover(None)?;
                    let mut seen = std::collections::BTreeSet::new();
                    for effect in readback.kernel_effects {
                        let Some(record) = effect.projection else {
                            if effect.resolution.is_none() {
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            }
                            continue;
                        };
                        if seen.insert(record.identity.series_id.clone()) {
                            let config_projection::ConfigProjectionSubjectReadbackV1::Bound(
                                metadata,
                            ) = self
                                .projection_service
                                .resolve_preparation_subject_v1(&record.identity)?
                            else {
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            };
                            self.recover_expired(
                                None,
                                Some(E3PreparationRecoveryV1::Subject(&metadata)),
                            )?;
                        }
                    }
                    return Ok(());
                }
                E3PreparationRecoveryV1::Subject(expected) => {
                    let preparations = self
                        .preparations
                        .lock()
                        .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    if preparations.values().any(|owner| {
                        owner.identity == expected.record().identity && !owner.cleanup_complete
                    }) {
                        return Err(ConfigProjectionFailureV1::Conflict);
                    }
                    let config_projection::ConfigProjectionSubjectReadbackV1::Bound(current) = self
                        .projection_service
                        .resolve_preparation_subject_v1(&expected.record().identity)?
                    else {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    };
                    if current.record() != expected.record()
                        || current.projection_ref() != expected.projection_ref()
                    {
                        return Err(ConfigProjectionFailureV1::StaleRevision);
                    }
                    let readback = self.registry.recover(Some(&current.record().identity))?;
                    E3GatewayRuntimeAuthorityV1::revoke(
                        E3GatewayRevocationTargetV1::Recovered {
                            effects: &readback.kernel_effects,
                            service_instance_id: self
                                .service_instance_id
                                .get()
                                .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
                            exclusion: &self.exclusion,
                        },
                        &self.registry,
                    )
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    if current.record().managed_gateway.posture
                        == config_projection::ManagedGatewayProjectionPostureV1::Active
                    {
                        // The registry validates terminal handoff/leases/evidence and Recovered
                        // verifies current absence. An Active epoch never enters abandonment.
                        let config_projection::ConfigProjectionSubjectReadbackV1::Bound(after) =
                            self.projection_service
                                .resolve_preparation_subject_v1(&current.record().identity)?
                        else {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        };
                        if after.record() != current.record()
                            || after.projection_ref() != current.projection_ref()
                        {
                            return Err(ConfigProjectionFailureV1::StaleRevision);
                        }
                        return Ok(());
                    }
                    self.projection_service.publish_preparation_abandonment(
                        config_projection::E3PreparationAbandonmentTargetV1::Recovered(&current),
                        config_projection::SecretHandoffStateV1::Failed,
                        config_projection::Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        None,
                    )?;
                    return Ok(());
                }
            }
        }
        let mut clock = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut clock) } != 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut preparations = self
            .preparations
            .lock()
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        for (preparation_id, owned) in preparations.iter_mut() {
            if owned.cleanup_complete {
                continue;
            }
            let expired = (clock.tv_sec, clock.tv_nsec)
                >= (
                    owned.expires_deadline.seconds,
                    owned.expires_deadline.nanoseconds,
                );
            if owned.terminal.is_none()
                && !expired
                && failed_preparation != Some(preparation_id.as_str())
            {
                continue;
            }
            self.cleanup_prepared_gateway_v1(owned, expired)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use config_projection::{
        CanonicalDirectoryV1 as ProjectionDirectoryV1, ConfigProjectionCodecV1,
        ConfigProjectionHsaAuthorityV1, ConfigProjectionRegistryV1,
        ConfiguredAcceptedHomeAuthorityV1, InstalledAcceptedHomeBootstrapRecordV1, Timestamp,
    };
    use sha2::{Digest, Sha256};
    use std::fs::{self, File};
    use std::os::fd::{AsFd, BorrowedFd};
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use transport_api_types::*;

    fn current_account(uid: libc::uid_t) -> String {
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; 16 * 1024];
        assert_eq!(
            unsafe {
                libc::getpwuid_r(
                    uid,
                    pwd.as_mut_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut result,
                )
            },
            0
        );
        assert!(!result.is_null());
        unsafe { std::ffi::CStr::from_ptr((*result).pw_name) }
            .to_str()
            .unwrap()
            .to_string()
    }

    fn configured(home: &std::path::Path) -> ConfiguredAcceptedHomeAuthorityV1 {
        use std::os::unix::fs::MetadataExt;
        let descriptor = File::open(home).unwrap();
        let accepted_home =
            ProjectionDirectoryV1::capture_linux_from_fd(descriptor.as_fd()).unwrap();
        let metadata = descriptor.metadata().unwrap();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix(
                accepted_home.physical_path.as_str(),
                &current_account(uid),
                uid,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().unwrap(),
            host_context_commitment: carrier.host_context_commitment,
            intended_account: current_account(uid),
            intended_uid: u64::from(uid),
            intended_gid: u64::from(gid),
            accepted_home,
            installed_at: Timestamp("2026-09-10T00:00:00.000000Z".to_string()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("record_hash");
        let mut payload = BTreeMap::new();
        payload.insert("record", value);
        record.record_hash = ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &payload,
        )
        .unwrap();
        ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(record).unwrap()
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
                binary_path: "/var/lib/substrate/world-deps/codex-runtime/bin/codex".into(),
            },
            retained_worker_launch_authority: Some(sample_retained_worker_launch_authority_proof()),
            e2_launch_activation: Some(e3a_sample_fresh_spawn_activation()),
            config_projection: e3a_sample_projection_carrier(),
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
        let mut nonsecret = serde_json::to_value(&request).unwrap();
        nonsecret.as_object_mut().unwrap().remove("integrated_auth");
        nonsecret
            .as_object_mut()
            .unwrap()
            .remove("preparation_idempotency_key");
        let payload = serde_json::json!({
            "credential_shape": {"backend_id": request.backend_id, "ordered_field_names": ["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"]},
            "request": nonsecret,
        });
        request.preparation_idempotency_key = format!(
            "e3pik_{}",
            ConfigProjectionCodecV1::domain_sha256(
                "substrate.e3.config-projection-prepare-idempotency.v1",
                &payload
            )
            .unwrap()
        );
        request
    }

    struct OrderingParent {
        root: PathBuf,
        fail: AtomicBool,
        reads: AtomicUsize,
    }
    impl ConfigProjectionHsaAuthorityV1 for OrderingParent {
        fn with_locked_parent(
            &self,
            operation: &mut dyn for<'fd> FnMut(
                BorrowedFd<'fd>,
            )
                -> Result<(), ConfigProjectionFailureV1>,
        ) -> Result<(), ConfigProjectionFailureV1> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                return Err(ConfigProjectionFailureV1::HashInvalid);
            }
            let root = File::open(self.root.join("authority-v1"))
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            operation(root.as_fd())
        }
    }

    #[tokio::test]
    async fn e3_e_preparation_resolves_authoring_before_authentication_and_attempt_ownership() {
        let temp =
            tempfile::tempdir_in(PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache"))
                .unwrap();
        let home = temp.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir(home.join("authority-v1")).unwrap();
        fs::set_permissions(home.join("authority-v1"), fs::Permissions::from_mode(0o700)).unwrap();
        let accepted = Arc::new(configured(&home));
        let authority = Arc::new(
            substrate_shell::OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(
                &accepted,
            )
            .unwrap(),
        );
        let parent = Arc::new(OrderingParent {
            root: home,
            fail: AtomicBool::new(false),
            reads: AtomicUsize::new(0),
        });
        let registry = Arc::new(ConfigProjectionRegistryV1::open(parent.clone()).unwrap());
        let service =
            Arc::new(AgentConfigProjectionServiceV1::new(registry.clone(), accepted).unwrap());
        // Keep exclusion in Recovering: this test cannot admit kernel effects or children.
        let exclusion =
            crate::e3_child_security::E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        let manager = Arc::new(E3ConfigProjectionPreparationManagerV1::new(
            authority, service, registry, exclusion,
        ));
        parent.fail.store(true, Ordering::SeqCst);
        let before = parent.reads.load(Ordering::SeqCst);
        let request = e3e_preparation_fixture();
        request.validate().unwrap();
        let wire = serde_json::to_vec(&request).unwrap();
        let result = manager.prepare(hyper::Body::from(wire)).await;
        assert_eq!(result, Err(ConfigProjectionFailureV1::HashInvalid));
        assert_eq!(parent.reads.load(Ordering::SeqCst), before + 1);
        assert!(manager.preparations.lock().unwrap().is_empty());
        // Malformed ingress must not even begin the authoring transaction.
        let before = parent.reads.load(Ordering::SeqCst);
        assert_eq!(
            manager.prepare(hyper::Body::from("{}")).await,
            Err(ConfigProjectionFailureV1::Malformed)
        );
        assert_eq!(parent.reads.load(Ordering::SeqCst), before);
        assert!(manager.preparations.lock().unwrap().is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "explicit test-owned authenticated HSA fixture and root cgroup/netfilter environment"]
    async fn e3_e_authenticated_preparation_response_and_cancellation() -> anyhow::Result<()> {
        use anyhow::Context;
        use config_projection::*;
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        anyhow::ensure!(
            unsafe { libc::geteuid() } == 0,
            "isolated manager fixture requires root"
        );
        let recovery_phase = std::env::var("E3_E_RECOVERY_PHASE").ok();
        let live_registered_restart =
            std::env::var_os("E3_E_LIVE_REGISTERED_RESTART_FIXTURE").is_some();
        let registered_restart = live_registered_restart
            || std::env::var_os("E3_E_REGISTERED_RESTART_FIXTURE").is_some();
        if recovery_phase.as_deref() == Some("active-series") {
            for (phase, expected) in [("active-writer", 79), ("active-recovered", 0)] {
                let status = std::process::Command::new(std::env::current_exe()?)
                    .args([
                        "e3_e_authenticated_preparation_response_and_cancellation",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("E3_E_RECOVERY_PHASE", phase)
                    .status()?;
                anyhow::ensure!(status.code() == Some(expected), "{phase} failed: {status}");
            }
            eprintln!("E3_SYNTHETIC_TERMINAL_ACTIVE_ACTUAL_PROCESS_REOPEN_PASSED");
            return Ok(());
        }
        if recovery_phase.is_none() {
            for (phase, expected) in [
                (
                    "writer",
                    if live_registered_restart {
                        77
                    } else if registered_restart {
                        75
                    } else {
                        73
                    },
                ),
                ("recovered-boundary", 74),
                ("recovered", 0),
            ] {
                let status = std::process::Command::new(std::env::current_exe()?)
                    .args([
                        "e3_e_authenticated_preparation_response_and_cancellation",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("E3_E_RECOVERY_PHASE", phase)
                    .status()?;
                anyhow::ensure!(
                    status.code() == Some(expected),
                    "{phase} process failed: {status}"
                );
            }
            eprintln!("E3_RECOVERED_SEPARATE_PROCESS_INTERRUPTION_AND_RETRY_PASSED");
            return Ok(());
        }
        let fixture_path = std::env::var_os("E3_E_AUTHENTICATED_MANAGER_FIXTURE")
            .ok_or_else(|| anyhow::anyhow!("explicit authenticated fixture required"))?;
        let artifact_path = std::env::var_os("E3_E_MANAGER_ARTIFACT_FIXTURE_READ")
            .ok_or_else(|| anyhow::anyhow!("explicit synthetic artifact fixture required"))?;
        let fixture: serde_json::Value = serde_json::from_slice(&fs::read(fixture_path)?)?;
        let home = PathBuf::from(fixture["accepted_home_path"].as_str().unwrap());
        let workspace = PathBuf::from(fixture["workspace_root_path"].as_str().unwrap());
        let accepted = Arc::new(configured(&home));
        let authority = Arc::new(
            substrate_shell::OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(
                &accepted,
            )
            .context("open authenticated fixture authority")?,
        );
        struct InterruptedParent {
            inner: Arc<substrate_shell::OpenedConfigProjectionHsaAuthorityV1>,
            interrupt_after: Mutex<Option<PathBuf>>,
        }
        impl ConfigProjectionHsaAuthorityV1 for InterruptedParent {
            fn with_locked_parent(
                &self,
                operation: &mut dyn for<'fd> FnMut(
                    BorrowedFd<'fd>,
                )
                    -> Result<(), ConfigProjectionFailureV1>,
            ) -> Result<(), ConfigProjectionFailureV1> {
                self.inner.with_locked_parent(operation)?;
                let mut marker = self.interrupt_after.lock().unwrap();
                if marker.as_ref().is_some_and(|path| path.exists()) {
                    marker.take();
                    if std::env::var("E3_E_RECOVERY_PHASE").as_deref() == Ok("writer") {
                        // Simulate loss of the entire service process, without running owner drops.
                        unsafe { libc::_exit(73) };
                    }
                    if std::env::var("E3_E_RECOVERY_PHASE").as_deref() == Ok("recovered-boundary") {
                        unsafe { libc::_exit(74) };
                    }
                    return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
                }
                Ok(())
            }
        }
        let interrupted = Arc::new(InterruptedParent {
            inner: authority.clone(),
            interrupt_after: Mutex::new(None),
        });
        let registry = Arc::new(
            ConfigProjectionRegistryV1::open(interrupted.clone())
                .context("open fixture projection registry")?,
        );
        authority
            .with_locked_parent(&mut |_| {
                eprintln!("E3_MANAGER_AUTHENTICATED_PARENT_CALLBACK_REACHED");
                Ok(())
            })
            .context("enter authenticated fixture parent transaction before registry creation")?;
        let store = registry
            .recover(None)
            .map(|readback| readback.store)
            .context("recover fixture registry")?;
        if recovery_phase.as_deref() == Some("active-recovered") {
            let before = registry.recover(None)?;
            let record = before
                .kernel_effects
                .iter()
                .find_map(|effect| effect.projection.as_ref())
                .context("Active fixture missing")?;
            anyhow::ensure!(
                record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active,
                "fixture is not Active"
            );
            let service = Arc::new(AgentConfigProjectionServiceV1::new(
                registry.clone(),
                accepted,
            )?);
            let exclusion =
                crate::e3_child_security::E3PrivilegedChildExclusionV1::new_recovering()?;
            let manager = E3ConfigProjectionPreparationManagerV1::new(
                authority,
                service.clone(),
                registry.clone(),
                exclusion.clone(),
            );
            anyhow::ensure!(
                manager.preparations.lock().unwrap().is_empty()
                    && exclusion.acquire_non_e3_child().is_err(),
                "startup admitted work or reconstructed preparation"
            );
            if std::env::var_os("E3_E_ACTIVE_LATER_HEADLESS").is_some() {
                let headless = before
                    .kernel_effects
                    .iter()
                    .find(|effect| effect.projection.is_none())
                    .context("later headless fixture absent")?;
                anyhow::ensure!(
                    headless.intent.series_id == record.identity.series_id
                        && headless.resolution.is_none()
                        && headless.child_cgroup.is_none(),
                    "later fixture not unresolved and unregistered"
                );
                let E3KernelEffectKindV1::CreateChildCgroup {
                    expected_relative_path,
                    ..
                } = &headless.intent.effect
                else {
                    anyhow::bail!("headless fixture not a cgroup");
                };
                anyhow::ensure!(
                    PathBuf::from("/sys/fs/cgroup")
                        .join(expected_relative_path)
                        .is_dir(),
                    "later headless effect already absent"
                );
            }
            manager.recover_expired(
                None,
                Some(E3PreparationRecoveryV1::Startup {
                    service_instance_id: &format!("wsi_{}", uuid::Uuid::now_v7()),
                }),
            )?;
            exclusion.require_recovering_v1()?;
            let ConfigProjectionSubjectReadbackV1::Bound(metadata) =
                service.resolve_preparation_subject_v1(&record.identity)?
            else {
                anyhow::bail!("Active subject absent");
            };
            anyhow::ensure!(metadata.record() == record, "startup changed Active epoch");
            anyhow::ensure!(
                service
                    .publish_preparation_abandonment(
                        E3PreparationAbandonmentTargetV1::Recovered(&metadata),
                        SecretHandoffStateV1::Failed,
                        record.created_at.clone(),
                        None
                    )
                    .is_err(),
                "Active entered preparation abandonment"
            );
            exclusion.finish_recovery()?;
            manager.recover_expired(None, Some(E3PreparationRecoveryV1::Subject(&metadata)))?;
            anyhow::ensure!(
                exclusion.acquire_non_e3_child().is_ok()
                    && manager.preparations.lock().unwrap().is_empty(),
                "terminal observation retained a preparation owner"
            );
            let after = registry.recover(None)?;
            for old in &before.kernel_effects {
                let exact = after
                    .kernel_effects
                    .iter()
                    .find(|effect| effect.intent == old.intent)
                    .context("Active history disappeared")?;
                anyhow::ensure!(
                    exact.projection == old.projection
                        && (exact.resolution == old.resolution || (old.projection.is_none() && old.resolution.is_none() && exact.child_cgroup.is_none() && exact.resolution.as_ref().is_some_and(|resolution| resolution.disposition == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent)))
                        && exact.boundary == old.boundary
                        && exact.terminal_child_evidence == old.terminal_child_evidence,
                    "Active retry rewrote terminal objects or observations"
                );
            }
            fs::remove_dir(
                PathBuf::from("/sys/fs/cgroup/substrate").join(&record.identity.world_id),
            )?;
            if std::env::var_os("E3_E_ACTIVE_LATER_HEADLESS").is_some() {
                eprintln!(
                    "E3_ACTIVE_STARTUP_LATER_HEADLESS_RESOLVED_BEFORE_ELIGIBILITY_AND_ADMISSION"
                );
            }
            eprintln!("E3_ACTIVE_STARTUP_SUBJECT_EXACT_TERMINAL_OBSERVATION_WITHOUT_ABANDONMENT");
            return Ok(());
        }
        if matches!(
            recovery_phase.as_deref(),
            Some("recovered" | "recovered-boundary" | "recovered-parent-absent")
        ) {
            return (|| -> anyhow::Result<()> {
                use crate::gateway_runtime::{
                    E3GatewayRevocationTargetV1, E3GatewayRuntimeAuthorityV1,
                };
                let exclusion =
                    crate::e3_child_security::E3PrivilegedChildExclusionV1::new_recovering()?;
                let instance = format!("wsi_{}", uuid::Uuid::now_v7());
                let before = registry.recover(None)?;
                let record = before
                    .kernel_effects
                    .iter()
                    .find_map(|effect| effect.projection.as_ref())
                    .context("restarted fixture has no retained projection")?;
                let identity = record.identity.clone();
                let evidence = before
                    .kernel_effects
                    .iter()
                    .flat_map(|effect| &effect.terminal_child_evidence)
                    .map(ConfigProjectionCodecV1::encode_canonical_json)
                    .collect::<Result<Vec<_>, _>>()?;
                anyhow::ensure!(
                    !evidence.is_empty()
                        || (registered_restart
                            && recovery_phase.as_deref() == Some("recovered-boundary")),
                    "interrupted terminal evidence absent"
                );
                if registered_restart && !evidence.is_empty() {
                    let processes: Vec<_> = before
                        .kernel_effects
                        .iter()
                        .flat_map(|effect| &effect.child_processes)
                        .collect();
                    anyhow::ensure!(processes.len() == 1, "registered restart process missing");
                    let original = before
                        .kernel_effects
                        .iter()
                        .flat_map(|effect| &effect.terminal_child_evidence)
                        .find_map(|proof| proof.ordered_terminal_processes.first())
                        .context("recovered process observation missing")?;
                    anyhow::ensure!(
                        if live_registered_restart {
                            matches!(
                                original.observation,
                                E3TerminalProcessObservationKindV1::RecoveryObservedTerminal {
                                    pidfd_open_errno: None,
                                    pidfd_kill_errno: None,
                                    pidfd_became_readable: Some(true),
                                    waitid_errno: Some(libc::ECHILD),
                                    proc_identity: E3RecoveryProcIdentityV1::Absent,
                                    ..
                                }
                            )
                        } else {
                            matches!(
                                original.observation,
                                E3TerminalProcessObservationKindV1::RecoveryObservedTerminal {
                                    pidfd_open_errno: Some(libc::ESRCH),
                                    proc_identity: E3RecoveryProcIdentityV1::Absent,
                                    ..
                                }
                            )
                        },
                        "recovery did not retain actual PID absence observation"
                    );
                }
                let config = before
                    .kernel_effects
                    .iter()
                    .find_map(|effect| effect.gateway_config.as_ref())
                    .context("recorded config missing")?
                    .clone();
                let config_root = PathBuf::from(&config.root.physical_path);
                if let Some(group) = before
                    .kernel_effects
                    .iter()
                    .find_map(|effect| effect.child_cgroup.as_ref())
                {
                    let path =
                        PathBuf::from("/sys/fs/cgroup").join(&group.cgroup.cgroup_relative_path);
                    if path.exists() && !live_registered_restart {
                        let descendant = path.join("unclassified-recovered-fixture");
                        fs::create_dir(&descendant)?;
                        let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                            E3GatewayRevocationTargetV1::Recovered {
                                effects: &before.kernel_effects,
                                service_instance_id: &instance,
                                exclusion: &exclusion,
                            },
                            &registry,
                        );
                        anyhow::ensure!(
                            rejected.is_err() && config_root.exists(),
                            "recovered descendant did not retain cleanup"
                        );
                        exclusion.require_recovering_v1()?;
                        fs::remove_dir(descendant)?;
                    }
                }
                if recovery_phase.as_deref() == Some("recovered-boundary") {
                    let boundary = before
                        .kernel_effects
                        .iter()
                        .find_map(|effect| effect.boundary.as_ref())
                        .context("restart boundary missing")?;
                    anyhow::ensure!(
                        boundary.posture == GatewayAccessPostureV1::DenyAllDormant,
                        "interruption requires original Deny boundary"
                    );
                    *interrupted.interrupt_after.lock().unwrap() = Some(
                        home.join("authority-v1/agent-config-projection-v1/gateway-boundaries")
                            .join(&boundary.access_boundary_id)
                            .join(format!("{:020}.json", boundary.revision + 1)),
                    );
                }
                if config_root.exists() {
                    let unexpected = config_root.join("unexpected-recovered-fixture");
                    fs::write(&unexpected, b"nonsecret fixture")?;
                    let readback = registry.recover(None)?;
                    let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                        E3GatewayRevocationTargetV1::Recovered {
                            effects: &readback.kernel_effects,
                            service_instance_id: &instance,
                            exclusion: &exclusion,
                        },
                        &registry,
                    );
                    anyhow::ensure!(
                        rejected.is_err() && config_root.join("config.toml").is_file(),
                        "recovered config failure did not retain cleanup"
                    );
                    exclusion.require_recovering_v1()?;
                    fs::remove_file(unexpected)?;
                }
                for attempt in 0..2 {
                    let readback = registry.recover(None)?;
                    if attempt == 1 && recovery_phase.as_deref() != Some("recovered-parent-absent")
                    {
                        let registration = readback
                            .kernel_effects
                            .iter()
                            .find_map(|effect| effect.child_cgroup.as_ref())
                            .context("resolved group binding missing")?;
                        let path = PathBuf::from("/sys/fs/cgroup")
                            .join(&registration.cgroup.cgroup_relative_path);
                        fs::create_dir(&path)?;
                        let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                            E3GatewayRevocationTargetV1::Recovered {
                                effects: &readback.kernel_effects,
                                service_instance_id: &instance,
                                exclusion: &exclusion,
                            },
                            &registry,
                        );
                        anyhow::ensure!(
                            rejected.is_err(),
                            "resolved retry accepted a substituted cgroup"
                        );
                        fs::remove_dir(path)?;
                        exclusion.require_recovering_v1()?;
                    }
                    E3GatewayRuntimeAuthorityV1::revoke(
                        E3GatewayRevocationTargetV1::Recovered {
                            effects: &readback.kernel_effects,
                            service_instance_id: &instance,
                            exclusion: &exclusion,
                        },
                        &registry,
                    )
                    .with_context(|| format!("actual Recovered branch attempt {attempt}"))?;
                    let current = registry.recover(None)?;
                    let after = current
                        .kernel_effects
                        .iter()
                        .flat_map(|effect| &effect.terminal_child_evidence)
                        .map(ConfigProjectionCodecV1::encode_canonical_json)
                        .collect::<Result<Vec<_>, _>>()?;
                    anyhow::ensure!(
                        after == evidence,
                        "Recovered retry rewrote terminal evidence"
                    );
                    anyhow::ensure!(
                        current
                            .kernel_effects
                            .iter()
                            .all(|effect| effect.resolution.is_some()),
                        "unresolved recovered effects"
                    );
                    anyhow::ensure!(
                        !PathBuf::from(&config.root.physical_path).exists(),
                        "recorded config remains after recovery"
                    );
                }
                let service = Arc::new(AgentConfigProjectionServiceV1::new(
                    registry.clone(),
                    accepted,
                )?);
                let expired_bytes = if registered_restart {
                    let ConfigProjectionSubjectReadbackV1::Bound(metadata) =
                        service.resolve_preparation_subject_v1(&identity)?
                    else {
                        anyhow::bail!("expiry fixture subject unbound");
                    };
                    service.publish_preparation_abandonment(
                        E3PreparationAbandonmentTargetV1::Recovered(&metadata),
                        SecretHandoffStateV1::Expired,
                        Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        None,
                    )?;
                    let root = home.join("authority-v1/agent-config-projection-v1");
                    let handoff = root
                        .join("handoffs")
                        .join(&record.nonsecret_handoff.secret_handoff_ref.handoff_id)
                        .join("head.json");
                    let consumer = format!(
                        "cpc_{}",
                        record
                            .nonsecret_handoff
                            .credential_source_ref
                            .preparation_id
                            .strip_prefix("e3p_")
                            .unwrap()
                    );
                    let lease = root
                        .join("leases")
                        .join(&identity.series_id)
                        .join(consumer)
                        .join("head.json");
                    Some([
                        (handoff.clone(), fs::read(handoff)?),
                        (lease.clone(), fs::read(lease)?),
                    ])
                } else {
                    None
                };
                let manager = E3ConfigProjectionPreparationManagerV1::new(
                    authority.clone(),
                    service.clone(),
                    registry.clone(),
                    exclusion.clone(),
                );
                anyhow::ensure!(
                    manager.preparations.lock().unwrap().is_empty(),
                    "restart reconstructed an old owner"
                );
                manager.recover_expired(
                    None,
                    Some(E3PreparationRecoveryV1::Startup {
                        service_instance_id: &instance,
                    }),
                )?;
                anyhow::ensure!(
                    manager
                        .recover_expired(
                            None,
                            Some(E3PreparationRecoveryV1::Startup {
                                service_instance_id: &instance
                            })
                        )
                        .is_err(),
                    "duplicate startup replaced process identity"
                );
                let ConfigProjectionSubjectReadbackV1::Bound(metadata) =
                    service.resolve_preparation_subject_v1(&identity)?
                else {
                    anyhow::bail!("recovered subject unbound");
                };
                anyhow::ensure!(
                    exclusion.acquire_non_e3_child().is_err(),
                    "Recovered branch opened admission"
                );
                exclusion.finish_recovery()?;
                manager.recover_expired(None, Some(E3PreparationRecoveryV1::Subject(&metadata)))?;
                let world_parent =
                    PathBuf::from("/sys/fs/cgroup/substrate").join(&identity.world_id);
                if recovery_phase.as_deref() == Some("recovered-parent-absent") {
                    anyhow::ensure!(
                        !world_parent.exists(),
                        "absent-parent fixture regained a parent"
                    );
                    eprintln!("E3_RECOVERED_REGISTERED_PARENT_ABSENCE_AND_ORIGINAL_LIVE_TERMINAL_PROOF_REUSED");
                } else {
                    fs::remove_dir(world_parent)?;
                }
                anyhow::ensure!(
                    manager.preparations.lock().unwrap().is_empty(),
                    "recovered abandonment constructed an owner"
                );
                if let Some(retained) = expired_bytes {
                    for (path, bytes) in retained {
                        anyhow::ensure!(
                            fs::read(path)? == bytes,
                            "startup rewrote Expired handoff or released lease"
                        );
                    }
                    eprintln!("E3_RECOVERED_EXPIRED_HANDOFF_AND_RELEASE_BYTES_REUSED");
                }
                eprintln!("E3_RECOVERED_DESCENDANT_CONFIG_FAILURE_AND_STARTUP_GATE_PASSED");
                eprintln!("E3_RECOVERED_ORIGINAL_EVIDENCE_CONFIG_REMOVAL_AND_ABANDONMENT_PASSED");
                Ok(())
            })();
        }
        let service = Arc::new(
            AgentConfigProjectionServiceV1::new(registry.clone(), accepted)
                .context("open fixture projection service")?,
        );
        let seal = |domain: &str,
                    key: &str,
                    field: &str,
                    mut value: serde_json::Value|
         -> anyhow::Result<String> {
            value.as_object_mut().unwrap().remove(field);
            Ok(ConfigProjectionCodecV1::domain_sha256(
                domain,
                &serde_json::json!({key:value}),
            )?)
        };
        let mut effective = EffectiveSubstrateConfigSourceV1 {
            schema_version: 1,
            authority_store_id: store.authority_store_id.clone(),
            accepted_home: store.accepted_home.clone(),
            workspace_root: ProjectionDirectoryV1::capture_linux_from_fd(
                File::open(&workspace)?.as_fd(),
            )?,
            values: E3EffectiveConfigInputV1 {
                llm_enabled: true,
                agents_enabled: true,
                world_enabled: true,
                default_execution_scope: "world".into(),
                default_cli_mode: "persistent".into(),
                managed_gateway_enabled: true,
                managed_gateway_mode: "in_world".into(),
                default_backend_id: "cli:codex-world".into(),
            },
            ordered_explain_origins: [
                "llm.enabled",
                "llm.gateway.enabled",
                "llm.gateway.mode",
                "llm.routing.default_backend",
                "agents.enabled",
                "agents.defaults.execution.scope",
                "agents.defaults.cli.mode",
                "world.enabled",
            ]
            .into_iter()
            .map(|key| E3ConfigExplainOriginV1 {
                key: key.into(),
                source_kind: E3ConfigExplainOriginKindV1::Default,
                source_location: None,
            })
            .collect(),
            source_revision: String::new(),
            source_hash: String::new(),
        };
        let mut revision = serde_json::to_value(&effective)?;
        revision.as_object_mut().unwrap().remove("source_revision");
        revision.as_object_mut().unwrap().remove("source_hash");
        effective.source_revision = format!(
            "ecsr1_{:x}",
            Sha256::digest(ConfigProjectionCodecV1::encode_canonical_json(&revision)?)
        );
        effective.source_hash = seal(
            "substrate.e3.effective-substrate-config-source.v1",
            "source",
            "source_hash",
            serde_json::to_value(&effective)?,
        )?;
        let inventory_path = home.join("agents/codex.yaml");
        let metadata = fs::metadata(&inventory_path)?;
        let raw = fs::read(&inventory_path)?;
        let hash = format!("{:x}", Sha256::digest(&raw));
        let mut inventory = AgentInventorySourceMaterialV1 {
            inventory_scope: "global".into(),
            accepted_root: store.accepted_home.clone(),
            relative_path: "agents/codex.yaml".into(),
            file_device_id: metadata.dev(),
            file_inode: metadata.ino(),
            byte_length: raw.len() as u64,
            raw_bytes_sha256: hash.clone(),
            source_revision: format!("aisr1_{hash}"),
            source_hash: String::new(),
        };
        inventory.source_hash = seal(
            "substrate.e3.agent-inventory-source.v1",
            "source",
            "source_hash",
            serde_json::to_value(&inventory)?,
        )?;
        // The installer boundary is an explicit synthetic input. HSA/source authentication,
        // native publication, listener/cgroup/config identities and lifecycle below are real.
        // No artifact is executed and this fixture never becomes installed provenance.
        let mut manifest: TrustedRuntimeArtifactManifestV1 =
            serde_json::from_slice(&fs::read(artifact_path)?)?;
        manifest.authority_store_id = store.authority_store_id.clone();
        manifest.manifest_hash = seal(
            "substrate.e3.runtime-artifact-manifest.v1",
            "manifest",
            "manifest_hash",
            serde_json::to_value(&manifest)?,
        )?;
        let child = home.join("authority-v1/agent-config-projection-v1");
        let owner = fs::metadata(&home)?;
        let manifest_dir = child.join("runtime-artifacts").join(&manifest.manifest_id);
        if recovery_phase.as_deref() != Some("resume-fresh") {
            fs::create_dir(&manifest_dir)?;
            std::os::unix::fs::chown(&manifest_dir, Some(owner.uid()), Some(owner.gid()))?;
            fs::set_permissions(&manifest_dir, fs::Permissions::from_mode(0o700))?;
        }
        for (path, bytes) in [
            (
                child
                    .join("inputs/effective-config")
                    .join(format!("{}.json", effective.source_hash)),
                ConfigProjectionCodecV1::encode_canonical_json(&effective)?,
            ),
            (
                child
                    .join("inputs/agent-inventory")
                    .join(format!("{}.json", inventory.source_hash)),
                ConfigProjectionCodecV1::encode_canonical_json(&inventory)?,
            ),
            (
                manifest_dir.join("00000000000000000001.json"),
                ConfigProjectionCodecV1::encode_canonical_json(&manifest)?,
            ),
        ] {
            if recovery_phase.as_deref() == Some("resume-fresh") {
                anyhow::ensure!(
                    fs::read(&path)? == bytes,
                    "restart changed immutable authoring fixture bytes"
                );
                continue;
            }
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
            file.write_all(&bytes)?;
            std::os::unix::fs::chown(&path, Some(owner.uid()), Some(owner.gid()))?;
            file.sync_all()?;
            File::open(path.parent().unwrap())?.sync_all()?;
        }
        let mut input_ref = ConfigProjectionAuthoringInputRefV1 {
            authority_store_id: store.authority_store_id.clone(),
            effective_config_source_hash: effective.source_hash,
            agent_inventory_source_hash: inventory.source_hash,
            runtime_artifact_manifest_id: manifest.manifest_id,
            runtime_artifact_manifest_revision: manifest.revision,
            runtime_artifact_manifest_hash: manifest.manifest_hash,
            input_ref_hash: String::new(),
        };
        input_ref.input_ref_hash = input_ref.canonical_hash().map_err(anyhow::Error::msg)?;
        registry.resolve(None, Some(&input_ref))?;
        let activation: E2MemberLaunchActivationCarrierV1 =
            serde_json::from_value(fixture["e2_launch_activation"].clone())?;
        let mut request = e3e_preparation_fixture();
        request.preparation_id = format!("e3p_{}", uuid::Uuid::now_v7());
        request.orchestration_session_id = activation.orchestration_session_id.clone();
        request.participant_id = activation.retained_participant_id.clone();
        request.orchestrator_participant_id = activation.caller_participant_id.clone();
        request.run_id = activation.bootstrap_run_id.clone();
        request.world_id = activation.target_world.world_id.clone();
        request.world_generation = activation.target_world.world_generation;
        request.authoring_input_ref = input_ref;
        request.retained_worker_launch_authority = Some(serde_json::from_value(
            fixture["retained_worker_launch_authority"].clone(),
        )?);
        request.e2_launch_activation = activation;
        let mut nonsecret = serde_json::to_value(&request)?;
        nonsecret.as_object_mut().unwrap().remove("integrated_auth");
        nonsecret
            .as_object_mut()
            .unwrap()
            .remove("preparation_idempotency_key");
        request.preparation_idempotency_key = format!(
            "e3pik_{}",
            ConfigProjectionCodecV1::domain_sha256(
                "substrate.e3.config-projection-prepare-idempotency.v1",
                &serde_json::json!({
                    "credential_shape": {"backend_id": request.backend_id, "ordered_field_names": ["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"]},
                    "request": nonsecret,
                })
            )?
        );
        request.validate().map_err(anyhow::Error::msg)?;
        let world_parent = PathBuf::from("/sys/fs/cgroup/substrate").join(&request.world_id);
        if recovery_phase.as_deref() != Some("resume-fresh") {
            fs::create_dir(&world_parent)?;
        }
        eprintln!("E3_MANAGER_TEST_WORLD {}", request.world_id);
        let prior_head = if recovery_phase.as_deref() == Some("resume-fresh") {
            let readback = registry.recover(None)?;
            let record = readback
                .kernel_effects
                .iter()
                .filter_map(|effect| effect.projection.as_ref())
                .max_by_key(|record| record.revision)
                .context("restart fixture history missing")?;
            Some((record.identity.clone(), record.revision))
        } else {
            None
        };
        let exclusion = crate::e3_child_security::E3PrivilegedChildExclusionV1::new_recovering()?;
        let manager = Arc::new(E3ConfigProjectionPreparationManagerV1::new(
            authority,
            service,
            registry.clone(),
            exclusion.clone(),
        ));
        let startup_headless = recovery_phase.as_deref() == Some("startup-headless");
        if startup_headless {
            anyhow::ensure!(
                fs::metadata("/proc/self/ns/net")?.ino() != fs::metadata("/proc/1/ns/net")?.ino(),
                "headless manager fixture requires isolated network namespace"
            );
            fs::set_permissions(&world_parent, fs::Permissions::from_mode(0o700))?;
            let mount = fs::metadata("/sys/fs/cgroup")?;
            let parent = fs::metadata(&world_parent)?;
            let series = format!("cps_{}", uuid::Uuid::now_v7());
            let fence = format!("cpf_{}", uuid::Uuid::now_v7());
            let group = format!("ecg_{}", uuid::Uuid::now_v7());
            let component = format!(
                "substrate-e3-{}",
                &format!("{:x}", Sha256::digest(group.as_bytes()))[..24]
            );
            let boundary = format!("gab_{}", uuid::Uuid::now_v7());
            let table = format!(
                "substrate_e3_{}",
                &format!("{:x}", Sha256::digest(boundary.as_bytes()))[..24]
            );
            for effect in [
                E3KernelEffectKindV1::CreateChildCgroup {
                    cgroup_registration_id: group,
                    role: E3TerminalProcessRoleV1::ManagedGateway,
                    parent_cgroup: CanonicalCgroupIdentityV1 {
                        cgroup_v2_mount_device_id: mount.dev(),
                        cgroup_v2_mount_inode: mount.ino(),
                        cgroup_directory_inode: parent.ino(),
                        cgroup_relative_path: format!("substrate/{}", request.world_id),
                    },
                    child_component: component.clone(),
                    expected_relative_path: format!("substrate/{}/{component}", request.world_id),
                },
                E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: boundary,
                    network_namespace_inode: fs::metadata("/proc/self/ns/net")?.ino(),
                    table_name: table.clone(),
                    chain_name: "gateway_output".into(),
                },
            ] {
                let mut intent = E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: series.clone(),
                    effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                    preparation_id: request.preparation_id.clone(),
                    fence_id: fence.clone(),
                    effect,
                    created_at: Timestamp(
                        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                    ),
                    intent_hash: String::new(),
                };
                intent.intent_hash = seal(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    "intent_hash",
                    serde_json::to_value(&intent)?,
                )?;
                registry.publish_kernel_effect_intent(&intent, None, None)?;
            }
            let child = world_parent.join(&component);
            fs::create_dir(&child)?;
            fs::set_permissions(&child, fs::Permissions::from_mode(0o700))?;
            fs::create_dir(child.join("empty-descendant"))?;
            fs::set_permissions(
                child.join("empty-descendant"),
                fs::Permissions::from_mode(0o700),
            )?;
            let mut nft = std::process::Command::new("/usr/bin/nft")
                .args(["-f", "-"])
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            {
                use std::io::Write;
                nft.stdin.take().unwrap().write_all(format!("add table inet {table}\nadd chain inet {table} gateway_output {{ type filter hook output priority -100; policy accept; }}\n").as_bytes())?;
            }
            anyhow::ensure!(
                nft.wait()?.success(),
                "headless fixture table creation failed"
            );
            anyhow::ensure!(
                registry
                    .recover(None)?
                    .kernel_effects
                    .iter()
                    .all(|effect| effect.resolution.is_none()
                        && effect.child_cgroup.is_none()
                        && effect.boundary.is_none())
                    && child.is_dir(),
                "fixture did not leave present unregistered effects"
            );
            anyhow::ensure!(
                exclusion.acquire_non_e3_child().is_err(),
                "startup admission opened before cleanup"
            );
        }
        manager.recover_expired(
            None,
            Some(E3PreparationRecoveryV1::Startup {
                service_instance_id: &format!("wsi_{}", uuid::Uuid::now_v7()),
            }),
        )?;
        if startup_headless {
            let recovered = registry.recover(None)?;
            anyhow::ensure!(recovered.kernel_effects.len() == 2 && recovered.kernel_effects.iter().all(|effect| effect.projection.is_none() && effect.child_cgroup.is_none() && effect.boundary.is_none() && effect.resolution.as_ref().is_some_and(|resolution| resolution.disposition == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent)), "manager startup did not resolve actual headless effects");
            exclusion.require_recovering_v1()?;
            anyhow::ensure!(
                exclusion.acquire_non_e3_child().is_err()
                    && manager.preparations.lock().unwrap().is_empty(),
                "startup opened admission or created a preparation"
            );
            exclusion.finish_recovery()?;
            anyhow::ensure!(
                exclusion.acquire_non_e3_child().is_ok(),
                "successful startup did not permit subsequent admission"
            );
            fs::remove_dir(&world_parent)?;
            eprintln!(
                "E3_MANAGER_STARTUP_PRESENT_UNREGISTERED_CGROUP_AND_BOUNDARY_BEFORE_ADMISSION"
            );
            return Ok(());
        }
        exclusion.finish_recovery()?;
        let outcome = Box::pin(async {
            let wire = serde_json::to_vec(&request)?;
            let response = manager.prepare(hyper::Body::from(wire.clone())).await?;
            response.validate().map_err(anyhow::Error::msg)?;
            if let Some((identity, revision)) = &prior_head {
                let map = manager.preparations.lock().unwrap();
                let owner = &map[&request.preparation_id];
                anyhow::ensure!(map.len() == 1 && owner.identity == *identity && response.config_projection.dormant_projection_ref.revision == revision + 1, "actual restart did not require fresh auth in the same series");
                eprintln!("E3_MANAGER_ACTUAL_RESTART_RECOVERED_HISTORY_THEN_FRESH_AUTH_SAME_SERIES");
            }
            anyhow::ensure!(
                manager.prepare(hyper::Body::from(wire.clone())).await? == response,
                "retry changed response"
            );
            if recovery_phase.as_deref() == Some("active-writer") {
                // Build synthetic terminal Active input from real, test-owned Dormant coordinates.
                // This fixture never launches an Active child or exercises an E3-F teardown.
                anyhow::ensure!(fs::metadata("/proc/self/ns/net")?.ino() != fs::metadata("/proc/1/ns/net")?.ino(), "Active fixture requires its isolated network namespace");
                let map = manager.preparations.lock().unwrap();
                let owner = &map[&request.preparation_id];
                let (boundary, groups, config) = owner.prepared_boundary.as_ref().context("fixture boundary missing")?;
                let ConfigProjectionSubjectReadbackV1::Bound(metadata) = manager.projection_service.resolve_preparation_subject_v1(&owner.identity)? else { anyhow::bail!("fixture subject absent"); };
                let dormant = metadata.record().clone();
                let projection_root = home.join("authority-v1/agent-config-projection-v1");
                let handoff_dir = projection_root.join("handoffs").join(&dormant.nonsecret_handoff.secret_handoff_ref.handoff_id);
                let mut handoff: serde_json::Value = ConfigProjectionCodecV1::decode_canonical_json(&fs::read(handoff_dir.join("revisions/00000000000000000001.json"))?)?;
                let write_fixture = |path: &std::path::Path, bytes: &[u8]| -> anyhow::Result<()> {
                    use std::io::Write;
                    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
                    file.write_all(bytes)?;
                    let owner = fs::metadata(&home)?;
                    std::os::unix::fs::chown(path, Some(owner.uid()), Some(owner.gid()))?;
                    file.sync_all()?;
                    File::open(path.parent().unwrap())?.sync_all()?;
                    Ok(())
                };
                let mut href = dormant.nonsecret_handoff.secret_handoff_ref.clone();
                for state in [SecretHandoffStateV1::Delivered, SecretHandoffStateV1::Consumed] {
                    handoff["predecessor_ref"] = serde_json::to_value(&href)?;
                    href.handoff_state_revision += 1;
                    handoff["handoff"]["state_revision"] = serde_json::json!(href.handoff_state_revision);
                    handoff["handoff"]["state"] = serde_json::to_value(state)?;
                    handoff["handoff"]["delivered_at"] = handoff["handoff"]["created_at"].clone();
                    if state == SecretHandoffStateV1::Consumed { handoff["handoff"]["consumed_at"] = handoff["handoff"]["delivered_at"].clone(); }
                    href.handoff_hash = seal("substrate.e3.config-projection-secret-handoff-revision.v1", "revision", "revision_hash", handoff.clone())?;
                    handoff["revision_hash"] = serde_json::json!(href.handoff_hash);
                    write_fixture(&handoff_dir.join("revisions").join(format!("{:020}.json", href.handoff_state_revision)), &ConfigProjectionCodecV1::encode_canonical_json(&handoff)?)?;
                }
                fs::write(handoff_dir.join("head.json"), ConfigProjectionCodecV1::encode_canonical_json(&href)?)?;
                File::open(handoff_dir.join("head.json"))?.sync_all()?;
                let seal_record = |record: &mut AgentConfigProjectionRecordV1| -> anyhow::Result<()> {
                    record.managed_gateway.projection_hash = seal("substrate.e3.managed-gateway-projection.v1", "projection", "projection_hash", serde_json::to_value(&record.managed_gateway)?)?;
                    record.nonsecret_handoff.projection_hash = seal("substrate.e3.nonsecret-handoff-projection.v1", "projection", "projection_hash", serde_json::to_value(&record.nonsecret_handoff)?)?;
                    record.record_hash = seal("substrate.e3.agent-config-projection-record.v1", "record", "record_hash", serde_json::to_value(&record)?)?;
                    Ok(())
                };
                let id = |prefix: &str| format!("{prefix}{}", uuid::Uuid::now_v7());
                let digest = || "22".repeat(32);
                let launch_input: ManagedGatewayLaunchInputV1 = ConfigProjectionCodecV1::decode_canonical_json(
                    &fs::read(projection_root.join("gateway-launch-inputs").join(format!("{}.json", owner.launch_input_id)))?)?;
            let group = &groups[0];
            let mut registration = E3ChildProcessRegistrationV1 {
                schema_version: 1, authority_store_id: dormant.identity.authority_store_id.clone(),
                series_id: dormant.identity.series_id.clone(), registration_id: id("ecp_"),
                cgroup_registration_id: group.cgroup_registration_id.clone(),
                cgroup_registration_hash: group.cgroup_registration_hash.clone(), fence_id: owner.fence_id.clone(),
                role: config_projection::E3TerminalProcessRoleV1::ManagedGateway, pid: 321, pid_start_time_ticks: 123,
                process_cgroup: group.cgroup.clone(), kernel_boot_id: group.kernel_boot_id.clone(),
                parent_service_instance_id: id("wsi_"), registered_at: dormant.created_at.clone(),
                registration_hash: String::new(),
            };
            registration.registration_hash = seal("substrate.e3.child-process-registration.v1", "registration", "registration_hash", serde_json::to_value(&registration)?)?;
            registry.publish_child_process_registration(&registration).unwrap();
            let identity = &dormant.identity;
            let mut security = config_projection::E3ChildSecurityAttestationV1 {
                schema_version: 1, child_role: config_projection::E3IsolatedChildRoleV1::ManagedGateway,
                projection_identity_hash: identity.identity_hash.clone(), pid: registration.pid,
                pid_start_time_ticks: registration.pid_start_time_ticks,
                real_uid: 1000, effective_uid: 1000, saved_uid: 1000, filesystem_uid: 1000,
                real_gid: 1000, effective_gid: 1000, saved_gid: 1000, filesystem_gid: 1000,
                supplementary_group_count: 0, cap_inheritable: "0000000000000000".into(),
                cap_permitted: "0000000000000000".into(), cap_effective: "0000000000000000".into(),
                cap_bounding: "0000000000000000".into(), cap_ambient: "0000000000000000".into(), cap_last_cap: 40,
                no_new_privs: true, dumpable: 0, tracer_pid: 0, kernel_boot_id: group.kernel_boot_id.clone(),
                user_namespace: config_projection::E3UserNamespaceAttestationV1 {
                    namespace_device_id: 4, namespace_inode: 5, owner_uid: 0,
                    parent_namespace_device_id: 4, parent_namespace_inode: 6,
                    uid_map: config_projection::E3LinuxIdMapExtentV1 { inside_id: 1000, outside_id: 1000, length: 1 },
                    gid_map: config_projection::E3LinuxIdMapExtentV1 { inside_id: 1000, outside_id: 1000, length: 1 },
                },
                seccomp_mode: 2, landlock_abi: 6, e2_enforcement_plan_hash: digest(),
                derived_support_ruleset_hash: digest(), role_narrowing_ruleset_hash: digest(), effective_landlock_hash: digest(),
                policy_snapshot_ref: identity.immutable_launch_cap.policy_snapshot_ref.clone(),
                policy_snapshot_hash: identity.immutable_launch_cap.policy_snapshot_hash.clone(),
                policy_snapshot_revision: identity.immutable_launch_cap.policy_snapshot_revision.clone(),
                enforcement_input_hash: digest(), denied_control_probe_hash: digest(), attestation_hash: String::new(),
            };
            security.attestation_hash = seal("substrate.e3.child-security-attestation.v1", "attestation", "attestation_hash", serde_json::to_value(&security)?)?;
            let input = &launch_input;
            let artifact = &identity.runtime_artifacts.managed_gateway;
            let mut complete_ack = config_projection::ManagedGatewayActivationAckV1 {
                schema_version: 1, authority_store_id: identity.authority_store_id.clone(), activation_ack_id: id("gaa_"),
                activation_intent_ref: input.activation_intent_ref.clone(), config_projection_identity_hash: identity.identity_hash.clone(),
                dormant_projection_ref: metadata.projection_ref().clone(), gateway_ref: input.gateway_ref.clone(),
                gateway_process_identity: config_projection::GatewayProcessIdentityV1 {
                    pid: registration.pid, pid_start_time_ticks: registration.pid_start_time_ticks, pidfd_inode: 99,
                    executable_device_id: artifact.device_id, executable_inode: artifact.inode, executable_sha256: artifact.sha256.clone(),
                    process_cgroup: group.cgroup.clone(), child_security_attestation_hash: security.attestation_hash.clone(),
                    secret_ready_attestation_hash: digest(),
                },
                child_security_attestation: security, listener_identity: input.listener_identity.clone(),
                access_boundary_ref: input.access_boundary_ref.clone(), secret_handoff_ref: href.clone(),
                secret_handoff_terminal_state: SecretHandoffStateV1::Consumed, gateway_ready_revision: 1,
                readiness_nonce: input.readiness_nonce.clone(), launch_input_ref: config_projection::ManagedGatewayLaunchInputRefV1 {
                    authority_store_id: input.authority_store_id.clone(), launch_input_id: input.launch_input_id.clone(),
                    launch_input_hash: input.launch_input_hash.clone(),
                }, observed_at: dormant.created_at.clone(), ack_hash: String::new(),
            };
            complete_ack.ack_hash = seal("substrate.e3.managed-gateway-activation-ack.v1", "ack", "ack_hash", serde_json::to_value(&complete_ack)?)?;
                let ack = ManagedGatewayActivationAckRefV1 { authority_store_id: complete_ack.authority_store_id.clone(), activation_ack_id: complete_ack.activation_ack_id.clone(), ack_hash: complete_ack.ack_hash.clone() };
                let mut ready = dormant.clone();
                ready.record_id = format!("cpr_{}", uuid::Uuid::now_v7()); ready.revision += 1;
                ready.predecessor_ref = Some(metadata.projection_ref().clone());
                ready.managed_gateway.posture = ManagedGatewayProjectionPostureV1::ReadyClosed;
                ready.managed_gateway.activation_ack_ref = Some(ack.clone());
                ready.nonsecret_handoff.activation_ack_ref = Some(ack.clone());
                ready.nonsecret_handoff.secret_handoff_ref = href;
                ready.nonsecret_handoff.observed_state = SecretHandoffStateV1::Consumed;
                ready.activation.gateway_activation_ack_ref = Some(ack.clone());
                seal_record(&mut ready)?;
                let ready_ref = registry.publish_ready_closed(metadata.projection_ref(), &ready, &complete_ack, owner.publication.as_ref().unwrap().held_consumer_lease().unwrap())?;
                let mut allow = boundary.clone();
                allow.predecessor_ref = Some(dormant.managed_gateway.access_boundary_ref.clone());
                allow.revision += 1; allow.posture = GatewayAccessPostureV1::AllowExactMember;
                let mut member_rule = allow.nftables_rules[0].clone();
                member_rule.role = NftablesRuleRoleV1::ExactMemberAccept;
                member_rule.rule_handle = allow.nftables_rules.iter().map(|rule| rule.rule_handle).max().unwrap() + 1;
                allow.nftables_rules.insert(1, member_rule);
                allow.boundary_hash = seal("substrate.e3.gateway-access-boundary.v1", "boundary", "boundary_hash", serde_json::to_value(&allow)?)?;
                let allow_ref = GatewayAccessBoundaryRefV1 { authority_store_id: store.authority_store_id.clone(), access_boundary_id: allow.access_boundary_id.clone(), revision: allow.revision, boundary_hash: allow.boundary_hash.clone() };
                write_fixture(&projection_root.join("gateway-boundaries").join(&allow.access_boundary_id).join(format!("{:020}.json", allow.revision)), &ConfigProjectionCodecV1::encode_canonical_json(&allow)?)?;
                // Dispose only this fixture's empty Dormant effects. The retained Active objects below are synthetic input.
                let mut empty_groups = Vec::new();
                for group in groups {
                    let path = PathBuf::from("/sys/fs/cgroup").join(&group.cgroup.cgroup_relative_path);
                    let events = fs::read(path.join("cgroup.events"))?;
                    let procs = fs::read(path.join("cgroup.procs"))?;
                    anyhow::ensure!(procs.is_empty() && String::from_utf8_lossy(&events).lines().any(|line| line == "populated 0"), "fixture cgroup not empty");
                    empty_groups.push(E3TerminalCgroupQuiescenceV1 { cgroup_registration_id: group.cgroup_registration_id.clone(), cgroup_registration_hash: group.cgroup_registration_hash.clone(), role: group.role, cgroup: group.cgroup.clone(), cgroup_events_sha256: format!("{:x}", Sha256::digest(events)), cgroup_procs_sha256: format!("{:x}", Sha256::digest(procs)), populated: false, ordered_live_pids: Vec::new() });
                    fs::remove_dir(path)?;
                }
                anyhow::ensure!(std::process::Command::new("/usr/bin/nft").args(["delete", "table", "inet", &boundary.nftables_chain.table]).status()?.success(), "dispose exact fixture table failed");
                let mut active = ready.clone();
                active.record_id = format!("cpr_{}", uuid::Uuid::now_v7()); active.revision += 1;
                active.predecessor_ref = Some(ready_ref.clone()); active.managed_gateway.posture = ManagedGatewayProjectionPostureV1::Active;
                active.managed_gateway.access_boundary_ref = allow_ref.clone();
                active.activation.released_at = Some(active.created_at.clone());
                active.activation.publication_fence = ConfigProjectionPublicationFenceV1::Released { fence_id: owner.fence_id.clone(), closed_record_ref: ready_ref.clone(), activation_ack_ref: ack.clone(), release_hash: ConfigProjectionCodecV1::domain_sha256("substrate.e3.config-projection-release.v1", &serde_json::json!({"activation_ack_ref":ack, "closed_record_ref":ready_ref, "fence_id":owner.fence_id}))? };
                seal_record(&mut active)?;
                let active_ref = registry.publish_active(&ready_ref, &active)?;
                let recovering = crate::e3_child_security::E3PrivilegedChildExclusionV1::new_recovering()?;
                let reject = || -> anyhow::Result<()> {
                    let readback = registry.recover(None)?;
                    anyhow::ensure!(crate::gateway_runtime::E3GatewayRuntimeAuthorityV1::revoke(crate::gateway_runtime::E3GatewayRevocationTargetV1::Recovered { effects: &readback.kernel_effects, service_instance_id: &format!("wsi_{}", uuid::Uuid::now_v7()), exclusion: &recovering }, &registry).is_err(), "incomplete Active history accepted");
                    recovering.require_recovering_v1()?;
                    Ok(())
                };
                reject()?;
                eprintln!("E3_ACTIVE_MISSING_TERMINAL_AUTHORITY_REJECTED");
                let mut revoked = allow.clone(); revoked.revision += 1; revoked.predecessor_ref = Some(allow_ref); revoked.posture = GatewayAccessPostureV1::Revoked;
                revoked.nftables_rules.retain(|rule| rule.role == NftablesRuleRoleV1::RejectRemainder);
                revoked.boundary_hash = seal("substrate.e3.gateway-access-boundary.v1", "boundary", "boundary_hash", serde_json::to_value(&revoked)?)?;
                for effect in registry.recover(None)?.kernel_effects {
                    let mut resolution = E3KernelEffectResolutionV1 { schema_version: 1, authority_store_id: store.authority_store_id.clone(), resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()), effect_intent_ref: E3KernelEffectIntentRefV1 { authority_store_id: store.authority_store_id.clone(), effect_intent_id: effect.intent.effect_intent_id.clone(), intent_hash: effect.intent.intent_hash.clone() }, disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent, observed_cgroup: effect.child_cgroup.as_ref().map(|group| group.cgroup.clone()), observed_nftables_table_handle: effect.boundary.as_ref().map(|_| 1), resolved_at: active.created_at.clone(), resolution_hash: String::new() };
                    resolution.resolution_hash = seal("substrate.e3.kernel-effect-resolution.v1", "resolution", "resolution_hash", serde_json::to_value(&resolution)?)?;
                    registry.publish_kernel_effect_resolution(&resolution, Some(&mut || Ok(effect.boundary.as_ref().map(|_| revoked.clone()))), Some(&active_ref))?;
                }
                empty_groups.sort_by_key(|group| match group.role { E3TerminalProcessRoleV1::ManagedGateway => 0, E3TerminalProcessRoleV1::ReadinessProbe => 2, E3TerminalProcessRoleV1::Codex => 1 });
                let mut proof = E3TerminalChildQuiescenceEvidenceV1 { schema_version: 1, authority_store_id: store.authority_store_id.clone(), series_id: active.identity.series_id.clone(), evidence_id: format!("tce_{}", uuid::Uuid::now_v7()), final_projection_ref: active_ref, world_id: active.identity.world_id.clone(), world_generation: active.identity.world_generation, ordered_terminal_processes: vec![E3TerminalProcessObservationV1 { registration_id: registration.registration_id.clone(), registration_hash: registration.registration_hash.clone(), role: registration.role, pid: registration.pid, pid_start_time_ticks: registration.pid_start_time_ticks, observation: E3TerminalProcessObservationKindV1::ParentWaitid { terminal_wait_status: 0 } }], ordered_empty_cgroups: empty_groups, observed_at: active.created_at.clone(), evidence_hash: String::new() };
                proof.evidence_hash = seal("substrate.e3.terminal-child-quiescence.v1", "evidence", "evidence_hash", serde_json::to_value(&proof)?)?;
                registry.publish_terminal_child_evidence(&proof).context("synthetic Active terminal proof")?;
                reject()?;
                eprintln!("E3_ACTIVE_HELD_LEASE_REJECTED");
                registry.release_consumer_lease(owner.publication.as_ref().unwrap().held_consumer_lease().unwrap(), active.created_at.clone())?;
                reject()?;
                anyhow::ensure!(PathBuf::from(&config.root.physical_path).join("config.toml").exists(), "Active observation removed config");
                eprintln!("E3_ACTIVE_PRESENT_CONFIG_REJECTED_WITHOUT_CLEANUP");
                fs::remove_file(PathBuf::from(&config.root.physical_path).join("config.toml"))?;
                fs::remove_dir(&config.root.physical_path)?;
                if std::env::var_os("E3_E_ACTIVE_LATER_HEADLESS").is_some() {
                    let registration = format!("ecg_{}", uuid::Uuid::now_v7());
                    let component = format!("substrate-e3-{}", &format!("{:x}", Sha256::digest(registration.as_bytes()))[..24]);
                    let mount = fs::metadata("/sys/fs/cgroup")?;
                    let parent = fs::metadata(&world_parent)?;
                    let mut intent = E3KernelEffectIntentV1 {
                        schema_version: 1, authority_store_id: store.authority_store_id.clone(), series_id: active.identity.series_id.clone(),
                        effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()), preparation_id: format!("e3p_{}", uuid::Uuid::now_v7()), fence_id: format!("cpf_{}", uuid::Uuid::now_v7()),
                        effect: E3KernelEffectKindV1::CreateChildCgroup { cgroup_registration_id: registration, role: E3TerminalProcessRoleV1::ManagedGateway,
                            parent_cgroup: CanonicalCgroupIdentityV1 { cgroup_v2_mount_device_id: mount.dev(), cgroup_v2_mount_inode: mount.ino(), cgroup_directory_inode: parent.ino(), cgroup_relative_path: format!("substrate/{}", request.world_id) },
                            child_component: component.clone(), expected_relative_path: format!("substrate/{}/{component}", request.world_id) },
                        created_at: Timestamp(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)), intent_hash: String::new()
                    };
                    anyhow::ensure!(intent.fence_id > owner.fence_id, "fixture did not place headless intent after Active fence in prior ordering");
                    intent.intent_hash = seal("substrate.e3.kernel-effect-intent.v1", "intent", "intent_hash", serde_json::to_value(&intent)?)?;
                    registry.publish_kernel_effect_intent(&intent, None, None)?;
                    fs::create_dir(world_parent.join(&component))?;
                    fs::set_permissions(world_parent.join(&component), fs::Permissions::from_mode(0o700))?;
                    eprintln!("E3_ACTIVE_WITH_LATER_SAME_SERIES_PRESENT_HEADLESS_CGROUP");
                }
                eprintln!("E3_SYNTHETIC_TERMINAL_ACTIVE_INPUT_DURABLE_NO_E3F_LIFECYCLE");
                unsafe { libc::_exit(79) };
            }
            if matches!(recovery_phase.as_deref(), Some("fresh" | "resume-fresh")) {
                let cancel = E3ConfigProjectionCancelRequestV1 {
                    schema_version: 1,
                    preparation_id: request.preparation_id.clone(),
                    preparation_idempotency_key: request.preparation_idempotency_key.clone(),
                    config_projection: response.config_projection.clone(),
                };
                manager.cancel(cancel.clone())?;
                let mut old = response.config_projection.dormant_projection_ref.clone();
                for expire in [true, false] {
                request.preparation_id = format!("e3p_{}", uuid::Uuid::now_v7());
                let mut nonsecret = serde_json::to_value(&request)?;
                nonsecret.as_object_mut().unwrap().remove("integrated_auth");
                nonsecret.as_object_mut().unwrap().remove("preparation_idempotency_key");
                request.preparation_idempotency_key = format!("e3pik_{}", ConfigProjectionCodecV1::domain_sha256(
                    "substrate.e3.config-projection-prepare-idempotency.v1",
                    &serde_json::json!({"credential_shape": {"backend_id": request.backend_id, "ordered_field_names": ["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"]}, "request": nonsecret}))?);
                let fresh_wire = serde_json::to_vec(&request)?;
                let fresh = manager.prepare(hyper::Body::from(fresh_wire.clone())).await?;
                let next = &fresh.config_projection.dormant_projection_ref;
                anyhow::ensure!(next.series_id == old.series_id && next.revision == old.revision + 1 && next.record_id != old.record_id, "fresh auth did not advance the same series");
                anyhow::ensure!(manager.prepare(hyper::Body::from(fresh_wire)).await? == fresh, "fresh exact retry changed response");
                anyhow::ensure!(manager.cancel(cancel.clone()).is_err(), "stale old cancellation accepted");
                eprintln!("E3_MANAGER_CANCEL_OR_EXPIRY_FRESH_AUTH_SAME_SERIES_SUCCESSOR_AND_EXACT_RETRY");
                if expire {
                    manager.preparations.lock().unwrap().get_mut(&request.preparation_id).unwrap().expires_deadline.seconds = 0;
                    manager.recover_expired(None, None)?;
                    {
                        let map = manager.preparations.lock().unwrap();
                        let owner = &map[&request.preparation_id];
                        anyhow::ensure!(owner.cleanup_complete && owner.terminal.as_ref().is_some_and(|terminal| terminal.0 == SecretHandoffStateV1::Expired), "expiry did not complete cleanup");
                        let ConfigProjectionSubjectReadbackV1::Bound(metadata) = manager.projection_service.resolve_preparation_subject_v1(&owner.identity)? else { anyhow::bail!("expired subject absent"); };
                        let handoff_id = &metadata.record().nonsecret_handoff.secret_handoff_ref.handoff_id;
                        let directory = home.join("authority-v1/agent-config-projection-v1/handoffs").join(handoff_id);
                        let head: SecretHandoffRefV1 = ConfigProjectionCodecV1::decode_canonical_json(&fs::read(directory.join("head.json"))?)?;
                        let terminal: ConfigProjectionSecretHandoffRevisionV1 = ConfigProjectionCodecV1::decode_canonical_json(&fs::read(directory.join("revisions").join(format!("{:020}.json", head.handoff_state_revision)))?)?;
                        anyhow::ensure!(serde_json::to_value(&terminal)?["handoff"]["state"] == "Expired", "durable expiry state missing");
                    }
                    old = next.clone();
                    eprintln!("E3_MANAGER_BOOTTIME_EXPIRY_DURABLE_TERMINAL_AND_RELEASE");
                    continue;
                }
                manager.cancel(E3ConfigProjectionCancelRequestV1 {
                    schema_version: 1,
                    preparation_id: request.preparation_id.clone(),
                    preparation_idempotency_key: request.preparation_idempotency_key.clone(),
                    config_projection: fresh.config_projection,
                })?;
                eprintln!("E3_MANAGER_FRESH_SUCCESSOR_CANCELLED_WITH_COMPLETE_HISTORY");
                }
                return Ok(());
            }
            if registered_restart && recovery_phase.as_deref() == Some("writer") {
                let readback = registry.recover(None)?;
                let group = readback
                    .kernel_effects
                    .iter()
                    .find_map(|effect| effect.child_cgroup.as_ref())
                    .context("registered fixture cgroup missing")?;
                let mut child = if live_registered_restart {
                    use std::os::unix::process::CommandExt;
                    use std::os::fd::AsRawFd;
                    let owner = fs::metadata(&home)?;
                    let (mut parent, child_channel) = std::os::unix::net::UnixStream::pair()?;
                    parent.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
                    let fd = child_channel.as_raw_fd();
                    let mut command = std::process::Command::new("/usr/bin/python3");
                    command.args(["-c", r#"import ctypes, os, socket, sys
channel = socket.socket(fileno=int(sys.argv[1]))
os.setgroups([])
libc = ctypes.CDLL(None, use_errno=True)
if libc.unshare(0x10000000) != 0:
    raise OSError(ctypes.get_errno(), 'fixture unshare')
channel.sendall(b'R')
assert channel.recv(1) == b'G'
os.setresgid(int(sys.argv[3]), int(sys.argv[3]), int(sys.argv[3]))
os.setresuid(int(sys.argv[2]), int(sys.argv[2]), int(sys.argv[2]))
channel.close()
os.execv('/bin/sleep', ['/bin/sleep', '600'])
"#]).arg(fd.to_string()).arg(owner.uid().to_string()).arg(owner.gid().to_string());
                    unsafe { command.pre_exec(move || { if libc::fcntl(fd, libc::F_SETFD, 0) < 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) } }); }
                    let mut child = command.spawn()?;
                    drop(child_channel);
                    let mapped = (|| -> anyhow::Result<()> {
                        use std::io::{Read, Write};
                        let mut ready = [0]; parent.read_exact(&mut ready)?;
                        anyhow::ensure!(ready == *b"R", "fixture namespace readiness mismatch");
                        let process = format!("/proc/{}", child.id());
                        fs::write(format!("{process}/setgroups"), "deny")?;
                        fs::write(format!("{process}/uid_map"), format!("{0} {0} 1\n", owner.uid()))?;
                        fs::write(format!("{process}/gid_map"), format!("{0} {0} 1\n", owner.gid()))?;
                        parent.write_all(b"G")?;
                        Ok(())
                    })();
                    if let Err(error) = mapped { let _ = child.kill(); let _ = child.wait(); return Err(error); }
                    child
                } else { std::process::Command::new("/bin/sleep").arg("30").spawn()? };
                let registered = (|| -> anyhow::Result<()> {
                    let pid = child.id();
                    let cgroup =
                        PathBuf::from("/sys/fs/cgroup").join(&group.cgroup.cgroup_relative_path);
                    fs::write(cgroup.join("cgroup.procs"), pid.to_string())?;
                    let membership = fs::read_to_string(format!("/proc/{pid}/cgroup"))?;
                    anyhow::ensure!(
                        membership.trim() == format!("0::/{}", group.cgroup.cgroup_relative_path),
                        "fixture process did not enter exact cgroup"
                    );
                    if live_registered_restart {
                        let owner = fs::metadata(&home)?;
                        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                        loop {
                            let uid_map = fs::read_to_string(format!("/proc/{pid}/uid_map"))?;
                            let gid_map = fs::read_to_string(format!("/proc/{pid}/gid_map"))?;
                            if uid_map.split_whitespace().collect::<Vec<_>>() == [owner.uid().to_string(), owner.uid().to_string(), "1".into()]
                                && gid_map.split_whitespace().collect::<Vec<_>>() == [owner.gid().to_string(), owner.gid().to_string(), "1".into()]
                                && fs::read(format!("/proc/{pid}/cmdline"))?.starts_with(b"/bin/sleep\0") { break; }
                            anyhow::ensure!(std::time::Instant::now() < deadline && child.try_wait()?.is_none(), "live remnant fixture did not enter its protected namespace");
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
                    let start = stat
                        .rsplit_once(')')
                        .context("fixture process stat missing delimiter")?
                        .1
                        .split_whitespace()
                        .nth(19)
                        .context("fixture process start time missing")?
                        .parse()?;
                    let mut registration = E3ChildProcessRegistrationV1 {
                        schema_version: 1,
                        authority_store_id: group.authority_store_id.clone(),
                        series_id: group.series_id.clone(),
                        registration_id: format!("ecp_{}", uuid::Uuid::now_v7()),
                        cgroup_registration_id: group.cgroup_registration_id.clone(),
                        cgroup_registration_hash: group.cgroup_registration_hash.clone(),
                        fence_id: group.fence_id.clone(),
                        role: group.role,
                        pid,
                        pid_start_time_ticks: start,
                        process_cgroup: group.cgroup.clone(),
                        kernel_boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")?
                            .trim()
                            .to_owned(),
                        parent_service_instance_id: manager.service_instance_id.get().context("fixture writer instance absent")?.clone(),
                        registered_at: Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        registration_hash: String::new(),
                    };
                    registration.registration_hash = seal(
                        "substrate.e3.child-process-registration.v1",
                        "registration",
                        "registration_hash",
                        serde_json::to_value(&registration)?,
                    )?;
                    registry.publish_child_process_registration(&registration)?;
                    Ok(())
                })();
                if live_registered_restart && registered.is_ok() {
                    anyhow::ensure!(child.try_wait()?.is_none(), "registered remnant exited before interruption");
                    eprintln!("E3_RECOVERED_FIXTURE_REGISTERED_STILL_LIVE_CHILD {}", child.id());
                    unsafe { libc::_exit(77) };
                }
                let killed = child.kill();
                let waited = child.wait();
                registered?;
                killed?;
                waited?;
                eprintln!("E3_RECOVERED_FIXTURE_REGISTERED_ACTUAL_REAPED_CHILD");
                unsafe { libc::_exit(75) };
            }
            let (identity, config_root, cgroup_paths) = {
                let map = manager.preparations.lock().unwrap();
                let owned = &map[&request.preparation_id];
                anyhow::ensure!(
                    owned.projection.is_some() && owned.credential_source.is_some(),
                    "success lost capabilities"
                );
                let (boundary, groups, config) = owned.prepared_boundary.as_ref().unwrap();
                anyhow::ensure!(
                    boundary.posture == GatewayAccessPostureV1::DenyAllDormant,
                    "boundary not Dormant"
                );
                let lease = owned
                    .publication
                    .as_ref()
                    .unwrap()
                    .held_consumer_lease()
                    .unwrap();
                let (resolved, _) = registry.resolve(Some((&owned.identity, lease)), None)?;
                let Some(ConfigProjectionResolutionV1::Current {
                    record,
                    projection_ref,
                    ..
                }) = resolved
                else {
                    anyhow::bail!("manager head not current")
                };
                anyhow::ensure!(
                    projection_ref == response.config_projection.dormant_projection_ref,
                    "response head mismatch"
                );
                anyhow::ensure!(
                    record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant,
                    "record not Dormant"
                );
                anyhow::ensure!(
                    record.managed_gateway.activation_ack_ref.is_none(),
                    "prepare published ACK"
                );
                anyhow::ensure!(
                    record.native.root.root_id
                        == format!("cnr_{}", owned.fence_id.strip_prefix("cpf_").unwrap()),
                    "root attempt mismatch"
                );
                anyhow::ensure!(
                    owned.native_source_manifest.is_some(),
                    "native source missing"
                );
                anyhow::ensure!(record.created_at == owned.created_at, "timestamp changed");
                anyhow::ensure!(
                    config.sha256
                        == format!(
                            "{:x}",
                            Sha256::digest(fs::read(
                                PathBuf::from(&config.root.physical_path).join("config.toml")
                            )?)
                        ),
                    "config bytes mismatch"
                );
                let paths: Vec<_> = groups
                    .iter()
                    .map(|group| {
                        PathBuf::from("/sys/fs/cgroup").join(&group.cgroup.cgroup_relative_path)
                    })
                    .collect();
                for path in &paths {
                    anyhow::ensure!(
                        fs::read(path.join("cgroup.procs"))?.is_empty(),
                        "prepare spawned child"
                    );
                }
                (
                    owned.identity.clone(),
                    PathBuf::from(&config.root.physical_path),
                    paths,
                )
            };
            let cancel = E3ConfigProjectionCancelRequestV1 {
                schema_version: 1,
                preparation_id: request.preparation_id.clone(),
                preparation_idempotency_key: request.preparation_idempotency_key.clone(),
                config_projection: response.config_projection.clone(),
            };
            let projection_root = home.join("authority-v1/agent-config-projection-v1");
            let terminal_dir = projection_root
                .join("terminal-child-evidence")
                .join(&identity.series_id);
            *interrupted.interrupt_after.lock().unwrap() = Some(terminal_dir.clone());
            anyhow::ensure!(
                manager.cancel(cancel.clone()).is_err(),
                "terminal publication return was not interrupted"
            );
            anyhow::ensure!(
                config_root.exists()
                    && cgroup_paths.iter().all(|path| path.exists())
                    && exclusion.acquire_non_e3_child().is_err(),
                "terminal publication interruption lost ownership"
            );
            let terminal_paths = fs::read_dir(&terminal_dir)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            anyhow::ensure!(
                terminal_paths.len() == 1,
                "terminal publication was not unique"
            );
            let original_evidence = fs::read(&terminal_paths[0])?;
            let boundary_id = {
                let map = manager.preparations.lock().unwrap();
                map[&request.preparation_id]
                    .prepared_boundary
                    .as_ref()
                    .unwrap()
                    .0
                    .access_boundary_id
                    .clone()
            };
            let revoked_path = projection_root
                .join("gateway-boundaries")
                .join(boundary_id)
                .join("00000000000000000002.json");
            *interrupted.interrupt_after.lock().unwrap() = Some(revoked_path.clone());
            anyhow::ensure!(
                manager.cancel(cancel.clone()).is_err(),
                "Revoked publication return was not interrupted"
            );
            anyhow::ensure!(
                config_root.exists()
                    && cgroup_paths.iter().all(|path| path.exists())
                    && exclusion.acquire_non_e3_child().is_err(),
                "Revoked publication interruption removed recoverable coordinates"
            );
            let original_revoked = fs::read(&revoked_path)?;
            let descendant = cgroup_paths[0].join("unexpected-descendant");
            fs::create_dir(&descendant)?;
            let descendant_retry = manager.cancel(cancel.clone());
            fs::remove_dir(&descendant)?;
            anyhow::ensure!(
                descendant_retry.is_err()
                    && config_root.exists()
                    && cgroup_paths.iter().all(|path| path.exists())
                    && exclusion.acquire_non_e3_child().is_err(),
                "resolved-boundary retry ignored a new descendant"
            );
            let unexpected_config = config_root.join("unexpected");
            fs::write(&unexpected_config, b"nonsecret cleanup interruption")?;
            let config_retry = manager.cancel(cancel.clone());
            fs::remove_file(&unexpected_config)?;
            anyhow::ensure!(
                config_retry.is_err()
                    && config_root.join("config.toml").exists()
                    && cgroup_paths.iter().all(|path| !path.exists())
                    && exclusion.acquire_non_e3_child().is_err(),
                "config rejection lost ownership after kernel resolution"
            );
            let cancelled = manager.cancel(cancel.clone())?;
            anyhow::ensure!(
                fs::read(&terminal_paths[0])? == original_evidence
                    && fs::read(&revoked_path)? == original_revoked
                    && fs::read_dir(&terminal_dir)?.count() == 1,
                "cleanup retry rewrote terminal observations or Revoked bytes"
            );
            eprintln!("E3_MANAGER_INTERRUPTED_TERMINAL_AND_REVOKED_PUBLICATION_EXACT_RETRY");
            eprintln!("E3_MANAGER_RESOLVED_RETRY_DESCENDANT_AND_CONFIG_REJECTION_RETAINED");
            anyhow::ensure!(
                manager.cancel(cancel)? == cancelled,
                "cancel retry changed response"
            );
            anyhow::ensure!(
                matches!(
                    manager.prepare(hyper::Body::from(wire)).await,
                    Err(ConfigProjectionFailureV1::ExpiredPreparation)
                ),
                "terminal retry accepted"
            );
            anyhow::ensure!(!config_root.exists(), "config cleanup incomplete");
            anyhow::ensure!(
                cgroup_paths.iter().all(|path| !path.exists()),
                "cgroup cleanup incomplete"
            );
            let map = manager.preparations.lock().unwrap();
            let owned = &map[&request.preparation_id];
            anyhow::ensure!(owned.identity == identity, "identity changed");
            anyhow::ensure!(
                owned.cleanup_complete
                    && owned.gateway_authority.is_none()
                    && owned.credential_source.is_none()
                    && owned.projection.is_none(),
                "owners not released after cleanup"
            );
            anyhow::ensure!(
                exclusion.acquire_non_e3_child().is_ok(),
                "exclusion not released"
            );
            Ok::<(), anyhow::Error>(())
        })
        .await;
        // Never abandon test-owned effects on an assertion-free failure path.
        let cleanup = manager.recover_expired(Some(&request.preparation_id), None);
        if cleanup.is_ok() {
            fs::remove_dir(&world_parent)?;
        }
        outcome?;
        cleanup?;
        Ok(())
    }
}
