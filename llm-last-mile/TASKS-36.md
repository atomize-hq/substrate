# TASKS-36: Internal Retained World Worker Stop Closeout

Source spec: [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)  
Source plan: [PLAN-36.md](./PLAN-36.md)  
Source validation note: [NOTE-35-family-1-ordering-after-inspect-snapshot.md](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packet 4 complete on `2026-06-01`; docs truth is aligned and the final validation wall is green
Landed posture note: the stop action, policy allowlisting, exact retained-worker stop routing, and durable closeout behavior are landed repo-wide, but retained-worker stop routing is supported only on Linux in v1 and fails closed elsewhere with `unsupported_platform_or_posture`.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the stop contract and steering-policy allowlist expansion.
- Packet 2 lands authoritative retained-worker stop target resolution and already-terminal denial.
- Packet 3 lands internal dispatch wiring and durable closeout behavior.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Stop Contract And Policy Allowlist Expansion

Session goal:

1. add `stop_world_worker` as a typed internal action,
2. keep stop retained-worker-only in v1,
3. allow steering policy to explicitly admit the action without weakening deny-by-default behavior.

### Tasks

- [x] Task 1.1: Add the stop action, retained-only validation, and typed stop payload/outcome scaffolding
  - Acceptance: `stop_world_worker` is a valid internal dispatch action; request validation requires retained mode and exact `target_participant_id`; the contract exposes a typed stop payload and stop outcome suitable for durable closeout routing.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.2: Widen steering-policy parsing so `stop_world_worker` can be explicitly allowlisted
  - Acceptance: the effective policy/config model accepts `stop_world_worker` as an allowed world-dispatch action while keeping deny-by-default defaults unchanged when the action is absent.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs) if diagnostics or validation lists need to stay aligned with the new action id
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs) if internal ingress must recognize the new action during validation
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) only if Packet 1 lands user-visible policy truth immediately

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `stop_world_worker` is a valid internal action,
2. it validates as retained-only with exact target identity,
3. steering-policy parsing can explicitly allow the action.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Authoritative Retained-Worker Stop Target Resolution

Session goal:

1. resolve exact retained-worker stop targets from stored runtime truth,
2. reject already-terminal retained workers,
3. keep stop target resolution precise and bounded.

### Tasks

- [x] Task 2.1: Add exact retained-worker stop target resolution in the state store
  - Acceptance: stop target resolution is same-session-only, same-world-binding-only, and authoritative-caller-only; it accepts exact retained workers and rejects targets that are already terminal.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [x] Task 2.2: Add stable terminal-state rejection for already-stopped or otherwise terminal workers
  - Acceptance: repeated stop against terminal retained workers fails closed with a stable, reviewable error path instead of silently succeeding or routing into closeout again.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. stop targets resolve by exact retained identity,
2. already-terminal retained workers fail closed,
3. the target-resolution path remains bounded to retained-worker stop semantics.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Internal Dispatch Wiring And Durable Closeout

Session goal:

1. route stop through the internal dispatch layer,
2. enforce steering policy before stop execution,
3. drive durable closeout through existing stop/runtime helpers.

### Tasks

- [x] Task 3.1: Add `stop_world_worker` handling to the internal dispatch path
  - Acceptance: the orchestrator dispatch layer evaluates steering policy, resolves the stop target, and returns a typed stop outcome for exact retained workers.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)

- [x] Task 3.2: Reuse existing stop-closeout helpers for durable retained-worker stop
  - Acceptance: allowed stop requests drive durable stopped-terminal truth through the existing closeout/runtime mechanisms rather than introducing a second stop model; stop does not widen into cancel, continue, or fork behavior.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. steering-policy enforcement happens before stop execution,
2. allowed stop requests drive durable retained-worker closeout,
3. stop is proven distinct from cancel, continue, and fork behavior.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the landed stop scope,
2. keep retained-only stop and all later verb deferrals explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: the repo-local docs describe stop as internal, retained-worker-only in v1, Linux-only for routed closeout delivery, and a durable closeout action distinct from cancel; no wording implies `cancel_world_work`, `fork_world_worker`, approval autonomy, or Family-2 execution have landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/NOTE-35-family-1-ordering-after-inspect-snapshot.md`](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)
    - [`llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md`](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
    - [`llm-last-mile/PLAN-36.md`](./PLAN-36.md)
    - [`llm-last-mile/TASKS-36.md`](./TASKS-36.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and the full workspace tests are green; no public CLI regression or unintended Family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no planned source edits beyond rustfmt-only cleanup required to satisfy the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the stop surface is safely bounded,
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
4. the slice has not widened into `cancel_world_work`, `fork_world_worker`, approval/fork autonomy, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves retained-worker stop cannot be honest without broader active-work cancellation identity,
2. the existing stop-closeout truth is insufficient and the slice would require a second stop transport or second lifecycle model,
3. stop semantics unexpectedly require fork or approval policy decisions immediately,
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
