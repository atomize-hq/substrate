---
seam_id: SEAM-02
seam_slug: family-mapping-reporting
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-02-family-mapping-reporting.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - SEAM-01
  required_threads:
    - THR-01
  stale_triggers:
    - parser/input truth from SEAM-01 changes
    - family-table rules change
    - decision-line wording or placement changes
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
# SEAM-02 - Family Mapping And Decision-Line Reporting

## Seam Brief (Restated)

- **Goal / value**: turn normalized parser/input truth into stable manager-selection and reporting truth that later explicit-selector, fallback, docs, validation, and downstream persistence work can consume without reopening SEAM-01 semantics.
- **Type**: capability
- **Scope**
  - In:
    - distro-family matching for Debian or Ubuntu, Fedora or RHEL, Arch, and SUSE
    - availability-based manager selection for the mapped family, including Fedora or RHEL `dnf` then `yum`
    - stable decision-line template, timing, and suppression posture for os-release-based selection
    - publication of `pkg_manager.source=os_release`
  - Out:
    - parser/input hook ownership
    - explicit selector handling
    - ordered PATH fallback, warning line, and exit `4` posture
    - wrapper or docs propagation
    - validation topology ownership
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- **Verification**:
  - mapping must consume only `DETECTED_DISTRO_ID` and `DETECTED_DISTRO_ID_LIKE` from SEAM-01
  - Debian or Ubuntu, Fedora or RHEL, Arch, and SUSE cases must select the documented manager only when available in `PATH`
  - Fedora or RHEL must prefer `dnf` and fall back to `yum` only when `dnf` is unavailable
  - the stable decision line must be emitted exactly once after os-release selection succeeds and before any package-manager install command runs
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `SEAM-01`
  - Required threads:
    - `THR-01`
  - Stale triggers:
    - parser/input truth from SEAM-01 changes
    - family-table rules change
    - decision-line wording or placement changes
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-01`
  - Downstream blocked seams:
    - `SEAM-03`
    - `SEAM-04`
    - `SEAM-05`
    - `SEAM-06`
    - downstream pack `persist-detected-linux-distro-pkg-manager`
  - Contracts produced:
    - `C-03`
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` / `slice-3-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: later seams and the downstream persistence pack depend on family mapping and the stable decision line, so the handoff must be closed out explicitly rather than inferred from script diffs.
- **Expected contracts to publish**:
  - `C-03`
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-02` to `published`
  - `THR-08` to `published`
- **Likely downstream stale triggers**:
  - family-table rules change
  - decision-line wording, placement, or suppression changes
  - `pkg_manager.source=os_release` semantics change
- **Expected closeout evidence**:
  - landed mapping-table and availability-based selection behavior
  - landed decision-line emission and suppression evidence
  - contract publication accounting for `C-03` and `C-04`
  - downstream stale-trigger emission for `SEAM-03`, `SEAM-04`, `SEAM-05`, `SEAM-06`, and the persistence pack if mapping or reporting semantics shift during landing

## Slice index

- `S1` -> `slice-1-family-table-availability-selection.md`
- `S2` -> `slice-2-decision-line-contract-rendering.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-02-closeout.md`
