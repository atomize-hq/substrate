# Substrate Isolated Shell & Dependency Sync Plan

Status: **Authoring – ready for implementation once reviewed**

Scope: Replace the current host-wide shim injection with a Substrate-owned “pass-through” shell model, add automatic manager initialization + repair hints, and ship the world dependency sync workflow in one cohesive roadmap.

---

## 1. Objectives

1. **Isolated Shell**: Substrate must no longer modify the host’s PATH/rc files. Shims and manager hooks only activate inside Substrate-owned executions (interactive REPL, `substrate -c`, scripts, CI agents, world-agent invocations).
2. **Manager Parity**: Detect and auto-source common environment managers (nvm, pyenv, direnv, asdf, etc.) so that commands behave inside Substrate exactly as on the host, without manual user setup.
3. **Repair & Guidance**: When commands fail because a manager isn’t initialized, emit structured hints and offer self-healing options (`substrate shim doctor/repair`).
4. **World Dependency Sync**: Provide tooling (`substrate world deps …`) to install Linux/WSL equivalents of host tools inside the isolation world, keeping guest environments aligned with user expectations even though the host remains untouched.
5. **Cross-Platform Fidelity**: Maintain existing world backend behavior (Linux namespaces, macOS Lima, Windows WSL) and ensure the new features behave identically across them.

---

## 2. Current State Recap

- Installer writes `.substrate_bashenv`, prepends `~/.substrate/shims` to the host PATH, and instructs users to source it globally. Every host command is therefore shimmed today.
- Shim deployment (`substrate --shim-deploy`) copies/symlinks `substrate-shim` into `~/.substrate/shims` and writes a `.version`.
- The shell REPL (async) communicates with world agents over Unix sockets (Linux), Lima TCP/Vsock (macOS), or named pipes/WSL (Windows). It sources `~/.bashrc` via the PTY bootstrap script.
- World dependency sync (doc) is drafted but unimplemented: no manifest, no CLI.

Pain points:
- Host shells are polluted with Substrate shims even when not using Substrate.
- Manager-backed commands (nvm, pyenv) require manual tweaks.
- Guest worlds lack tooling parity with host installs.

---

## 3. Desired Behavior

1. Host PATH, rc files, login shells remain untouched after installation.
2. When launching any Substrate command, the process environment is built dynamically:
   - `PATH = "~/.substrate/shims:$SHIM_ORIGINAL_PATH"` (deduped) inside that process only.
   - Auto-generated manager snippet is sourced before executing user commands.
   - Manager hinting/logging/doctor features active.
3. Outside Substrate, commands execute exactly as before (no shims in PATH).
4. Users can inspect/repair manager state via explicit commands.
5. World dependency sync CLI detects host tools, inspects guest availability, and installs missing equivalents when approved.

---

## 4. Architecture Overview

### 4.1 Components

| Component | Responsibility |
| --- | --- |
| **Shim manifest** (`config/manager_hooks.yaml` + overlay) | Declarative mapping of managers/tools → detection probes, init snippets, repair hints, guest install recipes. |
| **Manager init module** (`crates/shell/src/manager_init.rs`) | Loads manifest, evaluates detection, generates `manager_init.sh`, and tracks status for telemetry. |
| **Substrate shell bootstrap** (`crates/shell/src/lib.rs`) | Builds per-session environment: prepends shims to PATH, sets `BASH_ENV`/PTY hook to source `manager_init.sh`, exports opt-out envs. |
| **Shim hint system** (`crates/shim/src/exec.rs`) | Matches stderr/stdout against manifest patterns, emits hints/log fields, rate-limits messages. |
| **Shim doctor/repair CLI** (`substrate shim doctor/repair`) | Reads detection results + recent hints, prints status, optionally appends recommended snippets to `~/.substrate_bashenv` for host-side use when requested. |
| **World deps CLI** (`substrate world deps status/install/sync`) | Uses the same manifest’s `guest_*` entries to check/install Linux equivalents within Lima/WSL. |
| **Installer adjustments** | Stop modifying host PATH; ensure `substrate` binary is on PATH; create empty `~/.substrate/manager_init.sh` for runtime use. |

### 4.2 Flow (Interactive Example)

1. User runs `substrate` from host shell (binary already on PATH).
2. Shell config step:
   - Load manifest (base + user overlay).
   - Detect managers (check files/env/commands).
   - Write aggregated snippet to `~/.substrate/manager_init.sh`.
   - Build command environment:
     - `PATH=~/.substrate/shims:$SHIM_ORIGINAL_PATH` (dedup) for this process.
     - `BASH_ENV` points to a tiny file that sources `manager_init.sh` and the legacy `~/.substrate_bashenv` (for compatibility).
3. Async REPL runs, connecting to the appropriate world backend.
4. Each shimmed command executes with merged PATH (host + runtime manager). On failure, shim matches errors, prints hints, logs events.
5. User can run `substrate shim doctor` to view detection/hints, or `substrate world deps sync` to install missing guest binaries.
6. `--no-world` runtime mode: when set, commands run directly on the host (no world delegation). Substrate skips manager-init/shim injection entirely and simply passes the command through after logging/policy checks; world deps CLI reports “world disabled”.
7. Upon exit, host shell PATH is exactly as before run.

---

## 5. Implementation Details

### 5.1 Manifest & Parser

- File: `config/manager_hooks.yaml` (versioned). User overlay at `~/.substrate/manager_hooks.local.yaml`.
- Schema:
  ```yaml
  version: 1
  managers:
    <name>:
      priority: 10            # lower runs earlier (optional)
      detect:
        files: ["$HOME/.nvm/nvm.sh"]
        commands: ["pyenv --version"]
        env:
          NVM_DIR: "$HOME/.nvm"
        script: "test -d ~/.asdf"
      init:
        shell: |              # POSIX snippet; sourced only inside Substrate
          export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
          [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
      errors:
        - "nvm: command not found"
        - "pyenv: .* command not found"
      repair_hint: |
        echo 'export NVM_DIR="$HOME/.nvm"' >> ~/.substrate_bashenv
        echo '[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"' >> ~/.substrate_bashenv
        source ~/.substrate_bashenv
      guest_detect:
        command: "node --version"         # run inside world agent
      guest_install:
        apt: |
          curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
          sudo apt-get install -y nodejs
  ```
- Parser in `crates/common/src/manager_manifest.rs`:
  - Validates schema, expands env vars, supports tilde.
  - Provides API for detection + init snippet retrieval.
  - Unit tests cover parsing, overlay merging, priority ordering.

### 5.2 Manager Detection & Snippet Generation

- Module `crates/shell/src/manager_init/mod.rs`.
- `fn detect_managers(manifest) -> Vec<ManagerState>` where `ManagerState` records detection result + reason.
- `fn build_snippet(states) -> String` merges all `init.shell` blocks (ordered by priority/name) with guards:
  ```sh
  # Generated by Substrate – do not edit
  for manager in states if detected && !skipped:
      emit comment + snippet
  ```
- Write snippet atomically to `~/.substrate/manager_init.sh`.
- Export `SUBSTRATE_MANAGER_INIT=~/.substrate/manager_init.sh`.
- Provide opt-outs:
  - `SUBSTRATE_SKIP_MANAGER_INIT=1` disables entire module.
  - `SUBSTRATE_SKIP_MANAGER_INIT_LIST` disables specific managers.
- Telemetry: record detection summary once per session (shell log entry).

### 5.3 Shell Bootstrap Changes

1. Remove host PATH manipulation from installer instructions; `.substrate_bashenv` only keeps helper exports (if any). Users no longer prepend `~/.substrate/shims` globally.
2. Update installer to support `--no-world` provisioning:
   - When invoked with `--no-world`, only place the substrate binary + minimal assets, skip world agent downloads, and mark install metadata (`~/.substrate/config.json`) with `world_enabled=false`.
   - Provide a follow-up command (`substrate world enable` or `substrate world install`) that upgrades the install in-place: downloads world assets, runs provisioning scripts, and flips `world_enabled=true`.
   - Default install continues to provision the world; documentation must explain both paths.
3. Inside `substrate_shell::run_shell`:
   - Build `PATH = format!("{shims}:{orig}")` **without** touching the parent process PATH (use `Command::env` when spawning child shells).
   - Set `BASH_ENV` (non-interactive) to `~/.substrate/manager_env.sh`, a generated file that sources both `manager_init.sh` and the legacy `.substrate_bashenv` (for existing env tweaks).
   - Update PTY bootstrap script (`BASH_PREEXEC_SCRIPT`) to source `$SUBSTRATE_MANAGER_INIT` before `~/.bashrc`.
   - Ensure Windows/macOS adapters respect the same environment injection (PowerShell support TBD via `init.powershell` fields).
   - When `config.no_world` (or CLI `--no-world`) is set, skip manager snippet sourcing + PATH injection and run commands directly on the host after logging/policy checks.

### 5.4 Shim Hinting

- Augment `substrate_shim::run_shim`:
  - Capture stderr/stdout for pattern matching.
  - After command completes with non-zero status, run manifest regex checks (unless `SUBSTRATE_SHIM_HINTS=0`).
  - Emit console hint + structured log: `{"manager_hint":{"name":"nvm","hint":"...","ts":...}}`.
  - Use per-session `HashSet` to avoid duplicate hints.
  - Tests: extend shim integration tests (pyenv/nvm scenarios) to confirm hints appear and logs contain metadata.

### 5.5 Shim Doctor / Repair CLI

- Add CLI branch under `substrate shim`:
  - `doctor`: prints table (name, detected?, init sourced?, last hint). `--json` outputs structured data.
  - `repair --manager <name>` (or `doctor --apply`) writes the manifest’s `repair_hint` to `~/.substrate_bashenv` (with backup) for users who explicitly want host shells to source the manager. Confirmation prompt unless `--yes`.
  - Implementation: reuse manager detection module + parse recent hints from `~/.substrate/trace.jsonl`.

### 5.6 World Dependency Sync

- CLI under `substrate world …`:
  0. `world enable` / `world install`:
     - For installs created with `--no-world`, download and provision the full world stack (world agent binaries, Lima/WSL assets) and flip the install metadata so future shells default to world-enabled mode.
     - Expose `--dry-run` and verbose flags for visibility; reuse existing provisioning scripts.
  1. `deps status`:
  1. `status`:
     - Enumerate manifest entries with `guest_detect`.
     - For each, run detection both on host (already known) and inside world agent via REST call (Linux) or forwarded tunnels (macOS/Windows). Use `world-backend-factory` to send `which` or version commands.
     - Output table: tool, host status, world status, install recipe available?
  2. `install <tool>`:
     - Execute manifest’s `guest_install.<provider>` script inside the world environment. Initially support `apt` for Lima/WSL, optional `custom` (arbitrary script). Provide dry-run.
  3. `sync [--all]`:
     - Compare host vs guest; prompt to install missing ones. `--all` auto-accepts (with `--yes`).
  4. (Optional future) `add` command for user-defined recipes appended to overlay manifest.
- Installer flag `--sync-deps`: after initial shim deployment, prompt user to run `substrate world deps sync --all`.

### 5.7 Configuration / Flags

- `SUBSTRATE_SKIP_MANAGER_INIT`
- `SUBSTRATE_SKIP_MANAGER_INIT_LIST`
- `SUBSTRATE_MANAGER_INIT_DEBUG=1` (logs detection decisions)
- `SUBSTRATE_SHIM_HINTS=0`
- `SUBSTRATE_WORLD_DEPS_MANIFEST=path` (override manifest for CLI/testing)
- Document in `docs/CONFIGURATION.md`.

### 5.8 Testing & Validation

1. **Unit tests**: manifest parser, detection logic, snippet writer.
2. **Shim integration tests**: ensure PATH injection limited to Substrate processes; verify manager hints/logging; new tests for `substrate_shim_test_bin`.
3. **Shell tests**: add e2e tests for `substrate -c` confirming shims not required on host PATH.
4. **Installer smoke**: run Linux/macOS install scripts, ensure host PATH unchanged, `substrate` still works, generated files exist.
5. **World deps tests**: use mock world agent or container to verify `status/install/sync`.
6. **Cross-platform**: manual QA on Linux, macOS (Lima), Windows (WSL). Confirm PATH injection, init sourcing, and CLI behavior.

### 5.9 Rollout Plan

1. **Phase A – Infrastructure**
   - Implement manifest parser + manager init module.
   - Update installer to stop modifying host PATH; ensure `substrate` still on PATH.
   - Adjust shell bootstrap to inject shims/managers per session.
   - Verify baseline behavior (commands run, shims active inside Substrate only).
2. **Phase B – Manager Support**
   - Populate manifest with Tier-1 managers (nvm, pyenv, direnv, asdf, conda).
   - Add shim hinting + doctor CLI (behind `SUBSTRATE_MANAGER_INIT_DEBUG`).
3. **Phase C – World Deps CLI**
   - Implement `world deps status/install/sync`, integrate with manifest `guest_*`.
   - Add installer prompt for optional sync.
4. **Phase D – Expansion**
   - Add Tier-2 managers (mise/rtx, rbenv, sdkman, bun, volta, goenv, etc.).
   - Add richer doctor/health reporting commands (no telemetry dashboards required).
   - Windows remains experimental; match Linux/mac behavior where feasible and plan future native rewrite (PowerShell init deferred).

---

## 6. Deliverables

1. New manifest + overlay infrastructure.
2. Manager init module, generated snippet, per-session PATH injection.
3. Updated installer/scripts/docs reflecting pass-through design.
4. Shim hinting, `substrate shim doctor/repair`.
5. `substrate world deps` CLI + installer integration.
6. Comprehensive tests and documentation.

---

## 7. Open Items (pre-implementation decisions)

1. Lima package manager choice: start with `apt` inside the guest (current Lima image), later add `brew`/custom steps if images change.
2. Telemetry schema vs doctor output: prefer local health commands; still log `manager_init`/`manager_hint` in trace JSON for support tooling.
3. Security considerations: ensure init snippets come from trusted manifests; clearly mark user overlays as executed code.
4. Windows: keep current WSL integration experimental, mirror Linux/mac behavior as much as possible, plan native rewrite later (no PowerShell-specific init now).

Resolve these before coding; document deviations via ADRs.

---

This plan supersedes previous shim/manager drafts and incorporates the world dependency sync design so implementation proceeds coherently across shell, shim, installer, and world subsystems.
