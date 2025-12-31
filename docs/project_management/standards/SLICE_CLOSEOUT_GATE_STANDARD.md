# Slice Closeout Gate Standard (Per-Slice End Gate)

Goal:
- Ensure each slice ends with **no drift** between the slice spec and what shipped, with durable evidence (commands run + results).

This gate runs after `X-integ` (or after `X-integ` final in the platform-fix model).

## When it runs
- At the end of each slice, after the integration task is green and merged back to the orchestration branch.
- Before starting the next slice (recommended) to keep drift contained.

## Required artifacts
For features that opt in (`tasks.json` meta: `execution_gates: true`):
- A closeout report per slice:
  - `docs/project_management/next/<feature>/<SLICE>-closeout_report.md` (e.g., `C0-closeout_report.md`)
- The slice final integration task (`<SLICE>-integ`) must reference the closeout report and require completing it.

## What it checks (no ambiguity)

### 1) Spec parity (“no drift”)
- Confirm the final behavior matches the slice spec’s acceptance criteria.
- Record any spec changes that occurred during the slice (and why they were required).

### 2) Cross-platform parity (when applicable)
- Record CI run ids/URLs for each required platform smoke run.
- If any platform-fix work happened:
  - record what was fixed and where (guards, path handling, deps),
  - confirm the merged result is green on all required platforms.

### 3) Smoke ↔ Manual parity (quality of validation)
- Confirm smoke scripts still represent the minimal runnable version of the manual testing playbook:
  - smoke runs the same commands/workflows,
  - smoke checks the same outputs/exit codes (at least the critical ones).

### 4) Audit trail completeness
- The slice closeout report includes:
  - key commands run (fmt/clippy/tests/integ checks),
  - smoke dispatch commands and results,
  - references to the slice spec + any relevant ADR/contract sections.

## Output and rules
- Fill `<SLICE>-closeout_report.md` and treat it as required evidence for moving on.
- If drift is discovered:
  - fix the implementation/tests/spec first,
  - then re-run the relevant checks and update the closeout report.

