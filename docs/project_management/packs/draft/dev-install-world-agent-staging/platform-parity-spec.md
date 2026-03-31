# dev-install-world-agent-staging — platform parity spec

This spec is authoritative for platform scope and validation evidence for the source pack at `docs/project_management/packs/draft/dev-install-world-agent-staging/`. The extracted `-fse` seam docs remain planning inputs only.

## Scope
- Linux carries the only behavior delta in this feature.
- macOS and Windows remain CI parity surfaces for the touched Rust and shell code paths.
- `REM-003` is a revalidation watchpoint, not a blocker, unless upstream overlap changes the current closeout-backed basis.

## Required platforms
- Behavior platforms: `linux`
- CI parity platforms: `linux`, `macos`, `windows`
- WSL required: `false`

## Cross-platform guarantees
- The accepted staged path set stays `bin/world-agent`, then `bin/linux/world-agent` wherever the touched code compiles.
- `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior stays unchanged on every platform.
- Windows keeps the existing unsupported posture for `substrate world enable`.
- The pack adds no new protocol, telemetry, or config-format surface on any platform.

## Platform matrix

### Linux
- Behavior delta:
  - `dev-install-substrate.sh --no-world` stages `world-agent` for the enable-later workflow.
  - `substrate world enable` gains the standard missing-artifact preflight and exit `3` remediation.
- Required evidence:
  - `smoke/linux-smoke.sh`
  - Linux sections of `manual_testing_playbook.md`
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
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh`
- Manual playbook required:
  - `docs/project_management/packs/draft/dev-install-world-agent-staging/manual_testing_playbook.md`
- Checkpoint CI required:
  - compile parity on `linux`, `macos`, and `windows`
  - feature smoke on `linux`
  - quick CI testing at `CP1`

## Acceptance criteria
- Linux feature smoke proves the staged-path rule, the missing-artifact failure, and the `world.enabled` ordering invariant.
- macOS compile parity proves the touched code remains buildable and testable without a macOS behavior delta.
- Windows compile parity proves the touched code remains buildable and preserves the unsupported enable posture.
