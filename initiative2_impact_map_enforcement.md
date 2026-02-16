# Initiative 2 — Impact Map Touch-Set: Make it Parseable, Validated, and Enforced at Task Finish (plus updated CI checkpoint sizing)

**Status:** Draft (implementation-ready)  
**Primary outcome:** Stop scope drift during execution by enforcing that every file touched by a triad task is pre-declared in `impact_map.md` (create/edit/deprecate/delete).

---

## 0. Executive summary

This initiative turns `impact_map.md` from a planning artifact into an **enforceable scope fence**.

Key changes:

1. **Impact map touch-set format becomes machine-parseable** by requiring repo-relative paths in backticks (`` `path/to/file` ``) under Create/Edit/Deprecate/Delete.
2. **Planning validation**: add `scripts/planning/validate_impact_map.py` and run it via `scripts/planning/lint.*`.
3. **Execution enforcement**: modify `scripts/triad/task_finish.sh` to refuse completion if the worktree diff touches files outside the declared touch set.
4. **Checkpoint boundary sizing update reminder**: as slices become smaller (Initiative 1), checkpoint groups default to **4–8** triads; touch-set enforcement is one of the safety levers that makes larger checkpoint groups viable.

---

## 1. Goals

### 1.1 Goals (must achieve)
- Make touch-set drift a **mechanical failure**, not a retrospective observation.
- Force scope changes to be reflected in planning docs **before** code is merged.
- Reduce integration agent workload by limiting “surprise surfaces”.

### 1.2 Non-goals
- Per-slice impact maps (future enhancement; this initiative enforces the feature-level touch set).
- Directory refactor (Initiative 3).
- Any change to “do not edit planning docs inside worktrees” rule (this stays).

---

## 2. Proposed impact_map.md format changes

### 2.1 Touch set sections must contain backticked paths (hard rule)

Under:

- `### Create`
- `### Edit`
- `### Deprecate`
- `### Delete`

Every bullet MUST contain exactly one repo-relative path wrapped in backticks.

**Example (new format):**

```md
### Edit
- `crates/substrate_cli/src/workspace/sync.rs` — implement gating + dry-run preview
- `crates/substrate_cli/src/main.rs` — wire subcommand
- `Cargo.toml` — add dependency X
- `Cargo.lock` — updated by cargo (expected)
```

Rules:
- Paths MUST be repo-relative (no absolute paths, no `../`).
- One bullet may include additional prose after the path (allowed), but the path must be present and unique.
- Paths may point to files or directories. If directories are allowed, they must be explicit and end with `/` and are treated as a prefix allowlist (see enforcement rules below). Prefer file-level paths whenever reasonable.

---

### 2.2 Touch set is authoritative for execution

New rule: **If a file is not in the touch set, it should not be changed by any triad task.**

If execution discovers a new necessary touch:
- Update `impact_map.md` on the **orchestration branch** (NOT in the worktree).
- Commit planning docs update.
- Then rerun/continue triads.

---

## 3. Planning-time validation

### 3.1 Add validator: validate_impact_map.py

**Add file:**
- `scripts/planning/validate_impact_map.py`

#### 3.1.1 Inputs
- `--feature-dir <path>`

#### 3.1.2 Parsing rules
- Locate `impact_map.md` at `<feature_dir>/impact_map.md`.
- Identify the “Touch set” sections:
  - `### Create`, `### Edit`, `### Deprecate`, `### Delete`
- Extract backticked tokens in those sections:
  - Use regex like `` `([^`]+)` ``.
- Classify each path into a bucket (create/edit/deprecate/delete) based on the section.

#### 3.1.3 Required checks (strict)
- Each section exists (Create/Edit/Deprecate/Delete).
- Each section contains at least one entry OR explicitly states `- None` (allowed).
- No placeholder tokens:
  - `<path>`
  - `TBD`, `TODO`, `WIP`, `None yet.`
- All extracted paths must be repo-relative:
  - Must not start with `/`
  - Must not contain `..` segments
- Paths must be unique across all four sections (no duplicates).
- Existence checks at planning time:
  - For `Edit`, `Deprecate`, `Delete`: the file/dir should exist in the repo at lint time.
  - For `Create`: the file should NOT exist yet (warn or strict; choose **warn** to avoid edge cases like “file already exists but is being populated”).
- Optional additional check:
  - If `Delete` is non-empty, require a rationale after the path (presence of `—` or `:` in the bullet line).

#### 3.1.4 Outputs (optional helper mode)
Implement a `--emit-json` flag that prints a single JSON object to stdout:

```json
{
  "create": ["pathA", "pathB"],
  "edit": ["pathC"],
  "deprecate": [],
  "delete": []
}
```

This makes it easy for `task_finish.sh` to reuse the parser (avoids reimplementing parsing in bash).

---

### 3.2 Wire validator into planning lint

**Modify:**
- `scripts/planning/lint.sh`
- `scripts/planning/lint.ps1`

Add:

```bash
echo "-- impact_map.md touch-set invariants"
python3 scripts/planning/validate_impact_map.py --feature-dir "${FEATURE_DIR}"
```

Place it near other planning validators (after spec_manifest validation is fine).

---

### 3.3 Update impact map template + standard docs

**Modify:**
- `docs/project_management/standards/templates/impact_map.md.tmpl`
  - Replace `<path>` with `` `<path>` `` in the template bullets and include a note that paths MUST be backticked.

**Modify:**
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
  - Add a non-negotiable rule:
    - “Touch set paths MUST be backticked and are linted and enforced at triad task finish.”

---

## 4. Execution-time enforcement (task_finish)

### 4.1 Enforcement point: task_finish.sh (recommended)

Enforce touch set at the point where tasks attempt to complete, because:
- it’s the last gate before code enters the integration merge path,
- it keeps the failure local to the offending task.

**Modify:**
- `scripts/triad/task_finish.sh`

Add a new step (unless `--verify-only` is used, in which case still run the enforcement):

1) Load `FEATURE_DIR` from `.taskmeta.json` (already done).
2) Parse allowed paths from `<feature_dir>/impact_map.md` by calling:
   - `python3 scripts/planning/validate_impact_map.py --feature-dir "$FEATURE_DIR_ABS" --emit-json`
3) Compute actual touched paths in the worktree:
   - Use `git status --porcelain=v1` for tracked + untracked.
   - Include:
     - modified files
     - added files
     - deleted files
     - renamed files (treat both old and new as touched)
4) Filter out ignored noise:
   - `target/`
   - `.git/`
   - OS junk: `.DS_Store`
   - Any triad-local runtime artifacts (should be none if your system is clean)
5) Verify that every touched file is in one of:
   - Create allowlist (exact match) OR
   - Edit/Deprecate/Delete allowlist (exact match) OR
   - Allowed directory prefix (if you choose to support directory entries ending in `/`)

If violations exist: **fail** with an actionable message.

---

### 4.2 Recommended allowlist semantics

To keep enforcement both strict and practical:

- If a touch-set entry ends with `/`, treat it as a directory prefix allow:
  - Example: `` `crates/foo/src/` `` allows any file under that prefix.
- Otherwise treat entries as exact file paths.

> Prefer exact paths; allow directory prefixes only when the set would otherwise be too noisy (e.g., generated modules, large but contained subtrees).

---

### 4.3 Failure message contract (what task_finish should print)

When failing, print:

- A summary:
  - `FAIL: unplanned file touches detected`
- The offending files list
- Instructions:
  - “Update `<feature_dir>/impact_map.md` on the orchestration branch, commit, then rerun task_finish.”

Example:

```text
FAIL: unplanned file touches detected (3)
- crates/world/src/surprise.rs
- crates/cli/src/unplanned.rs
- Cargo.lock

To proceed:
1) On orchestration branch: update docs/.../impact_map.md to include these paths (Create/Edit/...).
2) Commit planning docs update.
3) Re-run: make triad-task-finish TASK_ID="<task>"
```

---

### 4.4 Override / escape hatch (keep it rare and explicit)

Sometimes you need to finish a task even if the impact map is temporarily behind (e.g., emergency fix).

Add an explicit override:

- `scripts/triad/task_finish.sh --allow-unplanned-touch`

Rules:
- Only allowed for `type=investigation` tasks OR requires `--force` plus a mandatory reason string:
  - `--allow-unplanned-touch --reason "..."`

This prevents “just bypass it” becoming the norm.

---

## 5. CI checkpoint sizing (baked reminder)

As part of Initiative 1, the default checkpoint group sizing becomes:

- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

Touch-set enforcement reduces risk of larger checkpoint groups because it prevents silent scope creep between checkpoints.

No additional changes are required in this initiative beyond ensuring docs/templates reflect the new defaults (Initiative 1 owns the template updates, but mention here because it is safety-coupled).

---

## 6. Migration strategy

### 6.1 Minimal migration (recommended)
- Update `impact_map.md.tmpl` and standard.
- Update active Planning Packs’ `impact_map.md` by wrapping existing paths in backticks.
- Run planning lint and fix any failures.

### 6.2 Bulk migration (optional)
Write a one-off script to:
- find `<feature_dir>/impact_map.md`
- wrap `- path — ...` into `- `path` — ...` for bullets under Create/Edit/Deprecate/Delete.

Do this as a mechanical-only commit.

---

## 7. Test plan

### 7.1 Validator tests (manual)
- Intentionally insert:
  - a non-backticked path
  - an absolute path
  - a path with `../`
  - duplicate entries
- Confirm `validate_impact_map.py` fails.

### 7.2 task_finish enforcement tests (manual)
- Create a small test slice, change a file NOT listed in impact_map.
- Confirm `task_finish.sh` fails.
- Update impact_map on orchestration branch, commit, re-run finish.
- Confirm it passes.

---

## 8. Acceptance criteria for this initiative

- `impact_map.md` touch set paths are backticked and parseable.
- `scripts/planning/validate_impact_map.py` exists and runs in planning lint.
- `scripts/triad/task_finish.sh` blocks completion if unplanned file touches occur.
- Override exists but requires explicit acknowledgement and reason.
- At least one active Planning Pack successfully uses the enforcement without excessive friction.

