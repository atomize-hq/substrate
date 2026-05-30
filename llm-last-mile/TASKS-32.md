# TASKS-32: Internal Host-Orchestrator World Dispatch Bootstrap

Source spec: [SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md)  
Source plan: [PLAN-32.md](./PLAN-32.md)  
Source validation note: [REMAINING-family-1-scope-2026-05-30.md](./REMAINING-family-1-scope-2026-05-30.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: draft for review on `2026-05-30`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 adds the internal caller surface and shared dispatch contract.
- Packet 2 lands `run_world_task`.
- Packet 3 lands `spawn_world_worker`.
- Packet 4 locks the minimum gating, audit, docs, and final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Internal Caller Surface And Shared Dispatch Contract

Session goal:

1. create the first orchestrator-only control-plane entry seam,
2. define shared typed request/outcome scaffolding,
3. freeze exact required identity and action/mode validation.

### Tasks

- [ ] Task 1.1: Add a minimal orchestrator-only internal dispatch caller surface
  - Acceptance: an internal caller surface exists for the host orchestrator only; it is not exposed as a new public human CLI; it resolves the authoritative orchestration session and caller participant before any world execution attempt.
  - Verify:
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs) only if existing toolbox posture plumbing must expose the internal surface safely

- [ ] Task 1.2: Add typed bootstrap request/outcome contract and fail-closed validation
  - Acceptance: the slice has one shared contract for `run_world_task` and `spawn_world_worker`; invalid action/mode combinations fail closed; missing session/caller/backend/world-binding fields fail with stable, explanation-ready errors.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the internal caller surface exists and is orchestrator-only,
2. shared request/outcome validation exists for both in-scope verbs,
3. no public CLI behavior has changed.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: `run_world_task` Ephemeral Bootstrap

Session goal:

1. route one-shot world work through the new caller seam,
2. return a typed terminal outcome,
3. prove the bootstrap path without retained lifecycle complexity.

### Tasks

- [ ] Task 2.1: Wire `run_world_task` into the existing world execution seam
  - Acceptance: the internal dispatch surface can launch exact backend world work as one-shot `ephemeral` work using the existing runtime path; no alternate execution plane is introduced.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

- [ ] Task 2.2: Return a typed terminal outcome and keep the path non-retained
  - Acceptance: `run_world_task` returns an explanation-ready terminal outcome such as `completed`, `failed`, `cancelled`, or `needs_retained_followup`; it does not silently create retained worker state or family-2 durable obligations.
  - Verify:
    - `cargo test -p shell control -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - targeted shell integration tests

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. `run_world_task` works end-to-end through the current world execution seam,
2. the outcome is typed and terminal,
3. no retained-worker or obligation-ledger behavior has been implied accidentally.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: `spawn_world_worker` Retained Bootstrap

Session goal:

1. route retained worker allocation through the same caller seam,
2. return authoritative child identity and launch receipt,
3. stop short of ongoing steering support.

### Tasks

- [ ] Task 3.1: Wire `spawn_world_worker` through exact retained worker launch
  - Acceptance: the internal dispatch surface can allocate a retained world worker using the existing retained world-member runtime seam; exact backend and authoritative world-binding identity are preserved.
  - Verify:
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

- [ ] Task 3.2: Return authoritative retained worker receipt without widening steering
  - Acceptance: `spawn_world_worker` returns authoritative child identity and lineage/receipt fields; the slice does not claim `continue_world_worker`, messaging, or worker-to-host event support yet.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - targeted shell integration tests

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. retained worker allocation returns authoritative child identity,
2. exact backend identity still holds,
3. ongoing steering remains explicitly out of scope.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Minimal Gating, Audit, Docs, And Validation

Session goal:

1. lock the minimum safe boundary for the first family-1 slice,
2. align trace/docs with the actual internal surface,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Enforce orchestrator-only, same-session-only, and same-world-binding-only gating
  - Acceptance: unsupported callers, cross-session requests, cross-world-binding requests, and unsupported exact backend targets fail closed with stable deny buckets; no full policy-matrix rollout is required yet.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Expected files touched:
    - `crates/shell/src/execution/<new internal control-plane module>.rs`
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)

- [ ] Task 4.2: Add audit/docs coverage without widening the surface
  - Acceptance: any new trace rows or docs describe the caller surface as internal bootstrap-only; `docs/USAGE.md` and `docs/TRACE.md` remain honest about toolbox posture; no wording implies a broader execution plane than what actually landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/USAGE.md`](../docs/USAGE.md)
    - [`docs/TRACE.md`](../docs/TRACE.md)
    - [`llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md`](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md)
    - [`llm-last-mile/PLAN-32.md`](./PLAN-32.md)
    - [`llm-last-mile/TASKS-32.md`](./TASKS-32.md)

- [ ] Task 4.3: Run the final validation wall
  - Acceptance: formatting, clippy, the targeted shell/world-service suites, and the full workspace tests are green; no public CLI regression or unintended family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no planned source edits; this is the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the internal surface is safely bounded,
2. docs and trace wording are honest,
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
4. the slice has not widened into messaging, fork autonomy, or family-2 work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves the internal caller surface cannot be made real without a broader toolbox redesign,
2. retained-worker messaging turns out to be required for even the bootstrap slice,
3. the existing world execution seam cannot honestly support the two in-scope verbs,
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
