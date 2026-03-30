---
seam_id: SEAM-4
seam_slug: provisioning-routing-pacman-execution
type: platform
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
  - governance/seam-1-closeout.md
  - governance/seam-2-closeout.md
  - governance/seam-3-closeout.md
  required_threads:
  - THR-01
  - THR-02
  - THR-03
  stale_triggers:
  - C-02 changes probe/support-gate outcomes
  - C-03 changes pacman schema semantics or derived requirement assumptions
  - shared-file changes in world_enable or world-agent invalidate the provisioning
    execution basis
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---

# SEAM-4 - Provisioning routing and pacman execution

This seam is now active and exec-ready. Its authoritative pre-exec planning lives in `threaded-seams/seam-4-provisioning-routing-pacman-execution/`.

- **Goal / value**:
  - Turn the manager-aware contract, probe support gate, and pacman schema truth into one deterministic provisioning-time execution path for `substrate world enable --provision-deps`.
- **Scope**
  - In:
    - deriving normalized APT and pacman requirement sets from the effective enabled world-deps set
    - pacman requirement de-duplication and stable ordering
    - mixed-manager fail-closed behavior with no partial provisioning
    - request-profile routing boundary that keeps `world-deps-provision` internal
    - exact pacman command shape, no-op detection, and dry-run/verbose rendering
    - package-manager command failure posture for the provisioning path
  - Out:
    - defining probe precedence or supported-family mapping
    - defining pacman inventory schema and view surfaces
    - runtime read-only probe behavior and explicit-item scope
    - smoke/manual validation and cross-doc reconciliation landing
- **Primary interfaces**
  - Inputs:
    - `C-01` from `SEAM-1`
    - `C-02` from `SEAM-2`
    - `C-03` from `SEAM-3`
    - accepted pacman execution decision DR-0003
  - Outputs:
    - `C-04` provisioning execution contract
    - provisioning evidence consumed by runtime remediation and parity validation
- **Key invariants / rules**:
  - provisioning executes only the manager that matches the detected world manager
  - if both normalized manager sets are non-empty, provisioning exits `4` before any OS package-manager command runs
  - pacman execution is exactly `pacman -Sy --noconfirm --needed <packages...>` in normalized order
  - no AUR helpers, no retries, no lock-file intervention, and no fallback from pacman to apt or apt to pacman
  - `SUBSTRATE_WORLD_REQUEST_PROFILE` is not an operator control surface for this seam
  - dry-run performs no mutation while still showing the detected world manager and normalized requirement sets
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` publishing `C-01`
    - `SEAM-2` publishing `C-02`
    - `SEAM-3` publishing `C-03`
  - Transitive blockers:
    - adjacent shared-file work in `world_enable` and `world-agent` can stale the execution basis before decomposition
  - Direct consumers:
    - `SEAM-5`
    - `SEAM-6`
  - Derived consumers:
    - provisioning operators
    - request-profile and world-agent maintainers
- **Touch surface**:
  - Primary planning surface:
    - `slices/NASP2/NASP2-spec.md`
  - Likely downstream code surfaces once seam-local planning begins:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `crates/world-agent/src/service.rs`
    - `crates/shell/tests/world_enable.rs`
- **Verification**:
  - Because this seam **consumes** upstream contracts, verification may depend on accepted upstream evidence for `C-02` and `C-03`.
  - The first seam-local review should try to falsify:
    - whether any mixed-manager path can still partially provision
    - whether request-profile routing can still be influenced by host environment instead of the in-world probe
    - whether pacman no-op detection or dry-run rendering can become unstable across layers
  - A passing pre-exec posture should leave runtime remediation and validation seams able to consume one exact provisioning story.
- **Risks / unknowns**:
  - Risk:
    - shared-file overlap with staging/tracing work can stale `world_enable` and `world-agent` assumptions before this seam decomposes.
  - De-risk plan:
    - keep `REM-003` open until seam-local revalidation proves the shared touch surface is still current.
  - Risk:
    - older APT-only assumptions can hide in dry-run output, request-profile handling, or helper-script behavior.
  - De-risk plan:
    - make those falsification checks explicit in the first seam-local review.
- **Rollout / safety**:
  - This seam is the only place in the pack where pacman execution becomes an owned behavior. That makes fail-closed behavior and exact command shape load-bearing safety controls.
  - Linux host-native and Windows remain unsupported; no host mutation is allowed.
- **Downstream decomposition context**:
  - This seam is `active` because `SEAM-3` has now published `THR-03`, and the shared `world_enable` / `world-agent` touch surfaces were revalidated against the current repo state.
  - The most important threads are `THR-01`, `THR-02`, `THR-03`, and `THR-04`.
  - The seam-local review now fixes the owner contract baseline, request-profile boundary, shared-file touch surface, and exact pacman command construction before execution begins.
  - Source-plan lineage: `NASP-PWS-provisioning_wiring` and `NASP2`.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-04`
  - Threads likely to advance:
    - `THR-04` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the provisioning branch in the workflow diagram
    - the service/data-flow edge between dispatch/world-agent and package-manager execution
  - Downstream seams most likely to require revalidation:
    - `SEAM-5`
    - `SEAM-6`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
