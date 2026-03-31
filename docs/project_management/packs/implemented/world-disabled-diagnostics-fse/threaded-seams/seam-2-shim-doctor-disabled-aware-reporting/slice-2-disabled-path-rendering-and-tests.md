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
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Implement disabled shim-doctor path, rendering, and regression tests

- **User/system value**: `substrate shim doctor` becomes truthful for host-only installs by skipping disabled-mode probes and publishing exact contracts that downstream health and conformance work can consume.
- **Scope (in/out)**:
  - In:
    - gate world/world-deps collection on `effective_world_enabled`
    - short-circuit disabled-mode probes
    - emit the `C-02` / `C-03` JSON statuses and omission rules
    - render the exact `C-04` text lines
    - add regression tests proving the no-probe boundary and exact disabled-mode outputs
  - Out:
    - health summary behavior and docs alignment
    - cross-platform smoke orchestration beyond seam-local proof
- **Acceptance criteria**:
  - Disabled mode does not launch `substrate world doctor --json`.
  - Disabled mode does not compute world-deps applied state.
  - JSON surfaces publish the `disabled` / `skipped_disabled` statuses and omit forbidden legacy fields.
  - Human output prints the exact disabled contract lines and suppresses `Error:` in disabled mode.
  - Enabled-mode error visibility is preserved.
- **Dependencies**:
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `crates/shell/src/builtins/shim_doctor/output.rs`
  - `crates/shell/tests/shim_doctor.rs`
  - published `C-01` helper path from `SEAM-1`
- **Verification**:
  - `cargo test -p shell --test shim_doctor -- --nocapture`
  - targeted fixtures that would otherwise fail probes if the disabled branch were not active
  - optional shell-level repros showing disabled-mode JSON/text behavior
- **Rollout/safety**: preserve enabled-mode failure visibility and additive-only JSON.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### S2.T1 - Gate report building on the published classifier

- **Outcome**: disabled mode uses the published `C-01` helper and branches before any world or world-deps probes run.
- **Inputs/outputs**: `effective_world_enabled` from `SEAM-1` drives one disabled path and one enabled path in `shim_doctor/report.rs`.
- **Thread/contract refs**: `THR-01`, `THR-02`, `THR-03`
- **Acceptance criteria**:
  - disabled mode does not invoke the world-doctor subprocess
  - disabled mode does not compute applied world-deps state
  - enabled mode continues to gather the existing probe-backed data

Checklist:
- Implement: disabled short-circuit in `report.rs`
- Test: probe-negative and enabled-positive assertions in `shim_doctor.rs`
- Validate: prove the branch keys off `effective_world_enabled`, not legacy heuristics
- Cleanup: remove any leftover disabled-path probe fallback

#### S2.T2 - Publish disabled JSON/text behavior and exact-line tests

- **Outcome**: `shim doctor` exposes the owned status and exact-copy contracts with regression coverage.
- **Inputs/outputs**:
  - Inputs: disabled and enabled fixtures
  - Outputs: canonical status enums, omission rules, and exact text lines
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`, `C-02`, `C-03`, `C-04`
- **Acceptance criteria**:
  - disabled JSON asserts `.world.status = "disabled"` and `.world_deps.status = "skipped_disabled"`
  - disabled JSON omits forbidden legacy fields
  - disabled text asserts the exact `C-04` lines and absence of `Error:`
  - enabled-mode fixtures still show fail-visible behavior when probes fail

Checklist:
- Implement: output/rendering updates in `output.rs` and any report-shape updates in `report.rs`
- Test: exact-line and omission assertions in `shim_doctor.rs`
- Validate: compare emitted fields/lines against `S1`
- Cleanup: none
