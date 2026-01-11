# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and runs on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/_archived/world-overlayfs-enumeration/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration"`
2. Read: ADR + Executive Summary, `docs/project_management/_archived/world-overlayfs-enumeration/plan.md`, `docs/project_management/_archived/world-overlayfs-enumeration/tasks.json`, `docs/project_management/_archived/world-overlayfs-enumeration/session_log.md`, `docs/project_management/_archived/world-overlayfs-enumeration/WO0-spec.md`, and this prompt.
3. Set `F0-exec-preflight` status to `in_progress` in `tasks.json`; add START entry to `session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements

Fill `docs/project_management/_archived/world-overlayfs-enumeration/execution_preflight_report.md` with a concrete recommendation:
- **ACCEPT**: triads may begin.
- **REVISE**: do not start triads until the listed issues are resolved and the preflight is re-run.

At minimum, verify:
- The Planning Pack is complete and internally consistent (`plan.md`, `tasks.json`, `session_log.md`, spec, kickoff prompts).
- Smoke scripts run real commands/workflows and validate exit codes and key output.
- Manual testing playbook steps are runnable and aligned to smoke coverage.

## End Checklist

1. Set `F0-exec-preflight` status to `completed` in `tasks.json`; add END entry to `session_log.md` (include the recommendation and any required fixes).
2. Commit docs (`docs: finish F0-exec-preflight`).

