# TASKS-37: Internal Cancel World Work

Source spec: [SPEC-37-internal-cancel-world-work.md](./SPEC-37-internal-cancel-world-work.md)  
Source plan: [PLAN-37.md](./PLAN-37.md)  
Source validation note: [NOTE-36-family-1-ordering-after-stop-closeout.md](./NOTE-36-family-1-ordering-after-stop-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: drafted on `2026-06-01`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the cancel contract and steering-policy allowlist expansion.
- Packet 2 lands authoritative retained-worker cancel target resolution and explicit cancelled lifecycle truth.
- Packet 3 lands internal dispatch wiring and cancel-closeout behavior.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Cancel Contract And Policy Allowlist Expansion

Session goal:

1. add `cancel_world_work` as a typed internal action,
2. keep cancel retained-worker-only in Slice `37`,
3. allow steering policy to explicitly admit the action without weakening deny-by-default behavior.

### Tasks

- [ ] Task 1.1: Add the cancel action, retained-only validation, and typed cancel payload/outcome scaffolding
  - Acceptance: `cancel_world_work` is a valid internal dispatch action; request validation requires retained mode and exact `target_participant_id`; the contract exposes a typed cancel payload and cancel outcome suitable for retained-turn cancellation.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Widen steering-policy parsing so `cancel_world_work` can be explicitly allowlisted
  - Acceptance: the effective policy/config model accepts `cancel_world_work` as an allowed world-dispatch action while keeping deny-by-default defaults unchanged when the action is absent.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs) if diagnostics or validation lists need to stay aligned with the new action id
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs) if internal ingress must recognize the new action during validation
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) only if Packet 1 lands user-visible policy truth immediately

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `cancel_world_work` is a valid internal action,
2. it validates as retained-only with exact target identity,
3. steering-policy parsing can explicitly allow the action.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Retained Cancel Target Resolution And Lifecycle Truth

Session goal:

1. resolve exact retained-worker cancel targets from stored runtime truth,
2. reject non-live, idle, or already-terminal retained workers,
3. make cancelled terminal truth explicit and distinct from stopped truth.

### Tasks

- [ ] Task 2.1: Add exact retained-worker cancel target resolution in the state store
  - Acceptance: cancel target resolution is same-session-only, same-world-binding-only, and authoritative-caller-only; it accepts only exact retained workers that still have active cancelable work in flight.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Add explicit retained cancelled-state truth
  - Acceptance: retained participant/session state can surface cancelled terminal truth without reusing `stopped` or `failed`; repeated cancel against already-terminal workers fails closed with a stable, reviewable error path.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. cancel targets resolve by exact retained identity,
2. non-live or already-terminal retained workers fail closed,
3. explicit cancelled terminal truth exists and is distinct from stop semantics.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Internal Dispatch Wiring And Cancel Closeout

Session goal:

1. route cancel through the internal dispatch layer,
2. enforce steering policy before cancel execution,
3. interrupt active retained work in flight and persist typed cancel closeout.

### Tasks

- [ ] Task 3.1: Add `cancel_world_work` handling to the internal dispatch path
  - Acceptance: the orchestrator dispatch layer evaluates steering policy, resolves the cancel target, and returns a typed cancel outcome for exact retained workers with active work in flight.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)

- [ ] Task 3.2: Add the retained-turn cancel control seam and persist cancel closeout
  - Acceptance: allowed cancel requests interrupt active retained work in flight through a dedicated cancel-only internal transport, converge on explicit cancelled terminal truth, and do not widen into stop, inspect, continue, or fork behavior.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. steering-policy enforcement happens before cancel execution,
2. allowed cancel requests drive explicit cancelled terminal truth,
3. cancel is proven distinct from stop, inspect, continue, and fork behavior.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen cancel scope,
2. keep retained-only v1 cancel and all later verb deferrals explicit,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: repo-local docs describe cancel as internal, retained-worker-only in Slice `37`, Linux-routed in v1 if applicable, and distinct from stop; no wording implies active-ephemeral dual-target cancel, `fork_world_worker`, approval autonomy, or Family-2 execution have landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/NOTE-36-family-1-ordering-after-stop-closeout.md`](./NOTE-36-family-1-ordering-after-stop-closeout.md)
    - [`llm-last-mile/SPEC-37-internal-cancel-world-work.md`](./SPEC-37-internal-cancel-world-work.md)
    - [`llm-last-mile/PLAN-37.md`](./PLAN-37.md)
    - [`llm-last-mile/TASKS-37.md`](./TASKS-37.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no public CLI regression or unintended Family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no planned source edits beyond rustfmt-only cleanup required to satisfy the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the cancel surface is safely bounded,
2. config/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh spec-driven-development restart.

Proceed directly to the next packet only when:

1. the current packet’s verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into active-ephemeral dual-target cancel, `fork_world_worker`, approval/fork autonomy, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves retained-turn cancel cannot be honest without landing exact active-ephemeral task identity first,
2. cancel-closeout truth cannot be represented distinctly from stop without broader lifecycle redesign,
3. the transport/control seam needed for cancel would necessarily widen into other deferred verbs,
4. verification proves the planned order is wrong.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet’s verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or tasks was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions the next packet must keep.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.
