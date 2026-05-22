# Planning Quality Gate Report — world_process_exec_tracing_parity

## Metadata

- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
- Reviewed commit: `a1e37fa0ba6803a454d7a4e4b48af270bd66cc51`
- Reviewer: `Third-party reviewer (Codex)`
- Date (UTC): `2026-02-07`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)

```bash
export FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/packs/sequencing.json >/dev/null
# exit=0

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
# exit=0
```

### Planning lint (mechanical)

Reference: `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`

```bash
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=0
```

### Additional review commands (selected)

```bash
# Sequencing spine entry exists for the feature
jq -r '.sprints[] | select(.id=="world_process_exec_tracing_parity") | {order,id,title,directory,branch,status,sequence}' \
  docs/project_management/packs/sequencing.json
# exit=0

# tasks.json meta (schema/cross-platform/checkpoints)
jq -r '.meta|{schema_version,cross_platform,automation,behavior_platforms_required,ci_parity_platforms_required,checkpoint_boundaries}' \
  "$FEATURE_DIR/tasks.json"
# exit=0
```

## Required Inputs Read End-to-End (checklist)

Mark `YES` only if read end-to-end.

- ADR(s): `YES` (`docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`)
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WPEP0`..`WPEP3`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/`: `YES`
- `docs/project_management/packs/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/system/standards/triad/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/system/standards/planning/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts

- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → exit 0 (hard-ban + ambiguity scans passed)
- Notes: Mechanical scans passed; contract docs use explicit MUST/SHOULD language.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)

- Result: `PASS`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md` (DR-0001..DR-0009)
- Notes: Each major decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection; follow-ups map to slice IDs.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)

- Result: `PASS`
- Evidence:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
- Notes: Event types, diagnostics fields, reason codes, and smoke/playbook expectations are consistent.

### 4) Sequencing and dependency alignment

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json` sprint `world_process_exec_tracing_parity` includes `WPEP0..WPEP3`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` deps enforce WPEP0 → CP1 → WPEP1 → WPEP2 → WPEP3 → CP2
- Notes: First slice of the next checkpoint group depends on the prior checkpoint task (CP1), preventing checkpoint bypass.

### 5) Testability and validation readiness

- Result: `PASS`
- Evidence:
  - Manual playbook: `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
  - Smoke scripts: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/*`
- Notes: Smoke scripts exist for `meta.behavior_platforms_required` (linux/macos/windows) and are referenced by the manual playbook with expected exit codes.

### 5.1) Cross-platform parity task structure (schema v4)

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` meta (`schema_version=4`, `checkpoint_boundaries=["WPEP0","WPEP3"]`)
  - Only boundary slices define `*-integ-core` / `*-integ-<platform>` tasks.
- Notes: Non-boundary slices define only `X-integ` (v4 boundary-only platform-fix model).

### 6) Triad interoperability (execution workflow)

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` invariants validated via `make planning-lint`
  - kickoff prompt sentinel validated via `make planning-lint`
- Notes: Pack is automation-ready (`meta.automation.enabled=true`) and includes `ci_checkpoint_plan.md`.

## Findings (must be exhaustive)

### Finding 001 — Mechanical planning lint passes

- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → exit 0
- Impact: Confirms required artifacts exist; hard-ban/ambiguity scans pass; tasks.json invariants and sequencing alignment pass.
- Fix required (exact): none

### Finding 002 — Decision register satisfies “exactly two viable options” rule

- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md` (DR-0001..DR-0009)
- Impact: Implementation can proceed without unresolved “which approach?” forks.
- Fix required (exact): none

### Finding 003 — Spec manifest covers contract surfaces with one authoritative owner each

- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/spec_manifest.md` (coverage matrix)
- Impact: Reduces drift risk across ADR/specs/protocol/schema/security/env inventory.
- Fix required (exact): none

### Finding 004 — Protocol/schema/spec alignment for `process_events*` and `world_process_*` events

- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` (ProcessEvent + diagnostics)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` (required fields + join keys)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md`
- Impact: Avoids contract contradictions during implementation, especially around diagnostics and joinability.
- Fix required (exact): none

### Finding 005 — Smoke scripts and manual playbook are runnable and specify expected exit codes/outcomes

- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md` (Cases 1–3)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`
- Impact: Enables bounded, cross-platform validation without inventing ad-hoc commands during execution.
- Fix required (exact): none

### Finding 006 — CI checkpoint plan is bounded and deterministically wired into `tasks.json`

- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/ci_checkpoint_plan.md` (CP1/CP2 and bounds justification)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` (`CP1-ci-checkpoint` depends on `WPEP0-integ-core`; `WPEP1-*` depends on `CP1-ci-checkpoint`)
- Impact: Prevents starting downstream slices before bounded CI gates run at the defined seam.
- Fix required (exact): none

## Decision: ACCEPT

### If ACCEPT

- Summary: Planning Pack passes mechanical lint; decisions/spec ownership/contract consistency/sequencing/testability/checkpoint wiring meet the required standards.
- Next step: “Execution triads may begin.”

---

# Addendum — 2026-02-08 third-party re-review (post-ADR alignment)

## Metadata

- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
- Reviewed commit: `a1e37fa0ba6803a454d7a4e4b48af270bd66cc51`
- Reviewer: `Third-party reviewer (Codex)`
- Date (UTC): `2026-02-08`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"

# Mechanical lint (required)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=0

# ADR executive summary drift guard (fix after ADR edits)
make adr-fix ADR=docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md
# exit=0

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/packs/sequencing.json >/dev/null
# exit=0

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
# exit=0
```

## Findings (additive)

### Finding 007 — ADR decision summary aligns with “two-option decisions” rule

- Status: `VERIFIED`
- Evidence: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` (Decision Summary list for DR-0001)
- Impact: Prevents “Option C” drift in ADR summaries that could confuse implementers and reviewers.
- Fix required (exact): none

---

# Addendum — 2026-02-14 third-party drift review (docs consistency + execution readiness)

## Metadata

- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
- Reviewed commit: `9b5532aae65f55b998247253db85b10a1f27f4d1`
- Reviewer: `Third-party reviewer (Codex)`
- Date (UTC): `2026-02-14`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"

# Mechanical lint (required)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=0

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=0

jq -e . docs/project_management/packs/sequencing.json >/dev/null
# exit=0

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
# exit=0

# Evidence: smoke commands in task acceptance/kickoffs
rg -n "SUBSTRATE_SMOKE_SLICE_ID=WPEP(0|1|2|3)" "$FEATURE_DIR/tasks.json" "$FEATURE_DIR/kickoff_prompts/WPEP*-integ.md"
# exit=0

# Evidence: linux/macos smoke scripts are OS-gated (cannot both run on one host)
rg -n "linux smoke is supported only on Linux" "$FEATURE_DIR/smoke/linux-smoke.sh"
rg -n "macos smoke is supported only on macOS" "$FEATURE_DIR/smoke/macos-smoke.sh"
# exit=0

# Evidence: protocol requires ts_unix_ns but schema does not list it as required
rg -n "ts_unix_ns" "$FEATURE_DIR/PROTOCOL.md"
rg -n "## 3\\) World process event family" "$FEATURE_DIR/SCHEMA.md"
# exit=0
```

## Required Inputs Read End-to-End (checklist)

- ADR(s): `YES` (`docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`)
- `spec_manifest.md`: `YES`
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in the track: `YES` (`WPEP0`..`WPEP3`)
- `manual_testing_playbook.md`: `YES`
- All feature specs/contract docs in the manifest (`PROTOCOL.md`, `SCHEMA.md`, `SECURITY.md`, `contract.md`): `YES`
- `docs/project_management/packs/sequencing.json`: `YES`
- Standards (all listed in `docs/project_management/system/standards/planning/PLANNING_QUALITY_GATE_PROMPT.md`): `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts

- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → exit 0
- Notes: Hard-ban and ambiguity scans pass mechanically.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)

- Result: `PASS`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md` (DR-0001..DR-0009)
- Notes: Each DR entry is A/B with explicit selection and follow-ups.

### 3) Contract consistency (schema/protocol consistency; join fields)

- Result: `FAIL`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` requires `ts_unix_ns` in ProcessEvent (see “ProcessEvent (base fields; required)”)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` required fields list for `world_process_*` does not include `ts_unix_ns`
- Notes: Required-field set for `world_process_*` is not consistent across the protocol and trace schema docs.

### 4) Sequencing and dependency alignment

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/sequencing.json` sprint `world_process_exec_tracing_parity` sequences `WPEP0..WPEP3`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` enforces `WPEP1-*` depends on `CP1-ci-checkpoint`, and `WPEP2-*` depends on `WPEP1-integ`, and `WPEP3-*` depends on `WPEP2-integ`
- Notes: No slice starts before its prerequisites integrate.

### 5) Testability and validation readiness

- Result: `FAIL`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` includes both Linux and macOS smoke commands in single-task acceptance criteria (e.g., `WPEP1-integ`)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/kickoff_prompts/WPEP1-integ.md` requires running both OS-specific smoke scripts
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh` exits non-zero when not on Linux; `smoke/macos-smoke.sh` exits non-zero when not on macOS
- Notes: Several integration tasks have acceptance criteria that cannot be satisfied by a single agent on a single host OS, violating “runnable acceptance criteria” and the triad execution model.

### 5.1) Cross-platform parity task structure (schema v4+)

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` `meta.schema_version=4` and `meta.checkpoint_boundaries=["WPEP0","WPEP3"]`
  - Only boundary slices define `*-integ-core` / `*-integ-<platform>` tasks; non-boundary slices use only `X-integ`
- Notes: Boundary-only platform-fix model is applied correctly.

### 6) Triad interoperability (execution workflow)

- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` validated kickoff sentinel coverage.
- Notes: Worktree guardrails appear consistent with automation mode.

## Findings (additive)

### Finding 008 — Mechanical planning lint passes at reviewed commit

- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → exit 0
- Impact: Confirms the pack still passes mechanical gates (required artifacts, bans, tasks invariants, sequencing alignment).
- Fix required (exact): none

### Finding 009 — DEFECT: Integration task acceptance criteria require mutually exclusive OS-specific smoke scripts

- Status: `DEFECT`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` lines containing both:
    - `SUBSTRATE_SMOKE_SLICE_ID=WPEP1 ... smoke/linux-smoke.sh` and
    - `SUBSTRATE_SMOKE_SLICE_ID=WPEP1 ... smoke/macos-smoke.sh`
      (see `rg` hits around `tasks.json:393-407`)
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/kickoff_prompts/WPEP1-integ.md:15-16` requires both commands
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh:8` enforces Linux-only
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh:8` enforces macOS-only
- Impact: A single integration agent cannot satisfy the acceptance criteria on one host OS; execution will either stall or produce non-auditable “we skipped requirements” behavior.
- Fix required (exact):
  - For `WPEP0-integ`, `WPEP1-integ`, `WPEP2-integ`, `WPEP3-integ`:
    - Replace the paired Linux+macOS smoke commands in `acceptance_criteria`/`end_checklist` with platform-specific requirements that are runnable on the executing host (e.g., “On Linux run linux-smoke.sh; on macOS run macos-smoke.sh; on Windows run windows-smoke.ps1”), OR move smoke execution requirements entirely to the checkpoint ops tasks (`CP*-ci-checkpoint`) via `make feature-smoke` so the command is runnable from any host.
  - Apply the same correction to the corresponding kickoff prompts under `kickoff_prompts/`.
- If DEFECT: Alternative (one viable):
  - If the intent is “cross-platform smoke at bounded seams only”, remove smoke commands from non-checkpoint integration tasks and require smoke dispatch only in `CP2-ci-checkpoint` (and optionally set `CP1` `feature_smoke=true` if WPEP0 smoke is required early).

### Finding 010 — DEFECT: `ts_unix_ns` required in protocol but not specified in trace schema required set

- Status: `DEFECT`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` lists `ts_unix_ns` as a required base field for ProcessEvent.
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` required field list for `world_process_*` does not include `ts_unix_ns`.
- Impact: Implementers do not have a single authoritative “required fields” set for world process events; this increases the risk of drift between world-service payloads and persisted trace records.
- Fix required (exact):
  - Either (A) add `ts_unix_ns` to the required fields list in `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`, or
  - (B) change `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` to make `ts_unix_ns` explicitly optional and document whether it must/must-not be persisted into `trace.jsonl`.
- If DEFECT: Alternative (one viable):
  - Keep `ts_unix_ns` as protocol-required (for ordering/determinism) but explicitly document in `SCHEMA.md` as “transport-only; may be omitted from canonical trace” (if that is the intended contract) so implementers do not guess.

## Decision: FLAG FOR HUMAN REVIEW

### Summary

- The pack passes mechanical lint, but has two execution-blocking doc/contract defects: (1) integration acceptance criteria requiring mutually exclusive OS-specific smoke runs, and (2) a protocol/schema required-field mismatch for `ts_unix_ns`.
- Fix the defects above before starting execution triads so tasks remain runnable and contracts remain unambiguous.

---

# Re-review Addendum — world_process_exec_tracing_parity (Latest)

RECOMMENDATION: ACCEPT

## Metadata (re-review)

- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
- Reviewed commit: `9b5532aae65f55b998247253db85b10a1f27f4d1`
- Reviewer: Codex (third-party reviewer)
- Date (UTC): 2026-02-15
- Recommendation detail: supersedes prior FLAG decision above

## Evidence: Commands Run (verbatim; re-review)

```bash
export FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit 0
jq -e . docs/project_management/packs/sequencing.json >/dev/null
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

# Mechanical lint
FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" make planning-lint
# exit 0
```

## Gate Results (re-review; PASS/FAIL with evidence)

### Mechanical lint (non-negotiable)

- Result: `PASS`
- Evidence: `FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" make planning-lint` → exit 0

### Prior DEFECT 009 remediation check (smoke acceptance criteria runnable)

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` no longer requires mutually-exclusive OS smoke runs on a single integration task:
    - `WPEP1-integ` acceptance criteria are local/integration-only (`make integ-checks`), see `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` around `WPEP1-integ`.
  - Cross-platform smoke is scheduled in the checkpoint ops task:
    - `CP2-ci-checkpoint` acceptance criteria dispatches `make feature-smoke ... PLATFORM=behavior ...` (see `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` under `CP2-ci-checkpoint`).

### Prior DEFECT 010 remediation check (`ts_unix_ns` required-field alignment)

- Result: `PASS`
- Evidence:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` requires `ts_unix_ns`.
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` includes `ts_unix_ns` in the `world_process_*` required fields list (see `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` under “Required fields”).

## Findings (re-review; additive)

### Finding R-001 — Mechanical lint passes at reviewed commit

- Status: `VERIFIED`
- Evidence: `FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" make planning-lint` → exit 0
- Impact: Confirms the Planning Pack passes all mechanical gates and remains executable as-authored.
- Fix required (exact): none

### Finding R-002 — Previously flagged smoke acceptance-criteria defect is resolved

- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` no longer requires both `smoke/linux-smoke.sh` and `smoke/macos-smoke.sh` to be run in a single task’s acceptance criteria; cross-platform smoke dispatch is defined in `CP2-ci-checkpoint`.
- Impact: Integration tasks are runnable by a single agent on a single host OS; cross-platform validation remains bounded and deterministic via checkpoints.
- Fix required (exact): none

### Finding R-003 — Protocol/schema required-field mismatch for `ts_unix_ns` is resolved

- Status: `VERIFIED`
- Evidence: `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` lists `ts_unix_ns` as a required field for `world_process_*`.
- Impact: Implementers have a single coherent “required fields” contract across transport and persistence.
- Fix required (exact): none

## Decision: ACCEPT (re-review)

### Summary

- Mechanical lint passes and the previously flagged execution blockers are remediated; the pack is implementation-ready.

### Next step

- Execution triads may begin.
