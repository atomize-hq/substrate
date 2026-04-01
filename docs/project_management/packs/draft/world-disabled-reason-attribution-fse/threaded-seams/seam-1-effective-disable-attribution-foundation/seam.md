---
seam_id: SEAM-1
seam_slug: effective-disable-attribution-foundation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-effective-disable-attribution-foundation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - ADR-0037 winning-layer precedence changes for world.enabled=false
    - tokenized display rules change for workspace or global config paths
    - allowlisted env token display rules change for SUBSTRATE_OVERRIDE_WORLD
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
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-1 - Effective disable attribution foundation (threaded decomposition)

## Seam Brief (Restated)

- **Goal / value**:
  - Produce one replay-safe, single-source classifier for effective `world.enabled=false` attribution so later runtime copy and telemetry can consume a stable contract.
  - Eliminate precedence and redaction drift by binding attribution to the same provenance that resolves `world.enabled`.
- **Type**: integration (producer seam for `C-01` and `C-02`)
- **Scope**
  - In:
    - a shared helper/model representing the effective-disable source for `world.enabled=false` (no prose rendering)
    - a concrete layer vocabulary for effective-disable attribution (including the `source_unknown` fallback)
    - tokenized displays for `<workspace>/.substrate/workspace.yaml` and `$SUBSTRATE_HOME/config.yaml`
    - allowlisted env-token handling for `SUBSTRATE_OVERRIDE_WORLD` (token only; never raw values)
    - deterministic precedence + redaction coverage for all supported layers and for `source_unknown`
  - Out:
    - replay origin-summary copy changes (`SEAM-2`)
    - replay host-warning copy changes (`SEAM-2`)
    - telemetry schema changes (`SEAM-2`)
    - docs/smoke/manual lock-in (`SEAM-3`)
- **Touch surface (expected)**:
  - `crates/shell/src/execution/config_model.rs` (`world_disable_attribution(...)`)
  - `crates/shell/src/execution/routing/replay.rs` consumption begins in `SEAM-2`
  - `crates/shell/tests/replay_world.rs` (integration regression surface)
- **Verification**:
  - Produces contracts `C-01` (classifier result shape + layer vocabulary) and `C-02` (provenance precedence + redaction/tokenization semantics).
  - Evidence lives in deterministic unit tests for the winner-to-source mapping and negative assertions that no raw paths/env values leak.
- **Basis posture**:
  - Currentness: `current` (no upstream closeouts; binds directly to effective-config provenance)
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-2`, `SEAM-3` (consume `C-01`/`C-02`)
  - Contracts produced: `C-01`, `C-02`
  - Contracts consumed: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `SEAM-2` must not wire replay copy/telemetry against inferred semantics; it must consume a published foundation contract and revalidate its own assumptions against it.
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01` (`defined` -> `published`)
  - `THR-02` (`defined` -> `published`)
- **Likely downstream stale triggers**:
  - `SEAM-2` must revalidate if the classifier field set or layer vocabulary differs.
  - `SEAM-3` must revalidate if tokenized display or allowlist semantics differ.
- **Expected closeout evidence**:
  - commit(s) landing the helper/model and deterministic mapping tests
  - recorded layer vocabulary and redaction invariants, plus `source_unknown` fallback proof

## Slice index

- `S1` -> `slice-1-contract-definition-effective-disable-attribution.md`
- `S2` -> `slice-2-deterministic-precedence-redaction-tests.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
