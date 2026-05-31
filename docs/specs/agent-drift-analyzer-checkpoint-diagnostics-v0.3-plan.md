# Plan: Agent Drift Analyzer Checkpoint Diagnostics v0.3

## Scope

This plan implements:

- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-spec.md`

The goal is to add a compact checkpoint-diagnostics slice that is available in both:

- `checkpoints.jsonl` for machine-readable downstream use
- `summary.md` for operator-readable review

This slice should:

- preserve the current `v0.2` checkpoint-calibration metrics
- widen `checkpoints.jsonl` with compact machine-readable diagnostics for downstream consumers
- explain flagged checkpoints and task-frame evolution with current checkpoint data
- use interval-safe summary logic where cumulative checkpoint windows would otherwise double-count
- stay honest about heuristic semantics around working-set and verification signals

This slice should not:

- widen the compactor bundle contract
- add new drift classes
- rework checkpoint segmentation
- claim stronger phase or timeline semantics than the live analyzer supports

## Why This Slice Comes Next

The current crate state makes this the right next analyzer-only follow-up:

- `agent-session-compactor` already exports the analyzer-facing `v0.2` bundle contract with
  manifest-owned file entries, `source_file_id`, and file-scoped `turn_id_ref`
- `agent-drift-analyzer` already validates and resolves that contract at the input boundary
- the analyzer summary already covers checkpoint density and spacing, so the next highest-value gap
  is not more checkpoint count math; it is better operator interpretation of why checkpoints are
  being flagged and changing
- replay or sentinel consumers cannot depend on `summary.md` scraping, so the new diagnostics need a
  compact checkpoint-schema home as well
- the live analyzer already exposes richer surfaces than the older Packet 10A docs assumed:
  - command observations with `family`, `paths`, `read_like`, `write_like`, and
    `verification_like`
  - `task_frame.confidence`
  - per-checkpoint `drift_scores`
  - truth-artifact and working-set hints

Because those surfaces already exist, the best next slice is analyzer export/reporting work, not a
bundle or normalization redesign.

## Implementation Strategy

Build `v0.3` inside the analyzer checkpoint/export path:

- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- adjacent tests

Implementation order:

1. lock the new metric semantics and schema shape
2. widen the checkpoint schema with a compact diagnostics object
3. add helper logic for checkpoint-derived diagnostics and interval counters
4. aggregate top-level metrics with explicit sum/max/weighted rules
5. render a concise diagnostics block in `summary.md`
6. refresh targeted tests and any stale packet/smoke docs

This keeps the slice narrow and prevents drift into upstream compactor contract work.

## Major Components

### 1. Metric Contract And Schema Shape

Deliver first:

- lock exact definitions for:
  - flagged checkpoint rate
  - per-drift-class flagged frequency
  - task-frame transition count
  - task-frame confidence distribution
  - working-set churn
  - verification density
  - average evidence items per checkpoint
- lock the compact checkpoint diagnostics payload:
  - `task_frame_transitioned`
  - `working_set_changed`
  - `interval_command_count`
  - `interval_verification_command_count`
  - `evidence_item_count`
- lock whether each metric is summed, maxed, or weighted at the top level
- lock unavailable behavior and compact render format

Why first:

- summary rendering is where ambiguity becomes user-visible fastest
- replay-facing schema changes must be explicit before downstream assumptions drift

### 2. Checkpoint Schema Widening

Deliver second:

- bump checkpoint schema version from `v0.1` to `v0.2`
- add a compact `diagnostics` object to each checkpoint row
- keep the widening additive and analyzer-local

Why second:

- replay and sentinel consumers need a machine-readable home for the new diagnostics
- only a subset of the requested metrics need new fields, so the contract widening can stay compact

### 3. Checkpoint-Derived Diagnostics And Interval Counters

Deliver third:

- helper logic that computes checkpoint-local diagnostics and summary metrics:
  - flagged checkpoint rate
  - drift-class flagged counts and rates
  - task-frame transition count
  - task-frame confidence distribution
  - working-set churn from checkpoint snapshots
  - average evidence items per checkpoint
- reconstruct newly introduced compact-row intervals between adjacent checkpoint boundaries
- run `collect_command_observations` on those interval rows only
- persist interval command counts onto each checkpoint diagnostics payload

Why third:

- these helpers remain deterministic once the compact checkpoint payload is locked
- verification density is the only requested metric that cannot be computed honestly from cumulative
  checkpoint snapshots alone

### 4. Top-Level Aggregation And Rendering

Deliver fourth:

- add top-level aggregation with explicit sum/max/weighted rules
- add a compact diagnostics block after the existing top-level metrics
- add matching per-session diagnostics lines without turning the summary into a wall of detail
- ensure summary metrics align with the machine-readable checkpoint diagnostics payload

Why fourth:

- aggregation and formatting should follow settled helper semantics
- keeping the rendering late reduces churn while tests are still clarifying edge cases

### 5. Tests And Continuity Docs

Deliver fifth:

- extend `crates/agent-drift-analyzer/tests/export_bundle.rs`
- add or adjust helper-level tests only if the integration fixture cannot express the edge case
- refresh stale continuity docs if the live summary/export contract has moved beyond what
  `docs/specs/hybrid-drift-sentinel-implementation-order.md` or smoke docs currently say

Why fifth:

- the older checkpoint-calibration docs predate the richer analyzer context now present in the live
  crate
- the packet map should not keep describing the analyzer as if `v0.3` did not happen

## Sequencing

Sequential work:

1. metric contract and schema shape
2. checkpoint schema widening
3. checkpoint-derived diagnostics
4. verification-density interval slicing
5. top-level aggregation
6. summary rendering
7. doc refresh and bounded smoke

Parallel-safe work after the metric contract is locked:

- export-bundle test authoring can proceed in parallel with summary rendering
- continuity-doc refresh can proceed in parallel with smoke verification
- fixture enrichment for edge cases can proceed in parallel with top-level aggregation helpers
- downstream consumer review can proceed in parallel once the checkpoint diagnostics payload is
  locked

## Risks And Mitigations

### Risk 1: Heuristic metrics over-claim semantic meaning

Risk:

- `working_set_paths` come from directive-text and command-path hints, not a true work-area model
- verification density comes from command-family heuristics, not outcome-normalized verification

Mitigation:

- define both metrics literally
- render them as churn and density, not as phase shifts or success rates
- keep the blocked metrics ledger explicit in the docs

### Risk 2: Interval metrics accidentally double-count cumulative windows

Risk:

- checkpoints are exported from cumulative windows, so naive summary logic would count the same
  commands repeatedly

Mitigation:

- reconstruct interval rows from adjacent checkpoint boundaries
- add tests that fail if verification density is computed from full cumulative windows

### Risk 3: Task-frame transitions are mistaken for semantic phase transitions

Risk:

- the current task-frame identity is a checkpoint snapshot over objective, truth artifacts,
  working-set paths, tools, command families, and verification commands

Mitigation:

- define `task-frame transition count` only as identity changes between adjacent checkpoints
- explicitly refuse to label it a phase taxonomy in docs or summary text

### Risk 4: Evidence-count metrics inflate due to duplicates

Risk:

- the same evidence row may appear in task-frame evidence and drift-score evidence lists

Mitigation:

- dedupe evidence items by row-ref plus reason before counting
- cover the dedupe rule in targeted tests

### Risk 5: Summary bloat makes the operator report worse

Risk:

- too many added lines could turn `summary.md` from a scan-friendly report into a verbose dump

Mitigation:

- keep the new section compact
- do not add per-checkpoint prose beyond what already exists
- prefer aggregated lines over a proliferation of sub-metrics

### Risk 6: Checkpoint-schema widening creates downstream compatibility churn

Risk:

- replay or sentinel consumers may already assume the current checkpoint schema version and shape

Mitigation:

- keep the widening additive and compact
- bump `schema_version` explicitly
- cover checkpoint JSONL serialization in targeted tests
- keep all new semantics documented in the spec rather than implicit in exporter code

## Verification Checkpoints

### Checkpoint VD-A: Metric Semantics Are Locked

Verify:

- spec, plan, and tasks agree on numerator, denominator, aggregation mode, unavailable behavior, and
  the compact checkpoint diagnostics payload

Evidence:

- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-diagnostics-v0.3-tasks.md`

### Checkpoint VD-B: Checkpoint JSONL Carries The Diagnostics

Verify:

- emitted checkpoint rows serialize the compact diagnostics payload and bumped schema version

Evidence:

- `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`

### Checkpoint VD-C: Checkpoint-Derived Metrics Are Deterministic

Verify:

- checkpoint-only metrics produce stable results across repeated runs on the same fixture

Evidence:

- `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`

### Checkpoint VD-D: Interval Metrics Avoid Double Counting

Verify:

- verification density uses interval rows rather than cumulative checkpoint windows

Evidence:

- targeted export-bundle test coverage naming the interval behavior directly

### Checkpoint VD-E: Summary Remains Reviewable

Verify:

- the new diagnostics slice explains the checkpoint stream without overwhelming the existing summary

Evidence:

- rendered `summary.md` review in fixture tests
- bounded analyzer smoke output review

### Checkpoint VD-F: Continuity Docs Match Live Behavior

Verify:

- any stale packet or smoke examples are updated once `v0.3` lands

Evidence:

- refreshed implementation-order note and/or smoke guide output example if required

## Handoff To Later Slices

This plan is complete enough for a later follow-up only when all of the following are true:

- the `v0.3` diagnostics block is stable and useful on both fixture tests and a bounded real bundle
- replay and sentinel consumers can consume the new diagnostics from `checkpoints.jsonl`
- the remaining blocked metrics are clearly attributable to missing normalization or stronger
  semantics rather than to summary gaps
- future schema-widening requests are grounded in a concrete metric that `v0.3` could not support

## Deferred Metrics Ledger

The following ideas remain intentionally out of scope or blocked after `v0.3`:

- semantic phase transition count
  - blocked because current task-frame changes are snapshot identity changes, not a phase taxonomy
- time spent in each phase
  - blocked by the same missing phase semantics
- first grounding before first write
  - blocked by missing first-class read/edit event normalization and stronger action ordering rules
- first verification lag
  - blocked by the lack of a richer verification-command and outcome model
- stronger work-area shift modeling
  - blocked because current working-set paths are heuristic hints, not a robust area model
- stronger repetition score beyond current evidence/command heuristics
  - blocked because current repetition proof is limited to exact-dedupe and drift-score evidence
- true file-read versus file-edit counts
  - blocked by missing normalized event kinds
- tool success rate
  - blocked by missing stronger completion/outcome normalization

## Exit Criteria

The slice is ready to hand off to implementation when:

1. the `v0.3` metric contract is fully specified and repo-aligned
2. the checkpoint-schema widening needed for downstream consumers is explicit and compact
3. the implementation order is explicit about what must stay analyzer-only
4. the risks around heuristics, double counting, summary bloat, and compatibility churn are
   concretely addressed
5. the task list below is detailed enough for a focused implementation session
