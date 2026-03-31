# dev-install-world-agent-staging — contract surface

This file is the authoritative contract for the Linux enable-later workflow that starts with `scripts/substrate/dev-install-substrate.sh --no-world` and ends with `substrate world enable` in the standard dev-install flow.

Decision inputs:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md`
- `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md`

External standards:
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Authority handoff
- This file is authoritative for the user-visible contract owned by this feature.
- `scripts/substrate/install-substrate.sh` remains authoritative for production bundle install behavior outside the accepted `world-agent` path rule stated here.
- `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior stays unchanged. This feature owns the standard version-dir flow only.

## CLI

### Commands in scope
- Linux dev install:
  - `scripts/substrate/dev-install-substrate.sh [--prefix <path>] [--profile <debug|release>] [--version-label <name>] [--no-world]`
- Linux runtime enable:
  - `substrate world enable [--home <path>] [--profile <name>] [--dry-run] [--verbose] [--force] [--timeout <seconds>]`
- Linux helper flow:
  - `scripts/substrate/world-enable.sh [--home <path>] [--profile <name>] [--dry-run] [--verbose] [--force] [--no-sync-deps]`
- Linux production installer reference:
  - `scripts/substrate/install-substrate.sh [--no-world]`

### Definitions
- **Standard version-dir flow**: `substrate world enable` with `SUBSTRATE_WORLD_ENABLE_SCRIPT` unset.
- **Substrate home**: the path resolved by `--home`, then `SUBSTRATE_HOME`, then `~/.substrate`.
- **Standard version dir**: the directory that contains the canonical `bin/` directory for the active `substrate` binary in the standard version-dir flow.
  - Operationally: `standard_version_dir = dirname(dirname(realpath(<home>/bin/substrate)))`.
- **Accepted staged path set**:
  1. `<version_dir>/bin/world-agent`
  2. `<version_dir>/bin/linux/world-agent`
- **Selected dev-install profile**: the `--profile` value passed to `scripts/substrate/dev-install-substrate.sh`. Valid values are `debug` and `release`. The script default is `debug`.

### Linux `scripts/substrate/dev-install-substrate.sh --no-world`
- The script builds the selected profile and stages the `world-agent` bridge from `target/<profile>/world-agent` into both accepted staged paths.
- The script keeps `world.enabled: false` in `$SUBSTRATE_HOME/config.yaml`.
- The script skips Linux world provisioning and leaves the later provisioning step to `substrate world enable`.
- Re-running the script refreshes both staged world-agent links with `ln -sfn`.
- The selected dev-install profile controls the staged bridge target. The `substrate world enable --profile` flag does not change the bridge target.

### Linux `substrate world enable`
- Substrate home resolves in this order:
  1. `--home <path>`
  2. `SUBSTRATE_HOME`
  3. `~/.substrate`
- In the standard version-dir flow, the command resolves `<home>/bin/substrate`, then derives the standard version dir from that canonical binary target.
- In the standard version-dir flow, the command checks the accepted staged path set before helper launch.
- Either accepted staged executable path is sufficient. The command does not require both paths to exist to continue.
- If neither accepted path exists, the command exits `3` before helper launch, privileged provisioning, health verification, config writes, or manager-env writes.
- The missing-artifact failure prints one remediation block that names both accepted paths, `scripts/substrate/dev-install-substrate.sh --no-world`, and `cargo build -p world-agent`.
- `--dry-run` runs the same missing-artifact preflight. When the artifact is present, dry-run exits `0` and writes no config, helper log, manager-env export, or systemd state. When the artifact is missing, dry-run exits `3`.
- On the non-dry-run success path, `world.enabled` stays `false` until helper execution and health verification both succeed.

### Linux `scripts/substrate/world-enable.sh`
- The helper resolves `--home` before `SUBSTRATE_HOME`, then falls back to `~/.substrate`.
- The helper keeps `--profile release` as its default log label.
- This feature does not move the missing-artifact contract into the helper. The standard missing-artifact preflight runs before helper launch.

### Helper override carve-out
- When `SUBSTRATE_WORLD_ENABLE_SCRIPT` is set, helper discovery follows the override path exactly.
- The override path is outside the standard version-dir preflight guarantee owned by this feature.
- Tests or operators that use the override path remain responsible for the helper’s artifact expectations.

## Config and environment

### Config file
- Path: `$SUBSTRATE_HOME/config.yaml`
- Home resolution order:
  1. `substrate world enable --home <path>`
  2. `SUBSTRATE_HOME`
  3. `~/.substrate`
- Missing config file:
  - `substrate world enable` continues and creates the file after a successful non-dry-run enable.
- Invalid config file:
  - `substrate world enable` prints a warning, replaces the invalid file after a successful non-dry-run enable, and keeps `world.enabled: false` until enable success is verified.

### `world.enabled`
- Type: boolean.
- `scripts/substrate/dev-install-substrate.sh --no-world` writes `world.enabled: false`.
- `substrate world enable --dry-run` writes no config.
- `substrate world enable` writes `world.enabled: true` only after helper execution and health verification succeed.
- If `world.enabled` is absent, the enable workflow treats the persisted state as not yet enabled until the success path saves the config.

### Environment variables in scope
- `SUBSTRATE_HOME`
  - type: path
  - role: home override for config, logs, scripts, and helper wiring
  - precedence: lower than `--home`
- `SUBSTRATE_WORLD_ENABLE_SCRIPT`
  - type: path
  - role: helper override for advanced and test-only flows
  - precedence: when set, helper discovery uses this path directly
- `SUBSTRATE_WORLD_SOCKET`
  - type: path
  - role: readiness-socket override for the health verification step
  - feature posture: unchanged by this pack

## Filesystem and path semantics
- The accepted staged path set for the standard version-dir flow is:
  1. `<version_dir>/bin/world-agent`
  2. `<version_dir>/bin/linux/world-agent`
- The search order is fixed. The root `bin/world-agent` path wins when both paths exist.
- Linux dev-install uses the selected profile output under `target/<profile>/world-agent` as the source for both staged bridge links.
- The production installer keeps its current bundle-install posture. This pack only aligns the accepted path set and search order.

## Exit codes
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `0`
  - Linux dev-install completed successfully.
  - Linux `substrate world enable --dry-run` found an accepted staged artifact and completed the dry-run plan.
  - Linux `substrate world enable` completed helper execution and health verification.
- `2`
  - Existing CLI usage and unrecoverable config or path errors keep the inherited taxonomy meaning.
- `3`
  - In the standard version-dir flow, neither accepted staged world-agent path exists.
  - This exit code applies to dry-run and non-dry-run.
- `4`
  - `substrate world enable` remains unsupported on Windows.

## Platform guarantees

### Linux
- This feature changes Linux behavior.
- Linux `dev-install-substrate.sh --no-world` stages `world-agent` for the enable-later workflow.
- Linux `substrate world enable` gains the standard missing-artifact preflight and deterministic exit `3` remediation.

### macOS
- This feature adds no macOS behavior promise.
- macOS remains a CI parity surface for the touched code paths.
- The pack does not change the current helper-override or Lima-specific behavior.

### Windows
- `substrate world enable` remains unsupported.
- The operator-visible unsupported posture and exit code `4` remain unchanged.
- Windows remains a CI parity surface for the touched Rust code paths.

## Protected paths and invariants
- The standard missing-artifact preflight runs before helper launch and before privileged provisioning.
- `world.enabled` never flips to `true` before helper execution and health verification succeed.
- The feature does not add a new protocol, telemetry field, policy rule, or config file format.
- The feature does not run `cargo build` under `sudo`.
