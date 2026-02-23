# Work Lift in the Planning Workflow (Advisory-First)

This standard explains **where Work Lift fits** in the planning workflow and **how to run it** in three contexts:

- Intake / ADR markdown (discovery-time sizing + early split signals)
- Planning Pack (`impact_map.md`) refinement (planning-time derived counts)
- Post-implementation calibration (git diff derived counts)

**Posture (non-negotiable):**
- Work Lift is **advisory-first**. It informs splitting decisions; it does not block by default.
- **Legacy compatibility** is required. No new mandatory lift requirements are introduced for legacy packs.
- Strict enforcement is intentionally deferred and must be an explicit opt-in (see decision posture in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`).

## Where lift fits

### Intake / ADR (discovery-time)
Use lift to:
- detect early split signals (triggers matter more than the numeric score),
- avoid “invented precision” by allowing unknown inputs (`null` / omitted) and letting confidence reflect uncertainty.

Run: `pm_lift.py from-intake` (see recipes below).

### Planning Pack refinement (planning-time)
Once a Planning Pack has an `impact_map.md` Touch Set, you can compute lift from the pack.

**Important:** pack-derived lift is meaningful only for **strict packs** where:
- `<pack_dir>/tasks.json` has `meta.slice_spec_version >= 2`, and
- `validate_impact_map.py --emit-json` emits non-empty allowlists (see `CONTRACT-4`).

Legacy packs (`meta.slice_spec_version` missing or `< 2`) intentionally emit **empty** allowlists under `--emit-json`, so `from-impact-map` will not reflect the authored Touch Set.

Run: `pm_lift.py from-impact-map` (see recipes below).

### Post-implementation calibration (after code changes)
Use lift on a git diff to:
- compare realized touch surface against the planning-time estimates,
- calibrate whether lift thresholds/triggers are too sensitive or too lax.

Run: `pm_lift.py from-git-diff` (see recipes below).

## Prerequisites / gotchas

- Run commands from inside a **git checkout** (lift uses `git rev-parse`, `git diff`, and may use `git ls-files` for prefix expansion).
- Intake/ADR markdown must contain **exactly one** Lift Vector block wrapped by the required markers:
  - See: `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md` (`CONTRACT-1`)
- Planning Pack lift is meaningful only for strict packs (`tasks.json.meta.slice_spec_version >= 2`), because `validate_impact_map.py --emit-json` is the authoritative source of Touch Set allowlists (`CONTRACT-4`).

## How to run (direct script recipes)

All commands support `--emit-json` for machine-readable output (see `CONTRACT-3`).

### Intake / ADR markdown (`from-intake`)

```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py \
  from-intake \
  --intake <path/to/intake_or_adr.md> \
  [--emit-json]
```

To add a valid Lift Vector block to an intake/ADR file, follow:
- `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`
- `docs/project_management/system/schemas/work_lift_vector.schema.json`

### Planning Pack (`from-impact-map`)

```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py \
  from-impact-map \
  --feature-dir <path/to/pack_dir> \
  [--emit-json]
```

Notes:
- `<pack_dir>` must be under `docs/project_management/packs/<bucket>/<feature>`.
- For strict Touch Set rules (including prefix entry semantics), see:
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`

### Git diff calibration (`from-git-diff`)

```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py \
  from-git-diff \
  --git-range <base>..<head> \
  [--emit-json]
```

Example:
```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py from-git-diff --git-range HEAD~1..HEAD
```

## Interpreting results (avoid overfitting)

Work Lift is designed to be **time-free** and deterministic; it is not a “days estimate”.

Prioritize interpretation in this order:
1) **Triggers**: hard split signals (canonical list + rationale live in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`, especially D8).
2) **Confidence + missing inputs**: low confidence means “don’t over-index on the score”; use it to drive follow-up questions (see D7).
3) **Score + estimated slices**: advisory signal; useful for rough shaping (see D9 for the lift→slices mapping and ADR slice cap guidance).

### `--emit-json` (machine output)

When `--emit-json` is provided, the output contract is:
- `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` (`CONTRACT-3:pm_lift_emit_json_v1`)

On success, stdout is JSON-only with required keys including:
`model_version`, `lift_score`, `estimated_slices`, `confidence`, `triggers`, `missing_inputs`, `vector`, `derived`.

## Makefile entry points (opt-in)

These targets are **opt-in** helpers and do not change default planning lint behavior.

### Intake / ADR

```bash
make pm-lift-intake FILE=docs/project_management/intake/adrs/queued/ADR-000X-example.md
make pm-lift-intake FILE=/tmp/pm_lift_intake_example.md EMIT_JSON=1
```

### Planning Pack

```bash
make pm-lift-pack PACK=docs/project_management/packs/active/warn-config-global-show-workspace-overrides
make pm-lift-pack PACK=docs/project_management/packs/active/warn-config-global-show-workspace-overrides EMIT_JSON=1
```

### Git diff calibration

```bash
make pm-lift-diff BASE=HEAD~1 HEAD=HEAD
make pm-lift-diff BASE=HEAD~1 HEAD=HEAD EMIT_JSON=1
```

## Advisory report (optional)

This repo provides an **optional** “advisory report” wrapper that:
- runs `pm_lift.py ... --emit-json` (CONTRACT-3),
- prints a short summary emphasizing **triggers + confidence** over the numeric score,
- never blocks by default, and is only integrated into planning lint when explicitly enabled.

Direct invocation:
```bash
python3 docs/project_management/system/scripts/planning/pm_lift_report.py --feature-dir <pack_dir>
python3 docs/project_management/system/scripts/planning/pm_lift_report.py --intake <path/to/intake_or_adr.md>
python3 docs/project_management/system/scripts/planning/pm_lift_report.py --git-range <base>..<head>
```

Opt-in planning lint integration (pack context only):
```bash
PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR=<pack_dir>
```

PowerShell:
```powershell
$env:PM_LIFT_ADVISORY="1"; make planning-lint-ps FEATURE_DIR=<pack_dir>
```

Field mapping (pinned to CONTRACT-3):
- `Lift Score` → `lift_score`
- `Estimated slices` → `estimated_slices`
- `Confidence` → `confidence`
- `Triggers` (top N, excluding `missing_inputs:*`) → `triggers`
- `Missing inputs` (top M) → `missing_inputs`

Phrasing rules (pinned):
- Advisory language only (no “must split”).
- Emphasize triggers + confidence over score.
- Print machine trigger tokens verbatim (no additional heuristics).

## Strict mode (opt-in)

Strict mode is an explicit, post-calibration enforcement posture. It is **not enabled by default** and must remain gated to avoid breaking legacy packs.

See:
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md`

## Happy-path walkthrough (3 commands)

Prereqs:
- `python3` on PATH
- run from repo root, inside a git checkout

1) Intake/ADR (example file you create locally; do not commit):
```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py from-intake --intake /tmp/pm_lift_intake_example.md
```

2) Planning Pack (strict pack example):
```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py from-impact-map --feature-dir docs/project_management/packs/active/warn-config-global-show-workspace-overrides
```

3) Git diff calibration:
```bash
python3 docs/project_management/system/scripts/planning/pm_lift.py from-git-diff --git-range HEAD~1..HEAD
```

What to look for:
- `Triggers:` list: split signals
- `Missing inputs:` list: which fields were unknown/unspecified
- `Confidence:` value: whether the score should be treated as a rough hint vs a reliable signal

## References (canonical)

- Decision log (policy posture + semantics): `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`
- Lift Vector rubric (authoring): `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md` (`CONTRACT-1`)
- Lift Vector schema: `docs/project_management/system/schemas/work_lift_vector.schema.json` (`CONTRACT-1`)
- Lift model config v1: `docs/project_management/system/schemas/work_lift_model.v1.json` (`CONTRACT-2`)
- `pm_lift --emit-json` contract: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` (`CONTRACT-3`)
- `validate_impact_map --emit-json` contract: `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md` (`CONTRACT-4`)
