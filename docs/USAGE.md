# Usage Guide

Daily usage patterns and integration examples for Substrate.

## Shell Usage

### Interactive Mode

```bash
substrate
# Substrate Shell v0.1.0
# Session ID: 018d1234-5678-7abc-def0-123456789abc
# Logging to: ~/.trace_shell.jsonl
substrate> git status
substrate> npm test
substrate> exit
```

### Command Execution

```bash
# Single command
substrate -c "git status && npm test"

# Script execution with state preservation
substrate -f deploy.sh

# Pipe commands
echo "date" | substrate

# CI mode with strict error handling
substrate --ci -c "make test"
```

### Built-in Commands

The shell includes cross-platform built-ins:
- `cd [dir]` - Change directory (supports `cd -`)
- `pwd` - Print working directory
- `export VAR=value` - Set environment variables
- `unset VAR` - Remove environment variables
- `exit` / `quit` - Exit the shell

## PTY Support

Substrate automatically uses PTY for interactive commands:

### Automatic Detection

**TUI Applications**: `vim`, `less`, `top`, `htop`, `fzf`, `lazygit`
**Interactive Shells**: `bash`, `zsh` (without `-c` flag)
**Container Commands**: `docker run -it`, `kubectl exec -it`
**Git Interactive**: `git add -p`, `git rebase -i`, `git commit`

### Manual Control

```bash
# Force PTY for any command
substrate -c ":pty ls -la"

# Environment controls
export SUBSTRATE_FORCE_PTY=1      # Force PTY globally
export SUBSTRATE_DISABLE_PTY=1    # Disable PTY globally
export SUBSTRATE_PTY_DEBUG=1      # Enable PTY debug logging
```

## Command Interception

### Automatic Deployment

Substrate automatically deploys shims on first run:

```bash
# First run deploys shims automatically
substrate

# Check deployment status
substrate --shim-status
```

### Manual Shim Management

```bash
# Force redeployment (useful after updates)
substrate --shim-deploy

# Remove all shims
substrate --shim-remove

# Skip automatic deployment for this run
substrate --shim-skip

# Disable automatic deployment permanently
export SUBSTRATE_NO_SHIMS=1
substrate
```

### PATH Configuration

To use shims for command interception:

```bash
# Configure PATH
export PATH="$HOME/.substrate/shims:$PATH"
export SHIM_ORIGINAL_PATH="$PATH"
hash -r
```

### Claude Code Integration

Proven integration pattern for AI assistants:

```bash
# 1. Set up non-interactive shell environment
./scripts/create_bashenv.sh
export BASH_ENV="$HOME/.substrate_bashenv"

# 2. Use hash pinning for reliable resolution
hash -r
hash -p "$HOME/.substrate/shims/git" git
hash -p "$HOME/.substrate/shims/npm" npm

# 3. Verify integration
which git  # Should show shim path
git --version  # Should work with logging
```

## Log Analysis

Commands are logged in structured JSONL format:

```bash
# View recent commands
tail -5 ~/.trace_shell.jsonl | jq '.command'

# Analyze session activity
jq 'select(.session_id == "your-session-id")' ~/.trace_shell.jsonl

# Command frequency analysis
jq -r '.command' ~/.trace_shell.jsonl | sort | uniq -c | sort -nr

# Performance analysis
jq '.duration_ms // empty' ~/.trace_shell.jsonl | awk '{sum+=$1} END {print "avg:", sum/NR "ms"}'
```

## Security Features

### Credential Redaction

Automatic redaction of sensitive information:

```bash
# Original command
curl -H "Authorization: Bearer secret123" api.example.com

# Logged as
["curl", "-H", "Authorization: ***", "api.example.com"]

# Disable redaction for debugging
SHIM_LOG_OPTS=raw curl -H "Authorization: Bearer token" api.com
```

### Emergency Bypass

```bash
# Bypass single command
SHIM_BYPASS=1 git status

# Complete system rollback
./scripts/rollback.sh
```

## Troubleshooting

### Debug Mode

```bash
# Enable comprehensive debugging
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
export SUBSTRATE_PTY_DEBUG=1

# Test with debugging
substrate -c "git status"
```

### Common Solutions

| Problem | Solution |
|---------|----------|
| Commands not intercepted | Check PATH order, run `hash -r` |
| Infinite loops | Ensure shim directory not in `SHIM_ORIGINAL_PATH` |
| PTY issues | Check `SUBSTRATE_DISABLE_PTY`, enable debug mode |
| Permission denied | Verify shim binary permissions |

### Log File Management

```bash
# Check log file permissions
ls -la ~/.trace_shell.jsonl

# Manual log rotation
mv ~/.trace_shell.jsonl ~/.trace_shell.jsonl.$(date +%Y%m%d)
gzip ~/.trace_shell.jsonl.*
```

## Integration Examples

### CI/CD Pipelines

```bash
# Strict error handling
substrate --ci -c "npm test && npm build"

# Continue on error
substrate --ci --no-exit-on-error -f integration-tests.sh
```

### Development Workflows

```bash
# Interactive development
substrate
substrate> git checkout feature-branch
substrate> npm install
substrate> npm test
substrate> git commit -m "Add feature"

# Automated workflows
substrate -f scripts/deploy.sh
```

For more advanced usage patterns and future capabilities, see [VISION.md](VISION.md).