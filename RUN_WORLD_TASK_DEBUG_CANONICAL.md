# `run_world_task` / retained-world-member debug synthesis

Last updated: 2026-06-26  
Status: working canonical memo for this issue family; live code/tests/docs remain final authority.

## Purpose

This file merges the six 2026-06-25 subagent debug logs into one bounded source-of-truth memo we can reuse during troubleshooting without rereading every handoff.

It is intentionally:
- detailed enough to preserve the current diagnosis,
- narrow enough to avoid context bloat,
- explicit about what is **confirmed**, **likely**, and **not yet proven**.

## Source inputs

Merged from:
- `handoffs/2026-06-25-subagent1-run-world-task-transport-and-dispatch.md`
- `handoffs/2026-06-25-subagent2-runtime-lifecycle-vs-spec63.md`
- `handoffs/2026-06-25-subagent3-world-side-effect-sync-and-host-visibility.md`
- `handoffs/2026-06-25-subagent4-host-orchestrator-parked-resume-routing.md`
- `handoffs/2026-06-25-subagent5-design-spec-contract-crosswalk.md`
- `handoffs/2026-06-25-subagent6-gitnexus-execution-flow-trace.md`

Primary repo-truth surfaces repeatedly cited by those logs:
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/member_runtime.rs`
- `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`
- `crates/world-service/tests/member_runtime_world_placement_v1.rs`
- `docs/USAGE.md`
- `docs/WORLD.md`
- `docs/internals/world/workspace_sync_filesystem_model.md`
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`
- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md`
- `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`

## Active symptom family

The live issue being debugged is the combination of:

1. a `run_world_task` invocation that reports terminal success and yields a real `task_run_id`,
2. the expected file not appearing at the host repo root afterward, and
3. later fresh attempts sometimes failing early with:
   - `failed to launch run_world_task over world member dispatch`

The current repo shows **two independent mechanisms** that can explain those two symptoms:

- overlay-first world writes before workspace sync, and
- retained-slot preservation keyed by `orchestration_session_id`, `world_generation`, and `backend_id`.

Live code alone does **not** prove the current repro hit both.

## Latest observed repros (2026-06-26)

These are the newest concrete reproductions this memo should track.

### Repro A: host-only toolbox session fails with `missing_world_binding`

In a `~/.substrate/bin/substrate --no-world` session, a direct toolbox UDS `spawn_world_worker` request targeting `cli:codex-world` failed with:

> `missing_world_binding: orchestration session ... has no authoritative world binding`

What this means:

- this session posture was effectively **host-only**,
- the toolbox request path did **not** bootstrap the first authoritative world binding,
- the request failed before any world worker could perform the file write.

Why this matters:

- this is a valid contract check and confirms that a host-only toolbox socket is **not** enough to start fresh world dispatch,
- but it is **not** the most interesting blocker by itself, because the session had no authoritative world binding to begin with.

### Repro B: world-bound-looking session still fails on exact binding, then follow-up is stale

In a later normal `~/.substrate/bin/substrate` session, the host runtime/toolbox surface appeared to have a live world binding, but a direct toolbox UDS `spawn_world_worker` request still failed with:

> `HTTP 400 Bad Request error: {"error":"member_dispatch.world_id mismatch (expected wld_..., got wld_...)"}`  

and a direct `continue_world_worker` follow-up on the returned retained participant then failed with:

> `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

What this means:

- the newer blocker is **not just** “host-only session has no binding,”
- the more current blocker is that a session can look world-bound from one surface while `world-service` rejects the dispatch on **exact authoritative world binding**,
- and the retained participant created by that failed attempt is not continue-routable afterward.

Why this matters:

- this is the strongest current repro for the active mismatch,
- it fits the broader memo theme of shell/runtime state skew and stricter exact binding/routability checks deeper in the stack,
- and it is the clearest currently tracked blocker for toolbox-driven world-worker launch through the direct UDS path.

## Synthesized working model

### 1) Missing host-visible file is probably **not** the primary runtime failure

Current repo truth strongly supports this interpretation:

- `run_world_task` is translated into typed world-member dispatch, not a host-root direct write path.
- Under full isolation, relative writes land in the **world overlay view first**.
- Host visibility for those writes is expected only after workspace reconciliation / sync from the world view back to the host workspace.
- The one-shot `run_world_task` path currently terminates on streamed start/event/exit frames and does **not** itself verify or apply the requested file side effect onto the host repo root.

Therefore:

> `completed` + real `task_run_id` + no host-visible file is currently consistent with the documented overlay/sync model.

This does **not** prove the prompt noop'd, and it does **not** prove world writes are broken.

### 2) Current `world-service` no longer unconditionally deletes clean bootstrap state

The older retained-worker bug was:

- a clean bootstrap exit could surface resumable/continuity identity,
- but `world-service` would still unregister/delete the retained member,
- causing later resume/follow-up to fail even though shell state implied continuity.

Current code shape no longer does that in the same unconditional way:

- `world-service` now preserves retained state on clean bootstrap exit when resumable session identity is surfaced (`uaa_session_id`-based continuity).
- `submit_turn` path and retained lifecycle tests appear aligned with the intended parked/resume contract.
- Packet 4’s narrowed proof target also appears aligned to “public follow-up over an already-created retained world-member slot,” not to a misleading pure public `start -> turn` story.

So the best current reading is:

> the old unconditional-delete behavior is not present in current `crates/world-service/src/member_runtime.rs`; whether it explains the current repro is not established by code alone.

`SPEC-63` remains draft lifecycle authority, not a closed-issue marker.

### 3) Leading code-path hypothesis: **mode-blind retained-slot preservation**

The strongest converged hypothesis is:

- shell treats `run_world_task` as a **terminal, ephemeral** path;
- but current `world-service` launch/preservation logic preserves clean bootstrap state based on `exit == 0` plus surfaced `uaa_session_id`, without an explicit action-specific guard excluding `run_world_task`’s `awm_...` participants;
- shell then returns a one-shot terminal result **without** retained shell state, while `world-service` may still retain a member slot keyed by:
  - `orchestration_session_id`
  - `world_generation`
  - `backend_id`

That creates a plausible shell/world-service state skew:

- shell thinks the one-shot task ended terminally,
- `world-service` still owns a retained slot for that session/world/backend,
- the next fresh launch can fail immediately as a duplicate retained-member / duplicate retained-slot allocation.

This is the leading code-path hypothesis for:

> later retries failing early with `failed to launch run_world_task over world member dispatch`

The file-visibility symptom and the later launch failure are therefore probably adjacent but distinct:

- **file symptom** -> likely overlay/sync visibility
- **retry launch failure** -> likely hidden retained-slot poisoning

### 4) There is also a broader shell/runtime contract mismatch

Separate from the likely retained-slot bug, the shell-side model still looks inconsistent across layers.

Most important mismatch:

- parked/awaiting-attention session truth is largely derived from **host detach continuity**,
- that state does **not** prove new world-member launch is realizable,
- exact retained-member follow-up exists on Linux via `/v1/member_turn/stream`, but the public lifecycle remains host-rooted and detached world follow-up fails closed until `substrate agent reattach` restores an active host owner,
- internal exact-target continue still appears to require stronger “authoritative-live” truth,
- some member startup/bootstrap paths still require a **live orchestrator parent**.

Net effect:

> a session can look healthy, parked, and resumable at the shell/control-plane surface while deeper world dispatch is already stale, non-routable, or launch-blocked.

The newest 2026-06-26 repro sharpens this further:

- one session failed honestly as **host-only / missing authoritative world binding**,
- a later session failed in the more important way: it appeared world-bound from one runtime surface, but `world-service` rejected dispatch on **exact `world_id` mismatch**, and the retained participant left behind by that attempt was immediately **not authoritative-live** for follow-up.

So the broader issue is not just one bug; it is also a **cross-layer contract mismatch** about what “routable after park/detach/bootstrap exit” actually means.

## What the cited runtime/docs actually establish

The cited code/tests/docs directly establish:

- the member-dispatch transport path for `run_world_task`,
- overlay-first write visibility under full isolation,
- separate host reconciliation / workspace sync semantics,
- retained-slot lifecycle keyed by session/world-generation/backend,
- stricter exact-target continue routing than detached host continuity alone.

They do **not** independently rule every other debugging hypothesis in or out.

## Current ownership read

Current repo truth suggests **split ownership**, not a confirmed owner transfer.

Best current ownership split:

- the old parked/resume lifecycle bug was correctly in the `SPEC-63` family;
- the still-live issue now looks more adjacent to the **direct member / world-dispatch bridge** family;
- the more relevant design owner appears to be:
  - `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md`
  - plus the `SPEC-62` adjacency / any follow-on bridge spec
  - while router/inbox/auto-attach docs remain downstream surfaces rather than primary owners.

Most likely architecture tension:

> the transitional direct `cli:codex-world` member bridge is being asked to behave like a fully truth-owned durable retained runtime, and that assumption may be too strong.

## Highest-value next discriminators

These are the best next debugging checks to separate confirmed truth from still-likely theory:

1. **Capture the exact world-service error body on the second fresh launch**
   - Goal: confirm or falsify duplicate retained-slot / duplicate retained-member rejection.

   Note: the newest concrete repro already added another first-class discriminator here:
   - exact `member_dispatch.world_id mismatch`
   - followed by `stale_linkage` on direct follow-up

2. **Inspect pending diff or world-side file presence immediately after the first completed run**
   - Goal: separate:
     - prompt noop,
     - overlay-only successful write,
     - sync/reconciliation failure.

3. **Confirm whether an ephemeral `run_world_task` left a retained member in `world-service` without a matching retained shell participant**
   - Goal: directly prove or disprove the shell/world-service split-brain hypothesis.

4. **Test the same retained worker through both routing surfaces**
   - public follow-up path vs internal exact-target continue
   - Goal: confirm or falsify the shell-side routing-definition split.

5. **Probe parked-host -> new member launch without manual reattach**
   - Goal: confirm whether parked session truth overstates actual launch viability because deeper paths still require a live parent.

## Short operational summary

If we need the shortest honest current diagnosis:

- the missing host file is most likely a **workspace sync / host visibility** issue, not proof of non-execution;
- the later fresh-launch failure is most likely a **retained-slot poisoning** issue caused by over-broad preservation of ephemeral `run_world_task`;
- the newest direct toolbox repro also shows a second live blocker family: **authoritative world-binding mismatch / stale follow-up routability**;
- and beneath both is a broader **shell/runtime contract mismatch** around parked host continuity vs world dispatch realizability.

## Guardrails for future troubleshooting

When using this file during debugging, keep these constraints explicit:

- treat live code/tests/docs as authority over this memo;
- do not treat “completed” as proof that the requested file side effect was host-visible;
- do not treat parked/resumable host posture as proof that world-member routing remains valid;
- do not widen this memo into a generic retained-worker architecture doc;
- if a later repro disproves duplicate retained-slot rejection, revise this file immediately because that is currently the leading runtime hypothesis.

## Recommended use

Use this file as the compact canonical memo for:

- issue recap,
- troubleshooting session bootstrap,
- follow-on debug packet planning,
- future subagent briefings.

Do **not** use it as a substitute for reading the live code when making implementation changes.
