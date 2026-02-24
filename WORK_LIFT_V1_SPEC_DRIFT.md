# Work Lift v1 — Spec Drift Report (Decision Log vs Implementation)

Date: 2026-02-23

This document captures **spec drift** between:

- Decision log: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D1–D10)
- Implementation (docs + tools + contracts), primarily:
  - `docs/project_management/system/schemas/work_lift_vector.schema.json` (CONTRACT-1)
  - `docs/project_management/system/schemas/work_lift_model.v1.json` (CONTRACT-2)
  - `docs/project_management/system/scripts/planning/pm_lift.py` (reference implementation)
  - `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` (CONTRACT-3)
  - `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md` (CONTRACT-4)
  - `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md`

Normativity note:
- During implementation, the seams pack (`work_lift_v1_seams/**`) defines “what to implement”, but this report is strictly about **drift from the decision log** to what is currently shipped/implemented under `docs/project_management/system/*` and in `pm_lift.py`.

---

## Drift index

| drift_id | severity | area | decision-log claim | implementation reality |
| --- | --- | --- | --- | --- |
| DRIFT-0001 | major | confidence enum | `lift_confidence` is `low\|medium\|high` | `confidence` is `high\|low` (v1) |
| DRIFT-0002 | major | vector schema | `contract.behavior_deltas` is `int` | schema allows `integer\|null` |
| DRIFT-0003 | major | prefix confidence | prefix entries can be `low` *unless* other inputs specified + prefixes narrow | any prefix entry forces `confidence="low"` |
| DRIFT-0004 | major | workstream triggers | workstream triggers include `estimated_total_slices > 8` and `touch.boundary_crossings > 4` | v1 model config does not implement these triggers |
| DRIFT-0005 | major | frontmatter summary | frontmatter summary fields exist and “can be updated by scripts” | no frontmatter update tool exists; output contract uses different key names |
| DRIFT-0006 | minor | stale TODOs | “define weights / decide null behavior / decide first gates” are open | already implemented + pinned in contracts/docs |
| DRIFT-0007 | minor | blowup math | `+ 10 * (behavior_deltas - 1)` + `null -> 0` | implementation clamps at `max(0, behavior_deltas - 1)` |
| DRIFT-0008 | minor | strict invariants | initial enforcement candidates are `behavior_deltas==1` + ADR slice cap | strict mode additionally requires `confidence=="high"` and `missing_inputs` empty |

---

## DRIFT-0001 — Confidence enum + naming drift (`lift_confidence` vs `confidence`)

Severity: major

Decision log:
- Frontmatter summary explicitly defines `lift_confidence (low|medium|high)`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:58`.

Implementation:
- CONTRACT-3 pins `confidence` to `"high" | "low"`: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md:37`.
- Model config pins `confidence.enum` to `["high","low"]`: `docs/project_management/system/schemas/work_lift_model.v1.json:62`.
- `pm_lift.py` enforces `confidence in ("high","low")`: `docs/project_management/system/scripts/planning/pm_lift.py:679`.

Impact:
- A reader treating the decision log as canonical will expect a third state (`medium`) and the `lift_confidence` field name, neither of which exist in the current v1 contracts or tooling.

Resolution options:
- Update decision log D3 to match v1 contracts (`confidence: high|low`), or
- Introduce `medium` into v1 contracts/tooling (not recommended; would require coordinated contract changes).

---

## DRIFT-0002 — `contract.behavior_deltas` type drift (decision says `int`; schema allows `null`)

Severity: major

Decision log:
- Canonical field list says: `contract.behavior_deltas (int)`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:152`.

Implementation:
- Schema allows `behavior_deltas` to be `integer|null` (min 1 when integer): `docs/project_management/system/schemas/work_lift_vector.schema.json:73`.
- `pm_lift.py` treats missing/`null` numeric values as “missing inputs” (converted to 0 for scoring): `docs/project_management/system/scripts/planning/pm_lift.py:485` and `:340`.
- Strict mode requires `vector.contract.behavior_deltas == 1` (intake strict): `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md:54`.

Impact:
- Decision log implies “always present integer”, while the shipped schema explicitly permits “unknown at discovery time”.

Resolution options:
- Update decision log D6 to reflect schema reality (`integer|null` allowed at discovery-time; strict-mode later enforces `==1`).

---

## DRIFT-0003 — Prefix-entry confidence nuance drift (decision allows exceptions; v1 forces low)

Severity: major

Decision log:
- Prefix confidence guidance: “`lift_confidence` should be `low` **unless** all other numeric inputs are fully specified and the prefixes are narrow”: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:236`.

Implementation:
- CONTRACT-3 pins: `confidence == "low"` iff `missing_inputs` non-empty **OR** prefix entries exist: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md:57`.
- Model config encodes `touch_set_contains_prefix_entries -> low` with no exception mechanism: `docs/project_management/system/schemas/work_lift_model.v1.json:64`.
- `pm_lift.py` implements that rule exactly: `docs/project_management/system/scripts/planning/pm_lift.py:670`.

Impact:
- Decision log implies a future “high confidence with narrow prefixes” path that does not exist in v1; consumers should expect `low` whenever prefixes are present.

Resolution options:
- Update decision log D7 prefix confidence language to match v1 (always low if any prefix), or
- Extend v2 confidence rules to support “narrow prefix exception” explicitly (keep v1 immutable).

---

## DRIFT-0004 — Workstream trigger drift (D8 lists triggers not present in v1 model config)

Severity: major

Decision log:
- Workstream triggers include:
  - `estimated_total_slices > 8`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:252`
  - `touch.boundary_crossings > 4`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:253`
  - `lift_score > 60`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:254`

Implementation:
- v1 model config workstream triggers include only `likely_split:lift_score>60`: `docs/project_management/system/schemas/work_lift_model.v1.json:78`.
- There is no concept of `estimated_total_slices` in `pm_lift.py` output (CONTRACT-3 is per-input): `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md:32`.

Impact:
- Decision log lists workstream-level triggers that require aggregation across multiple items (`estimated_total_slices`) or rely on a stable taxonomy (`boundary_crossings`) that is explicitly out-of-scope elsewhere.

Resolution options:
- Update decision log D8 to label `estimated_total_slices` / `boundary_crossings > 4` as deferred / non-v1, or
- Add v2 workstream-level tooling that aggregates across items and emits those triggers (separate capability from per-item `pm_lift`).

---

## DRIFT-0005 — Frontmatter summary drift (specified fields + update expectation vs no implementation)

Severity: major

Decision log:
- D3 defines frontmatter summary fields and implies script update is expected: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:54`.
- Fields named there:
  - `lift_model_version`, `lift_score`, `lift_estimated_slices`, `lift_confidence`, `lift_split_triggers`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:55`.

Implementation:
- No tooling exists that rewrites intake/ADR/frontmatter in-place (scope explicitly excludes it): `work_lift_v1_seams/scope_brief.md:18`.
- The stable machine contract uses different key names:
  - `model_version`, `estimated_slices`, `confidence`, `triggers`: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md:32`.

Impact:
- Decision log describes a frontmatter “summary interface” that does not exist today, and it uses field names that do not match CONTRACT-3.

Resolution options:
- Treat frontmatter summary as explicitly deferred (update decision log D3 accordingly), or
- Introduce a new idempotent tool (v2 scope) that writes frontmatter summaries, with an explicit mapping from CONTRACT-3 fields → frontmatter keys.

---

## DRIFT-0006 — Stale “Open Questions / TODOs” in decision log (already resolved in v1 contracts)

Severity: minor

Decision log (still lists as open):
- “Define the deterministic formula (weights)…”: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:309`.
- “Decide how to handle `null` fields…”: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:310`.
- “Decide which invariants become hard gates first…”: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:311`.

Implementation reality:
- Weights + selection semantics are pinned in `docs/project_management/system/schemas/work_lift_model.v1.json`.
- Null handling + confidence is pinned in CONTRACT-3 and the model config: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md:55`.
- Initial strict-mode invariant set + opt-in mechanism are pinned in `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md:42`.

Impact:
- Readers may conclude key design decisions are still unresolved even though they are already shipped/pinned elsewhere.

Resolution options:
- Replace the “Open Questions” list with a “Resolved by v1 contracts” note + pointers, or
- Move genuinely-open items to a new “v2/vNext” section (e.g., subsystem taxonomy for `boundary_crossings`, workstream-level aggregation).

---

## DRIFT-0007 — `behavior_deltas` blowup math is underspecified vs v1 implementation clamp

Severity: minor

Decision log:
- Blowup rule is stated as: `+ 10 * (contract.behavior_deltas - 1)`: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:207`.
- Null handling states numeric `null -> 0` for scoring: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:231`.

Implementation:
- `pm_lift.py` applies the blowup with an explicit clamp: `max(0, behavior_deltas - 1)`: `docs/project_management/system/scripts/planning/pm_lift.py:546`.

Impact:
- As written, the decision log + null-handling rule implies `behavior_deltas=null` could contribute a negative blowup term (if interpreted literally), which is not how v1 behaves.

Resolution options:
- Update decision log D7 to specify the clamp explicitly (recommended), or
- Disallow `behavior_deltas=null` in schema/tooling (not recommended; breaks discovery-time unknown semantics).

---

## DRIFT-0008 — Strict-mode invariant set is more specific than the decision log’s enforcement-candidate note

Severity: minor

Decision log:
- D4 sets posture: advisory-first; enforce only the most reliable invariants: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:117`.
- D8 suggests initial enforcement candidates:
  - `contract.behavior_deltas == 1`
  - ADR slice cap (`estimated_slices <= 3`): `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:258`.

Implementation:
- Strict mode intake invariants additionally require:
  - `confidence == "high"`
  - `missing_inputs` empty
  - `vector.contract.behavior_deltas == 1`
  - `estimated_slices <= 3`: `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md:52`.

Impact:
- The decision log does not reflect the stricter v1 strict-mode invariant set now published in planning standards.

Resolution options:
- Update decision log D4/D8 to explicitly reference the strict-mode standard and/or list the full v1 strict invariant set.

---

## Notes on “not drift” (intentional gaps)

These are *not* treated as spec drift in this report because they are explicitly deferred/out-of-scope:

- Deterministic taxonomy for `touch.boundary_crossings` (out-of-scope): `work_lift_v1_seams/scope_brief.md:17`.
- Default enforcement in workflow/lint without explicit opt-in (strict mode must remain gated): `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md:9`.
