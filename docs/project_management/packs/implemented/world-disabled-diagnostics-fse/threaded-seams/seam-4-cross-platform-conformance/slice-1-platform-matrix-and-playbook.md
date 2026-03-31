---
slice_id: S1
seam_id: SEAM-4
slice_kind: delivery
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
  - THR-02
  - THR-03
  - THR-04
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S1 - Establish the platform proof matrix and operator-playbook assertions

- **User/system value**: the conformance seam starts from one explicit Linux/macOS/Windows proof matrix instead of ad hoc smoke runs, so downstream reviewers can tell exactly which contract each platform assertion covers.
- **Scope (in/out)**:
  - In:
    - map disabled mode, enabled-but-broken mode, and invalid-config fail-fast behavior to concrete Linux/macOS/Windows proof surfaces
    - pin which repo-native scripts or commands prove `C-02` through `C-05`
    - define the evidence bundle shape that `S2` and `S3` must produce
  - Out:
    - net-new runtime behavior
    - speculative platform automation that is not already grounded in repo-native smoke or doctor workflows
- **Acceptance criteria**:
  - the matrix names Linux, macOS, and Windows proof paths for disabled, enabled-but-broken, and invalid-config behavior
  - each proof path is tied back to the landed `C-02` through `C-05` contracts
  - the matrix names the concrete repo-native surfaces to exercise: targeted `shim_doctor` / `shim_health` regressions, `scripts/mac/smoke.sh`, `scripts/windows/wsl-smoke.ps1`, and Linux doctor/health commands
  - the matrix calls out shared-file overlap that must be revalidated before closeout
- **Dependencies**:
  - `../../governance/seam-2-closeout.md`
  - `../../governance/seam-3-closeout.md`
  - `crates/shell/tests/shim_doctor.rs`
  - `crates/shell/tests/shim_health.rs`
  - `scripts/mac/smoke.sh`
  - `scripts/windows/wsl-smoke.ps1`
- **Verification**:
  - the proof matrix must be executable using the named repo-native test and smoke surfaces
  - the planned assertions must include exact disabled copy and omission behavior, not just generic success/failure
- **Rollout/safety**: keep the seam evidence-first; if a platform surface cannot prove the contract, that gap becomes an explicit blocker rather than an implicit assumption.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### S1.T1 - Define the cross-platform proof matrix

- **Outcome**: `S2` can execute platform-native evidence collection without guessing which contract, command, or script each platform must prove.
- **Inputs/outputs**: seam closeouts from `SEAM-2` / `SEAM-3`, repo-native tests/scripts, and a deterministic platform-by-platform evidence checklist.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`, `THR-05`, `C-02`, `C-03`, `C-04`, `C-05`
- **Acceptance criteria**: the matrix explicitly distinguishes disabled-mode truth, enabled-but-broken fail visibility, and invalid-config exit-2 behavior.

Checklist:
- Implement: matrix and evidence checklist updates in seam-local planning/closeout prep artifacts
- Test: targeted `shim_doctor` / `shim_health` regressions remain the Linux proof anchor
- Validate: every planned platform assertion maps to a published upstream contract
- Cleanup: none
