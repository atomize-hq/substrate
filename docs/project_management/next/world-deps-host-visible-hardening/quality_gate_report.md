# Planning Quality Gate Report — world-deps-host-visible-hardening

## Metadata
- Feature directory: `docs/project_management/next/world-deps-host-visible-hardening/`
- Reviewed commit: `ab279a546d5f810a579c323b3a7a40122fc7f177`
- Reviewer: third-party planning pack reviewer (external)
- Date (UTC): `2026-02-16`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/world-deps-host-visible-hardening"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit=0

# Mechanical planning lint (required)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=2
# Missing required path: docs/project_management/next/world-deps-host-visible-hardening/ci_checkpoint_plan.md

# tasks.json invariants (required)
make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit=2
# Key failures (non-exhaustive; see full output in command log):
# - tasks[*].worktree must be non-empty (multiple integration tasks are null)
# - boundary slice wiring invalid (WDH1/WDH3 integration_task must point to *-integ-core)
# - missing/required automation fields (git_branch, required_make_targets, merge_to_orchestration, platform)
# - missing required ops task: FZ-feature-cleanup
# - WSL bundled coverage not represented in Linux platform-fix/checkpoint tasks

# ADR executive summary drift (required when ADRs referenced)
make adr-check ADR=docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md
# exit=2
# ADR_BODY_SHA256 mismatch (found 9f5a5e..., expected eea9a0...)

make adr-check ADR=docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md
# exit=0

# Required-field audit (template minimum)
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
# exit=0

# Sequencing membership (required by planning lint rubric)
rg -n "world-deps-host-visible-hardening" docs/project_management/next/sequencing.json
# exit=1 (no matches)

# Kickoff prompt sentinel coverage (required by planning lint rubric)
rg -n "Do not edit planning docs inside the worktree\\." "$FEATURE_DIR/kickoff_prompts" -S
# exit=0 (found only in WDH0-code.md and WDH0-test.md)

# Ambiguity scan (required by planning lint rubric)
rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' '\\b(should|could|might|maybe)\\b' "$FEATURE_DIR"
# exit=0 (matches found; must be fixed)
# - impact_map.md:48
# - smoke/_core.sh:63
```

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES`
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WDH0-spec.md`, `WDH1-spec.md`, `WDH2-spec.md`, `WDH3-spec.md`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/`: `YES`
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md` contains banned ambiguity word(s).
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/_core.sh` contains banned ambiguity word(s).
- Notes: Planning lint ambiguity scan is violated (`should` present outside `decision_register.md`).

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` DR-0001/DR-0002/DR-0003/DR-0004/DR-0005: Option B is documented as not meeting the contract (“does not solve the problem” / “fails the contract” / “unlocks none”).
- Notes: Standard requires two viable options; several Option B entries are explicitly non-viable strawmen.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md` states “unless explicitly allowed by policy” but defines only env-var overrides (no policy key).
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` requires override inputs use `SUBSTRATE_OVERRIDE_*` (WDH2 introduces `SUBSTRATE_WORLD_EXEC_GUARD*`).
- Notes: Override surface and taxonomy are not consistent across authoritative inputs/specs.

### 4) Sequencing and dependency alignment
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/sequencing.json` has no entry for `docs/project_management/next/world-deps-host-visible-hardening`.
  - `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md` explicitly states sequencing was not reviewed (`Sequencing alignment: reviewed: NO`).
- Notes: Mechanical lint requires sequencing membership; missing entry blocks execution scheduling and violates rubric.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/manual_testing_playbook.md` exists and is runnable, but `tasks.json` fails mechanical validation, so automation and checkpoint execution cannot proceed deterministically.
  - `docs/project_management/next/world-deps-host-visible-hardening/kickoff_prompts/*.md` sentinel missing in most prompts (planning lint failure mode).
- Notes: Validation artifacts exist, but task graph + automation metadata are not execution-ready.

### 5.1) Cross-platform parity task structure (schema v4)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` meta: `schema_version=4`, `cross_platform=true`, `automation.enabled=true`, `checkpoint_boundaries=["WDH1","WDH3"]`.
  - `make planning-lint` fails due missing `ci_checkpoint_plan.md`.
  - `make planning-validate` fails for boundary-only platform-fix wiring (integration_task, merge_to_orchestration, platform fields, WSL bundled coverage).
- Notes: Required checkpoint plan + deterministic wiring are not present.

### 6) Triad interoperability (execution workflow)
- Result: `FAIL`
- Evidence:
  - `make planning-validate FEATURE_DIR="$FEATURE_DIR"` fails (automation-required task fields missing).
  - Kickoff prompt sentinel missing in most prompts under `docs/project_management/next/world-deps-host-visible-hardening/kickoff_prompts/`.
- Notes: This pack cannot be executed by the triad automation runner as-is.

## Findings (must be exhaustive)

### Finding 001 — Missing `ci_checkpoint_plan.md` (required for cross-platform automation packs)
- Status: `DEFECT`
- Evidence:
  - `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → `Missing required path: docs/project_management/next/world-deps-host-visible-hardening/ci_checkpoint_plan.md`
- Impact: Mechanical quality gate fails immediately; checkpoint boundaries (`WDH1`, `WDH3`) are not defined/wired in an auditable way.
- Fix required (exact): Add `docs/project_management/next/world-deps-host-visible-hardening/ci_checkpoint_plan.md` using `docs/project_management/standards/templates/ci_checkpoint_plan.md.tmpl`, defining bounded checkpoint groups and the required ops checkpoint task ids.
- If DEFECT: Alternative (one viable): Disable checkpoints/automation by setting `tasks.json` `meta.automation.enabled=false` (and aligning schema/validation expectations accordingly), accepting higher manual orchestration cost.

### Finding 002 — `tasks.json` fails mechanical validation for schema v4 + automation
- Status: `DEFECT`
- Evidence:
  - `make planning-validate FEATURE_DIR="$FEATURE_DIR"` reports:
    - integration task `worktree` fields are null (must be non-empty strings),
    - missing automation-required fields (e.g., `git_branch`, `required_make_targets`, `merge_to_orchestration`, `platform`),
    - boundary slice wiring invalid (`WDH1-code`/`WDH1-test` and `WDH3-code`/`WDH3-test` integration_task must be `*-integ-core`),
    - missing required ops task `FZ-feature-cleanup`.
- Impact: Task graph is not executable by automation; checkpoint/boundary model cannot be run deterministically; cross-platform parity cannot be enforced.
- Fix required (exact): Update `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` until `make planning-validate FEATURE_DIR="$FEATURE_DIR"` exits `0`, including:
  - add `ci_checkpoint_plan.md`-referenced checkpoint ops tasks,
  - add `FZ-feature-cleanup`,
  - add missing automation fields to every task,
  - correct boundary integration_task wiring and boundary deps,
  - add `platform` for platform-fix tasks and represent WSL bundled coverage in Linux parity tasks/checklists.
- If DEFECT: Alternative (one viable): Reduce to a non-automation pack (manual execution) by disabling automation and removing schema v4 checkpoint expectations; keep cross-platform behavior smoke in manual playbook.

### Finding 003 — Feature is not present in `sequencing.json`
- Status: `DEFECT`
- Evidence:
  - `rg -n "world-deps-host-visible-hardening" docs/project_management/next/sequencing.json` → exit `1` (no matches).
- Impact: Mechanical lint fails; prerequisites cannot be reasoned about against the global sequencing spine; execution readiness cannot be certified.
- Fix required (exact): Add a sprint entry in `docs/project_management/next/sequencing.json` that points at `docs/project_management/next/world-deps-host-visible-hardening` and enumerates slice ids `WDH0..WDH3` in order.
- If DEFECT: Alternative (one viable): If this work is intentionally unscheduled, move the directory under `docs/project_management/_archived/` and reference it from sequencing as an archived pointer (but then it is not an execution-ready “next/” pack).

### Finding 004 — ADR-0011 executive summary hash drift (adr-check fails)
- Status: `DEFECT`
- Evidence:
  - `make adr-check ADR=docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` fails with `ADR_BODY_SHA256 mismatch`.
- Impact: ADR integrity gate is failing; downstream Planning Packs cannot rely on the ADR’s executive summary being in sync with the authoritative body.
- Fix required (exact): Update `ADR_BODY_SHA256` in `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` to match the current body (or revert body edits) so `make adr-check ADR=...` exits `0`.
- If DEFECT: Alternative (one viable): If the body changes are intentional, re-run the standard ADR hash update workflow and record the reason for change in the ADR history/notes.

### Finding 005 — Ambiguity scan violation (`should` in feature directory)
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md:48` contains `should`.
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/_core.sh:63` contains `should`.
- Impact: Mechanical ambiguity gate fails; contract documents must be strict (“MUST/SHALL” style) to avoid execution drift.
- Fix required (exact): Replace ambiguous words with unambiguous wording (or remove the sentence) in:
  - `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/smoke/_core.sh`
- If DEFECT: Alternative (one viable): Move any non-normative language into `decision_register.md` (which is excluded from the ambiguity scan), keeping other artifacts strictly unambiguous.

### Finding 006 — Kickoff prompt sentinel missing in most kickoff prompts
- Status: `DEFECT`
- Evidence:
  - Sentinel appears only in:
    - `docs/project_management/next/world-deps-host-visible-hardening/kickoff_prompts/WDH0-code.md`
    - `docs/project_management/next/world-deps-host-visible-hardening/kickoff_prompts/WDH0-test.md`
  - Missing in (non-exhaustive list; see command output in evidence section):
    - `.../kickoff_prompts/F0-exec-preflight.md`
    - `.../kickoff_prompts/WDH0-integ.md`
    - `.../kickoff_prompts/WDH1-*.md`, `WDH2-*.md`, `WDH3-*.md`
- Impact: Mechanical kickoff-prompt lint fails; increased risk that agents will edit planning docs inside worktrees, violating the execution standard.
- Fix required (exact): Add the exact line `- Do not edit planning docs inside the worktree.` to every file under `docs/project_management/next/world-deps-host-visible-hardening/kickoff_prompts/`.
- If DEFECT: Alternative (one viable): Remove automation enablement and enforce “no docs edits” via manual workflow only (higher risk; not recommended).

### Finding 007 — Decision register violates “two viable options” rule for major decisions
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` DR-0001/DR-0002/DR-0003/DR-0004/DR-0005 Option B is described as failing to meet the contract (“does not solve the problem”, “fails the contract”, “unlocks none”).
- Impact: Decisions are not reviewable against two realistic alternatives; increases risk of missing a viable design that better matches constraints.
- Fix required (exact): For each affected DR, replace Option B with a genuinely viable alternative (or split decisions so each DR has two viable options), preserving the “exactly 2 options” format.
- If DEFECT: Alternative (one viable): Escalate the decision(s) into a small ADR addendum for any DR where only one viable option exists, explicitly overriding the “two viable options” requirement.

### Finding 008 — WDH2 override surface conflicts with inputs and its own stated posture (“allowed by policy”)
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md:12-34` says “unless explicitly allowed by policy” but defines only env-var overrides (`SUBSTRATE_WORLD_EXEC_GUARD*`).
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix B.2.*) states override inputs remain `SUBSTRATE_OVERRIDE_*`.
- Impact: Cross-doc contract inconsistency; introduces a new env var surface that may violate established override taxonomy, and leaves “policy allow” unspecified.
- Fix required (exact): Choose and specify one authoritative override surface for WDH2 and make it consistent everywhere (spec + manual playbook + smoke + tasks):
  - either define a policy key (and remove/rename env vars accordingly), or
  - keep env vars but rename them into `SUBSTRATE_OVERRIDE_*` taxonomy and remove “allowed by policy” claims.
- If DEFECT: Alternative (one viable): Defer WDH2 as a follow-up Planning Pack and remove WDH2 slice from this pack so the remaining slices can be executed without policy/override-surface ambiguity.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint/validation fails (missing `ci_checkpoint_plan.md`, invalid `tasks.json`, missing sequencing entry, ADR hash drift, kickoff prompt sentinel gaps, ambiguity scan violations). This Planning Pack is not execution-ready.
- Required human decisions (explicit):
  - Confirm whether this pack is intended to be automation-enabled schema v4 with CI checkpoints; if yes, define checkpoints and required ops tasks; if no, disable automation and revise validation expectations.
  - Confirm the authoritative override surface for WDH2 (policy vs override env vars) and align with ADR-0018 env var taxonomy.
  - Assign sequencing (which sprint/order) and add to `docs/project_management/next/sequencing.json`.
- Blockers to execution:
  - `make planning-lint FEATURE_DIR="$FEATURE_DIR"` must exit `0`.
  - `make planning-validate FEATURE_DIR="$FEATURE_DIR"` must exit `0`.
  - `make adr-check ADR=docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` must exit `0`.

---

## Review Pass 2 — Remediation Re-Review (2026-02-16)

RECOMMENDATION: ACCEPT

## Metadata (Pass 2)
- Feature directory: `docs/project_management/next/world-deps-host-visible-hardening/`
- Reviewed commit: `ab279a546d5f810a579c323b3a7a40122fc7f177`
- Reviewer: remediation agent
- Date (UTC): `2026-02-16`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/world-deps-host-visible-hardening"

# ADR drift guard (required when ADRs referenced)
make adr-check ADR=docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md
# exit=0

# Mechanical planning lint (required)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=0

# tasks.json invariants (required)
make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit=0

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit=0
```

## Required Inputs Read End-to-End (Pass 2 checklist)
- ADR(s): `YES`
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WDH0-spec.md`, `WDH1-spec.md`, `WDH2-spec.md`, `WDH3-spec.md`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/`: `YES`
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence) — Pass 2

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence:
  - `make planning-lint FEATURE_DIR="$FEATURE_DIR"` ambiguity scan passes (exit `0`).

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` DR-0001..DR-0005 Option B entries are viable alternatives with explicit tradeoffs and a single selection.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - WDH2 override inputs use `SUBSTRATE_OVERRIDE_*` taxonomy (ADR-0006/ADR-0018 alignment) and no longer claim unspecified policy allow.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` includes `directory="docs/project_management/next/world-deps-host-visible-hardening"` with slices `WDH0..WDH3` in order.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - `manual_testing_playbook.md` references required smoke scripts.
  - Smoke scripts exist for behavior platforms and are referenced from integration tasks.

### 5.1) Cross-platform parity task structure (schema v4)
- Result: `PASS`
- Evidence:
  - `tasks.json` meta: `schema_version=4`, `cross_platform=true`, `automation.enabled=true`, `checkpoint_boundaries=["WDH1","WDH3"]`.
  - `ci_checkpoint_plan.md` exists and validates; checkpoint ops tasks exist and are wired.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `make planning-validate FEATURE_DIR="$FEATURE_DIR"` exits `0`.
  - Kickoff prompt sentinel present in every file under `kickoff_prompts/`.

## Findings (Pass 2 status)

### Finding 001 — Missing `ci_checkpoint_plan.md` (required for cross-platform automation packs)
- Status: `VERIFIED`
- Fix applied:
  - Added `docs/project_management/next/world-deps-host-visible-hardening/ci_checkpoint_plan.md`.

### Finding 002 — `tasks.json` fails mechanical validation for schema v4 + automation
- Status: `VERIFIED`
- Fix applied:
  - Updated `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` to satisfy `make planning-validate` and schema v4 checkpoint wiring.

### Finding 003 — Feature is not present in `sequencing.json`
- Status: `VERIFIED`
- Fix applied:
  - Added `docs/project_management/next/world-deps-host-visible-hardening` sprint entry to `docs/project_management/next/sequencing.json`.

### Finding 004 — ADR-0011 executive summary hash drift (adr-check fails)
- Status: `VERIFIED`
- Fix applied:
  - Updated `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` `ADR_BODY_SHA256` via `make adr-fix`.

### Finding 005 — Ambiguity scan violation (`should` in feature directory)
- Status: `VERIFIED`
- Fix applied:
  - Rewrote ambiguity matches in `impact_map.md` and `smoke/_core.sh`.

### Finding 006 — Kickoff prompt sentinel missing in most kickoff prompts
- Status: `VERIFIED`
- Fix applied:
  - Added the sentinel line to every file under `kickoff_prompts/` (including new checkpoint/cleanup prompts).

### Finding 007 — Decision register violates “two viable options” rule for major decisions
- Status: `VERIFIED`
- Fix applied:
  - Rewrote Option B entries for DR-0001..DR-0005 in `decision_register.md` as viable alternatives.

### Finding 008 — WDH2 override surface conflicts with inputs and its own stated posture (“allowed by policy”)
- Status: `VERIFIED`
- Fix applied:
  - Aligned override inputs to `SUBSTRATE_OVERRIDE_*` taxonomy and removed unspecified policy-allow claims (`WDH2-spec.md`, dependent prompts/docs).

## Decision: ACCEPT or FLAG (Pass 2)

### If ACCEPT
- Summary: Mechanical lint and validation pass; cross-doc contracts are consistent; checkpoints are defined and wired; sequencing entry exists; kickoff prompt sentinels and ambiguity rubric checks pass.
- Next step: Execution triads may begin.
