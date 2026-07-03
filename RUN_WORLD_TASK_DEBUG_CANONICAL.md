# `run_world_task` / retained-world-member debug synthesis

Last updated: 2026-07-03 (stop-path design/spec follow-through)  
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
- `DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md`
- `llm-last-mile/SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`
- `llm-last-mile/PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`
- `llm-last-mile/TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`

Primary repo-truth surfaces repeatedly cited by those logs:
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agent_runtime/control.rs`
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

The live issue family has narrowed, but it has also moved through several distinct retained-fork stages. Some earlier blockers are now closed by real manual smoke; the active work is no longer the original bootstrap mismatch, no longer the first exposed `authoritative-live` seam, no longer best summarized as only the stream-launch/bootstrap wrapper or post-registration child-publication gate either, and no longer centered on exact retained continue. The newest valid smoke now gets all the way through retained fork creation, exact retained continue on both the source worker and the fork child, and leaves a later seam: exact retained stop on the original retained source worker now failing in the stop-closeout path, with host-visible file semantics still secondary.

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
5. resumed targeted host-turn continuity on both host surfaces:
   - the root cause was in `submit_host_prompt_turn`: it could prefer the frozen `PromptSubmitRuntime.uaa_session_handle_id` even after later turns refreshed the persisted continuity truth,
   - the current patch now prefers persisted host-attach-contract continuity first, then the live manifest `internal.uaa_session_id`, and only falls back to the frozen runtime field last,
   - the REPL strict follow-up path now passes,
   - the public `substrate agent start --backend cli:codex-host ...` then `substrate agent turn --session ... --backend cli:codex-host ...` strict follow-up smoke also now passes,
   - and the validated public smoke kept the same `orchestration_session_id`, surfaced `resumed_from_participant_id`, preserved the same UAA session id across `start` and `turn`, and returned the correct semantic answer: `Just reply OK`.

The still-live issues being debugged are now the combination of:

1. the latest valid world-bound smoke now shows:
   - `agent start`: pass,
   - retained `spawn_world_worker`: pass,
   - retained `fork_world_worker`: pass,
   - the fork call returned a real child `participant_id`,
   - exact retained `continue_world_worker` on the source worker: pass,
   - exact retained `continue_world_worker` on the fork child: pass;
2. but retained follow-up/control lifecycle is still broken later in the same smoke:
   - generic same-backend follow-up now correctly fails closed with:
     - `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`
   - exact retained `stop_world_worker` on the original retained source worker now fails in live smoke with the new closeout seam:
     - `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`;
3. the older retained successor `authoritative-live` seam, the intermediate stream-launch/bootstrap wrapper seam, the later post-registration durability/publication seam, and the later exact-retained-continue seam all exposed real bugs, but the newest smoke now places the active seam **after** successful fork creation and successful exact retained continue rather than before it;
4. the strongest current read is therefore no longer “fork child creation/publication never happened,” no longer “backend-only post-fork routing still picks the wrong target,” and no longer “exact retained continue still uses an over-strict `authoritative-live` seam after fork succeeds,” but rather “backend-only routing now ambiguity-closes correctly, exact retained continue is green, and source-worker stop now falls over in private-stop delivery / durable closeout”;
5. retained-worker follow-up/control operations still remain intentionally split by seam boundary:
   - retained `fork_world_worker` and retained-mode `cancel_world_work` are supposed to keep successor-lineage authority,
   - **branch-local landed / committed truth (`484a17b6`):** successor-authority lineage is now accepted for retained follow-up selector resolution, including retained `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker`, and exact retained continue no longer depends on owner PID liveness as a correctness condition,
   - the remaining live failure is later than lineage/world-binding selection and later than exact retained continue routing; it now sits specifically in source-worker `stop_world_worker` private-stop delivery / durable closeout,
6. the branch-local shell observability patch and timeout-split / durable-publication wait improvements were both useful and review-clean during this chain, and later landed retained-worker routing work moved the seam far enough that the newest smoke now reaches source-worker stop closeout;
7. the Boyle/Nash follow-up patch chain improved same-backend routing, workspace-root handling, and test coverage, and committed `484a17b6` changed retained follow-up and public-target selector behavior enough that generic backend-only post-fork routing now ambiguity-closes instead of stale-linking to a single target while exact retained continue goes green on both exact handles;
8. adjacent retained-mode `cancel_world_work` has not yet been fully re-smoked through the newer retained-fork patch chain and may still expose either the older `authoritative-live` seam or a newer downstream eligibility seam,
9. worker turn execution failure (`codex exited non-zero`) remains unresolved and distinct from the retained follow-up/control-plane failures,
10. and the older host-visible-file question still remains secondary to those routing/control-plane issues.

The current repo still exposes multiple relevant seams, but they no longer all sit at the same priority:

- the host-visible file/write question remains open and likely still points at overlay/sync behavior,
- while the top active blockers have shifted away from the old retained-worker bootstrap mismatch, then through retained successor `authoritative-live`, then through the temporary stream-launch/bootstrap wrapper seam, then through the post-registration durability/publication seam, then through backend-only post-fork retained-target ambiguity, then through exact retained continue routability, and now to source-worker `stop_world_worker` failing in private-stop delivery / durable stop closeout after successful fork creation.

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
- the first concrete divergence is now narrowed to retained-worker bootstrap context:
  - `run_world_task` keeps `prepared.session.workspace_root` through launch,
  - `spawn_world_worker` resolves contract against `prepared.session.workspace_root`, then switches to `std::env::current_dir()` before launch,
  - and that downstream cwd likely drives the wrong world/policy/project-dir selection, causing the retained-worker `world_id` mismatch;
- the same bootstrap-context drift likely also exists in the fork/bootstrap clone path;
- retained-worker follow-up across successor public host participants is still broken, but that now looks separate from the bootstrap bug;
- and the worker execution failure itself should stay deferred until bootstrap parity and successor-authority/linkage are fixed.

### Repro E (2026-06-30): retained-worker bootstrap parity now passes live, but successor `fork` / retained `cancel` still fail with `authoritative-live`

In fresh manual smoke on Linux:

- `substrate agent start --backend cli:codex-world --scope world --prompt 'Reply with exactly START_OK' --json` succeeded;
- the world-bound orchestration session surfaced stable:
  - `orchestration_session_id = 019f192a-bdc2-7990-b1fa-e99fa5ba4a0a`
  - `world_id = wld_019f192a-bdcb-7762-b380-23700093c61a`
  - `world_generation = 0`
- a later public host turn using `spawn_world_worker` in retained mode against that same orchestration session succeeded and returned a real retained worker:
  - `participant_id = ash_019f192d-c0d7-7153-852a-f6acb572f56c`
- this same smoke also preserved the same backend-native continuity handle:
  - `uaa session id = 019f192a-c0d8-72c2-b113-196152a50544`

That means:

- the older retained-worker bootstrap `member_dispatch.world_id mismatch` bug did **not** reproduce in this smoke;
- retained-worker bootstrap parity is now behaving consistently enough to stop treating it as the top live blocker.

But the next successor-host follow-up smokes then failed live:

- retained `fork_world_worker` against the correct retained worker id failed with:

> `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

- retained-mode `cancel_world_work` against that same retained worker failed with the same core error:

> `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

What this means:

- this is **not** the earlier bootstrap mismatch bug anymore;
- this is **not** the earlier “wrong participant id” operator mistake once the correct retained worker id is used;
- and this is no longer best explained as a lineage-match bug alone.

The two most recent 2026-06-30 probe passes converged on the same narrower diagnosis:

- design intent says clean retained bootstrap exit should leave a worker parked/resumable and still controllable later via:
  - `orchestration_session_id`
  - exact retained `participant_id`
  - exact world binding
- live code says the first failing gate is now the **AuthoritativeLive** predicate, before the newer lineage-aware successor logic matters;
- clean retained bootstrap exit clears runtime ownership/liveness flags strongly enough that the worker is treated as no longer `authoritative-live`,
  even though design intent expects it to remain a valid retained control target for later successor-authorized retained follow-ups.

Why this matters:

- retained-worker bootstrap parity should now be read as materially improved / passing in live smoke;
- the next active retained-worker bug is now narrower:
  - successor-authorized retained `fork_world_worker`
  - and likely retained-mode `cancel_world_work`
  are still rejected because parked/resumable retained workers are being downgraded too far after clean bootstrap exit;
- this is therefore best read as a **wrong liveness predicate / authoritative-live gate** bug rather than the older world-binding bootstrap bug.

### Repro F (2026-07-01 late): retained fork moved through wrapper recovery, then advanced past publication into retained-target `stale_linkage`

Later 2026-07-01 smokes and patch/review cycles moved the retained `fork_world_worker` failure through multiple real stages, and the newest probe changed which stage should now be treated as current.

Important status clarification:

- in this chronology, **branch-local landed / committed** means committed on the current branch `HEAD` (even if not on `origin/...`);
- **local/worktree truth** means review/smoke/probe truth observed locally but **not** committed on current branch `HEAD`;
- unless a specific commit id is cited inline or a bucket explicitly says **branch-local landed / committed**, the stage shifts below should be read as **local/worktree truth** rather than committed `HEAD` truth.

Those 2026-07-01 observations now sort into three different truth buckets:

1. **branch-local landed / committed truth (`HEAD`, ahead of `origin/...`)**
   - `7732d839` (`Refactor retained member management in MemberRuntimeManager`) and `1374775e` (`Implement stale fork child slot pruning and add smoke test for retained world worker`) are both real landed packets for this stage, and they were not world-service-only;
   - on the world-service side, that landed chain taught `MemberRuntimeManager` to prune stale retained fork-child slot entries before registering a replacement child;
   - and it added branch coverage / smoke support for:
     - replacing a stale fork child that is still present but no longer bootstrapping/resumable,
     - repairing a stale retained slot whose old child entry is already missing from `by_participant_id`,
     - while still rejecting a genuinely second live fork child in the same slot.
   - on the shell side, the same landed chain also included:
     - the retained-bootstrap wrapper-era launch text:
       - `failed to launch world member dispatch stream for retained worker bootstrap ... in orchestration session ...`
     - the first durable-publication `fork_lineage_persist_failed` / `missing_fork_child_registration` path in `crates/shell/src/execution/orchestrator_world_dispatch.rs`,
     - including the more specific `missing_target_participant` / `missing_stop_transport` detail surfaced out of the shell-side durable-publication gate.

2. **later local shell patch-chain truth, separated by provenance**
   - **local/worktree truth only:** shell observability during this chain added pre-wrapper logging in the retained stream-launch helper so the inner `ApiError` is preserved before the already-landed outer:
     - `failed to launch world member dispatch stream for retained worker bootstrap ...`
     wrapper context flattens it;
   - **local/worktree truth only:** timeout-split / durable-publication wait work widened child participant visibility budget while keeping private stop-transport publication on the original bounded budget, and added targeted regression coverage for that split;
   - **local/worktree truth only:** the Boyle/Nash follow-up patch chain is review-clean and improved same-backend routing, authoritative workspace-root handling, and test coverage;
   - the newest smoke on top of the combined landed/local chain now shows that fork create is green, backend-only follow-up ambiguity-closes, exact retained continue is green on both workers, and the remaining live seam is source-worker stop closeout.

3. **latest live smoke chronology**

1. after the earlier shell-side stale-owner / successor-authority work was advanced in committed history and then continued through later local iterations, the old:

> `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

was no longer the first retained-fork blocker;

2. the next valid fork smoke then failed during child bootstrap with the pre-wrapper form:

> `failed to launch spawn_world_worker over world member dispatch`

which then became the landed retained-bootstrap wrapper form in `7732d839`:

> `failed to launch world member dispatch stream for retained worker bootstrap ... in orchestration session ...`

These two labels should be read as sequential forms of the same wrapper-era bootstrap stage, not as unrelated seams.

That wrapper-era stage led to the landed retained-slot / shell wrapper / first durability-gate work in `7732d839`, and then to later local shell observability/orchestrator-id/error-chain patch iterations;

3. after the earlier stream-launch/bootstrap blocker moved out of the way on a clean rerun, a subsequent valid fork smoke got farther and again failed with:

> `fork_lineage_persist_failed: failed to persist explicit fork lineage for child ... after authoritative registration ... (missing_fork_child_registration: ... missing_target_participant, missing_stop_transport ...)`

which re-exposed the shell-side durability/publication seam after the first `Registered` event;

4. one malformed-tool-call pass that surfaced:

> `invalid_tool_arguments`

should be read as a contract/prompt issue on that probe input, **not** as a runtime regression in the retained fork path;

5. after the local timeout-split patch was reviewed clean and the smoke was re-run without the malformed-tool-call mistake, the next clean live seam remained:

> `fork_lineage_persist_failed: failed to persist explicit fork lineage for child ... after authoritative registration ... (missing_fork_child_registration: ... missing_target_participant, missing_stop_transport ...)`

6. the next valid smoke then advanced one full stage farther:
   - retained `spawn_world_worker`: pass
   - retained `fork_world_worker`: pass
   - the fork call succeeded and returned a real child `participant_id`
   - but the next generic same-backend follow-up then failed with:

> `stale_linkage: orchestration session 019f207d-47e5-75b0-aef7-559eb20193e6 backend cli:codex-world no longer has an authoritative retained turn target`

   - `stop_world_worker` against the original retained source worker `ash_019f207f-fe23-74d3-bf02-6c84072be53f` then failed with:

> `stale_linkage: orchestration session 019f207d-47e5-75b0-aef7-559eb20193e6 retained worker ash_019f207f-fe23-74d3-bf02-6c84072be53f is not linked to authoritative orchestrator ...`

   - and `stop_world_worker` against the fork child failed with the same retained-worker / authoritative-orchestrator `stale_linkage` shape.

What this means:

- the old retained `authoritative-live` fork seam was real and has committed code history behind it, but it is no longer the top live blocker;
- the temporary stream-launch/bootstrap wrapper seam was also real, but the wrapper itself is already landed history in `7732d839`; the later local observability patch was useful because it preserved the inner `ApiError` instead of losing it behind that landed wrapper context;
- the post-registration durability/publication failure was also real, but it is now best read as an earlier 2026-07-01 stage rather than the newest top blocker;
- the timeout-split patch improved the shell-side wait logic and its coverage, and the newest smoke now proves fork creation can move past that earlier gate;
- the strongest current probe read has therefore moved later in the lifecycle:
  - direct retained `fork_world_worker` can now succeed and surface a real child `participant_id`,
  - but same-backend authoritative retained-target selection no longer finds an authoritative retained turn target afterward,
  - and exact-target `stop_world_worker` on both source and child now fail on retained-worker / authoritative-orchestrator linkage,
  - so the current seam is best described as post-fork authoritative retained-target selection / `stale_linkage`, not failed fork creation itself.

### Repro G (2026-07-02 earlier): backend-only post-fork routing ambiguity-closed before exact retained continue was fixed

In the newest live smoke on top of the later local shell/test patch chain:

- world-bound `agent start`: pass;
- retained `spawn_world_worker`: pass;
- retained `fork_world_worker`: pass;
- and the fork call returned a real child `participant_id`.

What changed relative to Repro F:

- generic backend-only follow-up after fork no longer fails with:
  - `stale_linkage: orchestration session ... backend cli:codex-world no longer has an authoritative retained turn target`
- it now correctly fails closed with:

> `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`

- and at that earlier stage exact-handle retained `continue_world_worker` still failed in live smoke with:

> `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

Why this mattered at that stage:

- fork create was already green in live smoke;
- backend-only post-fork routing was already behaving more honestly by ambiguity-closing instead of pretending there was one authoritative retained target;
- and that stage narrowed the then-live seam to exact retained continue rather than fork-lineage persistence.

### Repro H (2026-07-02 later): exact retained continue is green on both workers; source stop now fails in stop closeout

In the newest later live smoke on top of committed `484a17b6` plus the retained-fork patch chain already captured above:

- world-bound `agent start`: pass;
- retained `spawn_world_worker`: pass;
- retained `fork_world_worker`: pass;
- the fork call returned a real child `participant_id`;
- exact retained `continue_world_worker` against the original retained source worker: pass;
- exact retained `continue_world_worker` against the fork child: pass.

The next active failure moved later again:

- generic backend-only follow-up remains correctly ambiguity-closed with:

> `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`

- but exact retained `stop_world_worker` against the original retained source worker now fails with the new stop-closeout seam:

> `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`

Later stop-focused investigation changed the highest-confidence read again:

- an earlier working hypothesis was that the stop failure might still be explained by host-posture or narrow public-control gating once the host session normalized to a detached state;
- later live persisted-state inspection weakened that explanation materially:
  - the host orchestration session still surfaced as `parked_resumable`,
  - the target retained source worker still persisted:
    - `control_owner_retained = true`
    - `event_stream_active = true`
    - `completion_observer_retained = true`
    - `ownership_valid = true`
- the actual private stop socket path still existed on disk for that retained control target,
- but direct connect against that stop socket returned `ConnectionRefused`.

That is the current live contradiction:

- persisted target-worker ownership truth still says the retained control owner / event stream / completion observer are present and valid,
- while the actual stop transport behind that truth is already dead enough to refuse a connection.

Why this matters:

- exact retained continue is no longer the top live blocker;
- the retained-worker selector / lineage / exact-continue `authoritative-live` seam has moved from live blocker to fixed-live-smoke stage;
- the earlier host-posture / public-control-gating suspicion is now materially weaker than the transport contradiction above;
- the remaining bug is later in the lifecycle and narrower:
  - exact retained stop on the source retained worker is now failing in private-stop transport delivery plus durable closeout observation,
  - so the active seam should now be read as stop-closeout reachability/closeout semantics, not continue-routability or fork-lineage persistence.

### Landed since the previous memo snapshot

The seam-moving patches that now matter most to the live state are:

1. **`382a63a8` (`Refactor retained worker authority checks and enhance follow-up resolution for non-authoritative live workers`)**
   - moved retained follow-up resolution away from the older strict authoritative-live shape;
   - advanced exact retained follow-up eligibility for non-authoritative-live workers;
   - materially shifted the live seam away from the first exposed successor `authoritative-live` gate.

2. **`7732d839` (`Refactor retained member management in MemberRuntimeManager`)**
   - introduced retained-slot cleanup/restructure needed for retained fork-child handling;
   - carried the wrapper-era retained bootstrap / dispatch diagnostics that were needed to get past the earlier bootstrap and publication stages.

3. **`1374775e` (`Implement stale fork child slot pruning and add smoke test for retained world worker`)**
   - made stale fork-child slot replacement explicit;
   - added retained-world-worker smoke coverage that helped validate fork advancement past the earlier stale-slot seam.

4. **`484a17b6` (`Refactor tool invocation contract and enhance retained worker handling`)**
   - is the most important committed `HEAD` packet for the latest shift;
   - removed owner PID liveness as a correctness condition for exact retained follow-up routing;
   - tightened exact retained continue/stop target resolution and successor-authority handling;
   - and aligns with the newest live smoke where exact retained `continue_world_worker` is now green for both source and child.

Which previously-plausible assumptions later smoke has now disproven or materially weakened:

- **disproven:** the retained path is still primarily blocked by the old bootstrap/world-binding mismatch;
- **disproven:** fork child creation/publication never really happened;
- **disproven:** backend-only post-fork routing still quietly chooses one retained world target instead of failing closed on ambiguity;
- **disproven:** exact retained `continue_world_worker` is still the current live blocker after fork;
- **materially weakened / likely wrong as primary diagnosis:** source-worker stop is failing mainly because the detached host session posture or narrow public-control gate no longer treats the session as stoppable.
  - later live state instead shows:
    - host session posture still `parked_resumable`,
    - target retained worker ownership truth bits still all `true`,
    - stop socket path still present,
    - but the stop transport itself refusing connections.

Current provenance boundary:

- the retained-fork / exact-continue seam-moving work above is **branch-local landed / committed truth** on current `HEAD`;
- the newest stop seam is **current live local/worktree truth** observed on top of that code:
  - `owner_unreachable ... failed to connect to private stop transport ... durable stop closeout was not observed`
- the newer stop-path contradiction is also **current live local/worktree truth**, not committed history:
  - host session still `parked_resumable`,
  - target retained source worker still persists `control_owner_retained/event_stream_active/completion_observer_retained/ownership_valid = true`,
  - stop socket path still exists,
  - direct connect returns `ConnectionRefused`;
- as of this memo update, no later committed `HEAD` patch is yet recorded here as resolving that stop-closeout seam.

## 2026-07-03 stop-path probe, design-validation, and Slice 64 follow-through

This is the main delta since the last meaningful memo snapshot.

### Current live-state floor

Distinguish these buckets explicitly:

1. **branch-local landed / committed truth**
   - current `HEAD` contains the seam-moving retained follow-up work already cited above, especially:
     - `382a63a8`
     - `7732d839`
     - `1374775e`
     - `484a17b6`
   - committed code truth here is:
     - the retained follow-up selector/authority direction has materially changed,
     - owner PID liveness is no longer treated as a correctness condition for exact retained follow-up routing,
     - and there is still **no committed `HEAD` proof** here that exact retained `stop_world_worker` on the original source worker is solved.

2. **current live smoke truth**
   - the newest valid smoke floor is:
     - retained `fork_world_worker`: green
     - exact retained `continue_world_worker` on the original source worker: green
     - exact retained `continue_world_worker` on the fork child: green
   - the active red seam is still:

> `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`

   - the private stop connect failure in the later probe was a real refused transport:
     - direct connect against the existing retained private stop socket returned `ConnectionRefused`.

3. **current local/worktree implementation truth**
   - there is newer stop-path work in:
     - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
     - `crates/shell/src/execution/agents_cmd.rs`
   - the stop-path worktree direction now includes detached-closeout / refused-transport handling experiments and targeted tests such as:
     - `dispatch_contract_stop_world_worker_persists_parked_resumable_closeout_when_private_stop_socket_is_stale`
     - `detached_stop_world_worker_closeout_availability_rechecks_fresh_session_truth_after_refused_transport`
   - but this remains **local/worktree truth only**, and the latest live source-stop smoke is still red, so do **not** describe stop as fixed.

### Important stop-path probe findings after the latest smoke

The later stop-focused probe changed the highest-confidence diagnosis in three important ways:

1. **stale refused socket is real**
   - the private stop socket path can still exist for the retained source worker,
   - yet the actual transport behind that path can already be dead enough to refuse connect with `ConnectionRefused`.

2. **persisted truth often matches the contradiction shape after the failure**
   - after the failed source stop, live persisted state could still show:
     - host session posture `parked_resumable`
     - `control_owner_retained = true`
     - `event_stream_active = true`
     - `completion_observer_retained = true`
     - `ownership_valid = true`
   - that means the contradiction is not just “public stop gate said no”; persisted orchestration/runtime truth can still present a live retained-control picture while the private stop transport is already refusing.

3. **the current implementation direction had one-shot fallback timing assumptions**
   - the repo/worktree stop experiments now clearly revolve around one bounded transport publication / connect / recheck window in `crates/shell/src/execution/orchestrator_world_dispatch.rs`,
   - which is useful as a probe and test-shaping step,
   - but it is also why the seam should still be read as unresolved stop-closeout authority/proof work rather than as a finished transport retry fix.

### Important design findings highlighted by the design-validation pass

The new design-validation artifact is:

- `DESIGN_VALIDATION_MEMO_STOP_FALLBACK_AND_HOST_OWNERSHIP_2026-07-03.md`

That memo's important conclusions for this debug family are:

1. **stop direction now looks likely misaligned toward transport/liveness**
   - the design-validation memo's read is that stop is intended as durable control-plane closeout, not “whatever the private stop transport can currently deliver”.

2. **durable authority is orchestration-session rooted, not attached-host rooted**
   - the currently attached host participant is a sanctioned execution client, not the durable authority root.

3. **private stop transport is delivery-only, not lifecycle truth**
   - missing/refused transport is fail-closed delivery truth,
   - not by itself proof that the retained worker is durably stopped.

4. **fork/continue direction appears mostly aligned**
   - the latest design read does not point to equivalent architectural drift in retained `fork_world_worker` / exact retained `continue_world_worker`;
   - that matches the latest live smoke where those surfaces are green.

### New spec-driven-development artifacts and current status

The new Slice 64 chain now exists:

- `llm-last-mile/SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`
- `llm-last-mile/PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`
- `llm-last-mile/TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`

Provenance/status note:

- **repo-file truth:** the files currently still carry `draft for review` headers;
- **latest local planning status from the investigation chain:** the `SPEC-64` -> `PLAN-64` -> `TASKS-64` chain is currently being treated locally as review-clean / execution-ready planning input for this stop seam, but that is stronger than the on-disk headers themselves.

What that means operationally:

1. the current **local execution posture** is to proceed from `TASKS-64` one task at a time, even though `TASKS-64` itself is written as a packet-based execution model;
2. Packet 1 was deliberately split into smaller implementation-safe units because this seam is easy to confuse;
3. the planning direction captured by the local Slice 64 chain is to correct stop authority / fail-closed closeout semantics without reopening public stop, general attach design, or unrelated retained-worker verbs.

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

For the still-broken retained-worker path, the better leading hypotheses at that 2026-06-30 stage were:

- retained-worker parked/resumable state is being validated too strictly by an `authoritative-live` predicate on successor follow-up,
- retained `fork_world_worker` and retained-mode `cancel_world_work` still hit that liveness gate even after bootstrap/world binding succeeds,
- and any remaining worker lifecycle/control-plane truth can only be analyzed cleanly after that retained successor gate is fixed.

Design-backed interpretation:

> the observed `stale_linkage` behavior likely represents a contract bug relative to session-rooted retained-worker ownership; the open implementation question is the exact normalization / authority-transfer mechanism across successor host participants, not whether successor follow-up should be possible at all.

The older retained-worker probes sharpened that interpretation further:

- retained-worker authority is session-rooted under `orchestration_session_id`, not pinned to one attached host participant;
- `spawn_world_worker` is supposed to use the authoritative session `world_id` / `world_generation` from the bound session, not caller-remembered values;
- retained successor `fork_world_worker` and retained-mode `cancel_world_work` are the follow-up/control operations expected to remain successor-authorized;
- **historical code/test-boundary read:** retained `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker` had been described earlier as strict direct-link surfaces;
- **current branch-local landed / committed truth (`484a17b6`):** successor-authority lineage is now accepted for those retained follow-up selectors too, exact retained continue no longer depends on owner PID liveness, and the remaining live failure sits later at source-worker stop closeout rather than lineage selection;
- `participant_id` is the authoritative retained-worker control handle;
- `resumed_from_participant_id` is lineage/audit metadata, not the control selector;
- so absent a genuine world replacement / generation rollover, `stale_linkage` currently reads as design-wrong rather than expected contract enforcement.

The newest 2026-06-30 probes added one more refinement for that stage:

- retained-worker bootstrap parity can now succeed in live smoke;
- the real remaining fork/cancel failure is the `authoritative-live` requirement firing too early on parked/resumable retained workers;
- so at that stage the top live bug had moved again:
  - from retained-worker bootstrap parity
  - to retained-worker successor-authorized **authoritative-live** gating for parked/resumable retained workers;
- worker execution failure should still stay behind that fix.

That 2026-06-30 read is now historical only. Later 2026-07-01 local patch-chain smokes first moved the retained fork seam into an earlier stream-launch/bootstrap wrapper, then a later clean rerun moved past that wrapper and re-exposed the post-registration durability/publication seam, then a later valid smoke moved to post-fork authoritative retained-target selection / `stale_linkage`, and the newest 2026-07-02 smoke narrows that further again:

- backend-only post-fork follow-up now ambiguity-closes with `ambiguous_backend_slot`,
- while exact retained `continue_world_worker` still fails on the `authoritative-live` seam.

### 4) There is also a broader shell/runtime contract mismatch and a REPL/CLI parity requirement

Separate from the likely retained-slot bug, the shell-side model still looks inconsistent across layers.

Most important mismatch:

- parked/awaiting-attention session truth is largely derived from **host detach continuity**,
- that state does **not** prove new world-member launch is realizable,
- exact retained-member follow-up exists on Linux via `/v1/member_turn/stream`, but the public lifecycle remains host-rooted and detached world follow-up fails closed until `substrate agent reattach` restores an active host owner,
- internal exact-target continue still appears to require stronger “authoritative-live” truth,
- some member startup/bootstrap paths still require a **live orchestrator parent**.

Product-model clarification that must stay explicit here:

- PID liveness is **not** part of the intended retained-worker validity model;
- backends are headless CLI-process based, currently primarily Codex via `codex exec`;
- a backend PID is expected to exist only while actively executing a turn and is expected to disappear between turns;
- therefore a dead owner PID between turns must **not** be treated as evidence that a retained worker is invalid, non-resumable, or structurally stale.

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
- and the later continuity-selector precedence fix closed the resumed targeted host-turn semantic continuity bug on both REPL and public CLI.

So by the end of the 2026-06-30 stage, the live mismatch was narrower than “reattach is broken” or “all world dispatch is broken.” At that point the main remaining runtime failures were the retained parked/resumable `authoritative-live` seam for successor `fork_world_worker`, then adjacent retained `cancel_world_work`, and only later worker execution. That ordering is now superseded by the later 2026-07-01 / 2026-07-02 patch-chain smokes, which moved first through the wrapper seam, then through the post-registration durability/publication seam, then through backend-only post-fork retained-target ambiguity, then through exact retained continue, and now point first at source-worker `stop_world_worker` failing in private-stop delivery / durable stop closeout after successful fork creation.

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
   - the same UAA session id persists across the resumed targeted host turn
   - ordinary commands still work or fail for the right reason:
     - `ls` succeeds
     - `pwd` succeeds
     - `cd ../` is blocked by the caged-root guard
   - and the resumed targeted host answer now correctly comes from the Substrate session-local first turn rather than broader outer thread / `AGENTS.md` context

2. **CLI-equivalent parity regression**
   - `substrate agent start --backend cli:codex-host --prompt "just reply OK!"`
   - normal shell command in between, e.g. `substrate -c "ls"` or `pwd`
   - `substrate agent turn --session <same_session> --backend cli:codex-host --prompt "tell me what my last message said"`
   - the same durable orchestration session is reused
   - `resumed_from_participant_id` is present
   - the same UAA session id persists across `start` and `turn`
   - and the answer now correctly comes back as `Just reply OK`

Both should also validate:

- exact `orchestration_session_id` continuity,
- expected participant successor behavior,
- persisted UAA/Codex continuity markers when present,
- shell execution in between does not invalidate the durable session,
- and later follow-up does not silently create a new unrelated session.

## Highest-value next discriminators

These are the best next debugging checks to separate confirmed truth from still-likely theory:

1. **Investigate exact retained stop closeout on the original source worker next**
   - Goal: preserve the current confirmed smoke truth:
     - retained `spawn_world_worker`: pass
     - retained `fork_world_worker`: pass and returns a real child `participant_id`
     - backend-only post-fork follow-up now correctly ambiguity-closes
     - exact retained `continue_world_worker` on the source worker: pass
     - exact retained `continue_world_worker` on the fork child: pass
     - current blocker is exact retained `stop_world_worker` on the source worker failing with `owner_unreachable ... failed to connect to private stop transport ... durable stop closeout was not observed`
   - Focus the next discriminator on why exact retained stop on the original source worker now loses private-stop reachability and durable closeout after the newer successor-authority / exact-continue routing work already proved the worker is otherwise correctly selectable and continue-routable.
   - Preserve the newest contradiction explicitly while debugging:
     - host session posture can still be `parked_resumable`,
     - target retained source worker can still persist `control_owner_retained/event_stream_active/completion_observer_retained/ownership_valid = true`,
     - the stop socket path can still exist,
     - yet the transport connect itself returns `ConnectionRefused`.
   - Most likely seams:
     - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
     - `crates/shell/src/execution/agent_runtime/control.rs`
     - retained source-runtime / private stop transport lifecycle in `crates/shell/src/repl/async_repl.rs`

2. **Use the already-clean timeout-split / durable-publication work only as stage-separation evidence**
   - Goal: preserve the earlier finding that the post-registration publication seam was real, while also keeping clear that the newest valid smoke now moves past it.
   - Current read:
     - the earlier `missing_fork_child_registration ... missing_target_participant, missing_stop_transport` seam was a real intermediate stage,
     - but the newest valid smoke now proves retained `fork_world_worker` can return a child, exact retained continue can run on both workers, and the active failure sits later at source-worker stop closeout.

3. **Then re-check retained-mode `cancel_world_work`**
   - Goal: establish whether `cancel_world_work` still shares the newer source-stop closeout reachability seam on current code, or whether later retained-fork changes shifted cancel differently.

4. **Separate worker execution failure from control-plane failure**
   - Goal: keep `continue_world_worker` worker-turn `codex exited non-zero` analysis separate from retained fork/cancel launch and control-plane failures.

5. **Keep ordinary-command survivability as a regression baseline**
   - Goal: preserve live `ls` / `pwd` success and expected caged-root `cd ../` denial after park while working retained-worker seams.

6. **Keep the REPL and CLI parity regressions green**
   - Goal: confirm both wrappers continue to preserve the same durable session truth, the same resumed continuity markers, and the correct session-local conversation continuity.

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

2. **The host-turn continuity-selector precedence bug is now fixed**
   - Root cause:
     - `submit_host_prompt_turn` could still prefer the frozen `PromptSubmitRuntime.uaa_session_handle_id`,
     - while later turns had already refreshed the persisted host-attach continuity truth and/or the live manifest truth,
     - so resumed prompt-bearing turns could carry the wrong continuity handle even though the durable session lineage looked correct.
   - Landed fix:
     - prefer persisted host-attach-contract continuity first,
     - then live manifest `internal.uaa_session_id`,
     - then frozen runtime `uaa_session_handle_id` last.
   - The targeted unit coverage in `crates/shell/src/execution/agent_runtime/control.rs` now exercises both precedence cases directly.

3. **The strengthened REPL continuity regression is now meaningful**
   - The hardened REPL regression now verifies:
     - same `orchestration_session_id`,
     - same continuity selector,
     - no intervening host relaunch from ordinary `ls`,
     - and valid participant reuse or successor lineage on the later targeted host turn.
   - Review validated that test hardening.

4. **The shim logger redaction fix is separate and healthy**
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
- and the later targeted host follow-up continuity seam is now fixed by the continuity-selector precedence repair rather than remaining a live bug.

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

## Later committed branch history and local patch-chain observations

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
  - while the next active runtime focus has moved on to retained-worker seams rather than resumed targeted host-turn continuity.

### 4. Earlier committed retained-fork authority / linkage history

- The earlier committed branch history for the old retained-fork authority stage includes:
  - `382a63a8` — `Refactor retained worker authority checks and enhance follow-up resolution for non-authoritative live workers`
  - `02940fb6` — `Refactor retained worker linkage validation and bootstrap context`
- Those commits are the supportable committed history for the earlier retained-fork `authoritative-live` / linkage stage.
- They should not be conflated with the later 2026-07-01 local patch chain around slot registration, orchestrator-id preservation, post-registration durability gating, or the later temporary stream-launch wrapper seam.

### 5. Landed retained-slot cleanup in current `HEAD`

- `7732d839` — `Refactor retained member management in MemberRuntimeManager`
- `1374775e` — `Implement stale fork child slot pruning and add smoke test for retained world worker`
- This branch-local landed chain includes both world-service and shell-side retained-fork work.
- On the world-service side, it prunes a stale retained fork-child slot before registering a replacement child.
- Added coverage now proves:
  - a stale child still present in `by_participant_id` but no longer bootstrapping/resumable is evicted and replaced,
  - a stale retained slot whose prior child entry is already missing from `by_participant_id` is repaired and replaced,
  - a genuinely second live fork child in the same slot is still rejected.
- On the shell side, the same landed chain already includes:
  - the retained-bootstrap wrapper wording in `execute_spawn_world_worker_stream(...)`,
  - the first shell durable-publication gate that can fail with `fork_lineage_persist_failed ... missing_fork_child_registration ...`,
  - and the specific `missing_target_participant` / `missing_stop_transport` detail path in `orchestrator_world_dispatch.rs`.

### 6. Later local retained-fork patch-chain observations (dirty worktree, not committed `HEAD` unless separately cited)

- Later 2026-07-01 retained-fork work in the dirty worktree moved through several narrower seams:
  - shell-side observability to preserve the inner stream-launch `ApiError` before the already-landed wrapper context,
  - shell-side retained source orchestrator identity preservation for fork child bootstrap,
  - shell-side error-chain preservation cleanup for fork bootstrap errors,
  - shell-side bounded wait split between child visibility and stop-transport publication before treating the child receipt as usable,
  - and Boyle/Nash follow-up work improving same-backend routing, authoritative workspace-root handling, and retained-fork test coverage.
- Later 2026-07-02 retained-worker routing work is now partially promoted into committed `HEAD`:
  - `484a17b6` — `Refactor tool invocation contract and enhance retained worker handling`
  - this landed packet materially moved the seam forward by:
    - differentiating follow-up target requirements by action instead of treating retained follow-up as one coarse bucket,
    - accepting successor-authority lineage for retained follow-up selector resolution across `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker`,
    - changing exact retained continue so owner PID liveness is no longer a correctness condition,
    - teaching public-target resolution to ambiguity-close honestly when multiple same-backend retained turn targets remain authoritative,
    - and tightening REPL retained-runtime reuse around targeted retained turns.
- The previously memoized “Volta” / “Zeno” observations should therefore now be split as:
  - **committed `HEAD` truth:** the core exact-continue routing / successor-authority / owner-PID-liveness changes are landed in `484a17b6`,
  - **local/worktree truth only:** any later observations that are not traceable to a cited commit id still remain local smoke chronology rather than committed branch history.
- Those observations are useful debug chronology.
- But absent specific commit refs, the remaining later observations should still be read as **local patch/review/smoke iterations**, not as committed branch history.
- The newest smoke on top of that chain now shows:
  - fork create is green,
  - backend-only post-fork routing is correctly ambiguity-closed,
  - exact retained continue is green on both the source worker and the fork child,
  - and the remaining live seam is source-worker `stop_world_worker` private-stop delivery / durable stop closeout.

### 7. Positive success regression was dead code, then fixed

- Review later discovered the intended positive regression for:
  - parked host
  - ordinary unprefixed world-backed command
  - later targeted host follow-up continuity
  was present but not actually registered as a test.
- A tiny follow-up patch added the missing test attributes.
- Review validated that final test-enablement patch.
- So the positive regression is now actually executing; that was enablement, not a behavior fix by itself.

### 8. Clippy-only patch was separate and healthy

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
- and after the continuity-selector precedence patch the semantic answer is now also correct: it comes from the session-local first turn rather than broader outer conversation context.

Interpretation:

- ordinary unprefixed-command survivability after parked host has materially improved in live smoke and should no longer be described as the main live bug,
- transport/session reuse is not the primary failure on this seam anymore,
- and the resumed targeted host-turn continuity seam is now healthy enough to stop treating it as the top live bug family.

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
   - REPL and public CLI now both validate the same durable continuity seam end to end,
   - the concrete fix was continuity-selector precedence inside `submit_host_prompt_turn`,
   - so this seam is no longer the top active live blocker.

4. **Seam D: retained-worker bootstrap parity**
   - this seam now appears materially improved:
     - fresh public world start succeeded,
     - later retained `spawn_world_worker` succeeded,
     - and the old retained-worker first-dispatch `world_id mismatch` did not reproduce in the newest smoke,
   - so it should no longer be treated as the top live blocker,
   - though the underlying workspace-root / cwd repair remains part of the landed explanation for why this improved.

5. **Seam E: retained-worker successor authority / authoritative-live normalization**
   - successor-authorized retained `fork_world_worker` and retained-mode `cancel_world_work` still fail live with:
     - `stale_linkage ... retained worker ... is no longer authoritative-live`
   - design intent says a retained worker that exits clean bootstrap should become parked/resumable, not become an invalid control target,
   - this seam was the next active bug after bootstrap parity improved,
   - but it should now be read as a previously exposed / committed-history retained-fork stage rather than the current top live blocker, because later smokes moved through the wrapper seam, then through the post-registration durability/publication seam, then through backend-only post-fork ambiguity closure, then through exact retained continue, and now to source-worker stop closeout.

6. **Seam F: worker execution after continue**
   - `continue_world_worker` reaching the retained worker and then failing with `codex exited non-zero` remains real,
   - but this should stay behind bootstrap parity and successor-authority fixes in the live queue so execution failure is not conflated with launch/control-plane bugs.

7. **Seam G: retained fork post-registration durability/publication**
   - this seam was real in the earlier 2026-07-01 chain:
     - `fork_lineage_persist_failed ... missing_fork_child_registration ... missing_target_participant, missing_stop_transport ...`
   - the recent timeout-split patch improved the local wait logic and tested it,
   - and the newest smoke now shows retained `fork_world_worker` can move past that earlier gate rather than stopping there.

8. **Seam H: backend-only post-fork routing ambiguity closure**
   - the newest valid smoke now shows:
     - retained `spawn_world_worker`: pass
     - retained `fork_world_worker`: pass
     - real child `participant_id`: returned
   - and the next same-backend follow-up now fails honestly with:
     - `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`
   - so backend-only post-fork routing should now be read as correctly ambiguity-closed rather than the current bug.

9. **Seam I: exact retained continue after fork is now green**
   - the newest later smoke now passes exact-handle retained `continue_world_worker` on:
     - the original retained source worker,
     - and the fork child.
   - committed `484a17b6` materially explains that movement:
     - exact retained continue no longer depends on owner PID liveness,
     - successor-authority lineage is accepted during retained follow-up selector resolution,
     - and targeted retained follow-up now reaches the real later seam.

10. **Seam J: exact retained stop on the source worker now fails in private-stop delivery / durable closeout**
   - the newest later smoke now fails exact-handle retained `stop_world_worker` on the original retained source worker with:
     - `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`
   - this is now the top live blocker after fork create, backend-only ambiguity closure, and exact retained continue all went green.

### Public `agent start` / `agent turn` continuity strict smoke now validates semantically

Historical chronology:

- before the latest patch, later prompt-bearing turns could refresh persisted continuity truth while `submit_host_prompt_turn` still preferred the frozen `PromptSubmitRuntime.uaa_session_handle_id`;
- that let transport/session lineage look correct while the resumed turn still hydrated the wrong continuity handle.

Current validated smoke:

1. `substrate agent start --backend cli:codex-host --prompt 'just reply OK!'`
2. ordinary shell command outside the agent surface succeeds
3. `substrate agent turn --session <same> --backend cli:codex-host --prompt 'tell me what my last message said'`
4. the same orchestration session is reused
5. `resumed_from_participant_id` is present
6. the same UAA session id persists across `start` and `turn`
7. the semantic answer comes back correctly as `Just reply OK`

Interpretation:

- the public durable-session continuity surface is now transport-correct and semantically correct on this strict follow-up seam;
- the fix is specifically the continuity-selector precedence repair:
  - persisted host attach contract continuity first,
  - then live manifest continuity,
  - then frozen runtime continuity last;
- this seam should now be read as validated current truth, not an active live repro.

## What is now actually committed / historically trustworthy vs what is only local patch-chain truth

In this section:

- **branch-local landed / committed** means committed on the current branch `HEAD`;
- **upstream/origin history** means present on `origin/...`;
- **local/worktree truth** means review/smoke/probe observations that are not committed on current branch `HEAD`.

Later 2026-07-01 retained-fork stage changes without commit ids should be read as **local/worktree truth** only.

At this point the following narrow claims appear trustworthy:

1. **One-shot `run_world_task` stale-binding mismatch is no longer the primary live blocker**
   - that earlier seam appears repaired enough that later runtime smoke moved on to different failures.

2. **Ordinary-command survivability after parked targeted host turn is materially improved in live smoke**
   - after `::cli:codex-host just reply OK!`, ordinary unprefixed `ls` and `pwd` succeed and `cd ../` is denied by the caged-root guard as expected.

3. **Resumed targeted host turns now appear to reuse transport/session identity correctly**
   - same `orchestration_session_id` is reused,
   - `resumed_from_participant_id` is present,
   - the same UAA session id persists across the resumed turn,
   - and semantic conversation continuity is now correct on both the REPL and strict public CLI follow-up smokes.

4. **The parked-session survivability slice that removed the old `no live orchestrator parent` first failure is landed**
   - that older failure is no longer the first blocker on the ordinary unprefixed path.

5. **The current test split is now reflected in adjacent coverage**
   - targeted `::cli:codex-world ...` fail-closed coverage exists
   - explicit `:pty` positive persistent-session routing coverage exists
   - the positive parked-host -> ordinary unprefixed implicit-PTY command -> targeted host resume regression now actually executes

6. **But the live failure has moved above the current regression focus**
   - tests currently validate important routing boundaries,
   - while the retained-worker queue has since moved through local 2026-07-01 patch-chain stages:
     - retained successor `authoritative-live` gating,
     - then the landed world-service retained-slot cleanup,
     - then temporary stream-launch/bootstrap wrapper recovery,
     - then re-exposed post-registration child durability/publication failure,
      - then backend-only post-fork ambiguity closure,
      - then exact retained continue routing repair,
      - and with the newest probe now pointing at source-worker stop closeout rather than failed fork creation itself.

7. **ID taxonomy / continuity documentation is landed enough for this seam**
   - the canonical internal doc exists:
     - `docs/internals/agent_runtime/session_identity_and_continuity.md`
   - key backlink docs were updated:
     - `docs/contracts/agent-event-envelope.md`
     - `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
   - only a low-severity unrelated `docs/TRACE.md` stale absolute-link cleanup remains.

8. **Invalidated vs retained patch slices are now clearer**
   - invalidated:
     - the generic member-bootstrap retry patch
     - the later narrower retry patch that still widened the seam incorrectly
   - retained:
     - parked-session survivability work that got past the old live-parent failure
     - the shared-metadata-only narrowing for ordinary unprefixed binding repair
     - the positive regression enablement patch that made the intended success test actually run

9. **The retained-worker adapter seam boundary is now explicitly validated in tests**
   - retained `fork_world_worker` and retained-mode `cancel_world_work` keep successor-lineage authority,
   - **branch-local landed / committed truth (`484a17b6`):** retained `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker` also now accept successor-authority lineage during retained follow-up selector resolution,
   - so the current live blocker is no longer generic retained authority or fork-lineage persistence,
   - targeted adapter follow-up tests now pass on that newer selector/lineage behavior,
   - and the remaining live seam should now be read as later stop-closeout reachability / durable closeout rather than lineage boundary resolution itself.

10. **The old retained `authoritative-live` fork seam has supportable committed history**
   - the supportable committed branch history here is:
     - `382a63a8` — `Refactor retained worker authority checks and enhance follow-up resolution for non-authoritative live workers`
     - `02940fb6` — `Refactor retained worker linkage validation and bootstrap context`
   - those are the safe inline commit refs for the earlier retained-fork authority / linkage stage.

11. **The later shell observability / timeout-split / Boyle-Nash follow-up chain should still be read as local/worktree truth unless separately cited, but the core exact-continue routing work is now committed `HEAD`**
   - those later 2026-07-01 stages materially changed the observed live failure point,
   - the shell observability and timeout-split / durable-publication wait improvements were useful and review-clean in the local chain,
   - the Boyle/Nash follow-up work also improved same-backend routing, authoritative workspace-root handling, and coverage,
   - `484a17b6` is now the cited committed ref for the core successor-authority / exact-continue / owner-PID-liveness changes that this memo previously described as later local-only truth,
   - but absent further explicit commit refs the remaining observations should not be described here as committed `HEAD` history.

12. **The current top blocker under the newest valid smoke is exact retained `stop_world_worker` on the original source worker after successful fork and continue**
   - the newest valid 2026-07-02 smoke now shows:
     - retained `spawn_world_worker`: pass
     - retained `fork_world_worker`: pass
     - fork returned a real child `participant_id`
     - exact retained `continue_world_worker` on the source worker: pass
     - exact retained `continue_world_worker` on the fork child: pass
   - backend-only follow-up now correctly fails closed with:
     - `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`
   - but exact-handle retained `stop_world_worker` on the original retained source worker now fails with:
     - `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`
   - session lineage, retained-worker lineage, world binding, and exact retained continue are already structurally correct on that path,
   - so the current top blocker is no longer the temporary stream-launch/bootstrap wrapper seam, no longer the post-registration child-publication seam, no longer backend-only post-fork target selection itself, and no longer best described as exact retained continue routability or fork-lineage persistence.

## Updated active blockers

The open bug buckets should now be read in this order:

1. **successful retained fork create now leaves exact retained stop on the source worker as the top live seam**
   - current valid smoke still shows:
     - world-bound `agent start`: pass
     - retained `spawn_world_worker`: pass
     - retained `fork_world_worker`: pass
     - fork returned a real child `participant_id`
     - exact retained `continue_world_worker` on the source worker: pass
     - exact retained `continue_world_worker` on the fork child: pass
   - backend-only post-fork follow-up now correctly fails closed with:
     - `ambiguous_backend_slot: orchestration session ... has multiple authoritative retained turn targets for backend cli:codex-world (...)`
   - but exact-handle retained `stop_world_worker` on the original source worker now fails with:
     - `owner_unreachable: failed to deliver stop_world_worker to retained worker ... and durable stop closeout was not observed (failed to connect to private stop transport ...; owner_unreachable: timed out waiting for retained worker ... to reach durable stopped closeout)`
   - session lineage, retained-worker lineage, world binding, and exact retained continue remain structurally correct here,
   - so this source-worker stop-closeout seam is now the top live blocker.
2. **the older retained `authoritative-live` seam should now be read as code-addressed / historically important, not the current top blocker**
   - it was a real live seam,
   - it drove multiple shell-side stale-owner / authority / continue-fork-command narrowing patches,
   - but the newest valid fork smoke has moved past that stage.
3. **the temporary retained fork stream-launch/bootstrap wrapper seam should now be read as intermediate, not current top-of-queue**
   - it was real and useful because it exposed a hidden inner `ApiError`,
   - but the newest valid rerun moved past it.
4. **the post-registration durability/publication seam should now be read as earlier 2026-07-01 stage history, not the newest top blocker**
   - `fork_lineage_persist_failed ... missing_fork_child_registration ... missing_target_participant, missing_stop_transport ...` was real,
   - the timeout-split / durable-publication wait work was useful,
   - but the newest smoke now gets past fork creation and exposes a later lifecycle seam.
5. **adjacent retained-mode `cancel_world_work` still needs renewed live validation after the later retained-fork patch chain**
   - older live smoke showed the same `authoritative-live` class,
   - and cancel now needs to be checked specifically against the newer stop-closeout reachability seam rather than the already-closed backend-only routing or exact-continue issues.
6. **worker turn execution failure (`codex exited non-zero`) remains a separate follow-up issue**
   - keep this deferred until retained fork/cancel control-plane posture is stable again.
7. **the current strongest diagnosis has moved from publication timing to source-worker stop closeout after fork**
   - the local timeout-split patch was reviewed clean and added useful coverage,
   - committed `484a17b6` plus the newer selector work now make backend-only routing fail closed as ambiguity and exact retained continue pass,
   - but the newest valid smoke still shows source-worker stop failing in private-stop delivery / durable closeout,
   - session lineage, retained-worker lineage, world binding, and exact retained continue remain structurally correct on that failing path,
   - later stop-focused investigation also weakens the simpler host-posture/public-control-gating explanation:
     - persisted host posture can still be `parked_resumable`,
     - persisted target-worker ownership truth can still remain all-true,
     - and the stop socket path can still exist while connect returns `ConnectionRefused`,
   - so the active local direction should center on private-stop reachability / durable stop closeout after fork rather than fork-lineage persistence.
8. **implicit-PTY ordinary-command caller-boundary clarification/coverage is still useful, but it is no longer the top live blocker**
9. **test harness still does not fully model every live caller boundary**
10. **host-visible file/write semantics remain open but are not first in priority**
11. **low-severity unrelated `docs/TRACE.md` stale absolute-link cleanup remains**
12. **enum/type cleanup remains post-stabilization hardening, not the next live blocker**
   - keep the pre-UAA enum/type cleanup as structural hardening after the retained-worker queue above is stabilized.

Do not describe ordinary-command survivability after parked host as the primary open bug in this memo anymore:

- the newest live smoke shows meaningful success on `ls` / `pwd` and the expected caged-root denial on `cd ../`,
- and the resumed targeted host-turn continuity seam is now validated enough that retained-worker behavior should take over as the active runtime focus.

## Next planned landing order

1. **Make retained exact stop / durable closeout the active patch direction**
   - next live target is no longer exact retained continue;
   - current local worktree direction should be framed around why successful fork plus successful exact continue still leaves exact retained `stop_world_worker` on the source worker failing with:
     - `owner_unreachable ... failed to connect to private stop transport ... durable stop closeout was not observed`
   - current local execution posture is to work from `llm-last-mile/TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md` one task at a time, layered on top of its packet-based structure;
   - Packet 1 was deliberately split into smaller implementation-safe units because this seam is easy to confuse;
   - treat this as a stop-closeout / private-stop reachability seam, not a revived fork or continue seam.

2. **Then re-check stop against the fork child**
   - after the source-worker stop seam is understood/fixed, re-smoke exact retained `stop_world_worker` against the fork child;
   - verify whether source-stop and child-stop share the same transport/closeout seam or diverge.

3. **Then re-check retained-mode `cancel_world_work`**
   - older live smoke tied it to the same retained successor queue,
   - but it now needs fresh validation after the later fork patch chain and after the stop-closeout seam is addressed.

4. **Only then separate worker execution failure**
   - revisit `continue_world_worker` `codex exited non-zero` only after retained fork/cancel control-plane seams are stable again, so worker execution is analyzed from a correct baseline.

5. **Keep ordinary-command survivability as the live regression baseline**
   - do not regress parked-host -> ordinary unprefixed `ls` / `pwd` success, expected caged-root `cd ../` denial, or the adjacent implicit-PTY regression while fixing retained-worker/runtime seams.

6. **Keep the docs/contract work stable, with only bounded cleanup left**
   - avoid reopening the landed ID taxonomy / continuity docs unless retained-worker work proves a real contract gap;
   - the remaining `docs/TRACE.md` stale absolute-link cleanup is low severity and unrelated to the active runtime bugs.

7. **Do the structural enum/type cleanup before the UAA boundary as post-stabilization hardening**
   - replace ambiguous `Option<prompt>` / launch-policy semantics with an explicit pre-UAA representation so ordinary commands and prompt-bearing turns cannot be conflated accidentally.
   - this is still worthwhile for readability and safety, but it is not the next live patch target.

## Short operational summary

If we need the shortest honest current diagnosis:

- the missing host file is still most likely a **workspace sync / host visibility** issue, not proof of non-execution;
- the host-visible side-effect contract is still **open**;
- the earlier one-shot REPL first-dispatch `run_world_task` stale-binding mismatch is no longer the first live blocker;
- retained-worker bootstrap parity now appears materially improved in live smoke:
  - public world `agent start` succeeds
  - retained `spawn_world_worker` succeeds
  - retained `fork_world_worker` succeeds and returns a real child `participant_id`
- the older retained-worker bootstrap/world-binding mismatch is no longer the active blocker;
- ordinary-command survivability after parked targeted host turn is materially better in live smoke:
  - `ls` succeeds
  - `pwd` succeeds
  - `cd ../` is denied by the caged-root guard as expected
- explicit `:pty` should not be described as fail-closed on this memo's current truth;
- REPL and public CLI both now appear to reuse the same transport/session continuity markers on resumed targeted host turns;
- the resumed targeted host-turn continuity seam is now validated enough on both surfaces, including the strict public `start` -> `turn` smoke returning `Just reply OK`;
- branch-local landed `382a63a8`, `7732d839`, `1374775e`, and `484a17b6` are the seam-moving committed packets that got the retained path from bootstrap/world-binding and exact-continue failure into the current later stop-closeout seam;
- exact retained `continue_world_worker` is now green in live smoke on both exact handles:
  - source returns `SOURCE_CONTINUE_OK`
  - fork child returns `CHILD_CONTINUE_OK`
- generic same-backend follow-up after fork now correctly fails closed with `ambiguous_backend_slot ... multiple authoritative retained turn targets for backend cli:codex-world (...)`, which is expected once two retained world workers are live in the same orchestration session;
- the newest valid retained fork smoke now shows:
  - retained `fork_world_worker` succeeds
  - the fork call returns a real child `participant_id`
  - exact retained continue on the source worker succeeds
  - exact retained continue on the fork child succeeds
  - but exact retained `stop_world_worker` on the source worker now fails with:
    - `owner_unreachable ... failed to connect to private stop transport ... durable stop closeout was not observed`
- the newest stop-path investigation also shows the active contradiction is no longer best summarized as detached host posture or selector denial:
  - host session can still persist as `parked_resumable`
  - target retained source worker can still persist `control_owner_retained = true`
  - target retained source worker can still persist `event_stream_active = true`
  - target retained source worker can still persist `completion_observer_retained = true`
  - target retained source worker can still persist `ownership_valid = true`
  - the stop socket path can still exist
  - but direct connect still returns `ConnectionRefused`
- still-open likely bug buckets are therefore now:
  - exact retained stop private-stop reachability / durable closeout after successful `fork_world_worker`,
  - then child-stop validation,
  - then renewed retained `cancel_world_work` validation,
  - then worker non-zero execution failures,
  - then remaining caller-boundary / harness realism gaps,
  - and host-visible file/write semantics;
- the enum/type cleanup remains useful post-stabilization hardening, but it should stay behind the retained-worker queue rather than acting as the next live patch target.

## Guardrails for future troubleshooting

When using this file during debugging, keep these constraints explicit:

- treat live code/tests/docs as authority over this memo;
- do not treat “completed” as proof that the requested file side effect was host-visible;
- do not treat parked/resumable host posture as proof that world-member routing remains valid;
- do not assume `reattach` is still the active blocker unless a fresh repro shows it; the latest smoke says it is not;
- do not use shell session JSON as co-authoritative Linux world-binding proof for the ordinary unprefixed REPL command repair seam;
- do not assume shared-world metadata absence/unreadability is repairable on this REPL surface unless the design/docs explicitly say so;
- do not describe `active_attached` or auto-attach in process-liveness terms; the transcript clarified that all durable truth is Substrate-owned and backend execution is headless/ephemeral;
- do not reintroduce owner PID liveness as a correctness condition for exact retained follow-up routing; committed `484a17b6` explicitly moved that out of the contract;
- do not collapse the current source-stop seam back into a generic host-posture/public-control gating story unless fresh state disproves the latest contradiction:
  - current live local truth says the host session can still be `parked_resumable`,
  - the target retained worker can still persist all four ownership truth bits as `true`,
  - the stop socket path can still exist,
  - and the transport can still refuse connects anyway;
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
