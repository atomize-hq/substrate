### S2 — Prefix expansion + derived touch counts (raw vs effective) are specified and hardened

- **User/system value**: Make prefix handling safe and inspectable by (a) counting authored directory tokens correctly in raw counts and (b) optionally expanding them deterministically for estimation, bounded by discount/cap, with clear diagnostics.
- **Scope (in/out)**:
  - In:
    - Specify how to compute:
      - raw counts (directory tokens count as 1),
      - effective counts for lift estimation (discounted/capped per prefix),
      - diagnostics (explicit vs prefix counts, per-prefix expansion sizes).
    - Ensure prefix expansion is deterministic and bounded (no globs, no prediction, no non-deterministic filesystem walking).
  - Out:
    - Changing `pm_lift --emit-json` output contract (owned by SEAM-3 / CONTRACT-3).
    - Deciding enforcement posture for prefix usage (a future policy decision; here we degrade confidence only).
- **Acceptance criteria**:
  - Prefix expansion policy is pinned (cap/discount) and is treated as part of SEAM-4’s invariant semantics.
  - Derived-count computations are testable in isolation (pure logic with injectable prefix-expansion source).
  - Diagnostics are sufficient for reviewers to understand the influence of prefixes (per-prefix expanded counts and effective contribution).
- **Dependencies**:
  - Consumes: `CONTRACT-4:impact_map_emit_json_v1` (input shape)
  - Coordinated consumer: SEAM-3 uses these semantics when wiring `pm_lift from-impact-map` (do not duplicate wiring tasks here).
- **Verification**:
  - Unit-style tests over fixtures covering:
    - explicit-only entries,
    - prefix-only entries,
    - mixed entries,
    - large expansions where cap binds,
    - empty expansions (prefix matches nothing).

#### S2.T1 — Publish the “raw vs effective” counting spec (including discount/cap math)

- **Outcome**: A crisp definition of how to interpret Touch Set tokens for lift estimation, consistent across tools.
- **Inputs/outputs**:
  - Inputs:
    - SEAM-4 brief invariants (`EXPAND_DISCOUNT`, `EXPAND_CAP`)
    - CONTRACT-4 spec (S1)
  - Outputs:
    - A short section added to the CONTRACT-4 spec doc (or a sibling doc) that defines:
      - raw counting rules,
      - effective counting rules,
      - deterministic expansion source (`git ls-files <prefix>` or equivalent),
      - confidence degradation trigger when prefixes exist.
- **Implementation notes**:
  - Include concrete examples:
    - one prefix with 3 matches → effective +`0.6`,
    - one prefix with 100 matches → effective +`2.0` (cap binds).
  - State that effective counts are for lift estimation only; raw counts reflect authored intent.
- **Acceptance criteria**:
  - Readers can implement the counting logic without reading `pm_lift.py`.

Checklist:
- Implement:
  - Add spec text + examples.
- Validate:
  - Confirm math matches the invariant constants exactly.
- Cleanup:
  - Keep it short and deterministic.

#### S2.T2 — Implement derived-count computation as a small, testable helper (no `pm_lift` wiring here)

- **Outcome**: A helper that turns CONTRACT-4 JSON into a derived touch-count structure (raw + effective + diagnostics) in a deterministic way.
- **Inputs/outputs**:
  - Inputs:
    - CONTRACT-4 JSON object
    - A deterministic expansion provider for prefixes (injectable; e.g., a callable or interface)
  - Outputs:
    - New helper module colocated with planning scripts, e.g.:
      - `docs/project_management/system/scripts/planning/impact_map_touch_counts.py`
    - Export a single pure function such as:
      - `compute_impact_map_touch_counts(impact_map_emit_json, *, expand_prefix) -> dict`
- **Implementation notes**:
  - Keep deterministic behavior explicit:
    - sort all output lists/maps by key,
    - treat missing keys as empty arrays (but prefer enforcing presence via S1 tests),
    - never read the filesystem directly; expansion comes from injected provider.
  - Output should include at minimum:
    - per-section: explicit count, prefix entry count, raw count, effective count, per-prefix expanded counts,
    - an overall “prefix present” boolean or derived `dir_prefixes` passthrough.
  - This helper is a seam-owned artifact; SEAM-3 may choose to import it for `pm_lift from-impact-map` wiring.
- **Acceptance criteria**:
  - Helper outputs match the spec (S2.T1) for all fixture cases.

Checklist:
- Implement:
  - Write the helper module and pure function.
  - Keep I/O (git/subprocess) out of the core function.
- Test:
  - Add unit tests (S2.T3) with stubbed prefix expansion provider.
- Cleanup:
  - Keep public function signature stable and minimal.

#### S2.T3 — Add fixtures + tests for prefix semantics (cap/discount + diagnostics)

- **Outcome**: Regression coverage that prevents accidental changes to the prefix semantics.
- **Inputs/outputs**:
  - Inputs:
    - Helper module (S2.T2)
    - CONTRACT-4 fixture JSON blobs (can reuse from S1.T3 fixtures)
  - Outputs:
    - New tests under `docs/project_management/system/scripts/planning/tests/` asserting:
      - raw counts treat prefixes as 1,
      - effective counts apply cap/discount per prefix,
      - diagnostics include per-prefix expanded counts.
- **Implementation notes**:
  - Use a stub expansion provider returning deterministic lists per prefix.
  - Include at least one case where cap binds and one where expansion is empty.
- **Acceptance criteria**:
  - Tests fail on any change to discount/cap math or diagnostic field presence.

Checklist:
- Implement:
  - Add unit tests for explicit-only, prefix-only, mixed cases.
- Validate:
  - Ensure assertions are resilient to additive fields but strict on required semantics.
- Cleanup:
  - Keep fixtures minimal; avoid “giant” expansion lists (use counts).

