# PLAN-39: Internal Retained Worker Approval And Fork Obligation Bootstrap

Source spec: [SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md](./SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-38 Family-1 widening slice  
Status: implemented on `2026-06-03`
Landed posture note: retained-worker `approval_request`, `fork_request`, and `fork_recommendation` acceptance, deny-by-default worker-event policy gating, and durable local obligation projection are landed repo-wide, but typed host approval/control responses, active-ephemeral exact task-identity widening, child auto-allocation, and Family-2 router/attach execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and the landed slice stayed bounded to retained-worker approval/fork obligation bootstrap.

## Objective

Land the first post-Slice-38 Family-1 widening slice by accepting retained-worker `approval_request`, `fork_request`, and `fork_recommendation` events as exact-identity, policy-gated, durable local obligations, without widening into auto-fork, active-ephemeral task identity, Family-2 router execution, or a new public approval-response surface.

This slice is complete only when all of the following are true:

1. the retained-worker event classifier accepts `approval_request`, `fork_request`, and `fork_recommendation` under exact identity and exact world/session boundary truth,
2. a deny-by-default policy surface explicitly gates those worker event classes,
3. accepted worker events persist authoritative local obligations with stable kinds and projection semantics,
4. `ApprovalRequired` and `ForkRequest` remain compatible with the already-landed local attach projection,
5. accepted `fork_request` still requires later explicit host-issued `fork_world_worker` before any child is allocated,
6. `approval_response`, `fork_command`, `control_directive`, `control_ack`, active-ephemeral inspect/cancel widening, auto-fork, and Family-2 router execution remain deferred.

## Plan Summary

The repo already has the prerequisites that make worker-requested approval/fork obligation bootstrap the next honest slice:

1. the seven-verb Family-1 internal control-plane vocabulary is now landed,
2. `fork_world_worker` already exists as the explicit host-issued child-allocation action,
3. the local obligation ledger already has canonical `ApprovalRequired`, `ForkRequest`, and `ForkRecommendation` kinds,
4. the local attach projection already knows that `ApprovalRequired` and `ForkRequest` are attach-eligible obligation kinds,
5. the retained-worker event classifier already has a narrow typed event floor and an explicit deferred-wire-label fail-closed path.

What the repo lacked before Packets `1` through `3` landed was:

1. accepted runtime worker event classes for `approval_request`, `fork_request`, and `fork_recommendation`,
2. minimal deny-by-default worker-event permission keys for approval/fork autonomy,
3. authoritative persistence of those accepted events as local obligations from the live `continue_world_worker` path,
4. regression coverage proving accepted worker requests stay distinct from executed host control-plane actions.

That mattered because the obligation/router design stack already assumed worker producers such as `approval_request` and `fork_request`, but the live runtime had still rejected those labels. The cleanest next move was therefore to freeze the first deferred worker-event subset and its policy/obligation projection before attempting Family-2 router execution or active-ephemeral task-identity widening.

The narrowest honest implementation order was:

1. freeze the newly accepted worker event classes and their policy surface first,
2. freeze their durable obligation mapping and projection semantics second,
3. wire live `continue_world_worker` persistence and regressions third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. accept `approval_request`, `fork_request`, and `fork_recommendation` as live retained-worker event classes,
2. add the minimum deny-by-default policy keys needed to gate those classes explicitly,
3. map accepted worker events into canonical local obligation-ledger records,
4. preserve local attach-projection compatibility for `ApprovalRequired` and `ForkRequest`,
5. keep accepted worker requests explanation-ready and distinct from fulfilled host actions.

### What does not change

1. no new public CLI or toolbox approval/fork response surface,
2. no worker auto-fork,
3. no direct child allocation from worker `fork_request`,
4. no active-ephemeral inspect/cancel widening,
5. no router/daemon execution redesign,
6. no host-global inbox or remote ingress layering,
7. no widening of deferred labels `approval_response`, `fork_command`, `control_directive`, `control_ack`, or generic `attention_required`.

## Implementation Order

### Packet 1: Worker Event Contract And Policy Surface

Goal:

1. freeze the first accepted deferred worker event classes,
2. add minimal deny-by-default policy parsing and denial buckets for approval/fork worker autonomy,
3. keep all remaining deferred labels fail-closed.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. targeted contract and policy tests

Why first:

1. durable persistence should consume a frozen worker-event vocabulary,
2. the runtime needs explicit deny-by-default policy truth before it can accept these classes safely,
3. later packets should not infer autonomy semantics from ad hoc event handling.

Verification checkpoint:

1. `approval_request`, `fork_request`, and `fork_recommendation` are accepted worker event classes,
2. the new policy keys are deny-by-default,
3. explanation-ready denials exist for disallowed worker approval/fork events,
4. remaining deferred labels still fail closed.

### Packet 2: Durable Obligation Mapping And Projection Truth

Goal:

1. freeze the obligation mapping for the newly accepted worker event classes,
2. keep review, attention, and attach semantics explicit and durable,
3. preserve compatibility with the current local obligation-ledger and attach-projection runtime naming.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
4. targeted obligation/state-store tests

Why second:

1. the event classes need one authoritative durable meaning before live dispatch wiring persists them,
2. this keeps the runtime from creating one-off ad hoc inbox rows or prompt-like side effects,
3. attach-projection compatibility must be explicit before live persistence exercises it.

Verification checkpoint:

1. `approval_request` persists `ApprovalRequired`,
2. `fork_request` persists `ForkRequest`,
3. `fork_recommendation` persists `ForkRecommendation`,
4. `ApprovalRequired` and `ForkRequest` keep current local attach eligibility,
5. `ForkRecommendation` remains non-attach-eligible by default.

### Packet 3: Continue Routing And Durable Persistence

Goal:

1. wire the live `continue_world_worker` path to persist the new accepted worker event classes,
2. keep worker requests exact-identity and host-mediated,
3. prove that accepted `fork_request` does not become child allocation by itself.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. targeted shell integration/regression tests

Why third:

1. dispatch wiring should consume a frozen contract and durable mapping,
2. this packet proves the slice is live runtime behavior rather than doc-only policy/schema expansion,
3. the key safety property is behavioral: accepted request does not equal executed fork.

Verification checkpoint:

1. accepted approval/fork worker events persist obligations through live continue handling,
2. identity drift, boundary drift, and denied policy states fail closed before persistence,
3. accepted `fork_request` creates durable obligation state only,
4. later explicit `fork_world_worker` remains the only child-allocation path.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the newly landed worker-event and obligation-projection truth,
2. keep all deferred autonomy, task-identity, and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-39-internal-retained-worker-approval-and-fork-obligation-bootstrap.md`
3. `llm-last-mile/PLAN-39.md`
4. `llm-last-mile/TASKS-39.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `39` as worker-event acceptance plus durable obligation bootstrap, not router execution,
2. docs keep host-issued `fork_world_worker` distinct from worker-requested fork,
3. docs keep approval response/control-directive widening, active-ephemeral inspect/cancel, auto-fork, and Family-2 execution explicitly deferred.

Verification checkpoint:

1. docs remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence later approval/control widening or active-ephemeral identity work without reopening Slice `39`.

## Risks And Mitigations

1. Risk: accepted worker `fork_request` is treated as implicit child allocation.
   Mitigation: keep `fork_request` as an obligation producer only; retain `fork_world_worker` as the only allocation action.
2. Risk: obligation persistence regresses into hidden prompt injection or inbox-only projection.
   Mitigation: route accepted worker events into the canonical local obligation ledger and keep summaries explanation-ready rather than prompt-bearing.
3. Risk: policy widening becomes too broad and accidentally lands generic control-directive/autonomy semantics.
   Mitigation: limit the accepted worker-event classes to `approval_request`, `fork_request`, and `fork_recommendation` only.
4. Risk: the slice implicitly reopens Family-2 router design because attach projection is adjacent.
   Mitigation: reuse the currently landed local attach-projection compatibility only; do not redesign router ownership, claiming, or execution.
5. Risk: approval/fork event handling drags active-ephemeral task identity into scope.
   Mitigation: keep the slice retained-worker-only and leave `RunWorldTaskOutcomeV1` unchanged.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because durable mapping depends on a frozen accepted worker-event vocabulary and policy surface.
2. Packet 2 before Packet 3 because live continue routing should persist one authoritative obligation mapping rather than inventing it inline.
3. Packet 3 before Packet 4 because docs and validation should reflect final live behavior.

### Can be parallelized later

1. broader control-directive and approval-response design work after Slice `39` is stable,
2. later active-ephemeral task-identity work after the retained-worker obligation slice is closed,
3. Family-2 router execution after the producer side is frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell obligation -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `39`

If Slice `39` lands cleanly as narrow worker-event and obligation bootstrap first, the next follow-on work should remain:

1. richer host-to-worker typed approval/control response widening only if still needed,
2. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
3. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `39` implementation scope.
