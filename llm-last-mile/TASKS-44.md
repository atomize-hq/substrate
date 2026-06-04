# TASKS-44: Internal Retained Worker Control Ack Bootstrap

Source spec: [SPEC-44-internal-retained-worker-control-ack-bootstrap.md](./SPEC-44-internal-retained-worker-control-ack-bootstrap.md)  
Source plan: [PLAN-44.md](./PLAN-44.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: drafted on `2026-06-04`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 admits `control_ack` into the retained-worker event contract and freezes the exact-causation guard.
- Packet 2 lands live outcome semantics for acknowledged control directives and proves `control_ack` stays non-durable.
- Packet 3 lands fail-closed regression proof for invalid acknowledgement contexts.
- Packet 4 aligns docs/config truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Event Contract Admission And Exact-Causation Guard

Session goal:

1. admit `control_ack` as an in-scope retained-worker event class,
2. keep it non-attention-driving by default,
3. accept it only when the originating request payload is the typed host `control_directive`.

### Tasks

- [ ] Task 1.1: Admit `control_ack` in the retained-worker event contract
  - Acceptance: `ContinueWorldWorkerEventClassV1` accepts `control_ack`; `control_ack` is removed from deferred-worker rejection coverage; `control_ack` is non-attention-driving by default.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Enforce typed host `control_directive` causation before accepting `control_ack`
  - Acceptance: `control_ack` fails closed for generic prompt-based continue and typed host `approval_response` or `clarification_response`; exact participant/backend/session/world binding rules remain intact.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted tests adjacent to the touched implementation

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `control_ack` is an admitted retained-worker event class,
2. `control_ack` is non-attention-driving by default,
3. `control_ack` is accepted only for typed host `control_directive` turns,
4. other deferred worker labels still remain deferred.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Live Outcome Semantics And No-Durable-Side-Effect Behaviour

Session goal:

1. surface accepted `control_ack` through the live `continue_world_worker` outcome,
2. keep acknowledgement semantics honest about receipt versus completion,
3. keep `control_ack` outside the durable obligation path.

### Tasks

- [ ] Task 2.1: Surface acknowledged control directives truthfully on the live outcome path
  - Acceptance: a typed host `control_directive` may return `worker_event=control_ack` on the existing live `continue_world_worker` path; the summary stays explicit about acknowledgement and does not imply task completion.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [ ] Task 2.2: Prove `control_ack` leaves no durable obligation side effects
  - Acceptance: accepted `control_ack` is not persisted as an obligation, does not set attention-required posture by default, and does not widen inbox/review semantics.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. acknowledged typed host `control_directive` delivery may surface `control_ack`,
2. summaries remain honest about acknowledgement versus completion,
3. `control_ack` produces no durable obligation side effects,
4. prior approval/clarification/control-directive behaviour remains green.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Fail-Closed Regression Proof

Session goal:

1. prove invalid `control_ack` contexts fail closed,
2. preserve exact identity and world-binding safety,
3. preserve existing unsupported-worker-event cancellation behaviour.

### Tasks

- [ ] Task 3.1: Prove invalid acknowledgement contexts fail closed
  - Acceptance: `control_ack` is rejected with stable explanation-ready errors when it appears on non-control-directive turns or under identity/session/world drift.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [ ] Task 3.2: Preserve deferred-worker and cancel-on-unsupported behaviour outside Slice 44 scope
  - Acceptance: `fork_command`, host `control_directive`, and `attention_required` remain deferred worker labels; unsupported labels still trigger the same fail-closed stream cancellation behaviour.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. invalid `control_ack` contexts fail closed,
2. exact participant/backend/session/world drift still fails closed,
3. unsupported worker labels outside Slice `44` still cancel or reject correctly,
4. the slice has not widened into `fork_command`, optional `progress_ack`, or durable control review.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `44` scope,
2. keep `fork_command`, optional `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning and config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `44` as immediate worker `control_ack` over the existing `continue_world_worker` seam, not durable control review, generalized control transport, or `fork_command`.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-44-internal-retained-worker-control-ack-bootstrap.md`](./SPEC-44-internal-retained-worker-control-ack-bootstrap.md)
    - [`llm-last-mile/PLAN-44.md`](./PLAN-44.md)
    - [`llm-last-mile/TASKS-44.md`](./TASKS-44.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests are green; no unintended widening into `fork_command`, optional `progress_ack`, transport-schema redesign, active-ephemeral identity, or Family-2 execution appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - final validation may require bounded follow-up fixes inside already-touched Slice `44` runtime surfaces, but must not reopen `fork_command`, optional `progress_ack`, active-ephemeral identity, or Family-2 router execution

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `44` surface is safely bounded,
2. config/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh `spec-driven-development` restart.

Proceed directly to the next packet only when:

1. the current packet's verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into `fork_command`, optional `progress_ack`, transport-schema redesign, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest `control_ack` causation requires a broader host/worker transport envelope first,
2. the acknowledgement semantics prove they must be durable or attention-driving after all,
3. accepting `control_ack` requires a separate dedicated policy key rather than reuse of `control_directives_allowed`,
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
