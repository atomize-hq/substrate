# Spec: Internal Family-2 Router-Owned Session Auto-Attach Execution Boundary

Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Prior slices:
- [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](./SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)
- [PLAN-31.md](./PLAN-31.md)
- [TASKS-31.md](./TASKS-31.md)
- [SPEC-31_25.md](./SPEC-31_25.md)
- [PLAN-31_25.md](./PLAN-31_25.md)
- [TASKS-31_25.md](./TASKS-31_25.md)
- [SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md](./SPEC-47-internal-active-ephemeral-task-identity-and-inspect-cancel-widening.md)
- [PLAN-47.md](./PLAN-47.md)
- [TASKS-47.md](./TASKS-47.md)
Related design stack:
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-07`
Landed posture note: the first internal host-side router-owned local-session auto-attach execution boundary is landed repo-wide; it remains Linux-only in v1, local-session-only, deny-by-default through `workflow.router.enabled` plus the relevant existing retained-obligation gate, continuity-first on persisted attach truth, and bounded away from cross-host ingress, public daemon UX, worker continuation, and review-state resolution.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope Slice `48` stabilization follow-up, and no broader host-targeting, ingress, or workflow-engine work was reopened.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `47` closed the current Family-1 control-plane track, so the next honest implementation-bearing slice is Family 2 rather than another retained-worker message or active-ephemeral widening slice.
2. The cited design stack is coherent enough to freeze Slice `48` now. The remaining questions are execution-boundary choices, not missing product intent.
3. The first Family-2 slice should choose a host-side router-owned work-loop boundary, not synchronous inline evaluation inside ordinary retained-worker control flows and not world-service-owned attach logic.
4. Slice `48` should stay Linux-first and local-only:
   - local orchestration-session obligations only,
   - local persisted host attach contract only,
   - no host-global inbox,
   - no remote ingress,
   - no wrong-host routing beyond fail-closed local checks.
5. Slice `48` should reuse the already-landed local obligation ledger and attach helper surfaces rather than renaming or redesigning them first. The current runtime names:
   - `NotEligible`
   - `Eligible`
   - `Claimed`
   - `Satisfied`
   - `FailedClosed`
   - `Superseded`
   should be treated as the current implementation-era equivalents of the forward design states:
   - `not_requested`
   - `queued`
   - `claimed`
   - `completed`
   - `dead_letter`
   - `cancelled`
6. The minimum deny-by-default policy surface for Slice `48` should reuse existing policy families instead of inventing a brand-new top-level schema:
   - `workflow.router.enabled` gates router-owned indirect execution,
   - existing `agents.world_dispatch.obligations.*` and `agents.world_dispatch.fork.requests_allowed` gates remain the per-obligation-kind allow/deny inputs.
7. Successful router-owned attach restores a sanctioned host execution client only. It does not auto-resolve obligation review state, auto-submit prompts, continue workers, or broaden public `substrate agent ...` behavior.
8. Broader host-targeted obligation identity fields such as `origin_host_id`, `target_host_id`, and ingress metadata remain future scope and must stay out of Slice `48` except for fail-closed preservation of the boundary.

If any of these are wrong, correct them before implementation.

## Objective

Build the first Family-2 execution-boundary slice so Substrate has a real host-side router-owned automatic attach path for detached local orchestration sessions with eligible unresolved obligations, using exact session truth, continuity-first attach restoration, fail-closed policy gating, and session-scoped coalescing, without widening into cross-host ingress, workflow-engine breadth, public daemon UX, obligation-review resolution, or worker continuation logic.

Primary runtime story:

1. a detached local orchestration session has one or more unresolved eligible obligations,
2. a host-side router-owned internal work-loop or equivalent internal entrypoint evaluates local session and obligation truth,
3. router-owned policy checks confirm indirect execution is allowed and the obligation kind remains eligible under current policy,
4. the router claims at most one obligation per orchestration session for the attach episode,
5. the router chooses continuity attach first and fresh attach only when continuity is unavailable but persisted attach truth remains valid,
6. the router launches the existing sanctioned hidden-owner helper path to restore a host execution client,
7. success records attach satisfaction/supersession on the local obligation set without resolving review state,
8. failure records explanation-ready fail-closed attach outcome without silently retrying or continuing work.

## Observed Repo Floor

The current repo already has most of the local mechanics this slice needs:

1. the canonical local deferred-work artifact exists as [`OrchestrationObligationRecord`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) with explicit obligation kind, review state, and attach state,
2. local session posture already derives detached attention from obligation truth through [`AgentRuntimeStateStore`](../crates/shell/src/execution/agent_runtime/state_store.rs),
3. local session-scoped attach claiming and attach-settlement helpers already exist in [`state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs),
4. a real router-style execution helper already exists as [`execute_session_auto_attach(...)`](../crates/shell/src/execution/agent_runtime/auto_attach.rs),
5. continuity-first versus fresh-attach fallback already routes through the sanctioned hidden-owner helper launch seam in [`auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs),
6. manual exact-session `reattach` already exists and shares the same persisted attach-contract authority model,
7. policy/config schema already publishes:
   - `workflow.router.*`
   - `agents.world_dispatch.obligations.*`
   - `agents.world_dispatch.fork.requests_allowed`
   even though the current auto-attach claim/execute path does not yet consult a dedicated router policy layer.

The key remaining gap is that the repo still has no non-test production-owned call site for `execute_session_auto_attach(...)`:

1. attach claim and execution machinery exists,
2. session-scoped coalescing and settlement helpers exist,
3. but no host-side router-owned work-loop or equivalent internal entrypoint owns the production evaluation -> claim -> attach -> settle cycle.

That is why Slice `48` is the next honest Family-2 slice.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing local Family-2 runtime truth expected to be reused and widened:
  - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- Existing policy/config surfaces expected to be consulted or minimally widened:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
  - [`docs/reference/policy/contract.md`](../docs/reference/policy/contract.md)
- Existing sanctioned host attach launch path expected to remain authoritative:
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- Existing trace contract that may need narrow router-owned derived-event reuse:
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

Targeted validation floor:

```bash
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `crates/shell/src/execution/agent_runtime/auto_attach.rs`
  - existing claim selection, attach-mode selection, hidden-owner launch, and failure-settlement helpers
  - Slice `48` should keep this file as the semantic center of router-owned session auto-attach execution
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative local session load, obligation persistence, claim coalescing, and post-attach settlement
  - Slice `48` should keep session-scoped coalescing and durable attach-state convergence here
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - current local obligation and attach-state model
  - Slice `48` may add narrow helpers or comments, but should not redesign the whole forward identity envelope
- `crates/shell/src/execution/`
  - host-side runtime wiring, policy lookup, and any new internal router-owned work-loop or entrypoint
  - Slice `48` should introduce the production call site here rather than inside world-service
- `crates/broker/src/` and `crates/shell/src/execution/policy_model.rs`
  - deny-by-default policy schema and effective-policy resolution
  - Slice `48` should reuse existing `workflow.router.*` and world-dispatch obligation gates unless implementation proves a missing key is unavoidable
- `docs/`
  - policy and trace contract docs that must stay honest if router-owned auto-attach becomes live runtime truth
- `llm-last-mile/`
  - repo-local planning/spec artifacts for this orchestration family

## Code Style

Follow the existing shell/runtime style: exact session identity, fail-closed policy checks, deterministic session-level coalescing, and explanation-ready attach outcomes.

Preferred style:

```rust
if !policy.workflow_router_enabled {
    anyhow::bail!(
        "router_auto_attach_disabled: workflow.router.enabled must be true before router-owned automatic attach may execute"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at policy, attach-contract, and session-boundary checks,
2. keep router-owned auto-attach local-session scoped and exact-identity only,
3. do not infer eligibility from raw events when durable obligation truth is available,
4. keep continuity-first attach selection explicit and explanation-ready,
5. keep successful attach distinct from obligation review resolution,
6. keep the router out of worker continuation, approval, clarification, or hidden prompt submission.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/effective-policy tests
- shell regression suites

Test levels for this slice:

1. unit tests for router-owned eligibility and fail-closed policy behavior:
   - `workflow.router.enabled` is required,
   - obligation kinds remain deny-by-default unless the relevant existing world-dispatch gate allows them,
   - attached sessions, terminal sessions, missing attach contracts, and stale boundary truth fail closed
2. unit tests for session-scoped coalescing and claim semantics:
   - at most one claimed obligation per orchestration session,
   - deterministic leader selection remains stable,
   - duplicate or sibling claims fail closed
3. attach execution tests:
   - continuity attach is preferred when valid,
   - fresh attach is attempted only when continuity is unavailable but persisted attach truth remains valid,
   - no hidden prompt submission occurs,
   - success restores attached host truth and settles only attach-state substates
4. manual-reattach interaction and regression tests:
   - successful manual `reattach` converges queued or claimed router work without duplicate attach launch,
   - successful router attach does not resolve obligation review state,
   - unresolved sibling obligations remain unresolved after attach restoration
5. observability tests:
   - router-owned decisions remain explanation-ready,
   - if trace families are emitted, they preserve exact session/obligation/backend joins without introducing a broader trace redesign

Coverage expectations:

1. every accepted router-owned attach path has exact session-boundary and local-policy coverage,
2. success, cancellation/supersession, and fail-closed outcomes remain distinct,
3. no test implies cross-host routing, obligation review resolution, or worker continuation,
4. Linux-first local behavior is the only required runtime floor for this slice.

## Boundaries

- Always:
  - keep Slice `48` internal-only and host-side
  - keep router-owned auto-attach driven by durable local obligations, not raw stream events
  - keep session-scoped coalescing exact and fail-closed
  - keep continuity-first attach restoration explicit
  - keep success semantics limited to restoring attached host ownership plus attach-state settlement
- Ask first:
  - introducing a new public CLI surface for router or daemon lifecycle
  - renaming the current obligation attach-state enum to the forward design vocabulary
  - widening into host-global inbox, remote ingress, or cross-host targeting fields
  - inventing a brand-new top-level policy namespace if existing `workflow.router.*` plus world-dispatch gates are sufficient
- Never:
  - auto-submit obligation summaries as prompts
  - treat attach restoration as obligation resolution
  - move router-owned auto-attach into world-service ownership for this slice
  - silently continue, approve, clarify, or fork worker work from the router path
  - widen Slice `48` into a general workflow engine or public toolbox execution plane

## Success Criteria

Slice `48` is complete only when all of the following are true:

1. the repo has a real non-test host-side router-owned execution path for local session auto-attach rather than a dead helper only,
2. router-owned execution is deny-by-default and requires `workflow.router.enabled` plus the relevant existing per-kind world-dispatch gates,
3. router-owned execution evaluates only local exact session truth, local obligations, and persisted attach-contract truth,
4. at most one obligation may lead an attach episode per orchestration session at a time,
5. continuity attach is preferred when valid, fresh attach is fallback only when continuity is unavailable but persisted attach truth remains valid, and neither path injects hidden prompts,
6. successful attach restores attached host truth and settles attach substates without resolving review state,
7. manual `reattach` and router-owned attach converge without duplicate launch,
8. public CLI semantics, cross-host ingress, broader obligation identity envelope, and worker continuation remain explicitly out of scope.

## Open Questions

1. Should the first shipping router-owned work-loop surface as a dedicated hidden `substrate` internal entrypoint or as a shell-owned bootstrap path under an existing host-side lifecycle? Default assumption for this spec: either is acceptable as long as the production-owned call site is host-side, internal-only, and not world-service-owned.
2. Should router-owned attach decisions reuse the documented `workflow_router_*` derived trace family immediately, or emit a narrower internal trace shape first? Default assumption for this spec: reuse the existing workflow-router component vocabulary only if exact join fields can be emitted without reopening broader trace-schema work.
3. Should the current attach-state enum names be renamed now to match the forward design vocabulary? Default assumption for this spec: no; preserve current runtime names and document their semantic mapping so Slice `48` stays execution-boundary scoped.
