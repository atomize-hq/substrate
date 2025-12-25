# Planning Quality Gate Report Template

This file is a template. The **quality gate reviewer** must copy it into the feature Planning Pack as:

- `docs/project_management/next/<feature>/quality_gate_report.md`

The report is an auditable artifact. It is required before execution triads begin.

---

# Planning Quality Gate Report — <feature>

## Metadata
- Feature directory: `docs/project_management/next/<feature>/`
- Reviewed commit: `<git sha>`
- Reviewer: `<name/role>`
- Date (UTC): `<YYYY-MM-DD>`
- Recommendation: `ACCEPT` | `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)
Paste the exact commands run and their results/exit codes.

### Required preflight (minimum)

These are the minimum commands that must appear in this report (or an explicit equivalent), with exit codes recorded:

```bash
# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
jq -e . docs/project_management/next/sequencing.json >/dev/null

# tasks.json required-field audit
python - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"] if isinstance(data,dict) and "tasks" in data else data
required=[
  "id","name","type","phase","status","description",
  "references","acceptance_criteria","start_checklist","end_checklist",
  "worktree","integration_task","kickoff_prompt",
  "depends_on","concurrent_with"
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get("id","<no id>"),m))
if missing:
  for tid,m in missing:
    print(tid,":",", ".join(m))
  raise SystemExit(1)
print("OK: tasks.json required fields present")
PY
```

### Planning lint (mechanical)
Reference: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`

- `<command>` → `<exit code>` → `<notes>`
- `<command>` → `<exit code>` → `<notes>`

### Additional review commands (if any)
- `<command>` → `<exit code>` → `<notes>`

## Required Inputs Read End-to-End (checklist)
Mark `YES` only if read end-to-end.

- ADR(s): `YES|NO`
- `plan.md`: `YES|NO`
- `tasks.json`: `YES|NO`
- `session_log.md`: `YES|NO`
- All specs in scope: `YES|NO`
- `decision_register.md` (if present/required): `YES|NO|N/A`
- `integration_map.md` (if present/required): `YES|NO|N/A`
- `manual_testing_playbook.md` (if present/required): `YES|NO|N/A`
- Feature smoke scripts under `smoke/` (if required): `YES|NO|N/A`
- `docs/project_management/next/sequencing.json`: `YES|NO`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES|NO`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES|NO`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS|FAIL`
- Evidence: `<file path + exact section>`
- Notes: `<what was verified or what violates the rule>`

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS|FAIL|N/A`
- Evidence: `<decision_register.md DR-xxxx, and supporting specs>`
- Notes:

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS|FAIL`
- Evidence:
- Notes:

### 4) Sequencing and dependency alignment
- Result: `PASS|FAIL`
- Evidence:
  - `docs/project_management/next/sequencing.json` entries: `<ids>`
  - `docs/project_management/next/<feature>/tasks.json` deps: `<ids>`
- Notes:

### 5) Testability and validation readiness
- Result: `PASS|FAIL`
- Evidence:
  - Manual playbook sections: `<paths>`
  - Smoke scripts: `<paths>`
  - `tasks.json` integration end_checklist includes smoke: `<task ids>`
- Notes:

### 6) Triad interoperability (execution workflow)
- Result: `PASS|FAIL`
- Evidence:
  - `tasks.json` required fields present
  - kickoff prompts include “no docs edits in worktrees”
- Notes:

## Findings (must be exhaustive)

### Finding 001 — <title>
- Status: `VERIFIED` | `DEFECT`
- Evidence: `<file path + exact location>`
- Impact: `<why it matters>`
- Fix required (exact): `<single, explicit change>`
- If DEFECT: Alternative (one viable): `<option + why>`

### Finding 002 — <title>
- Status: `VERIFIED` | `DEFECT`
- Evidence:
- Impact:
- Fix required (exact):
- If DEFECT: Alternative (one viable):

## Decision: ACCEPT or FLAG

### If ACCEPT
- Summary: `<why it is execution-ready>`
- Next step: “Execution triads may begin.”

### If FLAG FOR HUMAN REVIEW
- Summary: `<why it is not execution-ready>`
- Required human decisions (explicit): `<list>`
- Blockers to execution: `<list>`
