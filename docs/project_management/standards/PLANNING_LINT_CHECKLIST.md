# Planning Lint Checklist (Mechanical Quality Gate)

This checklist is **mechanical** and **non-negotiable**. The quality gate reviewer must run it and record results in:
- `docs/project_management/next/<feature>/quality_gate_report.md`
  - using `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`

If any check fails, the Planning Pack is not execution-ready.

---

## 0) Define scope

Set a shell variable (examples):

- `export FEATURE_DIR="docs/project_management/next/world_deps_selection_layer"`
- `export FEATURE_DIR="docs/project_management/next/world-sync"`

All checks below use `$FEATURE_DIR`.

---

## 1) Required files exist

These must exist:
- `$FEATURE_DIR/plan.md`
- `$FEATURE_DIR/tasks.json`
- `$FEATURE_DIR/session_log.md`
- `$FEATURE_DIR/kickoff_prompts/`

If the feature is decision-heavy or cross-platform, these must exist:
- `$FEATURE_DIR/decision_register.md`
- `$FEATURE_DIR/integration_map.md`
- `$FEATURE_DIR/manual_testing_playbook.md`
- `$FEATURE_DIR/smoke/`
  - `$FEATURE_DIR/smoke/linux-smoke.sh`
  - `$FEATURE_DIR/smoke/macos-smoke.sh`
  - `$FEATURE_DIR/smoke/windows-smoke.ps1`

---

## 2) Hard-ban and ambiguity scan (planning outputs only)

Run these against `$FEATURE_DIR` and fix any matches.

### 2.1 Hard bans
Forbidden anywhere in the feature planning outputs:
- `TBD`, `TODO`, `WIP`, `TBA`
- `open question`
- `etc.`, `and so on`

Command:
```bash
rg -n --hidden --glob '!**/.git/**' '\\b(TBD|TODO|WIP|TBA)\\b|open question|\\betc\\.|and so on' "$FEATURE_DIR"
```

### 2.2 Ambiguity bans (behavior/contracts)
Forbidden in behavior-level contracts:
- `should`, `could`, `might`, `maybe`
- `optional`, `optionally` (allowed only inside Option A/Option B comparisons in decision_register)

Command:
```bash
rg -n --hidden --glob '!**/.git/**' '\\b(should|could|might|maybe|optionally|optional)\\b' "$FEATURE_DIR"
```

If matches appear inside `decision_register.md` option comparisons, verify:
- exactly two options exist,
- the entry ends with one selected option.

---

## 3) `tasks.json` shape validation

### 3.1 JSON parse
```bash
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
```

### 3.2 Required task fields exist
This must report **no missing fields**:
```bash
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
```

### 3.3 Integration tasks require smoke scripts when smoke exists
If `$FEATURE_DIR/smoke/` exists, then every `type=integration` task must:
- reference the smoke scripts in `references`, and/or
- require running the platform smoke script in `end_checklist`.

Command:
```bash
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
```

---

## 4) Kickoff prompt coverage and triad workflow requirements

### 4.1 Kickoff prompts exist for every task
Each task `kickoff_prompt` must exist.

Command:
```bash
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
```

### 4.2 Kickoff prompts contain “no docs edits in worktrees”
Every kickoff prompt must contain the exact rule:
- `Do not edit docs/tasks/session_log.md inside the worktree.`

Command:
```bash
rg -n 'Do not edit docs/tasks/session_log\\.md inside the worktree\\.' "$FEATURE_DIR/kickoff_prompts"
```

If any kickoff prompt is missing this rule, add it.

---

## 5) Manual playbook + smoke script linkage

If `$FEATURE_DIR/manual_testing_playbook.md` exists, it must:
- include explicit commands and expected exit codes/output,
- reference all smoke scripts under `$FEATURE_DIR/smoke/`.

Command:
```bash
if test -f "$FEATURE_DIR/manual_testing_playbook.md"; then
  rg -n 'smoke/(linux-smoke\\.sh|macos-smoke\\.sh|windows-smoke\\.ps1)' "$FEATURE_DIR/manual_testing_playbook.md"
fi
```

---

## 6) Sequencing alignment

Verify:
- `docs/project_management/next/sequencing.json` includes the feature sprint entry and points to this directory.
- Task dependencies (`depends_on`) do not violate `sequencing.json`.

Minimum command:
```bash
jq -e --arg dir "$FEATURE_DIR" '.sprints[] | select(.directory==$dir) | .id' docs/project_management/next/sequencing.json
```

If a dependency exists in tasks.json that implies a sequencing change, update `sequencing.json` (or remove the dependency and fix the plan/specs) and record the final outcome in the feature’s `integration_map.md` or alignment report.

