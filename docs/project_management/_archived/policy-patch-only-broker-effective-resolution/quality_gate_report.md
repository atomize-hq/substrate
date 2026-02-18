# Planning Quality Gate Report — policy-patch-only-broker-effective-resolution

## Metadata
- Feature directory: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/`
- Reviewed commit: `19c2e853e602c0713a6dd4028df88a95e0bfff19`
- Reviewer: third-party reviewer (Codex)
- Date (UTC): `2026-01-17`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
export FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit: 0

jq -e . docs/project_management/packs/sequencing.json >/dev/null
# exit: 0

# tasks.json required-field audit
python3 - <<'PY'
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
# exit: 0
```

### Planning lint (mechanical)
Reference: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`

```bash
make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit: 0

make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit: 0
```

### Additional review commands (if any)

```bash
jq -r '.sprints[] | select(.directory=="docs/project_management/_archived/policy-patch-only-broker-effective-resolution") | {order,id,branch,status,sequence}' \
  docs/project_management/packs/sequencing.json
```

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES` (`docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`, `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`)
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C0-spec.md`, `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md`)
- `decision_register.md`: `YES`
- `integration_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/`: `YES`
- `docs/project_management/packs/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C0-spec.md` (inputs/contract/exit codes)
- Notes: Contracts for patch-only policy + broker canonical resolution are singular and testable.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/decision_register.md` (DR-0001)
- Notes: Two viable options (A/B) with explicit selection and tradeoffs.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` (“Manual validation” section points to a different playbook/smoke location)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md` (feature-local playbook + smoke list)
- Notes: Validation pointers disagree (ADR vs feature pack), increasing the risk of running the wrong validation set.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json` sprint `policy_patch_only_broker_effective_resolution` lists `C0` then `C1`
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` has `C1-*` depending on `C0-integ`
- Notes: No task begins before its prerequisites.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md` (acceptance criteria include shim + world-agent fail-closed)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh` (C1 runs `--command "echo …"` only)
  - `scripts/dev/substrate_shell_driver` (forces `--no-world` by default)
- Notes: The current smoke/manual set does not provide a runnable check for world-agent and is not clearly exercising the shim interception path.

### 5.1) Cross-platform parity task structure (schema v2+)
- Result: `PASS`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` meta: `schema_version=3`, `behavior_platforms_required`, `ci_parity_platforms_required`
  - Per slice tasks exist: `X-integ-core`, `X-integ-<platform>`, `X-integ`
- Notes: Matches the required platform-fix integration model.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` required fields present (script audited)
  - Kickoff prompts include “Do not edit planning docs inside the worktree.”
- Notes: Mechanical interoperability checks pass.

## Findings (must be exhaustive)

### Finding 001 — Mechanical planning lint passes
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` exit `0` (see “Evidence: Commands Run”)
- Impact: Planning Pack is mechanically valid.
- Fix required (exact): none

### Finding 002 — Decision register meets the “two viable options” rule
- Status: `VERIFIED`
- Evidence: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/decision_register.md` (DR-0001)
- Impact: The main architectural placement choice is reviewable and auditable.
- Fix required (exact): none

### Finding 003 — Sequencing and task dependencies are aligned
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/sequencing.json` sprint `policy_patch_only_broker_effective_resolution`
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` (`C1-*` depends on `C0-integ`)
- Impact: Work can proceed without dependency violations.
- Fix required (exact): none

### Finding 004 — Validation set does not fully cover C1 acceptance criteria (shim + world-agent)
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md` (“Acceptance criteria” includes shim + world-agent)
  - `scripts/dev/substrate_shell_driver` (adds `--no-world` by default)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh` (`run_c1` uses `--command "echo …"` only)
- Impact: The plan cannot currently be validated end-to-end against its stated cross-component scope; regressions in shim/world-agent fail-closed behavior can slip through.
- Fix required (exact):
  - Add explicit, runnable validation for shim interception and world-agent behavior to either:
    - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md` + `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/*`, or
    - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md` + `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` (list exact required test commands that cover shim + world-agent).
- Alternative (one viable): If smoke must remain `--no-world`, require explicit `cargo test -p world-agent …` and `cargo test -p substrate-shim …` commands in C1 integration checklists and closeout report evidence to cover the missing surfaces.

### Finding 005 — Docs alignment is required by spec but not explicitly tracked in task references/acceptance
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/C1-spec.md` (requires updating `docs/CONFIGURATION.md`)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` (no `docs/CONFIGURATION.md` in task `references` / acceptance criteria)
- Impact: A required deliverable can be missed without failing any task acceptance criteria, reducing auditability.
- Fix required (exact): Add `docs/CONFIGURATION.md` to `references` and add an explicit acceptance criterion to `C1-integ` (or `C1-code`) stating `docs/CONFIGURATION.md` is updated per `C1-spec.md`.
- Alternative (one viable): Add a dedicated `C1-docs` task (type `code` or `ops`) that updates `docs/CONFIGURATION.md` and is required by `C1-integ` (only if the team wants to keep docs changes out of code/integration tasks).

### Finding 006 — ADR-0013 validation pointers disagree with this feature’s playbook/smoke locations
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` (“Manual validation” / “Smoke scripts” point to `workspace-config-policy-unification/*`)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md` (points to this feature’s `smoke/*`)
- Impact: Validators may run the wrong smoke scripts, undermining cross-platform parity confidence and CI automation.
- Fix required (exact): Update `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` to reference this feature’s `manual_testing_playbook.md` and `smoke/*` paths (or explicitly state both are required and why).
- Alternative (one viable): If the intention is to centralize validation in the ADR-0008 feature directory, remove/redirect this feature’s playbook/smoke paths and point tasks.json to the centralized ones.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes and the core contract is clear, but the validation/auditability wiring is incomplete for the stated C1 scope (shim/world-agent) and required docs alignment.
- Required human decisions (explicit):
  - Where shim/world-agent validation lives (smoke vs explicit test suites) for this feature.
  - Whether ADR-0013 should point validation at this feature directory or at `workspace-config-policy-unification/`.
- Blockers to execution:
  - Fill the validation coverage gap in Finding 004.
  - Make docs alignment auditable in tasks/criteria per Finding 005.

---

# Planning Quality Gate Report — policy-patch-only-broker-effective-resolution (Re-review Pass 2)

RECOMMENDATION: ACCEPT

## Metadata
- Feature directory: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/`
- Reviewed commit: `19c2e853e602c0713a6dd4028df88a95e0bfff19`
- Reviewer: third-party reviewer (Codex)
- Date (UTC): `2026-01-17`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution"

jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit: 0

jq -e . docs/project_management/packs/sequencing.json >/dev/null
# exit: 0

make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit: 0

make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit: 0
```

## Findings (delta vs prior pass)

### Finding 004 — Validation covers shim + world-agent (C1)
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md` (“Additional required validation (C1 only): shim + world-agent” section)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` task `C1-integ-core` acceptance criteria + end_checklist include:
    - `cargo test -p substrate-shim -- --nocapture`
    - `cargo test -p world-agent -- --nocapture`
- Impact: C1 acceptance criteria is runnable and auditable beyond the `--no-world` CLI smoke subset.
- Fix required (exact): none

### Finding 005 — `docs/CONFIGURATION.md` update is explicitly tracked
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/tasks.json` tasks `C1-integ-core` and `C1-integ` include `docs/CONFIGURATION.md` in `references` and as an explicit acceptance criterion.
- Impact: Required docs deliverable is owned by a task and cannot be silently skipped.
- Fix required (exact): none

### Finding 006 — ADR validation pointers match this feature’s playbook/smoke scripts
- Status: `VERIFIED`
- Evidence: `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` (Validation Plan references this feature’s `manual_testing_playbook.md` and `smoke/*`)
- Impact: Operators/CI are pointed at the correct validation assets.
- Fix required (exact): none

## Decision: ACCEPT or FLAG

### If ACCEPT
- Summary: Mechanical checks are green and prior DEFECT findings are resolved with explicit, runnable validation wiring.
- Next step: Execution triads may begin after `F0-exec-preflight` is completed.
