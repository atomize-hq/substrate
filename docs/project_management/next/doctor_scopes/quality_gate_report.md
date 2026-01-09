# Planning Quality Gate Report — doctor_scopes

## Metadata
- Feature directory: `docs/project_management/next/doctor_scopes/`
- Reviewed commit: `184ee0d3cbf3484b4867c29cc66951d4ccc47715`
- Reviewer: `third-party reviewer (Codex CLI)`
- Date (UTC): `2026-01-09`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/doctor_scopes"

# Planning lint (mechanical)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit 0

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit 0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit 0

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
# exit 0

# tasks.json invariants (schema + task model)
make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit 0
```

## Required Inputs Read End-to-End (checklist)

- ADR(s): `YES` (`docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`)
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`docs/project_management/next/doctor_scopes/DS0-spec.md`)
- `decision_register.md`: `YES`
- `integration_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/`: `YES` (`docs/project_management/next/doctor_scopes/smoke/`)
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` (exit codes) includes overlapping/unclear categories: “agent unreachable” (`3`) vs “not provisioned” (`4`) (`DS0-spec.md` around lines 52–56).
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` (JSON contract) defines `world.status=="unreachable"` but does not define a distinct `world.status` for “not provisioned” (`DS0-spec.md` around lines 149–152).
- Notes: The plan is mostly unambiguous, but the `substrate world doctor` error classification is not precise enough to implement consistently.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/doctor_scopes/decision_register.md` entries do not follow the required format (missing “Problem/Context”, “Cascading implications”, “Risks”, “Unlocks”, “Quick wins”, “Rationale”, and “Follow-up tasks”) (e.g. `decision_register.md` lines 5–16).
  - Required format: `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard) lines 147–204.
- Notes: Two options are provided, but the register is not audit-ready per the repo standard.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - Exit code taxonomy is referenced consistently, but the “unreachable vs not provisioned” classification appears in both ADR and spec without a concrete discriminator:
    - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md` lines 79–85
    - `docs/project_management/next/doctor_scopes/DS0-spec.md` lines 52–56
- Notes: This will likely yield platform-divergent behavior for the same failure mode.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` includes sprint `doctor_scopes` with `DS0` spec (`sequencing.json` lines ~172–178).
  - `docs/project_management/next/doctor_scopes/tasks.json` dependencies enforce internal ordering (F0 → DS0-code/test → DS0-integ-core → platform-fix → DS0-integ) (see `tasks.json` task `depends_on` fields).
- Notes: No task starts before its declared prerequisites.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - Manual playbook Windows “world doctor” step does not validate DS0’s required “unsupported” contract fields (`manual_testing_playbook.md` lines 84–90 vs DS0 acceptance criteria).
  - DS0 requires: `ok=false`, `status=unsupported`, exit code `4` for Windows (`docs/project_management/next/doctor_scopes/DS0-spec.md` acceptance criteria near end).
- Notes: Linux/macOS commands are concrete and runnable; Windows section is currently too weak to catch contract regressions.

### 5.1) Cross-platform parity task structure (schema v2+)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/doctor_scopes/tasks.json` meta: `schema_version: 3`, `behavior_platforms_required: ["linux","macos"]`, `ci_parity_platforms_required: ["linux","macos","windows"]`.
  - Per slice: `DS0-integ-core`, `DS0-integ-linux`, `DS0-integ-macos`, `DS0-integ-windows`, `DS0-integ` exist and are dependency-wired.
- Notes: Matches the standards’ platform-fix integration model.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `tasks.json` required fields present (python audit above).
  - Kickoff prompts include the sentinel: “Do not edit planning docs inside the worktree.” (lint runner PASS).
- Notes: Task automation model is consistent and runnable.

## Findings (must be exhaustive)

### Finding 001 — Mechanical lint passes (required gate)
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/doctor_scopes"` → exit `0` (see commands).
- Impact: Confirms the Planning Pack meets mechanical requirements (presence, hard bans, schema checks).
- Fix required (exact): `none`

### Finding 002 — Cross-platform parity task structure is present and wired
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/doctor_scopes/tasks.json` meta `schema_version: 3` + tasks `DS0-integ-core`, `DS0-integ-{linux,macos,windows}`, `DS0-integ`.
- Impact: Execution can proceed through core integration, platform-fix, then final aggregation without workflow drift.
- Fix required (exact): `none`

### Finding 003 — Decision register is not audit-ready per repo standard
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/doctor_scopes/decision_register.md` uses a shortened format (e.g. `DR-0001` at lines 5–16).
  - Required format + traceability requirements: `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` lines 147–204.
- Impact: Decision quality and auditability requirements are not met; downstream triads lack explicit risks/implications/unlocks and follow-up mapping.
- Fix required (exact): Update every `DR-XXXX` entry in `docs/project_management/next/doctor_scopes/decision_register.md` to match the required template, including explicit follow-up tasks mapped to concrete `tasks.json` task IDs.
- If DEFECT: Alternative (one viable): Reduce the decision register to only the truly “major” decisions and fully template those, moving minor notes into the spec; still include follow-up task mapping for every remaining decision.

### Finding 004 — Decision ↔ task traceability is missing in `tasks.json`
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/doctor_scopes/tasks.json` task references include `docs/project_management/next/doctor_scopes/decision_register.md` but do not reference specific `DR-00XX` ids (e.g., `DS0-code` references list).
  - Standard requirement: `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` lines 198–204.
- Impact: Post-hoc auditing cannot reliably answer “which task implemented which decision.”
- Fix required (exact): Update each relevant task’s `references` entries in `docs/project_management/next/doctor_scopes/tasks.json` to include DR ids, e.g. `docs/project_management/next/doctor_scopes/decision_register.md (DR-0001, DR-0002, ...)`, scoped per task.
- If DEFECT: Alternative (one viable): Add a single “Decision coverage” section to `docs/project_management/next/doctor_scopes/integration_map.md` mapping DR ids → task ids, and then reference that section from each task (still less direct than references-per-task).

### Finding 005 — `substrate world doctor` exit code contract is ambiguous (unreachable vs not provisioned)
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` lines 52–56 (`3`: unreachable; `4`: disabled/not provisioned or missing prereqs).
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` lines 149–152 (JSON `world.status` includes `disabled|unreachable|ok|missing_prereqs`, but no distinct “not provisioned” status).
  - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md` lines 79–85 includes “world disabled/not provisioned” under exit `4`.
- Impact: Different implementations may choose exit `3` vs `4` for the same “socket missing/service not present” condition, breaking automation.
- Fix required (exact): In both `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md` and `docs/project_management/next/doctor_scopes/DS0-spec.md`, define a concrete discriminator for exit `3` vs `4` (and, if needed, add/rename a `world.status` value so JSON matches exit semantics).
- If DEFECT: Alternative (one viable): Treat all “agent unreachable” states (including “not provisioned”) as exit `3`, and reserve exit `4` strictly for “world disabled” and “agent reachable but missing required primitives”; this requires removing “not provisioned” from the exit `4` description.

### Finding 006 — Windows manual playbook does not validate the DS0 “unsupported” world doctor contract
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/doctor_scopes/manual_testing_playbook.md` lines 84–90 only assert `platform=="windows"` for `world doctor`.
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` requires explicit Windows unsupported semantics and exit code `4` (acceptance criteria section).
- Impact: A regression (e.g., `ok=true` or missing `status=unsupported`) could slip through manual validation.
- Fix required (exact): Update the Windows “world doctor” playbook command to assert the required fields (`ok==false` and `world.status=="unsupported"` or the equivalent contract keys once finalized), plus exit code `4`.
- If DEFECT: Alternative (one viable): Remove the Windows world-doctor manual step entirely and replace it with a unit/integration test requirement in the DS0 test acceptance criteria, if Windows is strictly CI-parity-only and manual validation is not expected.

## Decision: ACCEPT or FLAG

### FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes and the task graph is runnable, but the decision register/traceability requirements are not met and the `world doctor` error/exit-code contract is ambiguous.
- Required human decisions (explicit):
  - Decide and document the authoritative exit-code mapping for “world enabled but not provisioned” vs “world enabled but transport failure”, including how to detect the difference (or explicitly collapse the cases).
- Blockers to execution:
  - Bring `docs/project_management/next/doctor_scopes/decision_register.md` up to the required decision template (including follow-up tasks).
  - Add DR id references in `docs/project_management/next/doctor_scopes/tasks.json` (or equivalent traceability mechanism that meets the standard).
  - Clarify and align the exit code + JSON status contracts for `substrate world doctor`.

---

# Planning Quality Gate Report — doctor_scopes (Pass 2 / remediation re-review)

## Metadata
- Feature directory: `docs/project_management/next/doctor_scopes/`
- Reviewed commit: `852bc1d36f86c6f3a353a59ad9ece7d1816e2d94`
- Reviewer: `remediation agent (Codex CLI)`
- Date (UTC): `2026-01-09`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/doctor_scopes"

make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit 0

make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit 0

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit 0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit 0
```

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` defines an explicit discriminator for world-doctor exit `3` vs `4` and adds `world.status=="not_provisioned"` (`DS0-spec.md` lines 52–70 and 161–170).
- Notes: Exit code and JSON status semantics are implementable without platform-divergent interpretation.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/doctor_scopes/decision_register.md` entries follow the required template, including explicit follow-up task mapping (example: `DR-0001` at line 10, `DR-0008` at line 256).
- Notes: Each decision has exactly two viable options and a single selected recommendation.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - Exit code discriminator is aligned between:
    - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md` (Exit codes section)
    - `docs/project_management/next/doctor_scopes/DS0-spec.md` (Exit codes + JSON contracts)
- Notes: ADR and spec describe the same exit code semantics and detection rules.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` contains sprint `doctor_scopes` with `DS0` spec.
  - `docs/project_management/next/doctor_scopes/tasks.json` maintains the DS0 dependency graph (F0 → code/test → integ-core → platform-fix → integ).
- Notes: No task starts before declared prerequisites.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - Windows manual playbook validates `unsupported` contract fields for world doctor and asserts exit code `4` (`docs/project_management/next/doctor_scopes/manual_testing_playbook.md` lines 84–90).
  - Linux/macOS smoke scripts and manual playbook commands assert stable JSON keys/values and exit code expectations.
- Notes: Manual and smoke validation are runnable and assert required contract fields.

## Findings (must be exhaustive)

### Finding 001 — Mechanical lint passes (required gate)
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/doctor_scopes"` → exit `0`
- Impact: Confirms the Planning Pack meets mechanical requirements (presence, hard bans, schema checks).
- Fix required (exact): `none`

### Finding 002 — Cross-platform parity task structure is present and wired
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/doctor_scopes/tasks.json` meta `schema_version: 3` and required integration tasks exist and are dependency-wired.
- Impact: Execution can proceed through core integration, platform-fix, then final aggregation without workflow drift.
- Fix required (exact): `none`

### Finding 003 — Decision register is audit-ready per repo standard
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/doctor_scopes/decision_register.md` entries follow the required template and include follow-up tasks mapped to `tasks.json` IDs.
- Impact: Decisions are auditable and actionable during triad execution.
- Fix required (exact): `none`

### Finding 004 — Decision ↔ task traceability is present in `tasks.json`
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/doctor_scopes/tasks.json` references include DR ids for tasks that implement decisions (`DS0-code` and `DS0-test`).
- Impact: Auditing can answer “which task implemented which decision.”
- Fix required (exact): `none`

### Finding 005 — `substrate world doctor` exit code contract is unambiguous
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/doctor_scopes/DS0-spec.md` defines the `3` vs `4` discriminator and adds `world.status=="not_provisioned"`.
  - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md` aligns to the same discriminator.
- Impact: Automation and scripts can rely on stable exit codes and statuses.
- Fix required (exact): `none`

### Finding 006 — Windows manual playbook validates the DS0 “unsupported” world doctor contract
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/doctor_scopes/manual_testing_playbook.md` asserts `ok==false`, `host.status=="unsupported"`, `world.status=="unsupported"`, and exit code `4`.
- Impact: Manual validation catches regressions in Windows unsupported semantics.
- Fix required (exact): `none`

## Decision: ACCEPT or FLAG

### ACCEPT
- Summary: Blocking defects from the prior pass are resolved; contracts are unambiguous, decisions are audit-ready, and validation steps are runnable.
- Next step: Execution triads may begin.
