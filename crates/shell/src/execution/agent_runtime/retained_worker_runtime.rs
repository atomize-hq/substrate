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
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1, ResumeHandleHashInputV1,
    RetainedWorkerObjectHashInputV1,
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
    pub(crate) retained_participant_id: String,
    pub(crate) authority_revision_after: u64,
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
        _authority: &HostSessionAuthority,
        _plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<RetainedWorkerRegistrationResultV1, RetainedWorkerRuntimeError> {
        Err(RetainedWorkerRuntimeError(
            "retained registration is not implemented".into(),
        ))
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::execution::agent_runtime::host_session_authority::facade::{
        AuthorityObservationV1, HostSessionAuthority,
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
}
