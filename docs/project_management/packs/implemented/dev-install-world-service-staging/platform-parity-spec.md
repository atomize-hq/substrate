# dev-install-world-service-staging — platform parity spec

This spec is authoritative for platform scope and validation evidence for the source pack at `docs/project_management/packs/draft/dev-install-world-service-staging/`. The extracted `-fse` seam docs remain planning inputs only.

## Scope
- Linux carries the only behavior delta in this feature.
- macOS and Windows remain CI parity surfaces for the touched Rust and shell code paths.
- `REM-003` is a revalidation watchpoint, not a blocker, unless upstream overlap changes the current closeout-backed basis.

## Required platforms
- Behavior platforms: `linux`
- CI parity platforms: `linux`, `macos`, `windows`
- WSL required: `false`

## Cross-platform guarantees
- The accepted staged path set stays `bin/world-service`, then `bin/linux/world-service` wherever the touched code compiles, and the symlinks resolve to absolute `${REPO_ROOT}/target/<profile>/world-service` targets.
- `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior stays unchanged on every platform.
- Windows keeps the existing unsupported posture for `substrate world enable`.
- The pack adds no new protocol, telemetry, or config-format surface on any platform.

## Platform matrix

### Linux
- Behavior delta:
  - `dev-install-substrate.sh --no-world` stages `world-service` for the enable-later workflow.
  - `substrate world enable` gains the standard missing-artifact preflight and exit `3` remediation.
- Required evidence:
  - `smoke/linux-smoke.sh`
  - Linux sections of `manual_testing_playbook.md`
  - `tests/installers/install_smoke.sh` as dev-install staging / `C-04` regression evidence only
  - checkpoint compile parity and quick CI testing

### macOS
- Behavior delta:
  - none
- Required evidence:
  - compile parity at `CP1-ci-checkpoint`
  - local parity fixes through `DIWAS1-integ-macos` if the checkpoint reports a macOS regression
- No-change guarantee:
  - the feature does not widen the macOS enable contract or the Lima-specific workflow

### Windows
- Behavior delta:
  - none
- Required evidence:
  - compile parity at `CP1-ci-checkpoint`
  - local parity fixes through `DIWAS1-integ-windows` if the checkpoint reports a Windows regression
- No-change guarantee:
  - `substrate world enable` remains unsupported with exit `4`

## Validation evidence
- Smoke scripts required:
  - `docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh`
- Installer smoke required:
  - `tests/installers/install_smoke.sh` for dev-install staging regression coverage only
- Manual playbook required:
  - `docs/project_management/packs/draft/dev-install-world-service-staging/manual_testing_playbook.md`
- Checkpoint CI required:
  - compile parity on `linux`, `macos`, and `windows`
  - feature smoke on `linux`
  - quick CI testing at `CP1`

## Acceptance criteria
- Linux feature smoke proves the staged-path rule, the missing-artifact failure, and the `world.enabled` ordering invariant.
- Installer smoke proves dev-install staging and `C-04` regression coverage without widening the Linux behavior delta.
- macOS compile parity proves the touched code remains buildable and testable without a macOS behavior delta.
- Windows compile parity proves the touched code remains buildable and preserves the unsupported enable posture.
