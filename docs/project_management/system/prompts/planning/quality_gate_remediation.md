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
- `<FEATURE_DIR>/quality_gate_report.md` (the most recent review pass and all findings)
- The feature’s planning pack:
  - `spec_manifest.md`
  - `decision_register.md`, `impact_map.md`, `plan.md`, `tasks.json`, `session_log.md`
  - all specs in the feature/track
  - `manual_testing_playbook.md`
  - feature `smoke/*` scripts when `tasks.json` `meta.behavior_platforms_required` is non-empty or the quality gate findings say smoke coverage is required
- ADR(s) referenced by the pack
- `docs/project_management/packs/sequencing.json`
- Standards (read end-to-end):
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
  - `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
  - `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`
  - `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`
  - `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`

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
   - If the feature is cross-platform (`tasks.json` `meta.cross_platform=true`), ensure the remediation preserves the required execution model:
     - Platform scopes are explicit and non-overlapping:
       - `meta.behavior_platforms_required` (smoke-required platforms),
       - `meta.ci_parity_platforms_required` (compile/CI parity platforms).
     - If `meta.behavior_platforms_required` is non-empty, the pack must include concrete feature smoke scripts under `<FEATURE_DIR>/smoke/` for each required behavior platform, and `manual_testing_playbook.md` must reference each required script explicitly.
     - Integration task model matches schema version:
       - Schema v2/v3: per-slice platform-fix (`X-integ-core`, `X-integ-<platform>`, `X-integ` for every slice).
       - Schema v4+: boundary-only platform-fix:
         - `pre-planning/ci_checkpoint_plan.md` exists,
         - `tasks.json` `meta.checkpoint_boundaries` matches `pre-planning/ci_checkpoint_plan.md` checkpoint group endings,
         - only boundary slices define `*-integ-core` / `*-integ-<platform>` tasks; normal slices use only `X-integ`.
         - checkpoint ops tasks exist and are wired:
           - `CPk-ci-checkpoint` depends on the checkpoint boundary slice’s `*-integ-core`,
           - the first slice of the next group depends on `CPk-ci-checkpoint` (so work cannot proceed past the checkpoint without completing the CI gate).
   - If you touch an ADR:
     - Run `make adr-fix ADR=<path>` (or equivalent) so the exec-summary drift guard is updated.

3) Re-run mechanical checks (required)
   - `export FEATURE_DIR="<FEATURE_DIR>"`
   - Run:
     - `make planning-lint FEATURE_DIR="$FEATURE_DIR"`
     - (optional; strict packs) `PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"`
     - (recommended; strict packs) `make pm-lift-pack PACK="$FEATURE_DIR"`
     - (recommended; strict packs) `make pm-lift-pack PACK="$FEATURE_DIR" EMIT_JSON=1`
     - `make planning-validate FEATURE_DIR="$FEATURE_DIR"`
     - `jq -e . "$FEATURE_DIR/tasks.json" >/dev/null`
     - `jq -e . docs/project_management/packs/sequencing.json >/dev/null`
   - If any check fails, fix and re-run until all pass.

4) Audit trail
   - Append a START/END remediation entry to `<FEATURE_DIR>/session_log.md` including:
     - the defects addressed (by Finding ID),
     - files changed,
     - exact commands run + exit codes.

5) Handoff for re-review
   - Do not change the previous reviewer’s findings.
   - Request a fresh quality gate re-review using:
     - `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`
   - The re-review must append a new pass to `quality_gate_report.md` and produce either:
     - `RECOMMENDATION: ACCEPT`, or
     - `RECOMMENDATION: FLAG FOR HUMAN REVIEW` with remaining blockers.

Output requirements (in your response):
- Summarize which Finding IDs were addressed and where (file paths).
- List all commands run with exit codes.
- If blocked: list required human decisions explicitly.
```
