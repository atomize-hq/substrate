//! Exact A1 host-session authority facade.

use std::fmt;
#[cfg(test)]
use std::path::Path;

use super::canonical_json;
use super::hash::canonical_sha256;
#[cfg(test)]
use super::schema::TimestampV1;
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AuthoritativeLineageHashInputV1,
    AuthorityObjectCommitmentV1, AuthorityObjectRefV1, CanonicalDirectoryV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
    HostAttachContractHashInputV1, HostAttachContractV1, HostSessionPostureV1,
    PolicyObjectHashInputV1, WorldBindingV1,
};
use super::store::{
    self, BootstrapClassificationV1, ExpectedRevisionsV1, ObjectPublicationOutcomeV1,
    ObjectVerificationContextV1, TransactionCommitOutcomeV1,
};
use super::store_schema::{
    DurableSessionAuthorityV1, HostSessionPostTurnApplicationV1,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionIntentStateV2,
    RetainedWorkerAuthorityRegistrationRequestV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1, StateRootV1,
};
use super::transition::{verify_applied_start, ApplyHostSessionTransitionRequestV1};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::trusted_fs::EntryKind;
use super::trusted_fs::TrustedAuthorityRoot;

#[derive(Debug)]
pub(crate) struct HostSessionAuthority {
    root: TrustedAuthorityRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityObservationV1 {
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) root_revision: u64,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedSessionAuthorityV1 {
    pub(crate) root_revision: u64,
    pub(crate) authority: DurableSessionAuthorityV1,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthorityParticipantRoleV1 {
    Orchestrator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedAuthorityCallerV1 {
    pub(crate) participant_id: String,
    pub(crate) role: AuthorityParticipantRoleV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) descriptor: AgentDescriptorV1,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedCurrentAuthorityV1 {
    pub(crate) observation: AuthorityObservationV1,
    pub(crate) authority: DurableSessionAuthorityV1,
    pub(crate) caller: ResolvedAuthorityCallerV1,
    pub(crate) host_attach_contract: HostAttachContractV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
    pub(crate) bound_state_store: super::super::state_store::BoundAgentRuntimeStateStore,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedReservationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedApplicationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAuthorityPreconditionV1 {
    pub(crate) authority_store_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReservedRetainedWorkerRegistrationV1 {
    pub(crate) request: RetainedWorkerAuthorityRegistrationRequestV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) retained_worker_ref: AuthorityObjectRefV1,
    pub(crate) descriptor_bytes: Vec<u8>,
    pub(crate) resume_handle_bytes: Vec<u8>,
    pub(crate) retained_worker_bytes: Vec<u8>,
    pub(crate) joined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppliedRetainedWorkerRegistrationV1 {
    pub(crate) registration: RetainedWorkerAuthorityRegistrationV1,
    pub(crate) joined: bool,
}

impl ResolvedSessionAuthorityV1 {
    pub(crate) fn observation(&self) -> AuthorityObservationV1 {
        AuthorityObservationV1 {
            authority_store_id: self.authority_store_id.clone(),
            bootstrap_home: self.bootstrap_home.clone(),
            orchestration_session_id: self.authority.orchestration_session_id.clone(),
            root_revision: self.root_revision,
            authority_revision: self.authority.authority_revision,
            authority_record_commitment: self.authority_record_commitment.clone(),
            authoritative_lineage_commitment: self.authoritative_lineage_commitment.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityFacadeError(String);

impl fmt::Display for AuthorityFacadeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AuthorityFacadeError {}

pub(crate) struct OpenedBootstrapHomeV1<'authority> {
    root: &'authority TrustedAuthorityRoot,
}

impl OpenedBootstrapHomeV1<'_> {
    pub(crate) fn identity(&self) -> Result<&CanonicalDirectoryV1, AuthorityFacadeError> {
        self.revalidate()?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Ok(self.root.identity())
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(unsupported_platform())
        }
    }

    pub(crate) fn revalidate(&self) -> Result<(), AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.root
                .revalidate()
                .map_err(|error| AuthorityFacadeError(error.to_string()))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(unsupported_platform())
        }
    }

    pub(crate) fn read_config_yaml(&self) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file("config.yaml")
    }

    pub(crate) fn read_policy_yaml(&self) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file("policy.yaml")
    }

    pub(crate) fn read_agent_inventory_yaml(
        &self,
    ) -> Result<Vec<(String, Vec<u8>)>, AuthorityFacadeError> {
        self.read_agent_inventory_yaml_after_revalidation(|| {})
    }

    fn read_agent_inventory_yaml_after_revalidation(
        &self,
        after_revalidation: impl FnOnce(),
    ) -> Result<Vec<(String, Vec<u8>)>, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.revalidate()?;
            after_revalidation();
            let root = self.root.directory();
            let Some(kind) = root.entry_kind("agents").map_err(trusted_fs_error)? else {
                self.revalidate()?;
                return Ok(Vec::new());
            };
            if kind != EntryKind::Directory {
                return Err(AuthorityFacadeError(
                    "bootstrap-home agents entry is not a directory".into(),
                ));
            }
            let directory = root.open_directory("agents").map_err(trusted_fs_error)?;
            let mut files = Vec::new();
            for entry in directory.entries().map_err(trusted_fs_error)? {
                if !entry.name.ends_with(".yaml") {
                    continue;
                }
                if entry.kind != EntryKind::RegularFile {
                    return Err(AuthorityFacadeError(
                        "bootstrap-home YAML inventory entry is not a regular file".into(),
                    ));
                }
                let bytes = directory
                    .open_file_entry(&entry)
                    .and_then(|file| file.read_all())
                    .map_err(trusted_fs_error)?;
                directory
                    .revalidate_entry(&entry)
                    .map_err(trusted_fs_error)?;
                files.push((entry.name, bytes));
            }
            files.sort_by(|left, right| left.0.cmp(&right.0));
            self.revalidate()?;
            Ok(files)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = after_revalidation;
            Err(unsupported_platform())
        }
    }

    fn read_optional_file(&self, name: &str) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file_after_revalidation(name, || {})
    }

    fn read_optional_file_after_revalidation(
        &self,
        name: &str,
        after_revalidation: impl FnOnce(),
    ) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.revalidate()?;
            after_revalidation();
            let root = self.root.directory();
            let Some(kind) = root.entry_kind(name).map_err(trusted_fs_error)? else {
                self.revalidate()?;
                return Ok(None);
            };
            if kind != EntryKind::RegularFile {
                return Err(AuthorityFacadeError(
                    "bootstrap-home configuration entry is not a regular file".into(),
                ));
            }
            let bytes = root
                .open_file(name)
                .and_then(|file| file.read_all())
                .map_err(trusted_fs_error)?;
            self.revalidate()?;
            Ok(Some(bytes))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = (name, after_revalidation);
            Err(unsupported_platform())
        }
    }
}

impl HostSessionAuthority {
    pub(super) fn trusted_root(&self) -> &TrustedAuthorityRoot {
        &self.root
    }

    pub(crate) fn from_trusted_root(
        root: TrustedAuthorityRoot,
    ) -> Result<Self, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            root.revalidate()
                .map_err(|error| AuthorityFacadeError(error.to_string()))?;
            Ok(Self { root })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = root;
            Err(unsupported_platform())
        }
    }

    #[cfg(test)]
    pub(crate) fn open(path: &Path) -> Result<Self, AuthorityFacadeError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        Self::from_trusted_root(root)
    }

    pub(crate) fn bootstrap_home(&self) -> OpenedBootstrapHomeV1<'_> {
        OpenedBootstrapHomeV1 { root: &self.root }
    }

    pub(crate) fn classify(&self) -> BootstrapClassificationV1 {
        store::classify_opened(&self.root)
    }

    pub(crate) fn bootstrap(&self) -> Result<StateRootV1, AuthorityFacadeError> {
        store::bootstrap_opened(&self.root).map_err(store_error)
    }

    pub(crate) fn read_root(&self) -> Result<StateRootV1, AuthorityFacadeError> {
        store::read_opened_root(&self.root).map_err(store_error)
    }

    pub(crate) fn prepare_typed_object(
        &self,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, AuthorityFacadeError> {
        store::prepare_typed_object_opened(
            &self.root,
            expected_root_revision,
            reference,
            bytes,
            context,
        )
        .map_err(store_error)
    }

    pub(crate) fn compare_and_swap_root(
        &self,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, AuthorityFacadeError> {
        let current = self.read_root()?;
        if current.session_namespace_map != proposed.session_namespace_map
            || current.transition_intent_map != proposed.transition_intent_map
            || current.issuer_request_index != proposed.issuer_request_index
            || current.application_journal != proposed.application_journal
        {
            return Err(AuthorityFacadeError(
                "generic facade CAS cannot mutate authority or application history".into(),
            ));
        }
        if expected.authority.is_some() {
            return Err(AuthorityFacadeError(
                "generic facade CAS cannot consume an authority transition precondition".into(),
            ));
        }
        validate_current_authority_proofs(&current)?;
        validate_current_authority_proofs(proposed)?;
        store::compare_and_swap_opened_root_exact_current(&self.root, &current, expected, proposed)
            .map_err(store_error)
    }

    pub(crate) fn resolve_exact(
        &self,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedSessionAuthorityV1, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() {
            return Err(AuthorityFacadeError(
                "orchestration session ID must be exact and non-empty".into(),
            ));
        }
        let root = self.read_root()?;
        let record = root
            .session_namespace_map
            .get(orchestration_session_id)
            .ok_or_else(|| AuthorityFacadeError("exact authority session was not found".into()))?;
        let SessionNamespaceRecordV1::Authority(authority) = record else {
            return Err(AuthorityFacadeError(
                "exact session namespace record is not durable authority".into(),
            ));
        };
        let authority = authority.as_ref().clone();
        let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
        let persisted_commitment = exact_current_authority_proof(
            &root,
            &authority.orchestration_session_id,
            authority.authority_revision,
        )?;
        if persisted_commitment != &authority_record_commitment {
            return Err(AuthorityFacadeError(
                "durable application proof does not commit to current authority".into(),
            ));
        }
        let authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })?;
        let resolved = ResolvedSessionAuthorityV1 {
            root_revision: root.root_revision,
            authority,
            authority_record_commitment,
            authoritative_lineage_commitment,
            authority_store_id: root.authority_store_id,
            bootstrap_home: root.bootstrap_home,
        };
        if expected.is_some_and(|value| value != &resolved.observation()) {
            return Err(AuthorityFacadeError(
                "stale or mismatched exact authority observation".into(),
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn resolve_current_exact(
        &self,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedCurrentAuthorityV1, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() {
            return Err(AuthorityFacadeError(
                "orchestration session ID must be exact and non-empty".into(),
            ));
        }
        let root = store::read_opened_root_v2(&self.root).map_err(store_error)?;
        let Some(SessionNamespaceRecordV1::Authority(authority)) =
            root.session_namespace_map.get(orchestration_session_id)
        else {
            return Err(AuthorityFacadeError(
                "exact V2 session namespace record is not current durable authority".into(),
            ));
        };
        let authority = authority.as_ref().clone();
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } = &authority.origin;
        let intent = root
            .transition_intent_map
            .get(intent_id)
            .ok_or_else(|| AuthorityFacadeError("current authority has no Start intent".into()))?;
        if intent.issuer_request_id != *issuer_request_id
            || intent.payload_commitment != *payload_commitment
            || intent.orchestration_session_id != orchestration_session_id
        {
            return Err(AuthorityFacadeError(
                "current authority origin conflicts with its Start intent".into(),
            ));
        }
        let (claim_id, authority_revision_after, active_authoritative_participant_id) =
            match &intent.state {
                HostSessionTransitionIntentStateV2::Applied {
                    claim_id,
                    authority_revision_before: None,
                    authority_revision_after,
                    active_authoritative_participant_id,
                    resulting_posture: HostSessionPostureV1::ActiveAttached,
                    startup_ownership,
                    post_turn,
                    ..
                } if matches!(
                    startup_ownership.as_ref(),
                    HostSessionStartupOwnershipApplicationV1::Pending {
                        expected_run_id,
                        expected_authority_revision,
                        expected_active_authoritative_participant_id,
                    } if expected_run_id == &intent.run_id
                        && expected_authority_revision == authority_revision_after
                        && expected_active_authoritative_participant_id
                            == active_authoritative_participant_id
                ) && post_turn.as_ref()
                    == &HostSessionPostTurnApplicationV1::NotApplicable =>
                {
                    (
                        claim_id,
                        *authority_revision_after,
                        active_authoritative_participant_id,
                    )
                }
                _ => {
                    return Err(AuthorityFacadeError(
                        "current authority is not a complete applied A1.2a Start".into(),
                    ))
                }
            };
        if authority_revision_after != 1
            || active_authoritative_participant_id != &intent.target_authoritative_participant_id
        {
            return Err(AuthorityFacadeError(
                "applied Start authority identity is inconsistent".into(),
            ));
        }
        let expected_claim_revision = intent
            .intent_revision
            .checked_sub(1)
            .ok_or_else(|| AuthorityFacadeError("applied Start revision underflow".into()))?;
        verify_applied_start(
            self,
            &root,
            intent,
            &ApplyHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
                expected_intent_revision: expected_claim_revision,
                claim_id: claim_id.clone(),
                expected_claim_revision,
            },
        )
        .map_err(|error| AuthorityFacadeError(error.to_string()))?;

        let descriptor_bytes = store::read_typed_object_v2_opened(
            &self.root,
            root.root_revision,
            &intent.descriptor_ref,
            None,
        )
        .map_err(store_error)?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(&descriptor_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let attach_bytes = store::read_typed_object_v2_opened(
            &self.root,
            root.root_revision,
            &intent.host_attach_contract_ref,
            None,
        )
        .map_err(store_error)?;
        let attach: HostAttachContractHashInputV1 = canonical_json::from_slice(&attach_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let policy_bytes = store::read_typed_object_v2_opened(
            &self.root,
            root.root_revision,
            &attach.contract.policy_ref,
            None,
        )
        .map_err(store_error)?;
        let policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if descriptor.schema_version != 1
            || descriptor.descriptor.schema_version != 1
            || attach.schema_version != 1
            || attach.contract.schema_version != 1
            || policy.schema_version != 1
            || attach.contract.descriptor_ref != intent.descriptor_ref
            || attach.contract.backend_id != descriptor.descriptor.backend_id
            || attach.contract.execution_scope != descriptor.descriptor.execution_scope
            || attach.contract.protocol != descriptor.descriptor.protocol
            || (descriptor.descriptor.execution_scope
                == super::schema::AgentExecutionScopeV1::World
                && authority.world_binding.is_none())
            || authority.host_attach_contract_ref.as_ref() != Some(&intent.host_attach_contract_ref)
            || authority.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
            || authority.current_policy_revision.as_ref() != Some(&policy.policy_revision)
        {
            return Err(AuthorityFacadeError(
                "applied Start descriptor, attach contract, or policy truth is inconsistent".into(),
            ));
        }

        let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
        let authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })?;
        let observation = AuthorityObservationV1 {
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            orchestration_session_id: authority.orchestration_session_id.clone(),
            root_revision: root.root_revision,
            authority_revision: authority.authority_revision,
            authority_record_commitment,
            authoritative_lineage_commitment,
        };
        if expected.is_some_and(|value| value != &observation) {
            return Err(AuthorityFacadeError(
                "stale or mismatched exact current-authority observation".into(),
            ));
        }
        let active_participant_id = authority
            .active_authoritative_participant_id
            .clone()
            .ok_or_else(|| AuthorityFacadeError("current authority has no active caller".into()))?;
        if active_participant_id != intent.target_authoritative_participant_id
            || !authority
                .authoritative_participant_lineage
                .contains(&active_participant_id)
        {
            return Err(AuthorityFacadeError(
                "current authority caller is not the applied Start orchestrator".into(),
            ));
        }
        let bound_state_store =
            super::super::state_store::AgentRuntimeStateStore::for_bootstrap_home(
                &self.bootstrap_home(),
            )
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if bound_state_store.bootstrap_home_identity() != &root.bootstrap_home {
            return Err(AuthorityFacadeError(
                "bound StateStore does not match current authority bootstrap home".into(),
            ));
        }
        Ok(ResolvedCurrentAuthorityV1 {
            observation,
            authority,
            caller: ResolvedAuthorityCallerV1 {
                participant_id: active_participant_id,
                role: AuthorityParticipantRoleV1::Orchestrator,
                descriptor_ref: intent.descriptor_ref.clone(),
                descriptor: descriptor.descriptor,
            },
            host_attach_contract: attach.contract,
            current_policy: policy,
            bound_state_store,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_retained_worker_registration(
        &self,
        registration_request_id: &str,
        orchestration_session_id: &str,
        expected_authority: &RetainedWorkerAuthorityPreconditionV1,
        retained_participant_id: &str,
        descriptor_bytes: Vec<u8>,
        resume_handle_bytes: Vec<u8>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        let input = retained_reservation_input(
            registration_request_id,
            orchestration_session_id,
            expected_authority,
            retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
        )?;
        store::reserve_retained_worker_registration_opened(&self.root, &input, build_worker)
            .map(reserved_registration)
            .map_err(store_error)
    }

    pub(crate) fn publish_reserved_retained_object(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
    ) -> Result<ObjectPublicationOutcomeV1, AuthorityFacadeError> {
        store::publish_reserved_retained_object_opened(
            &self.root,
            &store_reservation(reserved),
            reference,
            bytes,
        )
        .map_err(store_error)
    }

    pub(crate) fn apply_reserved_retained_worker_registration(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
    ) -> Result<AppliedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        store::apply_reserved_retained_worker_registration_opened(
            &self.root,
            &store_reservation(reserved),
        )
        .map(|applied| AppliedRetainedWorkerRegistrationV1 {
            registration: applied.registration,
            joined: applied.joined,
        })
        .map_err(store_error)
    }

    #[cfg(test)]
    pub(crate) fn apply_reserved_retained_worker_registration_with_crash_point(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        crash_point: RetainedApplicationCrashPointV1,
    ) -> Result<AppliedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        store::apply_reserved_retained_worker_registration_with_crash_point_opened(
            &self.root,
            &store_reservation(reserved),
            match crash_point {
                RetainedApplicationCrashPointV1::BeforeRootPublication => {
                    store::RetainedApplicationCrashPointV1::BeforeRootPublication
                }
                RetainedApplicationCrashPointV1::AfterRootPublication => {
                    store::RetainedApplicationCrashPointV1::AfterRootPublication
                }
            },
        )
        .map(|applied| AppliedRetainedWorkerRegistrationV1 {
            registration: applied.registration,
            joined: applied.joined,
        })
        .map_err(store_error)
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_retained_worker_registration_at(
        &self,
        registration_request_id: &str,
        orchestration_session_id: &str,
        expected_authority: &RetainedWorkerAuthorityPreconditionV1,
        retained_participant_id: &str,
        descriptor_bytes: Vec<u8>,
        resume_handle_bytes: Vec<u8>,
        registered_at: TimestampV1,
        crash_point: Option<RetainedReservationCrashPointV1>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        let input = retained_reservation_input(
            registration_request_id,
            orchestration_session_id,
            expected_authority,
            retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
        )?;
        store::reserve_retained_worker_registration_at_opened(
            &self.root,
            &input,
            registered_at,
            crash_point.map(|point| match point {
                RetainedReservationCrashPointV1::BeforeRootPublication => {
                    store::RetainedReservationCrashPointV1::BeforeRootPublication
                }
                RetainedReservationCrashPointV1::AfterRootPublication => {
                    store::RetainedReservationCrashPointV1::AfterRootPublication
                }
            }),
            build_worker,
        )
        .map(reserved_registration)
        .map_err(store_error)
    }
}

fn retained_reservation_input(
    registration_request_id: &str,
    orchestration_session_id: &str,
    expected_authority: &RetainedWorkerAuthorityPreconditionV1,
    retained_participant_id: &str,
    descriptor_bytes: Vec<u8>,
    resume_handle_bytes: Vec<u8>,
) -> Result<store::RetainedWorkerReservationInputV1, AuthorityFacadeError> {
    if registration_request_id.is_empty()
        || orchestration_session_id.is_empty()
        || retained_participant_id.is_empty()
        || expected_authority.authority_store_id.is_empty()
        || expected_authority.authority_revision == 0
    {
        return Err(AuthorityFacadeError(
            "retained registration plan identity is invalid".into(),
        ));
    }
    Ok(store::RetainedWorkerReservationInputV1 {
        issuer_request_id: format!("retained-worker-registration:{registration_request_id}"),
        orchestration_session_id: orchestration_session_id.to_owned(),
        expected_authority_store_id: expected_authority.authority_store_id.clone(),
        expected_authority_revision: expected_authority.authority_revision,
        expected_authority_commitment: expected_authority.authority_record_commitment.clone(),
        retained_participant_id: retained_participant_id.to_owned(),
        descriptor_bytes,
        resume_handle_bytes,
    })
}

fn reserved_registration(
    reserved: store::RetainedWorkerReservationV1,
) -> ReservedRetainedWorkerRegistrationV1 {
    ReservedRetainedWorkerRegistrationV1 {
        request: reserved.request,
        descriptor_ref: reserved.descriptor_ref,
        resume_handle_ref: reserved.resume_handle_ref,
        retained_worker_ref: reserved.retained_worker_ref,
        descriptor_bytes: reserved.descriptor_bytes,
        resume_handle_bytes: reserved.resume_handle_bytes,
        retained_worker_bytes: reserved.retained_worker_bytes,
        joined: reserved.joined,
    }
}

fn store_reservation(
    reserved: &ReservedRetainedWorkerRegistrationV1,
) -> store::RetainedWorkerReservationV1 {
    store::RetainedWorkerReservationV1 {
        request: reserved.request.clone(),
        descriptor_ref: reserved.descriptor_ref.clone(),
        resume_handle_ref: reserved.resume_handle_ref.clone(),
        retained_worker_ref: reserved.retained_worker_ref.clone(),
        descriptor_bytes: reserved.descriptor_bytes.clone(),
        resume_handle_bytes: reserved.resume_handle_bytes.clone(),
        retained_worker_bytes: reserved.retained_worker_bytes.clone(),
        joined: reserved.joined,
    }
}

fn store_error(error: store::BootstrapError) -> AuthorityFacadeError {
    AuthorityFacadeError(error.to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn trusted_fs_error(error: super::trusted_fs::TrustedFsError) -> AuthorityFacadeError {
    AuthorityFacadeError(error.to_string())
}

fn unsupported_platform() -> AuthorityFacadeError {
    AuthorityFacadeError("A1 host-session authority is unsupported on this platform".into())
}

pub(super) fn exact_current_authority_proof<'root>(
    root: &'root StateRootV1,
    orchestration_session_id: &str,
    authority_revision: u64,
) -> Result<&'root AuthorityObjectCommitmentV1, AuthorityFacadeError> {
    let mut commitment = None;
    let mut highest_revision = None;
    for journal in root.application_journal.values() {
        let Some(intent) = root.transition_intent_map.get(&journal.intent_id) else {
            continue;
        };
        if intent.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let phases = std::iter::once(&journal.initial_application).map(|initial| {
            (
                initial.authority_revision_after,
                &initial.authority_record_commitment,
            )
        });
        let phases = phases.chain(journal.post_turn_application.as_ref().map(|post_turn| {
            (
                post_turn.authority_revision_after,
                &post_turn.authority_record_commitment,
            )
        }));
        for (revision, candidate) in phases {
            highest_revision =
                Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
            if revision != authority_revision {
                continue;
            }
            if commitment.replace(candidate).is_some() {
                return Err(AuthorityFacadeError(
                    "current authority revision has ambiguous application proof".into(),
                ));
            }
        }
    }
    if highest_revision.is_some_and(|revision| revision > authority_revision) {
        return Err(AuthorityFacadeError(
            "current authority revision is behind durable application proof".into(),
        ));
    }
    let Some(commitment) = commitment else {
        return Err(AuthorityFacadeError(
            "current authority revision has no durable application proof".into(),
        ));
    };
    Ok(commitment)
}

fn validate_current_authority_proofs(root: &StateRootV1) -> Result<(), AuthorityFacadeError> {
    for record in root.session_namespace_map.values() {
        let SessionNamespaceRecordV1::Authority(authority) = record else {
            continue;
        };
        let computed = canonical_commitment(&authority_hash_input(authority))?;
        let persisted = exact_current_authority_proof(
            root,
            &authority.orchestration_session_id,
            authority.authority_revision,
        )?;
        if persisted != &computed {
            return Err(AuthorityFacadeError(
                "proposed authority has no matching durable application proof".into(),
            ));
        }
    }
    Ok(())
}

fn canonical_commitment<T>(value: &T) -> Result<AuthorityObjectCommitmentV1, AuthorityFacadeError>
where
    T: super::validation::CanonicalHashInputV1,
{
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(value)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?,
    })
}

fn authority_hash_input(
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

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::HostSessionAuthority;
    use crate::execution::agent_runtime::host_session_authority::trusted_fs::{
        ensure_private_substrate_home, TrustedAuthorityRoot,
    };

    fn write_private(path: &std::path::Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[test]
    fn opened_facade_rejects_lexical_home_replacement_without_touching_replacement() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        let authority = HostSessionAuthority::open(&home).unwrap();
        authority.bootstrap().unwrap();
        assert_eq!(
            authority.classify(),
            super::BootstrapClassificationV1::ValidExisting
        );
        let retained = parent.path().join("retained");
        fs::rename(&home, &retained).unwrap();
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        assert!(authority.read_root().is_err());
        assert!(!home.join("authority-v1").exists());
        assert!(retained.join("authority-v1/state-root-v1.json").exists());
    }

    #[test]
    fn accepted_home_replacement_before_facade_handoff_fails_closed() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        let accepted = ensure_private_substrate_home(&home, unsafe { libc::geteuid() }).unwrap();
        let retained = parent.path().join("retained");
        fs::rename(&home, &retained).unwrap();
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        assert!(HostSessionAuthority::from_trusted_root(accepted).is_err());
        assert!(!home.join("authority-v1").exists());
        assert!(!retained.join("authority-v1").exists());

        let replacement = TrustedAuthorityRoot::open(&home).unwrap();
        assert_eq!(
            replacement.identity().physical_path,
            home.display().to_string()
        );
    }

    #[test]
    fn absent_bootstrap_entries_revalidate_identity_before_success() {
        for entry in ["config", "policy", "agents"] {
            let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                        .join(".cache")
                });
            fs::create_dir_all(&safe_parent).unwrap();
            let parent = tempfile::tempdir_in(safe_parent).unwrap();
            fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let home = parent.path().join("home");
            fs::create_dir(&home).unwrap();
            fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
            let authority = HostSessionAuthority::open(&home).unwrap();
            let bootstrap_home = authority.bootstrap_home();
            match entry {
                "config" => assert_eq!(bootstrap_home.read_config_yaml().unwrap(), None),
                "policy" => assert_eq!(bootstrap_home.read_policy_yaml().unwrap(), None),
                "agents" => assert!(bootstrap_home
                    .read_agent_inventory_yaml()
                    .unwrap()
                    .is_empty()),
                _ => unreachable!(),
            }
            let retained = parent.path().join("retained");
            let replacement = || {
                fs::rename(&home, &retained).unwrap();
                fs::create_dir(&home).unwrap();
                fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
            };

            let result = match entry {
                "config" => bootstrap_home
                    .read_optional_file_after_revalidation("config.yaml", replacement)
                    .map(|_| ()),
                "policy" => bootstrap_home
                    .read_optional_file_after_revalidation("policy.yaml", replacement)
                    .map(|_| ()),
                "agents" => bootstrap_home
                    .read_agent_inventory_yaml_after_revalidation(replacement)
                    .map(|_| ()),
                _ => unreachable!(),
            };
            assert!(
                result.is_err(),
                "absent {entry} must close with revalidation"
            );
            assert!(fs::read_dir(&home).unwrap().next().is_none());
            assert!(fs::read_dir(&retained).unwrap().next().is_none());
        }
    }

    #[cfg(unix)]
    #[test]
    fn explicit_bootstrap_descriptors_reject_symlinks_and_non_files() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        let external_config = parent.path().join("external-config.yaml");
        write_private(&external_config, b"world:\n  enabled: true\n");
        std::os::unix::fs::symlink(&external_config, home.join("config.yaml")).unwrap();
        fs::create_dir(home.join("policy.yaml")).unwrap();
        fs::set_permissions(home.join("policy.yaml"), fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir(home.join("agents")).unwrap();
        fs::set_permissions(home.join("agents"), fs::Permissions::from_mode(0o700)).unwrap();
        let external_agent = parent.path().join("external-agent.yaml");
        write_private(
            &external_agent,
            b"version: 1\nid: escaped\nconfig:\n  kind: cli\n",
        );
        std::os::unix::fs::symlink(&external_agent, home.join("agents/escaped.yaml")).unwrap();

        let authority = HostSessionAuthority::open(&home).unwrap();
        let bootstrap_home = authority.bootstrap_home();
        assert!(bootstrap_home.read_config_yaml().is_err());
        assert!(bootstrap_home.read_policy_yaml().is_err());
        assert!(bootstrap_home.read_agent_inventory_yaml().is_err());
    }

    #[test]
    fn explicit_config_preserves_conditional_policy_parsing() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        write_private(&home.join("config.yaml"), b"{}\n");
        write_private(&home.join("policy.yaml"), b"world_fs: [\n");
        let authority = HostSessionAuthority::open(&home).unwrap();
        let bootstrap_home = authority.bootstrap_home();

        crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
            parent.path(),
            &crate::execution::config_model::CliConfigOverrides::default(),
            &bootstrap_home,
        )
        .expect("in_world config must not parse malformed policy");

        write_private(
            &home.join("config.yaml"),
            b"llm:\n  gateway:\n    mode: host_only\n",
        );
        assert!(
            crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
                parent.path(),
                &crate::execution::config_model::CliConfigOverrides::default(),
                &bootstrap_home,
            )
            .is_err()
        );
    }

    #[test]
    fn explicit_bootstrap_home_drives_config_policy_snapshot_inventory_and_state_store() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let accepted_home = parent.path().join("accepted");
        let ambient_home = parent.path().join("ambient");
        for home in [&accepted_home, &ambient_home] {
            fs::create_dir(home).unwrap();
            fs::set_permissions(home, fs::Permissions::from_mode(0o700)).unwrap();
            fs::create_dir(home.join("agents")).unwrap();
            fs::set_permissions(home.join("agents"), fs::Permissions::from_mode(0o700)).unwrap();
        }
        write_private(
            &accepted_home.join("config.yaml"),
            b"world:\n  enabled: false\n",
        );
        write_private(
            &ambient_home.join("config.yaml"),
            b"world:\n  enabled: true\n",
        );
        write_private(&accepted_home.join("policy.yaml"), b"id: policy-a\n");
        write_private(&ambient_home.join("policy.yaml"), b"id: policy-b\n");
        write_private(
            &accepted_home.join("agents/accepted.yaml"),
            br#"version: 1
id: accepted
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: accepted
  capabilities:
    llm: true
"#,
        );
        write_private(
            &ambient_home.join("agents/ambient.yaml"),
            br#"version: 1
id: ambient
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: ambient
  capabilities:
    llm: true
"#,
        );

        let authority = HostSessionAuthority::open(&accepted_home).unwrap();
        let bootstrap_home = authority.bootstrap_home();
        let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
            &ambient_home,
            &crate::execution::config_model::CliConfigOverrides::default(),
            &bootstrap_home,
        )
        .unwrap();
        assert!(!config.world.enabled);
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            &ambient_home,
            &bootstrap_home,
        )
        .unwrap();
        assert_eq!(policy.id, "policy-a");
        let snapshot =
            crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
                &ambient_home,
                &bootstrap_home,
            )
            .unwrap();
        assert!(!snapshot.snapshot_hash.is_empty());
        let post_acceptance_cwd = parent.path().join("post-acceptance-cwd");
        fs::create_dir(&post_acceptance_cwd).unwrap();
        fs::set_permissions(&post_acceptance_cwd, fs::Permissions::from_mode(0o700)).unwrap();
        let inventory =
            crate::execution::agent_inventory::load_effective_agent_inventory_for_bootstrap_home(
                &post_acceptance_cwd,
                &policy,
                &bootstrap_home,
            )
            .unwrap();
        assert!(inventory.contains_key("accepted"));
        assert!(!inventory.contains_key("ambient"));
        assert_eq!(
            inventory.get("accepted").unwrap().path,
            accepted_home.join("agents/accepted.yaml")
        );
        let state_store =
            crate::execution::agent_runtime::AgentRuntimeStateStore::for_bootstrap_home(
                &bootstrap_home,
            )
            .unwrap();
        assert_eq!(
            state_store.bootstrap_home_identity(),
            bootstrap_home.identity().unwrap()
        );
        assert_eq!(
            state_store.bootstrap_home_identity().physical_path,
            accepted_home.display().to_string()
        );

        let retained = parent.path().join("retained");
        fs::rename(&accepted_home, &retained).unwrap();
        fs::create_dir(&accepted_home).unwrap();
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700)).unwrap();
        write_private(
            &accepted_home.join("config.yaml"),
            b"world:\n  enabled: true\n",
        );
        assert!(
            crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
                &ambient_home,
                &crate::execution::config_model::CliConfigOverrides::default(),
                &bootstrap_home,
            )
            .is_err()
        );
        assert!(!accepted_home.join("agents").exists());
    }
}
