# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the standard feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch. No worktree is used.
- Standard: `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/quality_gate_report.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `tasks.json`; add START entry to `session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/execution_preflight_report.md` with a concrete recommendation:
- **ACCEPT**: triads may begin.
- **REVISE**: do not start triads until the listed issues are fixed and the preflight is re-run.

At minimum, verify:
- The planning quality gate remains `ACCEPT`.
- `tasks.json` keeps schema v4, `cross_platform=true`, `execution_gates=true`, `automation.enabled=true`, and `meta.checkpoint_boundaries=["LAITDP1","LAITDP2"]`.
- `LAITDP0-code` and `LAITDP0-test` both depend on `F0-exec-preflight`.
- `execution_preflight_report.md` and the three slice closeout reports exist.
- This pack currently has no `smoke/` directory and its current touch set is planning/spec/docs-only, so the preflight must record the docs-only CI/smoke posture explicitly rather than inventing execution smoke coverage.
- The advisory CI audit + evidence ledger tooling exists and can be referenced by later checkpoint tasks:
  - `scripts/ci-audit/ci_audit.sh`
  - `scripts/ci-audit/ci_audit_record.sh`
- The validator suite is green on the orchestration checkout:
  - `jq -e . docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json >/dev/null`
  - `python3 docs/project_management/system/scripts/planning/validate_tasks_json.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`
  - `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture"`
  - `make planning-micro-lint FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" OWNED_PATHS="plan.md tasks.json session_log.md quality_gate_report.md execution_preflight_report.md kickoff_prompts slices/LAITDP0 slices/LAITDP1 slices/LAITDP2"`

## End Checklist

1. Update `execution_preflight_report.md`.
2. Set `F0-exec-preflight` status to `completed` in `tasks.json`; add END entry to `session_log.md`; commit docs (`docs: finish F0-exec-preflight`).
3. Do not start `LAITDP0-code` or `LAITDP0-test` until the report recommends `ACCEPT`.
