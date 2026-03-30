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
  - shared-file changes in world_enable or world-agent invalidate the provisioning execution basis
  - request-profile routing changes widen operator control or host-environment influence
  - dry-run or verbose rendering drifts from the normalized requirement contract
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
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Implement provisioning routing and pacman execution

- **User/system value**:
  - Operators get one deterministic provisioning path for apt-backed or pacman-backed prerequisites, with fail-closed mixed-manager handling and stable dry-run output.
- **Scope (in/out)**:
  - In:
    - derive normalized APT and pacman requirement sets from the effective enabled world-deps set
    - reject mixed-manager enabled sets before mutation
    - preserve the internal `world-deps-provision` request-profile boundary
    - execute the exact pacman command shape in normalized order
    - keep dry-run, verbose, and no-op rendering stable across runner, dispatch, world-agent, and helper layers
  - Out:
    - redefining the probe/support-gate contract
    - redefining pacman inventory schema or view rendering
    - runtime read-only probe and remediation behavior
    - smoke/manual validation and shared-doc reconciliation landing
- **Acceptance criteria**:
  - Mixed-manager paths exit `4` before any package-manager command runs.
  - Request-profile routing remains internal and cannot be redirected by `SUBSTRATE_WORLD_REQUEST_PROFILE`.
  - Pacman execution uses `pacman -Sy --noconfirm --needed <packages...>` in normalized order only.
  - Dry-run and verbose rendering stay stable and manager-aware.
  - Linux host-native and Windows remain fail-closed with no host mutation.
- **Dependencies**:
  - `SEAM-2` publishes `C-02` and advances `THR-02`.
  - `SEAM-3` publishes `C-03` and advances `THR-03`.
- **Verification**:
  - `crates/shell/tests/world_enable_provision_deps_wdap0.rs`
  - provisioning tests covering mixed-manager rejection, pacman execution shape, no-op detection, and dry-run / verbose rendering
- **Rollout/safety**:
  - Keep fail-closed behavior load-bearing; do not widen into fallback, retries, or host mutation.
- **Review surface refs**:
  - `../../review_surfaces.md#R1`
  - `../../review_surfaces.md#R2`
  - `../../review_surfaces.md#R3`

#### S2.T1 - Route provisioning through one manager-specific execution path

- **Outcome**:
  - The provisioning path derives normalized requirement sets, rejects mixed-manager scope before mutation, and routes through one manager-specific execution path only.
- **Inputs/outputs**:
  - Inputs: effective enabled world-deps set, `C-02`, `C-03`
  - Outputs: concrete `C-04` behavior for routing and request-profile handling
- **Implementation notes**:
  - Keep the shared touch surface centered on `provision_deps.rs`, `dispatch/world_ops.rs`, `world-agent/src/service.rs`, and `scripts/substrate/world-enable.sh`.
- **Acceptance criteria**:
  - No partial provisioning and no host-environment routing override.

#### S2.T2 - Execute pacman deterministically and render dry-run/verbose output

- **Outcome**:
  - Pacman provisioning uses the exact command shape, normalized order, and stable operator-visible rendering across dry-run and verbose paths.
- **Inputs/outputs**:
  - Inputs: normalized pacman requirement set, internal provisioning profile
  - Outputs: deterministic pacman execution and rendering behavior
- **Implementation notes**:
  - Keep the exact pacman command shape and log/output surfaces aligned with `C-04`.
- **Acceptance criteria**:
  - No AUR helpers, retries, lock-file intervention, or fallback paths.
