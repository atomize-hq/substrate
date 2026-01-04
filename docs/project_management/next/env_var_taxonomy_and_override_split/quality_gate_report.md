# Planning Quality Gate Report — env_var_taxonomy_and_override_split

## Metadata
- Feature directory: `docs/project_management/next/env_var_taxonomy_and_override_split/`
- Reviewed commit: `<git sha>`
- Reviewer: `<name/role>`
- Date (UTC): `<YYYY-MM-DD>`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split"
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
jq -e . docs/project_management/next/sequencing.json >/dev/null
make planning-validate FEATURE_DIR="$FEATURE_DIR"
make planning-lint FEATURE_DIR="$FEATURE_DIR"
```

## Required Inputs Read End-to-End (checklist)
- ADR(s): `NO`
- `plan.md`: `NO`
- `tasks.json`: `NO`
- `session_log.md`: `NO`
- All specs in scope: `NO`
- `decision_register.md`: `NO`
- `integration_map.md`: `NO`
- `manual_testing_playbook.md`: `NO`
- Feature smoke scripts under `smoke/`: `NO`
- `docs/project_management/next/sequencing.json`: `NO`
- Standards: `NO`

## Findings (if any)

### Finding 001 — <title>
- Status: `VERIFIED` | `DEFECT`
- Evidence: `<file path + exact location>`
- Impact: `<why it matters>`
- Fix required (exact): `<single, explicit change>`

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: This report is a stub and must be authored by a third-party quality gate reviewer.
- Blockers to execution: A reviewer must run the mechanical checks and update this report with evidence and a final recommendation.

