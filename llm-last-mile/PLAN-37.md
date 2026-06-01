# PLAN-37: Internal Cancel World Work

Source spec: [SPEC-37-internal-cancel-world-work.md](./SPEC-37-internal-cancel-world-work.md)  
Source validation note: [NOTE-36-family-1-ordering-after-stop-closeout.md](./NOTE-36-family-1-ordering-after-stop-closeout.md)  
Plan type: sixth implementation-bearing Family-1 control-plane slice  
Status: drafted on `2026-06-01`

## Objective

Ship the first cancel-bearing Family-1 control-plane slice by adding an internal retained-turn `cancel_world_work` verb that interrupts active retained work in flight and persists explicit cancelled terminal truth without widening into active-ephemeral dual-target cancel, stop-closeout reuse, fork lineage, or Family-2 router/attach work.

This slice is complete only when all of the following are true:

1. `cancel_world_work` exists as a typed internal dispatch action,
2. the action is retained-worker-only in v1 and requires exact `target_participant_id`,
3. steering policy can explicitly allow or deny the action,
4. allowed Linux requests cancel active retained work in flight and return a typed cancel outcome distinct from stop closeout,
5. authoritative runtime/session state exposes explicit cancelled terminal truth,
6. active-ephemeral cancel, `fork_world_worker`, approval/fork autonomy, and Family-2 router/attach work remain deferred.

## Plan Summary

The repo already has the prerequisites that make `cancel_world_work` the next honest Family-1 slice:

1. typed internal dispatch contract for `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker`,
2. exact orchestrator/session/worker/world-binding validation,
3. the deny-by-default steering-policy floor,
4. retained-runtime records that already carry live ownership and control metadata such as `latest_run_id` and `cancel_supported`,
5. Linux-first routed world-dispatch posture and internal toolbox ingress coverage.

What the repo still lacks is:

1. a typed internal cancel action,
2. authoritative retained-worker cancel target resolution,
3. explicit retained `cancelled` lifecycle truth,
4. routed cancel behavior distinct from stop closeout,
5. any exact active-ephemeral task target surface.

That matters because the design stack still frames cancel as dual-target, but the current tree does not yet have the exact task identity needed for active-ephemeral inspect/cancel. The narrowest honest implementation order is therefore:

1. freeze `cancel_world_work` as retained active-turn cancel first,
2. land explicit cancel lifecycle truth distinct from stop,
3. route cancel through the internal dispatch layer,
4. leave active-ephemeral cancel widening and fork for later slices.

`fork_world_worker` does not come first because it still drags in lineage and autonomy policy. Family 2 does not come first because it remains downstream of the Family-1 control-plane vocabulary. Full dual-target cancel does not belong in Slice `37` because the exact active-ephemeral task-target seam is not yet runtime truth.

## Locked Decisions

### What changes

1. add `cancel_world_work` to the internal dispatch action vocabulary,
2. add a typed cancel payload and typed cancel outcome contract,
3. widen steering-policy allowlist parsing so the action can be explicitly permitted,
4. add authoritative retained-worker cancel target resolution with active-work eligibility checks,
5. add explicit retained cancelled terminal truth distinct from stop closeout,
6. route cancel through the internal dispatch layer and a dedicated cancel-only retained runtime control seam.

### What does not change

1. no new public human CLI,
2. no active-ephemeral cancel in this slice,
3. no task-identity widening for `run_world_task`,
4. no `fork_world_worker`,
5. no approval-request or fork-request autonomy rollout,
6. no Family-2 router/attach execution work,
7. no reuse of stopped closeout terminology or outcome types for cancel,
8. no shared generic retained-control bus in Slice `37`.

## Implementation Order

### Packet 1: Cancel Contract And Policy Allowlist Expansion

Goal:

1. freeze the cancel action, payload, and outcome contract,
2. allow steering policy to explicitly admit `cancel_world_work`,
3. keep deny-by-default behavior intact when the action is not allowlisted,
4. freeze v1 scope as retained-worker-only.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. targeted contract and policy tests

Why first:

1. the runtime needs a frozen cancel action and outcome shape before lifecycle work can stay coherent,
2. steering-policy parsing must know the action before dispatch enforcement can reuse it,
3. later packets should reuse one stable action string and one stable typed outcome.

Verification checkpoint:

1. `cancel_world_work` is a valid internal dispatch action,
2. it validates as retained-only with exact target identity,
3. effective policy parsing can explicitly allow the action without relaxing deny-by-default defaults.

### Packet 2: Retained Cancel Target Resolution And Lifecycle Truth

Goal:

1. resolve exact retained-worker cancel targets from authoritative state-store truth,
2. reject non-live, idle, or already-terminal targets explicitly,
3. add explicit cancelled lifecycle truth distinct from stopped/failed.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/session.rs`
3. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
4. targeted lifecycle and state-store tests

Why second:

1. cancel is fundamentally an exact-target and cancel-eligibility problem before it is a routing problem,
2. the current tree does not yet model retained cancelled state, so lifecycle truth must be frozen before routed cancel can stay honest,
3. getting this right keeps cancel from degenerating into either stop or a fuzzy best-effort interrupt.

Verification checkpoint:

1. exact retained-worker cancel targets resolve only inside the authoritative session/world binding,
2. non-live or already-terminal retained workers fail closed,
3. cancelled terminal truth is explicit and distinct from stopped truth.

### Packet 3: Internal Dispatch Wiring And Cancel Closeout

Goal:

1. route `cancel_world_work` through the internal dispatch layer,
2. enforce steering policy before executing cancel behavior,
3. interrupt active retained work in flight and persist typed cancel closeout.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. targeted shell integration tests

Why third:

1. dispatch wiring should consume a frozen contract and a trusted cancel target resolver,
2. cancel routing should use explicit cancelled-state semantics rather than inventing them on the fly,
3. a dedicated cancel-only transport keeps this packet bounded and avoids prematurely inventing a shared internal control bus,
4. this packet proves cancel is a real control-plane verb rather than a doc-only placeholder.

Verification checkpoint:

1. disallowed cancel requests fail closed through the steering-policy layer,
2. allowed cancel requests interrupt active retained work in flight,
3. cancel does not widen into stop, inspect, continue, or fork behavior.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the now-frozen cancel scope,
2. keep retained-only v1 cancel and all later verb deferrals explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/NOTE-36-family-1-ordering-after-stop-closeout.md`
3. `llm-last-mile/SPEC-37-internal-cancel-world-work.md`
4. `llm-last-mile/PLAN-37.md`
5. `llm-last-mile/TASKS-37.md`
6. targeted test suites

What this packet must enforce:

1. docs describe cancel as internal, retained-worker-only in Slice `37`, and distinct from stop,
2. docs keep active-ephemeral cancel, fork, approval autonomy, and Family-2 work explicitly deferred,
3. no wording implies full dual-target cancel has landed if the implementation only freezes retained-turn cancel.

Verification checkpoint:

1. docs remain honest about scope and deferrals,
2. validation is green,
3. the next Family-1 slice can be active-ephemeral cancel widening or fork sequencing rather than cancel cleanup.

## Risks And Mitigations

1. Risk: cancel collapses into stop because stop is the only landed closeout model.
   Mitigation: add explicit cancelled lifecycle truth and typed cancel outcomes; do not reuse stopped state or stop summaries.
2. Risk: Slice `37` quietly widens into active-ephemeral dual-target cancel.
   Mitigation: freeze retained active-turn-only scope and reject any design that depends on missing task-target identity.
3. Risk: cancel accepts parked or otherwise non-running retained workers and becomes a fuzzy lifecycle rewrite.
   Mitigation: make active cancel eligibility an explicit state-store gate and pin it in tests.
4. Risk: new cancelled state truth regresses public session-control/status surfaces.
   Mitigation: cover public-control regression suites and keep state additions narrow and reviewable.
5. Risk: implementation pressure turns cancel into a generic shared control bus for deferred verbs.
   Mitigation: lock Slice `37` to a dedicated cancel-only transport and treat any shared control bus as a later explicit design choice.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because target resolution and lifecycle truth need a frozen cancel contract and policy action.
2. Packet 2 before Packet 3 because dispatch wiring should consume one trusted cancel target-resolution path and explicit cancelled-state semantics.
3. Packet 3 before Packet 4 because docs and validation should reflect final routed behavior.

### Can be parallelized later

1. additional regression expansion across retained cancel/reporting surfaces after the cancel contract is stable,
2. docs wording and config alignment during the final packet once the routed cancel semantics are frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `37`

If Slice `37` lands cleanly as retained active-turn cancel first, the next Family-1 order should be:

1. active-ephemeral exact task identity plus dual-target inspect/cancel widening if still needed,
2. `fork_world_worker` only after the cancel family is real,
3. broader approval/fork autonomy work after the later verb family is in place,
4. Family-2 router/attach implementation only as its own downstream execution track.

That follow-on order is part of this plan discipline. It is not part of Slice `37` implementation scope.
