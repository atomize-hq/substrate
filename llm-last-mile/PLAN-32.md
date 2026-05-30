# PLAN-32: Internal Host-Orchestrator World Dispatch Bootstrap

Source spec: [SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md)  
Source validation note: [REMAINING-family-1-scope-2026-05-30.md](./REMAINING-family-1-scope-2026-05-30.md)  
Plan type: first implementation-bearing family-1 control-plane slice  
Status: implemented and validated on `2026-05-30`

## Objective

Ship the first internal host-orchestrator to world control-plane slice by introducing a narrow orchestrator-only dispatch bootstrap surface that reuses the existing world-member runtime seam.

This slice is complete only when all of the following are true:

1. the host orchestrator has a real internal control-plane entry seam for initial world dispatch,
2. only `run_world_task` and `spawn_world_worker` are added,
3. exact identity and authoritative world binding are enforced before launch,
4. the slice returns typed outcomes and authoritative child identity,
5. current public human caller surfaces remain unchanged,
6. retained-worker messaging and fuller steering policy remain deferred rather than partially implied.

## Plan Summary

The repo already has the difficult runtime primitives:

1. exact world-member launch through `member_dispatch`,
2. exact retained follow-up through `member_turn_submit`,
3. authoritative session and world-binding truth in `state_store.rs`,
4. narrow public and REPL caller surfaces for humans/operators,
5. introspection-only toolbox posture.

What it does not have is the internal control-plane bridge that would let a host orchestrator call into those primitives directly.

The narrowest honest implementation order is therefore:

1. establish the internal caller surface and typed request/outcome contract,
2. land `run_world_task` first as the smallest end-to-end dispatch verb,
3. land `spawn_world_worker` second as the retained bootstrap verb,
4. finish with the minimum safety gating, audit, and validation needed to keep the slice truthful.

Messaging/steering does not come first because it depends on retained worker identity and a real dispatch entry seam. Policy does not come first because the fuller matrix is safest to harden against a concrete allocation surface instead of an entirely speculative one.

## Locked Decisions

### What changes

1. add a minimal internal orchestrator-only caller surface,
2. add typed request/outcome contract support for:
   - `run_world_task`
   - `spawn_world_worker`
3. add exact-identity validation and explanation-ready denials for those verbs,
4. reuse the existing world-member launch path rather than creating a separate executor.

### What does not change

1. no new public human CLI,
2. no general toolbox rollout,
3. no retained-worker messaging/event protocol,
4. no worker-requested fork,
5. no public selector widening,
6. no family-2 obligation-ledger or auto-attach changes.

## Implementation Order

### Packet 1: Internal Caller Surface And Shared Dispatch Contract

Goal:

1. create the minimal orchestrator-only control-plane/toolbox entry seam,
2. define a shared typed request/outcome contract for allocation/bootstrap,
3. pin exact required identity and action/mode validation before execution exists.

Primary touch surface:

1. new internal control-plane/toolbox module under `crates/shell/src/execution/`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `docs/TRACE.md` if toolbox/control-plane audit rows are introduced

Why first:

1. family 1 is specifically missing the host-orchestrator caller seam,
2. both in-scope verbs need one contract and one identity model,
3. later packets should implement against a frozen caller shape rather than inventing per-verb shortcuts.

Verification checkpoint:

1. request validation rejects unsupported action/mode combinations,
2. caller resolution requires authoritative orchestrator/session truth,
3. no public CLI behavior changes.

### Packet 2: `run_world_task` Ephemeral Bootstrap

Goal:

1. route `run_world_task` over the existing world execution seam,
2. return a typed terminal outcome,
3. keep the behavior one-shot and not silently retained.

Primary touch surface:

1. internal control-plane/toolbox module
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
4. `crates/world-service/src/member_runtime.rs`
5. targeted shell/world-service tests

Why second:

1. it is the smallest end-to-end dispatch verb,
2. it exercises the new caller seam without retained lifecycle complexity,
3. it proves the request/outcome model before retained worker receipts are added.

Verification checkpoint:

1. exact backend launch works through the existing runtime seam,
2. the typed outcome is terminal and explanation-ready,
3. invalid session/world-binding/backend cases fail closed.

### Packet 3: `spawn_world_worker` Retained Bootstrap

Goal:

1. route retained worker allocation through the same caller seam,
2. return authoritative retained worker identity and launch receipt,
3. keep ongoing steering explicitly out of scope.

Primary touch surface:

1. internal control-plane/toolbox module
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
5. `crates/world-service/src/member_runtime.rs`

Why third:

1. it builds directly on the allocation contract proven by Packet 2,
2. retained worker launch is more complex because it must return durable child identity,
3. keeping messaging out of scope makes the retained slice narrow and reviewable.

Verification checkpoint:

1. retained worker launch returns authoritative `participant_id` and lineage fields,
2. exact backend identity remains preserved,
3. no implicit `continue_world_worker` or public follow-up widening appears.

### Packet 4: Minimal Gating, Audit, Docs, And Validation

Goal:

1. land the minimum safety boundary needed for a truthful first slice,
2. align docs and trace expectations, including the shipped asymmetry between authoritative retained-worker bootstrap visibility and terminal-only `run_world_task` visibility,
3. run the validation wall.

Primary touch surface:

1. internal control-plane/toolbox module
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `docs/TRACE.md`
4. `docs/USAGE.md`
5. test suites in `crates/shell/tests/` and `crates/world-service/tests/`

What this packet must enforce:

1. orchestrator-only caller identity,
2. same-session-only,
3. same-world-binding-only,
4. exact backend targeting,
5. stable deny buckets for unsupported actions and invalid bindings.

What this packet explicitly does not add:

1. full config-schema policy matrix,
2. fork autonomy,
3. retained-worker follow-up messaging,
4. obligation-ledger event production.

Verification checkpoint:

1. trace/audit rows are explanation-ready if introduced,
2. docs describe the surface as internal and bootstrap-only,
3. validation wall is green.

## Risks And Mitigations

1. Risk: the slice quietly expands into a broad toolbox rollout.
   Mitigation: keep the caller surface orchestrator-only and limited to two verbs.
2. Risk: retained-worker messaging leaks into the bootstrap slice.
   Mitigation: treat any `continue`, `approval`, `question`, `blocked`, or `fork` path as explicitly out of scope and fail closed.
3. Risk: the slice invents a second world execution plane.
   Mitigation: require reuse of `member_dispatch` / current world execution plumbing.
4. Risk: minimal gating is too weak and the slice becomes unsafe to ship.
   Mitigation: keep same-session, same-world-binding, exact backend, and orchestrator-only checks mandatory from the first packet.
5. Risk: family-2 obligations start being written opportunistically from dispatch outcomes.
   Mitigation: keep `run_world_task` and `spawn_world_worker` on immediate typed outcomes only in this slice.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packets 2 and 3 because both verbs need the caller surface and shared contract first.
2. Packet 2 before Packet 3 because the ephemeral one-shot path is the simpler proof of the dispatch bootstrap.
3. Packet 3 before Packet 4 because docs and audit should reflect the final in-scope verb set.

### Can be parallelized later

1. integration test expansion across shell and world-service after the request/outcome contract is stable,
2. docs and trace wording during the final packet after the runtime contract is frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice 32

If slice `32` lands cleanly, the next family-1 order should be:

1. retained-world-worker messaging and steering,
2. fuller host-to-world steering policy hardening,
3. any later fork or worker-request autonomy work.

That follow-on order is part of the plan discipline for this repo. It is not part of this slice's implementation scope.
