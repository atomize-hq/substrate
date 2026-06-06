# PLAN-47: Internal Active-Ephemeral Task Identity And Inspect/Cancel Widening

Source spec: [SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: post-Slice-46 identity-model widening slice  
Status: Packets `1`-`4` are complete and green on the current tree as of `2026-06-06`

## Objective

Land the active-ephemeral identity-model slice by surfacing exact runtime-owned `task_run_id` truth for in-flight `run_world_task` execution and widening `inspect_world_worker` / `cancel_world_work` so they may target one exact active ephemeral task, without widening into retained continuation, public control UX, stop/fork redesign, durable obligation work, or Family-2 router execution.

This slice is complete only when all of the following are true:

1. active ephemeral `run_world_task` work has exact runtime-owned `task_run_id` truth while in flight,
2. `inspect_world_worker` and `cancel_world_work` accept `mode=ephemeral` only with that exact task identity,
3. active-ephemeral inspect returns authoritative non-mutating snapshot truth,
4. active-ephemeral cancel interrupts one exact active task and returns truthful closeout distinct from retained cancel and stop,
5. terminal one-shot outcomes remain terminal and non-durable,
6. retained inspect/cancel behavior remains exact and bounded,
7. public caller surfaces, worker autonomy widening, and Family-2 work remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md) has been reviewed and is the source of truth for scope before implementation planning advances.

Packet status on the current tree:

1. Packet `1` exact `task_run_id` contract and dual-target validation work is landed.
2. Packet `2` authoritative active-task tracking and inspect snapshot truth is landed.
3. Packet `3` routed active-ephemeral inspect/cancel behavior is landed.
4. Packet `4` doc alignment and the validation wall are landed and green.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - owns the typed `run_world_task`, `inspect_world_worker`, and `cancel_world_work` contract surface
   - must freeze exact `task_run_id` semantics before routing work begins
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - already owns ephemeral launch, active execute-stream handling, retained inspect, and retained cancel
   - is the natural place to surface active-task identity and route dual-target inspect/cancel behavior
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - may be reused only if exact active-task resolution cannot remain transient and shell-local
   - must not be widened casually into a retained-like active-task ledger
4. `crates/transport-api-types/src/lib.rs` and `crates/world-service/src/service.rs`
   - only become implementation-bearing if exact active-task identity cannot be surfaced cleanly from the existing execute-stream / cancel seam
5. `docs/CONFIGURATION.md`
   - must stay honest if the action/mode validity matrix for inspect/cancel becomes broader runtime truth

## Plan Summary

This section records the planning rationale that was true when Slice `47` opened. On the current tree, Packets `1`-`4` are landed and the validation wall is green; the summary below explains why the packet order was chosen.

After Slice `46`, the retained-worker control plane was no longer the narrowest remaining seam. The repo already had retained inspect, retained cancel, retained stop, retained fork, retained continue, host responses, and the retained control/fork/progress message loop.

What remained at slice start was the identity gap that kept active-ephemeral inspect/cancel out of scope:

1. the dispatch contract still rejected `inspect_world_worker` and `cancel_world_work` in `mode=ephemeral`,
2. `RunWorldTaskOutcomeV1` still exposed no typed `task_run_id`,
3. the runtime had active execute/cancel truth under the hood but did not yet surface it as exact control-plane identity.

That made Slice `47` an identity-model slice first, not a router slice:

1. exact task identity must be frozen before active inspect/cancel can be honest,
2. active-task resolution must be authoritative before dual-target routing can be safe,
3. cancel/inspect wiring should then consume that frozen identity rather than inventing it inline.

The narrowest honest implementation order was therefore:

1. freeze exact `task_run_id` contract and dual-target validity rules first,
2. add authoritative active-task tracking and snapshot truth second,
3. wire active-ephemeral inspect/cancel over that exact identity third,
4. finish with docs and the validation wall.

That implementation order is now complete through Packet `4`, and no broader Slice `47` scope widening was required to close the slice.

## Locked Decisions

### What changes

1. surface exact runtime-owned `task_run_id` truth for active `run_world_task` execution,
2. widen `inspect_world_worker` and `cancel_world_work` so `mode=ephemeral` is valid only with exact `task_run_id`,
3. add authoritative active-ephemeral task resolution and snapshot truth,
4. route exact active-task cancel behavior distinctly from retained cancel and stop,
5. preserve retained inspect/cancel behavior unchanged aside from the new dual-target routing split.

### What does not change

1. no new public CLI or toolbox control surface,
2. no fuzzy active-task selection,
3. no future continuation contract for ephemeral work,
4. no widening into `stop_world_worker`, retained `fork_world_worker`, or worker autonomy redesign,
5. no durable obligation or inbox projection for active-ephemeral task identity,
6. no Family-2 router/attach execution redesign,
7. no broad transport rewrite unless exact task identity cannot be surfaced otherwise.

## Implementation Order

### Packet 1: Exact Task Identity Contract And Dual-Target Validity

Goal:

1. freeze typed `task_run_id` runtime truth for active `run_world_task`,
2. widen `inspect_world_worker` and `cancel_world_work` validation so `mode=ephemeral` is accepted only with exact task identity,
3. keep retained exact-target semantics intact.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. targeted shell contract tests

Why first:

1. active-task resolution and routing need one frozen identity shape,
2. the repo must stop treating active-ephemeral inspect/cancel as a policy question when the real gap is contract identity,
3. later packets should consume one stable `task_run_id` contract rather than deciding identity ad hoc.

Verification checkpoint:

1. active `run_world_task` runtime truth can surface typed `task_run_id`,
2. `inspect_world_worker` and `cancel_world_work` accept `mode=ephemeral` only with exact task identity,
3. retained inspect/cancel still require exact retained `target_participant_id`.

### Packet 2: Authoritative Active-Task Tracking And Snapshot Truth

Goal:

1. add authoritative active-task tracking for in-flight ephemeral work,
2. resolve exact active-task identity only within the authoritative session/backend/world binding,
3. project a typed active-ephemeral inspect snapshot without mutating lifecycle state.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs` only if a narrow helper/registry is required
3. targeted shell/state resolution tests

Why second:

1. inspect and cancel are both consumers of exact active-task truth,
2. terminal teardown and race handling need one authoritative source of "still active" versus "already terminal",
3. keeping this layer explicit prevents inspect/cancel from quietly reopening one-shot lifecycle semantics.

Verification checkpoint:

1. active tasks resolve only inside the authoritative orchestration session and world binding,
2. inspect snapshots are truthful and non-mutating,
3. terminal or unknown task identities fail closed with stable errors.

### Packet 3: Dispatch Wiring For Active-Ephemeral Inspect And Cancel

Goal:

1. route active-ephemeral inspect through the internal dispatch layer,
2. route active-ephemeral cancel through the exact active-task seam,
3. keep cancel closeout truthful and distinct from retained cancel/stop behavior.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs` only if an explicit closeout helper is needed
3. `crates/transport-api-types/src/lib.rs` and `crates/world-service/src/service.rs` only if exact task identity cannot stay entirely shell-side
4. targeted shell integration tests

Why third:

1. routed behavior should consume a frozen contract and a trusted active-task resolver,
2. this is where exact-task cancel races and explanation-ready terminal behavior become real,
3. the slice should prove dual-target inspect/cancel without reopening broader router or retained lifecycle work.

Verification checkpoint:

1. allowed active-ephemeral inspect returns an authoritative snapshot for one exact task,
2. allowed active-ephemeral cancel interrupts one exact active task and returns truthful closeout,
3. retained inspect/cancel behavior remains exact and non-regressed.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `47` scope,
2. keep one-shot terminal semantics and deferred later work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md`
3. `llm-last-mile/PLAN-47.md`
4. `llm-last-mile/TASKS-47.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `47` as exact active-ephemeral task identity plus dual-target inspect/cancel widening,
2. docs do not imply future continuation, retained promotion, public control UX, or Family-2 router execution,
3. docs keep stop/fork/worker autonomy widening explicitly out of scope.

Verification checkpoint:

1. docs and config truth remain honest,
2. validation is green,
3. the next follow-on slice can stay separate from Slice `47` rather than reopening task identity.

## Risks And Mitigations

1. Risk: `task_run_id` becomes a fuzzy or unstable alias rather than exact runtime-owned identity.
   Mitigation: bind it to existing runtime-owned execution truth and fail closed after terminal teardown.
2. Risk: active-task tracking quietly becomes a durable retained-like registry.
   Mitigation: keep the active-task contract transient, exact, and scoped only to in-flight ephemeral work.
3. Risk: cancel and inspect race terminal completion and return misleading results.
   Mitigation: make "active versus already terminal" an explicit authoritative check with explanation-ready error paths.
4. Risk: implementation pressure widens the slice into public caller syntax, stop, or retained fork semantics.
   Mitigation: freeze internal-only, dual-target inspect/cancel scope and reject other lifecycle changes in this slice.
5. Risk: the slice turns into a transport rewrite.
   Mitigation: first exhaust the existing execute-stream and cancel seam; widen transport only if exact task identity cannot be surfaced otherwise.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because active-task resolution depends on one frozen `task_run_id` contract.
2. Packet 2 before Packet 3 because routed inspect/cancel behavior should consume one authoritative active-task resolver.
3. Packet 3 before Packet 4 because docs and validation should reflect final behavior rather than provisional identity assumptions.

### Can be parallelized later

1. any richer progress-causation or typed host-message envelope work only if Slice `47` proves the current narrow retained/ephemeral split insufficient,
2. broader worker autonomy widening only if the live repo still needs it after exact active-ephemeral identity is frozen,
3. Family-2 router/attach execution only as its own downstream track.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test -p transport-api-types -- --nocapture
cargo test -p world-service -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `47`

If Slice `47` lands cleanly as exact active-ephemeral identity plus dual-target inspect/cancel widening first, the next follow-on work should remain:

1. any remaining worker-autonomy or richer message-envelope widening only if the live repo still needs it,
2. any further active-ephemeral lifecycle broadening only if exact task identity exposes a real additional gap,
3. Family-2 router/attach execution only as its own downstream implementation track.
