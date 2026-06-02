# TASKS-38: Internal Retained World Worker Fork

Source spec: [SPEC-38-internal-retained-world-worker-fork.md](./SPEC-38-internal-retained-world-worker-fork.md)  
Source plan: [PLAN-38.md](./PLAN-38.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: proposed on `2026-06-02`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the fork contract and steering-policy allowlist expansion.
- Packet 2 lands authoritative retained source-target resolution and explicit lineage truth.
- Packet 3 lands internal dispatch wiring and retained child allocation through the existing bootstrap seam.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Fork Contract And Policy Allowlist Expansion

Session goal:

1. add `fork_world_worker` as a typed internal action,
2. keep fork host-initiated and retained-to-retained in Slice `38`,
3. allow steering policy to explicitly admit the action without weakening deny-by-default behavior.

### Tasks

- [ ] Task 1.1: Add the fork action, retained-only validation, and typed fork payload/outcome scaffolding
  - Acceptance: `fork_world_worker` is a valid internal dispatch action; request validation requires retained mode and exact retained source `target_participant_id`; the contract exposes a typed fork payload plus a typed fork outcome that freezes explicit source and child identity expectations.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Widen steering-policy parsing so `fork_world_worker` can be explicitly allowlisted
  - Acceptance: the effective policy/config model accepts `fork_world_worker` as an allowed world-dispatch action while keeping deny-by-default defaults unchanged when the action is absent; REPL-facing internal toolbox fork ingress continues to validate malformed requests and reject denied requests before the Packet 1 unsupported-dispatch stub. Packet 1 regression anchors are `orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_validates_fork_requests_before_packet_one_unsupported_dispatch`, and `orchestrator_world_dispatch_surface_rejects_denied_fork_requests_before_packet_one_unsupported_dispatch`.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_one_unsupported_dispatch -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch_surface_validates_fork_requests_before_packet_one_unsupported_dispatch -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch_surface_rejects_denied_fork_requests_before_packet_one_unsupported_dispatch -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs) if diagnostics or validation lists need to stay aligned with the new action id
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs) if internal ingress must recognize the new action during validation
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) only if Packet 1 lands user-visible policy truth immediately

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `fork_world_worker` is a valid internal action,
2. it validates as retained-only with exact source identity,
3. the typed fork outcome freezes explicit source and child identity expectations,
4. steering-policy parsing can explicitly allow the action,
5. REPL-facing internal toolbox fork ingress proves validation and steering-policy gating before the Packet 1 unsupported-dispatch stub through `orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_validates_fork_requests_before_packet_one_unsupported_dispatch`, and `orchestrator_world_dispatch_surface_rejects_denied_fork_requests_before_packet_one_unsupported_dispatch`.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Retained Source Resolution And Lineage Truth

Session goal:

1. resolve exact retained source-worker fork targets from stored runtime truth,
2. reject invalidated or terminal retained source workers,
3. make source-to-child lineage explicit and distinct from plain spawn semantics.

### Tasks

- [ ] Task 2.1: Add exact retained source-worker fork target resolution in the state store
  - Acceptance: fork source-target resolution is same-session-only, same-world-binding-only, and authoritative-caller-only; it accepts only exact retained source workers that are still valid fork sources under the live lifecycle model.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Add explicit fork lineage truth for the child outcome/state surface
  - Acceptance: authoritative runtime/session state can surface explicit source-to-child lineage without collapsing the fork path into plain spawn terminology; invalidated or terminal source workers fail closed with a stable, reviewable error path.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. fork source targets resolve by exact retained identity,
2. invalidated or terminal source workers fail closed,
3. explicit source-to-child lineage truth exists and is distinct from plain spawn semantics.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Internal Dispatch Wiring And Child Allocation

Session goal:

1. route fork through the internal dispatch layer,
2. enforce steering policy before child allocation,
3. allocate one retained child through the existing bootstrap seam and persist explicit lineage.

### Tasks

- [ ] Task 3.1: Add `fork_world_worker` handling to the internal dispatch path
  - Acceptance: the orchestrator dispatch layer evaluates steering policy, resolves the exact source worker, and returns a typed fork outcome for exact retained source workers.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)

- [ ] Task 3.2: Reuse the retained bootstrap seam to allocate the child and surface explicit lineage
  - Acceptance: allowed fork requests allocate one retained child through the existing retained bootstrap path, persist explicit source-to-child lineage, respect retained-worker caps, and do not widen into worker-requested autonomy, plain spawn-without-lineage, or active-ephemeral child mode.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. steering-policy enforcement happens before fork allocation,
2. allowed fork requests allocate one retained child and return explicit lineage,
3. fork is proven distinct from worker-requested autonomy, plain spawn-without-lineage, and active-ephemeral child mode.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen fork scope,
2. keep host-initiated retained-to-retained v1 fork and all broader deferrals explicit,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: repo-local docs describe fork as internal, host-initiated, exact-source, and retained-to-retained in Slice `38`; no wording implies worker-requested fork, fork recommendations, auto-fork, active-ephemeral task identity widening, approval autonomy, or Family-2 execution have landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/NOTE-37-family-1-ordering-after-cancel-closeout.md`](./NOTE-37-family-1-ordering-after-cancel-closeout.md)
    - [`llm-last-mile/SPEC-38-internal-retained-world-worker-fork.md`](./SPEC-38-internal-retained-world-worker-fork.md)
    - [`llm-last-mile/PLAN-38.md`](./PLAN-38.md)
    - [`llm-last-mile/TASKS-38.md`](./TASKS-38.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no public CLI regression or unintended Family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - validation-only unless narrow in-scope stabilization is required to keep the landed fork path green; any such follow-up must not reopen deferred autonomy, task-identity widening, or Family-2 work

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the fork surface is safely bounded,
2. config/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh spec-driven-development restart.

Proceed directly to the next packet only when:

1. the current packet’s verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into worker-requested fork autonomy, active-ephemeral task identity widening, approval/fork autonomy, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves host-initiated fork cannot be honest without worker-requested autonomy or dedicated fork policy-schema widening first,
2. child-lineage truth cannot be represented distinctly from plain spawn without broader lifecycle redesign,
3. the retained bootstrap seam cannot carry fork lineage without a larger architecture shift,
4. verification proves the planned order is wrong.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet’s verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or tasks was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions the next packet must keep.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.
