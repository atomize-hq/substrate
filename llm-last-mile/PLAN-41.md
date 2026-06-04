# PLAN-41: Internal Retained Follow-Up And Blocked Obligation Hardening

Source spec: [SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md](./SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-40 producer-hardening slice  
Status: implemented on `2026-06-03`
Landed posture note: retained-worker `follow_up_question` and `blocked` now have deny-by-default policy gates, exact durable `FollowUpRequired`/`Blocked` persistence on `continue_world_worker`, and stable compatibility projection/attach semantics repo-wide, but host `clarification_response`, broader control classes, active-ephemeral exact task identity, and Family-2 router execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and the landed slice stayed bounded to retained-worker producer hardening over the existing `continue_world_worker` seam.

## Objective

Land the narrow producer-hardening slice that makes retained-worker `follow_up_question` and `blocked` events deny-by-default, exact-identity, durable local obligations on the live `continue_world_worker` path, without widening into `clarification_response`, broader control directives, active-ephemeral identity, or Family-2 router execution.

This slice is complete only when all of the following are true:

1. dedicated deny-by-default policy gates exist for retained-worker `follow_up_question` and `blocked`,
2. accepted `follow_up_question` persists `FollowUpRequired` under exact session/worker/world truth,
3. accepted `blocked` persists `Blocked` under exact session/worker/world truth,
4. the already-landed compatibility projection and attach-priority semantics remain truthful for those obligations,
5. approval/fork behavior from Slices `39` and `40` remains unchanged,
6. `clarification_response`, broader control classes, active-ephemeral inspect/cancel widening, and Family-2 router execution remain deferred.

## Plan Summary

The repo already has the prerequisites that make this the next honest slice:

1. the Family-1 dispatch surface is already closed at seven verbs,
2. the retained-worker classifier already accepts `follow_up_question` and `blocked`,
3. both classes are already attention-driving by default,
4. the local obligation ledger already contains `FollowUpRequired` and `Blocked`,
5. compatibility projection and local auto-attach logic already understand those obligation kinds.

What the repo lacked before Packets `1` through `3` landed was narrower than a new host-response slice:

1. dedicated deny-by-default policy keys for those two retained-worker producer classes,
2. live `continue_world_worker` persistence of accepted `follow_up_question` as `FollowUpRequired`,
3. live `continue_world_worker` persistence of accepted `blocked` as `Blocked`,
4. regression coverage proving the producer path is durable while approval/fork behavior stays stable.

That mattered because the design stack already treated follow-up and blocked as first-class attention-driving obligation kinds, but the live producer path had stopped short of durable truth. The cleanest move was therefore to close that producer gap before attempting typed host `clarification_response` or any downstream Family-2 execution work.

The narrowest honest implementation order was:

1. freeze the missing policy dimensions and denial buckets first,
2. freeze the obligation mapping and projection semantics second,
3. wire live persistence and producer-path regressions third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. add the minimum deny-by-default policy keys for retained-worker `follow_up_question` and `blocked`,
2. enforce those gates on the live `continue_world_worker` event path,
3. map accepted `follow_up_question` into canonical `FollowUpRequired`,
4. map accepted `blocked` into canonical `Blocked`,
5. preserve exact identity and existing compatibility projection semantics for both durable kinds.

### What does not change

1. no new dispatch verb,
2. no typed host `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, or `fork_command`,
3. no public CLI or toolbox widening,
4. no active-ephemeral inspect/cancel widening,
5. no router/daemon execution redesign,
6. no host-global inbox or remote ingress layering,
7. no new obligation kinds.

## Implementation Order

### Packet 1: Policy Surface And Denial Coverage

Goal:

1. freeze the missing deny-by-default policy dimensions,
2. add explanation-ready denials for disallowed `follow_up_question` and `blocked`,
3. keep existing approval/fork policy behavior intact.

Primary touch surface:

1. `crates/shell/src/execution/policy_model.rs`
2. `crates/broker/src/policy.rs`
3. `crates/broker/src/effective_policy.rs`
4. targeted policy tests

Why first:

1. the producer path should not persist new durable obligations without explicit policy truth,
2. the design matrix already calls out the missing dimensions,
3. later packets should build on deny-by-default behavior rather than implicit allowlists.

Verification checkpoint:

1. `follow_up_allowed` and `blocked_allowed` default to `false`,
2. explicit enables merge correctly,
3. denied producer events fail with stable explanation-ready errors,
4. approval/fork gates from Slices `39` and `40` still behave unchanged.

### Packet 2: Obligation Mapping And Projection Semantics

Goal:

1. freeze the durable mapping from retained-worker event class to obligation kind,
2. preserve exact binding metadata and truthful attention flags,
3. keep compatibility projection and attach semantics stable.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs` only if narrow helper or test widening is required
3. `crates/shell/src/execution/agent_runtime/state_store.rs` only if compatibility projection coverage needs a narrow patch
4. targeted shell tests

Why second:

1. the repo already has the destination durable kinds, so the contract to freeze next is the exact producer-to-ledger mapping,
2. projection semantics should remain explicit before the live path is treated as complete,
3. this keeps the slice honest about reusing landed infrastructure rather than inventing new ledger behavior.

Verification checkpoint:

1. accepted `follow_up_question` maps to `FollowUpRequired`,
2. accepted `blocked` maps to `Blocked`,
3. durable payload metadata keeps exact worker/session/backend/world truth,
4. `FollowUpRequired` and `Blocked` remain compatible with the already-landed inbox and auto-attach projections.

### Packet 3: Live Continue-Path Persistence And Regression Proof

Goal:

1. wire the live `continue_world_worker` path to persist the new obligations,
2. prove that disallowed events do not persist anything,
3. prove that approval/fork and public status/control behavior does not regress.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell integration and regression tests

Why third:

1. the live runtime path should consume the frozen policy and mapping semantics,
2. this packet proves the slice is real producer-path behavior rather than doc-only wiring,
3. regression proof matters because the same path already handles approval/fork semantics from Slices `39` and `40`.

Verification checkpoint:

1. accepted `follow_up_question` persists exactly one `FollowUpRequired` obligation,
2. accepted `blocked` persists exactly one `Blocked` obligation,
3. denied or invalid events leave no new durable obligation behind,
4. existing approval/fork persistence and public projection tests remain green.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `41` scope,
2. keep `clarification_response`, active-ephemeral identity, and Family-2 work explicitly deferred,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-41-internal-retained-follow-up-and-blocked-obligation-hardening.md`
3. `llm-last-mile/PLAN-41.md`
4. `llm-last-mile/TASKS-41.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `41` as producer-side follow-up and blocked obligation hardening, not typed host clarification or transport redesign,
2. docs keep broader host control/response widening explicitly deferred,
3. docs keep Family-2 router execution downstream.

Verification checkpoint:

1. docs remain honest about scope,
2. validation is green,
3. the next follow-on slice can be `clarification_response` against exact unresolved `FollowUpRequired` obligations without reopening Slice `41`.

## Risks And Mitigations

1. Risk: the slice quietly widens into typed host `clarification_response`.
   Mitigation: keep the slice strictly producer-side and treat host response binding as the explicit follow-on slice.
2. Risk: follow-up and blocked events persist without dedicated policy truth.
   Mitigation: land the new deny-by-default keys and denial coverage before producer-path persistence.
3. Risk: the slice invents new ledger semantics even though the durable kinds already exist.
   Mitigation: reuse `FollowUpRequired` and `Blocked` exactly and limit any ledger/state-store edits to narrow compatibility helpers or tests.
4. Risk: approval/fork behavior regresses on the shared `continue_world_worker` path.
   Mitigation: keep regression tests for existing approval/fork producer logic in the validation wall.
5. Risk: the slice is misclassified as Family-2 work because auto-attach already understands these obligation kinds.
   Mitigation: limit scope to the live producer path and preserve downstream router ownership exactly as it is.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because obligation persistence should not outrun explicit policy truth.
2. Packet 2 before Packet 3 because the live path should consume one frozen mapping/projection contract.
3. Packet 3 before Packet 4 because docs and validation should reflect final runtime behavior.

### Can be parallelized later

1. typed host `clarification_response` after Slice `41` is stable,
2. broader host control/ack classes after `clarification_response` if still needed,
3. active-ephemeral exact task identity work as its own later slice,
4. Family-2 router execution after Family-1 producer and host-response semantics are frozen enough for downstream attach behavior.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell obligation -- --nocapture
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `41`

If Slice `41` lands cleanly as narrow producer hardening first, the next follow-on work should remain:

1. typed host `clarification_response` only, bound to exact unresolved `FollowUpRequired` obligations over the same retained `continue_world_worker` seam,
2. broader typed host-to-worker classes such as `progress_ack`, `control_directive`, `control_ack`, and `fork_command` only if still needed,
3. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
4. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `41` implementation scope.
