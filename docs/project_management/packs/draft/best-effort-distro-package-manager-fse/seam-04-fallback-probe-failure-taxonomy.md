---
seam_id: SEAM-04
seam_slug: fallback-probe-failure-taxonomy
type: capability
status: proposed
execution_horizon: future
plan_version: v2
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v2
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
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-04 - Fallback Probe And Failure Taxonomy

## Goal / value

Finish the hosted-installer decision pipeline with one deterministic fallback rule set, one exact multi-manager warning, and one exact no-manager failure posture.

## Scope

### In

- fixed ordered PATH probe
- multi-manager detection and warning template
- `pkg_manager.source=path_probe`
- exit `4` no-manager selection posture
- final fallback selection after upstream stages do not choose a manager

### Out

- explicit selector ownership
- wrapper/docs propagation
- validation topology ownership

## Primary interfaces

### Inputs

- upstream parser/input truth
- upstream mapping/reporting truth
- upstream explicit selector contract
- host `PATH`

### Outputs

- path-probe selection
- warning-line contract
- exit `4` remediation posture

## Key invariants / rules

1. fixed probe order is authoritative, not raw PATH ordering
2. multi-manager hosts warn and still select deterministically
3. no-manager branch exits `4` instead of silently continuing
4. warning line appears before the decision line when both are emitted

## Dependencies

### Direct blockers

- `SEAM-01`
- `SEAM-02`
- `SEAM-03`

### Transitive blockers

- None

### Direct consumers

- `SEAM-05`
- `SEAM-06`

### Derived consumers

- operators

## Touch surface

- `scripts/substrate/install-substrate.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- source evidence in `decision_register.md`, `BEDPM1-spec.md`, and `minimal_spec_draft.md`

## Verification

- exactly one supported manager in PATH selects that manager with `path_probe`
- multiple supported managers emit the exact warning line once and select the earliest manager in fixed order
- no supported manager selected after earlier stages yields exit `4` with required remediation elements
- warning placement relative to decision-line output matches the source contract

## Risks / unknowns

- fallback logic drifting from explicit-selector boundaries
- warning template becoming duplicated across docs/tests
- no-manager branch being treated as generic installer failure

## Rollout / safety

- this seam closes the decision tree and determines the operator-visible fallback posture
- later propagation seams must reuse its warning and exit `4` truth verbatim

## Downstream decomposition context

### Why this seam is `future`

It should not be deeply planned until explicit-selector behavior is concrete because both seams share the same final decision pipeline and suppression rules.

### Which threads matter most

- `THR-03`
- `THR-04`

### What the first seam-local review should focus on

- exact fixed probe order implementation
- warning-line wording and placement
- exit `4` remediation completeness

### Expected seam-local slice themes

- path-probe implementation
- warning-line branch
- exit `4` branch
- seam-exit publication of fallback truth

## Expected seam-exit concerns

### Contracts likely to publish

- `C-07`

### Threads likely to advance

- `THR-04` to `published`

### Review-surface areas likely to shift after landing

- decision pipeline R1
- operator-facing surface R2

### Downstream seams most likely to require revalidation

- `SEAM-05`
- `SEAM-06`
