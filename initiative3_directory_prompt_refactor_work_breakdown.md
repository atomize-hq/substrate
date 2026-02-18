# Initiative 3 — Directory & Prompt Architecture Refactor: Work Breakdown

Source spec:
- `initiative3_directory_prompt_refactor.md`

Decomposition strategy:
- **Risk-first / dependency-first**: ship a compatibility surface first so directory moves do not break enforcement or triad orchestration.

---

## Feature brief

### Goal
Restructure the project management system + artifacts so prompts/standards/scripts are portable and planning packs scale (slices as directories), while keeping all existing enforcement and triad workflows functioning during the transition.

### Primary user
Repo maintainers + engineers/agents running the planning/triad automation via `make` and `scripts/*`.

### Problem / why now
- Current PM content is interleaved (system + artifacts), creating path chaos and agents reading the wrong docs.
- Planning packs don’t scale in a flat directory; slice docs and kickoff prompts become unmanageable.
- Enforcement and orchestration currently depend on stable paths (validators, worktree discovery, planning pack roots).

### In scope
- Define and implement a **compatibility surface** so old entrypoints keep working while new paths are introduced.
- Introduce `docs/project_management/system/` and migrate standards/templates/prompts into it (no “prompt bloat” in standards).
- Add a **single-purpose planning agent dispatcher** exposed as a `make` target plus an underlying script.
- Move PM scripts into the system, with wrappers preserved at old paths.
- Migrate ADRs to `docs/project_management/adrs/<bucket>/`.
- Migrate planning packs from `docs/project_management/next|future` to `docs/project_management/packs/<bucket>/`.
- Introduce slice directories (`slices/<SLICE_ID>/...`) for spec/closeout + slice kickoff prompts.
- Preserve CI checkpoint sizing defaults **4–8** in templates and standards.

### Out of scope
- Changing the triad model or introducing live steering.
- Redesigning ADR content rules (only relocating/migrating).
- Changing feature semantics (this is structural/migrational).

### Success criteria
- Existing workflows keep working throughout:
  - `make planning-lint FEATURE_DIR=...`
  - `scripts/triad/task_start.sh ...` and `scripts/triad/task_finish.sh ...`
  - impact map Touch Set enforcement continues to run on orchestration worktree.
- New structure exists and is canonical:
  - system under `docs/project_management/system/`
  - artifacts under `docs/project_management/adrs/` and `docs/project_management/packs/`
  - slices under `.../slices/<SLICE_ID>/`
- Planning agent dispatcher can generate at least `spec_manifest.md` and `impact_map.md` independently.

### Constraints
- Tech:
  - Cross-platform scripts exist (`.sh` + `.ps1`); changes must not assume only one shell.
  - Use repo-local, deterministic paths; avoid machine-specific paths in generated artifacts.
- Time/sequence:
  - Do not move directories until compatibility wrappers + path resolution exist.
- Compliance/security:
  - Do not regress enforcement: `validate_impact_map.py` and orchestration worktree discovery must remain callable.

### Unknowns / risks
- Active/in-flight worktrees store feature-dir paths in `.taskmeta.json` and registry files; moving packs can strand worktrees.
- Mechanical rewrites across packs and logs can create large diffs and conflict with concurrent work.
- Some scripts hardcode `docs/project_management/next/` in invariants and help text and must be made root-aware.

---

## Vertical slices

## Slice S1: Compatibility surface + path resolution foundation

**User value**: Directory refactors can start without breaking enforcement, triad orchestration, or developer muscle memory.

**Scope**
- In:
  - Define a compatibility contract and implement **stable entrypoints**:
    - `scripts/planning/*` remains callable (including `scripts/planning/validate_impact_map.py`)
    - `scripts/triad/*` remains callable (including worktree discovery behavior)
  - Introduce a shared “PM path resolver” used by:
    - planning lint + planning validators
    - triad scripts that interpret `.taskmeta.json feature_dir`
  - Accept both planning pack roots during migration:
    - legacy: `docs/project_management/next/<feature>`
    - new: `docs/project_management/packs/<bucket>/<feature>`
- Out:
  - Moving any directories (scripts, packs, ADRs) in this slice.

**Acceptance criteria**
- [ ] A single resolver exists with explicit outputs for: `PM_ROOT`, `PM_SYSTEM_ROOT`, `PM_ADRS_ROOT`, `PM_PACKS_ROOT`, and default packs bucket (`active`).
- [ ] Existing `make planning-lint` / `make planning-validate` accepts `FEATURE_DIR` pointing at either `.../next/<feature>` or `.../packs/<bucket>/<feature>`.
- [ ] Triad scripts that ban “planning doc edits inside worktrees” use the resolver-derived planning roots (not a hardcoded `docs/project_management/next/` string).

**Dependencies**
- Blocks on: none
- Unblocks: S2–S9

**Verification**
- Automated:
  - Run `make planning-lint FEATURE_DIR=docs/project_management/next/<feature>` on an existing pack.
  - Run `make planning-lint FEATURE_DIR=docs/project_management/packs/active/<feature>` on a staged test pack (once S5 introduces packs).
- Manual/demo:
  - Run `scripts/triad/task_finish.sh --verify-only --task-id <id>` inside an existing worktree and confirm guardrails reference both roots.

### Task S1.T1: Define and implement PM root/path resolver

**Outcome**: A single authoritative resolver used by scripts instead of hardcoded `docs/project_management/next/...` paths.

**Inputs/outputs**
- Inputs (prereqs):
  - Environment variables (exact names):
    - `PM_ROOT` (default `docs/project_management`)
    - `PM_SYSTEM_ROOT` (default `${PM_ROOT}/system`)
    - `PM_ADRS_ROOT` (default `${PM_ROOT}/adrs`)
    - `PM_PACKS_ROOT` (default `${PM_ROOT}/packs`)
    - `PM_DEFAULT_PACK_BUCKET` (default `active`)
- Outputs (artifacts):
  - Primary resolver implementation (Python):
    - `scripts/planning/pm_paths.py`
  - CLI contract (exact):
    - `python3 scripts/planning/pm_paths.py print-roots` prints JSON to stdout with keys:
      - `pm_root`, `pm_system_root`, `pm_adrs_root`, `pm_packs_root`, `pm_default_pack_bucket`
    - `python3 scripts/planning/pm_paths.py resolve-feature-dir --feature-dir <path>` prints a normalized repo-relative feature dir to stdout
  - A documented contract for “resolved feature dir” behavior:
    - If `--feature-dir` points at `.../next/<feature>`: accept as-is
    - If `--feature-dir` points at `.../packs/<bucket>/<feature>`: accept as-is

**Acceptance criteria**
- [ ] Resolver returns repo-relative paths (not absolute) suitable for embedding in `.taskmeta.json`.
- [ ] Resolver fails with an actionable error when `PM_ROOT` is set but does not exist.

**Implementation notes**
- Where: `scripts/planning/*`, `scripts/triad/*`, and future `docs/project_management/system/scripts/...`
- Approach:
  - Use the Python resolver as the single source of truth; shell scripts call it to obtain roots/prefixes.
  - Keep error messages single-line and actionable (print expected env vars + example values).

**Test notes**
- Manual:
  - Set `PM_ROOT` to a temp dir and verify scripts fail fast with the expected message.

**Risk / rollback**
- Risk: resolver changes script behavior if defaults are wrong.
- Rollback: revert the resolver + first consumer commit; do not carry a permanent dual-logic fallback.

**Checklist**
- [ ] Implement resolver with explicit defaults and error messages
- [ ] Add a minimal “print resolved roots” mode for debugging
- [ ] Update at least one consumer script to use the resolver (smoke validate)
- [ ] Document resolver contract in `docs/project_management/system/README.md` (created in S2)

### Task S1.T2: Update triad planning-doc-edit guardrails to be root-aware

**Outcome**: Triad scripts enforce “do not edit planning docs inside worktrees” across both `next/` and `packs/` layouts.

**Inputs/outputs**
- Inputs:
  - Resolver from S1.T1
- Outputs:
  - Updated triad guard logic that bans edits under:
    - `${PM_ROOT}/next/` (legacy)
    - `${PM_PACKS_ROOT}/` (new)

**Acceptance criteria**
- [ ] `scripts/triad/task_finish.sh` refuses worktree changes under either planning root with an updated, precise error message.

**Checklist**
- [ ] Replace hardcoded `^docs/project_management/next/` with resolver-derived prefixes
- [ ] Update error messages to mention both roots
- [ ] Run a local dry-run/verify-only path check in an existing worktree

---

## Slice S2: System root scaffolding + prompt/standard/template separation (copy-first)

**User value**: A portable, canonical PM “system” exists without breaking current paths, enabling gradual migration.

**Scope**
- In:
  - Create `docs/project_management/system/` scaffolding.
  - Copy (not move) existing standards/templates/prompts into the new structure.
  - Convert standards that embed large prompts into link-only stubs pointing at prompt files.
  - Preserve CI checkpoint sizing defaults **4–8** in the moved/copied artifacts.
- Out:
  - Removing legacy files.
  - Moving scripts (handled in S4).

**Acceptance criteria**
- [ ] `docs/project_management/system/` exists with `README.md` and the planned subdirs (standards/templates/prompts/scripts).
- [ ] Prompt-like standards are replaced by stubs that link to `system/prompts/...`.
- [ ] CI checkpoint template and standard still specify defaults `min=4`, `max=8`.

**Dependencies**
- Blocks on: S1
- Unblocks: S3, S4

**Verification**
- Manual/demo:
  - Open the new system README and confirm it explains where prompts live and how they are referenced.
  - Confirm the CI checkpoint defaults are present in both the template and standard.

### Task S2.T1: Add `docs/project_management/system/` directory scaffold + README

**Outcome**: Canonical system root exists and documents the portability contract and resolved roots.

**Inputs/outputs**
- Inputs:
  - Resolver contract from S1.T1
- Outputs:
  - New directories:
    - `docs/project_management/system/standards/`
    - `docs/project_management/system/templates/`
    - `docs/project_management/system/prompts/`
    - `docs/project_management/system/scripts/`
  - `docs/project_management/system/README.md` with:
    - description of system vs artifacts
    - root env var contract:
      - `PM_ROOT`
      - `PM_SYSTEM_ROOT`
      - `PM_ADRS_ROOT`
      - `PM_PACKS_ROOT`
      - `PM_DEFAULT_PACK_BUCKET`

**Checklist**
- [ ] Create directories
- [ ] Write `system/README.md` with the compatibility contract summary
- [ ] Verify links are repo-relative and do not hardcode `next/`

### Task S2.T2: Copy standards/templates/prompts into system and stub-link old prompt-like files

**Outcome**: Standards become normative, prompts become runnable, and templates become scaffold inputs.

**Inputs/outputs**
- Outputs:
  - Copies of:
    - `docs/project_management/standards/**` → `docs/project_management/system/standards/**` (structure may change)
    - `docs/project_management/standards/templates/**` → `docs/project_management/system/templates/**`
    - prompt files → `docs/project_management/system/prompts/**`
  - Link-only stubs left behind where appropriate (exact files listed in the spec).

**Acceptance criteria**
- [ ] No “giant prompts” remain embedded in standards files that are intended to be normative.
- [ ] All moved/copied prompt references are valid paths.

**Checklist**
- [ ] Identify prompt-like standards in current tree and move/copy into `system/prompts`
- [ ] Replace original prompt-like standards with link stubs (or keep as deprecated copies with a deprecation banner)
- [ ] Confirm CI checkpoint template and standard still contain 4–8 defaults

---

## Slice S3: Planning agent dispatcher exposed via Make + underlying script

**User value**: You can run focused planning agents (one output each) without running “the whole planning pack”.

**Scope**
- In:
  - Implement `docs/project_management/system/scripts/planning/run_planning_agent.sh` (and optional PowerShell equivalent).
  - Add a Make entrypoint named `pm-run-planning-agent` that calls the script.
  - Add initial planning prompts:
    - `spec_manifest_agent.md`
    - `impact_map_agent.md`
- Out:
  - Full pack generation.
  - Migrating existing packs (handled later).

**Acceptance criteria**
- [ ] `make pm-run-planning-agent FEATURE_DIR=... AGENT=spec_manifest` produces exactly one updated artifact file.
- [ ] `make pm-run-planning-agent FEATURE_DIR=... AGENT=impact_map` produces exactly one updated artifact file.
- [ ] Script passes Codex invocation using the established safe pattern (`codex exec --dangerously-bypass-approvals-and-sandbox --cd ...`).

**Dependencies**
- Blocks on: S1, S2
- Unblocks: S4–S9 (provides new prompt library infrastructure)

**Verification**
- Manual/demo:
  - Run dispatcher against a small sample pack and confirm only the intended output file changes.

### Task S3.T1: Define dispatcher contract (agents, outputs, CLI/env)

**Outcome**: Concrete contract exists so downstream tasks are not ambiguous.

**Inputs/outputs**
- Outputs:
  - Exact supported `AGENT` identifiers (initial set):
    - `spec_manifest`
    - `impact_map`
  - For each agent: exact output file path relative to `FEATURE_DIR`:
    - `spec_manifest` → `<FEATURE_DIR>/spec_manifest.md`
    - `impact_map` → `<FEATURE_DIR>/impact_map.md`
  - Make variables contract:
    - `FEATURE_DIR` (required)
    - `AGENT` (required)
    - `CODEX_PROFILE` (optional)
    - `CODEX_MODEL` (optional)
    - `CODEX_JSONL` (optional, `1` to enable)

**Acceptance criteria**
- [ ] A single doc section in `system/scripts/planning/run_planning_agent.sh` (or adjacent README) lists supported agents and their exact output files.

**Checklist**
- [ ] Write contract into script help output (`--help`)
- [ ] Ensure contract uses resolver-derived roots for any system inputs

### Task S3.T2: Implement `pm-run-planning-agent` Make target + `run_planning_agent.sh`

**Outcome**: Dispatcher is runnable via Make and directly via script.

**Checklist**
- [ ] Add Make target `pm-run-planning-agent` that forwards vars to the script
- [ ] Implement script argument parsing:
  - `--feature-dir`
  - `--agent`
  - `--codex-profile`, `--codex-model`, `--codex-jsonl`
- [ ] Select prompt file by agent id under `docs/project_management/system/prompts/planning/`
- [ ] Ensure single-output rule is enforced by prompt instructions and post-run validation:
  - script checks `git diff --name-only` within `FEATURE_DIR` only contains the intended file
- [ ] Add clear failure messages when the agent edits extra files

---

## Slice S4: Move scripts into `system/scripts` with compatibility wrappers preserved

**User value**: The PM system becomes portable; old script entrypoints keep working.

**Scope**
- In:
  - Move `scripts/planning/*` → `docs/project_management/system/scripts/planning/*`
  - Move `scripts/triad/*` → `docs/project_management/system/scripts/triad/*`
  - Leave wrappers in `scripts/planning/*` and `scripts/triad/*` that `exec` the new locations.
- Out:
  - Removing wrappers (handled in S9).

**Acceptance criteria**
- [ ] `scripts/planning/lint.sh` still works (as wrapper) and runs the moved implementation.
- [ ] `scripts/triad/task_finish.sh` still works (as wrapper) and runs the moved implementation.
- [ ] Impact map validator remains callable from the stable path `scripts/planning/validate_impact_map.py` (wrapper).

**Dependencies**
- Blocks on: S1, S2, S3 (for the target system script locations + resolver)
- Unblocks: S5–S9 (all future work should operate out of `system/scripts`)

**Verification**
- Manual/demo:
  - Run a planning lint and a triad task start/finish on an existing feature and confirm wrapper behavior.

### Task S4.T1: Move scripts + add Bash wrappers at legacy entrypoints

**Outcome**: Old paths are preserved as stable entrypoints.

**Inputs/outputs**
- Outputs:
  - Moved scripts in `docs/project_management/system/scripts/...`
  - Wrappers in `scripts/...` that:
    - compute repo root deterministically
    - `exec` the new script with `"$@"`

**Checklist**
- [ ] Move files
- [ ] Add wrappers for every moved script (planning + triad)
- [ ] Update internal relative-path references inside moved scripts to be repo-root anchored
- [ ] Run `shellcheck` on wrappers (if installed) or at least basic execution tests

---

## Slice S5: Introduce `packs/` buckets + sequencing.json, keep dual-root support

**User value**: New planning pack root exists and can be used immediately for new work without breaking existing packs in `next/`.

**Scope**
- In:
  - Create `docs/project_management/packs/{draft,queued,active,implemented,superseded}/`.
  - Copy `docs/project_management/next/sequencing.json` → `docs/project_management/packs/sequencing.json`.
  - Update planning lint + other scripts to reference sequencing via resolver roots and accept both locations during transition.
- Out:
  - Moving existing packs (handled in S7).

**Acceptance criteria**
- [ ] `make planning-lint FEATURE_DIR=docs/project_management/packs/active/<feature>` works for a pack created under packs.
- [ ] Sequencing checks accept `docs/project_management/packs/sequencing.json` as canonical but still allow legacy reads until migration completes.

**Dependencies**
- Blocks on: S1, S4
- Unblocks: S7

**Verification**
- Automated:
  - Create a tiny sample pack under `packs/active` and run planning lint.

### Task S5.T1: Add packs buckets + sequencing.json copy and script updates

**Outcome**: New packs root exists and scripts are root-aware.

**Checklist**
- [ ] Create `packs/` bucket dirs
- [ ] Copy sequencing.json into `packs/sequencing.json`
- [ ] Update planning lint scripts to:
  - read sequencing.json from `packs/` when present
  - otherwise fall back to legacy `next/sequencing.json` with a warning
- [ ] Update Makefile help/error strings to mention packs paths (keep legacy examples until S9)

---

## Slice S6: Migrate ADRs to `adrs/<bucket>/` and update references

**User value**: ADRs have one canonical home and are no longer scattered across planning packs.

**Scope**
- In:
  - Move legacy ADRs from `docs/project_management/next/**` into `docs/project_management/adrs/{draft,queued,implemented,superseded}/`.
  - Update references inside packs and docs.
  - Update ADR exec summary checker to accept new locations and reference patterns.
- Out:
  - Changing ADR semantics.

**Acceptance criteria**
- [ ] No ADR files remain under packs directories after the migration.
- [ ] `make adr-check ADR=docs/project_management/adrs/<bucket>/ADR-....md` works.
- [ ] Planning lint ADR reference scan accepts the new ADR root.

**Dependencies**
- Blocks on: S1, S5
- Unblocks: S7 (pack moves should not drag ADRs around)

**Verification**
- Automated:
  - Run ADR checker on a moved ADR
  - Run planning lint on a pack that references an ADR path and confirm it resolves

### Task S6.T1: Mechanical ADR move + reference rewrite

**Outcome**: ADR locations are canonical and references updated.

**Checklist**
- [ ] Move legacy ADRs from `docs/project_management/next/**` → `docs/project_management/adrs/implemented/` (or correct bucket)
- [ ] Move `docs/project_management/next/<feature>/ADR-*.md` → correct ADR bucket
- [ ] Rewrite references in packs and docs:
  - legacy ADR paths under `docs/project_management/next/**` → canonical paths under `docs/project_management/adrs/<bucket>/`
- [ ] Update ADR README to document the new canonical root

---

## Slice S7: Migrate packs from `next|future` to `packs/<bucket>` (path rewrite + in-flight worktree safety)

**User value**: Planning packs are in the canonical location without breaking triad execution for in-flight worktrees.

**Scope**
- In:
  - Move active packs: `docs/project_management/next/<feature>` → `docs/project_management/packs/active/<feature>`
  - Move future **planning packs** currently under `docs/project_management/future/` (as identified and mapped in S7.T0) into `docs/project_management/packs/draft|queued/`
  - Mechanical rewrite of pack-internal references (`docs/project_management/next/` → `docs/project_management/packs/active/`)
  - Safe handling for in-flight worktrees (exact rule):
    - If any existing worktree’s `.taskmeta.json feature_dir` equals `docs/project_management/next/<feature>`, run the S7.T1 migration tool for that `<feature>` before moving the directory.
    - Otherwise, proceed with the directory move.
- Out:
  - Removing legacy support (handled in S9).

**Acceptance criteria**
- [ ] `make planning-lint FEATURE_DIR=docs/project_management/packs/active/<feature>` succeeds for migrated packs.
- [ ] Triad scripts can still locate the orchestration feature dir even if `.taskmeta.json feature_dir` points at the old location (until worktree migration completes), or a migration tool updates it deterministically.

**Dependencies**
- Blocks on: S1, S4, S5, S6
- Unblocks: S8, S9

**Verification**
- Manual/demo:
  - Pick one feature with no active worktrees and migrate it end-to-end.
  - Run triad task start/finish flows referencing the migrated feature dir.

### Task S7.T0 (Spike): Inventory “future/” and define deterministic migration mapping

**Outcome**: A concrete, reviewable mapping exists for what in `docs/project_management/future/` is a planning pack and where it should land under `packs/`.

**Inputs/outputs**
- Outputs (artifacts):
  - `docs/project_management/packs/future_migration_plan.json` containing an array of entries with exact keys:
    - `src_dir` (repo-relative)
    - `dst_bucket` (`draft` or `queued`)
    - `dst_dir` (repo-relative)
  - A deterministic classification rule documented at the top of the file:
    - A “future planning pack” is a directory that contains `tasks.json` at its root and a `kickoff_prompts/` directory at its root.

**Acceptance criteria**
- [ ] Every `src_dir` in the plan exists at the time the plan is generated.
- [ ] Every `dst_dir` is under `docs/project_management/packs/<dst_bucket>/`.

**Checklist**
- [ ] Walk `docs/project_management/future/` and identify pack-like directories using the exact rule above
- [ ] Decide `dst_bucket` for each entry and record it in `future_migration_plan.json`
- [ ] Get review on the plan before moving directories

### Task S7.T1: Add “in-flight worktree migration” tool for feature_dir path changes

**Outcome**: Moving packs does not strand active worktrees.

**Inputs/outputs**
- Outputs:
  - A script that updates:
    - `<worktree>/.taskmeta.json` field `feature_dir` from old to new
    - triad registry file `<git-common-dir>/triad/features/<feature>/worktrees.json` `feature_dir` field (and any per-entry derived paths if present)
  - Exact invocation contract (must be defined):
    - `--from docs/project_management/next/<feature>`
    - `--to docs/project_management/packs/active/<feature>`
    - `--dry-run` support

**Checklist**
- [ ] Implement migration script with dry-run + clear output
- [ ] Add a safety check: refuse if the repo has uncommitted changes (unless `--allow-dirty` with explicit justification)
- [ ] Validate on one local worktree in a controlled test

### Task S7.T2: Move directories + mechanical reference rewrite

**Outcome**: Packs live under `packs/` and internal references are consistent.

**Checklist**
- [ ] Move `next/<feature>` → `packs/active/<feature>` (one feature at a time, then batch)
- [ ] Rewrite pack-internal references:
  - `docs/project_management/next/` → `docs/project_management/packs/active/`
- [ ] Move future planning packs listed in `docs/project_management/packs/future_migration_plan.json` to their mapped `dst_dir`
- [ ] Update `Makefile` examples and scripts that default to `next/` to default to `packs/active/` (keep legacy compatibility until S9)

---

## Slice S8: Slice directories migration + validator/lint updates (dual support first)

**User value**: Planning packs scale cleanly; slice docs and slice kickoff prompts live with the slice and validators support the new layout.

**Scope**
- In:
  - Introduce `slices/<SLICE_ID>/` in packs and move:
    - `<SLICE_ID>-spec.md`
    - `<SLICE_ID>-closeout_report.md`
    - slice kickoff prompts `kickoff_prompts/<SLICE_ID>-*.md`
  - Update validators and lint to support:
    - feature-level kickoff prompts: `<feature>/kickoff_prompts/*.md`
    - slice-level kickoff prompts: `<feature>/slices/<slice>/kickoff_prompts/*.md`
  - Update sentinel scanning to be recursive across all kickoff prompts.
- Out:
  - Removing old layout support immediately (handled in S9 after migration completes).

**Acceptance criteria**
- [ ] `scripts/planning/validate_tasks_json.py` accepts slice kickoff prompts in the new location.
- [ ] Planning lint sentinel check scans all kickoff prompts recursively (feature + slices).
- [ ] A migrated feature passes `make planning-lint FEATURE_DIR=...`.

**Dependencies**
- Blocks on: S7
- Unblocks: S9

**Verification**
- Automated:
  - Run `make planning-lint` on a migrated feature with slice directories.

### Task S8.T1: Update validators and lint for slice directory layout (dual-root)

**Outcome**: Validation supports both old and new kickoff prompt locations during the transition.

**Checklist**
- [ ] Update `validate_tasks_json.py` kickoff prompt path checks to allow both locations
- [ ] Update planning lint kickoff sentinel scan to:
  - find all `kickoff_prompts` dirs under feature dir recursively
  - enforce sentinel in every `*.md` prompt file (excluding `README.md`)
- [ ] Negative checks (exact):
  - Create a temp pack directory with:
    - `tasks.json` that references a slice kickoff prompt path under `slices/<SLICE_ID>/kickoff_prompts/`
    - a kickoff prompt missing the sentinel line `Do not edit planning docs inside the worktree.`
  - Run `scripts/planning/lint.sh --feature-dir <temp_pack_dir>` and confirm:
    - exit code is non-zero
    - stderr contains `Missing sentinel in kickoff prompt:`

### Task S8.T2: Mechanical slice directory migration for one feature + batch plan

**Outcome**: The feature’s slices are organized as directories and tasks.json references are updated.

**Checklist**
- [ ] Create `slices/<SLICE_ID>/kickoff_prompts/` for each slice
- [ ] Move slice spec/closeout into slice dirs
- [ ] Move slice kickoff prompts into slice dirs
- [ ] Update `tasks.json`:
  - update `kickoff_prompt` paths for slice tasks
  - update any spec/closeout references
- [ ] Run `make planning-lint FEATURE_DIR=...` and fix any violations

---

## Slice S9: Remove legacy compatibility + finalize docs and defaults

**User value**: The system is clean: one canonical structure, no “two ways” forever.

**Scope**
- In:
  - Remove wrappers in legacy `scripts/` once all callers are updated.
  - Remove/retire `docs/project_management/next/` and any legacy notes.
  - Update docs and Makefile help text to reference only `packs/` paths.
- Out:
  - Further feature evolution (future initiatives).

**Acceptance criteria**
- [ ] No operational scripts rely on `docs/project_management/next/` existing.
- [ ] Documentation references only `docs/project_management/system/`, `adrs/`, and `packs/`.

**Dependencies**
- Blocks on: S8

**Verification**
- Automated:
  - `rg -n "docs/project_management/next" scripts docs Makefile` returns only archived/historical content (or zero).

### Task S9.T1: Remove wrappers + legacy paths + update docs

**Outcome**: Canonical structure only; old paths removed.

**Checklist**
- [ ] Remove legacy wrappers and any now-dead scripts
- [ ] Remove or archive `docs/project_management/next/` after packs are migrated
- [ ] Update Makefile to require `FEATURE_DIR` under `packs/<bucket>/`
- [ ] Run planning lint and at least one triad workflow end-to-end against a migrated feature

---

## Dependency graph (text)

- S1 blocks S2
- S1 blocks S3
- S1 blocks S4
- S4 blocks S5
- S5 blocks S6
- S6 blocks S7
- S7 blocks S8
- S8 blocks S9

## Risks / unknowns (and de-risking plan)

- In-flight worktrees break when packs move
  - De-risk: S7.T1 migration tool + “no active worktrees” policy per feature move
- Mechanical rewrite produces huge diffs and merge conflicts
  - De-risk: do moves as single-purpose commits (ADR move, then pack move, then slice move)
- Cross-platform script drift
  - De-risk: keep `.ps1` parity tasks adjacent to `.sh` tasks; verify on at least one Windows/PowerShell runner when possible

## Milestones

- M1: “Safe to refactor” foundation landed (S1 complete)
- M2: System root exists and prompts are separated (S2 + S3 complete)
- M3: Scripts are system-owned with legacy wrappers (S4 complete)
- M4: Packs and ADRs canonicalized (S6 + S7 complete)
- M5: Slices-as-directories fully adopted and legacy removed (S8 + S9 complete)

## Workstreams

### WS-COMPAT: Compatibility + path resolver
**Scope**: Implement resolver, update guardrails and lint to be root-aware.
**Touch surface**:
- `scripts/planning/*`, `scripts/triad/*`, `Makefile`

### WS-SYSTEM: System root + prompt library
**Scope**: Add `docs/project_management/system/` scaffold, prompts/standards/templates separation.
**Touch surface**:
- `docs/project_management/system/**`, `docs/project_management/standards/**`

### WS-DISPATCH: Planning agent dispatcher
**Scope**: Add `pm-run-planning-agent` Make target and `run_planning_agent.*` scripts and initial prompts.
**Touch surface**:
- `Makefile`, `docs/project_management/system/scripts/planning/**`, `docs/project_management/system/prompts/planning/**`

### WS-MIGRATE: ADR + packs migrations
**Scope**: Move ADRs and packs and rewrite references; build migration tooling for in-flight worktrees.
**Touch surface**:
- `docs/project_management/adrs/**`, `docs/project_management/packs/**`, migration scripts under planning/system scripts

### WS-VALIDATORS: Slice directory support in validators/lint
**Scope**: Update validators and lint to support new slice directory layout and recursive kickoff prompt sentinel scanning.
**Touch surface**:
- `scripts/planning/*.py`, `docs/project_management/system/scripts/planning/**`

### WS-INT: Integration
**Scope**: Resolve cross-workstream coupling, run end-to-end validation (planning lint + triad start/finish) and ensure no legacy path regressions.
**Depends on**:
- WS-COMPAT
- WS-SYSTEM
- WS-DISPATCH
- WS-MIGRATE
- WS-VALIDATORS
**Touch surface**:
- `Makefile`, `scripts/**`, `docs/project_management/**`
