---
seam_id: SEAM-05
seam_slug: wrapper-doc-propagation
type: integration
status: proposed
execution_horizon: future
plan_version: v2
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v2
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

# SEAM-05 - Wrapper And Doc Propagation

## Goal / value

Make the final operator-facing entrypoints and docs faithfully reuse the installer contract instead of weakening or rephrasing it.

## Scope

### In

- `scripts/substrate/install.sh` pass-through for exits `0`, `2`, `3`, and `4`
- `docs/INSTALLATION.md` propagation of precedence, reporting, warning, and remediation truth
- `docs/reference/env/contract.md` propagation of `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
- Linux-only no-change posture for macOS and Windows

### Out

- repo harness ownership
- checkpoint execution and downstream handoff

## Primary interfaces

### Inputs

- mapping/reporting truth
- explicit-selector truth
- fallback/warning/failure truth

### Outputs

- wrapper parity contract
- no-drift docs contract

## Key invariants / rules

1. wrapper preserves feature exits rather than collapsing them
2. docs reuse contract truth instead of redefining it
3. env docs keep hook and precedence semantics exact
4. macOS and Windows remain explicit no-change platforms

## Dependencies

### Direct blockers

- `SEAM-02`
- `SEAM-03`
- `SEAM-04`

### Transitive blockers

- `SEAM-01`

### Direct consumers

- `SEAM-06`
- `SEAM-07`

### Derived consumers

- operators and maintainers

## Touch surface

- `scripts/substrate/install.sh`
- `docs/INSTALLATION.md`
- `docs/reference/env/contract.md`
- source evidence in `BEDPM2-spec.md` and `contract.md`

## Verification

- wrapper preserves exits `0`, `2`, `3`, and `4`
- installation docs restate the exact precedence chain and warning posture without drift
- env docs keep allowed values, hook semantics, and Linux-only scope exact
- no new operator-facing ambiguity is introduced by wrapper or doc wording

## Risks / unknowns

- wrapper path masking upstream failure classes
- docs paraphrasing contract truth into conflicting wording
- env-hook wording widening semantics beyond the source contract

## Rollout / safety

- wrapper change is additive preservation, not new selection logic
- docs should describe already-selected contract truth, not invent it

## Downstream decomposition context

### Why this seam is `future`

It must consume stable upstream decision semantics first. Extraction keeps it future so later review can validate parity rather than rediscover contract meaning.

### Which threads matter most

- `THR-04`
- `THR-05`

### What the first seam-local review should focus on

- wrapper exit propagation boundary
- exact doc reuse of precedence, warning, and remediation wording
- env-doc treatment of `SUBSTRATE_INSTALL_OS_RELEASE_PATH`

### Expected seam-local slice themes

- wrapper pass-through changes
- installation-doc propagation
- env-contract propagation
- seam-exit evidence for parity and no-drift propagation

## Expected seam-exit concerns

### Contracts likely to publish

- `C-08`
- `C-09`

### Threads likely to advance

- `THR-05` to `published`

### Review-surface areas likely to shift after landing

- operator-facing surface R2
- validation topology R3 because docs and wrapper become assertion inputs

### Downstream seams most likely to require revalidation

- `SEAM-06`
- `SEAM-07`
