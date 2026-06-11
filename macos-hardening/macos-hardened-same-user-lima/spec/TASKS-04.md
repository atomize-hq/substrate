# TASKS-04: PTY, Non-PTY, Doctor, and Readiness Transport Convergence

Source spec:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)

Source plan:
- [`PLAN-04.md`](./PLAN-04.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `04` as the next seam,
2. Slice `05` remains reserved for backend policy input parity,
3. Slice `06` remains reserved for routed-path-first doctor/smoke/readiness
   truth and docs/script cutover,
4. the implementation owner accepts that this slice is source-driven and must
   use current official Lima docs for forwarding, SSH, `limactl shell`, VZ,
   environment-variable, and breaking-change semantics,
5. GitNexus impact analysis is treated as mandatory before editing any touched
   runtime symbols, and `gitnexus_detect_changes()` is treated as mandatory
   before committing,
6. helper scripts and top-level macOS docs remain out of default scope unless
   an explicit contradiction requires expansion.

## Slice contract

This slice should land one shell-side consumer-convergence contract for:

1. selected `WorldTransport` reuse across PTY and persistent-session runtime
   consumers,
2. selected-transport-first doctor/readiness probing in macOS runtime code,
3. consistent `SUBSTRATE_WORLD_SOCKET` override behavior across the touched
   consumers,
4. explicit compatibility-only or fallback classification for any retained TCP
   or guest-direct proof path.

This slice must **not**:

1. implement backend policy input parity,
2. rewrite `scripts/mac/lima-doctor.sh` or `scripts/mac/smoke.sh`,
3. rewrite `docs/WORLD.md` or `docs/reference/world/platforms/macos-lima-setup.md`,
4. widen into ingress, mount, listener, or guest-unit hardening work,
5. reopen Slice `03` transport constants except for tiny reuse-oriented helper
   exports.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `crates/shell/src/execution/platform_world/mod.rs`,
3. `crates/shell/src/execution/platform/macos.rs`,
4. `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`,
5. `crates/shell/src/execution/routing/dispatch/world_ops.rs`,
6. targeted tests for the touched shell transport surfaces only.

Treat edits to `crates/world-mac-lima/`, `scripts/mac/`, `docs/WORLD.md`,
`docs/reference/world/platforms/macos-lima-setup.md`, or backend-policy carrier
contracts as scope expansion unless the orchestrator can point to a direct
contradiction that the user explicitly approves for this slice.

## Execution packets

### Packet 1: Consumer contract freeze and symbol-impact gate

Session goal:

1. confirm the authority stack, source gate, and GitNexus gate,
2. freeze one shell-side consumer contract for selected transport reuse,
3. identify the exact symbols that still duplicate transport behavior.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and symbol-impact gate
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `1`, milestone `1.1`, milestone
    `1.2`, milestone `1.3`, `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    port-forwarding, SSH, `limactl shell`, VZ, environment-variable, and
    breaking-change docs. Before editing any touched symbol, run GitNexus
    impact analysis for the shell-side transport consumers that will change. If
    GitNexus reports a stale index, refresh it first.
  - Verify:
    - manual authority review
    - manual official-source review
    - GitNexus impact review recorded before edits
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md`

- [x] Task 1.2: Freeze the shell-side consumer helper boundary
  - Acceptance: the slice identifies one obvious helper layer or consumer
    contract for turning selected transport into WebSocket, agent-client, or
    readiness connections. The docs make it explicit which shell surfaces are in
    scope now and which remain future slice work.
  - Verify:
    - `rg -n "WorldTransport|SUBSTRATE_WORLD_SOCKET|/v1/stream|doctor|readiness" crates/shell/src/execution/platform_world/mod.rs crates/shell/src/execution/platform/macos.rs crates/shell/src/execution/routing/dispatch/world_persistent_session.rs crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - manual coherence review
  - Files:
    - `crates/shell/src/execution/platform_world/mod.rs` only if helper shaping is required
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. there is one obvious shell-side consumer contract to aim at,
2. the affected symbols and official-source claims are explicit,
3. the slice has not yet absorbed policy parity or docs/script cutover work.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: PTY and persistent-session convergence

Session goal:

1. make PTY and persistent-session bootstrap reuse the same selected transport
   behavior,
2. remove duplicated per-transport WebSocket setup where practical,
3. keep policy payload and world-routing content unchanged.

#### Tasks

- [x] Task 2.1: Converge persistent-session transport use
  - Acceptance: `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
    no longer behaves like an isolated transport consumer. It reuses the
    selected transport contract, preserves async readiness behavior, and keeps
    `SUBSTRATE_WORLD_SOCKET` override handling explicit.
  - Verify:
    - `cargo test -p shell macos_no_override_current_thread_start_uses_async_readiness_without_panic -- --nocapture`
    - `cargo test -p shell macos_socket_override_bypasses_platform_async_readiness -- --nocapture`
  - Files:
    - `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
    - helper surface from Packet `1` only if required

- [x] Task 2.2: Converge PTY transport use without widening into policy work
  - Acceptance: `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    reuses the same transport connection behavior for PTY operations rather than
    maintaining a parallel ladder. Request payload semantics, policy snapshot
    content, and world-network routing remain unchanged.
  - Verify:
    - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
    - targeted `cargo test -p shell -- --list | rg "persistent|macos_"` review as needed before final command selection
    - manual diff review against policy-building code paths
  - Files:
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - helper surface from Packet `1` only if required

### Packet 2 checkpoint

Packet `2` is complete only when:

1. PTY and persistent-session consumers no longer carry obviously divergent
   transport ladders,
2. override behavior remains explicit and test-covered,
3. no backend-policy or world-routing payload work has been silently absorbed.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: macOS doctor/readiness runtime convergence

Session goal:

1. make `platform/macos.rs` prove the selected transport first,
2. keep guest-direct probing explicit fallback material only,
3. avoid widening into helper scripts or top-level docs.

#### Tasks

- [x] Task 3.1: Converge selected-transport-first doctor probing
  - Acceptance: `crates/shell/src/execution/platform/macos.rs` no longer looks
    like an independent transport authority. It tries the selected host-visible
    transport contract first, treats compatibility TCP consistently if retained,
    and leaves guest-direct `limactl shell` probing clearly fallback-only.
  - Verify:
    - `cargo test -p shell doctor_ok_json -- --nocapture`
    - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
    - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
  - Files:
    - `crates/shell/src/execution/platform/macos.rs`
    - helper surface from Packet `1` only if required

- [x] Task 3.2: Preserve endpoint-regression expectations without widening into gateway refactors
  - Acceptance: existing gateway endpoint assumptions still match the converged
    transport consumer story, but the slice does not widen into broader gateway
    lifecycle redesign.
  - Verify:
    - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
    - manual review that `crates/shell/src/builtins/world_gateway.rs` stays
      unchanged unless a direct contradiction required a tiny fix
  - Files:
    - `crates/shell/src/builtins/world_gateway.rs` only if required
    - otherwise no file touch needed beyond regression coverage

### Packet 3 checkpoint

Packet `3` is complete only when:

1. selected-transport-first behavior is visible in the macOS doctor/runtime
   code,
2. guest-direct probing remains fallback-only,
3. helper scripts and top-level docs remain untouched,
4. Slice `06` is still preserved for docs/script cutover.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and next-slice handoff clarity

Session goal:

1. validate that Slice `04` stayed narrow,
2. run final targeted tests and GitNexus change-scope verification,
3. leave a clean handoff to Slice `05` and Slice `06`.

#### Tasks

- [x] Task 4.1: Final targeted regression and GitNexus scope check
  - Acceptance: the final implementation passes targeted tests, formatting
    remains clean, and GitNexus detect-changes confirms the affected symbols and
    execution flows are limited to the intended shell-side transport-consumer
    surfaces.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo test -p shell macos_no_override_current_thread_start_uses_async_readiness_without_panic -- --nocapture`
    - `cargo test -p shell macos_socket_override_bypasses_platform_async_readiness -- --nocapture`
    - `cargo test -p shell doctor_ok_json -- --nocapture`
    - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
    - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
    - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
    - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
    - `git diff --stat -- crates/shell/src/execution/platform_world/mod.rs crates/shell/src/execution/platform/macos.rs crates/shell/src/execution/routing/dispatch/world_persistent_session.rs crates/shell/src/execution/routing/dispatch/world_ops.rs macos-hardening/macos-hardened-same-user-lima/spec`
    - `git status --short`
    - `gitnexus_detect_changes()` before committing
  - Files:
    - all files touched by this slice only as required by final cleanup

- [x] Task 4.2: Validate explicit deferral to Slice `05` and Slice `06`
  - Acceptance: the touched docs and final closeout make it explicit that
    backend policy input parity remains Slice `05`, while routed-path-first
    doctor/smoke/readiness truth and docs/script cutover remain Slice `06`.
  - Verify:
    - `rg -n "Slice 05|Slice 06|policy parity|doctor|readiness|smoke|docs" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-04.md`

### Packet 4 checkpoint

Packet `4` is complete only when:

1. Slice `04` stayed bounded,
2. GitNexus scope verification is consistent with the intended shell-side
   consumer-convergence seam,
3. Slice `05` remains the next honest backend-policy seam,
4. Slice `06` remains the later script/docs cutover seam,
5. the planning stack is coherent enough for a short future prompt to continue
   without hidden assumptions.
