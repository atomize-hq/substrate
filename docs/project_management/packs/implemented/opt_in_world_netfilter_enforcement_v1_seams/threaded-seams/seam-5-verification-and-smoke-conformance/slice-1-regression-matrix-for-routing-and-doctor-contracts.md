---
slice_id: S1
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation or world_network routing semantics"
    - "Any change to doctor JSON field naming or shell/shim passthrough"
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
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
  - C-05
  - C-06
  - C-07
open_remediations: []
candidate_subslices: []
---
### S1 - Regression matrix for routing and doctor contracts

- **User/system value**: the core config, routing, and doctor handoffs become one regression boundary instead of several focused tests that can drift independently.
- **Scope (in/out)**:
  - In:
    - extend regression coverage for allow-all, deny-all, and restrictive `net_allowed` postures under `world.net.filter`
    - pin world doctor and shim doctor preservation of the published `C-07` block
    - make the expected relationship between requested routing and doctor observability explicit in tests
  - Out:
    - privileged nftables install/apply validation
    - macOS Lima smoke workflow changes
- **Acceptance criteria**:
  - `crates/shell/tests/world_request_net_allowed_snapshot.rs` covers allow-all, deny-all, and restrictive routing postures against the effective host gate.
  - `crates/shell/tests/doctor_scopes_ds0.rs` and `crates/shell/tests/shim_doctor.rs` pin the published `requested`, `enabled`, `world_netfilter_enable_present`, and `last_failure_reason` fields without platform-specific drift.
  - `crates/agent-api-types/src/lib.rs` continues to prove additive schema compatibility for the doctor block while the conformance matrix references the same published field names.
- **Dependencies**:
  - landed closeouts `../../governance/seam-1-closeout.md`, `../../governance/seam-3-closeout.md`, and `../../governance/seam-4-closeout.md`
- **Verification**:
  - `cargo test -p agent-api-types -- --nocapture`
  - `cargo test -p shell --test world_request_net_allowed_snapshot -- --nocapture`
  - `cargo test -p shell --test doctor_scopes_ds0 -- --nocapture`
  - `cargo test -p shell --test shim_doctor -- --nocapture`
- **Rollout/safety**:
  - additive tests only; no contract ownership changes
- **Review surface refs**:
  - `review.md`

#### S1.T1 - Extend the routing regression matrix for allow-all, deny-all, and restrictive policy postures

- **Outcome**: host-side request construction and canonicalized `net_allowed` behavior cannot drift without one regression suite failing.
- **Files**:
  - `crates/shell/tests/world_request_net_allowed_snapshot.rs`
  - `docs/reference/config/world.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `C-01`
  - `C-02`
  - `C-03`
  - `C-04`
- **Acceptance criteria**:
  - the test matrix distinguishes allow-all (`["*"]`) from restrictive allowlists and deny-all (`[]`)
  - the matrix makes the host-gate dependency on `world.net.filter` explicit
- **Test notes**:
  - update or add focused assertions rather than broad fixture rewrites

Checklist:
- Implement: routing/conformance assertions
- Test: targeted shell test reruns
- Validate: docs still match the tested allow-all versus deny-all semantics

#### S1.T2 - Pin doctor and shim preservation of the published netfilter status block

- **Outcome**: operators and downstream tooling keep seeing one stable observability contract when requested isolation is in play.
- **Files**:
  - `crates/agent-api-types/src/lib.rs`
  - `crates/shell/tests/doctor_scopes_ds0.rs`
  - `crates/shell/tests/shim_doctor.rs`
- **Thread/contract refs**:
  - `THR-05`
  - `C-07`
- **Acceptance criteria**:
  - the published doctor field names remain additive and explicitly asserted in shell/shim surfaces
  - the conformance suite covers both enabled and disabled/failure-reporting permutations
- **Test notes**:
  - keep assertions aligned with the exact field names already published in `SEAM-4`

Checklist:
- Implement: doctor/shim conformance assertions
- Test: targeted schema and shell doctor suites
- Validate: requested/enabled/failure reason semantics stay aligned across surfaces
