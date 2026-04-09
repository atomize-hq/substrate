---
slice_id: S3
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S3 - Ownership and archived drift normalization

- **User/system value**: downstream seams can consume one operator boundary without re-importing stale ADR links, archived command order, or outdated ownership language.
- **Scope (in/out)**:
  - In:
    - normalize ADR-0040 related-doc drift that still points at outdated pack locations
    - make archived gateway-planning artifacts explicitly non-authoritative where they conflict with `C-01`
    - align pack-root planning surfaces so `SEAM-2`, `SEAM-3`, and `SEAM-4` can cite one ownership split and one command boundary
  - Out:
    - field-level status schema, policy, runtime-parity, or manual-validation content owned by later seams
- **Acceptance criteria**:
  - ADR-0040 related-doc references no longer point downstream planners at stale ownership paths
  - archived gateway planning no longer reads like the current operator contract
  - the pack-root planning surfaces and `C-01` use the same ownership split and command-family wording
- **Dependencies**:
  - `C-01`
  - `THR-01`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/_archived/next/llm_gateway_in_world/contract.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- **Verification**:
  - readback diff across ADR and planning surfaces
  - downstream seam briefs still inherit the same operator boundary after normalization
- **Rollout/safety**: keep stale archival evidence available as history without letting it remain normative.
- **Review surface refs**: `../../review_surfaces.md` R1

#### S3.T1 - Correct ADR and pack-root ownership drift

- **Outcome**: the current repo points downstream planners at the live ownership surfaces instead of stale pack paths or half-updated ADR references.
- **Inputs/outputs**:
  - Inputs: `C-01`, ADR-0040, spec manifest, impact map
  - Outputs: aligned related-doc references and pack-root ownership wording
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep the edits narrow to ownership and command-boundary drift
  - avoid pulling later seam content into this docs-only slice
- **Acceptance criteria**:
  - downstream planners no longer have to choose between `packs/active` and `packs/implemented` references for the same external owner
- **Test notes**:
  - N/A beyond readback validation
- **Risk/rollback notes**:
  - if a referenced external doc moved again, update the pointer here rather than letting the drift propagate downstream

Checklist:
- Implement: ADR and pack-root docs updates
- Test: N/A
- Validate: read back the touched links and ownership wording
- Cleanup: remove stale pointers that conflict with `C-01`

#### S3.T2 - Mark archived gateway planning as historical evidence only

- **Outcome**: archived gateway planning remains usable as evidence without competing with the current operator contract.
- **Inputs/outputs**:
  - Inputs: archived gateway contract/order wording, current `C-01` wording
  - Outputs: explicit non-authority posture or normalized references that stop archived command ordering from leaking forward
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - preserve historical context while making the current operator contract unmistakable
- **Acceptance criteria**:
  - no current planning or operator doc implies archived command ordering is still normative
  - downstream seams can cite `C-01` without caveats about archived ambiguity
- **Test notes**:
  - N/A beyond readback validation
- **Risk/rollback notes**:
  - avoid deleting historical evidence when a smaller non-authority clarification will do

Checklist:
- Implement: archive-warning or drift-normalization updates
- Test: N/A
- Validate: compare archive wording to `C-01`
- Cleanup: remove duplicate current-contract wording from archived references where appropriate
