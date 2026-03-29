---
slice_id: S2
seam_id: SEAM-07
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - macOS Lima-backed behavior-evidence expectations change
    - compile parity or CI quick requirements change
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-06
contracts_produced:
  - C-11
contracts_consumed:
  - C-10
open_remediations: []
candidate_subslices: []
---
### S2 - macOS-hosted behavior evidence

- **User/system value**: macOS checkpoint evidence proves real Lima-backed Linux installer behavior instead of stopping at parity-only signals.
- **Scope (in/out)**:
  - In: the macOS-hosted behavior-evidence record that depends on the Lima-backed Linux smoke path
  - Out: new macOS-native package-manager-selection logic or non-checkpoint validation work
- **Acceptance criteria**:
  - hosted evidence names the Lima-backed Linux path explicitly
  - hosted evidence stays tied to the checkpoint boundary
  - no artifact implies native macOS package-manager-selection behavior
- **Dependencies**:
  - `S1`
  - `../../governance/seam-06-closeout.md`
- **Verification**:
  - review proves hosted evidence is behavior coverage, not compile-only parity
