//! Component-only retained-worker authority registration.
//!
//! This module deliberately has no production ingress caller. It owns immutable
//! retained-object construction while `HostSessionAuthority` owns durable
//! reservation and authority mutation.

#![allow(
    dead_code,
    reason = "R0 is a component proof and deliberately has no production ingress caller"
)]

use std::fmt;

use super::host_session_authority::canonical_json;
#[cfg(test)]
use super::host_session_authority::facade::RetainedReservationCrashPointV1;
use super::host_session_authority::facade::{
    ReservedRetainedWorkerRegistrationV1, RetainedWorkerAuthorityPreconditionV1,
};
#[cfg(test)]
use super::host_session_authority::schema::TimestampV1;
use super::host_session_authority::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1, PolicyObjectHashInputV1,
    ResumeHandleHashInputV1, RetainedWorkerObjectHashInputV1,
};
use super::host_session_authority::store_schema::{
    RetainedWorkerAuthorityRegistrationRequestStateV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1,
};
use super::host_session_authority::validation::ValidatedCanonicalV1;
use super::host_session_authority::HostSessionAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRegistrationPlanV1 {
    pub(crate) registration_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) expected_authority: RetainedWorkerAuthorityPreconditionV1,
    pub(crate) retained_participant_id: String,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) internal_uaa_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRegistrationResultV1 {
    pub(crate) registration_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) retained_participant_id: String,
    pub(crate) retained_worker_ref: super::host_session_authority::schema::AuthorityObjectRefV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment_after:
        super::host_session_authority::schema::AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedRetainedTargetV1 {
    pub(crate) registration: RetainedWorkerAuthorityRegistrationV1,
    pub(crate) current_authority_revision: u64,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) resume_handle: ResumeHandleHashInputV1,
    pub(crate) retained_worker: RetainedWorkerObjectHashInputV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RetainedObjectPublicationCrashPointV1 {
    Descriptor,
    ResumeHandle,
    RetainedWorker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRuntimeError(String);

impl fmt::Display for RetainedWorkerRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RetainedWorkerRuntimeError {}

#[derive(Debug, Default)]
pub(crate) struct RetainedWorkerRuntime;

impl RetainedWorkerRuntime {
    pub(crate) fn reserve_registration(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, RetainedWorkerRuntimeError> {
        let (descriptor_bytes, resume_handle_bytes) = Self::immutable_inputs(plan)?;
        authority
            .reserve_retained_worker_registration(
                &plan.registration_request_id,
                &plan.orchestration_session_id,
                &plan.expected_authority,
                &plan.retained_participant_id,
                descriptor_bytes,
                resume_handle_bytes,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    Self::retained_worker_bytes(
                        plan,
                        descriptor_ref,
                        resume_handle_ref,
                        policy_ref,
                        world_binding,
                    )
                },
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    #[cfg(test)]
    fn reserve_registration_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
        registered_at: TimestampV1,
        crash_point: Option<RetainedReservationCrashPointV1>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, RetainedWorkerRuntimeError> {
        let (descriptor_bytes, resume_handle_bytes) = Self::immutable_inputs(plan)?;
        authority
            .reserve_retained_worker_registration_at(
                &plan.registration_request_id,
                &plan.orchestration_session_id,
                &plan.expected_authority,
                &plan.retained_participant_id,
                descriptor_bytes,
                resume_handle_bytes,
                registered_at,
                crash_point,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    Self::retained_worker_bytes(
                        plan,
                        descriptor_ref,
                        resume_handle_ref,
                        policy_ref,
                        world_binding,
                    )
                },
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    fn immutable_inputs(
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<(Vec<u8>, Vec<u8>), RetainedWorkerRuntimeError> {
        let descriptor = AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: plan.descriptor.clone(),
        };
        descriptor
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World {
            return Err(RetainedWorkerRuntimeError(
                "retained descriptor must be world-scoped".into(),
            ));
        }
        let descriptor_bytes = canonical_json::to_vec(&descriptor)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle = ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: plan.orchestration_session_id.clone(),
            participant_id: plan.retained_participant_id.clone(),
            backend_id: plan.descriptor.backend_id.clone(),
            protocol: plan.descriptor.protocol.clone(),
            internal_uaa_session_id: plan.internal_uaa_session_id.clone(),
        };
        resume_handle
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle_bytes = canonical_json::to_vec(&resume_handle)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        Ok((descriptor_bytes, resume_handle_bytes))
    }

    fn retained_worker_bytes(
        plan: &RetainedWorkerRegistrationPlanV1,
        descriptor_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        resume_handle_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        policy_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        world_binding: &super::host_session_authority::schema::WorldBindingV1,
    ) -> Result<Vec<u8>, &'static str> {
        let worker = RetainedWorkerObjectHashInputV1 {
            schema_version: 1,
            orchestration_session_id: plan.orchestration_session_id.clone(),
            participant_id: plan.retained_participant_id.clone(),
            world_binding: world_binding.clone(),
            descriptor_ref: descriptor_ref.clone(),
            resume_handle_ref: resume_handle_ref.clone(),
            policy_ref: policy_ref.clone(),
        };
        worker
            .validate()
            .map_err(|_| "validate retained-worker object")?;
        canonical_json::to_vec(&worker).map_err(|_| "encode retained-worker object")
    }

    pub(crate) fn publish_reserved_object_graph(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        self.publish_reserved_object_graph_with(authority, reserved, |_| Ok(()))
    }

    fn publish_reserved_object_graph_with(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        mut after_publication: impl FnMut(usize) -> Result<(), RetainedWorkerRuntimeError>,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        for (index, (reference, bytes)) in [
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
        ]
        .into_iter()
        .enumerate()
        {
            authority
                .publish_reserved_retained_object(reserved, reference, bytes)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
            after_publication(index)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn publish_reserved_object_graph_with_crash_point(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        crash_point: RetainedObjectPublicationCrashPointV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let stop_after = match crash_point {
            RetainedObjectPublicationCrashPointV1::Descriptor => 0,
            RetainedObjectPublicationCrashPointV1::ResumeHandle => 1,
            RetainedObjectPublicationCrashPointV1::RetainedWorker => 2,
        };
        self.publish_reserved_object_graph_with(authority, reserved, |index| {
            if index == stop_after {
                Err(RetainedWorkerRuntimeError(
                    "injected crash after retained object publication".into(),
                ))
            } else {
                Ok(())
            }
        })
    }

    pub(crate) fn register_retained_target(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<RetainedWorkerRegistrationResultV1, RetainedWorkerRuntimeError> {
        self.register_retained_target_with(authority, plan, |_| Ok(()))
    }

    fn register_retained_target_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
        after_reservation: impl FnOnce(
            &ReservedRetainedWorkerRegistrationV1,
        ) -> Result<(), RetainedWorkerRuntimeError>,
    ) -> Result<RetainedWorkerRegistrationResultV1, RetainedWorkerRuntimeError> {
        let mut reserved = self.reserve_registration(authority, plan)?;
        after_reservation(&reserved)?;
        if matches!(
            reserved.request.state,
            RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ) {
            if let Err(publication_error) = self.publish_reserved_object_graph(authority, &reserved)
            {
                let refreshed = self.reserve_registration(authority, plan)?;
                if matches!(
                    refreshed.request.state,
                    RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
                ) {
                    return Err(publication_error);
                }
                reserved = refreshed;
            }
        }
        let applied = authority
            .apply_reserved_retained_worker_registration(&reserved)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        Ok(RetainedWorkerRegistrationResultV1 {
            registration_id: applied.registration.registration_id.clone(),
            authority_store_id: plan.expected_authority.authority_store_id.clone(),
            orchestration_session_id: applied.registration.orchestration_session_id.clone(),
            retained_participant_id: applied.registration.retained_participant_id.clone(),
            retained_worker_ref: applied.registration.retained_worker_ref.clone(),
            authority_revision_after: applied.registration.authority_revision_after,
            authority_record_commitment_after: applied
                .registration
                .authority_record_commitment_after,
        })
    }

    pub(crate) fn resolve_retained_target(
        &self,
        authority: &HostSessionAuthority,
        target: &RetainedWorkerRegistrationResultV1,
    ) -> Result<ResolvedRetainedTargetV1, RetainedWorkerRuntimeError> {
        let current = authority
            .resolve_current_exact(&target.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if current.observation.authority_store_id != target.authority_store_id
            || current.observation.orchestration_session_id != target.orchestration_session_id
            || current.observation.authority_revision < target.authority_revision_after
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target authority store, session, or revision is inexact".into(),
            ));
        }
        let root = authority
            .read_a12a_root()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let SessionNamespaceRecordV1::Authority(root_authority) = root
            .session_namespace_map
            .get(&target.orchestration_session_id)
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("retained target session is absent".into())
            })?
        else {
            return Err(RetainedWorkerRuntimeError(
                "retained target session has no durable authority".into(),
            ));
        };
        if root.root_revision != current.observation.root_revision
            || root.authority_store_id != target.authority_store_id
            || root_authority.as_ref() != &current.authority
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target authority snapshot changed during resolution".into(),
            ));
        }
        let registrations = root
            .retained_worker_registration_journal
            .iter()
            .filter(|(registration_id, registration)| {
                *registration_id == &target.registration_id
                    && registration.registration_id == target.registration_id
            })
            .collect::<Vec<_>>();
        let [(_, registration)] = registrations.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "retained target has no unique registration proof".into(),
            ));
        };
        let requests = root
            .retained_worker_registration_request_index
            .values()
            .filter(|request| request.registration_id == target.registration_id)
            .collect::<Vec<_>>();
        let [request] = requests.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "retained target has no unique registration request".into(),
            ));
        };
        if registration.orchestration_session_id != target.orchestration_session_id
            || registration.retained_participant_id != target.retained_participant_id
            || registration.retained_worker_ref != target.retained_worker_ref
            || registration.authority_revision_after != target.authority_revision_after
            || registration.authority_record_commitment_after
                != target.authority_record_commitment_after
            || request.issuer_request_id != registration.issuer_request_id
            || request.registration_id != registration.registration_id
            || !matches!(
                &request.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } if *authority_revision_after == registration.authority_revision_after
                    && authority_record_commitment_after
                        == &registration.authority_record_commitment_after
            )
            || current
                .authority
                .authoritative_participant_lineage
                .iter()
                .filter(|participant| *participant == &target.retained_participant_id)
                .count()
                != 1
            || current
                .authority
                .retained_worker_refs
                .iter()
                .filter(|reference| *reference == &target.retained_worker_ref)
                .count()
                != 1
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target request, proof, or authority membership is inexact".into(),
            ));
        }
        let highest_proof_revision = root
            .retained_worker_registration_journal
            .values()
            .filter(|candidate| {
                candidate.orchestration_session_id == target.orchestration_session_id
            })
            .map(|candidate| candidate.authority_revision_after)
            .max()
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("retained target authority proof is absent".into())
            })?;
        if highest_proof_revision != current.authority.authority_revision {
            return Err(RetainedWorkerRuntimeError(
                "retained target proof revision is behind current authority".into(),
            ));
        }

        let descriptor_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.descriptor_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.resume_handle_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let worker_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.retained_worker_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let policy_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.current_policy_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(&descriptor_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle: ResumeHandleHashInputV1 = canonical_json::from_slice(&resume_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let retained_worker: RetainedWorkerObjectHashInputV1 =
            canonical_json::from_slice(&worker_bytes)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let current_policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        descriptor
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        resume_handle
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        retained_worker
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        current_policy
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World
            || resume_handle.orchestration_session_id != target.orchestration_session_id
            || resume_handle.participant_id != target.retained_participant_id
            || resume_handle.backend_id != descriptor.descriptor.backend_id
            || resume_handle.protocol != descriptor.descriptor.protocol
            || retained_worker.orchestration_session_id != target.orchestration_session_id
            || retained_worker.participant_id != target.retained_participant_id
            || retained_worker.world_binding != registration.world_binding
            || retained_worker.descriptor_ref != registration.descriptor_ref
            || retained_worker.resume_handle_ref != registration.resume_handle_ref
            || retained_worker.policy_ref != registration.current_policy_ref
            || current.authority.world_binding.as_ref() != Some(&retained_worker.world_binding)
            || current.authority.current_policy_ref.as_ref() != Some(&retained_worker.policy_ref)
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target immutable object graph is inexact".into(),
            ));
        }
        Ok(ResolvedRetainedTargetV1 {
            registration: (*registration).clone(),
            current_authority_revision: current.authority.authority_revision,
            descriptor: descriptor.descriptor,
            resume_handle,
            retained_worker,
            current_policy,
        })
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use super::*;
    use crate::execution::agent_runtime::host_session_authority::facade::{
        AuthorityObservationV1, HostSessionAuthority, RetainedApplicationCrashPointV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentExecutionScopeV1, HostAttachCapabilitiesV1, HostAttachExecutionClientStartV1,
        HostAttachLaunchKnobsV1, HostAttachModePreferenceV1, HostSessionAuthorityPreconditionV1,
        HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
        HostSessionTransitionModeV1, PolicyObjectHashInputV1, RuntimeBackendKindV1, TimestampV1,
        WorkspaceBindingV1, WorldBindingV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentStateV2;
    use crate::execution::agent_runtime::host_session_authority::transition::{
        ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
        IssueHostSessionTransitionRequestV1, StartContractMaterialV1,
        TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1, TransitionIssueOutcomeV1,
    };

    fn timestamp(value: &str) -> TimestampV1 {
        TimestampV1::parse(value).unwrap()
    }

    fn started_authority() -> (
        tempfile::TempDir,
        HostSessionAuthority,
        AuthorityObservationV1,
    ) {
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
        let root = authority.bootstrap().unwrap();
        let binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home,
            authority_store_id: root.authority_store_id,
        };
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: "r0-start-intent".into(),
            issuer_request_id: "retained-worker-registration:transition-start".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: "r0-session".into(),
            shell_trace_session_id: "r0-trace".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "r0-orchestrator".into(),
            target_participant_lease_token: b"r0-start-lease".to_vec(),
            run_id: "r0-start-run".into(),
            resulting_authoritative_lineage: vec!["r0-orchestrator".into()],
            workspace_binding: binding,
            world_binding: Some(WorldBindingV1 {
                world_id: "r0-world".into(),
                world_generation: 7,
            }),
            start_contract: StartContractMaterialV1 {
                descriptor: AgentDescriptorV1 {
                    schema_version: 1,
                    agent_id: "codex".into(),
                    backend_id: "cli:codex".into(),
                    backend_kind: RuntimeBackendKindV1::Codex,
                    protocol: "substrate.agent.session".into(),
                    execution_scope: AgentExecutionScopeV1::Host,
                    binary_path: "/usr/bin/codex".into(),
                },
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "r0-policy".into(),
                    canonical_policy_snapshot_sha256:
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
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
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
                },
            },
            transition_input: None,
        };
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
            .unwrap()
        else {
            panic!("Start issuance must commit")
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "r0-start-claim".into(),
            claimant_attempt_id: "r0-start-attempt".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                timestamp("2026-07-14T12:01:00.000000000Z"),
                30,
            )
            .unwrap()
        else {
            panic!("Start claim must commit")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("Start must remain claimed")
        };
        let application = ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id,
            issuer_request_id: claimed.issuer_request_id,
            payload_commitment: claimed.payload_commitment,
            expected_intent_revision: claimed.intent_revision,
            claim_id: claim_request.claim_id,
            expected_claim_revision: claim_revision,
        };
        assert!(matches!(
            authority
                .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
                .unwrap(),
            TransitionApplicationOutcomeV1::Applied(_)
        ));
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        (parent, authority, observation)
    }

    fn plan(observation: AuthorityObservationV1) -> RetainedWorkerRegistrationPlanV1 {
        RetainedWorkerRegistrationPlanV1 {
            registration_request_id: "spawn-request-1".into(),
            orchestration_session_id: "r0-session".into(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: observation.authority_store_id,
                authority_revision: observation.authority_revision,
                authority_record_commitment: observation.authority_record_commitment,
            },
            retained_participant_id: "r0-retained-1".into(),
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "codex-worker".into(),
                backend_id: "cli:codex-worker".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::World,
                binary_path: "/usr/bin/codex".into(),
            },
            internal_uaa_session_id: "uaa-retained-1".into(),
        }
    }

    fn object_files(path: &std::path::Path) -> Vec<String> {
        fn visit(root: &std::path::Path, path: &std::path::Path, found: &mut Vec<String>) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    visit(root, &entry.path(), found);
                } else {
                    found.push(
                        entry
                            .path()
                            .strip_prefix(root)
                            .unwrap()
                            .display()
                            .to_string(),
                    );
                }
            }
        }
        let mut found = Vec::new();
        visit(path, path, &mut found);
        found.sort();
        found
    }

    fn durable_authority_mut(
        root: &mut crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV2,
    ) -> &mut crate::execution::agent_runtime::host_session_authority::store_schema::DurableSessionAuthorityV1{
        match root.session_namespace_map.get_mut("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_mut(),
            _ => panic!("fixture must retain durable authority"),
        }
    }

    #[test]
    fn reservation_uses_production_start_authority_and_publishes_no_object() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let reserved = RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .expect("valid retained registration must reserve");

        assert_eq!(reserved.request.retained_participant_id, "r0-retained-1");
        assert!(matches!(
            reserved.request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ));
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(
            root.retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id),
            Some(&reserved.request)
        );
        assert!(root.retained_worker_registration_journal.is_empty());
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn exact_reservation_retry_joins_and_changed_plan_fails_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let registered_at = timestamp("2026-07-14T12:02:00.000000000Z");
        let first = runtime
            .reserve_registration_at(&authority, &plan, registered_at.clone(), None)
            .unwrap();
        assert!(!first.joined);
        let fixed_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let fixed_objects = object_files(&object_root);

        let joined = runtime
            .reserve_registration_at(&authority, &plan, registered_at.clone(), None)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(
            joined,
            ReservedRetainedWorkerRegistrationV1 {
                joined: true,
                ..first.clone()
            }
        );
        assert_eq!(authority.read_a12a_root().unwrap(), fixed_root);

        let mut conflicts = Vec::new();
        let mut changed = plan.clone();
        changed.retained_participant_id = "r0-retained-changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.orchestration_session_id = "r0-session-changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.expected_authority.authority_revision += 1;
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.expected_authority.authority_record_commitment =
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            };
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.backend_id = "cli:changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.protocol = "changed.protocol".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.binary_path = "/usr/bin/changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.internal_uaa_session_id = "uaa-changed".into();
        conflicts.push((changed, registered_at.clone()));
        conflicts.push((plan.clone(), timestamp("2026-07-14T12:02:01.000000000Z")));
        let mut changed = plan.clone();
        changed.registration_request_id = "spawn-request-2".into();
        conflicts.push((changed, registered_at));

        for (conflict, conflict_time) in conflicts {
            assert!(runtime
                .reserve_registration_at(&authority, &conflict, conflict_time, None)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), fixed_root);
            assert_eq!(object_files(&object_root), fixed_objects);
        }
    }

    #[test]
    fn reservation_crash_windows_restart_with_zero_or_exact_reserved_state() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let registered_at = timestamp("2026-07-14T12:02:00.000000000Z");
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(runtime
            .reserve_registration_at(
                &authority,
                &plan,
                registered_at.clone(),
                Some(RetainedReservationCrashPointV1::BeforeRootPublication),
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        assert!(runtime
            .reserve_registration_at(
                &authority,
                &plan,
                registered_at.clone(),
                Some(RetainedReservationCrashPointV1::AfterRootPublication),
            )
            .is_err());
        let durable = authority.read_a12a_root().unwrap();
        let request = durable
            .retained_worker_registration_request_index
            .values()
            .next()
            .unwrap()
            .clone();
        assert!(matches!(
            request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ));
        assert_eq!(object_files(&object_root), objects_before);

        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let joined = runtime
            .reserve_registration_at(&restarted, &plan, registered_at, None)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(joined.request, request);
        assert_eq!(restarted.read_a12a_root().unwrap(), durable);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn transition_issuer_cannot_be_reused_for_retained_registration() {
        let (_parent, authority, observation) = started_authority();
        let mut conflict = plan(observation);
        conflict.registration_request_id = "transition-start".into();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &conflict,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn strict_root_rejects_duplicate_participant_across_reserved_requests() {
        let (_parent, authority, observation) = started_authority();
        let reserved = RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let mut root = authority.read_a12a_root().unwrap();
        let mut duplicate = reserved.request;
        duplicate.issuer_request_id = "retained-worker-registration:duplicate".into();
        duplicate.registration_id = "rr_11111111111111111111111111111111".into();
        duplicate.descriptor_ref_id = "ao_11111111111111111111111111111111".into();
        duplicate.resume_handle_ref_id = "ao_22222222222222222222222222222222".into();
        duplicate.retained_worker_ref_id = "ao_33333333333333333333333333333333".into();
        root.retained_worker_registration_request_index
            .insert(duplicate.issuer_request_id.clone(), duplicate);

        assert!(root.validate().is_err());
    }

    #[test]
    fn reserved_object_identity_collision_check_is_global_across_kinds() {
        let (_parent, authority, _observation) = started_authority();
        let root = authority.read_a12a_root().unwrap();
        let policy = PolicyObjectHashInputV1 {
            schema_version: 1,
            policy_revision: "orphan-policy".into(),
            canonical_policy_snapshot_sha256:
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
        };
        let bytes = canonical_json::to_vec(&policy).unwrap();
        let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_99999999999999999999999999999999".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(&policy).unwrap(),
            },
        };
        let home = _parent.path().join("home");
        crate::execution::agent_runtime::host_session_authority::store::prepare_typed_object_v2_test(
            &home,
            root.root_revision,
            &reference,
            &bytes,
        )
        .unwrap();

        assert!(!crate::execution::agent_runtime::host_session_authority::store::reserved_object_ref_id_is_globally_absent_test(
            &home,
            &reference.ref_id,
        )
        .unwrap());
    }

    #[test]
    fn reserved_retry_rejects_changed_current_policy_or_world_binding() {
        let (_parent, authority, observation) = started_authority();
        let plan = plan(observation);
        RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root = authority.read_a12a_root().unwrap();
        let validate = |candidate: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV2| {
            crate::execution::agent_runtime::host_session_authority::store::retained_reservation_authority_matches_test(
                candidate,
                &plan.orchestration_session_id,
                &plan.expected_authority.authority_store_id,
                plan.expected_authority.authority_revision,
                &plan.expected_authority.authority_record_commitment,
            )
        };
        validate(&root).unwrap();

        let mut changed_world = root.clone();
        let crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) = changed_world
            .session_namespace_map
            .get_mut(&plan.orchestration_session_id)
            .unwrap()
        else {
            panic!("production Start must establish authority")
        };
        authority.world_binding.as_mut().unwrap().world_generation += 1;
        assert!(validate(&changed_world).is_err());

        let mut changed_policy = root;
        let crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) = changed_policy
            .session_namespace_map
            .get_mut(&plan.orchestration_session_id)
            .unwrap()
        else {
            panic!("production Start must establish authority")
        };
        authority.current_policy_ref.as_mut().unwrap().ref_id =
            "ao_88888888888888888888888888888888".into();
        assert!(validate(&changed_policy).is_err());
    }

    #[test]
    fn runtime_owned_object_validation_rejects_before_hsa_mutation() {
        let (_parent, authority, observation) = started_authority();
        let valid = plan(observation);
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        let mut malformed_descriptor = valid.clone();
        malformed_descriptor.descriptor.agent_id.clear();
        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &malformed_descriptor,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let mut malformed_resume = valid.clone();
        malformed_resume.internal_uaa_session_id.clear();
        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &malformed_resume,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let canonical = |ref_id: &str,
                         object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1| {
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
                ref_id: ref_id.into(),
                object_kind,
                schema_version: 1,
                commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
                },
            }
        };
        assert!(RetainedWorkerRuntime::retained_worker_bytes(
            &valid,
            &canonical(
                "ao_11111111111111111111111111111111",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::AgentDescriptor,
            ),
            &canonical(
                "ao_22222222222222222222222222222222",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::ResumeHandle,
            ),
            &canonical(
                "ao_33333333333333333333333333333333",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
            ),
            &WorldBindingV1 {
                world_id: String::new(),
                world_generation: 7,
            },
        )
        .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn exact_reserved_graph_publication_writes_three_orphans_without_root_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .expect("exact reserved graph must publish");

        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        let objects_after = object_files(&object_root);
        assert_eq!(objects_after.len(), objects_before.len() + 3);
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.descriptor_ref.ref_id))));
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.resume_handle_ref.ref_id))));
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.retained_worker_ref.ref_id))));
    }

    #[test]
    fn object_publication_crash_points_restart_and_exact_join() {
        for crash_point in [
            RetainedObjectPublicationCrashPointV1::Descriptor,
            RetainedObjectPublicationCrashPointV1::ResumeHandle,
            RetainedObjectPublicationCrashPointV1::RetainedWorker,
        ] {
            let (_parent, authority, observation) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let reserved = runtime
                .reserve_registration_at(
                    &authority,
                    &plan(observation),
                    timestamp("2026-07-14T12:02:00.000000000Z"),
                    None,
                )
                .unwrap();
            let root_before = authority.read_a12a_root().unwrap();
            let object_root = _parent.path().join("home/authority-v1/objects");
            let objects_before = object_files(&object_root);

            assert!(runtime
                .publish_reserved_object_graph_with_crash_point(&authority, &reserved, crash_point,)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            let durable_count = match crash_point {
                RetainedObjectPublicationCrashPointV1::Descriptor => 1,
                RetainedObjectPublicationCrashPointV1::ResumeHandle => 2,
                RetainedObjectPublicationCrashPointV1::RetainedWorker => 3,
            };
            assert_eq!(
                object_files(&object_root).len(),
                objects_before.len() + durable_count
            );

            let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
            runtime
                .publish_reserved_object_graph(&restarted, &reserved)
                .unwrap();
            assert_eq!(restarted.read_a12a_root().unwrap(), root_before);
            let completed = object_files(&object_root);
            assert_eq!(completed.len(), objects_before.len() + 3);
            runtime
                .publish_reserved_object_graph(&restarted, &reserved)
                .unwrap();
            assert_eq!(object_files(&object_root), completed);
            assert_eq!(restarted.read_a12a_root().unwrap(), root_before);
        }
    }

    #[test]
    fn substituted_reserved_graph_rejects_before_any_object_or_root_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut conflicts = Vec::new();

        let mut wrong_kind = reserved.clone();
        wrong_kind.descriptor_ref.object_kind = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy;
        conflicts.push(wrong_kind);
        let mut wrong_schema = reserved.clone();
        wrong_schema.resume_handle_ref.schema_version = 2;
        conflicts.push(wrong_schema);
        let mut wrong_bytes = reserved.clone();
        wrong_bytes.descriptor_bytes.push(b' ');
        conflicts.push(wrong_bytes);
        let mut wrong_commitment = reserved.clone();
        wrong_commitment.retained_worker_ref.commitment =
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
            };
        conflicts.push(wrong_commitment);

        let mut wrong_descriptor = reserved.clone();
        let mut descriptor: AgentDescriptorHashInputV1 =
            canonical_json::from_slice(&wrong_descriptor.descriptor_bytes).unwrap();
        descriptor.descriptor.backend_id = "cli:substituted".into();
        descriptor.descriptor.protocol = "substituted.protocol".into();
        wrong_descriptor.descriptor_bytes = canonical_json::to_vec(&descriptor).unwrap();
        conflicts.push(wrong_descriptor);

        let mut wrong_resume = reserved.clone();
        let mut resume: ResumeHandleHashInputV1 =
            canonical_json::from_slice(&wrong_resume.resume_handle_bytes).unwrap();
        resume.orchestration_session_id = "other-session".into();
        resume.participant_id = "other-participant".into();
        wrong_resume.resume_handle_bytes = canonical_json::to_vec(&resume).unwrap();
        conflicts.push(wrong_resume);

        let mut wrong_worker = reserved.clone();
        let mut worker: RetainedWorkerObjectHashInputV1 =
            canonical_json::from_slice(&wrong_worker.retained_worker_bytes).unwrap();
        worker.world_binding.world_id = "other-world".into();
        worker.world_binding.world_generation += 1;
        worker.policy_ref.ref_id = "ao_77777777777777777777777777777777".into();
        worker.descriptor_ref.ref_id = "ao_66666666666666666666666666666666".into();
        worker.resume_handle_ref.ref_id = "ao_55555555555555555555555555555555".into();
        wrong_worker.retained_worker_bytes = canonical_json::to_vec(&worker).unwrap();
        conflicts.push(wrong_worker);

        let mut wrong_request_scope = reserved.clone();
        wrong_request_scope.request.world_binding.world_id = "other-world".into();
        wrong_request_scope.request.current_policy_ref.ref_id =
            "ao_44444444444444444444444444444444".into();
        conflicts.push(wrong_request_scope);

        for conflict in conflicts {
            assert!(runtime
                .publish_reserved_object_graph(&authority, &conflict)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            assert_eq!(object_files(&object_root), objects_before);
        }
    }

    #[test]
    fn atomic_registration_appends_only_the_reserved_authority_link() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let before = authority.read_a12a_root().unwrap();
        let before_authority = match before.session_namespace_map.get("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_ref().clone(),
            _ => panic!("production Start must establish durable authority"),
        };
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        let result = runtime
            .register_retained_target(&authority, &plan)
            .expect("exact retained registration must apply atomically");

        let after = authority.read_a12a_root().unwrap();
        let request = after
            .retained_worker_registration_request_index
            .get("retained-worker-registration:spawn-request-1")
            .unwrap();
        let journal = after
            .retained_worker_registration_journal
            .get(&request.registration_id)
            .unwrap();
        let after_authority = match after.session_namespace_map.get("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_ref(),
            _ => panic!("retained registration must preserve durable authority"),
        };

        assert_eq!(after.root_revision, before.root_revision + 2);
        assert_eq!(after.transition_intent_map, before.transition_intent_map);
        assert_eq!(after.issuer_request_index, before.issuer_request_index);
        assert_eq!(after.application_journal, before.application_journal);
        assert_eq!(
            after_authority.authority_revision,
            before_authority.authority_revision + 1
        );
        assert_eq!(
            after_authority.authoritative_participant_lineage,
            [
                before_authority
                    .authoritative_participant_lineage
                    .as_slice(),
                std::slice::from_ref(&plan.retained_participant_id),
            ]
            .concat()
        );
        assert_eq!(
            after_authority.retained_worker_refs,
            [
                before_authority.retained_worker_refs.as_slice(),
                std::slice::from_ref(&journal.retained_worker_ref),
            ]
            .concat()
        );
        let mut expected_authority = before_authority.clone();
        expected_authority.authority_revision += 1;
        expected_authority
            .authoritative_participant_lineage
            .push(plan.retained_participant_id.clone());
        expected_authority
            .retained_worker_refs
            .push(journal.retained_worker_ref.clone());
        expected_authority.updated_at = request.registered_at.clone();
        assert_eq!(*after_authority, expected_authority);
        assert!(matches!(
            &request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after,
                authority_record_commitment_after,
            } if *authority_revision_after == after_authority.authority_revision
                && authority_record_commitment_after == &journal.authority_record_commitment_after
        ));
        assert_eq!(
            journal.authority_revision_before,
            before_authority.authority_revision
        );
        assert_eq!(
            journal.authority_revision_after,
            after_authority.authority_revision
        );
        assert_eq!(journal.registered_at, request.registered_at);
        assert_eq!(after.object_index.len(), before.object_index.len() + 3);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
        assert_eq!(result.registration_id, request.registration_id);
        assert_eq!(result.retained_participant_id, plan.retained_participant_id);
        assert_eq!(
            result.authority_revision_after,
            after_authority.authority_revision
        );
    }

    #[test]
    fn final_root_crash_windows_restart_and_join_one_application() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .unwrap();
        let reserved_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let durable_objects = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration_with_crash_point(
                &reserved,
                RetainedApplicationCrashPointV1::BeforeRootPublication,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), reserved_root);
        assert_eq!(object_files(&object_root), durable_objects);

        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let applied = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(!applied.joined);
        let applied_root = restarted.read_a12a_root().unwrap();
        let joined = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(joined.registration, applied.registration);
        assert_eq!(restarted.read_a12a_root().unwrap(), applied_root);
        assert_eq!(object_files(&object_root), durable_objects);

        let (_parent, authority, observation) = started_authority();
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .unwrap();
        assert!(authority
            .apply_reserved_retained_worker_registration_with_crash_point(
                &reserved,
                RetainedApplicationCrashPointV1::AfterRootPublication,
            )
            .is_err());
        let applied_root = authority.read_a12a_root().unwrap();
        assert!(matches!(
            applied_root
                .retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id)
                .unwrap()
                .state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
        ));
        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let joined = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(restarted.read_a12a_root().unwrap(), applied_root);
    }

    #[test]
    fn application_rejects_missing_reserved_object_without_root_or_object_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        authority
            .publish_reserved_retained_object(
                &reserved,
                &reserved.descriptor_ref,
                &reserved.descriptor_bytes,
            )
            .unwrap();
        authority
            .publish_reserved_retained_object(
                &reserved,
                &reserved.resume_handle_ref,
                &reserved.resume_handle_bytes,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration(&reserved)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn stale_parallel_reservation_rejects_application_with_zero_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = plan(observation.clone());
        let mut second_plan = plan(observation);
        second_plan.registration_request_id = "spawn-request-2".into();
        second_plan.retained_participant_id = "r0-retained-2".into();
        second_plan.descriptor.agent_id = "codex-worker-2".into();
        second_plan.internal_uaa_session_id = "uaa-retained-2".into();
        let first = runtime
            .reserve_registration_at(
                &authority,
                &first_plan,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let stale = runtime
            .reserve_registration_at(
                &authority,
                &second_plan,
                timestamp("2026-07-14T12:02:01.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &first)
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &stale)
            .unwrap();
        authority
            .apply_reserved_retained_worker_registration(&first)
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration(&stale)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn strict_root_rejects_incomplete_or_noncontiguous_applied_registration() {
        let (_parent, authority, observation) = started_authority();
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan(observation))
            .unwrap();
        let applied = authority.read_a12a_root().unwrap();
        applied.validate().unwrap();
        let request = applied
            .retained_worker_registration_request_index
            .values()
            .next()
            .unwrap()
            .clone();

        let mut missing_journal = applied.clone();
        missing_journal.retained_worker_registration_journal.clear();
        assert!(missing_journal.validate().is_err());

        let mut reserved_with_journal = applied.clone();
        reserved_with_journal
            .retained_worker_registration_request_index
            .get_mut(&request.issuer_request_id)
            .unwrap()
            .state = crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved;
        assert!(reserved_with_journal.validate().is_err());

        let mut journal_without_request = applied.clone();
        journal_without_request
            .retained_worker_registration_request_index
            .clear();
        assert!(journal_without_request.validate().is_err());

        let mut missing_index = applied.clone();
        missing_index
            .object_index
            .remove(&request.retained_worker_ref_id);
        assert!(missing_index.validate().is_err());

        let mut duplicate_lineage = applied.clone();
        durable_authority_mut(&mut duplicate_lineage)
            .authoritative_participant_lineage
            .push(request.retained_participant_id.clone());
        assert!(duplicate_lineage.validate().is_err());

        let mut duplicate_worker_ref = applied.clone();
        let worker_ref =
            durable_authority_mut(&mut duplicate_worker_ref).retained_worker_refs[0].clone();
        durable_authority_mut(&mut duplicate_worker_ref)
            .retained_worker_refs
            .push(worker_ref);
        assert!(duplicate_worker_ref.validate().is_err());

        let mut skipped_revision = applied.clone();
        durable_authority_mut(&mut skipped_revision).authority_revision += 1;
        assert!(skipped_revision.validate().is_err());

        let mut proof_ahead_of_authority = applied.clone();
        durable_authority_mut(&mut proof_ahead_of_authority).authority_revision = 1;
        assert!(proof_ahead_of_authority.validate().is_err());

        let mut forged_lineage_commitment = applied;
        forged_lineage_commitment
            .retained_worker_registration_journal
            .get_mut(&request.registration_id)
            .unwrap()
            .authoritative_lineage_commitment_after = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            };
        assert!(forged_lineage_commitment.validate().is_err());
    }

    #[test]
    fn lost_response_whole_operation_retry_joins_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let first = runtime.register_retained_target(&authority, &plan).unwrap();
        let applied_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let applied_objects = object_files(&object_root);

        let joined = runtime
            .register_retained_target(&authority, &plan)
            .expect("lost response retry must join the Applied registration");

        assert_eq!(joined, first);
        assert_eq!(authority.read_a12a_root().unwrap(), applied_root);
        assert_eq!(object_files(&object_root), applied_objects);
    }

    #[test]
    fn reserved_observation_joins_peer_application_during_publication() {
        let (_parent, authority, observation) = started_authority();
        let peer_authority = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut peer_result = None;

        let joined = runtime
            .register_retained_target_with(&authority, &plan, |_| {
                peer_result = Some(runtime.register_retained_target(&peer_authority, &plan)?);
                Ok(())
            })
            .expect("a Reserved observation must join a peer that reaches Applied first");

        assert_eq!(Some(joined), peer_result);
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
    }

    #[test]
    fn exact_resolution_rejects_substituted_target_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let target = runtime.register_retained_target(&authority, &plan).unwrap();
        let resolved = runtime
            .resolve_retained_target(&authority, &target)
            .unwrap();
        assert_eq!(
            resolved.registration.registration_id,
            target.registration_id
        );
        assert_eq!(resolved.descriptor, plan.descriptor);
        assert_eq!(
            resolved.resume_handle.participant_id,
            target.retained_participant_id
        );
        assert_eq!(
            resolved.retained_worker.participant_id,
            target.retained_participant_id
        );
        assert_eq!(resolved.retained_worker.world_binding.world_id, "r0-world");
        assert_eq!(resolved.current_policy.policy_revision, "r0-policy");
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut conflicts = Vec::new();

        let mut conflict = target.clone();
        conflict.authority_store_id = "as_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.orchestration_session_id = "other-session".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.registration_id = "rr_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.retained_participant_id = "other-participant".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.retained_worker_ref.ref_id = "ao_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.authority_revision_after += 1;
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.authority_record_commitment_after = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
        };
        conflicts.push(conflict);

        for conflict in conflicts {
            assert!(runtime
                .resolve_retained_target(&authority, &conflict)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            assert_eq!(object_files(&object_root), objects_before);
        }
    }

    #[test]
    fn unreserved_orphan_object_never_grants_retained_authority() {
        let (_parent, authority, observation) = started_authority();
        let descriptor = AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: plan(observation.clone()).descriptor,
        };
        let bytes = canonical_json::to_vec(&descriptor).unwrap();
        let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_88888888888888888888888888888888".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::AgentDescriptor,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(&descriptor).unwrap(),
            },
        };
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        crate::execution::agent_runtime::host_session_authority::store::prepare_typed_object_v2_test(
            &_parent.path().join("home"),
            root_before.root_revision,
            &reference,
            &bytes,
        )
        .unwrap();
        let root_after = authority.read_a12a_root().unwrap();
        assert_eq!(root_after, root_before);
        assert!(root_after
            .retained_worker_registration_request_index
            .is_empty());
        assert!(root_after.retained_worker_registration_journal.is_empty());
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 1);
        let fake_target = RetainedWorkerRegistrationResultV1 {
            registration_id: "rr_88888888888888888888888888888888".into(),
            authority_store_id: observation.authority_store_id,
            orchestration_session_id: observation.orchestration_session_id,
            retained_participant_id: "orphan-participant".into(),
            retained_worker_ref: reference,
            authority_revision_after: observation.authority_revision,
            authority_record_commitment_after: observation.authority_record_commitment,
        };
        assert!(RetainedWorkerRuntime
            .resolve_retained_target(&authority, &fake_target)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 1);
    }

    #[test]
    fn two_retained_links_preserve_pending_start_and_resolve_exactly() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = plan(observation);
        let first = runtime
            .register_retained_target(&authority, &first_plan)
            .unwrap();
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        let mut second_plan = plan(observation);
        second_plan.registration_request_id = "spawn-request-2".into();
        second_plan.retained_participant_id = "r0-retained-2".into();
        second_plan.descriptor.agent_id = "codex-worker-2".into();
        second_plan.internal_uaa_session_id = "uaa-retained-2".into();
        let second = runtime
            .register_retained_target(&authority, &second_plan)
            .unwrap();

        let first_resolved = runtime.resolve_retained_target(&authority, &first).unwrap();
        let second_resolved = runtime
            .resolve_retained_target(&authority, &second)
            .unwrap();
        assert_eq!(first_resolved.current_authority_revision, 3);
        assert_eq!(second_resolved.current_authority_revision, 3);
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_journal.len(), 2);
        let HostSessionTransitionIntentStateV2::Applied {
            startup_ownership, ..
        } = &root.transition_intent_map["r0-start-intent"].state
        else {
            panic!("production Start must remain Applied")
        };
        assert!(matches!(
            startup_ownership.as_ref(),
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionStartupOwnershipApplicationV1::Pending {
                expected_authority_revision: 1,
                ..
            }
        ));
        let object_root = _parent.path().join("home/authority-v1/objects");
        let root_before = root.clone();
        let objects_before = object_files(&object_root);
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        let mut duplicate = plan(observation);
        duplicate.registration_request_id = "spawn-request-duplicate".into();
        assert!(runtime
            .register_retained_target(&authority, &duplicate)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let mut reused_ref = root_before;
        let registrations = reused_ref
            .retained_worker_registration_journal
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let first_ref = registrations[0].retained_worker_ref.clone();
        let second_id = registrations[1].registration_id.clone();
        reused_ref
            .retained_worker_registration_journal
            .get_mut(&second_id)
            .unwrap()
            .retained_worker_ref = first_ref;
        assert!(reused_ref.validate().is_err());
    }

    const RETAINED_SUBPROCESS_HOME: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_HOME";
    const RETAINED_SUBPROCESS_STORE: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_STORE";
    const RETAINED_SUBPROCESS_COMMITMENT: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_COMMITMENT";
    const RETAINED_SUBPROCESS_VARIANT: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT";

    #[test]
    fn retained_registration_subprocess_worker() {
        let Some(home) = std::env::var_os(RETAINED_SUBPROCESS_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let current = (0..32)
            .find_map(|_| {
                let resolved = authority.resolve_current_exact("r0-session", None).ok();
                if resolved.is_none() {
                    std::thread::yield_now();
                }
                resolved
            })
            .expect("subprocess must acquire a stable authority snapshot")
            .observation;
        let mut plan = plan(current);
        plan.expected_authority.authority_store_id =
            std::env::var(RETAINED_SUBPROCESS_STORE).unwrap();
        plan.expected_authority.authority_revision = 1;
        plan.expected_authority.authority_record_commitment = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: std::env::var(RETAINED_SUBPROCESS_COMMITMENT).unwrap(),
        };
        if std::env::var_os(RETAINED_SUBPROCESS_VARIANT).as_deref()
            == Some(std::ffi::OsStr::new("conflict"))
        {
            plan.retained_participant_id = "r0-retained-conflict".into();
            plan.descriptor.agent_id = "codex-worker-conflict".into();
            plan.internal_uaa_session_id = "uaa-retained-conflict".into();
        }
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan)
            .unwrap();
    }

    #[test]
    fn two_process_identical_registration_converges_to_one_authority_link() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex,
        } = &observation.authority_record_commitment
        else {
            panic!("durable authority commitment must be canonical")
        };
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::retained_registration_subprocess_worker";
        let spawn = || {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(RETAINED_SUBPROCESS_HOME, _parent.path().join("home"))
                .env(RETAINED_SUBPROCESS_STORE, &observation.authority_store_id)
                .env(RETAINED_SUBPROCESS_COMMITMENT, digest_hex)
                .spawn()
                .unwrap()
        };
        let mut first = spawn();
        let mut second = spawn();
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        let SessionNamespaceRecordV1::Authority(authority_record) =
            &root.session_namespace_map["r0-session"]
        else {
            panic!("retained registration must preserve authority")
        };
        assert_eq!(authority_record.authority_revision, 2);
        assert_eq!(authority_record.retained_worker_refs.len(), 1);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
        assert_eq!(
            authority_record.authoritative_participant_lineage,
            vec!["r0-orchestrator".to_owned(), "r0-retained-1".to_owned()]
        );
    }

    #[test]
    fn two_process_conflicting_registration_has_one_winner_and_zero_partial_authority() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex,
        } = &observation.authority_record_commitment
        else {
            panic!("durable authority commitment must be canonical")
        };
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::retained_registration_subprocess_worker";
        let spawn = |variant: &str| {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(RETAINED_SUBPROCESS_HOME, _parent.path().join("home"))
                .env(RETAINED_SUBPROCESS_STORE, &observation.authority_store_id)
                .env(RETAINED_SUBPROCESS_COMMITMENT, digest_hex)
                .env(RETAINED_SUBPROCESS_VARIANT, variant)
                .spawn()
                .unwrap()
        };
        let mut first = spawn("exact");
        let mut second = spawn("conflict");
        let first_success = first.wait().unwrap().success();
        let second_success = second.wait().unwrap().success();
        assert_ne!(first_success, second_success);

        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        let SessionNamespaceRecordV1::Authority(authority_record) =
            &root.session_namespace_map["r0-session"]
        else {
            panic!("retained registration must preserve authority")
        };
        assert_eq!(authority_record.authority_revision, 2);
        assert_eq!(authority_record.retained_worker_refs.len(), 1);
        assert_eq!(authority_record.authoritative_participant_lineage.len(), 2);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
    }
}
