# TASKS-48: Internal Family-2 Router-Owned Session Auto-Attach Execution Boundary

Source spec: [SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md)  
Source plan: [PLAN-48.md](./PLAN-48.md)  
Source remaining-scope note: [REMAINING-family-2-scope-2026-05-30.md](./REMAINING-family-2-scope-2026-05-30.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: Packets 1-4 landed; slice 48 closed on `2026-06-07` after Packet 4 aligned docs and reran the validation wall

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `48` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Execution Packets

This slice was planned as four sequential `/incremental-implementation` sessions, but Packets 1-3 are already landed in code and now serve as the frozen floor for Packet 4 closeout.

- Packet 1 is landed and should not be reopened unless the contract changes.
- Packet 2 is landed and should not be reopened unless the contract changes.
- Packet 3 is landed and should not be reopened unless the contract changes.
- Packet 4 is landed; no active implementation packets remain for slice 48.

Treat the Packet 4 checkpoint as green repo floor for this slice. Slice 48 is now closed against the landed Packet 1-4 floor.

## Packet 1: Policy And Attach-Semantic Contract Freeze

Session goal:

1. make router-owned local session auto-attach explicitly deny-by-default,
2. wire policy evaluation through `workflow.router.enabled` plus the relevant existing world-dispatch obligation/fork gates,
3. keep the current local attach-state enum names stable while freezing their semantics against the forward design vocabulary.

### Tasks

- [x] Task 1.1: Enforce the router-owned auto-attach policy gate
  - Acceptance: router-owned auto-attach fails closed unless `workflow.router.enabled` is true, and the claimed obligation kind remains allowed under the relevant existing world-dispatch gate (`approval_allowed`, `follow_up_allowed`, `blocked_allowed`, or `fork.requests_allowed` as applicable).
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

- [x] Task 1.2: Freeze current local attach-state semantics without renaming the artifact
  - Acceptance: the runtime and tests make it explicit that current local attach states map semantically to the forward design states, and no Packet 1 change widens into a whole-record schema rename.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. router-owned auto-attach is deny-by-default,
2. the top-level router gate and per-kind eligibility rules are explicit and tested,
3. current local attach-state semantics are frozen without rename churn,
4. missing or disallowed policy truth fails closed before attach execution begins.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Production-Owned Router Execution Path

Session goal:

1. add the first real non-test host-side router-owned execution path,
2. evaluate local exact session and obligation truth through that path,
3. run the existing local attach executor from one internal work-loop or equivalent internal entrypoint.

### Tasks

- [x] Task 2.1: Introduce the host-side router-owned execution entrypoint
  - Acceptance: the repo has a non-test host-side path that can discover local detached sessions with eligible obligations and invoke router-owned auto-attach execution; the call site is internal-only and not world-service-owned.
  - Verify:
    - targeted unit or integration coverage for the chosen host-side execution path
    - `cargo test -p shell auto_attach -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - one new or existing host-side execution file under [`crates/shell/src/execution/`](../crates/shell/src/execution/)

- [x] Task 2.2: Keep attach mode selection and boundary checks exact on the new path
  - Acceptance: the new host-side execution path reuses exact local session truth, keeps continuity attach preferred, keeps fresh attach fail-closed fallback only, and does not launch when the session is attached, terminal, or missing attach-contract truth.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. a real host-side non-test call site exists,
2. router-owned execution stays local-only and exact-session scoped,
3. continuity-first and fail-closed fresh attach semantics remain intact,
4. no world-service-owned or public CLI-owned attach execution path is introduced.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Settlement, Manual Reattach Convergence, And Observability

Session goal:

1. prove one attach episode per orchestration session,
2. make successful manual `reattach` and router-owned attach converge without duplicate launch,
3. keep router-owned outcomes explanation-ready.

### Tasks

- [x] Task 3.1: Prove session-scoped settlement and manual-reattach convergence
  - Acceptance: successful router-owned attach marks the claimed obligation satisfied and sibling eligible obligations superseded without resolving review state; successful manual `reattach` cancels or converges queued/claimed router work without duplicate attach launch.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)

- [x] Task 3.2: Emit explanation-ready router-owned outcomes without widening the slice
  - Acceptance: router-owned evaluation, claim, attach success, supersession/cancellation, and fail-closed outcomes are explanation-ready with exact session/obligation/backend joins, but the packet does not widen into a general workflow engine or public daemon trace surface.
  - Verify:
    - targeted tests for the chosen event or trace helper
    - `cargo test -p shell auto_attach -- --nocapture`
  - Files:
    - host-side router execution file under [`crates/shell/src/execution/`](../crates/shell/src/execution/)
    - [`docs/TRACE.md`](../docs/TRACE.md) only if real trace publication lands in this packet

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. one attach episode per orchestration session is proven,
2. manual `reattach` and router-owned attach converge without duplicate launch,
3. success does not resolve review state,
4. router-owned outcomes are explanation-ready and bounded.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Landed Docs Alignment And Final Validation

Session goal:

1. align policy and runtime docs with the actual Slice `48` boundary,
2. keep broader host-targeting and ingress work explicitly deferred,
3. run the validation wall.

### Tasks

- [x] Task 4.1: Align policy and orchestration docs with the landed Slice `48` boundary
  - Acceptance: docs describe Slice `48` as the first internal host-side router-owned local session auto-attach execution boundary, keep the deny-by-default gates honest, and do not imply cross-host inbox, public daemon UX, or worker continuation have landed.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`docs/reference/policy/contract.md`](../docs/reference/policy/contract.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) if touched
    - [`llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md`](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md)
    - [`llm-last-mile/PLAN-48.md`](./PLAN-48.md)
    - [`llm-last-mile/TASKS-48.md`](./TASKS-48.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell and broker suites, and full workspace tests are green against the bounded Slice `48` file set; this task validates the slice and does not become an open-ended cleanup bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if a validation command fails because Slice `48` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `48` boundary is safely bounded,
2. docs and policy truth are honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh `spec-driven-development` restart.

Proceed directly to the next packet only when:

1. the current packet's verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into cross-host ingress, public router daemon UX, workflow-engine breadth, or worker continuation logic.

Reopen spec, plan, or tasks only if one of these is true:

1. the repo cannot support a real host-side router-owned execution path without a broader daemon/service decision than this slice allows,
2. the existing policy namespaces prove insufficient and a new top-level schema is unavoidably required,
3. current attach-state semantics cannot stay stable without a larger obligation-schema redesign,
4. convergence tests prove manual `reattach` and router-owned attach need a broader public control-surface change first.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether the slice stayed bounded to host-side local router-owned attach execution rather than widening into broader Family-2 or workflow-engine scope.
