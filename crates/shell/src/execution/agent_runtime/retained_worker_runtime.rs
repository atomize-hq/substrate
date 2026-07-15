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
use super::host_session_authority::facade::{
    ReservedRetainedWorkerRegistrationV1, RetainedReservationCrashPointV1,
};
use super::host_session_authority::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, ResumeHandleHashInputV1,
    RetainedWorkerObjectHashInputV1, TimestampV1,
};
use super::host_session_authority::{AuthorityObservationV1, HostSessionAuthority};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRegistrationPlanV1 {
    pub(crate) registration_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) expected_authority: AuthorityObservationV1,
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
        self.reserve_registration_with(authority, plan, None, None)
    }

    fn reserve_registration_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
        registered_at: Option<TimestampV1>,
        crash_point: Option<RetainedReservationCrashPointV1>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, RetainedWorkerRuntimeError> {
        let descriptor_bytes = canonical_json::to_vec(&AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: plan.descriptor.clone(),
        })
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle_bytes = canonical_json::to_vec(&ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: plan.orchestration_session_id.clone(),
            participant_id: plan.retained_participant_id.clone(),
            backend_id: plan.descriptor.backend_id.clone(),
            protocol: plan.descriptor.protocol.clone(),
            internal_uaa_session_id: plan.internal_uaa_session_id.clone(),
        })
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let orchestration_session_id = plan.orchestration_session_id.clone();
        let retained_participant_id = plan.retained_participant_id.clone();
        authority
            .reserve_retained_worker_registration(
                &plan.registration_request_id,
                &plan.orchestration_session_id,
                &plan.expected_authority,
                &plan.retained_participant_id,
                descriptor_bytes,
                resume_handle_bytes,
                registered_at,
                crash_point,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                        schema_version: 1,
                        orchestration_session_id: orchestration_session_id.clone(),
                        participant_id: retained_participant_id.clone(),
                        world_binding: world_binding.clone(),
                        descriptor_ref: descriptor_ref.clone(),
                        resume_handle_ref: resume_handle_ref.clone(),
                        policy_ref: policy_ref.clone(),
                    })
                    .map_err(|_| "encode retained-worker object")
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
        self.reserve_registration_with(authority, plan, Some(registered_at), crash_point)
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
    use crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority;
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
            issuer_request_id: "r0-start-request".into(),
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
            expected_authority: observation,
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
        changed.expected_authority.orchestration_session_id = "r0-session-changed".into();
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
}
