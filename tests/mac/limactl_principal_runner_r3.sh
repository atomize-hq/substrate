#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="limactl-principal-runner-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EXECUTOR="${REPO_ROOT}/src/bin/substrate-lifecycle-macos.rs"

fail() {
  printf '[%s] FAIL: %s\n' "${SCRIPT_NAME}" "$*" >&2
  exit 1
}

python3 - "${EXECUTOR}" <<'PY'
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_text()

required = [
    "struct MacLimaPrincipalV1",
    "fn mac_resolve_lima_principal_v1(",
    "fn mac_revalidate_lima_principal_v1(",
    "fn mac_spawn_fixed_lima_child_after_validation_v1(",
    "fn mac_spawn_fixed_lima_child_v1(",
    "fn mac_verify_lima_control_state_ownership_v1(",
    "libc::setgroups(0, std::ptr::null())",
    "libc::setgid(gid)",
    "libc::setuid(uid)",
    '.env("HOME", &principal.home)',
    '.env("LIMA_HOME", lima_home)',
    '.env("PATH", path)',
    'const MAC_LIMA_PROFILE_CHILD_FD_V1: libc::c_int = 3;',
    'const MAC_LIMA_COPY_CHILD_FD_V1: libc::c_int = 4;',
    '"/dev/fd/3"',
    '"/dev/fd/4"',
    "libc::dup2(inherited_fd, child_fd)",
    "flags & !libc::FD_CLOEXEC",
    "fn mac_require_absent_lima_owner_overlays_v1(",
    "fn mac_verify_lima_inherited_input_descriptor_v1(",
    "MacLimaInheritedInputRoleV1::PostPmCopySource",
    "MAC_LIMA_ROOT_INPUT_MAX_BYTES_V1",
    "status_flags & libc::O_ACCMODE != libc::O_RDONLY",
    "for descriptor in 3..fd_scan_limit as libc::c_int",
    "mac_verify_lima_control_state_ownership_v1(principal, lima_home)",
    "fs::Permissions::from_mode(0o444)",
]
for token in required:
    if token not in source:
        raise SystemExit(f"missing principal-runner control: {token}")

resolver = source[source.index("fn mac_join_lima_principal_v1("):]
resolver = resolver[:resolver.index("\nfn mac_revalidate_lima_principal_v1(")]
for token in [
    "getpwnam_r",
    "getpwuid_r",
    "pw_uid",
    "pw_gid",
    "pw_dir",
    "uid == 0",
    "gid == 0",
    "mac_require_canonical_absolute_home_v1",
]:
    if token not in resolver:
        raise SystemExit(f"principal resolver does not bind {token}")

runner = source[source.index("fn mac_spawn_fixed_lima_child_after_validation_v1("):]
runner = runner[:runner.index("\n#[cfg(target_os = \"macos\")]\nfn mac_spawn_fixed_lima_stream_child_v1(")]
ordered = [
    "libc::setgroups(0, std::ptr::null())",
    "libc::setgid(gid)",
    "libc::setuid(uid)",
]
positions = [runner.index(token) for token in ordered]
if positions != sorted(positions):
    raise SystemExit("credential drop is not supplementary-groups -> GID -> UID")
if ".env_clear()" not in runner or ".pre_exec(" not in runner:
    raise SystemExit("principal runner does not scrub environment and own pre-exec drop")
for token in [
    "mac_require_root_owned_immutable_tool_path_v1(tool)?",
    "mac_revalidate_lima_principal_v1(carrier, principal)?",
    "mac_validate_fixed_lima_argument_plan_v1(arguments, inherited_input)?",
    "mac_verify_lima_inherited_input_descriptor_v1(inherited_input)?",
    "mac_require_absent_lima_owner_overlays_v1(principal, lima_home)?",
]:
    if token not in runner:
        raise SystemExit(f"production runner does not preserve validation gate: {token}")
if "child_input.identity != input.identity" not in runner:
    raise SystemExit("fresh per-spawn inherited input does not exact-join retained physical identity")

overlay = source[source.index("fn mac_require_absent_lima_owner_overlays_v1("):]
overlay = overlay[:overlay.index("\nfn mac_verify_lima_control_state_ownership_v1(")]
for token in [
    'Path::new(&principal.home)',
    'expected.as_path()',
    'config.as_path()',
    'metadata.mode() & 0o022 != 0',
]:
    if token not in overlay:
        raise SystemExit(f"owner-overlay gate does not harden all canonical directories: {token}")

outcome = source[source.index("fn mac_run_fixed_lima_command_outcome_v1("):]
outcome = outcome[:outcome.index("\n#[cfg(target_os = \"macos\")]\nfn mac_collect_fixed_lima_child_outcome_v1(")]
for token in [
    "mac_validate_lima_after_command_v1(carrier, principal, lima_home)",
    "mac_join_fixed_lima_outcome_and_post_validation_v1(outcome, post_validation)",
]:
    if token not in outcome:
        raise SystemExit(f"production post-command drift gate is missing: {token}")

guard = source[source.index("impl R6RetainedLimaChildGuardV1 {"):]
guard = guard[:guard.index("\n#[cfg(target_os = \"macos\")]\nimpl std::ops::Deref")]
for token in [
    "mac_validate_lima_after_command_v1(",
    "mac_join_r6_exit_status_and_post_validation_v1(status, post_validation)",
]:
    if token not in guard:
        raise SystemExit(f"R6 terminal acceptance omits preserving-first post-validation: {token}")
guard_drop = source[source.index("impl Drop for R6RetainedLimaChildGuardV1 {"):]
guard_drop = guard_drop[:guard_drop.index("\nfn main()")]
if "mac_require_absent_lima_owner_overlays_v1(&self.principal, &self.lima_home)" not in guard_drop:
    raise SystemExit("R6 disconnect cleanup omits owner-overlay drift observation")

dual_failure_test = source[source.index("fn lima_nonzero_outcome_preserves_first_when_post_validation_also_fails("):]
dual_failure_test = dual_failure_test[:dual_failure_test.index("\n    #[", 5)]
for token in ["Some(77)", "first child failure", "later owner-overlay drift", "first < later"]:
    if token not in dual_failure_test:
        raise SystemExit(f"dual-failure preserving-first regression is incomplete: {token}")

r6_dual_failure_test = source[source.index("fn r6_nonzero_exit_preserves_first_when_post_validation_also_fails("):]
r6_dual_failure_test = r6_dual_failure_test[:r6_dual_failure_test.index("\n    #[", 5)]
for token in ["Some(91)", "later R6 owner-overlay drift", "first < later"]:
    if token not in r6_dual_failure_test:
        raise SystemExit(f"R6 dual-failure preserving-first regression is incomplete: {token}")

live_proof = source[source.index("fn live_lima_principal_child_drops_root_groups_and_reads_only_profile_fd("):]
live_proof = live_proof[:live_proof.index("\n    #[test]", 5)]
for token in [
    "extern char **environ;",
    "unexpected environment key",
    'fputs("environment=exact\\n", stdout)',
    'output.contains("environment=exact\\n")',
    "same-byte replacement inode must be denied",
    "fresh per-spawn Lima input changed retained physical identity",
    '("drift-default-proof", "default.yaml")',
    '("drift-override-proof", "override.yaml")',
    "mac_validate_lima_post_effect_state_v1(&proof_principal, &lima_home)",
    "nonzero_arguments",
    "Some(&profile)",
    "timeout_arguments",
    "Some(&copy_source)",
    "disconnect_arguments",
    "MacFixedLimaChildIoV1::Streaming",
]:
    if token not in live_proof:
        raise SystemExit(f"bounded live principal/FD proof is incomplete: {token}")

spawn_arguments = [
    value.strip()
    for value in re.findall(r"(?:std::process::)?Command::new\(([^)\n]*)", source)
]
if any("limactl" in value or "MAC_LIMA_TOOL_PATH_V1" in value for value in spawn_arguments):
    raise SystemExit("a retained publisher limactl call directly names the tool outside the runner")
variable_spawns = [
    value
    for value in spawn_arguments
    if not value.startswith('"')
    and value != "&fake"
    and not value.startswith("std::env::current_exe(")
]
if variable_spawns != ["tool"] or "Command::new(tool)" not in runner:
    raise SystemExit(
        f"a variable executable bypasses the one principal runner: {variable_spawns}"
    )

stage_one = source[source.index("fn execute_closed_mac_lima_stage_one_effect_v1("):]
stage_one = stage_one[:stage_one.index("\n#[cfg(not(target_os = \"macos\"))]")]
if "&principal" not in stage_one or "MacLimaInheritedInputV1" not in source:
    raise SystemExit("Stage-1 does not use the exact principal and retained profile descriptor")
if '"start"' not in stage_one or "MAC_LIMA_PROFILE_CHILD_PATH_V1" not in stage_one:
    raise SystemExit("Stage-1 start does not consume the inherited read-only profile FD")

post_pm = source[source.index("fn mac_execute_post_pm_effect_plan_v1("):]
post_pm = post_pm[:post_pm.index("\n/// Read only the exact action-specific observation")]
for token in [
    "mac_run_fixed_lima_copy_source_v1(",
    "MacPostPmEffectPrimitiveV1::Artifact",
    "MacPostPmEffectPrimitiveV1::Embedded",
]:
    if token not in post_pm:
        raise SystemExit(f"post-PM root-private copy bypasses inherited FD handoff: {token}")
if ".display().to_string()" in post_pm:
    raise SystemExit("post-PM copy exposes a root-private pathname to non-root limactl")

r6 = source[source.index("fn open_pm_bound_guest_pairing_data_session_v1("):]
r6 = r6[:r6.index("\nfn ", 5)]
if "mac_spawn_fixed_lima_stream_child_v1(" not in r6 or "Command::new(tool)" in r6:
    raise SystemExit("R6 retained data limactl child bypasses the principal stream runner")

for forbidden in [
    'format!("{}/.lima", principal.home)',
    "sudo -u",
    "/bin/sh -c",
]:
    if forbidden in runner:
        raise SystemExit(f"principal runner contains forbidden wrapper/derivation: {forbidden}")

print(f"[{Path(sys.argv[1]).name}] principal-runner source controls verified")
PY

printf '[%s] PASS: retained macOS limactl calls are bound to the exact non-root principal runner and profile FD\n' "${SCRIPT_NAME}"
