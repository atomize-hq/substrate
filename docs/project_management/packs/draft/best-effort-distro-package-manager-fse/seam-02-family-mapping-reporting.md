---
seam_id: SEAM-02
seam_slug: family-mapping-reporting
type: capability
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v2
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

## Goal / value

Turn normalized input truth into stable manager-selection and reporting truth that all later decision stages and docs can inherit.

## Scope

### In

- distro-family matching for Debian/Ubuntu, Fedora/RHEL, Arch, and SUSE
- availability-based mapped selection, including Fedora/RHEL `dnf` then `yum`
- stable decision-line template, timing, and suppression posture for the os-release stage
- publication of selected-manager and reporting vocabulary inherited by later seams

### Out

- parser/input hook ownership
- explicit selector handling
- ordered PATH fallback and warning line
- wrapper/docs propagation
- validation topology ownership

## Primary interfaces

### Inputs

- normalized `distro_id`
- normalized `distro_id_like`
- hook/input truth from `SEAM-01`
- host `PATH` availability checks for mapped managers

### Outputs

- mapped manager selection when family rules match
- `pkg_manager.source=os_release`
- stable decision-line contract

## Key invariants / rules

1. family rules consume `SEAM-01` truth only
2. mapping selects only when the mapped manager is actually available
3. decision line is emitted exactly once when a concrete manager is selected by this stage
4. later seams inherit this reporting contract instead of restating it

## Dependencies

### Direct blockers

- `SEAM-01`

### Transitive blockers

- None

### Direct consumers

- `SEAM-03`
- `SEAM-04`
- `SEAM-05`
- `SEAM-06`

### Derived consumers

- downstream persistence pack

## Touch surface

- `scripts/substrate/install-substrate.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- source evidence in `decision_register.md`, `BEDPM0-spec.md`, and `minimal_spec_draft.md`

## Verification

- Debian/Ubuntu, Arch, Fedora/RHEL, and SUSE family cases select the documented manager when available
- Fedora/RHEL prefers `dnf` and falls back to `yum` only when required
- decision-line wording and placement follow the source contract
- no-selection branches fall through cleanly to later seams without partial reporting drift

## Risks / unknowns

- family-table and availability logic drifting apart during decomposition
- decision-line suppression behavior becoming split across seams
- downstream persistence assuming a broader family table than v1 allows

## Rollout / safety

- this seam still changes Linux-only behavior
- selection/reporting truth must stabilize before explicit selectors and fallback are reviewed

## Downstream decomposition context

### Why this seam is `next`

It is the second foundational contract: once `SEAM-01` lands, this seam can be provisionally planned without reopening parser/input truth.

### Which threads matter most

- `THR-01`
- `THR-02`
- `THR-08`

### What the first seam-local review should focus on

- completeness of the family table
- exact decision-line wording, timing, and suppression
- clean separation between os-release selection and later explicit/fallback stages

### Expected seam-local slice themes

- family-table implementation
- availability-based selection behavior
- decision-line rendering and timing
- seam-exit publication of mapping/reporting truth

## Expected seam-exit concerns

### Contracts likely to publish

- `C-03`
- `C-04`

### Threads likely to advance

- `THR-02` to `published`
- `THR-08` to `published`

### Review-surface areas likely to shift after landing

- decision pipeline R1
- operator-facing surface R2

### Downstream seams most likely to require revalidation

- `SEAM-03`
- `SEAM-04`
- `SEAM-05`
- `SEAM-06`
