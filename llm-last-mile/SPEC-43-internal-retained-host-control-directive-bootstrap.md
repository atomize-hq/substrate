# Spec: Internal Retained Host Control Directive Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-42-internal-retained-host-clarification-response-bootstrap.md](./SPEC-42-internal-retained-host-clarification-response-bootstrap.md)
- [PLAN-42.md](./PLAN-42.md)
- [TASKS-42.md](./TASKS-42.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)  
Phase: `SPECIFY`  
Status: aligned with landed Slice `43` scope on `2026-06-04`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `42` is fully landed on the current tree, so typed host `clarification_response` is already the last landed host-to-worker widening on `continue_world_worker`.
2. The next honest Family-1 slice is not `progress_ack` first. `progress_ack` is smaller in meaning, but it currently has no exact durable causation anchor comparable to `ApprovalRequired` or `FollowUpRequired`, while `control_directive` can stand alone as an exact retained-worker instruction class.
3. Slice `43` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. To stay smaller than a transport redesign, typed `control_directive` should compile onto the existing `MemberTurnSubmitRequestV1.prompt` seam through a canonical renderer rather than widening `transport-api-types` in this slice.
5. The first typed `control_directive` slice should stay narrowly enumerated. The payload should carry:
   - a required `directive_kind`,
   - optional `directive_text` only as a caller-supplied bounded metadata label recognized for the chosen directive kind,
   - optional `thread_id`,
   - no new public control plane or generalized typed control envelope.
6. The minimum deny-by-default host-control gate for this slice lands as the dedicated `agents.world_dispatch.control.control_directives_allowed` policy key outside the obligation-bound keys.
7. `control_ack`, `fork_command`, `progress_ack`, active-ephemeral inspect/cancel widening, public control UX, and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Observed Repo Truth

The current repo already provides most of the floor this slice needs:

1. the live `continue_world_worker` request payload now accepts free-form prompt text plus three narrow typed host payloads, `approval_response`, `clarification_response`, and `control_directive`, on the same retained seam in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
2. the live runtime now proves the narrow host bootstrap pattern for all three landed typed host payloads: exact retained-worker targeting, deny-by-default typed payload gating, deterministic prompt rendering, and post-delivery side effects only after successful submit in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
3. the retained-worker messaging design already defines `control_directive` as a canonical host-to-worker operational class with examples such as pause, reduce scope, summarize, checkpoint, and prepare handoff,
4. the live repo now lands the narrow typed host `control_directive` bootstrap while still treating `control_ack` and `fork_command` as deferred typed surfaces, and the current request contract still rejects typed `worker_continue_control_directive`,
5. unlike approvals and follow-up clarification, `control_directive` does not need a durable unresolved-obligation consumer target to be meaningful, which makes it a cleaner next bootstrap than `progress_ack`.

That means the landed slice stays narrow: a typed host payload and dedicated gate, a small initial directive-kind surface, and reuse of the retained delivery seam without inventing broader operational transport.

## Objective

Build the first post-Slice-42 host operational steering slice so the host orchestrator can send typed `control_directive` messages to an exact retained worker through the existing `continue_world_worker` seam, with deny-by-default policy gating and deterministic implementation-owned prompt rendering, without widening into `control_ack`, `fork_command`, `progress_ack`, active-ephemeral task identity, Family-2 router execution, or a new public control UI.

Primary runtime story:

1. the host issues `continue_world_worker` with a typed `control_directive` payload for an exact retained worker,
2. Substrate validates exact caller, target, session, and world-binding truth,
3. Substrate policy-gates the typed control directive separately from generic `continue_world_worker`,
4. Substrate renders a deterministic control-directive prompt over the existing member-turn transport and submits it to the exact retained worker,
5. successful delivery returns a truthful outcome summary without pretending `control_ack` has landed,
6. no broader control transport, no obligation closeout model, and no child allocation semantics are introduced in this slice.

## Current Landed Runtime Note

1. the live `continue_world_worker` request payload now accepts free-form prompt text plus optional `thread_id`, typed `approval_response`, typed `clarification_response`, and typed `control_directive`,
2. `control_ack` and `fork_command` remain deferred typed surfaces even though the narrow host-side `control_directive` bootstrap is now landed,
3. the live transport submit seam still carries a single prompt string rather than a typed host-message envelope, so typed host control directives should compile onto that prompt seam through a canonical renderer,
4. the live repo now exposes a dedicated deny-by-default `agents.world_dispatch.control.control_directives_allowed` policy gate and an implementation-owned typed control-directive payload contract,
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

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - widen the `continue_world_worker` payload contract to represent typed host control directives without adding a new dispatch verb
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - validate control-directive payloads, render canonical prompt text, submit through the existing member-turn seam, and keep the outcome honest about delivery without implying `control_ack`
- `crates/shell/src/execution/policy_model.rs`
  - add the minimum deny-by-default policy parsing needed to gate host control directives explicitly
- `crates/broker/src/policy.rs`
  - parse and validate the corresponding policy keys so YAML truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics aligned with the new control-directive gate
- `docs/CONFIGURATION.md`
  - document the new control-directive policy key if it becomes repo truth in this slice
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, and deterministic side effects that happen only after authoritative success.

Preferred style:

```rust
if !policy.world_dispatch_control_directives_allowed() {
    anyhow::bail!(
        "control_directive_not_allowed: host orchestrators may not send typed control directives under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, policy, and delivery boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every typed control directive,
3. keep the initial directive taxonomy narrow and explicit rather than allowing arbitrary open-ended control kinds,
4. keep canonical prompt rendering deterministic and traceable rather than caller-authored ad hoc prompt assembly,
5. keep `control_ack`, `fork_command`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new `continue_world_worker` typed host-control contract:
   - typed `control_directive` payloads validate only with allowed directive kinds,
   - optional `directive_text` canonicalizes only to recognized bounded metadata labels for the selected directive kind and fails closed for blanks or free-form detail,
   - optional `thread_id` handling stays fail-closed for blanks,
   - generic prompt-based continue remains valid,
   - deferred host classes such as `control_ack` and `fork_command` still remain out of scope
2. unit tests for policy parsing and denial buckets:
   - the new control-directive policy key is deny-by-default,
   - explanation-ready denials remain stable for disallowed typed control directives,
   - existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact
3. dispatch and integration tests:
   - typed control directives render onto the existing member-turn submit seam deterministically,
   - successful delivery reports a truthful summary without implying later `control_ack`,
   - delivery failure returns explanation-ready errors,
   - public status/control surfaces remain honest without widening their public contract

Coverage expectations:

1. every typed control-directive path has exact-identity and policy-gating coverage,
2. directive rendering is deterministic and reviewable,
3. denial and delivery-failure paths are explanation-ready,
4. the slice does not widen into `control_ack`, `fork_command`, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - keep the initial `control_directive` kind set narrow and explicit
  - keep canonical prompt rendering deterministic and implementation-owned
  - keep delivery outcomes honest about the lack of typed `control_ack` in this slice
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of compiling onto the current prompt seam
  - adding typed `control_ack`, `fork_command`, or `progress_ack` in the same slice
  - adding a public CLI or toolbox surface for typed control directives
  - widening active-ephemeral inspect/cancel in the same slice
  - coupling the slice to Family-2 router/daemon execution or host-global inbox work
- Never:
  - imply that `control_ack` landed when only host-side directive delivery exists
  - treat a generic free-form continue prompt as equivalent to a typed `control_directive`
  - invent a broad arbitrary control language without explicit bounded directive kinds
  - widen into `control_ack`, `fork_command`, or active-ephemeral task identity without a separate slice

## Success Criteria

Slice `43` is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed `control_directive` host payload under exact identity and exact boundary truth,
2. the live policy surface exposes the minimum deny-by-default gate needed for typed host control directives,
3. typed control directives reuse the existing member-turn prompt transport without pretending a broader host-message transport has landed,
4. the initial directive-kind surface is narrow, deterministic, and implementation-owned,
5. successful delivery returns an honest typed-control outcome without implying `control_ack`,
6. `control_ack`, `fork_command`, `progress_ack`, active-ephemeral inspect/cancel widening, and Family-2 router execution all remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. typed worker `control_ack` bound to host `control_directive` causation,
2. typed host `fork_command`,
3. optional typed `progress_ack` if still needed after the operational steering slice lands,
4. active-ephemeral exact task identity for inspect/cancel,
5. any later end-to-end typed host-message transport redesign beyond the existing prompt seam.
