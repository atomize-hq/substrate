# Spec: Internal Active-Ephemeral Task Identity And Inspect/Cancel Widening

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-35-internal-retained-world-worker-inspect-snapshot.md](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)
- [PLAN-35.md](./PLAN-35.md)
- [TASKS-35.md](./TASKS-35.md)
- [SPEC-37-internal-cancel-world-work.md](./SPEC-37-internal-cancel-world-work.md)
- [PLAN-37.md](./PLAN-37.md)
- [TASKS-37.md](./TASKS-37.md)
- [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)
- [PLAN-46.md](./PLAN-46.md)
- [TASKS-46.md](./TASKS-46.md)
Related design stack:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: drafted on `2026-06-05`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `46` is fully landed on the current tree, so the retained-worker control, acknowledgement, fork-command, and progress-ack surfaces are no longer the next planning frontier.
2. The next honest Family-1 slice is exact active-ephemeral task identity plus dual-target `inspect_world_worker` / `cancel_world_work` widening, not Family-2 router execution and not a broader public caller-surface change.
3. Slice `47` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. The narrowest exact active-task identity should reuse runtime-owned in-flight execution truth and surface a typed `task_run_id` rather than invent fuzzy aliases, role-based lookup, or heuristic selection.
5. Active-ephemeral inspect/cancel should apply only while `run_world_task` is still active in flight. Terminal one-shot outcomes remain terminal and do not become future-routable retained state.
6. `continue_world_worker`, `stop_world_worker`, retained `fork_world_worker`, worker autonomy widening, durable obligation redesign, and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Objective

Build the first post-Slice-46 identity-model slice so the host orchestrator can obtain exact runtime-owned identity for active `run_world_task` work and use that identity to inspect or cancel one exact active ephemeral task through the existing internal dispatch family, without widening into retained continuation, public control UX, lifecycle unification, Family-2 router execution, or a second execution plane.

Primary runtime story:

1. the host issues `run_world_task`,
2. Substrate surfaces exact runtime-owned active-task identity as typed `task_run_id` runtime truth for that in-flight ephemeral dispatch,
3. while the task is still active, the host may issue `inspect_world_worker` or `cancel_world_work` with `mode=ephemeral` and that exact `task_run_id`,
4. Substrate validates exact caller, session, backend, world-binding, and active-task identity truth before serving inspect or cancel behavior,
5. `inspect_world_worker` returns a typed authoritative active-ephemeral snapshot without mutating lifecycle state,
6. `cancel_world_work` interrupts that one exact in-flight task and returns truthful cancel closeout distinct from retained-worker cancel and stop semantics,
7. terminal one-shot outcomes remain terminal, non-routable, and non-durable.

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. the live internal dispatch vocabulary already includes `run_world_task`, `inspect_world_worker`, and `cancel_world_work` on the same control-plane family,
2. the current request contract still rejects `inspect_world_worker` and `cancel_world_work` in `mode=ephemeral`, which means active-ephemeral widening is not yet runtime truth,
3. the live `RunWorldTaskOutcomeV1` still returns a terminal one-shot outcome with no typed `task_run_id`,
4. the live `run_world_task` path already has runtime-owned active execution truth under the hood:
   - exact `request_id`,
   - exact backend/world-binding routing,
   - streamed execute `span_id`,
   - existing `cancel_execute(span_id, sig)` transport for active interruption,
5. the authoritative state store currently models retained sessions, retained participants, and durable obligations, but it does not yet expose a first-class active-ephemeral task resolver or snapshot contract,
6. retained inspect and retained cancel are already landed as Linux-first routed behavior, so this slice is a widening of existing verbs rather than a new verb family.

That means the remaining gap is identity-model work first: the repo needs exact active-ephemeral task identity plus authoritative live-task resolution before dual-target inspect/cancel can be honest.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing internal dispatch/runtime truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing retained/runtime state surfaces that must stay coherent if reused:
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- Existing transport/cancel seam expected to be reused or widened only if exact task identity cannot be surfaced without it:
  - [`crates/transport-api-types/src/lib.rs`](../crates/transport-api-types/src/lib.rs)
  - [`crates/world-service/src/service.rs`](../crates/world-service/src/service.rs)
- Existing public/runtime behavior that must not regress:
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
  - [`crates/shell/tests/`](../crates/shell/tests)

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted validation floor:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test -p transport-api-types -- --nocapture
cargo test -p world-service -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - widen the typed contract so active-ephemeral runtime truth can expose `task_run_id`
  - admit `inspect_world_worker` / `cancel_world_work` in `mode=ephemeral` only when exact task identity is supplied
  - keep retained targeting and retained outcomes intact
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - surface exact active-task identity during `run_world_task`
  - resolve active-ephemeral inspect/cancel targets authoritatively
  - route inspect snapshot and cancel behavior without widening into retained semantics
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - only if implementation needs a narrow authoritative registry/helper for exact active-task resolution
  - do not widen the durable retained/obligation model unless exact task identity cannot stay transient
- `crates/shell/src/execution/agent_runtime/control.rs`
  - only if active-ephemeral cancel closeout needs a shared helper distinct from retained closeout
- `crates/transport-api-types/src/lib.rs`
  - only if the existing execute-stream and cancel carrier cannot surface exact active-task identity without a narrow typed addition
- `crates/world-service/src/service.rs`
  - only if the matching active-task identity carrier must be surfaced or normalized on the world-service side
- `docs/CONFIGURATION.md`
  - keep action/mode truth honest if Slice `47` changes the valid runtime matrix for `inspect_world_worker` or `cancel_world_work`
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, typed contracts, fail-closed routing, and truthful lifecycle summaries.

Preferred style:

```rust
if request.action == WorldDispatchActionV1::InspectWorldWorker
    && request.mode == WorldDispatchModeV1::Ephemeral
    && request.task_run_id.as_deref().is_none()
{
    anyhow::bail!(
        "missing_dispatch_field: inspect_world_worker requires task_run_id for mode ephemeral"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane and transport boundaries,
2. treat active-ephemeral identity as exact runtime-owned truth, never as a fuzzy lookup,
3. keep active-task inspect snapshot behavior non-mutating,
4. keep cancel semantics exact-task only and distinct from retained cancel/stop closeout,
5. remove active-task routability when the task reaches terminal one-shot outcome,
6. keep `continue`, retained worker control, fork autonomy, and Family-2 logic out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- shell regression suites
- transport/world-service contract tests if a narrow carrier change is required

Test levels for this slice:

1. unit tests for exact active-task identity contract:
   - `run_world_task` surfaces typed `task_run_id` runtime truth,
   - `inspect_world_worker` accepts `mode=ephemeral` only with exact `task_run_id`,
   - `cancel_world_work` accepts `mode=ephemeral` only with exact `task_run_id`,
   - retained exact-target behavior remains intact
2. unit tests for active-task resolution:
   - same-session only,
   - same-world-binding only,
   - exact backend only,
   - active-only and fail-closed after terminal teardown
3. integration tests for active-ephemeral inspect:
   - live task returns authoritative snapshot,
   - terminal or unknown task ids fail closed,
   - inspect stays non-mutating
4. integration tests for active-ephemeral cancel:
   - allowed cancel interrupts one exact active task,
   - cancel closeout is distinct from retained-worker cancel and stop,
   - repeated or late cancel against terminal tasks fails closed with stable errors
5. regression tests proving:
   - retained inspect/cancel behavior stays green,
   - `run_world_task` remains one-shot and non-durable,
   - no public `substrate agent ...` semantics change,
   - no Family-2 ledger, inbox, or router widening is introduced

Coverage expectations:

1. every accepted active-ephemeral path has exact-identity tests,
2. terminal-race behavior is explanation-ready and pinned in tests,
3. inspect snapshot and cancel closeout remain truthful under Linux-first runtime coverage,
4. retained routing remains exact and non-regressed,
5. transport/world-service tests are extended if the active-task carrier changes there.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - freeze exact runtime-owned `task_run_id` truth before widening inspect/cancel
  - keep active-ephemeral inspect snapshot-only and non-mutating
  - keep active-ephemeral cancel exact-task only and explanation-ready on races
  - preserve one-shot terminal semantics for `run_world_task`
- Ask first:
  - widening public CLI or toolbox selection/syntax
  - turning active-ephemeral task identity into a durable retained-like registry
  - redesigning transport/world-service schema beyond the minimum exact identity carrier
  - coupling Slice `47` to Family-2 router/daemon execution or host-global ingress
- Never:
  - infer active task targets from prompts, role labels, or recency
  - silently reopen terminal one-shot tasks for future continuation
  - collapse active-ephemeral cancel into retained cancel or stop semantics
  - widen this slice into stop, fork, approval autonomy, or retained lifecycle redesign

## Success Criteria

Slice `47` is complete only when all of the following are true:

1. active `run_world_task` execution has exact runtime-owned `task_run_id` runtime truth,
2. `inspect_world_worker` and `cancel_world_work` may target `mode=ephemeral` only when the caller supplies that exact `task_run_id`,
3. active-ephemeral inspect returns a typed authoritative active snapshot and fails closed after terminal teardown,
4. active-ephemeral cancel interrupts one exact active task and returns truthful cancel closeout distinct from retained cancel and stop semantics,
5. terminal one-shot outcomes remain terminal, non-routable, and non-durable,
6. retained inspect/cancel behavior stays green and exact,
7. public caller surfaces, broader worker autonomy, and Family-2 router/attach execution remain explicitly out of scope.

## Open Questions

1. Should `task_run_id` be the streamed execute `span_id` surfaced directly, or a normalized wrapper over that same runtime-owned identity? Default assumption for this spec: reuse the existing runtime-owned execute identity if it can be surfaced without ambiguity.
2. Should active-ephemeral inspect snapshot include only lifecycle/progress truth in v1, or also a narrow continuity hint when registration metadata exists? Default assumption for this spec: keep the snapshot narrow and non-resumable unless implementation proves richer output is effectively free.
