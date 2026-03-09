# SEAM-4 — Pack-derived lift inputs (Impact Map + prefix expansion) (threaded decomposition)

## Seam Brief (Restated)

- **Seam ID**: SEAM-4
- **Name**: Lift from Planning Pack artifacts
- **Goal / user value**: Compute a reliable advisory lift signal from Planning Packs (especially `impact_map.md`) without requiring manual counts, while handling directory prefixes safely.
- **Type**: capability
- **Scope**
  - In:
    - Derive Touch Set counts from `impact_map.md` using `validate_impact_map.py --emit-json` (CONTRACT-4).
    - Count directory/prefix tokens as **1** in raw counts (for the Lift Vector inputs).
    - Expand prefixes deterministically from the repo file list (repo-root-relative) using `git ls-files <prefix>` for *lift estimation only*, discounted and capped per prefix entry.
    - Degrade confidence when prefix entries are present (since expansion reflects current `HEAD`).
    - Surface diagnostics so reviewers can see how prefixes influenced the estimate.
  - Out:
    - Editing/reformatting `impact_map.md` or rewriting Touch Sets.
    - Attempting to “predict” future files beyond the current repo state.
    - Introducing enforcement gates (advisory-only here; enforcement is a downstream policy decision).
- **Key invariants / rules**
  - Prefix expansion must be deterministic and bounded:
    - `EXPAND_DISCOUNT = 0.20`
    - `EXPAND_CAP = 10`
    - Per-prefix effective contribution: `min(expanded, EXPAND_CAP) * EXPAND_DISCOUNT` (max `2.0`).
  - Prefix presence must degrade confidence unless explicitly overridden by a future policy decision.
- **Touch surface**
  - `docs/project_management/system/scripts/planning/validate_impact_map.py` (producer; source-of-truth for CONTRACT-4 output)
  - `docs/project_management/system/scripts/planning/pm_lift.py` (consumer, via SEAM-3; do not duplicate wiring work here)
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
  - Spec doc: `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md` (CONTRACT-4)
  - Conformance tests: `docs/project_management/system/scripts/planning/tests/test_validate_impact_map_emit_json_contract.py`
  - Prefix counting spec: `docs/project_management/system/scripts/planning/impact_map_touch_counts_v1.md` (raw vs effective)
  - Pure helper: `docs/project_management/system/scripts/planning/impact_map_touch_counts.py`
  - Unit tests: `docs/project_management/system/scripts/planning/tests/test_impact_map_touch_counts.py`
- **Verification**
  - Use at least one Planning Pack with:
    - explicit file entries only,
    - directory/prefix entries,
    - mixed entries (explicit + prefixes).
  - Confirm:
    - raw counts match authored tokens,
    - effective counts reflect discount/cap,
    - confidence is downgraded when prefixes exist,
    - prefix diagnostics are emitted for auditability.
  - Coordination note: any change to CONTRACT-4 field names/types/semantics MUST be coordinated with SEAM-3 consumption in `pm_lift.py` to avoid silent drift.
- **Threading constraints**
  - Upstream blockers:
    - SEAM-3 (`CONTRACT-3:pm_lift_emit_json_v1`) for stable result output semantics and where derived diagnostics ultimately land.
  - Downstream blocked seams: SEAM-5
  - Contracts produced (owned):
    - `CONTRACT-4:impact_map_emit_json_v1` (JSON output for `validate_impact_map.py --emit-json`)
  - Contracts consumed:
    - None (inputs are Planning Pack artifacts + repo file listing for deterministic expansion).

## Slice index

- `S1` → `slice-1-contract-4-impact-map-emit-json.md`: publish and lock CONTRACT-4 so downstream seams can consume pack-derived allowlists deterministically (including prefix entry signaling)
- `S2` → `slice-2-prefix-expansion-and-derived-counts.md`: specify and harden the prefix expansion + raw/effective counting policy with fixtures/tests (implementation wiring into `pm_lift` is owned by SEAM-3)

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `CONTRACT-4:impact_map_emit_json_v1`
    - Definition: `validate_impact_map.py --emit-json` returns a JSON object with per-action allowlists (`create/edit/deprecate/delete`) and a signal for directory-prefix entries (e.g., `dir_prefixes`).
    - Produced by: S1 (explicit contract spec + conformance guardrails) and protected by S2 (fixtures/tests for prefix semantics and determinism).
- **Contracts consumed**:
  - None (this seam defines the pack-derived input contract rather than consuming other seam-owned contracts).
- **Dependency edges honored**:
  - `SEAM-4 blocks SEAM-5`: S1 publishes a stable derived-input contract so Planning Pack workflows can compute lift without bespoke parsing.
  - `SEAM-4 feeds SEAM-3`: SEAM-3 consumes CONTRACT-4 when implementing `pm_lift from-impact-map` derived inputs (explicitly called out in SEAM-3 S2).
- **Parallelization notes**:
  - What can proceed now:
    - S1 can land independently (contract spec + validator conformance + docs).
    - S2 can land largely independently (fixtures/tests/spec for prefix counting/expansion), with no need to change `pm_lift.py` wiring inside this seam.
  - What must coordinate:
    - Any change to `validate_impact_map.py --emit-json` field names/types/semantics must be coordinated with SEAM-3 consumption (CONTRACT-3/derived signals), to avoid silent drift.
