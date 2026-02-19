# Initiative 2 (Rev A) — Impact Map Touch-Set Enforcement: Work Breakdown

Source spec:
- `initiative2_impact_map_enforcement_revA.md`

Goal:
- Make `impact_map.md` Touch Set a machine-validated + task-finish-enforced scope fence so triad tasks cannot complete when they touch undeclared paths.

Rollout gating (non-negotiable):
- **STRICT** mode only when `<feature_dir>/tasks.json` has `meta.slice_spec_version >= 2`.
- **LEGACY** mode when missing or `< 2`: validator and task_finish enforcement must not block completion.

---

## Feature brief

### Primary outcome
Stop scope drift during execution by enforcing that every path touched by a triad task is pre-declared in `impact_map.md` under Create/Edit/Deprecate/Delete.

### Primary user / JTBD
Engineers and agents running triads need a deterministic “unplanned touches” failure at `task_finish`, with actionable remediation (“update impact_map on orchestration branch; commit; rerun task_finish”).

### In scope
- `docs/project_management/system/scripts/planning/validate_impact_map.py` (strict + legacy gating) with `--emit-json`.
- Wire validator into planning lint (`docs/project_management/system/scripts/planning/lint.sh`, `docs/project_management/system/scripts/planning/lint.ps1`) via a single call.
- Update `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl` to a legacy-safe skeleton using `- None`.
- Update `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md` to document strict format + enforcement behavior.
- Enforce Touch Set at `scripts/triad/task_finish.sh` (strict only), using:
  - `.taskmeta.json created_from_sha` baseline,
  - NUL-delimited git output (`-z`) to avoid parsing bugs,
  - rename semantics (`R*` counts old+new; `C*` counts new),
  - directory-prefix allowlist entries (`.../`) via `dir_prefixes`.
- Add explicit override flags to task_finish:
  - `--allow-unplanned-touch`
  - `--reason "<text>"` (required when override used)
  - override must still print the unplanned list and must be logged in the machine-parseable summary.

### Out of scope
- Per-slice impact maps.
- Directory refactor work.
- Changing the “do not edit planning docs inside worktrees” rule.

### Hard constraints / deterministic contracts
- Parsing/enforcement reads **only** `impact_map.md`’s `## Touch set (explicit)` region and only the four H3 sections:
  - `### Create`, `### Edit`, `### Deprecate`, `### Delete`
- Ignore fenced code blocks (` ``` ` … ` ``` `) inside the Touch Set region.
- In strict mode, any other `### ...` inside the Touch Set region is a failure (prevents silent mis-parses).
- Bullet grammar in strict mode:
  - Column-0 `- ` or `* `
  - Exactly one backticked token (`` `...` ``) per bullet
  - No nested/indented bullets in Touch Set sections
  - Empty section is exactly `- None` (case-sensitive, no extra text, no backticks)
- Path normalization in strict mode:
  - repo-relative, POSIX `/` separators, no `..` segments, no globbing, no backslashes
  - leading `./` allowed but normalized away
  - directory allow entries must end with `/` and are prefix allows

---

## Vertical slices (value-first; gated strict rollout)

### S1 — Planning-time Touch Set validation (gated)

**User value**
- Strict packs get deterministic lint failures for malformed Touch Sets and a single JSON format that task_finish can consume.
- Legacy packs are not blocked.

**Done definition**
- `validate_impact_map.py` exists, is wired into planning lint, and behaves correctly in strict vs legacy.
- Template and standard docs updated so new packs are compatible by default.

**Acceptance criteria**
- Strict pack: invalid Touch Set fails `make planning-lint FEATURE_DIR=...` with actionable error messages.
- Legacy pack: validator never blocks; it warns (or silently skips) and exits 0.
- `--emit-json` prints JSON-only, stable shape; legacy emits JSON-only empty lists.

#### S1.T1 — Implement validator gating + region parser (strict/legacy)

**Outcome**
- New validator that selects strict/legacy mode and can isolate the Touch Set region and sections deterministically.

**Inputs/outputs**
- Input: `--feature-dir <path>`
- Optional: `--mode strict|legacy` (overrides auto-derivation)
- Optional: `--emit-json` (JSON-only output)

**Implementation notes**
- Add file: `docs/project_management/system/scripts/planning/validate_impact_map.py`
- Mode derivation (when `--mode` not provided):
  - Read `<feature_dir>/tasks.json`
  - STRICT if `meta.slice_spec_version` is an int >= 2
  - Else LEGACY
- Parse only:
  - The region under `## Touch set (explicit)` until next `## ` header or EOF
  - Within region, parse the four required H3 headings and their content
  - Skip fenced blocks inside the region (lines from ``` to next ``` inclusive)
  - In STRICT: fail if any `### ` heading is present that is not one of the four required headings

**Sub-task checklist**
- Add `docs/project_management/system/scripts/planning/validate_impact_map.py` with argparse and exit codes.
- Implement:
  - repo root discovery via `git rev-parse --show-toplevel`
  - reading `<feature_dir>/tasks.json` (for auto mode)
  - extracting the Touch Set region
  - section splitting for Create/Edit/Deprecate/Delete
  - fenced-block skipping
- LEGACY behavior:
  - if `--emit-json`: output JSON-only empty allowlists and exit 0
  - else: print one-line WARN to stderr and exit 0
- STRICT behavior:
  - if required headers missing: fail
- Verify with manual runs on one strict pack and one legacy pack.

#### S1.T2 — Implement strict checks + normalization + existence rules + `--emit-json`

**Outcome**
- Validator enforces strict bullet grammar and path normalization and produces allowlists for enforcement.

**Implementation notes**
- Strict checks:
  - `impact_map.md` exists
  - `## Touch set (explicit)` exists
  - all four H3 headings exist
  - each section either:
    - exactly `- None`, OR
    - one or more valid bullets
  - strict Touch Set must have at least one non-None entry across all sections
  - placeholder ban: strict mode rejects placeholder tokens (see `initiative2_impact_map_enforcement_revA.md` §4.1.3)
  - exactly one backticked token per bullet
  - uniqueness across all buckets
- Normalization:
  - reject: absolute (`/`), `~`, drive letters, backslashes, `..` segments, glob tokens
  - normalize: strip leading `./`
  - forward slashes only
- Existence checks relative to repo root:
  - Edit/Deprecate/Delete: must exist (file or dir depending on trailing `/`)
  - Create: if exists, warn to stderr but do not fail
- `--emit-json` output shape (JSON-only):
  - `create|edit|deprecate|delete` lists (normalized)
  - `dir_prefixes` list (normalized, endswith `/`)

**Sub-task checklist**
- Implement strict bullet parsing:
  - only accept column-0 `- ` or `* `
  - reject any indentation in Touch Set region
  - treat `- None` as the only valid “empty section” content
- Extract backticked token and enforce “exactly one token” per bullet.
- Apply path normalization and validation.
- Track duplicates and fail strict with a list of offending duplicates.
- Implement existence checks, producing WARN lines to stderr for create-existing cases.
- Implement `--emit-json`:
  - suppress all non-JSON output (send logs to stderr; stdout must be pure JSON)
- Manual validation matrix (strict pack):
  - non-backticked path
  - bullet with two backticked tokens
  - nested bullet
  - `../` path
  - backslashes
  - duplicate across sections
  - all sections `- None`

#### S1.T3 — Wire validator into lint + update template + update standard

**Outcome**
- Lint runs validator; templates/standards reflect the new contract and are legacy-safe.

**Implementation notes**
- Modify:
  - `docs/project_management/system/scripts/planning/lint.sh`
  - `docs/project_management/system/scripts/planning/lint.ps1`
  - add a step: `python3 docs/project_management/system/scripts/planning/validate_impact_map.py --feature-dir ...`
  - validator is responsible for gating strict vs legacy via tasks.json
- Modify template: `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`
  - Touch Set sections become:
    - `- None` in each of the four sections
  - Add note: strict packs require ≥1 non-None entry and backticked paths
- Modify standard: `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
  - document Touch Set bullet grammar and path rules
  - document that enforcement runs at triad task finish for strict packs only

**Sub-task checklist**
- Add lint step in bash and pwsh linters near other validators.
- Update the impact map template to remove `<path>` placeholders (use `- None`).
- Update the impact map standard with:
  - strict format requirements
  - gating behavior
  - enforcement behavior and remediation workflow
- Run `make planning-lint` on at least one strict pack after updating its impact_map format (if needed).

---

### S2 — Execution-time enforcement in `task_finish` (gated)

**User value**
- Strict packs cannot “finish” a triad task that touches undeclared paths.
- Failures are local, fast, and actionable.

**Done definition**
- `task_finish.sh` enforces allowlists in strict packs using NUL-safe git output and baseline semantics.
- Override exists, requires reason, prints unplanned list, and is auditable.

**Acceptance criteria**
- Strict pack: touching an undeclared file causes `task_finish` failure with the specified output.
- Strict pack: exact match allowlist and directory prefix allowlist both work.
- Strict pack: rename requires allowlisting old+new; copy requires allowlisting new.
- Legacy pack: enforcement is skipped and does not block task completion.
- Override: requires non-empty reason; prints unplanned list; logs override in machine-parseable summary.

#### S2.T1 — Add gating + allowlist load to `task_finish.sh`

**Outcome**
- task_finish decides strict vs legacy via `meta.slice_spec_version` and loads allowlists via validator `--emit-json` in strict.

**Implementation notes**
- Modify: `scripts/triad/task_finish.sh`
- Add CLI flags:
  - `--allow-unplanned-touch`
  - `--reason "<text>"` (required if override set)
- In legacy:
  - print WARN and skip enforcement entirely
- In strict:
  - call: `python3 docs/project_management/system/scripts/planning/validate_impact_map.py --feature-dir "$FEATURE_DIR_ABS" --emit-json`
  - if validator exits non-zero: fail task_finish (planning docs must be fixed first)

**Sub-task checklist**
- Extend arg parser to capture override and reason.
- Validate: if override set, reason must be present and non-empty.
- Read `slice_spec_version` from `${TASKS_JSON}`.
- Implement:
  - if strict: capture JSON and parse allowlists + prefixes using `jq`
  - if legacy: warn and continue
- Add machine-parseable summary fields for enforcement status (see S2.T3).

#### S2.T2 — Compute touched paths since worktree creation baseline (NUL-safe)

**Outcome**
- Deterministic touched-path set computed from baseline plus staged/unstaged/untracked.

**Implementation notes**
- Baseline:
  - `BASE_SHA` = `.taskmeta.json created_from_sha`
  - `MB` = `git merge-base "$BASE_SHA" HEAD`
- Collect (NUL-delimited):
  - `git diff --name-status -z -M "$MB..HEAD"`
  - `git diff --name-status -z -M --cached`
  - `git diff --name-status -z -M`
  - `git ls-files -z --others --exclude-standard`
- Rename handling:
  - `R*`: both old and new
  - `C*`: new only
- Always ignore:
  - `.taskmeta.json`

**Sub-task checklist**
- Implement a NUL-safe parser for `--name-status -z` output (do not rely on whitespace splitting).
- Deduplicate touched paths.
- Add a debug-only print path list behind an env var (optional) if you need to troubleshoot.
- Verify locally by creating:
  - a rename
  - a copy
  - a delete
  - an untracked file
  - staged-only changes

#### S2.T3 — Enforce allowlist + deterministic failure output + override logging

**Outcome**
- Blocks task_finish on unplanned touches (strict); override allows but logs reason.

**Implementation notes**
- Allowed if:
  - exact match in any bucket (`create|edit|deprecate|delete`), OR
  - matches any `dir_prefixes` entry (prefix match)
- Failure output contract (stderr):
  - `FAIL: unplanned file touches detected (N)`
  - one path per line prefixed with `- `
  - remediation steps (update impact_map on orchestration branch, commit, rerun task_finish)
- Override behavior:
  - still prints the unplanned list
  - proceeds after printing
  - logs reason in task_finish’s machine-parseable stdout summary

**Sub-task checklist**
- Implement allow check function for exact and prefix matches.
- If violations and no override: exit non-zero with failure contract message.
- If violations and override: print list and continue.
- Extend machine-parseable stdout contract:
  - `CHECKS=...impact_map_touchset:<skipped|enforced|overridden>...`
  - if overridden: `IMPACT_MAP_OVERRIDE_REASON=<text>`
- Validate end-to-end on a strict pack worktree:
  - touch a non-allowlisted file → fail
  - update impact_map on orchestration branch and retry → pass
  - rename file → require old+new allowlisting

---

### S3 — Migration tooling (optional; reduces adoption friction)

**User value**
- Converts active packs’ Touch Sets mechanically to the new strict format with minimal review overhead.

**Done definition**
- A mechanical migration script exists and is idempotent.
- At least one active strict pack is migrated and passes planning lint.

**Acceptance criteria**
- Script only modifies Touch Set region and only the four H3 sections.
- Second run produces no diffs.
- Migrated strict pack passes `make planning-lint`.

#### S3.T1 — Add migration script for Touch Set bullets

**Outcome**
- One-off converter for Touch Set bullets.

**Implementation notes**
- New file: `docs/project_management/system/scripts/planning/migrate_impact_map_touchset_v1_to_v2.py`
- Behavior:
  - locate `## Touch set (explicit)` region
  - under Create/Edit/Deprecate/Delete:
    - convert `- path — ...` → `- `path` — ...` when a single path-like token exists
    - convert `<path>` placeholder to `- None`
  - do not touch other sections or fenced code blocks

**Sub-task checklist**
- Implement in a deterministic way and preserve line endings as-is.
- Add `--check` option to fail if changes would be made (optional).
- Validate on one sample file and rerun to ensure idempotence.

#### S3.T2 — Migrate at least one active strict Planning Pack

**Outcome**
- Demonstrate low-friction adoption.

**Sub-task checklist**
- Pick one strict pack under `docs/project_management/_archived/next/*` with `meta.slice_spec_version >= 2`.
- Run migration script.
- Fix any strict validation failures.
- Run `make planning-lint FEATURE_DIR=...` and record results (for reviewer notes / PR description).

---

## Dependency graph (text)

- `S1` blocks `S2` (task_finish consumes validator `--emit-json` contract).
- `S2` can ship without `S3`.
- `S3` depends on `S1` (format rules + validator exist).

Critical path:
- `S1.T1 → S1.T2 → S1.T3 → S2.T1 → S2.T2 → S2.T3`

---

## Risks / unknowns and mitigations

- Risk: NUL-delimited parsing in bash is error-prone.
  - Mitigation: keep parsing NUL-safe; if it becomes brittle, add a tiny Python helper that prints touched paths as JSON and consume it with `jq`.
- Risk: enforcement friction for common auto-touched files (for example `Cargo.lock`).
  - Mitigation: require explicit allowlisting; keep override rare and auditable.

---

## Milestones

- M1 (after S1): strict packs get planning-lint validation for Touch Set; templates/standards updated.
- M2 (after S2): strict packs are scope-fenced at task_finish with audited override.
- M3 (optional, after S3): migration reduces adoption overhead across existing packs.

---

## Workstreams

### WS-VAL — Validator + lint wiring
Touch surface:
- `docs/project_management/system/scripts/planning/validate_impact_map.py`
- `docs/project_management/system/scripts/planning/lint.sh`
- `docs/project_management/system/scripts/planning/lint.ps1`

Tasks:
- `S1.T1`, `S1.T2`, `S1.T3`

### WS-ENF — task_finish enforcement
Touch surface:
- `scripts/triad/task_finish.sh`

Tasks:
- `S2.T1`, `S2.T2`, `S2.T3`

### WS-DOC — Standards + templates
Touch surface:
- `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`

Tasks:
- Doc portion of `S1.T3`

### WS-INT — Integration validation
Depends on:
- WS-VAL, WS-ENF, WS-DOC

Purpose:
- Run end-to-end validation on one strict pack worktree and confirm:
  - planning lint behavior
  - task_finish strict enforcement behavior
  - override behavior + logging
