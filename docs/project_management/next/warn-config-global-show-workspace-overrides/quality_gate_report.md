# Planning Gate Report — warn-config-global-show-workspace-overrides

Date (UTC): 2026-01-30T00:04:13Z

Reviewer: (pending)

Plan: `docs/project_management/next/warn-config-global-show-workspace-overrides/plan.md`
ADR: `docs/project_management/next/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`

## Recommendation

**FLAG FOR HUMAN REVIEW**

Reason:
- Planning pack was generated/filled, but local lint/quality checks were not run in this environment
  (owner will run them).

## Checklist (what to validate)

### Doc completeness
- [ ] Plan/spec/contract/decision register are present and consistent.
- [ ] Tasks are triad-structured and references resolve.
- [ ] Manual testing playbook exists and is mirrored by smoke scripts.
- [ ] ADR exists and has a valid `ADR_BODY_SHA256` (run `make adr-check` or the script below).

### Suggested commands (run by owner)

> NOTE: Do not run these in this environment if you are deferring linting/quality gates to a separate workflow.

- Planning lint:
  - `scripts/planning/lint.sh --feature-dir docs/project_management/next/warn-config-global-show-workspace-overrides`
- Tasks schema check:
  - `scripts/planning/check_tasks_json.py --feature-dir docs/project_management/next/warn-config-global-show-workspace-overrides`
- ADR exec summary drift check:
  - `scripts/planning/check_adr_exec_summary.py --adr docs/project_management/next/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- Kickoff prompt sentinel check:
  - `scripts/planning/ensure_kickoff_prompt_sentinel.py --feature-dir docs/project_management/next/warn-config-global-show-workspace-overrides`

## Evidence

- Commands run: (pending)
- Outputs/artifacts: (pending)
