# Kickoff: WDH1-integ (integration)

## Scope
- Final CP1 integration: core + any platform-fix branches (linux/macos/windows as needed).
- Spec: `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`

## Requirements
- Do not edit planning docs inside the worktree.
- Confirm CP1 checkpoint evidence is recorded in `docs/project_management/next/world-deps-host-visible-hardening/session_log.md`.
- Merge required platform-fix branches (if any) and run `make integ-checks`.

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WDH1-integ"`
