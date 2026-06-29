# `run_world_task` / retained-world-member debug synthesis

Last updated: 2026-06-29  
Status: working canonical memo for this issue family; live code/tests/docs remain final authority.

## Purpose

This file merges the 2026-06-25 debug packets plus the later 2026-06-27 through 2026-06-29 manual smokes / transcript clarifications into one bounded source-of-truth memo we can reuse during troubleshooting without rereading every handoff.

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
- `.claude/handoffs/2026-06-27-075006-reattach-session2-runtime-attach.md`

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
- `llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md`

## Active symptom family

The live issue family has narrowed. Some earlier blockers are now closed by real manual smoke; the active work is now around retained-worker control-plane/linkage behavior and REPL parity after parked host turns.

What is now **confirmed fixed** by fresh manual smoke:

1. public world-scoped lifecycle path:
   - `substrate agent start --backend cli:codex-world --scope world ...`
   - then `reattach`
   - then `stop`
   - all succeeded on the same orchestration session lineage.
2. REPL first-dispatch `run_world_task` mismatch bug:
   - in a fresh REPL, the **first** sequential `run_world_task` no longer failed with `member_dispatch.world_id mismatch`,
   - both first and second launches succeeded,
   - both returned real `task_run_id` values.
3. world-mode lazy startup regression:
   - `substrate --world`
   - immediate `exit`
   - `~/.substrate/run/agent-hub/sessions` diff before/after was empty,
   - so no bogus persisted session dir was created on startup alone.
4. ordinary-command survivability after parked targeted host turn:
   - in a fresh REPL:
     - `::cli:codex-host just reply OK!`
     - then ordinary `ls` succeeds
     - then ordinary `pwd` succeeds
     - `cd ../` is blocked by the caged-root guard as expected

The still-live issues being debugged are now the combination of:

1. retained world-worker launch/follow-up paths still diverging from the repaired one-shot `run_world_task` path,
2. retained-worker follow-up/control operations failing across successor host participants with `stale_linkage`,
3. resumed targeted host turns still sourcing the wrong conversation/context on both REPL and public `agent start|turn` even when transport/session reuse appears correct,
4. worker turn execution failure (`codex exited non-zero`) remaining unresolved and distinct from the control-plane/linkage failures,
5. and the older host-visible-file question still remaining secondary to those routing/control-plane issues.

The current repo still exposes multiple relevant seams, but they no longer all sit at the same priority:

- the host-visible file/write question remains open and likely still points at overlay/sync behavior,
- while the top active blocker has shifted to resumed targeted host-turn continuity/context scoping, with retained-worker control-plane/linkage issues still immediately behind it.

Live code alone does **not** settle the host-visible write question, and the latest manual smoke only proves that certain routing/control surfaces now succeed. It does **not** yet prove the full end-to-end host-visible side-effect contract is settled.

## Latest observed repros

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

### Repro B (2026-06-26): world-bound-looking session still fails on exact binding, then follow-up is stale

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

### Repro C (2026-06-28): public world start plus reattach/stop works; REPL first-dispatch `run_world_task` also works

In a fresh public smoke on Linux:

- `substrate agent start --backend cli:codex-world --scope world --prompt "Reply with exactly START_OK" --json` succeeded,
- `substrate agent reattach --session <same orchestration_session_id> --json` succeeded,
- `substrate agent stop --session <same orchestration_session_id> --json` succeeded.

What this means:

- the current landed reattach/runtime-attach work is behaving consistently with the intended contract,
- there is no evidence in this smoke that `reattach` is still attempting prompt-free backend/UAA resume,
- the active blocker has moved away from generic parked-host reattach failure.

In a separate fresh interactive `~/.substrate/bin/substrate` REPL session, the shell-owned orchestrator then attempted two sequential toolbox `run_world_task` launches targeting `cli:codex-world` in the same current session/world binding. The results were:

> First `run_world_task` succeeded and returned `task_run_id = spn_019f0b77-f050-7ed1-9bd8-16ec37a1e82a`

> Second `run_world_task` succeeded and returned `task_run_id = spn_019f0b77-ff75-7ae3-9585-0a521a09efd6`

Separately, a fresh `substrate --world` followed by immediate `exit`, with `~/.substrate/run/agent-hub/sessions` diffed before/after, produced no new persisted session directories.

Why this matters:

- public world-scoped `start -> reattach -> stop` is currently healthy,
- the old first-dispatch REPL `run_world_task` mismatch bug is no longer the newest blocker,
- and world-mode lazy startup is behaving correctly again.
- but these smokes validated routing/lifecycle surfaces, not the full host-visible write contract.

### Repro D (2026-06-28): retained worker path still mismatches, then suffers stale-linkage and active-member blockage

In newer manual smoke:

- fresh REPL `spawn_world_worker` still failed on first dispatch with exact `member_dispatch.world_id mismatch (...)`;
- public `substrate agent start --backend cli:codex-world --scope world ...` with a `run_world_task` prompt succeeded and returned a real `task_run_id`;
- public `substrate agent turn --session ... --backend cli:codex-host` with `spawn_world_worker` succeeded and returned retained worker participant `ash_019f0bed-b582-7f50-ace2-67cc1eea6f60`;
- `continue_world_worker` reached that worker but the worker turn itself failed with:

> `codex exited non-zero: ExitStatus(unix_wait_status(256)) (stderr redacted)`

- later `inspect_world_worker` against that retained worker failed with exact `stale_linkage`, saying the retained worker was no longer linked to the new authoritative orchestrator participant;
- later `stop_world_worker` against that same retained worker failed with the same `stale_linkage` class;
- later `run_world_task` was then blocked because the retained worker still counted as active, failing with:

> `a retained world member is already active for orchestration_session_id ... backend_id cli:codex-world ...`

Why this matters:

- the repaired one-shot `run_world_task` path and the retained-worker path have clearly diverged,
- `spawn_world_worker` still appears to need the same kind of first-dispatch binding repair that `run_world_task` received,
- the retained-worker follow-up contract across successor public host participants is still broken,
- and the worker execution failure itself should stay separate from the control-plane/linkage bugs.

## Synthesized working model

### 1) Missing host-visible file is probably **not** the primary runtime failure, but it remains explicitly open

Current repo truth strongly supports this interpretation:

- `run_world_task` is translated into typed world-member dispatch, not a host-root direct write path.
- Under full isolation, relative writes land in the **world overlay view first**.
- Host visibility for those writes is expected only after workspace reconciliation / sync from the world view back to the host workspace.
- The one-shot `run_world_task` path currently terminates on streamed start/event/exit frames and does **not** itself verify or apply the requested file side effect onto the host repo root.

Therefore:

> `completed` + real `task_run_id` + no host-visible file is currently consistent with the documented overlay/sync model.

This does **not** prove the prompt noop'd, and it does **not** prove world writes are broken.

It also does **not** prove the host-visible side-effect contract is fully healthy. Because lower retained-worker / routing / parity surfaces are still unstable, successful one-shot `run_world_task` smoke should be read as evidence that dispatch succeeded, not as conclusive proof that host-visible write semantics are fully settled.

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

### 3) Earlier retained-slot poisoning was real, but is no longer the best primary explanation for the repaired one-shot REPL repro

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

That hypothesis still matters historically and remains relevant to older failure shapes, especially the earlier `awm_*` cleanup work already landed in `world-service`.

But the later 2026-06-28 REPL smoke narrows the current priority:

> if both first and second sequential `run_world_task` launches now succeed in the same fresh REPL session, then first-dispatch one-shot `run_world_task` binding mismatch is no longer the strongest active blocker.

For the still-broken retained-worker path, the better leading hypotheses are:

- `spawn_world_worker` still has a first-dispatch authoritative world-binding mismatch path not yet repaired equivalently,
- retained-worker authority is being validated too literally against successor public host participants,
- and worker lifecycle/control-plane truth can remain inconsistent after a failed or successor-routed follow-up.

Design-backed interpretation:

> the observed `stale_linkage` behavior likely represents a contract bug relative to session-rooted retained-worker ownership; the open implementation question is the exact normalization / authority-transfer mechanism across successor host participants, not whether successor follow-up should be possible at all.

That points more toward:

- retained-worker binding refresh/repair,
- successor-participant linkage / authoritative-live normalization,
- and separation of worker execution failure from worker control-plane authority truth

rather than toward the earlier one-shot `run_world_task` bootstrap issue alone.

### 4) There is also a broader shell/runtime contract mismatch and a REPL/CLI parity requirement

Separate from the likely retained-slot bug, the shell-side model still looks inconsistent across layers.

Most important mismatch:

- parked/awaiting-attention session truth is largely derived from **host detach continuity**,
- that state does **not** prove new world-member launch is realizable,
- exact retained-member follow-up exists on Linux via `/v1/member_turn/stream`, but the public lifecycle remains host-rooted and detached world follow-up fails closed until `substrate agent reattach` restores an active host owner,
- internal exact-target continue still appears to require stronger “authoritative-live” truth,
- some member startup/bootstrap paths still require a **live orchestrator parent**.

Net effect:

> a session can look healthy, parked, and resumable at the shell/control-plane surface while deeper world dispatch is already stale, non-routable, or launch-blocked.

The 2026-06-28 and 2026-06-29 transcripts also sharpened the intended product contract:

- REPL and CLI are UX wrappers over the same underlying lifecycle mechanisms; they should not diverge semantically.
- Inside REPL, ordinary commands like `ls` / `pwd` / `cd` should behave like `substrate -c ...`.
- Inside REPL, `::cli:codex-* prompt` should behave like `substrate agent start|turn`, with the REPL mainly handling session tracking and related niceties.
- The REPL therefore has three distinct input classes:
  - targeted, prompt-bearing `::cli:<backend> ...`
  - ordinary unprefixed shell/world commands like `ls`, `pwd`, `cd`
  - explicit `:pty ...` commands
- Those classes must stay distinct.
- UAA/Codex does not support promptless starts or turns by product decision, so ordinary commands cannot be modeled as promptless targeted launches.

Older smokes exposed a distinct REPL bug class:

- fresh REPL
- `::cli:codex-host just reply OK!`
- clean park
- next `ls`
- failure: `member launch requires exactly one live orchestrator parent, but none is active`

Best current read:

> after a cleanly parked REPL `::cli:codex-host` turn, the enclosing REPL/world session incorrectly loses the live orchestrator parent required for later ordinary commands, which appears to conflate attached/live parent truth with durable orchestration session truth.

The older 2026-06-26 repro sharpens this further:

- one session failed honestly as **host-only / missing authoritative world binding**,
- a later session failed in the more important way: it appeared world-bound from one runtime surface, but `world-service` rejected dispatch on **exact `world_id` mismatch**, and the retained participant left behind by that attempt was immediately **not authoritative-live** for follow-up.

That older failure mattered because it exposed a broader **cross-layer contract mismatch** about what “routable after park/detach/bootstrap exit” actually means.

The newer smokes add one more refinement:

- the public `start -> reattach -> stop` lifecycle can now behave correctly,
- the one-shot REPL `run_world_task` first-dispatch path can now also behave correctly,
- ordinary unprefixed REPL commands after a parked targeted host turn now materially improve in live smoke:
  - `ls` succeeds
  - `pwd` succeeds
  - `cd ../` is blocked by the caged-root guard as expected
- while the retained-worker path and resumed targeted host-turn semantic continuity are still wrong.

So the current live mismatch is narrower than “reattach is broken” or “all world dispatch is broken.” The main remaining parity failure is now resumed targeted host-turn continuity/context sourcing, not ordinary-command survivability after park.

## What the cited runtime/docs actually establish

The cited code/tests/docs directly establish:

- the member-dispatch transport path for `run_world_task`,
- overlay-first write visibility under full isolation,
- separate host reconciliation / workspace sync semantics,
- retained-slot lifecycle keyed by session/world-generation/backend,
- stricter exact-target continue routing than detached host continuity alone.
- prompt-free `reattach` is intentionally Substrate-owned control-plane recovery, not a UAA/backend resume operation.

They do **not** independently rule every other debugging hypothesis in or out.

The 2026-06-27 and 2026-06-28 transcript/manual-smoke set also establish these now-current truths:

- the local UAA contract in `/home/spenser/__Active_code/unified-agent-api` intentionally rejects prompt-free resume,
- the landed attach/runtime changes were specifically designed to keep prompt-free owner recovery downstream in Substrate,
- the public `reattach` path can now succeed without sending a hidden prompt or invoking prompt-free backend/UAA resume.

The same transcript also established the posture/auto-attach language we should preserve:

- do **not** describe any posture as implying a continuously running background backend process;
- durable orchestration state is exclusively **Substrate-owned**;
- backends run headless/ephemerally as needed, for Codex via `codex exec`;
- `active_attached` should be read as attached/ready **now** in Substrate-owned session terms, not as proof that a backend process has been continuously alive;
- auto-attach is obligation/inbox-driven host ownership restoration, not an always-live agent loop.

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

## Parity regressions to preserve

These are now part of the intended contract, not just optional nice-to-have tests.

1. **REPL parity regression**
   - fresh REPL
   - `::cli:codex-host just reply OK!`
   - ordinary unprefixed `ls`
   - ordinary unprefixed `pwd`
   - ordinary unprefixed `cd ../`
   - `::cli:codex-host what was the exact, full message I sent as the first user message in this session?`
   - the same parked durable orchestration session is reused
   - `resumed_from_participant_id` is present on the resumed targeted host turn
   - the same UAA session id appears to persist
   - ordinary commands still work or fail for the right reason:
     - `ls` succeeds
     - `pwd` succeeds
     - `cd ../` is blocked by the caged-root guard
   - but the resumed targeted host answer should come from the Substrate session-local `just reply OK!`, not from broader outer thread / `AGENTS.md` context

2. **CLI-equivalent parity regression**
   - `substrate agent start --backend cli:codex-host --prompt "just reply OK!"`
   - normal shell command in between, e.g. `substrate -c "ls"` or `pwd`
   - `substrate agent turn --session <same_session> --backend cli:codex-host --prompt "tell me what my last message said"`
   - the same durable orchestration session is reused
   - `resumed_from_participant_id` is present
   - the same UAA session id appears to persist
   - but the answer should still be effectively `just reply OK!`, not broader outer conversation context

Both should also validate:

- exact `orchestration_session_id` continuity,
- expected participant successor behavior,
- persisted UAA/Codex continuity markers when present,
- shell execution in between does not invalidate the durable session,
- and later follow-up does not silently create a new unrelated session.

## Highest-value next discriminators

These are the best next debugging checks to separate confirmed truth from still-likely theory:

1. **Trace resumed targeted host-turn continuity / context hydration**
   - Goal: explain why REPL and public `agent start|turn` reuse the same orchestration session and resumed-participant lineage, but still answer from broader outer conversation context instead of the session-local prompt history.
   - Most likely seams:
     - host-turn resume/context assembly near the REPL targeted-turn path
     - public `agent start|turn` continuity handoff
     - pre-UAA conversation payload sourcing on resumed targeted turns

2. **Trace `spawn_world_worker` first-dispatch world-binding handoff**
   - Goal: identify where the retained-worker launch path still disagrees on `world_id` / `world_generation` while one-shot `run_world_task` no longer does.
   - Most likely seams:
     - `crates/shell/src/repl/async_repl.rs`
     - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
     - `crates/shell/src/execution/repl_persistent_session.rs`

3. **Trace retained-worker authority/linkage across successor host participants**
   - Goal: explain why `inspect_world_worker` / `stop_world_worker` fail with `stale_linkage` after a later public host participant becomes authoritative.

4. **Separate worker execution failure from control-plane failure**
   - Goal: keep `continue_world_worker` worker-turn `codex exited non-zero` analysis separate from binding/linkage bugs so they do not get conflated.

5. **Keep ordinary-command survivability as a regression baseline**
   - Goal: preserve live `ls` / `pwd` success and expected caged-root `cd ../` denial after park while fixing resumed targeted host-turn continuity.

6. **Prove the REPL and CLI parity regressions together**
   - Goal: confirm both wrappers preserve the same durable session truth, the same resumed continuity markers, and the correct session-local conversation continuity.

7. **Keep the file-visibility question secondary**
   - Goal: only revisit host-visible side-effect sync once the retained-worker and REPL parity/control-plane bugs are cleanly separated.

## 2026-06-29 patch sequence and current truth

The memo above now needs one more layer of truth added from the most recent patch/review/smoke loop.

### What landed and validated

1. **Parked detached host authority is now accepted for later targeted host turns**
   - A narrow REPL patch changed ordinary host follow-up routing so clean park no longer fails immediately on:
     > `member launch requires exactly one live orchestrator parent, but none is active`
   - Review validated the production seam.
   - Follow-up test hardening then proved the later `::cli:codex-host ...` turn is using the same durable Substrate orchestration-session continuity rather than merely replaying fixture-side prompt memory.

2. **The strengthened REPL continuity regression is now meaningful**
   - The hardened REPL regression now verifies:
     - same `orchestration_session_id`,
     - same continuity selector,
     - no intervening host relaunch from ordinary `ls`,
     - and valid participant reuse or successor lineage on the later targeted host turn.
   - Review validated that test hardening.

3. **The shim logger redaction fix is separate and healthy**
   - A separate session fixed the `substrate-shim` header redaction failure where `X-API-Key: ...` could leak in one mixed redaction path.
   - That fix is unrelated to the world-dispatch / parked-session issue family and should not be conflated with the REPL/runtime work here.

### What landed but was invalidated

Two successive patches tried to fix the stale world-binding problem by repairing REPL authoritative binding before ordinary member startup.

Both were **partially right but incomplete / incorrect**:

1. **First stale-binding patch**
   - Added REPL authoritative binding synchronization before ordinary member bootstrap.
   - Review invalidated it because the helper short-circuited on any persisted `session.json` binding and therefore never reached shared-world metadata repair when persisted session truth itself was stale.
   - That same bug also broke the `run_world_task` retry helper once it was routed through the shared sync helper.

2. **Second stale-binding precedence patch**
   - Changed precedence so shared-world metadata wins over stale persisted session binding.
   - Review validated the local helper logic and targeted tests.
   - But live manual smoke still failed on the real CLI/REPL surface, so this was still **not** the full runtime fix.

### What the latest live runtime smoke established

This older live smoke was the most important discriminator at that stage, but it is no longer current top-line truth after the later 2026-06-29 fixes and newest smoke:

1. fresh REPL
2. `::cli:codex-host just reply OK!`
3. host parks cleanly
4. `ls`
5. failure:
   > `HTTP 400 Bad Request error: {"error":"member_dispatch.world_id mismatch (expected wld_..., got wld_019f08e8-dce6-74b0-9dc0-743ad2d297d8)"}`

At that point, the smoke mattered because it proved:

- the old live-parent gate was no longer the active blocker on this path;
- ordinary REPL world-command/member bootstrap was still sending a stale `world_id`;
- and the stale `world_id` was stable across attempts, which strongly suggested a stale REPL/session-side binding source rather than random launch drift.

That older smoke should now be read as historical seam chronology only. Newer 2026-06-29 live smoke advanced this further:

- after `::cli:codex-host just reply OK!`, ordinary unprefixed `ls` succeeds,
- ordinary unprefixed `pwd` succeeds,
- `cd ../` is blocked by the caged-root guard as expected,
- and the remaining user-visible bug is no longer ordinary-command `world_id mismatch`, but resumed targeted host-turn continuity/context sourcing.

Another important discriminator from that same older smoke set:

- a **fresh REPL with no prior host turn** can still run `ls` successfully;
- the failure is specifically introduced **after** the host orchestrator parks and later ordinary commands try to relaunch/bootstrap a world member.

### What Wegener clarified

Wegener’s probe was the most useful narrowing step so far.

He found:

1. **The stale `world_id` is coming from REPL/session truth, not from `agent status` projection**
   - The parked orchestration session row still held old `wld_019f08e8-...`.
   - Shared-world metadata for the same orchestration session already held the newer authoritative `wld_019f115b-...`.
   - The failed member bootstrap then copied that old stale binding into the member manifest.

2. **`world-service` is behaving correctly**
   - The rejection is the exact world-service guard:
     - `validate_member_dispatch_binding(...)`
   - So the service is not the bug here; it is exposing the stale client-side binding.

3. **The missing behavior is retry/repair on member bootstrap**
   - `run_world_task` already has explicit mismatch retry/repair.
   - ordinary REPL world-command bootstrap does not.
   - `spawn_world_worker` bootstrap also does not.

4. **Current tests are architecturally too weak for this seam**
   - The REPL world-service stub does **not** model an independent authoritative binding strongly enough.
   - It accepts whatever `dispatch.world_id` the client sends instead of rejecting stale dispatches the way the real `world-service` does.
   - That is why we got false-green test results while the real CLI smoke still failed.

5. **`agent status` weirdness is mostly diagnostic fallout**
   - `agent list` correctly shows `codex-world` as a world-scoped backend.
   - The strange later `agent status` `source_kind=trace_fallback` / `role=member` / `execution.scope=host` rows are symptoms of failed member launches and degraded projection/trace metadata, not the root cause of the stale dispatch.

### Updated interpretation

The memo above is no longer current enough on the patch/review loop.

The stale-binding issue did not stop at “ordinary REPL command needs retry/repair.” The later patch sequence established a more precise shape:

> ordinary **unprefixed** REPL world commands got the narrow repair seam, detached targeted `::cli:codex-world ...` follow-up still remains fail-closed, and Linux authority must come from shared-world metadata rather than shell session truth. Explicit `:pty` no longer belongs in that fail-closed bucket.

That led to several thin patches and reviews.

## Later landed patches and review outcomes

### 1. Generic member-bootstrap retry patch was invalidated

- A first patch added ordinary member-bootstrap retry/repair too low in the shared bootstrap path.
- Review found it was too broad because it also affected targeted world follow-up bootstrap, could leave ghost first-attempt bootstrap state, and matched the retry trigger too loosely.
- This patch should be treated as rejected.

### 2. Narrower ordinary-command retry patch was also invalidated

- A later patch tried to narrow the retry to ordinary REPL command bootstrap and tighten the mismatch classifier.
- Review still invalidated it because:
  - it still relied on the wrong authority source in tests and helper flow,
  - it still widened beyond the exact design boundary,
  - and the positive success regression either did not run or was not strong enough at that stage.

### 3. Shared-metadata-only ordinary-command repair path landed

- A subsequent patch sequence did land the important design split:
  - ordinary **unprefixed** REPL world commands use the repair seam,
  - targeted `::cli:codex-world ...` remains fail-closed,
  - explicit `:pty` continues to route through persistent-session PTY execution rather than the removed fail-closed expectation,
  - and ordinary mismatch repair no longer uses shell session truth when shared-world metadata is absent.
- Review still found one remaining authority broadening in the wider REPL sync path, which was then tightened so the REPL sync helper also stopped consuming unreadable-metadata live-participant fallback on this surface.
- Current status:
  - this narrowing is what got the ordinary-command path past the older parked-session survivability failure and then past the old first `world_id mismatch` failure,
  - and later live smoke shows ordinary-command survivability has now materially improved,
  - while the remaining live user-visible bug has moved to resumed targeted host-turn continuity/context sourcing across REPL and public `agent start|turn`.

### 4. Positive success regression was dead code, then fixed

- Review later discovered the intended positive regression for:
  - parked host
  - ordinary unprefixed world-backed command
  - later targeted host follow-up continuity
  was present but not actually registered as a test.
- A tiny follow-up patch added the missing test attributes.
- Review validated that final test-enablement patch.
- So the positive regression is now actually executing; that was enablement, not a behavior fix by itself.

### 5. Clippy-only patch was separate and healthy

- A separate thin patch bundled arguments for `start_remote_member_runtime_with_binding_retry` to fix `clippy::too_many_arguments`.
- That patch was a pure signature/parameter-struct cleanup and should not be conflated with runtime behavior changes.

## What the current runtime smoke now says

The latest real CLI/REPL smoke is now more important than the earlier `world_id mismatch` smoke.

### REPL ordinary unprefixed commands after parked targeted host turn

Observed in the newest live smoke:

1. fresh REPL
2. `::cli:codex-host just reply OK!`
3. host parks cleanly
4. ordinary unprefixed `ls`
5. ordinary unprefixed `pwd`
6. ordinary unprefixed `cd ../`
7. later targeted host follow-up asking:
   - `what was the exact, full message I sent as the first user message in this session?`

Results:

- `ls` succeeds,
- `pwd` succeeds,
- `cd ../` is blocked by the caged-root guard as expected,
- the later targeted host turn resumes over the same durable orchestration-session continuity,
- but the semantic answer is wrong: it comes from broader outer conversation / `AGENTS.md` context rather than the Substrate session-local `just reply OK!`.

Interpretation:

- ordinary unprefixed-command survivability after parked host has materially improved in live smoke and should no longer be described as the main live bug,
- transport/session reuse is not the primary failure on this seam anymore,
- the remaining user-visible bug is resumed targeted host-turn context sourcing.

### Explicit `:pty` directive truth

Observed in adjacent regression coverage:

- `:pty echo hello` routes to persistent-session exec passthrough with the stripped command payload.

Interpretation:

- explicit `:pty` is not currently a fail-closed targeted-world seam;
- stale memo language claiming explicit `:pty` fail-closed coverage existed should now be read as superseded history, not current truth.

### Latest seam read from the two most recent probe passes

Best current seam split:

1. **Seam A: ordinary unprefixed commands**
   - live smoke now shows parked host -> ordinary unprefixed `ls` / `pwd` success and expected caged-root `cd ../` denial,
   - adjacent regression coverage also covers the implicit-PTY ordinary-command lane,
   - so this seam has advanced past the earlier first-failure descriptions,
   - but that should not be over-read as closure for every caller boundary.

2. **Seam B: implicit PTY ordinary commands**
   - adjacent coverage now shows a positive parked-host implicit-PTY ordinary-command resume path,
   - so this seam is narrower than before,
   - but it is no longer the top live blocker.

3. **Seam C: resumed targeted host-turn semantic continuity**
   - REPL and public CLI both now show reuse of transport/session identity,
   - but both can still source the wrong conversation history on the resumed targeted host turn,
   - so this is the top active live seam.

### Public `agent start` / `agent turn` continuity is still semantically wrong

Observed:

1. `substrate agent start --backend cli:codex-host --prompt 'just reply OK!'`
2. ordinary shell command outside the agent surface succeeds
3. `substrate agent turn --session <same> --backend cli:codex-host --prompt 'tell me what my last message said'`
4. the same orchestration session is reused
5. `resumed_from_participant_id` is present
6. the same UAA session id appears to persist
7. but the reply still comes from broader conversation context, not effectively `just reply OK!`

Interpretation:

- the public durable-session continuity surface now looks transport-correct but semantically wrong;
- the remaining problem is not “new session accidentally created” but “wrong context hydrated onto the resumed targeted turn”;
- this seam matches the REPL failure closely enough that they should now be treated as one top-priority continuity/context-sourcing bug family.

## What is now actually landed and trusted

At this point the following narrow claims appear trustworthy:

1. **One-shot `run_world_task` stale-binding mismatch is no longer the primary live blocker**
   - that earlier seam appears repaired enough that later runtime smoke moved on to different failures.

2. **Ordinary-command survivability after parked targeted host turn is materially improved in live smoke**
   - after `::cli:codex-host just reply OK!`, ordinary unprefixed `ls` and `pwd` succeed and `cd ../` is denied by the caged-root guard as expected.

3. **Resumed targeted host turns now appear to reuse transport/session identity correctly**
   - same `orchestration_session_id` is reused,
   - `resumed_from_participant_id` is present,
   - the same UAA session id appears to persist,
   - but semantic conversation continuity is still wrong.

4. **The parked-session survivability slice that removed the old `no live orchestrator parent` first failure is landed**
   - that older failure is no longer the first blocker on the ordinary unprefixed path.

5. **The current test split is now reflected in adjacent coverage**
   - targeted `::cli:codex-world ...` fail-closed coverage exists
   - explicit `:pty` positive persistent-session routing coverage exists
   - the positive parked-host -> ordinary unprefixed implicit-PTY command -> targeted host resume regression now actually executes

6. **But the live failure has moved above the current regression focus**
   - tests currently validate important routing boundaries,
   - while the newest live bug is resumed targeted host-turn semantic continuity/context sourcing across both REPL and public CLI.

7. **Invalidated vs retained patch slices are now clearer**
   - invalidated:
     - the generic member-bootstrap retry patch
     - the later narrower retry patch that still widened the seam incorrectly
   - retained:
     - parked-session survivability work that got past the old live-parent failure
     - the shared-metadata-only narrowing for ordinary unprefixed binding repair
     - the positive regression enablement patch that made the intended success test actually run

## Updated active blockers

The open bug buckets should now be read in this order:

1. **Resumed targeted host-turn continuity/context scoping is still wrong across both REPL and public `agent start|turn`**
2. **retained-worker bootstrap parity (`spawn_world_worker`) is still open**
3. **retained-worker authority/linkage across successor host participants (`stale_linkage`) remains open**
4. **worker turn execution failure (`codex exited non-zero`) remains a separate issue**
5. **implicit-PTY ordinary-command caller-boundary clarification/coverage is still useful, but it is no longer the top live blocker**
6. **test harness still does not fully model every live caller boundary or resumed-context seam**
7. **host-visible file/write semantics remain open but are not first in priority**

Do not describe ordinary-command survivability after parked host as the primary open bug in this memo anymore:

- the newest live smoke shows meaningful success on `ls` / `pwd` and the expected caged-root denial on `cd ../`,
- so the main remaining failure is semantic continuity on resumed targeted host turns.

## Next planned landing order

1. **Fix resumed targeted host-turn continuity/context sourcing first**
   - target the wrong prompt/history hydration on resumed targeted host turns in both REPL and public `agent start|turn`.

2. **Keep ordinary-command survivability as the live regression baseline**
   - do not regress parked-host -> ordinary unprefixed `ls` / `pwd` success, expected caged-root `cd ../` denial, or the adjacent implicit-PTY regression while fixing continuity.

3. **Then return to retained-worker bootstrap/linkage seams**
   - resume `spawn_world_worker` first-dispatch parity and successor-linkage work once the top continuity/context bug is cleanly separated.

4. **Only after the live continuity bug is fixed, do the structural enum/type cleanup before the UAA boundary**
   - replace ambiguous `Option<prompt>` / launch-policy semantics with an explicit pre-UAA representation so ordinary commands and prompt-bearing turns cannot be conflated accidentally.
   - this is still wanted, but it should be treated as hardening/refactor work after the live continuity bug is closed, not as the next patch.

## Short operational summary

If we need the shortest honest current diagnosis:

- the missing host file is still most likely a **workspace sync / host visibility** issue, not proof of non-execution;
- the host-visible side-effect contract is still **open**;
- the earlier one-shot REPL first-dispatch `run_world_task` stale-binding mismatch is no longer the first live blocker;
- ordinary-command survivability after parked targeted host turn is materially better in live smoke:
  - `ls` succeeds
  - `pwd` succeeds
  - `cd ../` is denied by the caged-root guard as expected
- explicit `:pty` should not be described as fail-closed on this memo's current truth;
- REPL and public CLI both now appear to reuse the same transport/session continuity markers on resumed targeted host turns;
- but the top active bug is still resumed targeted host-turn context sourcing, because the answer is coming from broader outer conversation context instead of the session-local `just reply OK!`;
- still-open likely bug buckets are:
  - resumed targeted host-turn continuity/context scoping,
  - public durable-session continuity semantics,
  - retained-worker bootstrap parity,
  - retained-worker successor-linkage / `stale_linkage`,
  - worker non-zero execution failures,
  - and remaining harness realism gaps.

## Guardrails for future troubleshooting

When using this file during debugging, keep these constraints explicit:

- treat live code/tests/docs as authority over this memo;
- do not treat “completed” as proof that the requested file side effect was host-visible;
- do not treat parked/resumable host posture as proof that world-member routing remains valid;
- do not assume `reattach` is still the active blocker unless a fresh repro shows it; the latest smoke says it is not;
- do not use shell session JSON as co-authoritative Linux world-binding proof for the ordinary unprefixed REPL command repair seam;
- do not assume shared-world metadata absence/unreadability is repairable on this REPL surface unless the design/docs explicitly say so;
- do not describe `active_attached` or auto-attach in process-liveness terms; the transcript clarified that all durable truth is Substrate-owned and backend execution is headless/ephemeral;
- do not say explicit `:pty` fail-closed coverage exists; current adjacent truth is positive persistent-session `:pty` routing coverage;
- do not treat REPL as a semantically distinct lifecycle surface from CLI; parity is part of the product contract;
- distinguish:
  - ordinary unprefixed REPL commands,
  - `:pty` commands,
  - targeted `::cli:...` prompt-bearing turns,
  because the intended repair/fail-closed behavior differs across those surfaces;
- do not widen this memo into a generic retained-worker architecture doc;
- if later repros keep showing “first `spawn_world_worker` dispatch mismatches, later launch or public follow-up succeeds,” prioritize retained-worker first-dispatch world-binding and successor-linkage seams over the already-repaired one-shot `run_world_task` path.

## Recommended use

Use this file as the compact canonical memo for:

- issue recap,
- troubleshooting session bootstrap,
- follow-on debug packet planning,
- future subagent briefings.

Do **not** use it as a substitute for reading the live code when making implementation changes.
