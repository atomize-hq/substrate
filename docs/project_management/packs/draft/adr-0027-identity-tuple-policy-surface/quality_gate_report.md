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

---

RECOMMENDATION: FLAG FOR HUMAN REVIEW

# Planning Quality Gate Report — adr-0027-identity-tuple-policy-surface

## Metadata
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- Reviewed commit: `3c463b66c0a40cb43535829879d286e803a39f1e`
- Reviewer: `Codex (third-party reviewer)`
- Date (UTC): `2026-04-24`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
```
- Exit code: `0`
- Notes: `tasks.json` is valid JSON.

```bash
jq -e . docs/project_management/packs/sequencing.json >/dev/null
```
- Exit code: `0`
- Notes: `sequencing.json` is valid JSON.

```bash
export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
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
- Exit code: `0`
- Notes: required-field audit passed.

### Planning lint (mechanical)
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `0` → planning lint passed; `impact_map.md` emitted warn-only duplicate-create entries because the Create touch-set lists files that now exist.
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make planning-validate FEATURE_DIR="$FEATURE_DIR"` → `0` → `validate_tasks_json.py` passed.

### Work Lift advisory (recommended)
- `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` → `0` → `Lift Score (v1): 97`, `Estimated slices: 9`, `Confidence: low`.
- `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface" EMIT_JSON=1` → `0` → JSON emitted with `lift_score=97`, `estimated_slices=9`, `confidence="low"`.
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `0` → lint passed and printed the same advisory; top triggers were `likely_split:crates_touched>2`, `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`, and `split_required:estimated_slices>3`.

### Additional review commands (if any)
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` → `0` → slice-spec validation passed.
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` → `0` → `ci_checkpoint_plan.md` validation passed.
- `git rev-parse HEAD` → `0` → reviewed commit is `3c463b66c0a40cb43535829879d286e803a39f1e`.
- `rg -n 'Do not edit planning docs inside the worktree\.' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/kickoff_prompts docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/*/kickoff_prompts` → `0` → every referenced kickoff prompt includes the required sentinel.
- `rg -n 'decision_register.md \(DR-ITPS-01\)|decision_register.md \(DR-ITPS-02\)' docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json` → `0` → `ITPS0` and `ITPS1` triad tasks now trace the accepted decision-register entries.
- `rg -n 'substrate config show --explain|substrate policy current show --explain' docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` → `0` → both ADRs still contain config-view assertions that conflict with the pack’s selected operator-surface contract.

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES`
- `spec_manifest.md`: `YES` (`pre-planning/spec_manifest.md`)
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES`
- `decision_register.md` (if present/required): `YES`
- `impact_map.md` (if present/required): `YES` (`pre-planning/impact_map.md`)
- `manual_testing_playbook.md` (if present/required): `YES`
- Feature smoke scripts under `smoke/` (if required): `YES`
- `docs/project_management/packs/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

Additional standards reviewed end-to-end:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
- `docs/project_management/system/standards/planning/PLANNING_README.md`
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`
- `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:90-95`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:253-260`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:348-356`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:12-17`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:49-75`
- Notes: the accepted pack contract is singular: `substrate policy current show --explain` is the authoritative merged tuple-policy surface and `substrate config show --explain` is config-root only. ADR-0043 and ADR-0027 still describe tuple-policy visibility/provenance on config explain surfaces, so the contract is not zero-ambiguity.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:14-124`
- Notes: both decisions are structured as exactly two viable options with explicit pros, cons, cascading implications, risks, unlocks, quick wins, and a single accepted selection.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:71-124`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md:194-197`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:69-96`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:258-260`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:354-356`
- Notes: the pack-local docs consistently lock tuple-policy provenance to the policy effective view, but the governing ADR layer still says the config explain surface includes the new keys and provenance. The same contract is described two different ways.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json` entries: `docs/project_management/packs/sequencing.json:834-853`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json` deps: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:1-27`, `:621-889`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
- Notes: the sequencing spine, plan, and schema v4 boundary-only checkpoint wiring agree on `ITPS0 → ITPS1 → ITPS2 → ITPS3`, with `CP1-ci-checkpoint` anchored after `ITPS3-integ-core`.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - Manual playbook sections: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:69-238`
  - Smoke scripts: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/_core.sh:25-89`, `smoke/linux-smoke.sh:1-11`, `smoke/macos-smoke.sh:1-11`, `smoke/windows-smoke.ps1:36-90`
  - Standard: `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md:46-48`
  - `tasks.json` integration smoke references: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:664-880`
- Notes: the playbook requires observable checks for explain JSON shape, gateway status tuple publication, and router/provider/protocol/auth-authority deny wording, but the smoke scripts only assert a few exit codes and doc-presence checks. They do not yet represent the minimal runnable version of the declared manual validation flow.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `PASS`
- Evidence:
  - `tasks.json` meta: `schema_version = 4`, `cross_platform = true`, `behavior_platforms_required = ["linux","macos"]`, `ci_parity_platforms_required = ["linux","macos","windows"]`, `checkpoint_boundaries = ["ITPS3"]` at `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:2-26`
  - Boundary tasks: `ITPS3-integ-core`, `ITPS3-integ-linux`, `ITPS3-integ-macos`, `ITPS3-integ-windows`, `ITPS3-integ`, and `CP1-ci-checkpoint` at `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:621-889`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
- Notes: the pack uses the correct schema v4 boundary-only model. Linux and macOS are behavior platforms; Windows is modeled as CI parity only.

### 5.2) Execution-gate readiness
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:18-19`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:30-63`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/execution_preflight_report.md:1-69`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-closeout_report.md:1-9`
- Notes: the required execution-gate surfaces exist and are wired correctly. The current `execution_preflight_report.md` still recommends `REVISE`, but that is an execution-time follow-on gate, not a missing planning surface.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `tasks.json` required fields present via the required-field audit command (`exit 0`)
  - kickoff prompts include “Do not edit planning docs inside the worktree.” across all referenced prompt paths
  - decision-register traceability is present in `tasks.json` for `ITPS0` and `ITPS1` task references
- Notes: the task shape, prompt inventory, orchestration-only docs discipline, and slice-to-decision traceability are compatible with the current triad runner.

## Findings (must be exhaustive)

### Finding 001 — Governing ADRs still contradict the accepted tuple-policy inspection contract
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:95`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:259`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:355-356`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:82-99`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:53-75`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:71-96`
- Impact: zero-ambiguity and cross-doc consistency both fail. Execution tasks would inherit a pack-local contract that says the policy view is authoritative while the governing ADR layer still claims config explain surfaces include tuple-policy keys and provenance.
- Fix required (exact): update ADR-0043 `User Contract > CLI` and `Validation Plan > Tests`, plus ADR-0027 `Validation Plan > Tests`, so they state that `substrate policy current show --explain` is the authoritative merged tuple-policy surface and that `substrate config show --explain` remains config-root inspection only without claiming tuple-policy provenance.
- If DEFECT: Alternative (one viable): if dual-surface ownership is intentional, revise `DR-ITPS-02`, `contract.md`, `manual_testing_playbook.md`, `pre-planning/spec_manifest.md`, and the slice specs together so the selected contract becomes internally consistent everywhere.

### Finding 002 — Smoke scripts do not yet exercise the declared validation contract
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md:46-48`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:69-238`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/_core.sh:81-87`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/windows-smoke.ps1:82-88`
- Impact: the pack is not validation-ready. The smoke layer only proves a narrow CLI bootstrap and invalid-input exit-code path, not the feature’s core observable contract around explain JSON shape, gateway status tuple publication, or tuple-axis deny wording. That leaves `CP1-ci-checkpoint` and the platform-fix tasks without a meaningful behavior-validation entrypoint.
- Fix required (exact): extend the smoke scripts so they assert the observable outputs promised by the manual playbook, at minimum the `substrate policy current show --json --explain` JSON/explain contract and at least one gateway-status or tuple-axis deny path that checks exit code `5` plus locked wording.
- If DEFECT: Alternative (one viable): narrow `manual_testing_playbook.md`, `ITPS3-spec.md`, and the checkpoint/platform-fix acceptance criteria so the declared validation surface matches the smaller smoke contract you actually intend to run.

### Finding 003 — Decision register quality is execution-grade
- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:14-124`
- Impact: the pack already contains two concrete A/B decisions with explicit tradeoffs and a single selected direction. The remaining work is alignment and validation coverage, not decision structure.
- Fix required (exact): none.

### Finding 004 — Schema v4 checkpoint and cross-platform task wiring are coherent
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md:3-43`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:2-26`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:621-889`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:30-53`
  - `docs/project_management/packs/sequencing.json:834-853`
- Impact: no resequencing or task-graph rewrite is needed. Once the contract and smoke defects are fixed, the current checkpoint topology is usable as-is.
- Fix required (exact): none.

### Finding 005 — Execution-gate surfaces and triad prompt sentinels are present
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/execution_preflight_report.md:1-69`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-closeout_report.md:1-9`
  - kickoff prompt sentinel coverage command returned `0`
- Impact: the pack is structurally prepared for execution gating and orchestration discipline once the remaining blockers are removed.
- Fix required (exact): none.

### Finding 006 — Slice sizing is coherent despite the low-confidence Work Lift warning
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md:17-43`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md:3-18`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md:3-14`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-spec.md:3-14`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md:3-14`
- Impact: no grab-bag slice blocker was found. Each slice still describes one behavior delta even though the pack-level Work Lift estimate is high and low-confidence.
- Fix required (exact): none.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: the pack is mechanically lint-clean and structurally well-wired, but it is still not execution-ready. The governing ADR layer does not yet match the pack’s accepted tuple-policy inspection contract, and the smoke scripts do not yet validate the observable behavior that the manual playbook and checkpoint tasks say matters.
- Required human decisions (explicit):
  - Decide whether `DR-ITPS-02` remains the selected contract. If yes, align ADR-0043 and ADR-0027 to the policy-view-only ownership model. If no, revise the pack-local contract, playbook, slice specs, and tasks together before execution.
  - Decide whether checkpoint smoke must validate the gateway-status and tuple-axis deny surfaces. If yes, deepen the smoke scripts. If no, narrow the declared manual-validation and checkpoint acceptance surface so it matches the lighter smoke contract.
- Blockers to execution:
  - `ADR-0043` still uses ambiguous or conflicting config/policy inspection wording, and `ADR-0027` still claims config explain output includes tuple-policy provenance.
  - The smoke scripts do not yet cover the core observable behavior promised by `manual_testing_playbook.md`.
  - Because `meta.execution_gates=true`, `F0-exec-preflight` would still need to be rerun after those defects are fixed and the pack is re-reviewed.

---

RECOMMENDATION: ACCEPT

# Planning Quality Gate Report — adr-0027-identity-tuple-policy-surface

## Metadata
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- Reviewed commit: `3c463b66c0a40cb43535829879d286e803a39f1e`
- Reviewer: `Codex (third-party reviewer)`
- Date (UTC): `2026-04-24`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
```
- Exit code: `0`
- Notes: `tasks.json` is valid JSON.

```bash
jq -e . docs/project_management/packs/sequencing.json >/dev/null
```
- Exit code: `0`
- Notes: `sequencing.json` is valid JSON.

```bash
export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
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
- Exit code: `0`
- Notes: required-field audit passed.

### Planning lint (mechanical)
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `0` → planning lint passed on the remediated pack.
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `0` → planning lint passed with advisory enabled; Work Lift remained warn-only.
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make planning-validate FEATURE_DIR="$FEATURE_DIR"` → `0` → planning validation passed.

### Work Lift advisory (recommended)
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make pm-lift-pack PACK="$FEATURE_DIR"` → `0` → `Lift Score (v1): 97`, `Estimated slices: 9`, `Confidence: low`.
- `export FEATURE_DIR="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"; make pm-lift-pack PACK="$FEATURE_DIR" EMIT_JSON=1` → `0` → JSON emitted with `lift_score=97`, `estimated_slices=9`, `confidence="low"`, and missing-input advisories only.

### Additional review commands (if any)
- `python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface` → `0` → slice specs validated.
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface` → `0` → CI checkpoint plan validated.
- `bash docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/linux-smoke.sh` → `0` → Linux smoke validated the policy explain contract, schema-invalid rejection, gateway status JSON shape, and router/provider deny wording.
- `make adr-fix ADR='docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md'` → `0` → executive summary hash refreshed after contract-alignment edits.
- `make adr-fix ADR='docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md'` → `0` → executive summary hash refreshed after contract-alignment edits.

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES`
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- `quality_gate_report.md` when present: `YES`
- All specs in scope: `YES`
- `decision_register.md` (if present/required): `YES`
- `impact_map.md` (if present/required): `YES`
- `manual_testing_playbook.md` (if present/required): `YES`
- Feature smoke scripts under `smoke/` (if required): `YES`
- `pre-planning/ci_checkpoint_plan.md` when required: `YES`
- `execution_preflight_report.md` when required: `YES`
- `docs/project_management/packs/sequencing.json`: `YES`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`: `YES`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`: `YES`
- `docs/project_management/system/standards/planning/PLANNING_README.md`: `YES`
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
- `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`: `YES`
- `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`
- `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md` (required because `meta.execution_gates=true`): `YES`
- `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md` (required because `meta.execution_gates=true`): `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/spec_manifest.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:15-16`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:54-75`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:95-96`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:355-356`
- Notes: the authoritative inspection contract is singular and testable. `substrate policy current show --explain` owns merged tuple-policy provenance for `llm.constraints.*`, while `substrate config show --explain` remains config-root inspection only.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:14-70`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md:71-124`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json`
- Notes: both major decisions retain exactly two viable options, explicit tradeoffs, explicit selection, and task traceability through `references`.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md:54-126`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:80-240`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:95-96`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:260-261`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:355-356`
- Notes: ADRs, pack-local specs, playbook, and exit-code mapping now describe the same contract. Exit code `2` remains schema invalidity, `4` transport unavailable, and `5` policy/safety deny, matching the declared taxonomy for this pack.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json:834-853`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-24`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:621-922`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:21-53`
- Notes: slice order and boundary-only checkpoint wiring remain deterministic. `CP1-ci-checkpoint` is correctly anchored after `ITPS3-integ-core`, and the platform-fix fanout cannot begin until that checkpoint completes.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - Manual playbook sections: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:37-41`, `:80-240`
  - Smoke scripts: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/_core.sh:28-52`, `:273-330`, `smoke/linux-smoke.sh:1-11`, `smoke/macos-smoke.sh:1-11`, `smoke/windows-smoke.ps1:51-120`
  - `tasks.json` integration end_checklist includes smoke: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:664-889`
- Notes: the smoke layer now covers the minimum runnable version of the declared manual validation contract. Linux and macOS behavior-platform smoke asserts authoritative policy explain JSON/provenance, schema-invalid exit `2`, machine-readable gateway status tuple publication at exit `4`, and router/provider deny wording at exit `5`. Windows remains compile-parity only and automates the policy-inspection plus schema-invalid subset accordingly.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-24`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:621-922`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:21-53`
- Notes: the pack uses schema v4 correctly. Only the checkpoint-boundary slice `ITPS3` owns `*-integ-core` and `*-integ-<platform>` tasks, while earlier slices use normal `X-integ` tasks. WSL is not mis-modeled as a behavior platform.

### 5.2) Execution-gate readiness
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:19-24`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:30-63`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/execution_preflight_report.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-closeout_report.md:1-9`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-closeout_report.md:1-9`
- Notes: the execution-gate surfaces, kickoff prompt inventory, slice closeout surfaces, and post-checkpoint integration tasks are all present and linked correctly. This review pass resolves the prior procedural blocker by moving the latest quality gate recommendation to `ACCEPT`.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - required-field audit command returned `0`
  - planning lint kickoff sentinel check returned `0`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/kickoff_prompts/F0-exec-preflight.md:8`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/kickoff_prompts/CP1-ci-checkpoint.md:8`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/kickoff_prompts/FZ-feature-cleanup.md:8`
- Notes: `tasks.json` remains executable under the current triad runner, kickoff prompts are present at the referenced paths, and the no-doc-edits sentinel discipline is preserved.

## Findings (must be exhaustive)

### Finding 001 — ADR layer is aligned to the selected policy-authoritative inspection contract
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:95-96`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:260-261`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:355-356`
- Impact: the last remaining contract contradiction between ADRs and the pack-local surfaces is removed, so execution tasks now inherit one deterministic operator contract.
- Fix required (exact): none.

### Finding 002 — Smoke validation now covers the declared observable contract
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md:37-41`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/_core.sh:273-330`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/windows-smoke.ps1:103-120`
  - `bash docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/linux-smoke.sh` → `0`
- Impact: checkpoint smoke and platform-fix validation now have a meaningful behavior-level entrypoint instead of doc-presence checks only.
- Fix required (exact): none.

### Finding 003 — Schema v4 checkpoint and cross-platform wiring remain coherent
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:3-24`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:621-922`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md:21-53`
  - `docs/project_management/packs/sequencing.json:834-853`
- Impact: no resequencing, task-graph repair, or checkpoint-model rewrite is required before execution.
- Fix required (exact): none.

### Finding 004 — Execution-gate and triad-runner surfaces are ready
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:19-24`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json:30-63`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/execution_preflight_report.md`
  - kickoff prompt sentinel validation passed in planning lint
- Impact: `F0-exec-preflight`, slice closeout reports, boundary integration tasks, and orchestration-only prompt discipline are all in place for execution.
- Fix required (exact): none.

### Finding 005 — Work Lift remains advisory-only and does not expose a slice-sizing blocker
- Status: `VERIFIED`
- Evidence:
  - `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` → `0`
  - `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface" EMIT_JSON=1` → `0`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md:17-43`
- Impact: the pack-level lift score is still high and low-confidence, but the slice plan is not a grab-bag and does not block execution readiness under the current standards.
- Fix required (exact): none.

## Decision: ACCEPT or FLAG

### If ACCEPT
- Summary: the required mechanical checks passed, the ADR and pack-local documents now agree that `substrate policy current show --explain` is the authoritative tuple-policy surface, the smoke layer now validates the declared observable contract, and the schema v4 checkpoint/execution-gate wiring remains correct. The pack is execution-ready.
- Next step: `Execution triads may begin.`

RECOMMENDATION: ACCEPT
