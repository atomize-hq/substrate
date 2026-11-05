//! Ordinary Linux selection birth, recovery and held Host gateway admission.
//! Executing HSA admission may reconcile and acquire separate transactions.
//! Explicit inspection callbacks retain existing read-only sources. Host admission borrows the genuine reviewed run
//! observation and supplies no transport, provider or process-effect permission.

use super::host_session_authority::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1,
    DurableSessionAuthorityOriginV1, HostAttachContractHashInputV1, HostAttachContractV1,
    HostSessionTransitionModeV1, PolicyObjectHashInputV1, RuntimeBackendKindV1, WorkspaceBindingV1,
    WorldBindingV1,
};
use super::host_session_authority::store_schema::HostSessionTransitionIntentStateV2;
use super::host_session_authority::trusted_fs::{
    TrustedAuthorityRoot, TrustedDirectory, TrustedWorkspaceRoot,
};
use super::host_session_authority::{
    canonical_json, HostSessionAuthority, ResolvedCurrentAuthorityV1,
};
use crate::execution::agent_inventory::{AgentInventoryEntryV1, AgentInventorySnapshotV3};
use crate::execution::config_model::{AgentExecutionScope, SubstrateConfig};
use anyhow::{ensure, Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Read,
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};
use substrate_gateway::auth::codex_auth_context::{
    CodexAccountIdSource, CodexIntegratedAuthHandoff,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
};
use transport_api_types::{
    AgentConnectionAuthV1, AgentConnectionBillingV1, AgentConnectionProfileV1,
    AgentHarnessFamilyV1, AgentSelectionOriginV1, AgentSelectionPlacementV1,
    AgentSelectionSourceScopeV1, AgentSelectionSourceV1, PlatformPrincipalV1,
    ResolvedAgentSelectionV1,
};

const MAX_CONTEXT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OrdinaryStartSelectionsV1 {
    pub(crate) native: ResolvedAgentSelectionV1,
    pub(crate) requested: ResolvedAgentSelectionV1,
}

impl OrdinaryStartSelectionsV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        self.native.validate().map_err(anyhow::Error::msg)?;
        self.requested.validate().map_err(anyhow::Error::msg)?;
        ensure!(
            self.native.placement == AgentSelectionPlacementV1::Host,
            "native Start role must be Host"
        );
        ensure!(
            self.requested.placement != AgentSelectionPlacementV1::Host
                || self.native == self.requested,
            "Host requested/native roles conflict"
        );
        Ok(())
    }

    pub(crate) fn validate_captured(
        &self,
        config: &SubstrateConfig,
        snapshot: &AgentInventorySnapshotV3,
    ) -> Result<()> {
        self.validate()?;
        if self.requested.placement == AgentSelectionPlacementV1::World {
            let hub = config.agents.hub.orchestrator_agent_id.trim();
            ensure!(
                hub == self.native.logical_agent_id || hub == self.native.realized_agent_id,
                "native role is not the configured hub orchestrator"
            );
        }
        for selected in [&self.native, &self.requested] {
            let (agent, backend) = match selected.selection_origin {
                AgentSelectionOriginV1::ExplicitAgent => {
                    (Some(selected.logical_agent_id.as_str()), None)
                }
                AgentSelectionOriginV1::ExplicitBackend => {
                    (None, Some(selected.backend_id.as_str()))
                }
                AgentSelectionOriginV1::ConfiguredDefault => (None, None),
            };
            let scope = match selected.placement {
                AgentSelectionPlacementV1::Host => AgentExecutionScope::Host,
                AgentSelectionPlacementV1::World => AgentExecutionScope::World,
            };
            ensure!(
                snapshot.resolve_selection(config, agent, backend, scope)? == *selected,
                "selection differs from captured original decision"
            );
        }
        Ok(())
    }
}

/// Semantic birth data and exact source digests; never raw YAML/MCP env values.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapturedOrdinarySourcesV1 {
    config: SubstrateConfig,
    config_sources: Vec<AgentSelectionSourceV1>,
    preference_source: Option<AgentSelectionSourceV1>,
    profiles: Vec<AgentConnectionProfileV1>,
    inventory: BTreeMap<String, AgentInventoryEntryV1>,
}

impl CapturedOrdinarySourcesV1 {
    pub(crate) fn capture(
        config: &SubstrateConfig,
        snapshot: &AgentInventorySnapshotV3,
        selections: &OrdinaryStartSelectionsV1,
    ) -> Result<Self> {
        selections.validate_captured(config, snapshot)?;
        let mut inventory = BTreeMap::new();
        for selected in [&selections.native, &selections.requested] {
            let mut entry = snapshot
                .entries
                .get(&selected.realized_agent_id)
                .context("selected runtime entry absent")?
                .clone();
            ensure!(
                entry.file.config.api.is_none() && entry.file.config.cli.is_some(),
                "ordinary selected birth requires the supported CLI entry"
            );
            // Metadata is arbitrary user annotation, not a launch permission.
            // Preserve permission fields and exact raw source SHA without copying
            // arbitrary annotations, unrelated adapters, or inline MCP settings.
            if let Some(overlay) = &mut entry.file.policy_overlay {
                overlay.metadata = None;
            }
            inventory.insert(selected.realized_agent_id.clone(), entry);
        }
        Ok(Self {
            config: config.clone(),
            config_sources: config.selection_sources.clone(),
            preference_source: config.selection_preference_source.clone(),
            profiles: config.selection_profiles.clone(),
            inventory,
        })
    }

    pub(crate) fn effective_config(&self) -> SubstrateConfig {
        let mut config = self.config.clone();
        config.selection_sources = self.config_sources.clone();
        config.selection_preference_source = self.preference_source.clone();
        config.selection_profiles = self.profiles.clone();
        config
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum OrdinaryCodexAccountIdSourceV1 {
    Explicit,
    JwtFallback,
}

/// Nonsecret binding only. No handoff, SecretString, token, or credential value.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryRoleAccountBindingV1 {
    principal: PlatformPrincipalV1,
    profile_id: String,
    profile_revision: String,
    profile_source_sha256: String,
    account_id: Option<String>,
    account_id_source: Option<OrdinaryCodexAccountIdSourceV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryRoleAccountsV1 {
    native: OrdinaryRoleAccountBindingV1,
    requested: OrdinaryRoleAccountBindingV1,
}

fn capture_role_account(
    selected: &ResolvedAgentSelectionV1,
    principal: &PlatformPrincipalV1,
    policy: &substrate_broker::Policy,
) -> Result<(
    OrdinaryRoleAccountBindingV1,
    Option<transport_api_types::GatewayCliCodexIntegratedAuthV1>,
)> {
    selected.validate().map_err(anyhow::Error::msg)?;
    let mut binding = OrdinaryRoleAccountBindingV1 {
        principal: principal.clone(),
        profile_id: selected.connection_profile.id.clone(),
        profile_revision: selected.connection_profile.revision.clone(),
        profile_source_sha256: selected.connection_profile.source.sha256.clone(),
        account_id: None,
        account_id_source: None,
    };
    let mut gateway_handoff = None;
    if selected.selection.harness == AgentHarnessFamilyV1::Codex {
        let profile = &selected.connection_profile;
        ensure!(
            profile.billing == AgentConnectionBillingV1::Subscription
                && profile.upstream_base_url == "https://chatgpt.com/backend-api"
                && matches!(&profile.auth, AgentConnectionAuthV1::CodexAccount { reference } if reference == "configured-account"),
            "unsupported selected Codex account/profile route"
        );
        let home = crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
        let handoff = crate::builtins::world_gateway::resolve_cli_codex_integrated_auth(
            policy,
            &selected.backend_id,
            &home.join(".codex/auth.json"),
        )?;
        // Existing canonical gateway handoff fields are transient memory only.
        // The existing resolver extracts identity; it does not authenticate a JWT
        // signature, credentials, subscription, entitlement, or current permission.
        let mut fields = HashMap::from([(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
            handoff.access_token.clone(),
        )]);
        if let Some(account) = &handoff.account_id {
            fields.insert(
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.to_string(),
                account.clone(),
            );
        }
        let resolved = CodexIntegratedAuthHandoff::from_fields(&fields)?.resolve()?;
        binding.account_id = Some(resolved.account_id);
        binding.account_id_source = Some(match resolved.account_id_source {
            CodexAccountIdSource::Explicit => OrdinaryCodexAccountIdSourceV1::Explicit,
            CodexAccountIdSource::JwtFallback => OrdinaryCodexAccountIdSourceV1::JwtFallback,
        });
        gateway_handoff = Some(handoff);
    }
    if selected.selection.harness == AgentHarnessFamilyV1::ClaudeCode {
        let profile = &selected.connection_profile;
        ensure!(
            profile.billing == AgentConnectionBillingV1::Api
                && profile.upstream_base_url == "https://api.anthropic.com"
                && matches!(&profile.auth, AgentConnectionAuthV1::ApiKeyEnvironment { name } if name == "ANTHROPIC_API_KEY"),
            "unsupported selected existing Claude API-key profile route"
        );
    }
    Ok((binding, gateway_handoff))
}

fn capture_role_accounts(
    selections: &OrdinaryStartSelectionsV1,
    principal: &PlatformPrincipalV1,
    policy: &substrate_broker::Policy,
) -> Result<OrdinaryRoleAccountsV1> {
    // Resolve each actual role against its own selected profile/backend policy;
    // never copy the host's account into the worker because a label matches.
    Ok(OrdinaryRoleAccountsV1 {
        native: capture_role_account(&selections.native, principal, policy)?.0,
        requested: capture_role_account(&selections.requested, principal, policy)?.0,
    })
}

/// Closed immutable ordinary artifact outside every closed HSA namespace.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeLaunchContextV1 {
    version: u32,
    pub(crate) intent_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) original_participant_id: String,
    pub(crate) original_run_id: String,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) native_home: super::host_session_authority::schema::CanonicalDirectoryV1,
    pub(crate) original_world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) opaque_launch_plan_digest: String,
    pub(crate) selections: OrdinaryStartSelectionsV1,
    pub(crate) sources: CapturedOrdinarySourcesV1,
    accounts: OrdinaryRoleAccountsV1,
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(canonical_json::to_vec(&serde_json::to_value(value)?)?)
}
fn decode<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    ensure!(
        bytes.len() <= MAX_CONTEXT_BYTES,
        "ordinary context exceeds bound"
    );
    let value: serde_json::Value = canonical_json::from_slice(bytes)?;
    let decoded: T = serde_json::from_value(value)?;
    ensure!(
        encode(&decoded)? == bytes,
        "ordinary context is not closed/canonical"
    );
    Ok(decoded)
}
fn digest(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"substrate.ordinary-start-selection-issuer.v1\0");
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}
fn context_directory(root: &TrustedAuthorityRoot, create: bool) -> Result<TrustedDirectory> {
    let descend = |parent: &TrustedDirectory, name| -> Result<TrustedDirectory> {
        Ok(if create {
            parent.create_directory(name)?
        } else {
            parent.open_directory(name)?
        })
    };
    let run = descend(root.directory(), "run")?;
    let hub = descend(&run, "agent-hub")?;
    descend(&hub, "ordinary-start-contexts")
}
fn read_context(directory: &TrustedDirectory, name: &str) -> Result<Vec<u8>> {
    directory.open_file(name)?;
    let name = std::ffi::CString::new(name)?;
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: held directory, single validated component, exact writable stat.
    ensure!(
        unsafe {
            libc::fstatat(
                directory.borrow_fd().as_raw_fd(),
                name.as_ptr(),
                named.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0,
        "ordinary context named identity unavailable"
    );
    // SAFETY: successful fstatat initialized stat.
    let named = unsafe { named.assume_init() };
    // SAFETY: held directory and CString single component; successful fd owned below.
    let fd = unsafe {
        libc::openat(
            directory.borrow_fd().as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    ensure!(fd >= 0, "ordinary context open refused");
    // SAFETY: successful openat returns a newly owned fd.
    let file = unsafe { File::from_raw_fd(fd) };
    let before = file.metadata()?;
    ensure!(
        before.is_file()
            && before.nlink() == 1
            && before.dev() == named.st_dev
            && before.ino() == named.st_ino
            && before.len() <= MAX_CONTEXT_BYTES as u64,
        "ordinary context file identity/bound invalid"
    );
    let mut bytes = Vec::new();
    (&file)
        .take((MAX_CONTEXT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    ensure!(
        bytes.len() <= MAX_CONTEXT_BYTES
            && before.len() == after.len()
            && before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.uid() == after.uid()
            && before.mode() == after.mode()
            && after.nlink() == 1
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "ordinary context changed during read"
    );
    let mut end = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: same held directory/component, exact writable stat.
    ensure!(
        unsafe {
            libc::fstatat(
                directory.borrow_fd().as_raw_fd(),
                name.as_ptr(),
                end.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0,
        "ordinary context disappeared"
    );
    // SAFETY: successful fstatat initialized stat.
    let end = unsafe { end.assume_init() };
    ensure!(
        end.st_dev == after.dev()
            && end.st_ino == after.ino()
            && end.st_nlink == 1
            && end.st_size == after.len() as i64,
        "ordinary context named entry changed"
    );
    directory.open_file(name.to_str()?)?;
    Ok(bytes)
}

/// Scoped custody only: this guard never resolves current HSA authority.
struct OriginalStartContextComparisonGuardV1<'selection> {
    root: &'selection TrustedAuthorityRoot,
    workspace: TrustedWorkspaceRoot,
    directory: TrustedDirectory,
    directory_metadata: std::fs::Metadata,
    name: String,
    file: File,
    metadata: std::fs::Metadata,
    bytes: Vec<u8>,
}

fn same_original_context_metadata_v1(
    before: &std::fs::Metadata,
    after: &std::fs::Metadata,
) -> bool {
    before.dev() == after.dev()
        && before.ino() == after.ino()
        && before.uid() == after.uid()
        && before.gid() == after.gid()
        && before.mode() == after.mode()
        && before.nlink() == 1
        && after.nlink() == 1
        && before.len() == after.len()
        && before.mtime() == after.mtime()
        && before.mtime_nsec() == after.mtime_nsec()
        && before.ctime() == after.ctime()
        && before.ctime_nsec() == after.ctime_nsec()
}

fn capture_original_start_context_for_helper_comparison_v1<'selection>(
    root: &'selection TrustedAuthorityRoot,
    context: &NativeLaunchContextV1,
) -> Result<OriginalStartContextComparisonGuardV1<'selection>> {
    root.revalidate()?;
    ensure!(
        root.identity() == &context.workspace_binding.authority_store_root,
        "original comparison root refused"
    );
    let workspace = TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
    let directory = context_directory(root, false)?;
    let directory_file = File::from(directory.borrow_fd().try_clone_to_owned()?);
    let directory_metadata = directory_file.metadata()?;
    let name = format!("{}.json", context.intent_id);
    let component = std::ffi::CString::new(name.as_str())?;
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: fixed held directory, validated single component, writable stat.
    ensure!(
        unsafe {
            libc::fstatat(
                directory.borrow_fd().as_raw_fd(),
                component.as_ptr(),
                named.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0,
        "original comparison named entry refused"
    );
    // SAFETY: successful fstatat initialized stat.
    let named = unsafe { named.assume_init() };
    // SAFETY: fixed held directory and validated single component; fd owned below.
    let fd = unsafe {
        libc::openat(
            directory.borrow_fd().as_raw_fd(),
            component.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            0,
        )
    };
    ensure!(fd >= 0, "original comparison file refused");
    // SAFETY: openat returned a new owned descriptor.
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.uid() == directory_metadata.uid()
            && metadata.mode() & 0o7777 == 0o600
            && metadata.dev() == directory_metadata.dev()
            && metadata.dev() == named.st_dev
            && metadata.ino() == named.st_ino
            && metadata.len() <= MAX_CONTEXT_BYTES as u64,
        "original comparison file identity refused"
    );
    // Validate the named file through the existing private-file/ACL reader while
    // the direct fd and its before-read metadata are already retained.
    directory.open_file(&name)?;
    let mut bytes = vec![0; metadata.len() as usize];
    std::os::unix::fs::FileExt::read_exact_at(&file, &mut bytes, 0)?;
    let guard = OriginalStartContextComparisonGuardV1 {
        root,
        workspace,
        directory,
        directory_metadata,
        name,
        file,
        metadata,
        bytes,
    };
    guard.finalize_original_start_context_comparison_v1()?;
    Ok(guard)
}

impl OriginalStartContextComparisonGuardV1<'_> {
    fn finalize_original_start_context_comparison_v1(&self) -> Result<()> {
        // Evaluate every physical finalizer even if a file check refuses.
        let file_result = (|| -> Result<()> {
            let after = self.file.metadata()?;
            ensure!(
                same_original_context_metadata_v1(&self.metadata, &after),
                "original comparison held file changed"
            );
            let mut bytes = vec![0; self.bytes.len()];
            std::os::unix::fs::FileExt::read_exact_at(&self.file, &mut bytes, 0)?;
            ensure!(
                bytes == self.bytes
                    && same_original_context_metadata_v1(&self.metadata, &self.file.metadata()?),
                "original comparison held bytes changed"
            );
            let component = std::ffi::CString::new(self.name.as_str())?;
            let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: retained directory and component, exact writable stat.
            ensure!(
                unsafe {
                    libc::fstatat(
                        self.directory.borrow_fd().as_raw_fd(),
                        component.as_ptr(),
                        named.as_mut_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } == 0,
                "original comparison entry disappeared"
            );
            // SAFETY: successful fstatat initialized stat.
            let named = unsafe { named.assume_init() };
            ensure!(
                named.st_dev == self.metadata.dev()
                    && named.st_ino == self.metadata.ino()
                    && named.st_nlink == 1
                    && named.st_size == self.metadata.len() as i64,
                "original comparison entry substituted"
            );
            self.directory.open_file(&self.name)?;
            let reopened = context_directory(self.root, false)?;
            let held = File::from(self.directory.borrow_fd().try_clone_to_owned()?).metadata()?;
            let current = File::from(reopened.borrow_fd().try_clone_to_owned()?).metadata()?;
            ensure!(
                held.dev() == self.directory_metadata.dev()
                    && held.ino() == self.directory_metadata.ino()
                    && current.dev() == self.directory_metadata.dev()
                    && current.ino() == self.directory_metadata.ino()
                    && held.uid() == self.directory_metadata.uid()
                    && held.mode() == self.directory_metadata.mode(),
                "original comparison context directory changed"
            );
            ensure!(
                same_original_context_metadata_v1(&self.metadata, &self.file.metadata()?),
                "original comparison file changed during finalization"
            );
            Ok(())
        })();
        let workspace_result = self.workspace.revalidate();
        let root_result = self.root.revalidate();
        ensure!(
            file_result.is_ok() && workspace_result.is_ok() && root_result.is_ok(),
            "original comparison physical custody refused"
        );
        Ok(())
    }
}

impl NativeLaunchContextV1 {
    pub(crate) fn issuer_request_id(&self) -> Result<String> {
        Ok(format!("req_{}", digest(&encode(self)?)))
    }
    fn context_sha256(&self) -> Result<String> {
        Ok(format!("{:x}", Sha256::digest(encode(self)?)))
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1
                && self.opaque_launch_plan_digest.len() == 64
                && self
                    .opaque_launch_plan_digest
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "unsupported ordinary birth context"
        );
        for id in [
            &self.intent_id,
            &self.orchestration_session_id,
            &self.shell_trace_session_id,
            &self.original_participant_id,
            &self.original_run_id,
        ] {
            ensure!(
                !id.is_empty()
                    && id.len() <= 256
                    && id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-')),
                "invalid ordinary birth identity"
            );
        }
        ensure!(
            Path::new(&self.native_home.physical_path)
                == Path::new(&self.workspace_binding.authority_store_root.physical_path)
                    .join("run/agent-hub/ordinary-native-homes")
                    .join(&self.orchestration_session_id)
                && matches!(
                    self.native_home.physical_identity,
                    super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { .. }
                ),
            "ordinary native history identity/path is invalid"
        );
        self.selections.validate()?;
        let native = &self.selections.native;
        let kind = match native.selection.harness {
            AgentHarnessFamilyV1::Codex => RuntimeBackendKindV1::Codex,
            AgentHarnessFamilyV1::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
        };
        ensure!(
            self.descriptor.schema_version == 1
                && self.descriptor.agent_id == native.realized_agent_id
                && self.descriptor.backend_id == native.backend_id
                && self.descriptor.backend_kind == kind
                && self.descriptor.execution_scope == AgentExecutionScopeV1::Host,
            "original native descriptor/selection conflict"
        );
        ensure!(
            self.selections.requested.placement != AgentSelectionPlacementV1::World
                || self.original_world_binding.is_some(),
            "original World selection has no birth binding"
        );
        for (selected, account) in [
            (&self.selections.native, &self.accounts.native),
            (&self.selections.requested, &self.accounts.requested),
        ] {
            ensure!(
                selected.config_sources == self.sources.config_sources
                    && self.sources.profiles.contains(&selected.connection_profile),
                "original selection source/profile conflict"
            );
            let entry = self
                .sources
                .inventory
                .get(&selected.realized_agent_id)
                .context("original runtime entry missing")?;
            ensure!(
                entry.file.version == 3
                    && entry.file.id == selected.realized_agent_id
                    && entry.file.config.enabled
                    && entry.file.config.api.is_none()
                    && entry.file.config.cli.is_some(),
                "original runtime entry identity conflict"
            );
            ensure!(
                account.profile_id == selected.connection_profile.id
                    && account.profile_revision == selected.connection_profile.revision
                    && account.profile_source_sha256 == selected.connection_profile.source.sha256,
                "original account/profile binding conflict"
            );
            ensure!(
                matches!(&account.principal, PlatformPrincipalV1::Unix { account, uid } if !account.is_empty() && *uid != 0),
                "original account principal invalid"
            );
            if selected.selection.harness == AgentHarnessFamilyV1::Codex {
                ensure!(
                    account
                        .account_id
                        .as_ref()
                        .is_some_and(|id| !id.trim().is_empty())
                        && account.account_id_source.is_some(),
                    "original Codex account identity missing"
                );
            } else {
                ensure!(
                    account.account_id.is_none() && account.account_id_source.is_none(),
                    "non-Codex role carries a Codex account"
                );
            }
        }
        Ok(())
    }
}

pub(crate) struct NativeStartContextInputsV1<'a> {
    pub(crate) authority: &'a Arc<HostSessionAuthority>,
    pub(crate) intended_principal: &'a PlatformPrincipalV1,
    pub(crate) effective_policy: &'a substrate_broker::Policy,
    pub(crate) workspace_binding: &'a WorkspaceBindingV1,
    pub(crate) world_binding: Option<&'a WorldBindingV1>,
    pub(crate) intent_id: &'a str,
    pub(crate) orchestration_session_id: &'a str,
    pub(crate) shell_trace_session_id: &'a str,
    pub(crate) participant_id: &'a str,
    pub(crate) run_id: &'a str,
    pub(crate) descriptor: &'a AgentDescriptorV1,
    pub(crate) opaque_launch_plan_digest: &'a str,
    pub(crate) selections: &'a OrdinaryStartSelectionsV1,
    pub(crate) sources: &'a CapturedOrdinarySourcesV1,
}

pub(crate) fn stage_native_start_context_from_ordinary_configured(
    configured: Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>,
    input: NativeStartContextInputsV1<'_>,
) -> Result<NativeLaunchContextV1> {
    let verifier = OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
        Arc::clone(&configured),
    )?;
    let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()?;
    ensure!(
        &principal == input.intended_principal,
        "ordinary birth principal changed"
    );
    ensure!(
        input.authority.bootstrap_home().identity()? == verifier.root.identity(),
        "ordinary birth authority differs from fixed configured home"
    );
    configured.revalidate()?;
    let context = stage_native_start_context(input)?;
    configured.revalidate()?;
    Ok(context)
}

pub(crate) fn stage_native_start_context(
    input: NativeStartContextInputsV1<'_>,
) -> Result<NativeLaunchContextV1> {
    let held = OrdinarySelectionAuthorityV1::from_held_authority(
        Arc::clone(input.authority),
        input.intended_principal,
    )?;
    ensure!(
        held.root.identity() == &input.workspace_binding.authority_store_root,
        "birth accepted-home identity conflict"
    );
    let workspace = TrustedWorkspaceRoot::open_exact(&input.workspace_binding.workspace_root)?;
    let project = crate::execution::workspace::find_workspace_root(Path::new(
        &input.workspace_binding.workspace_root.physical_path,
    ))
    .unwrap_or_else(|| PathBuf::from(&input.workspace_binding.workspace_root.physical_path));
    // Check live bytes only at birth. Resume authenticates the immutable original
    // record and issuer, never reselects or requires unchanged current config.
    for (source, inventory) in input
        .sources
        .config_sources
        .iter()
        .map(|s| (s, false))
        .chain([
            (&input.selections.native.inventory_source, true),
            (&input.selections.requested.inventory_source, true),
        ])
    {
        if source.scope == AgentSelectionSourceScopeV1::BuiltIn {
            continue;
        }
        let root = match source.scope {
            AgentSelectionSourceScopeV1::Global => {
                PathBuf::from(&held.root.identity().physical_path)
            }
            AgentSelectionSourceScopeV1::Workspace if inventory => project.clone(),
            AgentSelectionSourceScopeV1::Workspace => project.join(".substrate"),
            AgentSelectionSourceScopeV1::BuiltIn => {
                anyhow::bail!("unexpected built-in physical source")
            }
        };
        let path = root.join(&source.relative_path);
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .context("captured birth source unavailable")?;
        let metadata = file.metadata()?;
        ensure!(
            metadata.is_file()
                && metadata.nlink() == 1
                && metadata.len() <= MAX_CONTEXT_BYTES as u64,
            "invalid captured birth source"
        );
        let mut bytes = Vec::new();
        (&file)
            .take((MAX_CONTEXT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        let actual = crate::execution::config_model::selection_source_from_bytes(
            &root,
            &source.relative_path,
            source.scope,
            &bytes,
        )?;
        let after = file.metadata()?;
        let named = std::fs::symlink_metadata(&path)?;
        ensure!(
            bytes.len() <= MAX_CONTEXT_BYTES
                && actual == *source
                && metadata.len() == after.len()
                && metadata.ctime() == after.ctime()
                && metadata.ctime_nsec() == after.ctime_nsec()
                && metadata.mtime() == after.mtime()
                && metadata.mtime_nsec() == after.mtime_nsec()
                && named.dev() == after.dev()
                && named.ino() == after.ino()
                && named.is_file()
                && named.nlink() == 1,
            "captured birth source changed"
        );
    }
    // Both role bindings resolve before the first immutable context file exists.
    let accounts = capture_role_accounts(
        input.selections,
        input.intended_principal,
        input.effective_policy,
    )?;
    let mut sources = input.sources.clone();
    sources
        .inventory
        .get_mut(&input.selections.native.realized_agent_id)
        .and_then(|entry| entry.file.config.cli.as_mut())
        .context("original native CLI entry missing")?
        .binary = input.descriptor.binary_path.clone();
    let context = NativeLaunchContextV1 {
        version: 1,
        intent_id: input.intent_id.into(),
        orchestration_session_id: input.orchestration_session_id.into(),
        shell_trace_session_id: input.shell_trace_session_id.into(),
        original_participant_id: input.participant_id.into(),
        original_run_id: input.run_id.into(),
        workspace_binding: input.workspace_binding.clone(),
        native_home: capture_native_history_identity(
            &held.root,
            input.orchestration_session_id,
            true,
        )?
        .0,
        original_world_binding: input.world_binding.cloned(),
        descriptor: input.descriptor.clone(),
        opaque_launch_plan_digest: input.opaque_launch_plan_digest.into(),
        selections: input.selections.clone(),
        sources,
        accounts,
    };
    context.validate()?;
    let bytes = encode(&context)?;
    ensure!(
        bytes.len() <= MAX_CONTEXT_BYTES,
        "ordinary context exceeds bound"
    );
    let directory = context_directory(&held.root, true)?;
    let name = format!("{}.json", context.intent_id);
    match directory.create_file(&name) {
        Ok(mut file) => {
            file.write_all(&bytes)?;
            file.sync()?;
            directory.sync()?;
        }
        Err(error) if error.is_already_exists() => ensure!(
            read_context(&directory, &name)? == bytes,
            "original immutable birth retry conflicts"
        ),
        Err(error) => return Err(error.into()),
    }
    ensure!(
        read_context(&directory, &name)? == bytes,
        "birth context publication changed"
    );
    held.root.revalidate()?;
    input.authority.bootstrap_home().revalidate()?;
    workspace.revalidate()?;
    Ok(context)
}

fn capture_native_history_identity(
    root: &TrustedAuthorityRoot,
    original_session_id: &str,
    create: bool,
) -> Result<(
    super::host_session_authority::schema::CanonicalDirectoryV1,
    TrustedDirectory,
)> {
    ensure!(
        !original_session_id.is_empty()
            && original_session_id.len() <= 256
            && original_session_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-')),
        "invalid original native history session"
    );
    let descend = |parent: &TrustedDirectory, name| -> Result<TrustedDirectory> {
        Ok(if create {
            parent.create_directory(name)?
        } else {
            parent.open_directory(name)?
        })
    };
    root.revalidate()?;
    let run = descend(root.directory(), "run")?;
    let hub = descend(&run, "agent-hub")?;
    let homes = descend(&hub, "ordinary-native-homes")?;
    let directory = descend(&homes, original_session_id)?;
    let metadata = std::fs::metadata(native_directory_descriptor_path(&directory)?)?;
    let identity = super::host_session_authority::schema::CanonicalDirectoryV1 {
        physical_path: Path::new(&root.identity().physical_path)
            .join("run/agent-hub/ordinary-native-homes")
            .join(original_session_id)
            .to_str()
            .context("native history path is not UTF-8")?
            .into(),
        physical_identity:
            super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
    };
    let named = std::fs::symlink_metadata(&identity.physical_path)?;
    ensure!(
        named.is_dir() && (named.dev(), named.ino()) == (metadata.dev(), metadata.ino()),
        "native history named identity differs from held directory"
    );
    root.revalidate()?;
    Ok((identity, directory))
}

fn native_directory_descriptor_path(directory: &TrustedDirectory) -> Result<PathBuf> {
    let path = PathBuf::from(format!(
        "/proc/{}/fd/{}",
        std::process::id(),
        directory.borrow_fd().as_raw_fd()
    ));
    let metadata =
        std::fs::metadata(&path).context("held native history descriptor is unavailable")?;
    ensure!(
        metadata.is_dir(),
        "native history descriptor is not a directory"
    );
    directory.metadata()?;
    Ok(path)
}

/// Data claims remain independently checked by the existing service. This type
/// holds the authenticated original source; it is not a current Host-run verifier.
pub(crate) struct OrdinaryNativePromptContextV1 {
    selection: Arc<AuthenticatedOrdinarySelectionV1>,
    runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
}

/// Holds original source/history and an exact service-owned Host turn lease.
/// Acquire remains refused until genuine selected-native caller binding is proved and wired.
pub(crate) struct PreparedOrdinaryNativePromptV1 {
    selection: Arc<AuthenticatedOrdinarySelectionV1>,
    runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    _directory: TrustedDirectory,
    gateway: Arc<HostGatewayTurnLeaseV1>,
    pub(crate) home: PathBuf,
    pub(crate) model: String,
    pub(crate) openai_base_url: String,
    pub(crate) anthropic_base_url: String,
}

impl OrdinaryNativePromptContextV1 {
    pub(crate) fn new(
        selection: Arc<AuthenticatedOrdinarySelectionV1>,
        runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        descriptor: &super::validator::RuntimeSelectionDescriptor,
    ) -> Result<Self> {
        selection.require_ordinary_configured_authority()?;
        runtime.validate().map_err(anyhow::Error::msg)?;
        let original = &selection.context.descriptor;
        let kind = match descriptor.backend_kind {
            super::mapping::AgentRuntimeBackendKind::Codex => RuntimeBackendKindV1::Codex,
            super::mapping::AgentRuntimeBackendKind::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
        };
        ensure!(
            descriptor.execution_scope == AgentExecutionScope::Host
                && original.agent_id == descriptor.agent_id
                && original.backend_id == descriptor.backend_id
                && original.backend_kind == kind
                && original.protocol == descriptor.protocol
                && Path::new(&original.binary_path) == descriptor.binary_path,
            "ordinary native bridge descriptor differs from original Start"
        );
        let result = Self { selection, runtime };
        result.validate_runtime_claims()?;
        Ok(result)
    }

    fn validate_runtime_claims(&self) -> Result<()> {
        self.selection.revalidate()?;
        let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
            orchestration_session_id,
            participant_id,
            backend_id,
            ..
        } = &self.runtime
        else {
            anyhow::bail!("native bridge requires explicit HostActor claims");
        };
        ensure!(
            orchestration_session_id == self.selection.original_session_id()
                && participant_id == self.selection.current_participant_id()
                && backend_id == &self.selection.native_selection().backend_id
                && self.selection.current.caller.descriptor == self.selection.context.descriptor,
            "ordinary native runtime claims differ from held authority"
        );
        Ok(())
    }

    /// Uses only the new strict authenticated Host turn API. Acquire currently
    /// refuses while genuine selected-native capability delivery remains unproved.
    /// Policy/account checks and supervision retain observational/in-flight windows.
    pub(crate) async fn prepare(
        &self,
        acquisition: Arc<HostGatewayAcquisitionCustodyV1>,
    ) -> Result<PreparedOrdinaryNativePromptV1> {
        self.validate_runtime_claims()?;
        let request = self
            .selection
            .prepare_native_gateway_request(&self.runtime)?;
        let claim = request
            .ordinary_selection
            .context("native Host selection omitted")?;
        let directory = self.selection.open_native_history()?;
        let home = native_directory_descriptor_path(&directory)?;
        let (gateway, wiring) = acquire_host_gateway_turn_v3(claim, acquisition).await?;
        if self
            .selection
            .prepare_native_gateway_request(&self.runtime)
            .is_err()
        {
            let closed = gateway.close_and_wait().await;
            anyhow::bail!(
                "ordinary native selection changed after acquire; gateway cleanup checked={}",
                closed.is_ok()
            );
        }
        Ok(PreparedOrdinaryNativePromptV1 {
            selection: Arc::clone(&self.selection),
            runtime: self.runtime.clone(),
            _directory: directory,
            gateway,
            home,
            model: self.selection.native_selection().routed_model_id.clone(),
            openai_base_url: wiring.openai_base_url,
            anthropic_base_url: wiring.anthropic_base_url,
        })
    }
}

pub(crate) fn validate_native_loopback_endpoint(endpoint: &str) -> Result<()> {
    let port = endpoint
        .strip_prefix("http://127.0.0.1:")
        .and_then(|port| port.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .context("ordinary native requires a managed IPv4 loopback endpoint")?;
    ensure!(
        endpoint == format!("http://127.0.0.1:{port}"),
        "ordinary native requires a canonical managed loopback endpoint"
    );
    Ok(())
}

impl PreparedOrdinaryNativePromptV1 {
    pub(crate) fn revalidate(&self) -> Result<()> {
        self.selection
            .prepare_native_gateway_request(&self.runtime)?;
        let current = self.selection.open_native_history()?;
        ensure!(
            std::fs::metadata(native_directory_descriptor_path(&current)?)?.ino()
                == std::fs::metadata(&self.home)?.ino(),
            "held native history changed"
        );
        Ok(())
    }

    pub(crate) fn gateway_lease(&self) -> Arc<HostGatewayTurnLeaseV1> {
        Arc::clone(&self.gateway)
    }

    // This borrows only the per-turn local caller capability, never provider or
    // cleanup credentials. Delivery does not enable the service admission gate.
    pub(crate) fn local_route_capability(&self) -> &str {
        &self.gateway._local_route_capability
    }

    pub(crate) fn workspace_root(&self) -> &str {
        self.selection.workspace_root()
    }
}

#[derive(Clone)]
enum OrdinaryConfiguredHomeCustodyV1 {
    Service(Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>),
    Ordinary(Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>),
}

impl OrdinaryConfiguredHomeCustodyV1 {
    fn revalidate(&self) -> Result<()> {
        match self {
            Self::Service(held) => Ok(held.revalidate()?),
            Self::Ordinary(held) => Ok(held.revalidate()?),
        }
    }
}

/// Executing ordinary verifier. Construct from genuine configured custody, never
/// a request-selected path, digest, process UID, or mutable StateStore sidecar.
///
/// ```no_run
/// # #[cfg(target_os = "linux")]
/// # fn inspect(configured: std::sync::Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>) -> anyhow::Result<()> {
/// use substrate_shell::OrdinarySelectionAuthorityV1;
/// let verifier = OrdinarySelectionAuthorityV1::from_configured_accepted_home(configured)?;
/// let original = verifier.authenticate("aos_original_session")?;
/// original.revalidate()?;
/// assert_eq!(original.native_selection().placement,
///     transport_api_types::AgentSelectionPlacementV1::Host);
/// # Ok(())
/// # }
/// ```
pub struct OrdinarySelectionAuthorityV1 {
    root: Arc<TrustedAuthorityRoot>,
    authority: Arc<HostSessionAuthority>,
    configured: Option<OrdinaryConfiguredHomeCustodyV1>,
    owner_uid: u32,
}

impl OrdinarySelectionAuthorityV1 {
    /// Revalidate the existing sealed service configuration capability. This
    /// opens existing authority; it performs no bootstrap, issuance or upgrade.
    pub fn from_configured_accepted_home(
        configured: Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self> {
        configured.revalidate()?;
        let uid = u32::try_from(configured.intended_uid())?;
        let identity = configured.accepted_home();
        let root = TrustedAuthorityRoot::open_for_owner(Path::new(&identity.physical_path), uid)?;
        let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
            &identity.physical_identity;
        ensure!(
            root.identity().physical_path == identity.physical_path
                && root.identity().physical_identity
                    == super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
                        device_id: *device_id,
                        inode: *inode
                    },
            "configured ordinary root changed"
        );
        let authority_root =
            TrustedAuthorityRoot::open_for_owner(Path::new(&identity.physical_path), uid)?;
        ensure!(
            authority_root.identity() == root.identity(),
            "configured ordinary root changed during open"
        );
        configured.revalidate()?;
        root.revalidate()?;
        Ok(Self {
            root: Arc::new(root),
            authority: Arc::new(HostSessionAuthority::from_trusted_root(authority_root)?),
            configured: Some(OrdinaryConfiguredHomeCustodyV1::Service(configured)),
            owner_uid: uid,
        })
    }

    /// Bind ordinary execution to the distinct fixed-source configured seal.
    ///
    /// ```no_run
    /// # #[cfg(target_os = "linux")]
    /// # fn inspect(carrier: &transport_api_types::InstallBootstrapContextCarrierV1) -> anyhow::Result<()> {
    /// let configured = std::sync::Arc::new(
    ///     config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority(carrier)?);
    /// let verifier = substrate_shell::OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(configured)?;
    /// # let _ = verifier;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_ordinary_configured_accepted_home(
        configured: Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self> {
        configured.revalidate()?;
        let (principal, _) =
            crate::execution::install_bootstrap::current_unix_principal_and_home()?;
        let PlatformPrincipalV1::Unix { uid, .. } = &principal else {
            anyhow::bail!("ordinary configured principal is not Unix");
        };
        ensure!(
            u64::from(*uid) == configured.intended_uid(),
            "ordinary configured principal changed"
        );
        let identity = configured.accepted_home();
        let authority_root =
            TrustedAuthorityRoot::open_for_owner(Path::new(&identity.physical_path), *uid)?;
        let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
            &identity.physical_identity;
        ensure!(
            authority_root.identity().physical_path == identity.physical_path
                && authority_root.identity().physical_identity
                    == super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
                        device_id: *device_id,
                        inode: *inode
                    },
            "ordinary configured physical root changed"
        );
        let mut verifier = Self::from_held_authority(
            Arc::new(HostSessionAuthority::from_trusted_root(authority_root)?),
            &principal,
        )?;
        configured.revalidate()?;
        verifier.configured = Some(OrdinaryConfiguredHomeCustodyV1::Ordinary(configured));
        Ok(verifier)
    }

    fn from_held_authority(
        authority: Arc<HostSessionAuthority>,
        principal: &PlatformPrincipalV1,
    ) -> Result<Self> {
        let PlatformPrincipalV1::Unix { uid, .. } = principal else {
            anyhow::bail!("ordinary authority requires a trusted Unix principal");
        };
        crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
        let held = authority.bootstrap_home();
        let identity = held.identity()?.clone();
        let root = TrustedAuthorityRoot::open_for_owner(Path::new(&identity.physical_path), *uid)?;
        ensure!(
            root.identity() == &identity,
            "held ordinary accepted home changed"
        );
        held.revalidate()?;
        root.revalidate()?;
        let owner_uid = *uid;
        Ok(Self {
            root: Arc::new(root),
            authority,
            configured: None,
            owner_uid,
        })
    }

    /// Authenticate original selection. Actual caller/World/policy/account and
    /// current applied run still require their separate executing admission.
    pub fn authenticate(
        &self,
        original_session_id: &str,
    ) -> Result<AuthenticatedOrdinarySelectionV1> {
        if let Some(configured) = &self.configured {
            configured.revalidate()?;
        }
        self.root.revalidate()?;
        let current = self
            .authority
            .resolve_current_exact(original_session_id, None)?;
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            ..
        } = &current.authority.origin;
        let bytes = read_context(
            &context_directory(&self.root, false)?,
            &format!("{intent_id}.json"),
        )?;
        let context: NativeLaunchContextV1 = decode(&bytes)?;
        context.validate()?;
        ensure!(
            context.intent_id == *intent_id
                && context.issuer_request_id()? == *issuer_request_id
                && context.orchestration_session_id == original_session_id,
            "original selected issuer binding missing or substituted"
        );
        let result = AuthenticatedOrdinarySelectionV1 {
            root: Arc::clone(&self.root),
            authority: Arc::clone(&self.authority),
            current,
            context: Arc::new(context),
            configured: self.configured.clone(),
            owner_uid: self.owner_uid,
            world_observers: Arc::new(
                super::ordinary_world_observers::OrdinaryWorldObserverTasksV1::default(),
            ),
        };
        result.revalidate()?;
        Ok(result)
    }
}

/// Opaque ordinary selection/current-run and policy/account authentication.
/// This grants no process placement, startup, transport or provider authority.
/// Revalidation is a fresh boundary check, not a lock across later effects.
pub struct OrdinaryHostGatewayAdmissionV1 {
    configured: OrdinaryConfiguredHomeCustodyV1,
    root: Arc<TrustedAuthorityRoot>,
    authority: Arc<HostSessionAuthority>,
    owner_uid: u32,
    workspace: TrustedWorkspaceRoot,
    context: Arc<NativeLaunchContextV1>,
    claim: transport_api_types::OrdinaryGatewaySelectionContextV1,
    original_payload: super::host_session_authority::schema::AuthorityObjectCommitmentV1,
    original_application: super::host_session_authority::schema::AuthorityObjectRefV1,
    current_application: super::host_session_authority::schema::AuthorityObjectRefV1,
}

impl OrdinarySelectionAuthorityV1 {
    /// Authenticate exact original selection, a genuine current Host run, and
    /// fresh native policy/account custody under this configured authority.
    ///
    /// ```no_run
    /// # #[cfg(target_os = "linux")]
    /// # fn inspect(authority: &substrate_shell::OrdinarySelectionAuthorityV1,
    /// # claim: &transport_api_types::OrdinaryGatewaySelectionContextV1) -> anyhow::Result<()> {
    /// let admission = authority.authenticate_host_gateway_v1(claim)?;
    /// admission.revalidate_before_effect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn authenticate_host_gateway_v1(
        &self,
        claim: &transport_api_types::OrdinaryGatewaySelectionContextV1,
    ) -> Result<OrdinaryHostGatewayAdmissionV1> {
        let configured = self
            .configured
            .as_ref()
            .context("ordinary Host gateway requires configured authority custody")?;
        let (context, observation) = authenticate_host_gateway_observation(
            &self.authority,
            &self.root,
            configured,
            self.owner_uid,
            claim,
        )?;
        let run = observation.current_run();
        let admission = OrdinaryHostGatewayAdmissionV1 {
            configured: configured.clone(),
            root: Arc::clone(&self.root),
            authority: Arc::clone(&self.authority),
            owner_uid: self.owner_uid,
            workspace: TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?,
            original_payload: run.original_start_payload_commitment().clone(),
            original_application: run.original_start_application_result_ref().clone(),
            current_application: run.application_result_ref().clone(),
            context: Arc::new(context),
            claim: claim.clone(),
        };
        admission.revalidate_before_effect()?;
        Ok(admission)
    }
}

impl OrdinaryHostGatewayAdmissionV1 {
    /// The immutable original native selection; never a request-selected tuple.
    pub fn native_selection(&self) -> &ResolvedAgentSelectionV1 {
        &self.context.selections.native
    }

    /// The configured principal joined to both original role bindings.
    pub fn principal(&self) -> &PlatformPrincipalV1 {
        &self.context.accounts.native.principal
    }

    /// The exact original/current physically held workspace.
    pub fn workspace_root(&self) -> &str {
        &self.context.workspace_binding.workspace_root.physical_path
    }

    /// Resolve a transient handoff for this exact admitted native account. This
    /// is credential custody for a separate service-owned managed Host process;
    /// it confers neither installation nor service-instance freshness authority.
    ///
    /// ```no_run
    /// # #[cfg(target_os = "linux")]
    /// # fn inspect(admission: &substrate_shell::OrdinaryHostGatewayAdmissionV1) -> anyhow::Result<()> {
    /// let handoff = admission.resolve_managed_host_auth_v1()?;
    /// drop(handoff); // Hand directly to secret-FD custody; never serialize to native state.
    /// # Ok(()) }
    /// ```
    pub fn resolve_managed_host_auth_v1(
        &self,
    ) -> Result<transport_api_types::GatewayIntegratedAuthPayloadV1> {
        self.revalidate_before_effect()?;
        let home = self.authority.bootstrap_home();
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            Path::new(self.workspace_root()),
            &home,
        )?;
        let (binding, codex) =
            capture_role_account(self.native_selection(), self.principal(), &policy)?;
        let original = &self.context.accounts.native;
        ensure!(
            binding.principal == original.principal
                && binding.profile_id == original.profile_id
                && binding.profile_revision == original.profile_revision
                && binding.profile_source_sha256 == original.profile_source_sha256
                && binding.account_id == original.account_id,
            "managed Host account binding changed"
        );
        let handoff = match self.native_selection().selection.harness {
            AgentHarnessFamilyV1::Codex => transport_api_types::GatewayIntegratedAuthPayloadV1 {
                backend_id: self.native_selection().backend_id.clone(),
                cli_codex: Some(codex.context("native Codex handoff missing")?),
                api_env: None,
            },
            AgentHarnessFamilyV1::ClaudeCode => {
                crate::builtins::world_gateway::resolve_claude_code_integrated_auth(
                    &self.native_selection().backend_id,
                    &policy,
                )?
                .context("native Claude handoff missing")?
            }
        };
        self.revalidate_before_effect()?;
        Ok(handoff)
    }

    /// Reacquire a borrowed genuine proof internally and repeat the full
    /// original/current, policy and native-account permission checks.
    ///
    /// Callers still need independently admitted placement and ownership, and
    /// fresh permission for Acquire, gateway/native start and retry, and explicit
    /// Revalidate. Periodic supervision leaves observational/in-flight windows;
    /// this does not establish per-provider-request or atomic revocation.
    pub fn revalidate_before_effect(&self) -> Result<()> {
        self.workspace.revalidate()?;
        let (context, observation) = authenticate_host_gateway_observation(
            &self.authority,
            &self.root,
            &self.configured,
            self.owner_uid,
            &self.claim,
        )?;
        ensure!(
            encode(&context)? == encode(self.context.as_ref())?
                && observation
                    .current_run()
                    .original_start_payload_commitment()
                    == &self.original_payload
                && observation
                    .current_run()
                    .original_start_application_result_ref()
                    == &self.original_application
                && observation.current_run().application_result_ref() == &self.current_application,
            "ordinary Host gateway original or current application changed"
        );
        self.workspace.revalidate()?;
        observation.revalidate()?;
        self.root.revalidate()?;
        self.configured.revalidate()?;
        Ok(())
    }
}

fn authenticate_host_gateway_observation<'authority>(
    authority: &'authority HostSessionAuthority,
    root: &TrustedAuthorityRoot,
    configured: &OrdinaryConfiguredHomeCustodyV1,
    owner_uid: u32,
    claim: &transport_api_types::OrdinaryGatewaySelectionContextV1,
) -> Result<(
    NativeLaunchContextV1,
    super::host_session_authority::current_host_run::CurrentHostGatewayRunObservationV1<'authority>,
)> {
    claim.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        owner_uid != 0,
        "ordinary Host gateway requires a nonroot configured principal"
    );
    let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
        orchestration_session_id,
        participant_id,
        backend_id,
        run_id,
    } = &claim.runtime
    else {
        anyhow::bail!("ordinary Host gateway requires exact HostActor claims");
    };
    configured.revalidate()?;
    root.revalidate()?;
    let home = authority.bootstrap_home();
    ensure!(
        home.identity()? == root.identity(),
        "configured authority root differs"
    );
    home.revalidate()?;
    let observation = authority.observe_current_host_gateway_run_v1(
        orchestration_session_id,
        participant_id,
        backend_id,
        run_id,
    )?;
    let original = observation.original_start();
    let bytes = read_context(
        &context_directory(root, false)?,
        &format!(
            "{}.json",
            observation.current_run().original_start_intent_id()
        ),
    )?;
    let context: NativeLaunchContextV1 = decode(&bytes)?;
    validate_host_gateway_original_join(&context, &observation, claim, owner_uid)?;
    let workspace = TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
    let (native_home, directory) =
        capture_native_history_identity(root, &context.orchestration_session_id, false)?;
    ensure!(
        native_home == context.native_home,
        "original native history changed"
    );
    directory.metadata()?;
    check_host_gateway_permission(authority, &context, observation.current_policy())?;
    ensure!(
        read_context(
            &context_directory(root, false)?,
            &format!("{}.json", original.intent_id)
        )? == bytes,
        "immutable original selected context changed"
    );
    workspace.revalidate()?;
    home.revalidate()?;
    root.revalidate()?;
    configured.revalidate()?;
    observation.revalidate()?;
    Ok((context, observation))
}

fn validate_host_gateway_original_join(
    context: &NativeLaunchContextV1,
    observation: &super::host_session_authority::current_host_run::CurrentHostGatewayRunObservationV1<'_>,
    claim: &transport_api_types::OrdinaryGatewaySelectionContextV1,
    owner_uid: u32,
) -> Result<()> {
    context.validate()?;
    let original = observation.original_start();
    let current = observation.current_run();
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        ..
    } = &original.state
    else {
        anyhow::bail!("ordinary Host gateway original Start is not Applied");
    };
    ensure!(
        original.mode == HostSessionTransitionModeV1::Start
            && original.intent_id == context.intent_id
            && original.intent_id == current.original_start_intent_id()
            && original.issuer_request_id == context.issuer_request_id()?
            && original.issuer_request_id == current.original_start_issuer_request_id()
            && &original.payload_commitment == current.original_start_payload_commitment()
            && application_result_ref == current.original_start_application_result_ref()
            && original.orchestration_session_id == context.orchestration_session_id
            && original.orchestration_session_id == current.orchestration_session_id()
            && original.shell_trace_session_id == context.shell_trace_session_id
            && original.target_authoritative_participant_id == context.original_participant_id
            && original.run_id == context.original_run_id
            && original.run_id == current.original_start_run_id()
            && original.workspace_binding == context.workspace_binding
            && original.world_binding == context.original_world_binding
            && observation.original_descriptor() == &context.descriptor
            && context.descriptor.backend_id == current.backend_id()
            && context.workspace_binding.authority_store_id == current.authority_store_id(),
        "ordinary Host gateway original/current selected birth does not join"
    );
    let expected = transport_api_types::OrdinaryStartSelectionCarrierV1 {
        schema_version: 1,
        authority_store_id: context.workspace_binding.authority_store_id.clone(),
        original_start_intent_id: context.intent_id.clone(),
        original_start_issuer_request_id: context.issuer_request_id()?,
        original_session_id: context.orchestration_session_id.clone(),
        context_sha256: context.context_sha256()?,
        native_selection_sha256: context.selections.native.selection_digest.clone(),
        requested_selection_sha256: context.selections.requested.selection_digest.clone(),
    };
    expected.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        claim.start == expected,
        "received ordinary carrier differs from authenticated birth"
    );
    for principal in [
        &context.accounts.native.principal,
        &context.accounts.requested.principal,
    ] {
        ensure!(
            matches!(principal, PlatformPrincipalV1::Unix { uid, .. } if *uid == owner_uid),
            "original role principal differs from configured authority"
        );
        crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
    }
    Ok(())
}

fn check_host_gateway_permission(
    authority: &HostSessionAuthority,
    context: &NativeLaunchContextV1,
    current_policy: &PolicyObjectHashInputV1,
) -> Result<()> {
    use super::dispatch_contract::*;
    let home = authority.bootstrap_home();
    let workspace = Path::new(&context.workspace_binding.workspace_root.physical_path);
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        workspace, &home,
    )?;
    let snapshot = crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
        workspace, &home,
    )?;
    ensure!(
        snapshot.snapshot_hash == current_policy.canonical_policy_snapshot_sha256,
        "current native policy no longer joins authenticated authority"
    );
    let selected = &context.selections.native;
    let envelope = DispatchRequestEnvelope {
        caller_kind: DispatchCallerKind::HumanStart,
        baseline_kind: DispatchBaselineKind::InventoryLaunch,
        backend_id: Some(selected.backend_id.clone()),
        orchestration_session_id: Some(context.orchestration_session_id.clone()),
        requested_execution_scope_override: Some(AgentExecutionScope::Host),
        capability_overrides: DispatchCapabilityOverrideSet::default(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: AgentExecutionScope::Host,
            host_execution_client_start: HostExecutionClientStart::StartNow,
            attach_mode_preference: AttachModePreference::ContinuityRequired,
        },
        has_prompt_payload: true,
    };
    let contract = resolve_inventory_contract_for_exact_backend(
        workspace,
        &context.sources.effective_config(),
        &context.sources.inventory,
        &policy,
        &envelope,
        AgentExecutionScope::Host,
    )
    .map_err(|error| anyhow::anyhow!("{error}"))?
    .context("current policy refuses original native runtime")?;
    let descriptor = super::validator::materialize_runtime_descriptor(&contract)
        .map_err(|error| anyhow::anyhow!("{}", error.reason))?;
    let kind = match descriptor.backend_kind {
        super::mapping::AgentRuntimeBackendKind::Codex => RuntimeBackendKindV1::Codex,
        super::mapping::AgentRuntimeBackendKind::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
    };
    ensure!(
        kind == context.descriptor.backend_kind
            && descriptor.agent_id == context.descriptor.agent_id
            && descriptor.backend_id == context.descriptor.backend_id
            && descriptor.protocol == context.descriptor.protocol
            && descriptor.binary_path == Path::new(&context.descriptor.binary_path)
            && descriptor.execution_scope == AgentExecutionScope::Host
            && contract.capabilities.session_start
            && contract.capabilities.session_resume
            && contract.capabilities.llm
            && contract.capabilities.event_stream,
        "current native runtime/capability permission differs from original Start"
    );
    let effective = &contract.effective_policy;
    ensure!(
        !effective.llm_require_approval,
        "ordinary native approval admission unavailable"
    );
    ensure!(
        !effective.world_fs_require_world && !effective.world_fs_caged_required,
        "current policy requires World placement; managed Host gateway refused"
    );
    ensure!(
        effective
            .llm_allowed_backends
            .iter()
            .any(|backend| backend == &selected.backend_id),
        "current LLM policy refuses original native backend"
    );
    let (protocol, auth) = match selected.selection.harness {
        AgentHarnessFamilyV1::Codex => ("openai.responses", "codex_subscription"),
        AgentHarnessFamilyV1::ClaudeCode => ("anthropic.messages", "anthropic_api_key"),
    };
    for (axis, value, allowed) in [
        (
            "router",
            "substrate_gateway",
            &effective.llm_constraints_routers,
        ),
        (
            "provider",
            selected.selection.provider.as_str(),
            &effective.llm_constraints_providers,
        ),
        ("protocol", protocol, &effective.llm_constraints_protocols),
        (
            "auth authority",
            auth,
            &effective.llm_constraints_auth_authorities,
        ),
    ] {
        ensure!(
            allowed.is_empty() || allowed.iter().any(|candidate| candidate == value),
            "current native gateway {axis} permission is revoked"
        );
    }
    let (binding, handoff) =
        capture_role_account(selected, &context.accounts.native.principal, effective)?;
    let original = &context.accounts.native;
    ensure!(
        binding.principal == original.principal
            && binding.profile_id == original.profile_id
            && binding.profile_revision == original.profile_revision
            && binding.profile_source_sha256 == original.profile_source_sha256
            && binding.account_id == original.account_id,
        "current native account/profile custody conflicts with original Start"
    );
    // Same-account token refresh is permitted by the existing resolver. No
    // secret is returned from this authentication result or retained in it.
    drop(handoff);
    if selected.selection.harness == AgentHarnessFamilyV1::ClaudeCode {
        drop(
            crate::builtins::world_gateway::resolve_claude_code_integrated_auth(
                &selected.backend_id,
                effective,
            )?
            .context("native Claude handoff missing")?,
        );
    }
    let end = crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
        workspace, &home,
    )?;
    ensure!(
        end.snapshot_hash == snapshot.snapshot_hash,
        "native policy changed during authentication"
    );
    Ok(())
}

/// Opaque original-selection result with a genuine held executing authority.
/// It grants no current-run, policy, account-entitlement or transport permission.
pub struct AuthenticatedOrdinarySelectionV1 {
    root: Arc<TrustedAuthorityRoot>,
    authority: Arc<HostSessionAuthority>,
    current: ResolvedCurrentAuthorityV1,
    context: Arc<NativeLaunchContextV1>,
    configured: Option<OrdinaryConfiguredHomeCustodyV1>,
    owner_uid: u32,
    world_observers: Arc<super::ordinary_world_observers::OrdinaryWorldObserverTasksV1>,
}

impl AuthenticatedOrdinarySelectionV1 {
    /// Borrow the exact retained original context under read-only source custody.
    /// No HSA resolution, account acquisition, staging or current-policy grant.
    pub(crate) fn with_original_start_context_for_helper_comparison_v1(
        &self,
        compare: impl FnOnce(&NativeLaunchContextV1) -> Result<()>,
    ) -> Result<()> {
        if !matches!(
            self.configured.as_ref(),
            Some(OrdinaryConfiguredHomeCustodyV1::Ordinary(_))
        ) {
            anyhow::bail!("selected helper configured custody refused");
        }
        self.with_original_start_context_read_source_v1(compare)?
            .map_err(|_| anyhow::anyhow!("selected helper source comparison refused"))
    }

    fn with_original_start_context_read_source_v1<T>(
        &self,
        read: impl FnOnce(&NativeLaunchContextV1) -> Result<T>,
    ) -> Result<Result<T>> {
        self.with_original_start_context_guarded_source_v1(|context, _verify| read(context))
    }

    fn with_original_start_context_guarded_source_v1<T>(
        &self,
        read: impl FnOnce(&NativeLaunchContextV1, &dyn Fn() -> Result<()>) -> Result<T>,
    ) -> Result<Result<T>> {
        let Some(configured) = self.configured.as_ref() else {
            anyhow::bail!("selected helper configured custody refused");
        };
        configured
            .revalidate()
            .map_err(|_| anyhow::anyhow!("selected helper configured custody refused"))?;
        self.root
            .revalidate()
            .map_err(|_| anyhow::anyhow!("selected helper root custody refused"))?;
        let captured =
            capture_original_start_context_for_helper_comparison_v1(&self.root, &self.context);
        let semantic = match &captured {
            Ok(guard) => (|| {
                // Both independently sealed capabilities may guard existing
                // sources. Producer/helper authority remains ordinary-only.
                let accepted = match configured {
                    OrdinaryConfiguredHomeCustodyV1::Service(held) => held.accepted_home(),
                    OrdinaryConfiguredHomeCustodyV1::Ordinary(held) => held.accepted_home(),
                };
                ensure!(
                    accepted.physical_path == self.root.identity().physical_path
                        && matches!((&accepted.physical_identity, &self.root.identity().physical_identity),
                            (config_projection::DirectoryPhysicalIdentityV1::Linux { device_id: a, inode: b },
                             super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { device_id: c, inode: d })
                            if a == c && b == d),
                    "selected helper accepted root differs"
                );
                self.context.validate()?;
                let decoded: NativeLaunchContextV1 = decode(&guard.bytes)?;
                decoded.validate()?;
                ensure!(
                    guard.bytes == encode(self.context.as_ref())?,
                    "selected helper original encoding differs"
                );
                let verify = || -> Result<()> {
                    guard
                        .finalize_original_start_context_comparison_v1()
                        .map_err(|_| anyhow::anyhow!("selected helper source integrity refused"))?;
                    configured
                        .revalidate()
                        .map_err(|_| anyhow::anyhow!("selected helper source integrity refused"))?;
                    self.root
                        .revalidate()
                        .map_err(|_| anyhow::anyhow!("selected helper source integrity refused"))
                };
                read(&decoded, &verify)
            })(),
            Err(_) => Err(anyhow::anyhow!("selected helper context capture refused")),
        };
        let context_exit = captured
            .as_ref()
            .map(|guard| guard.finalize_original_start_context_comparison_v1());
        let configured_exit = configured.revalidate();
        let root_exit = self.root.revalidate();
        if !matches!(context_exit, Ok(Ok(()))) || configured_exit.is_err() || root_exit.is_err() {
            anyhow::bail!("selected helper source integrity refused");
        }
        Ok(semantic)
    }

    /// Read current identity while retaining original selection source custody.
    /// The callback is synchronous and receives no refreshed send authority.
    #[cfg(target_os = "linux")]
    pub(crate) fn with_ordinary_inspection_sources_v1<T>(
        &self,
        inspect: impl FnOnce(
            &NativeLaunchContextV1,
            &super::host_session_authority::facade::OrdinarySelectionReadSourceV1,
        ) -> Result<T>,
    ) -> Result<T> {
        self.with_original_start_context_read_source_v1(|context| {
            self.authority.with_ordinary_selection_read_source_v1(&context.orchestration_session_id, |facts| {
                let current = &facts.current;
                let original = &facts.original_start;
                ensure!(current.observation.authority_store_id == context.workspace_binding.authority_store_id
                    && current.observation.bootstrap_home == context.workspace_binding.authority_store_root
                    && self.root.identity() == &context.workspace_binding.authority_store_root
                    && original.intent_id == context.intent_id
                    && original.issuer_request_id == context.issuer_request_id()?
                    && original.orchestration_session_id == context.orchestration_session_id
                    && original.shell_trace_session_id == context.shell_trace_session_id
                    && original.target_authoritative_participant_id == context.original_participant_id
                    && original.run_id == context.original_run_id
                    && original.workspace_binding == context.workspace_binding
                    && original.world_binding == context.original_world_binding
                    && facts.original_descriptor == context.descriptor
                    && facts.original_contract.backend_id == context.descriptor.backend_id
                    && current.authority.workspace_binding == context.workspace_binding
                    && current.authority.authoritative_participant_lineage.contains(&context.original_participant_id),
                    "ordinary inspection original/current identity differs");
                for principal in [&context.accounts.native.principal, &context.accounts.requested.principal] {
                    ensure!(matches!(principal, PlatformPrincipalV1::Unix { uid, .. } if *uid == self.owner_uid),
                        "ordinary inspection role principal differs");
                    crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
                }
                let history = self.open_native_history()?;
                let workspace = TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
                workspace.revalidate()?;
                let outcome = inspect(context, facts);
                // Original descriptors stay live through the consumer. Reopening
                // here verifies the same named identity, never substitutes custody.
                let history_exit = history.metadata();
                let named_history_exit = self.open_native_history();
                let workspace_exit = workspace.revalidate();
                if history_exit.is_err() || named_history_exit.is_err() || workspace_exit.is_err() {
                    anyhow::bail!("ordinary inspection physical sources changed");
                }
                outcome
            }).map_err(|error| anyhow::anyhow!(error.to_string()))?
        })?
    }

    /// Existing-only accepted-control scope. Admission is not required for an
    /// ephemeral task. The same original read/Root/Supervisor guards and native
    /// context verifier survive the synchronous callback and every refusal.
    #[cfg(target_os = "linux")]
    pub(crate) fn with_ordinary_accepted_control_sources_v1<T>(
        &self,
        operation: impl FnOnce(
            &NativeLaunchContextV1,
            &super::host_session_authority::facade::ResolvedCurrentAuthorityV1,
            &super::host_session_authority::store::AcceptedWorkAuthorityPhysicalSnapshotV1,
            Option<&[u8]>,
            &dyn Fn() -> Result<()>,
        ) -> Result<T>,
    ) -> Result<T> {
        self.with_original_start_context_guarded_source_v1(|_, verify_context| {
            self.with_ordinary_inspection_sources_v1(|context, source| {
                let history = self.open_native_history()?;
                let workspace =
                    TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
                source
                    .with_accepted_control_sources(|snapshot, supervisor, verify_sources| {
                        let verify = || -> Result<()> {
                            verify_context()?;
                            verify_sources().map_err(|error| {
                                anyhow::anyhow!(
                                    "ordinary accepted original source changed: {error:?}"
                                )
                            })?;
                            workspace.revalidate()?;
                            history.metadata()?;
                            self.open_native_history()?;
                            Ok(())
                        };
                        verify()?;
                        let result =
                            operation(context, &source.current, snapshot, supervisor, &verify);
                        verify()?;
                        result
                    })
                    .map_err(|error| anyhow::anyhow!("{error}"))?
            })
        })?
    }

    /// Source custody for a supervisor control commit. The consumer separately
    /// validates the exact control action and its current policy; this callback
    /// supplies no read-to-mutation or later-send permission.
    #[cfg(target_os = "linux")]
    pub(crate) fn with_ordinary_supervisor_control_sources_v1<T>(
        &self,
        target: Option<&str>,
        operation: impl FnOnce(
            &NativeLaunchContextV1,
            &super::host_session_authority::facade::OrdinarySupervisorControlReadSourceV1,
            &dyn Fn() -> Result<()>,
        ) -> Result<T>,
    ) -> Result<T> {
        self.with_original_start_context_guarded_source_v1(|context, verify_context| {
            let source = self.authority.capture_ordinary_supervisor_control_source_for_target_v1(&context.orchestration_session_id, target)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let facts = &source.facts;
            let current = &facts.current;
            let original = &facts.original_start;
            ensure!(current.observation.authority_store_id == context.workspace_binding.authority_store_id
                && current.observation.bootstrap_home == context.workspace_binding.authority_store_root
                && self.root.identity() == &context.workspace_binding.authority_store_root
                && original.intent_id == context.intent_id && original.issuer_request_id == context.issuer_request_id()?
                && original.orchestration_session_id == context.orchestration_session_id
                && original.shell_trace_session_id == context.shell_trace_session_id
                && original.target_authoritative_participant_id == context.original_participant_id
                && original.run_id == context.original_run_id && original.workspace_binding == context.workspace_binding
                && original.world_binding == context.original_world_binding && facts.original_descriptor == context.descriptor
                && facts.original_contract.backend_id == context.descriptor.backend_id
                && current.authority.workspace_binding == context.workspace_binding
                && current.authority.authoritative_participant_lineage.contains(&context.original_participant_id),
                "ordinary control original/current source differs");
            for principal in [&context.accounts.native.principal, &context.accounts.requested.principal] {
                ensure!(matches!(principal, PlatformPrincipalV1::Unix { uid, .. } if *uid == self.owner_uid), "ordinary control role principal differs");
                crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
            }
            let history = self.open_native_history()?;
            let workspace = TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
            let verify = || -> Result<()> {
                verify_context()?;
                source.verify_sources().map_err(|error| anyhow::anyhow!("{error}"))?;
                workspace.revalidate()?;
                history.metadata()?;
                self.open_native_history()?;
                Ok(())
            };
            verify()?;
            let outcome = operation(context, &source, &verify);
            verify()?;
            outcome
        })?
    }

    /// Hold original native/configured/workspace/history and admission-owner
    /// facts through a control publication. These callbacks grant no action:
    /// the owner authenticates fresh Stop permission and exact work coverage.
    /// `record_publication` is called only after its actual storage publication
    /// succeeds, including when later semantic work refuses.
    pub(crate) fn with_ordinary_lifecycle_control_sources_v1<T>(
        &self,
        target: &str,
        operation: impl FnOnce(
            &NativeLaunchContextV1,
            &super::host_session_authority::facade::OrdinaryRetainedLifecycleControlReadSourceV1,
            &dyn Fn(Option<&str>) -> Result<()>,
            &dyn Fn(&[u8]) -> Result<()>,
        ) -> Result<T>,
    ) -> Result<T> {
        self.with_original_start_context_guarded_source_v1(|context, verify_context| {
            let source = self.authority.capture_ordinary_retained_lifecycle_control_source_v1(&context.orchestration_session_id, target)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let facts = &source.facts;
            let current = &facts.current;
            let original = &facts.original_start;
            ensure!(current.observation.authority_store_id == context.workspace_binding.authority_store_id
                && current.observation.bootstrap_home == context.workspace_binding.authority_store_root
                && self.root.identity() == &context.workspace_binding.authority_store_root
                && original.intent_id == context.intent_id && original.issuer_request_id == context.issuer_request_id()?
                && original.orchestration_session_id == context.orchestration_session_id
                && original.shell_trace_session_id == context.shell_trace_session_id
                && original.target_authoritative_participant_id == context.original_participant_id
                && original.run_id == context.original_run_id && original.workspace_binding == context.workspace_binding
                && original.world_binding == context.original_world_binding && facts.original_descriptor == context.descriptor
                && facts.original_contract.backend_id == context.descriptor.backend_id
                && current.authority.workspace_binding == context.workspace_binding
                && current.authority.authoritative_participant_lineage.contains(&context.original_participant_id),
                "ordinary lifecycle original/current source differs");
            for principal in [&context.accounts.native.principal, &context.accounts.requested.principal] {
                ensure!(matches!(principal, PlatformPrincipalV1::Unix { uid, .. } if *uid == self.owner_uid), "ordinary lifecycle role principal differs");
                crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
            }
            let history = self.open_native_history()?;
            let workspace = TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?;
            let verify_native = || -> Result<()> {
                verify_context()?;
                workspace.revalidate()?;
                history.metadata()?;
                self.open_native_history()?;
                Ok(())
            };
            let verify = |owned_temp: Option<&str>| -> Result<()> {
                verify_native()?;
                source.verify_before_publication(owned_temp).map_err(|error| anyhow::anyhow!("{error}"))
            };
            let published = std::cell::RefCell::new(None::<Vec<u8>>);
            let record_publication = |bytes: &[u8]| -> Result<()> {
                // Remember the actual owner's known publication even if an
                // original-source or native finalizer now refuses the result.
                ensure!(published.borrow().is_none(), "lifecycle scope permits one admission publication");
                *published.borrow_mut() = Some(bytes.to_vec());
                source.verify_after_publication(bytes).map_err(|error| anyhow::anyhow!("{error}"))?;
                verify_native()
            };
            verify(None)?;
            let outcome = operation(context, &source, &verify, &record_publication);
            let final_sources = match published.borrow().as_ref() {
                Some(bytes) => source.verify_after_publication(bytes),
                None => source.verify_before_publication(None),
            };
            // Finalize independent original captures on success and refusal.
            let final_native = verify_native();
            final_sources.map_err(|error| anyhow::anyhow!("{error}"))?;
            final_native?;
            outcome
        })?
    }

    /// Refresh original selection after genuine registration while retaining
    /// the same held authority and configured seal. This does not verify a
    /// current applied Host run or current policy/account permission.
    pub(crate) fn refresh_original_selection(&self) -> Result<Self> {
        let mut refreshed = OrdinarySelectionAuthorityV1 {
            root: Arc::clone(&self.root),
            authority: Arc::clone(&self.authority),
            configured: self.configured.clone(),
            owner_uid: self.owner_uid,
        }
        .authenticate(self.original_session_id())?;
        refreshed.world_observers = Arc::clone(&self.world_observers);
        Ok(refreshed)
    }

    pub(crate) fn retain_issued_world_observer(&self, task: tokio::task::JoinHandle<()>) {
        self.world_observers.retain_issued_world_observer(task);
    }

    pub(crate) async fn join_issued_world_observers(&self) -> Result<()> {
        self.world_observers.join_issued_world_observers().await
    }

    /// Separate transactions are not atomic. Root-only movement is permitted
    /// while the authenticated authority/caller remains identical. Registration
    /// advances authority and requires refresh from the same genuine held Arc;
    /// consumers must revalidate again immediately before their actual effect.
    pub fn revalidate(&self) -> Result<()> {
        if let Some(configured) = &self.configured {
            configured.revalidate()?;
        }
        self.root.revalidate()?;
        let context = &self.context;
        let held = self.authority.bootstrap_home();
        ensure!(
            held.identity()? == &context.workspace_binding.authority_store_root
                && self.root.identity() == held.identity()?,
            "ordinary configured home custody changed"
        );
        let current = self
            .authority
            .resolve_current_exact(&context.orchestration_session_id, None)?;
        ensure!(
            current.observation.authority_record_commitment
                == self.current.observation.authority_record_commitment
                && current.caller == self.current.caller,
            "ordinary current authority/caller changed; refresh required"
        );

        let (original_descriptor, original_contract, _) = self.original_start_material()?;
        let preserved = self.authority.read_preserved_start_root_v2()?;
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } = &current.authority.origin;
        let original = preserved
            .transition_intent_map
            .get(intent_id)
            .context("genuine original Start missing")?;
        ensure!(
            preserved.authority_store_id == context.workspace_binding.authority_store_id
                && preserved.bootstrap_home == context.workspace_binding.authority_store_root
                && original.mode == HostSessionTransitionModeV1::Start
                && matches!(
                    &original.state,
                    HostSessionTransitionIntentStateV2::Applied { .. }
                )
                && original.intent_id == context.intent_id
                && original.issuer_request_id == *issuer_request_id
                && original.payload_commitment == *payload_commitment
                && original.orchestration_session_id == context.orchestration_session_id
                && original.shell_trace_session_id == context.shell_trace_session_id
                && original.target_authoritative_participant_id == context.original_participant_id
                && original.run_id == context.original_run_id
                && original.workspace_binding == context.workspace_binding
                && original.world_binding == context.original_world_binding,
            "original applied Start does not join immutable selected birth"
        );
        ensure!(
            original_descriptor == context.descriptor
                && original_contract.backend_id == context.descriptor.backend_id
                && current.authority.workspace_binding == context.workspace_binding
                && current
                    .authority
                    .authoritative_participant_lineage
                    .contains(&context.original_participant_id),
            "ordinary original/current lineage conflict"
        );
        ensure!(
            context.issuer_request_id()? == *issuer_request_id
                && read_context(
                    &context_directory(&self.root, false)?,
                    &format!("{}.json", context.intent_id)
                )? == encode(context.as_ref())?,
            "immutable original selected context changed"
        );
        for principal in [
            &context.accounts.native.principal,
            &context.accounts.requested.principal,
        ] {
            ensure!(
                matches!(principal, PlatformPrincipalV1::Unix { uid, .. } if *uid == self.owner_uid),
                "original role principal differs from configured authority"
            );
            crate::execution::install_bootstrap::unix_account_home_for_principal(principal)?;
        }
        self.open_native_history()?;
        // No live config/inventory reads here. Registration does not rewrite the
        // original World binding into the actual current World binding.
        TrustedWorkspaceRoot::open_exact(&context.workspace_binding.workspace_root)?
            .revalidate()?;
        let end = self
            .authority
            .resolve_current_exact(&context.orchestration_session_id, None)?;
        ensure!(
            end.observation.authority_record_commitment
                == current.observation.authority_record_commitment
                && end.caller == current.caller
                && end.authority.origin == current.authority.origin,
            "ordinary observations changed across executing verification"
        );
        held.revalidate()?;
        self.root.revalidate()?;
        if let Some(configured) = &self.configured {
            configured.revalidate()?;
        }
        Ok(())
    }
    pub(crate) fn require_ordinary_configured_authority(
        &self,
    ) -> Result<Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>> {
        self.revalidate()?;
        match self.configured.as_ref() {
            Some(OrdinaryConfiguredHomeCustodyV1::Ordinary(held)) => Ok(Arc::clone(held)),
            Some(OrdinaryConfiguredHomeCustodyV1::Service(_)) | None => anyhow::bail!(
                "ordinary producer requires its genuine ordinary configured authority"
            ),
        }
    }

    pub(crate) fn open_native_history(&self) -> Result<TrustedDirectory> {
        let (identity, directory) =
            capture_native_history_identity(&self.root, self.original_session_id(), false)?;
        ensure!(
            identity == self.context.native_home,
            "original native history was substituted"
        );
        Ok(directory)
    }

    /// Current native-role identity extraction and transient gateway custody.
    /// This does not authenticate a credential, signature, or entitlement.
    pub(crate) fn resolve_native_gateway_handoff(
        &self,
        policy: &substrate_broker::Policy,
    ) -> Result<transport_api_types::GatewayIntegratedAuthPayloadV1> {
        self.revalidate()?;
        let (binding, codex) =
            capture_role_account(self.native_selection(), self.native_principal(), policy)?;
        let original = &self.context.accounts.native;
        ensure!(
            binding.principal == original.principal
                && binding.profile_id == original.profile_id
                && binding.profile_revision == original.profile_revision
                && binding.profile_source_sha256 == original.profile_source_sha256
                && binding.account_id == original.account_id,
            "current native account/profile custody conflicts with original Start"
        );
        let payload = match self.native_selection().selection.harness {
            AgentHarnessFamilyV1::Codex => transport_api_types::GatewayIntegratedAuthPayloadV1 {
                backend_id: self.native_selection().backend_id.clone(),
                cli_codex: Some(codex.context("native Codex handoff missing")?),
                api_env: None,
            },
            AgentHarnessFamilyV1::ClaudeCode => {
                crate::builtins::world_gateway::resolve_claude_code_integrated_auth(
                    &self.native_selection().backend_id,
                    policy,
                )?
                .context("native Claude handoff missing")?
            }
        };
        self.revalidate()?;
        Ok(payload)
    }

    fn prepare_native_gateway_request(
        &self,
        runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    ) -> Result<transport_api_types::GatewayLifecycleRequestV1> {
        use super::dispatch_contract::*;
        self.revalidate()?;
        let home = self.held_authority().bootstrap_home();
        let workspace = Path::new(self.workspace_root());
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            workspace, &home,
        )?;
        let snapshot =
            crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
                workspace, &home,
            )?;
        ensure!(
            snapshot.snapshot_hash == self.current_policy_snapshot_sha256(),
            "current native held policy no longer joins authority"
        );
        let selected = self.native_selection();
        let envelope = DispatchRequestEnvelope {
            caller_kind: DispatchCallerKind::HumanStart,
            baseline_kind: DispatchBaselineKind::InventoryLaunch,
            backend_id: Some(selected.backend_id.clone()),
            orchestration_session_id: Some(self.original_session_id().into()),
            requested_execution_scope_override: Some(AgentExecutionScope::Host),
            capability_overrides: DispatchCapabilityOverrideSet::default(),
            attach_launch_knobs: AttachLaunchKnobs {
                requested_execution_scope: AgentExecutionScope::Host,
                host_execution_client_start: HostExecutionClientStart::StartNow,
                attach_mode_preference: AttachModePreference::ContinuityRequired,
            },
            has_prompt_payload: true,
        };
        let contract = resolve_inventory_contract_for_exact_backend(
            workspace,
            &self.captured_effective_config(),
            self.captured_inventory(),
            &policy,
            &envelope,
            AgentExecutionScope::Host,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?
        .context("current policy refuses original native runtime")?;
        let descriptor = super::validator::materialize_runtime_descriptor(&contract)
            .map_err(|error| anyhow::anyhow!("{}", error.reason))?;
        ensure!(
            descriptor.agent_id == self.context.descriptor.agent_id
                && descriptor.backend_id == self.context.descriptor.backend_id
                && descriptor.protocol == self.context.descriptor.protocol
                && descriptor.binary_path == Path::new(&self.context.descriptor.binary_path)
                && contract.capabilities.session_start
                && contract.capabilities.session_resume
                && contract.capabilities.llm
                && contract.capabilities.event_stream,
            "current native runtime/capability permission differs from original Start"
        );
        let effective = &contract.effective_policy;
        ensure!(
            !effective.llm_require_approval,
            "ordinary native gateway requires an unavailable approval admission"
        );
        ensure!(
            effective
                .llm_allowed_backends
                .iter()
                .any(|backend| backend == &selected.backend_id),
            "current LLM policy refuses original native backend"
        );
        let (protocol, auth) = match selected.selection.harness {
            AgentHarnessFamilyV1::Codex => ("openai.responses", "codex_subscription"),
            AgentHarnessFamilyV1::ClaudeCode => ("anthropic.messages", "anthropic_api_key"),
        };
        for (axis, value, allowed) in [
            (
                "router",
                "substrate_gateway",
                &effective.llm_constraints_routers,
            ),
            (
                "provider",
                selected.selection.provider.as_str(),
                &effective.llm_constraints_providers,
            ),
            ("protocol", protocol, &effective.llm_constraints_protocols),
            (
                "auth authority",
                auth,
                &effective.llm_constraints_auth_authorities,
            ),
        ] {
            ensure!(
                allowed.is_empty() || allowed.iter().any(|candidate| candidate == value),
                "current native gateway {axis} permission is revoked"
            );
        }
        // Validate fresh native-account custody; never put the returned secret
        // into this request, a carrier, a native environment, or native config.
        drop(self.resolve_native_gateway_handoff(effective)?);
        let end = crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
            workspace, &home,
        )?;
        ensure!(
            end.snapshot_hash == snapshot.snapshot_hash,
            "native policy changed during preparation"
        );
        self.revalidate()?;
        let request = transport_api_types::GatewayLifecycleRequestV1 {
            schema_version: 2,
            ordinary_selection: Some(transport_api_types::OrdinaryGatewaySelectionContextV1 {
                schema_version: 1,
                start:
                    crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                        self,
                    )?,
                runtime: runtime.clone(),
            }),
            profile: None,
            cwd: Some(self.workspace_root().into()),
            env: None,
            agent_id: selected.realized_agent_id.clone(),
            policy_snapshot: snapshot.snapshot,
            world_network: None,
            integrated_auth: None,
            identity_tuple: None,
            placement_posture: None,
        };
        request
            .validate_identity_contract()
            .map_err(anyhow::Error::msg)?;
        Ok(request)
    }

    pub fn native_selection(&self) -> &ResolvedAgentSelectionV1 {
        &self.context.selections.native
    }
    pub fn requested_selection(&self) -> &ResolvedAgentSelectionV1 {
        &self.context.selections.requested
    }
    pub fn authority_store_id(&self) -> &str {
        &self.context.workspace_binding.authority_store_id
    }
    pub fn original_session_id(&self) -> &str {
        &self.context.orchestration_session_id
    }
    pub fn orchestration_session_id(&self) -> &str {
        self.original_session_id()
    }
    pub fn original_start_intent_id(&self) -> &str {
        &self.context.intent_id
    }
    pub fn original_start_issuer_request_id(&self) -> Result<String> {
        self.context.issuer_request_id()
    }
    pub fn context_sha256(&self) -> Result<String> {
        self.context.context_sha256()
    }
    pub fn original_world_binding(&self) -> Option<(&str, u64)> {
        self.context
            .original_world_binding
            .as_ref()
            .map(|w| (w.world_id.as_str(), w.world_generation))
    }
    pub fn current_world_binding(&self) -> Option<(&str, u64)> {
        self.current
            .authority
            .world_binding
            .as_ref()
            .map(|w| (w.world_id.as_str(), w.world_generation))
    }
    pub fn current_participant_id(&self) -> &str {
        &self.current.caller.participant_id
    }
    pub fn current_backend_id(&self) -> &str {
        &self.current.caller.descriptor.backend_id
    }
    /// Stored current HSA policy identity only. Filesystem policy revocation,
    /// account permission and effect admission require independent held-root
    /// policy evaluation/revalidation even if this digest remains unchanged.
    pub fn current_policy_snapshot_sha256(&self) -> &str {
        &self.current.current_policy.canonical_policy_snapshot_sha256
    }
    pub fn current_authority_revision(&self) -> u64 {
        self.current.observation.authority_revision
    }
    pub fn current_root_revision(&self) -> u64 {
        self.current.observation.root_revision
    }
    pub fn workspace_root(&self) -> &str {
        &self.context.workspace_binding.workspace_root.physical_path
    }
    pub fn native_codex_account_id(&self) -> Option<&str> {
        self.context.accounts.native.account_id.as_deref()
    }
    pub fn requested_codex_account_id(&self) -> Option<&str> {
        self.context.accounts.requested.account_id.as_deref()
    }
    pub fn native_principal(&self) -> &PlatformPrincipalV1 {
        &self.context.accounts.native.principal
    }
    pub fn requested_principal(&self) -> &PlatformPrincipalV1 {
        &self.context.accounts.requested.principal
    }
    /// Recheck existing policy-gated supported custody and extracted identity.
    /// This is neither credential/JWT authentication nor entitlement proof.
    pub(crate) fn revalidate_requested_account(
        &self,
        current_policy: &substrate_broker::Policy,
    ) -> Result<()> {
        self.resolve_requested_gateway_handoff(current_policy)
            .map(drop)
    }
    /// Existing supported Codex/Claude handoff, held transiently for gateway secret-FD
    /// custody only. Never serialize/log/Debug this return into native state,
    /// selection artifacts or a carrier. No identity/entitlement permission is
    /// inferred from the payload; the caller owns current policy/effect guards.
    pub(crate) fn resolve_requested_gateway_handoff(
        &self,
        current_policy: &substrate_broker::Policy,
    ) -> Result<transport_api_types::GatewayIntegratedAuthPayloadV1> {
        self.revalidate()?;
        let (current, handoff) = capture_role_account(
            self.requested_selection(),
            self.requested_principal(),
            current_policy,
        )?;
        ensure!(
            current.principal == self.context.accounts.requested.principal
                && current.profile_id == self.context.accounts.requested.profile_id
                && current.profile_revision == self.context.accounts.requested.profile_revision
                && current.profile_source_sha256
                    == self.context.accounts.requested.profile_source_sha256
                && current.account_id == self.context.accounts.requested.account_id,
            "current requested account/profile custody conflicts with original Start"
        );
        let payload = match self.requested_selection().selection.harness {
            AgentHarnessFamilyV1::Codex => transport_api_types::GatewayIntegratedAuthPayloadV1 {
                backend_id: self.requested_selection().backend_id.clone(),
                cli_codex: Some(handoff.context("supported requested Codex handoff missing")?),
                api_env: None,
            },
            AgentHarnessFamilyV1::ClaudeCode => {
                crate::builtins::world_gateway::resolve_claude_code_integrated_auth(
                    &self.requested_selection().backend_id,
                    current_policy,
                )?
                .context("supported selected Claude API-key handoff unavailable")?
            }
        };
        self.revalidate()?;
        Ok(payload)
    }
    /// Existing source-owned preserved Start intent and immutable public refs.
    /// Registration may advance authority revision; this authenticates original
    /// birth only and neither synthesizes history nor verifies a current run.
    pub(crate) fn original_start_material(
        &self,
    ) -> Result<(
        AgentDescriptorV1,
        HostAttachContractV1,
        PolicyObjectHashInputV1,
    )> {
        let root = self.authority.read_preserved_start_root_v2()?;
        let original = root
            .transition_intent_map
            .get(self.original_start_intent_id())
            .context("original Start intent missing")?;
        ensure!(
            root.authority_store_id == self.context.workspace_binding.authority_store_id
                && root.bootstrap_home == self.context.workspace_binding.authority_store_root
                && original.mode == HostSessionTransitionModeV1::Start
                && matches!(
                    &original.state,
                    HostSessionTransitionIntentStateV2::Applied { .. }
                )
                && original.issuer_request_id == self.context.issuer_request_id()?
                && original.orchestration_session_id == self.context.orchestration_session_id
                && original.workspace_binding == self.context.workspace_binding
                && original.world_binding == self.context.original_world_binding,
            "original applied Start public material does not join selected birth"
        );
        let attach: HostAttachContractHashInputV1 =
            canonical_json::from_slice(&self.authority.read_authority_object_v2_at(
                root.root_revision,
                &original.host_attach_contract_ref,
            )?)?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(
            &self
                .authority
                .read_authority_object_v2_at(root.root_revision, &original.descriptor_ref)?,
        )?;
        let policy: PolicyObjectHashInputV1 = canonical_json::from_slice(
            &self
                .authority
                .read_authority_object_v2_at(root.root_revision, &attach.contract.policy_ref)?,
        )?;
        ensure!(
            attach.schema_version == 1
                && attach.contract.schema_version == 1
                && descriptor.schema_version == 1
                && descriptor.descriptor == self.context.descriptor
                && attach.contract.descriptor_ref == original.descriptor_ref
                && attach.contract.backend_id == descriptor.descriptor.backend_id
                && attach.contract.protocol == descriptor.descriptor.protocol
                && attach.contract.execution_scope == descriptor.descriptor.execution_scope
                && policy.schema_version == 1,
            "original typed Start material conflicts"
        );
        Ok((descriptor.descriptor, attach.contract, policy))
    }
    pub(crate) fn captured_effective_config(&self) -> SubstrateConfig {
        self.context.sources.effective_config()
    }
    pub(crate) fn captured_inventory(&self) -> &BTreeMap<String, AgentInventoryEntryV1> {
        &self.context.sources.inventory
    }
    pub(crate) fn held_authority(&self) -> &Arc<HostSessionAuthority> {
        &self.authority
    }
    #[allow(
        dead_code,
        reason = "authenticated native launch context accessor is retained for staged ordinary consumers"
    )]
    pub(crate) fn context(&self) -> Arc<NativeLaunchContextV1> {
        Arc::clone(&self.context)
    }
}

#[allow(
    dead_code,
    reason = "held-authority ordinary selection recovery is exercised by tests before production adoption"
)]
pub(crate) fn authenticate_ordinary_selection_from_authority(
    authority: Arc<HostSessionAuthority>,
    original_session_id: &str,
) -> Result<AuthenticatedOrdinarySelectionV1> {
    let current = authority.resolve_current_exact(original_session_id, None)?;
    let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.authority.origin;
    // The genuine held home supplies identity; the immutable record supplies the
    // originally bound principal. Root owner validation rejects any substitution.
    let held = authority.bootstrap_home();
    let identity = held.identity()?.clone();
    let metadata = std::fs::metadata(&identity.physical_path)?;
    let super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
        device_id,
        inode,
    } = &identity.physical_identity
    else {
        anyhow::bail!("ordinary held authority is not Linux");
    };
    ensure!(
        metadata.dev() == *device_id && metadata.ino() == *inode,
        "genuine held ordinary root name changed"
    );
    // Owner validation here is supplemental to the genuine held HSA capability,
    // exact physical identity and its revalidation; a bare path/UID never enters
    // this entrypoint. Do not substitute the ambient process UID (service may be root).
    let root =
        TrustedAuthorityRoot::open_for_owner(Path::new(&identity.physical_path), metadata.uid())?;
    ensure!(
        root.identity() == &identity,
        "genuine held ordinary home changed"
    );
    held.revalidate()?;
    let context: NativeLaunchContextV1 = decode(&read_context(
        &context_directory(&root, false)?,
        &format!("{intent_id}.json"),
    )?)?;
    context.validate()?;
    let verifier = OrdinarySelectionAuthorityV1::from_held_authority(
        authority,
        &context.accounts.native.principal,
    )?;
    verifier.authenticate(original_session_id)
}

#[cfg(test)]
pub(crate) fn with_selected_helper_component_fixture_for_test(
    world: bool,
    interleaved_r0: bool,
    producer: impl Fn(
        &super::control::HiddenOwnerHelperLaunchPlan,
        &OrdinaryStartSelectionsV1,
        &str,
        &str,
        crate::execution::config_model::AgentExecutionScope,
    ) -> Result<String>,
) -> Result<()> {
    tests::with_selected_helper_component_fixture_for_test(world, interleaved_r0, producer)
}

#[cfg(test)]
mod tests {
    use super::super::host_session_authority::{schema::*, transition::*};
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    struct Fixture {
        _env: crate::execution::AuthorityEnvTestGuard,
        _temp: tempfile::TempDir,
        authority: Arc<HostSessionAuthority>,
        context: NativeLaunchContextV1,
        home: PathBuf,
        claude: ResolvedAgentSelectionV1,
    }

    fn fixture(apply: bool) -> Fixture {
        let env = crate::execution::AuthorityEnvTestGuard::preserve();
        // Both canonical keys are synthetic and supplied before the factory;
        // this fixture never falls through to an OS-account auth.json read.
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "synthetic-ordinary-token",
        );
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("accepted");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir(&home).unwrap();
        std::fs::create_dir(&workspace).unwrap();
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::create_dir(home.join("agents")).unwrap();
        std::fs::set_permissions(home.join("agents"), std::fs::Permissions::from_mode(0o700))
            .unwrap();
        let config = "agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex-host\n";
        let mut inventory: serde_yaml::Value =
            serde_yaml::from_str(include_str!("../../../../../config/agents/codex.yaml")).unwrap();
        inventory["config"]["placements"]["host"]["cli"]["binary"] = "/bin/true".into();
        inventory["config"]["placements"]["world"]["selection"]["model"] =
            "synthetic-worker-model".into();
        for (path, bytes) in [
            (home.join("config.yaml"), config.as_bytes().to_vec()),
            (
                home.join("agents/codex.yaml"),
                serde_yaml::to_string(&inventory).unwrap().into_bytes(),
            ),
        ] {
            std::fs::write(&path, bytes).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let claude_path = home.join("agents/claude_code.yaml");
        std::fs::write(
            &claude_path,
            include_str!("../../../../../config/agents/claude_code.yaml"),
        )
        .unwrap();
        std::fs::set_permissions(claude_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let root = TrustedAuthorityRoot::open(&home).unwrap();
        let authority = Arc::new(HostSessionAuthority::from_trusted_root(root).unwrap());
        let root = authority.bootstrap().unwrap();
        let bootstrap = authority.bootstrap_home();
        let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
            &workspace,
            &Default::default(),
            &bootstrap,
        )
        .unwrap();
        let policy = substrate_broker::Policy {
            llm_secrets_env_allowed: vec![
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.into(),
            ],
            ..substrate_broker::Policy::default()
        };
        let inventory =
            crate::execution::agent_inventory::load_agent_inventory_snapshot_for_bootstrap_home_v3(
                &workspace, &policy, &bootstrap,
            )
            .unwrap();
        let claude = inventory
            .resolve_selection(
                &config,
                Some("claude_code"),
                None,
                AgentExecutionScope::Host,
            )
            .unwrap();
        let native = inventory
            .resolve_selection(&config, None, None, AgentExecutionScope::Host)
            .unwrap();
        let requested = inventory
            .resolve_selection(&config, Some("codex"), None, AgentExecutionScope::World)
            .unwrap();
        let selections = OrdinaryStartSelectionsV1 { native, requested };
        let sources = CapturedOrdinarySourcesV1::capture(&config, &inventory, &selections).unwrap();
        let metadata = std::fs::metadata(&workspace).unwrap();
        let binding = WorkspaceBindingV1 {
            workspace_root: CanonicalDirectoryV1 {
                physical_path: workspace.to_str().unwrap().into(),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: metadata.dev(),
                    inode: metadata.ino(),
                },
            },
            authority_store_root: root.bootstrap_home,
            authority_store_id: root.authority_store_id,
        };
        let descriptor = AgentDescriptorV1 {
            schema_version: 1,
            agent_id: selections.native.realized_agent_id.clone(),
            backend_id: selections.native.backend_id.clone(),
            backend_kind: RuntimeBackendKindV1::Codex,
            protocol: "substrate.agent.session".into(),
            execution_scope: AgentExecutionScopeV1::Host,
            binary_path: "/bin/true".into(),
        };
        let world = WorldBindingV1 {
            world_id: "world_selected_fixture".into(),
            world_generation: 1,
        };
        let principal = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .unwrap()
            .0;
        let context = stage_native_start_context(NativeStartContextInputsV1 {
            authority: &authority,
            intended_principal: &principal,
            effective_policy: &policy,
            workspace_binding: &binding,
            world_binding: Some(&world),
            intent_id: "hti_selected_fixture",
            orchestration_session_id: "aos_selected_fixture",
            shell_trace_session_id: "trace_selected_fixture",
            participant_id: "ash_selected_fixture",
            run_id: "run_selected_fixture",
            descriptor: &descriptor,
            opaque_launch_plan_digest: &"a".repeat(64),
            selections: &selections,
            sources: &sources,
        })
        .unwrap();
        if apply {
            let request = IssueHostSessionTransitionRequestV1 {
                intent_id: context.intent_id.clone(),
                issuer_request_id: context.issuer_request_id().unwrap(),
                mode: HostSessionTransitionModeV1::Start,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
                orchestration_session_id: context.orchestration_session_id.clone(),
                shell_trace_session_id: context.shell_trace_session_id.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::Repl,
                    caller_participant_id: None,
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                source_authoritative_participant_id: None,
                target_authoritative_participant_id: context.original_participant_id.clone(),
                target_participant_lease_token: b"synthetic-selected-lease".to_vec(),
                run_id: context.original_run_id.clone(),
                resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
                workspace_binding: binding,
                world_binding: Some(world),
                start_contract: StartContractMaterialV1 {
                    descriptor,
                    policy: PolicyObjectHashInputV1 {
                        schema_version: 1,
                        policy_revision: "p1".into(),
                        canonical_policy_snapshot_sha256: "b".repeat(64),
                    },
                    capabilities: HostAttachCapabilitiesV1 {
                        session_resume: true,
                        session_fork: true,
                        session_stop: true,
                        status_snapshot: true,
                        event_stream: true,
                    },
                    launch_knobs: HostAttachLaunchKnobsV1 {
                        requested_execution_scope: AgentExecutionScopeV1::Host,
                        host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                        attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
                    },
                },
                transition_input: None,
            };
            let issued = authority.issue_start(&request).unwrap();
            let intent = match issued {
                TransitionIssueOutcomeV1::Issued(v) | TransitionIssueOutcomeV1::Joined(v) => v,
            };
            let claimed = authority
                .claim_start(&ClaimHostSessionTransitionRequestV1 {
                    intent_id: intent.intent_id,
                    issuer_request_id: intent.issuer_request_id,
                    payload_commitment: intent.payload_commitment,
                    expected_intent_revision: intent.intent_revision,
                    claim_id: "clm_selected_fixture".into(),
                    claimant_attempt_id: "atm_selected_fixture".into(),
                })
                .unwrap();
            let intent = match claimed {
                TransitionClaimOutcomeV1::Claimed(v)
                | TransitionClaimOutcomeV1::Reclaimed(v)
                | TransitionClaimOutcomeV1::Joined(v) => v,
            };
            let HostSessionTransitionIntentStateV2::Claimed {
                claim_id,
                claim_revision,
                ..
            } = intent.state
            else {
                panic!("genuine claim required");
            };
            authority
                .apply_start(&ApplyHostSessionTransitionRequestV1 {
                    intent_id: intent.intent_id,
                    issuer_request_id: intent.issuer_request_id,
                    payload_commitment: intent.payload_commitment,
                    expected_intent_revision: claim_revision,
                    claim_id,
                    expected_claim_revision: claim_revision,
                })
                .unwrap();
        }
        Fixture {
            _env: env,
            _temp: temp,
            authority,
            context,
            home,
            claude,
        }
    }

    /// Component-only checked source and real captured issuer/application. The
    /// caller supplies the actual public key producer, not an expected digest.
    pub(super) fn with_selected_helper_component_fixture_for_test(
        world: bool,
        interleaved_r0: bool,
        producer: impl Fn(
            &super::super::control::HiddenOwnerHelperLaunchPlan,
            &OrdinaryStartSelectionsV1,
            &str,
            &str,
            AgentExecutionScope,
        ) -> Result<String>,
    ) -> Result<()> {
        use super::super::control::*;
        use super::super::host_session_authority::store_schema::*;
        use super::super::orchestration_session::*;
        use std::os::fd::AsFd;
        let fixture = fixture(false);
        let root = fixture._temp.path().join("selected-component-bootstrap");
        std::fs::create_dir(&root)?;
        std::fs::create_dir(root.join("records"))?;
        for path in [&root, &root.join("records")] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o750))?;
        }
        let PlatformPrincipalV1::Unix { account, uid } = &fixture.context.accounts.native.principal
        else {
            anyhow::bail!("Unix component required");
        };
        let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                fixture.home.to_str().unwrap(),
                account,
                *uid,
            )
            .map_err(anyhow::Error::msg)?,
        )
        .map_err(anyhow::Error::msg)?;
        let mut record = config_projection::InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().map_err(anyhow::Error::msg)?,
            host_context_commitment: carrier.host_context_commitment.clone(),
            intended_account: account.clone(),
            intended_uid: u64::from(*uid),
            intended_gid: u64::from(unsafe { libc::getegid() }),
            accepted_home: config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
                File::open(&fixture.home)?.as_fd(),
            )?,
            installed_at: config_projection::Timestamp("2026-11-01T00:00:00.000000Z".into()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record)?;
        value.as_object_mut().unwrap().remove("record_hash");
        record.record_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &serde_json::json!({"record":value}),
        )?;
        let mut head = config_projection::InstalledAcceptedHomeBootstrapHeadV1 {
            schema_version: 1,
            head_record_hash: record.record_hash.clone(),
            predecessor_head_hash: None,
            updated_at: record.installed_at.clone(),
            head_hash: String::new(),
        };
        let mut value = serde_json::to_value(&head)?;
        value.as_object_mut().unwrap().remove("head_hash");
        head.head_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap-head.v1",
            &serde_json::json!({"head":value}),
        )?;
        for (path, bytes) in [
            (
                root.join("records")
                    .join(format!("{}.json", record.record_hash)),
                config_projection::ConfigProjectionCodecV1::encode_canonical_json(&record)?,
            ),
            (
                root.join("active.json"),
                config_projection::ConfigProjectionCodecV1::encode_canonical_json(&head)?,
            ),
        ] {
            std::fs::write(&path, bytes)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o640))?;
        }
        std::fs::write(root.join("lock"), b"")?;
        std::fs::set_permissions(root.join("lock"), std::fs::Permissions::from_mode(0o600))?;
        let configured = Arc::new(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::
            from_component_source_for_test(&root, &carrier)?);
        let authority = &fixture.authority;
        let original_context = &fixture.context;
        let mut selections = original_context.selections.clone();
        if !world {
            selections.requested = selections.native.clone();
        }
        let world_binding = if world {
            original_context.original_world_binding.clone()
        } else {
            None
        };
        let policy = substrate_broker::Policy {
            llm_secrets_env_allowed: vec![
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.into(),
            ],
            ..substrate_broker::Policy::default()
        };
        let context = stage_native_start_context_from_ordinary_configured(
            Arc::clone(&configured),
            NativeStartContextInputsV1 {
                authority,
                intended_principal: &original_context.accounts.native.principal,
                effective_policy: &policy,
                workspace_binding: &original_context.workspace_binding,
                world_binding: world_binding.as_ref(),
                intent_id: "hti_selected_comparison",
                orchestration_session_id: "aos_selected_comparison",
                shell_trace_session_id: "trace_selected_comparison",
                participant_id: "ash_selected_comparison",
                run_id: "run_selected_comparison",
                descriptor: &original_context.descriptor,
                opaque_launch_plan_digest: &"a".repeat(64),
                selections: &selections,
                sources: &original_context.sources,
            },
        )?;
        let binding = context.workspace_binding.clone();
        let descriptor = context.descriptor.clone();
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: context.intent_id.clone(),
            issuer_request_id: context.issuer_request_id().unwrap(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: context.original_participant_id.clone(),
            target_participant_lease_token: b"synthetic-selected-lease".to_vec(),
            run_id: context.original_run_id.clone(),
            resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
            workspace_binding: binding,
            world_binding: world_binding.clone(),
            start_contract: StartContractMaterialV1 {
                descriptor,
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "p1".into(),
                    canonical_policy_snapshot_sha256: "b".repeat(64),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
                },
            },
            transition_input: None,
        };
        let issued = authority
            .issue_start_at(
                &request,
                TimestampV1::parse("2026-07-14T12:00:00.000000000Z")?,
                300,
            )
            .unwrap();
        let intent = match issued {
            TransitionIssueOutcomeV1::Issued(v) | TransitionIssueOutcomeV1::Joined(v) => v,
        };
        let claimed = authority
            .claim_start_at(
                &ClaimHostSessionTransitionRequestV1 {
                    intent_id: intent.intent_id,
                    issuer_request_id: intent.issuer_request_id,
                    payload_commitment: intent.payload_commitment,
                    expected_intent_revision: intent.intent_revision,
                    claim_id: "clm_selected_fixture".into(),
                    claimant_attempt_id: "atm_selected_fixture".into(),
                },
                TimestampV1::parse("2026-07-14T12:01:00.000000000Z")?,
                30,
            )
            .unwrap();
        let intent = match claimed {
            TransitionClaimOutcomeV1::Claimed(v)
            | TransitionClaimOutcomeV1::Reclaimed(v)
            | TransitionClaimOutcomeV1::Joined(v) => v,
        };
        let HostSessionTransitionIntentStateV2::Claimed {
            claim_id,
            claim_revision,
            ..
        } = intent.state
        else {
            panic!("genuine claim required");
        };
        authority
            .apply_start_at(
                &ApplyHostSessionTransitionRequestV1 {
                    intent_id: intent.intent_id,
                    issuer_request_id: intent.issuer_request_id,
                    payload_commitment: intent.payload_commitment,
                    expected_intent_revision: claim_revision,
                    claim_id,
                    expected_claim_revision: claim_revision,
                },
                TimestampV1::parse("2026-07-14T12:01:10.000000000Z")?,
            )
            .unwrap();

        let original = authority
            .read_preserved_start_root_v2()?
            .transition_intent_map
            .get(&context.intent_id)
            .unwrap()
            .clone();
        let HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            authority_revision_after,
            applied_at,
            ..
        } = &original.state
        else {
            anyhow::bail!("Applied component required")
        };
        std::env::set_var("SUBSTRATE_HOME", &fixture.home);
        let listener = register_durable_start_startup_prompt_listener(
            &context.orchestration_session_id,
            &context.original_participant_id,
        )?;
        let descriptor = ResolvedRuntimeDescriptor {
            agent_id: context.descriptor.agent_id.clone(),
            backend_id: context.descriptor.backend_id.clone(),
            backend_kind: ResolvedRuntimeBackendKind::Codex,
            protocol: context.descriptor.protocol.clone(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: context.descriptor.binary_path.clone(),
        };
        let scope = if world {
            AgentExecutionScope::World
        } else {
            AgentExecutionScope::Host
        };
        let mut helper = HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Start,
            descriptor: descriptor.clone(),
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: context.orchestration_session_id.clone(),
                shell_trace_session_id: context.shell_trace_session_id.clone(),
                workspace_root: context
                    .workspace_binding
                    .workspace_root
                    .physical_path
                    .clone(),
                world_id: world_binding.as_ref().map(|world| world.world_id.clone()),
                world_generation: world_binding.as_ref().map(|world| world.world_generation),
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: context.original_participant_id.clone(),
                lease_token: "synthetic-selected-lease".into(),
                run_id: context.original_run_id.clone(),
                resumed_from_participant_id: None,
                internal_uaa_session_id: None,
            },
            host_attach_contract: Some(HostAttachContract {
                backend_id: descriptor.backend_id.clone(),
                execution_scope: AgentExecutionScope::Host,
                protocol: descriptor.protocol.clone(),
                launch_descriptor: descriptor,
                capabilities: HostAttachCapabilities {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                attach_launch_knobs: HostAttachLaunchKnobs {
                    requested_execution_scope: AgentExecutionScope::Host,
                    host_execution_client_start: HostAttachExecutionClientStart::StartNow,
                    attach_mode_preference: HostAttachModePreference::ContinuityRequired,
                },
                effective_policy: serde_json::json!({"component":"selected-original"}),
                continuity_uaa_session_id: None,
            }),
            startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: "selected \u{0001}\n\"\\ prompt λ".into(),
                stream_path: listener.path().to_path_buf(),
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: Some(selections.requested.backend_id.clone()),
                public_scope: Some(scope),
            }),
            source_orchestration_session_id: None,
        };
        let digest = |domain: &str, bytes: &[u8]| {
            let mut hash = Sha256::new();
            hash.update(domain.as_bytes());
            hash.update((bytes.len() as u64).to_be_bytes());
            hash.update(bytes);
            format!("{:x}", hash.finalize())
        };
        let prompt_sha = digest(
            "substrate.a1.3-p1.public-start-prompt.v1",
            helper
                .startup_prompt
                .as_ref()
                .unwrap()
                .prompt_text
                .as_bytes(),
        );
        let key = producer(
            &helper,
            &selections,
            &prompt_sha,
            &selections.requested.backend_id,
            scope,
        )?;
        helper.startup_prompt.as_mut().unwrap().request_key_sha256 = Some(key.clone());
        helper.startup_prompt.as_mut().unwrap().prompt_sha256 = Some(prompt_sha.clone());
        let mut hash = Sha256::new();
        hash.update(b"substrate.a1.3-p1.start-transaction-id.v1");
        hash.update([0]);
        hash.update(key.as_bytes());
        hash.update([0]);
        hash.update(context.orchestration_session_id.as_bytes());
        let transaction = StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: format!("stx_{:x}", hash.finalize()),
            request_key_sha256: key,
            prompt_sha256: prompt_sha,
            authority_store_id: context.workspace_binding.authority_store_id.clone(),
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            authoritative_participant_id: context.original_participant_id.clone(),
            backend_id: context.descriptor.backend_id.clone(),
            protocol: context.descriptor.protocol.clone(),
            workspace_root: context
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: helper.session.world_id.clone(),
            world_generation: helper.session.world_generation,
            public_backend_id: selections.requested.backend_id.clone(),
            public_scope: if world { "world" } else { "host" }.into(),
            start_intent_id: context.intent_id.clone(),
            start_issuer_request_id: context.issuer_request_id()?,
            start_payload_commitment: original.payload_commitment.clone(),
            start_application_result_ref: application_result_ref.clone(),
            start_run_id: context.original_run_id.clone(),
            start_authority_revision: *authority_revision_after,
            created_at: applied_at.clone(),
            updated_at: applied_at.clone(),
            state: StartTransactionStateV1::PromptNotSubmitted,
        };
        authority.begin_start_transaction(&transaction)?;
        let selected =
            OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(configured)?
                .authenticate(&context.orchestration_session_id)?;
        let selected = Arc::new(selected);
        let observed = authority.observe_current_host_gateway_run_v1(
            &context.orchestration_session_id,
            &context.original_participant_id,
            &context.descriptor.backend_id,
            &context.original_run_id,
        )?;
        let proof = observed.current_run();
        proof.verify_selected_helper_launch_v1(&helper, None, &selected)?;
        assert!(
            proof.verify_helper_launch_v1(&helper, None).is_err(),
            "no selected-to-legacy fallback"
        );
        let path = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", context.intent_id));
        let original_bytes = std::fs::read(&path)?;
        // Compare refusal leaves the entire authority home byte-exact.
        type SelectedHelperHomeInventoryV1 = Vec<(PathBuf, Vec<u8>, u64, u64)>;
        let inventory = || -> Result<SelectedHelperHomeInventoryV1> {
            fn walk(path: &Path, out: &mut SelectedHelperHomeInventoryV1) -> Result<()> {
                let mut entries = std::fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
                entries.sort_by_key(|entry| entry.path());
                for entry in entries {
                    let path = entry.path();
                    let metadata = std::fs::symlink_metadata(&path)?;
                    if metadata.is_dir() {
                        walk(&path, out)?;
                    } else if metadata.is_file() {
                        out.push((
                            path.clone(),
                            std::fs::read(path)?,
                            metadata.dev(),
                            metadata.ino(),
                        ));
                    }
                }
                Ok(())
            }
            let mut out = Vec::new();
            walk(&fixture.home, &mut out)?;
            Ok(out)
        };
        let baseline = inventory()?;
        use super::super::host_session_authority::world_bound_controller::{
            AuthenticatedWorldBoundControllerRunV1, WorldBoundControllerEntryV1,
        };
        let controller = AuthenticatedWorldBoundControllerRunV1::authenticate_received_helper(
            Arc::clone(&selected),
            &helper,
            None,
        );
        let controller = if world {
            let controller = controller?;
            controller.verify_workspace()?;
            assert_eq!(controller.entry(), WorldBoundControllerEntryV1::Start);
            assert_eq!(controller.intent_id(), context.intent_id);
            assert_eq!(controller.run_id(), context.original_run_id);
            assert_eq!(controller.world_binding(), world_binding.as_ref().unwrap());
            assert!(Arc::ptr_eq(controller.selection(), &selected));
            assert_eq!(
                controller.origin().caller.participant_id,
                context.original_participant_id
            );
            for change in ["run", "generation", "native_role"] {
                let mut substituted = helper.clone();
                match change {
                    "run" => substituted.participant.run_id.push_str("-substituted"),
                    "generation" => {
                        substituted.session.world_generation =
                            Some(helper.session.world_generation.unwrap() + 1)
                    }
                    "native_role" => {
                        substituted.descriptor.execution_scope = AgentExecutionScope::World
                    }
                    _ => unreachable!(),
                }
                assert!(
                    AuthenticatedWorldBoundControllerRunV1::authenticate_received_helper(
                        Arc::clone(&selected),
                        &substituted,
                        None,
                    )
                    .is_err(),
                    "issued World controller substitution accepted: {change}"
                );
            }
            Some(controller)
        } else {
            assert!(
                controller.is_err(),
                "ordinary Host plus None cannot become a World controller"
            );
            None
        };
        assert_eq!(
            inventory()?,
            baseline,
            "controller classification must not write authority/source"
        );
        for (pointer, value) in [
            ("/participant/lease_token", serde_json::json!("wrong-lease")),
            ("/participant/run_id", serde_json::json!("wrong-run")),
            ("/session/workspace_root", serde_json::json!("/wrong")),
            (
                "/session/shell_trace_session_id",
                serde_json::json!("wrong-trace"),
            ),
            ("/descriptor/backend_id", serde_json::json!("cli:other")),
            (
                "/startup_prompt/prompt_text",
                serde_json::json!("wrong-prompt"),
            ),
            (
                "/startup_prompt/stream_path",
                serde_json::json!("/wrong/listener"),
            ),
            (
                "/host_attach_contract/effective_policy",
                serde_json::json!({"other":true}),
            ),
            ("/mode", serde_json::json!("attach")),
        ] {
            let mut changed = serde_json::to_value(&helper)?;
            *changed.pointer_mut(pointer).unwrap() = value;
            let changed: HiddenOwnerHelperLaunchPlan = serde_json::from_value(changed)?;
            assert!(
                proof
                    .verify_selected_helper_launch_v1(&changed, None, &selected)
                    .is_err(),
                "{pointer}"
            );
            assert_eq!(baseline, inventory()?);
        }

        // A different genuinely captured, closed context is not this issued selection.
        std::fs::write(&path, encode(original_context)?)?;
        assert!(proof
            .verify_selected_helper_launch_v1(&helper, None, &selected)
            .is_err());
        std::fs::write(&path, &original_bytes)?;
        proof.verify_selected_helper_launch_v1(&helper, None, &selected)?;

        // A stable FIFO substituted after genuine acquisition must refuse
        // without opening a blocking description or entering the callback.
        let fifo_before = inventory()?;
        let backup = path.with_extension("fifo-retained");
        std::fs::rename(&path, &backup)?;
        let fifo_name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())?;
        // SAFETY: owned fixture path, newly absent entry, private permissions.
        ensure!(
            unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) } == 0,
            "fixture FIFO creation failed"
        );
        let entered = std::cell::Cell::new(false);
        let refused = selected.with_original_start_context_for_helper_comparison_v1(|_| {
            entered.set(true);
            Ok(())
        });
        assert!(refused.is_err() && !entered.get());
        assert!(proof
            .verify_selected_helper_launch_v1(&helper, None, &selected)
            .is_err());
        std::fs::remove_file(&path)?;
        std::fs::rename(&backup, &path)?;
        assert_eq!(fifo_before, inventory()?);
        proof.verify_selected_helper_launch_v1(&helper, None, &selected)?;

        // Same bytes in a different named inode must fail on success AND refusal.
        for refusal in [false, true] {
            let backup = path.with_extension("held");
            let error = selected
                .with_original_start_context_for_helper_comparison_v1(|_| {
                    std::fs::rename(&path, &backup)?;
                    std::fs::write(&path, &original_bytes)?;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
                    if refusal {
                        anyhow::bail!("deliberate semantic refusal");
                    }
                    Ok(())
                })
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "selected helper source integrity refused"
            );
            std::fs::remove_file(&path)?;
            std::fs::rename(&backup, &path)?;
        }

        for refusal in [false, true] {
            for substitution in ["bytes", "directory", "workspace", "root"] {
                let target = match substitution {
                    "bytes" => path.clone(),
                    "directory" => path.parent().unwrap().to_path_buf(),
                    "workspace" => {
                        PathBuf::from(&context.workspace_binding.workspace_root.physical_path)
                    }
                    "root" => fixture.home.clone(),
                    _ => unreachable!(),
                };
                let backup = target.with_extension("retained-comparison");
                let error = selected
                    .with_original_start_context_for_helper_comparison_v1(|_| {
                        if substitution == "bytes" {
                            std::fs::write(&target, b"tampered-context")?;
                        } else {
                            std::fs::rename(&target, &backup)?;
                            std::fs::create_dir(&target)?;
                            std::fs::set_permissions(
                                &target,
                                std::fs::Permissions::from_mode(0o700),
                            )?;
                        }
                        if refusal {
                            anyhow::bail!("deliberate semantic refusal");
                        }
                        Ok(())
                    })
                    .unwrap_err();
                assert_eq!(
                    error.to_string(),
                    "selected helper source integrity refused",
                    "{substitution}"
                );
                if substitution == "bytes" {
                    std::fs::write(&target, &original_bytes)?;
                }
                if substitution != "bytes" {
                    std::fs::remove_dir(&target)?;
                    std::fs::rename(&backup, &target)?;
                }
            }
        }
        let before_hsa_checks = inventory()?;
        super::super::host_session_authority::current_host_run::verify_selected_helper_snapshot_finalizers_for_test(
            proof,&helper,&selected)?;
        assert_eq!(before_hsa_checks, inventory()?);

        let registry = if let Some(controller) = controller.as_ref() {
            use super::super::state_store::{
                WorldControllerSessionEnrollmentV1, WorldWorkReceiptRegistry,
            };
            let before_hsa = std::fs::read(fixture.home.join("authority-v1/state-root-v1.json"))?;
            let origin = &controller.origin().observation;
            let registry = WorldWorkReceiptRegistry::bind(
                &fixture.home,
                &origin.bootstrap_home,
                &origin.authority_store_id,
            )?;
            assert_eq!(
                registry.enroll_ordinary_world_controller_session(controller)?,
                WorldControllerSessionEnrollmentV1::Created
            );
            let enrolled = inventory()?;
            assert_eq!(
                registry.enroll_ordinary_world_controller_session(controller)?,
                WorldControllerSessionEnrollmentV1::Joined
            );
            assert_eq!(
                inventory()?,
                enrolled,
                "exact enrollment retry must not rewrite source"
            );
            assert_eq!(
                std::fs::read(fixture.home.join("authority-v1/state-root-v1.json"))?,
                before_hsa
            );
            let held = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(authority).map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let cut = super::super::state_store::authenticate_ordinary_world_work_session_cut_v1(
                held.snapshot(),
                selected.original_session_id(),
                controller.world_binding(),
            )?;
            assert!(
                cut.proposals().is_empty() && cut.acceptances().is_empty(),
                "enrollment may not fabricate accepted work"
            );
            held.verify_sources()
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let sources = held
                .capture_world_controller_completion_sources(selected.original_session_id())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let attention = super::super::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(
                &sources, controller.world_binding(),
            )?;
            assert!(
                !attention.cut().has_unresolved(),
                "genuine enrolled empty work has complete guarded coverage"
            );
            attention.verify_sources()?;
            let codec_inventory = inventory()?;
            super::super::host_session_authority::world_bound_controller::exercise_world_controller_completion_codecs_for_test(controller, attention.cut())?;
            assert_eq!(
                inventory()?,
                codec_inventory,
                "codec fixtures must not publish runtime truth"
            );
            super::super::state_store::exercise_canonical_materialization_root_lock_for_test(
                &registry,
                &fixture.home,
            )?;
            Some(registry)
        } else {
            None
        };
        selected.with_original_start_context_for_helper_comparison_v1(|_| Ok(()))?;
        // Retained full source identities survive mutable current configuration.
        std::fs::write(
            fixture.home.join("config.yaml"),
            "agents:\n  enabled: false\n",
        )?;
        proof.verify_selected_helper_launch_v1(&helper, None, &selected)?;
        super::super::host_session_authority::current_host_run::exercise_selected_helper_resumes_for_test(
            authority,&selected,&helper,interleaved_r0)?;
        if let (Some(controller), Some(registry)) = (controller.as_ref(), registry.as_ref()) {
            let before = inventory()?;
            assert!(
                registry
                    .enroll_ordinary_world_controller_session(controller)
                    .is_err(),
                "a genuine old Start cannot reinitialize coverage after successor history advances"
            );
            assert_eq!(
                inventory()?,
                before,
                "stale enrollment must not rewrite source"
            );
        }
        let listener_path = listener.path().to_path_buf();
        let listener_inode = std::fs::symlink_metadata(&listener_path)?;
        drop(listener);
        let remaining = std::fs::symlink_metadata(&listener_path)?;
        ensure!(
            listener_inode.dev() == remaining.dev() && listener_inode.ino() == remaining.ino(),
            "listener changed"
        );
        std::fs::remove_file(listener_path)?;
        Ok(())
    }

    #[test]
    fn test_supplied_codex_role_accounts_are_bound_independently() {
        let fixture = fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        assert_ne!(
            selected.native_selection().selection.model,
            selected.requested_selection().selection.model
        );
        assert_ne!(
            selected.native_selection().placement,
            selected.requested_selection().placement
        );
        assert_eq!(
            selected.native_codex_account_id(),
            Some("synthetic-ordinary-account")
        );
        assert_eq!(
            selected.requested_codex_account_id(),
            Some("synthetic-ordinary-account")
        );
        assert_eq!(selected.native_principal(), selected.requested_principal());
        let claude = capture_role_account(
            &fixture.claude,
            selected.native_principal(),
            &substrate_broker::Policy::default(),
        )
        .unwrap();
        assert!(claude.0.account_id.is_none());
        assert_eq!(fixture.claude.selection.model, "claude-sonnet-4-5-20250929");
        assert_eq!(
            fixture.claude.selection.harness,
            AgentHarnessFamilyV1::ClaudeCode
        );
        let policy = substrate_broker::Policy {
            llm_secrets_env_allowed: vec![
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.into(),
            ],
            ..substrate_broker::Policy::default()
        };
        selected.revalidate_requested_account(&policy).unwrap();
        // A refreshed token for the same pinned account remains supported.
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "synthetic-refreshed-token",
        );
        selected.revalidate_requested_account(&policy).unwrap();
        let mut revoked = policy.clone();
        revoked.llm_secrets_env_allowed.clear();
        assert!(selected.revalidate_requested_account(&revoked).is_err());
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-other-account",
        );
        assert!(selected.revalidate_requested_account(&policy).is_err());
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        selected.revalidate().unwrap();
    }

    #[test]
    fn test_issuer_context_excludes_credentials_and_rejects_duplicate_fields() {
        let fixture = fixture(true);
        let bytes = encode(&fixture.context).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains("synthetic-ordinary-token"));
        assert!(!text.contains("synthetic-selected-lease"));
        assert!(!format!("{:?}", fixture.context).contains("synthetic-ordinary-token"));
        assert!(!text.contains("access_token"));
        assert!(!text.contains("target_participant_lease_token"));
        assert!(decode::<NativeLaunchContextV1>(
            format!("{{\"version\":1,{}", &text[1..]).as_bytes()
        )
        .is_err());
        let mut substituted = fixture.context.clone();
        substituted.accounts.requested.account_id = Some("other-account".into());
        assert_ne!(
            substituted.issuer_request_id().unwrap(),
            fixture.context.issuer_request_id().unwrap()
        );
    }

    #[test]
    fn test_original_start_artifact_requires_genuine_applied_origin() {
        let fixture = fixture(false);
        assert!(authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id
        )
        .is_err());
        let fixture = self::fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        assert_eq!(
            selected.original_start_intent_id(),
            fixture.context.intent_id
        );
        assert_eq!(selected.current_authority_revision(), 1);
        let path = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", fixture.context.intent_id));
        let original = std::fs::read(&path).unwrap();
        let mut modified = fixture.context.clone();
        modified.accounts.native.account_id = Some("substituted-account".into());
        std::fs::write(&path, encode(&modified).unwrap()).unwrap();
        assert!(selected.revalidate().is_err());
        std::fs::write(&path, original).unwrap();
        selected.revalidate().unwrap();
    }

    #[test]
    fn test_captured_selection_recovery_ignores_changed_current_defaults() {
        let fixture = fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        let before = selected.native_selection().clone();
        std::fs::write(
            fixture.home.join("config.yaml"),
            "agents:\n  enabled: false\n",
        )
        .unwrap();
        std::fs::write(
            fixture.home.join("agents/codex.yaml"),
            "malformed current inventory\n",
        )
        .unwrap();
        selected.revalidate().unwrap();
        let recovered = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        assert_eq!(recovered.native_selection(), &before);
        assert!(recovered.captured_effective_config().agents.enabled);
        assert_eq!(
            recovered.native_codex_account_id(),
            Some("synthetic-ordinary-account")
        );
    }

    #[test]
    fn test_original_selection_survives_genuine_retained_registration() {
        use super::super::host_session_authority::facade::RetainedWorkerAuthorityPreconditionV1;
        use super::super::host_session_authority::validation::ValidatedCanonicalV1;
        let fixture = fixture(true);
        let session = &fixture.context.orchestration_session_id;
        // Test-support installed record supplies a genuine configured seal;
        // it does not establish unprivileged installed-record readability.
        use std::os::fd::AsFd;
        let accepted_home = config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
            std::fs::File::open(&fixture.home).unwrap().as_fd(),
        )
        .unwrap();
        let PlatformPrincipalV1::Unix { account, uid } =
            fixture.context.accounts.native.principal.clone()
        else {
            panic!("Unix fixture");
        };
        let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                &accepted_home.physical_path,
                &account,
                uid,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = config_projection::InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().unwrap(),
            host_context_commitment: carrier.host_context_commitment,
            intended_account: account,
            intended_uid: u64::from(uid),
            intended_gid: u64::from(unsafe { libc::getegid() }),
            accepted_home,
            installed_at: config_projection::Timestamp("2026-10-29T00:00:00.000000Z".into()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("record_hash");
        record.record_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &serde_json::json!({ "record": value }),
        )
        .unwrap();
        let configured = Arc::new(
            config_projection::ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(record)
                .unwrap(),
        );
        let selected =
            OrdinarySelectionAuthorityV1::from_configured_accepted_home(Arc::clone(&configured))
                .unwrap()
                .authenticate(session)
                .unwrap();
        assert!(matches!(
            selected.configured.as_ref(),
            Some(OrdinaryConfiguredHomeCustodyV1::Service(held)) if Arc::ptr_eq(held, &configured)
        ));
        let original_native = selected.native_selection().clone();
        let original_requested = selected.requested_selection().clone();
        let original_issuer = selected.original_start_issuer_request_id().unwrap();
        let original_digest = selected.context_sha256().unwrap();
        let original_context_bytes = encode(&fixture.context).unwrap();
        let before = fixture
            .authority
            .resolve_current_exact(session, None)
            .unwrap();
        let participant = "ash_selected_retained_fixture";
        let mut descriptor = fixture.context.descriptor.clone();
        descriptor.agent_id = original_requested.realized_agent_id.clone();
        descriptor.backend_id = original_requested.backend_id.clone();
        descriptor.execution_scope = AgentExecutionScopeV1::World;
        let descriptor_bytes = canonical_json::to_vec(&AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: descriptor.clone(),
        })
        .unwrap();
        let resume_bytes = canonical_json::to_vec(&ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: session.clone(),
            participant_id: participant.into(),
            backend_id: descriptor.backend_id.clone(),
            protocol: descriptor.protocol.clone(),
            internal_uaa_session_id: "uaa_selected_retained_fixture".into(),
        })
        .unwrap();
        let reserved = fixture
            .authority
            .reserve_retained_worker_registration(
                "selected-retained-registration",
                session,
                &RetainedWorkerAuthorityPreconditionV1 {
                    authority_store_id: before.observation.authority_store_id.clone(),
                    authority_revision: before.observation.authority_revision,
                    authority_record_commitment: before
                        .observation
                        .authority_record_commitment
                        .clone(),
                },
                participant,
                descriptor_bytes,
                resume_bytes,
                |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    let worker = RetainedWorkerObjectHashInputV1 {
                        schema_version: 1,
                        orchestration_session_id: session.clone(),
                        participant_id: participant.into(),
                        world_binding: world_binding.clone(),
                        descriptor_ref: descriptor_ref.clone(),
                        resume_handle_ref: resume_handle_ref.clone(),
                        policy_ref: policy_ref.clone(),
                    };
                    worker
                        .validate()
                        .map_err(|_| "invalid synthetic retained worker")?;
                    canonical_json::to_vec(&worker).map_err(|_| "encode synthetic retained worker")
                },
            )
            .unwrap();
        for (reference, bytes) in [
            (
                &reserved.descriptor_ref,
                reserved.descriptor_bytes.as_slice(),
            ),
            (
                &reserved.resume_handle_ref,
                reserved.resume_handle_bytes.as_slice(),
            ),
            (
                &reserved.retained_worker_ref,
                reserved.retained_worker_bytes.as_slice(),
            ),
        ] {
            fixture
                .authority
                .publish_reserved_retained_object(&reserved, reference, bytes)
                .unwrap();
        }
        fixture
            .authority
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        let after = fixture
            .authority
            .resolve_current_exact(session, None)
            .unwrap();
        assert!(after.observation.authority_revision > before.observation.authority_revision);
        assert!(after.observation.root_revision > before.observation.root_revision);
        // Old observation needs refresh; registration is not a Host-run successor.
        assert!(selected.revalidate().is_err());
        let refreshed = selected.refresh_original_selection().unwrap();
        assert!(Arc::ptr_eq(
            refreshed.held_authority(),
            selected.held_authority()
        ));
        assert!(matches!(
            refreshed.configured.as_ref(),
            Some(OrdinaryConfiguredHomeCustodyV1::Service(held)) if Arc::ptr_eq(held, &configured)
        ));
        assert!(Arc::ptr_eq(&refreshed.root, &selected.root));
        assert_eq!(refreshed.owner_uid, selected.owner_uid);
        refreshed.revalidate().unwrap();
        assert_eq!(
            refreshed.current_authority_revision(),
            after.observation.authority_revision
        );
        assert_eq!(
            refreshed.original_start_issuer_request_id().unwrap(),
            original_issuer
        );
        assert_eq!(refreshed.context_sha256().unwrap(), original_digest);
        assert_eq!(refreshed.native_selection(), &original_native);
        assert_eq!(refreshed.requested_selection(), &original_requested);
        assert_eq!(
            refreshed.context().original_run_id,
            fixture.context.original_run_id
        );
        assert_eq!(
            encode(refreshed.context().as_ref()).unwrap(),
            original_context_bytes
        );
        let path = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", fixture.context.intent_id));
        assert_eq!(std::fs::read(path).unwrap(), original_context_bytes);
        std::fs::set_permissions(&fixture.home, std::fs::Permissions::from_mode(0o750)).unwrap();
        assert!(refreshed.refresh_original_selection().is_err());
        std::fs::set_permissions(&fixture.home, std::fs::Permissions::from_mode(0o700)).unwrap();
        refreshed.revalidate().unwrap();
    }

    #[test]
    fn test_native_role_handoff_refresh_and_revocation() {
        let fixture = fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        let mut policy = substrate_broker::Policy {
            llm_secrets_env_allowed: vec![
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.into(),
            ],
            ..substrate_broker::Policy::default()
        };
        let before = encode(&fixture.context).unwrap();
        selected.resolve_native_gateway_handoff(&policy).unwrap();
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "synthetic-native-refresh",
        );
        let refreshed = selected.resolve_native_gateway_handoff(&policy).unwrap();
        assert_eq!(
            refreshed.cli_codex.unwrap().access_token,
            "synthetic-native-refresh"
        );
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-native-other-account",
        );
        assert!(selected.resolve_native_gateway_handoff(&policy).is_err());
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        policy.llm_secrets_env_allowed.clear();
        assert!(selected.resolve_native_gateway_handoff(&policy).is_err());
        assert_eq!(encode(&fixture.context).unwrap(), before);
        assert!(!String::from_utf8(before)
            .unwrap()
            .contains("synthetic-native-refresh"));
    }

    #[test]
    fn test_native_history_is_issuer_bound_and_reopened_exactly() {
        let fixture = fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        let original = fixture.context.native_home.clone();
        let issuer = fixture.context.issuer_request_id().unwrap();
        for marker in ["start", "resume-one", "resume-two"] {
            let directory = selected.open_native_history().unwrap();
            let path = native_directory_descriptor_path(&directory).unwrap();
            let status = std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg("printf '%s\\n' \"$NATIVE_MARKER\" >> \"$NATIVE_HOME/history-proof\"")
                .env("NATIVE_HOME", &path)
                .env("NATIVE_MARKER", marker)
                .status()
                .unwrap();
            assert!(status.success());
            drop(directory);
            let recovered = authenticate_ordinary_selection_from_authority(
                Arc::clone(&fixture.authority),
                &fixture.context.orchestration_session_id,
            )
            .unwrap();
            assert_eq!(recovered.context.native_home, original);
            assert_eq!(
                recovered.original_start_issuer_request_id().unwrap(),
                issuer
            );
        }
        assert_eq!(
            std::fs::read_to_string(Path::new(&original.physical_path).join("history-proof"))
                .unwrap(),
            "start\nresume-one\nresume-two\n"
        );
    }

    #[test]
    fn test_native_history_substitution_refuses_and_preserves_original_context() {
        let fixture = fixture(true);
        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        let path = PathBuf::from(&fixture.context.native_home.physical_path);
        let moved = path.with_extension("held-original");
        let before = encode(&fixture.context).unwrap();
        std::fs::rename(&path, &moved).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(selected.revalidate().is_err());
        assert!(selected.open_native_history().is_err());
        std::fs::remove_dir(&path).unwrap();
        std::fs::rename(&moved, &path).unwrap();
        selected.revalidate().unwrap();
        let original = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", fixture.context.intent_id));
        assert_eq!(std::fs::read(original).unwrap(), before);
    }

    #[test]
    fn test_native_history_missing_field_is_rejected() {
        let fixture = fixture(true);
        let mut value = serde_json::to_value(&fixture.context).unwrap();
        value.as_object_mut().unwrap().remove("native_home");
        let bytes = canonical_json::to_vec(&value).unwrap();
        assert!(decode::<NativeLaunchContextV1>(&bytes).is_err());
    }

    #[test]
    fn test_ordinary_producer_refuses_none_and_service_custody() {
        let fixture = fixture(true);
        let unsealed = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        assert!(unsealed.require_ordinary_configured_authority().is_err());
        use std::os::fd::AsFd;
        let accepted_home = config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
            std::fs::File::open(&fixture.home).unwrap().as_fd(),
        )
        .unwrap();
        let PlatformPrincipalV1::Unix { account, uid } =
            fixture.context.accounts.native.principal.clone()
        else {
            panic!("Unix fixture");
        };
        let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                &accepted_home.physical_path,
                &account,
                uid,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = config_projection::InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().unwrap(),
            host_context_commitment: carrier.host_context_commitment,
            intended_account: account,
            intended_uid: u64::from(uid),
            intended_gid: u64::from(unsafe { libc::getegid() }),
            accepted_home,
            installed_at: config_projection::Timestamp("2026-10-29T00:00:00.000000Z".into()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("record_hash");
        record.record_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &serde_json::json!({ "record": value }),
        )
        .unwrap();
        let configured = Arc::new(
            config_projection::ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(record)
                .unwrap(),
        );

        let service = OrdinarySelectionAuthorityV1::from_configured_accepted_home(configured)
            .unwrap()
            .authenticate(&fixture.context.orchestration_session_id)
            .unwrap();
        assert!(service.require_ordinary_configured_authority().is_err());

        // Service custody may read original sources without becoming an ordinary
        // helper/producer or gaining any execution permission.
        let entered = std::cell::Cell::new(false);
        assert!(unsealed
            .with_ordinary_inspection_sources_v1(|_, _| {
                entered.set(true);
                Ok(())
            })
            .is_err());
        assert!(!entered.get(), "an unsealed reader entered its consumer");
        assert!(service
            .with_original_start_context_for_helper_comparison_v1(|_| {
                entered.set(true);
                Ok(())
            })
            .is_err());
        assert!(!entered.get(), "service custody became helper authority");

        let root_path = fixture.home.join("authority-v1/state-root-v1.json");
        let context_path = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", fixture.context.intent_id));
        let root_before = std::fs::read(&root_path).unwrap();
        let context_before = std::fs::read(&context_path).unwrap();
        service
            .with_ordinary_inspection_sources_v1(|context, facts| {
                entered.set(true);
                assert_eq!(encode(context)?, encode(&fixture.context)?);
                assert_eq!(facts.original_start.intent_id, context.intent_id);
                assert_eq!(
                    facts.current.observation.authority_store_id,
                    context.workspace_binding.authority_store_id
                );
                Ok(())
            })
            .expect("genuine service custody must support guarded source reads");
        assert!(entered.get());
        assert_eq!(std::fs::read(&root_path).unwrap(), root_before);
        assert_eq!(std::fs::read(&context_path).unwrap(), context_before);
        assert!(service.require_ordinary_configured_authority().is_err());

        // Service reads retain the same original physical guard through both
        // successful and refused consumer results.
        for refusal in [false, true] {
            let backup = context_path.with_extension("service-original");
            let result = service.with_original_start_context_guarded_source_v1(|_, verify| {
                verify()?;
                std::fs::rename(&context_path, &backup)?;
                std::fs::write(&context_path, &context_before)?;
                std::fs::set_permissions(&context_path, std::fs::Permissions::from_mode(0o600))?;
                if refusal {
                    anyhow::bail!("service consumer deliberately refused");
                }
                Ok(())
            });
            assert_eq!(
                result.unwrap_err().to_string(),
                "selected helper source integrity refused"
            );
            std::fs::remove_file(&context_path).unwrap();
            std::fs::rename(&backup, &context_path).unwrap();
            service.revalidate().unwrap();
        }
        assert_eq!(std::fs::read(&root_path).unwrap(), root_before);
        assert_eq!(std::fs::read(&context_path).unwrap(), context_before);
    }

    #[test]
    fn test_selected_bridge_rejects_unsealed_selection_without_generic_fallback() {
        let fixture = fixture(true);
        let selected = Arc::new(
            authenticate_ordinary_selection_from_authority(
                Arc::clone(&fixture.authority),
                &fixture.context.orchestration_session_id,
            )
            .unwrap(),
        );
        let descriptor = super::super::validator::RuntimeSelectionDescriptor {
            agent_id: fixture.context.descriptor.agent_id.clone(),
            backend_id: fixture.context.descriptor.backend_id.clone(),
            backend_kind: super::super::mapping::AgentRuntimeBackendKind::Codex,
            protocol: fixture.context.descriptor.protocol.clone(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from(&fixture.context.descriptor.binary_path),
        };
        let runtime = transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
            orchestration_session_id: selected.original_session_id().into(),
            participant_id: selected.current_participant_id().into(),
            backend_id: selected.native_selection().backend_id.clone(),
            run_id: fixture.context.original_run_id.clone(),
        };
        let error = match crate::execution::prompt_fulfillment::PromptFulfillmentBridge::for_ordinary_descriptor(
            &descriptor, selected, runtime,
        ) { Ok(_) => panic!("unsealed ordinary input must refuse"), Err(error) => error };
        assert!(error
            .to_string()
            .contains("genuine ordinary configured authority"));
    }

    #[test]
    fn test_native_gateway_request_pins_original_selection_and_rechecks_real_policy() {
        let fixture = fixture(false);
        let policy_path = fixture.home.join("policy.yaml");
        let policy_bytes = b"id: native-component\nagents:\n  allowed_backends: [cli:codex-host, cli:codex-world]\nllm:\n  allowed_backends: [cli:codex-host]\n  constraints:\n    routers: [substrate_gateway]\n    providers: [openai]\n    protocols: [openai.responses]\n    auth_authorities: [codex_subscription]\n  secrets:\n    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n";
        std::fs::write(&policy_path, policy_bytes).unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let bootstrap_home = fixture.authority.bootstrap_home();
        let workspace = Path::new(
            &fixture
                .context
                .workspace_binding
                .workspace_root
                .physical_path,
        );
        // The real held-root producer supplies both identities to the genuine
        // Start request; no guessed digest, current-run shim, or cap factory.
        let snapshot =
            crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
                workspace,
                &bootstrap_home,
            )
            .unwrap();
        let context = &fixture.context;
        let authority = &fixture.authority;
        let binding = context.workspace_binding.clone();
        let world = context.original_world_binding.clone().unwrap();
        let descriptor = context.descriptor.clone();
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: context.intent_id.clone(),
            issuer_request_id: context.issuer_request_id().unwrap(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: context.original_participant_id.clone(),
            target_participant_lease_token: b"synthetic-selected-lease".to_vec(),
            run_id: context.original_run_id.clone(),
            resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
            workspace_binding: binding,
            world_binding: Some(world),
            start_contract: StartContractMaterialV1 {
                descriptor,
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: snapshot.snapshot_hash.clone(),
                    canonical_policy_snapshot_sha256: snapshot.snapshot_hash.clone(),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
                },
            },
            transition_input: None,
        };
        let issued = authority.issue_start(&request).unwrap();
        let intent = match issued {
            TransitionIssueOutcomeV1::Issued(v) | TransitionIssueOutcomeV1::Joined(v) => v,
        };
        let claimed = authority
            .claim_start(&ClaimHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: intent.intent_revision,
                claim_id: "clm_selected_fixture".into(),
                claimant_attempt_id: "atm_selected_fixture".into(),
            })
            .unwrap();
        let intent = match claimed {
            TransitionClaimOutcomeV1::Claimed(v)
            | TransitionClaimOutcomeV1::Reclaimed(v)
            | TransitionClaimOutcomeV1::Joined(v) => v,
        };
        let HostSessionTransitionIntentStateV2::Claimed {
            claim_id,
            claim_revision,
            ..
        } = intent.state
        else {
            panic!("genuine claim required");
        };
        authority
            .apply_start(&ApplyHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: claim_revision,
                claim_id,
                expected_claim_revision: claim_revision,
            })
            .unwrap();

        let selected = authenticate_ordinary_selection_from_authority(
            Arc::clone(&fixture.authority),
            &fixture.context.orchestration_session_id,
        )
        .unwrap();
        assert_eq!(
            selected.current_policy_snapshot_sha256(),
            snapshot.snapshot_hash
        );
        let runtime = transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
            orchestration_session_id: selected.original_session_id().into(),
            participant_id: selected.current_participant_id().into(),
            backend_id: selected.native_selection().backend_id.clone(),
            run_id: fixture.context.original_run_id.clone(),
        };
        // This directly exercises the private request producer only. No client
        // is constructed and no service, gateway, native or provider effect runs.
        let request = selected.prepare_native_gateway_request(&runtime).unwrap();
        assert_eq!(request.schema_version, 2);
        assert_eq!(
            request.agent_id,
            selected.native_selection().realized_agent_id
        );
        assert_eq!(request.cwd.as_deref(), Some(selected.workspace_root()));
        assert_eq!(
            serde_json::to_value(&request.policy_snapshot).unwrap(),
            serde_json::to_value(&snapshot.snapshot).unwrap()
        );
        assert!(request.env.is_none());
        assert!(request.integrated_auth.is_none());
        let carried = request.ordinary_selection.as_ref().unwrap();
        assert_eq!(carried.schema_version, 1);
        assert_eq!(carried.runtime, runtime);
        assert_eq!(
            carried.start.native_selection_sha256,
            selected.native_selection().selection_digest
        );
        assert_eq!(
            carried.start.requested_selection_sha256,
            selected.requested_selection().selection_digest
        );
        assert_ne!(
            carried.start.native_selection_sha256,
            carried.start.requested_selection_sha256
        );
        let bytes = serde_json::to_vec(&request).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains("synthetic-ordinary-token"));
        assert!(!text.contains("access_token"));
        let decoded: transport_api_types::GatewayLifecycleRequestV1 =
            serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
        let mut unknown = serde_json::to_value(&request).unwrap();
        unknown["invented_permission"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<transport_api_types::GatewayLifecycleRequestV1>(unknown)
                .is_err()
        );

        std::fs::write(
            fixture.home.join("config.yaml"),
            "agents:\n  enabled: false\n",
        )
        .unwrap();
        std::fs::write(
            fixture.home.join("agents/codex.yaml"),
            "malformed current inventory\n",
        )
        .unwrap();
        let pinned = selected.prepare_native_gateway_request(&runtime).unwrap();
        assert_eq!(serde_json::to_vec(&pinned).unwrap(), bytes);
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "synthetic-native-request-refresh",
        );
        assert_eq!(
            serde_json::to_vec(&selected.prepare_native_gateway_request(&runtime).unwrap())
                .unwrap(),
            bytes
        );
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-revoked-native-account",
        );
        assert!(selected.prepare_native_gateway_request(&runtime).is_err());
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        let original_policy = std::str::from_utf8(policy_bytes).unwrap();
        for revoked in [
            original_policy.replace("  allowed_backends: [cli:codex-host]\n", "  allowed_backends: []\n"),
            original_policy.replace("    providers: [openai]\n", "    providers: [anthropic]\n"),
            original_policy.replace("    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n", "    env_allowed: []\n"),
        ] {
            assert_ne!(revoked, original_policy);
            std::fs::write(&policy_path, revoked).unwrap();
            selected.revalidate().unwrap();
            assert!(selected.prepare_native_gateway_request(&runtime).is_err());
            std::fs::write(&policy_path, policy_bytes).unwrap();
            assert_eq!(serde_json::to_vec(&selected.prepare_native_gateway_request(&runtime).unwrap()).unwrap(), bytes);
        }
        // The fixture still carries None configured custody; exercising this
        // private component does not qualify ordinary production construction.
        assert!(selected.require_ordinary_configured_authority().is_err());
    }

    fn host_gateway_component_fixture() -> (Fixture, OrdinarySelectionAuthorityV1) {
        let fixture = fixture(false);
        let policy_path = fixture.home.join("policy.yaml");
        let policy_bytes = b"id: native-component\nagents:\n  allowed_backends: [cli:codex-host, cli:codex-world]\nllm:\n  allowed_backends: [cli:codex-host]\n  constraints:\n    routers: [substrate_gateway]\n    providers: [openai]\n    protocols: [openai.responses]\n    auth_authorities: [codex_subscription]\n  secrets:\n    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n";
        std::fs::write(&policy_path, policy_bytes).unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let bootstrap_home = fixture.authority.bootstrap_home();
        let workspace = Path::new(
            &fixture
                .context
                .workspace_binding
                .workspace_root
                .physical_path,
        );
        // The real held-root producer supplies both identities to the genuine
        // Start request; no guessed digest, current-run shim, or cap factory.
        let snapshot =
            crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
                workspace,
                &bootstrap_home,
            )
            .unwrap();
        let context = &fixture.context;
        let authority = &fixture.authority;
        let binding = context.workspace_binding.clone();
        let world = context.original_world_binding.clone().unwrap();
        let descriptor = context.descriptor.clone();
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: context.intent_id.clone(),
            issuer_request_id: context.issuer_request_id().unwrap(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: context.original_participant_id.clone(),
            target_participant_lease_token: b"synthetic-selected-lease".to_vec(),
            run_id: context.original_run_id.clone(),
            resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
            workspace_binding: binding,
            world_binding: Some(world),
            start_contract: StartContractMaterialV1 {
                descriptor,
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: snapshot.snapshot_hash.clone(),
                    canonical_policy_snapshot_sha256: snapshot.snapshot_hash.clone(),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
                },
            },
            transition_input: None,
        };
        let issued = authority.issue_start(&request).unwrap();
        let intent = match issued {
            TransitionIssueOutcomeV1::Issued(v) | TransitionIssueOutcomeV1::Joined(v) => v,
        };
        let claimed = authority
            .claim_start(&ClaimHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: intent.intent_revision,
                claim_id: "clm_selected_fixture".into(),
                claimant_attempt_id: "atm_selected_fixture".into(),
            })
            .unwrap();
        let intent = match claimed {
            TransitionClaimOutcomeV1::Claimed(v)
            | TransitionClaimOutcomeV1::Reclaimed(v)
            | TransitionClaimOutcomeV1::Joined(v) => v,
        };
        let HostSessionTransitionIntentStateV2::Claimed {
            claim_id,
            claim_revision,
            ..
        } = intent.state
        else {
            panic!("genuine claim required");
        };
        authority
            .apply_start(&ApplyHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: claim_revision,
                claim_id,
                expected_claim_revision: claim_revision,
            })
            .unwrap();

        use std::os::fd::AsFd;
        let accepted_home = config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
            std::fs::File::open(&fixture.home).unwrap().as_fd(),
        )
        .unwrap();
        let PlatformPrincipalV1::Unix { account, uid } =
            fixture.context.accounts.native.principal.clone()
        else {
            panic!("Unix fixture");
        };
        let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
            transport_api_types::InstallBootstrapContextV1::new_unix(
                &accepted_home.physical_path,
                &account,
                uid,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = config_projection::InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().unwrap(),
            host_context_commitment: carrier.host_context_commitment,
            intended_account: account,
            intended_uid: u64::from(uid),
            intended_gid: u64::from(unsafe { libc::getegid() }),
            accepted_home,
            installed_at: config_projection::Timestamp("2026-10-29T00:00:00.000000Z".into()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("record_hash");
        record.record_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &serde_json::json!({ "record": value }),
        )
        .unwrap();
        let configured = Arc::new(
            config_projection::ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(record)
                .unwrap(),
        );
        let verifier =
            OrdinarySelectionAuthorityV1::from_configured_accepted_home(configured).unwrap();
        (fixture, verifier)
    }

    fn host_gateway_claim(
        context: &NativeLaunchContextV1,
    ) -> transport_api_types::OrdinaryGatewaySelectionContextV1 {
        transport_api_types::OrdinaryGatewaySelectionContextV1 {
            schema_version: 1,
            start: transport_api_types::OrdinaryStartSelectionCarrierV1 {
                schema_version: 1,
                authority_store_id: context.workspace_binding.authority_store_id.clone(),
                original_start_intent_id: context.intent_id.clone(),
                original_start_issuer_request_id: context.issuer_request_id().unwrap(),
                original_session_id: context.orchestration_session_id.clone(),
                context_sha256: context.context_sha256().unwrap(),
                native_selection_sha256: context.selections.native.selection_digest.clone(),
                requested_selection_sha256: context.selections.requested.selection_digest.clone(),
            },
            runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
                orchestration_session_id: context.orchestration_session_id.clone(),
                participant_id: context.original_participant_id.clone(),
                backend_id: context.descriptor.backend_id.clone(),
                run_id: context.original_run_id.clone(),
            },
        }
    }

    #[test]
    fn test_host_gateway_admission_full_original_current_policy_account_join() {
        // The existing test-support installed record is a component seal, not
        // evidence of actual installation or an ordinary production birth.
        let (fixture, verifier) = host_gateway_component_fixture();
        let claim = host_gateway_claim(&fixture.context);
        let state_path = fixture.home.join("authority-v1/state-root-v1.json");
        let state_before = std::fs::read(&state_path).unwrap();
        let admission = verifier.authenticate_host_gateway_v1(&claim).unwrap();
        assert_eq!(
            admission.native_selection(),
            &fixture.context.selections.native
        );
        assert_eq!(
            admission.principal(),
            &fixture.context.accounts.native.principal
        );
        assert_eq!(
            admission.workspace_root(),
            fixture
                .context
                .workspace_binding
                .workspace_root
                .physical_path
        );
        admission.revalidate_before_effect().unwrap();
        assert_eq!(std::fs::read(&state_path).unwrap(), state_before);

        for field in 0..8 {
            let mut changed = claim.clone();
            match field {
                0 => changed.start.authority_store_id.push_str("-other"),
                1 => changed.start.original_start_intent_id.push_str("-other"),
                2 => changed
                    .start
                    .original_start_issuer_request_id
                    .push_str("-other"),
                3 => changed.start.original_session_id.push_str("-other"),
                4 => changed.start.context_sha256 = "f".repeat(64),
                5 => changed.start.native_selection_sha256 = "e".repeat(64),
                6 => changed.start.requested_selection_sha256 = "d".repeat(64),
                _ => {
                    let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
                        run_id,
                        ..
                    } = &mut changed.runtime
                    else {
                        unreachable!()
                    };
                    run_id.push_str("-other");
                }
            }
            assert!(
                verifier.authenticate_host_gateway_v1(&changed).is_err(),
                "claim field {field}"
            );
        }

        // Captured original config and inventory remain pinned. Current policy
        // and the current native account are freshly checked on every call.
        std::fs::write(
            fixture.home.join("config.yaml"),
            "agents:\n  enabled: false\n",
        )
        .unwrap();
        std::fs::write(
            fixture.home.join("agents/codex.yaml"),
            "malformed live inventory\n",
        )
        .unwrap();
        admission.revalidate_before_effect().unwrap();
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "synthetic-host-bridge-refresh",
        );
        admission.revalidate_before_effect().unwrap();
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-other-account",
        );
        assert!(admission.revalidate_before_effect().is_err());
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        admission.revalidate_before_effect().unwrap();

        let policy_path = fixture.home.join("policy.yaml");
        let policy_bytes = std::fs::read(&policy_path).unwrap();
        let policy = std::str::from_utf8(&policy_bytes).unwrap();
        for revoked in [
            policy.replace("  allowed_backends: [cli:codex-host]\n", "  allowed_backends: []\n"),
            policy.replace("    providers: [openai]\n", "    providers: [anthropic]\n"),
            policy.replace("    routers: [substrate_gateway]\n", "    routers: [other_router]\n"),
            policy.replace("    protocols: [openai.responses]\n", "    protocols: [anthropic.messages]\n"),
            policy.replace("    auth_authorities: [codex_subscription]\n", "    auth_authorities: [anthropic_api_key]\n"),
            policy.replace("llm:\n", "llm:\n  require_approval: true\n"),
            policy.replace("    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n", "    env_allowed: []\n"),
        ] {
            assert_ne!(revoked, policy);
            std::fs::write(&policy_path, revoked).unwrap();
            assert!(admission.revalidate_before_effect().is_err());
            std::fs::write(&policy_path, &policy_bytes).unwrap();
            admission.revalidate_before_effect().unwrap();
        }

        let context_path = fixture
            .home
            .join("run/agent-hub/ordinary-start-contexts")
            .join(format!("{}.json", fixture.context.intent_id));
        let context_bytes = std::fs::read(&context_path).unwrap();
        let mut substituted: NativeLaunchContextV1 = decode(&context_bytes).unwrap();
        substituted.opaque_launch_plan_digest = "c".repeat(64);
        let substituted_bytes = encode(&substituted).unwrap();
        assert_ne!(context_bytes, substituted_bytes);
        std::fs::write(&context_path, substituted_bytes).unwrap();
        // A matching replacement DTO/digest still cannot replace the genuine
        // original Applied issuer/payload binding.
        assert!(verifier
            .authenticate_host_gateway_v1(&host_gateway_claim(&substituted))
            .is_err());
        assert!(admission.revalidate_before_effect().is_err());
        std::fs::write(&context_path, context_bytes).unwrap();
        admission.revalidate_before_effect().unwrap();
        assert_eq!(std::fs::read(&state_path).unwrap(), state_before);

        // An actual registration invalidates the old borrowed observation but
        // preserves this admission's exact original/current application join.
        use crate::execution::agent_runtime::host_session_authority::facade::RetainedWorkerAuthorityPreconditionV1;
        use crate::execution::agent_runtime::retained_worker_runtime::{
            RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntime,
        };
        let before = fixture
            .authority
            .resolve_current_exact(&fixture.context.orchestration_session_id, None)
            .unwrap();
        let old_observation = verifier
            .authority
            .observe_current_host_gateway_run_v1(
                &fixture.context.orchestration_session_id,
                &fixture.context.original_participant_id,
                &fixture.context.descriptor.backend_id,
                &fixture.context.original_run_id,
            )
            .unwrap();
        let mut worker = fixture.context.descriptor.clone();
        worker.agent_id = fixture
            .context
            .selections
            .requested
            .realized_agent_id
            .clone();
        worker.backend_id = fixture.context.selections.requested.backend_id.clone();
        worker.execution_scope = AgentExecutionScopeV1::World;
        RetainedWorkerRuntime
            .register_retained_target(
                &fixture.authority,
                &RetainedWorkerRegistrationPlanV1 {
                    registration_request_id: "host-bridge-registration".into(),
                    orchestration_session_id: fixture.context.orchestration_session_id.clone(),
                    expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                        authority_store_id: before.observation.authority_store_id,
                        authority_revision: before.observation.authority_revision,
                        authority_record_commitment: before.observation.authority_record_commitment,
                    },
                    retained_participant_id: "host-bridge-worker".into(),
                    descriptor: worker,
                    internal_uaa_session_id: "host-bridge-native-worker".into(),
                },
            )
            .unwrap();
        assert!(old_observation.revalidate().is_err());
        let registered_state = std::fs::read(&state_path).unwrap();
        admission.revalidate_before_effect().unwrap();
        assert_eq!(std::fs::read(state_path).unwrap(), registered_state);
    }

    #[test]
    fn test_host_gateway_admission_requires_configured_custody() {
        let fixture = fixture(true);
        let principal = &fixture.context.accounts.native.principal;
        let unsealed = OrdinarySelectionAuthorityV1::from_held_authority(
            Arc::clone(&fixture.authority),
            principal,
        )
        .unwrap();
        let claim = host_gateway_claim(&fixture.context);
        let error = unsealed.authenticate_host_gateway_v1(&claim).err().unwrap();
        assert!(error.to_string().contains("configured authority custody"));
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "ordinary_world_integration_tests.rs"]
mod ordinary_world_integration_tests;

/// Checked finite acquisition disposition. NoAcquireSent is not a Released fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostGatewayCleanupOutcomeV1 {
    NoAcquireSent,
    Released,
    Unknown,
}
// Held only by the finite client attempt after uncertainty. A failed Release
// exchange cannot erase original cleanup credentials or authorize a fresh key.
// No retry/expiry or durable history is inferred; the service remains owner of
// its actual Child until an authenticated cleanup exchange reaches it.
struct HostGatewayCleanupBindingV1 {
    _client: Arc<transport_api_client::AgentClient>,
    _service_instance_id: String,
    _acquire_id: String,
    _cleanup_token: String,
    _selection: transport_api_types::OrdinaryGatewaySelectionContextV1,
    _route_verifier: String,
    _handle: Option<String>,
}
struct HostGatewayAcquisitionStateV1 {
    closed: bool,
    cleanup_binding: Option<HostGatewayCleanupBindingV1>,
    sender: Option<tokio::sync::watch::Sender<Option<HostGatewayCleanupOutcomeV1>>>,
}
/// Attached to NativeAttempt before preparation can issue any request. A delivered
/// lease is only a locator; this retained owner also joins pending acquisition.
pub(crate) struct HostGatewayAcquisitionCustodyV1 {
    state: std::sync::Mutex<HostGatewayAcquisitionStateV1>,
    close: Arc<HostGatewayCloseV1>,
    terminal: tokio::sync::watch::Receiver<Option<HostGatewayCleanupOutcomeV1>>,
}
impl HostGatewayAcquisitionCustodyV1 {
    pub(crate) fn new() -> Self {
        let (sender, terminal) =
            tokio::sync::watch::channel(Some(HostGatewayCleanupOutcomeV1::NoAcquireSent));
        Self {
            state: std::sync::Mutex::new(HostGatewayAcquisitionStateV1 {
                closed: false,
                cleanup_binding: None,
                sender: Some(sender),
            }),
            close: Arc::new(HostGatewayCloseV1::new()),
            terminal,
        }
    }
    // Gate start and close together: settlement cannot observe NoAcquireSent and
    // then permit a later request. Taking the sender is the single attempt latch.
    pub(crate) fn begin(
        &self,
    ) -> Result<tokio::sync::watch::Sender<Option<HostGatewayCleanupOutcomeV1>>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        ensure!(!state.closed, "Host acquisition already closed");
        let sender = state
            .sender
            .take()
            .context("Host acquisition already attempted")?;
        sender.send_replace(None);
        Ok(sender)
    }
    pub(crate) fn request_close(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.closed = true;
        self.close.cancel();
    }
    pub(crate) async fn close_and_wait(&self) -> Result<HostGatewayCleanupOutcomeV1> {
        self.request_close();
        let mut terminal = self.terminal.clone();
        loop {
            let outcome = *terminal.borrow();
            match outcome {
                Some(HostGatewayCleanupOutcomeV1::Unknown) => {
                    anyhow::bail!("submission_outcome_unknown: Host gateway cleanup unacknowledged")
                }
                Some(outcome) => return Ok(outcome),
                None => {}
            }
            if terminal.changed().await.is_err() {
                anyhow::bail!(
                    "submission_outcome_unknown: Host acquisition cleanup owner unavailable"
                );
            }
        }
    }
}

/// An opaque exact service lease locator, never a cleanup credential or HSA capability.
/// Credential and response-loss cleanup stay in the independent acquisition owner.
pub(crate) struct HostGatewayTurnLeaseV1 {
    client: Arc<transport_api_client::AgentClient>,
    acquire_id: String,
    service_instance_id: String,
    // Opaque transient custody until the genuine native delivery vehicle is proved.
    _local_route_capability: String,
    handle: String,
    acquisition: Arc<HostGatewayAcquisitionCustodyV1>,
}
impl HostGatewayTurnLeaseV1 {
    pub(crate) fn request_close(&self) {
        self.acquisition.request_close();
    }

    pub(crate) async fn revalidate_before_spawn(&self) -> std::io::Result<()> {
        use transport_api_types::{
            HostGatewayTurnOperationV1 as Op, HostGatewayTurnResultV1 as Reply,
        };
        if self.acquisition.close.is_cancelled() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Host turn admission closed",
            ));
        }
        let response = execute_bound_host_turn_v3(
            &self.client,
            &self.service_instance_id,
            None,
            transport_api_types::HostGatewayTurnRequestV1 {
                schema_version: 1,
                operation: Op::Revalidate {
                    acquire_id: self.acquire_id.clone(),
                    lease_handle: self.handle.clone(),
                },
            },
        )
        .await;
        if !matches!(
            response,
            Ok(transport_api_types::HostGatewayTurnResponseV1 {
                result: Reply::Revalidated { .. },
                ..
            })
        ) || self.acquisition.close.is_cancelled()
        {
            self.request_close();
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Host turn admission refused",
            ));
        }
        Ok(())
    }

    pub(crate) fn acquisition(&self) -> &Arc<HostGatewayAcquisitionCustodyV1> {
        &self.acquisition
    }
    pub(crate) async fn close_and_wait(&self) -> Result<()> {
        self.acquisition.close_and_wait().await.map(|_| ())
    }
}
impl Drop for HostGatewayTurnLeaseV1 {
    fn drop(&mut self) {
        self.request_close();
    }
}

async fn acquire_host_gateway_turn_v3(
    claim: transport_api_types::OrdinaryGatewaySelectionContextV1,
    acquisition: Arc<HostGatewayAcquisitionCustodyV1>,
) -> Result<(
    Arc<HostGatewayTurnLeaseV1>,
    transport_api_types::GatewayClientWiringV1,
)> {
    use base64::Engine;
    use transport_api_types::{HostGatewayTurnOperationV1 as Op, HostGatewayTurnResultV1 as Reply};
    let client = Arc::new(transport_api_client::AgentClient::unix_socket(
        "/run/substrate.sock",
    )?);
    // Generate once before the first request; no new key or automatic Acquire retry
    // can escape an ambiguous response. The token exists only in this finite owner.
    let token: [u8; 32] = rand::random();
    let cleanup_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token);
    let route_bytes: [u8; 32] = rand::random();
    let local_route_capability =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(route_bytes);
    let acquire_id = transport_api_types::HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(
        &cleanup_token,
    )
    .map_err(anyhow::Error::msg)?;
    let terminal_sender = acquisition.begin()?;
    let close = Arc::clone(&acquisition.close);
    let (mut ready, acquired) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let service_instance_id = tokio::select! {
            biased;
            _ = close.cancelled() => Err(anyhow::anyhow!("Host acquisition closed before Observe")),
            epoch = observe_host_gateway_instance_v3(&client) => epoch,
            _ = ready.closed() => Err(anyhow::anyhow!("Host observation disposed before Acquire")),
        };
        let service_instance_id = match service_instance_id {
            Ok(epoch) => epoch,
            Err(_) => {
                let _ = ready.send(Err(anyhow::anyhow!(
                    "Host service epoch unavailable; Acquire not sent"
                )));
                // Observe cannot reserve or spawn: no Acquire was polled.
                let _ = terminal_sender.send(Some(HostGatewayCleanupOutcomeV1::NoAcquireSent));
                return;
            }
        };
        let route_verifier =
            match transport_api_types::HostGatewayInstanceRequestV3::route_verifier_from_capability(
                &service_instance_id,
                &acquire_id,
                &local_route_capability,
            ) {
                Ok(verifier) => verifier,
                Err(_) => {
                    let _ = ready.send(Err(anyhow::anyhow!("Host local route binding invalid")));
                    let _ = terminal_sender.send(Some(HostGatewayCleanupOutcomeV1::NoAcquireSent));
                    return;
                }
            };
        acquisition
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cleanup_binding = Some(HostGatewayCleanupBindingV1 {
            _client: Arc::clone(&client),
            _service_instance_id: service_instance_id.clone(),
            _acquire_id: acquire_id.clone(),
            _cleanup_token: cleanup_token.clone(),
            _selection: claim.clone(),
            _route_verifier: route_verifier.clone(),
            _handle: None,
        });
        // All four original values now belong to this finite owner before the
        // first Acquire send: epoch, key, cleanup credential and exact claim.
        let request = transport_api_types::HostGatewayTurnRequestV1 {
            schema_version: 1,
            operation: Op::Acquire {
                acquire_id: acquire_id.clone(),
                selection: Box::new(claim),
                cleanup_token: cleanup_token.clone(),
            },
        };
        if close.is_cancelled() || ready.is_closed() {
            let _ = ready.send(Err(anyhow::anyhow!(
                "Host acquisition disposed before Acquire"
            )));
            acquisition
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .cleanup_binding = None;
            let _ = terminal_sender.send(Some(HostGatewayCleanupOutcomeV1::NoAcquireSent));
            return;
        }
        let response = tokio::select! {
            _ = close.cancelled() => Err(anyhow::anyhow!("Host acquisition cancelled; cleanup required")),
            response = execute_bound_host_turn_v3(&client, &service_instance_id, Some(&route_verifier), request) => response,
            _ = ready.closed() => Err(anyhow::anyhow!("Host acquisition observer disposed")),
        };
        let mut handle = None;
        match response {
            Ok(transport_api_types::HostGatewayTurnResponseV1 {
                result:
                    Reply::Acquired {
                        lease_handle,
                        client_wiring,
                        ..
                    },
                ..
            }) => {
                handle = Some(lease_handle.clone());
                if let Some(binding) = acquisition
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .cleanup_binding
                    .as_mut()
                {
                    binding._handle = Some(lease_handle.clone());
                }
                let lease = Arc::new(HostGatewayTurnLeaseV1 {
                    client: Arc::clone(&client),
                    acquire_id: acquire_id.clone(),
                    service_instance_id: service_instance_id.clone(),
                    _local_route_capability: local_route_capability,
                    handle: lease_handle,
                    acquisition: Arc::clone(&acquisition),
                });
                if ready.send(Ok((lease, client_wiring))).is_err() {
                    close.cancel();
                }
            }
            _ => {
                close.cancel();
                let _ = ready.send(Err(anyhow::anyhow!(
                    "submission_outcome_unknown: Host acquisition refused or unresolved"
                )));
            }
        }
        while !close.is_cancelled() {
            tokio::select! {
                _ = close.cancelled() => break,
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
            }
            let Some(lease_handle) = handle.as_ref() else {
                break;
            };
            let request = transport_api_types::HostGatewayTurnRequestV1 {
                schema_version: 1,
                operation: Op::Revalidate {
                    acquire_id: acquire_id.clone(),
                    lease_handle: lease_handle.clone(),
                },
            };
            let check = tokio::select! {
                check = execute_bound_host_turn_v3(&client, &service_instance_id, None, request) => check,
                _ = close.cancelled() => break,
            };
            if !matches!(
                check,
                Ok(transport_api_types::HostGatewayTurnResponseV1 {
                    result: Reply::Revalidated { .. },
                    ..
                })
            ) {
                close.cancel();
            }
        }
        // Cleanup is attempted with the same key/credential even when Acquire's
        // response was lost. Unknown, restart or transport loss never means Released.
        let release = execute_bound_host_turn_v3(
            &client,
            &service_instance_id,
            None,
            transport_api_types::HostGatewayTurnRequestV1 {
                schema_version: 1,
                operation: Op::Release {
                    acquire_id,
                    lease_handle: handle,
                    cleanup_token,
                },
            },
        )
        .await;
        let outcome = match release {
            Ok(transport_api_types::HostGatewayTurnResponseV1 {
                result: Reply::Released { .. },
                ..
            }) => HostGatewayCleanupOutcomeV1::Released,
            _ => HostGatewayCleanupOutcomeV1::Unknown,
        };
        if outcome == HostGatewayCleanupOutcomeV1::Released {
            acquisition
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .cleanup_binding = None;
        }
        let _ = terminal_sender.send(Some(outcome));
    });
    acquired
        .await
        .context("submission_outcome_unknown: Host acquisition owner unavailable")?
}

struct HostGatewayCloseV1 {
    sender: tokio::sync::watch::Sender<bool>,
    receiver: tokio::sync::watch::Receiver<bool>,
}
impl HostGatewayCloseV1 {
    fn new() -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(false);
        Self { sender, receiver }
    }
    fn cancel(&self) {
        self.sender.send_replace(true);
    }
    fn is_cancelled(&self) -> bool {
        *self.receiver.borrow()
    }
    async fn cancelled(&self) {
        let mut receiver = self.receiver.clone();
        loop {
            if *receiver.borrow() {
                return;
            }
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}

async fn observe_host_gateway_instance_v3(
    client: &transport_api_client::AgentClient,
) -> Result<String> {
    use transport_api_types::{
        HostGatewayInstanceOperationV3 as Op, HostGatewayInstanceResultV3 as Reply,
    };
    let response = client
        .gateway_host_instance(transport_api_types::HostGatewayInstanceRequestV3 {
            schema_version: 3,
            operation: Op::Observe {},
        })
        .await?;
    let Reply::Observed {
        service_instance_id,
    } = response.result
    else {
        anyhow::bail!("Host service epoch observation refused");
    };
    Ok(service_instance_id)
}

async fn execute_bound_host_turn_v3(
    client: &transport_api_client::AgentClient,
    original_epoch: &str,
    route_verifier: Option<&str>,
    request: transport_api_types::HostGatewayTurnRequestV1,
) -> Result<transport_api_types::HostGatewayTurnResponseV1> {
    use transport_api_types::{
        HostGatewayInstanceOperationV3 as Op, HostGatewayInstanceResultV3 as Reply,
    };
    let response = client
        .gateway_host_instance(transport_api_types::HostGatewayInstanceRequestV3 {
            schema_version: 3,
            operation: Op::Bound {
                service_instance_id: original_epoch.into(),
                operation: request.operation,
                route_verifier: route_verifier.map(str::to_owned),
            },
        })
        .await?;
    let Reply::Bound {
        service_instance_id,
        result,
    } = response.result
    else {
        anyhow::bail!("Host service instance result refused");
    };
    ensure!(
        service_instance_id == original_epoch,
        "Host service epoch changed; ownership unresolved"
    );
    Ok(transport_api_types::HostGatewayTurnResponseV1 {
        schema_version: 1,
        result,
    })
}

#[cfg(all(test, target_os = "linux"))]
#[path = "privileged_host_fixture_preview.rs"]
mod privileged_host_fixture_preview;
