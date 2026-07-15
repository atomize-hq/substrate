# Phase And Gate Map

## Status Vocabulary

- **COMPLETE:** exit gate proven and authority reconciled.
- **ACTIVE:** the only phase authorized for implementation or docs work.
- **BLOCKED:** an earlier exit gate is not satisfied.
- **CONDITIONAL:** created only if a named failing witness appears.
- **BOUNDARY ONLY:** scope is known, but no implementation specification exists.

At most one implementation phase may be active. Docs-only authority repair may precede it.

## Master Sequence

| Phase | Status | Deliverable | Entry gate | Exit gate |
|---|---|---|---|---|
| `PACK-0` | COMPLETE | This context pack | R6/R7 authority correction `6362edf4e` landed | Pack cross-doc checks passed and root active docs link to it. |
| `R6-C.0A` | COMPLETE | Pass 1 authority remediation | Fresh review findings recorded in `01` and `05` | Canonical finding/root/R6/R7 gates and proof wording agree at `d3dcda785`; fresh review is clean. |
| `R6-C.1-SPEC` | COMPLETE | R6-C.1 SPEC/PLAN/TASKS | `R6-C.0A` complete | Expanded control matrix, expected decisions, files, commands, and stop rules landed and received fresh `REVIEW CLEAN` at `ea19b39a7`. |
| `R6-C.1-CONTROLS` | COMPLETE | Acceptance controls only | SATISFIED — R6-C.1 docs landed and received fresh `REVIEW CLEAN` at `ea19b39a7` | SATISFIED — all thirteen controls have deterministic results (`10 PASS / 3 preserved RED`) and the controls wall is recorded at `5618f7864`. |
| [`R6-GAP-*`](05-proof-decision-regression-ledger.md#named-r6-gap-status-subledger) | COMPLETE | One bounded scorer-specific gap phase per proven red | SATISFIED — the named-gap subledger instantiated all three preserved reds sequentially | SATISFIED — every named witness has review-clean focused proof and the final authority transition is landed. |
| `R6-REPLAY` | COMPLETE | Bounded real-rollout/replay closeout | SATISFIED — controls and all conditional fixes complete; packet transition series `1ff592823` + `7839a7f47` fresh independent `REVIEW CLEAN`; active packet `none` | SATISFIED — phase-owned proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` is fresh independent built-in `default` `REVIEW CLEAN`; exact replay controls pass `4 x 1 / 1`, family filters pass `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`, full analyzer passes `402 / 402`, and diff check is green. Sticky authority remains `HistoricalOnly / 20`, unflagged; old `Recovered / 20` is historical baseline only. No ordinary replay gap remains. |
| `R6-CLOSE` | COMPLETE | R6 `CLOSED` authority reconciliation | SATISFIED — replay closeout proof receipt is fresh-review-clean and no ordinary gap is open | SATISFIED — `CTX-R6-17` assigns every material surface a terminal disposition; all root/R6/R7 gate/status docs agree; and transition/review-fix series `13b14d5f1` + `50446e6d6` received fresh independent built-in `default` `REVIEW CLEAN` with no actionable findings. |
| `R7-PROMOTE` | COMPLETE | Promote the R7 authority family to implementation-ready | SATISFIED — R6 finding says `CLOSED`, terminal scorer table is complete, and authority stack agrees | SATISFIED — promotion series `455d0ed90` + `876ac55de` makes the R7 MAP/SPEC/PLAN/TASKS implementation-ready and received fresh independent built-in `default` `REVIEW CLEAN`; transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` also received fresh independent built-in `default` `REVIEW CLEAN`; no R7 task started. |
| `R7-0` | COMPLETE | Docs lock and sanitized evidence matrix | SATISFIED — `R7-PROMOTE` is complete; transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` and `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` are fresh independent `REVIEW CLEAN` | SATISFIED — fixture-only `R7-0.2` commit `fa85cd4b8` received fresh independent built-in `default` `REVIEW CLEAN`; focused parser/privacy passes `2 / 2`, full compactor passes `25 / 25` including end-to-end `2 / 2`, and privacy scans over `24` rows found zero private markers and zero raw UUIDs. No production symbol changed. |
| `R7-1` | COMPLETE | Compactor linkage and direct-child closure | SATISFIED — `R7-0` exit gate and transition/fix series `339744dff` + `d20cac6a9` are fresh independent built-in `default` `REVIEW CLEAN` | SATISFIED — R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` plus behavior/static checkpoint and checkpoint-doc commit `1cae7d693` are complete and fresh independent built-in `default` `REVIEW CLEAN`; `CTX-R7-02` is proven. |
| `R7-2` | COMPLETE | Analyzer link graph and checkpoint v0.8 delegation contract | SATISFIED — `R7-1` exit gate is fresh-review-clean; operator decision `R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded seam | SATISFIED — R7-2 task commits `c60d05f77`, `9403c8a24`, and series `7af2ae517` + `75a353e46` plus behavior/static checkpoint and checkpoint-doc commit `78a168c09` are complete and fresh independent built-in `default` `REVIEW CLEAN`; `CTX-R7-03` is proven. |
| `R7-3` | ACTIVE AT ENTRY ONLY / `R7-3.1` UNSTARTED | Child-visible progress separation | SATISFIED FOR ENTRY ONLY — `R7-2` exit gate is fresh-review-clean at checkpoint-doc commit `78a168c09`; transition/fix series `e27d82580` + `305e40bf2` is fresh independent built-in `default` `REVIEW CLEAN` after fix `305e40bf2` corrected the first review's stale R7 plan paragraph; active packet is `none`; Prompt 1 selectors are prepared and eligible but have not been invoked; `R7-3.1` is next, unchecked, and unstarted | PENDING — `R7-3.1`, `R7-3.2`, and the R7-3 checkpoint are complete and fresh-review-clean. |
| `R7-4..R7-6` | BLOCKED | Delegated scorer guardrails through minimal sentinel compatibility | `R7-3` exit gate review-clean, then each ordered predecessor phase complete | R7 acceptance, real-corpus proof, and minimal sentinel compatibility are review-clean. |
| `R8-SPEC` | BLOCKED | R8 MAP/SPEC/PLAN/TASKS | R7 closed with stable analyzer contract | R8 consolidation/integration interfaces, migration, proof wall, and non-goals are review-clean. |
| `R8-IMPLEMENT` | BOUNDARY ONLY | Sentinel interpretation consolidation/integration | R8 docs landed | Replay/live share one seam, compatibility is centralized, presentation stays presentation-first. |

`PACK-0`, `R6-C.0A`, `R6-C.1-SPEC`, `R6-C.1-CONTROLS`, and aggregate `R6-GAP-*` are complete.
`R6-GAP-DET-OPAQUE-PARENT` and `R6-GAP-TGG-TRUTH-PATH-ACTION` remain complete at their recorded
review-clean series. Final route `R6-GAP-WPB-EMPTY-AUTHORITY` is complete after
implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh
independent built-in `default` `REVIEW CLEAN` with exact, protected, family, checkpoint,
full-analyzer, and static proof green. Authority transition series `56bb9966f` + `07a3b1fe5`
received fresh independent built-in `default` `REVIEW CLEAN` and activates exactly one concrete
phase: `R6-REPLAY`, initially with active packet `none`. Replay work completed `CTX-R6-01` through
fresh independent review-clean series `a0089c8de` + `968a4377f`. Trusted witness `60cde3dd7` then
preserved `CTX-R6-02` behavioral RED: checkpoint `5` is `TroubleshootingFrontier / Stalled`, score
`Active / 30 / High`, flagged, but evidence misattributes failed calls `420`/`474` to successful
siblings `421`/`475`. Historical witness `60cde3dd7` remains preserved. Bounded implementation/proof
commit `6eda87e60` makes exact `CTX-R6-02` green at its locked `Stalled / Active` contract, passes the
complete ordered packet wall and full analyzer `402 / 402`, and received fresh independent built-in
`default` `REVIEW CLEAN`. `R6-GAP-DET-REPLAY-STALL` and `CTX-R6-02` are complete. Packet transition
series `1ff592823` + `7839a7f47` received fresh independent built-in `default` `REVIEW CLEAN`;
`R6-REPLAY` is complete with packet `none`. Phase-owned exact `CTX-R6-01`, exact `CTX-R6-02`,
renamed sticky, and exact `CTX-R6-06` each pass `1 / 1`. The Manifest E family wall passes
`21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, `169 / 169`, full analyzer `402 / 402`, and diff check.
Proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` received fresh independent built-in
`default` `REVIEW CLEAN`, so no ordinary replay gap remains. Sticky authority remains
`HistoricalOnly / 20`, unflagged, and `Recovered / 20` remains historical baseline only.
On 2026-07-15, `CTX-R6-17` reconfirmed the four exact replay controls at `1 / 1` each, the Manifest E
family filters at `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`, the full analyzer with
all suites green, and `git diff --check`. It assigns `dead_end_thrash` and `semantic_goal_drift`
**Cutover complete** plus `truth_grounding_gap`, `wrong_plan_branch`, and `scoring/mod.rs`
**Fit-for-purpose exception**. Transition/review-fix series `13b14d5f1` + `50446e6d6` is fresh
independent built-in `default` `REVIEW CLEAN` with no actionable findings. `R6-CLOSE` is complete and
R6 is `CLOSED`. Promotion series `455d0ed90` + `876ac55de` makes the R7 authority family
implementation-ready and received fresh independent built-in `default` `REVIEW CLEAN`;
`R7-PROMOTE` is complete. Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` +
`faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` are fresh independent built-in `default`
`REVIEW CLEAN`, completing `R7-0`. Transition/fix series `339744dff` + `d20cac6a9` also received
fresh independent built-in `default` `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`,
`4d122cd9f`, and `e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`; checkpoint-doc
commit `1cae7d693` is also fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1
exit gate. Transition commit `6a8797c15` received `CHANGES REQUIRED` for stale uncommitted-state
wording; fix `4ee469014` corrected it, and a fresh independent built-in `default` re-review returned
`REVIEW CLEAN` for the full series. R7-1 remains complete. R7-2 task commits `c60d05f77` and
`9403c8a24`, plus R7-2.3 series `7af2ae517` + `75a353e46`, received fresh independent built-in
`default` `REVIEW CLEAN`; the fix reconciled the summary-vs-checkpoint blocker. The R7-2 task and
behavior/static checkpoint proof is complete: input `16 / 16`, delegation matches `39`, checkpoint
matches `172`, full analyzer `417 / 417`, and static gates green. Checkpoint-doc commit `78a168c09`
received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-2 exit gate and
proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet `none`;
transition/fix series `e27d82580` + `305e40bf2` received fresh independent built-in `default`
`REVIEW CLEAN` after fix `305e40bf2` corrected the first review's stale R7 plan paragraph. `R7-3.1`
is next, unchecked, and unstarted;
R7-3 analyzer/production implementation has not started. `R7-4..R7-6` plus R8 remain blocked.
Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and eligible but have not
been invoked.

## R6-C.0A — Closure-Audit Authority Remediation

**Result:** **COMPLETE** at `d3dcda785` after the seven authority defects were corrected and the
remediation received fresh `REVIEW CLEAN`.

### Required changes

- downgrade broad replay honesty to partially proven;
- distinguish frozen posture invariance from integrated scorer improvement;
- expand `truth_grounding_gap` controls with truth-touching action and actionful-planning cases;
- expand `wrong_plan_branch` controls with empty-authority path-bearing action;
- replace “still open” at closure with terminal disposition categories;
- correct semantic fixture-integrity and dispatcher-order proof descriptions; and
- label pre-cutover R6 MAP/root status prose historical or superseded.

### Stop conditions

- a change would reopen `semantic_goal_drift` without new behavior-level evidence;
- the remediation attempts scorer implementation;
- root and packet authority cannot be reconciled without a product decision.

## R6-C.1 — Acceptance Controls

### Required control families

**`dead_end_thrash`**

- regressing progress;
- opaque delegated-parent activity;
- long-autonomous versus many-short-conversational behavior, or explicit proven equivalence;
- integrated advancing repeated-failure replay witness; and
- integrated true-stall replay witness.

**`truth_grounding_gap`**

- research/planning with no triggering action;
- successful-but-ungrounded verification;
- long-turn invariance;
- opaque parent orchestration;
- truth-path-touching write/verification before a read; and
- actionful planning/research versus implementation with equivalent evidence.

**`wrong_plan_branch`**

- read-only exploration;
- sanctioned replan/path pivot;
- opaque parent orchestration; and
- path-bearing write/verification with no authoritative truth or working-set scope.

### Decision rule

Add tests before production changes. If current behavior passes and matches the scorer's
responsibility, record fit-for-purpose proof. If behavior fails honestly, preserve the witness and
open one narrow fix packet. If the old charter wording is the problem, narrow the wording rather
than create irrelevant behavior.

## R6 Close Gate

R6 may say `CLOSED` only when each material scorer is one of:

1. **cutover complete**;
2. **fit-for-purpose exception** with behavioral proof;
3. **merged/deprecated** with migration proof; or
4. **explicitly deferred outside R6 with justification**, named owner, and trigger.

No ordinary implementation gap or acceptance-proof gap may remain open at R6 closure.

## R7 Gate

R7 extends the stable R6 baseline. Its first cut remains:

- reciprocal structured direct-parent/direct-child linkage;
- compactor-owned raw parsing;
- analyzer-owned typed topology and separate trajectories;
- direct children only;
- no new drift class by default; and
- minimal sentinel compatibility, leaving broad interpretation consolidation to R8.

Any R7 evidence that reveals an ordinary single-session scorer defect routes back to a separately
owned baseline defect; R7 must not absorb it silently.

## R8 Gate

R8 — **Sentinel Interpretation Consolidation / Integration** begins only after R7 stabilizes the
analyzer-owned checkpoint/delegation contract. Before implementation, create and review a dedicated
R8 MAP/SPEC/PLAN/TASKS family. The current root boundary is not an implementation plan.

The future R8 spec must preserve:

- analyzer semantic ownership;
- replay/live behavior parity;
- centralized version/compatibility handling;
- presentation-first operator surfaces; and
- no scheduler/adjudication redesign unless separately justified.
