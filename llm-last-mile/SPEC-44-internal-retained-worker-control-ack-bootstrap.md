# Spec: Internal Retained Worker Control Ack Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-40-internal-retained-host-approval-response-bootstrap.md](./SPEC-40-internal-retained-host-approval-response-bootstrap.md)
- [PLAN-40.md](./PLAN-40.md)
- [TASKS-40.md](./TASKS-40.md)
- [SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md](./SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md)
- [PLAN-41.md](./PLAN-41.md)
- [TASKS-41.md](./TASKS-41.md)
- [SPEC-42-internal-retained-host-clarification-response-bootstrap.md](./SPEC-42-internal-retained-host-clarification-response-bootstrap.md)
- [PLAN-42.md](./PLAN-42.md)
- [TASKS-42.md](./TASKS-42.md)
- [SPEC-43-internal-retained-host-control-directive-bootstrap.md](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)
- [PLAN-43.md](./PLAN-43.md)
- [TASKS-43.md](./TASKS-43.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)  
Phase: `SPECIFY`  
Status: drafted on `2026-06-04`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `43` is fully landed on the current tree, so typed host `control_directive` is already the latest landed host-to-worker widening on `continue_world_worker`.
2. The next honest Family-1 slice is the paired worker-side acknowledgement for that already-landed control-directive seam: typed worker `control_ack` only, not host `fork_command` and not optional host `progress_ack`.
3. Slice `44` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. To stay smaller than a transport redesign, typed worker `control_ack` should be accepted only on the existing `continue_world_worker` stream and only as an immediate event bound to a just-sent typed host `control_directive`, rather than by widening `transport-api-types` into a new host/worker message envelope.
5. This slice should not introduce a new durable obligation kind or inbox projection:
   - `control_ack` is a quick acknowledgement, not an unresolved review item,
   - `control_ack` should remain non-attention-driving by default,
   - `control_ack` should not persist into the local obligation ledger.
6. The minimum policy posture for this slice should reuse the already-landed `agents.world_dispatch.control.control_directives_allowed` gate rather than adding a second dedicated `control_ack` allow switch:
   - the host must already be allowed to send the typed `control_directive`,
   - the worker-side `control_ack` is only accepted as immediate same-stream causation for that already-authorized directive.
7. `fork_command`, optional `progress_ack`, active-ephemeral inspect/cancel widening, public control UX, and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. the live `continue_world_worker` request payload already accepts the typed host `control_directive` payload on the same retained seam in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
2. the live runtime already proves the host-side control-directive delivery discipline: exact retained-worker targeting, deny-by-default host control gating, deterministic prompt rendering, and truthful delivery-only outcomes in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
3. the retained-worker messaging design already treats `control_ack` as the canonical worker-side acknowledgement paired to host `control_directive`,
4. the live `continue_world_worker` classifier still rejects `control_ack` as a deferred worker event label, which means the immediate acknowledgement half of the Slice `43` control loop is still unlanded,
5. the current obligation persistence path already distinguishes immediate events from durable obligation-producing events, so this slice can stay narrow by admitting `control_ack` without reopening the local ledger model.

That means the remaining gap is smaller than a new control class: the repo needs the immediate worker acknowledgement for the already-landed host control-directive seam, but it should land without inventing new durable state or broader transport.

## Objective

Build the first post-Slice-43 worker-acknowledgement slice so a retained worker may emit typed `control_ack` on the live `continue_world_worker` stream in response to an already-landed typed host `control_directive`, under exact retained-worker identity and exact same-stream causation truth, without widening into `fork_command`, optional `progress_ack`, a durable obligation kind, active-ephemeral task identity, Family-2 router execution, or a new public control UI.

Primary runtime story:

1. the host issues `continue_world_worker` with a typed `control_directive` payload for an exact retained worker,
2. Substrate validates exact caller, target, session, and world-binding truth and submits the deterministic control-directive prompt on the existing retained member-turn seam,
3. the targeted retained worker may emit typed `control_ack` on that same live stream,
4. Substrate accepts `control_ack` only when exact participant, backend, session, and world-binding truth still match and the originating request payload was the typed host `control_directive`,
5. Substrate surfaces the acknowledgement as an immediate live outcome on the existing `continue_world_worker` result path,
6. `control_ack` remains non-attention-driving, produces no durable obligation, and does not imply work completion,
7. `fork_command`, optional `progress_ack`, and broader typed host/worker message transport all remain deferred.

## Current Landed Runtime Note

1. the live `continue_world_worker` request payload now accepts free-form prompt text plus optional `thread_id`, typed `approval_response`, typed `clarification_response`, and typed `control_directive`,
2. the live retained-worker event path accepts `reply`, `progress_update`, `result`, `failure`, `follow_up_question`, `blocked`, `approval_request`, `fork_request`, and `fork_recommendation`,
3. the live retained-worker event path still rejects `control_ack` as a deferred worker event class even when a typed host `control_directive` was just delivered,
4. the live obligation persistence path already skips immediate non-obligation events, so `control_ack` can stay outside the durable ledger in this slice,
5. the live repo still has no typed active-ephemeral `task_run_id`, so active-ephemeral inspect/cancel widening remains separate later work.

## Frozen Direction

This spec freezes the following product and runtime direction:

1. `control_ack` lands only as a worker-to-host event on the existing `continue_world_worker` stream,
2. `control_ack` is accepted only as same-stream acknowledgement of a typed host `control_directive`,
3. `control_ack` remains immediate-channel only and does not become a durable obligation kind,
4. `control_ack` stays non-attention-driving by default unless a future slice explicitly reopens that posture,
5. the slice reuses the existing host control gate rather than inventing a second independent policy switch for the acknowledgement,
6. the slice must not imply `fork_command`, optional `progress_ack`, or generalized control transport.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing retained-worker event/runtime truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing policy/config truth expected to document the slice:
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
- Existing world-member submit transport reused without schema widening in this slice:
  - [`crates/transport-api-types/src/lib.rs`](../crates/transport-api-types/src/lib.rs)
  - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

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
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - widen the retained-worker event contract to admit `control_ack` as an in-scope immediate worker event and remove it from the deferred-worker list
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - validate `control_ack` same-stream causation, preserve exact retained-worker identity checks, keep acknowledgement non-durable, and report truthful live outcome semantics
- `docs/CONFIGURATION.md`
  - document that the existing `control_directives_allowed` gate now admits the paired immediate `control_ack` worker acknowledgement and still does not imply `fork_command`, broader control transport, or durable control obligations
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, deterministic side effects, and narrow typed contracts.

Preferred style:

```rust
if matches!(event_class, ContinueWorldWorkerEventClassV1::ControlAck)
    && !matches!(
        prepared.request.payload,
        WorldDispatchPayloadV1::WorkerContinueControlDirective(_)
    )
{
    anyhow::bail!(
        "unsupported_worker_event_class: continue_world_worker accepts control_ack only for typed control_directive delivery"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, classification, and delivery boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for `control_ack`,
3. bind `control_ack` to the same live `continue_world_worker` stream that carried the typed host `control_directive`,
4. keep acknowledgement semantics truthful: accepted directive receipt or application acknowledgement only, not task completion,
5. do not persist `control_ack` into the local obligation ledger,
6. keep `fork_command`, optional `progress_ack`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the retained-worker event contract:
   - `control_ack` is accepted as an in-scope worker event class,
   - `control_ack` is non-attention-driving by default,
   - `control_ack` is removed from deferred-worker rejection coverage,
   - deferred worker labels such as `fork_command`, host `control_directive`, and `attention_required` remain out of scope
2. dispatch and classification tests:
   - `control_ack` is accepted only when the originating `continue_world_worker` request payload is the typed host `control_directive`,
   - `control_ack` fails closed for generic prompt-based continue, typed `approval_response`, or typed `clarification_response`,
   - exact participant/backend/session/world binding rules remain identical to the rest of the retained-worker event seam
3. live delivery/outcome tests:
   - a typed host `control_directive` may receive `control_ack` on the same live stream,
   - the outcome remains honest about acknowledgement versus work completion,
   - `control_ack` leaves no durable obligation side effects,
   - approval/clarification/control-directive behavior from prior slices remains green

Coverage expectations:

1. every accepted `control_ack` path has exact-identity and exact-causation coverage,
2. `control_ack` remains immediate-only and non-durable,
3. denial paths are explanation-ready,
4. the slice does not widen into `fork_command`, optional `progress_ack`, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - accept `control_ack` only as same-stream acknowledgement of a typed host `control_directive`
  - keep `control_ack` non-attention-driving and non-durable in this slice
  - keep outcome summaries honest about acknowledgement versus completion
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of staying on the current prompt seam
  - adding a new dedicated `control_ack` policy key instead of reusing `control_directives_allowed`
  - adding typed `fork_command` or optional `progress_ack` in the same slice
  - persisting `control_ack` into the obligation ledger or inbox projection
  - widening active-ephemeral inspect/cancel in the same slice
- Never:
  - accept `control_ack` on a generic or non-control-directive `continue_world_worker` turn
  - imply that `control_ack` means the retained worker completed the requested work
  - persist `control_ack` as if it were an unresolved obligation
  - widen into `fork_command`, optional `progress_ack`, or active-ephemeral task identity without a separate slice

## Success Criteria

Slice `44` is complete only when all of the following are true:

1. the retained-worker event contract accepts typed `control_ack` under exact identity and exact boundary truth,
2. `control_ack` is accepted only on a typed host `control_directive` `continue_world_worker` turn and fails closed otherwise,
3. successful typed host `control_directive` delivery may surface an immediate `control_ack` without implying broader control transport,
4. `control_ack` remains non-attention-driving and produces no durable obligation side effects,
5. delivery summaries and public projection remain honest about acknowledgement versus completion,
6. `fork_command`, optional `progress_ack`, active-ephemeral inspect/cancel widening, and Family-2 router execution all remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. typed host `fork_command`,
2. optional typed host `progress_ack`,
3. any later durable treatment of control acknowledgements beyond the immediate stream,
4. active-ephemeral exact task identity for inspect/cancel,
5. any later end-to-end typed host/worker message transport redesign beyond the existing prompt seam.
