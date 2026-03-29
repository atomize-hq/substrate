---
seam_id: SEAM-06
seam_slug: validation-evidence-topology
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-06-validation-evidence-topology.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
    - SEAM-03
    - SEAM-04
    - SEAM-05
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - wrapper/doc contract changes
    - repo harness path changes
    - smoke-wrapper topology changes
    - manual evidence expectations change
    - macOS Lima-backed verification path changes
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
# SEAM-06 - Validation And Evidence Topology

## Seam Brief (Restated)

- **Goal / value**: own one authoritative validation model so repo tests, feature smoke, and manual evidence all reinforce the same contract instead of forming competing definitions.
- **Type**: conformance
- **Scope**
  - In:
    - `tests/installers/pkg_manager_detection_smoke.sh` as the authoritative repo harness
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` as a thin wrapper over the repo harness
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` as the human evidence path
    - `scripts/mac/smoke.sh` and related documentation for macOS-hosted Lima-backed verification of the Linux installer path
    - contract-to-assertion coverage for parser, mapping, selectors, fallback, warning, remediation, and wrapper parity
  - Out:
    - checkpoint execution itself
    - downstream readiness publication
- **Touch surface**:
  - `tests/installers/pkg_manager_detection_smoke.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - `scripts/mac/smoke.sh`
  - `docs/WORLD.md`
- **Verification**:
  - repo harness proves the full precedence chain, source vocabulary, and wrapper/doc parity inputs
  - smoke wrapper calls the harness and returns its result without introducing new assertions
  - manual playbook covers the selected Debian, Arch, flag, env, fallback, and remediation evidence cases
  - macOS-hosted verification proves the Lima-backed path reaches the same Linux installer contract and produces behavior evidence rather than compile-only parity
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `SEAM-01`
    - `SEAM-02`
    - `SEAM-03`
    - `SEAM-04`
    - `SEAM-05`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
    - `THR-05`
  - Stale triggers:
    - wrapper/doc contract changes
    - repo harness path changes
    - smoke-wrapper topology changes
    - manual evidence expectations change
    - macOS Lima-backed verification path changes
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-01`
    - `SEAM-02`
    - `SEAM-03`
    - `SEAM-04`
    - `SEAM-05`
  - Downstream blocked seams:
    - `SEAM-07`
  - Contracts produced:
    - `C-10`
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - `C-05`
    - `C-06`
    - `C-07`
    - `C-08`
    - `C-09`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` / `slice-3-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: checkpoint and handoff work must consume one published validation topology record instead of inferring authority from mixed harness, smoke, and manual evidence edits.
- **Expected contracts to publish**:
  - `C-10`
- **Expected threads to publish / advance**:
  - `THR-06` to `published`
- **Likely downstream stale triggers**:
  - repo harness path changes
  - smoke-wrapper topology changes
  - manual evidence expectations change
  - macOS Lima-backed verification path changes
- **Expected closeout evidence**:
  - landed authoritative repo harness coverage for the published installer, wrapper, and doc contracts
  - landed smoke-wrapper alignment that keeps the harness authoritative
  - landed manual evidence and macOS-hosted verification updates that reuse the same topology

## Slice index

- `S1` -> `slice-1-repo-harness-and-smoke-wrapper-topology.md`
- `S2` -> `slice-2-manual-evidence-and-macos-hosted-verification.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeout consumed for promotion: `../../governance/seam-05-closeout.md`
