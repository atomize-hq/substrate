---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - platform support wording drift
    - checkpoint wording drift
    - upstream contract wording drift
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
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations:
  - REM-002
candidate_subslices: []
---
### S1 - Freeze platform evidence claim boundaries

- **User/system value**: smoke, playbook, and checkpoint work can execute against one explicit claim boundary instead of rediscovering how much Linux, macOS, and Windows behavior may be asserted.
- **Scope (in/out)**:
  - In: claim-boundary wording in `manual_testing_playbook.md`, `platform-parity-spec.md`, and checkpoint-facing evidence summaries
  - Out: new runtime behavior, new helper bundle assets, or changing the upstream cleanup/discovery contracts themselves
- **Acceptance criteria**:
  - macOS wording is explicit that scope is helper discovery, validation, and managed cleanup only
  - Windows wording remains compile parity only
  - checkpoint and playbook wording names the landed upstream contract surfaces `C-01`..`C-04`
- **Dependencies**:
  - `review.md`
  - `../../threading.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - pass condition: evidence surfaces can be reviewed against one stable claim boundary without reopening platform support scope
- **Rollout/safety**:
  - refuse overclaimed platform support
  - treat wording drift as a blocker, not a documentation cleanup later
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
