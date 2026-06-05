# Spec: Internal Retained Host Progress Ack Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-43-internal-retained-host-control-directive-bootstrap.md](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)
- [PLAN-43.md](./PLAN-43.md)
- [TASKS-43.md](./TASKS-43.md)
- [SPEC-44-internal-retained-worker-control-ack-bootstrap.md](./SPEC-44-internal-retained-worker-control-ack-bootstrap.md)
- [PLAN-44.md](./PLAN-44.md)
- [TASKS-44.md](./TASKS-44.md)
- [SPEC-45-internal-retained-host-fork-command-bootstrap.md](./SPEC-45-internal-retained-host-fork-command-bootstrap.md)
- [PLAN-45.md](./PLAN-45.md)
- [TASKS-45.md](./TASKS-45.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)  
Phase: `SPECIFY`  
Status: drafted on `2026-06-05`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `45` is fully landed on the current tree, so typed host `fork_command` is now the latest landed host-to-worker widening on `continue_world_worker`.
2. The next honest Family-1 slice is optional typed host `progress_ack`:
   - the design stack already defines `progress_ack` as a host-to-worker message class paired with worker `progress_update`,
   - `progress_update` is already a live retained-worker event class on the current tree,
   - active-ephemeral exact task identity and Family-2 router execution still require broader widening than this delivery-only acknowledgement slice.
3. Slice `46` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. To stay smaller than a transport redesign, typed `progress_ack` should compile onto the existing `MemberTurnSubmitRequestV1.prompt` seam through a canonical renderer rather than widening `transport-api-types` in this slice.
5. The first typed `progress_ack` slice should stay narrow and delivery-only:
   - it targets one exact retained worker,
   - it may carry optional `thread_id` continuity only,
   - it does not consume a durable obligation,
   - it does not imply completion, pause, or fork approval.
6. Because the live normalized `ContinueWorldWorkerEventV1` surface does not expose a first-class progress-event identifier, the initial `progress_ack` slice should not invent an exact durable causation model; exact retained-worker identity plus optional thread continuity is the narrowest honest anchor on the current tree.
7. The minimum deny-by-default host-progress gate for this slice should land at `agents.world_dispatch.control.progress_acks_allowed`, parallel to the existing `control_directives_allowed` surface but separate from obligation and fork policy keys.
8. Active-ephemeral inspect/cancel widening and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. the live `continue_world_worker` request payload already accepts free-form prompt text plus the narrow typed host payloads `approval_response`, `clarification_response`, `control_directive`, and `fork_command` on the same retained seam in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
2. the live retained-worker event path already accepts `progress_update` as an in-scope worker event class and treats it as non-attention-driving by default in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
3. the live runtime already proves the host-side typed payload pattern for that seam: exact retained-worker targeting, deny-by-default typed payload gating, deterministic implementation-owned rendering, and truthful delivery semantics in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
4. the current request contract still rejects typed `worker_continue_progress_ack`, so the host cannot yet send a narrow typed acknowledgement back over the live retained seam,
5. the live policy model already distinguishes obligation-bound host responses, control directives, and fork commands with separate deny-by-default gates, but it still exposes no dedicated host `progress_ack` gate.

That means the remaining gap is narrower than active-ephemeral identity work and smaller than any transport redesign: the repo needs one optional typed host `progress_ack` class on the existing retained continue seam, but it should land as a delivery-only acknowledgement rather than inventing a durable progress consumer model.

## Objective

Build the first post-Slice-45 optional host acknowledgement slice so the host orchestrator can send typed `progress_ack` messages to an exact retained worker through the existing `continue_world_worker` seam, with deny-by-default progress-ack policy gating, deterministic implementation-owned prompt rendering, and truthful delivery-only outcomes, without widening into durable progress obligations, control directives, fork commands, active-ephemeral task identity, Family-2 router execution, or a new public progress UI.

Primary runtime story:

1. a retained worker emits `progress_update` on the existing live `continue_world_worker` stream,
2. the host later issues `continue_world_worker` with a typed `progress_ack` payload for that exact retained worker,
3. Substrate validates exact caller, target, session, backend, and world-binding truth,
4. Substrate policy-gates typed host `progress_ack` separately from generic `continue_world_worker`, obligation-bound responses, typed `control_directive`, and typed `fork_command`,
5. Substrate renders a deterministic progress-ack prompt over the existing member-turn transport and submits it to the exact retained worker,
6. successful delivery returns a truthful delivery-only summary without implying durable progress closeout, task completion, pause, or child allocation,
7. `progress_ack` remains optional, internal-only, and non-durable.

## Current Landed Runtime Note

1. the live `continue_world_worker` request payload now accepts free-form prompt text plus optional `thread_id`, typed `approval_response`, typed `clarification_response`, typed `control_directive`, and typed `fork_command`,
2. the live retained-worker event path already accepts `progress_update` while still rejecting typed host `worker_continue_progress_ack`,
3. the live transport submit seam still carries a single prompt string rather than a typed host-message envelope, so typed host progress acknowledgements should compile onto that prompt seam through a canonical renderer,
4. the live repo already distinguishes durable obligation consumers from delivery-only typed host messages, and `progress_update` itself is non-attention-driving by default,
5. the live repo still has no typed active-ephemeral `task_run_id`, so active-ephemeral inspect/cancel widening remains separate later work.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing internal dispatch/runtime truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing policy surfaces expected to widen:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
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
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `src/`
  - thin top-level CLI entrypoints for `substrate` and `substrate-shim`
- `crates/shell/src/execution/agent_runtime/`
  - shared retained-worker dispatch contracts and runtime message/outcome types
  - Slice `46` typed payload and render-contract changes belong here
- `crates/shell/src/execution/`
  - shell-side world-dispatch orchestration and policy parsing
  - Slice `46` delivery routing, summary wording, and shell-local gate enforcement belong here
- `crates/shell/tests/`
  - integration and regression suites for shell/runtime behavior
  - retained-worker continue regressions for this slice should live here when they need end-to-end delivery proof
- `crates/broker/src/`
  - YAML policy parsing and effective-policy diagnostics used to keep broker truth aligned with shell enforcement
- `crates/transport-api-types/src/`
  - shared transport request/response schema
  - this slice is expected to reuse this structure without widening it
- `crates/world-service/src/`
  - in-world execution and member-turn handling behind the existing transport seam
  - this slice should consume the existing seam rather than redesign it
- `docs/`
  - operator-facing and configuration documentation
  - `docs/CONFIGURATION.md` is the expected documentation surface if a new host progress-ack gate becomes repo truth
- `llm-last-mile/`
  - repo-local planning/design/spec artifacts for the retained-worker orchestration track
  - this spec, its plan, and its tasks belong here and must stay aligned

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, deterministic side effects, and narrow delivery semantics rather than implied workflow changes.

Preferred style:

```rust
if !policy.world_dispatch_progress_acks_allowed() {
    anyhow::bail!(
        "progress_ack_not_allowed: host orchestrators may not send typed progress acknowledgements under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch and policy boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every typed host `progress_ack`,
3. keep `progress_ack` distinct from both generic prompt-based continue and typed `control_directive`:
   - generic continue remains free-form conversational follow-up,
   - typed `control_directive` remains operational steering,
   - typed `progress_ack` is only a narrow host acknowledgement that progress was seen,
4. keep outcome summaries explicit that progress acknowledgement is delivery-only and does not imply completion or durable closeout,
5. keep active-ephemeral identity, fork autonomy, and Family-2 routing out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new `continue_world_worker` typed host-progress-ack contract:
   - typed `progress_ack` payloads validate with the required exact retained-worker target and optional non-empty `thread_id`,
   - generic prompt-based continue remains valid,
   - deferred host classes outside the landed set remain fail-closed
2. unit tests for policy parsing and denial buckets:
   - the new host progress-ack policy key is deny-by-default,
   - explanation-ready denials remain stable for disallowed typed host `progress_ack`,
   - existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact
3. dispatch and integration tests:
   - accepted typed host `progress_ack` renders onto the existing member-turn submit seam deterministically,
   - exact retained-worker targeting remains mandatory,
   - successful delivery summaries remain acknowledgement-only and do not imply completion, pause, or fork handling
4. regression tests:
   - `progress_update` remains non-attention-driving by default,
   - typed host `progress_ack` does not create durable obligations or mutate existing approval/follow-up/fork control flow,
   - active-ephemeral inspect/cancel still remains deferred

Coverage expectations:

1. every typed host `progress_ack` path has exact-identity and policy-gating coverage,
2. progress acknowledgement stays delivery-only and non-durable,
3. denial and delivery-failure paths remain explanation-ready,
4. the slice does not widen into active-ephemeral identity, fork/control redesign, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - keep typed host `progress_ack` delivery-only and fail closed
  - keep summaries honest that acknowledgement does not mean completion
  - preserve exact retained-worker identity and optional thread continuity when provided
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of compiling onto the current prompt seam
  - adding a durable progress obligation, inbox projection, or post-delivery closeout model in the same slice
  - requiring a brand-new exact event-id causation surface before landing the narrow typed acknowledgement
  - adding public CLI or toolbox surface for typed host progress acknowledgements
  - coupling the slice to active-ephemeral inspect/cancel widening or Family-2 router execution
- Never:
  - treat a generic free-form continue prompt as equivalent to typed `progress_ack`
  - imply that sending `progress_ack` means the worker completed, paused, or received new scope
  - widen into control directives, fork commands, or active-ephemeral identity without a separate slice
  - silently create durable obligation state from a typed host `progress_ack`

## Success Criteria

Slice `46` is complete only when all of the following are true:

1. `cargo test -p shell dispatch_contract -- --nocapture` proves `continue_world_worker` accepts a typed host `progress_ack` payload under exact retained-worker identity and optional thread continuity while generic prompt-based continue still works.
2. `cargo test -p shell policy_model -- --nocapture` and `cargo test -p substrate-broker -- --nocapture` prove `agents.world_dispatch.control.progress_acks_allowed` exists as the minimum host progress-ack gate, defaults to deny, and returns explanation-ready denials when disallowed.
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` proves accepted typed host `progress_ack` messages render deterministically onto the existing retained member-turn prompt seam and return delivery-only summaries without requiring a `transport-api-types` or `world-service` schema change in this slice.
4. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture` plus the targeted shell suites above prove typed host `progress_ack` creates no durable obligation side effects, does not imply completion, pause, or fork handling, and keeps active-ephemeral inspect/cancel behavior deferred.

## Open Questions

1. Whether a later slice should add an exact host-visible `progress_update` event identifier or `causation_message_id` surface before any richer progress-review workflow lands. Default assumption for this bootstrap slice: no, because the current repo only needs a narrow optional acknowledgement over the existing seam, not a broader typed host-message transport redesign.
