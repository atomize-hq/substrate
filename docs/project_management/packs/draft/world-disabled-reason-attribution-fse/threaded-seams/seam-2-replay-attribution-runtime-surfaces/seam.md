---
seam_id: SEAM-2
seam_slug: replay-attribution-runtime-surfaces
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-replay-attribution-runtime-surfaces.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 changes the classifier field set, layer vocabulary, or replay-safe API placement
    - replay copy fragments or recorded-host punctuation change from the contract baseline
    - telemetry field names, enum values, or omission rules change from the contract baseline
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-2 - Replay attribution runtime surfaces (threaded decomposition)

## Seam Brief (Restated)

- **Goal / value**:
  - Publish effective-disable attribution in replay origin summaries, host warnings, and `replay_strategy` so operators and trace consumers see the same winning-layer story.
  - Preserve all existing replay-local opt-out fragments and selection behavior while making effective host fallback intelligible.
- **Type**: capability (consumer of `C-01` / `C-02`, producer of `C-03` / `C-04`)
- **Scope**
  - In:
    - origin-summary wiring for effective-disable attribution
    - host-warning wiring for effective-disable attribution
    - recorded-host format lock: `host (recorded; <reason>)`
    - additive `replay_strategy` provenance fields and `origin_reason_code` extension values
    - omission rules so replay-local opt-out cases do not emit `world_disable_source`
  - Out:
    - new replay subcommands or JSON envelopes
    - helper placement changes beyond the published `SEAM-1` contract
    - smoke-wrapper and manual-playbook authoring
    - broad operator-doc lock-in outside the later conformance seam
- **Touch surface (expected)**:
  - `crates/shell/src/execution/routing/replay.rs`
  - `crates/replay/src/replay/executor.rs`
  - `crates/shell/tests/replay_world.rs`
- **Verification**:
  - This seam consumes `C-01` and `C-02` exactly as published in `../../governance/seam-1-closeout.md`; it does not redefine precedence or redaction semantics.
  - This seam produces `C-03` and `C-04`; readiness means the replay-copy rules, telemetry field set, omit rules, and verification loci are concrete enough to implement without guessing.
  - Verification later depends on exact-string replay tests, trace-field assertions, and explicit opt-out omission checks.
- **Basis posture**:
  - Currentness: `current` because `../../governance/seam-1-closeout.md` publishes the upstream handoff this seam consumes.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none while `../../governance/seam-1-closeout.md` remains the published source of truth for `C-01` and `C-02`
    - any later `SEAM-1` drift that changes the helper/result shape, runtime fragments, or redaction posture forces revalidation before this seam lands
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-03`
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `SEAM-3` must consume closeout-backed proof that replay stderr and `replay_strategy` expose one stable runtime contract, not merely provisional planning intent.
- **Expected contracts to publish**:
  - `C-03`
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-03` (`defined` -> `published`)
  - `THR-04` (`defined` -> `published`)
- **Likely downstream stale triggers**:
  - reason-fragment or recorded-host punctuation drift
  - `origin_reason_code` or `world_disable_source` schema drift
  - replay-local opt-out cases starting to emit `world_disable_source`
- **Expected closeout evidence**:
  - replay tests proving exact origin-summary and host-warning fragments
  - replay tests proving `world_disable_source` emit/omit rules and redaction
  - replay tests or trace assertions proving the additive `origin_reason_code` values

## Slice index

- `S1` -> `slice-1-contract-definition-replay-attribution-runtime-surfaces.md`
- `S2` -> `slice-2-origin-summary-and-host-warning-wiring.md`
- `S3` -> `slice-3-replay-strategy-telemetry-and-omission-rules.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
