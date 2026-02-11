# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Docs-only task; runs on the orchestration checkout (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/execution_preflight_report.md`

## Start checklist
1. Ensure orchestration branch is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
2. Read: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/plan.md`, `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/tasks.json`, specs, and this prompt.

## Requirements
- Do not edit planning docs inside the worktree.
- Run:
  - `make planning-validate FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
  - `make planning-lint FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
  - `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/_core.sh`
  - `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/linux-smoke.sh`
  - `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh`
- Update `execution_preflight_report.md` with an explicit recommendation:
  - **ACCEPT**: triads may begin
  - **REVISE**: do not start triads until the listed issues are fixed

## End checklist
1. Update `execution_preflight_report.md`.
2. Mark task complete (operator-owned): update `tasks.json` + `session_log.md` on orchestration branch.
