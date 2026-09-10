#![cfg(all(unix, target_os = "linux"))]

use base64::Engine as _;
use hyper::body::HttpBody;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::time::timeout;
use transport_api_types::{
    AuthorityObjectKindV1, AuthorityObjectRefV1, DispatchPolicyCommitmentRefCarrierV1,
    DispatchPolicySnapshotCarrierV1, E2DispatchPolicyReservationRefCarrierV1,
    E2LaunchRequestCommitmentV1, E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
    ExecuteRequest, MemberDispatchRequest, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    MemberTurnSubmitRequestV1, OpaqueAuthorityCommitmentV1, PolicySnapshotV3,
    PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
    RetainedTurnPolicyCommitmentSubjectV1, RetainedWorkerAdmissionCommitmentCarrierV1,
    RetainedWorkerAuthorityObjectCommitmentV1, RetainedWorkerLaunchAuthorityProofV1,
    RetainedWorkerLaunchWorldBindingV1, WorldBindingRefV1, WorldWorkAcceptanceContextV1,
};
use world_api::{
    SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerAction,
    SharedWorldOwnerSpec, WorldReuseMode, WorldSpec,
};
use world_service::WorldService;

const SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";

fn minimal_policy_snapshot() -> PolicySnapshotV3 {
    PolicySnapshotV3 {
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
    }
}

fn make_member_dispatch_request(
    cwd: &Path,
    binary_path: &Path,
    seed_home: &Path,
    binding: &SharedWorldBindingSnapshot,
    orchestration_session_id: &str,
    participant_id: &str,
    run_id: &str,
) -> ExecuteRequest {
    let mut env = HashMap::new();
    env.insert(
        "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".to_string(),
        "1".to_string(),
    );
    env.insert(
        SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
        seed_home.display().to_string(),
    );
    env.insert(
        "SUBSTRATE_WORLD_PROJECT_DIR".to_string(),
        cwd.display().to_string(),
    );
    ExecuteRequest {
        profile: None,
        cmd: String::new(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "member-runtime-retained-lifecycle-test".to_string(),
        budget: None,
        policy_snapshot: minimal_policy_snapshot(),
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        acceptance_context: None,
        member_dispatch: Some(MemberDispatchRequest::V1(MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: orchestration_session_id.to_string(),
            participant_id: participant_id.to_string(),
            orchestrator_participant_id: "ash_orchestrator_retained_lifecycle_test".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: run_id.to_string(),
            world_id: binding.world_id.clone(),
            world_generation: binding.world_generation,
            initial_prompt: Some("bootstrap prompt".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: binary_path.display().to_string(),
            },
            retained_worker_launch_authority: None,
            e2_launch_activation: None,
        })),
    }
}

fn attach_exact_retained_launch_authority(request: &mut ExecuteRequest) {
    let canonical_policy = request
        .policy_snapshot
        .canonicalize()
        .expect("canonical policy snapshot");
    let canonical_policy_bytes =
        serde_json::to_vec(&canonical_policy).expect("serialize canonical policy");
    request.policy_snapshot = canonical_policy;
    let mut hasher = Sha256::new();
    hasher.update(&canonical_policy_bytes);
    let policy_revision = format!("{:x}", hasher.finalize());
    let dispatch = request
        .member_dispatch
        .as_mut()
        .and_then(MemberDispatchRequest::as_v1_mut)
        .expect("V1 member dispatch request");
    let commitment = |value: char| RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: value.to_string().repeat(64),
    };
    dispatch.retained_worker_launch_authority = Some(RetainedWorkerLaunchAuthorityProofV1 {
        schema_version: 1,
        authority_store_id: "has_member_runtime_readiness".to_string(),
        issuer_request_id: "req_member_runtime_readiness".to_string(),
        canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
            schema_version: 1,
            algorithm: "hmac-sha-256".to_string(),
            key_id: "adk_member_runtime_readiness".to_string(),
            digest_hex: "a".repeat(64),
        },
        registration_id: "rwr_registration_member_runtime_readiness".to_string(),
        registration_commitment: commitment('b'),
        authority_revision_after: 2,
        authority_record_commitment_after: commitment('c'),
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        caller_participant_id: dispatch.orchestrator_participant_id.clone(),
        retained_participant_id: dispatch.participant_id.clone(),
        bootstrap_run_id: dispatch.run_id.clone(),
        transport_claim_id: "rtc_member_runtime_readiness".to_string(),
        backend_id: dispatch.backend_id.clone(),
        protocol: dispatch.protocol.clone(),
        world_binding: RetainedWorkerLaunchWorldBindingV1 {
            world_id: dispatch.world_id.clone(),
            world_generation: dispatch.world_generation,
        },
        current_policy_ref_id: "ao_policy_member_runtime_readiness".to_string(),
        current_policy_revision: policy_revision.clone(),
        retained_worker_ref_id: "ao_worker_member_runtime_readiness".to_string(),
        retained_worker_commitment: commitment('d'),
    });
    let cap_ref = DispatchPolicyCommitmentRefCarrierV1 {
        authority_store_id: "authority-store-e2-world-service-live".to_string(),
        commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
        exact_linkage_hash: "a".repeat(64),
    };
    dispatch.e2_launch_activation = Some(E2MemberLaunchActivationCarrierV1 {
        schema_version: 1,
        activation_id: format!("e2a_{}", "a".repeat(32)),
        launch_kind: E2MemberLaunchKindV1::FreshSpawn,
        reservation_ref: Some(E2DispatchPolicyReservationRefCarrierV1 {
            authority_store_id: cap_ref.authority_store_id.clone(),
            reservation_id: "dpr_member_runtime_readiness".to_string(),
            reservation_hash: "e".repeat(64),
        }),
        commitment_ref: cap_ref.clone(),
        immutable_worker_cap_ref: cap_ref,
        immutable_worker_cap_created_revision: 7,
        immutable_worker_cap_application_revision: 9,
        policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD
            .encode(&canonical_policy_bytes),
        policy_snapshot_byte_length: canonical_policy_bytes.len() as u64,
        policy_snapshot_ref: AuthorityObjectRefV1 {
            ref_id: "ao_11111111111111111111111111111111".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: "1".repeat(64),
            },
        },
        policy_snapshot_hash: policy_revision.clone(),
        policy_snapshot_revision: policy_revision.clone(),
        reason: Some("authenticated retained-worker launch cap".to_string()),
        request_id: "req_member_runtime_readiness".to_string(),
        idempotency_key: "idem_member_runtime_readiness".to_string(),
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        caller_participant_id: dispatch.orchestrator_participant_id.clone(),
        caller_backend_id: dispatch.backend_id.clone(),
        target_backend_id: dispatch.backend_id.clone(),
        retained_participant_id: dispatch.participant_id.clone(),
        bootstrap_run_id: dispatch.run_id.clone(),
        source_participant_id: None,
        target_world: WorldBindingRefV1 {
            world_id: dispatch.world_id.clone(),
            world_generation: dispatch.world_generation,
        },
        parent_policy_ref: AuthorityObjectRefV1 {
            ref_id: "ao_22222222222222222222222222222222".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: "2".repeat(64),
            },
        },
        parent_policy_revision: "parent-member-runtime-readiness".to_string(),
        request_commitment: E2LaunchRequestCommitmentV1::HmacSha256 {
            key_id: "dpk_member_runtime_readiness".to_string(),
            domain: "substrate.dispatch_policy_commitment.spawn_request.v1".to_string(),
            digest_hex: "3".repeat(64),
        },
        registry_publication_revision: 9,
    });
    dispatch
        .e2_launch_activation
        .as_ref()
        .expect("E2 launch activation")
        .validate()
        .expect("valid E2 launch activation");
}

fn write_seed_home(temp: &Path) -> PathBuf {
    let seed_home = temp.join("seed-home");
    fs::create_dir_all(&seed_home).expect("create seed home");
    fs::write(
        seed_home.join("auth.json"),
        r#"{"account_id":"acct_test","access_token":"token_test"}"#,
    )
    .expect("write seed auth");
    fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n").expect("write seed config");
    seed_home
}

fn make_member_turn_submit_request(
    orchestration_session_id: &str,
    participant_id: &str,
    world_id: &str,
    world_generation: u64,
    run_id: &str,
    prompt: &str,
) -> MemberTurnSubmitRequestV1 {
    MemberTurnSubmitRequestV1 {
        schema_version: 1,
        orchestration_session_id: orchestration_session_id.to_string(),
        participant_id: participant_id.to_string(),
        orchestrator_participant_id: "ash_orchestrator_retained_lifecycle_test".to_string(),
        backend_id: "cli:codex".to_string(),
        run_id: run_id.to_string(),
        world_id: world_id.to_string(),
        world_generation,
        prompt: prompt.to_string(),
        policy_snapshot_carrier: None,
        acceptance_context: None,
    }
}

fn write_clean_bootstrap_then_resume_member_runtime(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-clean-bootstrap-then-resume.sh");
    let count_path = temp.join("member-runtime-clean-bootstrap-then-resume.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/member-runtime-$count.args\"\ncat > \"$SCRIPT_DIR/member-runtime-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  exit 0\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\",\"item_id\":\"msg-2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write clean-bootstrap member runtime");
    let mut perms = fs::metadata(&path)
        .expect("clean-bootstrap member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set clean-bootstrap member runtime permissions");
    (path, count_path)
}

fn write_clean_bootstrap_fail_submit_turn_then_resume_member_runtime(
    temp: &Path,
) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-clean-bootstrap-fail-submit-turn-then-resume.sh");
    let count_path = temp.join("member-runtime-clean-bootstrap-fail-submit-turn-then-resume.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/member-runtime-fail-then-resume-$count.args\"\ncat > \"$SCRIPT_DIR/member-runtime-fail-then-resume-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  exit 0\nfi\nif [ \"$count\" -eq 2 ]; then\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\",\"item_id\":\"msg-2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt failure\"}}}}\\n'\n  exit 17\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\"}}\\n'\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\",\"item_id\":\"msg-3\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success after failure\"}}}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\"}}\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write fail-then-resume member runtime");
    let mut perms = fs::metadata(&path)
        .expect("fail-then-resume member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fail-then-resume member runtime permissions");
    (path, count_path)
}

fn write_member_runtime_without_session_handle(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-without-session-handle.sh");
    let count_path = temp.join("member-runtime-without-session-handle.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/no-session-handle-$count.args\"\ncat > \"$SCRIPT_DIR/no-session-handle-$count.stdin\"\nprintf 'bootstrap-without-session-handle\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write no-session-handle member runtime");
    let mut perms = fs::metadata(&path)
        .expect("no-session-handle member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set no-session-handle member runtime permissions");
    (path, count_path)
}

fn write_e2_enforcement_member_runtime(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-e2-enforcement");
    let source_path = temp.join("member-runtime-e2-enforcement.c");
    let count_path = temp.join("member-runtime-e2-enforcement.count");
    fs::write(temp.join("exact-a.txt"), "allowed A\n").expect("write exact A");
    fs::write(temp.join("exact-b.txt"), "allowed B\n").expect("write exact B");
    fs::write(temp.join("sibling.txt"), "denied sibling\n").expect("write sibling");
    let outside = fs::canonicalize(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("Cargo.toml"),
    )
    .expect("resolve outside repository manifest");
    std::os::unix::fs::symlink(&outside, temp.join("final-link.txt"))
        .expect("create final symlink escape");
    std::os::unix::fs::symlink(
        outside.parent().expect("outside executable parent"),
        temp.join("ancestor-link"),
    )
    .expect("create ancestor symlink escape");

    let c_literal = |path: &Path| {
        serde_json::to_string(&path.display().to_string()).expect("C-compatible path literal")
    };
    let source = r#"
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

static int has_arg(int argc, char **argv, const char *needle) {
    for (int i = 1; i < argc; i++) {
        if (strstr(argv[i], needle) != NULL) return 1;
    }
    return 0;
}

static int can_read(const char *path) {
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) return 0;
    close(fd);
    return 1;
}

static void write_count(const char *value) {
    FILE *file = fopen(__COUNT_PATH__, "w");
    if (file == NULL) return;
    fputs(value, file);
    fclose(file);
}

int main(int argc, char **argv) {
    if (has_arg(argc, argv, "--version")) {
        puts("codex 1.2.3");
        return 0;
    }
    if (has_arg(argc, argv, "features")) {
        puts("{\"features\":[\"add_dir\"]}");
        return 0;
    }
    if (has_arg(argc, argv, "--help")) {
        puts("Usage: codex --add-dir");
        return 0;
    }

    char prompt[256] = {0};
    if (fgets(prompt, sizeof(prompt), stdin) == NULL) prompt[0] = '\0';
    const char *tag = "BOOTSTRAP";
    if (has_arg(argc, argv, "TURN_A") || strstr(prompt, "TURN_A") != NULL) tag = "TURN_A";
    if (has_arg(argc, argv, "TURN_B") || strstr(prompt, "TURN_B") != NULL) tag = "TURN_B";
    if (has_arg(argc, argv, "SYMLINK") || strstr(prompt, "SYMLINK") != NULL) tag = "SYMLINK";

    if (strcmp(tag, "BOOTSTRAP") == 0) {
        write_count("1");
        puts("{\"type\":\"thread.started\",\"thread_id\":\"thread-member-e2\"}");
        puts("{\"type\":\"turn.started\",\"thread_id\":\"thread-member-e2\",\"turn_id\":\"turn-bootstrap\"}");
        puts("{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-e2\",\"turn_id\":\"turn-bootstrap\"}");
        return 0;
    }

    int exact_a = can_read(__EXACT_A__);
    int exact_b = can_read(__EXACT_B__);
    int sibling = can_read(__SIBLING__);
    int outside = can_read(__OUTSIDE__);
    printf("{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-e2\"}\n");
    printf("{\"type\":\"turn.started\",\"thread_id\":\"thread-member-e2\",\"turn_id\":\"turn-e2\"}\n");
    printf("{\"type\":\"item.completed\",\"thread_id\":\"thread-member-e2\",\"turn_id\":\"turn-e2\",\"item_id\":\"msg-e2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{\"text\":\"%s:A=%d:B=%d:SIBLING=%d:OUTSIDE=%d\"}}\n", tag, exact_a, exact_b, sibling, outside);
    printf("{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-e2\",\"turn_id\":\"turn-e2\"}\n");
    return 0;
}
"#
    .replace("__COUNT_PATH__", &c_literal(&count_path))
    .replace("__EXACT_A__", &c_literal(&temp.join("exact-a.txt")))
    .replace("__EXACT_B__", &c_literal(&temp.join("exact-b.txt")))
    .replace("__SIBLING__", &c_literal(&temp.join("sibling.txt")))
    .replace("__OUTSIDE__", &c_literal(&outside));
    fs::write(&source_path, source).expect("write E2 enforcement runtime source");
    let output = Command::new("cc")
        .args(["-O2", "-Wall", "-Wextra", "-o"])
        .arg(&path)
        .arg(&source_path)
        .output()
        .expect("compile E2 enforcement member runtime");
    assert!(
        output.status.success(),
        "compile E2 enforcement runtime failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    (path, count_path)
}

fn e2_exact_read_snapshot(paths: &[&str]) -> PolicySnapshotV3 {
    let allow_list: Vec<String> = paths.iter().map(|path| (*path).to_string()).collect();
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
            deny_enforcement: None,
            caged_required: true,
            discover: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: allow_list.clone(),
                deny_list: Vec::new(),
            }),
            read: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list,
                deny_list: Vec::new(),
            }),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: false,
                allow_list: Vec::new(),
                deny_list: Vec::new(),
            },
        },
    }
    .canonicalize()
    .expect("canonical exact-read E2 snapshot")
}

fn attach_e2_policy_carrier(
    request: &mut MemberTurnSubmitRequestV1,
    snapshot: PolicySnapshotV3,
    revision: &str,
) {
    let bytes = serde_json::to_vec(&snapshot).expect("exact E1 snapshot bytes");
    let message_id = format!("wwm_{}", uuid::Uuid::now_v7());
    request.acceptance_context = Some(WorldWorkAcceptanceContextV1 {
        schema_version: 1,
        proposed_acceptance_record_id: format!("wwa_{}", uuid::Uuid::now_v7()),
        request_id: request.run_id.clone(),
        message_id: Some(message_id.clone()),
        caller_backend_id: "cli:codex".to_string(),
        host_transition_correlation: None,
    });
    request.policy_snapshot_carrier = Some(DispatchPolicySnapshotCarrierV1 {
        schema_version: 1,
        immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e2-world-service-live".to_string(),
            commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
            exact_linkage_hash: "a".repeat(64),
        },
        immutable_worker_cap_created_revision: 7,
        immutable_worker_cap_application_revision: 9,
        subject: RetainedTurnPolicyCommitmentSubjectV1 {
            retained_participant_id: request.participant_id.clone(),
            active_run_id: request.run_id.clone(),
            message_id: Some(message_id),
        },
        orchestration_session_id: request.orchestration_session_id.clone(),
        caller_participant_id: request.orchestrator_participant_id.clone(),
        caller_backend_id: "cli:codex".to_string(),
        target_backend_id: request.backend_id.clone(),
        target_world: WorldBindingRefV1 {
            world_id: request.world_id.clone(),
            world_generation: request.world_generation,
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
        policy_snapshot_revision: revision.to_string(),
        reason: Some("authenticated retained-turn exact-file narrowing".to_string()),
    });
    request.validate().expect("valid E2 retained-turn request");
}

async fn next_optional_stream_frame_value<B>(body: &mut B, buffer: &mut Vec<u8>) -> Option<Value>
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            let payload = &line[..line.len() - 1];
            return Some(serde_json::from_slice(payload).expect("valid stream frame json value"));
        }

        match timeout(Duration::from_secs(5), body.data())
            .await
            .expect("timed out waiting for stream chunk")
        {
            Some(Ok(chunk)) => buffer.extend_from_slice(&chunk),
            Some(Err(err)) => panic!("stream chunk error: {err}"),
            None => {
                assert!(
                    buffer.is_empty(),
                    "stream ended mid-frame with leftover bytes: {buffer:?}"
                );
                return None;
            }
        }
    }
}

fn frame_start_span_id(frame: &Value) -> Option<&str> {
    if frame.get("type")?.as_str() == Some("start") {
        return frame.get("span_id")?.as_str();
    }
    frame.get("Start")?.get("span_id")?.as_str()
}

fn frame_exit(frame: &Value) -> Option<(i32, &str)> {
    if frame.get("type")?.as_str() == Some("exit") {
        let exit = frame.get("exit")?.as_i64()?;
        let span_id = frame.get("span_id")?.as_str()?;
        return Some((exit as i32, span_id));
    }
    let exit = frame.get("Exit")?.get("exit")?.as_i64()?;
    let span_id = frame.get("Exit")?.get("span_id")?.as_str()?;
    Some((exit as i32, span_id))
}

fn frame_event(frame: &Value) -> Option<&Value> {
    if frame.get("type").and_then(Value::as_str) == Some("event") {
        return frame.get("event");
    }
    frame.get("Event")?.get("event")
}

#[derive(Debug)]
struct StreamSummary {
    frames: Vec<Value>,
    saw_registered: bool,
    exit: Option<i32>,
}

struct RetainedLifecycleHarness {
    _tempdir: TempDir,
    service: WorldService,
    binding: SharedWorldBindingSnapshot,
    count_path: PathBuf,
    orchestration_session_id: &'static str,
    participant_id: &'static str,
}

async fn collect_stream_summary<B>(body: &mut B, span_id: &str) -> StreamSummary
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    let mut buffer = Vec::new();
    let mut frames = Vec::new();
    let mut saw_registered = false;
    let mut exit = None;

    while let Some(frame) = next_optional_stream_frame_value(body, &mut buffer).await {
        frames.push(frame.clone());
        if let Some(event) = frame_event(&frame) {
            if event.get("kind").and_then(Value::as_str) == Some("registered") {
                saw_registered = true;
            }
            continue;
        }
        if let Some((code, exit_span_id)) = frame_exit(&frame) {
            assert_eq!(exit_span_id, span_id, "unexpected exit span: {frame:?}");
            exit = Some(code);
        }
    }

    StreamSummary {
        frames,
        saw_registered,
        exit,
    }
}

fn read_invocation_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("read invocation count")
        .trim()
        .parse()
        .expect("parse invocation count")
}

type MemberRuntimeScriptWriter = fn(&Path) -> (PathBuf, PathBuf);

async fn launch_retained_lifecycle_harness(
    write_member_runtime: MemberRuntimeScriptWriter,
    orchestration_session_id: &'static str,
    participant_id: &'static str,
    run_id: &str,
    authority_managed: bool,
    positive_world_generation: bool,
    isolate_from_runtime_scratch_roots: bool,
) -> Option<RetainedLifecycleHarness> {
    let tmp = if isolate_from_runtime_scratch_roots {
        let repository_root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .expect("resolve repository root");
        tempfile::Builder::new()
            .prefix(".substrate-e2-retained-landlock-")
            .tempdir_in(repository_root)
            .expect("E2 retained-turn project tempdir outside /tmp and /dev")
    } else {
        tempdir().expect("tempdir")
    };
    let member_turn_state_root = tmp.path().join("member-turn-state");
    fs::create_dir(&member_turn_state_root).expect("create injected member-turn state root");
    let mut state_permissions = fs::metadata(&member_turn_state_root)
        .expect("member-turn state root metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt as _;
    state_permissions.set_mode(0o700);
    fs::set_permissions(&member_turn_state_root, state_permissions)
        .expect("secure member-turn state root permissions");
    let service =
        match WorldService::new_with_member_turn_state_root_for_test(&member_turn_state_root) {
            Ok(svc) => svc,
            Err(err) => {
                eprintln!("skipping retained member lifecycle test: service init failed: {err}");
                return None;
            }
        };

    let seed_home = write_seed_home(tmp.path());
    let (member_binary, count_path) = write_member_runtime(tmp.path());
    let reuse_mode = if authority_managed {
        WorldReuseMode::GenericCompatible
    } else {
        WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        })
    };
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode,
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: tmp.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
        backend_policy: None,
    };
    let mut world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!(
                "skipping retained member lifecycle test: failed to ensure shared world: {err}"
            );
            return None;
        }
    };
    if positive_world_generation && !authority_managed {
        let mut replacement_spec = world_spec.clone();
        replacement_spec.reuse_mode = WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.to_string(),
            action: SharedWorldOwnerAction::ReplaceExpectedGeneration {
                expected_generation: 0,
                reason: "E2 retained-turn positive-generation fixture".to_string(),
            },
        });
        world = match service.ensure_session_world(&replacement_spec) {
            Ok(world) => world,
            Err(err) => {
                eprintln!(
                    "skipping retained member lifecycle test: failed to replace shared world: {err}"
                );
                return None;
            }
        };
    }
    let binding = if authority_managed {
        SharedWorldBindingSnapshot {
            orchestration_session_id: orchestration_session_id.to_string(),
            world_id: world.id.clone(),
            world_generation: u64::from(positive_world_generation),
            binding_state: SharedWorldBindingState::Active,
        }
    } else {
        let Some(binding) = world.shared_binding.clone() else {
            eprintln!("skipping retained member lifecycle test: shared world binding missing");
            return None;
        };
        binding
    };

    let mut request = make_member_dispatch_request(
        tmp.path(),
        &member_binary,
        &seed_home,
        &binding,
        orchestration_session_id,
        participant_id,
        run_id,
    );
    if authority_managed {
        attach_exact_retained_launch_authority(&mut request);
    }
    let launch_response = service
        .execute_stream(request)
        .await
        .unwrap_or_else(|error| panic!("member launch should succeed: {error:#}"));
    let mut launch_body = launch_response.into_body();
    let mut launch_buffer = Vec::new();
    let launch_start = next_optional_stream_frame_value(&mut launch_body, &mut launch_buffer)
        .await
        .expect("expected launch start frame");
    let launch_span_id = frame_start_span_id(&launch_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
    let launch_summary = collect_stream_summary(&mut launch_body, &launch_span_id).await;
    if authority_managed {
        assert!(
            launch_summary
                .frames
                .first()
                .and_then(frame_event)
                .is_some_and(|event| {
                    event.get("kind").and_then(Value::as_str) == Some("registered")
                }),
            "authority-managed bootstrap must publish the exact Registered session-handle event before gateway status or progress events; frames: {:?}",
            launch_summary.frames
        );
        assert!(
            launch_summary.frames.iter().skip(1).any(|frame| frame
                .to_string()
                .contains("external sandbox exec policy enabled")),
            "the pre-registration gateway warning must be preserved after Registered; frames: {:?}",
            launch_summary.frames
        );
    }
    assert!(
        launch_summary.saw_registered,
        "bootstrap with a surfaced session handle must register before it exits; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        launch_summary.exit,
        Some(0),
        "bootstrap must exit cleanly before later submit-turn proof begins; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "bootstrap exit proof must complete before the follow-up submit turn runs"
    );

    Some(RetainedLifecycleHarness {
        _tempdir: tmp,
        service,
        binding,
        count_path,
        orchestration_session_id,
        participant_id,
    })
}

async fn launch_clean_bootstrap_exit_harness() -> Option<RetainedLifecycleHarness> {
    launch_retained_lifecycle_harness(
        write_clean_bootstrap_then_resume_member_runtime,
        "orch-member-runtime-retained-clean-exit",
        "ash_member_runtime_retained_clean_exit",
        "run-member-runtime-retained-bootstrap",
        false,
        false,
        false,
    )
    .await
}

async fn launch_authority_managed_clean_bootstrap_exit_harness() -> Option<RetainedLifecycleHarness>
{
    let fixture_id = uuid::Uuid::now_v7();
    let orchestration_session_id =
        Box::leak(format!("orch-member-runtime-authority-readiness-{fixture_id}").into_boxed_str());
    let participant_id =
        Box::leak(format!("ash_member_runtime_authority_readiness_{fixture_id}").into_boxed_str());
    launch_retained_lifecycle_harness(
        write_clean_bootstrap_then_resume_member_runtime,
        orchestration_session_id,
        participant_id,
        "run-member-runtime-authority-readiness-bootstrap",
        true,
        false,
        false,
    )
    .await
}

async fn launch_failed_submit_turn_harness() -> Option<RetainedLifecycleHarness> {
    launch_retained_lifecycle_harness(
        write_clean_bootstrap_fail_submit_turn_then_resume_member_runtime,
        "orch-member-runtime-retained-failed-turn",
        "ash_member_runtime_retained_failed_turn",
        "run-member-runtime-retained-failed-turn-bootstrap",
        false,
        false,
        false,
    )
    .await
}

async fn submit_turn_summary(
    harness: &RetainedLifecycleHarness,
    run_id: &str,
    prompt: &str,
) -> StreamSummary {
    let submit_response = harness
        .service
        .submit_member_turn_stream(make_member_turn_submit_request(
            harness.orchestration_session_id,
            harness.participant_id,
            &harness.binding.world_id,
            harness.binding.world_generation,
            run_id,
            prompt,
        ))
        .await
        .expect("submitted turn should succeed");
    let mut submit_body = submit_response.into_body();
    let mut submit_buffer = Vec::new();
    let submit_start = next_optional_stream_frame_value(&mut submit_body, &mut submit_buffer)
        .await
        .expect("expected submit_turn start frame");
    let submit_span_id = frame_start_span_id(&submit_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {submit_start:?}"));
    collect_stream_summary(&mut submit_body, &submit_span_id).await
}

fn make_e2_turn_request(
    harness: &RetainedLifecycleHarness,
    run_id: &str,
    prompt: &str,
    snapshot: PolicySnapshotV3,
    revision: &str,
) -> MemberTurnSubmitRequestV1 {
    let mut request = make_member_turn_submit_request(
        harness.orchestration_session_id,
        harness.participant_id,
        &harness.binding.world_id,
        harness.binding.world_generation,
        run_id,
        prompt,
    );
    attach_e2_policy_carrier(&mut request, snapshot, revision);
    request
}

async fn try_submit_e2_turn_summary(
    harness: &RetainedLifecycleHarness,
    request: MemberTurnSubmitRequestV1,
) -> Result<StreamSummary, String> {
    let submit_response = harness
        .service
        .submit_member_turn_stream(request)
        .await
        .map_err(|error| error.to_string())?;
    let mut submit_body = submit_response.into_body();
    let mut submit_buffer = Vec::new();
    let submit_start = next_optional_stream_frame_value(&mut submit_body, &mut submit_buffer)
        .await
        .ok_or_else(|| "submitted E2 turn returned no Start frame".to_string())?;
    let submit_span_id = frame_start_span_id(&submit_start)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("expected E2 submit Start frame, got {submit_start:?}"))?;
    Ok(collect_stream_summary(&mut submit_body, &submit_span_id).await)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_with_session_handle_registers_then_leaves_parked_worker_ready_for_follow_up(
) {
    let Some(harness) = launch_clean_bootstrap_exit_harness().await else {
        return;
    };
    assert_eq!(
        read_invocation_count(&harness.count_path),
        1,
        "bootstrap harness must stop after clean exit and leave the parked retained worker ready for Packet 3 follow-up proofs"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authority_managed_member_runtime_emits_registered_before_pre_registration_gateway_events()
{
    let Some(harness) = launch_authority_managed_clean_bootstrap_exit_harness().await else {
        return;
    };
    assert_eq!(
        read_invocation_count(&harness.count_path),
        1,
        "authority-managed readiness proof must use exactly one member bootstrap invocation"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_with_session_handle_requires_later_submit_turn_resumability(
) {
    let Some(harness) = launch_clean_bootstrap_exit_harness().await else {
        return;
    };

    let submit_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-follow-up",
        "follow-up prompt",
    )
    .await;
    assert_eq!(
        submit_summary.exit,
        Some(0),
        "parked retained follow-up submit_turn must exit cleanly; frames: {:?}",
        submit_summary.frames
    );
    assert!(
        submit_summary
            .frames
            .iter()
            .any(|frame| frame.to_string().contains("follow-up prompt success")),
        "parked retained follow-up submit_turn must surface the resumed follow-up output; frames: {:?}",
        submit_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        2,
        "parked retained follow-up submit_turn must invoke the resumed member runtime after bootstrap exit"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2_retained_turn_carrier_drives_actual_per_turn_landlock_without_accumulation() {
    let fixture_id = uuid::Uuid::now_v7();
    let orchestration_session_id =
        Box::leak(format!("orch-member-runtime-e2-enforcement-{fixture_id}").into_boxed_str());
    let participant_id =
        Box::leak(format!("ash_member_runtime_e2_enforcement_{fixture_id}").into_boxed_str());
    let Some(harness) = launch_retained_lifecycle_harness(
        write_e2_enforcement_member_runtime,
        orchestration_session_id,
        participant_id,
        "run-member-runtime-e2-bootstrap",
        true,
        true,
        true,
    )
    .await
    else {
        return;
    };

    let turn_a = try_submit_e2_turn_summary(
        &harness,
        make_e2_turn_request(
            &harness,
            "run-member-runtime-e2-turn-a",
            "TURN_A",
            e2_exact_read_snapshot(&["exact-a.txt"]),
            "policy-revision-e2-turn-a",
        ),
    )
    .await
    .expect("turn A carrier must install before execution");
    assert_eq!(turn_a.exit, Some(0), "turn A frames: {:?}", turn_a.frames);
    assert!(
        turn_a.frames.iter().any(|frame| frame
            .to_string()
            .contains("TURN_A:A=1:B=0:SIBLING=0:OUTSIDE=0")),
        "turn A must allow only its exact file under actual Landlock enforcement; frames: {:?}",
        turn_a.frames
    );
    assert_eq!(read_invocation_count(&harness.count_path), 1);

    let turn_b = try_submit_e2_turn_summary(
        &harness,
        make_e2_turn_request(
            &harness,
            "run-member-runtime-e2-turn-b",
            "TURN_B",
            e2_exact_read_snapshot(&["exact-b.txt"]),
            "policy-revision-e2-turn-b",
        ),
    )
    .await
    .expect("turn B carrier must install independently");
    assert_eq!(turn_b.exit, Some(0), "turn B frames: {:?}", turn_b.frames);
    assert!(
        turn_b.frames.iter().any(|frame| frame
            .to_string()
            .contains("TURN_B:A=0:B=1:SIBLING=0:OUTSIDE=0")),
        "turn B must replace turn A rather than accumulating it; frames: {:?}",
        turn_b.frames
    );
    assert_eq!(read_invocation_count(&harness.count_path), 1);

    let mut wrong_cap = make_e2_turn_request(
        &harness,
        "run-member-runtime-e2-wrong-cap",
        "TURN_A",
        e2_exact_read_snapshot(&["exact-a.txt"]),
        "policy-revision-e2-wrong-cap",
    );
    wrong_cap
        .policy_snapshot_carrier
        .as_mut()
        .expect("carrier")
        .immutable_worker_cap_ref
        .commitment_id = "dpc_018f0f3a-9b2c-7def-8abc-0123456789ae".to_string();
    wrong_cap.validate().expect("structurally valid wrong cap");
    let error = try_submit_e2_turn_summary(&harness, wrong_cap)
        .await
        .expect_err("substituted immutable cap must fail before execution");
    assert!(error.contains("worker-cap identity changed"), "{error}");
    assert_eq!(read_invocation_count(&harness.count_path), 1);

    for (run_id, allowed_path) in [
        (
            "run-member-runtime-e2-final-symlink",
            "final-link.txt".to_string(),
        ),
        (
            "run-member-runtime-e2-ancestor-symlink",
            format!(
                "ancestor-link/{}",
                std::env::current_exe()
                    .expect("current executable")
                    .file_name()
                    .expect("current executable file name")
                    .to_string_lossy()
            ),
        ),
    ] {
        let result = try_submit_e2_turn_summary(
            &harness,
            make_e2_turn_request(
                &harness,
                run_id,
                "SYMLINK_ESCAPE_MUST_NOT_EXECUTE",
                e2_exact_read_snapshot(&[allowed_path.as_str()]),
                run_id,
            ),
        )
        .await;
        match result {
            Ok(summary) => assert_ne!(
                summary.exit,
                Some(0),
                "symlink policy installation unexpectedly executed: {:?}",
                summary.frames
            ),
            Err(error) => assert!(
                error.contains("failed") || error.contains("landlock") || error.contains("exited"),
                "unexpected fail-closed symlink error: {error}"
            ),
        }
        assert_eq!(
            read_invocation_count(&harness.count_path),
            1,
            "read-only retained turns must not mutate the bootstrap counter"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_non_zero_submitted_turn_exit_cleans_active_turn_slot_without_deleting_retained_worker(
) {
    let Some(harness) = launch_failed_submit_turn_harness().await else {
        return;
    };

    let failed_turn_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-failed-follow-up",
        "follow-up prompt that fails",
    )
    .await;
    assert_eq!(
        failed_turn_summary.exit,
        Some(17),
        "non-zero submitted turn should surface its failure exit without implicitly closing retained continuity; frames: {:?}",
        failed_turn_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        2,
        "the failed submitted turn should run exactly once before the recovery follow-up"
    );

    let resumed_turn_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-recovery-follow-up",
        "follow-up prompt after failure",
    )
    .await;
    assert_eq!(
        resumed_turn_summary.exit,
        Some(0),
        "a later submitted turn should still resume the retained worker after a non-zero turn exit; frames: {:?}",
        resumed_turn_summary.frames
    );
    assert!(
        resumed_turn_summary
            .frames
            .iter()
            .any(|frame| frame
                .to_string()
                .contains("follow-up prompt success after failure")),
        "retained follow-up after a failed turn must surface resumed output instead of acting like the worker was deleted; frames: {:?}",
        resumed_turn_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        3,
        "non-zero submitted turn exit must clear active-turn bookkeeping without unregistering the retained worker"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_without_session_handle_fails_closed_for_submit_turn() {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!(
                "skipping retained member no-session-handle test: service init failed: {err}"
            );
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let seed_home = write_seed_home(tmp.path());
    let (member_binary, count_path) = write_member_runtime_without_session_handle(tmp.path());
    let orchestration_session_id = "orch-member-runtime-no-session-handle";
    let participant_id = "ash_member_runtime_no_session_handle";
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        }),
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: tmp.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
        backend_policy: None,
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!(
                "skipping retained member no-session-handle test: failed to ensure shared world: {err}"
            );
            return;
        }
    };
    let Some(binding) = world.shared_binding.clone() else {
        eprintln!("skipping retained member no-session-handle test: shared world binding missing");
        return;
    };

    let launch_response = service
        .execute_stream(make_member_dispatch_request(
            tmp.path(),
            &member_binary,
            &seed_home,
            &binding,
            orchestration_session_id,
            participant_id,
            "run-member-runtime-no-session-handle-bootstrap",
        ))
        .await
        .expect("member launch should succeed");
    let mut launch_body = launch_response.into_body();
    let mut launch_buffer = Vec::new();
    let launch_start = next_optional_stream_frame_value(&mut launch_body, &mut launch_buffer)
        .await
        .expect("expected launch start frame");
    let launch_span_id = frame_start_span_id(&launch_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
    let launch_summary = collect_stream_summary(&mut launch_body, &launch_span_id).await;
    assert!(
        !launch_summary.saw_registered,
        "bootstrap without a surfaced session handle must not promise retained registration; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        launch_summary.exit,
        Some(0),
        "negative coverage must still exercise a clean bootstrap exit; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "inverse coverage must verify bootstrap exited before submit_turn is attempted"
    );

    let err = service
        .submit_member_turn_stream(make_member_turn_submit_request(
            orchestration_session_id,
            participant_id,
            &binding.world_id,
            binding.world_generation,
            "run-member-runtime-no-session-handle-follow-up",
            "follow-up prompt",
        ))
        .await
        .expect_err("missing session handle must fail closed for submit_turn");
    assert!(
        err.to_string().contains("is not retained"),
        "unexpected submit_turn error: {err}"
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "fail-closed inverse must not run a resumptive follow-up invocation"
    );
}
