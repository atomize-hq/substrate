---
seam_id: SEAM-06
seam_slug: validation-evidence-topology
type: conformance
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
    - SEAM-04
    - SEAM-05
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - any upstream contract changes
    - repo harness path changes
    - smoke-wrapper topology changes
    - manual evidence expectations change
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

# SEAM-06 - Validation And Evidence Topology

## Goal / value

Own one authoritative validation model so repo tests, feature smoke, and manual evidence all reinforce the same contract instead of forming competing definitions.

## Scope

### In

- `tests/installers/pkg_manager_detection_smoke.sh` as the authoritative repo harness
- `smoke/linux-smoke.sh` as a thin wrapper over the repo harness
- `manual_testing_playbook.md` as the human evidence path
- contract-to-assertion coverage for parser, mapping, selectors, fallback, warning, remediation, and wrapper parity

### Out

- checkpoint execution itself
- downstream readiness publication

## Primary interfaces

### Inputs

- all upstream installer, wrapper, and doc contracts

### Outputs

- one authoritative validation topology
- one manual evidence model
- checkpoint-ready evidence inputs

## Key invariants / rules

1. repo harness is the behavior authority
2. feature-local smoke wrapper adds no second assertion contract
3. manual playbook references the same topology and expected stderr/exit outcomes
4. validation evidence remains contract-shaped, not ad hoc

## Dependencies

### Direct blockers

- `SEAM-01`
- `SEAM-02`
- `SEAM-03`
- `SEAM-04`
- `SEAM-05`

### Transitive blockers

- None

### Direct consumers

- `SEAM-07`

### Derived consumers

- future maintenance and regressions

## Touch surface

- `tests/installers/pkg_manager_detection_smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
- source evidence in `BEDPM3-spec.md`, `decision_register.md`, and `ci_checkpoint_plan.md`

## Verification

- repo harness proves the full precedence chain and source vocabulary
- smoke wrapper calls the harness and returns its result without introducing new assertions
- manual playbook covers the selected Debian, Arch, flag, env, and failure-path evidence cases
- wrapper parity and remediation branches are explicitly asserted through the validation topology

## Risks / unknowns

- validation assets drifting into duplicate authorities
- manual playbook and smoke wrapper going stale relative to the repo harness
- checkpoint consuming evidence that is not contract-complete

## Rollout / safety

- this seam is conformance work with real product value: it prevents contract drift
- it must not become a generic cleanup seam

## Downstream decomposition context

### Why this seam is `future`

It must consume the final operator-facing contract from upstream seams before review can lock the validation topology.

### Which threads matter most

- `THR-05`
- `THR-06`

### What the first seam-local review should focus on

- whether the repo harness covers every published contract
- whether smoke wrapper thinness is preserved
- whether manual evidence cases match the same truth

### Expected seam-local slice themes

- authoritative repo harness work
- smoke-wrapper alignment
- manual evidence updates
- seam-exit publication of evidence topology truth

## Expected seam-exit concerns

### Contracts likely to publish

- `C-10`

### Threads likely to advance

- `THR-06` to `published`

### Review-surface areas likely to shift after landing

- validation topology R3
- checkpoint/handoff R4

### Downstream seams most likely to require revalidation

- `SEAM-07`
