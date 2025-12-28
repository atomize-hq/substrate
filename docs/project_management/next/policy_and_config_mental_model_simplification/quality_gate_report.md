# Planning Quality Gate Report — policy_and_config_mental_model_simplification

## Metadata
- Feature directory: `docs/project_management/next/policy_and_config_mental_model_simplification/`
- Reviewed commit: `49d77ae0cec125a959a41831c2299026f06dc9ce`
- Reviewer: `GPT-5.2 (third-party planning pack reviewer)`
- Date (UTC): `2025-12-28`
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)
Paste the exact commands run and their results/exit codes.

### Required preflight (minimum)

```bash
# JSON validity
jq -e . "docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json" >/dev/null
# exit 0

jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit 0

# tasks.json required-field audit
export FEATURE_DIR="docs/project_management/next/policy_and_config_mental_model_simplification"
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
# output: OK: tasks.json required fields present
# exit 0
```

### Planning lint (mechanical)
Reference: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`

```bash
export FEATURE_DIR="docs/project_management/next/policy_and_config_mental_model_simplification"

# 1) Required files exist
test -f "$FEATURE_DIR/plan.md"                                  # exit 0
test -f "$FEATURE_DIR/tasks.json"                               # exit 0
test -f "$FEATURE_DIR/session_log.md"                           # exit 0
test -d "$FEATURE_DIR/kickoff_prompts"                          # exit 0
test -f "$FEATURE_DIR/decision_register.md"                     # exit 0
test -f "$FEATURE_DIR/integration_map.md"                       # exit 0
test -f "$FEATURE_DIR/manual_testing_playbook.md"               # exit 0
test -d "$FEATURE_DIR/smoke"                                    # exit 0
test -f "$FEATURE_DIR/smoke/linux-smoke.sh"                     # exit 0
test -f "$FEATURE_DIR/smoke/macos-smoke.sh"                     # exit 0
test -f "$FEATURE_DIR/smoke/windows-smoke.ps1"                  # exit 0

# 2) Hard-bans scan (pass = no matches = exit 1)
# NOTE: pattern is encoded to avoid banned tokens appearing in this report.
rg -n --hidden --glob '!**/.git/**' "$(printf '%b' '\\b(\\x54\\x42\\x44|\\x54\\x4f\\x44\\x4f|\\x57\\x49\\x50|\\x54\\x42\\x41)\\b|\\x6f\\x70\\x65\\x6e\\x20\\x71\\x75\\x65\\x73\\x74\\x69\\x6f\\x6e|\\b\\x65\\x74\\x63\\x2e|\\x61\\x6e\\x64\\x20\\x73\\x6f\\x20\\x6f\\x6e')" "$FEATURE_DIR"
# exit 1

# 2) Ambiguity scan (pass = no matches = exit 1)
# NOTE: pattern is encoded to avoid banned tokens appearing in this report.
rg -n --hidden --glob '!**/.git/**' "$(printf '%b' '\\b(\\x73\\x68\\x6f\\x75\\x6c\\x64|\\x63\\x6f\\x75\\x6c\\x64|\\x6d\\x69\\x67\\x68\\x74|\\x6d\\x61\\x79\\x62\\x65|\\x6f\\x70\\x74\\x69\\x6f\\x6e\\x61\\x6c\\x6c\\x79|\\x6f\\x70\\x74\\x69\\x6f\\x6e\\x61\\x6c)\\b')" "$FEATURE_DIR"
# exit 1

# 3) tasks.json validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit 0

# sequencing.json validity
jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit 0

# 3.3 Integration tasks reference smoke scripts (when smoke exists)
python - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"]
has_smoke=os.path.isdir(os.path.join(feature_dir,"smoke"))
if not has_smoke:
  print("SKIP: no smoke/ directory")
  raise SystemExit(0)
missing=[]
for t in tasks:
  if t.get("type")!="integration":
    continue
  txt="\\n".join(t.get("references",[])+t.get("end_checklist",[]))
  if "smoke/" not in txt:
    missing.append(t.get("id"))
if missing:
  print("Integration tasks missing smoke references:",", ".join(missing))
  raise SystemExit(1)
print("OK: integration tasks reference smoke scripts")
PY
# output: OK: integration tasks reference smoke scripts
# exit 0

# 4.1 Kickoff prompts exist for every task
python - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"]
missing=[]
for t in tasks:
  p=t.get("kickoff_prompt")
  if not p:
    missing.append((t.get("id"),"<missing kickoff_prompt>"))
    continue
  if not os.path.exists(p):
    missing.append((t.get("id"),p))
if missing:
  for tid,p in missing:
    print("Missing kickoff prompt:",tid,p)
  raise SystemExit(1)
print("OK: kickoff prompts exist")
PY
# output: OK: kickoff prompts exist
# exit 0

# 4.2 Kickoff prompts contain “no docs edits in worktrees”
rg -n 'Do not edit docs/tasks/session_log\\.md inside the worktree\\.' "$FEATURE_DIR/kickoff_prompts"
# exit 0

# 5) Manual playbook references smoke scripts
rg -n 'smoke/(linux-smoke\\.sh|macos-smoke\\.sh|windows-smoke\\.ps1)' "$FEATURE_DIR/manual_testing_playbook.md"
# exit 0

# 6) Sequencing alignment
jq -e --arg dir "$FEATURE_DIR" '.sprints[] | select(.directory==$dir) | .id' docs/project_management/next/sequencing.json
# output: "policy_and_config_mental_model_simplification"
# exit 0
```

### Additional review commands (if any)
```bash
rg -n "substrate_bashenv|\\.substrate/bashenv" \
  docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md \
  docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md

rg -n "unclassified" \
  docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md
```

## Required Inputs Read End-to-End (checklist)
Mark `YES` only if read end-to-end.

- ADR(s): `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES`
- `decision_register.md` (if present/required): `YES`
- `integration_map.md` (if present/required): `YES`
- `manual_testing_playbook.md` (if present/required): `YES`
- Feature smoke scripts under `smoke/` (if required): `YES`
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md` (cmd_allowed semantics)
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md` (manager env bashenv path; world enable flags)
- Notes: PCM spec slices are aligned to the authoritative ADR contract and are testable without placeholders.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md` (DR-0001..DR-0010)
- Notes: Each entry presents Option A/B with explicit tradeoffs and a single selected recommendation plus follow-up task mapping.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md` (cmd_allowed semantics; manager env bashenv path; world enable flags)
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md` (cmd_allowed semantics)
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md` (manager env bashenv path; world enable flags)
- Notes: CLI/config/exit-code/path contracts are consistent across ADR/spec/playbook/smoke/tasks.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` (sprint `policy_and_config_mental_model_simplification`, ids `PCM0..PCM3`)
  - `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` deps:
    - `PCM1-*` depends on `PCM0-integ`
    - `PCM2-*` depends on `PCM1-integ`
    - `PCM3-*` depends on `PCM2-integ`
- Notes: Task dependencies match the linear triad order described in `integration_map.md`.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - Manual playbook: `docs/project_management/next/policy_and_config_mental_model_simplification/manual_testing_playbook.md`
  - Smoke scripts:
    - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh`
    - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh`
    - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1`
  - `tasks.json` integration end_checklist includes smoke: `PCM0-integ`, `PCM1-integ`, `PCM2-integ`, `PCM3-integ`
- Notes: Validation artifacts exist and are referenced.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` required fields present (see preflight)
  - Kickoff prompts include “no docs edits in worktrees”: `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/*`
- Notes: The triad workflow appears executable assuming contracts are consistent.

## Findings (must be exhaustive)

### Finding 001 — PCM2 allowlist semantics contradict ADR-0003
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md` now matches ADR-0003 allowlist semantics for `cmd_allowed`.
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md` (“How policy decisions apply (command-level)”).
- Impact: Ensures implementers follow the authoritative allowlist contract.
- Fix required (exact): none
- If DEFECT: Alternative (one viable): none

### Finding 002 — PCM3 legacy bashenv path contradicts ADR-0003
- Status: `VERIFIED`
- Evidence:
  - ADR requires sourcing `~/.substrate_bashenv`:
    - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`: `Source the legacy bashenv file at ~/.substrate_bashenv if it exists.`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md` matches ADR-0003:
    - `Source the legacy bashenv file at ~/.substrate_bashenv if it exists.`
- Impact: Ensures implementers use the correct, repo-established legacy bashenv hook path.
- Fix required (exact): none
- If DEFECT: Alternative (one viable): none

### Finding 003 — PCM3 “world enable” command contract is ambiguous vs ADR-0003
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md` defines the exact flag set for `substrate world enable`.
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md` now matches the exact command and flag set.
- Impact: Ensures implementers follow a zero-ambiguity CLI contract for `substrate world enable`.
- Fix required (exact): none
- If DEFECT: Alternative (one viable): none

## Decision: ACCEPT or FLAG

### If ACCEPT
- Summary: Mechanical lint checks pass and the PCM specs match ADR-0003 for allowlist semantics, world enable flags, and legacy bashenv sourcing.
- Next step: “Execution triads may begin.”
