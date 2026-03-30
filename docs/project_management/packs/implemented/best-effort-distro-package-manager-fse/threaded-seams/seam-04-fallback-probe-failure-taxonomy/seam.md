---
seam_id: SEAM-04
seam_slug: fallback-probe-failure-taxonomy
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-04-fallback-probe-failure-taxonomy.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
    - SEAM-03
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - explicit-selector behavior changes
    - mapping/reporting truth changes
    - fixed probe order changes
    - warning template or exit `4` remediation changes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---
# SEAM-04 - Fallback Probe And Failure Taxonomy

## Seam Brief (Restated)

- **Goal / value**: finish the hosted-installer decision pipeline with one deterministic fallback rule set, one exact multi-manager warning, and one exact no-manager failure posture.
- **Type**: capability
- **Scope**
  - In:
    - fixed ordered PATH probe
    - multi-manager detection and warning template
    - `pkg_manager.source=path_probe`
    - exit `4` no-manager selection posture
    - final fallback selection after upstream stages do not choose a manager
  - Out:
    - explicit selector ownership
    - wrapper/docs propagation
    - validation topology ownership
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - exactly one supported manager in PATH selects that manager with `path_probe`
  - multiple supported managers emit the exact warning line once and select the earliest manager in fixed order
  - no supported manager selected after earlier stages yields exit `4` with required remediation elements
  - warning placement relative to decision-line output matches the source contract
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `SEAM-01`
    - `SEAM-02`
    - `SEAM-03`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - Stale triggers:
    - explicit-selector behavior changes
    - mapping/reporting truth changes
    - fixed probe order changes
    - warning template or exit `4` remediation changes
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-01`
    - `SEAM-02`
    - `SEAM-03`
  - Downstream blocked seams:
    - `SEAM-05`
    - `SEAM-06`
  - Contracts produced:
    - `C-07`
  - Contracts consumed:
    - `C-01`
    - `C-03`
    - `C-04`
    - `C-05`
    - `C-06`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4` / `slice-4-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: later wrapper/docs and validation seams must consume published fallback and exit `4` truth instead of reconstructing it from installer diffs.
- **Expected contracts to publish**:
  - `C-07`
- **Expected threads to publish / advance**:
  - `THR-04` to `published`
- **Likely downstream stale triggers**:
  - fixed probe order changes
  - warning template changes
  - `pkg_manager.source=path_probe` semantics change
  - exit `4` remediation wording changes
- **Expected closeout evidence**:
  - landed fixed-order path probe behavior
  - landed exact warning-line emission and ordering behavior
  - landed exit `4` remediation posture
  - downstream stale-trigger accounting for `SEAM-05` and `SEAM-06`

## Slice index

- `S1` -> `slice-1-path-probe-selection.md`
- `S2` -> `slice-2-multi-manager-warning-line.md`
- `S3` -> `slice-3-no-manager-exit-4.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-04-closeout.md`
