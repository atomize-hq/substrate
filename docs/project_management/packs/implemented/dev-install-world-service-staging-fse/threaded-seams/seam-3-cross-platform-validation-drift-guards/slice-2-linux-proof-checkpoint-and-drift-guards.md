---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - Linux smoke, installer smoke, manual cases, or checkpoint wording drift from the revalidated contract map
    - macOS or Windows parity artifacts overclaim supported behavior
    - shared runner or dev-install surfaces change after evidence is drafted
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
### S2 - Land Linux proof, checkpoint alignment, and drift guards

- **User/system value**: The feature becomes checkpointable and resistant to evidence drift because smoke, manual, parity, and checkpoint artifacts all point at the same landed behavior.
- **Scope (in/out)**:
  - In:
    - align `smoke/linux-smoke.sh` and installer smoke to the revalidated contract boundary
    - update `manual_testing_playbook.md`, `platform-parity-spec.md`, and checkpoint artifacts to match the landed Linux-only delta
    - record stale-trigger and follow-up guardrails for future helper-discovery, provisioning, or platform-claim drift
  - Out:
    - new behavior changes in `world enable` or dev-install
    - widened macOS provisioning or Windows runtime support
- **Acceptance criteria**:
  - Linux smoke proves the staged-path rule, missing-artifact failure, remediation content, and `world.enabled` ordering without drifting from upstream closeouts
  - installer smoke remains aligned to the dev-install staging contract and shows no unintended production-installer drift
  - manual playbook, parity spec, checkpoint plan, session log, and quality gate text stay bounded to the landed platform claims
  - stale-trigger records are explicit anywhere future overlap could invalidate the evidence basis

#### S2.T1 - Align Linux proof surfaces

- **Outcome**: smoke and manual evidence surfaces all point at the same revalidated runtime + staging contract.
- **Acceptance criteria**: Linux smoke, installer smoke, and manual cases agree on paths, exit codes, `readlink` expectations, and state-ordering behavior.

#### S2.T2 - Lock checkpoint and parity wording

- **Outcome**: checkpoint and platform-parity artifacts report only the platform claims that the pack actually owns.
- **Acceptance criteria**: macOS and Windows remain parity-only / unsupported where required, and checkpoint artifacts do not overclaim behavior outside the landed contract.

Checklist:
- Implement:
- Test:
- Validate:
- Cleanup:
