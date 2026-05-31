# PLAN-33: Internal Retained World Worker Continue And Event Bootstrap

Source spec: [SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)  
Source validation note: [NOTE-33-family-1-ordering-after-dispatch-bootstrap.md](./NOTE-33-family-1-ordering-after-dispatch-bootstrap.md)  
Plan type: second implementation-bearing family-1 control-plane slice  
Status: implemented and validated on `2026-05-31`

## Objective

Ship the first retained-worker follow-up slice for family 1 by adding an internal `continue_world_worker` control-plane path and a minimal typed worker-event contract on top of the already-landed retained member-turn seam.

This slice is complete only when all of the following are true:

1. the host orchestrator can continue one exact retained worker through an internal control-plane verb,
2. the path reuses the existing retained member-turn/runtime seam,
3. typed worker-event classification exists for the minimal in-scope event set,
4. attention-driving events remain explicit and typed,
5. public human caller surfaces remain unchanged,
6. fuller host-to-world steering policy hardening remains deferred rather than partially implied.

## Plan Summary

The repo already has the difficult runtime primitives needed for this slice:

1. authoritative retained-member routing and world-binding validation in the runtime seam,
2. exact public retained-member follow-up over `member_turn_submit`,
3. slice-32 internal dispatch contract scaffolding and authoritative orchestrator caller validation,
4. existing worker stream/thread metadata from the agent runtime,
5. a durable family-2 obligation/inbox stack that can remain adjacent rather than being reopened here.

What this slice was responsible for landing was the internal retained-worker continue contract that lets the host orchestrator steer a real retained worker through typed follow-up outcomes instead of falling back to human/operator caller paths.

The narrowest honest implementation order is therefore:

1. freeze the internal `continue_world_worker` request/target contract first,
2. reuse the existing retained member-turn seam for the actual continued turn,
3. classify the minimal worker-event subset needed for typed messaging truth,
4. finish with the minimum gating, docs, and validation needed to keep the slice honest.

That work is now landed on the current tree:

1. `continue_world_worker` exists as the internal retained-worker follow-up verb,
2. the path reuses the existing retained member-turn/runtime seam,
3. the first typed worker-event subset is classified explicitly, and
4. later steering-policy hardening remains the next follow-on instead of being partially implied here.

Messaging/continue comes before fuller steering-policy hardening because the repo already has hard identity boundaries but still lacks the first concrete retained-worker follow-up verb. The policy matrix is safer to harden after this slice makes the exact continue/event surface real instead of gating a hypothetical future contract.

## Locked Decisions

### What changes

1. add `continue_world_worker` as the next internal family-1 verb,
2. add a typed internal request/outcome shape for continued retained-worker turns,
3. add exact retained-worker target validation,
4. add typed classification for the minimal worker-event subset in scope,
5. keep surfaced `thread_id` semantics explicit where available.

### What does not change

1. no new public human CLI,
2. no toolbox execution-plane rollout,
3. no `inspect_world_worker`,
4. no `cancel_world_work`,
5. no `stop_world_worker`,
6. no `fork_world_worker`,
7. no approval or fork event classes in the first slice,
8. no fuller config/policy-matrix hardening in this slice,
9. no family-2 schema or router changes.

## Implementation Order

### Packet 1: Internal Continue Contract And Exact Target Resolution

Goal:

1. add `continue_world_worker` to the internal family-1 contract,
2. define the exact retained-worker target fields and payload shape,
3. fail closed before any world routing happens.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Why first:

1. retained-worker follow-up must be exact-targeted from day one,
2. later packets should build against a frozen continue contract instead of ad hoc helpers,
3. the policy-hardening follow-on depends on this slice exposing concrete axes to harden.

Verification checkpoint:

1. unsupported action/payload combinations fail closed,
2. exact retained-worker targeting is mandatory,
3. no public CLI behavior changes.

### Packet 2: `continue_world_worker` Over The Existing Retained Member-Turn Seam

Goal:

1. route `continue_world_worker` through the already-landed retained member-turn path,
2. preserve exact retained worker identity and world binding,
3. avoid inventing a second transport or runtime path.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
3. `crates/world-service/src/member_runtime.rs`
4. targeted shell/world-service tests

Why second:

1. the runtime seam already exists and is the truthful reuse path,
2. this proves the internal continue caller surface without yet widening into richer policy or lifecycle verbs,
3. typed event classification should happen after the raw routing path is real.

Verification checkpoint:

1. exact retained-member follow-up works through the existing runtime seam,
2. invalid session/world/participant/backend combinations fail closed,
3. no alternate execution plane has appeared.

### Packet 3: Minimal Typed Worker-Event Classification

Goal:

1. normalize the first worker-event subset into a typed internal outcome,
2. surface `attention_required` explicitly for unresolved attention-driving classes,
3. keep the slice narrower than the full later steering and policy matrix.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. `crates/shell/src/execution/prompt_fulfillment.rs` only if shared parsing reuse stays narrow
4. targeted shell tests

Event classes in scope:

1. `reply`
2. `progress_update`
3. `follow_up_question`
4. `blocked`
5. `result`
6. `failure`

Why third:

1. the routing path should be real before event normalization is frozen,
2. this captures the truthful messaging seam without widening into approvals, fork, or richer operational control,
3. the later steering-policy slice should harden against these concrete event classes and denial buckets.

Verification checkpoint:

1. in-scope event classes map to stable typed outcomes,
2. attention-driving classes remain explicit,
3. approval and fork classes are still out of scope.

### Packet 4: Minimal Gating, Docs, And Validation

Goal:

1. lock the minimum safe boundary for the first retained-worker follow-up slice,
2. align docs and planning notes with the actual in-tree runtime truth,
3. run the validation wall.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `docs/TRACE.md`
4. `llm-last-mile/NOTE-33-family-1-ordering-after-dispatch-bootstrap.md`
5. targeted test suites

What this packet must enforce:

1. orchestrator-only caller identity,
2. same-session-only,
3. same-world-binding-only,
4. exact backend-plus-retained-participant targeting,
5. stable deny buckets for unsupported continue/event shapes.

What this packet explicitly does not add:

1. full config-schema policy hardening,
2. approval or fork event classes,
3. inspect/cancel/stop/fork verbs,
4. new family-2 durable schema or router behavior.

Verification checkpoint:

1. docs remain honest about the surface being internal and narrow,
2. validation is green,
3. the next family-1 slice can safely be policy hardening instead of contract repair.

## Risks And Mitigations

1. Risk: the slice widens into the whole later messaging and steering matrix.
   Mitigation: keep `continue_world_worker` as the only new verb and keep event classes to the minimal subset above.
2. Risk: approvals or fork autonomy get pulled into the first follow-up slice.
   Mitigation: treat approval/fork classes as explicit out-of-scope denials for this slice.
3. Risk: implementation invents a second retained-worker turn transport.
   Mitigation: require reuse of the existing retained member-turn/runtime seam.
4. Risk: attention semantics drift into family-2 redesign.
   Mitigation: keep attention as typed event truth only; do not broaden the obligation ledger or router architecture here.
5. Risk: policy hardening starts getting implemented ad hoc inside this slice.
   Mitigation: preserve only the exact identity and boundary checks already required for safe routing, and defer configurable matrix work to the next slice.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because routing needs the frozen continue contract and exact target rules.
2. Packet 2 before Packet 3 because typed event classification should target a real in-tree continue path.
3. Packet 3 before Packet 4 because docs and final gates should reflect the actual typed event contract that landed.

### Can be parallelized later

1. targeted shell and world-service regression expansion after the continue contract is stable,
2. doc wording and trace alignment during the final packet once the runtime contract is frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice 33

If slice `33` lands cleanly, the next family-1 order should be:

1. fuller host-to-world steering policy hardening against the concrete continue/event surface,
2. later verb expansion such as inspect/cancel/stop/fork only after that hardening exists,
3. any later approval or worker-requested fork autonomy work after the above.

That follow-on order is part of the plan discipline for this repo. It is not part of this slice's implementation scope.
