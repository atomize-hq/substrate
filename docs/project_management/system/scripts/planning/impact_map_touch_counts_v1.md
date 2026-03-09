# SEAM-4 S2: Impact Map derived touch counts (raw vs effective) — v1

This document specifies how to compute **raw** and **effective** touch counts from
`CONTRACT-4:impact_map_emit_json_v1` output (`validate_impact_map.py --emit-json`).

This spec is SEAM-4-owned; it is intended to be implemented by a pure helper and optionally consumed by `pm_lift.py`.

---

## Terms

- **Explicit token**: a repo-relative path token that does **not** end with `/` (represents one file).
- **Prefix token**: a repo-relative path token that **does** end with `/` (represents a directory/prefix allow entry).
- **Raw counts**: what the authored Touch Set implies if every prefix token counts as **1**.
- **Effective counts**: lift-estimation-only counts derived by optionally expanding prefix tokens and applying discount/cap.

Prefix expansion is **for estimation only**:
- tools MUST NOT rewrite `impact_map.md`
- expansion reflects current repo `HEAD`, so prefix presence typically degrades confidence (policy is enforced by the consumer)

---

## Constants (v1 invariants)

- `EXPAND_DISCOUNT = 0.20`
- `EXPAND_CAP = 10`

Per-prefix effective contribution:
- `min(expanded_matches, EXPAND_CAP) * EXPAND_DISCOUNT`
- maximum contribution per prefix token: `2.0`

---

## Raw counting (normative)

For each section `S ∈ {create, edit, deprecate, delete}`:

- `explicit_files = count(tokens in S that do not end with "/")`
- `prefix_entries = count(tokens in S that end with "/")`
- `raw_count = explicit_files + prefix_entries`

---

## Effective counting (normative)

Effective counts are computed per section for lift estimation:

- Start: `effective_count = explicit_files`
- For each prefix token `p` in the section:
  - deterministically expand `p` to a list of repo files using a deterministic provider (e.g., `git ls-files <prefix>`)
  - `expanded_matches = len(expanded_files)`
  - add `min(expanded_matches, EXPAND_CAP) * EXPAND_DISCOUNT` to `effective_count`

If prefix expansion is disabled, prefix tokens contribute `0.0` to `effective_count`.

---

## Derived diagnostics (normative)

Implementations MUST surface per-section diagnostics:

- `explicit_files`, `prefix_entries`, `raw_count`, `effective_count`
- `prefix_expanded_counts`: map `{prefix_token -> expanded_matches}`

Implementations MUST also surface:

- `dir_prefixes`: sorted unique list of all prefix tokens across all sections
- `prefix_present`: boolean (`dir_prefixes` non-empty)

All outputs must be deterministic (sorted keys/lists).

---

## Examples

Example A (prefix with 3 matches):
- explicit files: 0
- prefix token `p/` expands to 3 files
- raw: `raw_count = 0 + 1 = 1`
- effective: `0 + min(3,10)*0.2 = 0.6`

Example B (prefix with 100 matches):
- effective: `min(100,10)*0.2 = 2.0` (cap binds)

Example C (prefix expansion disabled):
- explicit files: 1
- prefix entries: 1
- raw: `2`
- effective: `1.0` (prefix contributes `0.0`)

