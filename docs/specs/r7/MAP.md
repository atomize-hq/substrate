# R7 Map: Bounded Delegated-Session Semantics

Status: **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-6 COMPLETE / R7 CLOSED / `CTX-R7-06`
PROVEN / R8-SPEC SOLE ACTIVE PHASE IN PROGRESS / ACTIVE PACKET NONE**

R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself.

Promotion series
`455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`. The narrow
`R7-PROMOTE -> R7-0` transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. The docs-only `R7-0.1` series `a9e75f149` +
`55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh
independent built-in `default` `REVIEW CLEAN`, completing `R7-0`. Transition/fix series
`339744dff` + `d20cac6a9` also received fresh independent built-in `default` `REVIEW CLEAN`.
`R7-1.1` series `e65127720` + `685cf843b`, `R7-1.2` commit `4d122cd9f`, and `R7-1.3` commit
`e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. R7-1 implementation and its
checkpoint are complete. Checkpoint-doc commit `1cae7d693` received fresh independent built-in
`default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself.

R7-3 behavior/static proof at implementation HEAD `c7c6f35b8`: R7-3.1's placeholder acceptance
scaffold intentionally produced test-scaffold RED `0 / 1`; after replacement with the real
acceptance assertion, its exact target passed `1 / 1` and full `progress_acceptance` passed `4 / 4`.
R7-3.2's placeholder acceptance scaffolds intentionally produced test-scaffold RED `3 / 3`; after
replacement with the real acceptance assertions, exact `3 / 3` passed, and the checkpoint filter
passed `175` matched tests across targets. `cargo test -p agent-drift-analyzer -- --nocapture`
passes `421 / 421` aggregate. `cargo fmt --all -- --check`,
`cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and `git diff --check` are green;
both staged GitNexus gates were LOW / `0` affected processes.
At that R7-3 boundary, `cargo clippy --workspace --all-targets -- -D warnings` remained RED only in R7-6-owned sentinel test
constructors missing `Checkpoint.delegation` at
`crates/agent-drift-sentinel/tests/support/mod.rs:81`,
`crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs:48`, and
`crates/agent-drift-sentinel/tests/replay_input.rs:79`; route that witness to already-planned R7-6.1
and do not claim workspace clippy green.

R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` is fresh independent built-in `default`
`REVIEW CLEAN`. The first review returned `CHANGES_REQUIRED` with one P2/Important contract failure:
the witness hand-built `Verified` linkage with untruthful provenance. The fix routes it through
production `export_bundle` with a matching spawn call/result, child-origin `session_meta` at event
`0`, and discovery count `2`; fresh re-review returned `CLEAN`. R7-4.2 docs decision commit
`8a0790a3d` received fresh independent `CLEAN` with no findings and records existing `DriftClass`
values plus typed delegation context as sufficient for the proven evidence. At HEAD `8a0790a3d`,
`dead_end_thrash` passes `19 / 19`, the `semantic_goal_drift` filter passes `58 / 58` aggregate
(`56` library plus `2` acceptance), and full analyzer passes `422 / 422`; formatting, analyzer
clippy with `-D warnings`, and diff checks are green. At that R7-4 boundary, the known workspace-
clippy RED remained routed to R7-6.1 and was neither rerun nor fixed. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself.

## R6 Handoff

The attached planning direction correctly identified delegated-session opacity as the next major
architectural gap. Its proposed new `dead_end_thrash` cutover would duplicate the landed R6-1 core,
and the broader R6 closure charter is now closed:

- `R6-1` already cut `dead_end_thrash` over to analyzer-owned `SessionProgress`, including
  troubleshooting-frontier advancement suppression and non-advancing stall evidence.
- `R6-2`, `R6-3`, and the `R6-3.X.2B` / `2C` / `2D` chain landed the semantic-goal-drift consumer,
  rolling comparison, target hygiene, weighted relation routing, and explicit decision semantics.
- `R6-3.X.3` remains deferred. No new semantic-goal-drift work starts without new failing evidence.
- `truth_grounding_gap` and `wrong_plan_branch` completed their bounded gaps and are terminal
  **Fit-for-purpose exception** surfaces.
- conditional `R6-4` progress-reset migration remains deferred because the required reset-error
  evidence did not appear.

R6 is **CLOSED**. `R6-REPLAY`, `R6-CLOSE`, and `CTX-R6-17` are complete. The closure authority is
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. Commit `99efda8f9` remains historical
planning input, not authority for the current R7 phase status.

The R6 promotion entry gate is satisfied: every material scoring surface has a terminal disposition,
the broad R6 acceptance claims are proven or narrowed honestly, the named closure controls are
resolved, and the R6 finding plus root/R6/R7 gate/status stack agree. Promotion series `455d0ed90`
+ `876ac55de` reconciled the preserved MAP/SPEC/PLAN/TASKS to implementation-ready authority and
received fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-PROMOTE`. The narrow
status transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in
`default` `REVIEW CLEAN`. The exact `R7-0.1` contract `rg` passed, and series `a9e75f149` +
`55bea5fa5` + `faff68ac6` received fresh independent built-in `default` `REVIEW CLEAN`.
Fixture-only `R7-0.2` commit `fa85cd4b8` also received fresh independent built-in `default`
`REVIEW CLEAN`: all seven sanitized cases parse without failures, the focused target passes
`2 / 2`, the full compactor family passes `25 / 25` including end-to-end `2 / 2`, and all `24`
rows pass privacy scans with zero private markers and zero raw UUIDs. `R7-0` is complete and
review-clean. Transition/fix series `339744dff` + `d20cac6a9` received fresh independent built-in
`default` `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13`
are fresh independent built-in `default` `REVIEW CLEAN`. Focused delegation-link proof passes `6 /
6`; linked-closure end-to-end and CLI proof pass `6 / 6` and `2 / 2`; the full compactor wall passes
`36` unit/integration tests plus `3` doctests; formatting, clippy, diff, and staged GitNexus gates are
green. Link/session/file ordering is deterministic, and no raw private rollout data was added.
Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-1 exit gate. R7-1 is complete. Operator decision
`R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded high-impact analyzer seam. R7-2
task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 implementation/fix series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN` after the fix reconciled
the summary-vs-checkpoint blocker. At implementation HEAD `75a353e46`, input passes `16 / 16`,
delegation matches pass `39` total, checkpoint matches pass `172` total, full analyzer passes `417 /
417`, and format, `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and diff checks are green. Staged GitNexus gates
reported R7-2.1 LOW / `0` affected processes, R7-2.2 MEDIUM / `1`, R7-2.3 HIGH / `9` within the
authorized decision, and the fix MEDIUM / `2`. Public v0.8 with v0.7 compatibility, graph-derived
roles and ids, `Linked`/`Partial` visibility, fail-closed conflicts, deterministic `RowRef` evidence,
JSON-summary parity, and separate trajectories are proven. No R7-3, R7-4, sentinel, or R8 work
leaked into the series. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself.

## Live Linkage Evidence

The current raw Codex rollout shape exposes a deterministic direct-link seam:

1. A parent `spawn_agent` function-call output contains the spawned child `agent_id`.
2. The child's `session_meta.payload.source.subagent.thread_spawn` contains the matching
   `parent_thread_id`, `depth`, nickname, and role.
3. The compactor now preserves structured parent spawn-result and child-origin metadata with row
   provenance, emits serde-defaulted typed delegation links, and supports an explicit verified
   direct-child closure without turning linkage into message text.
4. The analyzer loads and validates a deterministic direct link graph over included bundle sessions;
   missing and conflicting link states remain bounded and legacy manifests load with an empty graph.
5. Public checkpoint v0.8 exports `DelegationContext`, topology, graph-derived role ids, visibility,
   confidence, and deterministic `RowRef` evidence. Verified links populate `DelegatingParent` and
   `DelegatedChild`; heuristic markers are fallback-only, and v0.7 remains readable.
6. Real-session sentinel coordination currently compacts one requested session and rejects analyzer
   output containing any additional session id.

This evidence fixes the ownership boundary: the compactor must preserve and validate raw linkage;
the analyzer must interpret linked trajectories; the sentinel must consume analyzer-owned semantics
rather than parse raw collaboration events itself.

## Non-Negotiable Invariant

> Never infer child implementation progress, child drift, or child completion from parent
> orchestration alone.

When child evidence is missing or a link is not verified, the supported result is bounded:
`delegating_parent`, `child_work_visibility=opaque|partial`, and insufficient evidence for child
semantics. Ordinary parent-visible orchestration may still be described as parent activity.

## Packet Order

The packet order below is implementation-ready. `R7-0` is **complete** after `R7-0.1` and
fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh independent `REVIEW CLEAN`.
Transition/fix series `339744dff` + `d20cac6a9` is also fresh independent built-in `default`
`REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
independent built-in `default` `REVIEW CLEAN`, and the R7-1 checkpoint is complete. Checkpoint-doc
commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-1 exit gate. Transition/fix series `6a8797c15` + `4ee469014` received fresh independent built-in
`default` `REVIEW CLEAN`; its first review's stale uncommitted-state finding is fixed. R7-1 remains
complete and `CTX-R7-02` remains proven. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself.

1. **R7-0 — docs lock and evidence fixtures.** Freeze the direct-link contract and sanitized
   positive/negative fixture matrix before production behavior changes.
2. **R7-1 — compactor linkage and bounded child closure.** Preserve parent/child metadata, verify
   reciprocal direct links, and add an explicit opt-in linked-child bundle closure.
3. **R7-2 — analyzer link graph and public trajectory roles.** Promote verified topology into an
   analyzer-owned public checkpoint surface while preserving separate session trajectories.
4. **R7-3 — child-visible progress separation.** Reuse each child's own `SessionProgress`; do not
   copy child progress into parent progress or synthesize it from waits/results.
5. **R7-4 — delegated scorer guardrails.** Apply existing scorers per trajectory and prevent
   parent-side opacity or waiting from becoming claims about child drift. The production-shaped
   R7-4.1 parent/child witness is expressible with existing drift classes plus typed delegation
   context: parent waits remain cleared while child-local repetition remains
   `dead_end_thrash`. No new drift class was added; a future distinct, unexpressible delegated
   failure mode requires its own evidence-backed reviewed packet.
6. **R7-5 — delegated acceptance and real-corpus proof.** Prove linked, missing, conflicting,
   multi-child, and non-delegated cases with explicit topology strata.
7. **R7-6 — minimal sentinel compatibility.** Accept and render analyzer-owned linked trajectories
   in replay/live without performing the broader R8 interpretation consolidation.

## Deferred Beyond This Family

- recursive or arbitrary-depth execution graphs; the first supported model is direct parent to
  child, with deeper descendants remaining explicit bounded residue;
- inferred links based only on prose, filenames, or temporal proximity;
- aggregating child progress into one parent progress status;
- a new delegated-session drift class; R7-4 evidence does not require one, and any future distinct
  failure mode requires a separate evidence-backed reviewed packet;
- learned delegation or progress models;
- R8 replay/live interpretation consolidation;
- reopening `R6-3.X.3`, conditional `R6-4`, or closed semantic-goal-drift packets.
