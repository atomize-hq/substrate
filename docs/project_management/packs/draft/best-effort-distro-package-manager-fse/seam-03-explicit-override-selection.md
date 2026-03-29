---
seam_id: SEAM-03
seam_slug: explicit-override-selection
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

# SEAM-03 - Explicit Override Selection

## Goal / value

Own the operator-controlled selector stages so forced manager selection is explicit, deterministic, and fail-closed before fallback behavior is considered.

## Scope

### In

- `--pkg-manager`
- `PKG_MANAGER`
- supported explicit values
- selector precedence between flag and env
- `pkg_manager.source=flag|env`
- exit `2` invalid-override posture
- exit `3` explicit-manager-missing posture

### Out

- parser/input ownership
- family mapping and decision-line foundation
- ordered PATH fallback, warning, and exit `4`
- wrapper/docs propagation
- validation topology ownership

## Primary interfaces

### Inputs

- CLI args
- environment variables
- upstream mapping/reporting truth for lower-precedence fallback stages

### Outputs

- explicit manager selection
- `pkg_manager.source=flag|env`
- exit `2` / `3` contract publication

## Key invariants / rules

1. flag outranks env
2. valid explicit selectors never fall through to lower-precedence stages
3. invalid explicit selectors fail with exit `2`
4. explicit selectors missing from `PATH` fail with exit `3`

## Dependencies

### Direct blockers

- `SEAM-01`
- `SEAM-02`

### Transitive blockers

- None

### Direct consumers

- `SEAM-04`
- `SEAM-05`
- `SEAM-06`

### Derived consumers

- operators and env docs

## Touch surface

- `scripts/substrate/install-substrate.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- source evidence in `decision_register.md` and `BEDPM1-spec.md`

## Verification

- flag beats env
- env beats os-release mapping and path probe
- invalid flag or env produces exit `2` with required remediation elements
- explicit selected manager missing from `PATH` produces exit `3` with required remediation elements

## Risks / unknowns

- selector and fallback responsibilities bleeding together during decomposition
- docs or tests narrowing or widening the allowed vocabulary
- explicit-stage failures being collapsed by later propagation seams

## Rollout / safety

- fail-closed posture is the safety boundary for operator-forced behavior
- this seam must not silently recover from invalid or unavailable explicit selectors

## Downstream decomposition context

### Why this seam is `future`

It depends on both parser/input and mapping/reporting truth being landed first. Extraction keeps it at seam-brief depth because later review must validate the shared decision pipeline with `SEAM-04`.

### Which threads matter most

- `THR-02`
- `THR-03`

### What the first seam-local review should focus on

- selector precedence boundaries
- exit `2` / `3` wording and evidence requirements
- decision-line interaction when an explicit selector succeeds

### Expected seam-local slice themes

- flag parsing and validation
- env override handling
- explicit-stage failure branches
- seam-exit publication of explicit-selector truth

## Expected seam-exit concerns

### Contracts likely to publish

- `C-05`
- `C-06`

### Threads likely to advance

- `THR-03` to `published`

### Review-surface areas likely to shift after landing

- decision pipeline R1
- operator-facing surface R2

### Downstream seams most likely to require revalidation

- `SEAM-04`
- `SEAM-05`
- `SEAM-06`
