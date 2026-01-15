# Planning Quality Gate Report — workspace-config-policy-unification

## Metadata
- Feature directory: `docs/project_management/next/workspace-config-policy-unification/`
- Reviewed commit: `11facec1017caa4e0c0648f9463ff86b90174498`
- Reviewer: `third-party planning pack reviewer (Codex CLI)`
- Date (UTC): `2026-01-15`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)
```bash
FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit=0

make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit=0

make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=0

python - <<'PY'
import json, os
feature_dir=os.environ.get("FEATURE_DIR","docs/project_management/next/workspace-config-policy-unification")
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
# exit=0 (OK: tasks.json required fields present)
```

### Additional review commands
```bash
jq -r '.sprints[] | select(.id=="workspace_config_policy_unification") | {order,id,title,status,directory,plan,sequence}' docs/project_management/next/sequencing.json
# exit=0
```

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES` (`docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, plus `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`)
- `plan.md`: `YES` (`docs/project_management/next/workspace-config-policy-unification/plan.md`)
- `tasks.json`: `YES` (`docs/project_management/next/workspace-config-policy-unification/tasks.json`)
- `session_log.md`: `YES` (`docs/project_management/next/workspace-config-policy-unification/session_log.md`)
- All specs in scope: `YES` (`docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md` … `docs/project_management/next/workspace-config-policy-unification/WCU5-spec.md`)
- `decision_register.md` (if present/required): `YES` (`docs/project_management/next/workspace-config-policy-unification/decision_register.md`)
- `integration_map.md` (if present/required): `YES` (`docs/project_management/next/workspace-config-policy-unification/integration_map.md`)
- `manual_testing_playbook.md` (if present/required): `YES` (`docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md`)
- Feature smoke scripts under `smoke/` (if required): `YES` (`docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`, `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`, `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`)
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification"` → `OK: planning lint passed` (exit `0`)
- Notes: Mechanical hard-ban and ambiguity scans passed.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/decision_register.md:11`
- Notes: Entries provide only Pros/Cons; required tradeoff fields (implications/risks/unlocks/quick wins) and explicit follow-up task mapping are not present.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS` (with noted coverage gaps)
- Evidence: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md:148`, `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md:30`, `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh:17`
- Notes: Core command spellings, scope model (`current|global|workspace`), and canonical paths are consistent; however some ADR contract surfaces are not validated by playbook/smoke/tasks (see Findings 004/005).

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` sprint entry: `workspace_config_policy_unification` (order `29`)
  - `docs/project_management/next/workspace-config-policy-unification/tasks.json:215`
- Notes: Slice dependencies enforce WCU1→WCU5 ordering via prior-slice integration tasks; code/test concurrency only within a slice.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:197`, `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md:30`
- Notes: Manual playbook and smoke scripts are runnable, but task acceptance criteria generally do not include runnable commands with expected exit codes/output; several ADR-mandated behaviors are also not explicitly covered by validation artifacts.

### 5.1) Cross-platform parity task structure (schema v2)
- Result: `N/A`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:2`
- Notes: `meta.schema_version=3` but this pack does not declare `meta.ci_parity_platforms_required` (or legacy `meta.platforms_required`), so the `X-integ-core` / `X-integ-<platform>` / `X-integ` structure is not required by the mechanical standard.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:8`, `make planning-lint …` kickoff prompt sentinel check passed (exit `0`)
- Notes: Automation metadata and kickoff sentinel are present; tasks include required fields.

## Findings (must be exhaustive)

### Finding 001 — Mechanical planning lint passes
- Status: `VERIFIED`
- Evidence: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md:1` and `make planning-lint FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification"` (exit `0`)
- Impact: Confirms the pack meets baseline mechanical quality gates and can be executed by the triad tooling.
- Fix required (exact): none

### Finding 002 — Decision register does not meet the decision-quality standard
- Status: `DEFECT`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/decision_register.md:11` (entries contain Pros/Cons only)
- Impact: The plan does not meet the stated requirement that decisions include complete tradeoffs (implications/risks/unlocks/quick wins) and explicit follow-up task mapping; audit review is not implementation-ready under the provided checklist.
- Fix required (exact): Update each DR entry to include implications/risks/unlocks/quick wins and explicitly identify which `tasks.json` task IDs implement/validate the decision.
- If DEFECT: Alternative (one viable): Move the decision register content into ADR-0008 (as an explicit “Decision Register” section) and explicitly map each decision to `tasks.json` task IDs there, then delete or supersede the feature-local decision register to avoid a second, incomplete source of truth.

### Finding 003 — Decisions are not traceable to triad task IDs via `references`
- Status: `DEFECT`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:53` (task references omit `decision_register.md`; none of the tasks reference `decision_register.md (DR-xxxx)`)
- Impact: Review checklist “Auditability” fails: a future executor cannot reliably prove which tasks implement which explicit decisions, and the decision register cannot be verified as “done” per slice.
- Fix required (exact): Add `docs/project_management/next/workspace-config-policy-unification/decision_register.md (DR-xxxx)` to the `references` list of each implementing task (at minimum: the relevant `WCU*-code`, `WCU*-test`, and `WCU*-integ` tasks per decision).
- If DEFECT: Alternative (one viable): If the intent is “ADR-only decisions”, delete `decision_register.md` and ensure ADR-0008’s contract section is the sole decision record, then update tasks to reference the ADR sections instead of a decision register.

### Finding 004 — Validation artifacts do not cover some ADR-0008 contract surfaces
- Status: `DEFECT`
- Evidence: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md:317` (defines `substrate workspace init [PATH] [--force] [--examples]`); `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md:52` (does not validate `--force`/`--examples`)
- Impact: The plan risks landing a partial ADR implementation without an explicit validation tripwire; downstream consumers may assume contract surfaces exist and behave correctly.
- Fix required (exact): Add explicit validation steps (manual playbook + smoke + at least one slice acceptance criteria) for `workspace init --force` and `workspace init --examples`, including expected exit codes/output and file effects (or explicit non-effects).
- If DEFECT: Alternative (one viable): If these flags are intentionally deferred, mark them as explicit non-goals in the ADR-0008 “Non-Goals” section and remove the commands from the ADR user contract to keep the contract and plan aligned.

### Finding 005 — Validation artifacts do not cover the full `config ... set` update syntax declared in ADR-0008
- Status: `DEFECT`
- Evidence: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md:192` (declares `key=value`, `key+=value`, `key-=value`); `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md:22` (only covers `+=` and `reset` for `world.deps.enabled`)
- Impact: Implementation may diverge from the ADR contract (e.g., omit `-=`), or add it ad-hoc without tests/playbook coverage; either way, the plan is not implementation-ready under a “contract must be testable” rubric.
- Fix required (exact): Either (a) add spec + validation coverage for `key-=value` list removal semantics, including expected exit codes/output, or (b) explicitly remove/deny `-=` from the ADR user contract and document the supported mutation forms as final.
- If DEFECT: Alternative (one viable): Adopt dedicated `add/remove` subcommands (already listed as an option in `decision_register.md`) and explicitly de-scope operator syntax in the ADR to reduce parsing ambiguity and make playbook assertions clearer.

### Finding 006 — Sequencing readiness is enforced by task dependencies
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:215` (WCU2 depends on WCU1-integ; WCU3 depends on WCU2-integ; WCU4 depends on WCU3-integ; WCU5 depends on WCU4-integ.)
- Impact: Prevents starting a slice before prerequisites are integrated, reducing integration churn and contract drift.
- Fix required (exact): none

### Finding 007 — Smoke scripts exist for all required behavior platforms and are referenced
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/workspace-config-policy-unification/tasks.json:6`, `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md:9`
- Impact: Provides a runnable, cross-platform validation path aligned with the plan’s declared behavior platforms.
- Fix required (exact): none

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes, but the Planning Pack fails decision-quality, auditability traceability, and validation coverage requirements from the provided reviewer checklist; it is not implementation-ready as written.
- Required human decisions (explicit):
  - Confirm whether ADR-0008’s declared CLI surfaces (`workspace init --force/--examples`, `key-=value`) are required in this pack or intentionally de-scoped, and update docs to make that final.
  - Confirm whether the decision register is required as a first-class artifact; if yes, require it to include complete tradeoffs + task mapping.
- Blockers to execution:
  - Fix Findings 002–005; re-run `make planning-lint` and update this report accordingly.
