# TASKS-34: Host-To-World Steering Policy Hardening For Landed Dispatch Surface

Source spec: [SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md](./SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md)  
Source plan: [PLAN-34.md](./PLAN-34.md)  
Source validation note: [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: completed on `2026-05-31`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the steering policy contract and stable denial vocabulary.
- Packet 2 lands pre-routing policy enforcement for the current three verbs.
- Packet 3 lands lifecycle-aware and concurrency-aware hardening.
- Packet 4 aligns config/docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Steering Policy Contract And Stable Denial Vocabulary

Session goal:

1. add a narrow implementation-bearing policy surface for current family-1 world dispatch,
2. preserve deny-by-default defaults,
3. freeze stable steering denial buckets.

### Tasks

- [x] Task 1.1: Add the narrow world-dispatch steering policy surface to the effective policy model
  - Acceptance: the effective policy/config model can represent steering enablement, allowed backends, allowed actions, allowed modes, same-session and same-world-binding defaults, explicit capability-narrowing permission, and the first in-scope concurrency caps for the landed three-verb surface.
  - Verify:
    - `cargo test -p broker pcm1_policy_ -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
  - Expected files touched:
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md) only if the packet lands user-visible config keys immediately

- [x] Task 1.2: Add stable steering denial vocabulary and contract helpers
  - Acceptance: the current slice has stable, explanation-ready denial buckets for disabled steering, backend/action/mode denial, cross-session/world-binding denial, capability-narrowing denial, concurrency-cap denial, and invalidated-worker denial; later packets can reuse those buckets without inventing new ad hoc strings.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell steering_policy_ -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the effective policy model has a narrow implementation-bearing world-dispatch surface for current verbs,
2. defaults remain deny-by-default,
3. stable steering denial buckets exist.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Pre-Routing Policy Enforcement For Current Verbs

Session goal:

1. enforce steering policy before any world routing occurs,
2. harden `run_world_task`, `spawn_world_worker`, and `continue_world_worker` against disallowed action/mode/backend/boundary requests,
3. preserve the existing runtime seam for allowed requests.

### Tasks

- [x] Task 2.1: Gate `run_world_task` and `spawn_world_worker` by steering enablement, action, mode, backend, and exact boundary truth
  - Acceptance: one-shot and retained bootstrap requests fail closed when steering is disabled, the action or mode is not allowed, the backend is not allowed, or same-session/same-world-binding truth is not satisfied; allowed requests still flow through the already-landed dispatch/runtime path.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - targeted shell integration tests

- [x] Task 2.2: Gate `continue_world_worker` by steering policy and retained-worker exact routability
  - Acceptance: `continue_world_worker` is denied unless steering is enabled, the action/mode/backend are allowed, the exact retained target remains in the same authoritative session/world binding, and the worker remains routable under the current lifecycle truth.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell integration tests

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. policy enforcement happens before world routing,
2. current verbs are deny-by-default unless explicitly allowed,
3. allowed requests still use the existing runtime seam.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Lifecycle-Aware And Concurrency-Aware Hardening

Session goal:

1. turn invalidated/non-routable retained-worker truth into stable policy-visible denials,
2. enforce the first in-scope concurrency caps for the current verb surface,
3. keep the already-landed continue-event truth narrow.

### Tasks

- [x] Task 3.1: Add invalidated/non-routable worker denials and in-scope concurrency cap enforcement
  - Acceptance: invalidated or otherwise non-routable retained workers fail with stable denials instead of generic stale-linkage errors; the current policy surface can cap live retained workers and concurrent ephemeral dispatch for the landed three-verb slice.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell integration tests

- [x] Task 3.2: Keep current continue-event policy hooks narrow and defer approval/fork/control expansion
  - Acceptance: the already-landed event classes remain limited to `reply`, `progress_update`, `follow_up_question`, `blocked`, `result`, and `failure`; `follow_up_question` and `blocked` keep explicit attention semantics; approval/fork/control-directive classes remain deferred and do not pull family-2 producer behavior into this slice.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell integration tests

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. invalidated/non-routable retained workers fail with stable policy-visible denials,
2. in-scope concurrency caps are enforced,
3. approval/fork/control expansion remains deferred.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Config/Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the actual policy surface that landed,
2. keep scope honest about current verbs and current deferrals,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align config/trace/planning truth without widening the slice
  - Acceptance: the repo-local docs describe the steering-policy layer as internal, deny-by-default, and limited to the current landed three-verb surface; no wording implies later verbs, router-owned attach execution, or broader approval/fork policy have already landed.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) only if new audit/trace rows are introduced
    - [`llm-last-mile/NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md`](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)
    - [`llm-last-mile/SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md`](./SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md)
    - [`llm-last-mile/PLAN-34.md`](./PLAN-34.md)
    - [`llm-last-mile/TASKS-34.md`](./TASKS-34.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted policy/shell/world-service suites, and the full workspace tests are green; no public CLI regression or unintended family-2 coupling appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - no planned source edits; this is the final gate after the implementation tasks above

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the steering-policy surface is safely bounded,
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
4. the slice has not widened into later verbs, approval/fork autonomy, or family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. implementation proves the current policy surface cannot honestly be kept narrow to the landed three-verb family-1 surface,
2. the existing runtime seam cannot be policy-hardened without first landing a deferred later verb,
3. concurrency or invalidation truth turns out to require a broader lifecycle or obligation redesign immediately,
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
