# Planning Quality Gate Report — world-deps-packages-bundles-contract

RECOMMENDATION: FLAG FOR HUMAN REVIEW

## Metadata
- Feature directory: `docs/project_management/next/world-deps-packages-bundles-contract/`
- Reviewed commit: `aae8f707da0bccff835b708cb79dbc5102b15f49`
- Reviewer: `Codex (GPT-5.2), third-party reviewer`
- Date (UTC): `2026-02-13`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
export FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract"

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
# stdout: OK: tasks.json required fields present
# exit 0
```

### Planning lint (mechanical)
Reference: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`

```bash
export FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract"
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit 2
# FAIL: docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md:
#   ADR_BODY_SHA256 mismatch (found f98e101d96918048c0407b0d8c26c127fcdef4c566d4a954b1a770cbdfc28696,
#   expected 41fcd002c7e017054b2e4812420598a60ad6041277ebcfc9e41f881b5a83b29f)
```

### Additional review commands (if any)

```bash
make adr-check ADR=docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md
# exit 2
```

## Required Inputs Read End-to-End (checklist)
Mark `YES` only if read end-to-end.

- ADR(s): `YES`
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES`
- `decision_register.md` (if present/required): `YES`
- `impact_map.md` (if present/required): `YES`
- `manual_testing_playbook.md` (if present/required): `YES`
- Feature smoke scripts under `smoke/` (if required): `YES`
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract"` ran hard-ban + ambiguity scans and did not report violations (it failed later due to ADR hash drift).
- Notes: Hard-ban and ambiguity scans were executed as part of planning lint.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence: `docs/project_management/next/world-deps-packages-bundles-contract/decision_register.md` (DR-0002, DR-0003).
- Notes: DR-0002 and DR-0003 do not meet the Decision Register template requirements (missing required sections and explicit follow-up task mapping to task IDs).

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`
  - `docs/project_management/next/world_deps_packages_bundles_contract.md`
  - `docs/project_management/next/world-deps-packages-bundles-contract/manual_testing_playbook.md`
  - `docs/project_management/next/world-deps-packages-bundles-contract/smoke/_core.sh`
- Notes: CLI surface, config paths, and exit-code taxonomy references are consistent across the reviewed documents.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` sprint id: `world_deps_packages_bundles_contract`
  - `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json` slice deps:
    - `WDP1-code depends_on=WDP0-integ`
    - `WDP2-code depends_on=WDP1-integ`
    - `WDP3-code depends_on=CP1-ci-checkpoint,WDP2-integ`
- Notes: `tasks.json` ordering and checkpoint gating align with `sequencing.json`.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - Playbook: `docs/project_management/next/world-deps-packages-bundles-contract/manual_testing_playbook.md`
  - Smoke: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`, `docs/project_management/next/world-deps-packages-bundles-contract/smoke/macos-smoke.sh`, `docs/project_management/next/world-deps-packages-bundles-contract/smoke/_core.sh`
- Notes: Smoke scripts exist, are referenced by the playbook, and encode exit-code assertions for key fail-closed paths.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json` meta:
    - `schema_version=4`
    - `behavior_platforms_required=["linux","macos"]`
    - `ci_parity_platforms_required=["linux","macos"]`
    - `checkpoint_boundaries=["WDP2","WDP5"]`
  - Boundary-only platform-fix tasks exist only for boundary slices:
    - `WDP2-integ-core`, `WDP2-integ-linux`, `WDP2-integ-macos`
    - `WDP5-integ-core`, `WDP5-integ-linux`, `WDP5-integ-macos`
- Notes: Task model matches schema v4+ boundary-only platform-fix.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - Kickoff prompt sentinel present in all prompts under `docs/project_management/next/world-deps-packages-bundles-contract/kickoff_prompts/`.
  - Required task fields present (see preflight audit).
- Notes: Automation + worktree execution conventions are present and mechanically valid.

## Findings (must be exhaustive)

### Finding 001 — Mechanical lint failure: ADR-0017 executive summary hash drift
- Status: `DEFECT`
- Evidence:
  - `make planning-lint FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract"` fails with:
    - `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md: ADR_BODY_SHA256 mismatch (found f98e… expected 41fc…)`
  - Repro: `make adr-check ADR=docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` (exit 2)
- Impact: Planning Pack fails the mechanical quality gate; execution triads must not begin.
- Fix required (exact): Update `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` `ADR_BODY_SHA256` to `41fcd002c7e017054b2e4812420598a60ad6041277ebcfc9e41f881b5a83b29f` (or run `make adr-fix ADR=docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`), then re-run planning lint.
- If DEFECT: Alternative (one viable): Remove references to ADR-0017 from this Planning Pack’s required inputs if it is not actually a constraint for this feature; then re-run planning lint.

### Finding 002 — Decision register entries do not meet the 2-option decision standard
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-packages-bundles-contract/decision_register.md`:
    - DR-0002 lacks required sections (`Unlocks`, `Quick wins / low-hanging fruit`) for both options and has no explicit follow-up task mapping.
    - DR-0003 lacks required sections (`Cascading implications`, `Risks`, `Unlocks`, `Quick wins / low-hanging fruit`) and has no explicit follow-up tasks section.
- Impact: The plan is not implementation-ready because major execution-shaping choices are not fully justified/auditable; downstream tasks cannot reliably trace why specific tradeoffs were chosen.
- Fix required (exact): Update DR-0002 and DR-0003 to match the required template in `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (include all required sections for Option A and Option B and add explicit “Follow-up tasks (explicit)” mapping to concrete `tasks.json` task IDs).
- If DEFECT: Alternative (one viable): If a decision is not actually required to execute ADR-0011, mark it as `Superseded` and remove it from task references; keep only decisions that are execution-critical.

### Finding 003 — Decision-to-task traceability is missing in tasks.json
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json` contains references to `decision_register.md` but does not reference specific DR ids (no `DR-` references).
- Impact: Auditability requirement is not met; reviewers and integration agents cannot deterministically verify which tasks implement which decisions.
- Fix required (exact): Add explicit DR references in `tasks.json` `references` for the tasks that implement each decision, using the standard format:
  - `docs/project_management/next/world-deps-packages-bundles-contract/decision_register.md (DR-000X)`
- If DEFECT: Alternative (one viable): Add a “Decision → Tasks” mapping table to `decision_register.md` (per DR) and ensure each task references the relevant DR by id at least once (either in `tasks.json` references or in the integration END checklist requirements).

### Finding 004 — Checkpoint prompts do not close the dependency loop for non-failing platform-fix tasks
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json`: boundary final integration depends on platform-fix tasks:
    - `WDP2-integ depends_on=[..., WDP2-integ-linux, WDP2-integ-macos]` (see around `id: WDP2-integ`)
  - `docs/project_management/next/world-deps-packages-bundles-contract/kickoff_prompts/CP1-ci-checkpoint.md` instructs: “Start only failing platform-fix tasks” but does not describe how to mark non-failing platform-fix tasks complete/no-op so `WDP2-integ` can start.
  - Repo helper exists but is not referenced: `scripts/triad/mark_noop_platform_fixes_completed.sh`.
- Impact: Execution can stall at the boundary slice even when smoke is fully green (platform-fix tasks remain `pending` and block the final integrator task).
- Fix required (exact): Update `docs/project_management/next/world-deps-packages-bundles-contract/kickoff_prompts/CP1-ci-checkpoint.md` and `.../CP2-ci-checkpoint.md` to include a deterministic completion step for non-failing platform-fix tasks (e.g., run `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir ... --slice-id <slice> --from-smoke-run <run_id>` from the orchestration checkout), and require recording the smoke run evidence in `session_log.md`.
- If DEFECT: Alternative (one viable): Change `tasks.json` so `WDP2-integ` / `WDP5-integ` do not depend on platform-fix tasks by default, and instead depend only on the checkpoint task; start platform-fix tasks only when smoke fails.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: Mechanical planning lint fails, and decision/auditability/workflow gaps remain; this pack is not execution-ready.
- Required human decisions (explicit):
  - Confirm whether ADR-0017 is truly a required dependency for this feature; if yes, fix its `ADR_BODY_SHA256` drift; if no, remove it from the Planning Pack’s required inputs.
  - Decide whether platform-fix tasks are always required (even on green smoke) or are optional; align `tasks.json` deps + checkpoint prompts accordingly.
- Blockers to execution:
  - `make planning-lint FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract"` must pass.
  - Decision register DR-0002 and DR-0003 must be brought into template compliance with explicit follow-up task mapping.
