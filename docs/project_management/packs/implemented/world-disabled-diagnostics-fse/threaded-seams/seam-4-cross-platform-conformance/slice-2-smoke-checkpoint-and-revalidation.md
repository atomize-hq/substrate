---
slice_id: S2
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
### S2 - Execute smoke/checkpoint evidence and shared-file revalidation

- **User/system value**: conformance evidence becomes closeout-ready instead of remaining a plan-only matrix, while shared-file overlap is revalidated before future packs assume parity exists.
- **Scope (in/out)**:
  - In:
    - run the targeted Linux regression anchors in `shim_doctor.rs` and `shim_health.rs`
    - execute macOS and Windows smoke flows with explicit disabled, enabled-but-broken, and invalid-config expectations
    - capture the Linux doctor/health proof commands needed to mirror the non-Linux assertions
    - revalidate shared-file overlap in `health.rs`, `shim_doctor/report.rs`, `shim_doctor/output.rs`, and `docs/USAGE.md`
  - Out:
    - new runtime fixes for discovered regressions
    - unrelated platform provisioning changes beyond what the existing smoke and doctor flows already own
- **Acceptance criteria**:
  - evidence exists for Linux, macOS, and Windows disabled-mode truth, enabled-but-broken fail visibility, and invalid-config fail-fast behavior
  - the evidence bundle traces each platform run back to `C-02` through `C-05`
  - any shared-file drift that would invalidate the proof bundle is either captured in stale triggers or raised as an explicit remediation
  - the resulting evidence is concrete enough that `S3` can update `governance/seam-4-closeout.md` without ambiguity
- **Dependencies**:
  - `crates/shell/tests/shim_doctor.rs`
  - `crates/shell/tests/shim_health.rs`
  - `scripts/mac/smoke.sh`
  - `scripts/windows/wsl-smoke.ps1`
  - Linux doctor and health workflows available through the repo toolchain
- **Verification**:
  - `cargo test -p shell --test shim_doctor -- --nocapture`
  - `cargo test -p shell --test shim_health -- --nocapture`
  - platform-native smoke or doctor commands on Linux, macOS, and Windows
  - explicit comparison of the collected evidence against the `C-02` through `C-05` contracts
- **Rollout/safety**: if any platform fails to prove the published contract, stop at a remediation rather than normalizing partial evidence into closeout truth.
- **Review surface refs**: `../../review_surfaces.md#r2---cli--service--data-flow`, `../../review_surfaces.md#r3---touch-surface-map`

#### S2.T1 - Capture conformance evidence and revalidate shared diagnostics surfaces

- **Outcome**: the seam produces one evidence bundle that is strong enough for closeout and for future diagnostics packs to inherit explicit stale triggers.
- **Inputs/outputs**: planned matrix from `S1`, repo-native tests/scripts, and a closeout-ready evidence record plus stale-trigger annotations.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`, `THR-05`, `C-02`, `C-03`, `C-04`, `C-05`
- **Acceptance criteria**:
  - Linux, macOS, and Windows evidence is explicit and replayable
  - the shared-file revalidation statement names `health.rs`, `shim_doctor/report.rs`, `shim_doctor/output.rs`, and `docs/USAGE.md`
  - any blocker discovered here becomes a concrete remediation instead of an implicit note

Checklist:
- Implement: evidence capture and stale-trigger updates in seam-local or governance closeout artifacts
- Test: targeted regression runs plus platform-native smoke/doctor commands
- Validate: compare collected outputs against the published disabled status, omission, and exact-copy contracts
- Cleanup: none
