---
seam_id: SEAM-03
seam_slug: explicit-override-selection
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-03-explicit-override-selection.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - parser/input truth changes
    - mapping/reporting truth changes
    - supported manager vocabulary changes
    - exit `2` / `3` remediation requirements change
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
# SEAM-03 - Explicit Override Selection

## Seam Brief (Restated)

- **Goal / value**: make operator-forced package-manager selection deterministic and fail-closed before fallback behavior is considered.
- **Type**: capability
- **Scope**
  - In:
    - `--pkg-manager`
    - `PKG_MANAGER`
    - supported explicit values
    - selector precedence between flag and env
    - `pkg_manager.source=flag|env`
    - exit `2` invalid-override posture
    - exit `3` explicit-manager-missing posture
    - inherited stable decision-line interaction when an explicit selector succeeds
  - Out:
    - parser/input ownership
    - os-release family mapping and base decision-line ownership
    - ordered PATH fallback, warning line, and exit `4`
    - wrapper/docs propagation
    - validation topology ownership
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - flag beats env
  - env beats os-release mapping and path probe
  - valid explicit selectors never fall through to lower-precedence stages
  - invalid explicit selectors fail with exit `2` and required remediation content
  - explicit selected manager missing from `PATH` fails with exit `3` and required remediation content
  - explicit-selector success reuses the inherited stable decision line with `pkg_manager.source=flag|env`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `SEAM-01`
    - `SEAM-02`
  - Required threads:
    - `THR-01`
    - `THR-02`
  - Stale triggers:
    - parser/input truth changes
    - mapping/reporting truth changes
    - supported manager vocabulary changes
    - exit `2` / `3` remediation requirements change
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-01`
    - `SEAM-02`
  - Downstream blocked seams:
    - `SEAM-04`
    - `SEAM-05`
    - `SEAM-06`
  - Contracts produced:
    - `C-05`
    - `C-06`
  - Contracts consumed:
    - `C-01`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4` / `slice-4-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: later fallback, wrapper/docs, and validation seams must consume published explicit-selector truth instead of inferring precedence or failure posture from script diffs.
- **Expected contracts to publish**:
  - `C-05`
  - `C-06`
- **Expected threads to publish / advance**:
  - `THR-03` to `published`
- **Likely downstream stale triggers**:
  - selector precedence changes
  - supported manager vocabulary changes
  - exit `2` / `3` remediation wording changes
  - decision-line interaction for explicit-selector success changes
- **Expected closeout evidence**:
  - landed flag-over-env precedence
  - landed `pkg_manager.source=flag|env` behavior
  - landed explicit failure taxonomy for exits `2` and `3`
  - downstream stale-trigger accounting for `SEAM-04`, `SEAM-05`, and `SEAM-06`

## Slice index

- `S1` -> `slice-1-flag-selector-precedence.md`
- `S2` -> `slice-2-env-selector-selection.md`
- `S3` -> `slice-3-explicit-failure-taxonomy.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-03-closeout.md`
