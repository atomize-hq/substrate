# PLAN-35: Internal Retained World Worker Inspect Snapshot

Source spec: [SPEC-35-internal-retained-world-worker-inspect-snapshot.md](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)  
Source validation note: [REMAINING-family-1-scope-2026-05-31-post-slice-34.md](./REMAINING-family-1-scope-2026-05-31-post-slice-34.md)  
Plan type: fourth implementation-bearing Family-1 control-plane slice  
Status: implemented and validated on `2026-06-01`  
Landed posture note: the inspect contract, policy allowlisting, and ingress validation are landed repo-wide, but retained-worker inspect snapshot routing is Linux-only in v1 and fails closed on non-Linux builds.

## Objective

Ship the first inspect-bearing Family-1 control-plane slice by adding an internal retained-worker `inspect_world_worker` verb that returns authoritative snapshot truth without introducing a new execution plane or widening into cancellation, stopping, or fork policy.

This slice is complete only when all of the following are true:

1. `inspect_world_worker` exists as a typed internal dispatch action,
2. the action is retained-worker-only in v1 and requires exact `target_participant_id`,
3. steering policy can explicitly allow or deny the action,
4. allowed Linux requests return a typed authoritative retained-worker snapshot,
5. the snapshot is non-mutating and store-backed,
6. active-ephemeral inspect, cancel, stop, fork, approval/fork autonomy, and Family-2 router/attach work remain deferred.

## Plan Summary

The repo already has the prerequisites that make inspect the smallest honest next slice:

1. typed internal dispatch contract for `run_world_task`, `spawn_world_worker`, and `continue_world_worker`,
2. exact orchestrator/session/worker/world-binding validation,
3. the deny-by-default steering-policy floor,
4. authoritative stored runtime/session snapshots that already drive status-facing logic elsewhere in the shell.

What the repo still lacks is a typed internal inspect action that exposes that authoritative retained-worker truth directly to the host orchestrator.

Current landed runtime note:

1. contract and policy admission are not Linux-specific,
2. the routed retained-worker snapshot outcome is Linux-only in v1,
3. non-Linux builds reject retained inspect routing rather than approximating it.

The narrowest honest implementation order is therefore:

1. freeze the inspect action, payload, outcome, and policy allowlist expansion first,
2. add authoritative retained-worker inspect target resolution and snapshot projection,
3. wire the new inspect action through the internal dispatch layer without adding a new runtime transport,
4. finish with docs and the validation wall.

`cancel_world_work` and `stop_world_worker` do not come first because they mutate lifecycle state and therefore commit the repo to larger transition semantics. `fork_world_worker` does not come first because it drags in lineage and autonomy policy. Active-ephemeral inspect does not come first because it would require an additional exact task identity surface that is not yet part of the landed control plane.

## Locked Decisions

### What changes

1. add `inspect_world_worker` to the internal dispatch action vocabulary,
2. add a typed inspect payload and typed inspect outcome contract,
3. widen steering-policy allowlist parsing so the action can be explicitly permitted,
4. add authoritative retained-worker inspect resolution and snapshot projection from stored runtime truth,
5. keep the existing runtime steering transport untouched.

### What does not change

1. no new public human CLI,
2. no live world-service inspect RPC,
3. no active-ephemeral inspect in this slice,
4. no `cancel_world_work`,
5. no `stop_world_worker`,
6. no `fork_world_worker`,
7. no approval-request or fork-request autonomy rollout,
8. no Family-2 router/attach execution work,
9. no lifecycle mutation as part of inspect.

## Implementation Order

### Packet 1: Inspect Contract And Policy Allowlist Expansion

Goal:

1. freeze the inspect action, payload, and outcome contract,
2. allow steering policy to explicitly admit `inspect_world_worker`,
3. keep deny-by-default behavior intact when the action is not allowlisted.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. targeted contract and policy tests

Why first:

1. the runtime needs a frozen action and outcome shape before snapshot projection can be trusted,
2. steering-policy parsing must know the action before dispatch enforcement can reuse it,
3. later packets should reuse one stable action string and one stable typed outcome.

Verification checkpoint:

1. `inspect_world_worker` is a valid internal dispatch action,
2. it validates as retained-only with exact target identity,
3. effective policy parsing can explicitly allow the action without relaxing deny-by-default defaults.

### Packet 2: Authoritative Retained-Worker Snapshot Resolution

Goal:

1. resolve exact retained-worker inspect targets from authoritative state-store truth,
2. project a typed inspect snapshot without requiring the worker to be currently attached or live-routable,
3. preserve exact session and world-binding safety.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/orchestration_session.rs` only if an existing snapshot type needs a narrow reusable helper
3. targeted state-store tests

Why second:

1. inspect is fundamentally a snapshot projection problem before it is a dispatch wiring problem,
2. the state-store layer is where live, detached, invalidated, and terminal truth already converge,
3. getting this right keeps inspect from turning into accidental continuation logic.

Verification checkpoint:

1. exact retained-worker inspect targets resolve only inside the authoritative session/world binding,
2. invalidated and terminal retained workers still produce truthful inspect snapshots,
3. no snapshot path mutates lifecycle state.

### Packet 3: Internal Dispatch Wiring And Regression Coverage

Goal:

1. route `inspect_world_worker` through the internal dispatch layer,
2. enforce steering policy before serving inspect results,
3. return the typed retained-worker snapshot through a stable inspect outcome.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/repl/async_repl.rs`
4. targeted shell integration tests

Why third:

1. dispatch wiring should consume a frozen contract and a trusted snapshot resolver,
2. this packet proves inspect is a real control-plane verb rather than a doc-only action,
3. regression coverage belongs beside the final routed behavior.

Verification checkpoint:

1. disallowed inspect requests fail closed through the steering-policy layer,
2. allowed inspect requests return the projected retained-worker snapshot,
3. inspect does not continue, cancel, stop, or fork work.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the now-landed inspect scope,
2. keep the slice honest about retained-only inspect and remaining deferrals,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/REMAINING-family-1-scope-2026-05-31-post-slice-34.md`
3. `llm-last-mile/SPEC-35-internal-retained-world-worker-inspect-snapshot.md`
4. `llm-last-mile/PLAN-35.md`
5. `llm-last-mile/TASKS-35.md`
6. targeted test suites

What this packet must enforce:

1. docs describe inspect as internal, retained-worker-only in v1, and Linux-only for routed snapshot delivery,
2. docs keep snapshot inspection separate from execution-affecting verbs,
3. no wording implies cancel, stop, fork, approval autonomy, or Family-2 work have landed.

Verification checkpoint:

1. docs remain honest about scope and deferrals,
2. validation is green,
3. the next Family-1 slice can be cancel/stop/fork sequencing rather than inspect cleanup.

## Risks And Mitigations

1. Risk: inspect widens into active-ephemeral task introspection.
   Mitigation: keep v1 retained-worker-only and require exact `target_participant_id`.
2. Risk: inspect becomes a live runtime RPC rather than a store-backed snapshot.
   Mitigation: treat authoritative persisted state as the source of truth for this slice.
3. Risk: inspect accidentally enforces “must be live” semantics that make invalidated or terminal workers uninspectable.
   Mitigation: separate inspect target resolution from continue-style routability rules and pin non-live snapshot cases in tests.
4. Risk: inspect silently grows lifecycle mutation behavior.
   Mitigation: keep the outcome read-only and add regression tests proving no continuation/cancellation/stop side effects.
5. Risk: the slice leaks into fork or approval policy.
   Mitigation: keep payload and outcome narrow and document those flows as explicitly deferred.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because snapshot projection needs a frozen inspect contract and policy action.
2. Packet 2 before Packet 3 because dispatch wiring should consume one trusted target-resolution path.
3. Packet 3 before Packet 4 because docs and validation should reflect the final routed behavior.

### Can be parallelized later

1. additional regression expansion across shell status/reporting surfaces after the inspect snapshot shape is stable,
2. docs wording and trace/config alignment during the final packet once the inspect contract is frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p substrate-broker inspect_world_worker -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice 35

If Slice `35` lands cleanly, the next Family-1 order should be:

1. the next execution-affecting later verb slice, most likely `stop_world_worker` or `cancel_world_work` depending on which proves narrower against the live lifecycle model,
2. `fork_world_worker` only after that or only when the repo is ready to freeze fork lineage and autonomy policy,
3. broader approval/fork autonomy work after the later verb family is real,
4. Family-2 router/attach implementation only as its own downstream execution track.

That follow-on order is part of this plan discipline. It is not part of this slice’s implementation scope.
