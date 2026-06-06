# Hybrid Drift Remaining Gaps And Landing Order

## Why This Document Exists

The hybrid-drift stack has already landed a meaningful checkpoint-state cutover, but the review
loop kept re-suggesting that same slice as if it were still open. This document corrects that. It
records:

- what is already landed
- what is still missing
- which gap is actually blocking honest analyzer and sentinel behavior now
- the order the remaining work should land in

This file is intended to be the repo-root authority for the current follow-on sequence after
`v0.6A` / `v0.6B`.

## Ground Truth Sources

- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
- `.codex/handoffs/2026-06-04-180058-drift-sentinel-rollout-review.md`
- `target/hybrid-drift-evals/*/analyzer/summary.md`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`

## Current Status

### What Is Already Landed

The following slice is landed and should not be re-suggested as the top fix for sticky
`dead_end_thrash` in this worktree:

- analyzer-owned per-class `DriftState`
- checkpoint schema widening from `v0.2` to `v0.3`
- sentinel cutover to prefer analyzer-exported state for `v0.3`
- isolated `v0.2` replay/live compatibility fallback

That means the previous recommendation to “deepen the checkpoint-state module” is now closed as a
top-level architecture ask. The analyzer already exports state, and the sentinel already consumes
it for `v0.3`.

### What Is Still Broken

Semantic honesty is still wrong on real completed sessions.

The current analyzer still builds repeated-failure history from generic `ToolOutput` rows:

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `repetition_slice(...)`
  - `is_failure_row(...)`

That polluted failure surface feeds `dead_end_thrash`:

- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`

As a result, successful function-call output, bookkeeping output, and real failures still share
one shallow seam. The state module is landed, but it is being fed bad evidence.

### Why The Current Stack Still Misfires

The remaining failure is upstream of sentinel posture.

Today:

1. raw rows are classified too coarsely
2. repeated-failure loops are built from that coarse classification
3. recovery is evaluated against those loops
4. `dead_end_thrash` consumes the polluted loop history
5. sentinel correctly renders the analyzer state it receives

So the current wrong answer is not “sentinel posture logic is still missing.” The current wrong
answer is “the analyzer is still constructing the wrong state from the wrong evidence surface.”

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

## Gap 1: Explicit Outcome Evidence Is Missing

### Problem

The analyzer still treats generic `ToolOutput` as failure evidence. That is too shallow. The
interface between compactor rows and analyzer evidence does not distinguish:

- explicit failure
- explicit success
- neutral output
- bookkeeping output
- operator milestone output

### Why This Matters

Without this distinction:

- repeated successful tool outputs can manufacture repeated failure history
- recovery cannot honestly clear because the failure history is fake
- every later heuristic inherits a polluted foundation

### What Needs To Exist

One analyzer-owned evidence-classification module that turns raw rows into explicit outcome
evidence.

Minimum honest logic:

- `Error` rows count as failure evidence
- generic `ToolOutput` is neutral by default
- a `ToolOutput` row only counts as failure if it contains a trusted structured failure signal
- `Exit code: 0`, plan updates, bookkeeping text, and generic function-call output stay neutral
- ambiguous rows bias neutral, not failure

This is intentionally conservative. False positives are more damaging here than false negatives.

## Gap 2: `dead_end_thrash` Still Reads The Wrong Surface

### Problem

`dead_end_thrash` still consumes repeated-failure loops built from the coarse evidence seam. That
means the scorer is not actually reasoning about “repeated failure”; it is reasoning about
“repeated rows that happen to have kind `ToolOutput` or `Error`.”

### Why This Matters

Even if the exported `DriftState` logic is correct, the input to that state is still impure. The
scorer should only observe:

- repeated verification attempts
- repeated explicit failures
- clean recovery intervals

It should not observe:

- generic stdout/stderr text
- bookkeeping output
- reason-string conventions as the source of truth

### What Needs To Exist

`dead_end_thrash` should score from typed outcome evidence, not raw row kinds.

## Gap 3: The Stack Lacks Turn-Aware Context

### Problem

The analyzer summary already knows the session shape, but the current per-checkpoint model does not
make that shape first-class.

Right now the sentinel does not directly know:

- whether it is five minutes into one long agent turn
- whether it is inside a rapid user/agent back-and-forth
- whether several checkpoints happened within one user prompt
- whether the current turn is tool-heavy, verification-heavy, or commentary-heavy

### Why This Matters

The same `dead_end_thrash` pattern means different things in different turn contexts.

For example:

- eight checkpoints inside one user prompt can be a normal autonomous implementation run
- eight checkpoints across eight user prompts mean something very different

### What Needs To Exist

One analyzer-owned per-checkpoint turn-context module that emits fields such as:

- turn ordinal
- seconds since turn start
- rows since turn start
- checkpoints emitted in current turn
- prompts observed in current session
- turn execution mode
- turn activity mix

This should become a real module, not just a summary-only export.

## Gap 4: The Stack Does Not Identify Session Archetype

### Problem

The current stack largely treats every session as if repeated failure means the same thing. It does
not explicitly distinguish:

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

### What Needs To Exist

One analyzer-owned `session_archetype` module that classifies checkpoints using observable
evidence:

- kickoff prompt shape
- truth artifact usage
- tool mix
- write/test cadence
- user prompt cadence
- working-set concentration
- verification density

The classification should be additive and confidence-bearing. It does not need to be perfect on
day one, but it must be explicit.

## Gap 5: The Stack Does Not Model Progress Relative To Archetype

### Problem

Even after archetype is identified, the stack still needs to ask whether the session is advancing
inside that archetype. Right now repetition is over-weighted, while progress is under-modeled.

### Why This Matters

A troubleshooting session can fail the same command repeatedly while still making progress.

What matters is not “same command failed again.” What matters is whether the failure frontier is
moving.

### What Needs To Exist

One analyzer-owned `session_progress` module.

For troubleshooting, it should capture signals such as:

- identical verification command repeated
- failing frontier stable vs advancing
- failure class narrowing
- earlier failure replaced by later failure
- compile failure becoming test failure
- many failing tests becoming fewer failing tests
- edits overlapping the failing scope between retries

For planning, it should capture:

- candidate set narrowing
- objective sharpening
- artifact creation
- unresolved questions shrinking

For autonomous implementation, it should capture:

- commentary alignment with kickoff prompt
- verification wall progress
- working-set concentration
- expected-next-step stability or narrowing

## Gap 6: Real Rollout Acceptance Is Too Weak

### Problem

The current tests prove a lot of contract plumbing, but the review showed that green tests were not
enough to prove semantic honesty on real completed sessions.

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

## Gap 7: Sentinel Still Pays Duplicate Interpretation Cost

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
- session archetype classification such as troubleshooting vs planning vs review / closeout
  - falls under `R4`
- archetype-aware progress semantics such as frontier movement, narrowing candidate sets, or
  verification-wall advance
  - falls under `R5`
- broader drift-scorer redesign that combines typed outcome evidence with turn context, archetype,
  and progress modules
  - falls under `R6`
- broader replay/live checkpoint interpretation redesign inside sentinel
  - falls under `R7`

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
    `dead_end_thrash.state=recovered`, `raw_score=20`, and `flagged=false` in
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

### Acceptance

- checkpoints can distinguish “one long agent-side turn” from “many short conversational turns”
- replay/live consumers can inspect turn-local context directly

## Packet R4: Session Archetype Classification

### Objective

Add one analyzer-owned `session_archetype` module.

### Why Fourth

Turn context provides the raw structure; archetype turns that structure into meaning.

### Scope

- classify troubleshooting
- classify planning / brainstorming
- classify autonomous implementation
- classify review / closeout

### Acceptance

- each checkpoint has an archetype classification with confidence
- classifications are derived from observable evidence, not commentary alone

## Packet R5: Archetype-Aware Progress Model

### Objective

Add one analyzer-owned `session_progress` module that measures progress relative to archetype.

### Why Fifth

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

## Packet R6: Drift Scorer Cutover To Context-Aware Semantics

### Objective

Re-score `dead_end_thrash` and related drift classes using typed outcome evidence, turn context,
archetype, and progress modules.

### Why Sixth

This is where the earlier packets finally pay off. The scorers should become consumers of deeper
analyzer modules rather than home-grown heuristic islands.

### Scope

- `dead_end_thrash` cutover first
- revisit `truth_grounding_gap` only if the new context exposes obvious opportunities
- keep packet scope narrow; do not broaden into scheduler tuning here

### Acceptance

- troubleshooting sessions tolerate expected failures when the frontier advances
- long autonomous turns are evaluated differently from multi-turn conversational sessions
- flagged sessions become materially more honest on known replay artifacts

## Packet R7: Sentinel Interpretation Consolidation

### Objective

Collapse replay/live checkpoint interpretation duplication in sentinel.

### Why Last

This improves locality and maintainability, but it should not block the analyzer semantic fix.

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

If only one packet lands next, it should be `R1A`.

The next honest implementation target is:

- explicit outcome evidence classification
- repeated-failure cutover to that narrower evidence surface
- focused analyzer tests first, with bounded replay proof deferred to `R1C`

That is the current blocking gap. Everything else in this document should follow from there.

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
    - classify actions by trajectory history, not only by tool identity
    - separate `Exploration`, `Implementation`, `Verification`, and `Orchestration`
  - Why it matters here:
    - this maps closely to the missing turn-context seam in hybrid drift
    - the analyzer already exports enough coarse metrics to support a similar intent layer

- `Large Language Models as Zero-shot Dialogue State Tracker through Function Calling`
  - Link: <https://huggingface.co/papers/2402.10466>
  - Useful pattern:
    - keep an explicit evolving state object for the conversation
  - Why it matters here:
    - hybrid drift should likely maintain structured per-checkpoint turn state rather than
      repeatedly infer turn meaning from raw rows

- `Interpretable and Robust Dialogue State Tracking via Natural Language Summarization with LLMs`
  - Link: <https://huggingface.co/papers/2503.08857>
  - Useful pattern:
    - preserve an interpretable human-readable state representation alongside structured fields
  - Why it matters here:
    - this could inspire an optional checkpoint-level state summary for debugging and operator
      review, without replacing machine-readable fields

### Concrete Hybrid-Drift Ideas

- add per-checkpoint fields such as:
  - `turn_ordinal`
  - `seconds_since_turn_start`
  - `rows_since_turn_start`
  - `checkpoints_in_turn`
  - `turn_activity_mix`
  - `turn_intent_stage`
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
