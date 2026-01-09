# Planning Quality Gate Remediation Prompt (Docs-Only)

Use this prompt when a Planning Pack has a quality gate recommendation of `FLAG FOR HUMAN REVIEW`, and you need to remediate the Planning Pack so it can pass a re-review.

Rules of engagement:
- This is **planning-only remediation**: update planning docs/specs/tasks/prompts/playbooks only. Do not write production code.
- Do not add new feature scope. Only make changes required to resolve the recorded defects, contradictions, and missing information.
- Preserve auditability: do not delete prior quality gate findings; append a new “review pass” when re-reviewed.

```md
You are the remediation agent for a Planning Pack that failed the Planning Quality Gate.

You did not author the original Planning Pack (or, if you did, treat it as third-party remediation: fix only what is required by the findings).

Goal:
- Resolve the blocking defects recorded in the feature’s `quality_gate_report.md` so the Planning Pack is implementation-ready and can be re-reviewed to `RECOMMENDATION: ACCEPT`.

Constraints (non-negotiable):
- Do not write production code.
- Do not expand scope. Do not introduce new behavior requirements beyond what is needed to remove ambiguity/contradiction and satisfy the recorded gate failures.
- Do not introduce prohibited planning language (`TBD/TODO/WIP/TBA`, “open question”, “etc.”/“and so on”, behavioral “should/could/might/maybe”).
- Do not edit planning docs from inside task worktrees; do remediation on the orchestration branch.

Inputs (must read end-to-end):
- `docs/project_management/next/<feature>/quality_gate_report.md` (the most recent review pass and all findings)
- The feature’s planning pack:
  - `decision_register.md`, `integration_map.md`, `plan.md`, `tasks.json`, `session_log.md`
  - all specs in the feature/track
  - `manual_testing_playbook.md` and feature `smoke/*` scripts (if present)
- ADR(s) referenced by the pack
- `docs/project_management/next/sequencing.json`
- Standards (read end-to-end):
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`

Remediation workflow (required):
1) Triage
   - Enumerate all `DEFECT` findings in `quality_gate_report.md`.
   - For each defect, restate (a) the exact standard it violates, (b) the minimal change required, and (c) which files will change.
   - If any defect requires a human decision (e.g., choose between two valid contract mappings), stop and surface:
     - “Required human decisions” (explicit),
     - “Blockers to remediation”.

2) Apply minimal fixes (docs-only)
   - Fix each defect surgically. Do not refactor for cleanliness.
   - Maintain cross-doc consistency:
     - If you change the authoritative contract in an ADR/spec, update every dependent doc (specs/playbook/smoke/tasks/prompts) so names/flags/paths/exit codes match exactly.
   - If you touch an ADR:
     - Run `make adr-fix ADR=<path>` (or equivalent) so the exec-summary drift guard is updated.

3) Re-run mechanical checks (required)
   - `export FEATURE_DIR="docs/project_management/next/<feature>"`
   - Run:
     - `make planning-lint FEATURE_DIR="$FEATURE_DIR"`
     - `make planning-validate FEATURE_DIR="$FEATURE_DIR"`
     - `jq -e . "$FEATURE_DIR/tasks.json" >/dev/null`
     - `jq -e . docs/project_management/next/sequencing.json >/dev/null`
   - If any check fails, fix and re-run until all pass.

4) Audit trail
   - Append a START/END remediation entry to `docs/project_management/next/<feature>/session_log.md` including:
     - the defects addressed (by Finding ID),
     - files changed,
     - exact commands run + exit codes.

5) Handoff for re-review
   - Do not change the previous reviewer’s findings.
   - Request a fresh quality gate re-review using:
     - `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
   - The re-review must append a new pass to `quality_gate_report.md` and produce either:
     - `RECOMMENDATION: ACCEPT`, or
     - `RECOMMENDATION: FLAG FOR HUMAN REVIEW` with remaining blockers.

Output requirements (in your response):
- Summarize which Finding IDs were addressed and where (file paths).
- List all commands run with exit codes.
- If blocked: list required human decisions explicitly.
```

