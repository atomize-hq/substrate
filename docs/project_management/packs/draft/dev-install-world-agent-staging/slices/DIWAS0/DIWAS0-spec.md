# DIWAS0-spec — Linux world-enable missing-artifact preflight

## Behavior delta (single)
- Existing: the standard `substrate world enable` flow can reach helper execution without a staged `world-agent` and can fail with low-level helper output.
- New: the standard version-dir flow checks the accepted staged path set before helper launch and exits `3` with one remediation block when both paths are missing.
- Why: the operator gets a deterministic root-cause failure for the enable-later workflow.

## Scope
- Rust runner preflight in the standard version-dir flow
- dry-run and non-dry-run ordering
- config-write ordering for `world.enabled`
- helper-override carve-out

## Behavior (authoritative)
### Standard version-dir preflight
- When `SUBSTRATE_WORLD_ENABLE_SCRIPT` is unset, `substrate world enable` resolves the standard version dir from `<home>/bin/substrate`.
- The accepted staged path set is `<version_dir>/bin/world-agent`, then `<version_dir>/bin/linux/world-agent`.
- Either accepted executable path is sufficient.
- If neither accepted path exists, the command exits `3` before helper launch, provisioning, systemd mutation, health verification, config writes, or manager-env writes.

### Remediation rendering
- The missing-artifact failure prints one operator-facing remediation block.
- The remediation names both accepted staged paths.
- The remediation names `scripts/substrate/dev-install-substrate.sh --no-world`.
- The remediation names `cargo build -p world-agent`.

### Dry-run and config ordering
- `--dry-run` runs the same missing-artifact preflight.
- With an accepted staged artifact present, dry-run exits `0` and writes no config, helper log, manager-env export, or systemd state.
- With the staged artifact missing, dry-run exits `3`.
- On the non-dry-run success path, `world.enabled` stays `false` until helper execution and health verification both succeed.

### Override carve-out
- When `SUBSTRATE_WORLD_ENABLE_SCRIPT` is set, helper discovery follows the override path exactly.
- This slice does not add a version-dir artifact preflight promise for the override path.

## Acceptance criteria
- AC-DIWAS0-01: In the standard version-dir flow, `substrate world enable` resolves the version dir from `<home>/bin/substrate` and checks the accepted staged path set in this order: `<version_dir>/bin/world-agent`, then `<version_dir>/bin/linux/world-agent`.
- AC-DIWAS0-02: In the standard version-dir flow, either accepted executable path is sufficient and the command does not require both paths to exist to continue.
- AC-DIWAS0-03: When neither accepted path exists, `substrate world enable` exits `3` before helper launch, provisioning, systemd mutation, health verification, config writes, or manager-env writes.
- AC-DIWAS0-04: The missing-artifact failure prints one operator-facing remediation that names both accepted paths, `scripts/substrate/dev-install-substrate.sh --no-world`, and `cargo build -p world-agent`.
- AC-DIWAS0-05: `substrate world enable --dry-run` runs the same missing-artifact preflight; the missing-artifact path exits `3`, the present-artifact path exits `0`, and dry-run writes no config, manager-env export, helper log, or systemd state.
- AC-DIWAS0-06: On the non-dry-run success path, `world.enabled` stays `false` until helper execution and health verification both succeed.
- AC-DIWAS0-07: `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior remains unchanged and stays outside the standard version-dir preflight contract owned by this slice.

## Out of scope
- dev-install staging changes
- helper-override artifact discovery rules beyond the existing override path
- Windows support changes
