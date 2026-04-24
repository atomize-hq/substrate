# Planning Quality Gate Prompt (Third-Party Reviewer)

Use this prompt after a planning/research agent finishes a Planning Pack, before execution begins.

If the recommendation is `FLAG FOR HUMAN REVIEW`, the next step is planning-doc remediation using:
- `docs/project_management/system/prompts/planning/quality_gate_remediation.md`

````md
You are a fresh, third-party reviewer of a Planning Pack. You did not author the pack.

Goal:
- Determine whether the Planning Pack is execution-ready under the current triad + schema v4 planning standards.
- Produce an auditable quality gate report with either:
  - `RECOMMENDATION: ACCEPT`
  - `RECOMMENDATION: FLAG FOR HUMAN REVIEW`

Constraints:
- Do not write production code.
- Do not expand scope.
- Do not “improve” the pack beyond what is necessary to assess execution readiness.
- If you identify defects, propose the minimal exact fix required.
- Preserve audit history:
  - if `<FEATURE_DIR>/quality_gate_report.md` already exists, append a new review pass instead of deleting prior findings.

Inputs (must read end-to-end):
- ADR(s) for the feature/track
- `<FEATURE_DIR>/spec_manifest.md`
- `<FEATURE_DIR>/plan.md`
- `<FEATURE_DIR>/tasks.json`
- `<FEATURE_DIR>/session_log.md`
- `<FEATURE_DIR>/quality_gate_report.md` when present
- all specs in scope
- `<FEATURE_DIR>/decision_register.md` when present or required
- `<FEATURE_DIR>/impact_map.md` when present or required
- `<FEATURE_DIR>/manual_testing_playbook.md` when present or required
- feature smoke scripts under `<FEATURE_DIR>/smoke/` when required by `tasks.json` meta or the selected specs/playbook
- `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md` when the pack is automation-enabled and cross-platform
- `docs/project_management/packs/sequencing.json`
- Standards and templates (read end-to-end):
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
  - `docs/project_management/system/standards/planning/PLANNING_README.md`
  - `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
  - `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`
  - `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
- If `tasks.json` `meta.execution_gates=true`, also read:
  - `<FEATURE_DIR>/execution_preflight_report.md`
  - `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
  - `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md`

Required reviewer workflow (non-negotiable):

1) Run the required mechanical checks and record them in the report.
   Minimum evidence must include:

   ```bash
   export FEATURE_DIR="<FEATURE_DIR>"

   jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
   jq -e . docs/project_management/packs/sequencing.json >/dev/null

   python3 - <<'PY'
   import json, os
   feature_dir=os.environ["FEATURE_DIR"]
   path=os.path.join(feature_dir, "tasks.json")
   data=json.load(open(path, "r", encoding="utf-8"))
   tasks=data["tasks"] if isinstance(data, dict) and "tasks" in data else data
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

   Then run the mechanical lint/validator suite required by the current pack:
- `make planning-lint FEATURE_DIR="$FEATURE_DIR"`
- `make planning-validate FEATURE_DIR="$FEATURE_DIR"`
- For strict packs (`tasks.json.meta.slice_spec_version >= 2`), also include Work Lift evidence:
  - `make pm-lift-pack PACK="$FEATURE_DIR"`
  - `make pm-lift-pack PACK="$FEATURE_DIR" EMIT_JSON=1`
  - `PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"`
- If you run narrower helper checks such as `validate_tasks_json.py`, `validate_slice_specs.py`, `validate_ci_checkpoint_plan.py`, or `planning-micro-lint`, include them as additional evidence, not as a substitute for the required lint evidence.
- If any mechanical lint check fails, the recommendation must be `FLAG FOR HUMAN REVIEW`.

2) Use the report template structure exactly.
- Write or append to `<FEATURE_DIR>/quality_gate_report.md`.
- Follow `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`.
- Record exact commands, exit codes, and blocker notes.

3) Review the pack against these pass/fail categories.

### A. Zero-ambiguity contracts
- `spec_manifest.md` assigns each contract/protocol/schema/env-var surface to exactly one authoritative document.
- No unresolved placeholder language, hard-ban terms, ambiguity words, or contradictory authoritative surfaces remain.
- CLI/config/path/exit-code contracts are singular and testable.

### B. Decision quality
- Every major decision uses exactly two viable options.
- Pros/cons/cascading implications/risks/unlocks/quick wins are explicit.
- The selected option is justified and traceable.
- Decision Register entries map to concrete triad task IDs through `tasks.json` `references`.

### C. Cross-doc consistency
- ADR(s), specs, `decision_register.md`, `impact_map.md`, `manual_testing_playbook.md`, smoke scripts, kickoff prompts, and `tasks.json` all describe the same contract.
- CLI commands, exit codes, path semantics, config precedence, and platform scope match exactly across docs.
- Exit code semantics match `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` unless an ADR explicitly overrides them.

### D. Sequencing and dependency alignment
- `tasks.json` dependencies match `docs/project_management/packs/sequencing.json`.
- Slice order is coherent and execution cannot start or advance before prerequisites integrate.
- For checkpointed packs, the checkpoint tasks and any post-checkpoint slice gating are wired deterministically.

### E. Testability and validation readiness
- Acceptance criteria are runnable and observable.
- `manual_testing_playbook.md` exists where required and uses explicit commands plus expected exit codes/output.
- Required smoke scripts exist and are referenced explicitly by the playbook.
- Smoke scripts represent the minimal runnable version of the manual validation flow rather than toy checks.
- Slice sizing is acceptable under the task context-budget rule in `TASK_TRIADS_AND_FEATURE_SETUP.md`; flag grab-bag slices before execution begins.

### F. Cross-platform parity task structure
- If `tasks.json` `meta.cross_platform=true`, platform scope is explicit:
  - `meta.behavior_platforms_required`
  - `meta.ci_parity_platforms_required`
  - legacy alias `meta.platforms_required` only when applicable
  - WSL is modeled via `meta.wsl_required` / `meta.wsl_task_mode`, not by adding `"wsl"` to the platform arrays
- The integration task model matches schema version:
  - schema v2/v3: per-slice `X-integ-core`, `X-integ-<platform>`, `X-integ`
  - schema v4+: only checkpoint-boundary slices listed in `meta.checkpoint_boundaries` define `*-integ-core` and `*-integ-<platform>` tasks; normal slices use only `X-integ`
- For automation-enabled cross-platform packs, `pre-planning/ci_checkpoint_plan.md` exists, is code-grounded, uses bounded checkpoints, and aligns mechanically with `tasks.json`.
- Checkpoint ops tasks are wired correctly:
  - `CPk-ci-checkpoint` depends on the boundary slice `*-integ-core`
  - the next checkpoint group cannot begin before the prior checkpoint task completes

### G. Execution-gate readiness
- If `meta.execution_gates=true`, verify the Planning Pack includes:
  - `execution_preflight_report.md`
  - `F0-exec-preflight` task with kickoff prompt
  - per-slice closeout report surfaces
  - final slice integration tasks that reference and require completing those closeout reports
- Confirm these surfaces are present and linked correctly from `tasks.json`.
- Do not perform the execution preflight itself here; only verify the planning pack is prepared for it.

### H. Triad interoperability
- `tasks.json` task shape is executable under the current triad runner.
- Kickoff prompts exist at the referenced paths.
- Kickoff prompts include the sentinel: `Do not edit planning docs inside the worktree.`
- Orchestration-only docs discipline is preserved.

4) Produce exhaustive findings.
- Every material defect must appear as a `DEFECT` finding.
- Verified important properties should appear as `VERIFIED` findings when they materially affect execution readiness.
- For each defect, include:
  - exact evidence
  - why it blocks or weakens execution readiness
  - one exact minimal fix
  - one viable alternative only if useful

Decision rules:
- `ACCEPT` only if:
  - the required mechanical checks pass,
  - the pack is internally consistent,
  - cross-platform/checkpoint/execution-gate wiring is correct for the pack’s schema and meta,
  - validation assets are adequate for the declared platform scope,
  - no unresolved blocker remains.
- Otherwise return `FLAG FOR HUMAN REVIEW`.

Output requirements:
- Start the report with the template’s recommendation fields, not ad hoc prose.
- Use the report-template section names and ordering:
  - `## Metadata`
  - `## Evidence: Commands Run (verbatim)`
  - `## Required Inputs Read End-to-End (checklist)`
  - `## Gate Results (PASS/FAIL with evidence)`
  - `## Findings (must be exhaustive)`
  - `## Decision: ACCEPT or FLAG`
- In the final decision section:
  - if `ACCEPT`, state the pack is execution-ready
  - if `FLAG FOR HUMAN REVIEW`, list blockers to execution and any required human decisions explicitly
````
