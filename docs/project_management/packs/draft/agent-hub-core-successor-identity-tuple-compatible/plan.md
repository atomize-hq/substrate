# agent-hub-core-successor-identity-tuple-compatible — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
- Orchestration branch: `feat/agent-hub-core-successor-identity-tuple-compatible`
- Planning surfaces in scope:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `pre-planning/alignment_report.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `slices/AHCSITC0/AHCSITC0-spec.md`
  - `slices/AHCSITC1/AHCSITC1-spec.md`
  - `slices/AHCSITC2/AHCSITC2-spec.md`
  - `slices/AHCSITC3/AHCSITC3-spec.md`
  - `kickoff_prompts/`
  - `slices/AHCSITC0/kickoff_prompts/`
  - `slices/AHCSITC1/kickoff_prompts/`
  - `slices/AHCSITC2/kickoff_prompts/`
  - `slices/AHCSITC3/kickoff_prompts/`

## Goal
- Produce an execution-ready schema-v4 triad pack for `AHCSITC0` through `AHCSITC3`.
- Keep the accepted slice set intact and wire the checkpoint boundaries exactly at `AHCSITC2` and `AHCSITC3`.
- Keep slice-spec acceptance-criteria traceability exact for `*-code`, `*-test`, and final `*-integ` tasks.

## Guardrails
- Specs remain the source of truth for slice intent, sequencing, and acceptance criteria.
- Planning docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Keep automation enabled, the accepted slice order intact, and the pack cross-platform.
- Keep the schema-v4 checkpoint-boundary model intact for `AHCSITC2` and `AHCSITC3`.
- Do not drop slices, disable automation, or rewrite checkpoint boundaries to satisfy validators.

## Accepted Slice Ordering
- `AHCSITC0`: operator contract and identity presentation lock.
- `AHCSITC1`: orchestrator and member session protocol lock.
- `AHCSITC2`: fail-closed policy and telemetry publication lock.
- `AHCSITC3`: platform parity, compatibility, and validation closure.

## Checkpoint Rule
- `CP1-ci-checkpoint` validates the checkpoint group ending at `AHCSITC2`.
- `CP2-ci-checkpoint` validates the checkpoint group ending at `AHCSITC3`.
- `tasks.json` `meta.checkpoint_boundaries` stays `["AHCSITC2", "AHCSITC3"]`.
- `tasks.json` `meta.ci_parity_platforms_required` stays `["linux", "macos", "windows"]`.
- `tasks.json` `meta.behavior_platforms_required` stays `[]` until a later execution lane adds feature smoke scripts and explicitly widens the behavior-platform requirement.
- `AHCSITC0` and `AHCSITC1` stay normal schema-v4 slices with a single final `*-integ` task.
- `AHCSITC2` and `AHCSITC3` use the full boundary model: `*-integ-core`, `*-integ-linux`, `*-integ-macos`, `*-integ-windows`, and final `*-integ`.
- The platform-fix tasks for this pack are parity-only follow-up tasks. They do not imply required feature-smoke coverage.

## Validation Discipline
- Run `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"`.
- Run `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" OWNED_PATHS="plan.md tasks.json session_log.md quality_gate_report.md kickoff_prompts slices/AHCSITC0/kickoff_prompts slices/AHCSITC1/kickoff_prompts slices/AHCSITC2/kickoff_prompts slices/AHCSITC3/kickoff_prompts"`.

## Change Boundary
- This pass is limited to task-graph wiring, checkpoint wiring, and kickoff prompt generation for the accepted slice set.
- Topic specs, ADRs, and non-allowlisted tracked docs stay unchanged in this pass.
