# 2026-06-25 — subagent4 host/orchestrator parked-resume routing

## Short objective
Trace whether current shell-side parked/resume, participant, public-routing, and auto-attach truth can look healthy for a shell-owned host/orchestrator session while world-member / world-task dispatch is already broken, stale, or requires a stronger runtime condition than the persisted session state implies.

## Docs / code areas inspected
- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md`
- `llm-last-mile/DESIGN-retained-world-worker-messaging-and-steering-contract.md`
- `llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md`
- `llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md`
- `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md`
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`
- `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`
- `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/agent_runtime/control.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/world-service/src/member_runtime.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`

GitNexus notes:
- refreshed local index to current commit with `npx gitnexus analyze --name atomize-substrate .`
- `gitnexus query/context` CLI remained unusably slow here even after refresh, so final localization came from live source + tests

## Shell-side contract assumptions that still look sound
- `SPEC-30` / `public_root_start_world_scope_starts_attached_host_session_with_world_binding_truth` are still reflected in code: public `--scope world` start is host-first and does **not** allocate an eager world-member slot. `build_world_start_session_birth_plan()` projects public world identity while launching the hidden host helper.
- Host parked truth itself is internally consistent for host continuity only: `build_parked_host_runtime_snapshots()` + `valid_detached_host_continuity_posture()` require a detached-but-resumable host participant plus persisted attach continuity before posture normalizes to `parked_resumable` / `awaiting_attention`.
- `world-service` now preserves retained member registry state across clean bootstrap exit when the session handle surfaced (`finish_bootstrap(..., preserve_retained_member=true)`), so the old pure world-service deletion bug from the pre-`SPEC-63` seam is no longer the main shell-side suspicion.

## Shell-side contract assumptions that look suspicious
- Parked host/session truth is computed from **host continuity only**. It does not prove that any world member exists, is routable, or that new world-member launch remains realizable.
- Public world follow-up and internal exact-target continue do **not** use the same notion of a valid retained world worker.
- Member bootstrap for spawn/fork still requires a **live** orchestrator parent, which conflicts with the intended parked host / independent world-worker architecture if parked sessions are meant to keep delegating without a manual reattach.

## Exact places healthy parked session state can mask broken world-task/member dispatch
1. **Projected world start can look healthy without any world-member slot at all**
   - `crates/shell/src/execution/agents_cmd.rs:1349+` `build_world_start_session_birth_plan()` launches host helper but publishes public identity as world-scoped.
   - `crates/shell/tests/agent_public_control_surface_v1.rs:6047+` proves public world-root start persists exactly one host participant and zero member dispatch requests.
   - So “session has world binding/policy/backend” is not proof that `run_world_task`, `spawn_world_worker`, or retained follow-up is currently realizable.

2. **Toolbox/status world binding is parent-session truth only**
   - `crates/shell/src/execution/agents_cmd.rs:2980+` `toolbox_active_world_binding()` only checks parent `world_id/world_generation`.
   - A session can therefore report healthy active world binding even if there is no retained member slot, no active host owner, or a downstream world dispatch path is already stale.

3. **Parked host normalization ignores world dispatch viability**
   - `crates/shell/src/repl/async_repl.rs:7914+` `build_parked_host_runtime_snapshots()` clears host ownership, marks detached/parked, syncs host attach continuity, and never inspects retained worker routability or future member-launch realizability.
   - This makes the parked/awaiting-attention state honest about host detach continuity but silent about world dispatch health.

4. **Public world follow-up can select a parked member that internal exact-target routing would reject**
   - `crates/shell/src/execution/agent_runtime/state_store.rs:4732+` `public_turn_authoritative_candidates()` selects world candidates by backend + linkage + parent world binding, but does **not** require `is_authoritative_live()`.
   - `crates/shell/src/execution/agent_runtime/state_store.rs:1431+` `resolve_internal_continue_world_dispatch_target()` **does** require the target retained worker to be authoritative-live and owner-alive; otherwise it returns `stale_linkage: ... is no longer authoritative-live`.
   - That means a parked retained worker can still be publicly follow-up-routable through the typed submit path while the internal `continue_world_worker` exact-target path classifies the same worker as stale/invalidated.

5. **Spawn/fork-style member launch still hard-requires a live host owner**
   - `crates/shell/src/repl/async_repl.rs:6110+` `prepare_member_runtime_startup_for_descriptor()` calls `resolve_live_member_parent()`.
   - `crates/shell/src/repl/async_repl.rs:5950+` `resolve_live_member_parent()` requires exactly one live orchestrator parent and fails when none is active.
   - So a session can be perfectly parked/resume-eligible on the shell side but still be unable to launch a new world member until host ownership is actively restored.

6. **Internal world dispatch caller validation is weaker than later member-launch requirements**
   - `crates/shell/src/execution/agent_runtime/state_store.rs:1402+` `resolve_internal_world_dispatch_caller()` only checks that the caller is the authoritative orchestrator participant for an active session.
   - It does not require active-attached/living host ownership.
   - That lets a parked authoritative caller pass the front door, while a later launch/bootstrap path can still fail because deeper code requires a live owner/process.

7. **Auto-attach readiness is about detached host continuity, not dispatch realizability**
   - `crates/shell/src/execution/agent_runtime/state_store.rs:4892+` `classify_router_auto_attach_session_readiness()` promotes eligibility from detached host continuity.
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs:1630+` router discovery/attach triggering works from obligations + detached-session eligibility; it does not verify that the eventual world-worker route or spawn path is still valid before the session is described as a viable auto-attach candidate.

## What this most likely is
**Primary diagnosis: cross-layer contract mismatch.**

Why:
- Shell parked/session status is mostly telling the truth about **host continuity**, not world dispatch health.
- Some downstream dispatch paths still require stronger runtime conditions (live orchestrator owner, authoritative-live worker) than the parked host/session surfaces imply.
- There is also one likely **shell-side truth bug** inside that mismatch: `resolve_internal_continue_world_dispatch_target()` still treats a cleanly parked retained worker as unroutable stale linkage, which is in tension with `SPEC-63`’s parked-resume target contract and with the public typed-submit follow-up path.

So this does **not** read like “the parked session state is simply lying.” It reads like: parked host state is too narrow, and neighboring shell/runtime layers are not using one shared definition of “still routable after clean detach/bootstrap exit.”

## Next 3 highest-value discriminators
1. **Reproduce the same retained worker through both shells of routing**
   - same parked retained participant
   - first: public `agent turn --backend <world-backend>`
   - second: internal `continue_world_worker` exact-target
   - if public succeeds but internal exact-target returns `invalidated_worker_not_routable` / `stale_linkage`, that cleanly proves the shell-side routing-definition split.

2. **Probe parked host -> spawn/fork launch without manual reattach**
   - after host owner parks cleanly, issue the world-worker allocation path that hits `prepare_member_runtime_startup_for_descriptor()`
   - if it fails at `resolve_live_member_parent()` while session posture remains `parked_resumable` / `awaiting_attention`, that proves the parked-host vs live-parent mismatch directly.

3. **Capture router auto-attach outcomes for a detached session with eligible obligations**
   - verify whether the router actually restores attached host ownership before the next host-handling need, or only marks the session/obligation as attach-eligible while deeper world dispatch still fails later.
   - if attach eligibility/outcome logs look successful but the next launch still fails on live-parent or stale-target checks, the mismatch is definitely cross-layer rather than a single routing typo.

## Final take
Current shell code can absolutely report a host/orchestrator session as healthy parked/resumable while the world-task / world-member path is already non-realizable or split across inconsistent routing rules.

The strongest shell-side mismatch is:
- host parked posture is derived from detached host continuity only,
- public world follow-up tolerates a parked retained member by linkage,
- internal exact-target continue still wants that same worker to be authoritative-live,
- and member bootstrap for new world work still wants a live orchestrator parent.

Net: the user’s intended architecture (“host parked but resumable; world workers independent; auto-reattach only when host handling is needed”) is **not** represented by one coherent shell contract yet. The most honest label right now is **cross-layer contract mismatch with one likely shell-side exact-target bug still present**.
