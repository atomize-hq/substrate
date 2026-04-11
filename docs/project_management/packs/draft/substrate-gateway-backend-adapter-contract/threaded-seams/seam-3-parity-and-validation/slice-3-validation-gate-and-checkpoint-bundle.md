---
slice_id: S3
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - checkpoint cadence or platform scope changes
    - manual validation assertions widen beyond accepted upstream truth
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
---
### S3 - Validation gate and checkpoint bundle

#### Goal

Turn the checkpoint plan and manual validation assertions into one deterministic downstream validation bundle that can prove parity and compatibility without inventing new proof categories at closeout.

#### Dependencies

- `../../pre-planning/ci_checkpoint_plan.md`
- `../../review_surfaces.md`
- `review.md`

#### S3.T1 - Finalize the manual validation bundle

- **Outcome**:
  - `manual_testing_playbook.md` names the exact review assertions for operator, policy, event, and trace ownership surfaces.
- **Files**:
  - `../../manual_testing_playbook.md`
  - `../../review_surfaces.md`
- **Acceptance criteria**:
  - every validation assertion cites one accepted upstream owner
  - the validation bundle remains downstream of `C-01` through `C-04`

#### S3.T2 - Turn checkpoint intent into seam-local proof gates

- **Outcome**:
  - `pre-planning/ci_checkpoint_plan.md` is reconciled with the seam-local proof cadence for `CP2`.
- **Files**:
  - `../../pre-planning/ci_checkpoint_plan.md`
- **Acceptance criteria**:
  - the checkpoint plan states the exact compile-parity, feature-smoke, and platform-validation intent needed by this seam
  - the bundle is concrete enough for `S99` closeout evidence
