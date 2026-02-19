# Initiative 2 (Rev A) — Impact Map Touch-Set: Parseable, Validated, and Enforced at Task Finish

**Status:** Draft (implementation-ready)  
**Primary outcome:** Stop scope drift during execution by enforcing that every file touched by a triad task is pre-declared in `impact_map.md` (Create/Edit/Deprecate/Delete).  
**Rollout strategy:** **Gated** on `tasks.json` → `meta.slice_spec_version >= 2` to avoid breaking older packs.

---

## 0. Executive summary

This initiative turns `impact_map.md` from a planning artifact into an **enforceable scope fence**:

1. **Format hardening**: the Touch Set becomes machine-parseable by requiring exactly one **repo-relative** path wrapped in backticks per bullet under:
   - `### Create`
   - `### Edit`
   - `### Deprecate`
   - `### Delete`

2. **Planning-time validation**: add `docs/project_management/system/scripts/planning/validate_impact_map.py` and run it via `docs/project_management/system/scripts/planning/lint.*` (strict only for packs with `meta.slice_spec_version >= 2`).

3. **Execution-time enforcement**: modify `scripts/triad/task_finish.sh` to refuse completion if the task introduces changes to paths outside the declared touch set (strict only for packs with `meta.slice_spec_version >= 2`).

4. **Checkpoint boundary sizing reminder**: with smaller slices (Initiative 1), checkpoint groups default to **4–8** triads. Touch-set enforcement is a key safety lever that keeps larger checkpoint groups viable.

---

## 1. Goals

### 1.1 Goals (must achieve)
- Make scope drift a **mechanical failure**, not a retrospective observation.
- Force scope changes to be reflected in planning docs **before** code is merged.
- Reduce integration agent workload by limiting “surprise surfaces”.

### 1.2 Non-goals
- Per-slice impact maps (future enhancement; this initiative enforces the **feature-level** touch set).
- Directory refactor (Initiative 3).
- Changing the “do not edit planning docs inside worktrees” rule (this stays).

---

## 2. Enforcement gating (MUST HAVE)

### 2.1 Gate condition
Both **planning lint validation** and **task_finish enforcement** MUST be gated by:

- Read `<feature_dir>/tasks.json`
- If `meta.slice_spec_version >= 2`: **STRICT** mode
- Else (missing or `< 2`): **LEGACY** mode (skip or warn-only)

### 2.2 Legacy mode behavior
In legacy mode, the validator and task_finish enforcement MUST NOT block completion.

Recommended behavior:

- `validate_impact_map.py`: print a single-line warning like:

  `WARN: impact_map touch-set enforcement disabled (meta.slice_spec_version < 2).`

  Then exit 0.

- `task_finish.sh`: print a warning and skip touch-set enforcement:

  `WARN: skipping impact-map touch-set enforcement for legacy planning pack.`

This prevents bricking existing packs while enabling strict enforcement for all new packs (Initiative 1 scaffolds `slice_spec_version: 2` by default).

Clarification:
- In legacy mode, `validate_impact_map.py` MUST NOT fail even if `impact_map.md` is malformed or missing Touch Set headings (warn-only or skip).
- In legacy mode with `--emit-json`, the validator MUST output JSON only (no WARN lines) and SHOULD emit empty allowlists.

---

## 3. Proposed impact_map.md format changes (STRICT mode)

### 3.1 Only one source of truth: `## Touch set (explicit)`

All parsing and enforcement MUST ONLY read the Touch Set from:

- The `## Touch set (explicit)` section
- Specifically, the content under the H3 subsections:
  - `### Create`
  - `### Edit`
  - `### Deprecate`
  - `### Delete`

The parser MUST ignore all other sections of `impact_map.md`, even if they contain backticks.

Strictness:
- In strict mode, any `### ...` heading inside the Touch Set region that is not one of the four required headings MUST cause validation failure (prevents silent mis-parses due to typos or drift).

### 3.2 Bullet grammar (STRICT)

Within each of the four H3 sections, every entry MUST be either:

- A top-level bullet list item of the form:

  `- `path/to/file` — optional prose`

  or

  `* `path/to/file` — optional prose`

**Hard rules:**
- The line MUST start at column 0 with `- ` or `* ` (no indentation).
- The line MUST contain **exactly one** backticked token (`` `...` ``).
- The backticked token MUST be a path (see §3.3).
- Nested bullets are **not allowed** inside the Touch Set sections (they are rejected).
- If a section is empty, it MUST contain exactly one line:

  `- None`

  (case-sensitive; no backticks; no additional text on that line).

### 3.3 Path normalization rules (STRICT)

The backticked token (the path) MUST obey:

- Repo-relative (no leading `/`, no drive letters like `C:\`, no `~/`)
- No parent traversal segments:
  - MUST NOT contain `..` as a path segment
- Must use forward slashes `/` (POSIX-style)
  - Windows backslashes `\` are rejected
- A leading `./` is **allowed but normalized away**
  - e.g. `` `./Cargo.toml` `` is normalized to `Cargo.toml`
- Paths MUST NOT contain globbing (`*`, `?`, `[...]`) — treat the set as explicit, not pattern-based

#### 3.3.1 Directory allow entries
Directory allow entries MAY be supported, but only if:

- The backticked token ends with `/` (e.g., `` `crates/foo/src/` ``)
- It is treated as a **prefix allowlist**:
  - Any touched path `p` is allowed if `p.startswith("crates/foo/src/")`.

Directory entries MUST NOT be used as a substitute for “I’m not sure what we’ll touch”. Prefer file-level paths whenever reasonable.

### 3.4 Uniqueness rule (STRICT)
- Each normalized path MUST be unique across all four sections.
- A path cannot appear in both Create and Edit, etc.

This keeps classification clean and prevents “double-listing”.

---

## 4. Planning-time validation

### 4.1 Add validator: `docs/project_management/system/scripts/planning/validate_impact_map.py`

**New file:**
- `docs/project_management/system/scripts/planning/validate_impact_map.py`

#### 4.1.1 Inputs
- `--feature-dir <path>`
- Optional:
  - `--emit-json` (prints allowlists for downstream consumers)
  - `--mode strict|legacy` (optional; if omitted, derive from `tasks.json meta.slice_spec_version`)

#### 4.1.2 Parser requirements (STRICT)
- Read `<feature_dir>/impact_map.md`.
- Extract only the text within:
  - `## Touch set (explicit)` … until the next `## ` header (or EOF)
- Within that region, identify the four H3 sections and parse only:
  - lines until the next `### ` header or the end of the Touch Set region
- **Ignore fenced code blocks** inside the Touch Set region:
  - Any content between lines starting with ``` and the next ``` MUST be skipped entirely.
- In strict mode, encountering any `### ...` heading that is not Create/Edit/Deprecate/Delete MUST fail.

#### 4.1.3 Checks (STRICT)
The validator MUST fail (exit non-zero) if any of these are violated:

- `impact_map.md` exists.
- `## Touch set (explicit)` exists.
- The four required H3 headings exist: Create/Edit/Deprecate/Delete.
- Each section contains either:
  - exactly one `- None` line, OR
  - one or more valid bullet lines per §3.2
- No placeholder tokens in strict mode:
  - `<path>`
  - `TBD`, `TODO`, `WIP`, `None yet.`
- Each parsed bullet line contains exactly one backticked token.
- All paths pass normalization rules (§3.3).
- No duplicates across sections (§3.4).
- **At least one non-None entry exists across the whole Touch Set**  
  (i.e., the Touch Set cannot be entirely `- None` in strict mode).

#### 4.1.4 Existence checks (STRICT)
- For entries under:
  - `Edit`, `Deprecate`, `Delete`:
    - path MUST exist at lint time (file or directory, depending on trailing `/`)
- For entries under:
  - `Create`:
    - if path exists already: **WARN** (do not fail), because there are legitimate edge cases.
- For directory allow entries (`.../`):
  - the directory MUST exist for Edit/Deprecate/Delete
  - for Create, existence is a warn-only check

> Note: file existence is checked relative to repo root. Use `git rev-parse --show-toplevel` to locate it.

#### 4.1.5 Output: `--emit-json`
When `--emit-json` is used, output exactly one JSON object to stdout:

```json
{
  "create": ["pathA", "pathB"],
  "edit": ["pathC"],
  "deprecate": [],
  "delete": [],
  "dir_prefixes": ["crates/foo/src/"]
}
```

Where:
- `dir_prefixes` includes normalized directory entries (those ending in `/`), regardless of which bucket they were declared in (bucket classification doesn’t matter for enforcement; it matters for humans).

The validator MUST print JSON only (no extra logs) when `--emit-json` is used, so it is safe for bash scripts to consume.

Legacy mode with `--emit-json`:
- Output the same JSON shape with all lists empty, and exit 0.

---

### 4.2 Wire validator into planning lint (GATED)

**Modify:**
- `docs/project_management/system/scripts/planning/lint.sh`
- `docs/project_management/system/scripts/planning/lint.ps1`

Add (near other validators):

```bash
echo "-- impact_map.md touch-set invariants"
python3 docs/project_management/system/scripts/planning/validate_impact_map.py --feature-dir "${FEATURE_DIR}"
```

The validator itself is responsible for gating (strict vs legacy) by reading `tasks.json`.

---

### 4.3 Update impact map template + standard docs

**Modify:**
- `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`

Change the Touch Set scaffolding from `<path>` placeholders to a valid legacy-safe skeleton:

```md
### Create
- None

### Edit
- None

### Deprecate
- None

### Delete
- None
```

And add a note above the Touch Set:

> In strict packs (`meta.slice_spec_version >= 2`), at least one non-None entry is required and paths must be backticked.

**Modify:**
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`

Add a non-negotiable rule:

- Touch set bullets MUST follow §3.2 and §3.3
- Touch set is linted and enforced at triad task finish (strict packs only)

---

## 5. Execution-time enforcement (task_finish)

### 5.1 Enforcement point: `scripts/triad/task_finish.sh`

Enforce at task completion because it is the last gate before integration.

**Modify:**
- `scripts/triad/task_finish.sh`

Add a step (run in normal and `--verify-only` modes):

#### 5.1.1 Gate (STRICT vs LEGACY)
- Load `FEATURE_DIR` from `.taskmeta.json` (already done today).
- Read `<feature_dir>/tasks.json` and determine `meta.slice_spec_version`.
- If legacy: warn + skip enforcement.

#### 5.1.2 Load allowlist (STRICT)
Call the validator and parse JSON:

```bash
ALLOW_JSON="$(python3 docs/project_management/system/scripts/planning/validate_impact_map.py --feature-dir "$FEATURE_DIR_ABS" --emit-json)"
```

If the validator fails, task_finish MUST fail (planning docs must be fixed before completing the task).

#### 5.1.3 Define “touched files” baseline (STRICT)
**Touched = any path changed in this worktree since the worktree was created**, plus any untracked files.

Use `.taskmeta.json` field `created_from_sha` as the baseline.

Algorithm:

1) `BASE_SHA = created_from_sha` from `.taskmeta.json`
2) `MB = git merge-base "$BASE_SHA" HEAD`
3) Collect committed changes since baseline:

- Prefer NUL-delimited output to avoid whitespace/path parsing bugs:
  - `git diff --name-status -z -M "$MB".."HEAD"`

4) Also include any staged/unstaged changes (defensive):

- Prefer NUL-delimited output:
  - `git diff --name-status -z -M --cached`
  - `git diff --name-status -z -M`

5) Include untracked files:

- Prefer NUL-delimited output:
  - `git ls-files -z --others --exclude-standard`

#### 5.1.4 Rename handling (STRICT)
When parsing `--name-status` output:

- `R<score>  old  new` counts as **both** `old` and `new` being touched.
- `C<score>  old  new` counts as `new` being touched (optionally count both; pick one rule and document it).

This avoids loopholes where renames bypass allowlisting.

#### 5.1.5 Always-ignore set (STRICT)
Exclude triad system artifacts from touched detection:

- `.taskmeta.json`

Optionally exclude any other known internal artifacts if they exist in your environment (keep this list minimal; every ignore is a potential loophole).

#### 5.1.6 Verification logic (STRICT)
A touched file `p` is allowed if either:
- `p` is exactly in one of the allowlists (`create|edit|deprecate|delete`), OR
- `p` matches a directory prefix allow entry in `dir_prefixes`.

If any touched paths are not allowed:
- task_finish MUST fail
- print a deterministic actionable message (see §5.2)

---

### 5.2 Failure message contract

When failing, print:

- `FAIL: unplanned file touches detected (N)`
- The offending paths, one per line
- Instructions:
  - “Update `impact_map.md` on orchestration branch, commit, then rerun task_finish.”

Example:

```text
FAIL: unplanned file touches detected (3)
- crates/world/src/surprise.rs
- crates/cli/src/unplanned.rs
- Cargo.lock

To proceed:
1) On orchestration branch: update <feature_dir>/impact_map.md to include these paths (Create/Edit/Deprecate/Delete).
2) Commit planning docs update.
3) Re-run: scripts/triad/task_finish.sh --task-id <id>
```

---

### 5.3 Override / escape hatch (rare, explicit, logged)

Do not ship a bypass that becomes the default.

**Add flags to `task_finish.sh`:**
- `--allow-unplanned-touch`
- `--reason "<text>"` (required when override is used)

Rules:
- If `--allow-unplanned-touch` is set, `--reason` MUST be provided and non-empty.
- task_finish should still print the unplanned file list, but proceed.
- Record the override reason into the task artifact log (where you currently record summaries), so it is auditable.

> Do not use `--force` unless it is already established across your scripts; prefer the explicit `--allow-unplanned-touch` flag.

---

## 6. CI checkpoint sizing (baked reminder)

Default checkpoint group sizing (from Initiative 1):

- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

Touch-set enforcement reduces risk of larger checkpoint groups by preventing silent scope creep between checkpoints.

No additional checkpoint changes are required in this initiative beyond ensuring docs/templates remain consistent (Initiative 1 owns core updates).

---

## 7. Migration strategy

### 7.1 Minimal migration (recommended)
- Update the impact map standard + template.
- For **strict packs** (slice_spec_version >= 2):
  - Update `impact_map.md` Touch Set bullets to the new format:
    - `- `path` — ...`
  - Ensure at least one non-None entry exists.
- Run planning lint and fix failures.

### 7.2 Bulk migration (optional)
Write a one-off mechanical script to:
- locate `<feature_dir>/impact_map.md`
- within `## Touch set (explicit)`:
  - convert `- path — ...` into `- `path` — ...` for the four H3 sections
  - convert placeholder `<path>` lines into `- None`

Do as a mechanical-only commit.

---

## 8. Test plan

### 8.1 Validator tests (manual)
Create failing cases in a test pack with `slice_spec_version: 2`:

- a non-backticked path in Touch Set
- a bullet with two backticked tokens
- a nested bullet under Create
- a path with `../`
- a path with backslashes
- duplicate entries across sections
- all sections `- None`

Confirm validator fails with actionable messages.

### 8.2 task_finish enforcement tests (manual)
- Create a strict pack.
- Make a triad task commit touching a file not listed in impact_map.
- Confirm task_finish fails.
- Update impact_map on orchestration branch, commit, re-run finish.
- Confirm it passes.

Test rename handling:
- rename a file and commit
- ensure enforcement requires allowlisting old+new (per rule)

---

## 9. Acceptance criteria for this initiative

- `impact_map.md` Touch Set paths are parseable and validated in planning lint (strict packs only).
- `docs/project_management/system/scripts/planning/validate_impact_map.py` exists with gated strict behavior.
- `scripts/triad/task_finish.sh` blocks completion if unplanned file touches occur (strict packs only).
- Override exists but requires explicit acknowledgement and a reason and is logged.
- At least one active strict Planning Pack can complete triads without excessive friction.
