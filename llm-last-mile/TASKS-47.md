# TASKS-47: Internal Active-Ephemeral Task Identity And Inspect/Cancel Widening

Source spec: [SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md)  
Source plan: [PLAN-47.md](./PLAN-47.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packets `1`-`4` are complete and green on the current tree as of `2026-06-06`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `47` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 froze exact active-task identity and the dual-target contract surface on the live tree.
- Packet 2 landed authoritative active-task tracking and inspect snapshot truth on the live tree.
- Packet 3 landed routed active-ephemeral inspect/cancel over that exact identity on the live tree.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Exact Task Identity Contract And Dual-Target Surface

Session goal:

1. surface exact runtime-owned `task_run_id` truth for active `run_world_task`,
2. widen `inspect_world_worker` and `cancel_world_work` so `mode=ephemeral` is valid only with exact task identity,
3. preserve retained exact-target behavior unchanged.

### Tasks

- [x] Task 1.1: Add typed active-task identity to the dispatch contract
  - Acceptance: active `run_world_task` runtime truth can surface typed `task_run_id`, and the internal outcome/contract surface has one exact field for active-ephemeral identity rather than reusing prompt text, request aliases, or fuzzy selectors.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.2: Widen inspect/cancel validation to admit `mode=ephemeral` only with exact task identity
  - Acceptance: `inspect_world_worker` and `cancel_world_work` accept `mode=ephemeral` only when `task_run_id` is present and exact; retained requests still require exact `target_participant_id`; malformed or mixed-target requests fail closed before resolution.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. active `run_world_task` runtime truth has typed `task_run_id`,
2. active-ephemeral inspect/cancel validate only with exact task identity,
3. retained inspect/cancel exact-target behavior remains intact,
4. malformed or mixed-target requests fail closed before resolution.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Authoritative Active-Task Tracking And Inspect Snapshot Truth

Session goal:

1. track exact active ephemeral work authoritatively while it is in flight,
2. expose a typed inspect snapshot for one active task,
3. tear down routability cleanly once the task reaches terminal outcome.

### Tasks

- [x] Task 2.1: Add authoritative active-task tracking for in-flight ephemeral work
  - Acceptance: active ephemeral work is tracked by exact `task_run_id`, session, backend, and world binding while in flight, and terminal teardown removes the task from routable active state.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) if a narrow helper or registry is required

- [x] Task 2.2: Add authoritative active-ephemeral inspect snapshot projection
  - Acceptance: allowed `inspect_world_worker` requests with `mode=ephemeral` return a typed active snapshot for one exact active task, remain non-mutating, and fail closed for terminal or unknown task ids.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. active tasks resolve only by exact `task_run_id`,
2. inspect snapshots are authoritative and non-mutating,
3. terminal or unknown task ids fail closed with stable errors,
4. active-task teardown removes routability after terminal completion.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Routed Active-Ephemeral Inspect And Cancel

Session goal:

1. route active-ephemeral inspect through the internal dispatch layer,
2. route active-ephemeral cancel through the exact active-task seam,
3. keep cancel closeout truthful and separate from retained cancel/stop behavior.

### Tasks

- [x] Task 3.1: Add routed active-ephemeral inspect behavior
  - Acceptance: the orchestrator dispatch layer evaluates exact active-task identity and returns a typed authoritative inspect outcome for one active ephemeral task without invoking retained-worker snapshot logic.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [x] Task 3.2: Add exact active-ephemeral cancel routing and closeout
  - Acceptance: allowed `cancel_world_work` requests with `mode=ephemeral` interrupt one exact active task, return truthful closeout distinct from retained-worker cancel and stop, and fail closed on late/terminal races without reopening one-shot lifecycle semantics.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p transport-api-types -- --nocapture`
    - `cargo test -p world-service -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs) if an explicit closeout helper is needed
    - [`crates/transport-api-types/src/lib.rs`](../crates/transport-api-types/src/lib.rs) if the exact task identity carrier changes there
    - [`crates/world-service/src/service.rs`](../crates/world-service/src/service.rs) if matching service-side propagation is required

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. active-ephemeral inspect returns authoritative snapshot truth for one exact task,
2. active-ephemeral cancel interrupts one exact active task and returns truthful closeout,
3. retained inspect/cancel behavior remains exact and non-regressed,
4. terminal-race behavior is explanation-ready and fail-closed.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `47` scope,
2. keep one-shot terminal semantics and deferred later work explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning and config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `47` as exact active-ephemeral `task_run_id` plus dual-target inspect/cancel widening; no wording implies retained promotion, public control UX, stop/fork redesign, or Family-2 router execution have landed.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md`](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md)
    - [`llm-last-mile/PLAN-47.md`](./PLAN-47.md)
    - [`llm-last-mile/TASKS-47.md`](./TASKS-47.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, transport/world-service tests if touched, and full workspace tests are green against the bounded Slice `47` file set; this task validates the implementation and does not serve as an open-ended cleanup bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test -p transport-api-types -- --nocapture`
    - `cargo test -p world-service -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if any validation command fails because Slice `47` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `47` surface is safely bounded,
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
4. the slice has not widened into retained continuation, stop/fork redesign, public caller-surface changes, or Family-2 router execution.

Reopen spec, plan, or tasks only if one of these is true:

1. exact active-task identity cannot be surfaced honestly without a broader transport redesign than this slice allows,
2. active-ephemeral inspect/cancel cannot stay transient and exact without forcing a larger retained/durable model change,
3. terminal-race behavior proves the planned identity/routing order is wrong,
4. verification proves dual-target inspect/cancel needs a broader public or router-facing surface first.

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
