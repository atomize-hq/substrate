# Plan: Stop World Worker Authority And Fail-Closed Closeout Contract

Source spec: [SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md](./SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md)  
Primary validation memo: [../DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md](../DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md)  
Related architectural inputs:
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
- [PLAN-36.md](./PLAN-36.md)
- [SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md](./SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md)
- [PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md](./PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md)
- [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](./29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)
- [../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Plan type: bounded stop-contract correction  
Status: draft for review  
Implementation posture: correct the stop-world-worker authority, recovery, and caller-visible proof seam without widening into public CLI redesign, general attach redesign, or unrelated lifecycle work

## Objective

Implement the narrow contract correction frozen by `SPEC-64` so `stop_world_worker` behaves consistently with the repo's authority and lifecycle model.

This plan must land all of the following:

1. stop dispatch binds to orchestration-session authority rather than stale attached-host snapshots,
2. private stop transport is treated as delivery-only and never as sufficient proof of durable stop success,
3. when sanctioned recovery is available and exact-target authority still holds, one bounded authority-recovery attempt plus exactly one exact-target re-attempt occurs inside the same stop episode,
4. caller-visible success requires terminal proof for that same stop episode,
5. when durable closeout cannot be proven, the stop attempt fails closed without falsely stopping the worker,
6. Linux-first rollout posture is allowed only as sequencing and smoke-validation scope, not as contract variance.
7. caller-visible non-success stop outcomes in this slice keep the existing error/result shape and must instead guarantee stable, reviewable textual distinctions for the failure classes frozen by `SPEC-64`.

## Plan Summary

`SPEC-36` already landed the existence of the internal retained-worker stop verb. `SPEC-64` now narrows the remaining gap: current stop behavior can still drift if implementation treats the current attached host client or the private stop path as the owner of stop semantics instead of treating them as delivery details under durable orchestration-session authority.

The implementation work is therefore not a new stop feature. It is a contract repair across four linked seams:

1. authoritative stop target resolution,
2. authoritative attached-host ownership rebinding,
3. stop-episode recovery and one bounded re-attempt,
4. caller-visible success/failure proof.

For caller-visible non-success outcomes, this plan settles the only open planning choice left by `SPEC-64`: this slice does **not** introduce a new typed stop-outcome taxonomy. It keeps the existing result/error shape and requires stable textual distinctions for `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed` so implementation can stay narrow and avoid widening the caller contract beyond what this correction needs.

The narrowest honest order is:

1. freeze current failure modes and success conditions in tests first,
2. bind stop dispatch and targeting to current authoritative session and retained-worker truth,
3. implement required recovery and exact-target re-attempt logic through the existing stop delivery posture,
4. gate caller-visible success on terminal proof plus authoritative stopped closeout,
5. finish with Linux-first smoke validation and doc alignment.

Do **not** widen this slice into:

1. public `substrate agent stop` redesign,
2. `cancel_world_work` redesign,
3. router/inbox product behavior changes beyond consuming existing sanctioned recovery truth,
4. retained-worker lifecycle redesign beyond stop closeout proof,
5. new transport families or a second lifecycle model.

## Major Components And Dependencies

### 1. Authoritative target and owner resolution

Primary surfaces:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Why it matters:

1. this layer owns durable orchestration-session truth,
2. it decides whether the retained worker tuple and current authoritative attached host are still valid,
3. downstream recovery and delivery logic cannot be correct until this truth is explicit and fail-closed.

Dependencies:

1. `SPEC-64` exact-target authority tuple,
2. `ADR-0047` durable-session ownership,
3. `29.75` persisted attach-truth rules.

### 2. Stop-episode control and recovery orchestration

Primary surfaces:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Why it matters:

1. this is where one stop episode can distinguish initial delivery from sanctioned recovery,
2. this layer must enforce the required one bounded recovery attempt plus one exact-target re-attempt,
3. this is the highest-risk seam for accidentally broadening scope into general attach or transport redesign.

Dependencies:

1. authoritative target and owner resolution must already be trustworthy,
2. existing stop-closeout helpers and owner-path transport posture from Slice 36,
3. sanctioned recovery truth from the attach/session design stack.

### 3. Runtime closeout proof and lifecycle truth

Primary surfaces:

1. `crates/world-service/src/member_runtime.rs`
2. any narrow world-service stop-closeout helpers that currently surface terminal retained-worker stop state

Why it matters:

1. caller-visible success cannot rely on shell-side delivery success alone,
2. the stop episode needs authoritative retained-worker stopped truth,
3. this seam must remain consistent with `SPEC-63` and the lifecycle model's separation of lifecycle truth from transport liveness.

Dependencies:

1. existing retained-worker stop-closeout runtime truth,
2. exact retained-worker identity and binding checks,
3. no new lifecycle taxonomy.

### 4. Regression and smoke validation

Primary surfaces:

1. `crates/shell/tests/`
2. `crates/world-service/tests/`
3. existing Linux-first smoke/doctor surfaces as needed for honest post-implementation validation

Why it matters:

1. the contract correction is easy to regress by reintroducing transport-centric shortcuts,
2. recovery and caller-visible proof need joined tests, not isolated helpers only,
3. Linux-first rollout needs to be proven as implementation posture without turning into contract ambiguity.

Dependencies:

1. phases 1-3 must freeze the behavior first,
2. smoke validation depends on the routed stop path being final.

## Planning Defaults Locked For This Slice

Unless implementation evidence forces a spec/plan update before code lands, this plan assumes:

1. orchestration-session truth remains the only durable authority root,
2. attached-host ownership may be restored or replaced through sanctioned paths without changing durable session ownership,
3. the existing private stop transport family is retained as the delivery mechanism rather than replaced,
4. one bounded authority-recovery attempt plus exactly one bounded exact-target re-attempt is mandatory when `SPEC-64` says recovery is available,
5. caller-visible success requires both authoritative stopped closeout and terminal proof for that same stop episode,
6. caller-visible non-success stop outcomes keep the existing shape and distinguish failure classes through stable textual guarantees rather than a new typed taxonomy in this slice,
7. Linux-first is an implementation rollout boundary only; non-Linux remains fail-closed until an explicit later slice broadens support.

If implementation evidence forces a different contract, stop and update `SPEC-64` before code lands.

## Locked Decisions

### What changes

1. stop dispatch rebinds to current authoritative attached-host truth instead of trusting stale owner snapshots,
2. stop recovery becomes an explicit bounded episode rule rather than an implicit best-effort fallback,
3. exactly one recovery-triggered exact-target re-attempt is enforced when sanctioned recovery is available,
4. caller-visible success is gated by terminal proof plus authoritative stopped closeout,
5. caller-visible non-success outcomes preserve one existing shape but carry stable textual distinctions for the contractually relevant failure classes,
6. tests and smoke validation prove that dead or disappearing transport paths fail closed unless the required recovery path completes successfully.

### What does not change

1. no new public CLI contract,
2. no widening into `cancel_world_work`,
3. no second stop transport,
4. no general attach/router redesign,
5. no fuzzy worker selection,
6. no new retained-worker lifecycle family,
7. no new typed stop-outcome taxonomy in this slice,
8. no non-Linux stop widening in this slice.

## Implementation Order

### Phase 1: Freeze Current Failure Modes And Success Conditions

Goal:

1. make current authority-versus-transport drift reproducible in tests,
2. pin the required success rule before any implementation change,
3. establish the recovery and caller-visible proof harness first.

Primary touch surface:

1. `crates/shell/tests/`
2. `crates/world-service/tests/`
3. targeted adjacent unit tests in state-store, dispatch, or control modules as needed

Required work:

1. add or extend tests for stale attached-host owner paths,
2. add or extend tests proving dead/missing transport does not equal stopped success,
3. add or extend tests for caller-visible fail-closed when terminal proof is missing,
4. add harness coverage for the required recovery-then-one-retry path,
5. pin stable textual distinctions for the required caller-visible failure classes without introducing a new typed outcome family.

Why first:

1. this slice is correcting semantics, so the semantics need to be pinned before logic changes,
2. later implementation phases otherwise risk preserving transport-centric behavior under new names.

Verification checkpoint:

1. the negative cases are reproducible in automation,
2. the recovery path has a reviewable harness,
3. the caller-visible proof rule is pinned before implementation changes begin,
4. the textual distinction floor for non-success outcomes is pinned before implementation changes begin.

### Phase 2: Rebind Stop Dispatch To Current Authoritative Session And Worker Truth

Goal:

1. ensure stop always resolves against durable orchestration-session authority,
2. reject stale owner paths before delivery,
3. keep exact retained-worker identity and lineage checks fail-closed.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Required work:

1. resolve the current authoritative attached host from persisted/session truth at stop-dispatch time,
2. reject stale or disproven owner paths explicitly,
3. keep retained-worker exact-target resolution distinct from owner resolution,
4. preserve invalidation or fail-closed behavior when worker binding or lineage no longer matches authoritative truth.

Why second:

1. recovery and delivery logic are meaningless unless the correct owner and worker truths are established first,
2. this phase narrows the stop episode to one trusted authoritative context.

Verification checkpoint:

1. stale attached-host snapshots are rejected,
2. the exact retained-worker tuple remains required,
3. stop dispatch binds to durable session truth rather than transport discovery alone.

### Phase 3: Implement Required Recovery And One Exact-Target Re-Attempt

Goal:

1. turn recovery from an implementation accident into an explicit bounded stop-episode rule,
2. consume existing sanctioned recovery truth without widening into general attach redesign,
3. ensure only one re-attempt occurs and only for the same exact session and retained worker.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`

Required work:

1. distinguish initial delivery failure classes that qualify for required recovery from those that fail closed immediately,
2. invoke one bounded sanctioned recovery attempt when `SPEC-64` conditions hold,
3. if recovery succeeds, issue exactly one bounded exact-target stop re-attempt,
4. if recovery fails or exact-target authority can no longer be proven, fail closed without further retries.

Why third:

1. the bounded recovery rule depends on Phase 2's authoritative rebinding,
2. this is the smallest phase that can honestly satisfy the new contract without broadening scope.

Verification checkpoint:

1. one recovery attempt occurs when required,
2. at most one exact-target re-attempt occurs,
3. unsupported or disproven recovery paths fail closed without hidden fallback behavior.

### Phase 4: Gate Caller-Visible Success On Terminal Proof And Authoritative Closeout

Goal:

1. make stop success depend on both lifecycle truth and returned proof,
2. prevent shell-side delivery success from masquerading as durable closeout,
3. preserve explanation-ready fail-closed outcomes when proof is incomplete,
4. keep caller-visible non-success outcomes stable using the existing shape plus textual distinctions rather than a new typed taxonomy.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. narrow runtime stop-closeout result wiring in `crates/world-service/src/member_runtime.rs` only if needed to expose authoritative stopped proof honestly

Required work:

1. require terminal stop proof for the stop episode's caller-visible success path,
2. require authoritative stopped closeout in addition to returned proof,
3. preserve fail-closed caller-visible outcome when stop may have happened but the episode cannot prove it terminally,
4. keep later read-side observation distinct from retroactive stop-attempt success,
5. standardize the existing caller-visible non-success wording so `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed` remain stably distinguishable without changing the underlying error/result shape.

Why fourth:

1. recovery sequencing must already be final before caller-visible proof semantics can be wired honestly,
2. this phase is where transport-side optimism must finally be removed.

Verification checkpoint:

1. caller-visible success requires both proof and authoritative closeout,
2. missing terminal proof yields fail-closed caller-visible outcome,
3. later observed `stopped` state does not retroactively convert the earlier attempt to success,
4. the required non-success failure classes are stably distinguishable in caller-visible output using the existing shape.

### Phase 5: Linux-First Smoke Validation And Final Alignment

Goal:

1. prove the approved contract on the supported current rollout path,
2. keep Linux-first posture as implementation sequencing only,
3. align local docs and plan artifacts with the corrected stop contract.

Primary touch surface:

1. Linux-first shell/world-service smoke surfaces and existing regression suites
2. `llm-last-mile/` planning artifacts and any narrow local docs that describe stop semantics

Required work:

1. validate a positive path where sanctioned recovery succeeds and the exact-target re-attempt reaches terminal stopped proof,
2. validate a negative path where delivery dies and no durable closeout can be proven, producing fail-closed outcome,
3. confirm non-Linux remains fail-closed rather than approximating success through alternate semantics,
4. align plan/spec notes so Linux-first reads as rollout posture, not contract ambiguity.

Why fifth:

1. smoke validation should exercise the final routed behavior, not intermediate semantics,
2. docs and closeout notes should reflect the post-implementation truth only after code behavior is settled.

Verification checkpoint:

1. Linux-first positive and negative smoke paths are both proven,
2. non-Linux posture remains honest and fail-closed,
3. local docs and artifacts no longer imply transport-defined stop success.

## Risks And Mitigations

### Risk 1: Recovery work silently expands into general attach orchestration

Mitigation:

1. consume only sanctioned recovery truth already owned by session/attach surfaces,
2. keep the stop episode limited to one recovery attempt and one re-attempt,
3. reject any implementation that requires a broader attach worker or router redesign in this slice.

### Risk 2: Delivery success still leaks through as semantic stop success

Mitigation:

1. pin caller-visible proof rules in tests first,
2. require authoritative stopped closeout plus terminal proof,
3. treat any proof gap as fail closed even if later read-side state shows `stopped`.

### Risk 3: Owner rebinding and retained-worker targeting get conflated

Mitigation:

1. keep attached-host owner resolution and retained-worker target resolution as separate checks,
2. preserve the exact retained-worker tuple across recovery,
3. fail closed when recovery changes owner truth but cannot preserve target truth.

### Risk 4: Retry behavior broadens into indefinite or fuzzy fallback

Mitigation:

1. cap recovery to one bounded attempt,
2. cap post-recovery stop delivery to one exact-target re-attempt,
3. fail closed after that point with explanation-ready outcomes.

### Risk 5: Caller-visible non-success outcomes stay semantically ambiguous

Mitigation:

1. explicitly choose existing-shape-plus-textual-distinction posture in this plan,
2. pin the required failure-class wording in tests before implementation changes,
3. reject ad hoc message drift that collapses `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed` into indistinguishable generic failures.

### Risk 6: Linux-first rollout language gets mistaken for contract variability

Mitigation:

1. keep Linux-first language only in rollout, smoke, and validation sections,
2. keep normative contract language platform-agnostic in code comments and outcome checks,
3. ensure non-Linux remains explicitly fail-closed rather than semantically divergent.

## Sequencing And Parallelism

### Must stay sequential

1. Phase 1 before Phase 2 because authority/proof regressions must be frozen before behavior changes.
2. Phase 2 before Phase 3 because required recovery depends on trusted authoritative owner and target rebinding.
3. Phase 3 before Phase 4 because caller-visible proof rules depend on final stop-episode recovery behavior.
4. Phase 4 before Phase 5 because smoke validation must exercise the fully corrected contract.

### Can be parallelized later

1. targeted shell and world-service test authoring inside Phase 1 can be split once the exact failure matrix is agreed,
2. narrow doc/artifact wording updates in Phase 5 can proceed in parallel with final smoke execution once implementation behavior is stable,
3. any later typed-taxonomy proposal is explicitly out of scope for this slice and must not be mixed into `TASKS-64`.

## Verification Wall

Minimum validation before calling this slice complete:

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

## Review Gate Before TASKS

This plan is ready to decompose into `TASKS-64` only if reviewers agree that:

1. the scope is limited to the stop contract correction and does not smuggle in attach or router redesign,
2. the implementation order is correct: tests, authority rebinding, recovery, caller-visible proof, smoke validation,
3. the recovery rule is settled and not reopened,
4. Linux-first is understood as rollout posture only,
5. caller-visible non-success outcomes will keep the existing shape with stable textual distinctions rather than introducing a new typed taxonomy in this slice,
6. fail-closed semantics remain the default whenever durable closeout cannot be proven.
