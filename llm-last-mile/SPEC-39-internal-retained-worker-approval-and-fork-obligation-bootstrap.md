# Spec: Internal Retained Worker Approval And Fork Obligation Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Related design stack:
- [PLAN-38.md](./PLAN-38.md)
- [TASKS-38.md](./TASKS-38.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)  
Phase: `SPECIFY`  
Status: proposed on `2026-06-02`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `38` is fully landed on the current tree, so the live Family-1 internal action surface is now `run_world_task`, `spawn_world_worker`, `fork_world_worker`, `continue_world_worker`, `inspect_world_worker`, `cancel_world_work`, and `stop_world_worker`.
2. The next honest Family-1 slice is no longer another verb. It is the first widening of retained-worker deferred event semantics: `approval_request`, `fork_request`, and `fork_recommendation`.
3. Slice `39` should stay internal-only and orchestrator-facing. It should not widen public `substrate agent ...` surfaces, toolbox posture, or human direct-to-world messaging.
4. Accepted `fork_request` remains host-mediated in v1:
   - the worker may emit `fork_request`,
   - Substrate may persist a durable obligation,
   - the host may later issue explicit `fork_world_worker`,
   - the worker must not auto-fork.
5. Approval resolution does not require a new public CLI or a new router-owned continuation path in this slice. Slice `39` only needs the durable worker-to-host request surface; any richer typed host `approval_response` protocol can remain later work if the live `continue_world_worker` path is still sufficient for v1.
6. The canonical local obligation ledger and its current runtime naming stay in place for this slice:
   - `OrchestrationObligationKind::{ApprovalRequired,ForkRequest,ForkRecommendation}` remain the durable kinds,
   - attach-state naming remains the currently landed local runtime surface (`not_eligible`, `eligible`, `claimed`, `satisfied`, `failed_closed`, `superseded`),
   - this slice must not reopen the future host-global inbox envelope or the future queue-state naming described in design docs.
7. Family 2 router/attach execution remains downstream. This slice may make `approval_request` and `fork_request` obligations visible to existing local auto-attach eligibility, but it must not redesign router ownership, claim coordination, or remote ingress.

If any of these are wrong, correct them before implementation.

## Objective

Build the first post-Slice-38 Family-1 widening slice so retained workers can emit typed `approval_request`, `fork_request`, and `fork_recommendation` events that Substrate accepts, policy-gates, classifies, and persists as canonical local orchestration obligations, without widening into auto-fork, active-ephemeral task identity, Family-2 router execution, or a new public response surface.

Primary runtime story:

1. a retained worker emits `approval_request`, `fork_request`, or `fork_recommendation` during `continue_world_worker`,
2. Substrate validates exact identity, session, and world-binding truth before accepting the event,
3. Substrate evaluates the new deny-by-default notification/autonomy policy gates for those event classes,
4. accepted events become explicit local obligation-ledger records with stable kind, attention, and attach-eligibility semantics,
5. host posture and later auto-attach eligibility derive from those obligations rather than from hidden prompt injection,
6. accepted `fork_request` still requires a later explicit host-issued `fork_world_worker` action before any child worker is allocated.

Current landed runtime note:

1. `approval_request`, `approval_response`, `fork_request`, `fork_recommendation`, `fork_command`, `control_directive`, `control_ack`, and `attention_required` are still rejected as deferred wire labels in the `continue_world_worker` classifier,
2. the live local obligation ledger already has canonical kinds for `ApprovalRequired`, `ForkRequest`, and `ForkRecommendation`,
3. the live attach projection already treats `ApprovalRequired` and `ForkRequest` as router-auto-attach-eligible local kinds by default,
4. the live policy model still lacks dedicated worker-event permission keys for approval/fork request autonomy, exposing only the generic `agents.world_dispatch` action/mode/backend/boundary gates,
5. the live `RunWorldTaskOutcomeV1` still has no typed `task_run_id`, so active-ephemeral inspect/cancel widening remains separate later work.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing retained-worker event classification/runtime truth:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing local obligation-ledger and projection truth:
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- Existing policy surfaces expected to widen:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
- Existing attach projection/runtime surfaces expected to remain compatible:
  - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)

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
cargo test -p shell obligation -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - widen the retained-worker event contract to admit the first deferred approval/fork worker event classes
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - classify accepted worker events, apply new event-permission checks, and project accepted events into obligations without hidden prompt submission
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - keep the canonical durable obligation kinds, attention semantics, and attach-projection defaults aligned with the new worker-event acceptance path
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - persist deterministic obligation records and keep host-posture / attach-projection behavior authoritative
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
  - preserve exact session-scoped obligation ownership if runtime state helpers need widening
- `crates/shell/src/execution/policy_model.rs`
  - add minimal deny-by-default worker-event permission parsing for approval/fork autonomy
- `crates/broker/src/policy.rs`
  - parse and validate the corresponding policy keys so YAML truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics aligned with the new keys and denial buckets
- `crates/shell/tests/`
  - approval/fork worker-event classification, durable obligation persistence, auto-attach eligibility, and regression coverage
- `docs/CONFIGURATION.md`
  - document the new worker-event policy keys if they become repo truth in this slice
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, and durable state transitions that stay explicit and fail closed.

Preferred style:

```rust
if ContinueWorldWorkerEventClassV1::ForkRequest == event_class
    && !policy.world_dispatch_fork_requests_allowed()
{
    anyhow::bail!("fork_request_not_allowed: retained workers may not request fork under current policy");
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane and persistence boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every accepted worker event,
3. create durable obligations rather than synthesizing host prompts,
4. keep fork requests explicit and host-mediated,
5. keep worker recommendations distinct from worker requests,
6. keep Family-2 router execution, host-global ingress, and auto-fork out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for retained-worker event classification:
   - `approval_request` is accepted only when policy allows it,
   - `fork_request` is accepted only when policy allows it,
   - `fork_recommendation` is accepted only when policy allows it,
   - remaining deferred labels such as `approval_response`, `fork_command`, `control_directive`, `control_ack`, and `attention_required` still fail closed
2. unit tests for durable obligation mapping:
   - `approval_request` maps to `ApprovalRequired`,
   - `fork_request` maps to `ForkRequest`,
   - `fork_recommendation` maps to `ForkRecommendation`,
   - `attention_required`, `review_state`, `attach_state`, and world-binding fields stay explicit and reviewable
3. unit tests for policy parsing and denial buckets:
   - new worker-event permission keys are deny-by-default,
   - explanation-ready denials remain stable for disallowed approval/fork events,
   - allowed worker events do not weaken existing `agents.world_dispatch` action/mode/backend/boundary checks
4. state-store and projection tests:
   - accepted obligations become authoritative local session state,
   - `ApprovalRequired` and `ForkRequest` preserve existing local auto-attach eligibility,
   - `ForkRecommendation` remains reviewable but not auto-attach-eligible by default
5. integration/regression tests:
   - `continue_world_worker` can persist accepted approval/fork obligations without hidden prompt injection,
   - accepted `fork_request` does not allocate a child worker by itself,
   - later explicit `fork_world_worker` remains the only child-allocation path,
   - public status/control surfaces stay honest about the new durable obligation state without widening their public contract

Coverage expectations:

1. every newly accepted worker event class has exact-identity and policy-gating tests,
2. every persisted obligation class has deterministic summary/payload expectations,
3. denial paths remain explanation-ready,
4. accepted worker requests remain distinct from fulfilled host control-plane actions,
5. router/attach behavior remains local-projection-compatible without redesigning router ownership.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - create durable obligations, not synthetic host prompts
  - keep approval/fork worker requests exact-identity and fail-closed
  - require explicit host-issued `fork_world_worker` before any child allocation
  - preserve the currently landed local obligation-ledger and attach-state naming
- Ask first:
  - adding a public CLI or toolbox response surface for approval/fork handling
  - adding auto-fork, worker self-replication, or worker-issued `fork_command`
  - redesigning the attach-state vocabulary to match future greenfield queue naming
  - widening active-ephemeral inspect/cancel in the same slice
  - coupling the slice to Family-2 router/daemon execution or host-global inbox work
- Never:
  - auto-launch a child worker directly from a worker `fork_request`
  - treat `fork_recommendation` as equivalent to `fork_request`
  - mark an obligation resolved merely because attach processing succeeded
  - infer worker identity, session, or world binding heuristically
  - widen deferred labels like `approval_response`, `fork_command`, `control_directive`, or `control_ack` without a separate slice

## Success Criteria

Slice `39` is complete only when all of the following are true:

1. retained-worker `continue_world_worker` accepts typed `approval_request`, `fork_request`, and `fork_recommendation` events under exact identity and exact boundary truth,
2. the live policy surface exposes the minimum deny-by-default worker-event permission keys needed for those classes,
3. accepted worker events persist authoritative local obligations with stable kind, attention, review, attach, and world-binding semantics,
4. `ApprovalRequired` and `ForkRequest` integrate with the already-landed local attach projection without reopening router ownership,
5. `fork_request` remains host-mediated and does not allocate a child worker without a later explicit `fork_world_worker` action,
6. remaining deferred worker labels, active-ephemeral inspect/cancel widening, auto-fork, and Family-2 router execution all remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. typed host `approval_response` and richer control-directive/control-ack widening,
2. active-ephemeral exact task identity for inspect/cancel,
3. Family-2 router/daemon execution and host-global ingress,
4. any future auto-fork rubric.
