# Configuration Reference

Environment variables and advanced configuration options for Substrate.

## Core Environment Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `SHIM_ORIGINAL_PATH` | Clean PATH for binary resolution | *none* | `/usr/bin:/bin` |
| `SHIM_TRACE_LOG` | Log output destination | `~/.trace_shell.jsonl` | `/tmp/trace.jsonl` |
| `SHIM_SESSION_ID` | Session correlation ID | auto-generated | uuid-v7-string |
| `SHIM_DEPTH` | Nesting level tracking | `0` | `0`, `1`, `2`... |
| `SHIM_BYPASS` | Emergency bypass mode | *none* | `1` |

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

## Integration Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `BASH_ENV` | Bash startup script | *none* | `~/.substrate_bashenv` |
| `TRACE_LOG_MAX_MB` | Log rotation size limit | `50` | `100` |
| `TEST_MODE` | Skip TTY checks in tests | *none* | `1` |

## Debug Configuration

### Comprehensive Debugging

```bash
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
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

# Enable path resolution logging
export SHIM_LOG_OPTS=resolve
```

## Log Configuration

### Default Behavior

- **Location**: `~/.trace_shell.jsonl`
- **Format**: JSONL (one JSON object per line)
- **Permissions**: 0600 (user-only access)
- **Rotation**: External (via logrotate or similar)

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

# Combined options
export SHIM_LOG_OPTS=raw,resolve
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
ls -la ~/.trace_shell.jsonl
# -rw------- (user-only access)
```

## Performance Tuning

### Cache Management

```bash
# Disable caching for testing
export SHIM_CACHE_BUST=1

# Monitor cache effectiveness
jq '.duration_ms' ~/.trace_shell.jsonl | head -20  # Cold cache
jq '.duration_ms' ~/.trace_shell.jsonl | tail -20  # Warm cache
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
# Non-interactive mode
export BASH_ENV="$HOME/.substrate_bashenv"

# Strict error handling
substrate --ci -c "npm test"

# Continue on error
substrate --ci --no-exit-on-error -f test-suite.sh
```

### Development Tools

```bash
# Hash pinning for reliability
hash -p "$HOME/.cmdshim_rust/git" git
hash -p "$HOME/.cmdshim_rust/npm" npm
hash -p "$HOME/.cmdshim_rust/docker" docker

# Verify pinning
type git npm docker
```

For installation instructions, see [INSTALLATION.md](INSTALLATION.md).
For development configuration, see [DEVELOPMENT.md](DEVELOPMENT.md).