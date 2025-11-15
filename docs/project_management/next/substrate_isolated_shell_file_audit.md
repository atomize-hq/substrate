# Substrate Isolated Shell – File Audit

This checklist enumerates every file that must be added or updated to deliver the isolated shell, manager auto-init, shim doctor, and world dependency sync features. Entries are grouped by crate/module to make ownership and reviews straightforward.

## Top-Level / Workspace

| Path | Action | Notes |
| --- | --- | --- |
| `config/manager_hooks.yaml` | **New** | Shipping manifest for manager detection/repair/install metadata. Include comments + version header. |
| `config/manager_hooks.local.yaml` | Optional user override; document but do not ship by default. |
| `docs/README.md` | Update high-level overview (pass-through shims, world enable). |
| `docs/INSTALLATION.md` | Document `--no-world` install, new upgrade command, manager auto-init behavior. |
| `docs/USAGE.md` | Describe runtime manager support, shim doctor, world deps CLI. |
| `docs/CONFIGURATION.md` | Add new env vars (`SUBSTRATE_SKIP_MANAGER_INIT`, etc.). |
| `docs/RELEASE_NOTES.md` (or CHANGELOG) | Summaries of changes (if maintained). |

## Scripts / Provisioning

| Script | Action | Notes |
| --- | --- | --- |
| `scripts/substrate/install-substrate.sh` | Update to stop PATH hijack, add `--no-world`, emit metadata, call new upgrade command. |
| `scripts/substrate/dev-install-substrate.sh` | Mirror behavior for dev installs. |
| `scripts/substrate/uninstall-substrate.sh` | Ensure it cleans new files (manager init snippet, manifest overrides). |
| `scripts/substrate/world-enable.sh` (new) | Optional helper invoked by `substrate world enable` for provisioning upgrade. |
| `scripts/windows/install-substrate.ps1` | Match Linux/mac flow (as much as feasible) but keep world as experimental. |
| `scripts/windows/dev-install-substrate.ps1` | Same adjustments as above. |

## `crates/common`

| File | Action | Notes |
| --- | --- | --- |
| `src/lib.rs` | Export new manifest module. |
| `src/manager_manifest.rs` | **New** module parsing `config/manager_hooks.yaml`, providing typed structures + helpers. Include unit tests. |
| `src/manager_manifest/tests.rs` | Optional dedicated tests if file becomes large. |

## `crates/shim`

| File | Action | Notes |
| --- | --- | --- |
| `Cargo.toml` | Add dependency on `substrate-common` manifest module if needed (or serde for manifest). |
| `src/exec.rs` | Hook hint detection/logging, conditionally load manifest, emit structured fields, support pass-through for `--no-world`. |
| `src/context.rs` | Ensure `SHIM_ORIGINAL_PATH` fallbacks align with new injection flow. |
| `src/lib.rs` | Re-export new behaviors if necessary. |
| `tests/integration.rs` | Extend for hint logging and PATH isolation tests. |

## `crates/shell`

| File | Action | Notes |
| --- | --- | --- |
| `src/lib.rs` | Major updates: CLI flags, `ShellConfig` fields (`no_world`, `manager_init` paths), env injection logic, new subcommands (`shim doctor`, `shim repair`, `world enable/install`, `world deps …`). |
| `src/manager_init.rs` | **New**: detection + snippet generation and telemetry integration. |
| `src/shim_deploy.rs` | Ensure metadata aware of `--no-world` installs (if needed) but no PATH change. |
| `src/async_repl.rs` / `pty_exec.rs` | Ensure PTY bootstrap sources new snippet. |
| `src/world/mod.rs` or dedicated module | Add CLI plumbing for `world enable` and `world deps` commands. |
| `src/commands/shim_doctor.rs` (new) | Encapsulate doctor/repair logic. |
| `tests/` (integration) | Add shell-level tests to verify host PATH untouched and manager snippet used only inside Substrate. |

## `crates/world-*`

| File | Action | Notes |
| --- | --- | --- |
| `crates/world-agent/cli` | Expose API endpoints for world deps detection/install (if needed). |
| `crates/world-backend-factory` | Provide helpers for running detection/install commands inside the world (esp. Lima). |
| `crates/world-mac-lima` / `world-windows-wsl` | Ensure `world deps` CLI can talk through existing channels; mostly doc/config updates. |

## Tooling / CI

| File | Action | Notes |
| --- | --- | --- |
| `.github/workflows/*.yml` or CI scripts | Update smoke tests to cover pass-through mode and `world deps status`. |
| `docs/project_management/next/substrate_isolated_shell_plan.md` | Already created; keep in sync with implementation. |

Ensure each listed file has a clear owner before coding begins.***
