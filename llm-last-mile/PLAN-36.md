# PLAN-36: Internal Retained World Worker Stop Closeout

Source spec: [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)  
Source validation note: [NOTE-35-family-1-ordering-after-inspect-snapshot.md](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)  
Plan type: fifth implementation-bearing Family-1 control-plane slice  
Status: implemented on `2026-06-01`
Landed posture note: the stop contract, steering-policy allowlisting, exact-target retained-worker stop resolution, and durable closeout routing are landed repo-wide, but retained-worker stop routing is Linux-only in v1 and fails closed on non-Linux builds.
Validation note: Packet 4's validation wall is green, including a rustfmt-only cleanup in `crates/shell/src/execution/agent_runtime/state_store.rs` and `crates/shell/src/execution/orchestrator_world_dispatch.rs` that did not change behavior.

## Objective

Ship the first stop-bearing Family-1 control-plane slice by adding an internal retained-worker `stop_world_worker` verb that performs durable exact-target closeout without widening into active cancel, fork lineage, or a second stop/runtime model.

This slice is complete only when all of the following are true:

1. `stop_world_worker` exists as a typed internal dispatch action,
2. the action is retained-worker-only in v1 and requires exact `target_participant_id`,
3. steering policy can explicitly allow or deny the action,
4. allowed Linux requests drive durable retained-worker closeout using authoritative existing stop/runtime truth,
5. already-terminal retained workers fail closed,
6. `cancel_world_work`, `fork_world_worker`, approval/fork autonomy, and Family-2 router/attach work remain deferred.

## Plan Summary

The repo already has the prerequisites that make `stop_world_worker` the smallest honest next execution-affecting slice:

1. typed internal dispatch contract for `run_world_task`, `spawn_world_worker`, `continue_world_worker`, and `inspect_world_worker`,
2. exact orchestrator/session/worker/world-binding validation,
3. the deny-by-default steering-policy floor,
4. existing runtime/session states for `Stopping` and `Stopped`,
5. existing public stop-closeout helpers and private owner stop transport posture.

What the repo still lacks is a typed internal stop action that exposes those durable stop semantics directly to the host orchestrator for exact retained workers.

Current landed runtime note:

1. stop contract admission, policy parsing, and exact retained-worker target validation are repo-wide,
2. routed retained-worker stop outcomes drive durable closeout on Linux in v1 through the existing private owner stop surface,
3. non-Linux builds reject retained stop routing rather than widening into public stop or cancel behavior.

The narrowest honest implementation order is therefore:

1. freeze the stop action, payload, outcome, and policy allowlist expansion first,
2. add authoritative retained-worker stop target resolution and already-terminal denial,
3. wire the new stop action through the internal dispatch layer by reusing existing closeout/runtime truth,
4. finish with docs and the validation wall.

`cancel_world_work` does not come first because it spans active ephemeral and retained-turn targets, which means a broader target model than retained-worker stop. `fork_world_worker` does not come first because it drags in lineage and autonomy policy. Family 2 does not come first because it remains downstream of the Family-1 control-plane vocabulary.

## Locked Decisions

### What changes

1. add `stop_world_worker` to the internal dispatch action vocabulary,
2. add a typed stop payload and typed stop outcome contract,
3. widen steering-policy allowlist parsing so the action can be explicitly permitted,
4. add authoritative retained-worker stop target resolution with already-terminal rejection,
5. reuse existing stop-closeout helpers and transport posture instead of inventing a second stop plane.

### What does not change

1. no new public human CLI,
2. no active-ephemeral cancel/stop in this slice,
3. no `cancel_world_work`,
4. no `fork_world_worker`,
5. no approval-request or fork-request autonomy rollout,
6. no Family-2 router/attach execution work,
7. no new lifecycle taxonomy unrelated to durable retained-worker stop.

## Implementation Order

### Packet 1: Stop Contract And Policy Allowlist Expansion

Goal:

1. freeze the stop action, payload, and outcome contract,
2. allow steering policy to explicitly admit `stop_world_worker`,
3. keep deny-by-default behavior intact when the action is not allowlisted.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. targeted contract and policy tests

Why first:

1. the runtime needs a frozen stop action and outcome shape before closeout wiring can stay coherent,
2. steering-policy parsing must know the action before dispatch enforcement can reuse it,
3. later packets should reuse one stable action string and one stable typed outcome.

Verification checkpoint:

1. `stop_world_worker` is a valid internal dispatch action,
2. it validates as retained-only with exact target identity,
3. effective policy parsing can explicitly allow the action without relaxing deny-by-default defaults.

### Packet 2: Authoritative Retained-Worker Stop Target Resolution

Goal:

1. resolve exact retained-worker stop targets from authoritative state-store truth,
2. reject already-terminal retained workers explicitly,
3. preserve exact session and world-binding safety.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. targeted state-store tests

Why second:

1. stop is fundamentally an exact-target and terminal-eligibility problem before it is a routing problem,
2. the state-store layer already owns the authoritative retained-worker/session linkage truth,
3. getting this right keeps stop from becoming fuzzy or accidentally broad.

Verification checkpoint:

1. exact retained-worker stop targets resolve only inside the authoritative session/world binding,
2. already-terminal retained workers fail closed,
3. the target-resolution path stays distinct from inspect and continue semantics where needed.

### Packet 3: Internal Dispatch Wiring And Durable Closeout

Goal:

1. route `stop_world_worker` through the internal dispatch layer,
2. enforce steering policy before executing stop behavior,
3. drive durable retained-worker closeout through existing stop/runtime helpers.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. targeted shell integration tests

Why third:

1. dispatch wiring should consume a frozen contract and a trusted target resolver,
2. this packet proves stop is a real control-plane verb rather than a doc-only action,
3. closeout reuse should be verified against the final routed behavior, not only in isolation.

Verification checkpoint:

1. disallowed stop requests fail closed through the steering-policy layer,
2. allowed stop requests drive durable closeout for exact retained workers,
3. stop does not widen into cancel, continue, or fork behavior.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the now-landed stop scope,
2. keep the slice honest about retained-only stop and remaining deferrals,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/NOTE-35-family-1-ordering-after-inspect-snapshot.md`
3. `llm-last-mile/SPEC-36-internal-retained-world-worker-stop-closeout.md`
4. `llm-last-mile/PLAN-36.md`
5. `llm-last-mile/TASKS-36.md`
6. targeted test suites

What this packet must enforce:

1. docs describe stop as internal and retained-worker-only in v1,
2. docs keep durable stop separate from active cancel and fork/autonomy work, and keep routed closeout Linux-only in v1,
3. no wording implies `cancel_world_work`, `fork_world_worker`, approval autonomy, or Family-2 work have landed.

Verification checkpoint:

1. docs remain honest about scope and deferrals,
2. validation is green,
3. the next Family-1 slice can be cancel/fork sequencing rather than stop cleanup.

## Risks And Mitigations

1. Risk: stop widens into active-ephemeral cancel semantics.
   Mitigation: keep v1 retained-worker-only and require exact `target_participant_id`.
2. Risk: stop invents a new lifecycle model instead of reusing existing closeout truth.
   Mitigation: route through existing stop helpers and terminal state transitions wherever possible.
3. Risk: stop silently accepts already-terminal workers and hides real state errors.
   Mitigation: add explicit already-terminal denial and pin it in tests.
4. Risk: stop transport and detached closeout paths diverge semantically.
   Mitigation: hold the success criterion on authoritative stopped state, not on transport implementation details alone.
5. Risk: the slice leaks into fork or approval policy.
   Mitigation: keep payload and outcome narrow and document those flows as explicitly deferred.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because target resolution needs a frozen stop contract and policy action.
2. Packet 2 before Packet 3 because dispatch wiring should consume one trusted stop target-resolution path.
3. Packet 3 before Packet 4 because docs and validation should reflect the final routed behavior.

### Can be parallelized later

1. additional regression expansion across shell stop/reporting surfaces after the stop contract is stable,
2. docs wording and trace/config alignment during the final packet once the stop contract is frozen.

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
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice 36

If Slice `36` lands cleanly, the next Family-1 order should be:

1. `cancel_world_work` as the next broader execution-affecting later verb,
2. `fork_world_worker` only after that or only when the repo is ready to freeze fork lineage and autonomy policy,
3. broader approval/fork autonomy work after the later verb family is real,
4. Family-2 router/attach implementation only as its own downstream execution track.

That follow-on order is part of this plan discipline. It is not part of this slice’s implementation scope.
