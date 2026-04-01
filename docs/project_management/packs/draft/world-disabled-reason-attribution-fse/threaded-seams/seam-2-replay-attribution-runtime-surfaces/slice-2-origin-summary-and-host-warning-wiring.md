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
    - replay origin-summary fragments drift from the contract baseline
    - recorded-host punctuation changes
    - host-warning output rephrases or duplicates the effective-disable winner differently
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Origin summary and host-warning wiring

- **User/system value**: replay stderr explains effective host fallback with the same exact fragment in both the origin summary and the host warning, so operators can tell why replay stayed on host without reading telemetry.
- **Scope (in/out)**:
  - In: wire effective-disable attribution into origin summaries and host warnings, including the exact `host (recorded; <reason>)` format.
  - Out: telemetry field emission and docs lock-in (handled in S3 and later seams).
- **Acceptance criteria**:
  - Effective-disable origin summaries use the exact fragments frozen in S1.
  - Recorded-host output prints `host (recorded; <reason>)` exactly.
  - Host warnings use the same effective-disable fragment as the origin summary.
  - Replay-local opt-out fragments remain unchanged.
  - No additional replay lines are introduced.
- **Dependencies**:
  - `slice-1-contract-definition-replay-attribution-runtime-surfaces.md`
  - `crates/shell/src/execution/routing/replay.rs`
  - published `SEAM-1` attribution contract in `../../governance/seam-1-closeout.md`
- **Verification**:
  - Extend `crates/shell/tests/replay_world.rs` for env override, workspace config, global config, source-unknown, recorded-host, and replay-local opt-out cases.
  - Assert exact origin-summary and host-warning lines, not substring approximations.
- **Rollout/safety**:
  - Preserve replay selection behavior and all replay-local opt-out wording.
  - Treat any need for a new runtime fragment as a blocker, not as opportunistic copy cleanup.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### S2.T1 - Reuse the shared disable-attribution contract in replay copy

- **Outcome**: replay stderr consumes one shared attribution decision for effective-disable cases instead of formatting a separate precedence story locally.
- **Inputs/outputs**:
  - Inputs: selected replay origin, published `SEAM-1` attribution result, and replay-local toggle reasons.
  - Outputs: exact `[replay] origin: ...` and `[replay] warn: running on host (...)` lines.
- **Thread/contract refs**: `THR-03`; `C-01`, `C-02`, `C-03`
- **Implementation notes**:
  - Keep replay-local toggles (`--world`, `--no-world`, `SUBSTRATE_REPLAY_USE_WORLD`, `--flip-world`, recorded-origin cases) on their existing fragments.
  - Effective-disable cases must consume the shared attribution contract rather than reconstructing precedence or redaction rules from raw config inputs.
  - Recorded-host formatting is part of the contract; avoid “equivalent” punctuation.
- **Acceptance criteria**: origin summary and host warning always agree on the effective-disable fragment.
