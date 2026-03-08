# world-deps-apt-provisioning — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- The accepted execution slice order is `WDAP0` then `WDAP1`, and `tasks.json` is already wired to this order.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["WDAP0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "High-risk provisioning seam: introduce `world enable --provision-deps` and provisioning-time guard rails (no host OS mutation; request profile posture). Run compile parity + behavior smoke + quick CI testing before proceeding to runtime fail-early."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WDAP1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "End-to-end contract completion seam: runtime `world deps current sync|install` fail-early + deterministic remediation + operator-doc/contract reconciliation. Run full cross-platform validation before treating the feature as complete."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`WDAP0`) — provisioning surface + isolation/guard-rail seam

- Code-grounded boundary (high-risk mutation + subsystem seam):
  - `WDAP0` introduces a new operator-facing provisioning surface and ties together multiple subsystems (`shell` CLI + world-op routing + `world-agent` execution posture + platform provisioning scripts).
  - This is a safety-sensitive seam (“no host OS mutation” on Linux host-native) with platform/backends branching that is high risk for cross-platform drift.
- Surfaces stabilized at this checkpoint (from `spec_manifest.md` + `minimal_spec_draft.md` + `impact_map.md` touch set):
  - Provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` (guest backends only)
  - Provisioning-time execution posture (request `profile` semantics) and guard rails that prevent host-native Linux mutation
  - Primary code surfaces:
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `crates/world-agent/src/service.rs`
  - Platform provisioning integration points (planned):
    - `scripts/linux/world-provision.sh`
    - `scripts/mac/lima-warm.sh`
    - `scripts/windows/wsl-warm.ps1`
- Risk reduced by running cross-platform CI here (from `impact_map.md`):
  - Catch “builds on primary dev OS but not on other OSes” breakage early (compile parity).
  - Validate that provisioning behavior is guest-only and that Linux host-native fails closed with deterministic exit/posture (feature smoke).
  - Provide quick, broad regression signal on shared scripts and world-enable orchestration changes (quick CI testing).

### CP2 (`WDAP1`) — runtime fail-early + remediation + operator-doc seam

- Code-grounded boundary (contract completion seam):
  - By the end of `WDAP1`, the operator experience becomes coherent end-to-end:
    - provisioning installs APT/system packages via the explicit `world enable --provision-deps` workflow, and
    - runtime `world deps current sync|install` never executes APT/dpkg and emits deterministic remediation pointing back to provisioning.
- Surfaces stabilized at this checkpoint (from `spec_manifest.md` ownership + `impact_map.md`):
  - Runtime invariant and remediation text/exit behavior:
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/shell/src/builtins/world_deps/errors.rs`
    - `crates/shell/tests/world_deps_apt_install_wdp5.rs` (repurposed to assert “no runtime apt” + remediation)
  - Operator-doc updates and contradiction resolution targets (exact paths/headings enumerated in `minimal_spec_draft.md`):
    - `docs/reference/world/deps/README.md`
    - `docs/internals/world/deps.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `docs/CONFIGURATION.md`, `docs/WORLD.md`, `docs/COMMANDS.md`
- Risk reduced by running cross-platform CI here (from `impact_map.md`):
  - Ensures the runtime fail-early behavior is consistent across platforms/backends and doesn’t regress provisioning safety guard rails.
  - Ensures cross-document contract updates remain coherent and don’t reintroduce runtime APT semantics.
  - Full CI testing provides broader regression coverage because this feature touches shared scripts + world-agent behavior + shell builtins.

## Current execution-ready alignment

- `tasks.json` `meta.checkpoint_boundaries` is `["WDAP0", "WDAP1"]`.
- `CP1-ci-checkpoint` depends on `WDAP0-integ-core`.
- `WDAP1-code` and `WDAP1-test` depend on `WDAP0-integ` and `CP1-ci-checkpoint`, so checkpoint gating is explicit.
- `CP2-ci-checkpoint` depends on `WDAP1-integ-core`.
