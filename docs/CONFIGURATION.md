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

Release bundles place the base manifests under
`<prefix>/versions/<version>/config/` (`manager_hooks.yaml` and
`world-deps.yaml`). Workspace builds fall back to `config/manager_hooks.yaml`
and `scripts/substrate/world-deps.yaml` in the repository root.

## World Configuration

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_WORLD` | Force pass-through execution (`disabled`) | `enabled` | `disabled` |
| `SUBSTRATE_WORLD_ENABLED` | Cached world enablement flag (installer) | `1` | `0` |
| `SUBSTRATE_ANCHOR_MODE` | Anchor selection (`project`, `follow-cwd`, `custom`) | `project` | `follow-cwd` |
| `SUBSTRATE_ANCHOR_PATH` | Custom anchor directory (paired with `custom` mode) | shell launch directory | `/workspaces/substrate` |
| `SUBSTRATE_CAGED` | Enforce staying inside the resolved world root (`1`/`0`) | `1` | `0` |
| `SUBSTRATE_WORLD_DEPS_MANIFEST` | Override manifest for `world deps` | `<prefix>/versions/<version>/config/world-deps.yaml` | `/tmp/world_deps.yaml` |
| `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` | Force socket activation mode reporting (`socket_activation`, `manual`, or `unknown`) for diagnostics/tests | auto-detect via systemd | `socket_activation` |

Legacy environment variables `SUBSTRATE_WORLD_ROOT_MODE` / `SUBSTRATE_WORLD_ROOT_PATH` are still
parsed for compatibility.

### World root settings stack

Substrate resolves the world root from highest to lowest:

1. CLI flags: `--anchor-mode` / `--anchor-path` (also accepts `--world-root-mode` / `--world-root-path`)
2. Directory config: `.substrate/settings.toml` in the shell launch directory
3. Global config: `~/.substrate/config.toml` `[world]` table
4. Environment variables: `SUBSTRATE_ANCHOR_MODE` / `SUBSTRATE_ANCHOR_PATH`
5. Default: `project` mode anchored to the shell launch directory

The `caged` setting follows the same precedence stack (`--caged/--uncaged` -> dir config -> global
config -> env var `SUBSTRATE_CAGED` -> default `true`) and prevents leaving the resolved root even
when isolation is disabled.

Modes:

- `project` – anchor the overlay to the directory where `substrate` started.
- `follow-cwd` – recompute the root whenever the working directory changes.
- `custom` – use an explicit path (set via `anchor_path` or `--anchor-path`).

Both the global config and per-directory settings file use the same schema:

```toml
[world]
anchor_mode = "project"
anchor_path = ""
# Legacy keys are still parsed for compatibility:
root_mode = "project"
root_path = ""
caged = true
```

### Host-only driver helpers

- Use `scripts/dev/substrate_shell_driver` when invoking `target/debug/substrate` from shell scripts or automation. It resolves the workspace binary, exports `SUBSTRATE_WORLD=disabled` and `SUBSTRATE_WORLD_ENABLED=0`, and passes through all CLI arguments.
- Rust integration tests rely on `crates/shell/tests/common.rs::substrate_shell_driver()` to obtain an `assert_cmd::Command` with the same environment overrides. Reuse that helper instead of reimplementing binary lookup or TMPDIR wiring.

## Install Metadata (`~/.substrate/config.toml`)

The installer and `substrate world enable` command keep a small metadata file at
`~/.substrate/config.toml` with install-level fields under `[install]`. The
same file (and optional `.substrate/settings.toml` in a repo) can carry a
`[world]` table when you want a persistent root override:

```toml
[install]
world_enabled = true

[world]
root_mode = "project"
root_path = ""
caged = true
```

Unknown keys and extra tables are preserved for future expansion.

- Fresh installs write `world_enabled = true` unless `--no-world` is used.
- Use `--world` to force isolation for a single run even when install metadata
  or `SUBSTRATE_WORLD*` env vars disable it; metadata stays unchanged and
  `--no-world` still wins when provided.
- `substrate world enable` overwrites `[install]` after provisioning succeeds and repairs malformed metadata.
- Legacy installs that still have `config.json` are read automatically, but new writes use `config.toml`.
- The generated `~/.substrate/manager_env.sh` exports `SUBSTRATE_WORLD`,
  `SUBSTRATE_WORLD_ENABLED`, and `SUBSTRATE_CAGED` so shims and subprocesses read
  a consistent view of this metadata even before the CLI runs.
- Directory configs live at `.substrate/settings.toml` under the launch
  directory and only carry the `[world]` table shown above.

### Bootstrapping the config file

Use `substrate config init` whenever `~/.substrate/config.toml` is missing (or
after manually deleting/corrupting it). The command scaffolds the default
`[install]` and `[world]` tables, respects `SUBSTRATE_HOME`, and is available
before the shell/REPL starts. Pass `--force` to regenerate the file even if it
already exists. On Windows the same path lives under
`%USERPROFILE%\.substrate\config.toml`. Shell startup and the install scripts
emit a warning that points to this command whenever the file is absent, so
running `substrate config init` is the supported fix.

### Inspecting the config file

`substrate config show` prints the current global config in a stable, redacted
format. TOML is emitted by default for humans, while `--json` produces a
machine-friendly payload for automation. Both commands honor
`SUBSTRATE_HOME`/`%USERPROFILE%` overrides and exit non-zero with a reminder to
run `substrate config init` if the file is missing.

```bash
$ substrate config show
[install]
world_enabled = true

[world]
anchor_mode = "project"
anchor_path = ""
root_mode = "project"
root_path = ""
caged = true
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

`substrate config set key=value [...]` edits `config.toml` without opening a
text editor. Each dotted key is validated (anchor modes must be
`project`/`follow-cwd`/`custom`, boolean toggles accept `true/false/1/0`), and
all updates are applied atomically. Combine multiple assignments to keep related
fields in sync:

```bash
$ substrate config set install.world_enabled=false world.caged=false
substrate: updated config at /Users/alice/.substrate/config.toml
  - install.world_enabled: true -> false
  - world.caged: true -> false
```

Anchor overrides update the legacy aliases automatically so the `[world]` table
remains consistent:

```bash
$ substrate config set world.anchor_mode=custom world.anchor_path=/workspaces/substrate
substrate: updated config at /Users/alice/.substrate/config.toml
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
  "config_path": "/Users/alice/.substrate/config.toml",
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

Surface the new parity signals when archiving these artifacts:

- `summary.attention_required_managers` lists host-only managers that require a world sync.
- `summary.world_only_managers` lists tools present in the guest but missing locally.
- `summary.manager_states[].{name, parity, recommendation}` provides per-manager status plus the suggested remediation.

Example (macOS Sonoma / zsh, temp HOME):

```bash
TMP=$PWD/target/tests-tmp/macos-health
mkdir -p "$TMP/.substrate"
HOME=$TMP SUBSTRATE_MANAGER_MANIFEST=$TMP/manager_hooks.yaml \
  substrate health --json \
  | jq '.summary | {\n      attention_required_managers,\n      world_only_managers,\n      manager_states: [.manager_states[] | {name, parity, recommendation}]\n    }'
```

Need a legacy pipeline to inject snippets automatically? Run `substrate shim repair`
first, then export `BASH_ENV="$HOME/.substrate_bashenv"` explicitly for that job.

### Health Fixtures (Tests / Support)

To stub the expensive world checks, drop JSON fixtures under
`~/.substrate/health/`:

- `world_doctor.json` – consumed by `substrate shim doctor` and
  `substrate health` before falling back to `substrate world doctor --json`.
- `world_deps.json` – matches the `WorldDepsStatusReport` schema. Leave the
  file out to exercise the live `substrate world deps status --json` path.

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
