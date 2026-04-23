# llm-and-agent-identity-tuple-and-deployment-posture — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- Orchestration branch: `feat/llm-and-agent-identity-tuple-and-deployment-posture`
- Planning surfaces in scope:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `pre-planning/alignment_report.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `slices/LAITDP0/LAITDP0-spec.md`
  - `slices/LAITDP1/LAITDP1-spec.md`
  - `slices/LAITDP2/LAITDP2-spec.md`
  - `kickoff_prompts/`
  - `slices/LAITDP0/kickoff_prompts/`
  - `slices/LAITDP1/kickoff_prompts/`
  - `slices/LAITDP2/kickoff_prompts/`

## Goal
- Produce an execution-ready schema-v4 triad pack for `LAITDP0` through `LAITDP2`.
- Keep the accepted slice set intact and wire the checkpoint boundaries exactly at `LAITDP1` and `LAITDP2`.
- Keep slice-spec acceptance-criteria traceability exact for `*-code`, `*-test`, and `*-integ`.

## Guardrails
- Specs remain the source of truth for slice intent and acceptance criteria.
- Planning docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Keep automation enabled and cross-platform checkpoint wiring intact.
- Do not drop slices, downgrade schema shape, or disable checkpoint boundaries to satisfy validation.

## Accepted Slice Ordering
- `LAITDP0`: identity contract and schema lock.
- `LAITDP1`: policy and observability alignment lock.
- `LAITDP2`: platform rollout and validation lock.

## Checkpoint Rule
- `CP1-ci-checkpoint` closes the first checkpoint group after `LAITDP1-integ-core`.
- `CP2-ci-checkpoint` closes the second checkpoint group after `LAITDP2-integ-core`.
- `tasks.json` `meta.checkpoint_boundaries` stays `["LAITDP1", "LAITDP2"]`.
- `LAITDP1` and `LAITDP2` use the full boundary model: `*-integ-core`, `*-integ-linux`, `*-integ-macos`, `*-integ-windows`, and final `*-integ`.
- `LAITDP0` stays a normal schema-v4 slice with `LAITDP0-integ` as the only integration merge task.

## Validation Discipline
- Run `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" OWNED_PATHS="plan.md tasks.json pre-planning/ci_checkpoint_plan.md kickoff_prompts slices/LAITDP0/kickoff_prompts slices/LAITDP1/kickoff_prompts slices/LAITDP2/kickoff_prompts"`.

## Change Boundary
- This pass is limited to task-graph wiring, checkpoint wiring, quality-gate scaffolding, and kickoff prompt generation.
- `contract.md`, `identity-tuple-schema-spec.md`, `policy-spec.md`, `telemetry-spec.md`, `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md` stay unchanged in this pass.
