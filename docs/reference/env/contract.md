# Supported Environment Variable Contract (Operator-Facing)

Entry point: `docs/ENVIRONMENT_VARIABLES.md`  
Exhaustive inventory (not a contract): `docs/internals/env/inventory.md`

This document is the supported, operator-facing environment variable contract for Substrate. The repository implementation is the source of truth; this contract matches the current behavior of:

- Config resolution: `crates/shell/src/execution/config_model.rs`
- Invocation planning + env preparation: `crates/shell/src/execution/invocation/plan.rs`
- Shim behavior: `crates/shim/**`
- World backends + services: `crates/world-agent/**`, `crates/world-*/**`, `crates/forwarder/**`, `crates/host-proxy/**`
- Install/provision scripts: `scripts/substrate/**`, `scripts/mac/**`, `scripts/windows/**`

Only environment variables listed under **Supported Variables** are supported operator interfaces. Any variable not listed here is unsupported unless explicitly documented as “not supported”.

## Taxonomy

### Config override inputs (supported)
Operator-provided values that are read by a Substrate component and change observable behavior.

### Diagnostics toggles (supported, scoped)
Operator-provided values that enable/disable debugging, change logging/trace behavior, or relax safety guarantees. These are supported but explicitly scoped; they are not part of the stable “configuration schema”.

### Exported state variables (supported outputs; not override inputs)
Variables written by Substrate components (shell/shim/scripts) to describe the current session and propagate state to child processes. When a variable is “exported state”, operator-set values are overwritten by Substrate and are not treated as supported override inputs.

### Internal variables (not supported)
Variables used for internal coordination, tests, harnesses, and experiments. Setting them is unsupported and excluded from compatibility guarantees.

## Env vs config vs CLI (authoritative precedence)

### Substrate CLI effective config (`substrate`)
Substrate resolves an “effective config” for each invocation:

1. **Defaults** (built in)
2. **Global config file** at `$SUBSTRATE_HOME/config.yaml` (or `~/.substrate/config.yaml` if `SUBSTRATE_HOME` is unset)
3. **Environment overrides** via `SUBSTRATE_OVERRIDE_*` config variables (listed below)
4. **Workspace config file** at `<workspace_root>/.substrate/workspace.yaml` if a workspace root is detected
   - This workspace config **replaces** the effective config and discards global+env values.
5. **CLI flags** (when a flag exists for the setting) override the resolved config

Failure behavior for config-shaped env vars:
- Invalid values raise a user-facing error (printed to stderr).
- Exit code depends on the command surface that encountered the error:
  - Shell entrypoints (no subcommand): exit code `1` (error returned from invocation planning).
  - Config/policy commands: exit code `2` (user/config error).

### Other binaries/services
Components such as `substrate-shim`, `substrate-world-agent`, `host-proxy`, and `forwarder` do not consult the Substrate YAML config. Their behavior is controlled by their own supported environment variables listed below.

## Supported Variables

Each entry below is a stability promise: the variable name, parsing rules, and semantics are supported for operators.

### Config Override Inputs (Substrate CLI effective config)

#### `SUBSTRATE_HOME`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | OS string path; empty is treated as unset |
| Default if unset | `~/.substrate` (resolved via OS home directory) |
| Precedence | Process env only (no CLI flag); affects all config/path lookups |
| Scope | Global |
| Examples | `export SUBSTRATE_HOME=/srv/substrate` |
| Security notes | Value is written into `$SUBSTRATE_HOME/env.sh` when generated; it is not treated as sensitive. |

#### `SUBSTRATE_OVERRIDE_WORLD`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Case-insensitive string enum: `enabled` or `disabled` (empty after trim is ignored) |
| Default if unset | Enabled (subject to config/policy and `--no-world`) |
| Precedence | `--world` / `--no-world` flags override; workspace config overrides env; env overrides global config when no workspace config exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_WORLD=disabled substrate -c 'echo host-only'` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_ANCHOR_MODE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Case-insensitive string enum parsed by `WorldRootMode`: `workspace`, `follow-cwd`, `custom` (empty after trim is ignored) |
| Default if unset | `workspace` |
| Precedence | `--anchor-mode` flag overrides; workspace config overrides env; env overrides global config when no workspace config exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_ANCHOR_MODE=follow-cwd substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_ANCHOR_PATH`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path (no validation at parse time); when `SUBSTRATE_OVERRIDE_ANCHOR_MODE=custom`, the resolved path must exist and be a directory |
| Default if unset | Empty string in config; runtime resolution uses the launch directory when not in `custom` mode |
| Precedence | `--anchor-path` flag overrides; workspace config overrides env; env overrides global config when no workspace config exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_ANCHOR_MODE=custom SUBSTRATE_OVERRIDE_ANCHOR_PATH=/srv/project substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_CAGED`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Boolean parsed by `parse_bool_flag`: `true|false|1|0|yes|no|on|off` (case-insensitive) |
| Default if unset | `true` |
| Precedence | `--caged` / `--uncaged` flags override; workspace config overrides env; env overrides global config when no workspace config exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_CAGED=0 substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_POLICY_MODE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Case-insensitive string enum: `disabled`, `observe`, `enforce` (empty after trim is ignored) |
| Default if unset | `observe` |
| Precedence | Workspace config overrides env; env overrides global config when no workspace config exists; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_POLICY_MODE=enforce substrate -c 'rm -rf /tmp/deny-me'` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Boolean parsed by `parse_bool_flag`: `true|false|1|0|yes|no|on|off` (case-insensitive) |
| Default if unset | `false` |
| Precedence | Workspace config overrides env; env overrides global config when no workspace config exists; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC=1 substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_SYNC_DIRECTION`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Case-insensitive string enum: `from_world`, `from_host`, `both` (empty after trim is ignored) |
| Default if unset | `from_world` |
| Precedence | Workspace config overrides env; env overrides global config when no workspace config exists; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_SYNC_DIRECTION=both substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Case-insensitive string enum: `prefer_host`, `prefer_world`, `abort` (empty after trim is ignored) |
| Default if unset | `prefer_host` |
| Precedence | Workspace config overrides env; env overrides global config when no workspace config exists; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY=abort substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Comma-separated list; split on `,`, trimmed; empty items are dropped |
| Default if unset | No user excludes; Substrate always injects protected excludes: `.git/**`, `.substrate/**`, `.substrate-git/**` |
| Precedence | Workspace config overrides env; env overrides global config when no workspace config exists; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE='node_modules,dist' substrate` |
| Security notes | Values are recorded in trace metadata when tracing is enabled; access to trace logs reveals the configured patterns. |

### Config Override Inputs (Substrate CLI auxiliary manifests)

#### `SUBSTRATE_MANAGER_MANIFEST`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path (no validation at parse time) |
| Default if unset | Installed path `<prefix>/versions/<version>/config/manager_hooks.yaml` when installed; otherwise repo `config/manager_hooks.yaml` |
| Precedence | Env overrides built-in path detection; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_MANAGER_MANIFEST=/tmp/manager_hooks.yaml substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_WORLD_DEPS_MANIFEST`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path; when set (even to an empty string), `substrate world deps …` treats the override as required |
| Default if unset | Installed path `<prefix>/versions/<version>/config/world-deps.yaml` when installed; otherwise repo `scripts/substrate/world-deps.yaml` |
| Precedence | Env overrides built-in path detection; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_WORLD_DEPS_MANIFEST=/srv/world-deps.yaml substrate world deps status --json` |
| Security notes | Not sensitive. |

### Config Override Inputs (world backend endpoints)

#### `SUBSTRATE_WORLD_SOCKET`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | OS string path to a Unix domain socket; empty strings are treated as a path and break connectivity |
| Default if unset | Linux: `/run/substrate.sock` |
| Precedence | Env overrides the default socket path; no CLI flag exists |
| Scope | Run-only |
| Examples | `SUBSTRATE_WORLD_SOCKET=/run/substrate.sock substrate host doctor --json` |
| Security notes | Not sensitive; controls which agent socket receives execution requests. On macOS, setting this also bypasses Lima transport detection and connects directly to the provided Unix socket path. |

#### `SUBSTRATE_AGENT_ID`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String; not validated |
| Default if unset | `human` |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_AGENT_ID=ci substrate -c 'echo hello'` |
| Security notes | Recorded as the `agent_id` in agent execution requests and in downstream telemetry; do not embed secrets. |

### Diagnostics Toggles (Substrate CLI + shim + replay)

#### `SUBSTRATE_NO_SHIMS`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Presence-based flag: if set to any value (including `0`), shims are not deployed/used |
| Default if unset | Shims are auto-deployed (best-effort) and used when world is enabled |
| Precedence | `--shim-skip` flag OR `SUBSTRATE_NO_SHIMS` disables deployment |
| Scope | Run-only |
| Examples | `SUBSTRATE_NO_SHIMS=1 substrate --shim-status-json` |
| Security notes | Disabling shims reduces observability; fewer executions are traced via shim interception. |

#### `SUBSTRATE_FORCE_PTY`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Presence-based flag: if set to any value, Substrate allocates a PTY for every executed command |
| Default if unset | PTY is used only when `needs_pty()` heuristics detect an interactive/TUI command |
| Precedence | `:pty <cmd>` and `SUBSTRATE_FORCE_PTY` override `SUBSTRATE_DISABLE_PTY` |
| Scope | Run-only |
| Examples | `SUBSTRATE_FORCE_PTY=1 substrate -c 'vim --version'` |
| Security notes | PTY mode changes TTY detection for executed programs and increases PTY routing work and related logs. |

#### `SUBSTRATE_DISABLE_PTY`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Presence-based flag: if set to any value, PTY allocation is disabled unless forced |
| Default if unset | PTY is used when needed by heuristics or forced |
| Precedence | Overridden by `:pty <cmd>` and `SUBSTRATE_FORCE_PTY` |
| Scope | Run-only |
| Examples | `SUBSTRATE_DISABLE_PTY=1 substrate -c 'less README.md'` |
| Security notes | Disabling PTY breaks interactive programs that require a TTY; it does not disable tracing. |

#### `SUBSTRATE_PTY_DEBUG`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Presence-based flag: if set to any value, PTY debug logs are emitted |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_PTY_DEBUG=1 substrate --pty` |
| Security notes | Debug logging includes terminal sizing and routing details; it does not redact command strings. |

#### `SUBSTRATE_PTY_PIPELINE_LAST`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Presence-based flag: if set to any value, Substrate checks the last pipeline segment for PTY requirements |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_PTY_PIPELINE_LAST=1 substrate -c 'cmd | fzf'` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_REPLAY_VERBOSE`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | String flag: enabled only when the value is exactly `1` |
| Default if unset | Off |
| Precedence | Enabled when `SUBSTRATE_REPLAY_VERBOSE=1` OR `--replay-verbose` is set |
| Scope | Run-only (replay command) |
| Examples | `SUBSTRATE_REPLAY_VERBOSE=1 substrate --replay spn_...` |
| Security notes | Verbose replay prints additional command/cwd/context details to stderr. |

#### `SUBSTRATE_REPLAY_USE_WORLD`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Disable-only string: case-insensitive `0`, `false`, or `disabled` forces host replay; any other value has no effect |
| Default if unset | Replay uses the recorded execution origin, optionally flipped by `--flip-world` |
| Precedence | `--world` overrides, then `--no-world`, then this env var |
| Scope | Run-only (replay command) |
| Examples | `SUBSTRATE_REPLAY_USE_WORLD=disabled substrate --replay spn_...` |
| Security notes | Host-only replay reduces isolation; filesystem diffs and network scoping may differ. |

#### `SUBSTRATE_COPYDIFF_ROOT`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | OS string path; empty is treated as unset |
| Default if unset | Auto-selected scratch root (platform-dependent); Substrate tries multiple candidates |
| Precedence | If set and non-empty, the override is tried first; Substrate still falls back to other candidates on failure |
| Scope | Run-only (local backend / copy-diff fallback) |
| Examples | `SUBSTRATE_COPYDIFF_ROOT=/var/tmp/substrate-copydiff substrate --replay spn_...` |
| Security notes | Points at a scratch directory that will contain copies of project files while executing copy-diff. |

#### `SUBSTRATE_SHIM_HINTS`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Disable values (case-insensitive): `0`, `false`, `off`, `disabled`; any other set value forces hints on |
| Default if unset | Hints are enabled only when world features are enabled |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_SHIM_HINTS=0 substrate -c 'npm install'` |
| Security notes | Hint content is derived from known manager error patterns and is not redacted. |

#### `SUBSTRATE_POLICY_GIT_CACHE`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Disable values: exactly `0` or case-insensitive `false`; all other values keep caching enabled |
| Default if unset | Caching enabled |
| Precedence | Env only |
| Scope | Run-only (per-process) |
| Examples | `SUBSTRATE_POLICY_GIT_CACHE=0 substrate --trace spn_...` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_SKIP_MANAGER_INIT`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Truthy values (case-insensitive): `1`, `true`, `yes`, `on`; all other values are false |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_SKIP_MANAGER_INIT=1 substrate` |
| Security notes | Disabling manager init changes shell environment initialization; it does not disable tracing. |

#### `SUBSTRATE_SKIP_MANAGER_INIT_LIST`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Comma/whitespace separated list; items are lowercased; matching is against the manifest manager names lowercased |
| Default if unset | Empty (skip none) |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_SKIP_MANAGER_INIT_LIST='nvm, pyenv' substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_MANAGER_INIT_DEBUG`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Truthy values (case-insensitive): `1`, `true`, `yes`, `on`; all other values are false |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_MANAGER_INIT_DEBUG=1 substrate` |
| Security notes | Enables debug logging that includes detected managers and detection reasons. |

#### `SUBSTRATE_MANAGER_INIT_SHELL`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | OS string path to a shell executable; empty is treated as unset; existence is not validated at selection time |
| Default if unset | `$SHELL` when absolute+exists; otherwise first existing of `/bin/sh`, `/usr/bin/sh`, `/system/bin/sh`; otherwise `sh` |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_MANAGER_INIT_SHELL=/usr/local/bin/bash substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_WS_DEBUG`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Enabled only when the value is exactly `1` |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only (Linux world-agent PTY WS route) |
| Examples | `SUBSTRATE_WS_DEBUG=1 substrate --pty -c 'bash'` |
| Security notes | Prints routing diagnostics to stderr. |

#### `SHIM_TRACE_LOG`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle (dual-use export) |
| Type / allowed values | String path |
| Default if unset | `~/.substrate/trace.jsonl` (or `/tmp/.substrate/trace.jsonl` if no home directory is available) |
| Precedence | Env only; when running `substrate`, it sets `SHIM_TRACE_LOG` for child processes |
| Scope | Global (per-process) |
| Examples | `SHIM_TRACE_LOG=/tmp/trace.jsonl substrate -c 'echo hi'` |
| Security notes | Trace logs contain command strings and execution metadata; keep the file permissions restricted. |

#### `SHIM_ORIGINAL_PATH`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle (dual-use export) |
| Type / allowed values | PATH string; parsed using OS path splitting (`:` on Unix, `;` on Windows) |
| Default if unset | `PATH` |
| Precedence | When running `substrate`, the shell computes a clean original path and sets `SHIM_ORIGINAL_PATH` for child processes |
| Scope | Run-only |
| Examples | `SHIM_ORIGINAL_PATH=/usr/bin:/bin PATH="$HOME/.substrate/shims:$PATH" git status` |
| Security notes | Not sensitive; influences which real binaries the shim resolves and executes. |

#### `SHIM_SESSION_ID`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle (dual-use export) |
| Type / allowed values | String; not validated (Substrate uses UUIDv7 by default) |
| Default if unset | Auto-generated UUIDv7 per Substrate invocation |
| Precedence | If set before launching `substrate`, the provided value is used for the session; otherwise Substrate generates and exports one |
| Scope | Run-only |
| Examples | `SHIM_SESSION_ID=my-session substrate -c 'echo hi'` |
| Security notes | Recorded in trace entries as the session identifier; do not embed secrets. |

#### `SHIM_BYPASS`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Enabled only when the value is exactly `1` |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only (shim processes) |
| Examples | `SHIM_BYPASS=1 git status` |
| Security notes | Disables shim logging and tracing for the affected command invocation. |

#### `SHIM_LOG_OPTS`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Supported value: `raw` |
| Default if unset | Redaction enabled |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SHIM_LOG_OPTS=raw curl -H 'Authorization: Bearer secret' https://example.com` |
| Security notes | `raw` disables credential redaction and writes secrets into trace/log output. |

#### `SHIM_FSYNC`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Enabled only when the value is exactly `1` |
| Default if unset | Off |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SHIM_FSYNC=1 substrate -c 'echo hi'` |
| Security notes | Forces synchronous writes for the trace log; reduces performance. |

#### `TRACE_LOG_MAX_MB`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Unsigned integer (`u64`) megabytes; invalid values are ignored |
| Default if unset | `100` |
| Precedence | Env only |
| Scope | Global (per-process) |
| Examples | `TRACE_LOG_MAX_MB=10 substrate` |
| Security notes | Smaller values rotate more frequently; old rotated logs remain on disk per `TRACE_LOG_KEEP`. |

#### `TRACE_LOG_KEEP`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Unsigned integer (`usize`) count; invalid values are ignored |
| Default if unset | `3` |
| Precedence | Env only |
| Scope | Global (per-process) |
| Examples | `TRACE_LOG_KEEP=10 substrate` |
| Security notes | Controls how many rotated trace log files are retained on disk. |

#### `SHIM_TRACE_LOG_MAX_MB` (legacy)
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Unsigned integer (`u64`) megabytes; invalid values are ignored |
| Default if unset | None; `TRACE_LOG_MAX_MB` default applies |
| Precedence | Used only when `TRACE_LOG_MAX_MB` is unset |
| Scope | Global (per-process) |
| Examples | `SHIM_TRACE_LOG_MAX_MB=50 substrate` |
| Security notes | Legacy alias for `TRACE_LOG_MAX_MB`. |

### Config Override Inputs (world-agent service)

#### `SUBSTRATE_AGENT_TCP_PORT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer port `1..65535`; empty/0/invalid values are errors |
| Default if unset | TCP listener disabled (UDS via `/run/substrate.sock` remains) |
| Precedence | Env only; ignored when TCP listeners are inherited via socket activation |
| Scope | Global (service process) |
| Examples | `SUBSTRATE_AGENT_TCP_PORT=61337 substrate-world-agent` |
| Security notes | Binds a loopback TCP listener; treat as local-only unless you intentionally expose it. |

#### `SUBSTRATE_NETNS_GC_TTL_SECS`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Unsigned integer seconds (`u64`); invalid values use the default |
| Default if unset | `0` (TTL disabled) |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `SUBSTRATE_NETNS_GC_TTL_SECS=3600 substrate-world-agent` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_NETNS_GC_INTERVAL_SECS`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Unsigned integer seconds (`u64`); invalid values use the default |
| Default if unset | `600` (10 minutes); `0` disables periodic sweeps |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `SUBSTRATE_NETNS_GC_INTERVAL_SECS=0 substrate-world-agent` |
| Security notes | Not sensitive. |

### Config Override Inputs (host-proxy service)

#### `HOST_PROXY_SOCKET`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path |
| Default if unset | `~/.substrate/sock/agent.sock` |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `HOST_PROXY_SOCKET=/tmp/host-proxy.sock host-proxy` |
| Security notes | Socket permissions control which users are permitted to submit requests to the proxy. |

#### `SUBSTRATE_AGENT_TRANSPORT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Transport URI parsed by `AgentTransportConfig::from_uri`: `unix://<path>`, `tcp://<host>[:<port>]` (Windows: `named-pipe://…`) |
| Default if unset | Platform default (Unix: `unix:///run/substrate.sock`; Windows: named pipe) |
| Precedence | Overrides `AGENT_TRANSPORT` and `AGENT_SOCKET` when set and non-empty |
| Scope | Global (service process) |
| Examples | `SUBSTRATE_AGENT_TRANSPORT=unix:///run/substrate.sock host-proxy` |
| Security notes | Not sensitive. |

#### `AGENT_TRANSPORT` (compat)
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Transport URI (same as `SUBSTRATE_AGENT_TRANSPORT`) OR keywords `tcp`, `unix`/`uds` |
| Default if unset | Not set (falls back to `AGENT_SOCKET` or service default) |
| Precedence | Used only when `SUBSTRATE_AGENT_TRANSPORT` is unset or empty |
| Scope | Global (service process) |
| Examples | `AGENT_TRANSPORT=tcp AGENT_TCP_HOST=127.0.0.1 AGENT_TCP_PORT=61337 host-proxy` |
| Security notes | Not sensitive. |

#### `AGENT_TCP_HOST`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String host |
| Default if unset | `127.0.0.1` |
| Precedence | Consulted when `AGENT_TRANSPORT=tcp` (keyword form) |
| Scope | Global (service process) |
| Examples | `AGENT_TRANSPORT=tcp AGENT_TCP_HOST=127.0.0.1 AGENT_TCP_PORT=61337 host-proxy` |
| Security notes | Not sensitive. |

#### `AGENT_TCP_PORT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer `u16`; invalid values use the default |
| Default if unset | `17788` |
| Precedence | Consulted when `AGENT_TRANSPORT=tcp` (keyword form) |
| Scope | Global (service process) |
| Examples | `AGENT_TRANSPORT=tcp AGENT_TCP_PORT=61337 host-proxy` |
| Security notes | Not sensitive. |

#### `AGENT_SOCKET`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path |
| Default if unset | Not set (falls back to service default) |
| Precedence | Used when `SUBSTRATE_AGENT_TRANSPORT` and `AGENT_TRANSPORT` are unset/empty; also used when `AGENT_TRANSPORT=unix|uds` (keyword form) |
| Scope | Global (service process) |
| Examples | `AGENT_SOCKET=/run/substrate.sock host-proxy` |
| Security notes | Not sensitive. |

#### `MAX_BODY_SIZE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer bytes (`usize`); invalid values use the default |
| Default if unset | `10485760` (10 MiB) |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `MAX_BODY_SIZE=2097152 host-proxy` |
| Security notes | Not sensitive. |

#### `REQUEST_TIMEOUT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer seconds (`u64`); invalid values use the default |
| Default if unset | `30` |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `REQUEST_TIMEOUT=10 host-proxy` |
| Security notes | Not sensitive. |

#### `RATE_LIMIT_RPM`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer (`u32`); invalid values fall back to `60` |
| Default if unset | `60` |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `RATE_LIMIT_RPM=120 host-proxy` |
| Security notes | Not sensitive. |

#### `RATE_LIMIT_CONCURRENT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer (`u32`); invalid values fall back to `5` |
| Default if unset | `5` |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `RATE_LIMIT_CONCURRENT=2 host-proxy` |
| Security notes | Not sensitive. |

#### `AUTH_ENABLED`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Rust `bool` parse: case-insensitive `true` or `false`; invalid values are treated as `false` |
| Default if unset | `false` |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `AUTH_ENABLED=true AUTH_TOKEN_FILE=/etc/substrate/host-proxy.token host-proxy` |
| Security notes | Authentication secrets are expected to be stored in `AUTH_TOKEN_FILE`, not in this env var. |

#### `AUTH_TOKEN_FILE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path |
| Default if unset | None |
| Precedence | Env only |
| Scope | Global (service process) |
| Examples | `AUTH_ENABLED=true AUTH_TOKEN_FILE=/etc/substrate/host-proxy.token host-proxy` |
| Security notes | File contents are sensitive; ensure filesystem permissions are restricted. |

### Config Override Inputs (WSL world backend on Windows)

#### `SUBSTRATE_WSL_DISTRO`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String distro name |
| Default if unset | `substrate-wsl` |
| Precedence | Env only |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_WSL_DISTRO=substrate-wsl pwsh -File scripts/windows/wsl-warm.ps1` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_PROJECT_PATH`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path |
| Default if unset | Current working directory |
| Precedence | Env only |
| Scope | Run-only |
| Examples | `SUBSTRATE_PROJECT_PATH=C:\\code\\project substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_FORWARDER_PIPE`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path to Windows named pipe |
| Default if unset | `\\\\.\\pipe\\substrate-agent` |
| Precedence | Env only |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_FORWARDER_PIPE=\\\\.\\pipe\\substrate-agent substrate` |
| Security notes | Named pipe permissions control which users are permitted to connect. |

#### `SUBSTRATE_FORWARDER_TCP_ADDR`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Socket address parsed as `SocketAddr` (e.g. `127.0.0.1:17788`); invalid values are errors |
| Default if unset | Unset (TCP is controlled by `SUBSTRATE_FORWARDER_TCP`) |
| Precedence | Overrides `SUBSTRATE_FORWARDER_TCP*` when set |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_FORWARDER_TCP_ADDR=127.0.0.1:17788 substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_FORWARDER_TCP`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Truthy values (case-insensitive): `1`, `true`, `yes`; all other values disable TCP |
| Default if unset | Disabled |
| Precedence | Used only when `SUBSTRATE_FORWARDER_TCP_ADDR` is unset |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_FORWARDER_TCP=1 SUBSTRATE_FORWARDER_TCP_PORT=17788 substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_FORWARDER_TCP_HOST`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String host |
| Default if unset | `127.0.0.1` |
| Precedence | Used when `SUBSTRATE_FORWARDER_TCP=1` |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_FORWARDER_TCP=1 SUBSTRATE_FORWARDER_TCP_HOST=127.0.0.1 substrate` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_FORWARDER_TCP_PORT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Integer `u16`; invalid values use the default |
| Default if unset | `17788` |
| Precedence | Used when `SUBSTRATE_FORWARDER_TCP=1` |
| Scope | Global (Windows host process) |
| Examples | `SUBSTRATE_FORWARDER_TCP=1 SUBSTRATE_FORWARDER_TCP_PORT=17788 substrate` |
| Security notes | Not sensitive. |

### Config Override Inputs (WSL forwarder)

#### `SUBSTRATE_FORWARDER_TARGET`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | `tcp[:PORT]` or `uds[:PATH]` (mode is required; value is optional); invalid mode/port is an error |
| Default if unset | Uses config file (if present) or defaults to `tcp:61337` |
| Precedence | Env overrides the forwarder config file |
| Scope | Global (forwarder process) |
| Examples | `SUBSTRATE_FORWARDER_TARGET=uds:/run/substrate.sock substrate-forwarder` |
| Security notes | Not sensitive. |

### Config Override Inputs (install/uninstall + provisioning scripts)

#### `SUBSTRATE_ROOT`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path |
| Default if unset | `~/.substrate` |
| Precedence | Used by uninstall scripts as the prefix root; no CLI flag exists |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_ROOT=/opt/substrate scripts/substrate/uninstall.sh` |
| Security notes | Not sensitive; points to the installation root that will be removed/modified. |

#### `SUBSTRATE_INSTALL_REF`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String git ref/tag used by `scripts/substrate/install.sh` when fetching helper scripts |
| Default if unset | Resolved from `--version` (if present) or GitHub latest release tag; falls back to `main` when tag resolution fails |
| Precedence | Env overrides the wrapper’s version-ref detection |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_REF=v0.2.5 ./scripts/substrate/install.sh --version 0.2.5` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_INSTALL_WRAPPER_BASE_URL`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String base URL used by `scripts/substrate/install.sh` to fetch helper assets |
| Default if unset | `https://raw.githubusercontent.com/atomize-hq/substrate/<ref>/scripts/substrate` |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_WRAPPER_BASE_URL=https://example.invalid/substrate/scripts/substrate ./scripts/substrate/install.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_INSTALL_LATEST_API`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String URL queried by `scripts/substrate/install-substrate.sh` to resolve the latest release tag |
| Default if unset | `https://api.github.com/repos/atomize-hq/substrate/releases/latest` |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_LATEST_API=https://api.github.com/repos/atomize-hq/substrate/releases/latest ./scripts/substrate/install-substrate.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_INSTALL_GITHUB_TOKEN`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String bearer token |
| Default if unset | Unset (unauthenticated GitHub API requests) |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_GITHUB_TOKEN=… ./scripts/substrate/install-substrate.sh` |
| Security notes | Sensitive; the installer passes it to `curl` via an `Authorization:` header argument, which is visible to other users via process inspection on many systems. |

#### `SUBSTRATE_INSTALL_BASE_URL`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String base URL used to download release artifacts |
| Default if unset | `https://github.com/atomize-hq/substrate/releases/download` |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_BASE_URL=https://github.com/atomize-hq/substrate/releases/download ./scripts/substrate/install-substrate.sh --version 0.2.5` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_INSTALL_ARTIFACT_DIR`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String path to a directory containing pre-downloaded artifacts (bundle + `SHA256SUMS`) |
| Default if unset | Unset |
| Precedence | Used as the initial default; `--artifact-dir` / `--archive` CLI args override it |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_ARTIFACT_DIR=/tmp/substrate-artifacts ./scripts/substrate/install-substrate.sh --version 0.2.5` |
| Security notes | Not sensitive; directory contents include executables and checksums. |

#### `SUBSTRATE_INSTALL_ARCHIVE` (legacy)
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Same as `SUBSTRATE_INSTALL_ARTIFACT_DIR` |
| Default if unset | Unset |
| Precedence | Used only when `SUBSTRATE_INSTALL_ARTIFACT_DIR` is unset |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_ARCHIVE=/tmp/substrate-artifacts ./scripts/substrate/install-substrate.sh` |
| Security notes | Legacy alias for `SUBSTRATE_INSTALL_ARTIFACT_DIR`. |

#### `SUBSTRATE_INSTALL_PRIMARY_USER`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String username |
| Default if unset | Derived from the invoking user (sudo user when running under sudo; otherwise the current user) |
| Precedence | Env overrides automatic detection |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_PRIMARY_USER=alice ./scripts/substrate/install-substrate.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_INSTALL_NO_PATH`
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Enabled only when the value is exactly `1` |
| Default if unset | Off (installer updates shell rc PATH snippets when applicable) |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_INSTALL_NO_PATH=1 ./scripts/substrate/install-substrate.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_UNINSTALL_REF`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String git ref/tag used by `scripts/substrate/uninstall.sh` when fetching helper scripts |
| Default if unset | Falls back to `SUBSTRATE_INSTALL_REF`; otherwise resolves GitHub latest release tag; falls back to `main` when tag resolution fails |
| Precedence | Env overrides the wrapper’s ref selection |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_UNINSTALL_REF=v0.2.5 ./scripts/substrate/uninstall.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_UNINSTALL_WRAPPER_BASE_URL`
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | String base URL used by `scripts/substrate/uninstall.sh` to fetch helper assets |
| Default if unset | `https://raw.githubusercontent.com/atomize-hq/substrate/<ref>/scripts/substrate` |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_UNINSTALL_WRAPPER_BASE_URL=https://example.invalid/substrate/scripts/substrate ./scripts/substrate/uninstall.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_LIMA_SKIP_GUEST_BUILD` (macOS provisioning)
| Field | Value |
| --- | --- |
| Bucket | Diagnostics toggle |
| Type / allowed values | Integer string; the script expects `0` or `1` and aborts on non-integers (`set -euo pipefail` + `-eq`) |
| Default if unset | `0` |
| Precedence | Env only |
| Scope | Global (script invocation) |
| Examples | `SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 scripts/mac/lima-warm.sh` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_EXE` (Windows WSL smoke script)
| Field | Value |
| --- | --- |
| Bucket | Config override input |
| Type / allowed values | Windows path to `substrate.exe` |
| Default if unset | `scripts/windows/wsl-smoke.ps1` auto-discovers `target\\{debug|release}\\substrate.exe` under the provided project path |
| Precedence | Env overrides discovery |
| Scope | Global (script invocation) |
| Examples | `setx SUBSTRATE_EXE C:\\code\\substrate\\target\\release\\substrate.exe` |
| Security notes | Not sensitive. |

### Exported State Variables (written by Substrate)

#### `SUBSTRATE_WORLD`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | Case-insensitive string enum: `enabled` or `disabled` |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_WORLD"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_WORLD_ENABLED`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String `1` (enabled) or `0` (disabled) |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_WORLD_ENABLED"` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_ANCHOR_MODE`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | Canonical strings: `workspace`, `follow-cwd`, `custom` |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_ANCHOR_MODE"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_ANCHOR_PATH`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String path (may be empty) |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_ANCHOR_PATH"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_CAGED`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String `1` (caged) or `0` (uncaged) |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_CAGED"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_POLICY_MODE`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | Canonical strings: `disabled`, `observe`, `enforce` |
| Default if unset | Not relied on; Substrate sets it during invocation planning |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_POLICY_MODE"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_WORLD_ID`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String world identifier; unset when the world-agent backend is used on Linux |
| Default if unset | Unset until a local backend session is created |
| Precedence | Written by Substrate; operator-set values are overwritten/cleared |
| Scope | Run-only |
| Examples | `echo "${SUBSTRATE_WORLD_ID:-}"` |
| Security notes | Not sensitive; recorded in trace metadata when tracing is enabled. |

#### `SUBSTRATE_WORLD_FS_MODE`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | Canonical strings: `writable` or `read_only` |
| Default if unset | Not relied on; Substrate sets it from policy for each command |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_WORLD_FS_MODE"` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_WORLD_FS_ISOLATION`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | Policy-derived string (e.g. `workspace`, `full`) |
| Default if unset | Not relied on; Substrate sets it from policy for each command |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_WORLD_FS_ISOLATION"` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_WORLD_REQUIRE_WORLD`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String `1` or `0` |
| Default if unset | Not relied on; Substrate sets it from policy for each command |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_WORLD_REQUIRE_WORLD"` |
| Security notes | Not sensitive. |

#### `SUBSTRATE_MANAGER_INIT`
| Field | Value |
| --- | --- |
| Bucket | Exported state variable |
| Type / allowed values | String path |
| Default if unset | Unset until Substrate generates the manager init snippet |
| Precedence | Written by Substrate; operator-set values are overwritten |
| Scope | Run-only |
| Examples | `echo "$SUBSTRATE_MANAGER_INIT"` |
| Security notes | Not sensitive. |

## Non-supported Environment Variables

Substrate references additional environment variables for internal coordination, tests, compatibility shims, and experimental behavior. They are intentionally excluded from the supported contract.

- The exhaustive inventory lives at `docs/internals/env/inventory.md`.
- Setting any variable not listed under **Supported Variables** is unsupported unless explicitly documented as “not supported”.

## Validation Checklist

Use these commands to verify that this contract matches the implementation:

- Locate env override parsing for effective config: `rg -n \"apply_env_overrides\\(\" crates/shell/src/execution/config_model.rs`
- Confirm config precedence ordering: `rg -n \"resolve_effective_config\\(\" crates/shell/src/execution/config_model.rs`
- Audit supported env reads across runtime code: `rg -n \"env::var\\(\\\"(SUBSTRATE|SHIM|TRACE|AGENT|AUTH|HOST_PROXY|RATE_LIMIT|MAX_BODY|REQUEST_TIMEOUT)\" crates/**/src`
- Audit installer knobs: `rg -n \"SUBSTRATE_INSTALL_|SUBSTRATE_UNINSTALL_|SUBSTRATE_ROOT\" scripts/substrate`
- Run a runtime sanity check (host-only): `SUBSTRATE_WORLD=disabled substrate --shim-status-json`
- Run a host readiness check (Linux/macOS): `substrate host doctor --json`
- Run a world enforcement check (Linux/macOS): `substrate world doctor --json`
