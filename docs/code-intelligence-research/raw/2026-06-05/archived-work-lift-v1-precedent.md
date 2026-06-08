Status: exploratory
Scope: lift
Authority: non-canonical
Artifact-boundary impact: none

# Archived Work Lift v1 Precedent

This note preserves the practical precedent gathered from the archived
`system` tree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system`

It is precedent only, not authority for the current `9b83/substrate` worktree.

## Key implementation files

- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py)

## v1 input surface

- `touch.create_files`
- `touch.edit_files`
- `touch.delete_files`
- `touch.deprecate_files`
- `touch.crates_touched`
- `touch.boundary_crossings`
- `contract.cli_flags`
- `contract.config_keys`
- `contract.exit_codes`
- `contract.file_formats`
- `contract.behavior_deltas`
- `qa.new_test_files`
- `qa.new_test_cases`
- `docs.new_docs_files`
- `ops.new_smoke_steps`
- `ops.ci_changes`
- `risk.cross_platform`
- `risk.security_sensitive`
- `risk.concurrency_or_ordering`
- `risk.migration_or_backfill`
- `risk.unknowns_high`

## v1 scoring shape

- additive weighted base
- boolean risk multipliers
- unknowns addend
- `lift_score = ceil(score_unrounded)`
- `estimated_slices = max(1, ceil(lift_score / 12))`

### Confirmed weights and multipliers

- `create_files = 3`
- `edit_files = 2`
- `delete_files = 1`
- `deprecate_files = 1`
- `crates_touched = 4`
- `boundary_crossings = 3`
- `cli_flags = 3`
- `config_keys = 3`
- `exit_codes = 4`
- `file_formats = 5`
- `behavior_deltas_blowup = 10 * max(0, behavior_deltas - 1)`
- `new_test_files = 2`
- `new_test_cases = 1`
- `new_docs_files = 2`
- `new_smoke_steps = 3`
- `ci_changes = 3`
- `cross_platform = 1.15`
- `security_sensitive = 1.20`
- `concurrency_or_ordering = 1.15`
- `migration_or_backfill = 1.25`
- `unknowns_high_multiplier = 2`

## Confirmed signal paths

1. intake or ADR markdown with a fenced JSON Lift Vector block
2. planning-pack or impact-map derivation from `validate_impact_map.py --emit-json`
3. git-diff calibration through `git diff --name-status -M`

## Exact confidence mechanics preserved from v1

- missing or `null` numeric inputs scored as `0`
- those missing fields were still appended to `missing_inputs`
- any non-empty `missing_inputs` forced `confidence = low`
- prefix-derived touch sets also forced `confidence = low`

## Strong precedent worth keeping

- strict structured input contract
- deterministic scoring once the vector exists
- explicit `missing_inputs` tracking
- explicit confidence degradation when evidence is missing or prefix-derived
- explicit split triggers
- advisory-first posture

## Limitations worth not carrying forward unchanged

- heavy reliance on AI-authored or human-authored coarse vector counts
- shallow file-touch proxies as the core planner model
- no rich semantic dependency graph as the primary substrate
- scalar-first interpretation where richer graph structure is now plausible

## Old-to-new mapping worth preserving

### Keep

- structured vector or schema discipline
- deterministic computation
- explicit `missing_inputs` reporting
- explicit confidence degradation
- trigger-first and advisory-first interpretation

### Upgrade

- touch-only reasoning into graph-backed repository reasoning
- single coarse risk intuition into explicit dependency plus conflict structure
- generic unknowns into more structured missing-signal and confidence metadata
- optional diff calibration into richer causal and historical evidence

### Replace

- AI-authored coarse raw counts where deterministic extraction can derive the
  same signal family
- scalar-only planning posture
- file-list-only parallelization heuristics

## Confirmed trigger behavior preserved from v1

### Code-evaluated triggers

- `split_required:behavior_deltas>1`
- `likely_split:crates_touched>2`
- `likely_split:touch_files_sum>12`
- `likely_split:contract_surface_sum>4`
- `likely_split:lift_score>24`
- `split_required:estimated_slices>3`

### Prompt-guidance threshold

- `lift_score > 60` appeared in some user-facing guidance as a strong split
  signal, but this was prompt guidance rather than a trigger evaluated by the
  implementation itself

### Implementation quirk

- the config declared both `adr_candidate` and `workstream` trigger families
- `pm_lift.py` v1 only evaluated the `adr_candidate` rules in code

## Boundary caution

This note is here so the archived v1 example set stays locally referenceable.
It does not justify copying the archived model forward unchanged.
