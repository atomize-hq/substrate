---
slice_id: S2
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - runtime scope calculation changes for sync or explicit-item install
  - read-only probe handling changes for `dpkg-query` or `pacman -Q`
  - remediation wording drifts from the accepted `C-05` contract
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-03
  - THR-04
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-01
  - C-03
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S2 - Implement runtime fail-early and remediation

- **User/system value**:
  - Runtime operators get deterministic read-only fail-early handling for APT-backed and pacman-backed requirements, with exact remediation that points back to provisioning.
- **Scope (in/out)**:
  - In:
    - derive normalized runtime APT and pacman requirement sets from the in-scope item set
    - perform read-only `dpkg-query` and `pacman -Q` probes only
    - preserve explicit-item scope for `deps current install <ITEM...>`
    - render manager-aware missing requirements and backend-specific remediation
    - keep dry-run and verbose behavior stable for runtime fail-early paths
  - Out:
    - provisioning-time probe/support gate
    - pacman schema definition and inventory rendering
    - provisioning-time pacman mutation and mixed-manager execution behavior
    - smoke/manual validation and shared-doc reconciliation landing
- **Acceptance criteria**:
  - Runtime never mutates `apt`, `apt-get`, `dpkg`, or `pacman`.
  - Explicit-item installs are scoped only to the expanded requested items.
  - Unsatisfied APT-backed or pacman-backed requirements exit `4` before non-system-package mutation.
  - Remediation includes the exact provisioning command and backend-specific fail-closed guidance.
- **Dependencies**:
  - `SEAM-1` publishes `C-01`.
  - `SEAM-3` publishes `C-03`.
  - `SEAM-4` publishes `C-04`.
- **Verification**:
  - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
  - `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
  - `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- **Rollout/safety**:
  - Keep runtime read-only and fail-closed; do not widen into runtime mutation or provisioning fallback.
- **Review surface refs**:
  - `../../review_surfaces.md#R1`
  - `../../review_surfaces.md#R2`
  - `../../review_surfaces.md#R3`

#### S2.T1 - Enforce runtime read-only probes and explicit-item scope

- **Outcome**:
  - Runtime derives the correct in-scope APT and pacman requirement sets and probes them read-only without widening scope or mutating the world.
- **Inputs/outputs**:
  - Inputs: effective enabled set or explicit expanded install set, `C-03`, `C-04`
  - Outputs: concrete `C-05` behavior for runtime fail-early scope and probe posture
- **Implementation notes**:
  - Keep the main execution surface centered on `crates/shell/src/builtins/world_deps/surfaces.rs`.
- **Acceptance criteria**:
  - No runtime path calls a mutating package-manager command.

#### S2.T2 - Render deterministic missing-requirement remediation

- **Outcome**:
  - Runtime emits stable, manager-aware missing-requirement rendering and exact remediation wording for supported and fail-closed backend cases.
- **Inputs/outputs**:
  - Inputs: normalized runtime requirement sets and read-only probe results
  - Outputs: deterministic runtime stderr/stdout plus exact provisioning remediation
- **Implementation notes**:
  - Keep the exact provisioning command and backend guidance aligned with `C-05`.
- **Acceptance criteria**:
  - Rendering remains stable across APT-backed and pacman-backed requirements and always points back to `substrate world enable --provision-deps`.
