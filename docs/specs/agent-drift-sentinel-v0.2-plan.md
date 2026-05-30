# Plan: Agent Drift Sentinel v0.2

## Scope

This plan implements [agent-drift-sentinel-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-v0.2-spec.md:1).

The goal is to deliver a library-first crate with a thin CLI or sidecar that:

- consumes analyzer checkpoints
- evaluates replayed or live streams on explicit triggers
- optionally asks a small model to adjudicate bounded ambiguous cases
- emits operator-facing warnings without overwhelming the operator

This module is intentionally downstream of the compactor and analyzer and should not be built until
their seams are trustworthy.
Replay-mode validation should assume freshly regenerated current-schema analyzer bundles rather than
older dev/testing artifacts unless those artifacts are explicitly regenerated against the current
contract.

## Implementation Strategy

Build replay mode first, then live mode.

Implementation order:

1. load analyzer checkpoint artifacts
2. define scheduler state and trigger policy
3. render operator-facing replay output
4. add optional adjudication
5. only then explore live ingestion and runtime integration

This preserves locality: the sentinel remains a thin orchestration layer over analyzer semantics.

## Major Components

### 1. Replay Input Surface

Deliver first:

- checkpoint loading
- replay-window loading if needed
- stable checkpoint ordering and cursoring
- explicit rejection or non-use of stale dev/testing bundles that do not match the current analyzer
  contract

Why first:

- replay mode is the safest place to validate operator value before adding live complexity

### 2. Scheduler

Deliver second:

- trigger classes
- cooldown and debounce policy
- heartbeat policy
- repeated-failure fast-path handling

Goal:

- avoid wall-clock spam and warning floods

### 3. Operator Surface

Deliver third:

- concise warning summaries
- evidence excerpts or refs
- expected next step
- non-warning checkpoint handling

Why third:

- operator usefulness is the main product value of the sentinel

### 4. Optional Model Adjudication

Deliver fourth:

- bounded request shaping
- `gpt-5.4-mini`
- `medium` reasoning effort
- safe fallback to analyzer-only output

Critical rule:

- model output is an optional sharpening layer, never the sole explanation path

### 5. Live Mode

Deliver last:

- live trigger wiring
- live checkpoint ingestion
- later runtime integration hooks

This should remain explicitly deferred until replay mode proves useful.

## Sequencing

Sequential work:

1. replay input loading
2. scheduler
3. operator surface
4. adjudication
5. live mode

Parallel-safe work after replay input stabilizes:

- scheduler policy and operator rendering can be developed in parallel
- adjudication request shaping can be developed in parallel with replay-mode summary tests
- live-mode exploration can run in parallel with replay hardening only after replay output is
  judged useful

## Risks And Mitigations

### Risk 1: Warning fatigue

Risk:

- the sentinel could overfire and train operators to ignore it

Mitigation:

- replay-first validation
- cooldown and debounce rules
- clear distinction between silent checkpoints and visible warnings

### Risk 2: Sentinel duplicates analyzer logic

Risk:

- live integration may tempt the sentinel to re-score or reinterpret analyzer semantics

Mitigation:

- keep replay input and checkpoint contracts primary
- do not re-own drift logic in the sentinel crate

### Risk 3: Model adjudication becomes a crutch

Risk:

- ambiguous warning cases could drift into model-first behavior

Mitigation:

- adjudication disabled by default
- bounded inputs only
- safe degradation to analyzer-only output

### Risk 4: Live integration arrives too early

Risk:

- runtime wiring could consume time before the historical path is trustworthy

Mitigation:

- replay mode is a hard prerequisite
- live mode explicitly comes last in the sequencing

### Risk 5: Replay validation uses stale artifacts and creates false failures

Risk:

- older dev/testing bundles may not match the current compactor/analyzer schema and could be
  mistaken for sentinel regressions

Mitigation:

- use freshly regenerated current-schema bundles for replay validation
- treat stale artifact failures as input-quality problems unless backward compatibility is explicitly
  in scope

## Verification Checkpoints

### Checkpoint A: Replay Consumption Stable

Verify:

- the sentinel can consume analyzer checkpoints deterministically

Evidence:

- replay-mode tests against fixture checkpoint bundles
- replay validation notes that use freshly regenerated current-schema analyzer bundles rather than
  older dev/testing leftovers

### Checkpoint B: Scheduler Useful

Verify:

- cooldown, debounce, and trigger behavior reduce noise rather than add it

Evidence:

- scheduler tests
- replay traces that simulate repeated failure and normal progress

### Checkpoint C: Operator Surface Useful

Verify:

- warnings are concise, evidence-backed, and not overfired

Evidence:

- golden-output style tests
- human review of replay summaries

### Checkpoint D: Adjudication Safe

Verify:

- adjudication is optional, bounded, and degrades safely

Evidence:

- model-off default tests
- model-failure fallback tests

### Checkpoint E: Live Mode Ready

Verify:

- live-mode work begins only after replay mode is stable and useful

Evidence:

- explicit review gate, not just code readiness

## Handoff To Later Runtime Integration

This plan is complete enough for broader Substrate integration only when all of the following are
true:

- replay-mode operator value is proven
- trigger policy is conservative and understandable
- adjudication remains bounded and optional
- the sentinel binary is still thin enough to embed later as an internal library seam

## Exit Criteria

The plan is complete when:

1. the crate builds and tests cleanly
2. replay mode provides stable, reviewable operator output
3. scheduler behavior is explicit and not warning-spam-driven
4. optional adjudication is safe and non-essential
5. live mode is either validated as the next step or consciously deferred
