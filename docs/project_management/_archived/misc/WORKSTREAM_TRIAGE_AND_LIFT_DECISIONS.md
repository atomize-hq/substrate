# Workstream Triage + Lift Model — Decision Log

Last updated (UTC): 2026-02-23

This file lives at the **repo root on purpose** so we can track evolving workflow decisions without losing context or scattering notes across the project management system.

## Goals

- Maintain a **time-free** sizing and split heuristic that maps to our execution primitives (ADRs, slices, checkpoints), not wall-clock.
- Make workstream grouping stable: workstreams are **umbrella initiatives** spanning multiple ADRs/packs/work items (not tied 1:1 to an ADR codename).
- Enable incremental adoption with minimal breakage:
  - Advisory-first where possible
  - Strict enforcement gated by Planning Pack strict mode (`meta.slice_spec_version >= 2`)

## Terminology

- **ADR**: decision record; should represent **one behavior delta**.
- **Slice**: execution unit; each slice typically maps to a triad (code/test/integ).
- **Workstream**: umbrella initiative grouping multiple ADRs, packs, and work items.
- **Work Item (WI)**: execution/backlog unit that is not an ADR (implementation/cleanup/follow-up).
- **Lift Vector / Lift Score**: deterministic, time-free sizing signal computed from structured inputs.

## Decisions

### D1 — Workstream lifecycle is two-pass

Date: 2026-02-22

- We adopt **Workstream Triage + Workstream Refinement** as two passes:
  - **Pass A (Workstream Triage)** happens when an ADR is accepted (or queued). It groups the ADR into an umbrella workstream and records coarse size/risk signals.
  - **Pass B (Workstream Refinement)** happens after the Planning Pack is created and planning-lint is green. It replaces coarse estimates with pack-derived counts (actual slices/checkpoints/touch-set).

Rationale:
- ADRs usually arrive one at a time.
- Exact slice counts aren’t fully known until pack planning is done, but ADRs still need grouping and rough sizing.

### D2 — Use a deterministic, time-free Lift model (v1)

Date: 2026-02-22

- We drop “hours/days” from sizing inputs and instead use:
  - **Lift Vector** (counts + booleans)
  - **Lift Score** (deterministic formula)
  - **Split Triggers** (hard thresholds that force decomposition)
- Mapping is expressed in execution units:
  - Lift → estimated slices
  - Slices → checkpoint group count (4–8 slices per checkpoint as a default planning guideline)

### D3 — Discovery-time Lift Vector storage: JSON block + frontmatter summary

Date: 2026-02-22

- Discovery-time Lift Vector lives as an **embedded JSON block** (comment-fenced) inside intake/ADR markdown so it is deterministic to parse and works in either file type.
- V1 tooling emits computed outputs via `pm_lift.py ... --emit-json` (see `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` / CONTRACT-3).
- **V2 candidate**: an idempotent frontmatter writer that updates a small summary for quick scanning (mapping from CONTRACT-3 fields):
  - `lift_model_version` (maps from `model_version`)
  - `lift_score` (maps from `lift_score`)
  - `lift_estimated_slices` (maps from `estimated_slices`)
  - `lift_confidence` (`high|low`) (maps from `confidence`)
  - (optional) `lift_split_triggers` (array of strings) (maps from `triggers`)

Example (frontmatter summary — **V2 candidate**, not implemented by v1 tooling):
```yaml
lift_model_version: 1
lift_score: 18
lift_estimated_slices: 2
lift_confidence: low
lift_split_triggers: []
```

Example (embedded JSON block):
````md
<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 0,
    "edit_files": 0,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": null,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": ""
}
```
<!-- PM_LIFT_VECTOR:END -->
````

Notes:
- `null` is allowed for counts we can’t estimate deterministically at discovery-time; the script should reflect that in confidence/scoring behavior.
- The script (not the LLM) should compute `lift_score`, `estimated_slices`, `confidence`, `missing_inputs`, and `triggers` when possible (CONTRACT-3).

### D4 — Strict enforcement is gated by Planning Pack strict mode

Date: 2026-02-22

- Strict enforcement gates on `tasks.json meta.slice_spec_version >= 2`.
- Legacy packs remain compatible.
- Initial rollout should be advisory-first, then enforce only the most reliable invariants.

### D5 — Workstream IDs are initiative-themed (not ADR-codename themed)

Date: 2026-02-22

- Canonical Workstream registry IDs use: `WS-YYYYMM-<initiative_slug>`
- Workstreams are umbrellas spanning multiple ADRs/packs/WIs.
- Any “WS-<adr_codename>” values that appear in intake batching artifacts should be treated as **intake-local grouping tags**, not canonical workstream IDs, unless explicitly migrated.

### D6 — Lift Vector v1 fields (canonical)

Date: 2026-02-22

Lift is tracked as a structured vector of measurable signals. The intent is:

- **Discovery-time:** humans/agents can fill rough counts + booleans (with `null` allowed for unknowns).
- **Planning-time:** the computer can derive many counts deterministically from Planning Pack artifacts (especially `impact_map.md`).
- **Execution-time (optional):** lift can be computed from a git diff for calibration.

Canonical Lift Vector v1 (embedded JSON block shape):

- `model_version` (int): scoring model version; default `1` if missing
- `touch`
  - `create_files` (int|null): count from Impact Map `Create` (or `null` if unknown)
  - `edit_files` (int|null): count from `Edit` (or `null` if unknown)
  - `delete_files` (int|null): count from `Delete` (or `null` if unknown)
  - `deprecate_files` (int|null): count from `Deprecate` (or `null` if unknown)
  - `crates_touched` (int|null): rough count of crates/major modules touched
  - `boundary_crossings` (int|null): number of subsystems affected (requires a defined taxonomy)
- `contract`
  - `cli_flags` (int|null): new/changed commands/flags
  - `config_keys` (int|null): new/changed config keys
  - `exit_codes` (int|null): new/changed exit codes
  - `file_formats` (int|null): new/changed on-disk schemas/formats
  - `behavior_deltas` (int|null): should be `1` per ADR candidate; `>1` is an intentional “blow up” signal; discovery-time may be unknown (`null`/omitted), but strict-mode checks require `== 1` (see `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md`)
- `qa`
  - `new_test_files` (int|null)
  - `new_test_cases` (int|null) (or “new assertions” if we later rename)
- `docs`
  - `new_docs_files` (int|null)
- `ops`
  - `new_smoke_steps` (int|null)
  - `ci_changes` (int|null)
- `risk`
  - `cross_platform` (bool)
  - `security_sensitive` (bool)
  - `concurrency_or_ordering` (bool)
  - `migration_or_backfill` (bool)
  - `unknowns_high` (int|null): count of blocking unknowns
- `notes` (string)

Directory-prefix touch entries (Impact Map):

- When deriving `touch.*_files` from `impact_map.md`, touch sets may contain directory/prefix entries that imply “multiple files” but do not specify how many.
- Policy: count each directory/prefix entry as **1** toward the relevant `touch.*_files` count, and degrade confidence (see D7).
- We do **not** automatically add additional points per “subdirectory depth”; only the explicit entries matter.

Directory-prefix expansion (for deterministic lift computation):

- When computing Lift from `impact_map.md`, directory/prefix entries MAY be deterministically expanded to the current repo file list (e.g., via `git ls-files <prefix>`).
- Expansion is used for **lift estimation**, not for changing the authored impact map.
- Expanded files are **discounted and capped per prefix entry** to avoid a broad prefix dominating lift:
  - `EXPAND_DISCOUNT = 0.20` (each expanded file counts as 20% of a file)
  - `EXPAND_CAP = 10` expanded files per prefix entry
  - effective contribution per prefix entry: `min(expanded_files, EXPAND_CAP) * EXPAND_DISCOUNT` (max `2.0`)
- Explicit file tokens remain full weight (1.0 each).
- Presence of any directory/prefix entries should still degrade `confidence` (see D7), because expansion reflects current HEAD, not guaranteed future touch.

### D7 — Lift Score v1 formula + confidence rules (deterministic)

Date: 2026-02-22

We adopt a deterministic Lift Score v1 formula that maps to slices/checkpoints and is tunable over time.

Scoring (example v1; canonical until revised):

Base points:
- `+ 3 * touch.create_files`
- `+ 2 * touch.edit_files`
- `+ 1 * touch.delete_files`
- `+ 1 * touch.deprecate_files`
- `+ 4 * touch.crates_touched`
- `+ 3 * touch.boundary_crossings`

Contract surface:
- `+ 3 * contract.cli_flags`
- `+ 3 * contract.config_keys`
- `+ 4 * contract.exit_codes`
- `+ 5 * contract.file_formats`
- `+ 10 * max(0, contract.behavior_deltas - 1)` (intentional blow-up when `>1`; clamped so missing/`null` does not create a negative term)

QA / Docs / Ops:
- `+ 2 * qa.new_test_files`
- `+ 1 * qa.new_test_cases`
- `+ 2 * docs.new_docs_files`
- `+ 3 * ops.new_smoke_steps`
- `+ 3 * ops.ci_changes`

Risk multiplier (multiplicative):
- start multiplier `m = 1.0`
- `m *= 1.15` if `risk.cross_platform`
- `m *= 1.20` if `risk.security_sensitive`
- `m *= 1.15` if `risk.concurrency_or_ordering`
- `m *= 1.25` if `risk.migration_or_backfill`

Unknowns add-on:
- after applying `m`, add `+ 2 * risk.unknowns_high`

Rounding:
- round up to an integer (`ceil`).

Handling `null` inputs (discovery-time gotcha):

- If a numeric field is `null` (or omitted), treat it as **0 for scoring**, but set `confidence = low` and emit `missing_inputs:<json_path>` trigger tokens (see CONTRACT-3 in `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`).
- This avoids inventing precision while still allowing a computed output.

Handling directory/prefix inputs (confidence):

- In v1, any directory/prefix entries in the Touch Set force `confidence = low` and emit `touch_set_contains_prefix_entries` (CONTRACT-3).
- **V2 candidate**: a “narrow prefix” exception, with a mechanical definition of narrowness.

### D8 — Split triggers v1 (ADR vs workstream)

Date: 2026-02-22

Split triggers are the real value: hard signals that force decomposition.

Single ADR candidate triggers:
- `contract.behavior_deltas > 1` → **split into multiple ADRs**
- `touch.crates_touched > 2` → likely split (or require an explicit seam plan)
- `touch.create_files + touch.edit_files + touch.delete_files > 12` → likely split
- `contract.cli_flags + contract.config_keys + contract.exit_codes + contract.file_formats > 4` → likely split
- `lift_score > 24` → likely split into 2–3 ADRs (or 2–4 slices)

Workstream (umbrella) triggers:
- Implemented in v1 model config (per-item `pm_lift`):
  - `lift_score > 60` → likely split workstream (or create Phase 1/Phase 2 workstreams)
- **V2 candidates** (require aggregation and/or taxonomy; not part of per-item `pm_lift` v1):
  - `estimated_total_slices > 8` → split workstream (requires workstream-level aggregation across items)
  - `touch.boundary_crossings > 4` → split by subsystem (requires a defined subsystem taxonomy + consistent fill/derivation)

Enforcement posture:
- Start advisory-first; move the most reliable triggers to enforcement once calibrated.
- Initial best candidates for enforcement are:
  - `contract.behavior_deltas == 1` for ADR candidates
  - ADR slice cap (see D9)

### D9 — Mapping Lift → slices → checkpoints (time-free)

Date: 2026-02-22

We map Lift into execution primitives rather than time.

Estimated slices:
- `estimated_slices = max(1, ceil(lift_score / 12))`

ADR slice cap (standard):
- target: **1–3 slices per ADR**
- if `estimated_slices > 3` → split ADR (or explicitly justify why not)

Checkpoint grouping guideline:
- 4–8 slices per checkpoint group
- `checkpoint_groups = ceil(total_slices / 8)`
- (optional heuristic) target ~6 slices/checkpoint when shaping boundaries

### D10 — Lift rubric artifacts + pm_lift tooling sketch

Date: 2026-02-22

We keep the canonical *decision log* at the repo root (this file), but we want the actual rubric + machine-readable model to live under the PM system so it can be used for enforcement and automation.

Rubric location (approved):

- Machine-readable, enforceable artifacts live under:
  - `docs/project_management/system/schemas/work_lift_vector.schema.json` (JSON schema for the embedded Lift Vector block)
  - `docs/project_management/system/schemas/work_lift_model.v1.json` (weights/multipliers/triggers; scripts should not hardcode)
- Human-readable rubric lives under:
  - `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md` (explains Lift Vector fields, scoring, triggers, mapping; points to the schema/config)

Initial `pm_lift.py` sketch (tooling):

- Script path: `docs/project_management/system/scripts/planning/pm_lift.py`
- Modes (intended):
  - `from-intake --intake <path>`: parse `PM_LIFT_VECTOR` JSON block; compute `lift_score`, `estimated_slices`, `triggers`, `confidence`, and `missing_inputs` (see CONTRACT-3: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`)
  - `from-impact-map --feature-dir <pack>`: derive Touch Set counts from `impact_map.md` (optionally expand directory prefixes deterministically via `git ls-files` and apply `EXPAND_DISCOUNT`/`EXPAND_CAP`)
  - `from-git-diff --git-range <base..head>`: compute post-implementation lift signal from the diff (for calibration)
- Output:
  - default: human summary (score, slices, triggers)
  - `--emit-json`: machine summary for downstream consumers
- Integration posture:
  - advisory-first; then add strict-mode enforcement keyed off `tasks.json meta.slice_spec_version >= 2`

## Resolved in v1 (pinned by published artifacts)

- Deterministic formula (weights/multipliers/triggers/mapping) is pinned as a versioned config file:
  - `docs/project_management/system/schemas/work_lift_model.v1.json` (CONTRACT-2)
- `null` / missing numeric inputs are handled deterministically and surfaced as machine outputs:
  - see CONTRACT-3: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`
- Strict-mode initial invariant set + opt-in mechanism are pinned (no default enforcement):
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md` (opt-in `PM_LIFT_STRICT=1`, wrapper `pm_lift_strict_check.py`)

## V2 candidates (explicit)

- `confidence = medium` (additional confidence state beyond `high|low`, with deterministic rules)
- “Narrow prefix” exception (prefix entries do not always force `confidence=low`, with a mechanical definition of “narrow”)
- Workstream aggregation triggers:
  - `estimated_total_slices > 8` (requires aggregation across items in a workstream)
  - `touch.boundary_crossings > 4` (requires a defined taxonomy + consistent fill/derivation)
- Idempotent frontmatter summary writer (writes/updates the D3 summary keys from CONTRACT-3 outputs)
