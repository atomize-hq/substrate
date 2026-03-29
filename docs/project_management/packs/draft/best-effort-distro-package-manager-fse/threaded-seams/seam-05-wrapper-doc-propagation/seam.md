---
seam_id: SEAM-05
seam_slug: wrapper-doc-propagation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-05-wrapper-doc-propagation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - SEAM-02
    - SEAM-03
    - SEAM-04
  required_threads:
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - decision-line wording changes
    - exit taxonomy changes
    - warning or remediation wording changes
    - env-hook semantics change
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
# SEAM-05 - Wrapper And Doc Propagation

## Seam Brief (Restated)

- **Goal / value**: make the wrapper and operator-facing docs reuse the landed installer contract exactly, including feature exit preservation and fallback wording, instead of paraphrasing or collapsing it.
- **Type**: integration
- **Scope**
  - In:
    - `scripts/substrate/install.sh` pass-through for exits `0`, `2`, `3`, and `4`
    - `docs/INSTALLATION.md` propagation of precedence, decision-line, warning, and remediation truth
    - `docs/reference/env/contract.md` propagation of `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
    - macOS-hosted wording that clarifies the Lima-backed Linux path without claiming native macOS package-manager-selection logic
  - Out:
    - repo harness ownership
    - checkpoint execution
    - downstream handoff publication
- **Touch surface**:
  - `scripts/substrate/install.sh`
  - `docs/INSTALLATION.md`
  - `docs/reference/env/contract.md`
  - `docs/WORLD.md`
- **Verification**:
  - wrapper preserves exits `0`, `2`, `3`, and `4`
  - installation docs restate the precedence chain, warning posture, and remediation truth without drift
  - env docs keep allowed values, hook semantics, and Linux-only scope exact
  - macOS-hosted docs make the Lima-backed Linux path explicit without overstating native macOS behavior
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `SEAM-02`
    - `SEAM-03`
    - `SEAM-04`
  - Required threads:
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - Stale triggers:
    - decision-line wording changes
    - exit taxonomy changes
    - warning or remediation wording changes
    - env-hook semantics change
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-02`
    - `SEAM-03`
    - `SEAM-04`
  - Downstream blocked seams:
    - `SEAM-06`
    - `SEAM-07`
  - Contracts produced:
    - `C-08`
    - `C-09`
  - Contracts consumed:
    - `C-03`
    - `C-04`
    - `C-05`
    - `C-06`
    - `C-07`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4` / `slice-4-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: validation and checkpoint seams must consume one published wrapper/doc parity record instead of inferring operator-facing truth from mixed code and doc diffs.
- **Expected contracts to publish**:
  - `C-08`
  - `C-09`
- **Expected threads to publish / advance**:
  - `THR-05` to `published`
- **Likely downstream stale triggers**:
  - wrapper exit handling changes
  - decision-line wording or placement changes
  - warning or remediation wording changes
  - env-hook semantics change

## Slice index

- `S1` -> `slice-1-wrapper-exit-pass-through.md`
- `S2` -> `slice-2-installation-doc-contract-propagation.md`
- `S3` -> `slice-3-env-and-macos-hosted-doc-clarity.md`
- `S4` -> `slice-4-seam-exit-gate.md`
