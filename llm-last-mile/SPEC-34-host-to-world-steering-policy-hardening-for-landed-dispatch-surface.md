# Spec: Host-To-World Steering Policy Hardening For Landed Dispatch Surface

Source validation note: [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)  
Related design stack:
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)
- [PLAN-33.md](./PLAN-33.md)
- [TASKS-33.md](./TASKS-33.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Phase: `SPECIFY`  
Status: draft for review on `2026-05-31`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `32` and Slice `33` are already landed on the current tree, so `run_world_task`, `spawn_world_worker`, and `continue_world_worker` are real internal family-1 verbs rather than planned-only surfaces.
2. Slice `34` stays internal-only and orchestrator-only. It does not add a new public human CLI and it does not widen `substrate agent start|turn|reattach|fork|stop`.
3. The next missing seam is the separate deny-by-default steering-policy layer for those landed verbs, not new verb expansion and not family-2 router/attach execution.
4. This slice should harden only the currently landed verb surface:
   - `run_world_task`
   - `spawn_world_worker`
   - `continue_world_worker`
5. `inspect_world_worker`, `cancel_world_work`, `stop_world_worker`, `fork_world_worker`, approval-request policy, and worker fork autonomy remain out of scope for this slice.
6. Family 2 remains out of scope except for policy-boundary compatibility with already-landed attention/event truth. This slice must not redesign the obligation ledger, auto-attach queueing model, or router ownership.
7. Linux remains the source-of-truth implementation target. macOS/Lima parity may preserve compile/runtime posture only where the current internal dispatch seam already does so. Windows/WSL is out of scope.

If any of these are wrong, correct them before implementation.

## Objective

Ship the first implementation-bearing host-to-world steering-policy layer so the landed internal dispatch surface becomes explicitly deny-by-default, explanation-ready, and separated from execution-plane runtime capability truth.

Primary operator/runtime story:

1. the host orchestrator issues one of the already-landed internal world-dispatch requests,
2. Substrate evaluates a distinct steering-policy layer before world routing proceeds,
3. that policy layer decides whether the request is allowed based on:
   - steering enabled,
   - action,
   - mode,
   - backend,
   - same-session boundary,
   - same-world-binding boundary,
   - capability-narrowing permission,
   - current worker/session concurrency and routability truth where applicable,
4. allowed requests continue through the already-landed dispatch runtime path unchanged,
5. denied requests fail closed with stable, explanation-ready denial buckets,
6. public human caller surfaces remain unchanged,
7. later family-1 verbs and family-2 router/attach behavior remain deferred.

## Frozen Direction

This slice freezes the following:

1. steering-policy hardening comes before later family-1 verb expansion,
2. the policy layer is separate from runtime capability resolution,
3. the in-scope verbs are only:
   - `run_world_task`
   - `spawn_world_worker`
   - `continue_world_worker`
4. exact identity remains mandatory:
   - `orchestration_session_id`
   - `caller_participant_id`
   - exact `backend_id`
   - exact authoritative `world_id`
   - exact authoritative `world_generation`
   - exact retained `target_participant_id` for `continue_world_worker`
5. deny-by-default posture applies to:
   - global steering enablement,
   - allowed actions,
   - allowed modes,
   - allowed backends,
   - capability narrowing,
   - worker/session concurrency where in-scope,
6. invalidated or otherwise non-routable retained workers must fail closed for `continue_world_worker`,
7. the slice may introduce a minimal implementation-bearing policy/config surface conceptually equivalent to the design doc’s `agents.world_dispatch.*` dimensions, but it must not widen into a separate large policy redesign unrelated to current verbs,
8. already-landed minimal continue-event truth remains narrow:
   - `reply`
   - `progress_update`
   - `follow_up_question`
   - `blocked`
   - `result`
   - `failure`
9. approval, fork, inspect, cancel, stop, and router-owned attach continuation remain out of scope.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing dispatch/runtime truth:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- Existing policy/config surfaces:
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- Existing world execution seam:
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
  - [`crates/world-service/src/service.rs`](../crates/world-service/src/service.rs)
- Existing public/runtime behavior that must not regress:
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - [`docs/TRACE.md`](../docs/TRACE.md)
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)

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
cargo test -p shell policy_model -- --nocapture
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

- `crates/broker/src/`
  - policy data model and validation for implementation-bearing world-dispatch policy dimensions
- `crates/shell/src/execution/policy_model.rs`
  - config/patch plumbing if new policy keys or explain paths are introduced
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - steering-policy evaluation, denial buckets, and pre-routing enforcement
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - stable deny vocabulary and any narrow contract support for policy-aware gating
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative current-state checks for invalidated/terminal retained targets and any policy-needed counts
- `crates/shell/tests/`
  - policy/config, denial, concurrency, and regression coverage
- `docs/CONFIGURATION.md`
  - documentation for any landed policy/config surface
- `docs/TRACE.md`
  - policy denial or audit rows if this slice adds new structured trace semantics
- `llm-last-mile/`
  - this spec and the matching note/plan/tasks artifacts

## Code Style

Follow the existing shell/broker style: explicit fail-closed validation, explanation-ready denials, and no hidden blending of authorization with transport execution.

Preferred style:

```rust
if !policy.world_dispatch_enabled {
    anyhow::bail!("world_dispatch_disabled: host-to-world steering is disabled by effective policy");
}

if !policy.allowed_actions.contains(&request.action) {
    anyhow::bail!(
        "action_not_allowed: effective policy does not allow {}",
        request.action.as_str(),
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at policy and runtime boundaries,
2. keep denial reasons stable enough to pin in tests and explain to operators,
3. preserve exact identity routing rather than inferring from role, recency, or free-form text,
4. perform steering authorization before world routing proceeds,
5. keep approval/fork/control-directive expansion out of this slice,
6. keep router/attach projection behavior separate from control-plane policy enforcement.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- existing shell and world-service regression suites

Test levels for this slice:

1. unit tests for steering-policy parsing/validation:
   - defaults deny by default,
   - allowed backend/action/mode lists validate cleanly,
   - capability narrowing permission remains explicit,
   - any landed concurrency-cap fields validate correctly
2. unit tests for pre-routing policy enforcement:
   - `world_dispatch_disabled`
   - `backend_not_allowed`
   - `mode_not_allowed`
   - `action_not_allowed`
   - `cross_session_steering_denied`
   - `cross_world_binding_steering_denied`
   - `invalidated_worker_not_routable`
   - `worker_concurrency_cap_exceeded`
3. integration tests for landed verbs:
   - `run_world_task` allowed only when the policy admits `ephemeral`
   - `spawn_world_worker` allowed only when the policy admits `retained`
   - `continue_world_worker` allowed only for exact retained targets that remain routable
4. regression tests proving:
   - current public CLI semantics stay unchanged,
   - current runtime seam stays the only execution path,
   - approval/fork/inspect/cancel/stop remain deferred,
   - family-2 obligation or attach processing is not introduced by policy hardening alone

Coverage expectations:

1. each in-scope denial bucket has both positive and negative tests,
2. same-session and same-world-binding denials stay pinned explicitly,
3. invalidated or stale retained workers cannot be continued,
4. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - harden only the already-landed three-verb surface
  - separate steering authorization from runtime capability resolution
  - preserve exact session, worker, backend, and world-binding truth
  - keep denial buckets explanation-ready and test-pinned
- Ask first:
  - adding `inspect_world_worker`, `cancel_world_work`, `stop_world_worker`, or `fork_world_worker`
  - adding approval-request or fork-request autonomy policy in the same slice
  - broadening into router/daemon attach execution or obligation-ledger redesign
  - widening any public human CLI or toolbox posture beyond the current internal control plane
- Never:
  - allow fuzzy worker selection
  - treat attach restoration as worker continuation
  - synthesize prompts from obligations or policy denials
  - let runtime capability allowance imply steering authorization
  - silently widen this slice into family-2 projection logic

## Success Criteria

This slice is done only when all of the following are true:

1. a distinct deny-by-default steering-policy layer exists for the landed internal world-dispatch surface,
2. the policy layer can gate current verbs by enablement, backend, action, mode, and exact boundary truth,
3. `continue_world_worker` fails closed for invalidated or otherwise non-routable retained workers,
4. explanation-ready denial buckets are stable and test-pinned,
5. any landed config/policy surface remains narrow and specific to the current verb set,
6. public human CLI behavior remains unchanged,
7. approval/fork/inspect/cancel/stop and family-2 router/attach work remain deferred,
8. the validation wall is green.

## Open Questions

1. No open design question is being left for implementation. This slice should implement the current design stack narrowly against the already-landed three-verb surface.
