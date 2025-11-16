# Substrate Isolated Shell – Data & Behavior Map

This document enumerates every new/updated data type, trait, environment variable, or behavior the feature set introduces. Entries are grouped per file as described in the file audit.

## `crates/common/src/manager_manifest.rs` (new)

- **Structs**
  - `ManagerManifest { version: u32, managers: Vec<ManagerSpec> }`
  - `ManagerSpec { name: String, priority: u8, detect: DetectSpec, init: InitSpec, errors: Vec<RegexSpec>, repair_hint: String, guest: GuestSpec }`
  - `DetectSpec { files: Vec<PathBuf>, commands: Vec<String>, env: HashMap<String, String>, script: Option<String> }`
  - `InitSpec { shell: Option<String>, powershell: Option<String> }`
  - `GuestSpec { detect_cmd: Option<String>, install: Option<InstallSpec> }`
  - `InstallSpec { apt: Option<String>, custom: Option<String> }`
  - `RegexSpec { pattern: String }`
- **APIs**
  - `ManagerManifest::load(base: &Path, overlay: Option<&Path>) -> Result<Self>`
  - `ManagerManifest::resolve_for_platform(&self, platform: Platform) -> Vec<ManagerSpec>`
  - Helper functions for env expansion, priority sorting.
- **Behavior**
  - Validate duplicates, ensure regex compiles, expand `$HOME`/`~`.
  - Provide `into_runtime_state()` used by shell shim modules.

## `crates/shell/src/manager_init.rs` (new)

- **Structs**
  - `ManagerState { name: String, detected: bool, reason: Option<String>, snippet: Option<String> }`
- **Functions**
  - `fn detect_and_generate(manifest_paths: ManifestPaths, cfg: ManagerInitConfig) -> Result<ManagerInitResult>`
  - `fn write_snippet(path: &Path, content: &str)`
  - `fn telemetry_payload(states: &[ManagerState]) -> serde_json::Value`
- **Config Struct**
  - `ManagerInitConfig { skip_all: bool, skip_list: HashSet<String>, platform: Platform, debug: bool }`
- **Env Vars**
  - `SUBSTRATE_SKIP_MANAGER_INIT`
  - `SUBSTRATE_SKIP_MANAGER_INIT_LIST`
  - `SUBSTRATE_MANAGER_INIT_DEBUG`

## `crates/shell/src/lib.rs` (existing)

- **CLI Additions**
  - Flags: `--no-world`, `--shim-doctor`, `--shim-repair`, `world deps status/install/sync`, `world enable`.
- **`ShellConfig` Updates**
  - New fields: `no_world: bool`, `manager_init_path: PathBuf`, `manager_env_path: PathBuf`.
- **Behavior**
  - Build per-session PATH without touching parent env.
  - Generate `manager_env.sh` referencing `manager_init.sh` + legacy `.substrate_bashenv`.
  - When `no_world`, skip manager injection and run host commands directly after policy logging.
  - Publish `manager_init` telemetry event at session start.

## `crates/shell/src/commands/shim_doctor.rs` (new)

- **Structs**
  - `ShimDoctorReport { manifest: ManifestInfo, path: PathDoctorStatus, trace_log: PathBuf, skip_all_requested: bool, states: Vec<ManagerDoctorState>, hints: Vec<HintRecord> }`
  - `ManifestInfo { base: PathBuf, overlay: Option<PathBuf>, overlay_exists: bool }`
  - `PathDoctorStatus { shim_dir: PathBuf, shim_dir_exists: bool, path_first_entry: Option<String>, host_contains_shims: bool, shim_first_in_path: bool, bashenv_path: PathBuf, bashenv_exists: bool }`
  - `ManagerDoctorState { name: String, detected: bool, reason: Option<String>, init_sourced: bool, snippet: Option<String>, repair_available: bool, last_hint: Option<HintRecord> }`
  - `HintRecord { name: String, hint: String, pattern: Option<String>, last_seen: DateTime<Utc> }`
  - `RepairOutcome::{Applied { manager, bashenv_path, backup_path }, Skipped { manager, reason } }`
- **Behavior**
  - `substrate shim doctor` reuses manifest detection + override rules, parses `~/.substrate/trace.jsonl` for the latest `manager_hint` entries, and prints either a text table (managers, PATH health, recent hints) or the JSON report above.
  - `substrate shim repair --manager <name> [--yes]` writes a delimited snippet block into `~/.substrate_bashenv`, creates/refreshes `~/.substrate_bashenv.bak`, logs a `shim_repair` telemetry event, and skips duplicate blocks by replacing them in-place when re-run.

## `crates/shell/src/world/mod.rs` (existing/new modules)

- **New Subcommands**
  - `WorldCommand::Enable`, `WorldDeps { action: Status|Install|Sync }`.
- **Behavior**
  - `Enable`: triggers provisioning scripts, updates config, ensures world agent reachable.
  - `Deps`: runs detection/installation via `world-backend-factory`, surfaces results.

## `crates/shim/src/exec.rs`

- **New Data**
  - Regex table loaded from manifest to match stderr.
  - `manager_hint_emitted: HashSet<String>` (per process) for dedup.
- **Behavior**
  - On non-zero exit, run matching logic; emit console + JSON entry with `manager_hint`.
  - When `no_world` env set (propagated from shell), bypass world logic entirely and just execute host command.

## `crates/shim/tests/integration.rs`

- **Test Scenarios**
  - Ensure shim respects host PATH when not injected (pass-through).
  - Validate hint logging when manager command missing.
  - `--no-world` path: confirm commands run without manager snippet.

## Scripts / Metadata

- `install-substrate.sh`
  - Recognize `--no-world`, write `~/.substrate/config.json` (new) with `{"world_enabled": true/false}`.
  - Stop exporting PATH entries.
  - Create empty `~/.substrate/manager_init.sh`.
- `world-enable.sh` (new)
  - Accepts flags, runs provisioning, updates config.

## Environment Variables Summary

- `SUBSTRATE_SKIP_MANAGER_INIT`
- `SUBSTRATE_SKIP_MANAGER_INIT_LIST`
- `SUBSTRATE_MANAGER_INIT_DEBUG`
- `SUBSTRATE_MANAGER_INIT_SHELL`
- `SUBSTRATE_SHIM_HINTS` (existing switch)
- `SUBSTRATE_WORLD_ENABLED` (derived from config; ensures shim sees `no_world`)
- `SUBSTRATE_WORLD_DEPS_MANIFEST`

Document and enforce each variable across shell + shim to avoid drift.***
