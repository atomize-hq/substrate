---
seam_id: SEAM-3
seam_slug: health-disabled-aware-summary
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-health-disabled-aware-summary.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - THR-02 or THR-03 change after `SEAM-2` closeout
    - disabled-mode summary or guidance suppression drifts in crates/shell/src/builtins/health.rs before landing
    - docs/USAGE.md or shim_health fixtures drift from landed health behavior
    - json-envelope or attribution work obscures the embedded shim status fields before closeout
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
# SEAM-3 - Health disabled-aware summary

## Seam Brief (Restated)

- **Goal / value**: make `substrate health` a trustworthy operator signal for host-only-by-choice installs by consuming landed shim status contracts, treating disabled/skipped as non-error, and aligning docs/examples to shipped behavior.
- **Type**: capability
- **Scope**
  - In:
    - consume published `.shim.world.status` and `.shim.world_deps.status` from `SEAM-2`
    - render deterministic disabled/skipped text for `substrate health`
    - set `summary.world_ok = null` and omit summary error fields when disabled
    - emit empty `world_deps_missing` / `world_deps_blocked` arrays when disabled
    - suppress enabled-world `substrate world deps current` guidance when disabled
    - align `docs/USAGE.md` to the landed machine-readable contract
  - Out:
    - shim-doctor report generation internals
    - shared classifier plumbing
    - cross-platform smoke orchestration beyond seam-local proof
- **Touch surface**: `crates/shell/src/builtins/health.rs`, `crates/shell/tests/shim_health.rs`, `docs/USAGE.md`.
- **Verification**:
  - disabled `substrate health --json` must emit `summary.world_ok = null`, omit summary error fields, and keep `world_deps_missing` / `world_deps_blocked` empty
  - disabled text must print the exact contract lines and suppress enabled-world world-deps guidance
  - enabled-mode failures must remain fail-visible
  - docs/examples must describe `.world.status` / `.world_deps.status` as the canonical machine-readable contract
- **Basis posture**:
  - Currentness: current (revalidated against `governance/seam-1-closeout.md`, `governance/seam-2-closeout.md`, and the landed shim status contracts)
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`
  - Required threads: `THR-01`, `THR-02`, `THR-03`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers resolved: `THR-01`, `THR-02`, and `THR-03` are published and revalidated for this seam
  - Downstream blocked seams: `SEAM-4`
  - Contracts produced: `C-05`
  - Contracts consumed: `C-01`, `C-02`, `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it publishes the final disabled-aware health summary contract that downstream conformance and operator documentation must consume without reinvention.
- **Expected contracts to publish**: `C-05`
- **Expected threads to publish / advance**: `THR-05`
- **Likely downstream stale triggers**: summary field-path drift, guidance suppression drift, docs/example drift, or any regression back to legacy error-string aggregation.
- **Expected closeout evidence**: landed summary derivation in `health.rs`, disabled summary/docs tests, and a recorded manual repro for disabled health text/JSON behavior.

## Slice index

- `S1` -> `slice-1-contract-definition-c-05.md`
- `S2` -> `slice-2-disabled-summary-docs-and-tests.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
