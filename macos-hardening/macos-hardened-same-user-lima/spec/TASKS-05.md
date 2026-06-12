# TASKS-05: Backend Policy Input Parity

Source spec:
- [`SPEC-05-backend-policy-input-parity.md`](./SPEC-05-backend-policy-input-parity.md)

Source plan:
- [`PLAN-05.md`](./PLAN-05.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `05` as the next seam,
2. Slice `06` remains reserved for routed-path-first doctor/smoke/readiness
   truth plus docs/script cutover,
3. the implementation owner accepts that this slice is repo-first and only
   needs targeted external-source validation if implementation unexpectedly
   widens into guest lifecycle or transport semantics,
4. GitNexus impact analysis is treated as mandatory before editing any touched
   contract or backend symbols, and `gitnexus_detect_changes()` is treated as
   mandatory before committing,
5. helper scripts and top-level macOS docs remain out of default scope unless
   an explicit contradiction requires expansion.

## Slice contract

This slice should land one explicit parity contract for:

1. carrying broker-resolved policy/world inputs into backend-mediated macOS
   execution,
2. removing permissive backend-local policy synthesis,
3. keeping `world_network` and related world-routing truth intact on the
   backend path,
4. making `apply_policy(...)` semantically real for the macOS backend,
5. preserving `shared_world` propagation while parity is fixed.

This slice must **not**:

1. rewrite `scripts/mac/lima-doctor.sh` or `scripts/mac/smoke.sh`,
2. rewrite `docs/WORLD.md` or
   `docs/reference/world/platforms/macos-lima-setup.md`,
3. reopen the Slice `03` / Slice `04` transport contract,
4. redesign broker policy language,
5. widen into ownership-boundary, ingress, listener, or guest-unit hardening
   work.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `crates/world-api/src/lib.rs`,
3. `crates/world-mac-lima/src/lib.rs`,
4. `crates/shell/src/execution/policy_snapshot.rs` only if the widened
   contract needs a shared authoritative builder update,
5. `crates/shell/src/execution/routing/dispatch/world_ops.rs` only if a
   backend-facing builder/helper must be aligned,
6. `crates/shell/src/repl/async_repl.rs` only if persistent-session bootstrap
   needs the same parity carrier,
7. `crates/shell/src/builtins/world_gateway.rs` only if a shared backend helper
   is reused there,
8. `crates/replay/src/replay/executor.rs` only as shared-contract fallout if
   `WorldSpec` / `ExecRequest` shape changes require it,
9. targeted tests for the touched shared-contract and backend surfaces only.

Treat edits to `scripts/mac/`, top-level docs, or transport-contract files as
scope expansion unless the orchestrator can point to a direct contradiction
that the user explicitly approves for this slice.

## Execution packets

### Packet 1: Carrier contract freeze and symbol-impact gate

Session goal:

1. confirm the authority stack, repo truth, and GitNexus gate,
2. freeze the exact backend-facing parity carrier seam,
3. identify the exact symbols whose change determines the slice blast radius.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, repo-floor gap, and symbol-impact gate
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `1`, milestone `1.2`, milestone
    `1.3`, `DESIGN-macos-policy-input-parity.md`, and the live repo truth in
    `crates/world-api/src/lib.rs`, `crates/world-mac-lima/src/lib.rs`,
    `crates/shell/src/execution/policy_snapshot.rs`,
    `crates/shell/src/execution/routing/dispatch/world_ops.rs`,
    `crates/shell/src/repl/async_repl.rs`, and
    `crates/replay/src/replay/executor.rs`. Before editing any touched symbol,
    run GitNexus impact analysis for the shared-contract and backend symbols
    that will change. If GitNexus reports a stale index, refresh it first. If
    any relevant symbol returns `HIGH` or `CRITICAL` impact, surface that
    warning before editing proceeds.
  - Verify:
    - manual authority review
    - manual repo-truth review
    - GitNexus impact review recorded before edits
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md`

- [x] Task 1.2: Freeze the backend-facing parity carrier boundary
  - Acceptance: the slice identifies one obvious place where backend-mediated
    macOS execution obtains resolved policy/world inputs. The docs make it
    explicit whether the carrier lives on `WorldSpec`, `ExecRequest`, or an
    adjacent shared backend type, and they keep that decision bounded rather
    than inventing a macOS-only side channel.
  - Verify:
    - `rg -n "WorldSpec|ExecRequest|policy_snapshot|world_network|apply_policy|convert_exec_request" crates/world-api/src/lib.rs crates/world-mac-lima/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs crates/replay/src/replay/executor.rs`
    - manual coherence review
  - Files:
    - `crates/world-api/src/lib.rs` only if Packet `2` carrier shaping is already obvious
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. there is one obvious backend-facing parity carrier to aim at,
2. the affected symbols and fallout surfaces are explicit,
3. any `HIGH` / `CRITICAL` GitNexus warnings have been surfaced before edits,
4. the slice has not yet absorbed Slice `06` docs/script cutover.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Widen the shared backend contract

Session goal:

1. widen the shared backend contract just enough to carry policy/world-input
   truth,
2. keep serde/default behavior coherent,
3. limit fallout to intentional shared consumers.

#### Tasks

- [x] Task 2.1: Add the minimum shared parity carrier
  - Acceptance: `crates/world-api/src/lib.rs` or an adjacent shared backend
    contract can carry the resolved policy/world inputs needed by the macOS
    backend. The carrier no longer forces backend-local reconstruction from
    `WorldFsMode` alone. Serialization/default behavior stays coherent, and any
    replay or test fallout is intentional rather than accidental.
  - Verify:
    - `cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture`
    - `rg -n "WorldSpec|ExecRequest|policy_snapshot|world_network" crates/world-api/src/lib.rs crates/replay/src/replay/executor.rs crates/world-mac-lima/src/lib.rs`
  - Files:
    - `crates/world-api/src/lib.rs`
    - `crates/replay/src/replay/executor.rs` only if the shared type shape changes require it
    - targeted shared-contract tests

- [x] Task 2.2: Align authoritative builders to the widened carrier without creating a second policy source of truth
  - Acceptance: whichever shell-side builder or bootstrap helper feeds the
    backend path now populates the widened carrier from authoritative
    broker/shell-resolved inputs. No second local policy-resolution path is
    invented to make the backend compile.
  - Verify:
    - `cargo test -p shell world_network_policy_canonicalizes_snapshot_net_allowed -- --nocapture`
    - `cargo test -p shell world_network_policy_requests_isolation_for_restrictive_allowlist -- --nocapture`
    - manual diff review against `crates/shell/src/execution/policy_snapshot.rs`
  - Files:
    - `crates/shell/src/execution/policy_snapshot.rs` only if required
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs` only if required
    - `crates/shell/src/repl/async_repl.rs` only if required
    - `crates/shell/src/builtins/world_gateway.rs` only if required

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the shared backend contract can represent the required parity inputs,
2. authoritative shell/broker resolution still feeds that contract,
3. no macOS-only hidden side channel was introduced to avoid shared-contract
   widening.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Implement backend parity semantics

Session goal:

1. make `world-mac-lima` consume the widened carrier,
2. remove permissive backend-local policy synthesis,
3. make `apply_policy(...)` semantically real without widening into Slice `06`.

#### Tasks

- [x] Task 3.1: Remove synthetic permissive snapshot generation from `convert_exec_request(...)`
  - Acceptance: `crates/world-mac-lima/src/lib.rs`
    `MacLimaBackend::convert_exec_request(...)` consumes the carried
    policy/world-input truth instead of synthesizing a permissive
    `PolicySnapshotV3` from `WorldFsMode` alone. The backend-mediated request
    path now preserves `world_network` and retains `shared_world`.
  - Verify:
    - `cargo test -p world-mac-lima convert_exec_request_propagates_env_fs_mode -- --nocapture`
    - `cargo test -p world-mac-lima -- --nocapture`
    - `rg -n "convert_exec_request|PolicySnapshotV3|world_network|shared_world" crates/world-mac-lima/src/lib.rs`
  - Files:
    - `crates/world-mac-lima/src/lib.rs`
    - targeted `world-mac-lima` tests

- [x] Task 3.2: Make `apply_policy(...)` semantically real without widening into docs or transport redesign
  - Acceptance: `MacLimaBackend::apply_policy(...)` no longer only stores
    `fs_mode`. It now updates, reconciles, or fail-closes backend session
    policy state by design, and that behavior is explicit enough for later
    readiness evidence work. The slice still avoids docs/script cutover and
    transport redesign.
  - Verify:
    - `cargo test -p world-mac-lima -- --nocapture`
    - manual review that `apply_policy(...)` is no longer a semantic no-op
    - `rg -n "apply_policy|fs_mode|policy_snapshot|world_network" crates/world-mac-lima/src/lib.rs`
  - Files:
    - `crates/world-mac-lima/src/lib.rs`
    - targeted `world-mac-lima` tests

### Packet 3 checkpoint

Packet `3` is complete only when:

1. backend-mediated macOS execution no longer relies on permissive local policy
   synthesis,
2. `world_network` and related parity inputs no longer disappear on the
   backend path,
3. `apply_policy(...)` is no longer semantically empty,
4. Slice `06` docs/script cutover remains untouched.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and next-slice handoff clarity

Session goal:

1. validate that Slice `05` stayed narrow,
2. run final targeted tests and GitNexus change-scope verification,
3. leave a clean handoff to Slice `06`.

#### Tasks

- [x] Task 4.1: Final targeted regression and GitNexus scope check
  - Acceptance: the final implementation passes targeted tests, formatting
    remains clean, and GitNexus detect-changes confirms the affected symbols
    and execution flows are limited to the intended shared-contract and macOS
    backend parity surfaces.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture`
    - `cargo test -p world-mac-lima -- --nocapture`
    - `cargo test -p shell world_network_policy_canonicalizes_snapshot_net_allowed -- --nocapture`
    - `cargo test -p shell world_network_policy_requests_isolation_for_restrictive_allowlist -- --nocapture`
    - `git diff --stat -- crates/world-api/src/lib.rs crates/world-mac-lima/src/lib.rs crates/shell/src/execution/policy_snapshot.rs crates/shell/src/execution/routing/dispatch/world_ops.rs crates/shell/src/repl/async_repl.rs crates/shell/src/builtins/world_gateway.rs crates/replay/src/replay/executor.rs macos-hardening/macos-hardened-same-user-lima/spec`
    - `git status --short`
    - `gitnexus_detect_changes()` before committing
  - Files:
    - all files touched by this slice only as required by final cleanup

- [x] Task 4.2: Validate explicit deferral to Slice `06`
  - Acceptance: the touched docs and final closeout make it explicit that
    routed-path-first doctor/smoke/readiness truth plus docs/script cutover
    remain Slice `06`, and that Slice `05` only fixed backend policy parity.
  - Verify:
    - `rg -n "Slice 06|doctor|smoke|readiness|docs cutover|policy parity" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md`

### Packet 4 checkpoint

Packet `4` is complete only when:

1. Slice `05` stayed bounded,
2. GitNexus scope verification is consistent with the intended shared-contract
   and backend-parity seam,
3. Slice `06` remains the next honest readiness/docs seam,
4. the planning stack is coherent enough for a short future prompt to continue
   without hidden assumptions.
