# PLAN-34: Host-To-World Steering Policy Hardening For Landed Dispatch Surface

Source spec: [SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md](./SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md)  
Source validation note: [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)  
Plan type: third implementation-bearing family-1 control-plane slice  
Status: implemented and validated on `2026-05-31`

## Objective

Ship the first implementation-bearing host-to-world steering-policy layer by hardening the already-landed internal dispatch surface with deny-by-default authorization that remains separate from execution-plane runtime capability truth.

This slice is complete only when all of the following are true:

1. `run_world_task`, `spawn_world_worker`, and `continue_world_worker` are gated by an explicit steering-policy layer before world routing runs,
2. the steering-policy layer can deny by enablement, backend, action, mode, exact boundary truth, and current routability/concurrency truth where in scope,
3. denials are explanation-ready and stable,
4. the runtime seam remains unchanged for allowed requests,
5. public human caller surfaces remain unchanged,
6. later family-1 verbs and family-2 attach/router work remain deferred rather than partially implied.

## Plan Summary

The repo already has the hard runtime substrate:

1. typed internal dispatch contract for `run_world_task`, `spawn_world_worker`, and `continue_world_worker`,
2. exact orchestrator/session/worker/world-binding validation,
3. retained-worker event bootstrap for the minimal current continue surface,
4. a local-first obligation/attach/router design stack that remains downstream of control-plane authorization.

What the repo still lacks is the separate steering-policy layer that decides whether these already-real verbs may run at all.

The narrowest honest implementation order is therefore:

1. freeze the implementation-bearing steering-policy dimensions and stable deny buckets first,
2. apply those gates to the landed verbs before any world routing happens,
3. add lifecycle-aware and concurrency-aware denials for the current three-verb surface,
4. finish with docs, config truth, and the validation wall.

Later verb expansion does not come first because the steering-policy design explicitly exists to harden the current control plane before new verbs widen it. Family-2 router/attach work does not come first because it depends on obligations and attach recovery, not on replacing missing control-plane authorization.

## Locked Decisions

### What changes

1. add a distinct steering-policy layer for the currently landed family-1 verb surface,
2. add a narrow implementation-bearing policy/config surface for the in-scope world-dispatch dimensions,
3. add stable denial buckets for steering enablement, action, mode, backend, boundary, invalidation, and concurrency failures,
4. harden `continue_world_worker` against invalidated or otherwise non-routable retained workers as a policy-visible deny case,
5. keep the landed runtime seam as the sole execution path.

### What does not change

1. no new public human CLI,
2. no router/daemon attach implementation,
3. no obligation-ledger redesign,
4. no `inspect_world_worker`,
5. no `cancel_world_work`,
6. no `stop_world_worker`,
7. no `fork_world_worker`,
8. no approval-request or fork-request autonomy policy rollout,
9. no second execution plane.

## Implementation Order

### Packet 1: Steering Policy Contract And Stable Denial Vocabulary

Goal:

1. freeze the implementation-bearing policy dimensions for the current family-1 surface,
2. define stable steering denial buckets,
3. keep the policy layer separate from runtime capability resolution.

Primary touch surface:

1. `crates/broker/src/policy.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
4. targeted policy/config tests

Why first:

1. the runtime needs a frozen policy shape before it can enforce anything consistently,
2. later packets should deny through one shared vocabulary instead of ad hoc strings,
3. docs and tests need stable names for the denial dimensions.

Verification checkpoint:

1. the effective policy model has a narrow implementation-bearing world-dispatch surface for current verbs,
2. defaults remain deny-by-default,
3. denial vocabulary is stable and testable.

### Packet 2: Pre-Routing Policy Enforcement For Current Verbs

Goal:

1. apply the steering-policy layer before any world routing happens,
2. gate `run_world_task`, `spawn_world_worker`, and `continue_world_worker` by enablement, action, mode, backend, and exact boundary truth,
3. preserve the current runtime path for allowed requests.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
4. targeted shell integration tests

Why second:

1. this is the first place where the separate policy layer becomes runtime truth,
2. it proves the policy-vs-execution separation against the landed verbs,
3. later lifecycle-aware and concurrency-aware checks should build on the same gate.

Verification checkpoint:

1. disallowed action/mode/backend cases fail closed before routing,
2. same-session and same-world-binding checks remain explicit,
3. allowed requests still flow through the existing runtime seam.

### Packet 3: Lifecycle-Aware And Concurrency-Aware Hardening

Goal:

1. add policy-visible denials for invalidated/non-routable retained workers,
2. enforce the first in-scope concurrency caps for `run_world_task` and `spawn_world_worker`,
3. keep already-landed continue-event truth narrow without widening into approval/fork or family-2 producer behavior.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
4. targeted shell tests

Why third:

1. concurrency and routability checks should layer on top of the shared pre-routing gate,
2. the lifecycle doc now has concrete runtime truth for invalidation and retained routability,
3. this is the narrowest place to harden current continue behavior without widening into later verbs or family-2 persistence.

Verification checkpoint:

1. invalidated/non-routable retained workers fail with stable policy-visible denials,
2. in-scope concurrency caps are enforced with stable denials,
3. approval/fork/control-directive expansion remains deferred.

### Packet 4: Config/Docs Alignment And Final Validation

Goal:

1. align the repo-local docs with the actual policy surface that landed,
2. make trace/config wording honest and narrow,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `docs/TRACE.md`
3. `llm-last-mile/NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md`
4. `llm-last-mile/SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md`
5. `llm-last-mile/PLAN-34.md`
6. `llm-last-mile/TASKS-34.md`
7. targeted test suites

What this packet must enforce:

1. docs describe the surface as internal and current-verb-only,
2. docs keep control-plane authorization separate from runtime capability truth,
3. no wording implies router/attach work, approval policy, or later verbs have already landed.

What this packet explicitly does not add:

1. broad new public policy UX,
2. attach/review workflow execution,
3. inspect/cancel/stop/fork verbs,
4. approval or fork autonomy policy.

Verification checkpoint:

1. docs remain honest about scope and deferrals,
2. validation is green,
3. the next family-1 slice can be later verb expansion rather than policy repair.

## Risks And Mitigations

1. Risk: the slice widens into a full policy-schema redesign unrelated to current verbs.
   Mitigation: keep the landed policy surface narrowly scoped to current world-dispatch dimensions for `run_world_task`, `spawn_world_worker`, and `continue_world_worker`.
2. Risk: implementation blends steering authorization with execution-plane runtime capability checks.
   Mitigation: evaluate steering policy before routing and keep capability-resolution truth as a separate step.
3. Risk: later verb semantics leak in through generic policy abstractions.
   Mitigation: keep later verbs explicit out-of-scope denials instead of adding half-supported scaffolding.
4. Risk: concurrency or invalidation checks become implicit runtime accidents rather than policy-visible denials.
   Mitigation: add stable denial buckets and pin them in tests.
5. Risk: family-2 obligation/router behavior is dragged into the policy slice.
   Mitigation: keep current continue-event truth as typed outcomes only and do not introduce new obligation or attach execution behavior in this slice.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because enforcement needs a frozen policy contract and denial vocabulary.
2. Packet 2 before Packet 3 because lifecycle/concurrency checks should plug into one real gate instead of inventing side paths.
3. Packet 3 before Packet 4 because docs and validation should reflect the final hardened surface.

### Can be parallelized later

1. additional regression expansion across shell/world-service after the gate shape is stable,
2. doc wording and trace/config alignment during the final packet once the effective policy surface is frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice 34

If slice `34` lands cleanly, the next family-1 order should be:

1. later verb expansion on top of the hardened policy layer, starting with the smallest truthful next control verbs,
2. only after that, broader approval/fork autonomy work and any policy widening those verbs require,
3. family-2 router/attach implementation only as its own downstream execution track, not as a substitute for family-1 control-plane hardening.

That follow-on order is part of the plan discipline for this repo. It is not part of this slice’s implementation scope.
