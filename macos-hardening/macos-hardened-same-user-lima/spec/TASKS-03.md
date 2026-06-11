# TASKS-03: Canonical Guest Endpoint and Transport Contract

Source spec:
- [`SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](./SPEC-03-canonical-guest-endpoint-and-transport-contract.md)

Source plan:
- [`PLAN-03.md`](./PLAN-03.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`  
Status: draft task set  
Execution model: three sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `03` as the next seam,
2. Slice `04` remains reserved for PTY/non-PTY/doctor/readiness transport
   convergence,
3. the implementation owner accepts that this slice is source-driven and must
   use current official Lima docs for forwarding and SSH semantics,
4. GitNexus impact analysis is treated as mandatory before editing any touched
   transport symbols, and `gitnexus_detect_changes()` is treated as mandatory
   before committing,
5. top-level macOS docs and shell doctor UX remain out of default scope unless
   an explicit contradiction requires expansion.

## Slice contract

This slice should land the first code-owned transport authority for:

1. the canonical guest endpoint `/run/substrate.sock`,
2. the managed host UDS path for the Lima-backed path,
3. the retained compatibility TCP port, only if one still exists after
   centralization,
4. transport-kind metadata and endpoint derivation shared across the
   backend-owned transport surfaces,
5. the rule that SSH TCP fallback is not automatic default forwarding when the
   guest service remains UDS-only.

This slice must **not**:

1. implement broad PTY/non-PTY/doctor/readiness convergence,
2. rewrite `docs/WORLD.md` or `docs/reference/world/platforms/macos-lima-setup.md`,
3. widen into backend policy parity,
4. widen into ingress, mount, listener, or guest-unit hardening work,
5. reopen Slice `02` support or breakglass classifications.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`
2. `crates/world-mac-lima/src/transport.rs`
3. `crates/world-mac-lima/src/forwarding.rs`
4. `crates/world-mac-lima/src/lib.rs`
5. `crates/shell/src/execution/platform_world/mod.rs`
6. `crates/shell/src/builtins/world_gateway.rs` only if needed to remove a
   direct contract contradiction
7. targeted tests for the touched transport surfaces only

Treat edits to `crates/shell/src/execution/platform/macos.rs`,
`crates/shell/src/execution/routing/dispatch/world_ops.rs`,
`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`,
`scripts/mac/lima-doctor.sh`, or top-level macOS docs as scope expansion unless
the orchestrator can point to a direct contradiction that the user explicitly
approves for this slice.

## Execution packets

### Packet 1: Shared transport authority and stale-constant freeze

Session goal:

1. confirm the authority stack, source gate, and GitNexus gate,
2. centralize the canonical transport constants and endpoint descriptions,
3. remove stale `7788` from the transport authority itself.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and symbol-impact gate
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `1`, milestone `1.1`,
    `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    port-forwarding, SSH, `limactl shell`, VZ, and breaking-changes docs.
    Before editing any touched symbol, run GitNexus impact analysis for the
    transport symbols that will change. If GitNexus reports a stale index,
    refresh it first.
  - Verify:
    - manual authority review
    - manual official-source review
    - GitNexus impact review recorded before edits
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md`

- [x] Task 1.2: Centralize the canonical transport authority in `world-mac-lima`
  - Acceptance: one shared authority in `world-mac-lima` defines the canonical
    guest socket path, managed host UDS path, retained compatibility TCP port
    if applicable, and transport-kind metadata. `transport.rs` no longer
    advertises `127.0.0.1:7788`.
  - Verify:
    - `rg -n "7788|17788|/run/substrate.sock|agent.sock" crates/world-mac-lima/src`
    - `cargo test -p world-mac-lima -- --nocapture`
  - Files:
    - `crates/world-mac-lima/src/transport.rs`
    - `crates/world-mac-lima/src/forwarding.rs`
    - targeted tests in `crates/world-mac-lima/src/`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. there is one obvious transport authority for the macOS backend,
2. stale `7788` is gone from the centralized transport surface,
3. the slice has not yet absorbed doctor/readiness or top-level docs work.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Backend adoption and minimal shell-facing alignment

Session goal:

1. make backend transport call sites consume the shared authority,
2. align the minimum shell-facing transport mappings needed to remove direct
   contradiction,
3. keep broader consumer convergence deferred to Slice `04`.

#### Tasks

- [x] Task 2.1: Remove backend-local transport drift
  - Acceptance: `crates/world-mac-lima/src/lib.rs` and related backend transport
    paths consume the shared authority rather than stale ad hoc literals.
    Backend probing no longer targets `7788`, and forwarding code keeps the
    guest UDS-first contract explicit.
  - Verify:
    - `rg -n "7788|17788|/run/substrate.sock|agent.sock" crates/world-mac-lima/src`
    - `cargo test -p world-mac-lima -- --nocapture`
  - Files:
    - `crates/world-mac-lima/src/lib.rs`
    - `crates/world-mac-lima/src/forwarding.rs`
    - `crates/world-mac-lima/src/transport.rs`

- [x] Task 2.2: Align shell-visible transport mapping without absorbing Slice `04`
  - Acceptance: `crates/shell/src/execution/platform_world/mod.rs` and
    `crates/shell/src/builtins/world_gateway.rs` no longer contradict the
    backend-owned transport contract on guest socket/host socket/compatibility
    port facts. If `17788` remains, it has one explicit meaning and one shared
    value. `crates/shell/src/execution/platform/macos.rs` remains deferred
    unless a direct contradiction forces user-approved expansion.
  - Verify:
    - `rg -n "7788|17788|agent.sock|SUBSTRATE_WORLD_SOCKET" crates/shell/src/execution/platform_world/mod.rs crates/shell/src/builtins/world_gateway.rs`
    - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
    - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
  - Files:
    - `crates/shell/src/execution/platform_world/mod.rs`
    - `crates/shell/src/builtins/world_gateway.rs` only if required
    - targeted tests for the touched shell surfaces

### Packet 2 checkpoint

Packet `2` is complete only when:

1. backend and shell-visible transport mappings no longer disagree on the same
   contract,
2. any retained `17788` path is explicit compatibility material rather than a
   scattered magic literal,
3. SSH TCP fallback is not described as automatic default forwarding,
4. doctor/readiness convergence still remains future Slice `04` work.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Targeted validation and next-slice handoff clarity

Session goal:

1. validate that Slice `03` stayed narrow,
2. run final targeted tests and GitNexus change-scope verification,
3. leave a clean handoff to Slice `04`.

#### Tasks

- [x] Task 3.1: Final targeted regression and GitNexus scope check
  - Acceptance: the final implementation passes targeted tests, formatting
    remains clean, and GitNexus detect-changes confirms the affected symbols and
    execution flows are limited to the intended transport contract surfaces.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo test -p world-mac-lima -- --nocapture`
    - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
    - `cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture`
    - `git diff --stat -- crates/world-mac-lima crates/shell/src/execution/platform_world/mod.rs crates/shell/src/builtins/world_gateway.rs macos-hardening/macos-hardened-same-user-lima/spec`
    - `git status --short`
    - `gitnexus_detect_changes()` before committing
  - Files:
    - all files touched by this slice only as required by final cleanup

- [x] Task 3.2: Validate explicit deferral to Slice `04`
  - Acceptance: the touched docs and final closeout make it explicit that
    PTY/non-PTY/doctor/readiness transport convergence remains Slice `04`, and
    that top-level operator-doc cutover remains later work.
  - Verify:
    - `rg -n "Slice 04|doctor|readiness|PTY|non-PTY|docs cutover" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. Slice `03` stayed bounded,
2. GitNexus scope verification is consistent with the intended transport seam,
3. Slice `04` remains the next honest consumer-convergence slice,
4. the planning stack is coherent enough for a short future prompt to continue
   without hidden assumptions.

## Prompt artifact expectation

After Packet `3` planning is stable, the slice should have one ready-to-paste
prompt artifact for fresh orchestration sessions. That prompt artifact should:

1. provide one prompt per packet,
2. require official-Lima source verification before any transport-contract
   change,
3. require GitNexus impact analysis before symbol edits and
   `gitnexus_detect_changes()` before commits,
4. require a fresh GPT-5.4 high implementation subagent using
   `$incremental-implementation`,
5. require a fresh GPT-5.4 high review subagent using
   `$code-review-and-quality`,
6. require fix subagents when review finds issues,
7. require commits between implementation, review-driven fix rounds, and the
   next packet boundary.
