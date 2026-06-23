# TASKS-63: Retained World Worker Parked-Resume Session-Handle Contract

Source spec: [SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md](./SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md)  
Source plan: [PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md](./PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md)  
Phase: `TASKS`  
Execution model: five sequential implementation packets  
Status: draft for review

## Phase Gate

These tasks assume:

1. retained continuity and process continuity must be separated once resumable identity has been surfaced,
2. the smallest acceptable implementation is preserving the existing retained registry entry in place,
3. a minimal internal split between retained membership and active bootstrap/turn bookkeeping is acceptable only if preserving the entry in place becomes too tangled,
4. non-zero submitted-turn exit must not automatically kill retained continuity once resumable identity exists,
5. v1 should prefer inferred parked truth over adding a new persisted parked field,
6. the lifecycle seam must be proven in `world-service` first, then one shell/public regression should prove routing truth.

Do not begin implementation until the spec and plan are accepted.

If implementation evidence proves a new persisted parked field is required or forces a broader contract change, stop and update the spec/plan before continuing.

## Execution Packets

This slice should be implemented as five sequential packets:

1. pin the lifecycle seam in `world-service` tests,
2. separate retained registry ownership from bootstrap-process ownership,
3. make parked submitted-turn resume work and keep turn failure separate from worker death,
4. keep one shell/public regression for exact-target routing truth,
5. run the final validation wall.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Pin The Lifecycle Seam In World-Service

Session goal:

1. reproduce the actual bug where bootstrap exits after surfacing resumable identity,
2. lock success to retained continuity surviving that exit,
3. lock failure to the inverse case where no resumable handle was surfaced.

### Tasks

- [ ] Task 1.1: Add focused `world-service` lifecycle regression coverage for bootstrap-exit parked resume
  - Acceptance: `member_runtime` coverage explicitly proves the joined seam in runtime terms: a retained worker registers, surfaces resumable session identity, exits bootstrap cleanly, remains resumable for a later submit-turn, and the inverse no-session-handle path fails closed instead of pretending parked continuity.
  - Verify:
    - `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
    - [`crates/world-service/tests/`](../crates/world-service/tests/) only if a black-box lifecycle harness is strictly needed

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the active lifecycle seam reproduces in automation,
2. the positive parked-resume success condition is explicit,
3. the no-session-handle inverse case is pinned,
4. the proof does not depend on bootstrap process liveness being artificially held open.

Do not start Packet 2 until Packet 1 is reviewed and green.

## Packet 2: Preserve Retained Continuity Across Clean Bootstrap Exit

Session goal:

1. stop treating bootstrap exit as retained-worker deletion,
2. keep the runtime change minimal and local to `world-service`,
3. preserve exact retained slot ownership semantics.

### Tasks

- [ ] Task 2.1: Separate retained registry ownership from bootstrap-process cleanup
  - Acceptance: once authoritative retained identity plus surfaced resumable session identity exist, clean bootstrap exit no longer unregisters the retained worker; bootstrap/launcher cleanup remains allowed, but retained continuity survives independently of process liveness.
  - Verify:
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `rg -n "unregister_member|remember_uaa_session_id|launcher_dir|bootstrap" crates/world-service/src/member_runtime.rs`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

- [ ] Task 2.2: Keep parked truth inferred rather than adding a persisted parked field
  - Acceptance: the repaired runtime can represent a parked retained worker using existing retained-registry truth plus no active bootstrap/turn slot plus surfaced resumable handle, without introducing a new persisted parked field; if implementation proves that inference is ambiguous or incorrect, the task must stop for spec/plan update instead of silently widening the contract.
  - Verify:
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `rg -n "parked|resume|active_turn_span_id|uaa_session_id" crates/world-service/src/member_runtime.rs`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. retained continuity survives clean bootstrap exit,
2. no duplicate retained-slot or stale-owner drift is introduced,
3. bootstrap resource cleanup no longer implies retained closeout,
4. no new persisted parked field has been added without explicit review.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Make Parked Resume Work And Keep Turn Failure Non-Terminal By Default

Session goal:

1. let `submit_turn` resume parked retained workers via surfaced session handle,
2. preserve exact binding validation,
3. keep non-zero submitted-turn exit separate from retained-worker death.

### Tasks

- [ ] Task 3.1: Make `submit_turn` resume parked retained workers through preserved retained membership
  - Acceptance: `submit_turn` can locate and resume a parked retained worker through the preserved retained registry entry using surfaced `uaa_session_id`, while exact validation of `participant_id`, `orchestration_session_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, and `world_generation` remains fail-closed.
  - Verify:
    - `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
    - [`crates/world-service/tests/`](../crates/world-service/tests/) only if the lifecycle seam cannot be proven honestly in file-local tests

- [ ] Task 3.2: Keep non-zero submitted-turn exit from automatically killing retained continuity
  - Acceptance: after resumable identity exists, non-zero submitted-turn exit cleans up active-turn bookkeeping but does not automatically unregister or delete the retained worker unless explicit stop, invalidation, or equivalent terminal closeout semantics occur.
  - Verify:
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `rg -n "submit_turn|unregister_turn|clear_reserved_turn_slot|unregister_member" crates/world-service/src/member_runtime.rs`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. later submit-turn succeeds after clean bootstrap exit,
2. exact-binding mismatch still fails closed,
3. missing surfaced resume handle still fails directly,
4. non-zero submitted-turn exit no longer implies retained-worker deletion,
5. explicit terminal closeout still removes resumability.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Keep One Shell/Public Regression For Exact-Target Routing Truth

Session goal:

1. prove the repaired runtime seam still aligns with shell/control-plane targeting,
2. avoid widening the fix into a shell-side lifecycle redesign,
3. keep one durable public proof wall.

### Tasks

- [ ] Task 4.1: Extend one Linux public regression across the bootstrap-exits-then-resume seam
  - Acceptance: one shell/public regression proves that `continue_world_worker` still targets the exact retained participant and succeeds after the retained worker has already exited bootstrap and entered parked/resumable posture; the regression must not rely solely on a held-open member runtime.
  - Verify:
    - `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture`
    - `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) only if a narrow routing mismatch is uncovered

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the public regression proves exact-target routing across parked handoff,
2. shell-side exact-target validation remains unchanged,
3. no broader shell lifecycle redesign has been introduced.

Do not start Packet 5 until Packet 4 verification is green.

## Packet 5: Final Validation Wall

Session goal:

1. prove the slice is fixed end to end,
2. keep the proof wall narrow to this seam,
3. confirm adjacent slices were not reopened.

### Tasks

- [ ] Task 5.1: Run the targeted validation wall for the retained parked/resume seam
  - Acceptance: formatting, lint, targeted `world-service` lifecycle tests, and the single shell/public parked-follow-up regression are green after the repair.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture`
    - `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture`
    - `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`
  - Files:
    - no planned source edits; validation only

### Packet 5 Checkpoint

Packet 5 is complete only when:

1. the world-service lifecycle seam is proven green,
2. the shell/public exact-target proof is green,
3. the slice did not widen into `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.
4. Packet 4 blocks Packet 5.

## Inter-Packet Review Rules

After each packet:

1. confirm its checkpoint is satisfied,
2. confirm retained continuity is no longer coupled to bootstrap-process liveness,
3. confirm non-zero submitted-turn exit still does not automatically kill retained continuity,
4. confirm no new persisted parked field was added unless explicitly justified and reviewed,
5. confirm the slice remains bounded to the active retained parked/resume bug seam.
