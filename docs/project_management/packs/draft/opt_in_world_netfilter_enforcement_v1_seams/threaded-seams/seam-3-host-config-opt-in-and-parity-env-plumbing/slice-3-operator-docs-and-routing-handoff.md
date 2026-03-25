---
slice_id: S3
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to host routing semantics for when isolate_network is requested"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-04
  - C-05
  - C-06
open_remediations:
  - REM-003
  - REM-004
candidate_subslices: []
---
### S3 - Publish operator docs/examples and the downstream routing handoff

- **User/system value**: operators can understand exactly when filtering is requested, and downstream `SEAM-1` gets an unambiguous handoff contract instead of a planning placeholder.
- **Scope (in/out)**:
  - In:
    - Update `docs/reference/config/world.md` and `docs/CONFIGURATION.md` with the three-way gate alignment:
      - `world.net.filter` answers whether the host may request enforcement
      - `WORLD_NETFILTER_ENABLE=1` answers whether the world backend may apply enforcement
      - policy `net_allowed` answers what is allowed once enforcement is requested
    - Add examples for allow-all, deny-all, and restrictive allowlist posture.
    - Record the downstream handoff rule that `SEAM-1` consumes: `isolate_network` is only requested when the effective config gate is enabled and policy `net_allowed` is restrictive after canonicalization.
  - Out:
    - Runtime routing implementation in `SEAM-1`
- **Acceptance criteria**:
  - `REM-003`'s operator-workflow gap is fully represented by this slice and the targeted docs.
  - The docs distinguish override input from exported parity output.
  - The examples make it clear that restrictive policy alone does not request enforcement unless the config gate is enabled.
  - The downstream handoff rule is explicit enough for `SEAM-1` to revalidate without inventing new semantics.
- **Dependencies**:
  - `S1` and `S2`
  - `../../threading.md` (`THR-03`)
- **Verification**:
  - Doc review against `review.md` mismatch hotspots.
  - Downstream planning check: `SEAM-1/S2` can cite this slice rather than a future-seam assumption.
- **Review surface refs**:
  - `review.md` (doc ambiguity hotspot)

Checklist:
- Implement: operator docs + examples
- Validate: examples cover allow-all, deny-all, and restrictive allowlist
- Handoff: state the downstream `SEAM-1` routing rule explicitly
- Cleanup: remove pack-local ambiguity once user-facing docs land
