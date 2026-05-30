# Spec: Internal Host-Orchestrator World Dispatch Bootstrap

Source validation note: [REMAINING-family-1-scope-2026-05-30.md](./REMAINING-family-1-scope-2026-05-30.md)  
Related design stack:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Phase: `SPECIFY`  
Status: draft for review on `2026-05-30`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `32` is internal-only. It does not add a new public human CLI and it does not widen `substrate agent start|turn|reattach|fork|stop`.
2. The first live family-1 caller surface should be a minimal orchestrator-only internal control-plane/toolbox seam, not a general toolbox rollout and not a second execution plane.
3. The existing world-member launch and exact follow-up runtime seams remain the execution substrate for this slice; they are not being replaced.
4. This slice is allocation/bootstrap only. It should support `run_world_task` and `spawn_world_worker`, but not retained-worker messaging, `continue_world_worker`, `fork_world_worker`, `cancel_world_work`, or `stop_world_worker`.
5. Family 2 remains out of scope except for dependency notes. This slice must not reopen obligation-ledger, inbox/review projection, router-owned auto-attach, or cross-host ingress design.
6. Linux remains the source-of-truth implementation target; supported macOS/Lima forwarded parity may be preserved only where the current runtime seam already supports it. Windows/WSL is out of scope.

If any of these are wrong, correct them before implementation.

## Objective

Build the first real host-orchestrator to world control-plane entry seam so an attached host orchestrator can request initial world work through a typed internal contract instead of relying on human/operator caller surfaces.

Primary operator/runtime story:

1. the host orchestrator calls an internal control-plane surface,
2. Substrate validates exact orchestration-session identity, caller identity, target backend, and authoritative world binding,
3. Substrate executes either:
   - `run_world_task` as one-shot `ephemeral` world work, or
   - `spawn_world_worker` as retained worker allocation,
4. Substrate returns a typed outcome:
   - ephemeral terminal outcome for `run_world_task`,
   - authoritative retained worker receipt for `spawn_world_worker`,
5. no ongoing conversational retained-worker messaging is claimed yet,
6. no family-2 durable obligation production is required yet.

## Frozen Direction

This slice freezes the following:

1. dispatch/bootstrap comes before retained-worker messaging and before fuller steering-policy hardening,
2. the first internal family-1 caller surface is orchestrator-only and control-plane only,
3. exact identity remains mandatory:
   - `orchestration_session_id`
   - `caller_participant_id`
   - exact `backend_id`
   - exact authoritative `world_id`
   - exact authoritative `world_generation`
4. `run_world_task` is the only `ephemeral` verb in scope,
5. `spawn_world_worker` is the only `retained` verb in scope,
6. capability overrides remain narrowing-only,
7. fuzzy routing, default worker selection, and synthetic host/world prompt shims remain out of scope.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing state/control truth:
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- Existing world execution seam:
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
  - [`crates/world-service/src/service.rs`](../crates/world-service/src/service.rs)
- Existing caller/runtime identity floor:
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
- Existing toolbox posture docs:
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - [`docs/TRACE.md`](../docs/TRACE.md)
  - [`docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md)

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
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

Future internal-surface smoke expectations once this slice exists:

```bash
substrate agent toolbox status --json
substrate agent toolbox env --json
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/`
  - request/outcome contract, exact-identity validation, and orchestration-session ownership checks
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - reuse of the existing world dispatch transport path
- `crates/world-service/src/member_runtime.rs`
  - authoritative world-member launch and receipt handling
- `crates/world-service/src/service.rs`
  - request adaptation if the internal control-plane bootstrap needs a transport-specific bridge
- `crates/shell/src/execution/`
  - a minimal new internal orchestrator-control/toolbox module or equivalent adapter surface
- `crates/shell/tests/`
  - new coverage for internal dispatch bootstrap, exact identity, and fail-closed denials
- `docs/TRACE.md`
  - internal toolbox/control-plane audit events if new trace rows are added
- `docs/USAGE.md`
  - documentation for toolbox/control-plane posture if the internal caller surface becomes real
- `llm-last-mile/`
  - this spec plus the matching plan/tasks artifacts

## Code Style

Follow the existing `shell` and `world-service` style: explicit fail-closed validation, stable contract wording, and narrow helpers around authoritative identity.

Preferred style:

```rust
match (&request.action, &request.mode) {
    (WorldDispatchAction::RunWorldTask, WorldDispatchMode::Ephemeral) => {}
    (WorldDispatchAction::SpawnWorldWorker, WorldDispatchMode::Retained) => {}
    _ => anyhow::bail!(
        "invalid_dispatch_action_mode: action {} is incompatible with mode {}",
        request.action.as_str(),
        request.mode.as_str(),
    ),
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at state and transport boundaries,
2. keep denials explanation-ready and stable enough for tests,
3. preserve exact target identity rather than inferring from role or recency,
4. reuse the existing world-member launch path instead of introducing an alternate execution stack,
5. keep retained-worker messaging and family-2 obligation production out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- existing shell and world-service regression suites

Test levels for this slice:

1. unit tests for request validation:
   - action/mode compatibility
   - exact required fields
   - same-session and same-world-binding checks
2. unit tests for minimal caller authorization:
   - orchestrator-only access
   - denied when authoritative parent/session truth is missing
3. integration tests for `run_world_task`:
   - exact backend launch
   - typed terminal outcome
   - fail-closed on stale linkage or invalid world binding
4. integration tests for `spawn_world_worker`:
   - authoritative retained worker receipt
   - exact child identity and lineage
   - no accidental public follow-up widening
5. regression tests proving:
   - current public CLI semantics stay unchanged
   - toolbox remains internal-only and does not become a general execution plane
   - no family-2 obligation rows are created just because slice `32` dispatch ran

Coverage expectations:

1. every accepted request kind in scope has both positive and negative tests,
2. same-session and same-world-binding denials are pinned explicitly,
3. stale or invalid retained linkage still fails closed,
4. public `agent start|turn|reattach|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `run_world_task` and `spawn_world_worker` as the only new verbs in scope
  - reuse the existing exact world-member runtime seam
  - keep exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` authoritative
- Ask first:
  - adding a public human CLI for the new verbs
  - widening the toolbox into a broader internal MCP rollout
  - introducing family-2 durable obligation or auto-attach semantics into this slice
  - adding Windows/WSL parity requirements
- Never:
  - add fuzzy routing or default worker selection
  - let a world worker silently become the new host orchestrator
  - treat public `agent turn` or REPL targeted turns as the host-orchestrator control plane
  - silently widen this slice into retained-worker messaging or fork autonomy

## Success Criteria

This slice is done only when all of the following are true:

1. an internal orchestrator-only caller surface exists for initial host-to-world dispatch,
2. `run_world_task` and `spawn_world_worker` each have a typed request/outcome path,
3. exact session, caller, backend, and world-binding checks are enforced before execution,
4. `run_world_task` returns a typed terminal outcome without claiming durable retained-worker messaging,
5. `spawn_world_worker` returns authoritative retained worker identity and launch receipt without claiming ongoing steering support,
6. public human CLI behavior remains unchanged,
7. no family-2 durable obligation/auto-attach behavior is implicitly coupled into this slice,
8. the validation wall is green.

## Open Questions

None required to draft the next implementation plan.

The remaining design questions are intentionally deferred into later slices:

1. retained-worker messaging and steering,
2. fuller deny-by-default steering policy dimensionality,
3. worker-requested fork, approval, and attention event families,
4. cross-host or host-global ingress.
