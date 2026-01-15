# Planning Quality Gate Report — workspace-config-policy-unification

## Metadata
- Feature directory: `docs/project_management/next/workspace-config-policy-unification/`
- Reviewed commit: `<git sha>`
- Reviewer: `<name/role>`
- Date (UTC): `<YYYY-MM-DD>`
- Recommendation: `ACCEPT` | `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)
Paste the exact commands run and their results/exit codes.

### Required preflight (minimum)
```bash
FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
jq -e . docs/project_management/next/sequencing.json >/dev/null

make planning-validate FEATURE_DIR="$FEATURE_DIR"
make planning-lint FEATURE_DIR="$FEATURE_DIR"
```

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES|NO`
- `plan.md`: `YES|NO`
- `tasks.json`: `YES|NO`
- `session_log.md`: `YES|NO`
- All specs in scope: `YES|NO`
- `decision_register.md`: `YES|NO`
- `integration_map.md`: `YES|NO`
- `manual_testing_playbook.md`: `YES|NO`
- Feature smoke scripts under `smoke/`: `YES|NO`
- `docs/project_management/next/sequencing.json`: `YES|NO`

## Findings (must be exhaustive)

### Finding 001 — Phase A/B gates are explicitly encoded
- Status: `VERIFIED|DEFECT`
- Evidence:
  - `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
  - `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Impact:
- Fix required (exact):

## Decision: ACCEPT or FLAG

### If ACCEPT
- Summary:
- Next step: “Execution triads may begin.”

### If FLAG FOR HUMAN REVIEW
- Summary:
- Required human decisions (explicit):
- Blockers to execution:
