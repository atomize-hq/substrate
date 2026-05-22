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
  stale_triggers:
    - "Any change to runtime failure taxonomy or WORLD_NETFILTER_ENABLE wording"
    - "Any platform-specific doctor rendering drift"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-04
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-07
open_remediations: []
candidate_subslices: []
---
### S2 - Thread runtime failure state into doctor output and shell renderers

- **User/system value**: the same runtime failure taxonomy operators see during failed isolated runs also appears in doctor output, so the system explains why enforcement is unavailable before the next run.
- **Scope (in/out)**:
  - In:
    - surface the actionable `last_failure_reason` from runtime/doctor state into the world doctor response
    - preserve the new netfilter block through shell-side world doctor JSON handling on Linux, macOS, and Windows
    - pin the resulting contract with focused doctor tests
  - Out:
    - broad privileged smoke coverage (`SEAM-5`)
    - changing the upstream failure taxonomy itself (`SEAM-2`)
- **Acceptance criteria**:
  - the world doctor payload carries a stable `last_failure_reason` when a prior requested-isolation attempt failed for a known `SEAM-2` cause
  - shell-side `substrate world doctor --json` keeps the block intact across supported platforms
  - focused tests cover both success and failure-shaped doctor payloads
- **Dependencies**:
  - published runtime failure taxonomy in `../../governance/seam-2-closeout.md`
  - doctor contract from `S1`
- **Verification**:
  - platform doctor JSON tests / fixtures in `crates/shell/tests`
  - world-service handler coverage for failure-state serialization
- **Rollout/safety**:
  - preserve existing doctor readiness output while adding actionable netfilter detail
- **Review surface refs**:
  - `review.md`

#### S2.T1 - Preserve runtime failure taxonomy in world doctor payloads

- **Outcome**: doctor output surfaces actionable failure reasons from the landed fail-closed runtime instead of generic "not enabled" messaging.
- **Files**:
  - `crates/world-service/src/handlers.rs`
  - runtime error/state surfaces consumed by the handler
- **Thread/contract refs**:
  - `THR-04`
  - `C-07`
- **Acceptance criteria**:
  - missing env guard, nft install failure, resolution failure, and cgroup attach failure remain distinguishable in the surfaced reason
  - absence of prior failure state serializes cleanly as null/none rather than bogus text
- **Test notes**:
  - add handler coverage for representative failure states and the no-failure case

Checklist:
- Implement: failure-state serialization
- Test: handler coverage for the four actionable failure classes
- Validate: doctor output stays actionable without inventing new runtime truth

#### S2.T2 - Keep shell-side world doctor output aligned across platforms

- **Outcome**: Linux, macOS, and Windows world doctor JSON all preserve the same additive netfilter block for downstream tests and operators.
- **Files**:
  - `crates/shell/src/execution/platform/linux.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/execution/platform/windows.rs`
  - focused doctor tests in `crates/shell/tests`
- **Thread/contract refs**:
  - `THR-05`
  - `C-07`
- **Acceptance criteria**:
  - platform adapters do not drop or rename the new netfilter block
  - tests pin the additive JSON contract in at least one shell-side doctor path
  - downstream `SEAM-5` smoke work can reference one stable doctor JSON shape
- **Test notes**:
  - refresh or add JSON assertions in existing doctor fixture tests

Checklist:
- Implement: platform doctor passthrough/rendering updates
- Test: shell doctor fixture coverage
- Validate: downstream conformance can treat the doctor block as one contract
