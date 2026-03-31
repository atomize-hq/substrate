---
pack_id: dev-install-world-agent-staging-seam-pack
pack_version: v1
pack_status: extracted
source_ref: dev-install-world-agent-staging.zip
execution_horizon:
  active_seam: SEAM-2
  next_seam: SEAM-3
---

# Scope Brief - dev-install-world-agent-staging

- **Goal**: Make the Linux “dev install with `--no-world`, enable later” workflow execution-ready by aligning dev-install staging with the standard version-dir runtime contract, adding one deterministic missing-artifact failure before privileged work, and preserving a checkpointable validation story across Linux, macOS, and Windows parity surfaces.
- **Why now**: The source pack identifies a sharp operator failure mode: Linux dev install can leave `substrate world enable` without a staged `world-agent`, which turns an “enable later” workflow into manual build recovery or a late provisioning failure. The contract also sits on shared `world enable` and dev-install surfaces that overlap other queued work, so the boundary needs explicit seam-level control before execution.
- **Primary user(s) + JTBD**: Linux developers and operators who run `scripts/substrate/dev-install-substrate.sh --no-world` and later expect `substrate world enable` to work without manual artifact staging, plus maintainers who need a narrow, checkpointable feature boundary with deterministic exit codes and parity evidence.
- **In-scope**:
  - standard version-dir preflight for `substrate world enable` when `SUBSTRATE_WORLD_ENABLE_SCRIPT` is unset
  - the accepted staged path set and search order: `<version_dir>/bin/world-agent`, then `<version_dir>/bin/linux/world-agent`
  - the sufficiency rule that either accepted staged executable path is enough to continue
  - deterministic exit `3` missing-artifact behavior with one remediation block that names both accepted paths, `scripts/substrate/dev-install-substrate.sh --no-world`, and `cargo build -p world-agent`
  - `--dry-run` and non-dry-run ordering, including no writes before success and `world.enabled` staying `false` until helper execution and health verification succeed
  - Linux `dev-install-substrate.sh --no-world` staging of `world-agent` into both accepted staged paths from the selected `--profile`, with `ln -sfn` refresh semantics
  - Linux-only behavior validation, plus macOS and Windows compile-parity / no-change evidence at the checkpoint boundary
- **Out-of-scope**:
  - enabling supported `substrate world enable` behavior on Windows beyond the existing unsupported exit `4`
  - widening macOS behavior promises beyond parity validation for the touched paths
  - building `world-agent` inside `substrate world enable`
  - changing `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior beyond keeping the carve-out explicit
  - new protocol, telemetry, trace-span, policy, or config-format surfaces
  - broad refactors in shared installer helpers outside the missing-artifact preflight and Linux staging boundary
  - solving the broader `cargo clean` helper-discovery brittleness owned by the adjacent helper-discovery work
- **Success criteria**:
  - `substrate world enable` in the standard version-dir flow resolves `<home>/bin/substrate`, derives the standard version dir, checks the accepted staged path set in fixed order, and exits `3` before helper launch or privileged work when neither path exists
  - the missing-artifact failure renders one operator-facing remediation block with the minimum required content in dry-run and non-dry-run flows
  - `--dry-run` shares the same preflight, exits `0` only when an accepted artifact exists, and writes no config, helper log, manager-env export, or systemd state
  - on the non-dry-run success path, `world.enabled` remains `false` until helper execution and health verification both succeed
  - Linux `scripts/substrate/dev-install-substrate.sh --no-world --profile <debug|release>` stages both accepted `world-agent` bridges to `target/<profile>/world-agent`, keeps `world.enabled: false`, skips provisioning, and refreshes stale links with `ln -sfn`
  - Linux feature smoke, installer smoke, manual playbook cases, and compile parity for `linux`, `macos`, and `windows` remain aligned to the landed contract and checkpoint boundary
- **Constraints**:
  - the source pack keeps the critical implementation touch set intentionally narrow: `crates/shell/src/builtins/world_enable/runner.rs`, `crates/shell/tests/world_enable.rs`, `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/world-enable.sh`, and `tests/installers/install_smoke.sh`
  - `substrate world enable --home` remains the authoritative state root; `--prefix` stays invalid on that command
  - `SUBSTRATE_WORLD_ENABLE_SCRIPT` remains an explicit override carve-out and is outside the standard version-dir preflight guarantee
  - the feature must remain orthogonal to future dependency-provisioning flag work on `world enable`
  - no `cargo build` invocation may run under `sudo`
  - the source pack’s accepted execution order remains `DIWAS0`, then `DIWAS1`, with a single checkpoint boundary after `DIWAS1`
- **External systems / dependencies**:
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/tests/world_enable.rs`
  - `scripts/substrate/dev-install-substrate.sh`
  - `scripts/substrate/world-enable.sh`
  - `tests/installers/install_smoke.sh`
  - `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`
  - shared exit-code taxonomy and the existing unsupported-platform posture
  - overlapping planning packs and ADRs on helper discovery, provisioning, and `world enable` flag growth
- **Known unknowns / risks**:
  - default helper output suppression means the missing-artifact remediation can disappear from the visible CLI path if implementation drifts back into the helper instead of the runner boundary
  - the source pack contains a scope ambiguity about whether `scripts/substrate/install-substrate.sh` is truly touched or only remains a production-reference posture guarded by installer smoke
  - selected-profile staging can look inconsistent with the helper’s default `--profile release` log label if the contract is not stated precisely during seam-local review
  - the accepted staged path rule and no-write ordering can stale quickly if overlapping helper-discovery or provisioning work lands first on shared `world enable` surfaces
  - this feature must not overclaim robustness against `cargo clean`; that broader fix belongs to adjacent helper-discovery work
- **Assumptions**:
  - the seam axis is workflow-first because the source pack already converged on a two-step user flow: make runtime enable fail deterministically, then make dev-install satisfy that contract
  - `SEAM-1` and `SEAM-2` are inferred as active and next from the source pack’s accepted slice order (`DIWAS0` then `DIWAS1`) and the single checkpoint after `DIWAS1`
  - the attached planning pack is treated as the authoritative basis for extraction; external ADRs referenced by the pack were not separately inspected here
  - seam-exit concerns are inferred from the source contract, slice specs, validation surfaces, and cross-queue scan rather than from landed code or existing closeout evidence
