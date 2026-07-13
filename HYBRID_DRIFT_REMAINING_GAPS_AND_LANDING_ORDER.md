# Hybrid Drift Remaining Gaps And Landing Order

## Why This Document Exists

The hybrid-drift stack has already landed a meaningful checkpoint-state cutover, but the review
loop kept re-suggesting that same slice as if it were still open. This document corrects that. It
records:

- what is already landed
- what is still missing
- which gap is actually blocking honest analyzer and sentinel behavior now
- the order the remaining work should land in

This file is the repo-root landing-order narrative for the hybrid-drift stack. Authority for the
current R6 closure decision lives in
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`, then `docs/specs/r6/MAP.md`, the R6
design, and the per-packet sets. The scoped R6 packets have landed, but R6 is **PARTIAL / CLOSURE
AUDIT REQUIRED** until the named acceptance controls close. The Ground Truth Sources list below is
historical context, not an exhaustive index of current authority.

> **Status note (2026-07-04):** `R5`/`R5.5`/`R5.75` structured-objective work has advanced well beyond the
> older root sequence, through `R6` scorer cutover and into `R6` real-world validation. The current blocker
> before any further `semantic_goal_drift` promotion is objective **target hygiene** (`R6-3.5`): a 110-session
> batch showed the signal's live fires were false positives caused by junk target extraction. Landing order is
> locked — `R6-3.5` extraction hardening → `R6-3.X.2` graduated distance → `R6-3.X.3` eligibility-bar revisit.
> **Do not loosen `semantic_goal_drift` eligibility until the `R6-3` validation gate is re-run.** Full
> delegated-session (subagent) semantics remain deferred to `R7`; `R6-3.5` only adds a bounded opaque-parent
> guardrail so parent-only traces do not over-claim. See `docs/specs/r6/R6-3.5/` and
> `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`.

> **Closure correction (2026-07-12):** scoped R6 packet landing is not authority that the broad R6
> scorer-context charter is closed. The live status is **PARTIAL / CLOSURE AUDIT REQUIRED** per
> `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. R7 documents are design-ready drafts
> but blocked from implementation until that finding reaches `CLOSED`.

## Ground Truth Sources

- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `docs/specs/r5/R5_75/` (the landed `R5.75` pre-`R6` hardening family, incl. `phase-1/SO/`)
- `docs/specs/r6/MAP.md`
- `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`
- `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
- `docs/specs/r6/R6-1/` and `docs/specs/r6/R6-2/` (the committed `R6` packet sets)
- `.codex/handoffs/2026-06-04-180058-drift-sentinel-rollout-review.md`
- `target/hybrid-drift-evals/*/analyzer/summary.md`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`

## Current Status

### What Is Already Landed

The following slices are landed and should not be re-suggested as the current top missing fix in
this worktree:

- analyzer-owned explicit outcome evidence from the `R1` family:
  - generic `ToolOutput` is neutral by default
  - only unambiguous failure markers such as explicit `Error` rows, leading `error:`, or non-zero
    exit-code output count as failure evidence
- analyzer-owned per-class `DriftState`
- checkpoint schema widening from `v0.2` to `v0.3`
- sentinel cutover to prefer analyzer-exported state for `v0.3`
- isolated `v0.2` replay/live compatibility fallback
- analyzer-owned turn context from `R3` with checkpoint schema `v0.4`
- replay/live trigger-headline canonicalization from `R3.5`
- the bounded analyzer-local delegation boundary from `R3.75`
- analyzer-owned session archetype from `R4` with checkpoint schema `v0.5`
- analyzer-owned session progress from `R5` with checkpoint schema `v0.6`

That means the old recommendations to “treat generic `ToolOutput` as the top missing seam,”
“re-open turn context,” or “finish delegation before doing any R4 planning” are now stale as
top-level architecture asks. Those foundations are already landed.

### What Is Still Open

The stack no longer lacks explicit checkpoint-local session meaning or first-cut progress state.

The current analyzer now exports deterministic, evidence-backed `session_archetype` and
`session_progress` state, and replay/live sentinel surfaces render the same compact archetype and
progress views for matching checkpoints.

The next open gap is no longer archetype identification or first-cut progress export. `R5.5`
landed the first hardening pass; the active pre-`R6` gap is the narrower `R5.75` follow-on family
that reconciles post-validation remaining issues before scorer cutover opens.

### Why The Current Stack Still Needs Follow-On Work

The remaining gap is no longer “sentinel posture logic is missing,” “generic `ToolOutput`
pollutes failure evidence,” or “checkpoint-local session archetype is absent.” The current honest
next step is:

1. keep `R5` landed as the first-cut progress layer
2. keep `R5.5` landed as the first hardening pass
3. close the adopted post-validation follow-on family (`R5.75`)
4. retune scorers to consume the hardened progress layer (`R6`)
5. extend delegated-session semantics beyond the current downgrade boundary (`R7`)

Now that `R4` is landed, later packets can consume typed session meaning instead of inferring it
from turn shape, objective wording, and command mix alone.

## Signals The Analyzer Already Has

The analyzer summary already exports several data points that should be used by follow-on modules.
These are not hypothetical; they are visible in `target/hybrid-drift-evals/*/analyzer/summary.md`.

Examples already exported:

- `Turns observed`
- `User prompts observed`
- `Checkpoints emitted`
- `Checkpoints per turn`
- `Checkpoints per user prompt`
- `Avg rows between checkpoints`
- `Avg seconds between checkpoints`
- `Longest flagged streak`
- `Task-frame transition count`
- `Task-frame confidence distribution`
- `Working-set churn`
- `Verification density`
- `Average evidence items per checkpoint`
- `Distinct task frames`
- `Truth artifacts referenced`
- `Verification commands observed`
- per-checkpoint `flagged`, `next`, and drift-class list

This means the stack already knows several facts that are currently underused:

- whether the session is one long agent-side turn or many short turns
- whether checkpoints are accumulating rapidly inside one prompt
- whether the task frame is stabilizing or churning
- whether verification is dense or sparse
- whether the current run is command-heavy, commentary-heavy, or verification-heavy

The gap is not raw data availability. The gap is that this data is mostly exported as summary
metrics instead of being promoted into one analyzer-owned per-checkpoint context module.

## Remaining Gaps

## Gap 1: Session Archetype Was Missing, And Is Now Landed

### Problem

The pre-`R4` stack largely treated every session as if repeated failure meant the same thing. That
gap is now closed on this worktree: checkpoints explicitly distinguish:

- troubleshooting
- planning / brainstorming
- autonomous implementation
- review / closeout

### Why This Matters

Each archetype has different honest progress signals.

- troubleshooting expects tool failures
- planning expects low verification density and more directive synthesis
- autonomous implementation expects steady write/test loops against the kickoff scope
- review / closeout expects narrowing scope and proof-oriented verification

### What Needed To Exist

One analyzer-owned `session_archetype` module that classifies checkpoints using observable
evidence:

- kickoff prompt shape
- truth artifact usage
- tool mix
- write/test cadence
- user prompt cadence
- working-set concentration
- verification density

The classification needed to be additive and confidence-bearing. That contract is now landed under
checkpoint schema `v0.5`, with replay/live compatibility preserved for legacy schemas.

## Gap 2: The Landed Progress Layer Still Needs Hardening Before Scorer Consumption

### Problem

`R5` added `session_progress`, but post-landing review showed a small number of remaining gaps that
can still make progress unsafe scorer input if left unresolved.

### Why This Matters

A troubleshooting session can fail the same command repeatedly while still making real progress, but a
scorer must not be fed false `advancing` when the same failure simply repeats after an overlapping
edit.

The same review pass also showed that objective extraction, JS/TS verifier recognition, delegated
parent-visible normalization, and real-rollout acceptance depth still need bounded hardening before
`R6` cutover.

### What Needs To Exist

One bounded post-landing hardening sequence that now spans:

- landed `R5.5` baseline work:
  - troubleshooting repeated-failure hardening
  - objective extraction hardening
  - JS/TS verifier attempt classification hardening
  - committed real-rollout acceptance deepening
  - delegated parent-visible normalization and limiting-evidence hygiene
  - the most misleading residual doc/code hygiene cleanup
- active `R5.75` follow-on work:
  - objective condensation / target extraction for giant pasted prompts
  - sparse readable-session fail-open instead of analyzer hard-abort
  - delegated parent-visible stabilization under validation pressure
  - zero-verifier anti-flap gating for long exploratory sessions
  - adapted external robustness fixtures before `R6`

## Gap 3: Real Rollout Acceptance Is Too Weak

### Problem

The current tests prove a lot of contract plumbing and a first-cut semantic wall, but the review
showed that green tests were still not enough to prove semantic honesty on real completed sessions.

### Why This Matters

Without a real rollout-tail acceptance seam, shallow heuristics can stay green while still being
wrong on the exact sessions the sentinel is meant to help with.

### What Needs To Exist

One bounded acceptance module that:

- runs compactor -> analyzer -> sentinel on known real-session artifacts
- asserts final checkpoint honesty against explicit rollout-tail markers
- distinguishes fixture-backed proof from true live proof

At minimum, one regression must prove:

- repeated successful tool output plus a normal `task_complete` tail does not end in active
  `dead_end_thrash`

## Gap 4: Sentinel Still Pays Duplicate Interpretation Cost

### Problem

Replay input, live input, and operator-surface logic still pay a duplicated checkpoint
interpretation tax.

The duplication includes:

- replay checkpoint contract checks
- live checkpoint contract checks
- compatibility handling
- some legacy posture interpretation paths

### Why This Matters

This is not the bug blocking honesty today, but it still hurts locality and AI navigability.
Schema widening and compatibility logic should not need to be paid for in multiple sentinel
modules.

### What Needs To Exist

One sentinel checkpoint-interpretation module that replay and live adapters both feed, leaving
operator-surface code with presentation-only responsibilities.

This is cleanup after the analyzer-side semantic gaps are fixed, not before.

## Recommended Landing Order

The correct next sequence is not “reopen checkpoint-state.” The correct next sequence is the
follow-on after checkpoint-state.

## Packet Family R1: Analyzer Outcome Evidence Fix

### Objective

Stop treating generic `ToolOutput` as failure evidence, fix the follow-on repeated
verification-loop honesty gap that bounded replay exposed, and keep the proof and downstream
sentinel cleanup split into reviewable packets.

### Why First

Everything else depends on this. Archetype-aware and turn-aware heuristics are not trustworthy if
the underlying failure surface is already polluted. The replay-proof work is still part of this
family, but it should follow the analyzer-semantic cutover instead of sharing the first landing.

### Packet Split

#### Packet R1A

- docs lock for the `R1A` / `R1B` / `R1C` / `R1D` / `R1E` / `R1F` family boundary
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- analyzer tests for classification and repeated-failure grouping

#### Packet R1B

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- analyzer tests for recovery, downgrade, and export determinism

#### Packet R1C

- screened bounded replay proof over a small non-subagent corpus
- honest status correction when the representative sticky replay still fails

#### Packet R1D

- analyzer-owned repeated verification-loop semantics and recovery fix
- focused regressions for successful verification tails and out-of-scope verifier loops

#### Packet R1E

- rerun the bounded non-subagent replay proof after `R1D`
- continuity-note refresh if the final proof surface changed materially

#### Packet R1F

- narrow sentinel trigger/presentation cleanup after the analyzer proof is already honest

### Intentionally Left Out Of The R1 Family

The following work is intentionally not part of the `R1` family as currently scoped:

- real rollout-tail acceptance hardening beyond the narrow analyzer regression shape
  - falls under `R2`
- per-checkpoint turn context, long-turn detection, and turn-local checkpoint density
  - falls under `R3`
- delegation-aware analyzer topology and child-work visibility boundary for delegated/subagent runs
  - falls under `R3.75`
- session archetype classification such as troubleshooting vs planning vs review / closeout
  - falls under `R4`
- archetype-aware progress semantics such as frontier movement, narrowing candidate sets, or
  verification-wall advance
  - falls under `R5`
- broader drift-scorer redesign that combines typed outcome evidence with turn context, archetype,
  and progress modules
  - falls under `R6`
- full delegated-session semantic support beyond the narrow delegation-aware boundary, including
  supported parent/child progress and drift interpretation
  - falls under `R7`
- broader replay/live checkpoint interpretation redesign inside sentinel
  - falls under `R8`

The `R1` family is only the analyzer-local evidence-surface correction, the repeated
verification-loop honesty fix it exposed, the bounded replay proof needed to show those narrower
analyzer semantics behave honestly on completed non-subagent tails, and the narrow sentinel
trigger/presentation cleanup that follows that proof.

### Acceptance

- `R1A` lands the analyzer-owned outcome-evidence seam and repeated-failure cutover
- `R1B` proves honest recovery/export semantics on the narrower failure surface
- `R1C` proves the remaining sticky case is real, names the analyzer-owned follow-on seam
  precisely, and keeps the proof status honest
- `R1D` fixes repeated verification-loop semantics on focused analyzer regressions
- `R1E` proves the fix on a small handful of completed non-subagent sessions rather than a single
  spot-check
- `R1F` keeps sentinel trigger/presentation labels from overstating analyzer posture after the
  proof lands
- those proof sessions exclude subagent / delegated runs, or are explicitly screened to prove no
  subagent usage was present
- the family still does not widen into turn context, session archetype, progress semantics, or
  broader sentinel redesign

R1 family continuity notes:

- `2026-06-05`: `R1C` screening was grounded, but the first bounded replay proof did not land:
  - screened delegated sessions `019e93f8-a5e9-7490-ac1a-955b74c92ad0` and
    `019e9406-6736-79a2-946b-8a603e557422` remained excluded by `multi_agent_v1`
    `spawn_agent` / `wait_agent` / `close_agent` markers
  - clean non-subagent control replays still ended with final `dead_end_thrash.state=cleared` and
    `raw_score=0`:
    `019e93fa-60d4-73d1-9092-014130b60e14`,
    `019e940c-a91b-7fe0-a967-b0bdd595b581`,
    `019e943c-668e-7a03-992b-6a98cf3055da`
  - screened session `019e9401-9d69-7190-a43e-9ee3be08b369` remained excluded from the
    successful-output-only control corpus because its rollout included `Exit code: 1` tool-output
    rows, and its rerun still ended with active `dead_end_thrash`
  - representative non-subagent sticky live session `019e894a-86c9-71e3-b57b-e3d3285f0988` still
    ended with final `dead_end_thrash.state=active` and `raw_score=100` when rerun into
    `target/hybrid-drift-evals/019e894a-86c9-71e3-b57b-e3d3285f0988-r1c-refresh/`
  - conclusion: `R1C` honestly routed the family to `R1D` / `R1E`; the bounded replay proof was
    not landed yet
- `2026-06-06`: `R1D` and `R1E` closed the analyzer-owned replay gap on the same screened
  non-subagent corpus:
  - clean control reruns still end with final `dead_end_thrash.state=cleared`, `raw_score=0`, and
    `flagged=false`:
    `019e93fa-60d4-73d1-9092-014130b60e14`,
    `019e940c-a91b-7fe0-a967-b0bdd595b581`,
    `019e943c-668e-7a03-992b-6a98cf3055da`
  - representative sticky rerun `019e894a-86c9-71e3-b57b-e3d3285f0988` now ends with final
    `dead_end_thrash.state=recovered`, `raw_score=20`, and `dead_end_thrash.flagged=false` in
    `target/hybrid-drift-evals/019e894a-86c9-71e3-b57b-e3d3285f0988-r1e/`
  - the final proof corpus did not widen, delegated sessions remain excluded, and
    `019e9401-9d69-7190-a43e-9ee3be08b369` remains outside the successful-output-only proof set
    because its rollout includes `Exit code: 1` tool-output rows
  - conclusion: the bounded replay proof is now landed as bounded replay proof, not live proof

## Packet R2: Analyzer Acceptance Fixture Hardening

### Objective

Add analyzer-owned acceptance fixture hardening that prevents the same semantic mistake from
reappearing after the `R1` family is fully landed.

### Why Second

`R1E` owns the final screened bounded replay proof and continuity refresh for the analyzer fix.
`R2` starts only after that proof lands and focuses on keeping a durable analyzer-local acceptance
guard in place for later packets.

### Scope

- analyzer acceptance fixture(s)
- analyzer-local replay/assertion hardening that does not reopen the `R1C` bounded proof story
- follow-on regression maintenance for later `R*` packets

### Acceptance

- analyzer acceptance coverage keeps the final checkpoint from regressing back to falsely active
  on already-screened success-tail shapes
- `R1E` remains the sole owner of screened bounded replay proof claims and continuity-note refresh

R2 continuity notes:

- `2026-06-06`: `R2` is now landed and closes as analyzer-local acceptance hardening:
  - committed analyzer fixtures freeze the screened `R1E` success-tail corpus rather than reading
    mutable `target/` or `~/.codex` state at test time
  - the committed corpus contains exactly the three cleared controls plus the one recovered sticky
    case:
    `019e93fa-60d4-73d1-9092-014130b60e14`,
    `019e940c-a91b-7fe0-a967-b0bdd595b581`,
    `019e943c-668e-7a03-992b-6a98cf3055da`,
    `019e894a-86c9-71e3-b57b-e3d3285f0988`
  - delegated sessions remain excluded by policy, and
    `019e9401-9d69-7190-a43e-9ee3be08b369` remains outside the success-tail corpus because it is
    not a successful-output-only tail
  - the acceptance wall now asserts final analyzer `dead_end_thrash` posture directly from the
    frozen bundle inputs instead of routing through sentinel output
  - conclusion at that point: the next open packet after `R2` was `R3-1`, not more `R2`
    replay/fixture work

## Packet R3: Per-Checkpoint Turn Context

### Objective

Promote turn shape from summary-only reporting into analyzer-owned per-checkpoint state.

### Why Third

This is the smallest honest step toward long-agent-turn awareness without yet forcing session
archetype logic into the scorers.

### Scope

- add per-checkpoint turn-context fields
- compute turn age and turn-local checkpoint density
- export that state with checkpoints and summary
- widen analyzer checkpoints explicitly from `v0.3` to `v0.4`
- keep session archetype, session progress, and drift-scorer retuning out of scope for this
  packet

### Packet Split

- `R3-1`: repo-doc contract lock only
- `R3-2`: analyzer-owned `TurnContext`, `TurnActivityMix`, and `TurnExecutionMode` types plus
  `v0.4` export
- `R3-3`: deterministic turn-slice derivation, analyzer summary output, and long-turn vs
  many-short-turn analyzer regressions
- `R3-4`: sentinel `v0.4` compatibility and compact replay/live turn-context presentation

### Acceptance

- checkpoints can distinguish “one long agent-side turn” from “many short conversational turns”
- replay/live consumers can inspect turn-local context directly

### Landing Status

- `2026-06-07`: `R3` is landed.
- analyzer emits `v0.4` checkpoints with structured `turn_context`
- analyzer summary exposes compact turn-context inspection
- sentinel replay/live consumers accept `v0.4` while preserving legacy `v0.2` and `v0.3`
- replay/live surfaces expose the same compact turn-context view for matching checkpoints
- focused analyzer and sentinel verification commands for the packet are green
- bounded manual proof on real rollout `019ea027-9bb6-7aa2-9d05-94f3ba997948` confirmed that the
  new turn-context surface is correct, but also exposed one remaining sentinel-local headline seam:
  replay still derives the trigger headline from `checkpoint.flagged`, while live checkpoint
  arrivals still headline from ingress event type, so the same flagged checkpoint can still render
  `@ scheduler_repeated_failure_trigger` in replay and `@ checkpoint_ready` in live even when the
  posture, drift, diagnostics, and turn-context lines all match

## Packet R3.5: Replay/Live Trigger Headline Canonicalization

### Objective

Make replay and live use one honest operator-facing trigger headline for the same checkpoint
without reopening analyzer semantics or broader sentinel redesign.

This is the `R3.5` follow-on packet, not the already-landed `R3-5` turn-context packet. Repo docs
and file names for this packet should therefore keep `R3.5` / `r3_5` naming rather than reusing
`R3-5` / `r3-5`.

### Why Next

`R3` proved the new turn-context surface on a real rollout, but that same proof also showed that
operators can still see a replay/live headline mismatch on the same checkpoint even when the
substantive fields match. This is a narrow sentinel-local presentation seam and should close
before session-archetype and progress work so later packets do not build on a known operator-facing
inconsistency.

### Scope

- canonicalize replay/live trigger-headline rendering for matched checkpoints
- keep ordinary checkpoint arrivals canonicalized as `checkpoint_ready`
- preserve analyzer checkpoint state and posture as the truth source for active, recovered, and
  historical-only interpretation
- keep `scheduler_repeated_failure_trigger` available only when the operator surface is actually
  describing a synthetic scheduler fast path rather than an ordinary checkpoint arrival
- keep the fix sentinel-local to replay/live presentation plumbing and focused regression proof
- keep `R3.75`, `R4`, `R5`, `R6`, and `R8` explicitly out of scope for this packet

### Acceptance

- the same matched replay/live checkpoint no longer renders different trigger headlines solely
  because replay derived the trigger from `checkpoint.flagged` while live used the ingress event
  type
- scheduler fast-path events still render distinctly when they are actually emitted
- posture, drift, diagnostics, and turn-context parity stay intact after the headline cutover
- the fix does not widen into analyzer schema changes, analyzer scorer changes, or full sentinel
  interpretation consolidation

## Packet R3.75: Delegation-Aware Analyzer Boundary

### Objective

Add one analyzer-owned delegation boundary that records visible delegation topology and child-work
visibility before later semantic packets harden single-agent assumptions into archetype, progress,
and scorer logic.

### Why After R3.5

`R3` and `R3.5` established the ordinary checkpoint structure and replay/live compatibility, but
delegated sessions are still screened out rather than modeled. The stack needs one narrow
delegation-aware seam before `R4` through `R6` start making deeper semantic claims.

### Scope

- classify ordinary single-agent versus visible delegated-session topology
- record whether child work is visible, partially visible, or opaque from the current checkpoint
- keep the first seam descriptive and confidence-bearing rather than evaluative
- keep `DelegationContext`, `DelegationTopology`, and `ChildWorkVisibility` analyzer-local during
  the first landing rather than widening checkpoint export past `v0.4`
- avoid claiming child intent, child progress, or delegated-specific failure modes in this packet
- treat separate child rollout files / child session ids as a reason to classify parent-visible
  evidence as `partial` or `opaque`, not as permission to invent stitched semantics in `R3.75`
- keep sentinel replay/live compatibility and presentation unchanged in this packet

### Acceptance

- delegated sessions are recognized as delegated rather than silently treated as ordinary
  single-agent sessions
- non-delegated sessions remain unchanged on the existing bounded non-subagent corpus
- later semantic packets have an analyzer-owned boundary they can use to cap confidence or suppress
  over-claims when child work is opaque
- `R7` remains the first packet allowed to add bounded parent/child linkage or supported delegated
  semantics beyond the `R3.75` downgrade boundary

## Packet R4: Session Archetype Classification

Status: landed on the current worktree.

### Objective

Add one analyzer-owned `session_archetype` module.

### Why After R3.75

Turn context provides the raw structure, and the delegation-aware boundary limits over-claiming on
delegated runs. Archetype turns that bounded structure into explicit session meaning.

### Scope

- classify troubleshooting
- classify planning / brainstorming
- classify autonomous implementation
- classify review / closeout

### Acceptance

- each checkpoint has an archetype classification with confidence
- classifications are derived from observable evidence, not commentary alone
- replay/live consumers accept `v0.5` while preserving legacy schema compatibility
- replay/live operator surfaces expose the same compact archetype view for matching checkpoints

## Packet R5: Archetype-Aware Progress Model

### Objective

Add one analyzer-owned `session_progress` module that measures progress relative to archetype.

### Why After R4

Archetype without progress is not enough. The main value comes from saying “this troubleshooting
session is moving the failure frontier” or “this planning session is converging.”

### Scope

- troubleshooting frontier advance / stability
- planning convergence
- implementation verification-wall progress
- review / closeout narrowing

### Acceptance

- progress state is explicit per checkpoint
- troubleshooting can distinguish repeated failure from advancing failure
- planning can distinguish healthy synthesis from meandering discussion

### Landing Status

- `2026-06-10`: `R5` is landed in code and docs.
- analyzer emits `schema_version = "v0.6"` checkpoints with explicit `session_progress`
- replay/live sentinel surfaces render the same compact progress view for matching checkpoints
- the first-cut bounded semantic wall is in place
- the remaining work is follow-on hardening and acceptance deepening, not unfinished `R5`
  surface-area landing

## Packet R5.5: Session Progress Hardening

### Objective

Harden the landed `R5` `session_progress` model before `R6` scorer consumption.

### Why Before R6

`R5` exposed a small number of analyzer-semantic issues that can make progress unsafe scorer input,
especially the troubleshooting repeated-failure overclaim. `R6` must not consume `advancing`
troubleshooting progress until `R5.5` closes that gap.

### Scope

- troubleshooting repeated-failure overclaim fix
- objective extraction hardening
- JS/TS verifier-attempt hardening
- bounded real-rollout corpus deepening
- delegated-parent progress normalization and counter-evidence hygiene
- `R5` doc/code hygiene

### Non-goals

- no public schema widening beyond `v0.6`
- no scorer retuning
- no scheduler or sentinel policy changes
- no full delegated parent/child semantic linkage

### Acceptance

- repeated-failure troubleshooting overclaim regressions are green
- objective extraction prefers the true `/goal` over boilerplate in the bounded repro cases
- JS/TS verifier commands contribute checkpoint-level progress evidence
- the committed real-rollout corpus includes implementation, closeout/review, and reopen/re-verify
  cases
- delegated parent-visible progress is normalized and limiting evidence remains visible
- the stack exits `R5.5` with a stronger baseline, but `R6` stays unopened until the narrower
  `R5.75` follow-on family closes

### Landing Status

- `R5.5` is historically landed on this worktree as the first post-`R5` hardening pass
- its remaining post-validation gaps no longer live as open `R5.5` packet debt
- the active pre-`R6` family is now `R5.75`

## Packet R5.75: Sequential Pre-`R6` Hardening And Validation

### Objective

Close the narrower post-validation gaps that remained after the landed `R5.5` hardening pass,
while keeping `R6` closed until those follow-on issues are test-green and smoke-proven.

### Why Before `R6`

Validation after `R5.5` showed the repo no longer needed a broad open-ended hardening family, but
it still needed a smaller sequence of fixes before scorer cutover could be called honest.

### Scope

- reconcile stale `R5.5` landed-vs-remaining authority wording
- objective condensation / target extraction for giant pasted prompts
- sparse readable-session fail-open instead of analyzer hard-abort
- delegated parent-visible stabilization under the known repro sessions
- zero-verifier anti-flap gating for long exploratory sessions
- adapted external robustness fixtures as secondary pre-`R6` evidence

### Acceptance

- landed `R5.5` work is not still presented as open implementation debt
- the active pre-`R6` authority stack consistently names `R5.75` as current and keeps `R6` closed
- the narrower objective, sparse-session, delegated, anti-flap, and adapted-fixture gaps are
  closed under the `R5.75` map

## Packet R6: Drift Scorer Cutover To Context-Aware Semantics

> **Rescoped (2026-06-27):** `R5.75` is complete, so `R6` is now the active seam and has been
> rescoped against the structured-objective work that landed inside `R5.75`. The authoritative `R6`
> map, the resolved `Decision Gate 0` (objective-consumption boundary), and the packet decomposition
> now live in `docs/specs/r6/MAP.md` and
> `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`. The original intent below is
> preserved and carried forward by that rescope.

### Objective

Re-score `dead_end_thrash` and related drift classes using typed outcome evidence, turn context,
archetype, and progress modules.

### Why After R5.75

This is where the earlier packets finally pay off. The scorers should become consumers of deeper
analyzer modules rather than home-grown heuristic islands, but only after the landed `R5` progress
layer has passed the landed `R5.5` baseline and the narrower `R5.75` follow-on gate.

### Scope

- `dead_end_thrash` cutover first
- revisit `truth_grounding_gap` only if the new context exposes obvious opportunities
- keep packet scope narrow; do not broaden into scheduler tuning here

### Acceptance

- troubleshooting sessions tolerate expected failures when the frontier advances
- long autonomous turns are evaluated differently from multi-turn conversational sessions
- flagged sessions become materially more honest on known replay artifacts

**Current proof posture:** the first and third claims are proven only at the bounded scopes recorded
in `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. The long-autonomous versus
multi-turn-conversational claim has construction-level proof but lacks a scorer-level behavioral A/B.
R6 remains partial until `R6-C.1` supplies that proof or narrows the wording honestly.

## Packet R7: Full Delegated-Session Support

### Objective

Extend the analyzer from delegation-aware boundary detection into supported delegated-session
semantics so parent-visible orchestration and child-visible work can be interpreted honestly before
sentinel interpretation is consolidated.

### Why After R6

`R3.75` only records delegation topology and visibility limits. `R4` through `R6` then make the
ordinary single-session semantics explicit and honest. Full delegated-session support should come
only after those ordinary semantics are stable, otherwise the repo risks mixing baseline semantic
uncertainty with delegation-specific complexity.

### Scope

- define supported delegated-session semantics beyond mere topology detection
- distinguish parent orchestration activity from child execution activity
- allow analyzer modules to describe parent-visible progress separately from child-visible progress
  when child evidence is available
- allow bounded parent/child linkage when delegated work is recorded in separate ordinary
  `rollout-*.jsonl` files with distinct child session ids
- add bounded delegated-session regression coverage for supported cases
- keep the packet analyzer-owned; do not broaden into sentinel interpretation consolidation here

### Acceptance

- delegated sessions are no longer only "detected and downgraded"; at least a bounded supported
  subset receives explicit semantic handling
- parent waiting or orchestration is not misread as direct child implementation progress
- delegated-session drift semantics are materially more honest on known delegated artifacts than the
  `R3.75` boundary alone
- `R8` can consume one stabilized analyzer semantic seam rather than inventing delegated-session
  interpretation itself

## Packet R8: Sentinel Interpretation Consolidation

### Objective

Collapse replay/live checkpoint interpretation duplication in sentinel.

### Why Last

This improves locality and maintainability, but it should not block the analyzer semantic fix or
the narrower `R3.5` replay/live trigger-headline cutover.

### Scope

- replay input
- live input
- operator-surface interpretation helpers

### Acceptance

- replay and live paths share one checkpoint-interpretation seam
- operator-surface code stays presentation-first
- compatibility logic is centralized

## Things That Should Not Happen Again

- Do not re-open “checkpoint-state ownership” as if it were the top missing slice.
- Do not tune sentinel posture first when analyzer evidence is still wrong.
- Do not evaluate future drift semantics only through synthetic fixtures.
- Do not build archetype-aware heuristics on top of generic `ToolOutput == failure`.
- Do not confuse session-level summary metrics with per-checkpoint context modules; both are
  useful, but they solve different problems.

## Immediate Next Action

`R3.5`, `R3.75`, `R4`, `R5`, `R5.5`, and `R5.75` are landed. The scoped R6 packet history is also
landed, but the broad R6 scorer-context charter is **PARTIAL / CLOSURE AUDIT REQUIRED**. The next
authority is `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`.

The next honest work target is:

- keep `R3` closed as the completed turn-context packet family
- keep `R3.5` closed as the completed replay/live trigger-headline canonicalization packet
- keep `R3.75` closed as the completed delegation-aware analyzer boundary
- keep `R4` closed as the completed session-archetype packet family
- keep `R5` closed as the landed archetype-aware progress packet family
- keep `R5.5` closed as the landed first hardening pass
- keep `R5.75` closed as the landed pre-`R6` hardening/validation family
- complete only the narrow `R6-C.1` acceptance-first controls named by the closure finding; do not
  inject every context layer into every scorer and do not reopen `semantic_goal_drift` without new
  failing evidence
- close R6 only after every material scorer is complete, intentionally exempt, or still open in a
  named bounded packet and the broad acceptance wording is proven or narrowed honestly
- preserve full delegated-session support as design-ready R7 draft work, blocked on an R6 `CLOSED`
  decision; do not begin R7 implementation or make it absorb ordinary single-session scorer gaps
- keep sentinel interpretation consolidation as `R8` behind the analyzer semantic packets

Commit `99efda8f9` remains in history as draft planning work; it is not R6 closure authority.

## Research-Informed Design Directions

This section collects external papers that appear directly relevant to the remaining hybrid-drift
packets. These are not treated as repo authority. They are here to provide concrete patterns,
taxonomies, and evaluation ideas that can be revisited during planning and design.

## How To Use This Section

Use these papers as design input for:

- naming additional analyzer modules
- shaping packet-level acceptance criteria
- deciding whether a heuristic should stay handwritten or evolve into a learned monitor
- avoiding reinvention when similar agent-trajectory problems have already been studied elsewhere

Do not cargo-cult any one framework. The value here is pattern extraction.

## Packet R3: Per-Checkpoint Turn Context

### Primary Design Pattern

Promote turn shape and turn-local execution state into explicit checkpoint fields, rather than
leaving them as summary-only aggregates.

### Papers

- `AgentLens: Revealing The Lucky Pass Problem in SWE-Agent Evaluation`
  - Link: <https://huggingface.co/papers/2605.12925>
  - Useful pattern:
    - classify trajectory behavior by recent history, not only by tool identity
  - Why it matters here:
    - this maps closely to the missing turn-context seam in hybrid drift
    - the analyzer already exports enough coarse metrics to support a coarse checkpoint-local
      activity mix and execution-mode layer
  - Deliberate `R3` boundary:
    - do not make `Exploration` / `Implementation` / `Verification` / `Orchestration` per-action
      stage labels the canonical `R3` artifact
    - keep AgentLens influence limited to observational turn-local structure, not evaluative or
      task-level scoring

- `Large Language Models as Zero-shot Dialogue State Tracker through Function Calling`
  - Link: <https://huggingface.co/papers/2402.10466>
  - Useful pattern:
    - keep an explicit evolving state object for the conversation
  - Why it matters here:
    - hybrid drift should likely maintain structured per-checkpoint turn state rather than
      repeatedly infer turn meaning from raw rows
  - Deliberate `R3` boundary:
    - do not adopt function-calling or LLM extraction as the state-construction mechanism
    - keep the state deterministic and analyzer-computed from landed telemetry

- `Interpretable and Robust Dialogue State Tracking via Natural Language Summarization with LLMs`
  - Link: <https://huggingface.co/papers/2503.08857>
  - Useful pattern:
    - preserve an interpretable human-readable state representation alongside structured fields
  - Why it matters here:
    - this could inspire an optional checkpoint-level state summary for debugging and operator
      review, without replacing machine-readable fields
  - Deliberate `R3` boundary:
    - any prose summary stays derived and presentation-only, never the canonical checkpoint
      contract

### Concrete Hybrid-Drift Ideas

- add per-checkpoint fields such as:
  - `turn_ordinal`
  - `seconds_since_turn_start`
  - `rows_since_turn_start`
  - `checkpoints_in_turn`
  - `turn_activity_mix`
  - `turn_execution_mode`
- keep these analyzer-owned and export them with checkpoint artifacts

## Packet R4: Session Archetype Classification

### Primary Design Pattern

Treat session type as explicit state with confidence, not as an implicit assumption buried inside a
single drift scorer.

### Papers

- `AgentLens: Revealing The Lucky Pass Problem in SWE-Agent Evaluation`
  - Link: <https://huggingface.co/papers/2605.12925>
  - Useful pattern:
    - assign context-sensitive intent labels from trajectory structure
  - Why it matters here:
    - this is the closest external analogue to the proposed `session_archetype` seam

- `Where LLM Agents Fail and How They can Learn From Failures`
  - Link: <https://huggingface.co/papers/2509.25370>
  - Useful pattern:
    - use a modular failure taxonomy spanning planning, memory, reflection, action, and system
      operations
  - Why it matters here:
    - session archetype and failure taxonomy should compose cleanly; hybrid drift may eventually
      want more than the current three drift classes

- `Exploring Autonomous Agents: A Closer Look at Why They Fail When Completing Tasks`
  - Link: <https://huggingface.co/papers/2508.13143>
  - Useful pattern:
    - align failure categories with task phase instead of using only outcome labels
  - Why it matters here:
    - hybrid drift should likely classify not just “what drift happened” but “during which mode of
      work it happened”

### Concrete Hybrid-Drift Ideas

- initial archetypes:
  - `troubleshooting`
  - `planning`
  - `autonomous_implementation`
  - `review_closeout`
- make the classifier additive and confidence-bearing
- derive it from observable signals:
  - kickoff prompt shape
  - truth artifact density
  - verification density
  - write/test cadence
  - turn-local checkpoint density

## Packet R5: Archetype-Aware Progress Model

### Primary Design Pattern

Measure progress relative to task mode. Repetition alone is too weak.

### Papers

- `Step-level Optimization for Efficient Computer-use Agents`
  - Link: <https://huggingface.co/papers/2604.27151>
  - Useful pattern:
    - separate a `Stuck Monitor` from a `Milestone Monitor`
  - Why it matters here:
    - hybrid drift should likely separate “progress stall” from “semantic drift from plan”

- `TRAJEVAL: Decomposing Code Agent Trajectories for Fine-Grained Diagnosis`
  - Link: <https://huggingface.co/papers/2603.24631>
  - Useful pattern:
    - decompose long trajectories into interpretable stages and score each stage separately
  - Why it matters here:
    - a progress model could track stage-local success rather than collapsing all motion into one
      `dead_end_thrash` heuristic

- `Language Server CLI Empowers Language Agents with Process Rewards`
  - Link: <https://huggingface.co/papers/2510.22907>
  - Useful pattern:
    - use diagnostic deltas and machine-checked facts as process reward
  - Why it matters here:
    - this is strong support for using “failure frontier moved” or “diagnostics improved” as a
      positive signal in troubleshooting sessions

- `Thinking Longer, Not Larger: Enhancing Software Engineering Agents via Scaling Test-Time Compute`
  - Link: <https://huggingface.co/papers/2503.23803>
  - Useful pattern:
    - allocate extra reasoning at critical development decision points instead of uniformly
  - Why it matters here:
    - hybrid drift may want to be stricter around stage transitions and milestone checkpoints than
      during routine execution inside one stable turn

### Concrete Hybrid-Drift Ideas

- troubleshooting progress:
  - same verification command, but later failure frontier
  - fewer failing tests
  - compile failure turns into assertion failure
  - edits overlap the failing scope between retries
- planning progress:
  - candidate set narrows
  - open questions shrink
  - more explicit artifact production
- implementation progress:
  - working set narrows
  - verification wall moves forward
  - expected next step becomes more concrete

## Packet R6: Drift Scorer Cutover To Context-Aware Semantics

### Primary Design Pattern

Use explicit context modules and validated failure hypotheses to score drift, rather than relying on
one coarse trajectory symptom.

### Papers

- `AgentRx: Diagnosing AI Agent Failures from Execution Trajectories`
  - Link: <https://huggingface.co/papers/2602.02475>
  - Useful pattern:
    - localize the critical failure step using auditable validation logs
  - Why it matters here:
    - hybrid drift should move toward “what was the decisive bad step?” instead of only “how long
      was the flagged streak?”

- `DoVer: Intervention-Driven Auto Debugging for LLM Multi-Agent Systems`
  - Link: <https://huggingface.co/papers/2512.06749>
  - Useful pattern:
    - validate failure hypotheses through targeted interventions, not only log reading
  - Why it matters here:
    - replay-time or fixture-time counterfactual checks could help validate whether a checkpoint
      really reflects thrash or just expected debugging motion

- `SWE-Shepherd: Advancing PRMs for Reinforcing Code Agents`
  - Link: <https://huggingface.co/papers/2604.10493>
  - Useful pattern:
    - dense step-level supervision for repository agents
  - Why it matters here:
    - if hybrid drift eventually wants learned scoring, this is closer to the right granularity
      than outcome-only supervision

- `Verbal Process Supervision Elicits Better Coding Agents`
  - Link: <https://huggingface.co/papers/2503.18494>
  - Useful pattern:
    - use process-level supervision signals rather than only end-state signals
  - Why it matters here:
    - strengthens the case that checkpoint-level drift semantics should be built from process
      evidence, not only final pass/fail outcomes

### Concrete Hybrid-Drift Ideas

- cut over `dead_end_thrash` first
- keep `truth_grounding_gap` unchanged unless the new context surfaces an obvious improvement
- explicitly score:
  - `stall without frontier movement`
  - `semantic drift from kickoff/plan/docs`
  - `expected debugging churn with positive progress`
- keep the first version rule-based and interpretable
- consider learned or hybrid monitors only after the rule-based signals and acceptance wall are
  stable

## Cross-Cutting Evaluation And Trace Design References

These papers are useful across multiple packets because they argue for trajectory-first evaluation
and richer process artifacts.

- `Reproducible, Explainable, and Effective Evaluations of Agentic AI for Software Engineering`
  - Link: <https://huggingface.co/papers/2604.01437>
  - Relevance:
    - argues for publishing and comparing full Thought-Action-Result style trajectories
    - supports the current hybrid-drift direction of grounding decisions in replay artifacts rather
      than final outcomes alone

- `A Trace-Based Assurance Framework for Agentic AI Orchestration: Contracts, Testing, and Governance`
  - Link: <https://huggingface.co/papers/2603.18096>
  - Relevance:
    - emphasizes trace contracts, first-violating-step localization, and deterministic replay
    - aligns well with Substrate’s existing trace-oriented architecture

- `Reliable Weak-to-Strong Monitoring of LLM Agents`
  - Link: <https://huggingface.co/papers/2508.19461>
  - Relevance:
    - useful if the sentinel later evolves into a hierarchical monitor, where cheap analyzers do
      broad filtering and expensive reasoning is only used for escalations

## Summary Of The Strongest External Patterns

The most promising external patterns for hybrid drift are:

- context-sensitive intent labeling from trajectory history, not tool identity alone
- separate monitors for stall versus drift
- stage-aware trajectory decomposition
- explicit per-checkpoint state tracking
- failure-hypothesis validation instead of direct drift guessing
- process rewards based on machine-checked diagnostic deltas

If later packets need one concise research starting point, begin with:

1. `AgentLens` for intent labels and process-quality framing
2. `Step-level Optimization for Efficient Computer-use Agents` for stall vs milestone monitoring
3. `TRAJEVAL` for stage decomposition
4. `AgentRx` for decisive-failure-step localization
5. `Lanser-CLI` for process rewards from diagnostic deltas
