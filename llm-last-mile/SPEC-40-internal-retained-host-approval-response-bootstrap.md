# Spec: Internal Retained Host Approval Response Bootstrap

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slice:
- [SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)
- [PLAN-39.md](./PLAN-39.md)
- [TASKS-39.md](./TASKS-39.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-03`
Landed posture note: typed host `approval_response` delivery over the existing `continue_world_worker` seam, deny-by-default approval-response policy gating, and exact post-delivery approval-obligation closeout are landed in the repo, but broader host control/fork directives, active-ephemeral exact task-identity widening, public approval UX, and Family-2 router/attach execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and no broader host-response/control, active-ephemeral identity, transport redesign, or Family-2 work was reopened.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `39` is fully landed on the current tree, so retained workers may already emit typed `approval_request`, `fork_request`, and `fork_recommendation` events that persist durable local obligations.
2. The next honest Family-1 slice is the first host-to-worker response widening, not another dispatch verb and not Family-2 router execution.
3. Slice `40` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. The narrowest honest host-response slice is `approval_response` only:
   - generic instruction-style follow-up remains on the existing free-form prompt path,
   - `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, and `fork_command` remain later work,
   - `fork_world_worker` remains the only child-allocation action.
5. To stay smaller than a transport redesign, typed `approval_response` should compile onto the existing `MemberTurnSubmitRequestV1.prompt` seam through a canonical renderer rather than widening `transport-api-types` in this slice.
6. A typed approval response should consume an existing unresolved `ApprovalRequired` obligation explicitly:
   - the payload should bind to exact approval causation such as `approval_obligation_id`,
   - successful delivery of an approve-style response should resolve that obligation,
   - successful delivery of a deny-style response should dismiss that obligation,
   - failure before delivery should leave the obligation unresolved.
7. Family 2 router/attach execution remains downstream. Slice `40` may reuse the local obligation ledger, but it must not redesign router ownership, attach processing, or host-global ingress.

If any of these are wrong, correct them before implementation.

## Objective

Build the first post-Slice-39 Family-1 host-response widening slice so the host orchestrator can send typed `approval_response` messages to an exact retained worker through the existing `continue_world_worker` seam, while explicitly consuming the matching `ApprovalRequired` obligation, without widening into control directives, fork commands, active-ephemeral task identity, Family-2 router execution, or a new public approval UI.

Primary runtime story:

1. a retained worker emits `approval_request`,
2. Substrate persists an `ApprovalRequired` obligation exactly as Slice `39` already defines,
3. the host later issues `continue_world_worker` with a typed `approval_response` payload bound to that exact unresolved obligation,
4. Substrate validates exact caller, target, session, and world-binding truth plus exact unresolved approval-obligation ownership,
5. Substrate policy-gates the typed approval response separately from generic `continue_world_worker`,
6. Substrate renders a deterministic approval-response prompt over the existing member-turn transport and submits it to the exact retained worker,
7. only after successful submission does Substrate resolve or dismiss the matching approval obligation,
8. broader control/fork host directives remain deferred.

Current landed runtime note:

1. the live `continue_world_worker` request payload now accepts either free-form prompt text plus optional `thread_id` or a typed `approval_response` payload bound to exact approval causation,
2. the live retained-worker event classifier still treats worker-originated `approval_response`, `fork_command`, `control_directive`, and `control_ack` as deferred wire labels,
3. the live transport submit seam still carries a single prompt string rather than a typed host-message envelope, so typed host approval responses compile onto that prompt seam through a canonical renderer,
4. the live policy model now exposes a dedicated deny-by-default host approval-response gate at `agents.world_dispatch.obligations.approval_response_allowed`, and the live obligation ledger closes matching `ApprovalRequired` records only after successful delivery,
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
  - widen the `continue_world_worker` payload contract to represent typed host approval responses without adding a new dispatch verb
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - validate approval-response targeting, render the canonical prompt, submit it through the existing member-turn seam, and sequence obligation closeout only after successful delivery
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - resolve or dismiss exact `ApprovalRequired` obligations through sanctioned control-plane action rather than ad hoc file mutation
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - keep review-state and resolved-at semantics aligned with typed approval-response consumption
- `crates/shell/src/execution/policy_model.rs`
  - add the minimum deny-by-default policy parsing needed to gate host approval responses explicitly
- `crates/broker/src/policy.rs`
  - parse and validate the corresponding policy keys so YAML truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics aligned with the new approval-response gate
- `docs/CONFIGURATION.md`
  - document the new approval-response policy key if it becomes repo truth in this slice
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, and deterministic side effects that happen only after authoritative success.

Preferred style:

```rust
if !policy.world_dispatch_approval_responses_allowed() {
    anyhow::bail!(
        "approval_response_not_allowed: host orchestrators may not send typed approval responses under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, persistence, and delivery boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every typed approval response,
3. resolve or dismiss approval obligations only after successful delivery into the live retained-worker turn seam,
4. keep canonical prompt rendering deterministic and traceable rather than caller-authored ad hoc prompt assembly,
5. keep `fork_command`, `control_directive`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new `continue_world_worker` typed host-response contract:
   - typed `approval_response` payloads validate only with exact approval-causation binding,
   - generic prompt-based continue remains valid,
   - deferred host classes such as `control_directive` and `fork_command` still remain out of scope
2. unit tests for obligation-consumer semantics:
   - approve-style responses resolve the matching `ApprovalRequired` obligation,
   - deny-style responses dismiss the matching `ApprovalRequired` obligation,
   - unresolved, missing, cross-session, or non-approval obligations fail closed
3. unit tests for policy parsing and denial buckets:
   - the new approval-response policy key is deny-by-default,
   - explanation-ready denials remain stable for disallowed typed approval responses,
   - existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact
4. dispatch and integration tests:
   - typed approval responses render onto the existing member-turn submit seam deterministically,
   - obligation resolution happens only after successful delivery,
   - delivery failure leaves the obligation unresolved,
   - public status/control surfaces stay honest about the resolved or dismissed obligation state without widening their public contract

Coverage expectations:

1. every typed approval-response path has exact-identity and policy-gating coverage,
2. approval-obligation closeout is deterministic and reviewable,
3. denial and delivery-failure paths are explanation-ready,
4. the slice does not widen into generic control directives, fork commands, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - require exact unresolved approval-obligation binding before sending a typed approval response
  - resolve or dismiss approval obligations only after successful delivery into the retained-worker seam
  - keep canonical prompt rendering deterministic and implementation-owned
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of compiling onto the current prompt seam
  - adding typed `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, or `fork_command` in the same slice
  - adding a public CLI or toolbox surface for approval review/response
  - widening active-ephemeral inspect/cancel in the same slice
  - coupling the slice to Family-2 router/daemon execution or host-global inbox work
- Never:
  - resolve an approval obligation before the typed response is actually delivered
  - treat a generic free-form continue prompt as equivalent to a typed `approval_response`
  - silently auto-fork or allocate a child worker from host approval handling
  - infer approval-obligation identity heuristically from prompt text alone
  - widen into `control_directive`, `fork_command`, or active-ephemeral task identity without a separate slice

## Success Criteria

Slice `40` is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed `approval_response` host payload under exact identity and exact boundary truth,
2. the live policy surface exposes the minimum deny-by-default gate needed for typed host approval responses,
3. a typed approval response must bind to an exact unresolved `ApprovalRequired` obligation and fails closed otherwise,
4. successful approval-response delivery resolves or dismisses the matching approval obligation deterministically,
5. typed approval responses reuse the existing member-turn prompt transport without pretending a broader host-message transport has landed,
6. `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, and Family-2 router execution all remain explicitly out of scope.

## Open Questions

None for this slice. The remaining broader questions are intentionally deferred:

1. broader typed host-to-worker classes such as `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, and `fork_command`,
2. active-ephemeral exact task identity for inspect/cancel,
3. Family-2 router/daemon execution and host-global ingress,
4. any later end-to-end typed host-message transport redesign beyond the existing prompt seam.
