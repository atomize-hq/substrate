---
seam_id: SEAM-4
seam_slug: provisioning-routing-pacman-execution
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-4-provisioning-routing-pacman-execution.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
  - ../../governance/seam-1-closeout.md
  - ../../governance/seam-2-closeout.md
  - ../../governance/seam-3-closeout.md
  required_threads:
  - THR-01
  - THR-02
  - THR-03
  stale_triggers:
  - C-02 changes probe/support-gate outcomes
  - C-03 changes pacman schema semantics or derived requirement assumptions
  - shared-file changes in world_enable or world-service invalidate the provisioning execution basis
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

## Seam Brief (Restated)

- **Goal / value**:
  - Turn the manager-aware contract, probe support gate, and pacman schema truth into one deterministic provisioning-time execution path for `substrate world enable --provision-deps`.
- **Type**: platform
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
- **Touch surface**:
  - `crates/shell/src/builtins/world_enable/runner/provision_deps.rs`
  - `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/world-service/src/service.rs`
  - `scripts/substrate/world-enable.sh`
  - `crates/shell/tests/world_enable_provision_deps_wdap0.rs`
- **Verification**:
  - Pre-exec verification for this producing seam must prove the provisioning contract is concrete in seam-local planning without relying on post-exec publication of `C-04`.
  - Review should try to falsify mixed-manager rejection, request-profile isolation, exact pacman command construction, and stable dry-run/verbose rendering.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
    - `../../governance/seam-3-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - Stale triggers:
    - `C-02` changes probe/support-gate outcomes
    - `C-03` changes pacman schema semantics or derived requirement assumptions
    - shared-file changes in `world_enable` or `world-service` invalidate the provisioning execution basis
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1`, `SEAM-2`, and `SEAM-3` have all published the inbound threads this seam consumes
  - Downstream blocked seams:
    - `SEAM-5`
    - `SEAM-6`
  - Contracts produced:
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `C-04` / `THR-04` is the promotion input for runtime fail-early handling and the final validation/reconciliation seam. The seam-exit gate makes provisioning routing and pacman execution a closeout-backed fact instead of an implicit reading of the implementation.
- **Expected contracts to publish**:
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-04`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - Any change to normalized requirement derivation, mixed-manager rejection, request-profile routing, pacman command shape, or dry-run/verbose rendering
- **Expected closeout evidence**:
  - Published `C-04` contract artifact location(s)
  - Provisioning routing evidence across supported and fail-closed paths
  - Thread-state update record for `THR-04`
  - Recorded review-surface deltas for any shared-file drift discovered during landing

## Slice index

- `S1` -> `slice-1-c-04-provisioning-contract-definition.md`
- `S2` -> `slice-2-provisioning-routing-and-pacman-execution.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
