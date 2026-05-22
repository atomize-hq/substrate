---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - accepted path, remediation, or selected-profile staging rules change upstream
    - platform parity wording drifts beyond the bounded Linux / macOS / Windows posture
    - checkpoint or manual evidence starts claiming more than the upstream closeouts support
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
  - THR-02
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S1 - Revalidate consumed contracts and freeze evidence boundaries

- **User/system value**: Every downstream proof surface starts from one closeout-backed contract baseline instead of ad hoc smoke or checkpoint assumptions.
- **Scope (in/out)**:
  - In:
    - revalidation of `THR-01`, `THR-02`, and `THR-03` against `SEAM-1` / `SEAM-2` closeouts
    - explicit evidence-to-contract mapping for smoke, manual, parity, and checkpoint artifacts
    - explicit platform-claim boundaries for Linux, macOS, and Windows
  - Out:
    - landing the smoke, manual, or checkpoint edits themselves
    - changing upstream runtime or staging behavior
- **Acceptance criteria**:
  - every evidence surface is mapped to one or more consumed contracts without ambiguity
  - Linux remains the only behavior-delta platform in the evidence boundary
  - macOS and Windows posture is explicit, narrow, and non-promissory
  - the overlap watchpoint from `REM-003` is recorded as a revalidation concern rather than silently ignored

#### Evidence-to-contract map

- `smoke/linux-smoke.sh` proves the landed Linux runtime and staging behavior consumed from `C-01`, `C-02`, `C-03`, and `C-04`.
- `tests/installers/install_smoke.sh` proves the dev-install staging boundary and guards against production-installer drift while consuming `C-04`.
- `manual_testing_playbook.md` carries operator-facing confirmation for path resolution, remediation behavior, `readlink` outputs, and success-path ordering.
- `platform-parity-spec.md`, checkpoint planning, and quality-gate artifacts must inherit the same bounded platform claims rather than widening them.

Checklist:
- Implement: N/A (contract / evidence-boundary slice)
- Test: N/A (contract / evidence-boundary slice)
- Validate: confirm all downstream evidence surfaces bind to closeout-backed upstream truth
- Cleanup: none
