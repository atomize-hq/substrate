---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to workspace detection for overrides"
    - "Any change to exported env naming or value shape"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Publish override and parity env surfaces for the host gate

- **User/system value**: operators and tests can override or inspect the host gate consistently without inventing ad hoc env behavior.
- **Scope (in/out)**:
  - In:
    - Add no-workspace-only override env `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1|0|true|false|yes|no|on|off`.
    - Export resolved parity state as `SUBSTRATE_WORLD_NET_FILTER=1|0`.
  - Out:
    - config schema/CLI patching (`S1`)
    - user-facing docs/examples (`S3`)
- **Acceptance criteria**:
  - The override env is ignored when a workspace exists, matching current override behavior.
  - The override env applies when no workspace exists and changes the effective config.
  - `SUBSTRATE_WORLD_NET_FILTER=1|0` is emitted alongside the other derived `SUBSTRATE_*` exports.
  - Exported parity reflects the resolved effective value, not raw input.
- **Dependencies**:
  - `S1` publishes `C-04`
  - `../../threading.md` (`C-05`, `C-06`, `THR-03`)
- **Verification**:
  - Add tests in `crates/shell/tests/ev0_override_split.rs` for no-workspace-only override behavior.
  - Add export parity assertions in `crates/shell/tests/world_enable.rs` or `crates/shell/src/builtins/world_enable/runner/manager_env.rs`.
- **Review surface refs**:
  - `review.md` (override leakage, parity drift)

Checklist:
- Implement: env override parse + export write
- Test: workspace-vs-no-workspace behavior
- Validate: exported value matches effective config
- Cleanup: avoid treating exported parity as an override input
