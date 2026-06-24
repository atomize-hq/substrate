# Plan: Retained World Worker Parked-Resume Session-Handle Contract

Source spec: [SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md](./SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md)  
Related architectural inputs:
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Plan type: bounded retained-lifecycle seam repair  
Status: draft for review  
Implementation posture: repair the parked/resumable retained-worker lifecycle seam without widening into bootstrap compatibility, projection, routing-family redesign, or unrelated worker behavior

## Objective

Repair the active retained-world-worker bug where shell/control-plane truth continues to route later `continue_world_worker` turns to an exact retained participant with surfaced resumable identity, but `world-service` currently models continuity as bootstrap-process liveness and unregisters that retained participant when bootstrap exits cleanly.

This plan must land:

1. world-service lifecycle ownership that separates retained continuity from bootstrap-process continuity once resumable identity has been surfaced,
2. a submitted-turn resume path that still works against a parked retained worker using the surfaced session handle,
3. closeout rules that separate non-terminal turn exit from retained-worker death,
4. joined regression coverage that proves `registered -> bootstrap exits -> later continue succeeds`,
5. one shell/public regression that preserves routing truth on the control-plane side.

## Plan Summary

The spec and lifecycle design now freeze the core truth:

1. retained continuity is owned by authoritative retained identity plus resumable session identity,
2. clean bootstrap exit after those identities are surfaced means `running -> parked`,
3. only explicit terminal lifecycle transitions close retained continuity.

The current implementation drift is narrower than a general orchestration redesign:

1. `world-service` registers the retained worker correctly,
2. it remembers surfaced `uaa_session_id`,
3. but it then unconditionally unregisters the retained worker when bootstrap completes,
4. so later `continue_world_worker` routing reaches a control-plane target that runtime no longer recognizes.

So the right plan is:

1. freeze the failing lifecycle seam first in world-service tests,
2. separate retained registry ownership from active bootstrap/turn process ownership at the smallest honest seam,
3. keep submitted-turn resume keyed to exact retained identity plus surfaced session handle,
4. treat non-zero submitted-turn exit as turn failure, not automatic retained-worker deletion, once resumable identity exists,
5. keep one shell/public regression proving exact-target routing still works across the parked-resume seam.

Do **not** widen this slice into:

1. `SPEC-62` bootstrap compatibility/auth/config work,
2. workspace sync or reconciliation,
3. router/inbox/auto-attach redesign,
4. fuzzy worker selection,
5. or broader retained-worker lifecycle changes beyond the active bug seam.

## Planning Defaults Locked For This Slice

Unless live source constraints force a different choice during implementation review, this plan assumes:

1. retained continuity and process continuity are separate once resumable identity is surfaced,
2. the smallest acceptable runtime change is preserving the existing retained registry entry in place,
3. a minimal internal split between retained membership and active bootstrap/turn bookkeeping is acceptable if preserving the entry in place becomes too tangled,
4. non-zero submitted-turn exit after resumable identity exists must not automatically destroy retained continuity,
5. v1 should prefer inferred parked truth over adding a new persisted parked field,
6. lifecycle proof should land first in `world-service`, then one shell/public regression should prove routing truth remains aligned.

If implementation evidence forces a different choice, that should update the plan/spec before code lands.

## Locked Decisions

### What changes

1. `world-service` stops equating clean bootstrap process exit with retained-worker deletion once authoritative retained identity plus resumable session identity have been surfaced.
2. The retained registry remains the runtime authority for parked/resumable workers even when no bootstrap process or submitted turn is active.
3. Submitted-turn resume continues to require exact retained identity and exact binding, but no longer requires bootstrap-process liveness as a hidden prerequisite.
4. Non-terminal submitted-turn exits are handled separately from explicit retained closeout.
5. Regression coverage becomes joined across the actual lifecycle seam instead of proving only held-open test stubs.

### What does not change

1. `SPEC-62` remains the bootstrap-compatibility slice.
2. Exact `participant_id`, `orchestration_session_id`, `backend_id`, `world_id`, and `world_generation` validation remain fail-closed.
3. Public tool vocabulary and request shapes remain unchanged.
4. No new fuzzy selector or alternate resume handle replaces exact retained targeting.
5. No broad router, inbox, attach, or reconciliation behavior lands in this slice.
6. No new persisted parked field is added unless implementation review proves inference is ambiguous or insufficient for correctness.

## Implementation Order

### Phase 1: Freeze The Negative Proof And Regression Harness In World-Service Tests

Goal:

1. make the actual bug reproducible where bootstrap exits cleanly after surfacing resumable identity,
2. pin the fail-closed inverse where no resumable handle was surfaced,
3. establish the harness that will later prove the positive parked-resume success condition once runtime behavior changes land,
4. avoid hiding the bug behind shell-only routing tests.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. `crates/world-service/tests/` only if a black-box harness is needed

Required changes:

1. add focused coverage for the lifecycle seam: retained registration, surfaced session handle, clean bootstrap exit, and an honest post-bootstrap follow-up seam that demonstrates where resumability currently breaks,
2. preserve existing exact-identity mismatch coverage,
3. add the fail-closed inverse: if resumable session identity was never surfaced, bootstrap exit must not promise parked-resumable continuity,
4. keep the test honest about bootstrap process exit rather than using only `ReadyAndHoldUntilCancel` style stubs.

Verification checkpoint:

1. the failing seam reproduces before the runtime fix,
2. the negative no-session-handle case is pinned,
3. the harness for the later positive parked-resume proof is in place,
4. the phase does not require the positive parked-resume success case to pass yet.

### Phase 2: Separate Retained Registry Ownership From Active Process Ownership

Goal:

1. make retained continuity independent of bootstrap-process liveness once resumable identity exists,
2. keep the implementation minimal and local to `world-service`,
3. make this the first behavior-changing phase that turns the positive parked-resume case green,
4. avoid introducing broader lifecycle infrastructure than this seam needs.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`

Required changes:

1. stop unconditional retained unregistration on clean bootstrap exit,
2. preserve the retained registry entry in place if possible,
3. if needed, introduce only the smallest internal separation between retained membership and active bootstrap/turn bookkeeping,
4. keep launcher-dir/bootstrap cleanup separate from retained continuity deletion when those meanings diverge,
5. keep exact retained slot ownership semantics intact.

Verification checkpoint:

1. a retained worker with surfaced resumable identity still exists in runtime registry after bootstrap exit,
2. no duplicate retained slot or stale-owner drift is introduced,
3. bootstrap resource cleanup still occurs where appropriate without destroying retained continuity,
4. the positive parked-resume proof goes green for the first time in this phase.

### Phase 3: Make Submitted-Turn Resume Work Against Parked Retained Workers

Goal:

1. allow `submit_turn` / `continue_world_worker` to resume a parked retained worker using the surfaced session handle,
2. preserve exact-target validation and fail-closed binding checks,
3. keep the resume seam truthful without requiring a new public contract.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs` only if a narrow integration adjustment is actually required

Required changes:

1. ensure `submit_turn` can locate a parked retained worker through the preserved retained registry entry,
2. keep surfaced `uaa_session_id` as the runtime-owned resume handle required for submitted-turn execution,
3. preserve exact validation of orchestration session, orchestrator participant, backend, world id, and world generation,
4. keep failure explicit when the retained worker exists but no surfaced resumable session handle exists.

Verification checkpoint:

1. a later submitted turn succeeds after clean bootstrap exit,
2. exact-binding mismatch still fails closed,
3. missing surfaced resume handle still fails with a direct explanation rather than implicit fallback behavior.

### Phase 4: Separate Turn Failure From Worker Death

Goal:

1. keep retained continuity alive across non-terminal submitted-turn failures once resumable identity exists,
2. reserve retained closeout for explicit terminal semantics,
3. avoid accidentally broadening the lifecycle model beyond the spec.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. targeted runtime tests adjacent to the member-runtime seam

Required changes:

1. review current submitted-turn completion/unregister paths for accidental retained closeout coupling,
2. preserve retained continuity after non-zero submitted-turn exit unless explicit stop, invalidation, or equivalent terminal closeout semantics are reached,
3. keep active-turn bookkeeping cleanup distinct from retained-worker deletion,
4. avoid inventing broader new lifecycle policy unless required to keep this seam coherent.

Verification checkpoint:

1. non-zero submitted-turn exit does not by itself delete a resumable retained worker,
2. explicit terminal closeout still removes resumability,
3. turn-slot cleanup and retained continuity no longer imply each other.

### Phase 5: Keep One Shell/Public Regression For Already-Created Retained World-Member Follow-Up Truth

Goal:

1. prove shell/control-plane routing still aligns with the repaired world-service lifecycle seam,
2. keep Packet 4 scoped to public follow-up over an already-created exact retained world-member participant with surfaced resumable identity,
3. cross-check Packet 4 against [`SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`](./SPEC-30-public-world-scoped-agent-start-and-capability-flags.md), which freezes public `--scope world` root start as host-first and explicitly does **not** eagerly allocate a world-member slot at `start` return,
4. keep one durable public proof wall for the exact-target contract without reinterpreting root-start projection as retained-member allocation truth.

Primary touch surface:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs` only if a narrow routing mismatch is uncovered

Required changes:

1. extend or relabel one Linux shell/public regression so an already-created retained world member can be followed up after bootstrap has exited, not only while a member runtime is artificially held open,
2. treat `public_turn_routes_linux_world_member_follow_up_through_typed_submit_path` as acceptable Packet 4 proof **only** for exact retained-member public follow-up because it first creates the retained world member through the REPL/private path and then exercises public follow-up over that exact participant,
3. state explicitly that the mixed REPL/public regression is **not** proof of pure public root `substrate agent start --backend cli:codex-world --scope world` -> public `substrate agent turn --backend cli:codex-world`,
4. keep the assertion focused on exact-target routing and successful submitted-turn delivery once the retained world-member slot already exists,
5. defer any pure-public startup-stabilization idea in `agents_cmd.rs` / `agent_runtime/control.rs` to a later spec/plan change if a higher authority ever changes the frozen Slice 30 host-first root-start contract,
6. preserve the current shell-side authoritative targeting and durable participant/session-handle truth without reintroducing bootstrap-process or prompt-transport liveness as authoritative continuity truth.

Verification checkpoint:

1. the public routing test proves `continue_world_worker` still targets the exact retained participant after parked handoff once that retained world-member slot already exists,
2. shell-side exact-target validation remains unchanged,
3. the mixed REPL/public regression is documented as acceptable Packet 4 proof for exact retained-member follow-up, not as proof of pure public root start -> turn,
4. the frozen Slice 30 host-first root-start regression (`public_root_start_world_scope_starts_attached_host_session_with_world_binding_truth`) remains authoritative for the rule that public world start does not eagerly allocate a world-member slot,
5. the repaired seam is proven without reopening broader shell lifecycle design.

## Risks And Mitigations

### Risk 1: The fix quietly grows into a broad retained-lifecycle rewrite

Mitigation:

1. keep the first fix local to `world-service` retained registry and submitted-turn seams,
2. prefer preserving the existing registry entry in place over introducing a larger new state model,
3. reject unrelated lifecycle cleanup from this slice unless it is directly required for correctness.

### Risk 2: Resource cleanup remains entangled with retained continuity

Mitigation:

1. explicitly separate launcher/bootstrap cleanup from retained-worker deletion,
2. add tests that assert continuity survives bootstrap exit while cleanup still happens where valid,
3. review unregister paths for shared helpers that currently conflate process teardown with lifecycle closeout.

### Risk 3: Parked truth becomes ambiguous without a new persisted field

Mitigation:

1. use the v1 inference default first: retained registry entry exists + no active bootstrap/turn slot + resumable handle exists + not terminal,
2. add a new explicit parked field only if implementation review proves inference is ambiguous or insufficient,
3. call out any such escalation before implementation widens the contract.

### Risk 4: Turn-failure semantics accidentally preserve truly dead workers forever

Mitigation:

1. keep explicit terminal closeout, stop, and invalidation as the only retained-destruction paths,
2. require tests for both non-terminal failure preservation and explicit terminal removal,
3. avoid broad “always preserve forever” behavior that ignores terminal semantics.

### Risk 5: The public regression still relies on hold-open test behavior

Mitigation:

1. make world-service lifecycle proof the first gate,
2. use only one shell/public regression afterward,
3. ensure the shell/public proof explicitly crosses the bootstrap-exits-then-resume seam.

## Sequencing And Parallelism

1. Phase 1 must land before behavior changes so the lifecycle seam stays pinned.
2. Phase 2 must land before Phase 3 because submitted-turn resume depends on preserved retained registry ownership.
3. Phase 3 must land before Phase 4 because turn-failure semantics matter only once parked resume works.
4. Phase 5 comes after the world-service seam is repaired so shell/public proof reflects real runtime truth instead of masking it.
5. This slice is mostly sequential; the only honest parallel work is drafting the shell/public regression while Phase 2-4 runtime changes are under review.

## Review Checkpoints

After each phase:

1. confirm the scoped verification checkpoint is green,
2. confirm the slice has not widened into `SPEC-62` bootstrap compatibility or config/auth projection,
3. confirm exact-target retained routing and exact-binding validation remain fail-closed,
4. confirm retained continuity is no longer coupled to bootstrap-process liveness,
5. confirm no new persisted parked field was added unless explicitly justified and reviewed.

## Final Verification Wall

Before declaring the slice ready for TASKS/implementation closeout, expect a proof wall equivalent to:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. targeted `world-service` member-runtime tests for parked-resume lifecycle and fail-closed inverse cases
4. targeted shell regression for exact-target parked follow-up routing
5. repo-truth confirmation that the implementation did not modify `SPEC-62` scope or adjacent broad design/docs beyond any narrowly necessary explanatory comment updates

## TASKS Dependency Note

No TASKS document is written in this phase. The next phase should decompose this plan into ordered tasks after plan review, with Phase 1 world-service lifecycle proof as the first mandatory task group.
