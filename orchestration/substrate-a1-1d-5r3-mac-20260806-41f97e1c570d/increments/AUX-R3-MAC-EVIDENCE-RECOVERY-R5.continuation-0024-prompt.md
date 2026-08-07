R5 CONTINUATION 0024 — MIGRATE EXACT CALLERS AND COMPLETE THE EXISTING BOOTSTRAP FD3 CONTRACT

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- continuation_nonce: f426b9c13f0079bc1e3d28ad13db88c61ff7144ee00198e01f11c561b9705c63
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- live remote must remain the expected base
- expected preserved two-test binary diff SHA-256: 9e4ffa66e20bc4d46c4885ef519cbf2b0b6afad0356fa6b10164e58fc953976c
- expected complete-file SHA-256 values:
  - crates/shell/tests/managed_lifecycle_v1.rs: 66fb76aed33596512e0df259db80ee25e5a4a361f05b718ba687b3674d3aeaee
  - tests/mac/lifecycle_r3.sh: 46abcb154bbe6ee99ed0bfcb87c715ec8bb178cc2697c32744b21378e82cd5d6

This blocker is adjudicated inside R5. Reverify those bindings first and preserve the existing
focused failing tests. Amendment 0024 corrects the path fence; it does not authorize R6.

## 1. Exact caller migration

Add only these caller surfaces to the R5 production fence:

- scripts/mac/lima-warm.sh: destroy_vm, ensure_vm_ready, stage_workspace,
  ensure_substrate_group, install_agent_from_host, install_cli_from_host,
  install_gateway_from_host, install_guest_binaries, bootstrap_guest_private_home,
  write_systemd_units, enable_socket_activation, write_layout_sentinel, and configure_guest.
- scripts/mac/lima-stop.sh: resolve_lima_stop_authority_v1 and invoke_mapped_lima_stop_v1.

Migrate every listed invocation from its legacy generic selector to exactly one literal wrapper tag:
stage_one_create or post_pm_action. stage_one_create is only the absent selected
mac.lima.instance/Create path with complete Stage-1 joins. Every existing-instance or guest
projection action is post_pm_action with the exact already-authorized closed role/action request,
and it must not carry LimaStageOneAuthorizationV1. Validate the supplied canonical request and fail
closed on mismatch; do not translate a legacy string into a caller-selected role/action. Remove all
generic compatibility and raw helper paths.

Minimum immediate private helpers in those two scripts are allowed only when directly reachable
from the named functions and recorded in the path-and-symbol ledger. No other caller path is mutable.

## 2. Existing retained bootstrap FD3, not a new transport

The landed control pack already specifies the retained macOS bootstrap channel: AF_UNIX
SOCK_SEQPACKET|SOCK_CLOEXEC, only the peer on descriptor 3, exact elevated executor, and
--publisher-bootstrap-fd 3. Complete that existing R5 path inside the original R5 production fence:

1. substrate-lifecycle-control must dispatch publisher-bootstrap before reading an ordinary stdin
   ManagedLifecycleControlRequestV1. After controlling-terminal confirmation, independently derive
   and issue the exact canonical PublisherBootstrapAuthorizationV1 from authoritative retained state
   and immediately send only it over the retained channel.
2. substrate-lifecycle-macos may add the exact --publisher-bootstrap-fd 3 entrypoint and minimum
   immediate frame/peer helpers required to consume one canonical authorization, verify the required
   peer, image, build, and terminal joins, execute bootstrap, and return one bounded response.
3. managed_lifecycle.rs and macos_client.rs may implement the exact socketpair, FD3-preserving exact
   executor launch, and response plumbing already required by 04-contracts-and-gates.md.
4. PublisherBootstrapAuthorizationV1 must not be a ManagedLifecycleControlRequestV1 field or arrive
   through ordinary stdin JSON, argv, environment, file, generated projection, script output, or a
   reusable record. Missing state, TTY, FD3, join, frame boundary, or attestation fails closed.

This is not authority for a new endpoint, transport family, generic session broker, or R6 guest
pairing session. Do not touch forwarding or guest-session code.

## 3. Native linker closure

Within the already allowed macos_client.rs path, replace the incorrect standalone xpc linker
declaration with the existing repository macOS System-library convention so the native R5 test can
link. Do not install an SDK/library/toolchain or add a dependency.

## 4. Test, review, and publication

Tests remain limited to crates/shell/tests/managed_lifecycle_v1.rs and tests/mac/lifecycle_r3.sh.
Add exact assertions for migrated callers, two-tag-only wrapper behavior, pre-stdin direct-bootstrap
dispatch, nonserialization, FD3 one-frame delivery, and fail-closed joins. Freeze the exact nine-path
implementation/test subject: the original seven R5 paths plus lima-warm.sh and lima-stop.sh.

No review cycle began before this blocker. After deterministic MAC-only proof is green, run the
normal fresh discovery burst/review and a different fresh closure review. Remediate valid P1/P2 only;
at most two causal supplemental cycles remain. Record P3/P4 in 06 without remediation cycles.

All original R5 descriptor, role/action, audit-before-decode, preservation, MAC-only, donor,
GitNexus-degraded/manual-fallback, subagent/reviewer, and one-fast-forward-publication boundaries
remain. Do not run Linux/Windows target commands, native actions, installation, Keychain/XPC/Lima
mutation, evidence, or npx gitnexus analyze.

On terminal CLEAN, publish exactly one R5 commit and return the original nonce-bound LANDED_CLEAN
receipt with next_increment AUX-R3-MAC-EVIDENCE-RECOVERY-R6. Do not dispatch R6. Use
gpt-5.6-terra at Extra High for every subagent and reviewer.
