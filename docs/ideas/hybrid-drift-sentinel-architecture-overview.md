# Hybrid Drift Sentinel Architecture Overview

## Purpose

This document is the umbrella design for the Hybrid Drift Sentinel effort.

It reconciles two earlier directions:

- `docs/ideas/hybrid-drift-sentinel-v0.1.md`
  - strong product framing for task-frame-based drift detection
- `docs/specs/hybrid-drift-sentinel-v0.1-spec.md`
  - strong implementation framing for deterministic session ingestion and compaction

The earlier docs overlap, but they do not describe the same module. This document defines the
big-picture shape so later work can split cleanly into separate design docs without mixing seams.

## Problem

We want a cross-repo system that can inspect long Codex work, infer the agent's current task frame
from observable evidence, and surface likely drift without requiring a known active plan file.

The first user is the human operator reviewing or watching a long run. That means the system must
optimize for:

- evidence before authority
- interpretability before autonomy
- deterministic auditability before model-heavy reasoning
- cross-repo utility without repo-specific planning conventions

## Core Product Principle

The semantic center is the `task frame`, not the `active plan`.

The system should ask:

- what is the user trying to get done
- what working set is the agent actually operating in
- which artifacts appear to govern the task
- does the current trajectory still fit the best available task-frame hypothesis

This keeps the design useful in repos with no stable plan doc and prevents the system from baking
false certainty into plan reconstruction.

## Why The Earlier Combined Spec Was Hard To Land

The earlier combined spec tried to describe two different responsibilities as one crate:

1. transcript compaction
2. drift inference and checkpointing

Those are related, but they are not the same module.

The main failure mode is shallow interfaces:

- a compactor wants archival fidelity, canonicalization, exact dedupe, and export
- a drift analyzer wants session-scoped evidence, repetition signals, thresholds, and checkpoints
- a live sentinel wants scheduling, escalation, and operator-facing warnings

When all three are forced into one module, locality gets worse and the interfaces start to leak.

## Recommended Architecture

Build this as a small stack of deep modules with explicit seams.

### 1. Parser And Ingestion Surfaces

Owned crates already provide the right ingestion seam:

- `unified-agent-api-wrapper-events`
- `unified-agent-api-codex`

These crates should own:

- bounded line ingestion
- rollout JSONL parsing
- wrapper JSONL / NDJSON parsing
- transport-specific record classification
- parser errors and raw-capture posture

They should not own compaction semantics or drift semantics.

If real-session pressure testing shows that `unified-agent-api-wrapper-events` or
`unified-agent-api-codex` is missing an important parser surface or exposing the wrong seam, pause
and plan the parser-crate change directly instead of cementing a downstream workaround.

### 2. `agent-session-compactor`

This is the first implementation module.

Product shape:

- library-first crate
- thin CLI binary for standalone pressure testing, Codex hook integration, and real-session usage
- later eligible to be consumed as an internal library inside the broader Substrate system without
  changing the core module contract

Interface:

- input:
  - Codex session artifacts from a resolved Codex home
  - optional wrapper-style event streams later
- output:
  - archival normalized rows
  - compact rows
  - dedupe audit
  - manifest and operator-readable summary

Responsibilities:

- discover session artifacts
- reuse owned parser surfaces instead of raw JSON traversal by default
- normalize records into a stable row model with provenance
- canonicalize text deterministically
- fold exact duplicates with stable keep-first semantics
- export reviewable file-backed artifacts

Non-responsibilities:

- infer task frame
- score drift classes
- schedule evaluations
- steer or notify a live agent

### 3. `agent-drift-analyzer`

This is the second implementation module.

Interface:

- input:
  - one session at a time
  - archival or analysis-safe derived rows from the compactor
  - compacted views only where compaction does not destroy needed evidence
- output:
  - deterministic checkpoints
  - task-frame hypotheses
  - drift scores
  - evidence references

Responsibilities:

- infer the current `task frame`
- assemble candidate truth artifacts and observed working set
- score exactly three initial drift classes:
  - `wrong_plan_branch`
  - `ignoring_repo_truth`
  - `dead_end_thrash`
- emit evidence-backed checkpoints

Critical design rule:

- `dead_end_thrash` must read a repetition-preserving evidence surface
- exact-deduped rows alone are not sufficient for thrash detection

Non-responsibilities:

- parse raw Codex files directly
- own live scheduling
- own operator UI

### 4. `agent-drift-sentinel`

This is the live or replay-side integration module.

Interface:

- input:
  - live event stream or replayed event stream
  - analyzer checkpoints
- output:
  - scheduled evaluations
  - operator-facing warnings or logs
  - optional adjudication output

Responsibilities:

- trigger scheduling
- live evaluation cadence
- optional small-model adjudication
- operator surface integration

This module should come after the deterministic historical path is trustworthy.

## Data Flow

```text
~/.codex/sessions/**
  -> unified-agent-api-codex / unified-agent-api-wrapper-events
  -> agent-session-compactor
  -> normalized archival rows + compact rows + dedupe audit
  -> agent-drift-analyzer
  -> checkpoints.jsonl
  -> agent-drift-sentinel
  -> operator warnings / timeline / later runtime integration
```

## Module Contracts

### Compactor Contract

The compactor is the transcript-to-row module.

It should own:

- source discovery
- source ordering
- provenance-preserving row construction
- canonical text hashing
- exact dedupe groups
- immutable source posture

It should not collapse into an inference module.

### Analyzer Contract

The analyzer is the row-to-checkpoint module.

It should own:

- session-scoped analysis
- task-frame inference
- working-set assembly
- drift thresholds
- evidence selection
- checkpoint schema

It should not depend on raw source layout details beyond what the row schema exposes.

### Sentinel Contract

The sentinel is the checkpoint-to-operator module.

It should own:

- when to evaluate
- whether to escalate
- how to present warnings
- later live/runtime integration

It should not redefine transcript parsing or row semantics.

## Deterministic-First Policy

The first trustworthy version should be deterministic by default.

Lock in these rules:

- use owned parser crates before bespoke parsing
- preserve source provenance for every derived row
- keep source session files immutable
- exact dedupe only in the baseline
- no semantic dedupe in archival outputs
- no model output as the sole basis for drift decisions

Optional model adjudication can arrive later as a secondary seam over bounded analyzer inputs.

## v0.1 Scope Recommendation

Treat v0.1 as two stacked historical modules, not a live sentinel:

1. `agent-session-compactor`
2. `agent-drift-analyzer`

This is the smallest useful shape that still proves the real product direction:

- we can normalize real Codex sessions
- we can preserve enough evidence to infer task frame
- we can score the three drift classes deterministically
- we can produce reviewable checkpoints for human evaluation

Do not make live scheduling a requirement for the first proof.

## Staged Delivery Plan

### Stage 1: Deterministic Compactor

Goal:

- prove transcript ingestion, normalization, canonicalization, and exact dedupe over real Codex
  session artifacts

Success output:

- `manifest.json`
- `rows.archival.jsonl`
- `rows.compact.jsonl`
- `dedupe-audit.jsonl`
- `summary.md`

Questions answered:

- can the owned parser surfaces cover the real session corpus cleanly
- is the row model rich enough for later analysis
- are canonicalization and dedupe stable and auditable

### Stage 2: Historical Drift Analyzer

Goal:

- prove task-frame inference and deterministic drift scoring over session-scoped row artifacts

Success output:

- `checkpoints.jsonl`
- evidence-backed checkpoint summaries

Questions answered:

- can task-frame inference work without a plan artifact
- can the three drift classes be explained with reviewable evidence
- which signals survive cross-repo usage

### Stage 3: Calibration And Evaluation

Goal:

- compare analyzer output against human judgment across known sessions

Success output:

- tuned thresholds
- known false-positive and false-negative classes
- evidence on whether operator trust is building

Questions answered:

- which thresholds are operationally useful
- which row features matter most
- whether any narrow machine-noise rules are worth adding

### Stage 4: Optional Live Sentinel

Goal:

- layer scheduling and optional adjudication on top of the proven analyzer

Success output:

- trigger scheduler
- operator-facing warning stream
- optional `gpt-5.4-mini` adjudication path

Questions answered:

- what live cadence is useful
- when to escalate to the model
- how to present warnings without overfiring

## Recommended Follow-On Design Docs

After this umbrella doc, split into focused design docs:

1. `agent-session-compactor-v0.1`
   - discovery
   - parser reuse
   - row schema
   - canonicalization
   - exact dedupe
   - export bundle

2. `agent-drift-analyzer-v0.1`
   - task-frame inference
   - candidate truth artifact logic
   - working-set logic
   - drift classes
   - threshold rules
   - checkpoint schema

3. `agent-drift-sentinel-v0.2` or later
   - live triggers
   - optional model adjudication
   - operator surface
   - runtime integration

## Current Locked Decisions

These decisions should remain stable unless later evidence forces a redesign:

- semantic center is `task frame`, not `active plan`
- baseline source corpus is Codex session artifacts, not Substrate trace
- owned parser crates are first-class dependencies, not optional extras
- module binaries should stay thin over library-first seams
- exact-hash dedupe is the archival baseline
- drift scoring is deterministic before any model layer
- model adjudication is optional and must never become the sole explanation path
- live integration is not required for the first useful proof

## Open Design Questions

These should move into the child design docs rather than stay vague in the umbrella plan:

- whether wrapper-style ingestion ships in the first compactor cut or behind an explicit input mode
- the exact row schema needed for drift evidence without overfitting to Codex transport details
- the minimum session-scoped evidence window needed for stable task-frame inference
- the checkpoint schema needed for replayable operator trust
- the threshold policy for escalation to optional model adjudication

## Planning Guidance

Use this document as the parent source of truth.

Then break work outward, not inward:

- do not start from live sentinel scheduling
- do not mix compaction semantics with checkpoint semantics
- do not treat historical transcript analysis and live operator warning as the same module
- do not reopen parser selection unless the owned crates fail a concrete corpus need

The intended planning order is:

1. umbrella architecture overview
2. compactor spec
3. analyzer spec
4. evaluation plan
5. optional live sentinel design
