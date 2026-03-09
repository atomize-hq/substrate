# Planning Work Lift — Strict Mode (Opt-in, Gated)

This standard defines **strict mode** for Work Lift. Strict mode is a **post-calibration**, **opt-in** enforcement surface that can fail when invariant checks do not pass.

Strict mode is designed to:
- preserve **advisory-first** lift usage by default,
- avoid breaking legacy Planning Packs,
- promote only the most reliable invariants to enforcement after calibration.

References (canonical):
- Policy posture + thresholds: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`
- Lift advisory workflow: `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
- `pm_lift --emit-json` contract: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` (`CONTRACT-3`)
- `validate_impact_map --emit-json` contract: `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md` (`CONTRACT-4`)

## What “strict mode” means

Strict mode is a wrapper-driven set of checks that:
- consumes stable machine outputs (CONTRACT-3/4),
- evaluates an explicit invariant set,
- exits non-zero only when strict mode is explicitly enabled.

Strict mode does **not** change scoring math or `pm_lift.py` output semantics.

## Eligibility (pinned)

Strict mode is eligible only for Planning Packs with:
- `<pack_dir>/tasks.json` containing `meta.slice_spec_version >= 2`.

Legacy packs (`meta.slice_spec_version` missing or `< 2`) are **not eligible** for strict-mode gating:
- strict checker prints `NOT ELIGIBLE: ...` and exits `0`.

## Opt-in mechanism (pinned)

Strict mode is enabled only when **both** are true:
- the target is eligible (pack-only; intake is always eligible), and
- `PM_LIFT_STRICT=1` is set.

Convenience entry point:
- Make target: `pm-lift-strict`

## Strict invariant set (v1, pinned)

These invariants are the v1 strict set. When strict mode is enabled, **all MUST pass**.

### Intake / ADR strict checks (`--intake <path>`)

Inputs:
- `pm_lift.py from-intake --intake <path> --emit-json` (CONTRACT-3)

Invariants (all MUST pass):
- `confidence == "high"`
- `missing_inputs` is empty
- `vector.contract.behavior_deltas == 1`
- `estimated_slices <= 3`

### Planning Pack strict checks (`--feature-dir <pack_dir>`)

Eligibility gate:
- If `tasks.json.meta.slice_spec_version < 2`: print `NOT ELIGIBLE` and exit `0`.

Inputs:
- `validate_impact_map.py --feature-dir <pack_dir> --emit-json` (CONTRACT-4)
- `pm_lift.py from-impact-map --feature-dir <pack_dir> --emit-json` (CONTRACT-3)

Invariants (all MUST pass):
- Prefix entries are forbidden in strict packs:
  - `dir_prefixes == []` from CONTRACT-4
- `pm_lift.py ... --emit-json` succeeds and stdout conforms to CONTRACT-3 required keys/types (additive keys allowed)

## How to run (examples)

Pack (strict-enabled, eligible only):
```bash
PM_LIFT_STRICT=1 python3 docs/project_management/system/scripts/planning/pm_lift_strict_check.py \
  --feature-dir docs/project_management/packs/active/<feature>
```

Intake/ADR:
```bash
PM_LIFT_STRICT=1 python3 docs/project_management/system/scripts/planning/pm_lift_strict_check.py \
  --intake docs/project_management/intake/adrs/<bucket>/ADR-000X-foo.md
```

Make (pack):
```bash
make pm-lift-strict PACK=docs/project_management/packs/active/<feature>
```

Make (intake):
```bash
make pm-lift-strict FILE=/tmp/pm_lift_intake_example.md
```

## Rollout stages + promotion criteria (pinned)

Stage 1 (now):
- Strict mode is opt-in only (`PM_LIFT_STRICT=1`).
- No default enforcement in lint or other workflow runners.

Promotion-to-default criteria (must be met before enabling strict checks by default anywhere):
- ≥ 20 calibration runs across ≥ 10 distinct eligible packs
- strict failure false-positive rate ≤ 5% across those runs
- any exceptions are explicitly documented in the allowlist below (path + rationale)

## Allowlist (v1, doc-only)

If calibration identifies legitimate exceptions, record them here. This is **documentation only** in v1; the strict checker does not consult it unless explicitly implemented later.

Allowlist entry template:
- `path`: `<pack_dir>` or `<intake_md>`
- `rationale`: why strict would fail and why it’s acceptable
- `review_by` (optional): `YYYY-MM-DD` date for re-evaluation

