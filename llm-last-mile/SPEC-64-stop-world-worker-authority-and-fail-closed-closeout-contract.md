# Spec: Stop World Worker Authority And Fail-Closed Closeout Contract

Source authorities:
- [../DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md](../DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](./SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)
- [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
- [PLAN-36.md](./PLAN-36.md)
- [SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md](./SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md)
- [PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md](./PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md)
- [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](./29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)
- [../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)

Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. `SPEC-36` remains the frozen baseline for the existence of `stop_world_worker` as an internal retained-worker control-plane verb, but its original landed implementation posture is not sufficient to answer the current authority-versus-transport contract question.
2. `DESIGN-world-worker-lifecycle-model.md` and `SPEC-63` are the current lifecycle authorities for retained-worker continuity, including the rule that surfaced retained identity plus resumable identity outlive bootstrap/transport liveness.
3. `stop_world_worker` remains internal-only and retained-worker-only in this correction slice. This spec does not widen the public `substrate agent ...` surface and does not redefine `cancel_world_work`.
4. The durable authority root remains the orchestration session plus persisted attach/session truth; the currently attached host participant is a replaceable sanctioned execution client, not the durable owner.
5. A stop implementation may reuse the existing private owner stop transport posture where available, but this spec must not let that transport posture redefine stop success semantics.

If any of these are wrong, correct them before implementation.

## Objective

Correct the `stop_world_worker` contract so implementation can proceed from one explicit rule set for authority, worker identity, transport failure, and closeout truth.

This spec freezes the narrow stop-specific correction:

1. who is authoritative when `stop_world_worker` is issued,
2. what retained worker identity and lineage must remain stable,
3. what `stop_world_worker` means as a durable control-plane closeout,
4. what the private stop transport may and may not imply,
5. how stop behaves when authoritative host attachment changes, disappears, or must be restored,
6. and what fail-closed outcomes must occur when delivery and durable lifecycle truth diverge.

This spec is intentionally narrow. It does not reopen general attach architecture, router product behavior, world-worker lifecycle taxonomy beyond the stop seam, or unrelated control verbs.

## Precedence

`SPEC-36` remains the baseline authority for these still-frozen points:

1. `stop_world_worker` exists as an internal retained-worker control-plane verb,
2. it remains distinct from `cancel_world_work`,
3. it must not widen into a new public CLI contract,
4. it must not introduce a second stop transport or a second lifecycle model.

`SPEC-64` supersedes and constrains `SPEC-36` for all stop-behavior questions where authority truth, attached-host ownership changes, transport liveness, recovery, or caller-visible proof semantics are involved.

In any conflict between:

1. `SPEC-36` implementation-era assumptions,
2. transport-centric interpretations of `SPEC-36`,
3. or repo code that treats private stop delivery posture as sufficient proof of stop success,

`SPEC-64` is the governing authority for:

1. who owns stop,
2. what recovery is required,
3. what counts as success,
4. and when the result must fail closed.

## Observed Repo Floor

The current design stack already freezes the following truths:

1. Durable authority belongs to the orchestration session, not to any one attached host client.
2. Attached host ownership may change across turns only through sanctioned restoration paths such as manual `reattach` or router-owned attach restoration.
3. Retained worker continuity is owned by authoritative identity and binding truth, not by bootstrap-process or transport liveness alone.
4. `stop_world_worker` is supposed to be durable retained-worker closeout, not generic cancel and not a transport-side heuristic.
5. The internal transport family is event-capable and terminal-result-based; owner disappearance before terminal result is a fail-closed transport outcome, not proof of durable stop success.

The contract gap this spec closes is therefore not the existence of stop. It is the missing rule for how stop must behave when authority truth says the worker is still live but the private stop delivery path is stale, missing, refused, or only later restored.

## Contract Slice

### In scope

1. The authority model for `stop_world_worker`.
2. Retained worker identity and lineage expectations required for exact-target stop.
3. Durable stop semantics versus private transport semantics.
4. Fail-closed behavior for missing, refused, stale, or late-published private stop delivery.
5. Recovery behavior when authoritative host attachment changes across turns.
6. Acceptance criteria for implementation and Linux-first smoke validation of the corrected stop contract.

### Explicitly out of scope

1. Public CLI redesign or a new public stop surface.
2. General `cancel_world_work` redesign.
3. Fork, continue, approval, inbox, or router feature expansion beyond what is required to define sanctioned authority recovery.
4. New worker-selection semantics beyond exact retained-worker identity and existing authoritative binding checks.
5. New lifecycle families, new durable attach baselines, or a second stop transport.

## Frozen Direction

### 1. Durable authority root

`stop_world_worker` is issued under orchestration-session authority.

That means:

1. the durable owner is the orchestration session plus persisted attach/session truth,
2. the currently attached host participant is the active sanctioned execution client for that session when one exists,
3. a host participant instance may change across turns without changing the durable authority root,
4. stale attached-host snapshots must not be treated as durable ownership truth.

### 2. Retained worker identity and lineage

The stop target remains one exact retained worker, not a fuzzy runtime slot.

Required exact target truth:

1. `orchestration_session_id`
2. exact retained `target_participant_id`
3. exact `backend_id`
4. exact authoritative `world_id`
5. exact authoritative `world_generation`
6. retained-worker lineage that still matches authoritative session/world truth

If that exact tuple is not provable, stop must fail closed.

### 3. Stop success semantics

`stop_world_worker` means explicit durable control-plane closeout of the retained worker.

A stop attempt is successful only when:

1. the request is accepted under authoritative session and worker truth,
2. the exact retained worker transitions to authoritative stopped closeout state,
3. later `continue_world_worker` against that worker is no longer permitted,
4. the closeout result is surfaced through a terminal stop outcome rather than inferred from transport disappearance.

### 4. Private stop transport semantics

The private stop transport is an implementation delivery path, not the semantic owner of stop.

It may imply only this:

1. a request was or was not delivered to the current sanctioned owner path,
2. a terminal success or failure result was or was not returned,
3. the owner path was or was not still live long enough to finish the exchange.

It must not imply any of the following by itself:

1. durable stop success because a socket or helper existed,
2. durable stop success because a delivery attempt was initiated,
3. durable stop success because the previous owner path disappeared,
4. durable stop success because a new owner path later appeared.

### 5. Exact fail-closed rule

If the stop implementation cannot prove durable closeout against the authoritative target, it must fail closed.

Fail closed means:

1. do not mark the worker `stopped`,
2. do not silently translate delivery failure into stop success,
3. do not silently widen into `cancel_world_work`,
4. do not silently retarget to a guessed host participant or guessed worker,
5. return an explanation-ready outcome that preserves whether the failure was missing authority, stale authority, refused transport, missing transport, or unsuccessful recovery.

### 6. Caller-visible proof rule

Caller-visible stop success requires terminal proof returned to the caller for that stop episode.

Therefore:

1. if authoritative stopped closeout is not proven, the attempt fails closed,
2. if authoritative stopped closeout may have happened but the caller did not receive terminal proof for that same stop episode, the attempt still fails closed from the caller's perspective,
3. later read-side observation may reveal that the worker is already `stopped`, but that later observation does not retroactively convert the earlier caller-visible attempt into a success.

## Required Runtime Semantics

### Authority model

1. The orchestration session remains the durable authority root for all stop decisions.
2. The attached host participant is authoritative only as the current sanctioned execution client for that session, not as an independent durable owner.
3. If sanctioned restoration changes the attached host participant between turns, the new attached participant becomes the authoritative live owner path for future delivery attempts.
4. A stop request must bind to current authoritative session truth at dispatch time; it must not trust a previously cached attached-host participant if persisted/session truth has changed.

### Retained worker identity and lineage expectations

1. The retained worker being stopped must remain the same exact retained participant across authority recovery.
2. Authority recovery may change the attached host participant, but it must not mutate retained worker identity, world binding, or lineage.
3. If authoritative recovery disproves the retained worker binding or lineage, the stop attempt fails closed or invalidates under existing lifecycle rules; it does not convert to successful stop.

### Stop as durable control-plane closeout

1. Stop is complete only on authoritative lifecycle closeout.
2. Transport completion without authoritative stopped state is not enough.
3. Authoritative stopped state without a terminal caller-visible result must still be treated as caller-visible fail-closed for that stop episode.
4. A successful stop consumes the worker's continuation path. A failed stop leaves the worker under its truthful preexisting lifecycle state unless an explicit independent invalidation or terminal transition occurred.

## Scenario Rules

### 1. Authoritative host participant changes across turns

When the authoritative attached host participant changes through sanctioned restoration:

1. the orchestration session remains the durable owner,
2. the new attached participant becomes the only valid live delivery target for the stop transport,
3. any stale private stop path for the old attached participant must be ignored as stale delivery state,
4. the retained worker target and lineage remain unchanged,
5. a stop attempt may proceed only after rebinding to current authoritative attached-host truth or explicitly failing closed if that truth is unavailable.

### 2. Worker ownership truth says live but private stop transport is dead

When authoritative worker/session truth says the retained worker is still live, parked, paused, running, or attention-pending, but the private stop transport is dead:

1. the worker must not be marked `stopped`,
2. the transport episode must be treated as failed closed,
3. implementation must attempt one bounded sanctioned authority recovery when persisted attach/session truth says recovery is still valid and currently available,
4. if recovery is not valid or not available, return fail-closed delivery failure while preserving live worker truth,
5. no silent downgrade to `cancelled`, `completed`, or `stopped` is allowed.

### 3. Missing vs refused vs late-published transport

These cases must be distinct:

1. Missing transport
   - no valid current private stop delivery path is published for the authoritative owner.
   - required behavior: if sanctioned recovery is currently available under authoritative attach/session truth, one bounded recovery attempt is required during the same stop episode; otherwise fail closed.

2. Refused transport
   - a delivery path exists but rejects the request because authority, binding, target, or owner truth is wrong or no longer current.
   - required behavior: if the refusal indicates stale owner truth and sanctioned recovery is currently available, one bounded recovery attempt is required; otherwise fail closed with a refused or stale-authority outcome and do not treat refusal as stop success.

3. Late-published transport
   - no valid delivery path existed at the start of the stop attempt, but a sanctioned new owner path appears later.
   - required behavior: the original stop attempt does not retroactively succeed merely because a path appeared later.
   - the only allowed success path is required bounded authority recovery followed by one exact-target stop re-attempt within the same stop episode when that recovery is currently available under authoritative truth.
   - otherwise the attempt fails closed and a later caller must retry explicitly.

### 4. Authority recovery succeeds vs fails

Authority recovery here means sanctioned restoration of authoritative attached-host ownership for the same orchestration session under existing attach truth.

Bounded authority recovery is required, not optional, when all of the following are true:

1. the stop attempt has not yet produced terminal proof,
2. authoritative session truth still permits sanctioned recovery,
3. the failure mode is stale, missing, or refused current owner delivery rather than disproven worker identity,
4. the recovery can target the same orchestration session and exact retained worker without broadening scope.

This requirement follows from the cited source stack:

1. durable authority belongs to the orchestration session rather than any one attached host client,
2. the authoritative attached host participant may legitimately change through sanctioned restoration,
3. transport failure is not lifecycle truth,
4. and fail-closed behavior should occur only after the stop episode has exhausted the sanctioned current-owner path that authoritative truth still says is available.

If authority recovery succeeds:

1. the recovered attached host participant becomes the live delivery target,
2. the retained worker identity tuple must still match exact authoritative truth,
3. the stop implementation must issue exactly one bounded exact-target re-attempt through the recovered owner path,
4. the stop attempt succeeds only if that re-attempt reaches authoritative stopped closeout and returns terminal success.

If authority recovery fails:

1. the stop attempt fails closed,
2. the worker remains in truthful non-stopped lifecycle state unless independently invalidated or closed for another explicit reason,
3. the outcome must explain whether recovery failed because attach truth was missing, attach truth was disproven, policy denied recovery, or recovery could not produce a sanctioned owner path.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` dispatch, control, state-store, and orchestration-session authority surfaces
- `crates/world-service` retained-worker lifecycle/runtime truth where stop closeout is ultimately realized
- existing private owner stop transport posture already referenced by `SPEC-36`
- `llm-last-mile/` spec/plan stack as design authority

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Current targeted contract suites to keep green while implementing this correction:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p world-service member_runtime -- --nocapture
```

Expected smoke-validation floor after implementation:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This correction is expected to stay narrow to these authority and closeout seams:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - binds stop requests to current authoritative session/worker truth
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained-worker target resolution, attached-host truth, and fail-closed rejection paths
- `crates/shell/src/execution/agent_runtime/control.rs`
  - stop closeout orchestration and bounded recovery posture
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
  - persisted attach/session authority inputs consumed by stop
- `crates/world-service/src/member_runtime.rs`
  - runtime lifecycle truth for final retained-worker stopped closeout
- `crates/shell/tests/` and `crates/world-service/tests/`
  - exact-target, authority-rebinding, fail-closed, and smoke regression coverage
- `llm-last-mile/`
  - this spec and the narrow follow-on plan/task artifacts

## Code Style

Preserve the repo's existing style for exact identity, explanation-ready fail-closed errors, and separation of lifecycle truth from delivery mechanics.

Preferred shape:

```rust
if !authoritative_stop_closeout_proven {
    anyhow::bail!("stop_world_worker failed_closed: durable closeout not proven");
}
```

Conventions:

1. validate current authoritative owner truth before delivery,
2. keep retained worker identity and host-owner identity distinct,
3. model transport failure separately from lifecycle success,
4. prefer explicit stale-authority or missing-authority branches over generic I/O failure wording,
5. do not let recovery helpers silently broaden authority or retarget workers.

## Testing Strategy

### Test levels

1. Dispatch and state-store tests
   - exact retained target required,
   - current authoritative owner required,
   - stale attached-host snapshots rejected,
   - stop does not silently widen into cancel.

2. Runtime lifecycle tests
   - stop success requires authoritative retained-worker stopped closeout,
   - a dead or disappearing transport path does not by itself produce stopped truth,
   - explicit terminal closeout still removes continuation eligibility.

3. Recovery-path tests
   - authoritative host change across turns rebinding works only through sanctioned restoration,
   - recovery success allows one bounded exact-target re-attempt,
   - recovery failure returns explanation-ready fail-closed outcome and preserves live worker truth.

4. Smoke validation
   - Linux-first retained-worker smoke proves stop success after authoritative host recovery for the same orchestration session,
   - Linux-first retained-worker smoke proves that dead private stop transport yields explicit fail-closed outcome and leaves the worker non-stopped until explicit closeout succeeds,
   - non-Linux posture remains fail-closed rather than approximating success through alternate semantics.

### Required acceptance boundary

Implementation is not complete until all of the following are true:

1. `stop_world_worker` success is gated by authoritative stopped state, not by transport attempt alone.
2. The implementation distinguishes orchestration-session authority from attached-host execution-client identity.
3. A changed authoritative attached host participant across turns does not break stop, but stale owner paths are never trusted.
4. When worker truth says live but transport is dead, the result is fail closed unless bounded sanctioned recovery reaches authoritative stop success.
5. Missing, refused, and late-published transport cases have stable, distinct, explanation-ready outcomes.
6. Stop never silently reclassifies failure as `stopped`, `cancelled`, or generic success.
7. A successful stop leaves no continuation path for the exact retained worker.

## Boundaries

- Always:
  - keep this slice narrow to the `stop_world_worker` contract correction
  - keep orchestration-session authority distinct from attached-host execution-client identity
  - hold success on authoritative stopped closeout, not on delivery posture alone
  - fail closed when exact authority, binding, or closeout truth cannot be proven
  - preserve exact retained-worker identity and lineage across authority recovery

- Ask first:
  - introducing new public CLI vocabulary or public stop semantics
  - broadening router or auto-attach product behavior beyond what is needed to define sanctioned recovery inputs
  - changing `cancel_world_work`, `fork_world_worker`, or general lifecycle taxonomy in the same slice
  - adding a second stop transport or a second durable lifecycle model

- Never:
  - treat a dead or disappearing private stop transport as proof of durable stop success
  - reconstruct durable authority from stale attached-host snapshots or permissive defaults
  - silently retarget stop to a guessed worker or guessed host owner
  - silently convert stop delivery failure into `cancel_world_work`
  - let late transport publication retroactively mark an earlier stop attempt successful

## Success Criteria

This spec is satisfied only when implementation and smoke validation prove all of the following:

1. `stop_world_worker` is explicitly enforced as durable retained-worker closeout under orchestration-session authority.
2. The authoritative attached host participant may change across turns, but only sanctioned current authority may receive a stop delivery attempt.
3. Retained worker identity and lineage remain exact and stable across authority recovery.
4. Durable stop success is reported only after authoritative stopped closeout is proven.
5. Private stop transport is treated only as a delivery mechanism and never as the semantic owner of stop.
6. Missing transport, refused transport, late-published transport, and failed authority recovery all fail closed without falsely stopping the worker.
7. When sanctioned recovery is available and exact-target authority can still be proven, stop requires one bounded authority recovery attempt plus exactly one bounded exact-target re-attempt within the same stop episode.
8. Linux-first smoke validation proves both the positive recovered-stop path and the negative dead-transport fail-closed path.

## Resolved Planning Decision

1. This slice keeps the existing caller-visible error/result shape and preserves stable textual distinctions for `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed`; it does not introduce a new typed stop-outcome taxonomy.

## Expected Follow-On Plan

At minimum, a narrow `PLAN-64` should follow this spec and sequence:

1. pin the current stop-authority failure modes in tests,
2. bind stop dispatch to current authoritative owner truth,
3. implement the required bounded recovery and exact-target re-attempt semantics without widening scope, with any Linux-first rollout nuance treated only as sequencing rather than as contract variance,
4. verify authoritative stopped closeout plus Linux-first smoke coverage for positive and negative paths.
