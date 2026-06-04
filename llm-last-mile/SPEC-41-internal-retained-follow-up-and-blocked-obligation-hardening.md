# Spec: Internal Retained Follow-Up And Blocked Obligation Hardening

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)
- [PLAN-39.md](./PLAN-39.md)
- [TASKS-39.md](./TASKS-39.md)
- [SPEC-40-internal-retained-host-approval-response-bootstrap.md](./SPEC-40-internal-retained-host-approval-response-bootstrap.md)
- [PLAN-40.md](./PLAN-40.md)
- [TASKS-40.md](./TASKS-40.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-03`
Landed posture note: retained-worker `follow_up_question` and `blocked` now have dedicated deny-by-default policy gates, exact durable `FollowUpRequired`/`Blocked` persistence on the existing `continue_world_worker` seam, and stable compatibility projection/attach semantics, but host `clarification_response`, broader control classes, active-ephemeral exact task identity, and Family-2 router/attach execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and no broader host-response/control, active-ephemeral identity, transport redesign, or Family-2 work was reopened.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `40` is landed on the current tree, so typed host `approval_response` is already the only post-Slice-39 host-to-worker widening that has gone live.
2. The next honest gap is producer-side, not another host-response slice: retained-worker `follow_up_question` and `blocked` events are already classified and attention-driving, and this slice closes their durable-obligation gap on the live `continue_world_worker` path.
3. This slice stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world messaging.
4. The canonical local obligation ledger remains the source of truth for this slice:
   - `OrchestrationObligationKind::FollowUpRequired` and `OrchestrationObligationKind::Blocked` already exist,
   - local attach-state naming remains the current landed runtime surface,
   - this slice must reuse those kinds rather than inventing a second pending-work shape.
5. The deny-by-default policy dimensions for this slice follow the existing design matrix and land as dedicated keys at `agents.world_dispatch.obligations.follow_up_allowed` and `agents.world_dispatch.obligations.blocked_allowed`.
6. `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, and Family-2 router/daemon execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. retained-worker `follow_up_question` and `blocked` are already accepted typed event classes in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs), and both are attention-driving by default,
2. the live `continue_world_worker` event classifier already preserves those event labels and typed payloads in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
3. the live policy surface now gates `approval_request`, `approval_response`, `follow_up_question`, `blocked`, `fork_request`, and `fork_recommendation`, so the retained-worker producer classes in this slice are deny-by-default behind dedicated permission keys,
4. the live `continue_world_worker` obligation projection now persists `ApprovalRequired`, `ForkRequest`, `ForkRecommendation`, `FollowUpRequired`, and `Blocked`, so accepted `follow_up_question` and `blocked` events now reach durable local obligation truth,
5. the canonical local obligation ledger already contains `FollowUpRequired` and `Blocked` kinds in [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs),
6. local compatibility projection and router auto-attach logic already know how to project and prioritize those kinds in [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) and [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs).

That means this slice closed a narrow producer gap by adding the missing policy gates and wiring the live producer path to reuse the already-landed durable obligation model.

## Objective

Build the first post-Slice-40 producer-hardening slice so retained-worker `follow_up_question` and `blocked` events become exact-identity, deny-by-default policy-gated, durable local obligations on the live `continue_world_worker` path, without widening into `clarification_response`, broader control directives, active-ephemeral task identity, or Family-2 router execution.

Primary runtime story:

1. a retained worker emits `follow_up_question` or `blocked` during `continue_world_worker`,
2. Substrate validates exact identity, session, and world-binding truth before accepting the event,
3. Substrate evaluates dedicated deny-by-default worker-event gates for `follow_up_question` and `blocked`,
4. accepted `follow_up_question` persists a canonical `FollowUpRequired` obligation,
5. accepted `blocked` persists a canonical `Blocked` obligation,
6. those obligations keep exact participant/backend/world binding and compatible attach/review projection semantics,
7. later host response work such as `clarification_response` remains a separate follow-on slice rather than being coupled into this producer hardening.

Current landed runtime note:

1. the live policy model now exposes dedicated deny-by-default retained-worker gates at `agents.world_dispatch.obligations.follow_up_allowed` and `agents.world_dispatch.obligations.blocked_allowed`,
2. the live retained-worker event path now persists accepted `follow_up_question` as `FollowUpRequired` and accepted `blocked` as `Blocked` with exact session/participant/backend/world truth,
3. denied `follow_up_question` and `blocked` events fail closed without leaving durable side effects,
4. the live repo still does not accept typed host `clarification_response`, broader host control classes, active-ephemeral exact task identity widening, or Family-2 router/attach execution in this slice.

## Frozen Direction

This spec freezes the following product and runtime direction:

1. no new dispatch verb is added,
2. no new public CLI or toolbox surface is added,
3. no typed host `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, or `fork_command` lands in this slice,
4. accepted `follow_up_question` must persist `FollowUpRequired` rather than staying prompt-only or inbox-only,
5. accepted `blocked` must persist `Blocked` rather than staying prompt-only or inbox-only,
6. both producer classes must be deny-by-default behind dedicated policy keys,
7. the slice must reuse the already-landed local ledger, compatibility projection, and attach-priority semantics rather than reopening Family 2 router ownership or queue design.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing retained-worker event/runtime truth:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing local obligation-ledger and projection truth:
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
- Existing policy surfaces expected to widen:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

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
cargo test -p shell policy_model -- --nocapture
cargo test -p shell obligation -- --nocapture
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - add dedicated policy enforcement for `follow_up_question` and `blocked`, and persist accepted events as canonical obligations on the live `continue_world_worker` path
- `crates/shell/src/execution/policy_model.rs`
  - add the minimum deny-by-default policy parsing needed for `follow_up_allowed` and `blocked_allowed`
- `crates/broker/src/policy.rs`
  - parse and validate the corresponding policy keys so YAML truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics aligned with the new worker-event gates
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - only if narrow helpers or tests need widening to keep `FollowUpRequired` and `Blocked` projection semantics explicit
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - only if compatibility projection or regression coverage needs a narrow patch
- `docs/CONFIGURATION.md`
  - document the new policy keys when they become repo truth
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed event classes, and durable side effects only after authoritative validation.

Preferred style:

```rust
if !policy.world_dispatch_follow_up_allowed() {
    anyhow::bail!(
        "follow_up_question_not_allowed: retained workers may not request host follow-up under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, policy, and persistence boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every persisted follow-up or blocked obligation,
3. reuse the already-landed `FollowUpRequired` and `Blocked` durable kinds rather than inventing a parallel queue or a new synthetic inbox dialect,
4. keep denial and persistence behavior explanation-ready and stable,
5. keep `clarification_response`, `control_directive`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new policy parsing and deny-by-default behavior:
   - `follow_up_allowed` and `blocked_allowed` default to `false`,
   - explicit enables are parsed and merged correctly,
   - explanation-ready denials remain stable
2. unit tests for `continue_world_worker` producer semantics:
   - accepted `follow_up_question` persists `FollowUpRequired`,
   - accepted `blocked` persists `Blocked`,
   - exact binding metadata and attention flags stay truthful,
   - disallowed events fail closed without persisting obligations
3. unit tests for compatibility projection:
   - `FollowUpRequired` remains compatible with follow-up inbox projection semantics,
   - `Blocked` remains compatible with runtime-alert style projection semantics,
   - attach-state and pending-attention behavior stay stable
4. integration/regression tests:
   - the live `continue_world_worker` path persists the new obligations,
   - approval/fork behavior from Slices `39` and `40` does not regress,
   - public status/control surfaces remain honest without widening their public contract

Coverage expectations:

1. both new policy keys have merge/default coverage,
2. both worker-event classes have durable producer-path coverage,
3. denial paths prove no hidden persistence side effect occurs,
4. the slice does not widen into host response/control classes or Family-2 execution.

## Boundaries

- Always:
  - keep the slice internal-only and producer-side
  - add dedicated deny-by-default gates for `follow_up_question` and `blocked`
  - persist accepted events as canonical `FollowUpRequired` and `Blocked` obligations on `continue_world_worker`
  - preserve exact worker/session/world binding in the durable record
  - reuse already-landed compatibility projection and attach semantics where possible
- Ask first:
  - landing `clarification_response` in the same slice
  - widening transport schemas or public CLI surfaces
  - changing router ownership or Family-2 queue/claim execution
  - widening active-ephemeral inspect/cancel in the same slice
  - adding new obligation kinds instead of reusing `FollowUpRequired` and `Blocked`
- Never:
  - treat accepted `follow_up_question` or `blocked` as prompt-only after this slice lands
  - persist these events without their dedicated policy gates
  - infer obligation identity from heuristic free text rather than the typed event class and exact session binding
  - widen into `clarification_response`, `control_directive`, `fork_command`, or active-ephemeral task identity without a separate slice

## Success Criteria

Slice `41` is complete only when all of the following are true:

1. the live policy surface exposes dedicated deny-by-default gates for retained-worker `follow_up_question` and `blocked`,
2. the `continue_world_worker` path fails closed when those events are disallowed,
3. accepted `follow_up_question` persists an exact `FollowUpRequired` obligation with stable compatibility projection semantics,
4. accepted `blocked` persists an exact `Blocked` obligation with stable compatibility projection semantics,
5. the durable records preserve exact worker/session/backend/world binding and truthful attention flags,
6. `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, and Family-2 router execution remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. typed host `clarification_response` bound to exact unresolved `FollowUpRequired` obligations,
2. broader host control/ack classes,
3. active-ephemeral exact task identity for inspect/cancel,
4. Family-2 router/daemon execution beyond the already-landed local compatibility projection.
