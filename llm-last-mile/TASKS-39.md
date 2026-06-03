# TASKS-39: Internal Retained Worker Approval And Fork Obligation Bootstrap

Source spec: [SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)  
Source plan: [PLAN-39.md](./PLAN-39.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packet 4 complete on `2026-06-03`; docs truth is aligned and the final validation wall is green
Landed posture note: retained-worker `approval_request`, `fork_request`, and `fork_recommendation` acceptance, deny-by-default worker-event policy gating, and durable local obligation projection are landed repo-wide, but typed host approval/control responses, active-ephemeral exact task-identity widening, child auto-allocation, and Family-2 router/attach execution remain deferred.
Validation note: final validation did not expose any in-scope stabilization follow-up, and no broader runtime feature work or deferred later surfaces were reopened.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the accepted worker event classes and minimal policy surface.
- Packet 2 lands durable obligation mapping and projection truth.
- Packet 3 lands live `continue_world_worker` persistence and regression coverage.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Worker Event Contract And Policy Surface

Session goal:

1. accept `approval_request`, `fork_request`, and `fork_recommendation` as live retained-worker event classes,
2. add the minimum deny-by-default policy keys needed for those classes,
3. keep all remaining deferred worker labels fail closed.

### Tasks

- [x] Task 1.1: Widen the retained-worker event contract for the first approval/fork worker event subset
  - Acceptance: `approval_request`, `fork_request`, and `fork_recommendation` are valid retained-worker event classes under exact identity and exact boundary truth; `approval_response`, `fork_command`, `control_directive`, `control_ack`, and generic `attention_required` remain deferred and fail closed.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.2: Add minimal deny-by-default policy parsing and denial coverage for approval/fork worker-event autonomy
  - Acceptance: policy/config truth can explicitly allow or deny `approval_request`, `fork_request`, and `fork_recommendation` worker events; denied events fail with stable explanation-ready errors; existing `agents.world_dispatch` action/mode/backend/boundary gates remain intact.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the newly accepted worker event subset is frozen,
2. the new policy keys are deny-by-default,
3. remaining deferred labels still fail closed,
4. policy denials are explanation-ready and do not weaken the existing `agents.world_dispatch` steering floor.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Durable Obligation Mapping And Projection Truth

Session goal:

1. map the new worker event classes into canonical local obligation kinds,
2. keep attention/review/attach semantics durable and explicit,
3. preserve compatibility with the current local obligation-ledger and attach-state naming.

### Tasks

- [x] Task 2.1: Freeze obligation-kind mapping and durable record semantics for accepted approval/fork worker events
  - Acceptance: `approval_request` persists `ApprovalRequired`, `fork_request` persists `ForkRequest`, and `fork_recommendation` persists `ForkRecommendation`; summaries, payloads, source identity, target backend, and world binding remain reviewable and exact.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [x] Task 2.2: Keep local attach-projection compatibility explicit for the new obligation producers
  - Acceptance: `ApprovalRequired` and `ForkRequest` preserve current local auto-attach eligibility semantics; `ForkRecommendation` remains non-attach-eligible by default; attach completion still does not imply obligation resolution.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs) if session-scoped projection helpers require updates

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. every newly accepted worker event class has one canonical local obligation mapping,
2. local attention/review/attach semantics stay explicit and durable,
3. router/attach compatibility is preserved without redesigning router execution.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Continue Routing And Durable Persistence

Session goal:

1. persist accepted approval/fork worker events from the live `continue_world_worker` path,
2. keep worker requests exact-identity and host-mediated,
3. prove that worker `fork_request` does not allocate a child worker on its own.

### Tasks

- [x] Task 3.1: Wire accepted worker approval/fork events through live `continue_world_worker` classification and persistence
  - Acceptance: exact retained-worker `continue_world_worker` events can persist `ApprovalRequired`, `ForkRequest`, and `ForkRecommendation` obligations when policy allows them; identity drift, boundary drift, or denied policy states fail closed before persistence.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 3.2: Prove accepted `fork_request` stays distinct from executed `fork_world_worker`
  - Acceptance: accepted `fork_request` creates durable obligation state only; no child worker is allocated until a later explicit host-issued `fork_world_worker` action; public status/control regressions stay green.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. live continue handling can persist the new worker-event subset,
2. denied and drifted paths fail closed before persistence,
3. accepted `fork_request` is proven distinct from executed child allocation.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `39` scope,
2. keep all deferred follow-on widening explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `39` as retained-worker approval/fork obligation bootstrap, not router execution or auto-fork; wording keeps typed host approval/control responses, active-ephemeral identity widening, and Family-2 router execution explicitly deferred.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md`](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)
    - [`llm-last-mile/PLAN-39.md`](./PLAN-39.md)
    - [`llm-last-mile/TASKS-39.md`](./TASKS-39.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no unintended widening into child auto-allocation, public control-surface changes, or Family-2 execution appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no additional in-scope stabilization follow-up was required during final validation on this branch

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `39` surface is safely bounded,
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
4. the slice has not widened into host-response typing, auto-fork, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest approval/fork worker-event acceptance requires a new public or toolbox response surface first,
2. obligation persistence cannot stay on the current local ledger/projection model without a broader artifact redesign,
3. accepted `fork_request` cannot stay distinct from child allocation without a larger control-plane change,
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
