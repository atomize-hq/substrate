---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - smoke assertion drift
    - manual playbook drift
    - checkpoint evidence drift
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
### S2 - Refresh cross-platform proof surfaces

- **User/system value**: operators and maintainers get one closeout-backed proof set that demonstrates Linux and macOS behavior claims plus Windows compile parity without overclaiming unsupported flows.
- **Scope (in/out)**:
  - In: `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`, and the paired manual/checkpoint evidence surfaces
  - Out: upstream cleanup or helper-discovery behavior changes, new platform enablement, or bundle-surface expansion
- **Acceptance criteria**:
  - Linux and macOS smoke assertions track the landed upstream contracts
  - Windows smoke remains compile parity only
  - manual and checkpoint evidence reference the same closeout-backed truth as smoke output
- **Dependencies**:
  - `slice-1-freeze-platform-evidence-boundaries.md`
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - pass condition: smoke and manual proof surfaces can be planned and reviewed against one current upstream truth set
- **Rollout/safety**:
  - preserve deterministic skip behavior on non-target hosts
  - record stale triggers instead of silently reusing outdated evidence
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
