# Spec: Internal Retained Host Clarification Response Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-40-internal-retained-host-approval-response-bootstrap.md](./SPEC-40-internal-retained-host-approval-response-bootstrap.md)
- [PLAN-40.md](./PLAN-40.md)
- [TASKS-40.md](./TASKS-40.md)
- [SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md](./SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md)
- [PLAN-41.md](./PLAN-41.md)
- [TASKS-41.md](./TASKS-41.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-04`
Landed posture note: typed host `clarification_response` now has a dedicated deny-by-default gate, exact unresolved `FollowUpRequired` binding, deterministic prompt rendering over the existing `continue_world_worker` seam, and post-delivery follow-up closeout, while broader host response/control classes, active-ephemeral exact task identity, and Family-2 router/attach execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and no broader host-response/control, active-ephemeral identity, transport redesign, or Family-2 work was reopened.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `41` is fully landed on the current tree, so retained-worker `follow_up_question` now persists canonical `FollowUpRequired` obligations with exact session/participant/backend/world truth.
2. The next honest Family-1 slice is the narrow host-side consumer for those obligations: typed host `clarification_response` only, not broader control directives and not Family-2 router execution.
3. Slice `42` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. To stay smaller than a transport redesign, typed `clarification_response` should compile onto the existing `MemberTurnSubmitRequestV1.prompt` seam through a canonical renderer rather than widening `transport-api-types` in this slice.
5. A typed clarification response should consume an existing unresolved `FollowUpRequired` obligation explicitly:
   - the payload should bind to exact follow-up causation such as `follow_up_obligation_id`,
   - the payload should carry non-empty clarification text,
   - successful delivery should resolve that matching follow-up obligation,
   - failure before delivery should leave the obligation unresolved.
6. The minimum deny-by-default host-response gate for this slice should land at `agents.world_dispatch.obligations.clarification_response_allowed`, mirroring the repo-local `approval_response_allowed` pattern from Slice `40`.
7. `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, public clarification UX, and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. the live `continue_world_worker` request payload already accepts either free-form prompt text or the narrow typed `approval_response` host payload on the same retained seam in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
2. the live runtime already proves the required host-response ordering discipline: exact obligation binding before delivery, deterministic prompt rendering, and post-delivery closeout only after successful submit in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) and [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs),
3. the retained-worker messaging design already treats `clarification_response` as a canonical host-to-worker conversational class paired with worker `follow_up_question`,
4. the live repo now persists `FollowUpRequired` obligations durably from accepted worker `follow_up_question` events, so the exact consumer-side target for this slice already exists,
5. the live repo now accepts a typed host `clarification_response` payload, enforces a dedicated deny-by-default clarification-response gate, and closes the matching follow-up obligation only after successful host delivery.

That means this slice closed a narrow consumer gap by adding the typed host payload and gate, binding it to exact unresolved `FollowUpRequired`, and reusing the already-landed retained delivery seam plus post-delivery closeout discipline.

## Objective

Build the first post-Slice-41 host follow-up consumer slice so the host orchestrator can send typed `clarification_response` messages to an exact retained worker through the existing `continue_world_worker` seam, while explicitly consuming the matching unresolved `FollowUpRequired` obligation, without widening into broader control directives, active-ephemeral task identity, Family-2 router execution, or a new public clarification UI.

Primary runtime story:

1. a retained worker emits `follow_up_question`,
2. Substrate persists a `FollowUpRequired` obligation exactly as Slice `41` already defines,
3. the host later issues `continue_world_worker` with a typed `clarification_response` payload bound to that exact unresolved obligation,
4. Substrate validates exact caller, target, session, and world-binding truth plus exact unresolved follow-up-obligation ownership,
5. Substrate policy-gates the typed clarification response separately from generic `continue_world_worker`,
6. Substrate renders a deterministic clarification-response prompt over the existing member-turn transport and submits it to the exact retained worker,
7. only after successful submission does Substrate resolve the matching follow-up obligation,
8. broader host control/ack/fork directives remain deferred.

## Current Landed Runtime Note

1. the live `continue_world_worker` request payload now accepts either free-form prompt text plus optional `thread_id` or a typed `approval_response` payload bound to exact approval causation,
2. the live retained-worker event path now persists `FollowUpRequired` durably for accepted `follow_up_question`,
3. the live transport submit seam still carries a single prompt string rather than a typed host-message envelope, so typed host clarification responses should compile onto that prompt seam through a canonical renderer,
4. the live policy model now exposes a dedicated clarification-response gate, and the live state store now closes the matching follow-up obligation only after successful typed host clarification delivery,
5. the live repo still has no typed active-ephemeral `task_run_id`, so active-ephemeral inspect/cancel widening remains separate later work.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing internal dispatch/runtime truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing local obligation consumer/state truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
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
cargo test -p shell state_store -- --nocapture
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
  - widen the `continue_world_worker` payload contract to represent typed host clarification responses without adding a new dispatch verb
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - validate clarification-response targeting, render the canonical prompt, submit it through the existing member-turn seam, and sequence follow-up-obligation closeout only after successful delivery
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - resolve exact `FollowUpRequired` obligations through sanctioned control-plane action rather than ad hoc file mutation
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - keep follow-up resolution semantics aligned with typed clarification-response consumption if narrow helper widening is required
- `crates/shell/src/execution/policy_model.rs`
  - add the minimum deny-by-default policy parsing needed to gate host clarification responses explicitly
- `crates/broker/src/policy.rs`
  - parse and validate the corresponding policy keys so YAML truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics aligned with the new clarification-response gate
- `docs/CONFIGURATION.md`
  - document the new clarification-response policy key if it becomes repo truth in this slice
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, and deterministic side effects that happen only after authoritative success.

Preferred style:

```rust
if !policy.world_dispatch_clarification_responses_allowed() {
    anyhow::bail!(
        "clarification_response_not_allowed: host orchestrators may not send typed clarification responses under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, persistence, and delivery boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every typed clarification response,
3. require exact unresolved `FollowUpRequired` binding before sending a typed clarification response,
4. resolve follow-up obligations only after successful delivery into the live retained-worker turn seam,
5. keep canonical prompt rendering deterministic and traceable rather than caller-authored ad hoc prompt assembly,
6. keep `control_directive`, `fork_command`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new `continue_world_worker` typed host-response contract:
   - typed `clarification_response` payloads validate only with exact follow-up causation binding and non-empty clarification text,
   - generic prompt-based continue remains valid,
   - deferred host classes such as `control_directive` and `fork_command` still remain out of scope
2. unit tests for follow-up-obligation consumer semantics:
   - successful clarification responses resolve the matching `FollowUpRequired` obligation,
   - missing, resolved, cross-session, wrong-kind, or boundary-drifted obligations fail closed,
   - unresolved obligations remain unchanged when delivery has not occurred
3. unit tests for policy parsing and denial buckets:
   - the new clarification-response policy key is deny-by-default,
   - explanation-ready denials remain stable for disallowed typed clarification responses,
   - existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact
4. dispatch and integration tests:
   - typed clarification responses render onto the existing member-turn submit seam deterministically,
   - obligation resolution happens only after successful delivery,
   - delivery failure leaves the follow-up obligation unresolved,
   - public status/control surfaces stay honest about resolved follow-up state without widening their public contract

Coverage expectations:

1. every typed clarification-response path has exact-identity and policy-gating coverage,
2. follow-up-obligation closeout is deterministic and reviewable,
3. denial and delivery-failure paths are explanation-ready,
4. the slice does not widen into generic control directives, fork commands, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - require exact unresolved `FollowUpRequired` binding before sending a typed clarification response
  - resolve follow-up obligations only after successful delivery into the retained-worker seam
  - keep canonical prompt rendering deterministic and implementation-owned
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of compiling onto the current prompt seam
  - adding typed `progress_ack`, `control_directive`, `control_ack`, or `fork_command` in the same slice
  - adding a public CLI or toolbox surface for follow-up review/response
  - widening active-ephemeral inspect/cancel in the same slice
  - coupling the slice to Family-2 router/daemon execution or host-global inbox work
- Never:
  - resolve a follow-up obligation before the typed clarification response is actually delivered
  - treat a generic free-form continue prompt as equivalent to a typed `clarification_response`
  - infer follow-up-obligation identity heuristically from prompt text alone
  - widen into `control_directive`, `fork_command`, or active-ephemeral task identity without a separate slice

## Success Criteria

Slice `42` is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed `clarification_response` host payload under exact identity and exact boundary truth,
2. the live policy surface exposes the minimum deny-by-default gate needed for typed host clarification responses,
3. a typed clarification response must bind to an exact unresolved `FollowUpRequired` obligation and fails closed otherwise,
4. successful clarification-response delivery resolves the matching follow-up obligation deterministically,
5. typed clarification responses reuse the existing member-turn prompt transport without pretending a broader host-message transport has landed,
6. `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, and Family-2 router execution all remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. broader typed host-to-worker classes such as `progress_ack`, `control_directive`, `control_ack`, and `fork_command`,
2. active-ephemeral exact task identity for inspect/cancel,
3. Family-2 router/daemon execution and host-global ingress,
4. any later end-to-end typed host-message transport redesign beyond the existing prompt seam.
