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
- Keep Windows in the checkpoint parity set while limiting required feature smoke to the declared behavior platforms.
- Do not drop slices, downgrade schema shape, or disable checkpoint boundaries to satisfy validation.

## Accepted Slice Ordering
- `LAITDP0`: identity contract and schema lock.
- `LAITDP1`: policy and observability alignment lock.
- `LAITDP2`: platform rollout and validation lock.

## Checkpoint Rule
- `CP1-ci-checkpoint` closes the first checkpoint group after `LAITDP1-integ-core`.
- `CP2-ci-checkpoint` closes the second checkpoint group after `LAITDP2-integ-core`.
- `tasks.json` `meta.checkpoint_boundaries` stays `["LAITDP1", "LAITDP2"]`.
- `tasks.json` `meta.behavior_platforms_required` stays `["linux", "macos"]`.
- `tasks.json` `meta.ci_parity_platforms_required` stays `["linux", "macos", "windows"]`.
- `LAITDP1` and `LAITDP2` use the full boundary model: `*-integ-core`, `*-integ-linux`, `*-integ-macos`, `*-integ-windows`, and final `*-integ`.
- `*-integ-windows` remains a parity-only follow-up task. It does not make Windows feature smoke a required gate.
- `LAITDP0` stays a normal schema-v4 slice with `LAITDP0-integ` as the only integration merge task.

## Execution Gate
- `F0-exec-preflight` is the first task for this pack.
- It fills `execution_preflight_report.md`, re-confirms `quality_gate_report.md` stays `ACCEPT`, reruns the pack validators on the orchestration checkout, and verifies `LAITDP0-code` and `LAITDP0-test` remain blocked on preflight completion.
- This pack uses the standard execution-gate lane expected by the triad wrapper prompts and automation helpers.

## Validation Discipline
- Run `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`.
- Run `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" OWNED_PATHS="plan.md tasks.json pre-planning/ci_checkpoint_plan.md kickoff_prompts slices/LAITDP0/kickoff_prompts slices/LAITDP1/kickoff_prompts slices/LAITDP2/kickoff_prompts"`.

## Change Boundary
- This pass is limited to task-graph wiring, checkpoint wiring, execution-gate posture, and kickoff prompt generation needed to keep Windows parity-only in CI.
- Semantic contract docs stay unchanged except where they must describe the updated behavior-platform versus CI-parity split.
