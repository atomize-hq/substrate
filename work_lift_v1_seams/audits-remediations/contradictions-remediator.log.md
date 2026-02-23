# Contradictions Remediation Log
Date: 2026-02-23

Input: `contradictions-audit.report.json`

## Triage

- **CX-0001 (critical)** — Lift Vector v1 nullability rules (schema contract drift).
- **CX-0002 (critical)** — Lift Vector v1 `model_version` required vs defaulting rules.
- **CX-0003 (major)** — `pm_lift.py from-git-diff` CLI argument form drift.
- **CX-0004 (major)** — Pack-derived lift claims vs legacy-mode `validate_impact_map.py --emit-json` behavior.

All issues are resolved by converging the planning docs on one concrete contract, grounded in:
- the canonical decision log (`WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`), and
- the current reference implementations (`docs/project_management/system/scripts/planning/pm_lift.py`, `.../validate_impact_map.py`).

## CX-0001 — Lift Vector v1: which numeric fields may be null

- **Contradiction**: One doc claimed missing numeric inputs may be `null` (treated as 0 + confidence downgrade), while another doc restricted `null` to only two numeric fields.
- **Resolution type**: Single truth (schema + rubric + decision log converge).
- **Chosen truth**:
  - Numeric **count** fields may be `null` to represent discovery-time “unknown”.
  - Tools treat `null` as 0 for scoring, degrade confidence, and emit deterministic `missing_inputs:<field>` triggers.

- **Evidence**:
  - `pm_lift.py` treats `None` as “missing” for any numeric field and converts it to 0 for scoring (`_num`, `missing.append(...)`). See `docs/project_management/system/scripts/planning/pm_lift.py:96-105`.
  - Canonical field list now encodes numeric fields as `int|null` where unknowns are meaningful. See `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:127-167`.

- **Docs changed**:
  - `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md:21` — clarified that numeric count inputs may be `null` (unknown) and required scoring semantics.
  - `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md:30-35` — removed the “only two fields may be null” restriction; defined `integer|null` semantics for numeric count fields.
  - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:139-167` — made nullability explicit in the canonical field list so downstream docs can’t drift.

- **Status**: Fixed

## CX-0002 — Lift Vector v1: is `model_version` required or defaultable?

- **Contradiction**: One doc stated `model_version` is required; another doc proposed defaulting to 1 when missing.
- **Resolution type**: Scoped truth (contract clarified; default rule made explicit).
- **Chosen truth**:
  - `model_version` is optional in authored vectors; if missing, tools default it to `1`.
  - Machine outputs should still include the resolved `model_version` so consumers aren’t forced to infer.

- **Evidence**:
  - `pm_lift.py` defaults `model_version` to 1 when missing: `model_version = int(vector.get("model_version") or 1)`. See `docs/project_management/system/scripts/planning/pm_lift.py:117-120`.
  - Canonical decision log now states the same default. See `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:139`.

- **Docs changed**:
  - `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md:31-34` — removed `model_version` from the “required keys” list; specified default-to-1 rule.
  - `work_lift_v1_seams/seam-3-pm-lift-core-engine.md:41-42` — kept the de-risk plan but aligned wording with the clarified contract.
  - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:139` — added a canonical, explicit default rule for `model_version`.

- **Status**: Fixed

## CX-0003 — CLI contract: `pm_lift.py from-git-diff` argument form

- **Contradiction**: One doc used `--git-range base..head`; another used positional `<base> <head>`.
- **Resolution type**: Outdated doc update (align docs to the implemented CLI).
- **Chosen truth**:
  - `pm_lift.py from-git-diff` takes `--git-range <base>..<head>`.

- **Evidence**:
  - CLI parser requires `--git-range`. See `docs/project_management/system/scripts/planning/pm_lift.py:481-483`.

- **Docs changed**:
  - `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md:50-57` — updated the recipe to use `--git-range`.

- **Status**: Fixed

## CX-0004 — Pack-derived lift: strict vs legacy semantics for `validate_impact_map.py --emit-json`

- **Contradiction**: Scope text implied pack-derived lift comes from the pack’s `impact_map.md` in general, while the CONTRACT-4 doc explicitly defines legacy-mode `--emit-json` as a stable shape with empty allowlists.
- **Resolution type**: Scope clarification (strict vs legacy boundary made explicit).
- **Chosen truth**:
  - Pack-derived lift from `impact_map.md` applies to **strict** packs.
  - Legacy packs still produce the same JSON object shape, but with empty allowlists (so derived touch counts are effectively zero).

- **Evidence**:
  - `validate_impact_map.py` legacy mode short-circuits and emits empty allowlists for `--emit-json`. See `docs/project_management/system/scripts/planning/validate_impact_map.py:331-336`.
  - `_emit_json` always emits the full object shape including `dir_prefixes`. See `docs/project_management/system/scripts/planning/validate_impact_map.py:297-312`.

- **Docs changed**:
  - `work_lift_v1_seams/scope_brief.md:19-28` — scoped pack-derived lift to strict packs and called out legacy empty-allowlist behavior.

- **Status**: Fixed

## Verification

- Ran `contradictions_scan.py` over `work_lift_v1_seams/` + `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`.
- Result: `contradictions-audit.scan.after.json` contains **0 candidate keys**.
