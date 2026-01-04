# Environment Variables Inventory (Internal)

Scaffold note: this file is the exhaustive inventory view (developer-focused). It intentionally includes internal/test and standard environment variables referenced by the repo, and is not a stability promise for operators.

Canonical catalog and taxonomy for environment variables referenced by Substrate code, scripts, and docs (excluding `docs/project_management/**`).

- Governing ADR: `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Taxonomy

### Namespaces
- `SUBSTRATE_*`: Substrate runtime surface area (state exports, override-only knobs, internal/test toggles).
- `SUBSTRATE_OVERRIDE_*` (planned by ADR-0006): operator/test override inputs to effective-config resolution (separate from exported state).
- `SHIM_*`: Substrate shim internal coordination and logging controls.
- `TRACE_*`: trace writer controls.
- `WORLD_*`: world backend controls.
- Standard env vars (e.g. `HOME`, `PATH`): consumed/preserved by Substrate but not owned by it.

### Variant classes
- `dual-use (legacy)`: exported by Substrate-owned scripts/runtime *and* treated as config override inputs today (ADR-0006 removes the override role).
- `state (exported)`: exported state used for propagation; should not be treated as config overrides after ADR-0006.
- `override input (planned)`: reserved `SUBSTRATE_OVERRIDE_*` names for config-shaped override inputs (ADR-0006).
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

## Planned Override Inputs (ADR-0006)

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

## Catalog

Primary references are “first hit” anchors; use repo search for deeper context.

| Name | Namespace | Variant | Primary Reference |
| --- | --- | --- | --- |
| `AGENT_SOCKET` | `standard` | standard | `crates/host-proxy/src/runtime.rs:358` |
| `AGENT_TCP_HOST` | `standard` | standard | `crates/host-proxy/src/runtime.rs:350` |
| `AGENT_TCP_PORT` | `standard` | standard | `crates/host-proxy/src/runtime.rs:351` |
| `AGENT_TRANSPORT` | `standard` | standard | `crates/host-proxy/src/runtime.rs:340` |
| `API_TOKEN` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:71` |
| `ASDF_DATA_DIR` | `standard` | standard | `config/manager_hooks.yaml:124` |
| `ASDF_DIR` | `standard` | standard | `config/manager_hooks.yaml:123` |
| `ASDF_MANAGER` | `standard` | standard | `crates/shell/tests/shim_health.rs:54` |
| `AUTH_ENABLED` | `standard` | standard | `crates/host-proxy/src/runtime.rs:320` |
| `AUTH_TOKEN_FILE` | `standard` | standard | `crates/host-proxy/src/runtime.rs:324` |
| `BASH_ENV` | `standard` | standard | `crates/shell/src/builtins/world_deps/guest.rs:162` |
| `BUN_INSTALL` | `standard` | standard | `config/manager_hooks.yaml:322` |
| `BUN_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:52` |
| `CARGO_PKG_VERSION` | `standard` | standard | `crates/shim/build.rs:8` |
| `CARGO_WORKSPACE_DIR` | `standard` | standard | `crates/shell/tests/common.rs:66` |
| `CMD` | `standard` | standard | `crates/shell/src/execution/manager_init/tests.rs:181` |
| `COLUMNS` | `standard` | standard | `crates/shell/src/execution/pty/io/runner.rs:118` |
| `CONDA_MANAGER` | `standard` | standard | `crates/shell/tests/shim_health.rs:62` |
| `CONDA_ROOT` | `standard` | standard | `config/manager_hooks.yaml:168` |
| `DEBIAN_FRONTEND` | `standard` | standard | `scripts/wsl/provision.sh:4` |
| `DETECTED_MARKER` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:20` |
| `DIRENV_MANAGER` | `standard` | standard | `crates/shell/tests/shim_health.rs:47` |
| `EXPORT_COMPLEX` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:84` |
| `FILE` | `standard` | standard | `crates/shell/src/execution/manager_init/tests.rs:174` |
| `GOENV_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:66` |
| `GOENV_ROOT` | `standard` | standard | `config/manager_hooks.yaml:386` |
| `HEALTHY_MARKER` | `standard` | standard | `crates/shell/tests/shim_health.rs:21` |
| `HOME` | `standard` | standard | `crates/host-proxy/src/config.rs:27` |
| `HOSTNAME` | `standard` | standard | `crates/trace/src/context.rs:162` |
| `HOST_PROXY_SOCKET` | `standard` | standard | `crates/host-proxy/src/runtime.rs:296` |
| `JSON_DETECTED` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:223` |
| `JSON_MISSING` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:233` |
| `LANG` | `standard` | standard | `crates/replay/src/replay/executor.rs:60` |
| `LC_ALL` | `standard` | standard | `crates/replay/src/replay/executor.rs:63` |
| `LD_PRELOAD` | `standard` | standard | `crates/shell/tests/replay_world.rs:1321` |
| `LINES` | `standard` | standard | `crates/shell/src/execution/pty/io/runner.rs:119` |
| `LOCALAPPDATA` | `standard` | standard | `crates/forwarder/src/config.rs:191` |
| `MANAGER_MARKER` | `standard` | standard | `crates/shell/tests/shell_env.rs:23` |
| `MANAGER_TEST_HOME` | `standard` | standard | `crates/common/src/manager_manifest/tests.rs:97` |
| `MAX_BODY_SIZE` | `standard` | standard | `crates/host-proxy/src/runtime.rs:304` |
| `MISE_DATA_DIR` | `standard` | standard | `crates/common/src/manager_manifest/tests.rs:341` |
| `MISE_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:24` |
| `MISSING_MARKER` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:30` |
| `NVM_DIR` | `standard` | standard | `config/manager_hooks.yaml:22` |
| `OLDPWD` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:366` |
| `OVERLAY` | `standard` | standard | `crates/shell/src/execution/manager.rs:396` |
| `OVERLAY_VALUE` | `standard` | standard | `crates/shell/tests/shell_env.rs:222` |
| `PATH` | `standard` | state (exported) | `config/manager_hooks.yaml:60` |
| `PATHEXT` | `standard` | standard | `crates/shim/src/context.rs:224` |
| `PATH_BEFORE_SUBSTRATE_SHIM` | `standard` | standard | `crates/shell/tests/support/mod.rs:83` |
| `PLAIN_VALUE` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:72` |
| `PROCESSOR_ARCHITECTURE` | `standard` | standard | `scripts/windows/wsl-warm.ps1:68` |
| `PWD` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:365` |
| `PYENV_MANAGER` | `standard` | standard | `crates/shell/tests/shim_health.rs:69` |
| `PYENV_ROOT` | `standard` | standard | `config/manager_hooks.yaml:58` |
| `PYTHONUNBUFFERED` | `standard` | standard | `crates/forwarder/src/wsl.rs:154` |
| `RATE_LIMIT_CONCURRENT` | `standard` | standard | `crates/host-proxy/src/runtime.rs:316` |
| `RATE_LIMIT_RPM` | `standard` | standard | `crates/host-proxy/src/runtime.rs:312` |
| `RBENV_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:38` |
| `RBENV_ROOT` | `standard` | standard | `config/manager_hooks.yaml:252` |
| `REPAIR_MANAGER` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:481` |
| `REQUEST_TIMEOUT` | `standard` | standard | `crates/host-proxy/src/runtime.rs:308` |
| `RTX_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:31` |
| `RUST_LOG` | `standard` | standard | `docs/CONFIGURATION.md:347` |
| `SAMPLE` | `standard` | standard | `crates/shell/src/execution/manager_init/tests.rs:147` |
| `SAMPLE_MANAGER` | `standard` | standard | `crates/shell/tests/shim_doctor.rs:367` |
| `SDKMAN_DIR` | `standard` | standard | `config/manager_hooks.yaml:291` |
| `SDKMAN_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:45` |
| `SHELL` | `standard` | standard | `crates/replay/src/replay/executor.rs:57` |
| `SHIM_ACTIVE` | `SHIM` | internal/shim | `crates/shell/src/execution/invocation/runtime.rs:220` |
| `SHIM_BUILD` | `SHIM` | internal/shim | `crates/shell/src/execution/invocation/plan.rs:84` |
| `SHIM_BYPASS` | `SHIM` | internal/shim | `crates/shim/src/context.rs:116` |
| `SHIM_CACHE_BUST` | `SHIM` | internal/shim | `crates/shim/src/context.rs:17` |
| `SHIM_CALLER` | `SHIM` | internal/shim | `crates/shell/src/execution/pty/io/runner.rs:99` |
| `SHIM_CALL_STACK` | `SHIM` | internal/shim | `crates/shell/src/execution/pty/io/runner.rs:100` |
| `SHIM_DEPTH` | `SHIM` | internal/shim | `crates/shim/src/context.rs:13` |
| `SHIM_FSYNC` | `SHIM` | internal/shim | `crates/shim/src/lib.rs:120` |
| `SHIM_LOG_OPTS` | `SHIM` | internal/shim | `crates/common/src/lib.rs:85` |
| `SHIM_ORIGINAL_PATH` | `SHIM` | state (exported) | `crates/shell/src/execution/invocation/plan.rs:498` |
| `SHIM_PARENT_CMD_ID` | `SHIM` | internal/shim | `crates/shell/src/execution/pty/io/runner.rs:95` |
| `SHIM_PARENT_SPAN` | `SHIM` | internal/shim | `crates/replay/src/replay/executor.rs:70` |
| `SHIM_SESSION_ID` | `SHIM` | internal/shim | `crates/replay/src/replay/executor.rs:69` |
| `SHIM_STATUS_JSON` | `SHIM` | internal/shim | `scripts/linux/world-socket-verify.sh:146` |
| `SHIM_TRACE_LOG` | `SHIM` | internal/shim | `crates/shell/src/builtins/shim_doctor/report.rs:270` |
| `SHIM_TRACE_LOG_MAX_MB` | `SHIM` | internal/shim | `crates/trace/src/output.rs:31` |
| `SHIM_VERSION` | `SHIM` | internal/shim | `docs/ARCHITECTURE.md:41` |
| `SKIP` | `standard` | standard | `crates/shell/src/execution/manager_init/tests.rs:229` |
| `SUBSTRATE_AGENT_ID` | `SUBSTRATE` | override-only / internal | `crates/replay/src/replay/executor.rs:807` |
| `SUBSTRATE_AGENT_PIPE` | `SUBSTRATE` | override-only / internal | `docs/BACKLOG.md:179` |
| `SUBSTRATE_AGENT_TCP_PORT` | `SUBSTRATE` | override-only / internal | `crates/world-agent/src/lib.rs:43` |
| `SUBSTRATE_AGENT_TRANSPORT` | `SUBSTRATE` | override-only / internal | `crates/host-proxy/src/runtime.rs:333` |
| `SUBSTRATE_ANCHOR_MODE` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:283` |
| `SUBSTRATE_ANCHOR_PATH` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:295` |
| `SUBSTRATE_BASHENV_ACTIVE` | `SUBSTRATE` | override-only / internal | `crates/shell/src/builtins/shim_doctor/repair.rs:18` |
| `SUBSTRATE_BIN` | `SUBSTRATE` | override-only / internal | `docs/manual_verification/linux_world_socket.md:63` |
| `SUBSTRATE_CAGED` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:299` |
| `SUBSTRATE_COMMAND_SUCCESS_EVENTS` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/agent_events.rs:74` |
| `SUBSTRATE_COPYDIFF_ROOT` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/replay_world.rs:1674` |
| `SUBSTRATE_CPU_LIMIT` | `SUBSTRATE` | override-only / internal | `docs/CONFIGURATION.md:455` |
| `SUBSTRATE_DEV_BIN` | `SUBSTRATE` | override-only / internal | `scripts/substrate/dev-shim-bootstrap.sh:38` |
| `SUBSTRATE_DEV_PREFIX` | `SUBSTRATE` | override-only / internal | `scripts/substrate/dev-shim-bootstrap.sh:39` |
| `SUBSTRATE_DISABLE_PTY` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/dispatch/registry.rs:719` |
| `SUBSTRATE_ENABLE_PREEXEC` | `SUBSTRATE` | override-only / internal | `crates/shell/src/scripts/bash_preexec.rs:16` |
| `SUBSTRATE_ENOSPC_PREFIX` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/replay_world.rs:1322` |
| `SUBSTRATE_EXE` | `SUBSTRATE` | override-only / internal | `docs/internals/env/inventory.md` |
| `SUBSTRATE_FORCE_PTY` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/dispatch/registry.rs:714` |
| `SUBSTRATE_FORWARDER_PIPE` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/backend.rs:64` |
| `SUBSTRATE_FORWARDER_TARGET` | `SUBSTRATE` | override-only / internal | `docs/internals/env/inventory.md` |
| `SUBSTRATE_FORWARDER_TARGET_ENDPOINT` | `SUBSTRATE` | override-only / internal | `crates/forwarder/src/wsl.rs:159` |
| `SUBSTRATE_FORWARDER_TARGET_HOST` | `SUBSTRATE` | override-only / internal | `crates/forwarder/src/wsl.rs:163` |
| `SUBSTRATE_FORWARDER_TARGET_MODE` | `SUBSTRATE` | override-only / internal | `crates/forwarder/src/wsl.rs:158` |
| `SUBSTRATE_FORWARDER_TARGET_PORT` | `SUBSTRATE` | override-only / internal | `crates/forwarder/src/wsl.rs:164` |
| `SUBSTRATE_FORWARDER_TCP` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/transport.rs:17` |
| `SUBSTRATE_FORWARDER_TCP_ADDR` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/transport.rs:10` |
| `SUBSTRATE_FORWARDER_TCP_HOST` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/transport.rs:25` |
| `SUBSTRATE_FORWARDER_TCP_PORT` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/transport.rs:27` |
| `SUBSTRATE_GRAPH` | `SUBSTRATE` | override-only / internal | `docs/internals/env/inventory.md` |
| `SUBSTRATE_GROUP` | `SUBSTRATE` | override-only / internal | `scripts/linux/world-provision.sh:57` |
| `SUBSTRATE_HOME` | `SUBSTRATE` | state (exported) | `crates/broker/src/profile/tests.rs:21` |
| `SUBSTRATE_INNER_LOGIN_SHELL` | `SUBSTRATE` | override-only / internal | `crates/world/src/exec.rs:212` |
| `SUBSTRATE_INSTALLER_EXPECT_SOCKET` | `SUBSTRATE` | override-only / internal | `tests/installers/install_smoke.sh:117` |
| `SUBSTRATE_INSTALL_ARCHIVE` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install-substrate.sh:23` |
| `SUBSTRATE_INSTALL_ARTIFACT_DIR` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install-substrate.sh:23` |
| `SUBSTRATE_INSTALL_BASE_URL` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install-substrate.sh:24` |
| `SUBSTRATE_INSTALL_GITHUB_TOKEN` | `SUBSTRATE` | override-only / internal | `docs/internals/env/inventory.md` |
| `SUBSTRATE_INSTALL_LATEST_API` | `SUBSTRATE` | override-only / internal | `docs/internals/env/inventory.md` |
| `SUBSTRATE_INSTALL_NO_PATH` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install-substrate.sh:1643` |
| `SUBSTRATE_INSTALL_PRIMARY_USER` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install-substrate.sh:118` |
| `SUBSTRATE_INSTALL_REF` | `SUBSTRATE` | override-only / internal | `docs/INSTALLATION.md:33` |
| `SUBSTRATE_INSTALL_WRAPPER_BASE_URL` | `SUBSTRATE` | override-only / internal | `scripts/substrate/install.sh:55` |
| `SUBSTRATE_LANDLOCK_HELPER_PATH` | `SUBSTRATE` | override-only / internal | `crates/world/src/exec.rs:184` |
| `SUBSTRATE_LANDLOCK_HELPER_SRC` | `SUBSTRATE` | override-only / internal | `crates/world/src/exec.rs:180` |
| `SUBSTRATE_LIMA_SKIP_GUEST_BUILD` | `SUBSTRATE` | override-only / internal | `scripts/mac/lima-warm.sh:12` |
| `SUBSTRATE_M5B_MANAGER_INIT_MARKER` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:67` |
| `SUBSTRATE_MANAGER_ENV` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing.rs:277` |
| `SUBSTRATE_MANAGER_ENV_ACTIVE` | `SUBSTRATE` | state (exported) | `crates/shell/src/execution/manager.rs:159` |
| `SUBSTRATE_MANAGER_INIT` | `SUBSTRATE` | state (exported) | `crates/shell/src/builtins/world_deps/runner.rs:138` |
| `SUBSTRATE_MANAGER_INIT_DEBUG` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/manager_init/config.rs:32` |
| `SUBSTRATE_MANAGER_INIT_SHELL` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/manager_init/runtime.rs:255` |
| `SUBSTRATE_MANAGER_MANIFEST` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/manager.rs:79` |
| `SUBSTRATE_MEM_LIMIT` | `SUBSTRATE` | override-only / internal | `docs/CONFIGURATION.md:458` |
| `SUBSTRATE_MOUNT_FS_MODE` | `SUBSTRATE` | override-only / internal | `crates/world/src/exec.rs:130` |
| `SUBSTRATE_NETNS_GC_INTERVAL_SECS` | `SUBSTRATE` | override-only / internal | `docs/WORLD.md:244` |
| `SUBSTRATE_NETNS_GC_TTL_SECS` | `SUBSTRATE` | override-only / internal | `crates/world-agent/src/handlers.rs:119` |
| `SUBSTRATE_NET_BUDGET` | `SUBSTRATE` | override-only / internal | `docs/CONFIGURATION.md:461` |
| `SUBSTRATE_NO_SHIMS` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/invocation/plan.rs:138` |
| `SUBSTRATE_ORIGINAL_BASH_ENV` | `SUBSTRATE` | override-only / internal | `crates/shell/src/builtins/world_deps/guest.rs:270` |
| `SUBSTRATE_OVERRIDE_ANCHOR_MODE` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_ANCHOR_PATH` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_CAGED` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_POLICY_MODE` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_SYNC_DIRECTION` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_OVERRIDE_WORLD` | `SUBSTRATE_OVERRIDE` | override input (planned) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_PARENT_SPAN` | `SUBSTRATE` | override-only / internal | `crates/telemetry-lib/src/correlation.rs:26` |
| `SUBSTRATE_POLICY_GIT_CACHE` | `SUBSTRATE` | override-only / internal | `crates/trace/src/util.rs:52` |
| `SUBSTRATE_POLICY_ID` | `SUBSTRATE` | override-only / internal | `crates/telemetry-lib/src/correlation.rs:29` |
| `SUBSTRATE_POLICY_MODE` | `SUBSTRATE` | dual-use (legacy) | `crates/broker/src/mode.rs:29` |
| `SUBSTRATE_PREFIX` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_enable.rs:21` |
| `SUBSTRATE_PROJECT_PATH` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/backend.rs:44` |
| `SUBSTRATE_PTY_DEBUG` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/pty/control.rs:106` |
| `SUBSTRATE_PTY_PIPELINE_LAST` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/dispatch/registry.rs:418` |
| `SUBSTRATE_REPLAY` | `SUBSTRATE` | override-only / internal | `crates/replay/src/replay/executor.rs:71` |
| `SUBSTRATE_REPLAY_USE_WORLD` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/replay.rs:67` |
| `SUBSTRATE_REPLAY_VERBOSE` | `SUBSTRATE` | override-only / internal | `crates/replay/src/replay/helpers.rs:35` |
| `SUBSTRATE_ROOT` | `SUBSTRATE` | state (exported) | `docs/internals/env/inventory.md` |
| `SUBSTRATE_SESSION_ID` | `SUBSTRATE` | override-only / internal | `crates/telemetry-lib/src/correlation.rs:24` |
| `SUBSTRATE_SHELL` | `SUBSTRATE` | override-only / internal | `crates/trace/src/span.rs:145` |
| `SUBSTRATE_SHIM_DEPLOY_DIR` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/support/mod.rs:81` |
| `SUBSTRATE_SHIM_HINTS` | `SUBSTRATE` | override-only / internal | `crates/shim/src/exec/logging.rs:83` |
| `SUBSTRATE_SHIM_ORIGINAL_PATH` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/support/mod.rs:80` |
| `SUBSTRATE_SHIM_PATH` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/shell_env.rs:77` |
| `SUBSTRATE_SKIP_MANAGER_INIT` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/manager_init/config.rs:25` |
| `SUBSTRATE_SKIP_MANAGER_INIT_LIST` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/manager_init/config.rs:28` |
| `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/socket_activation.rs:92` |
| `SUBSTRATE_SYNC_AUTO_SYNC` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:321` |
| `SUBSTRATE_SYNC_CONFLICT_POLICY` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:343` |
| `SUBSTRATE_SYNC_DIRECTION` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:331` |
| `SUBSTRATE_SYNC_EXCLUDE` | `SUBSTRATE` | dual-use (legacy) | `crates/shell/src/execution/config_model.rs:356` |
| `SUBSTRATE_TEST_CARGO_LOG` | `SUBSTRATE` | test/example | `tests/mac/installer_parity_fixture.sh:93` |
| `SUBSTRATE_TEST_CARGO_TARGET_ROOT` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:438` |
| `SUBSTRATE_TEST_FAKE_USER` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:130` |
| `SUBSTRATE_TEST_FILE_SENTINEL` | `SUBSTRATE` | test/example | `tests/mac/installer_parity_fixture.sh:94` |
| `SUBSTRATE_TEST_GROUP_ENTRY` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:325` |
| `SUBSTRATE_TEST_GROUP_EXISTS` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:135` |
| `SUBSTRATE_TEST_GROUP_LOG` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:131` |
| `SUBSTRATE_TEST_GROUP_MEMBERS` | `SUBSTRATE` | test/example | `tests/installers/install_state_smoke.sh:203` |
| `SUBSTRATE_TEST_HOME` | `SUBSTRATE` | test/example | `crates/common/src/manager_manifest/tests.rs:71` |
| `SUBSTRATE_TEST_LIMACTL_CAPTURE_DIR` | `SUBSTRATE` | test/example | `tests/mac/installer_parity_fixture.sh:91` |
| `SUBSTRATE_TEST_LIMACTL_LOG` | `SUBSTRATE` | test/example | `tests/mac/installer_parity_fixture.sh:90` |
| `SUBSTRATE_TEST_LINGER_LOG` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:132` |
| `SUBSTRATE_TEST_LINGER_STATE` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:136` |
| `SUBSTRATE_TEST_LINGER_STATE_OVERRIDE` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:136` |
| `SUBSTRATE_TEST_LOCAL_WORLD_ID` | `SUBSTRATE` | test/example | `crates/shell/src/execution/routing/dispatch/tests/linux_world.rs:14` |
| `SUBSTRATE_TEST_PRIMARY_USER` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:133` |
| `SUBSTRATE_TEST_SKIP_SOCKET` | `SUBSTRATE` | test/example | `crates/shell/tests/world_enable.rs:34` |
| `SUBSTRATE_TEST_SUBSTRATE_LOG` | `SUBSTRATE` | test/example | `tests/mac/installer_parity_fixture.sh:365` |
| `SUBSTRATE_TEST_SYSTEMCTL_LOG` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:220` |
| `SUBSTRATE_TEST_USER_GROUPS` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:134` |
| `SUBSTRATE_TEST_USER_GROUPS_OVERRIDE` | `SUBSTRATE` | test/example | `tests/installers/install_smoke.sh:134` |
| `SUBSTRATE_TEST_WORLD_EXIT` | `SUBSTRATE` | test/example | `crates/shell/tests/world_enable.rs:33` |
| `SUBSTRATE_TEST_WORLD_LOG` | `SUBSTRATE` | test/example | `crates/shell/tests/world_enable.rs:17` |
| `SUBSTRATE_TEST_WORLD_STDERR` | `SUBSTRATE` | test/example | `crates/shell/tests/world_enable.rs:29` |
| `SUBSTRATE_TEST_WORLD_STDOUT` | `SUBSTRATE` | test/example | `crates/shell/tests/world_enable.rs:25` |
| `SUBSTRATE_TRACE_LOG` | `SUBSTRATE` | override-only / internal | `crates/telemetry-lib/src/correlation.rs:30` |
| `SUBSTRATE_UNINSTALL_REF` | `SUBSTRATE` | override-only / internal | `scripts/substrate/uninstall.sh:89` |
| `SUBSTRATE_UNINSTALL_WRAPPER_BASE_URL` | `SUBSTRATE` | override-only / internal | `scripts/substrate/uninstall.sh:96` |
| `SUBSTRATE_WORLD` | `SUBSTRATE` | dual-use (legacy) | `crates/replay/src/replay/helpers.rs:19` |
| `SUBSTRATE_WORLD_AGENT_BIN` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/dispatch/world_ops.rs:412` |
| `SUBSTRATE_WORLD_DEPS_EXECUTOR_LOG` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:43` |
| `SUBSTRATE_WORLD_DEPS_FAIL_TOOL` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:45` |
| `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:50` |
| `SUBSTRATE_WORLD_DEPS_GUEST_LOG` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:31` |
| `SUBSTRATE_WORLD_DEPS_HOST_LOG` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:19` |
| `SUBSTRATE_WORLD_DEPS_MANIFEST` | `SUBSTRATE` | override-only / internal | `crates/shell/src/builtins/world_deps/runner.rs:52` |
| `SUBSTRATE_WORLD_DEPS_MARKER_DIR` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/world_deps.rs:49` |
| `SUBSTRATE_WORLD_ENABLED` | `SUBSTRATE` | override-only / internal | `crates/replay/src/replay/helpers.rs:22` |
| `SUBSTRATE_WORLD_ENABLE_SCRIPT` | `SUBSTRATE` | override-only / internal | `crates/shell/src/builtins/world_enable/runner.rs:58` |
| `SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR` | `SUBSTRATE` | override-only / internal | `crates/shell/src/builtins/world_enable/runner/verify.rs:61` |
| `SUBSTRATE_WORLD_FS_ISOLATION` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/platform/mod.rs:80` |
| `SUBSTRATE_WORLD_FS_MODE` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/platform/mod.rs:79` |
| `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST` | `SUBSTRATE` | override-only / internal | `crates/world/src/exec.rs:130` |
| `SUBSTRATE_WORLD_ID` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/platform_world/windows.rs:80` |
| `SUBSTRATE_WORLD_ROOT_MODE` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/replay_world.rs:1057` |
| `SUBSTRATE_WORLD_ROOT_PATH` | `SUBSTRATE` | override-only / internal | `crates/shell/tests/replay_world.rs:1058` |
| `SUBSTRATE_WORLD_SOCKET` | `SUBSTRATE` | override-only / internal | `crates/replay/src/replay/executor.rs:886` |
| `SUBSTRATE_WSL_DISTRO` | `SUBSTRATE` | override-only / internal | `crates/world-windows-wsl/src/backend.rs:43` |
| `SUBSTRATE_WS_DEBUG` | `SUBSTRATE` | override-only / internal | `crates/shell/src/execution/routing/dispatch/world_ops.rs:127` |
| `TEMP` | `standard` | standard | `docs/cross-platform/wsl_world_troubleshooting.md:121` |
| `TERM` | `standard` | standard | `crates/shell/src/execution/pty/io/runner.rs:111` |
| `TEST_ENV_KEY` | `standard` | standard | `crates/shell/src/execution/manager_init/tests.rs:63` |
| `TEST_MODE` | `standard` | standard | `crates/shell/src/execution/routing/dispatch/registry.rs:406` |
| `TIER2_HOME` | `standard` | standard | `crates/common/src/manager_manifest/tests.rs:324` |
| `TMPDIR` | `standard` | standard | `crates/shell/tests/common.rs:20` |
| `TRACE_LOG_FILE` | `TRACE` | internal/trace | `docs/internals/env/inventory.md` |
| `TRACE_LOG_KEEP` | `TRACE` | internal/trace | `crates/trace/src/output.rs:41` |
| `TRACE_LOG_MAX_MB` | `TRACE` | internal/trace | `crates/shell/tests/logging.rs:173` |
| `TRACE_LOG_PATH` | `TRACE` | internal/trace | `scripts/substrate/dev-uninstall-substrate.sh:478` |
| `UNSET_ME` | `standard` | standard | `crates/shell/src/execution/routing/builtin/tests.rs:100` |
| `USER` | `standard` | standard | `crates/replay/src/state.rs:303` |
| `USERNAME` | `standard` | standard | `crates/shim/src/logger.rs:82` |
| `USERPROFILE` | `standard` | standard | `crates/shell/src/execution/invocation/plan.rs:491` |
| `VOLTA_HOME` | `standard` | standard | `config/manager_hooks.yaml:354` |
| `VOLTA_MARKER` | `standard` | standard | `crates/shell/tests/manager_init.rs:59` |
| `WORLD_CAGED` | `WORLD` | internal/world | `scripts/substrate/dev-install-substrate.sh:62` |
| `WORLD_ENABLED` | `WORLD` | internal/world | `scripts/substrate/dev-install-substrate.sh:195` |
| `WORLD_ID` | `WORLD` | internal/world | `docs/WORLD.md:130` |
| `WORLD_PROVISION_FAILED` | `WORLD` | internal/world | `scripts/substrate/dev-install-substrate.sh:972` |
| `XDG_CACHE_HOME` | `standard` | standard | `crates/world/src/exec.rs:203` |
| `XDG_CONFIG_HOME` | `standard` | standard | `crates/world/src/exec.rs:204` |
| `XDG_RUNTIME_DIR` | `standard` | standard | `crates/shell/src/execution/invocation/plan.rs:360` |
