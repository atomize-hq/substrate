# Spec: Internal Retained World Worker Continue And Event Bootstrap

Source validation note: [NOTE-33-family-1-ordering-after-dispatch-bootstrap.md](./NOTE-33-family-1-ordering-after-dispatch-bootstrap.md)  
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md)
- [PLAN-32.md](./PLAN-32.md)
- [TASKS-32.md](./TASKS-32.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Phase: `SPECIFY`  
Status: draft for review on `2026-05-30`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `32` has already reached the retained-worker bootstrap floor on the current tree: internal dispatch validation is real, Linux `run_world_task` is real, and `spawn_world_worker` now returns authoritative retained-worker bootstrap identity instead of remaining stubbed.
2. The next family-1 slice remains internal-only and orchestrator-only. It does not widen public `substrate agent ...` caller surfaces and it does not turn toolbox into an execution plane.
3. The next narrow slice should reuse the already-landed exact retained-member follow-up/runtime seam rather than creating a second world execution path.
4. The first retained-worker follow-up slice should land `continue_world_worker` plus a minimal typed worker-event normalization contract, not the full later steering/action matrix.
5. Family 2 stays out of scope except for compatibility boundaries. This slice may expose event fields that later obligation/inbox projection can consume, but it must not redesign the obligation ledger, inbox schema, or auto-attach flow.

If any of these are wrong, correct them before implementation.

## Objective

Build the first real retained-worker follow-up control-plane slice so the host orchestrator can continue an exact retained world worker through an internal contract and receive typed worker-to-host event outcomes instead of relying on human/operator follow-up surfaces.

Primary runtime story:

1. the host orchestrator issues an internal `continue_world_worker` request against one exact retained worker,
2. Substrate validates exact orchestration-session identity, caller identity, retained worker identity, backend identity, and authoritative world binding,
3. Substrate routes the request over the already-landed retained member-turn seam,
4. Substrate returns a typed outcome/event envelope for the minimal in-scope classes,
5. attention-driving worker events remain explicit and typed,
6. fuller steering-policy hardening, approvals, fork autonomy, inspect/cancel/stop verbs, and public caller expansion remain deferred.

## Frozen Direction

This slice freezes the following:

1. retained-worker messaging comes before fuller steering-policy hardening,
2. `continue_world_worker` is the only new control-plane verb in scope,
3. exact `target_participant_id` is mandatory for every continued retained-worker turn,
4. the slice reuses the existing exact retained member-turn/runtime seam rather than inventing a new transport,
5. the first typed worker-event subset is:
   - `reply`
   - `progress_update`
   - `follow_up_question`
   - `blocked`
   - `result`
   - `failure`
6. `attention_required` is in scope as typed event semantics, not as a new family-2 architecture pass,
7. `thread_id` should remain explicit in surfaced event metadata when available, but v1 may simplify runtime behavior to one primary conversational thread per retained worker,
8. approval, fork, inspect, cancel, stop, and broader operational steering classes remain out of scope for this slice.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing family-1 bootstrap truth:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- Existing retained follow-up/runtime seam:
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
  - [`crates/world-service/src/service.rs`](../crates/world-service/src/service.rs)
- Existing prompt/event parsing surfaces that may be reused:
  - [`crates/shell/src/execution/prompt_fulfillment.rs`](../crates/shell/src/execution/prompt_fulfillment.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- Existing public/runtime behavior that must not regress:
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - [`docs/TRACE.md`](../docs/TRACE.md)

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

Existing targeted validation floor:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - internal retained-worker continue entrypoint, exact-target validation, and typed outcome routing
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - `continue_world_worker` action, payload, and typed minimal worker-event/outcome scaffolding
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained-worker target resolution and fail-closed identity checks
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - helper reuse or bridging for retained member-turn submission
- `crates/world-service/src/member_runtime.rs`
  - exact retained turn submission semantics and surfaced thread/turn metadata handling
- `crates/shell/src/execution/prompt_fulfillment.rs`
  - shared parsing helpers only if reuse keeps the event classification narrow and truthful
- `crates/shell/tests/`
  - internal continue contract, target validation, event classification, and regression coverage
- `docs/TRACE.md`
  - any new internal control-plane event/audit rows if introduced
- `llm-last-mile/`
  - this spec plus the matching plan/tasks artifacts and sequencing note

## Code Style

Follow the existing shell/world runtime style: exact identity, typed contracts, explanation-ready denials, and narrow reuse of existing world-member plumbing.

Preferred style:

```rust
match (&request.action, &request.payload) {
    (
        WorldDispatchActionV1::ContinueWorldWorker,
        WorldDispatchPayloadV1::WorkerContinue(payload),
    ) => validate_continue_payload(payload)?,
    _ => anyhow::bail!(
        "invalid_dispatch_payload: action {} requires matching typed payload",
        request.action.as_str(),
    ),
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane and transport boundaries,
2. fail closed on missing or mismatched retained-worker identity,
3. keep typed worker-event classification explicit rather than inferring from free-form text alone,
4. preserve exact `(orchestration_session_id, target_participant_id, backend_id, world_id, world_generation)` truth,
5. keep approval, fork, inspect, cancel, and stop classes out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- existing shell and world-service regression suites

Test levels for this slice:

1. unit tests for `continue_world_worker` request validation:
   - exact required fields
   - payload shape
   - exact retained-worker target requirements
2. unit tests for authoritative target resolution:
   - same-session only
   - same-world-binding only
   - authoritative orchestrator caller only
   - exact retained worker only
3. integration tests for continued retained turns:
   - the internal control-plane path reuses the retained member-turn seam
   - exact retained member identity is preserved
   - stale or invalidated retained targets fail closed
4. integration tests for minimal typed worker-event classification:
   - `reply`
   - `progress_update`
   - `follow_up_question`
   - `blocked`
   - `result`
   - `failure`
5. regression tests proving:
   - no public CLI behavior changes
   - no approval/fork/control-directive claims appear yet
   - no family-2 ledger or inbox schema expansion is required just to land this slice

Coverage expectations:

1. every accepted event class in scope has both positive and negative tests,
2. same-session and same-world-binding denials remain pinned explicitly,
3. ambiguous or missing retained target identity fails closed,
4. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `continue_world_worker` as the only new control-plane verb in scope
  - require exact retained worker identity before routing
  - reuse the existing retained member-turn/runtime seam
  - keep typed worker-event classes explicit and narrow
- Ask first:
  - adding `inspect_world_worker`, `cancel_world_work`, `stop_world_worker`, or `fork_world_worker`
  - adding approval or fork event classes to the first shipping slice
  - widening the public CLI or toolbox posture
  - coupling this slice to new family-2 schema or router behavior
- Never:
  - do fuzzy worker selection by recency, role name, or prompt text
  - create a second retained-worker execution plane
  - silently promote unresolved worker events into new family-2 architecture scope
  - widen this slice into the full steering policy/config matrix

## Success Criteria

This slice is done only when all of the following are true:

1. an internal orchestrator-only `continue_world_worker` caller surface exists,
2. exact retained-worker identity is required before any continued turn can route,
3. the slice reuses the existing retained member-turn/runtime seam,
4. the first typed worker-event subset in scope is surfaced through a stable internal outcome contract,
5. attention-driving worker events remain explicit and typed,
6. public human CLI behavior remains unchanged,
7. approval, fork, inspect, cancel, stop, and fuller steering-policy hardening remain deferred,
8. the validation wall is green.

## Open Questions

1. No open design question is being left for implementation.
