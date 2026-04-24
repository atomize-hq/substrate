RECOMMENDATION: FLAG FOR HUMAN REVIEW

# Planning Quality Gate Report — adr-0027-identity-tuple-policy-surface

## Metadata
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- Reviewed commit: `0d59ba174f4a5d7b523887b9613f99766be64130`
- Reviewer: `Codex (third-party reviewer)`
- Date (UTC): `2026-04-24`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
```
- Exit code: `0`
- Notes: `tasks.json` is valid JSON.

```bash
jq -e . docs/project_management/packs/sequencing.json >/dev/null
```
- Exit code: `0`
- Notes: `sequencing.json` is valid JSON.

```bash
FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface" python - <<'PY'
import json, os
feature_dir=os.environ['FEATURE_DIR']
path=os.path.join(feature_dir,'tasks.json')
data=json.load(open(path,'r',encoding='utf-8'))
tasks=data['tasks'] if isinstance(data,dict) and 'tasks' in data else data
required=[
  'id','name','type','phase','status','description',
  'references','acceptance_criteria','start_checklist','end_checklist',
  'worktree','integration_task','kickoff_prompt',
  'depends_on','concurrent_with'
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get('id','<no id>'),m))
if missing:
  for tid,m in missing:
    print(tid,':',', '.join(m))
  raise SystemExit(1)
print('OK: tasks.json required fields present')
PY
```
- Exit code: `0`
- Notes: required-field audit passed.

### Planning lint (mechanical)
- `FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `0` → planning lint passed; Work Lift advisory reported `Lift Score: 88`, `Estimated slices: 8`, `Confidence: low`; duplicate-create warnings from `impact_map.md` were warn-only.
- `FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make planning-validate FEATURE_DIR="$FEATURE_DIR"` → `0` → `validate_tasks_json.py` passed.

### Work Lift advisory (recommended)
- `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` → `0` → `Lift Score (v1): 88`, `Estimated slices: 8`, `Confidence: low`.
- `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface" EMIT_JSON=1` → `0` → JSON emitted with `lift_score=88`, `estimated_slices=8`, `confidence="low"`, and missing inputs including `ops.new_smoke_steps`.

### Additional review commands (if any)
- `git rev-parse HEAD` → `0` → reviewed commit is `0d59ba174f4a5d7b523887b9613f99766be64130`.
- `test -d docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke` → `1` → required `smoke/` directory is absent.
- `find docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface -maxdepth 2 -type f | sort | rg '/smoke/'` → `1` → no smoke scripts exist under the pack.
- `rg -n 'smoke/' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md` → `1` → no concrete `smoke/` script paths are referenced by the manual playbook, tasks, or plan.
- `rg -n 'decision_register.md|DR-ITPS' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json` → `1` → no task references point to the accepted decision-register entries.

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES`
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES`
- `decision_register.md` (if present/required): `YES`
- `impact_map.md` (if present/required): `YES`
- `manual_testing_playbook.md` (if present/required): `YES`
- Feature smoke scripts under `smoke/` (if required): `NO`
- `docs/project_management/packs/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

Additional standards reviewed end-to-end:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/system/standards/planning/PLANNING_README.md`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`
- `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md:194-195`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:47-75`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:33-60`
- Notes: `contract.md` and `manual_testing_playbook.md` make `substrate policy current show --explain` the authoritative merged inspection surface for `llm.constraints.*`, but `spec_manifest.md` still assigns visibility to both config and policy explain surfaces. The pack does not carry one unambiguous operator contract.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:1-124`
- Notes: both decisions are framed as exactly two viable options with explicit pros, cons, cascading implications, risks, unlocks, quick wins, and one accepted selection.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md:194-195`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md:150-155`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:53-75`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:35-60`
- Notes: the impact map explicitly called for removal of the stale config-view assignment, but the final pack still contains it in `spec_manifest.md`. The operator-surface contract therefore remains internally inconsistent.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json` entries: `adr_0027_identity_tuple_policy_surface` at `docs/project_management/packs/sequencing.json:831-859`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json` deps: `meta.checkpoint_boundaries` at `:3-27`, `ITPS3-integ-core` / platform-fix / checkpoint tasks at `:610-868`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
- Notes: the pack now appears in sequencing, the slice order is `ITPS0` → `ITPS1` → `ITPS2` → `ITPS3`, and the schema v4 checkpoint wiring is consistent with `CP1` after `ITPS3-integ-core`.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - Manual playbook sections: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:33-318`
  - Smoke scripts: none; `test -d docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke` exited `1`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-18`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:652-776`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:834-857`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md:239-250`
  - `docs/project_management/system/standards/planning/PLANNING_README.md:70-79`
- Notes: the pack declares `behavior_platforms_required = ["linux","macos","windows"]` and plans Linux/macOS/Windows behavior smoke in the checkpoint and platform-fix tasks, but there is no `smoke/` directory, no smoke scripts, and no manual-playbook linkage to concrete smoke paths.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `PASS`
- Evidence:
  - `tasks.json` meta: `schema_version = 4`, `cross_platform = true`, `behavior_platforms_required = ["linux","macos","windows"]`, `ci_parity_platforms_required = ["linux","macos","windows"]`, `checkpoint_boundaries = ["ITPS3"]` at `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-27`
  - Boundary tasks: `ITPS3-integ-core`, `ITPS3-integ-linux`, `ITPS3-integ-macos`, `ITPS3-integ-windows`, `ITPS3-integ`, and `CP1-ci-checkpoint` at `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:610-868`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
- Notes: the schema v4 task model is wired correctly for a single boundary slice. This pass does not waive the missing-smoke defect above.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `tasks.json` required fields present via the required-field audit command (`exit 0`)
  - kickoff prompts include “no docs edits in worktrees”: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/kickoff_prompts/ITPS0-code.md:1-22`, `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/kickoff_prompts/ITPS3-integ-core.md:1-20`, `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/kickoff_prompts/CP1-ci-checkpoint.md:1-18`
- Notes: task shape, kickoff inventory, and worktree sentinels are compatible with the triad runner. A separate auditability defect remains: decision-register items are not referenced from the implementing tasks.

## Findings (must be exhaustive)

### Finding 001 — `spec_manifest.md` still assigns tuple-policy visibility to both config and policy explain surfaces
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md:194-195`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:53-75`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:35-60`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md:150-155`
- Impact: zero-ambiguity and cross-doc consistency are broken. Execution tasks would inherit contradictory acceptance surfaces for the same `llm.constraints.*` keys.
- Fix required (exact): replace the config-view row in `pre-planning/spec_manifest.md` so only `substrate policy current show` / `substrate policy current show --explain` is authoritative for tuple-policy visibility and provenance.
- If DEFECT: Alternative (one viable): keep `substrate config show --explain` only as non-authoritative config-root inspection text, but then update every contract/playbook/spec reference so the policy view remains the sole authoritative merged surface.

### Finding 002 — Cross-platform behavior smoke is required by the pack but no smoke assets exist
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-18`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:652-776`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:834-857`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md:239-250`
  - `docs/project_management/system/standards/planning/PLANNING_README.md:70-74`
  - `test -d docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke` → exit `1`
  - `find docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface -maxdepth 2 -type f | sort | rg '/smoke/'` → exit `1`
  - `rg -n 'smoke/' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md` → exit `1`
- Impact: the checkpoint plan and platform-fix tasks cannot produce the required Linux/macOS/Windows behavior-smoke evidence. The pack is not validation-ready.
- Fix required (exact): add `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, and `smoke/windows-smoke.ps1`, then reference each script explicitly from `manual_testing_playbook.md` and the relevant checkpoint/platform-fix validation notes.
- If DEFECT: Alternative (one viable): if three-OS behavior guarantees are not truly required, narrow `meta.behavior_platforms_required` and remove the matching behavior-parity claims from the ADR/specs/tasks so the smoke obligation matches the real support surface.

### Finding 003 — Decision Register entries are not trace-linked from the implementing triad tasks
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md:207-213`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md:8-12`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md:33-35`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md:8-13`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md:55-67`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:72-77`
  - `rg -n 'decision_register.md|DR-ITPS' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json` → exit `1`
- Impact: the plan is not fully auditable. Execution evidence cannot show which tasks are responsible for closing `DR-ITPS-01` and `DR-ITPS-02`, even though the slice specs make those decisions part of the slice contract.
- Fix required (exact): add `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md (DR-ITPS-01)` and `(DR-ITPS-02)` to the `references` arrays of the tasks that implement and close those decisions, at minimum the `ITPS0` and `ITPS1` code/test/integration tasks.
- If DEFECT: Alternative (one viable): move decision closure into an explicit docs-only task and update the slice specs and task graph so decision ownership still maps one-to-one to concrete task IDs.

### Finding 004 — Decision register quality is strong and does not block execution by itself
- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:1-124`
- Impact: the pack already contains two concrete A/B decisions with explicit tradeoffs and a single chosen direction. Remediation should preserve those decisions unless product ownership deliberately changes them.
- Fix required (exact): none.

### Finding 005 — Schema v4 checkpoint and sequencing topology are coherent
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-27`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:610-868`
  - `docs/project_management/packs/sequencing.json:831-859`
- Impact: after remediation, the pack can use the existing task graph without reslicing or resequencing.
- Fix required (exact): none.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: the pack is mechanically lint-clean, but it is not execution-ready. It still contains one unresolved operator-surface contradiction, it does not provide the required cross-platform smoke assets for its declared behavior platforms, and it does not meet decision-to-task traceability requirements.
- Required human decisions (explicit):
  - Decide whether Linux, macOS, and Windows are truly all behavior platforms for this feature. If yes, keep the current scope and add the missing smoke assets. If no, narrow `meta.behavior_platforms_required` and the corresponding parity claims before execution.
  - Decide whether to keep `DR-ITPS-02` as written (`substrate policy current show --explain` is the sole authoritative merged inspection surface). If yes, remove the stale config-view row. If no, revise the decision register, ADR, contract, playbook, and manifest together before execution.
- Blockers to execution:
  - `pre-planning/spec_manifest.md` contradicts the accepted operator-surface contract.
  - `quality_gate_report.md` does not recommend `ACCEPT`, so `task_start.sh` will block feature execution.
  - No `smoke/` scripts exist for the declared Linux/macOS/Windows behavior platforms.
  - `tasks.json` does not trace the accepted decision-register entries to the implementing tasks.
