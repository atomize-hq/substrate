---
slice_id: S3
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - `origin_reason_code` values drift from the telemetry spec
    - `world_disable_source` emit/omit rules drift between effective-disable and replay-local opt-out cases
    - telemetry fields expose raw env values, raw paths, or extra keys
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S3 - replay_strategy telemetry and omission rules

- **User/system value**: trace consumers get one additive, machine-readable replay attribution contract that stays aligned with stderr copy and keeps secrets redacted.
- **Scope (in/out)**:
  - In: extend `replay_strategy` with the additive `origin_reason_code` values and optional `world_disable_source`, and enforce omission for replay-local opt-out cases.
  - Out: docs lock-in, smoke wrappers, and any non-additive trace schema change.
- **Acceptance criteria**:
  - `origin_reason_code` uses the exact `world_disabled_*` values from S1 for effective-disable attribution only.
  - `world_disable_source` is emitted only for effective-disable attribution and omitted for replay-local opt-out cases.
  - `world_disable_source` uses only tokenized path displays and safe env tokens.
  - Existing `replay_strategy` fields remain stable.
- **Dependencies**:
  - `slice-1-contract-definition-replay-attribution-runtime-surfaces.md`
  - `crates/replay/src/replay/executor.rs`
  - `crates/shell/tests/replay_world.rs`
- **Verification**:
  - Extend `crates/shell/tests/replay_world.rs` or adjacent replay assertions to pin `origin_reason`, `origin_reason_code`, and `world_disable_source`.
  - Cover env override, workspace patch, global patch, source-unknown, and replay-local opt-out omission cases.
- **Rollout/safety**:
  - Additive only; consumers that ignore unknown fields keep working.
  - Any need for a new enum or extra object key is a blocker until the contract baseline is explicitly revised.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### S3.T1 - Emit telemetry from the same attribution decision as stderr

- **Outcome**: replay stderr and `replay_strategy` stay aligned because they both consume one effective-disable attribution decision.
- **Inputs/outputs**:
  - Inputs: replay attribution decision from S2, published telemetry spec, and existing `replay_strategy` emission flow.
  - Outputs: exact `origin_reason`, additive `origin_reason_code`, and optional `world_disable_source`.
- **Thread/contract refs**: `THR-04`; `C-01`, `C-02`, `C-04`
- **Implementation notes**:
  - Do not parse or re-derive telemetry fields from human text after formatting.
  - `world_disable_source` must be omitted for replay-local opt-out cases even when `origin_reason` is present.
  - Keep `world_disable_source.layer` inside the published enum set `override_env | workspace_patch | global_patch | unknown`.
- **Acceptance criteria**: every emitted effective-disable telemetry object is additive, redacted, and omitted in replay-local opt-out cases.
