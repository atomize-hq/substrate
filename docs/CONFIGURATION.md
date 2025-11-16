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
| `SUBSTRATE_MANAGER_MANIFEST` | Override manifest path (base + overlay) | `config/manager_hooks.yaml` | `/tmp/manager_hooks.yaml` |
| `SUBSTRATE_MANAGER_INIT` | Generated manager snippet path | `~/.substrate/manager_init.sh` | `/custom/init.sh` |
| `SUBSTRATE_MANAGER_ENV` | Tiny script sourced via `BASH_ENV` | `~/.substrate/manager_env.sh` | `/tmp/manager_env.sh` |
| `SUBSTRATE_MANAGER_INIT_SHELL` | Force a specific shell for detect scripts | host `SHELL` or `/bin/sh` | `/usr/local/bin/bash` |
| `SUBSTRATE_SKIP_MANAGER_INIT` | Skip manager init entirely | `0` | `1` |
| `SUBSTRATE_SKIP_MANAGER_INIT_LIST` | Comma-separated managers to skip | *none* | `nvm,pyenv` |
| `SUBSTRATE_MANAGER_INIT_DEBUG` | Verbose detection logging | `0` | `1` |
| `SUBSTRATE_SHIM_HINTS` | Disable/enable shim hint emission | `1` | `0` |

## World Configuration

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SUBSTRATE_WORLD` | Force pass-through execution (`disabled`) | `enabled` | `disabled` |
| `SUBSTRATE_WORLD_ENABLED` | Cached world enablement flag (installer) | `1` | `0` |
| `SUBSTRATE_WORLD_DEPS_MANIFEST` | Override manifest for `world deps` | bundled manifest | `/tmp/world_deps.yaml` |

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
| `--version-json` | Output version info as JSON | `substrate --version-json` |
| `--legacy-repl` | Fall back to the legacy synchronous REPL | `substrate --legacy-repl` |

> **Note:** `--async-repl` is still accepted for compatibility but no longer
> required—the async loop is the default interactive experience.

## Integration Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `BASH_ENV` | Bash startup script | *none* | `~/.substrate_bashenv` |
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
substrate world doctor --json > artifacts/world_doctor.json
```

Need a legacy pipeline to inject snippets automatically? Run `substrate shim repair`
first, then export `BASH_ENV="$HOME/.substrate_bashenv"` explicitly for that job.

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
