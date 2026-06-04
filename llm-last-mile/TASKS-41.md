# TASKS-41: Internal Retained Follow-Up And Blocked Obligation Hardening

Source spec: [SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md](./SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md)  
Source plan: [PLAN-41.md](./PLAN-41.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: draft for review on `2026-06-03`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 lands the missing policy dimensions and denial coverage.
- Packet 2 freezes durable follow-up and blocked obligation mapping semantics.
- Packet 3 proves live `continue_world_worker` persistence and regression safety.
- Packet 4 aligns docs/config truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Policy Surface And Denial Coverage

Session goal:

1. add deny-by-default policy keys for retained-worker `follow_up_question` and `blocked`,
2. add explanation-ready denials for disallowed events,
3. preserve existing approval/fork policy behavior.

### Tasks

- [ ] Task 1.1: Add minimal deny-by-default policy parsing for `follow_up_allowed` and `blocked_allowed`
  - Acceptance: policy/config truth can explicitly allow or deny retained-worker `follow_up_question` and `blocked`; both keys default to `false`; merge/explain behavior remains aligned with the existing `agents.world_dispatch` patch surface.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

- [ ] Task 1.2: Enforce stable deny buckets for disallowed retained-worker `follow_up_question` and `blocked`
  - Acceptance: disallowed events fail with stable explanation-ready errors on the live `continue_world_worker` path; approval/fork gating from Slices `39` and `40` remains intact.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted tests adjacent to the touched implementation

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. both new policy keys are deny-by-default,
2. disallowed `follow_up_question` and `blocked` events fail closed with stable explanations,
3. approval/fork gates still behave exactly as before.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Durable Obligation Mapping And Projection Semantics

Session goal:

1. map accepted `follow_up_question` into `FollowUpRequired`,
2. map accepted `blocked` into `Blocked`,
3. preserve exact binding metadata and already-landed compatibility projection semantics.

### Tasks

- [ ] Task 2.1: Freeze `follow_up_question` to `FollowUpRequired` durable projection
  - Acceptance: accepted `follow_up_question` persists canonical `FollowUpRequired` with exact source participant, backend, world, and attention metadata; follow-up projection remains compatible with the existing inbox/attach behavior.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) if helper/test widening is required
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) if compatibility projection coverage needs a narrow patch

- [ ] Task 2.2: Freeze `blocked` to `Blocked` durable projection
  - Acceptance: accepted `blocked` persists canonical `Blocked` with exact source participant, backend, world, and attention metadata; blocked projection remains compatible with runtime-alert style inbox projection and existing attach-priority truth.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) if helper/test widening is required
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs) only if narrow regression coverage needs a patch
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) if compatibility projection coverage needs a narrow patch

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. both event classes map to their intended durable kinds,
2. exact binding metadata and attention flags stay truthful,
3. compatibility projection and attach-priority semantics remain stable.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Live Continue-Path Persistence And Regression Proof

Session goal:

1. prove the live `continue_world_worker` path persists the new obligations,
2. prove denied events do not leave durable side effects,
3. keep approval/fork and public projection behavior green.

### Tasks

- [ ] Task 3.1: Prove accepted `follow_up_question` and `blocked` events persist exactly one durable obligation each
  - Acceptance: the live retained-worker event path writes one canonical `FollowUpRequired` or `Blocked` record per accepted event, with stable event labels and payload projection; no duplicate or wrong-kind obligation is created.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [ ] Task 3.2: Prove denial and non-regression behavior on the shared producer path
  - Acceptance: denied `follow_up_question` and `blocked` events persist nothing; approval/fork producer behavior from Slices `39` and `40` stays green; public control/status projection remains honest without widening its public contract.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell auto_attach -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. live continue handling persists the new durable obligations exactly once,
2. denied events leave no persistence side effect,
3. existing approval/fork and public projection behavior remains green.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align docs/config truth with the frozen Slice `41` scope,
2. keep host `clarification_response`, active-ephemeral identity, and Family-2 execution explicitly deferred,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning and configuration docs without widening the slice
  - Acceptance: repo-local docs describe Slice `41` as retained-worker follow-up and blocked obligation hardening on the existing `continue_world_worker` seam, not typed host clarification, transport redesign, or Family-2 router execution.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md`](./SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md)
    - [`llm-last-mile/PLAN-41.md`](./PLAN-41.md)
    - [`llm-last-mile/TASKS-41.md`](./TASKS-41.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no unintended widening into typed host clarification, broader control directives, active-ephemeral identity, or Family-2 execution appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - final validation may require bounded follow-up fixes inside already-touched Slice `41` policy and producer-path surfaces, but must not reopen typed host clarification, active-ephemeral identity, or Family-2 router execution

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `41` surface is safely bounded,
2. config/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh `spec-driven-development` restart.

Proceed directly to the next packet only when:

1. the current packet’s verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into typed host clarification, broader control directives, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest retained-worker follow-up or blocked persistence requires broader dispatch or transport reshaping first,
2. the existing local ledger cannot carry the required `FollowUpRequired` or `Blocked` projection truth without a broader obligation-model change,
3. the shared `continue_world_worker` path cannot preserve approval/fork behavior while adding the new producer classes,
4. verification proves the planned packet order is wrong.

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
