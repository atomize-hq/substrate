# TASKS-35: Internal Retained World Worker Inspect Snapshot

Source spec: [SPEC-35-internal-retained-world-worker-inspect-snapshot.md](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)  
Source plan: [PLAN-35.md](./PLAN-35.md)  
Source validation note: [REMAINING-family-1-scope-2026-05-31-post-slice-34.md](./REMAINING-family-1-scope-2026-05-31-post-slice-34.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: proposed on `2026-05-31`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the inspect contract and steering-policy allowlist expansion.
- Packet 2 lands authoritative retained-worker inspect target resolution and snapshot projection.
- Packet 3 lands internal dispatch wiring and regression coverage.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Inspect Contract And Policy Allowlist Expansion

Session goal:

1. add `inspect_world_worker` as a typed internal action,
2. keep inspect retained-worker-only in v1,
3. allow steering policy to explicitly admit the action without weakening deny-by-default behavior.

### Tasks

- [ ] Task 1.1: Add the inspect action, retained-only validation, and typed inspect payload/outcome scaffolding
  - Acceptance: `inspect_world_worker` is a valid internal dispatch action; request validation requires retained mode and exact `target_participant_id`; the contract exposes a typed inspect payload and inspect outcome suitable for authoritative snapshot projection.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Widen steering-policy parsing so `inspect_world_worker` can be explicitly allowlisted
  - Acceptance: the effective policy/config model accepts `inspect_world_worker` as an allowed world-dispatch action while keeping deny-by-default defaults unchanged when the action is absent.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) only if Packet 1 lands user-visible policy truth immediately

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `inspect_world_worker` is a valid internal action,
2. it validates as retained-only with exact target identity,
3. steering-policy parsing can explicitly allow the action.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Authoritative Retained-Worker Snapshot Resolution

Session goal:

1. resolve exact retained-worker inspect targets from stored runtime truth,
2. return a typed snapshot for live and non-live retained workers,
3. keep inspect read-only.

### Tasks

- [ ] Task 2.1: Add exact retained-worker inspect target resolution in the state store
  - Acceptance: inspect target resolution is same-session-only, same-world-binding-only, and authoritative-caller-only; it accepts exact retained workers without requiring them to remain continue-routable.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Project a typed retained-worker inspect snapshot from authoritative stored truth
  - Acceptance: the inspect projection returns exact identity plus lifecycle/posture/status metadata for live, detached/attention, invalidated, and terminal retained workers without mutating any lifecycle state.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs) only if a narrow reusable snapshot helper is needed

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. inspect targets resolve by exact retained identity,
2. live and non-live retained workers can both yield truthful snapshots,
3. the snapshot path is read-only.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Internal Dispatch Wiring And Regression Coverage

Session goal:

1. route inspect through the internal dispatch layer,
2. enforce steering policy before serving inspect results,
3. prove inspect has no execution side effects.

### Tasks

- [ ] Task 3.1: Add `inspect_world_worker` handling to the internal dispatch path
  - Acceptance: the orchestrator dispatch layer evaluates steering policy, resolves the inspect target, and returns a typed inspect outcome without invoking a world-side execution transport.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 3.2: Add regression tests for inspect denial and non-mutating behavior
  - Acceptance: tests prove that disallowed inspect requests fail closed, allowed inspect requests return authoritative snapshots, and inspect does not continue, cancel, stop, or fork workers.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. steering-policy enforcement happens before inspect results are served,
2. allowed inspect requests return typed retained-worker snapshots,
3. inspect is proven non-mutating.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the landed inspect scope,
2. keep retained-only inspect and all later verb deferrals explicit,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: the repo-local docs describe inspect as internal, retained-worker-only in v1, and store-backed; no wording implies active-ephemeral inspect, cancel, stop, fork, approval autonomy, or Family-2 execution have landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/REMAINING-family-1-scope-2026-05-31-post-slice-34.md`](./REMAINING-family-1-scope-2026-05-31-post-slice-34.md)
    - [`llm-last-mile/SPEC-35-internal-retained-world-worker-inspect-snapshot.md`](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)
    - [`llm-last-mile/PLAN-35.md`](./PLAN-35.md)
    - [`llm-last-mile/TASKS-35.md`](./TASKS-35.md)

- [ ] Task 4.2: Run the final validation wall
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
    - no planned source edits; this is the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the inspect surface is safely bounded,
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
4. the slice has not widened into active-ephemeral inspect, cancel, stop, fork, approval/fork autonomy, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves retained-only inspect cannot be honest without active-ephemeral task identity,
2. the existing authoritative snapshot truth is insufficient and the slice would require a live world-service inspect RPC,
3. inspect semantics unexpectedly require lifecycle mutation or broader policy redesign,
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
