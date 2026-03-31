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
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-05
open_remediations: []
candidate_subslices: []
---
### S2 - Implement disabled health summary, docs alignment, and regression tests

- **User/system value**: `substrate health` becomes trustworthy for host-only installs by treating disabled/skipped as non-error, suppressing enabled-world guidance when disabled, and aligning docs/examples to the landed contract.
- **Scope (in/out)**:
  - In:
    - drive health summary derivation from the published shim status enums
    - emit disabled summary null/omission rules
    - render the exact disabled `C-05` text lines
    - suppress enabled-world world-deps guidance when disabled
    - add regression tests and docs updates proving the landed behavior
  - Out:
    - shim-doctor runtime internals
    - cross-platform smoke orchestration beyond seam-local proof
- **Acceptance criteria**:
  - Disabled summary JSON publishes `summary.world_ok = null`, omits summary error fields, and keeps world-deps arrays empty.
  - Disabled text prints the exact `C-05` lines and does not print enabled-world `substrate world deps current` guidance.
  - Enabled-mode failure visibility and remediation guidance remain intact.
  - `docs/USAGE.md` aligns with the landed status-enum contract and disabled summary behavior.
- **Dependencies**:
  - `crates/shell/src/builtins/health.rs`
  - `crates/shell/tests/shim_health.rs`
  - `docs/USAGE.md`
  - published `C-01`, `C-02`, and `C-03` from upstream seams
- **Verification**:
  - `cargo test -p shell --test shim_health -- --nocapture`
  - targeted fixtures proving disabled summary behavior and enabled-mode preservation
  - optional shell-level repros showing disabled health text/JSON behavior
- **Rollout/safety**: preserve enabled-mode failure visibility while removing false-negative disabled-mode attention signals.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### S2.T1 - Update summary derivation to consume the landed shim status contracts

- **Outcome**: disabled health summary behavior derives from `.shim.world.status` / `.shim.world_deps.status` instead of legacy error surfaces.
- **Inputs/outputs**: embedded shim payload from `shim_doctor::collect_report` drives one disabled path and one enabled path in `health.rs`.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-05`, `C-05`
- **Acceptance criteria**:
  - disabled mode emits `summary.world_ok = null`
  - disabled mode omits summary error fields and keeps world-deps arrays empty
  - enabled mode continues to gather fail-visible summary data

Checklist:
- Implement: disabled-aware summary branch in `health.rs`
- Test: disabled/forced-enabled assertions in `shim_health.rs`
- Validate: prove summary logic keys off status enums before legacy `ok` / `error` fields
- Cleanup: remove leftover disabled-path summary fallbacks

#### S2.T2 - Publish disabled health text/docs behavior and regression tests

- **Outcome**: `substrate health` exposes the owned summary/copy contract with regression coverage and docs alignment.
- **Inputs/outputs**:
  - Inputs: disabled and enabled fixtures plus landed upstream shim status semantics
  - Outputs: canonical summary JSON posture, exact text lines, guidance suppression, and aligned docs/examples
- **Thread/contract refs**: `THR-05`, `C-05`
- **Acceptance criteria**:
  - disabled text asserts the exact `C-05` lines
  - disabled text omits `substrate world deps current` guidance
  - enabled-mode fixtures still show attention-required behavior and guidance when appropriate
  - `docs/USAGE.md` matches the landed status-enum and summary contract

Checklist:
- Implement: rendering/docs updates in `health.rs` and `docs/USAGE.md`
- Test: exact-line, suppression, and JSON assertions in `shim_health.rs`
- Validate: compare emitted fields/lines/docs against `S1`
- Cleanup: none
