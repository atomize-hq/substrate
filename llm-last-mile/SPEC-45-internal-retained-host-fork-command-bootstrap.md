# Spec: Internal Retained Host Fork Command Bootstrap

ASSUMPTIONS I'M MAKING:

1. Slice `44` is fully landed on the current tree, so typed host `control_directive` and the paired same-stream worker `control_ack` are now the latest landed `continue_world_worker` widenings.
2. The next honest Family-1 slice is typed host `fork_command`, not optional `progress_ack` first:
   - `progress_ack` is still explicitly optional in the design stack,
   - `progress_ack` still lacks a stronger exact repo-truth anchor than the now-landed control loop,
   - the repo already has a landed exact-source `fork_world_worker` allocator with explicit lineage, so `fork_command` has a concrete execution counterpart to reuse.
3. Slice `45` stays internal-only and orchestrator-facing. It does not widen public `substrate agent ...` surfaces, toolbox behavior, or human direct-to-world UX.
4. To stay smaller than a transport redesign, typed `fork_command` should compile onto the existing `MemberTurnSubmitRequestV1.prompt` seam through a canonical renderer rather than widening `transport-api-types` in this slice.
5. The first typed `fork_command` slice should stay narrow and exact-source:
   - one exact retained source worker is targeted,
   - the payload carries the bounded child-work intent needed to reuse the landed fork bootstrap path,
   - Substrate remains the allocator; workers do not auto-fork.
6. The minimum deny-by-default host-fork gate for this slice should land at `agents.world_dispatch.fork.commands_allowed`, separate from worker-side `requests_allowed` / `recommendations_allowed` and separate from `control_directives_allowed`.
7. Optional `progress_ack`, active-ephemeral inspect/cancel widening, public fork UX, and Family-2 router/attach execution remain later work and must stay out of scope here.

If any of these are wrong, correct them before implementation.

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Prior slices:
- [SPEC-38-internal-retained-world-worker-fork.md](./SPEC-38-internal-retained-world-worker-fork.md)
- [PLAN-38.md](./PLAN-38.md)
- [TASKS-38.md](./TASKS-38.md)
- [SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)
- [PLAN-39.md](./PLAN-39.md)
- [TASKS-39.md](./TASKS-39.md)
- [SPEC-43-internal-retained-host-control-directive-bootstrap.md](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)
- [PLAN-43.md](./PLAN-43.md)
- [TASKS-43.md](./TASKS-43.md)
- [SPEC-44-internal-retained-worker-control-ack-bootstrap.md](./SPEC-44-internal-retained-worker-control-ack-bootstrap.md)
- [PLAN-44.md](./PLAN-44.md)
- [TASKS-44.md](./TASKS-44.md)
Related design stack:
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)  
Phase: `SPECIFY`  
Status: drafted on `2026-06-05`

## Observed Repo Floor

The current repo already provides most of the floor this slice needs:

1. the live `continue_world_worker` request payload already accepts free-form prompt text plus the narrow typed host payloads `approval_response`, `clarification_response`, and `control_directive` on the same retained seam in [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
2. the live runtime already proves the host-side typed payload pattern for that seam: exact retained-worker targeting, deny-by-default typed payload gating, deterministic implementation-owned rendering, and truthful delivery semantics in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
3. the landed `fork_world_worker` action already allocates one retained child with explicit source-to-child lineage, exact same-session/world binding, and Linux-routed bootstrap truth in [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
4. retained-worker `fork_request` and `fork_recommendation` events already persist durable local obligations, but they still require later explicit host action before any child worker is allocated,
5. the current retained-worker event classifier still treats `fork_command` as a deferred label and the current request contract still exposes no typed `worker_continue_fork_command` host payload.

That means the remaining gap is smaller than a new allocator and more specific than generic control: the repo needs a typed host `fork_command` class on the existing retained continue seam, but it should land by reusing already-landed fork bootstrap and lineage truth rather than inventing a second child-allocation plane.

## Objective

Build the first post-Slice-44 host fork-orchestration slice so the host orchestrator can send typed `fork_command` messages to an exact retained worker through the existing `continue_world_worker` seam, with deny-by-default fork-command policy gating, deterministic implementation-owned prompt rendering, and exact reuse of already-landed `fork_world_worker` bootstrap and lineage truth, without widening into worker auto-fork, optional `progress_ack`, active-ephemeral task identity, Family-2 router execution, or a new public fork UI.

Primary runtime story:

1. the host issues `continue_world_worker` with a typed `fork_command` payload for one exact retained source worker,
2. Substrate validates exact caller, target, session, backend, and world-binding truth,
3. Substrate policy-gates typed host `fork_command` separately from generic `continue_world_worker`, worker `fork_request`, worker `fork_recommendation`, and typed host `control_directive`,
4. Substrate renders a deterministic fork-command prompt over the existing member-turn transport and keeps exact same-worker causation explicit,
5. after successful authorization, Substrate may reuse the already-landed internal fork bootstrap path to allocate one retained child with explicit source-to-child lineage,
6. the outcome remains honest about both typed fork-command delivery and child-allocation truth,
7. worker-requested auto-fork, optional `progress_ack`, public control surfaces, active-ephemeral identity widening, and Family-2 router execution remain deferred.

## Current Landed Runtime Note

1. the live `continue_world_worker` request payload now accepts free-form prompt text plus optional `thread_id`, typed `approval_response`, typed `clarification_response`, and typed `control_directive`,
2. the live retained-worker event path now accepts `control_ack`, `fork_request`, and `fork_recommendation`, but still treats `fork_command` as deferred,
3. the live `fork_world_worker` action already returns explicit source and child identity plus authoritative same-session/world lineage,
4. the live policy model now distinguishes worker-side fork requests and recommendations from host-side control directives with dedicated deny-by-default keys, but it still exposes no dedicated host `fork_command` gate,
5. the live repo still has no typed active-ephemeral `task_run_id`, so active-ephemeral inspect/cancel widening remains separate later work.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing internal dispatch/runtime truth expected to widen:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing fork/bootstrap truth expected to be reused:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
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

Dev:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
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

The current repo structure relevant to this slice is:

- `src/`
  - thin top-level CLI entrypoints for `substrate` and `substrate-shim`
- `crates/shell/src/execution/agent_runtime/`
  - shared retained-worker dispatch contracts and authoritative runtime state
  - Slice `45` contract/state changes belong here if the typed host `fork_command` payload or exact-source retained-worker resolution changes
- `crates/shell/src/execution/`
  - shell-side world-dispatch orchestration and policy parsing
  - Slice `45` routing, prompt rendering, and shell-local policy enforcement belong here
- `crates/shell/tests/`
  - integration and regression suites for shell/runtime behavior
  - retained-worker dispatch regressions for this slice should live here when they need end-to-end routing proof
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
  - `docs/CONFIGURATION.md` is the expected documentation surface if a new host fork-command gate becomes repo truth
- `llm-last-mile/`
  - repo-local planning/design/spec artifacts for the retained-worker orchestration track
  - this spec, its plan, and its tasks belong here and must stay aligned

## Code Style

Follow the existing shell/runtime style: exact identity, explanation-ready denials, typed contracts, deterministic side effects, and reuse of already-landed authoritative bootstrap truth rather than parallel implementations.

Preferred style:

```rust
if !policy.world_dispatch_fork_commands_allowed() {
    anyhow::bail!(
        "fork_command_not_allowed: host orchestrators may not send typed fork commands under current policy"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at dispatch, policy, and fork-bootstrap boundaries,
2. preserve exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` joins for every typed host `fork_command`,
3. keep `fork_command` distinct from both generic prompt-based continue and direct `fork_world_worker`:
   - generic continue remains untyped conversational follow-up,
   - direct `fork_world_worker` remains the existing exact-source child-allocation action,
   - typed host `fork_command` is the retained messaging-class bootstrap that may reuse that landed allocation truth,
4. keep source-to-child lineage explicit and reviewable whenever allocation occurs,
5. keep worker auto-fork, optional `progress_ack`, and active-ephemeral identity out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for the new `continue_world_worker` typed host-fork contract:
   - typed `fork_command` payloads validate only with exact child-work intent fields and optional bounded metadata,
   - generic prompt-based continue remains valid,
   - deferred host classes such as optional `progress_ack` still remain out of scope
2. unit tests for policy parsing and denial buckets:
   - the new host fork-command policy key is deny-by-default,
   - explanation-ready denials remain stable for disallowed typed host fork commands,
   - existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact
3. dispatch and integration tests:
   - typed fork commands render onto the existing member-turn submit seam deterministically,
   - exact retained source-worker targeting remains mandatory,
   - successful typed fork commands reuse the landed fork bootstrap and preserve explicit lineage,
   - delivery or bootstrap failure returns explanation-ready errors and leaves no partial child-allocation side effects
4. regression tests:
   - worker `fork_request` and `fork_recommendation` remain host-mediated and do not silently become auto-fork,
   - typed host `fork_command` does not widen into generic control directives or a public fork UX,
   - active-ephemeral inspect/cancel still remains deferred

Coverage expectations:

1. every typed host `fork_command` path has exact-identity and policy-gating coverage,
2. child-allocation reuse stays on one authoritative lineage path,
3. denial and bootstrap-failure paths remain explanation-ready,
4. the slice does not widen into worker auto-fork, optional `progress_ack`, or transport-schema redesign.

## Boundaries

- Always:
  - keep the slice internal-only and orchestrator-facing
  - reuse `continue_world_worker` rather than adding a new dispatch verb
  - keep typed host `fork_command` exact-source and fail closed
  - reuse the already-landed fork bootstrap and lineage truth instead of inventing a second child-allocation plane
  - keep outcome summaries honest about delivery and allocation truth separately
- Ask first:
  - widening `transport-api-types` or `world-service` request schema instead of compiling onto the current prompt seam
  - adding worker auto-fork, worker-issued `fork_command`, or a broader fork-autonomy rubric in the same slice
  - adding optional typed `progress_ack` in the same slice
  - adding a public CLI or toolbox surface for typed host fork commands
  - widening active-ephemeral inspect/cancel in the same slice
  - coupling the slice to Family-2 router/daemon execution or host-global inbox work
- Never:
  - infer the source worker from fuzzy lookup
  - silently allocate a child through a second fork plane that bypasses the landed `fork_world_worker` truth
  - imply worker auto-fork or worker-requested fork satisfaction without explicit host action
  - widen into optional `progress_ack`, active-ephemeral identity, or Family-2 router execution without a separate slice

## Success Criteria

Slice `45` is complete only when all of the following are true:

1. `cargo test -p shell dispatch_contract -- --nocapture` proves `continue_world_worker` accepts the typed host `fork_command` payload only under exact retained-worker identity and boundary requirements while generic prompt-based continue still works.
2. `cargo test -p shell policy_model -- --nocapture` and `cargo test -p substrate-broker -- --nocapture` prove `agents.world_dispatch.fork.commands_allowed` exists as the minimum host fork-command gate, defaults to deny, and returns explanation-ready denials when disallowed.
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` proves accepted typed host `fork_command` messages render deterministically onto the existing retained member-turn prompt seam without requiring a `transport-api-types` or `world-service` schema change in this slice.
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` proves successful typed host `fork_command` handling reuses the already-landed fork bootstrap path, allocates at most one retained child, and preserves explicit source-to-child lineage.
5. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture` plus the targeted shell suites above prove this slice does not widen into worker auto-fork, optional `progress_ack`, active-ephemeral inspect/cancel behavior, or Family-2 router execution.

## Open Questions

1. Whether a later slice should allow typed host `fork_command` to bind directly to exact unresolved `ForkRequest` or `ForkRecommendation` obligations. Default assumption for this bootstrap slice: no, because the current repo already has explicit host `fork_world_worker` action truth and the narrower goal here is the typed host message-class bootstrap, not obligation-consumer redesign.
