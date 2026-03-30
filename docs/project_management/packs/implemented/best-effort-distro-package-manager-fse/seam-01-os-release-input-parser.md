---
seam_id: SEAM-01
seam_slug: os-release-input-parser
type: domain
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v2
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - os-release parsing rules change
    - alternate-input hook semantics change
    - `<unknown>` sentinel semantics change
    - downstream persistence pack diverges from inherited input truth
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

# SEAM-01 - os-release Input And Parser Contract

## Goal / value

Own the selected Linux input path, safe parser posture, normalized distro fields, and `<unknown>` behavior so every later seam starts from one trustworthy source of host truth.

## Scope

### In

- selected os-release input path selection
- `SUBSTRATE_INSTALL_OS_RELEASE_PATH` validation and absence semantics
- line-oriented extraction of `ID` and `ID_LIKE`
- normalization, quote stripping, duplicate-key handling, comment handling
- `<unknown>` sentinel semantics for missing or unusable input
- Linux-only, no-network, no-shell-execution posture

### Out

- distro-family mapping and availability-based selection
- stable decision-line wording
- explicit selectors and failure taxonomy
- wrapper/docs propagation
- validation harness ownership

## Primary interfaces

### Inputs

- `/etc/os-release`
- `SUBSTRATE_INSTALL_OS_RELEASE_PATH`

### Outputs

- normalized `distro_id`
- normalized `distro_id_like`
- `<unknown>` sentinel behavior
- parser/input contract published as `C-01` and `C-02`

## Key invariants / rules

1. parser never executes os-release content
2. only `ID` and `ID_LIKE` are read
3. invalid alternate input degrades to `<unknown>` and does not fall back to `/etc/os-release`
4. parser output stays deterministic for the same selected input

## Dependencies

### Direct blockers

- None

### Transitive blockers

- None

### Direct consumers

- `SEAM-02`
- `SEAM-03`
- `SEAM-04`
- `SEAM-06`

### Derived consumers

- downstream persistence pack

## Touch surface

- `scripts/substrate/install-substrate.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- source evidence in `decision_register.md`, `spec_manifest.md`, and `BEDPM0-spec.md`

## Verification

- parser produces normalized distro fields for accepted sample inputs
- invalid or unreadable alternate input yields `<unknown>` without fallback to `/etc/os-release`
- duplicate, quoted, and comment-bearing lines follow the selected parser rules
- downstream seams can consume parser/input truth without redefining it

## Risks / unknowns

- parser rule drift between source contract and implementation
- accidental source-time regressions in `install-substrate.sh`
- downstream persistence assuming unapproved parser behavior

## Rollout / safety

- Linux-only behavior delta
- no new config file or telemetry surface
- safe parser posture is the core risk reducer for the entire feature

## Downstream decomposition context

### Why this seam is `active`

Every later seam depends on trusted input truth. If this seam is vague, downstream decomposition would have to reopen parser, hook, and sentinel decisions.

### Which threads matter most

- `THR-01`
- `THR-07`

### What the first seam-local review should focus on

- selected-input precedence and invalid-path posture
- parser rule completeness
- `<unknown>` degradation boundary
- source-time safety of shell changes

### Expected seam-local slice themes

- selected-input hook handling
- parser-rule implementation
- sentinel and degradation behavior
- seam-exit publication of input-truth contracts

## Expected seam-exit concerns

### Contracts likely to publish

- `C-01`
- `C-02`

### Threads likely to advance

- `THR-01` to `published`
- `THR-07` to `published`

### Review-surface areas likely to shift after landing

- selected-input path notes in review surface R1
- downstream handoff assumptions in R4

### Downstream seams most likely to require revalidation

- `SEAM-02`
- `SEAM-03`
- `SEAM-04`
- `SEAM-06`
