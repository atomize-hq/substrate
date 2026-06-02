# PLAN-38: Internal Retained World Worker Fork

Source spec: [SPEC-38-internal-retained-world-worker-fork.md](./SPEC-38-internal-retained-world-worker-fork.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: seventh implementation-bearing Family-1 control-plane slice  
Status: proposed on `2026-06-02`

## Objective

Ship the first fork-bearing Family-1 control-plane slice by adding an internal exact-source `fork_world_worker` verb that allocates one retained child worker with explicit source-to-child lineage, without widening into worker-requested fork autonomy, active-ephemeral task identity, or Family-2 router/attach work.

This slice is complete only when all of the following are true:

1. `fork_world_worker` exists as a typed internal dispatch action,
2. the action is retained-only in v1 and requires exact retained source `target_participant_id`,
3. steering policy can explicitly allow or deny the action,
4. allowed Linux requests allocate one retained child through the existing retained bootstrap seam and return a typed fork outcome with explicit lineage,
5. the child remains in the same authoritative session/world binding,
6. worker-requested fork, fork recommendations, auto-fork, active-ephemeral inspect/cancel widening, approval/fork autonomy, and Family-2 router/attach work remain deferred.

## Plan Summary

The repo already has the prerequisites that make `fork_world_worker` the next honest Family-1 slice:

1. six typed internal dispatch actions are already landed: `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `inspect_world_worker`, `cancel_world_work`, and `stop_world_worker`,
2. exact orchestrator/session/worker/world-binding validation is already a live contract,
3. the deny-by-default steering-policy floor is already landed,
4. retained worker bootstrap already exists through `spawn_world_worker`,
5. spawn transport/outcome surfaces already carry lineage-adjacent `parent_participant_id` and `resumed_from_participant_id` fields even though direct spawn currently leaves them unset,
6. retained worker caps are already enforced through the live steering-policy concurrency model.

What the repo still lacks before Slice `38` can land is:

1. a typed internal `fork_world_worker` action,
2. policy-model allowlist support for the new action,
3. authoritative retained source-worker resolution for fork,
4. typed child-lineage outcome truth,
5. routed child allocation that reuses retained bootstrap without collapsing into plain spawn.

That matters because the post-Slice-37 design vocabulary is now missing one verb, not missing another widening of already-landed verbs. The live repo still rejects `inspect_world_worker` and `cancel_world_work` in `mode=ephemeral` and still exposes no typed task identity on `run_world_task`, so active-ephemeral widening remains later identity-model work rather than the next missing verb slice.

The narrowest honest implementation order is therefore:

1. freeze `fork_world_worker` as a typed retained-only action and allowlist surface first,
2. add authoritative retained source-target resolution and explicit child-lineage truth,
3. route fork through the internal dispatch layer by reusing the retained bootstrap seam,
4. finish with docs and the validation wall.

Broader worker-requested fork autonomy does not come first because the live runtime still treats `fork_request`, `fork_recommendation`, and related approval/control labels as deferred wire classes, and the current policy model still lacks dedicated fork-autonomy keys. Family 2 does not come first because it remains downstream of the Family-1 control-plane vocabulary.

## Locked Decisions

### What changes

1. add `fork_world_worker` to the internal dispatch action vocabulary,
2. add a typed fork payload and typed fork outcome contract,
3. widen steering-policy allowlist parsing so the action can be explicitly permitted,
4. add authoritative retained source-worker resolution with fail-closed non-terminal eligibility checks,
5. reuse existing retained bootstrap routing so fork allocates a child instead of inventing a second child-allocation plane,
6. return explicit source-to-child lineage in the typed outcome.

### What does not change

1. no new public human CLI,
2. no worker-requested fork, fork recommendations, or auto-fork in this slice,
3. no active-ephemeral child mode in this slice,
4. no active-ephemeral inspect/cancel widening,
5. no approval-request or fork-request autonomy rollout,
6. no new Family-2 router/attach execution work,
7. no dedicated fork policy-schema expansion beyond allowlisting the new action in the current generic steering-policy model.

## Implementation Order

### Packet 1: Fork Contract And Policy Allowlist Expansion

Goal:

1. freeze the fork action, payload, and outcome contract,
2. allow steering policy to explicitly admit `fork_world_worker`,
3. keep deny-by-default behavior intact when the action is not allowlisted,
4. freeze v1 scope as host-initiated retained-to-retained fork only.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. targeted contract and policy tests

Why first:

1. the runtime needs a frozen fork action and outcome shape before child-lineage routing can stay coherent,
2. steering-policy parsing must know the action before dispatch enforcement can reuse it,
3. later packets should reuse one stable action string and one stable typed fork outcome.

Verification checkpoint:

1. `fork_world_worker` is a valid internal dispatch action,
2. it validates as retained-only with exact source identity,
3. the typed fork outcome freezes explicit source and child identity expectations,
4. effective policy parsing can explicitly allow the action without relaxing deny-by-default defaults,
5. REPL-facing internal toolbox fork ingress still proves validation and steering-policy gating happen before the Packet 1 unsupported-dispatch stub.
   Required regression anchors:
   `orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_one_unsupported_dispatch`,
   `orchestrator_world_dispatch_surface_validates_fork_requests_before_packet_one_unsupported_dispatch`,
   `orchestrator_world_dispatch_surface_rejects_denied_fork_requests_before_packet_one_unsupported_dispatch`

### Packet 2: Retained Source Resolution And Lineage Truth

Goal:

1. resolve exact retained source-worker fork targets from authoritative state-store truth,
2. reject invalidated or terminal source workers explicitly,
3. freeze source-to-child lineage truth distinct from plain spawn semantics.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/session.rs`
3. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
4. targeted lifecycle and state-store tests

Why second:

1. fork is fundamentally an exact-source identity and lineage-registration problem before it is a routing problem,
2. the current tree already has retained child bootstrap but not fork-specific lineage truth,
3. getting this right keeps fork from degenerating into an untraceable second spawn path.

Verification checkpoint:

1. exact retained source-worker targets resolve only inside the authoritative session/world binding,
2. invalidated or terminal source workers fail closed,
3. source-to-child lineage truth is explicit and distinct from plain spawn outcomes.

### Packet 3: Internal Dispatch Wiring And Child Allocation

Goal:

1. route `fork_world_worker` through the internal dispatch layer,
2. enforce steering policy before executing fork allocation,
3. allocate one retained child through the existing retained bootstrap seam and persist explicit lineage.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/repl/async_repl.rs`
4. targeted shell integration tests

Why third:

1. dispatch wiring should consume a frozen contract and a trusted source resolver,
2. bootstrap reuse should be verified against the final routed behavior, not only in isolation,
3. this packet proves fork is a real control-plane verb rather than a doc-only placeholder.

Verification checkpoint:

1. disallowed fork requests fail closed through the steering-policy layer,
2. allowed fork requests allocate one retained child and return explicit lineage,
3. fork does not widen into worker-requested autonomy, plain spawn-without-lineage, or active-ephemeral child mode.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the now-frozen fork scope,
2. keep host-initiated retained-to-retained v1 fork and all broader fork/autonomy deferrals explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/NOTE-37-family-1-ordering-after-cancel-closeout.md`
3. `llm-last-mile/SPEC-38-internal-retained-world-worker-fork.md`
4. `llm-last-mile/PLAN-38.md`
5. `llm-last-mile/TASKS-38.md`
6. targeted test suites

What this packet must enforce:

1. docs describe fork as internal, host-initiated, exact-source, retained-to-retained in Slice `38`,
2. docs keep worker-requested fork, fork recommendations, auto-fork, approval autonomy, active-ephemeral widening, and Family-2 work explicitly deferred,
3. no wording implies dedicated fork-autonomy policy keys or deferred worker event classes have landed if the implementation only freezes the narrow host-initiated fork slice.

Verification checkpoint:

1. docs remain honest about scope and deferrals,
2. validation is green,
3. the next Family-1 slice can move to broader fork autonomy or later identity-model widening rather than fork cleanup.

## Risks And Mitigations

1. Risk: fork silently becomes plain spawn with no explicit lineage.
   Mitigation: require a typed fork outcome with explicit source and child identity.
2. Risk: Slice `38` quietly widens into worker-requested fork autonomy.
   Mitigation: keep deferred event classes deferred and rely only on the current action allowlist plus retained caps.
3. Risk: fork tries to solve active-ephemeral task identity at the same time.
   Mitigation: keep the slice retained-to-retained only and defer active-ephemeral widening explicitly.
4. Risk: child allocation invents a second retained bootstrap plane.
   Mitigation: reuse the existing retained bootstrap seam and constrain the slice to lineage-aware routing on top of it.
5. Risk: fork forces dedicated policy-schema keys immediately.
   Mitigation: keep Slice `38` on the already-landed generic steering-policy surface and defer autonomy-specific schema.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because source resolution and lineage truth need a frozen fork contract and policy action.
2. Packet 2 before Packet 3 because dispatch wiring should consume one trusted source-resolution path and explicit lineage semantics.
3. Packet 3 before Packet 4 because docs and validation should reflect final routed behavior.

### Can be parallelized later

1. additional regression expansion across fork lineage/reporting surfaces after the fork contract is stable,
2. docs wording and config alignment during the final packet once the routed fork semantics are frozen.

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

## Expected Follow-On Order After Slice `38`

If Slice `38` lands cleanly as narrow host-initiated retained-to-retained fork first, the next Family-1 follow-on work should remain:

1. broader worker-requested fork autonomy and approval-policy widening only if still needed,
2. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
3. Family-2 router/attach implementation only as its own downstream execution track.

That follow-on order is part of this plan discipline. It is not part of Slice `38` implementation scope.
