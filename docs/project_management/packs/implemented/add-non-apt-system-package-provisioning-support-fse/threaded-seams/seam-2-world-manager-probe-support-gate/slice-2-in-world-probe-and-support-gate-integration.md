---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - upstream THR-01 publication changes “in-world only” posture or probe inputs
  - world_enable helper-script staging changes relocate probe execution out of the world
  - world-service execution plumbing changes alter exit-code translation or stderr/stdout capture
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
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Implement in-world probe execution + support-gate wiring

- **User/system value**:
  - Provisioning and dry-run flows reach one deterministic probe + gate decision inside the world, so downstream routing can be manager-aware without host-derived drift.
- **Scope (in/out)**:
  - In:
    - execute the `C-02` probe inside the world backend
    - wire the gate outcome into the `world enable --provision-deps` flow so downstream routing consumes one stable decision
    - ensure all unsupported lanes fail closed with exit `4` and consistent diagnostics
  - Out:
    - requirement normalization and mixed-manager rejection (`SEAM-4`)
    - any pacman execution command wiring (`SEAM-4`)
    - runtime read-only probe work (`SEAM-5`)
- **Acceptance criteria**:
  - Probe runs in-world (never on host) for both real and dry-run paths.
  - Output is deterministic given identical `/etc/os-release` + pacman presence.
  - Unsupported lanes (Linux host-native, Windows WSL, unmapped/ambiguous/contradiction) exit `4` before any mutating OS package-manager command is attempted.
  - World-backend-unavailable lane exits `3` (per `C-01` / taxonomy).
- **Dependencies**:
  - `SEAM-1` publishes `C-01` and advances `THR-01` to `published` (revalidation prerequisite).
- **Verification**:
  - Unit tests for `/etc/os-release` parsing and mapping (fixtures for Debian-family, Arch-family, ambiguous, unmapped, and unreadable).
  - Integration tests for `world_enable` that assert:
    - probe executes in-world
    - exit codes and stdout/stderr diagnostics are stable across supported/unsupported lanes
- **Rollout/safety**:
  - Fail-closed only; do not broaden supported provisioning surfaces in this seam beyond the explicit `C-02` posture.
- **Review surface refs**:
  - `../../review_surfaces.md#R1`
  - `../../review_surfaces.md#R2`
  - `../../review_surfaces.md#R4`

#### S2.T1 - Implement `C-02` probe runner (in-world)

- **Outcome**:
  - One in-world probe runner returns normalized family + pacman presence (or an unsupported/contradiction classification).
- **Inputs/outputs**:
  - Inputs: `/etc/os-release` (`ID`, `ID_LIKE`), `command -v pacman`
  - Outputs: `apt | pacman | unsupported` + a deterministic reason string + exit code
- **Thread/contract refs**:
  - Consumes `C-01` (no host routing, fail closed)
  - Produces `C-02` behavior
- **Implementation notes**:
  - Prefer keeping parsing + mapping logic in Rust with unit tests; keep shell scripting as a thin adapter only when required by the `world_enable` helper mechanism.
- **Acceptance criteria**:
  - All `C-02` mapping and contradiction rules are implemented exactly.
- **Test notes**:
  - Add fixtures that exercise `ID_LIKE` tokenization and quoting edge cases.
- **Risk/rollback notes**:
  - Keep diagnostics stable; downstream seams will quote exit-code posture and reason labels.

Checklist:
- Implement:
- Test:
- Validate:
- Cleanup:

#### S2.T2 - Wire probe + gate into `world_enable` and dispatch plumbing

- **Outcome**:
  - `substrate world enable --provision-deps` (and dry-run) reaches the in-world probe and converts it into a stable routing input.
- **Inputs/outputs**:
  - Inputs: existing world backend selection and `world-service` execute/stream plumbing
  - Outputs: manager decision becomes a single routing input for downstream provisioning (`SEAM-4`)
- **Thread/contract refs**:
  - Inbound: `THR-01` / `C-01`
  - Outbound: `THR-02` / `C-02`
- **Implementation notes**:
  - Revalidate the touch surface:
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `crates/world-service/src/service.rs`
- **Acceptance criteria**:
  - Probe executes in-world and never uses host-derived routing inputs.
  - Unsupported lanes exit `4` before any mutating command path is reached.
- **Test notes**:
  - Add an integration test that asserts the probe runs even in dry-run mode while still doing no mutation.
- **Risk/rollback notes**:
  - Keep changes orthogonal to adjacent staging/tracing work; emit revalidation notes if boundaries moved.

Checklist:
- Implement:
- Test:
- Validate:
- Cleanup:
