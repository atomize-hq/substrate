---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the live Azure route or redaction posture changes in a way that invalidates the planned procedure or troubleshooting outputs
    - bounded diagnostics are insufficient to distinguish auth, URL, deployment, and route failures through the real gateway path
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-06
  - THR-07
contracts_produced:
  - C-08
contracts_consumed:
  - C-07
  - C-03
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver The Live Smoke Procedure And Troubleshooting Surface

- **User/system value**: a real operator can now configure and validate Azure-hosted Kimi through the landed gateway surface instead of debugging transport details ad hoc.
- **Scope (in/out)**:
  - In: land the bounded operator docs, any minimal runtime diagnostics, and the redacted evidence surface needed to make failures understandable.
  - Out: redefining `C-07`, redesigning router/public contracts, or building a general observability platform.
- **Acceptance criteria**:
  - operator-facing surfaces explain how to configure Azure credentials and deployment identifiers for the think/default routes without consulting code
  - the live smoke path runs through `/v1/messages` for both `Kimi-K2-Thinking` and `Kimi-K2.5`
  - troubleshooting surfaces distinguish auth, URL, deployment, and route failures with redacted evidence
  - any added diagnostics remain bounded and do not leak secrets or internal identities
- **Dependencies**: `S1`, `gateway/README.md`, `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, `gateway/src/cli/mod.rs`, `gateway/src/server/mod.rs`, and any bounded smoke harness or operator note introduced for Azure verification
- **Verification**:
  - pass condition: an operator can follow the documented smoke steps and interpret failures without reading provider code
  - redacted live evidence expectations remain explicit and capability-oriented
- **Rollout/safety**: do not require public identity changes or non-redacted evidence.

#### S2.T1 - Land Operator-Facing Smoke Guidance

- **Outcome**: README or adjacent operator notes describe the live smoke path for both Azure Kimi routes.
- **Thread/contract refs**: `THR-07`, `C-08`, `THR-06`, `C-07`

#### S2.T2 - Add Bounded Diagnostics Or Failure Surfacing

- **Outcome**: the runtime exposes enough redacted signals to separate auth, URL, deployment, and route failures during smoke verification.
- **Thread/contract refs**: `THR-07`, `C-08`, `C-07`

#### S2.T3 - Capture Redacted Evidence Expectations

- **Outcome**: the seam names what success evidence and failure evidence must be captured for closeout.
- **Thread/contract refs**: `THR-07`, `C-08`, `C-05`
