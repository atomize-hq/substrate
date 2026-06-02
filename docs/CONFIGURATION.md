# Configuration Reference

Environment variables and advanced configuration options for Substrate.

## Core Environment Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SHIM_ORIGINAL_PATH` | Clean PATH for binary resolution | *none* | `/usr/bin:/bin` |
| `SHIM_TRACE_LOG` | Log output destination | `~/.substrate/trace.jsonl` | `/tmp/trace.jsonl` |
| `SHIM_SESSION_ID` | Session correlation ID | auto-generated | uuid-v7-string |
| `SHIM_DEPTH` | Nesting level tracking | `0` | `0`, `1`, `2`... |
| `SHIM_BYPASS` | Emergency bypass mode | *none* | `1` |
| `SUBSTRATE_NO_SHIMS` | Disable automatic shim deployment | *none* | `1` |

## Advanced Shim Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SHIM_CALLER` | First command in chain | *none* | `npm` |
| `SHIM_CALL_STACK` | Command chain (max 8, deduped) | *none* | `npm,node` |
| `SHIM_PARENT_CMD_ID` | Links to shell command | *none* | uuid-v7-string |
| `SHIM_LOG_OPTS` | Logging options | *none* | `raw`, `resolve` |
| `SHIM_CACHE_BUST` | Force cache invalidation | *none* | `1` |

## Shell Configuration

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_FORCE_PTY` | Force PTY for all commands | *none* | `1` |
| `SUBSTRATE_DISABLE_PTY` | Disable PTY globally | *none* | `1` |
| `SUBSTRATE_PTY_DEBUG` | Enable PTY debug logging | *none* | `1` |
| `SUBSTRATE_PTY_PIPELINE_LAST` | PTY for last pipeline segment | *none* | `1` |

## Agent Hub Configuration

Agent Hub successor routing is configured through the normal Substrate config and policy files.

Config keys:
- `agents.hub.orchestrator_agent_id` selects the canonical host-scoped orchestrator agent for `substrate agent status` and `substrate agent doctor`.
- Agent inventory entries continue to define each agent's adapter kind and execution posture; the derived `backend_id` remains `<kind>:<agent_id>`.
- The shell-owned v1 runtime only realizes selected orchestrators with `config.kind=cli`, `protocol=substrate.agent.session`, and `cli.mode=persistent`.
- `config.cli.runtime_family` is the inventory-only runtime-realization truth for shell-owned UAA candidates. Supported values are `codex` and `claude_code`.
- Runtime realization still keys policy and exact backend routing off the derived `backend_id`. For example, `cli:codex_world` may realize the canonical `codex` runtime family while remaining a distinct exact backend id from `cli:codex`.
- `config.cli.binary` for the selected orchestrator must resolve on the host during `substrate agent doctor` and async REPL bootstrap.

Policy keys:
- `agents.allowed_backends` remains the allowlist for derived agent adapter ids such as `cli:codex` or `api:openai`.
- Existing `agents.allowed_backends` entries stay valid across the successor `substrate agent ...` command surface because the policy token is still the derived `backend_id`, not `client`, `router`, `protocol`, `provider`, or `auth_authority`.
- `agents.world_dispatch.*` is an internal steering-policy patch surface for orchestrator-owned host-to-world dispatch. It is deny-by-default, does not widen the public `substrate agent ...` CLI, and governs the live internal verbs `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker`.

Minimal example:

```yaml
agents:
  hub:
    orchestrator_agent_id: claude_code
```

```yaml
version: 1
id: claude_code
config:
  kind: cli
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    runtime_family: claude_code
    binary: claude
    mode: persistent
  capabilities:
    session_start: true
    session_resume: true
    session_fork: true
    session_stop: true
    status_snapshot: true
    event_stream: true
    llm: true
    mcp_client: true
```

```yaml
agents:
  allowed_backends:
    - cli:claude_code
    - cli:codex
```

## Manager Manifest & Init

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_MANAGER_MANIFEST` | Override manifest path (base + overlay) | `<prefix>/versions/<version>/config/manager_hooks.yaml` | `/tmp/manager_hooks.yaml` |
| `SUBSTRATE_MANAGER_INIT` | Generated manager snippet path | `~/.substrate/manager_init.sh` | `/custom/init.sh` |
| `SUBSTRATE_MANAGER_ENV` | Tiny script sourced via `BASH_ENV` | `~/.substrate/manager_env.sh` | `/tmp/manager_env.sh` |
| `SUBSTRATE_MANAGER_INIT_SHELL` | Force a specific shell for detect scripts | host `SHELL` or `/bin/sh` | `/usr/local/bin/bash` |
| `SUBSTRATE_SKIP_MANAGER_INIT` | Skip manager init entirely | `0` | `1` |
| `SUBSTRATE_SKIP_MANAGER_INIT_LIST` | Comma-separated managers to skip | *none* | `nvm,pyenv` |
| `SUBSTRATE_MANAGER_INIT_DEBUG` | Verbose detection logging | `0` | `1` |
| `SUBSTRATE_SHIM_HINTS` | Disable/enable shim hint emission | `1` | `0` |
| `SUBSTRATE_POLICY_GIT_CACHE` | Cache policy repo git hash across commands (`0`/`false` disables caching per process) | `1` | `0` |

Release bundles place the manager inventory under
`<prefix>/versions/<version>/config/manager_hooks.yaml` (used by manager init / shim hints).

`substrate world deps` (packages/bundles contract) does **not** read legacy `world-deps.yaml`
overlay plumbing. World deps now uses:
- Inventory directories: `$SUBSTRATE_HOME/deps/` (global) and `<workspace_root>/.substrate/deps/` (workspace chain)
- Enabled patches: `$SUBSTRATE_HOME/config.yaml` (global) and `<workspace_root>/.substrate/workspace.yaml` (workspace)

World deps probe toggles:

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_WORLD_DEPS_SKIP_APT` | Skip in-world APT requirement probes during `substrate world deps current (sync|install)` (treat requirements as satisfied) | *unset* | `1` |
| `SUBSTRATE_WORLD_DEPS_SKIP_PACMAN` | Skip in-world pacman requirement probes during `substrate world deps current (sync|install)` (treat requirements as satisfied) | *unset* | `1` |

## World Configuration

Substrate distinguishes between:

- `SUBSTRATE_OVERRIDE_*`: operator-provided override inputs (read by the effective-config resolver).
- `SUBSTRATE_*`: exported state (written by Substrate; not consulted as override inputs).

### Override inputs (`SUBSTRATE_OVERRIDE_*`)

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_OVERRIDE_WORLD` | Override `world.enabled` | *unset* | `disabled` |
| `SUBSTRATE_OVERRIDE_ANCHOR_MODE` | Override `world.anchor_mode` (`workspace`, `follow-cwd`, `custom`) | *unset* | `workspace` |
| `SUBSTRATE_OVERRIDE_ANCHOR_PATH` | Override `world.anchor_path` (paired with `custom`) | *unset* | `/workspaces/substrate` |
| `SUBSTRATE_OVERRIDE_CAGED` | Override `world.caged` (boolean) | *unset* | `0` |
| `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER` | Override `world.net.filter` (`true`, `false`, `1`, `0`, `yes`, `no`, `on`, `off`) | *unset* | `on` |
| `SUBSTRATE_OVERRIDE_POLICY_MODE` | Override `policy.mode` (`disabled`, `observe`, `enforce`) | *unset* | `enforce` |
| `SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC` | Override `sync.auto_sync` (boolean) | *unset* | `1` |
| `SUBSTRATE_OVERRIDE_SYNC_DIRECTION` | Override `sync.direction` (`from_world`, `from_host`, `both`) | *unset* | `from_host` |
| `SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY` | Override `sync.conflict_policy` (`prefer_host`, `prefer_world`, `abort`) | *unset* | `abort` |
| `SUBSTRATE_OVERRIDE_SYNC_EXCLUDE` | Override `sync.exclude` (comma-separated list) | *unset* | `.git,node_modules` |

Notes:
- When `<workspace_root>/.substrate/workspace.yaml` exists, `SUBSTRATE_OVERRIDE_*` is ignored for effective config.
- CLI flags override both config files and env overrides when a flag exists for the setting.
- `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER` follows the same rule: it only affects no-workspace runs. The derived
  `SUBSTRATE_WORLD_NET_FILTER` export below is output-only and is never read as an override input.

### Exported state (`SUBSTRATE_*`, output-only)

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_WORLD` | Effective `world.enabled` exported as `enabled|disabled` | (derived) | `disabled` |
| `SUBSTRATE_WORLD_ENABLED` | Effective `world.enabled` exported as `1|0` | (derived) | `0` |
| `SUBSTRATE_ANCHOR_MODE` | Effective `world.anchor_mode` exported | (derived) | `workspace` |
| `SUBSTRATE_ANCHOR_PATH` | Effective `world.anchor_path` exported | (derived) | `/workspaces/substrate` |
| `SUBSTRATE_CAGED` | Effective `world.caged` exported as `1|0` | (derived) | `1` |
| `SUBSTRATE_WORLD_NET_FILTER` | Effective `world.net.filter` exported as `1|0` | (derived) | `0` |
| `SUBSTRATE_POLICY_MODE` | Effective `policy.mode` exported | (derived) | `observe` |
| `SUBSTRATE_WORLD_FS_MODE` | Active world filesystem mode (policy-controlled: `writable` or `read_only`) | (derived) | `read_only` |

`SUBSTRATE_WORLD_NET_FILTER` always reports the resolved effective value after config precedence and any eligible
no-workspace override have been applied. It does not mean filtering is active by itself; it only reports whether the host
may request enforcement.

### Netfilter request examples

- Allow-all posture: `world.net.filter=true` and canonicalized `net_allowed=["*"]` export `SUBSTRATE_WORLD_NET_FILTER=1`,
  but the host still does not request `isolate_network`. `WORLD_NETFILTER_ENABLE=1` does not change that allow-all
  outcome.
- Deny-all posture: `world.net.filter=true` and canonicalized `net_allowed=[]` cause the host to request deny-all
  isolation. In a no-workspace run, `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=on` can enable that gate temporarily; in a
  workspace run, the override is ignored and the workspace config remains authoritative.
- Restrictive allowlist posture: `world.net.filter=true` and canonicalized
  `net_allowed=["github.com","crates.io"]` cause the host to request isolation with that allowlist. The same restrictive
  policy does not request enforcement when `world.net.filter=false`.

Other world-adjacent variables:

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_WORLD_REQUEST_PROFILE` | Sets the Agent API request `profile` for world-service executions (advanced/internal only). Built-in world-deps profiles such as `world-deps-provision` and `world-deps-probe` are reserved for Substrate’s own world-deps flows and are ignored when supplied through this env var; the operator-facing APT provisioning workflow remains `substrate world enable --provision-deps`. See `docs/reference/world/deps/README.md` and `docs/reference/world/deps/provisioning.md`. | *unset* | `wdap-smoke-profile` |
| `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` | Force socket activation mode reporting (`socket_activation`, `manual`, or `unknown`) for diagnostics/tests | auto-detect via systemd | `socket_activation` |
| `SUBSTRATE_SYSTEMCTL_TIMEOUT_MS` | Timeout (ms) for `systemctl show …` probes used by Linux socket-activation detection; prevents hangs when systemd/dbus is unhealthy | `2000` | `250` |

Legacy environment variables `SUBSTRATE_WORLD_ROOT_MODE` / `SUBSTRATE_WORLD_ROOT_PATH` are still
parsed for compatibility.

### World root settings stack

Substrate resolves world settings from highest to lowest:

1. CLI flags (when a flag exists): `--world` / `--no-world`, `--anchor-mode` / `--anchor-path`, `--caged` / `--uncaged`
2. Workspace config: `<workspace_root>/.substrate/workspace.yaml`
3. Environment overrides: `SUBSTRATE_OVERRIDE_*`
4. Global config: `~/.substrate/config.yaml`
5. Built-in defaults

The `caged` setting prevents leaving the resolved root even when isolation is disabled.

Modes:

- `workspace` – anchor the overlay to the directory where `substrate` started.
- `follow-cwd` – recompute the root whenever the working directory changes.
- `custom` – use an explicit path (set via `anchor_path` or `--anchor-path`).

Both the global config and workspace config use the same schema:

```yaml
world:
  anchor_mode: workspace
  anchor_path: ""
  env:
    # When false (default), the world environment does not forward host env vars beyond Substrate's
    # deterministic baseline (PATH/HOME/XDG/etc). When true, Substrate may forward a small safe
    # allowlist of host env vars (locale/terminal/timezone), as defined by the world env contract.
    inherit_from_host: false
  # Legacy keys are still parsed for compatibility:
  root_mode: workspace
  root_path: ""
  caged: true
```

### Host-only driver helpers

- Unix (bash): use `scripts/dev/substrate_shell_driver` when invoking `target/debug/substrate` from shell scripts or automation. It resolves the workspace binary and injects `--no-world` unless `--world/--no-world` is already provided.
- Windows (PowerShell): use `pwsh -File scripts\\dev\\substrate_shell_driver.ps1` with the same behavior (`--bin` / `SUBSTRATE_BIN` override; injects `--no-world` unless explicitly overridden).
- Rust integration tests rely on `crates/shell/tests/common.rs::substrate_shell_driver()` to obtain an `assert_cmd::Command` with the same environment overrides. Reuse that helper instead of reimplementing binary lookup or TMPDIR wiring.

## Install Metadata (`~/.substrate/config.yaml`)

The installer and `substrate world enable` command keep a small metadata file at
`~/.substrate/config.yaml` with install-level fields under `install:`. The same
file can also carry the `world:` and `policy:` mappings when you want persistent defaults:

```yaml
install:
  world_enabled: true
world:
  root_mode: project
  root_path: ""
  caged: true
```

Unknown keys and extra tables are preserved for future expansion.

- Fresh installs write `world_enabled = true` unless `--no-world` is used.
- Use `--world` to force isolation for a single run even when install metadata
  or `SUBSTRATE_OVERRIDE_WORLD=disabled` disables it; metadata stays unchanged and `--no-world` still wins when provided.
- `substrate world enable` overwrites `install:` after provisioning succeeds and repairs malformed metadata.
- Legacy installs that still have `config.json` are read automatically, but new writes use `config.yaml`.
- The generated `~/.substrate/manager_env.sh` exports derived `SUBSTRATE_*` state so shims and subprocesses observe a consistent view of the effective config.
- Supported Linux/macOS provisioning helpers also wire `SUBSTRATE_HOME` into the
  `substrate-world-service` systemd unit and keep that exact path writable via
  `ReadWritePaths`. WSL helper scripts are intentionally fail-closed in this slice
  and do not claim that placement contract yet.

### World filesystem mode

Policies control whether the world filesystem is writable (overlay/copy-diff) or forced to
`read_only` via `world_fs.mode` in the effective policy (resolved from policy patches).

- `world_fs.mode: writable` (default): project writes land in the world overlay.
- `world_fs.mode: read_only`: the project is remounted read-only so both relative and absolute
  project writes fail (requires `world_fs.require_world: true`).

The shell exports `SUBSTRATE_WORLD_FS_MODE` with the resolved value for telemetry/replay and
surfaces it in `substrate host doctor --json` and `substrate world doctor --json` so policy
enforcement is visible without extra flags.

### Policy patches (`policy.yaml`)

Substrate policy files on disk are always *patch-only*: a sparse YAML mapping where omitted keys mean
“inherit”. The broker resolves a single effective policy for the current directory and all execution
paths (shell/shim/world-service) consume that same resolved policy.

Effective policy resolution order:

1. Defaults (built-in)
2. Global policy patch: `$SUBSTRATE_HOME/policy.yaml` (usually `~/.substrate/policy.yaml`)
3. Workspace policy patch: `<workspace_root>/.substrate/policy.yaml` (only when a workspace exists and is enabled)

Workspace discovery and the disabled marker:

- A workspace root is any directory with `<workspace_root>/.substrate/workspace.yaml`.
- If `<workspace_root>/.substrate/workspace.disabled` exists, that workspace is treated as non-existent for
  policy discovery and effective policy resolution (so the workspace patch is ignored).

Patch schema and failure behavior:

- The patch file must be a YAML mapping (`{}`) or `null` (treated as an empty patch).
- The patch schema is strict: unknown keys, type mismatches, invariant violations, and unreadable-but-present
  patch files are policy resolution errors and block broker-dependent execution (exit code `2`).
- Per-key merge strategy is `replace` when a key is present in a patch layer (including lists like `cmd_denied`).

Minimal patch example (workspace or global):

```yaml
world_fs:
  require_world: true
```

Internal host-to-world steering example:

- This surface is internal-only and deny-by-default. The built-in defaults keep `agents.world_dispatch.enabled=false`, keep the allowlists empty, require exact same-session and same-world-binding truth, disallow capability narrowing, and set both current concurrency caps to `0`.
- Current action ids accepted by `agents.world_dispatch.allowed_actions` are `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `inspect_world_worker`, `cancel_world_work`, and `stop_world_worker`.
- `inspect_world_worker` remains internal, retained-worker-only in v1, and returns an authoritative store-backed snapshot instead of invoking world-side execution transport. Routed snapshot delivery is currently supported only on Linux in v1; non-Linux builds fail closed with `unsupported_platform_or_posture`.
- `cancel_world_work` is a valid internal allowlisted action in Packet 1, but Packet 1 only freezes the retained-worker-only contract/policy surface. Runtime dispatch routing still fails closed with `unsupported_dispatch_action` until later packets land routed cancel behavior.
- `stop_world_worker` remains internal, retained-worker-only in v1, and is a durable closeout action distinct from `cancel_world_work`. On Linux in v1, an allowlisted exact-target stop request reuses the existing private owner stop surface to drive authoritative stopped closeout; non-Linux builds fail closed with `unsupported_platform_or_posture`.
- Current mode ids are limited to `ephemeral` and `retained`.
- This patch surface does not imply active-ephemeral inspect, routed cancel execution beyond the Packet 1 `cancel_world_work` contract/policy surface, active-ephemeral or dual-target cancel semantics, later mutating verbs like `fork_world_worker`, router-owned attach execution, or broader approval/fork autonomy policy.

```yaml
agents:
  world_dispatch:
    enabled: true
    allowed_backends:
      - cli:codex_world
    allowed_actions:
      - run_world_task
      - spawn_world_worker
      - continue_world_worker
      - inspect_world_worker
    allowed_modes:
      - ephemeral
      - retained
    same_session_only: true
    same_world_binding_only: true
    allow_capability_narrowing: false
    max_live_retained_workers: 4
    max_concurrent_ephemeral: 2
```

Policy patch management (CLI):

- Global: `substrate policy global init|show|set|reset`
- Workspace: `substrate policy workspace show|set|reset` (and `substrate policy init` to create the workspace patch)
- Effective policy: `substrate policy current show --explain`

For a minimal end-to-end verification run, see `scripts/linux/agent-hub-isolation-verify.sh`.

### Bootstrapping the config file

Use `substrate config init` whenever `~/.substrate/config.yaml` is missing (or
after manually deleting/corrupting it). The command scaffolds the default
`install:` and `world:` mappings, respects `SUBSTRATE_HOME`, and is available
before the shell/REPL starts. Pass `--force` to regenerate the file even if it
already exists. On Windows the same path lives under
`%USERPROFILE%\.substrate\config.yaml`. Shell startup and the install scripts
emit a warning that points to this command whenever the file is absent, so
running `substrate config init` is the supported fix.

### Inspecting the config file

`substrate config show` prints the current global config in a stable, redacted
format. YAML is emitted by default for humans, while `--json` produces a
machine-friendly payload for automation. Both commands honor
`SUBSTRATE_HOME`/`%USERPROFILE%` overrides and exit non-zero with a reminder to
run `substrate config init` if the file is missing.

Note: `config show` renders only the global config file (`~/.substrate/config.yaml`); per-directory overrides from `.substrate/settings.yaml` are applied when running Substrate (settings stack), not merged into `config show` output.

```bash
$ substrate config show
install:
  world_enabled: true
world:
  anchor_mode: project
  anchor_path: ""
  root_mode: project
  root_path: ""
  caged: true
```

```bash
$ substrate config show --json
{
  "install": {
    "world_enabled": true
  },
  "world": {
    "anchor_mode": "project",
    "anchor_path": "",
    "root_mode": "project",
    "root_path": "",
    "caged": true
  }
}
```

Sensitive fields will be replaced with `*** redacted ***` once such values are
stored in the config.

### Updating the config file

`substrate config set key=value [...]` edits `config.yaml` without opening a
text editor. Each dotted key is validated (anchor modes must be
`project`/`follow-cwd`/`custom`, boolean toggles accept `true/false/1/0`), and
all updates are applied atomically. Combine multiple assignments to keep related
fields in sync:

```bash
$ substrate config set install.world_enabled=false world.caged=false
substrate: updated config at /Users/alice/.substrate/config.yaml
  - install.world_enabled: true -> false
  - world.caged: true -> false
```

Anchor overrides update the legacy aliases automatically so the `world:` mapping
remains consistent:

```bash
$ substrate config set world.anchor_mode=custom world.anchor_path=/workspaces/substrate
substrate: updated config at /Users/alice/.substrate/config.yaml
  - world.anchor_mode: "project" -> "custom"
  - world.root_mode (alias): "project" -> "custom"
  - world.anchor_path: "" -> "/workspaces/substrate"
  - world.root_path (alias): "" -> "/workspaces/substrate"
```

Pass `--json` for automation—the response lists every changed key with its old
and new values:

```bash
$ substrate config set --json world.anchor_mode=follow-cwd
{
  "config_path": "/Users/alice/.substrate/config.yaml",
  "changed": true,
  "changes": [
    {
      "key": "world.anchor_mode",
      "alias": false,
      "old_value": "project",
      "new_value": "follow-cwd"
    },
    {
      "key": "world.root_mode",
      "alias": true,
      "old_value": "project",
      "new_value": "follow-cwd"
    }
  ]
}
```

PowerShell users can wrap each assignment in quotes to preserve backslashes:

```powershell
PS> substrate config set "world.anchor_mode=custom" "world.anchor_path=C:\Work" "world.caged=false"
```

Invalid keys or values abort the run without touching the file. As with the
other `config` verbs, `SUBSTRATE_HOME` / `%USERPROFILE%` controls where the file
lives so tests can redirect writes safely.

## CLI Flags

### Shim Management

| Flag | Purpose | Example |
|------|---------|---------|
| `--shim-status` | Show deployment status and version | `substrate --shim-status` |
| `--shim-deploy` | Force redeployment of shims | `substrate --shim-deploy` |
| `--shim-remove` | Remove all deployed shims | `substrate --shim-remove` |
| `--shim-skip` | Skip automatic deployment for this run | `substrate --shim-skip` |

### Other Flags

| Flag | Purpose | Example |
|------|---------|---------|
| `-c <command>` | Execute command and exit | `substrate -c "ls -la"` |
| `-f <script>` | Execute script file | `substrate -f script.sh` |
| `--ci` | CI mode (no banner, strict errors) | `substrate --ci -c "npm test"` |
| `--no-exit-on-error` | Continue on error in CI mode | `substrate --ci --no-exit-on-error` |
| `--pty` | Force PTY for command | `substrate --pty -c "vim"` |
| `--world` | Force world isolation for this invocation (overrides disabled install/config/env) | `substrate --world -c "npm test"` |
| `--no-world` | Disable world isolation for this run | `substrate --no-world -c "npm test"` |
| `--anchor-mode <mode>` | Select anchor strategy (`project`, `follow-cwd`, `custom`) | `substrate --anchor-mode follow-cwd -c "npm test"` |
| `--anchor-path <path>` | Explicit anchor when using `custom` mode | `substrate --anchor-mode custom --anchor-path /opt/work` |
| `--caged` / `--uncaged` | Toggle local caged root guard | `substrate --uncaged --anchor-mode project` |
| `--version-json` | Output version info as JSON | `substrate --version-json` |
| `--legacy-repl` | Fall back to the legacy synchronous REPL | `substrate --legacy-repl` |

> **Note:** `--async-repl` is still accepted for compatibility but no longer
> required—the async loop is the default interactive experience.

## Integration Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `BASH_ENV` | Bash startup script | *none* | `~/.substrate/manager_env.sh` |
| `TRACE_LOG_MAX_MB` | Log rotation size limit | `100` | `200` |
| `TEST_MODE` | Skip TTY checks in tests | *none* | `1` |

## Debug Configuration

### Comprehensive Debugging

```bash
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw
export SUBSTRATE_PTY_DEBUG=1
export SHIM_CACHE_BUST=1
```

### Security Debugging

```bash
# Disable credential redaction
export SHIM_LOG_OPTS=raw

# Force filesystem sync for debugging
export SHIM_FSYNC=1
```

### Performance Analysis

```bash
# Force cache invalidation
export SHIM_CACHE_BUST=1

# Enable path resolution logging (sets resolve mode only)
export SHIM_LOG_OPTS=resolve
```

## Log Configuration

### Default Behavior

- **Location**: `~/.substrate/trace.jsonl`
- **Format**: JSONL (one JSON object per line)
- **Permissions**: 0600 (user-only access)
- **Rotation**: Built-in size-based rotation handled by the unified writer in `substrate-trace` (no shell-side rotation). Defaults:
  - `TRACE_LOG_MAX_MB=100` (rotate at ~100MB)
  - `TRACE_LOG_KEEP=3` (keep last 3 rotated files: `.1..=.<KEEP>`)
  - You can override these via environment variables. Use external tools only for archival/backup.

### Custom Log Destination

```bash
# Custom log file
export SHIM_TRACE_LOG="/var/log/substrate.jsonl"

# Temporary logging
export SHIM_TRACE_LOG="/tmp/debug.jsonl"
```

### Log Options

```bash
# Raw mode (no credential redaction)
export SHIM_LOG_OPTS=raw

# Include binary resolution paths
export SHIM_LOG_OPTS=resolve

# Switch between raw (no redaction) and resolve (log resolved paths) explicitly
# These modes are mutually exclusive
# export SHIM_LOG_OPTS=raw
# export SHIM_LOG_OPTS=resolve
```

## Security Settings

### Credential Protection

Default behavior redacts sensitive patterns:

- Authorization headers
- API keys and tokens
- Password flags
- Cookie values

Disable for debugging:

```bash
export SHIM_LOG_OPTS=raw
```

### File Permissions

Log files automatically created with restricted permissions:

```bash
ls -la ~/.substrate/trace.jsonl
# -rw------- (user-only access)
```

## Performance Tuning

### Cache Management

```bash
# Disable caching for testing
export SHIM_CACHE_BUST=1

# Monitor cache effectiveness
jq '.duration_ms' ~/.substrate/trace.jsonl | head -20  # Cold cache
jq '.duration_ms' ~/.substrate/trace.jsonl | tail -20  # Warm cache
```

### Resource Limits

Future configuration for Phase 4:

```bash
# CPU limits (planned)
export SUBSTRATE_CPU_LIMIT="2.0"

# Memory limits (planned)  
export SUBSTRATE_MEM_LIMIT="2Gi"

# Network egress budget (planned)
export SUBSTRATE_NET_BUDGET="1GB"
```

## Integration Patterns

### CI/CD Environments

```bash
# Non-interactive mode referencing the manifest + temp HOME
HOME=$PWD/target/tests-tmp/ci \
  SUBSTRATE_MANAGER_MANIFEST=$PWD/ci/manager_hooks.yaml \
  SHIM_TRACE_LOG=$PWD/target/tests-tmp/ci/trace.jsonl \
  substrate --ci -c "npm test"

# Capture a health snapshot for evidence logs
substrate shim doctor --json > artifacts/shim_doctor.json
substrate health --json > artifacts/substrate_health.json
substrate world doctor --json > artifacts/world_doctor.json
```

Key health fields to capture alongside these artifacts:

- `summary.missing_managers` lists managers not detected on the host.
- `summary.world_ok` / `summary.world_error` describe world backend health.
- `summary.world_deps_missing` / `summary.world_deps_blocked` describe enabled deps that are missing or require manual install.
- `summary.world_deps_error` is set when the world deps snapshot cannot be collected.

Example (macOS Sonoma / zsh, temp HOME):

```bash
TMP=$PWD/target/tests-tmp/macos-health
mkdir -p "$TMP/.substrate"
HOME=$TMP SUBSTRATE_MANAGER_MANIFEST=$TMP/manager_hooks.yaml \
  substrate health --json \
  | jq '.summary | {ok, missing_managers, world_ok, world_deps_missing, world_deps_blocked, world_deps_error}'
```

Need a legacy pipeline to inject snippets automatically? Run `substrate shim repair`
first, then export `BASH_ENV="$HOME/.substrate_bashenv"` explicitly for that job.

### Health Fixtures (Tests / Support)

To stub the expensive world checks, drop JSON fixtures under
`~/.substrate/health/`:

- `world_doctor.json` – consumed by `substrate shim doctor` and
  `substrate health` before falling back to `substrate world doctor --json`.
- `world_deps.json` – matches the world deps doctor snapshot schema used by
  `substrate shim doctor` / `substrate health` (leave it out to exercise the
  live world backend probes).

These overrides keep CI sandboxes deterministic while still exercising the same
code paths as production builds.

### Development Tools

```bash
# Inspect the runtime PATH Substrate will build
substrate -c 'printf "PATH -> %s\n" "$PATH"'

# Pin a shim manually (optional)
hash -p "$HOME/.substrate/shims/git" git

# Run doctor output during local debugging
substrate shim doctor
```

Use `SUBSTRATE_MANAGER_INIT_DEBUG=1` when iterating on manifest detection and
`SUBSTRATE_WORLD=disabled` if you need to run entirely on the host for a short
period.

For installation instructions, see [INSTALLATION.md](INSTALLATION.md).
For development configuration, see [DEVELOPMENT.md](DEVELOPMENT.md).
