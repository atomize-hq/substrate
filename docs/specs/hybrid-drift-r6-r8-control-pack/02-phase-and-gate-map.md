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
| `R6-REPLAY` | ACTIVE | Bounded real-rollout/replay closeout | SATISFIED — controls and all conditional fixes complete; authority transition series `56bb9966f` + `07a3b1fe5` fresh independent built-in `default` `REVIEW CLEAN` | OPEN — `CTX-R6-01` series `a0089c8de` + `968a4377f` is fresh independent `REVIEW CLEAN`; `CTX-R6-02` witness `60cde3dd7` is behavioral RED and routes to active packet `R6-GAP-DET-REPLAY-STALL`. Packet docs/annotation `200725001` and first authority correction `08fa86e94` are landed; the current final status correction and full Task `.0` series await fresh independent review. `CTX-R6-06` and the family wall follow only after packet completion. |
| `R6-CLOSE` | BLOCKED | R6 `CLOSED` authority reconciliation | Replay closeout green; no ordinary gap open | Every scorer has a terminal disposition and all root/R6/R7 status docs agree. |
| `R7-PROMOTE` | BLOCKED | Promote preserved R7 drafts to implementation-ready | R6 finding says `CLOSED` | R7 MAP/SPEC/PLAN/TASKS and root mirrors agree; implementation has not yet started. |
| `R7-0..R7-6` | BLOCKED | Bounded direct-child delegated-session support | R7 promoted | R7 acceptance, real-corpus proof, and minimal sentinel compatibility are review-clean. |
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
siblings `421`/`475`. `R6-REPLAY` remains active; active packet is
`R6-GAP-DET-REPLAY-STALL`. Packet docs/annotation `200725001` and first authority correction
`08fa86e94` are landed; this final status correction completes the current review-fix candidate, but
Task `.0` remains incomplete/not review-clean until the full series receives a fresh clean verdict.
The current action is fresh series review/fix, not Prompt 2 or Rust. DECISION REQUIRED
`R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` follows only after Task `.0` is review-clean. `CTX-R6-06`,
the family wall, R6 closeout, terminal scorer dispositions, and R7/R8 remain pending or blocked.

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
