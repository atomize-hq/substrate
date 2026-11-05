//! Authentication for the direct-host Resume protocol. Serializable bindings
//! describe a proof; this private-constructor capability also retains its real
//! inherited locked helper descriptor and exact current HSA joins.
use crate::execution::config_model::AgentExecutionScope;
use std::{
    fs::File,
    io::Read as _,
    os::{
        fd::{AsRawFd as _, FromRawFd as _},
        unix::fs::MetadataExt as _,
    },
    path::Path,
};

use super::{
    canonical_json,
    facade::exact_v3_authority_history_with_reader,
    hash::canonical_sha256,
    schema::*,
    store::{BootstrapError, DirectHostInboxTransactionV1},
    store_schema::*,
    trusted_fs::{DirectoryEntry, TrustedAuthorityRoot, TrustedDirectory, TrustedWorkspaceRoot},
    validation::ValidatedCanonicalV1 as _,
};
use crate::execution::agent_runtime::{
    control::{AuthorityManagedSuccessorLaunchPlanV1, OwnerHelperMode, ResolvedRuntimeBackendKind},
    obligation_ledger::direct_host_inbox::direct_host_inbox_commitment,
};

fn refusal(reason: &'static str) -> BootstrapError {
    BootstrapError::direct_host_inbox(reason)
}

/// No Deserialize, public fields, Clone, or unauthenticated constructor. The
/// ledger receives this only from HSA while the retained physical root is held.
pub(crate) struct AuthenticatedDirectHostResumeV1 {
    binding: DirectHostResumeBindingV1,
    birth_authority_commitment: AuthorityObjectCommitmentV1,
    descriptor: AgentDescriptorV1,
    native_session_id: String,
    guard: DirectHostResumeGuardV1,
    workspace: TrustedWorkspaceRoot,
}

impl AuthenticatedDirectHostResumeV1 {
    pub(crate) fn binding(&self) -> &DirectHostResumeBindingV1 {
        &self.binding
    }
    pub(crate) fn birth_authority_commitment(&self) -> &AuthorityObjectCommitmentV1 {
        &self.birth_authority_commitment
    }
    pub(crate) fn descriptor(&self) -> &AgentDescriptorV1 {
        &self.descriptor
    }
    pub(crate) fn native_session_id(&self) -> &str {
        &self.native_session_id
    }
    pub(crate) fn verify_active(
        &self,
        tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    ) -> Result<(), BootstrapError> {
        self.verify_guard()?;
        let Some(VersionedStateRoot::V3(root)) = tx.authority_root() else {
            return Err(refusal("direct-host producer has no V3 authority"));
        };
        let b = &self.binding;
        let intent = root
            .successor_transition_intent_map
            .get(&b.intent_id)
            .ok_or_else(|| refusal("direct-host producer intent disappeared"))?;
        let Some(SessionNamespaceRecordV1::Authority(authority)) =
            root.session_namespace_map.get(&b.orchestration_session_id)
        else {
            return Err(refusal("direct-host producer session disappeared"));
        };
        if root.authority_store_id != b.authority_store_id
            || root.bootstrap_home != b.bootstrap_home
            || intent.intent_revision != b.intent_revision_observed
            || intent.payload_commitment != b.payload_commitment
            || intent.run_id != b.run_id
            || intent.world_binding.is_some()
            || intent.workspace_binding != b.workspace_binding
            || authority.world_binding.is_some()
            || authority.workspace_binding != b.workspace_binding
            || authority.authority_revision != b.authority_revision_observed
            || authority.authoritative_participant_lineage != b.authoritative_lineage
            || authority.current_policy_ref != b.policy_ref
            || authority.current_policy_revision != b.policy_revision
            || authority.host_attach_contract_ref.as_ref() != Some(&b.host_attach_contract_ref)
            || (AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&direct_host_authority_hash_input(authority))
                    .map_err(|_| refusal("hash current direct-host producer authority"))?,
            }) != b.authority_record_commitment
            || authority.active_authoritative_participant_id.as_ref()
                != Some(&b.authoritative_participant_id)
            || !matches!(&intent.state, HostSessionTransitionIntentStateV3::Applied { claim_id,
                claimant_attempt_id, application_result_ref, post_turn, .. }
                if claim_id == &b.claim_id && claimant_attempt_id == &b.claimant_attempt_id
                && application_result_ref == &b.initial_application_result_ref
                && matches!(post_turn.as_ref(), HostSessionPostTurnApplicationV2::Pending {
                    expected_run_id, expected_authority_revision
                } if expected_run_id == &b.run_id && *expected_authority_revision == b.authority_revision_observed))
            || !matches!(&intent.input_handoff, HostSessionTransitionInputHandoffV1::Accepted {
                input_ref, run_id, acceptance_ref, ..
            } if input_ref == &b.transition_input_ref && run_id == &b.run_id && Some(acceptance_ref) == b.input_acceptance_ref.as_ref())
        {
            return Err(refusal(
                "stale or completed direct-host producer capability",
            ));
        }
        tx.verify()
    }
    pub(crate) fn verify_guard(&self) -> Result<(), BootstrapError> {
        self.guard.verify()?;
        self.workspace
            .revalidate()
            .map_err(|_| refusal("direct-host workspace identity changed"))
    }
}

struct DirectHostResumeGuardV1 {
    root: TrustedAuthorityRoot,
    directories: Vec<(DirectoryEntry, TrustedDirectory)>,
    guard_entry: DirectoryEntry,
    inherited: File,
    expected_device: u64,
    expected_inode: u64,
}

impl DirectHostResumeGuardV1 {
    fn open(home: &CanonicalDirectoryV1, intent_id: &str) -> Result<Self, BootstrapError> {
        if intent_id.is_empty()
            || Path::new(intent_id).components().count() != 1
            || !matches!(
                Path::new(intent_id).components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err(refusal("invalid direct-host helper intent identity"));
        }
        let fd = std::env::var("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD")
            .map_err(|_| refusal("direct-host Resume has no inherited helper guard"))?
            .parse::<i32>()
            .map_err(|_| refusal("direct-host helper guard descriptor is invalid"))?;
        if fd < 0 {
            return Err(refusal("direct-host helper guard descriptor is invalid"));
        }
        // SAFETY: fcntl validates the caller-supplied descriptor; only the new
        // duplicate is owned here. The inherited description remains untouched.
        let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
        if duplicate < 0 {
            return Err(refusal("duplicate inherited direct-host helper guard"));
        }
        // SAFETY: successful F_DUPFD_CLOEXEC returned this fresh owned fd.
        let inherited = unsafe { File::from_raw_fd(duplicate) };
        let root = TrustedAuthorityRoot::open(Path::new(&home.physical_path))
            .map_err(|_| refusal("open direct-host helper guard root"))?;
        if root.identity() != home {
            return Err(refusal("direct-host helper guard physical root changed"));
        }
        let mut directories: Vec<(DirectoryEntry, TrustedDirectory)> = Vec::new();
        for name in [
            "runtime-control",
            "durable-start",
            "successor-launch-guards",
        ] {
            let parent = directories
                .last()
                .map(|(_, dir)| dir)
                .unwrap_or_else(|| root.directory());
            let entry = parent
                .entries()
                .map_err(|_| refusal("enumerate helper guard directory"))?
                .into_iter()
                .find(|e| e.name == name)
                .ok_or_else(|| refusal("helper guard directory is absent"))?;
            let opened = parent
                .open_controlled_directory_entry(&entry)
                .map_err(|_| refusal("open retained helper guard directory"))?;
            directories.push((entry, opened));
        }
        let parent = &directories
            .last()
            .ok_or_else(|| refusal("helper guard parent is absent"))?
            .1;
        let name = format!("{intent_id}.lock");
        let guard_entry = parent
            .entries()
            .map_err(|_| refusal("enumerate helper guard file"))?
            .into_iter()
            .find(|e| e.name == name)
            .ok_or_else(|| refusal("helper guard file is absent"))?;
        let metadata = inherited
            .metadata()
            .map_err(|_| refusal("inspect inherited helper guard"))?;
        let guard = Self {
            root,
            directories,
            guard_entry,
            inherited,
            expected_device: metadata.dev(),
            expected_inode: metadata.ino(),
        };
        guard.verify()?;
        Ok(guard)
    }

    fn verify(&self) -> Result<(), BootstrapError> {
        self.root
            .revalidate()
            .map_err(|_| refusal("helper guard root changed"))?;
        let mut parent = self.root.directory();
        for (entry, directory) in &self.directories {
            parent
                .revalidate_entry(entry)
                .map_err(|_| refusal("helper guard directory identity changed"))?;
            directory
                .metadata()
                .map_err(|_| refusal("helper guard directory mode changed"))?;
            parent = directory;
        }
        parent
            .revalidate_entry(&self.guard_entry)
            .map_err(|_| refusal("helper guard file identity changed"))?;
        let name = std::ffi::CString::new(self.guard_entry.name.as_str())
            .map_err(|_| refusal("helper guard file name is invalid"))?;
        // Existing launch guards may be 0644; require owner-only writes rather
        // than changing their established creation mode or trusted_fs rules.
        // SAFETY: retained parent fd and validated single-component name live
        // through openat. No symlink/device/FIFO is accepted by metadata below.
        let raw = unsafe {
            libc::openat(
                parent.borrow_fd().as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
                0,
            )
        };
        if raw < 0 {
            return Err(refusal("open exact helper guard file"));
        }
        // SAFETY: raw is a fresh owned descriptor from successful openat.
        let named = unsafe { File::from_raw_fd(raw) };
        for file in [&named, &self.inherited] {
            let m = file
                .metadata()
                .map_err(|_| refusal("inspect exact helper guard file"))?;
            if !m.is_file()
                || m.uid() != unsafe { libc::geteuid() }
                || m.mode() & 0o022 != 0
                || m.nlink() != 1
                || m.dev() != self.expected_device
                || m.ino() != self.expected_inode
            {
                return Err(refusal(
                    "helper guard file does not join inherited locked description",
                ));
            }
        }
        let mut fdinfo = String::new();
        File::open(format!("/proc/self/fdinfo/{}", self.inherited.as_raw_fd()))
            .map_err(|_| refusal("observe inherited helper guard lock"))?
            .take(4097)
            .read_to_string(&mut fdinfo)
            .map_err(|_| refusal("read inherited helper guard lock"))?;
        if fdinfo.len() > 4096
            || !fdinfo.lines().any(|line| {
                let t = line.split_whitespace().collect::<Vec<_>>();
                t.len() == 9
                    && t[0] == "lock:"
                    && t[2] == "FLOCK"
                    && t[3] == "ADVISORY"
                    && t[4] == "WRITE"
                    && t[7] == "0"
                    && t[8] == "EOF"
            })
        {
            return Err(refusal("helper guard has no inherited exclusive flock"));
        }
        parent
            .revalidate_entry(&self.guard_entry)
            .map_err(|_| refusal("helper guard changed during verification"))?;
        Ok(())
    }
}

/// Construct only from the actual persisted Applied Resume and the inherited
/// helper lock. A copied JSON plan, matching flag, or missing world is insufficient.
pub(super) fn authenticate_direct_host_resume_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<AuthenticatedDirectHostResumeV1, BootstrapError> {
    let VersionedStateRoot::V3(root) = tx
        .authority_root()
        .ok_or_else(|| refusal("direct-host Resume authority is absent"))?
    else {
        return Err(refusal("direct-host Resume requires existing V3 authority"));
    };
    let root = root.clone();
    let expected = &plan.applied_transition;
    let intent = root
        .successor_transition_intent_map
        .get(&expected.intent_id)
        .ok_or_else(|| refusal("direct-host Resume intent is absent"))?
        .clone();
    if plan.schema_version != 1
        || plan.authority_store_id != root.authority_store_id
        || root.bootstrap_home != *tx.bootstrap_home()
        || intent.mode != HostSessionTransitionModeV1::ResumeOneTurn
        || intent.world_binding.is_some()
        || intent.workspace_binding.authority_store_id != root.authority_store_id
        || intent.workspace_binding.authority_store_root != root.bootstrap_home
        || expected.world_binding.is_some()
        || expected.mode != intent.mode
        || expected.intent_revision > intent.intent_revision
        || expected.payload_commitment != intent.payload_commitment
        || canonical_sha256(&direct_host_payload(&intent))
            .map_err(|_| refusal("validate current Resume payload"))?
            != canonical_sha256(&direct_host_payload(expected))
                .map_err(|_| refusal("validate helper Resume payload"))?
        || intent.payload_commitment
            != (AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&direct_host_payload(&intent))
                    .map_err(|_| refusal("hash direct-host Resume payload"))?,
            })
    {
        return Err(refusal(
            "direct-host Resume immutable helper and authority context disagree",
        ));
    }
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        active_authoritative_participant_id,
        authority_record_commitment,
        application_result_ref,
        resulting_posture,
        post_turn,
        applied_at,
        ..
    } = &intent.state
    else {
        return Err(refusal("direct-host Resume is not Applied"));
    };
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id: expected_claim,
        claimant_attempt_id: expected_attempt,
        authority_revision_before: expected_before,
        authority_revision_after: expected_after,
        active_authoritative_participant_id: expected_participant,
        resulting_posture: expected_posture,
        authority_record_commitment: expected_commitment,
        application_result_ref: expected_result,
        applied_at: expected_at,
        ..
    } = &expected.state
    else {
        return Err(refusal("helper plan is not an applied Resume"));
    };
    if (
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        active_authoritative_participant_id,
        authority_record_commitment,
        application_result_ref,
        resulting_posture,
        applied_at,
    ) != (
        expected_claim,
        expected_attempt,
        expected_before,
        expected_after,
        expected_participant,
        expected_commitment,
        expected_result,
        expected_posture,
        expected_at,
    ) {
        return Err(refusal("direct-host Resume application identity changed"));
    }
    let active = matches!(post_turn.as_ref(), HostSessionPostTurnApplicationV2::Pending {
        expected_run_id, expected_authority_revision
    } if expected_run_id == &intent.run_id && expected_authority_revision == authority_revision_after);
    if active_authoritative_participant_id != &intent.target_authoritative_participant_id
        || *resulting_posture != HostSessionPostureV1::ActiveAttached
    {
        return Err(refusal(
            "direct-host Resume applied actor or posture changed",
        ));
    }
    if !active {
        return Err(refusal(
            "completed helper cannot materialize local requests",
        ));
    }
    if !matches!(&expected.input_handoff, HostSessionTransitionInputHandoffV1::Pending {
        input_ref, run_id
    } if expected.transition_input_ref.as_ref() == Some(input_ref) && run_id == &expected.run_id)
    {
        return Err(refusal(
            "direct-host helper plan did not precede its input acceptance",
        ));
    }
    let HostSessionTransitionInputHandoffV1::Accepted {
        input_ref,
        run_id,
        acceptance_ref,
        accepted_at,
    } = &intent.input_handoff
    else {
        return Err(refusal(
            "direct-host materializer requires exact Accepted input",
        ));
    };
    if intent.transition_input_ref.as_ref() != Some(input_ref) || run_id != &intent.run_id {
        return Err(refusal(
            "direct-host Resume accepted input identity changed",
        ));
    }
    let acceptance: InputAcceptanceHashInputV1 = direct_host_read(tx, acceptance_ref)?;
    if acceptance.schema_version != 1
        || acceptance.intent_id != intent.intent_id
        || acceptance.run_id != intent.run_id
        || acceptance.input_ref != *input_ref
        || acceptance.accepting_participant_id != intent.target_authoritative_participant_id
        || acceptance.accepted_at != *accepted_at
    {
        return Err(refusal(
            "direct-host Resume input acceptance does not join helper",
        ));
    }
    let history = exact_v3_authority_history_with_reader(
        &root,
        &intent.orchestration_session_id,
        |reference| {
            tx.read_authority_object(reference)
                .map_err(|e| e.to_string())
        },
    )
    .map_err(|_| refusal("direct-host Resume exact authority ancestry is unknown"))?;
    let applied = history
        .get(authority_revision_after)
        .ok_or_else(|| refusal("direct-host Resume applied authority history is absent"))?;
    if history.iter().any(|(_, h)| {
        h.authority.world_binding.is_some()
            || h.authority.workspace_binding != intent.workspace_binding
            || h.authority.shell_trace_session_id != intent.shell_trace_session_id
    }) || applied.authority_record_commitment != *authority_record_commitment
        || applied.authority.authoritative_participant_lineage
            != intent.resulting_authoritative_lineage
        || applied
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(&intent.target_authoritative_participant_id)
        || (active && history.last_key_value().map(|(r, _)| r) != Some(authority_revision_after))
    {
        return Err(refusal(
            "direct-host Resume authority history has a different scope or actor",
        ));
    }
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision: source_revision,
        authority_record_commitment: source_commitment,
        active_authoritative_participant_id: source_participant,
        authoritative_lineage_commitment: source_lineage,
        lifecycle_posture: source_posture,
    } = &intent.authority_precondition
    else {
        return Err(refusal("direct-host Resume has no exact source authority"));
    };
    let source = history
        .get(source_revision)
        .ok_or_else(|| refusal("direct-host Resume source history is absent"))?;
    if source.authority_record_commitment != *source_commitment
        || source.authoritative_lineage_commitment != *source_lineage
        || source.authority.lifecycle_posture != *source_posture
        || source
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(source_participant)
        || intent.source_authoritative_participant_id.as_ref() != Some(source_participant)
        || authority_revision_before.as_ref() != Some(source_revision)
    {
        return Err(refusal(
            "direct-host Resume source precondition does not join ancestry",
        ));
    }
    let handle_ref = intent
        .resume_handle_ref
        .as_ref()
        .ok_or_else(|| refusal("direct-host Resume consumed no handle"))?;
    if !source
        .authority
        .internal_resume_handle_refs
        .contains(handle_ref)
    {
        return Err(refusal(
            "direct-host Resume handle is not in exact source authority",
        ));
    }
    let descriptor: AgentDescriptorHashInputV1 = direct_host_read(tx, &intent.descriptor_ref)?;
    let attach: HostAttachContractHashInputV1 =
        direct_host_read(tx, &intent.host_attach_contract_ref)?;
    let policy: PolicyObjectHashInputV1 = direct_host_read(tx, &attach.contract.policy_ref)?;
    let descriptor = descriptor.descriptor;
    let helper = &plan.helper_plan;
    let backend_kind = match helper.descriptor.backend_kind {
        ResolvedRuntimeBackendKind::Codex => RuntimeBackendKindV1::Codex,
        ResolvedRuntimeBackendKind::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
    };
    if descriptor.execution_scope != AgentExecutionScopeV1::Host
        || descriptor.agent_id != helper.descriptor.agent_id
        || descriptor.backend_id != helper.descriptor.backend_id
        || descriptor.backend_kind != backend_kind
        || descriptor.protocol != helper.descriptor.protocol
        || descriptor.binary_path != helper.descriptor.binary_path
        || helper.descriptor.execution_scope != AgentExecutionScope::Host
        || helper.mode != OwnerHelperMode::ResumeOneTurn
        || helper.session.orchestration_session_id != intent.orchestration_session_id
        || helper.session.shell_trace_session_id != intent.shell_trace_session_id
        || helper.session.workspace_root != intent.workspace_binding.workspace_root.physical_path
        || helper.session.world_id.is_some()
        || helper.session.world_generation.is_some()
        || helper.participant.participant_id != intent.target_authoritative_participant_id
        || helper.participant.run_id != intent.run_id
        || helper.participant.resumed_from_participant_id.as_ref() != Some(source_participant)
        || attach.contract.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.backend_id != descriptor.backend_id
        || attach.contract.protocol != descriptor.protocol
        || attach.contract.descriptor_ref != intent.descriptor_ref
        || applied.authority.host_attach_contract_ref.as_ref()
            != Some(&intent.host_attach_contract_ref)
        || applied.authority.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
        || applied.authority.current_policy_revision.as_ref() != Some(&policy.policy_revision)
    {
        return Err(refusal(
            "direct-host Resume descriptor, plan, policy or attach joins disagree",
        ));
    }
    let (handle_session, handle_participant, handle_backend, handle_protocol, native_session_id) =
        match handle_ref.schema_version {
            1 => {
                let h: ResumeHandleHashInputV1 = direct_host_read(tx, handle_ref)?;
                (
                    h.orchestration_session_id,
                    h.participant_id,
                    h.backend_id,
                    h.protocol,
                    h.internal_uaa_session_id,
                )
            }
            2 => {
                let h: StartContinuationHandleHashInputV2 = direct_host_read(tx, handle_ref)?;
                if h.authority_store_id != root.authority_store_id {
                    return Err(refusal("native Start handle store changed"));
                }
                (
                    h.orchestration_session_id,
                    h.participant_id,
                    h.backend_id,
                    h.protocol,
                    h.internal_uaa_session_id,
                )
            }
            _ => return Err(refusal("unsupported direct-host Resume handle version")),
        };
    if handle_session != intent.orchestration_session_id
        || !source
            .authority
            .authoritative_participant_lineage
            .contains(&handle_participant)
        || handle_backend != descriptor.backend_id
        || handle_protocol != descriptor.protocol
        || native_session_id.is_empty()
        || helper.participant.internal_uaa_session_id.as_ref() != Some(&native_session_id)
    {
        return Err(refusal("direct-host Resume native handle context changed"));
    }
    let application: ApplicationResultHashInputV1 = direct_host_read(tx, application_result_ref)?;
    if application.schema_version != 1
        || application.intent_id != intent.intent_id
        || application.mode != intent.mode
        || application.run_id != intent.run_id
        || application.applied_at != *applied_at
        || application.phase
            != (ApplicationResultPhaseV1::InitialTransition {
                authority_revision_before: *authority_revision_before,
                authority_revision_after: *authority_revision_after,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture: *resulting_posture,
                authority_record_commitment: authority_record_commitment.clone(),
                post_turn_pending_run_id: Some(intent.run_id.clone()),
            })
    {
        return Err(refusal(
            "direct-host Resume initial application result does not join",
        ));
    }
    let prefix = history
        .range(..=*authority_revision_after)
        .map(|(rev, a)| {
            (
                *rev,
                a.authority_record_commitment.clone(),
                a.authoritative_lineage_commitment.clone(),
            )
        })
        .collect::<Vec<_>>();
    let binding = DirectHostResumeBindingV1 {
        schema_version: 1,
        authority_store_id: root.authority_store_id.clone(),
        bootstrap_home: root.bootstrap_home.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        authority_history_commitment: direct_host_inbox_commitment(
            "substrate.direct-host-resume.authority-prefix.v1",
            &prefix,
        )?,
        authority_revision_observed: *authority_revision_after,
        authority_record_commitment: authority_record_commitment.clone(),
        policy_ref: applied.authority.current_policy_ref.clone(),
        policy_revision: applied.authority.current_policy_revision.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        intent_revision_observed: expected
            .intent_revision
            .checked_add(1)
            .ok_or_else(|| refusal("accepted intent revision overflow"))?,
        claim_id: claim_id.clone(),
        claimant_attempt_id: claimant_attempt_id.clone(),
        run_id: intent.run_id.clone(),
        initial_application_result_ref: application_result_ref.clone(),
        transition_input_ref: input_ref.clone(),
        input_acceptance_ref: Some(acceptance_ref.clone()),
        consumed_resume_handle_ref: handle_ref.clone(),
    };
    binding
        .validate()
        .map_err(|_| refusal("direct-host Resume binding is invalid"))?;
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| refusal("direct-host Resume helper carries no exact input"))?;
    tx.verify_direct_host_resume_input_and_lease(
        &intent,
        helper.participant.lease_token.as_bytes(),
        prompt.prompt_text.as_bytes(),
    )?;
    let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
        .map_err(|_| refusal("open exact direct-host workspace"))?;
    let guard = DirectHostResumeGuardV1::open(&root.bootstrap_home, &intent.intent_id)?;
    tx.verify()?;
    let authenticated = AuthenticatedDirectHostResumeV1 {
        birth_authority_commitment: history
            .get(&1)
            .ok_or_else(|| refusal("direct-host Resume birth authority is absent"))?
            .authority_record_commitment
            .clone(),
        binding,
        descriptor,
        native_session_id,
        guard,
        workspace,
    };
    authenticated.verify_active(tx)?;
    Ok(authenticated)
}

fn direct_host_read<T: serde::de::DeserializeOwned + serde::Serialize>(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    reference: &AuthorityObjectRefV1,
) -> Result<T, BootstrapError> {
    let bytes = tx.read_authority_object(reference)?;
    canonical_json::from_slice(&bytes)
        .map_err(|_| refusal("decode direct-host Resume authority object"))
}

fn direct_host_payload(
    intent: &HostSessionTransitionIntentV3,
) -> HostSessionTransitionPayloadHashInputV1 {
    HostSessionTransitionPayloadHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        mode: intent.mode,
        authority_precondition: intent.authority_precondition.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token_ref: intent.target_participant_lease_token_ref.clone(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        resume_handle_ref: intent.resume_handle_ref.clone(),
        transition_input_ref: intent.transition_input_ref.clone(),
        post_turn_disposition: intent.post_turn_disposition,
        transport_payload_ref: intent.transport_payload_ref.clone(),
        issued_at: intent.issued_at.clone(),
        expires_at: intent.expires_at.clone(),
    }
}

fn direct_host_authority_hash_input(
    authority: &DurableSessionAuthorityV1,
) -> DurableSessionAuthorityHashInputV1 {
    DurableSessionAuthorityHashInputV1 {
        schema_version: authority.schema_version,
        orchestration_session_id: authority.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        authority_revision: authority.authority_revision,
        origin: authority.origin.clone(),
        authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: authority.active_authoritative_participant_id.clone(),
        workspace_binding: authority.workspace_binding.clone(),
        world_binding: authority.world_binding.clone(),
        host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
        retained_worker_refs: authority.retained_worker_refs.clone(),
        internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
        lifecycle_posture: authority.lifecycle_posture,
        current_policy_ref: authority.current_policy_ref.clone(),
        current_policy_revision: authority.current_policy_revision.clone(),
    }
}

/// Enrollment identity comes from the authenticated original Start application,
/// not caller-supplied session metadata or a failed world lookup.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostBirthBindingV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) birth_authority: DurableSessionAuthorityV1,
    pub(crate) birth_authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) start_intent_id: String,
    pub(crate) start_issuer_request_id: String,
    pub(crate) start_payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) start_application_result_ref: AuthorityObjectRefV1,
    pub(crate) start_run_id: String,
    pub(crate) public_transaction_id: String,
    pub(crate) public_request_key_sha256: String,
    pub(crate) public_prompt_sha256: String,
    pub(crate) public_backend_id: String,
    pub(crate) public_scope: String,
}

/// This is an enrollment proof only. It cannot publish canonical obligations.
pub(crate) struct AuthenticatedDirectHostBirthV1 {
    binding: DirectHostBirthBindingV1,
    may_initialize: bool,
    workspace: TrustedWorkspaceRoot,
}

impl AuthenticatedDirectHostBirthV1 {
    pub(crate) fn binding(&self) -> &DirectHostBirthBindingV1 {
        &self.binding
    }
    pub(crate) fn may_initialize(&self) -> bool {
        self.may_initialize
    }
    pub(crate) fn verify_workspace(&self) -> Result<(), BootstrapError> {
        self.workspace
            .revalidate()
            .map_err(|_| refusal("direct-host enrollment workspace identity changed"))
    }
}

impl super::facade::HostSessionAuthority {
    pub(crate) fn enroll_direct_host_start(
        &self,
        resolved: &super::facade::ResolvedCurrentAuthorityV1,
    ) -> Result<(), BootstrapError> {
        let home = self.trusted_root().identity().clone();
        super::store::with_locked_direct_host_inbox_transaction(
            Path::new(&home.physical_path),
            Some(&home),
            |tx| {
                let birth = authenticate_direct_host_birth_locked(tx, resolved)?;
                crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::enroll_direct_host_inbox_session(tx, &birth)
            },
        )
    }
}

fn authenticate_direct_host_birth_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    resolved: &super::facade::ResolvedCurrentAuthorityV1,
) -> Result<AuthenticatedDirectHostBirthV1, BootstrapError> {
    let Some(VersionedStateRoot::V3(root)) = tx.authority_root() else {
        return Err(refusal(
            "direct-host enrollment requires an exact public Start transaction",
        ));
    };
    let root = root.clone();
    let session_id = &resolved.authority.orchestration_session_id;
    if root.authority_store_id != resolved.observation.authority_store_id
        || root.bootstrap_home != resolved.observation.bootstrap_home
        || root.bootstrap_home != *tx.bootstrap_home()
        || !matches!(root.session_namespace_map.get(session_id),
            Some(SessionNamespaceRecordV1::Authority(a)) if a.as_ref() == &resolved.authority)
        || resolved.authority.world_binding.is_some()
        || resolved.caller.descriptor.execution_scope != AgentExecutionScopeV1::Host
    {
        return Err(refusal(
            "direct-host enrollment current authority identity changed",
        ));
    }
    let (binding, may_initialize) =
        authenticate_direct_host_birth_from_root(tx, &root, session_id)?;
    if binding.descriptor_ref != resolved.caller.descriptor_ref {
        return Err(refusal(
            "direct-host enrollment resolved descriptor is not the original Start",
        ));
    }
    let workspace =
        TrustedWorkspaceRoot::open_exact(&binding.birth_authority.workspace_binding.workspace_root)
            .map_err(|_| refusal("open exact direct-host enrollment workspace"))?;
    tx.verify()?;
    Ok(AuthenticatedDirectHostBirthV1 {
        binding,
        may_initialize,
        workspace,
    })
}

fn authenticate_direct_host_birth_from_root(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    root: &StateRootV3,
    session_id: &str,
) -> Result<(DirectHostBirthBindingV1, bool), BootstrapError> {
    let history = exact_v3_authority_history_with_reader(root, session_id, |reference| {
        tx.read_authority_object(reference)
            .map_err(|e| e.to_string())
    })
    .map_err(|_| refusal("direct-host enrollment exact birth history is unknown"))?;
    let first = history
        .get(&1)
        .ok_or_else(|| refusal("direct-host enrollment has no original birth authority"))?;
    let birth = &first.authority;
    if birth.world_binding.is_some()
        || birth.authority_revision != 1
        || birth.active_authoritative_participant_id.is_none()
        || birth.authoritative_participant_lineage.len() != 1
        || birth.authoritative_participant_lineage.first()
            != birth.active_authoritative_participant_id.as_ref()
        || history.values().any(|h| {
            h.authority.world_binding.is_some()
                || h.authority.workspace_binding != birth.workspace_binding
        })
    {
        return Err(refusal(
            "direct-host enrollment birth is not authenticated Host plus persisted None",
        ));
    }
    let DurableSessionAuthorityOriginV1::StartIntent {
        intent_id,
        issuer_request_id,
        payload_commitment,
    } = &birth.origin;
    let intent = root
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| refusal("direct-host enrollment original Start is absent"))?;
    let HostSessionTransitionIntentStateV2::Applied {
        authority_revision_before,
        authority_revision_after,
        authority_record_commitment,
        application_result_ref,
        active_authoritative_participant_id,
        ..
    } = &intent.state
    else {
        return Err(refusal("direct-host enrollment Start was not applied"));
    };
    let transactions = root
        .start_transaction_map
        .values()
        .filter(|t| t.orchestration_session_id == session_id)
        .collect::<Vec<_>>();
    let [transaction] = transactions.as_slice() else {
        return Err(refusal(
            "direct-host enrollment public Start transaction is missing or ambiguous",
        ));
    };
    let descriptor: AgentDescriptorHashInputV1 = direct_host_read(tx, &intent.descriptor_ref)?;
    let attach: HostAttachContractHashInputV1 =
        direct_host_read(tx, &intent.host_attach_contract_ref)?;
    let policy: PolicyObjectHashInputV1 = direct_host_read(tx, &attach.contract.policy_ref)?;
    if intent.mode != HostSessionTransitionModeV1::Start
        || intent.world_binding.is_some()
        || intent.source_authoritative_participant_id.is_some()
        || intent.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
        || intent.issuer_request_id != *issuer_request_id
        || intent.payload_commitment != *payload_commitment
        || authority_revision_before.is_some()
        || *authority_revision_after != 1
        || *authority_record_commitment != first.authority_record_commitment
        || birth.active_authoritative_participant_id.as_ref()
            != Some(active_authoritative_participant_id)
        || intent.orchestration_session_id != session_id
        || intent.shell_trace_session_id != birth.shell_trace_session_id
        || intent.workspace_binding != birth.workspace_binding
        || descriptor.descriptor.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.descriptor_ref != intent.descriptor_ref
        || birth.host_attach_contract_ref.as_ref() != Some(&intent.host_attach_contract_ref)
        || birth.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
        || birth.current_policy_revision.as_ref() != Some(&policy.policy_revision)
        || transaction.authority_store_id != root.authority_store_id
        || transaction.start_intent_id != *intent_id
        || transaction.start_issuer_request_id != *issuer_request_id
        || transaction.start_payload_commitment != *payload_commitment
        || transaction.start_application_result_ref != *application_result_ref
        || transaction.start_run_id != intent.run_id
        || transaction.start_authority_revision != 1
        || transaction.authoritative_participant_id != *active_authoritative_participant_id
        || transaction.shell_trace_session_id != birth.shell_trace_session_id
        || transaction.workspace_root != birth.workspace_binding.workspace_root.physical_path
        || transaction.backend_id != descriptor.descriptor.backend_id
        || transaction.protocol != descriptor.descriptor.protocol
        || transaction.world_id.is_some()
        || transaction.world_generation.is_some()
    {
        return Err(refusal(
            "direct-host enrollment Start application/public transaction joins disagree",
        ));
    }
    let binding = DirectHostBirthBindingV1 {
        schema_version: 1,
        authority_store_id: root.authority_store_id.clone(),
        bootstrap_home: root.bootstrap_home.clone(),
        birth_authority: birth.clone(),
        birth_authority_record_commitment: first.authority_record_commitment.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        start_intent_id: intent.intent_id.clone(),
        start_issuer_request_id: intent.issuer_request_id.clone(),
        start_payload_commitment: intent.payload_commitment.clone(),
        start_application_result_ref: application_result_ref.clone(),
        start_run_id: intent.run_id.clone(),
        public_transaction_id: transaction.transaction_id.clone(),
        public_request_key_sha256: transaction.request_key_sha256.clone(),
        public_prompt_sha256: transaction.prompt_sha256.clone(),
        public_backend_id: transaction.public_backend_id.clone(),
        public_scope: transaction.public_scope.clone(),
    };
    tx.verify()?;
    Ok((
        binding,
        matches!(
            transaction.state,
            StartTransactionStateV1::PromptNotSubmitted
        ),
    ))
}

/// Historical enrollment verification uses durable authority history and the
/// exact original application/public transaction, never a live helper guard.
pub(crate) fn verify_direct_host_birth_binding_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    binding: &DirectHostBirthBindingV1,
) -> Result<(), BootstrapError> {
    let Some(VersionedStateRoot::V3(root)) = tx.authority_root() else {
        return Err(refusal("direct-host genesis has no V3 authority history"));
    };
    let root = root.clone();
    if root.authority_store_id != binding.authority_store_id
        || root.bootstrap_home != binding.bootstrap_home
        || root.bootstrap_home != *tx.bootstrap_home()
    {
        return Err(refusal(
            "direct-host genesis physical authority identity changed",
        ));
    }
    let (expected, _) = authenticate_direct_host_birth_from_root(
        tx,
        &root,
        &binding.birth_authority.orchestration_session_id,
    )?;
    if expected != *binding {
        return Err(refusal(
            "direct-host genesis does not match its durable birth proof",
        ));
    }
    tx.verify()
}

/// Reconstruct a persisted publication's actor/input/history joins without
/// reading released prompt/token bytes or requiring a historical live helper.
/// This is a read verifier, never a producer capability. A settled publication
/// also supplies its committed cut so the ledger can prove head ancestry.
pub(crate) fn verify_direct_host_resume_publication_binding_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    b: &DirectHostResumeBindingV1,
) -> Result<Option<DirectHostCanonicalAttentionCutV1>, BootstrapError> {
    let Some(VersionedStateRoot::V3(root)) = tx.authority_root() else {
        return Err(refusal(
            "durable direct-host publication has no V3 authority",
        ));
    };
    let root = root.clone();
    let mut reader = |r: &AuthorityObjectRefV1| tx.read_authority_object(r);
    verify_direct_host_binding_with_reader(&root, b, &mut reader)?;
    let intent = &root.successor_transition_intent_map[&b.intent_id];
    let cut = match &intent.state {
        HostSessionTransitionIntentStateV3::Applied { post_turn, .. } => match post_turn.as_ref() {
            HostSessionPostTurnApplicationV2::Pending { .. } => None,
            HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } => {
                // Canonical publication already belongs to the ledger before
                // HSA settlement. A typed terminal has no resumable cut; its
                // authenticated immutable producer/protocol joins survive,
                // while the ledger independently verifies original provenance.
                verify_direct_host_completed_join(&root, intent, completion_ref, &mut reader)?
                    .attention_cut
            }
            _ => {
                return Err(refusal(
                    "durable direct-host publication phase is unresolved",
                ))
            }
        },
        _ => return Err(refusal("durable direct-host publication is not Applied")),
    };
    tx.verify()?;
    Ok(cut)
}
/// Identity of a native Resume exchange observed by the actual helper. This is
/// evidence, not a producer capability; the entrypoint authenticates the plan,
/// Accepted input, inherited guard, descriptor and persisted consumed handle.
#[derive(Clone, Debug)]
pub(crate) struct DirectHostResumeActiveNativeV1 {
    pub(crate) schema_version: u32,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) native_session_id: String,
    pub(crate) thread_id: String,
    pub(crate) turn_id: String,
    pub(crate) sequence: DirectHostNativeSequenceV1,
}

fn verify_direct_host_active_native(
    native: &DirectHostResumeActiveNativeV1,
    descriptor: &AgentDescriptorV1,
    expected_native_session: &str,
) -> Result<(), BootstrapError> {
    if native.schema_version != 1
        || descriptor.execution_scope != AgentExecutionScopeV1::Host
        || native.backend_id != descriptor.backend_id
        || native.protocol != descriptor.protocol
        || native.native_session_id != expected_native_session
        || native.thread_id != expected_native_session
        || native.native_session_id.trim().is_empty()
        || native.turn_id.trim().is_empty()
    {
        return Err(refusal("active direct-host native identity does not join"));
    }
    let joins = match (&descriptor.backend_kind, &native.sequence) {
        (
            RuntimeBackendKindV1::Codex,
            DirectHostNativeSequenceV1::Codex {
                resumed_thread_id,
                started_turn_id,
            },
        ) => resumed_thread_id == &native.thread_id && started_turn_id == &native.turn_id,
        (
            RuntimeBackendKindV1::ClaudeCode,
            DirectHostNativeSequenceV1::ClaudeCode {
                initialized_session_id,
            },
        ) => initialized_session_id == &native.thread_id && native.turn_id == native.thread_id,
        _ => false,
    };
    if !joins {
        return Err(refusal("active direct-host native sequence does not join"));
    }
    Ok(())
}

impl super::facade::HostSessionAuthority {
    /// Publish durable typed local requests only during the actual Accepted
    /// native Resume exchange. Start and unbound/unenrolled histories cannot
    /// acquire canonical meaning from an otherwise matching native observation.
    pub(crate) fn materialize_direct_host_resume_active(
        &self,
        plan: &AuthorityManagedSuccessorLaunchPlanV1,
        native: &DirectHostResumeActiveNativeV1,
    ) -> Result<DirectHostCanonicalAttentionCutV1, BootstrapError> {
        self.trusted_root()
            .revalidate()
            .map_err(|_| refusal("revalidate active direct-host authority root"))?;
        let home = self.trusted_root().identity().clone();
        super::store::with_locked_direct_host_inbox_transaction(
            Path::new(&home.physical_path),
            Some(&home),
            |tx| {
                let producer = authenticate_direct_host_resume_locked(tx, plan)?;
                verify_direct_host_active_native(
                    native,
                    producer.descriptor(),
                    producer.native_session_id(),
                )?;
                crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::materialize_direct_host_inbox_locked(tx, &producer)
            },
        )
    }
}

#[cfg(test)]
mod direct_host_active_native_tests {
    use super::*;

    fn native_pair() -> (AgentDescriptorV1, DirectHostResumeActiveNativeV1) {
        (
            AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "agent".into(),
                backend_id: "codex-local".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "codex-jsonl".into(),
                execution_scope: AgentExecutionScopeV1::Host,
                binary_path: "/native".into(),
            },
            DirectHostResumeActiveNativeV1 {
                schema_version: 1,
                backend_id: "codex-local".into(),
                protocol: "codex-jsonl".into(),
                native_session_id: "thread".into(),
                thread_id: "thread".into(),
                turn_id: "turn".into(),
                sequence: DirectHostNativeSequenceV1::Codex {
                    resumed_thread_id: "thread".into(),
                    started_turn_id: "turn".into(),
                },
            },
        )
    }

    #[test]
    fn test_direct_host_resume_active_native_requires_exact_consumed_handle_and_descriptor() {
        let (descriptor, native) = native_pair();
        assert!(verify_direct_host_active_native(&native, &descriptor, "thread").is_ok());
        assert!(verify_direct_host_active_native(&native, &descriptor, "another-thread").is_err());
        let mut wrong = descriptor.clone();
        wrong.backend_id = "other-backend".into();
        assert!(verify_direct_host_active_native(&native, &wrong, "thread").is_err());
        wrong = descriptor.clone();
        wrong.execution_scope = AgentExecutionScopeV1::World;
        assert!(verify_direct_host_active_native(&native, &wrong, "thread").is_err());
    }

    #[test]
    fn test_direct_host_resume_active_native_requires_family_specific_sequence() {
        let (mut descriptor, mut native) = native_pair();
        native.sequence = DirectHostNativeSequenceV1::Codex {
            resumed_thread_id: "thread".into(),
            started_turn_id: "other-turn".into(),
        };
        assert!(verify_direct_host_active_native(&native, &descriptor, "thread").is_err());
        native.sequence = DirectHostNativeSequenceV1::ClaudeCode {
            initialized_session_id: "thread".into(),
        };
        assert!(verify_direct_host_active_native(&native, &descriptor, "thread").is_err());
        descriptor.backend_kind = RuntimeBackendKindV1::ClaudeCode;
        native.turn_id = "thread".into();
        assert!(verify_direct_host_active_native(&native, &descriptor, "thread").is_ok());
        native.sequence = DirectHostNativeSequenceV1::ClaudeCode {
            initialized_session_id: "other-thread".into(),
        };
        assert!(verify_direct_host_active_native(&native, &descriptor, "thread").is_err());
    }
}
struct DirectHostPersistedNativeContextV1 {
    descriptor: AgentDescriptorV1,
    native_session_id: String,
}

fn direct_host_public_read<T>(
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
    reference: &AuthorityObjectRefV1,
) -> Result<T, BootstrapError>
where
    T: serde::de::DeserializeOwned + serde::Serialize + super::validation::ValidatedCanonicalV1,
{
    let value: T = canonical_json::from_slice(&reader(reference)?)
        .map_err(|_| refusal("decode strict direct-host public object"))?;
    value
        .validate()
        .map_err(|_| refusal("validate strict direct-host public object"))?;
    Ok(value)
}

/// Durable actor/input/history correspondence without any erased secret, live
/// old helper guard or current external inbox read. Caller supplies a retained,
/// commitment-verifying public-object reader rooted in the exact physical home.
fn verify_direct_host_binding_with_reader(
    root: &StateRootV3,
    b: &DirectHostResumeBindingV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<DirectHostPersistedNativeContextV1, BootstrapError> {
    let intent = verify_direct_host_binding_prelude(root, b, reader)?;
    let history = exact_v3_authority_history_with_reader(root, &b.orchestration_session_id, |r| {
        reader(r).map_err(|e| e.to_string())
    })
    .map_err(|_| refusal("durable direct-host publication ancestry is unknown"))?;
    verify_direct_host_binding_after_prelude(root, b, intent, &history, reader)
}

/// Leaf verification: the caller has already reconstructed this exact prefix.
/// This function must never invoke exact-history reconstruction.
fn verify_direct_host_binding_against_prefix(
    root: &StateRootV3,
    b: &DirectHostResumeBindingV1,
    history: &std::collections::BTreeMap<u64, super::facade::ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<DirectHostPersistedNativeContextV1, BootstrapError> {
    let intent = verify_direct_host_binding_prelude(root, b, reader)?;
    verify_direct_host_binding_after_prelude(root, b, intent, history, reader)
}

/// Preserve the original binding validation and immutable object read ordering
/// before either reconstructing history or verifying an authenticated prefix.
fn verify_direct_host_binding_prelude<'root>(
    root: &'root StateRootV3,
    b: &DirectHostResumeBindingV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<&'root HostSessionTransitionIntentV3, BootstrapError> {
    b.validate()
        .map_err(|_| refusal("invalid durable direct-host publication binding"))?;
    let intent = root
        .successor_transition_intent_map
        .get(&b.intent_id)
        .ok_or_else(|| refusal("durable direct-host publication intent is absent"))?;
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        active_authoritative_participant_id,
        authority_record_commitment,
        application_result_ref,
        resulting_posture,
        applied_at,
        ..
    } = &intent.state
    else {
        return Err(refusal("durable direct-host publication was not Applied"));
    };
    let HostSessionTransitionInputHandoffV1::Accepted {
        input_ref,
        run_id,
        acceptance_ref,
        accepted_at,
    } = &intent.input_handoff
    else {
        return Err(refusal(
            "durable direct-host publication input was not Accepted",
        ));
    };
    if intent.schema_version != 3
        || accepted_at.as_str() < applied_at.as_str()
        || applied_at.as_str() < intent.issued_at.as_str()
        || root.authority_store_id != b.authority_store_id
        || root.bootstrap_home != b.bootstrap_home
        || intent.world_binding.is_some()
        || intent.mode != HostSessionTransitionModeV1::ResumeOneTurn
        || intent.workspace_binding != b.workspace_binding
        || intent.orchestration_session_id != b.orchestration_session_id
        || intent.shell_trace_session_id != b.shell_trace_session_id
        || intent.target_authoritative_participant_id != b.authoritative_participant_id
        || intent.resulting_authoritative_lineage != b.authoritative_lineage
        || intent.issuer_request_id != b.issuer_request_id
        || intent.payload_commitment != b.payload_commitment
        || intent.payload_commitment
            != (AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&direct_host_payload(intent))
                    .map_err(|_| refusal("hash durable Resume payload"))?,
            })
        || intent.run_id != b.run_id
        || run_id != &b.run_id
        || intent.descriptor_ref != b.descriptor_ref
        || intent.host_attach_contract_ref != b.host_attach_contract_ref
        || intent.resume_handle_ref.as_ref() != Some(&b.consumed_resume_handle_ref)
        || intent.transition_input_ref.as_ref() != Some(&b.transition_input_ref)
        || input_ref != &b.transition_input_ref
        || Some(acceptance_ref) != b.input_acceptance_ref.as_ref()
        || claim_id != &b.claim_id
        || claimant_attempt_id != &b.claimant_attempt_id
        || authority_revision_after != &b.authority_revision_observed
        || authority_record_commitment != &b.authority_record_commitment
        || application_result_ref != &b.initial_application_result_ref
        || active_authoritative_participant_id != &b.authoritative_participant_id
        || *resulting_posture != HostSessionPostureV1::ActiveAttached
        || intent.intent_revision < b.intent_revision_observed
    {
        return Err(refusal(
            "durable direct-host publication immutable identity disagrees",
        ));
    }
    let acceptance: InputAcceptanceHashInputV1 = direct_host_public_read(reader, acceptance_ref)?;
    if acceptance.intent_id != b.intent_id
        || acceptance.run_id != b.run_id
        || acceptance.input_ref != b.transition_input_ref
        || acceptance.accepting_participant_id != b.authoritative_participant_id
        || acceptance.accepted_at != *accepted_at
    {
        return Err(refusal(
            "durable direct-host publication acceptance does not join",
        ));
    }
    let application: ApplicationResultHashInputV1 =
        direct_host_public_read(reader, application_result_ref)?;
    if application.intent_id != b.intent_id
        || application.mode != intent.mode
        || application.run_id != b.run_id
        || application.applied_at != *applied_at
        || application.phase
            != (ApplicationResultPhaseV1::InitialTransition {
                authority_revision_before: *authority_revision_before,
                authority_revision_after: *authority_revision_after,
                active_authoritative_participant_id: b.authoritative_participant_id.clone(),
                resulting_posture: *resulting_posture,
                authority_record_commitment: b.authority_record_commitment.clone(),
                post_turn_pending_run_id: Some(b.run_id.clone()),
            })
    {
        return Err(refusal(
            "durable direct-host publication application does not join",
        ));
    }
    Ok(intent)
}

/// Shared history/native tail after the immutable prelude has authenticated the
/// borrowed intent. The root and intent cannot change through these references.
fn verify_direct_host_binding_after_prelude(
    root: &StateRootV3,
    b: &DirectHostResumeBindingV1,
    intent: &HostSessionTransitionIntentV3,
    history: &std::collections::BTreeMap<u64, super::facade::ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<DirectHostPersistedNativeContextV1, BootstrapError> {
    let HostSessionTransitionIntentStateV3::Applied {
        authority_revision_before,
        post_turn,
        applied_at,
        ..
    } = &intent.state
    else {
        return Err(refusal("durable direct-host publication was not Applied"));
    };
    verify_direct_host_public_birth_with_reader(
        root,
        &b.orchestration_session_id,
        history,
        reader,
    )?;
    let applied = history
        .get(&b.authority_revision_observed)
        .ok_or_else(|| refusal("durable direct-host publication applied authority is absent"))?;
    let prefix = history
        .range(..=b.authority_revision_observed)
        .map(|(rev, a)| {
            (
                *rev,
                a.authority_record_commitment.clone(),
                a.authoritative_lineage_commitment.clone(),
            )
        })
        .collect::<Vec<_>>();
    if history.values().any(|a| {
        a.authority.world_binding.is_some()
            || a.authority.workspace_binding != b.workspace_binding
            || a.authority.shell_trace_session_id != b.shell_trace_session_id
    }) || applied.authority_record_commitment != b.authority_record_commitment
        || applied.authority.authoritative_participant_lineage != b.authoritative_lineage
        || applied
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(&b.authoritative_participant_id)
        || applied.authority.current_policy_ref != b.policy_ref
        || applied.authority.current_policy_revision != b.policy_revision
        || applied.authority.host_attach_contract_ref.as_ref() != Some(&b.host_attach_contract_ref)
        || direct_host_inbox_commitment(
            "substrate.direct-host-resume.authority-prefix.v1",
            &prefix,
        )? != b.authority_history_commitment
    {
        return Err(refusal(
            "durable direct-host publication full authority prefix disagrees",
        ));
    }
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment: source_commitment,
        active_authoritative_participant_id: source_actor,
        authoritative_lineage_commitment,
        lifecycle_posture,
    } = &intent.authority_precondition
    else {
        return Err(refusal("durable Resume has no exact source precondition"));
    };
    let source = history
        .get(authority_revision)
        .ok_or_else(|| refusal("durable Resume source ancestry is absent"))?;
    if authority_revision_before.as_ref() != Some(authority_revision)
        || source.authority_record_commitment != *source_commitment
        || source.authoritative_lineage_commitment != *authoritative_lineage_commitment
        || source
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(source_actor)
        || intent.source_authoritative_participant_id.as_ref() != Some(source_actor)
        || source.authority.lifecycle_posture != *lifecycle_posture
        || !source
            .authority
            .internal_resume_handle_refs
            .contains(&b.consumed_resume_handle_ref)
    {
        return Err(refusal(
            "durable Resume source/consumed handle does not join",
        ));
    }
    let descriptor: AgentDescriptorHashInputV1 =
        direct_host_public_read(reader, &b.descriptor_ref)?;
    let attach: HostAttachContractHashInputV1 =
        direct_host_public_read(reader, &b.host_attach_contract_ref)?;
    let policy: PolicyObjectHashInputV1 =
        direct_host_public_read(reader, &attach.contract.policy_ref)?;
    if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.backend_id != descriptor.descriptor.backend_id
        || attach.contract.protocol != descriptor.descriptor.protocol
        || attach.contract.descriptor_ref != b.descriptor_ref
        || b.policy_ref.as_ref() != Some(&attach.contract.policy_ref)
        || b.policy_revision.as_ref() != Some(&policy.policy_revision)
    {
        return Err(refusal(
            "durable direct-host descriptor/attach/policy joins disagree",
        ));
    }

    let issuer = root
        .successor_issuer_request_index
        .get(&b.issuer_request_id)
        .ok_or_else(|| refusal("durable direct-host issuer index is absent"))?;
    let journal = root
        .successor_application_journal
        .get(&b.intent_id)
        .ok_or_else(|| refusal("durable direct-host initial journal is absent"))?;
    if issuer.schema_version != 1
        || issuer.issuer_request_id != b.issuer_request_id
        || issuer.intent_id != b.intent_id
        || issuer.orchestration_session_id != b.orchestration_session_id
        || issuer.payload_commitment != b.payload_commitment
        || journal.schema_version != 3
        || journal.intent_id != b.intent_id
        || journal.startup_terminal_application.is_some()
        || journal.initial_application.application_result_ref != b.initial_application_result_ref
        || journal.initial_application.authority_record_commitment != b.authority_record_commitment
        || journal.initial_application.authority_revision_before != *authority_revision_before
        || journal.initial_application.authority_revision_after != b.authority_revision_observed
        || journal.initial_application.applied_at != *applied_at
    {
        return Err(refusal(
            "durable direct-host issuer/application journal disagrees",
        ));
    }
    let (handle_session, handle_participant, handle_backend, handle_protocol, native_session_id) =
        match b.consumed_resume_handle_ref.schema_version {
            1 => {
                let h: ResumeHandleHashInputV1 =
                    direct_host_public_read(reader, &b.consumed_resume_handle_ref)?;
                (
                    h.orchestration_session_id,
                    h.participant_id,
                    h.backend_id,
                    h.protocol,
                    h.internal_uaa_session_id,
                )
            }
            2 => {
                let h: StartContinuationHandleHashInputV2 =
                    direct_host_public_read(reader, &b.consumed_resume_handle_ref)?;
                if h.authority_store_id != root.authority_store_id {
                    return Err(refusal(
                        "durable direct-host consumed native handle store changed",
                    ));
                }
                (
                    h.orchestration_session_id,
                    h.participant_id,
                    h.backend_id,
                    h.protocol,
                    h.internal_uaa_session_id,
                )
            }
            _ => {
                return Err(refusal(
                    "unsupported durable direct-host native handle version",
                ))
            }
        };
    if handle_session != b.orchestration_session_id
        || !source
            .authority
            .authoritative_participant_lineage
            .contains(&handle_participant)
        || handle_backend != descriptor.descriptor.backend_id
        || handle_protocol != descriptor.descriptor.protocol
        || native_session_id.trim().is_empty()
    {
        return Err(refusal(
            "durable direct-host native handle identity does not join",
        ));
    }
    match post_turn.as_ref() {
        HostSessionPostTurnApplicationV2::Pending {
            expected_run_id,
            expected_authority_revision,
        } if expected_run_id == &b.run_id
            && *expected_authority_revision == b.authority_revision_observed
            && history.last_key_value().map(|(r, _)| *r) == Some(b.authority_revision_observed)
            && intent.intent_revision == b.intent_revision_observed => {}
        HostSessionPostTurnApplicationV2::Applied { completion_ref, .. }
            if completion_ref.object_kind == AuthorityObjectKindV1::PostTurnCompletion
                && matches!(completion_ref.schema_version, 2 | 3)
                && Some(intent.intent_revision) == b.intent_revision_observed.checked_add(1) => {}
        _ => {
            return Err(refusal(
                "durable direct-host publication phase does not join",
            ))
        }
    }
    Ok(DirectHostPersistedNativeContextV1 {
        descriptor: descriptor.descriptor,
        native_session_id,
    })
}

fn verify_direct_host_terminal_native(
    native: &DirectHostNativeTerminalEvidenceV1,
    kind: &HostPostTurnProtocolEventKindV1,
    descriptor: &AgentDescriptorV1,
    expected_native_session: &str,
) -> Result<(), BootstrapError> {
    native
        .validate()
        .map_err(|_| refusal("invalid captured direct-host native terminal"))?;
    verify_direct_host_active_native(
        &DirectHostResumeActiveNativeV1 {
            schema_version: native.schema_version,
            backend_id: native.backend_id.clone(),
            protocol: native.protocol.clone(),
            native_session_id: native.native_session_id.clone(),
            thread_id: native.thread_id.clone(),
            turn_id: native.turn_id.clone(),
            sequence: native.sequence.clone(),
        },
        descriptor,
        expected_native_session,
    )?;
    let failed = matches!(
        native.terminal_event_kind.as_str(),
        "turn.failed" | "result.error"
    );
    let accepted_kind = match kind {
        HostPostTurnProtocolEventKindV1::ResumableClean
        | HostPostTurnProtocolEventKindV1::TerminalClean => !failed,
        HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
        } => failed,
        _ => false,
    };
    if !accepted_kind {
        return Err(refusal(
            "direct-host captured terminal is not an accepted target outcome",
        ));
    }
    Ok(())
}

fn direct_host_terminal_event_id(
    binding: &DirectHostResumeBindingV1,
    native: &DirectHostNativeTerminalEvidenceV1,
    kind: &HostPostTurnProtocolEventKindV1,
) -> Result<String, BootstrapError> {
    match direct_host_inbox_commitment(
        "substrate.direct-host.native-terminal-event.v1",
        &(binding, native, kind),
    )? {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            Ok(format!("direct-host-native-{digest_hex}"))
        }
        _ => Err(refusal("hash direct-host native terminal identity")),
    }
}

/// Common decoded fields for immutable V2/V3 completion joins. This is not a
/// canonical codec and cannot be serialized back into either stored version.
pub(super) struct VerifiedDirectHostCompletionV1 {
    pub(super) binding: DirectHostResumeBindingV1,
    pub(super) protocol_event_ref: AuthorityObjectRefV1,
    outcome: PostTurnCompletionOutcomeV1,
    attention_cut: Option<DirectHostCanonicalAttentionCutV1>,
    completed_at: TimestampV1,
    produced_resume_handle_ref: Option<AuthorityObjectRefV1>,
}

fn read_direct_host_completion(
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
    reference: &AuthorityObjectRefV1,
) -> Result<VerifiedDirectHostCompletionV1, BootstrapError> {
    if reference.object_kind != AuthorityObjectKindV1::PostTurnCompletion {
        return Err(refusal("direct-host join requires a completion"));
    }
    match reference.schema_version {
        2 => {
            let c: PostTurnCompletionHashInputV2 = direct_host_public_read(reader, reference)?;
            Ok(VerifiedDirectHostCompletionV1 {
                binding: c.binding,
                protocol_event_ref: c.protocol_event_ref,
                outcome: c.outcome,
                attention_cut: c.attention_cut,
                completed_at: c.completed_at,
                produced_resume_handle_ref: None,
            })
        }
        3 => {
            let c: PostTurnCompletionHashInputV3 = direct_host_public_read(reader, reference)?;
            Ok(VerifiedDirectHostCompletionV1 {
                binding: c.binding,
                protocol_event_ref: c.protocol_event_ref,
                outcome: c.outcome,
                attention_cut: c.attention_cut,
                completed_at: c.completed_at,
                produced_resume_handle_ref: c.produced_resume_handle_ref,
            })
        }
        _ => Err(refusal("unsupported direct-host completion version")),
    }
}

/// Authenticate the new handle's exact producer and native identity. Lineage
/// membership is insufficient. No history or completed-join call occurs here.
fn verify_direct_host_produced_handle(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    completion_ref: &AuthorityObjectRefV1,
    completion: &VerifiedDirectHostCompletionV1,
    native: &DirectHostNativeTerminalEvidenceV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<(), BootstrapError> {
    let Some(reference) = &completion.produced_resume_handle_ref else {
        return Ok(());
    };
    let handle: ResumeHandleHashInputV1 = direct_host_public_read(reader, reference)?;
    let b = &completion.binding;
    if reference.object_kind != AuthorityObjectKindV1::ResumeHandle
        || reference.schema_version != 1
        || handle.orchestration_session_id != b.orchestration_session_id
        || handle.participant_id != b.authoritative_participant_id
        || handle.participant_id != intent.target_authoritative_participant_id
        || handle.backend_id != native.backend_id
        || handle.protocol != native.protocol
        || handle.internal_uaa_session_id != native.native_session_id
    {
        return Err(refusal(
            "produced direct-host handle identity disagrees with target/native terminal",
        ));
    }
    let mut producers = 0;
    for parent in root.successor_transition_intent_map.values() {
        if let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &parent.state {
            if let HostSessionPostTurnApplicationV2::Applied {
                completion_ref: other_ref,
                ..
            } = post_turn.as_ref()
            {
                if other_ref.object_kind == AuthorityObjectKindV1::PostTurnCompletion
                    && other_ref.schema_version == 3
                {
                    let other: PostTurnCompletionHashInputV3 =
                        direct_host_public_read(reader, other_ref)?;
                    if other.produced_resume_handle_ref.as_ref() == Some(reference) {
                        producers += 1;
                        if other_ref.as_ref() != completion_ref || parent != intent {
                            return Err(refusal(
                                "direct-host handle has conflicting producing completion/intent",
                            ));
                        }
                    }
                }
            }
        }
    }
    if producers != 1 {
        return Err(refusal(
            "direct-host handle producer is missing or ambiguous",
        ));
    }
    Ok(())
}

/// The only V3 post-turn history leaf. Accept an already authenticated prefix;
/// never call the full completed join or reconstruct history from this leaf.
pub(super) fn verify_direct_host_completion_v3_edge(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    completion_ref: &AuthorityObjectRefV1,
    prefix: &std::collections::BTreeMap<u64, super::facade::ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<Option<AuthorityObjectRefV1>, BootstrapError> {
    if completion_ref.schema_version != 3 {
        return Err(refusal(
            "new direct-host history edge requires completion V3",
        ));
    }
    let c = read_direct_host_completion(reader, completion_ref)?;
    let b = &c.binding;
    let context = verify_direct_host_binding_against_prefix(root, b, prefix, reader)?;
    let event: PostTurnProtocolEventHashInputV2 =
        direct_host_public_read(reader, &c.protocol_event_ref)?;
    let native = event
        .native_terminal
        .as_ref()
        .ok_or_else(|| refusal("V3 history edge has no native terminal"))?;
    verify_direct_host_terminal_native(
        native,
        &event.kind,
        &context.descriptor,
        &context.native_session_id,
    )?;
    let expected_outcome = match event.kind {
        HostPostTurnProtocolEventKindV1::ResumableClean => {
            PostTurnCompletionOutcomeV1::ResumableClean
        }
        HostPostTurnProtocolEventKindV1::TerminalClean => {
            PostTurnCompletionOutcomeV1::TerminalClean
        }
        HostPostTurnProtocolEventKindV1::TerminalFailure { .. } => {
            PostTurnCompletionOutcomeV1::TerminalFailure
        }
    };
    let expected_posture = match &c.attention_cut {
        Some(cut) if cut.unresolved_attention.is_empty() => HostSessionPostureV1::ParkedResumable,
        Some(_) => HostSessionPostureV1::AwaitingAttention,
        None => HostSessionPostureV1::Terminal,
    };
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
        return Err(refusal("V3 completion history parent was not Applied"));
    };
    let HostSessionPostTurnApplicationV2::Applied {
        completion_ref: stored,
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        applied_at,
        ..
    } = post_turn.as_ref()
    else {
        return Err(refusal("V3 completion history has no post-turn edge"));
    };
    let accepted_at = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { accepted_at, .. } => accepted_at,
        _ => return Err(refusal("V3 completion history input was not Accepted")),
    };
    if root.successor_transition_intent_map.get(&b.intent_id) != Some(intent)
        || prefix.last_key_value().map(|(rev, _)| *rev) != Some(b.authority_revision_observed)
        || stored.as_ref() != completion_ref
        || *authority_revision_before != b.authority_revision_observed
        || authority_revision_before.checked_add(1) != Some(*authority_revision_after)
        || c.binding != event.binding
        || c.outcome != expected_outcome
        || *resulting_posture != expected_posture
        || event.event_id != direct_host_terminal_event_id(b, native, &event.kind)?
        || event.emitted_at != c.completed_at
        || *applied_at != c.completed_at
        || event.emitted_at.as_str() < accepted_at.as_str()
    {
        return Err(refusal(
            "V3 completion history target/event/prefix correspondence disagrees",
        ));
    }
    verify_direct_host_produced_handle(root, intent, completion_ref, &c, native, reader)?;
    Ok(c.produced_resume_handle_ref)
}

/// Full immutable V2/V3 completion/native/application/release join. This never
/// interprets current inbox/source state and never qualifies a world V1 event.
pub(super) fn verify_direct_host_completed_join(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    completion_ref: &AuthorityObjectRefV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<VerifiedDirectHostCompletionV1, BootstrapError> {
    let completion = read_direct_host_completion(reader, completion_ref)?;
    let b = &completion.binding;
    if b.intent_id != intent.intent_id
        || root.successor_transition_intent_map.get(&b.intent_id) != Some(intent)
    {
        return Err(refusal("direct-host completion belongs to another intent"));
    }
    let context = verify_direct_host_binding_with_reader(root, b, reader)?;
    let event: PostTurnProtocolEventHashInputV2 =
        direct_host_public_read(reader, &completion.protocol_event_ref)?;
    let native = event
        .native_terminal
        .as_ref()
        .ok_or_else(|| refusal("committed native terminal is absent"))?;
    verify_direct_host_terminal_native(
        native,
        &event.kind,
        &context.descriptor,
        &context.native_session_id,
    )?;
    let outcome = match event.kind {
        HostPostTurnProtocolEventKindV1::ResumableClean => {
            PostTurnCompletionOutcomeV1::ResumableClean
        }
        HostPostTurnProtocolEventKindV1::TerminalClean => {
            PostTurnCompletionOutcomeV1::TerminalClean
        }
        HostPostTurnProtocolEventKindV1::TerminalFailure { .. } => {
            PostTurnCompletionOutcomeV1::TerminalFailure
        }
    };
    let expected_posture = match completion.outcome {
        PostTurnCompletionOutcomeV1::ResumableClean => {
            let cut = completion
                .attention_cut
                .as_ref()
                .ok_or_else(|| refusal("resumable completion lacks exact cut"))?;
            if cut.unresolved_attention.is_empty() {
                HostSessionPostureV1::ParkedResumable
            } else {
                HostSessionPostureV1::AwaitingAttention
            }
        }
        PostTurnCompletionOutcomeV1::TerminalClean
        | PostTurnCompletionOutcomeV1::TerminalFailure => HostSessionPostureV1::Terminal,
    };
    let HostSessionTransitionIntentStateV3::Applied {
        application_result_ref: initial_application_ref,
        post_turn,
        ..
    } = &intent.state
    else {
        return Err(refusal("completed direct-host intent is not Applied"));
    };
    let HostSessionPostTurnApplicationV2::Applied {
        completion_ref: stored_completion,
        obligation_snapshot_ref,
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        application_result_ref,
        applied_at,
    } = post_turn.as_ref()
    else {
        return Err(refusal("direct-host post-turn is not committed"));
    };
    let accepted_at = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { accepted_at, .. } => accepted_at,
        _ => {
            return Err(refusal(
                "committed direct-host completion has no Accepted input",
            ))
        }
    };
    if completion.binding != event.binding
        || completion.outcome != outcome
        || event.event_id != direct_host_terminal_event_id(b, native, &event.kind)?
        || event.emitted_at != completion.completed_at
        || event.emitted_at.as_str() < accepted_at.as_str()
        || stored_completion.as_ref() != completion_ref
        || obligation_snapshot_ref.is_some()
        || *authority_revision_before != b.authority_revision_observed
        || Some(*authority_revision_after) != authority_revision_before.checked_add(1)
        || *resulting_posture != expected_posture
        || Some(intent.intent_revision) != b.intent_revision_observed.checked_add(1)
        || intent.updated_at != *applied_at
        || applied_at != &completion.completed_at
    {
        return Err(refusal(
            "direct-host immutable completion/native/posture correspondence disagrees",
        ));
    }
    let journal = root
        .successor_application_journal
        .get(&b.intent_id)
        .and_then(|j| j.post_turn_application.as_ref())
        .ok_or_else(|| refusal("direct-host committed post-turn journal is absent"))?;
    if journal.completion_ref != *completion_ref
        || journal.obligation_snapshot_ref.is_some()
        || journal.authority_revision_before != *authority_revision_before
        || journal.authority_revision_after != *authority_revision_after
        || journal.application_result_ref != **application_result_ref
        || journal.applied_at != *applied_at
    {
        return Err(refusal("direct-host committed journal and intent disagree"));
    }
    let application: ApplicationResultHashInputV1 =
        direct_host_public_read(reader, application_result_ref)?;
    if application.intent_id != b.intent_id
        || application.mode != HostSessionTransitionModeV1::ResumeOneTurn
        || application.run_id != b.run_id
        || application.applied_at != *applied_at
        || application.phase
            != (ApplicationResultPhaseV1::PostTurn {
                completion_ref: completion_ref.clone(),
                authority_revision_before: *authority_revision_before,
                authority_revision_after: *authority_revision_after,
                active_authoritative_participant_id: b.authoritative_participant_id.clone(),
                resulting_posture: *resulting_posture,
                authority_record_commitment: journal.authority_record_commitment.clone(),
            })
    {
        return Err(refusal("direct-host post-turn application does not join"));
    }
    let terminal_ref = match &intent.transport_payload_state {
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref,
        }
        | HostSessionTransitionTransportPayloadStateV1::Released {
            terminal_handoff_ref,
            ..
        } => terminal_handoff_ref,
        _ => {
            return Err(refusal(
                "direct-host completed result has no committed handoff",
            ))
        }
    };
    let terminal: TerminalHandoffHashInputV1 = direct_host_public_read(reader, terminal_ref)?;
    let transport = root
        .object_index
        .get(&intent.transport_payload_ref.ref_id)
        .ok_or_else(|| refusal("direct-host transport index is absent"))?;
    let transport_matches = match (&intent.transport_payload_state, &transport.storage_state) {
        (
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: a,
            },
            AuthorityObjectStorageStateV1::ReleaseEligible {
                terminal_handoff_ref: b,
            },
        ) => a == b,
        (
            HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref: a,
                released_at: at,
            },
            AuthorityObjectStorageStateV1::Released {
                terminal_handoff_ref: b,
                released_at: bt,
            },
        ) => a == b && at == bt,
        _ => false,
    };
    if terminal_ref.object_kind != AuthorityObjectKindV1::TerminalHandoff
        || terminal_ref.schema_version != 1
        || terminal.intent_id != b.intent_id
        || terminal.run_id != b.run_id
        || terminal.payload_commitment != b.payload_commitment
        || terminal.terminal_state != TerminalHandoffStateV1::Applied
        || terminal.application_result_ref.as_ref() != Some(initial_application_ref)
        || terminal.input_acceptance_ref != b.input_acceptance_ref
        || terminal.post_turn_completion_ref.as_ref() != Some(completion_ref)
        || terminal.post_turn_application_result_ref.as_ref()
            != Some(application_result_ref.as_ref())
        || terminal.recorded_at != *applied_at
        || !transport_matches
        || transport.schema_version != 1
        || transport.ref_id != intent.transport_payload_ref.ref_id
        || transport.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
        || transport.object_schema_version != intent.transport_payload_ref.schema_version
    {
        return Err(refusal(
            "direct-host committed handoff/transport correspondence disagrees",
        ));
    }
    verify_direct_host_produced_handle(root, intent, completion_ref, &completion, native, reader)?;
    Ok(completion)
}
fn verify_direct_host_public_birth_with_reader(
    root: &StateRootV3,
    session_id: &str,
    history: &std::collections::BTreeMap<u64, super::facade::ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<(), BootstrapError> {
    let first = history
        .get(&1)
        .ok_or_else(|| refusal("direct-host enrollment has no original birth authority"))?;
    let birth = &first.authority;
    if birth.world_binding.is_some()
        || birth.authority_revision != 1
        || birth.active_authoritative_participant_id.is_none()
        || birth.authoritative_participant_lineage.len() != 1
        || birth.authoritative_participant_lineage.first()
            != birth.active_authoritative_participant_id.as_ref()
        || history.values().any(|h| {
            h.authority.world_binding.is_some()
                || h.authority.workspace_binding != birth.workspace_binding
        })
    {
        return Err(refusal(
            "direct-host enrollment birth is not authenticated Host plus persisted None",
        ));
    }
    let DurableSessionAuthorityOriginV1::StartIntent {
        intent_id,
        issuer_request_id,
        payload_commitment,
    } = &birth.origin;
    let intent = root
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| refusal("direct-host enrollment original Start is absent"))?;
    let HostSessionTransitionIntentStateV2::Applied {
        authority_revision_before,
        authority_revision_after,
        authority_record_commitment,
        application_result_ref,
        active_authoritative_participant_id,
        ..
    } = &intent.state
    else {
        return Err(refusal("direct-host enrollment Start was not applied"));
    };
    let transactions = root
        .start_transaction_map
        .values()
        .filter(|t| t.orchestration_session_id == session_id)
        .collect::<Vec<_>>();
    let [transaction] = transactions.as_slice() else {
        return Err(refusal(
            "direct-host enrollment public Start transaction is missing or ambiguous",
        ));
    };
    let descriptor: AgentDescriptorHashInputV1 =
        direct_host_public_read(reader, &intent.descriptor_ref)?;
    let attach: HostAttachContractHashInputV1 =
        direct_host_public_read(reader, &intent.host_attach_contract_ref)?;
    let policy: PolicyObjectHashInputV1 =
        direct_host_public_read(reader, &attach.contract.policy_ref)?;
    if intent.mode != HostSessionTransitionModeV1::Start
        || intent.world_binding.is_some()
        || intent.source_authoritative_participant_id.is_some()
        || intent.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
        || intent.issuer_request_id != *issuer_request_id
        || intent.payload_commitment != *payload_commitment
        || authority_revision_before.is_some()
        || *authority_revision_after != 1
        || *authority_record_commitment != first.authority_record_commitment
        || birth.active_authoritative_participant_id.as_ref()
            != Some(active_authoritative_participant_id)
        || intent.orchestration_session_id != session_id
        || intent.shell_trace_session_id != birth.shell_trace_session_id
        || intent.workspace_binding != birth.workspace_binding
        || descriptor.descriptor.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.execution_scope != AgentExecutionScopeV1::Host
        || attach.contract.descriptor_ref != intent.descriptor_ref
        || birth.host_attach_contract_ref.as_ref() != Some(&intent.host_attach_contract_ref)
        || birth.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
        || birth.current_policy_revision.as_ref() != Some(&policy.policy_revision)
        || transaction.authority_store_id != root.authority_store_id
        || transaction.start_intent_id != *intent_id
        || transaction.start_issuer_request_id != *issuer_request_id
        || transaction.start_payload_commitment != *payload_commitment
        || transaction.start_application_result_ref != *application_result_ref
        || transaction.start_run_id != intent.run_id
        || transaction.start_authority_revision != 1
        || transaction.authoritative_participant_id != *active_authoritative_participant_id
        || transaction.shell_trace_session_id != birth.shell_trace_session_id
        || transaction.workspace_root != birth.workspace_binding.workspace_root.physical_path
        || transaction.backend_id != descriptor.descriptor.backend_id
        || transaction.protocol != descriptor.descriptor.protocol
        || transaction.world_id.is_some()
        || transaction.world_generation.is_some()
    {
        return Err(refusal(
            "direct-host enrollment Start application/public transaction joins disagree",
        ));
    }

    if transaction.schema_version != 1
        || root.start_transaction_map.get(&transaction.transaction_id) != Some(*transaction)
        || transaction.public_backend_id.trim().is_empty()
        || transaction.public_scope.trim().is_empty()
    {
        return Err(refusal(
            "durable direct-host public birth identity is invalid",
        ));
    }
    Ok(())
}

fn verify_direct_host_retry_plan(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    binding: &DirectHostResumeBindingV1,
    context: &DirectHostPersistedNativeContextV1,
) -> Result<(), BootstrapError> {
    let expected = &plan.applied_transition;
    if plan.schema_version != 1
        || plan.authority_store_id != root.authority_store_id
        || expected.intent_id != intent.intent_id
        || expected.payload_commitment != intent.payload_commitment
        || expected.world_binding.is_some()
        || expected.mode != HostSessionTransitionModeV1::ResumeOneTurn
        || expected.intent_revision.checked_add(1) != Some(binding.intent_revision_observed)
        || canonical_sha256(&direct_host_payload(expected))
            .map_err(|_| refusal("hash retry helper payload"))?
            != canonical_sha256(&direct_host_payload(intent))
                .map_err(|_| refusal("hash committed retry payload"))?
    {
        return Err(refusal(
            "committed direct-host retry plan identity disagrees",
        ));
    }
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        authority_record_commitment,
        application_result_ref,
        active_authoritative_participant_id,
        resulting_posture,
        startup_ownership,
        post_turn,
        applied_at,
        ..
    } = &expected.state
    else {
        return Err(refusal("retry helper did not carry Applied Resume"));
    };
    let HostSessionTransitionIntentStateV3::Applied {
        authority_revision_before: original_before,
        applied_at: original_applied_at,
        ..
    } = &intent.state
    else {
        return Err(refusal("committed retry lost initial Applied state"));
    };
    let helper = &plan.helper_plan;
    let descriptor = &context.descriptor;
    let backend_kind = match helper.descriptor.backend_kind {
        ResolvedRuntimeBackendKind::Codex => RuntimeBackendKindV1::Codex,
        ResolvedRuntimeBackendKind::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
    };
    if authority_revision_before != original_before
        || applied_at != original_applied_at
        || *resulting_posture != HostSessionPostureV1::ActiveAttached
        || startup_ownership.as_ref() != &HostSessionStartupOwnershipApplicationV1::NotApplicable
        || !matches!(post_turn.as_ref(), HostSessionPostTurnApplicationV2::Pending {
            expected_run_id, expected_authority_revision
        } if expected_run_id == &binding.run_id && *expected_authority_revision == binding.authority_revision_observed)
        || !matches!(&expected.input_handoff, HostSessionTransitionInputHandoffV1::Pending {
            input_ref, run_id
        } if input_ref == &binding.transition_input_ref && run_id == &binding.run_id)
        || claim_id != &binding.claim_id
        || claimant_attempt_id != &binding.claimant_attempt_id
        || *authority_revision_after != binding.authority_revision_observed
        || authority_record_commitment != &binding.authority_record_commitment
        || application_result_ref != &binding.initial_application_result_ref
        || active_authoritative_participant_id != &binding.authoritative_participant_id
        || helper.mode != OwnerHelperMode::ResumeOneTurn
        || helper.descriptor.execution_scope != AgentExecutionScope::Host
        || helper.descriptor.agent_id != descriptor.agent_id
        || helper.descriptor.backend_id != descriptor.backend_id
        || helper.descriptor.protocol != descriptor.protocol
        || backend_kind != descriptor.backend_kind
        || helper.descriptor.binary_path != descriptor.binary_path
        || helper.session.orchestration_session_id != binding.orchestration_session_id
        || helper.session.shell_trace_session_id != binding.shell_trace_session_id
        || helper.session.workspace_root != binding.workspace_binding.workspace_root.physical_path
        || helper.session.world_id.is_some()
        || helper.session.world_generation.is_some()
        || helper.participant.participant_id != binding.authoritative_participant_id
        || helper.participant.run_id != binding.run_id
        || helper.participant.resumed_from_participant_id
            != intent.source_authoritative_participant_id
        || helper.participant.internal_uaa_session_id.as_ref() != Some(&context.native_session_id)
    {
        return Err(refusal(
            "committed direct-host retry helper/native joins disagree",
        ));
    }
    // No prompt/token bytes, erased transition input, old inherited guard,
    // current source/caps or a fresh timestamp are read on this route.
    Ok(())
}

fn read_committed_direct_host_resume(
    read: &mut super::store::DirectHostResumeRecoveryReadV1<'_>,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    native: &DirectHostNativeTerminalEvidenceV1,
    kind: &HostPostTurnProtocolEventKindV1,
) -> Result<Option<super::transition::PostTurnResolutionOutcomeV1>, BootstrapError> {
    let root = read.state().clone();
    let intent = root
        .successor_transition_intent_map
        .get(&plan.applied_transition.intent_id)
        .ok_or_else(|| refusal("direct-host retry intent is absent"))?;
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
        return Ok(None);
    };
    let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
    else {
        return Ok(None);
    };
    let mut reader = |r: &AuthorityObjectRefV1| read.read_canonical_object(r);
    let completion = verify_direct_host_completed_join(&root, intent, completion_ref, &mut reader)?;
    let context = verify_direct_host_binding_with_reader(&root, &completion.binding, &mut reader)?;
    verify_direct_host_retry_plan(&root, intent, plan, &completion.binding, &context)?;
    let event: PostTurnProtocolEventHashInputV2 =
        direct_host_public_read(&mut reader, &completion.protocol_event_ref)?;
    if event.native_terminal.as_ref() != Some(native) || &event.kind != kind {
        return Err(refusal(
            "direct-host retry conflicts with original native terminal",
        ));
    }
    Ok(Some(
        super::transition::PostTurnResolutionOutcomeV1::Joined(intent.clone()),
    ))
}

fn direct_host_prepare_public_object(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    kind: AuthorityObjectKindV1,
    input: super::hash::CanonicalObjectHashInputV1<'_>,
) -> Result<super::store::GeneratedObjectV1, BootstrapError> {
    let bytes = super::hash::canonical_object_bytes(kind, input)
        .map_err(|_| refusal("encode direct-host settlement object"))?;
    tx.prepare_authority_object(kind, &bytes)
}

fn direct_host_insert_prepared(
    root: &mut StateRootV3,
    object: &super::store::GeneratedObjectV1,
) -> Result<(), BootstrapError> {
    let reference = &object.reference;
    let entry = AuthorityObjectIndexEntryV1 {
        schema_version: 1,
        ref_id: reference.ref_id.clone(),
        object_kind: reference.object_kind,
        object_schema_version: reference.schema_version,
        byte_length: object.byte_length,
        storage_state: AuthorityObjectStorageStateV1::Present,
    };
    if root
        .object_index
        .insert(reference.ref_id.clone(), entry.clone())
        .is_some_and(|old| old != entry)
    {
        return Err(refusal("direct-host settlement object index conflict"));
    }
    Ok(())
}

fn settle_direct_host_resume_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    native: &DirectHostNativeTerminalEvidenceV1,
    kind: &HostPostTurnProtocolEventKindV1,
    emitted_at: &TimestampV1,
) -> Result<super::transition::PostTurnResolutionOutcomeV1, BootstrapError> {
    use super::hash::CanonicalObjectHashInputV1 as Input;
    let producer = authenticate_direct_host_resume_locked(tx, plan)?;
    verify_direct_host_terminal_native(
        native,
        kind,
        producer.descriptor(),
        producer.native_session_id(),
    )?;
    #[cfg(test)]
    crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::exercise_direct_host_publication_boundaries_for_test(tx, &producer)?;
    let binding = producer.binding().clone();
    let outcome = match kind {
        HostPostTurnProtocolEventKindV1::ResumableClean => {
            PostTurnCompletionOutcomeV1::ResumableClean
        }
        HostPostTurnProtocolEventKindV1::TerminalClean => {
            PostTurnCompletionOutcomeV1::TerminalClean
        }
        HostPostTurnProtocolEventKindV1::TerminalFailure { .. } => {
            PostTurnCompletionOutcomeV1::TerminalFailure
        }
    };
    // Terminal clean/failure is authoritative independently of any resumable
    // attention cut. Only successful resumable completion refreshes/materializes
    // exhaustive carried+current source meaning. Unknown never parks/releases.
    let cut = if outcome == PostTurnCompletionOutcomeV1::ResumableClean {
        Some(crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::materialize_direct_host_inbox_locked(tx, &producer)?)
    } else {
        None
    };
    producer.verify_active(tx)?;
    let Some(VersionedStateRoot::V3(current)) = tx.authority_root() else {
        return Err(refusal("direct-host settlement authority disappeared"));
    };
    let current = current.clone();
    let intent = current
        .successor_transition_intent_map
        .get(&binding.intent_id)
        .ok_or_else(|| refusal("direct-host settlement intent disappeared"))?
        .clone();
    let Some(SessionNamespaceRecordV1::Authority(authority)) = current
        .session_namespace_map
        .get(&binding.orchestration_session_id)
    else {
        return Err(refusal("direct-host settlement session disappeared"));
    };
    let authority = authority.as_ref();
    let accepted_at = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { accepted_at, .. } => accepted_at,
        _ => return Err(refusal("direct-host settlement input is not Accepted")),
    };
    if emitted_at.as_str() < authority.updated_at.as_str()
        || emitted_at.as_str() < accepted_at.as_str()
    {
        return Err(refusal(
            "direct-host native terminal precedes authenticated application/input",
        ));
    }
    let resulting_posture = match &cut {
        Some(cut) if !cut.unresolved_attention.is_empty() => {
            HostSessionPostureV1::AwaitingAttention
        }
        Some(_) => HostSessionPostureV1::ParkedResumable,
        None => HostSessionPostureV1::Terminal,
    };
    let event_value = PostTurnProtocolEventHashInputV2 {
        schema_version: 2,
        binding: binding.clone(),
        protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: binding.authoritative_participant_id.clone(),
        },
        event_id: direct_host_terminal_event_id(&binding, native, kind)?,
        kind: kind.clone(),
        native_terminal: Some(native.clone()),
        emitted_at: emitted_at.clone(),
    };
    let event = direct_host_prepare_public_object(
        tx,
        AuthorityObjectKindV1::PostTurnProtocolEvent,
        Input::PostTurnProtocolEventV2(&event_value),
    )?;
    let produced_handle = if outcome == PostTurnCompletionOutcomeV1::ResumableClean {
        let value = ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: binding.orchestration_session_id.clone(),
            participant_id: binding.authoritative_participant_id.clone(),
            backend_id: producer.descriptor().backend_id.clone(),
            protocol: producer.descriptor().protocol.clone(),
            internal_uaa_session_id: native.native_session_id.clone(),
        };
        Some(direct_host_prepare_public_object(
            tx,
            AuthorityObjectKindV1::ResumeHandle,
            Input::ResumeHandle(&value),
        )?)
    } else {
        None
    };
    let completion_value = PostTurnCompletionHashInputV3 {
        schema_version: 3,
        binding: binding.clone(),
        protocol_event_ref: event.reference.clone(),
        outcome,
        attention_cut: cut.clone(),
        completed_at: emitted_at.clone(),
        produced_resume_handle_ref: produced_handle.as_ref().map(|h| h.reference.clone()),
    };
    let completion = direct_host_prepare_public_object(
        tx,
        AuthorityObjectKindV1::PostTurnCompletion,
        Input::PostTurnCompletionV3(&completion_value),
    )?;
    let mut next_authority = authority.clone();
    next_authority.authority_revision = authority
        .authority_revision
        .checked_add(1)
        .ok_or_else(|| refusal("direct-host authority revision overflow"))?;
    next_authority.lifecycle_posture = resulting_posture;
    next_authority.updated_at = emitted_at.clone();
    if let Some(handle) = &produced_handle {
        if next_authority
            .internal_resume_handle_refs
            .contains(&handle.reference)
        {
            return Err(refusal(
                "direct-host produced continuation was already appended",
            ));
        }
        next_authority
            .internal_resume_handle_refs
            .push(handle.reference.clone());
    }
    let authority_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&direct_host_authority_hash_input(&next_authority))
            .map_err(|_| refusal("hash direct-host settled authority"))?,
    };
    let application_value = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: binding.intent_id.clone(),
        mode: HostSessionTransitionModeV1::ResumeOneTurn,
        run_id: binding.run_id.clone(),
        phase: ApplicationResultPhaseV1::PostTurn {
            completion_ref: completion.reference.clone(),
            authority_revision_before: authority.authority_revision,
            authority_revision_after: next_authority.authority_revision,
            active_authoritative_participant_id: binding.authoritative_participant_id.clone(),
            resulting_posture,
            authority_record_commitment: authority_commitment.clone(),
        },
        applied_at: emitted_at.clone(),
    };
    let application = direct_host_prepare_public_object(
        tx,
        AuthorityObjectKindV1::ApplicationResult,
        Input::ApplicationResult(&application_value),
    )?;
    let terminal_value = TerminalHandoffHashInputV1 {
        schema_version: 1,
        intent_id: binding.intent_id.clone(),
        run_id: binding.run_id.clone(),
        payload_commitment: binding.payload_commitment.clone(),
        terminal_state: TerminalHandoffStateV1::Applied,
        application_result_ref: Some(binding.initial_application_result_ref.clone()),
        input_acceptance_ref: binding.input_acceptance_ref.clone(),
        post_turn_completion_ref: Some(completion.reference.clone()),
        post_turn_application_result_ref: Some(application.reference.clone()),
        recorded_at: emitted_at.clone(),
    };
    let terminal = direct_host_prepare_public_object(
        tx,
        AuthorityObjectKindV1::TerminalHandoff,
        Input::TerminalHandoff(&terminal_value),
    )?;
    let mut proposed = current.clone();
    proposed.root_revision = current
        .root_revision
        .checked_add(1)
        .ok_or_else(|| refusal("direct-host root revision overflow"))?;
    proposed.session_namespace_map.insert(
        binding.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(next_authority.clone())),
    );
    let next = proposed
        .successor_transition_intent_map
        .get_mut(&binding.intent_id)
        .ok_or_else(|| refusal("direct-host settlement intent disappeared"))?;
    next.intent_revision = next
        .intent_revision
        .checked_add(1)
        .ok_or_else(|| refusal("direct-host intent revision overflow"))?;
    next.updated_at = emitted_at.clone();
    next.transport_payload_state = HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
        terminal_handoff_ref: terminal.reference.clone(),
    };
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &mut next.state else {
        return Err(refusal("direct-host settlement lost Applied intent"));
    };
    *post_turn = Box::new(HostSessionPostTurnApplicationV2::Applied {
        completion_ref: Box::new(completion.reference.clone()),
        obligation_snapshot_ref: None,
        authority_revision_before: authority.authority_revision,
        authority_revision_after: next_authority.authority_revision,
        resulting_posture,
        application_result_ref: Box::new(application.reference.clone()),
        applied_at: emitted_at.clone(),
    });
    proposed
        .successor_application_journal
        .get_mut(&binding.intent_id)
        .ok_or_else(|| refusal("direct-host settlement journal disappeared"))?
        .post_turn_application = Some(PostTurnApplicationJournalV2 {
        completion_ref: completion.reference.clone(),
        obligation_snapshot_ref: None,
        authority_revision_before: authority.authority_revision,
        authority_revision_after: next_authority.authority_revision,
        authority_record_commitment: authority_commitment,
        application_result_ref: application.reference.clone(),
        applied_at: emitted_at.clone(),
    });
    proposed
        .object_index
        .get_mut(&intent.transport_payload_ref.ref_id)
        .ok_or_else(|| refusal("direct-host transport index disappeared"))?
        .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
        terminal_handoff_ref: terminal.reference.clone(),
    };
    for object in [&event, &completion, &application, &terminal]
        .into_iter()
        .chain(produced_handle.iter())
    {
        direct_host_insert_prepared(&mut proposed, object)?;
    }
    producer.verify_active(tx)?;
    #[cfg(test)]
    direct_host_settlement_cut_probe_for_test(tx, &producer)?;
    if let Some(expected) = &cut {
        let fresh = crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::read_direct_host_attention_cut_locked(tx, &producer)?;
        if &fresh != expected {
            return Err(refusal(
                "direct-host source/canonical cut changed before HSA commit",
            ));
        }
    }
    // tx retains exact source reads/manifests and the same physical root lock
    // through publication; the opaque helper/workspace guard is checked again
    // inside the final root publication callback.
    let committed = tx.commit_authority_root(&current, &proposed, || producer.verify_guard())?;
    Ok(super::transition::PostTurnResolutionOutcomeV1::Applied(
        committed.successor_transition_intent_map[&binding.intent_id].clone(),
    ))
}

impl super::facade::HostSessionAuthority {
    pub(crate) fn settle_direct_host_resume(
        &self,
        plan: &AuthorityManagedSuccessorLaunchPlanV1,
        native: &DirectHostNativeTerminalEvidenceV1,
        kind: HostPostTurnProtocolEventKindV1,
        emitted_at: &TimestampV1,
    ) -> Result<super::transition::PostTurnResolutionOutcomeV1, BootstrapError> {
        super::store::with_direct_host_resume_recovery(
            self.trusted_root(),
            |read| read_committed_direct_host_resume(read, plan, native, &kind),
            |tx| settle_direct_host_resume_locked(tx, plan, native, &kind, emitted_at),
        )
    }
}
#[cfg(test)]
mod direct_host_terminal_native_tests {
    use super::*;

    #[test]
    fn test_direct_host_resume_terminal_native_requires_genuine_accepted_target_failure() {
        let descriptor = AgentDescriptorV1 {
            schema_version: 1,
            agent_id: "agent".into(),
            backend_id: "backend".into(),
            backend_kind: RuntimeBackendKindV1::Codex,
            protocol: "codex-jsonl".into(),
            execution_scope: AgentExecutionScopeV1::Host,
            binary_path: "/native".into(),
        };
        let mut native = DirectHostNativeTerminalEvidenceV1 {
            schema_version: 1,
            backend_id: "backend".into(),
            protocol: "codex-jsonl".into(),
            native_session_id: "thread".into(),
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            sequence: DirectHostNativeSequenceV1::Codex {
                resumed_thread_id: "thread".into(),
                started_turn_id: "turn".into(),
            },
            terminal_event_kind: "turn.failed".into(),
            raw_event_sha256: "a".repeat(64),
            terminal_ordinal: 1,
        };
        let accepted_failure = HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
        };
        assert!(verify_direct_host_terminal_native(
            &native,
            &accepted_failure,
            &descriptor,
            "thread"
        )
        .is_ok());
        for kind in [
            HostPostTurnProtocolEventKindV1::ResumableClean,
            HostPostTurnProtocolEventKindV1::TerminalClean,
            HostPostTurnProtocolEventKindV1::TerminalFailure {
                reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
            },
            HostPostTurnProtocolEventKindV1::TerminalFailure {
                reason: HostPostTurnTerminalReasonV1::TargetFailedBeforeInputAcceptance,
            },
        ] {
            assert!(
                verify_direct_host_terminal_native(&native, &kind, &descriptor, "thread").is_err()
            );
        }
        native.raw_event_sha256.clear();
        assert!(verify_direct_host_terminal_native(
            &native,
            &accepted_failure,
            &descriptor,
            "thread"
        )
        .is_err());
        native.raw_event_sha256 = "b".repeat(64);
        native.terminal_event_kind = "eof".into();
        assert!(verify_direct_host_terminal_native(
            &native,
            &accepted_failure,
            &descriptor,
            "thread"
        )
        .is_err());
    }
}

/// Synthetic adversarial leaf/history checks over the genuine fixture's frozen
/// root. Alternate reader bytes intentionally isolate identity checks from the
/// existing canonical commitment verifier; no altered object is published.
#[cfg(test)]
pub(crate) fn assert_direct_host_completion_v3_refusals_for_test(
    authority: &super::facade::HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) {
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
        panic!("Applied required");
    };
    let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
    else {
        panic!("completion required");
    };
    let mut reader = |r: &AuthorityObjectRefV1| {
        authority
            .read_authority_object_v2_at(root.root_revision, r)
            .map_err(|_| refusal("read genuine fixture object"))
    };
    let c = read_direct_host_completion(&mut reader, completion_ref).unwrap();
    assert_direct_host_binding_pre_history_order_for_test(root, &c.binding, &mut reader);
    let produced = c.produced_resume_handle_ref.as_ref().unwrap();
    let history =
        exact_v3_authority_history_with_reader(root, &intent.orchestration_session_id, |r| {
            reader(r).map_err(|e| e.to_string())
        })
        .unwrap();
    let prefix = history
        .range(..=c.binding.authority_revision_observed)
        .map(|(r, a)| (*r, a.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        verify_direct_host_completion_v3_edge(root, intent, completion_ref, &prefix, &mut reader)
            .unwrap()
            .as_ref(),
        Some(produced)
    );
    assert!(verify_direct_host_completion_v3_edge(
        root,
        intent,
        completion_ref,
        &prefix,
        &mut |r| if r == produced {
            Err(refusal("missing produced handle"))
        } else {
            reader(r)
        }
    )
    .is_err());
    let original: ResumeHandleHashInputV1 = direct_host_public_read(&mut reader, produced).unwrap();
    for field in [
        "orchestration_session_id",
        "participant_id",
        "backend_id",
        "protocol",
        "internal_uaa_session_id",
    ] {
        let mut substituted = serde_json::to_value(&original).unwrap();
        substituted[field] = serde_json::json!("wrong-identity");
        let bad: ResumeHandleHashInputV1 = serde_json::from_value(substituted).unwrap();
        let bytes = canonical_json::to_vec(&bad).unwrap();
        assert!(
            verify_direct_host_completion_v3_edge(
                root,
                intent,
                completion_ref,
                &prefix,
                &mut |r| if r == produced {
                    Ok(bytes.clone())
                } else {
                    reader(r)
                }
            )
            .is_err(),
            "{field}"
        );
    }
    let mut stale = prefix.clone();
    stale.pop_last();
    assert!(verify_direct_host_completion_v3_edge(
        root,
        intent,
        completion_ref,
        &stale,
        &mut reader
    )
    .is_err());
    let mut duplicate_parent = root.clone();
    duplicate_parent
        .successor_transition_intent_map
        .insert("duplicate-parent".into(), intent.clone());
    assert!(verify_direct_host_completion_v3_edge(
        &duplicate_parent,
        intent,
        completion_ref,
        &prefix,
        &mut reader
    )
    .is_err());
    let mut conflict = intent.clone();
    conflict.intent_id = "conflicting-parent".into();
    duplicate_parent
        .successor_transition_intent_map
        .insert("duplicate-parent".into(), conflict);
    assert!(verify_direct_host_completion_v3_edge(
        &duplicate_parent,
        intent,
        completion_ref,
        &prefix,
        &mut reader
    )
    .is_err());
    for mutation in ["missing", "duplicate", "reordered"] {
        let mut altered = root.clone();
        let Some(SessionNamespaceRecordV1::Authority(current)) = altered
            .session_namespace_map
            .get_mut(&intent.orchestration_session_id)
        else {
            panic!("authority required");
        };
        match mutation {
            "missing" => current
                .internal_resume_handle_refs
                .retain(|r| r != produced),
            "duplicate" => current.internal_resume_handle_refs.push(produced.clone()),
            "reordered" => current.internal_resume_handle_refs.reverse(),
            _ => unreachable!(),
        }
        assert!(
            exact_v3_authority_history_with_reader(
                &altered,
                &intent.orchestration_session_id,
                |r| reader(r).map_err(|e| e.to_string())
            )
            .is_err(),
            "{mutation}"
        );
    }
    let mut wrong_commitment = root.clone();
    wrong_commitment
        .successor_application_journal
        .get_mut(&intent.intent_id)
        .unwrap()
        .post_turn_application
        .as_mut()
        .unwrap()
        .authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "0".repeat(64),
    };
    assert!(exact_v3_authority_history_with_reader(
        &wrong_commitment,
        &intent.orchestration_session_id,
        |r| reader(r).map_err(|e| e.to_string())
    )
    .is_err());
}

/// Focused ordering regressions over the genuine fixture binding. Failed
/// prelude reads must return before any history object is requested.
#[cfg(test)]
fn assert_direct_host_binding_pre_history_order_for_test(
    root: &StateRootV3,
    binding: &DirectHostResumeBindingV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) {
    let mut reads = Vec::new();
    let mut invalid = binding.clone();
    invalid.schema_version = 0;
    let error = verify_direct_host_binding_with_reader(root, &invalid, &mut |reference| {
        reads.push(reference.clone());
        Err(refusal(
            "unexpected history read before invalid binding refusal",
        ))
    })
    .err()
    .expect("invalid binding must refuse");
    assert_eq!(
        error.to_string(),
        "invalid durable direct-host publication binding"
    );
    assert!(reads.is_empty());
    invalid = binding.clone();
    invalid.intent_id = "missing-ordering-test-intent".into();
    let error = verify_direct_host_binding_with_reader(root, &invalid, &mut |reference| {
        reads.push(reference.clone());
        Err(refusal(
            "unexpected history read before immutable identity refusal",
        ))
    })
    .err()
    .expect("missing immutable intent must refuse");
    assert_eq!(
        error.to_string(),
        "durable direct-host publication intent is absent"
    );
    assert!(reads.is_empty());
    let acceptance = binding.input_acceptance_ref.as_ref().unwrap();
    let error = verify_direct_host_binding_with_reader(root, binding, &mut |reference| {
        reads.push(reference.clone());
        Err(refusal("ordered acceptance read refusal"))
    })
    .err()
    .expect("acceptance read must refuse");
    assert_eq!(error.to_string(), "ordered acceptance read refusal");
    assert_eq!(reads.as_slice(), std::slice::from_ref(acceptance));
    reads.clear();
    let error = verify_direct_host_binding_with_reader(root, binding, &mut |reference| {
        reads.push(reference.clone());
        if reference == acceptance {
            reader(reference)
        } else {
            Err(refusal("ordered application read refusal"))
        }
    })
    .err()
    .expect("application read must refuse");
    assert_eq!(error.to_string(), "ordered application read refusal");
    assert_eq!(
        reads,
        vec![
            acceptance.clone(),
            binding.initial_application_result_ref.clone()
        ]
    );
}

// Narrow same-binary fixture observation. Every callback sees the actual active
// HSA transaction and the sealed capability obtained by the normal helper.
#[cfg(test)]
thread_local! {
    static DIRECT_HOST_PROTOCOL_PROBE: std::cell::RefCell<Option<(String, Option<std::thread::JoinHandle<()>>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn direct_host_protocol_probe_mode_for_test() -> Option<String> {
    DIRECT_HOST_PROTOCOL_PROBE.with(|slot| slot.borrow().as_ref().map(|s| s.0.clone()))
}

#[cfg(test)]
pub(crate) fn with_direct_host_protocol_probe_for_test<T>(
    mode: &str,
    operation: impl FnOnce() -> T,
) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            DIRECT_HOST_PROTOCOL_PROBE.with(|slot| {
                if let Some((_, Some(writer))) = slot.borrow_mut().take() {
                    writer
                        .join()
                        .expect("normal ingress writer must finish after HSA unlock");
                }
            });
        }
    }
    DIRECT_HOST_PROTOCOL_PROBE.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some((mode.to_owned(), None));
    });
    let _reset = Reset;
    operation()
}

#[cfg(test)]
fn direct_host_settlement_cut_probe_for_test(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    producer: &AuthenticatedDirectHostResumeV1,
) -> Result<(), BootstrapError> {
    use crate::execution::agent_runtime::obligation_ledger::direct_host_inbox::{
        publish_direct_host_inbox_request, read_direct_host_inbox_source,
    };
    let mode = direct_host_protocol_probe_mode_for_test();
    if !matches!(mode.as_deref(), Some("cut-lock" | "stale-cut")) {
        return Ok(());
    }
    let source = read_direct_host_inbox_source(tx)?;
    let request = crate::execution::agent_runtime::host_inbox::HostInboxRecord::new(
        "other-enrolled-source-target",
        format!("after-cut-{}", producer.binding().run_id),
        crate::execution::agent_runtime::obligation_ledger::OrchestrationObligationKind::FollowUpRequired,
        "typed local arrival at the settlement boundary",
        gethostname::gethostname().to_string_lossy().trim().to_string(),
        "unverified-sender", "unverified-upstream",
    );
    if mode.as_deref() == Some("stale-cut") {
        // Simulate stale preparation within the held transaction. The production
        // ingress path cannot interleave here; cut-lock below proves exclusion.
        publish_direct_host_inbox_request(tx, &request)?;
        assert_ne!(
            read_direct_host_inbox_source(tx)?.head_commitment,
            source.head_commitment
        );
        return Ok(());
    }
    let home = Path::new(&producer.binding().bootstrap_home.physical_path).to_path_buf();
    let (started, attempted) = std::sync::mpsc::sync_channel(0);
    let (finished, completion) = std::sync::mpsc::sync_channel(1);
    let writer = std::thread::spawn(move || {
        let lock = File::open(home.join("authority-v1/lock/root.lock")).unwrap();
        // SAFETY: the owned descriptor lives through this nonblocking probe.
        assert_eq!(
            unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().kind(),
            std::io::ErrorKind::WouldBlock
        );
        started.send(()).unwrap();
        crate::execution::agent_runtime::AgentRuntimeStateStore::new()
            .unwrap()
            .persist_host_inbox_record(&request)
            .unwrap();
        finished.send(()).unwrap();
    });
    attempted
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(matches!(
        completion.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    assert_eq!(
        read_direct_host_inbox_source(tx)?.head_commitment,
        source.head_commitment
    );
    DIRECT_HOST_PROTOCOL_PROBE.with(|slot| {
        let mut state = slot.borrow_mut();
        assert!(state.as_ref().unwrap().1.is_none());
        state.as_mut().unwrap().1 = Some(std::thread::spawn(move || {
            writer.join().unwrap();
            completion
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
        }));
    });
    Ok(())
}

/// This models an already durable Released decision solely to exercise the
/// existing deletion/reconciliation primitive. There is no production Released
/// producer in this module; the fixture write is deliberately explicit.
#[cfg(test)]
pub(crate) fn assert_direct_host_released_reconciliation_for_test(
    authority: &super::facade::HostSessionAuthority,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) {
    let before = authority.read_a12b_root().unwrap();
    let intent_id = &plan.applied_transition.intent_id;
    let intent = &before.successor_transition_intent_map[intent_id];
    let HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
        terminal_handoff_ref,
    } = &intent.transport_payload_state
    else {
        panic!("genuine settled direct-host intent must be ReleaseEligible");
    };
    let home = Path::new(&before.bootstrap_home.physical_path);
    let transport = home.join(format!(
        "authority-v1/objects/transition-transport-payload/v1/{}.obj",
        intent.transport_payload_ref.ref_id
    ));
    let original = std::fs::read(&transport).unwrap();
    let mut released = before.clone();
    released.root_revision += 1;
    let released_at = intent.updated_at.clone();
    released
        .successor_transition_intent_map
        .get_mut(intent_id)
        .unwrap()
        .transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Released {
        terminal_handoff_ref: terminal_handoff_ref.clone(),
        released_at: released_at.clone(),
    };
    released
        .object_index
        .get_mut(&intent.transport_payload_ref.ref_id)
        .unwrap()
        .storage_state = AuthorityObjectStorageStateV1::Released {
        terminal_handoff_ref: terminal_handoff_ref.clone(),
        released_at,
    };
    released.validate().unwrap();
    let mut mismatched = released.clone();
    mismatched
        .object_index
        .get_mut(&intent.transport_payload_ref.ref_id)
        .unwrap()
        .storage_state = AuthorityObjectStorageStateV1::Present;
    assert!(
        mismatched.validate().is_err(),
        "intent/index Released correspondence is mandatory"
    );
    // The last production state still retains the payload before any Released
    // model is written. A reader/reconciliation retry cannot delete it early.
    assert!(
        super::store::reconcile_existing_v3_fixture_for_test(home).unwrap() == before,
        "strict V3 reconciliation must preserve the exact selected root"
    );
    assert_eq!(std::fs::read(&transport).unwrap(), original);
    let root_path = home.join("authority-v1/state-root-v1.json");
    let bytes = VersionedStateRoot::V3(released.clone())
        .to_canonical_bytes()
        .unwrap();
    std::fs::write(&root_path, &bytes).unwrap();
    assert_eq!(
        std::fs::read(&transport).unwrap(),
        original,
        "crash after durable Released and before unlink"
    );
    assert!(
        super::store::reconcile_existing_v3_fixture_for_test(home).unwrap() == released,
        "strict V3 reconciliation must preserve the exact selected root"
    );
    assert!(
        !transport.exists(),
        "existing reconciliation performs the actual unlink"
    );
    assert_eq!(std::fs::read(&root_path).unwrap(), bytes);
    assert!(
        super::store::reconcile_existing_v3_fixture_for_test(home).unwrap() == released,
        "strict V3 reconciliation must preserve the exact selected root"
    );
    assert!(!transport.exists(), "retry after unlink is idempotent");
    assert_eq!(authority.read_a12b_root().unwrap(), released);
    authority
        .resolve_exact_typed_history(&intent.orchestration_session_id)
        .unwrap()
        .unwrap();
    eprintln!("Released fixture model on genuine post-turn history: pre-release retention; durable decision before actual reconciliation unlink; exact repeated reconciliation/current history. Production Released producer remains unproved.");
}
