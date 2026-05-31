# TASKS-33: Internal Retained World Worker Continue And Event Bootstrap

Source spec: [SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)  
Source plan: [PLAN-33.md](./PLAN-33.md)  
Source validation note: [NOTE-33-family-1-ordering-after-dispatch-bootstrap.md](./NOTE-33-family-1-ordering-after-dispatch-bootstrap.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: completed on `2026-05-31`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 adds the internal continue contract and exact retained-worker target resolution.
- Packet 2 lands `continue_world_worker` over the existing retained member-turn seam.
- Packet 3 lands the minimal typed worker-event classification layer.
- Packet 4 locks gating, docs, and the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Internal Continue Contract And Exact Target Resolution

Session goal:

1. add `continue_world_worker` to the internal family-1 contract,
2. require exact retained-worker targeting,
3. fail closed before any routing occurs.

### Tasks

- [x] Task 1.1: Extend the internal dispatch contract with `continue_world_worker`
  - Acceptance: the internal family-1 contract has a `continue_world_worker` action, exact payload shape, and typed outcome scaffolding; unsupported action/payload combinations fail closed with stable errors.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [x] Task 1.2: Add authoritative retained-worker target resolution
  - Acceptance: the internal continue path resolves only the authoritative host orchestrator and an exact retained worker in the same orchestration session and authoritative world binding; ambiguous or stale targets fail closed.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `continue_world_worker` exists as an internal contract verb,
2. exact retained-worker target identity is mandatory,
3. no public CLI behavior has changed.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: `continue_world_worker` Over The Existing Retained Member-Turn Seam

Session goal:

1. route internal retained-worker follow-up through the already-landed runtime seam,
2. preserve exact target identity and world binding,
3. avoid inventing a new retained-worker transport path.

### Tasks

- [x] Task 2.1: Route `continue_world_worker` over the retained member-turn/runtime seam
  - Acceptance: the internal continue path submits exact retained follow-up through the existing member-turn/runtime seam; no second execution path is introduced.
  - Verify:
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

- [x] Task 2.2: Return a stable continued-turn outcome envelope
  - Acceptance: `continue_world_worker` returns a typed internal outcome envelope that preserves exact retained-worker identity and surfaced stream metadata needed by later event classification; invalidated or mismatched targets fail closed.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - targeted shell integration tests

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. internal retained-worker continue works through the existing runtime seam,
2. exact target identity is preserved end to end,
3. no alternate retained-worker transport has appeared.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Minimal Typed Worker-Event Classification

Session goal:

1. classify the first retained worker-event subset into stable typed outcomes,
2. keep attention semantics explicit,
3. stop short of approvals, fork, and broader steering classes.

### Tasks

- [x] Task 3.1: Add non-attention worker-event classes for the first retained-worker messaging slice
  - Acceptance: the internal continue outcome can classify and surface `reply`, `progress_update`, `result`, and `failure` without implying richer steering policy or lifecycle verbs.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [x] Task 3.2: Add explicit attention-driving worker-event classes and keep later classes deferred
  - Acceptance: the internal continue outcome can classify `follow_up_question` and `blocked`, set explicit `attention_required` semantics for them, and still reject approval/fork/control-directive classes as out of scope for this slice.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/prompt_fulfillment.rs`](../crates/shell/src/execution/prompt_fulfillment.rs) only if shared parsing reuse is needed
    - targeted shell integration tests

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. the minimal in-scope worker-event subset is typed and stable,
2. attention-driving classes remain explicit,
3. approvals, fork, inspect, cancel, and stop still remain out of scope.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Minimal Gating, Docs, And Validation

Session goal:

1. lock the minimum safe boundary for the first retained-worker continue slice,
2. align planning/docs with the actual post-32 runtime truth,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Enforce exact boundary checks for continued retained-worker turns
  - Acceptance: unsupported callers, cross-session requests, cross-world-binding requests, stale retained targets, and unsupported worker-event classes fail closed with stable deny buckets.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 4.2: Align the planning/docs truth without widening the slice
  - Acceptance: the repo-local sequencing note and any touched docs describe the slice as internal retained-worker continue plus minimal worker-event bootstrap only; no wording implies fuller policy hardening or broader verb coverage has already landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`llm-last-mile/NOTE-33-family-1-ordering-after-dispatch-bootstrap.md`](./NOTE-33-family-1-ordering-after-dispatch-bootstrap.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) only if new internal audit rows are introduced
    - [`llm-last-mile/SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md`](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)
    - [`llm-last-mile/PLAN-33.md`](./PLAN-33.md)
    - [`llm-last-mile/TASKS-33.md`](./TASKS-33.md)

- [x] Task 4.3: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell/world-service suites, and the full workspace tests are green; no public CLI regression or unintended family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no planned source edits; this is the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the retained-worker continue surface is safely bounded,
2. the planning/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh spec-driven-development restart.

Proceed directly to the next packet only when:

1. the current packet's verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the landed contract,
4. the slice has not widened into approval/fork classes, policy hardening, or family-2 architecture work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves slice `32` did not actually close retained worker allocation and this slice cannot stand on a real retained-worker bootstrap,
2. the existing retained member-turn seam cannot honestly carry the internal continue contract,
3. minimal typed worker-event classification turns out to require approval/fork/control classes immediately,
4. verification proves the planned order is wrong.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or tasks was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions the next packet must keep.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.
