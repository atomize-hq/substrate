# Environment Variables Inventory (Internal)

This is the exhaustive, repo-grounded inventory of environment variables that Substrate *reads and/or writes* today across Rust crates, scripts, tests/fixtures, and CI workflows.

It is not an operator-facing stability promise. The stability labels in the catalog are scoped to this repo (e.g., “test-only”, “script-only”, “internal”).

- Governing ADR (taxonomy intent, not a usage source of truth): `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Coverage Checklist

Evidence sources searched for concrete read/write usage (excluding `docs/**` as a source of truth for behavior):

- Rust (`crates/**`, `src/**`): `std::env::{var,var_os,set_var,remove_var}`, `env!/option_env!`, `cargo:rustc-env=...`, `Command::env/env_remove`, `getenv(...)`, and internal helpers that read env (e.g., `get_env_u64`, `parse_allowlist_env`).
- Shell scripts (`scripts/**`, `tests/**`, and shell fragments in `config/**`): `export VAR=...`, `${VAR:-...}` defaulting reads, and `VAR=value cmd` env-assignment prefixes.
- PowerShell (`scripts/**`): `$env:VAR` reads/writes (including parsing like `TryParse`).
- Config (`config/manager_hooks.yaml`): `detect.env` keys and embedded init/repair shell fragments.
- CI (`.github/workflows/*.yml`): `env:` mappings (write sites) plus any inline shell that uses env variables.

## Taxonomy

### Namespaces
- `SUBSTRATE_*`: Substrate runtime surface area (state exports, override-only knobs, internal/test toggles).
- `SUBSTRATE_OVERRIDE_*`: operator/test override inputs to effective-config resolution (separate from exported state).
- `SHIM_*`: Substrate shim internal coordination and logging controls.
- `TRACE_*`: trace writer controls.
- `WORLD_*`: world backend controls.
- Standard env vars (e.g. `HOME`, `PATH`): consumed/preserved by Substrate but not owned by it.

### Variant classes
- `dual-use (legacy)`: exported by Substrate-owned scripts/runtime *and* treated as config override inputs today (ADR-0006 removes the override role).
- `state (exported)`: exported state used for propagation; should not be treated as config overrides after ADR-0006.
- `override input`: `SUBSTRATE_OVERRIDE_*` names for config-shaped override inputs (ADR-0006).
- `override-only / internal`: non-config-shaped knobs and internal toggles; not exported as stable state by default.
- `internal/shim`, `internal/trace`, `internal/world`: coordination variables for internal components.
- `test/example`: fixtures/examples; not a supported interface.
- `standard`: external variables not owned by Substrate.

## Override Naming

- Planned config override inputs use the `SUBSTRATE_OVERRIDE_*` prefix (ADR-0006).
- Existing `*_OVERRIDE` suffix variables are currently used for non-config diagnostics/tests and test harness defaults:
  - `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE`
  - `SUBSTRATE_TEST_LINGER_STATE_OVERRIDE`
  - `SUBSTRATE_TEST_USER_GROUPS_OVERRIDE`

## Config Override Inputs (`SUBSTRATE_OVERRIDE_*`, ADR-0006)

These names are reserved for config-shaped override inputs and will be the only supported env override inputs for effective-config resolution once ADR-0006 is executed:

- `SUBSTRATE_OVERRIDE_WORLD`
- `SUBSTRATE_OVERRIDE_ANCHOR_MODE`
- `SUBSTRATE_OVERRIDE_ANCHOR_PATH`
- `SUBSTRATE_OVERRIDE_CAGED`
- `SUBSTRATE_OVERRIDE_POLICY_MODE`
- `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC`
- `SUBSTRATE_OVERRIDE_SYNC_DIRECTION`
- `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY`
- `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE`

Repo usage status (as of this inventory update): these override inputs are read by the shell effective-config resolver and are set by installers/tests; they are included in the catalog below.

## Exclusions (Not Read/Written In Code/Scripts/Tests/CI)

The previous scaffold inventory listed several names that do not have any read/write sites in `crates/`, `src/`, `scripts/`, `tests/`, or `.github/workflows/` (or were script-local variables, not environment variables).

### Docs-Only Mentions (No Read/Write Sites Found)
- `SUBSTRATE_AGENT_PIPE`
- `SUBSTRATE_CPU_LIMIT`
- `SUBSTRATE_GRAPH`
- `SUBSTRATE_MEM_LIMIT`
- `SUBSTRATE_NET_BUDGET`
- `TRACE_LOG_FILE`
- `WORLD_ID`

### Script-Local Variables (Not Environment Variables)
These are plain shell variables in scripts (assigned and used locally) and are not exported, defaulted from the environment, or passed as environment assignments to subprocesses:

- `SHIM_STATUS_JSON`
- `SUBSTRATE_GROUP`
- `TRACE_LOG_PATH`
- `WORLD_CAGED`
- `WORLD_ENABLED`

## Catalog

The catalog table is exhaustive for read/write sites found in `crates/`, `src/`, `scripts/`, `tests/`, `config/`, and `.github/workflows/`.

Column notes:
- `Direction`: `write` includes exports, `VAR=value cmd` env assignments, Rust `set_var/remove_var`, `Command::env/env_remove`, and build-time `cargo:rustc-env=...`.
- `Where set/read`: examples (first few hits); use `rg` for full occurrences.

| Name | Namespace | Variant class | Direction | Owner component(s) | Where set (examples) | Where read (examples) | Type/shape | Stability | Sensitive? | Logged/recorded? |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `SHIM_ACTIVE` | `SHIM` | internal/shim | both | shell, shim | crates/shell/src/execution/routing.rs:334; crates/shell/src/execution/invocation/runtime.rs:220 | crates/shim/src/context.rs:108; crates/shim/src/context.rs:136 | string | internal | no | no direct site |
| `SHIM_BUILD` | `SHIM` | internal/shim | read | shell, shim | — | crates/shell/src/execution/invocation/plan.rs:84; crates/shell/src/execution/routing/telemetry.rs:169 | string | internal | no | yes (shim execution log) |
| `SHIM_BYPASS` | `SHIM` | internal/shim | both | shim | crates/shim/tests/integration.rs:267 | crates/shim/src/context.rs:116 | string | internal | no | no direct site |
| `SHIM_CACHE_BUST` | `SHIM` | internal/shim | write | shim | crates/shim/tests/integration.rs:600; crates/shim/tests/integration.rs:972 | — | string | internal | no | no direct site |
| `SHIM_CALLER` | `SHIM` | internal/shim | both | shell, shim | crates/shell/src/execution/pty/io/runner.rs:99; crates/shell/src/execution/routing/dispatch/exec.rs:890 | crates/shim/src/context.rs:124; crates/shim/src/logger.rs:75 | string | internal | no | yes (shim execution log) |
| `SHIM_CALL_STACK` | `SHIM` | internal/shim | both | shell, shim | crates/shell/src/execution/pty/io/runner.rs:100; crates/shell/src/execution/routing/dispatch/exec.rs:891 | crates/shim/src/context.rs:130; crates/shim/src/logger.rs:76 | string | internal | no | yes (shim execution log) |
| `SHIM_DEPTH` | `SHIM` | internal/shim | both | shell, shim | crates/shim/src/context.rs:145; crates/shim/tests/integration.rs:109 | crates/shim/src/context.rs:87; crates/shim/src/context.rs:141 | string | internal | no | no direct site |
| `SHIM_FSYNC` | `SHIM` | internal/shim | read | trace | — | crates/trace/src/context.rs:114; crates/trace/src/output.rs:94 | string | internal | no | yes (trace component) |
| `SHIM_LOG_OPTS` | `SHIM` | internal/shim | both | common, shell, shim | crates/shell/tests/redaction.rs:90; crates/shim/src/logger.rs:282 | crates/common/src/lib.rs:85; crates/shell/src/execution/routing/dispatch/exec.rs:134 | enum (string) | internal | no | yes (shim execution log) |
| `SHIM_ORIGINAL_PATH` | `SHIM` | internal/shim | both | scripts, shell, shim, tests, trace | crates/shell/src/execution/routing.rs:329; crates/shell/src/execution/invocation/tests.rs:51 | crates/trace/src/span.rs:147; crates/shell/src/execution/invocation/tests.rs:47 | string | internal | no | yes (trace component) |
| `SHIM_PARENT_CMD_ID` | `SHIM` | internal/shim | both | shell, shim | crates/shell/src/execution/pty/io/runner.rs:95; crates/shell/src/execution/routing/dispatch/exec.rs:888 | crates/shim/src/logger.rs:77 | string | internal | no | yes (shim execution log) |
| `SHIM_PARENT_SPAN` | `SHIM` | internal/shim | both | replay, shell, shim, trace | crates/shell/src/execution/routing/dispatch/exec.rs:319; crates/replay/src/replay/executor.rs:70 | crates/trace/src/span.rs:160; crates/trace/src/span.rs:273 | string | internal | no | yes (trace component) |
| `SHIM_RUSTC_VERSION` | `SHIM` | internal/shim | both | shell | crates/shell/build.rs:7 | crates/shell/src/execution/invocation/plan.rs:85 | string | internal | no | no direct site |
| `SHIM_SESSION_ID` | `SHIM` | internal/shim | both | replay, shell, shim, trace | crates/shell/src/execution/routing.rs:328; crates/shell/src/execution/invocation/runtime.rs:218 | crates/trace/src/span.rs:141; crates/trace/src/span.rs:270 | string | internal | no | yes (trace component) |
| `SHIM_TRACE_LOG` | `SHIM` | internal/shim | both | scripts, shell, shim, trace | crates/shell/src/execution/routing.rs:330; crates/shell/src/execution/manager.rs:288 | crates/trace/src/context.rs:61; crates/shell/src/builtins/shim_doctor/report.rs:270 | string | internal | no | yes (trace component) |
| `SHIM_TRACE_LOG_MAX_MB` | `SHIM` | internal/shim | read | trace | — | crates/trace/src/output.rs:31 | string | internal | no | yes (trace component) |
| `SHIM_VERSION` | `SHIM` | internal/shim | both | shim | crates/shim/build.rs:29 | crates/shim/src/lib.rs:202 | string | internal | no | no direct site |
| `SUBSTRATE_AGENT_ID` | `SUBSTRATE` | override-only / internal | read | replay, shell, telemetry-lib, trace, world-backend | — | crates/world-windows-wsl/src/backend.rs:71; crates/trace/src/span.rs:143 | string | internal | no | yes (trace component) |
| `SUBSTRATE_AGENT_TCP_PORT` | `SUBSTRATE` | state (exported) | both | scripts, world-agent | crates/world-agent/src/lib.rs:547; crates/world-agent/src/lib.rs:561 | crates/world-agent/src/lib.rs:424 | string | internal | no | no direct site |
| `SUBSTRATE_AGENT_TRANSPORT` | `SUBSTRATE` | override-only / internal | read | host-proxy | — | crates/host-proxy/src/runtime.rs:333 | string | internal | no | no direct site |
| `SUBSTRATE_ANCHOR_MODE` | `SUBSTRATE` | state (exported) | both | replay, scripts, shell, trace, world-agent, world-backend | crates/shell/src/execution/env_scripts.rs:58; crates/replay/src/state.rs:138 | crates/trace/src/context.rs:169; crates/world-agent/src/service.rs:511 | string | internal | no | yes (trace component) |
| `SUBSTRATE_ANCHOR_PATH` | `SUBSTRATE` | state (exported) | both | replay, scripts, shell, trace, world-agent | crates/shell/src/execution/env_scripts.rs:62; crates/replay/src/state.rs:140 | crates/trace/src/context.rs:170; crates/world-agent/src/service.rs:516 | string | internal | no | yes (trace component) |
| `SUBSTRATE_BASHENV_ACTIVE` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/builtins/shim_doctor/repair.rs:21 | crates/shell/src/builtins/shim_doctor/repair.rs:18 | string | internal | no | no direct site |
| `SUBSTRATE_BIN` | `SUBSTRATE` | override-only / internal | both | scripts | scripts/validate_phase_d.sh:20 | scripts/mac/smoke.sh:11 | string | script-only | no | no direct site |
| `SUBSTRATE_CAGED` | `SUBSTRATE` | state (exported) | both | replay, scripts, shell, trace, world-backend | crates/shell/src/execution/settings/runtime.rs:15; crates/shell/src/execution/env_scripts.rs:56 | crates/trace/src/context.rs:174; crates/world/src/guard.rs:13 | string | internal | no | yes (trace component) |
| `SUBSTRATE_COMMAND_SUCCESS_EVENTS` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/execution/agent_events.rs:74 | string | internal | no | no direct site |
| `SUBSTRATE_COPYDIFF_ROOT` | `SUBSTRATE` | override-only / internal | both | shell, world-backend | crates/shell/tests/replay_world.rs:1674 | crates/world/src/copydiff.rs:130 | string | internal | no | no direct site |
| `SUBSTRATE_DEV_BIN` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/dev-shim-bootstrap.sh:38 | string | script-only | no | no direct site |
| `SUBSTRATE_DEV_PREFIX` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/dev-shim-bootstrap.sh:39 | string | script-only | no | no direct site |
| `SUBSTRATE_DISABLE_PTY` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing/dispatch/tests/pty.rs:598; crates/shell/src/execution/routing/dispatch/tests/pty.rs:602 | crates/shell/src/execution/routing/dispatch/registry.rs:719; crates/shell/src/execution/routing/dispatch/tests/pty.rs:960 | string | internal | no | no direct site |
| `SUBSTRATE_ENABLE_PREEXEC` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/scripts/bash_preexec.rs:16 | bool (0/1) | internal | no | no direct site |
| `SUBSTRATE_ENOSPC_PREFIX` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/replay_world.rs:1322; crates/shell/tests/replay_world.rs:1562 | crates/shell/tests/replay_world.rs:161 | string | test-only | no | no direct site |
| `SUBSTRATE_EXE` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/windows/wsl-smoke.ps1:110 | path | script-only | no | no direct site |
| `SUBSTRATE_FORCE_PTY` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing/dispatch/tests/pty.rs:573; crates/shell/src/execution/routing/dispatch/tests/pty.rs:584 | crates/shell/src/execution/routing/dispatch/registry.rs:714; crates/shell/src/execution/routing/dispatch/tests/pty.rs:572 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_PIPE` | `SUBSTRATE` | override-only / internal | read | world-backend | — | crates/world-windows-wsl/src/backend.rs:64 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TARGET` | `SUBSTRATE` | state (exported) | both | forwarder, scripts | crates/forwarder/src/config.rs:215; crates/forwarder/src/config.rs:296 | crates/forwarder/src/config.rs:27; scripts/windows/wsl-doctor.ps1:122 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TARGET_ENDPOINT` | `SUBSTRATE` | override-only / internal | both | forwarder | crates/forwarder/src/wsl.rs:159; crates/forwarder/src/wsl.rs:165 | crates/forwarder/src/wsl.rs:34 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TARGET_HOST` | `SUBSTRATE` | override-only / internal | both | forwarder | crates/forwarder/src/wsl.rs:163 | crates/forwarder/src/wsl.rs:26 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TARGET_MODE` | `SUBSTRATE` | override-only / internal | both | forwarder | crates/forwarder/src/wsl.rs:158; crates/forwarder/src/wsl.rs:162 | crates/forwarder/src/wsl.rs:24 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TARGET_PORT` | `SUBSTRATE` | override-only / internal | both | forwarder | crates/forwarder/src/wsl.rs:164 | crates/forwarder/src/wsl.rs:27 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TCP` | `SUBSTRATE` | override-only / internal | read | scripts, world-backend | — | crates/world-windows-wsl/src/transport.rs:17; scripts/windows/wsl-warm.ps1:213 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TCP_ADDR` | `SUBSTRATE` | override-only / internal | read | scripts, world-backend | — | crates/world-windows-wsl/src/transport.rs:10; scripts/windows/wsl-warm.ps1:211 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TCP_HOST` | `SUBSTRATE` | override-only / internal | read | world-backend | — | crates/world-windows-wsl/src/transport.rs:25 | string | internal | no | no direct site |
| `SUBSTRATE_FORWARDER_TCP_PORT` | `SUBSTRATE` | override-only / internal | read | scripts, world-backend | — | crates/world-windows-wsl/src/transport.rs:27; scripts/windows/wsl-warm.ps1:217 | int | internal | no | no direct site |
| `SUBSTRATE_HOME` | `SUBSTRATE` | state (exported) | both | broker, common, scripts, shell | crates/shell/src/builtins/world_enable/runner.rs:30; crates/shell/src/builtins/world_enable/runner/manager_env.rs:35 | crates/common/src/paths.rs:9; crates/common/src/paths.rs:25 | string | internal | no | no direct site |
| `SUBSTRATE_INNER_CMD` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world/src/exec.rs:269; crates/world-agent/src/internal_exec.rs:97 | string | internal | no | no direct site |
| `SUBSTRATE_INNER_LOGIN_SHELL` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world/src/exec.rs:212; crates/world-agent/src/internal_exec.rs:98 | string | internal | no | no direct site |
| `SUBSTRATE_INSTALLER_EXPECT_SOCKET` | `SUBSTRATE` | override-only / internal | read | tests | — | tests/installers/install_smoke.sh:117 | string | test-only | no | no direct site |
| `SUBSTRATE_INSTALL_ARCHIVE` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install-substrate.sh:23 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_ARTIFACT_DIR` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install-substrate.sh:23 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_BASE_URL` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install-substrate.sh:24 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_GITHUB_TOKEN` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/windows/install-substrate.ps1:51; scripts/substrate/install-substrate.sh:748 | string | script-only | yes | no direct site |
| `SUBSTRATE_INSTALL_LATEST_API` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/windows/install-substrate.ps1:50 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_NO_PATH` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install-substrate.sh:1643 | bool (0/1) | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_PRIMARY_USER` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install-substrate.sh:118 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_REF` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install.sh:19; scripts/substrate/uninstall.sh:89 | string | script-only | no | no direct site |
| `SUBSTRATE_INSTALL_WRAPPER_BASE_URL` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/install.sh:55 | string | script-only | no | no direct site |
| `SUBSTRATE_LANDLOCK_HELPER_PATH` | `SUBSTRATE` | state (exported) | both | world-backend | crates/world/src/exec.rs:184; crates/world/src/exec.rs:200 | crates/world/src/exec.rs:209 | string | internal | no | no direct site |
| `SUBSTRATE_LANDLOCK_HELPER_SRC` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world/src/exec.rs:180 | string | internal | no | no direct site |
| `SUBSTRATE_LIMA_SKIP_GUEST_BUILD` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/mac/lima-warm.sh:12 | string | script-only | no | no direct site |
| `SUBSTRATE_M5B_MANAGER_INIT_MARKER` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps.rs:584 | crates/shell/tests/world_deps.rs:67; crates/shell/tests/world_deps.rs:68 | string | test-only | no | no direct site |
| `SUBSTRATE_MANAGER_ENV` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing.rs:277; crates/shell/src/execution/routing.rs:279 | crates/shell/src/execution/routing/builtin/tests.rs:130 | string | internal | no | no direct site |
| `SUBSTRATE_MANAGER_ENV_ACTIVE` | `SUBSTRATE` | state (exported) | both | scripts, shell | crates/shell/src/execution/manager.rs:162; scripts/substrate/install-substrate.sh:863 | crates/shell/src/execution/manager.rs:159; scripts/substrate/install-substrate.sh:860 | string | internal | no | no direct site |
| `SUBSTRATE_MANAGER_INIT` | `SUBSTRATE` | state (exported) | both | scripts, shell | crates/shell/src/builtins/world_deps/runner.rs:138; crates/shell/src/builtins/world_deps/runner.rs:160 | crates/shell/src/execution/manager.rs:361; crates/shell/src/execution/manager.rs:371 | string | internal | no | no direct site |
| `SUBSTRATE_MANAGER_INIT_DEBUG` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/manager_init/tests.rs:115; crates/shell/src/execution/manager_init/tests.rs:132 | crates/shell/src/execution/manager_init/config.rs:32; crates/shell/src/execution/manager_init/tests.rs:111 | string | internal | no | no direct site |
| `SUBSTRATE_MANAGER_INIT_SHELL` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/execution/manager_init/runtime.rs:255 | string | internal | no | no direct site |
| `SUBSTRATE_MANAGER_MANIFEST` | `SUBSTRATE` | override-only / internal | both | shell, shim | crates/shell/tests/shell_env.rs:80; crates/shell/tests/fail_closed_semantics.rs:72 | crates/shell/src/execution/manager.rs:79; crates/shim/src/exec/logging.rs:194 | string | internal | no | no direct site |
| `SUBSTRATE_MOUNT_CWD` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world-agent/src/internal_exec.rs:94 | string | internal | no | no direct site |
| `SUBSTRATE_MOUNT_FS_MODE` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world/src/exec.rs:130; crates/world/src/exec.rs:153 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_MOUNT_PROJECT_DIR` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world-agent/src/internal_exec.rs:49; crates/world-agent/src/internal_exec.rs:75 | string | internal | no | no direct site |
| `SUBSTRATE_NETNS_GC_INTERVAL_SECS` | `SUBSTRATE` | override-only / internal | read | world-agent | — | crates/world-agent/src/lib.rs:118 | u64 (seconds) | internal | no | no direct site |
| `SUBSTRATE_NETNS_GC_TTL_SECS` | `SUBSTRATE` | override-only / internal | read | world-agent | — | crates/world-agent/src/handlers.rs:119; crates/world-agent/src/lib.rs:95 | u64 (seconds) | internal | no | no direct site |
| `SUBSTRATE_NO_SHIMS` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/invocation/tests.rs:54; crates/shell/tests/shim_deployment.rs:120 | crates/shell/src/execution/shim_deploy.rs:51; crates/shell/src/execution/invocation/tests.rs:50 | string | internal | no | no direct site |
| `SUBSTRATE_ORIGINAL_BASH_ENV` | `SUBSTRATE` | override-only / internal | both | scripts, shell | crates/shell/src/builtins/world_deps/guest.rs:270; crates/shell/src/builtins/world_deps/guest.rs:272 | crates/shell/src/execution/manager.rs:181; scripts/substrate/install-substrate.sh:882 | string | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_ANCHOR_MODE` | `SUBSTRATE_OVERRIDE` | override input | both | replay, scripts, shell, tests | scripts/substrate/install-substrate.sh:925; crates/shell/tests/config_show.rs:192 | crates/shell/src/execution/config_model.rs:299; crates/replay/src/state.rs:167 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_ANCHOR_PATH` | `SUBSTRATE_OVERRIDE` | override input | both | replay, scripts, shell, tests | scripts/substrate/install-substrate.sh:926; crates/shell/tests/config_show.rs:193 | crates/shell/src/execution/config_model.rs:311; crates/replay/src/state.rs:174 | string | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_CAGED` | `SUBSTRATE_OVERRIDE` | override input | both | replay, scripts, shell, tests | scripts/substrate/install-substrate.sh:924; crates/shell/tests/config_show.rs:194 | crates/shell/src/execution/config_model.rs:315; crates/replay/src/state.rs:178 | bool (0/1) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_POLICY_MODE` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | scripts/substrate/install-substrate.sh:927; crates/shell/tests/config_show.rs:195 | crates/shell/src/execution/config_model.rs:327 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | crates/shell/tests/config_show.rs:196 | crates/shell/src/execution/config_model.rs:339 | bool (0/1) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | crates/shell/tests/config_show.rs:198 | crates/shell/src/execution/config_model.rs:363 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_SYNC_DIRECTION` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | crates/shell/tests/config_show.rs:197 | crates/shell/src/execution/config_model.rs:351 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | crates/shell/tests/config_show.rs:199 | crates/shell/src/execution/config_model.rs:376 | string | internal | no | no direct site |
| `SUBSTRATE_OVERRIDE_WORLD` | `SUBSTRATE_OVERRIDE` | override input | both | scripts, shell, tests | scripts/substrate/install-substrate.sh:923; crates/shell/tests/config_show.rs:191 | crates/shell/src/execution/config_model.rs:283; crates/shell/src/builtins/world_deps/state.rs:184 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_PARENT_SPAN` | `SUBSTRATE` | override-only / internal | read | telemetry-lib | — | crates/telemetry-lib/src/correlation.rs:26 | string | internal | no | yes (telemetry correlation) |
| `SUBSTRATE_POLICY_GIT_CACHE` | `SUBSTRATE` | override-only / internal | read | trace | — | crates/trace/src/util.rs:52 | string | internal | no | yes (trace component) |
| `SUBSTRATE_POLICY_ID` | `SUBSTRATE` | override-only / internal | read | telemetry-lib | — | crates/telemetry-lib/src/correlation.rs:29 | string | internal | no | yes (telemetry correlation) |
| `SUBSTRATE_POLICY_MODE` | `SUBSTRATE` | state (exported) | both | broker, scripts, shell | crates/shell/src/execution/invocation/plan.rs:464; crates/shell/src/execution/env_scripts.rs:66 | crates/broker/src/mode.rs:29 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_PREFIX` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_enable.rs:134 | crates/shell/tests/world_enable.rs:21 | string | test-only | no | no direct site |
| `SUBSTRATE_PROJECT_PATH` | `SUBSTRATE` | override-only / internal | read | world-backend | — | crates/world-windows-wsl/src/backend.rs:44 | string | internal | no | no direct site |
| `SUBSTRATE_PTY_DEBUG` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/execution/pty/control.rs:106; crates/shell/src/execution/pty/io/runner.rs:130 | string | internal | no | no direct site |
| `SUBSTRATE_PTY_PIPELINE_LAST` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing/dispatch/tests/pty.rs:919; crates/shell/src/execution/routing/dispatch/tests/pty.rs:935 | crates/shell/src/execution/routing/dispatch/registry.rs:418; crates/shell/src/execution/routing/dispatch/tests/pty.rs:918 | string | internal | no | no direct site |
| `SUBSTRATE_REPLAY` | `SUBSTRATE` | override-only / internal | both | replay | crates/replay/src/replay/executor.rs:71 | crates/replay/src/state.rs:177; crates/replay/tests/integration.rs:245 | string | internal | no | no direct site |
| `SUBSTRATE_REPLAY_USE_WORLD` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing/replay.rs:123; crates/shell/src/execution/routing/replay.rs:127 | crates/shell/src/execution/routing/replay.rs:67; crates/shell/src/execution/routing/replay.rs:209 | string | internal | no | no direct site |
| `SUBSTRATE_REPLAY_VERBOSE` | `SUBSTRATE` | state (exported) | both | replay, shell, world-backend | crates/shell/src/execution/invocation/plan.rs:433; crates/shell/tests/replay_world.rs:240 | crates/world/src/overlayfs/mod.rs:210; crates/shell/src/execution/routing/replay.rs:326 | string | internal | no | no direct site |
| `SUBSTRATE_ROOT` | `SUBSTRATE` | state (exported) | both | scripts, tests | scripts/windows/dev-install-substrate.ps1:55; scripts/windows/dev-install-substrate.ps1:75 | scripts/substrate/uninstall.sh:11; tests/installers/install_state_smoke.sh:384 | string | script-only | no | no direct site |
| `SUBSTRATE_SESSION_ID` | `SUBSTRATE` | override-only / internal | read | telemetry-lib | — | crates/telemetry-lib/src/correlation.rs:24 | string | internal | no | yes (telemetry correlation) |
| `SUBSTRATE_SHELL` | `SUBSTRATE` | override-only / internal | read | shell, trace | — | crates/trace/src/span.rs:145; crates/trace/src/span.rs:274 | string | internal | no | yes (trace component) |
| `SUBSTRATE_SHIM_DEPLOY_DIR` | `SUBSTRATE` | override-only / internal | write | shell | crates/shell/tests/support/mod.rs:81 | — | string | test-only | no | no direct site |
| `SUBSTRATE_SHIM_HINTS` | `SUBSTRATE` | override-only / internal | both | shim | crates/shim/tests/integration.rs:700; crates/shim/tests/integration.rs:797 | crates/shim/src/exec/logging.rs:83; crates/shim/src/exec/logging.rs:180 | string | internal | no | no direct site |
| `SUBSTRATE_SHIM_ORIGINAL_PATH` | `SUBSTRATE` | override-only / internal | write | shell | crates/shell/tests/support/mod.rs:80 | — | string | test-only | no | no direct site |
| `SUBSTRATE_SHIM_PATH` | `SUBSTRATE` | override-only / internal | write | shell | crates/shell/tests/shell_env.rs:77; crates/shell/tests/shell_env.rs:220 | — | string | test-only | no | no direct site |
| `SUBSTRATE_SKIP_MANAGER_INIT` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/manager_init/tests.rs:113; crates/shell/src/execution/manager_init/tests.rs:124 | crates/shell/src/execution/manager_init/config.rs:25; crates/shell/src/execution/manager_init/tests.rs:109 | string | internal | no | no direct site |
| `SUBSTRATE_SKIP_MANAGER_INIT_LIST` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/manager_init/tests.rs:114; crates/shell/src/execution/manager_init/tests.rs:128 | crates/shell/src/execution/manager_init/config.rs:28; crates/shell/src/execution/manager_init/tests.rs:110 | string | internal | no | no direct site |
| `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/socket_activation.rs:54; crates/shell/tests/shell_env.rs:78 | crates/shell/src/execution/socket_activation.rs:92 | string | internal | no | no direct site |
| `SUBSTRATE_SYNC_AUTO_SYNC` | `SUBSTRATE` | state (exported) | write | shell, tests | crates/shell/tests/policy_routing_semantics.rs:96 | — | string | internal | no | no direct site |
| `SUBSTRATE_SYNC_CONFLICT_POLICY` | `SUBSTRATE` | state (exported) | write | shell, tests | crates/shell/tests/policy_routing_semantics.rs:98 | — | string | internal | no | no direct site |
| `SUBSTRATE_SYNC_DIRECTION` | `SUBSTRATE` | state (exported) | write | shell, tests | crates/shell/tests/policy_routing_semantics.rs:97 | — | string | internal | no | no direct site |
| `SUBSTRATE_SYNC_EXCLUDE` | `SUBSTRATE` | state (exported) | write | shell, tests | crates/shell/tests/policy_routing_semantics.rs:99 | — | string | internal | no | no direct site |
| `SUBSTRATE_TEST_CARGO_LOG` | `SUBSTRATE` | test/example | both | tests | tests/mac/installer_parity_fixture.sh:93 | tests/mac/installer_parity_fixture.sh:285 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_CARGO_TARGET_ROOT` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_smoke.sh:438 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_FAKE_USER` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_smoke.sh:130 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_FILE_SENTINEL` | `SUBSTRATE` | test/example | both | tests | tests/mac/installer_parity_fixture.sh:94 | tests/mac/installer_parity_fixture.sh:170; tests/mac/installer_parity_fixture.sh:296 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_GROUP_ENTRY` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_state_smoke.sh:204; tests/installers/install_smoke.sh:325 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_GROUP_EXISTS` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:135 | tests/installers/world_provision_smoke.sh:222; tests/installers/install_state_smoke.sh:202 | bool (0/1) | test-only | no | no direct site |
| `SUBSTRATE_TEST_GROUP_LOG` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:131 | tests/installers/world_provision_smoke.sh:238; tests/installers/world_provision_smoke.sh:251 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_GROUP_MEMBERS` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_state_smoke.sh:203 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_HOME` | `SUBSTRATE` | test/example | write | common | crates/common/src/manager_manifest/tests.rs:71 | — | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR` | `SUBSTRATE` | test/example | both | tests | tests/mac/installer_parity_fixture.sh:91 | tests/mac/installer_parity_fixture.sh:193 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LIMACTL_LOG` | `SUBSTRATE` | test/example | both | tests | tests/mac/installer_parity_fixture.sh:90 | tests/mac/installer_parity_fixture.sh:192 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LINGER_LOG` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:132 | tests/installers/world_provision_smoke.sh:265; tests/installers/install_state_smoke.sh:305 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LINGER_STATE` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:136 | tests/installers/world_provision_smoke.sh:264; tests/installers/install_state_smoke.sh:277 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LINGER_STATE_OVERRIDE` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_smoke.sh:136 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_LOCAL_WORLD_ID` | `SUBSTRATE` | test/example | both | shell | crates/shell/src/execution/routing/dispatch/tests/linux_world.rs:14; crates/shell/src/execution/routing/dispatch/tests/linux_world.rs:31 | crates/shell/src/execution/routing/dispatch/world_ops.rs:477 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_PRIMARY_USER` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:133 | tests/installers/world_provision_smoke.sh:193; tests/installers/install_state_smoke.sh:170 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_SKIP_SOCKET` | `SUBSTRATE` | test/example | both | shell | crates/shell/tests/world_enable.rs:341 | crates/shell/tests/world_enable.rs:34 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_SUBSTRATE_LOG` | `SUBSTRATE` | test/example | both | tests | tests/mac/installer_parity_fixture.sh:490 | tests/mac/installer_parity_fixture.sh:365 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_SYSTEMCTL_LOG` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/world_provision_smoke.sh:95; tests/installers/world_provision_smoke.sh:172 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_USER_GROUPS` | `SUBSTRATE` | test/example | both | tests | tests/installers/install_smoke.sh:134 | tests/installers/world_provision_smoke.sh:194; tests/installers/install_state_smoke.sh:144 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_USER_GROUPS_OVERRIDE` | `SUBSTRATE` | test/example | read | tests | — | tests/installers/install_smoke.sh:134 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_WORLD_EXIT` | `SUBSTRATE` | test/example | both | shell | crates/shell/tests/world_enable.rs:321 | crates/shell/tests/world_enable.rs:33 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_WORLD_LOG` | `SUBSTRATE` | test/example | both | shell | crates/shell/tests/world_enable.rs:135 | crates/shell/tests/world_enable.rs:17 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_WORLD_STDERR` | `SUBSTRATE` | test/example | both | shell | crates/shell/tests/world_enable.rs:218 | crates/shell/tests/world_enable.rs:29 | string | test-only | no | no direct site |
| `SUBSTRATE_TEST_WORLD_STDOUT` | `SUBSTRATE` | test/example | both | shell | crates/shell/tests/world_enable.rs:217 | crates/shell/tests/world_enable.rs:25 | string | test-only | no | no direct site |
| `SUBSTRATE_TRACE_LOG` | `SUBSTRATE` | state (exported) | both | telemetry-lib | crates/telemetry-lib/src/lib.rs:239; crates/telemetry-lib/src/lib.rs:248 | crates/telemetry-lib/src/correlation.rs:30 | string | internal | no | yes (telemetry correlation) |
| `SUBSTRATE_UNINSTALL_REF` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/uninstall.sh:89 | string | script-only | no | no direct site |
| `SUBSTRATE_UNINSTALL_WRAPPER_BASE_URL` | `SUBSTRATE` | override-only / internal | read | scripts | — | scripts/substrate/uninstall.sh:96 | string | script-only | no | no direct site |
| `SUBSTRATE_WORLD` | `SUBSTRATE` | state (exported) | both | replay, scripts, shell, shim | crates/shell/src/builtins/world_deps/state.rs:193; crates/shell/src/execution/invocation/tests.rs:52 | crates/shell/src/builtins/world_deps/state.rs:15; crates/shell/src/builtins/world_deps/state.rs:191 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_AGENT_BIN` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/execution/routing/dispatch/world_ops.rs:412 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps_layering.rs:143; crates/shell/tests/world_deps.rs:169 | crates/shell/tests/world_deps_layering.rs:41; crates/shell/tests/world_deps.rs:43 | string | test-only | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_FAIL_TOOL` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps.rs:906 | crates/shell/tests/world_deps.rs:45 | string | test-only | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` | `SUBSTRATE` | override-only / internal | both | scripts, shell | crates/shell/tests/world_deps_layering.rs:142; crates/shell/tests/world_deps.rs:168 | crates/shell/tests/world_deps_layering.rs:45; crates/shell/tests/world_deps.rs:50 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_GUEST_LOG` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps_layering.rs:141; crates/shell/tests/world_deps.rs:167 | crates/shell/tests/world_deps_layering.rs:29; crates/shell/tests/world_deps.rs:31 | string | test-only | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_HOST_LOG` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps_layering.rs:140; crates/shell/tests/world_deps.rs:166 | crates/shell/tests/world_deps_layering.rs:17; crates/shell/tests/world_deps.rs:19 | string | test-only | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_MANIFEST` | `SUBSTRATE` | state (exported) | both | shell | crates/shell/src/execution/routing/path_env.rs:166; crates/shell/tests/world_deps.rs:164 | crates/shell/src/builtins/world_deps/runner.rs:52; crates/shell/src/execution/routing/path_env.rs:63 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_DEPS_MARKER_DIR` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_deps_layering.rs:139; crates/shell/tests/world_deps.rs:165 | crates/shell/tests/world_deps_layering.rs:44; crates/shell/tests/world_deps.rs:49 | string | test-only | no | no direct site |
| `SUBSTRATE_WORLD_ENABLED` | `SUBSTRATE` | state (exported) | both | replay, scripts, shell, shim | crates/shell/src/builtins/world_deps/state.rs:194; crates/shell/src/execution/invocation/tests.rs:53 | crates/shell/src/builtins/world_deps/state.rs:18; crates/shell/src/builtins/world_deps/state.rs:192 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_ENABLE_SCRIPT` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_enable.rs:131 | crates/shell/src/builtins/world_enable/runner.rs:58 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR` | `SUBSTRATE` | override-only / internal | both | shell | crates/shell/tests/world_enable.rs:141 | crates/shell/src/builtins/world_enable/runner/verify.rs:61 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_FS_ISOLATION` | `SUBSTRATE` | state (exported) | both | shell, world-agent, world-backend | crates/shell/src/execution/routing/dispatch/exec.rs:127; crates/shell/src/execution/platform/mod.rs:80 | crates/world/src/exec.rs:76; crates/world-agent/src/service.rs:538 | enum (string) | internal | no | no direct site |
| `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST` | `SUBSTRATE` | override-only / internal | read | world-agent | — | crates/world-agent/src/internal_exec.rs:17 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST` | `SUBSTRATE` | override-only / internal | read | world-agent | — | crates/world-agent/src/internal_exec.rs:18 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_FS_MODE` | `SUBSTRATE` | state (exported) | both | replay, shell, trace, world-agent, world-backend | crates/shell/src/execution/routing/dispatch/exec.rs:126; crates/shell/src/execution/platform/mod.rs:79 | crates/world-windows-wsl/src/backend.rs:228; crates/trace/src/context.rs:173 | string | internal | no | yes (trace component) |
| `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST` | `SUBSTRATE` | override-only / internal | read | world-agent, world-backend | — | crates/world/src/exec.rs:130; crates/world/src/exec.rs:153 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_ID` | `SUBSTRATE` | state (exported) | both | shell, shim, telemetry-lib, trace | crates/shell/src/execution/platform_world/windows.rs:80; crates/shell/src/execution/platform_world/windows.rs:211 | crates/trace/src/span.rs:162; crates/trace/src/span.rs:280 | string | internal | no | yes (trace component) |
| `SUBSTRATE_WORLD_REQUIRE_WORLD` | `SUBSTRATE` | override-only / internal | read | shell, world-agent | — | crates/world-agent/tests/full_isolation_pty.rs:37; crates/world-agent/tests/full_isolation_nonpty.rs:72 | string | internal | no | no direct site |
| `SUBSTRATE_WORLD_ROOT_MODE` | `SUBSTRATE` | override-only / internal | both | replay, shell, trace | crates/shell/tests/replay_world.rs:1057; crates/shell/tests/replay_world.rs:1188 | crates/trace/src/context.rs:171 | string | internal | no | yes (trace component) |
| `SUBSTRATE_WORLD_ROOT_PATH` | `SUBSTRATE` | override-only / internal | both | replay, shell, trace | crates/shell/tests/replay_world.rs:1058; crates/shell/tests/replay_world.rs:1189 | crates/trace/src/context.rs:172 | string | internal | no | yes (trace component) |
| `SUBSTRATE_WORLD_SOCKET` | `SUBSTRATE` | override-only / internal | both | replay, scripts, shell | crates/shell/src/builtins/world_enable/runner/helper_script.rs:44; crates/shell/tests/world_enable.rs:132 | crates/shell/src/builtins/world_enable/runner/paths.rs:74; crates/shell/src/execution/socket_activation.rs:149 | string | internal | no | no direct site |
| `SUBSTRATE_WSL_DISTRO` | `SUBSTRATE` | override-only / internal | read | world-backend | — | crates/world-windows-wsl/src/backend.rs:43 | string | internal | no | no direct site |
| `SUBSTRATE_WS_DEBUG` | `SUBSTRATE` | override-only / internal | read | shell | — | crates/shell/src/execution/routing/dispatch/world_ops.rs:127 | string | internal | no | no direct site |
| `TRACE_LOG_KEEP` | `TRACE` | internal/trace | both | trace | crates/trace/src/tests.rs:135; crates/trace/src/tests.rs:168 | crates/trace/src/output.rs:41 | string | internal | no | yes (trace component) |
| `TRACE_LOG_MAX_MB` | `TRACE` | internal/trace | both | shell, trace | crates/trace/src/tests.rs:134; crates/trace/src/tests.rs:167 | crates/trace/src/output.rs:27 | string | internal | no | yes (trace component) |
| `WORLD_PROVISION_FAILED` | `WORLD` | internal/world | read | scripts | — | scripts/substrate/dev-install-substrate.sh:972 | bool (0/1) | script-only | no | no direct site |
| `ACTION` | `standard` | standard | write | scripts | scripts/triad/orch_ensure.sh:21 | — | string | script-only | no | no direct site |
| `AGENT_SOCKET` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:358; crates/host-proxy/src/runtime.rs:369 | string | external | no | no direct site |
| `AGENT_TCP_HOST` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:350 | string | external | no | no direct site |
| `AGENT_TCP_PORT` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:351 | string | external | no | no direct site |
| `AGENT_TRANSPORT` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:340 | string | external | no | no direct site |
| `API_TOKEN` | `standard` | standard | read | shell | — | crates/shell/src/execution/routing/builtin/tests.rs:71 | string | test-only | yes | no direct site |
| `ARTIFACT_DIR` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `ASDF_DATA_DIR` | `standard` | standard | both | other | config/manager_hooks.yaml:124; config/manager_hooks.yaml:135 | config/manager_hooks.yaml:124; config/manager_hooks.yaml:135 | string | external | no | no direct site |
| `ASDF_DIR` | `standard` | standard | both | other | config/manager_hooks.yaml:123; config/manager_hooks.yaml:134 | config/manager_hooks.yaml:123; config/manager_hooks.yaml:134 | string | external | no | no direct site |
| `ASDF_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_health.rs:54 | — | string | test-only | no | no direct site |
| `AUTH_ENABLED` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:320 | string | external | yes | no direct site |
| `AUTH_TOKEN_FILE` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:324 | string | external | yes | no direct site |
| `AUTOMATION` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:158 | string | script-only | no | no direct site |
| `BASH_ENV` | `standard` | standard | both | shell | crates/shell/src/builtins/world_deps/guest.rs:268; crates/shell/src/builtins/world_deps/guest.rs:371 | crates/shell/src/builtins/world_deps/guest.rs:162; crates/shell/src/execution/invocation/plan.rs:531 | string | external | no | no direct site |
| `BEFORE_MAIN` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `BIN` | `standard` | standard | read | scripts | — | scripts/validate_phase_d.sh:10 | string | script-only | no | no direct site |
| `BUILD_FLAGS` | `standard` | standard | write | scripts | scripts/substrate/dev-install-substrate.sh:667 | — | string | script-only | no | no direct site |
| `BUILD_GUEST_AGENT` | `standard` | standard | read | scripts | — | scripts/mac/lima-warm.sh:339 | string | script-only | no | no direct site |
| `BUILD_GUEST_CLI` | `standard` | standard | read | scripts | — | scripts/mac/lima-warm.sh:338 | string | script-only | no | no direct site |
| `BUILD_MANIFEST_NAME` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `BUN_INSTALL` | `standard` | standard | both | common, other, scripts | crates/common/src/manager_manifest/tests.rs:416; scripts/substrate/world-deps.yaml:18 | config/manager_hooks.yaml:322; config/manager_hooks.yaml:329 | string | external | no | no direct site |
| `BUN_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:138; crates/shell/tests/manager_init.rs:52 | — | string | test-only | no | no direct site |
| `CARGO_MANIFEST_DIR` | `standard` | standard | read | shell, shim | — | crates/shell/src/execution/manager.rs:87; crates/shell/src/execution/routing/path_env.rs:93 | string | external | no | no direct site |
| `CARGO_PKG_VERSION` | `standard` | standard | read | shell, shim, trace | — | crates/trace/src/context.rs:14; crates/shell/src/execution/shim_deploy.rs:114 | string | external | no | yes (trace component) |
| `CARGO_TARGET_DIR` | `standard` | standard | write | scripts | scripts/mac/lima-warm.sh:404; scripts/mac/lima-warm.sh:413 | — | string | script-only | no | no direct site |
| `CARGO_TERM_COLOR` | `standard` | standard | write | CI | .github/workflows/feature-smoke.yml; .github/workflows/ci-testing.yml | — | string | CI-only | no | no direct site |
| `CARGO_WORKSPACE_DIR` | `standard` | standard | read | shell, shim | — | crates/shell/tests/common.rs:66; crates/shim/tests/integration.rs:29 | string | test-only | no | no direct site |
| `CHECKS` | `standard` | standard | write | scripts | scripts/triad/task_finish.sh:24 | — | string | script-only | no | no direct site |
| `CI_FAILED_JOBS` | `standard` | standard | write | scripts | scripts/ci/dispatch_ci_testing.sh:35 | — | string | script-only | no | no direct site |
| `CI_FAILED_OSES` | `standard` | standard | write | scripts | scripts/ci/dispatch_ci_testing.sh:34 | — | string | script-only | no | no direct site |
| `CI_PASSED_OSES` | `standard` | standard | write | scripts | scripts/ci/dispatch_ci_testing.sh:33 | — | string | script-only | no | no direct site |
| `CI_URL` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `CI_WINDOW_MINUTES` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `CMD` | `standard` | standard | write | shell | crates/shell/src/execution/manager_init/tests.rs:181 | — | string | test-only | no | no direct site |
| `CODEX_CODE_EXIT` | `standard` | standard | write | scripts | scripts/triad/task_start_pair.sh:31 | — | string | script-only | no | no direct site |
| `CODEX_EXIT` | `standard` | standard | write | scripts | scripts/triad/task_start.sh:30; scripts/triad/task_start_platform_fixes.sh:31 | — | string | script-only | no | no direct site |
| `CODEX_TEST_EXIT` | `standard` | standard | write | scripts | scripts/triad/task_start_pair.sh:32 | — | string | test-only | no | no direct site |
| `COLUMNS` | `standard` | standard | both | shell | crates/shell/src/execution/pty/io/runner.rs:118; crates/shell/src/execution/pty/io/types.rs:301 | crates/shell/src/execution/pty/io/types.rs:240; crates/shell/src/execution/pty/io/types.rs:287 | string | external | no | no direct site |
| `CONDA_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_health.rs:62 | — | string | test-only | no | no direct site |
| `CONDA_ROOT` | `standard` | standard | both | other | config/manager_hooks.yaml:168; config/manager_hooks.yaml:170 | config/manager_hooks.yaml:166; config/manager_hooks.yaml:173 | string | external | no | no direct site |
| `CROSS_PLATFORM` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:155 | string | script-only | no | no direct site |
| `CURRENT_VERSION` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `DEBIAN_FRONTEND` | `standard` | standard | write | scripts | scripts/wsl/provision.sh:4 | — | string | script-only | no | no direct site |
| `DETECTED_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:20; crates/shell/tests/shim_doctor.rs:22 | — | string | test-only | no | no direct site |
| `DIRENV_DIR` | `standard` | standard | read | common | — | crates/common/src/manager_manifest/tests.rs:170 | string | test-only | no | no direct site |
| `DIRENV_HOME` | `standard` | standard | read | common | — | crates/common/src/manager_manifest/tests.rs:174 | string | test-only | no | no direct site |
| `DIRENV_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_health.rs:47 | — | string | test-only | no | no direct site |
| `DRY_RUN` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `EXPORT_COMPLEX` | `standard` | standard | both | shell | crates/shell/src/execution/routing/builtin/tests.rs:84 | crates/shell/src/execution/routing/builtin/tests.rs:88 | string | test-only | no | no direct site |
| `FAILED_PLATFORMS` | `standard` | standard | write | scripts | scripts/triad/task_start_platform_fixes.sh:26 | — | string | script-only | no | no direct site |
| `FAKE_ROOT` | `standard` | standard | read | tests | — | tests/installers/world_provision_smoke.sh:94; tests/installers/world_provision_smoke.sh:173 | string | test-only | no | no direct site |
| `FAKE_VERSION` | `standard` | standard | read | tests | — | tests/installers/install_smoke.sh:63; tests/mac/installer_parity_fixture.sh:6 | string | test-only | no | no direct site |
| `FEATURE` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:153 | string | script-only | no | no direct site |
| `FEATURE_DIR` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:154 | string | script-only | no | no direct site |
| `FILE` | `standard` | standard | write | shell | crates/shell/src/execution/manager_init/tests.rs:174 | — | string | test-only | no | no direct site |
| `GH_TOKEN` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | yes | no direct site |
| `GITHUB_PATH` | `standard` | standard | read | CI | — | .github/workflows/feature-smoke.yml:293; .github/workflows/feature-smoke.yml:330 | string | CI-only | no | no direct site |
| `GIT_COMMON_DIR` | `standard` | standard | write | scripts | scripts/triad/task_start.sh:210; scripts/triad/task_finish.sh:197 | — | string | script-only | no | no direct site |
| `GOENV_MARKER` | `standard` | standard | write | shell | crates/shell/tests/manager_init.rs:66 | — | string | test-only | no | no direct site |
| `GOENV_ROOT` | `standard` | standard | both | common, other | crates/common/src/manager_manifest/tests.rs:454; config/manager_hooks.yaml:386 | crates/common/src/manager_manifest/tests.rs:581; config/manager_hooks.yaml:386 | string | test-only | no | no direct site |
| `HEALTHY_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shim_health.rs:21; crates/shell/tests/shim_health.rs:23 | — | string | test-only | no | no direct site |
| `HOME` | `standard` | standard | both | CI, host-proxy, other, scripts, shell, telemetry-lib, tests, world-agent, world-backend | crates/shell/tests/world_enable.rs:129; crates/shell/tests/world_enable.rs:290 | crates/world/src/exec.rs:192; crates/world/src/exec.rs:202 | string | external | no | yes (telemetry correlation) |
| `HOSTNAME` | `standard` | standard | read | replay, trace | — | crates/trace/src/context.rs:162; crates/replay/src/state.rs:132 | string | external | no | yes (trace component) |
| `HOST_PROXY_SOCKET` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:296 | string | external | no | no direct site |
| `HOST_STATE_PATH` | `standard` | standard | read | scripts | — | scripts/substrate/install-substrate.sh:226 | string | script-only | no | no direct site |
| `IFS` | `standard` | standard | write | scripts | scripts/e2e/triad_e2e_phase2.sh:392; scripts/e2e/triad_e2e_phase2.sh:418 | — | string | script-only | no | no direct site |
| `INSTALLER_NAME` | `standard` | standard | read | scripts | — | scripts/substrate/install-substrate.sh:4 | string | script-only | no | no direct site |
| `JSON_DETECTED` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:223; crates/shell/tests/shim_doctor.rs:225 | — | string | test-only | no | no direct site |
| `JSON_MISSING` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:233; crates/shell/tests/shim_doctor.rs:235 | — | string | test-only | no | no direct site |
| `KEEP_MAC_DOCTOR_FIXTURE` | `standard` | standard | read | tests | — | tests/mac/lima_doctor_fixture.sh:17 | string | test-only | no | no direct site |
| `KVER` | `standard` | standard | write | scripts | scripts/check-host-prereqs.sh:12 | — | string | script-only | no | no direct site |
| `LANG` | `standard` | standard | both | replay, trace | crates/replay/src/replay/executor.rs:61 | crates/trace/src/context.rs:157; crates/replay/src/replay/executor.rs:60 | string | external | no | yes (trace component) |
| `LC_ALL` | `standard` | standard | both | replay | crates/replay/src/replay/executor.rs:64 | crates/replay/src/replay/executor.rs:63 | string | external | no | no direct site |
| `LD_PRELOAD` | `standard` | standard | write | shell | crates/shell/tests/replay_world.rs:1321; crates/shell/tests/replay_world.rs:1561 | — | string | test-only | no | no direct site |
| `LIMA_BUILD_PROFILE` | `standard` | standard | read | scripts, tests | — | scripts/mac/lima-warm.sh:8; tests/mac/installer_parity_fixture.sh:338 | string | script-only | no | no direct site |
| `LIMA_PROFILE_PATH` | `standard` | standard | read | scripts | — | scripts/mac/lima-warm.sh:5 | string | script-only | no | no direct site |
| `LIMA_VM_NAME` | `standard` | standard | read | scripts, tests | — | scripts/mac/lima-warm.sh:4; tests/mac/installer_parity_fixture.sh:336 | string | script-only | no | no direct site |
| `LINES` | `standard` | standard | both | shell | crates/shell/src/execution/pty/io/runner.rs:119; crates/shell/src/execution/pty/io/types.rs:295 | crates/shell/src/execution/pty/io/types.rs:236; crates/shell/src/execution/pty/io/types.rs:286 | string | external | no | no direct site |
| `LISTEN_FDNAMES` | `standard` | standard | both | world-agent | crates/world-agent/src/socket_activation.rs:155 | crates/world-agent/src/socket_activation.rs:117 | string | external | no | no direct site |
| `LISTEN_FDS` | `standard` | standard | both | world-agent | crates/world-agent/src/socket_activation.rs:153 | crates/world-agent/src/socket_activation.rs:49 | int | external | no | no direct site |
| `LISTEN_FD_START` | `standard` | standard | both | world-agent | crates/world-agent/src/socket_activation.rs:156 | crates/world-agent/src/socket_activation.rs:89 | string | external | no | no direct site |
| `LISTEN_PID` | `standard` | standard | both | world-agent | crates/world-agent/src/socket_activation.rs:154 | crates/world-agent/src/socket_activation.rs:65 | int | external | no | no direct site |
| `LOCALAPPDATA` | `standard` | standard | read | forwarder, scripts | — | crates/forwarder/src/config.rs:191; crates/forwarder/src/windows.rs:47 | string | external | no | no direct site |
| `LP1_PROVISION_PROFILE` | `standard` | standard | read | tests | — | tests/installers/world_provision_smoke.sh:7 | string | test-only | no | no direct site |
| `MAC_DOCTOR_SCENARIO` | `standard` | standard | read | tests | — | tests/mac/lima_doctor_fixture.sh:35 | string | test-only | no | no direct site |
| `MAC_DOCTOR_VIRT` | `standard` | standard | read | tests | — | tests/mac/lima_doctor_fixture.sh:156 | string | test-only | no | no direct site |
| `MANAGER_HOME` | `standard` | standard | read | common | — | crates/common/src/manager_manifest/tests.rs:107 | string | test-only | no | no direct site |
| `MANAGER_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shell_env.rs:23 | — | string | test-only | no | no direct site |
| `MANAGER_TEST_HOME` | `standard` | standard | write | common | crates/common/src/manager_manifest/tests.rs:97; crates/common/src/manager_manifest/tests.rs:99 | — | string | test-only | no | no direct site |
| `MANIFEST_JSON` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `MAX_BODY_SIZE` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:304 | string | external | no | no direct site |
| `MISE_DATA_DIR` | `standard` | standard | write | common | crates/common/src/manager_manifest/tests.rs:341 | — | string | test-only | no | no direct site |
| `MISE_HOME` | `standard` | standard | read | other | — | config/manager_hooks.yaml | string | external | no | no direct site |
| `MISE_MARKER` | `standard` | standard | write | shell | crates/shell/tests/manager_init.rs:24 | — | string | test-only | no | no direct site |
| `MISSING_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:30; crates/shell/tests/shim_doctor.rs:32 | — | string | test-only | no | no direct site |
| `NEW_MAIN` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `NEXT` | `standard` | standard | write | scripts | scripts/triad/task_start.sh:31; scripts/triad/task_start_integ_final.sh:27 | — | string | script-only | no | no direct site |
| `NEXT_VERSION` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `NVM_DIR` | `standard` | standard | both | other | config/manager_hooks.yaml:22; config/manager_hooks.yaml:31 | config/manager_hooks.yaml:22; config/manager_hooks.yaml:31 | string | external | no | no direct site |
| `OLDPWD` | `standard` | standard | both | shell | crates/shell/src/execution/routing/builtin/world_deps.rs:31 | crates/shell/src/execution/routing/builtin/tests.rs:366; crates/shell/src/execution/routing/builtin/tests.rs:383 | string | external | no | no direct site |
| `OVERLAY` | `standard` | standard | write | shell | crates/shell/src/execution/manager.rs:396 | — | string | external | no | no direct site |
| `OVERLAY_VALUE` | `standard` | standard | write | shell | crates/shell/tests/shell_env.rs:222 | — | string | test-only | no | no direct site |
| `PATH` | `standard` | standard | both | common, other, replay, scripts, shell, shim, tests, trace, world-backend | crates/common/src/manager_manifest/tests.rs:417; crates/common/src/manager_manifest/tests.rs:436 | crates/trace/src/context.rs:154; crates/shell/src/builtins/shim_doctor/report.rs:240 | path list | external | no | yes (trace component) |
| `PATHEXT` | `standard` | standard | read | shim | — | crates/shim/src/context.rs:224; crates/shim/src/resolver.rs:68 | string | external | no | no direct site |
| `PATH_BEFORE_SUBSTRATE_SHIM` | `standard` | standard | write | shell | crates/shell/tests/support/mod.rs:83 | — | string | test-only | no | no direct site |
| `PLAIN_VALUE` | `standard` | standard | read | shell | — | crates/shell/src/execution/routing/builtin/tests.rs:72 | string | test-only | no | no direct site |
| `PLATFORM` | `standard` | standard | read | scripts | — | scripts/substrate/install-substrate.sh:152; scripts/substrate/install-substrate.sh:222 | string | script-only | no | no direct site |
| `PROCESSOR_ARCHITECTURE` | `standard` | standard | read | scripts | — | scripts/windows/wsl-warm.ps1:68 | string | script-only | no | no direct site |
| `PROJECT` | `standard` | standard | both | scripts, tests | scripts/mac/lima-warm.sh:142 | tests/mac/installer_parity_fixture.sh:145 | string | script-only | no | no direct site |
| `PROMOTE_PUSH_TOKEN` | `standard` | standard | both | CI | .github/workflows/promote.yml | .github/workflows/promote.yml:88 | string | CI-only | yes | no direct site |
| `PWD` | `standard` | standard | both | shell | crates/shell/src/execution/routing/builtin/world_deps.rs:32 | crates/shell/src/execution/routing/builtin/tests.rs:365; crates/shell/src/execution/routing/builtin/tests.rs:410 | string | external | no | no direct site |
| `PYENV_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_health.rs:69 | — | string | test-only | no | no direct site |
| `PYENV_ROOT` | `standard` | standard | both | other | config/manager_hooks.yaml:58; config/manager_hooks.yaml:66 | config/manager_hooks.yaml:58; config/manager_hooks.yaml:66 | string | external | no | no direct site |
| `PYTHONUNBUFFERED` | `standard` | standard | write | forwarder | crates/forwarder/src/wsl.rs:154 | — | string | external | no | no direct site |
| `Path` | `standard` | standard | write | CI | .github/workflows/feature-smoke.yml:329 | — | string | CI-only | no | no direct site |
| `RATE_LIMIT_CONCURRENT` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:316 | string | external | no | no direct site |
| `RATE_LIMIT_RPM` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:312 | string | external | no | no direct site |
| `RBENV_MARKER` | `standard` | standard | write | shell | crates/shell/tests/manager_init.rs:38 | — | string | test-only | no | no direct site |
| `RBENV_ROOT` | `standard` | standard | both | common, other | crates/common/src/manager_manifest/tests.rs:377; config/manager_hooks.yaml:252 | crates/common/src/manager_manifest/tests.rs:555; config/manager_hooks.yaml:252 | string | test-only | no | no direct site |
| `RELEASE_COMMIT` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `RELEASE_PRERELEASE` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `RELEASE_ROOT` | `standard` | standard | write | scripts | scripts/substrate/world-enable.sh:7 | — | string | script-only | no | no direct site |
| `RELEASE_TAG` | `standard` | standard | write | CI | .github/workflows/release.yml:258; .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `RELEASE_TITLE` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `REPAIR_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:481; crates/shell/tests/shim_doctor.rs:484 | — | string | test-only | no | no direct site |
| `REPO_ROOT` | `standard` | standard | write | scripts | scripts/substrate/dev-uninstall-substrate.sh:402; scripts/substrate/dev-install-substrate.sh:663 | — | string | script-only | no | no direct site |
| `REQUESTED_VERSION` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `REQUEST_TIMEOUT` | `standard` | standard | read | host-proxy | — | crates/host-proxy/src/runtime.rs:308 | string | external | no | no direct site |
| `REQUIRED_TARGETS` | `standard` | standard | write | scripts | scripts/triad/task_finish.sh:180 | — | string | script-only | no | no direct site |
| `RTX_HOME` | `standard` | standard | read | other | — | config/manager_hooks.yaml | string | external | no | no direct site |
| `RTX_MARKER` | `standard` | standard | write | shell | crates/shell/tests/manager_init.rs:31 | — | string | test-only | no | no direct site |
| `RUNNER_TEMP` | `standard` | standard | read | CI | — | .github/workflows/feature-smoke.yml:322 | string | CI-only | no | no direct site |
| `RUNTIME` | `standard` | standard | write | scripts | scripts/check-host-prereqs.sh:40 | — | string | script-only | no | no direct site |
| `RUN_URL` | `standard` | standard | write | CI, scripts | scripts/ci/dispatch_ci_testing.sh:31; .github/workflows/promote.yml | — | string | script-only | no | no direct site |
| `RUST_LOG` | `standard` | standard | both | scripts | scripts/dev-entrypoint.sh:14 | scripts/dev-entrypoint.sh:14 | string | script-only | no | no direct site |
| `RUST_TEST_THREADS` | `standard` | standard | write | CI | .github/workflows/ci-testing.yml; .github/workflows/nightly.yml | — | string | CI-only | no | no direct site |
| `RUST_TOOLCHAIN` | `standard` | standard | both | CI | .github/workflows/feature-smoke.yml; .github/workflows/ci-testing.yml | .github/workflows/feature-smoke.yml:324; .github/workflows/feature-smoke.yml:338 | string | CI-only | no | no direct site |
| `SAMPLE` | `standard` | standard | write | shell | crates/shell/src/execution/manager_init/tests.rs:147 | — | string | test-only | no | no direct site |
| `SAMPLE_MANAGER` | `standard` | standard | write | shell | crates/shell/tests/shim_doctor.rs:367; crates/shell/tests/shim_doctor.rs:369 | — | string | test-only | no | no direct site |
| `SCRIPTS_ROOT` | `standard` | standard | write | scripts | scripts/mac/smoke.sh:9 | — | string | script-only | no | no direct site |
| `SCRIPT_DIR` | `standard` | standard | write | scripts, tests | scripts/substrate/install.sh:7; scripts/substrate/world-enable.sh:6 | — | string | script-only | no | no direct site |
| `SCRIPT_ROOT` | `standard` | standard | write | scripts | scripts/validate_phase_d.sh:4; scripts/linux/world-socket-verify.sh:73 | — | string | script-only | no | no direct site |
| `SDKMAN_DIR` | `standard` | standard | both | common, other | crates/common/src/manager_manifest/tests.rs:398; config/manager_hooks.yaml:291 | crates/common/src/manager_manifest/tests.rs:561; config/manager_hooks.yaml:291 | string | test-only | no | no direct site |
| `SDKMAN_MARKER` | `standard` | standard | write | shell | crates/shell/tests/manager_init.rs:45 | — | string | test-only | no | no direct site |
| `SHELL` | `standard` | standard | both | replay, scripts, shell, trace | crates/shell/tests/policy_routing_semantics.rs:105; crates/shell/tests/integration/support.rs:66 | crates/trace/src/context.rs:164; crates/shell/src/execution/invocation/plan.rs:562 | string | external | no | yes (trace component) |
| `SKIP` | `standard` | standard | write | shell | crates/shell/src/execution/manager_init/tests.rs:229 | — | string | test-only | no | no direct site |
| `SKIPPED_TASK_ID` | `standard` | standard | write | scripts | scripts/triad/mark_noop_platform_fixes_completed.sh:26 | — | string | script-only | no | no direct site |
| `SKIP_LOG` | `standard` | standard | read | tests | — | tests/installers/install_state_smoke.sh:35; tests/installers/install_state_smoke.sh:39 | string | test-only | no | no direct site |
| `SMOKE_RUN` | `standard` | standard | write | scripts | scripts/triad/task_finish.sh:25 | — | string | script-only | no | no direct site |
| `SMOKE_RUN_ID` | `standard` | standard | write | scripts | scripts/triad/mark_noop_platform_fixes_completed.sh:24; scripts/triad/task_start_platform_fixes.sh:25 | — | string | script-only | no | no direct site |
| `STATE_EVENTS` | `standard` | standard | both | scripts | scripts/substrate/install-substrate.sh:260; scripts/substrate/dev-install-substrate.sh:318 | scripts/substrate/install-substrate.sh:268; scripts/substrate/dev-install-substrate.sh:326 | string | script-only | no | no direct site |
| `STUB_BIN` | `standard` | standard | write | tests | tests/installers/install_state_smoke.sh:646; tests/installers/install_state_smoke.sh:737 | — | string | test-only | no | no direct site |
| `SUDO_USER` | `standard` | standard | read | scripts | — | scripts/substrate/dev-uninstall-substrate.sh:48; scripts/substrate/install-substrate.sh:122 | string | script-only | no | no direct site |
| `TAG_NAME` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `TARGET_SHA` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `TARGET_TRIPLE` | `standard` | standard | write | CI | .github/workflows/release.yml | — | string | CI-only | no | no direct site |
| `TEMP` | `standard` | standard | read | scripts | — | scripts/windows/wsl-warm.ps1:89; scripts/windows/wsl-warm.ps1:90 | string | script-only | no | no direct site |
| `TERM` | `standard` | standard | both | replay, shell, trace | crates/shell/src/execution/pty/io/runner.rs:112; crates/shell/src/execution/pty/io/runner.rs:113 | crates/trace/src/context.rs:165; crates/shell/src/execution/pty/io/runner.rs:111 | string | external | no | yes (trace component) |
| `TESTING_SHA` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `TEST_ENV_KEY` | `standard` | standard | write | shell | crates/shell/src/execution/manager_init/tests.rs:63; crates/shell/src/execution/manager_init/tests.rs:65 | — | string | test-only | yes | no direct site |
| `TEST_MODE` | `standard` | standard | both | shell | crates/shell/src/execution/routing/dispatch/tests/support.rs:17; crates/shell/src/execution/routing/dispatch/tests/support.rs:56 | crates/shell/src/execution/routing/dispatch/registry.rs:406; crates/shell/src/execution/routing/dispatch/tests/support.rs:15 | string | test-only | no | no direct site |
| `TEST_VAR` | `standard` | standard | read | replay | — | crates/replay/tests/integration.rs:241; crates/replay/tests/planner_executor.rs:117 | string | test-only | no | no direct site |
| `TIER2_HOME` | `standard` | standard | write | common | crates/common/src/manager_manifest/tests.rs:324; crates/common/src/manager_manifest/tests.rs:487 | — | string | test-only | no | no direct site |
| `TMPDIR` | `standard` | standard | both | scripts, shell, tests | crates/shell/tests/world_enable.rs:128; crates/shell/tests/world_enable.rs:289 | scripts/substrate/install-substrate.sh:76; scripts/substrate/install-substrate.sh:77 | string | external | no | no direct site |
| `UNSET_ME` | `standard` | standard | read | shell | — | crates/shell/src/execution/routing/builtin/tests.rs:100 | string | test-only | no | no direct site |
| `UPDATED_TASK_ID` | `standard` | standard | write | scripts | scripts/triad/mark_noop_platform_fixes_completed.sh:25 | — | string | script-only | no | no direct site |
| `USER` | `standard` | standard | read | replay, scripts, shim, trace | — | crates/trace/src/context.rs:152; crates/replay/src/state.rs:123 | string | external | no | yes (trace component) |
| `USERNAME` | `standard` | standard | read | shim, trace | — | crates/trace/src/context.rs:152; crates/shim/src/logger.rs:82 | string | external | no | yes (trace component) |
| `USERPROFILE` | `standard` | standard | both | CI, scripts, shell | crates/shell/tests/world_enable.rs:130; crates/shell/tests/world_enable.rs:291 | crates/shell/src/execution/invocation/plan.rs:491; crates/shell/tests/shim_deployment.rs:42 | string | external | no | no direct site |
| `V` | `standard` | standard | write | scripts | scripts/check-host-prereqs.sh:17; scripts/check-container-prereqs.sh:17 | — | string | script-only | no | no direct site |
| `VERSION_COMMIT` | `standard` | standard | write | CI | .github/workflows/promote.yml | — | string | CI-only | no | no direct site |
| `VOLTA_HOME` | `standard` | standard | both | common, other, shell | crates/common/src/manager_manifest/tests.rs:435; crates/common/src/manager_manifest/tests.rs:440 | crates/common/src/manager_manifest/tests.rs:575; config/manager_hooks.yaml:354 | string | test-only | no | no direct site |
| `VOLTA_MARKER` | `standard` | standard | write | shell | crates/shell/tests/shell_env.rs:29; crates/shell/tests/shim_doctor.rs:147 | — | string | test-only | no | no direct site |
| `WSL_REQUIRED` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:156 | string | script-only | no | no direct site |
| `WSL_SEPARATE` | `standard` | standard | read | scripts | — | scripts/planning/new_feature.sh:157 | string | script-only | no | no direct site |
| `XDG_CACHE_HOME` | `standard` | standard | read | world-backend | — | crates/world/src/exec.rs:203 | string | external | no | no direct site |
| `XDG_CONFIG_HOME` | `standard` | standard | read | world-backend | — | crates/world/src/exec.rs:204 | string | external | no | no direct site |
| `XDG_DATA_HOME` | `standard` | standard | read | world-agent, world-backend | — | crates/world/src/exec.rs:205; crates/world-agent/src/pty.rs:641 | string | external | no | no direct site |
| `XDG_RUNTIME_DIR` | `standard` | standard | both | shell, world-backend | crates/shell/tests/replay_world.rs:244; crates/shell/tests/replay_world.rs:438 | crates/world/src/copydiff.rs:73; crates/world/src/copydiff.rs:139 | string | external | no | no direct site |
