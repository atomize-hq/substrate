---
slice_id: S1
seam_id: SEAM-05
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - exit taxonomy changes
    - wrapper handling changes
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
  - THR-05
contracts_produced:
  - C-08
contracts_consumed:
  - C-06
  - C-07
open_remediations: []
candidate_subslices: []
---
### S1 - Wrapper exit pass-through

- **User/system value**: wrapper invocations preserve feature-specific failure classes so operators see the same contract-backed exit posture as the direct installer.
- **Scope (in/out)**:
  - In: `scripts/substrate/install.sh` pass-through for exits `0`, `2`, `3`, and `4`
  - Out: doc wording propagation, validation harness work, and checkpoint publication
- **Acceptance criteria**:
  - wrapper preserves direct installer exits `0`, `2`, `3`, and `4`
  - wrapper does not collapse feature-specific non-zero exits to `1`
  - wrapper behavior introduces no second exit taxonomy
- **Dependencies**:
  - `../../seam-05-wrapper-doc-propagation.md`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../seam-04-fallback-probe-failure-taxonomy/seam.md`
- **Verification**:
  - wrapper path proves exit-class parity for the feature-specific branches

