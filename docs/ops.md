# Substrate Operations Guide

## Overview

This guide covers operational aspects of the Substrate command tracing system based on the current implementation, including deployment, monitoring, troubleshooting, and maintenance procedures.

## Deployment

### Prerequisites

- Rust 1.74+ installed
- Git for cloning the repository
- Appropriate permissions for creating directories in `$HOME`

### Phase 1: Building and Basic Deployment

1. **Build the system**:

   ```bash
   # Clone and build all components
   git clone <repository-url>
   cd substrate
   cargo build --release
   
   # Built artifacts:
   # - target/release/substrate-shim        # Command interception shim
   # - target/release/substrate             # Custom shell
   # - target/release/substrate-supervisor  # Process supervisor (if built)
   ```

2. **Deploy shims using the staging script**:

   ```bash
   # Deploy shims to ~/.substrate/shims/
   ./scripts/stage_shims.sh
   
   # Script automatically uses target/release/substrate-shim if no argument provided
   # Or specify custom binary:
   # ./scripts/stage_shims.sh path/to/custom/substrate-shim
   ```

3. **Set up environment variables**:

   ```bash
   # Required: Clean PATH without shim directory
   export SHIM_ORIGINAL_PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
   
   # Required: Update PATH to include shims first
   export PATH="$HOME/.substrate/shims:$SHIM_ORIGINAL_PATH"
   
   # Optional: Specify log file (defaults to ~/.trace_shell.jsonl)
   export SHIM_TRACE_LOG="$HOME/.trace_shell.jsonl"
   
   # Clear shell command cache
   hash -r
   ```

4. **Verify basic deployment**:
   ```bash
   which git    # Should show: ~/.substrate/shims/git
   type git     # Should show shim first
   git --version  # Should work normally
   
   # Check logging
   tail -1 ~/.trace_shell.jsonl | jq '.'
   ```

### Phase 2: Non-Interactive Shell Support (Claude Code Integration)

1. **Create BASH_ENV file for non-interactive shells**:

   ```bash
   ./scripts/create_bashenv.sh
   ```

2. **Set up environment for non-interactive use**:

   ```bash
   export BASH_ENV="$HOME/.substrate_bashenv"
   ```

3. **Test non-interactive shell integration**:
   ```bash
   # Test with non-interactive bash
   bash -c 'which git; git --version'
   
   # Verify logging occurred
   tail -2 ~/.trace_shell.jsonl | jq '.command'
   ```

### Phase 3: Supervisor Integration

1. **Using the supervisor for managed execution**:

   ```bash
   # Direct supervisor usage (if supervisor binary built)
   target/release/substrate-supervisor git status
   
   # Or programmatic usage via supervisor API
   ```

## Current Implementation Status

### Available Components

- **substrate-shim**: ✅ Fully functional command interception
- **substrate**: ✅ Full shell with PTY support, interactive REPL
- **substrate-common**: ✅ Shared utilities and logging
- **substrate-supervisor**: ✅ Process management and environment setup

### Deployment Scripts

- **`scripts/stage_shims.sh`**: ✅ Working shim deployment
- **`scripts/create_bashenv.sh`**: ✅ Non-interactive shell setup
- **`scripts/rollback.sh`**: ✅ Emergency rollback functionality

## Performance Monitoring

### Key Metrics

Monitor these performance indicators with current implementation:

```bash
# Average shim overhead (requires jq)
jq '.duration_ms // empty' ~/.trace_shell.jsonl | awk '{sum+=$1; count++} END {if(count>0) print "avg:", sum/count "ms"}'

# 95th percentile latency
jq -r '.duration_ms // empty' ~/.trace_shell.jsonl | sort -n | awk 'END {print NR*0.95}' | xargs -I {} sed -n '{}p' <(jq -r '.duration_ms // empty' ~/.trace_shell.jsonl | sort -n)

# Command execution frequency
jq -r '.command // empty' ~/.trace_shell.jsonl | sort | uniq -c | sort -nr | head -10

# Session analysis
jq -r '.session_id // empty' ~/.trace_shell.jsonl | sort | uniq -c | sort -nr
```

### Performance Characteristics

- **Shim overhead**: Optimized for minimal latency with intelligent caching
- **Memory usage**: Designed for efficient resource utilization
- **Binary size**: Compact Rust binaries optimized for performance
- **Benchmarks**: Run `cargo bench` for detailed performance analysis
- **Cache effectiveness**: Reduces stat() calls after warmup

## Security Operations

### Log File Security

Substrate automatically creates log files with user-only permissions:

```bash
# Verify log file permissions
ls -la ~/.trace_shell.jsonl
# Should show: -rw------- (0o600 - user read/write only)
```

### Credential Redaction

Current redaction implementation automatically handles:

- **Key-value patterns**: `token=secret`, `password=mypass`, `SECRET=value`
- **Flag-value patterns**: `--token`, `--password`, `-p`, `-H`, `--header`

To disable redaction for debugging:

```bash
export SHIM_LOG_OPTS=raw
```

### Security Monitoring

```bash
# Check for potential credential leaks (when redaction is disabled)
grep -i "token\|password\|secret\|key" ~/.trace_shell.jsonl

# Monitor failed executions
jq 'select(.exit_code != 0 and .exit_code != null) | {ts, command, exit_code, argv}' ~/.trace_shell.jsonl

# Track signal terminations (potential security issues)
jq 'select(.term_signal != null) | {ts, command, term_signal, argv}' ~/.trace_shell.jsonl

# Session correlation analysis
jq 'select(.session_id) | {session_id, command, ts}' ~/.trace_shell.jsonl | head -20
```

## Error Monitoring and Troubleshooting

### Common Issues

#### 1. Commands Not Being Intercepted

**Symptoms**: Commands execute without logging, `which` shows system binary

**Diagnosis**:
```bash
echo $PATH  # Check if shim directory is first
which -a git  # Should show shim first, then system binary
echo $SHIM_ORIGINAL_PATH  # Should not contain shim directory
```

**Solution**:
```bash
# Ensure proper PATH setup
export PATH="$HOME/.substrate/shims:$SHIM_ORIGINAL_PATH"
hash -r  # Clear shell command cache
```

#### 2. Bypass Mode Activated

**Symptoms**: Commands skip tracing, logs show bypass=true

**Diagnosis**:
```bash
echo $SHIM_ACTIVE  # Should be unset in normal shells
echo $SHIM_BYPASS  # Should be unset for normal operation
```

**Solution**:
```bash
unset SHIM_ACTIVE
unset SHIM_BYPASS
```

#### 3. Permission Denied Errors

**Symptoms**: Shim binaries fail to execute

**Diagnosis**:
```bash
ls -la ~/.substrate/shims/  # Check shim permissions
file ~/.substrate/shims/.shimbin  # Verify binary type
```

**Solution**:
```bash
chmod +x ~/.substrate/shims/*
# Or re-run staging script
./scripts/stage_shims.sh
```

#### 4. PTY-Related Issues

**Symptoms**: Interactive commands don't work properly

**Diagnosis**:
```bash
echo $SUBSTRATE_DISABLE_PTY  # Should be unset for PTY support
echo $SUBSTRATE_FORCE_PTY    # Check PTY forcing
```

**Solution**:
```bash
# Enable PTY debug logging
export SUBSTRATE_PTY_DEBUG=1
substrate -c "vim test.txt"  # Test with interactive command

# Or disable PTY as escape hatch
export SUBSTRATE_DISABLE_PTY=1
```

### Diagnostic Commands

```bash
# Check shim binary integrity and installation
file ~/.substrate/shims/.shimbin
md5sum ~/.substrate/shims/.shimbin ~/.substrate/shims/git
ls -la ~/.substrate/shims/ | head -10

# Test path resolution and caching
SHIM_CACHE_BUST=1 SHIM_TRACE_LOG=/tmp/debug.jsonl git --version
jq '.duration_ms' /tmp/debug.jsonl  # Should be slower without cache

# Verify session correlation
jq 'select(.session_id) | .session_id' ~/.trace_shell.jsonl | sort | uniq -c

# Test emergency bypass
SHIM_BYPASS=1 git --version  # Should not create log entries
```

## Shell Operations

### Using Substrate Shell

```bash
# Interactive REPL mode
substrate

# Single command execution
substrate -c "git status && npm test"

# Script execution with state preservation
substrate -f script.sh

# CI mode with strict error handling
substrate --ci -c "make test"

# PTY mode for interactive commands
substrate -c ":pty vim file.txt"
```

### PTY Configuration

```bash
# Force PTY for all commands
export SUBSTRATE_FORCE_PTY=1

# Disable PTY globally (escape hatch)
export SUBSTRATE_DISABLE_PTY=1

# Enable PTY debug logging
export SUBSTRATE_PTY_DEBUG=1

# PTY for last segment in pipelines
export SUBSTRATE_PTY_PIPELINE_LAST=1
```

## Maintenance Procedures

### Log Management

Current implementation creates logs but doesn't include automatic rotation. Implement external rotation:

```bash
# Example logrotate configuration
cat > ~/.config/logrotate/substrate <<EOF
$HOME/.trace_shell.jsonl {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0600 $(whoami) $(id -gn)
}
EOF

# Manual rotation
mv ~/.trace_shell.jsonl ~/.trace_shell.jsonl.$(date +%Y%m%d)
gzip ~/.trace_shell.jsonl.*
```

### Cache Management

The resolution cache is automatically managed by the shim:

```bash
# Force cache invalidation for debugging
export SHIM_CACHE_BUST=1

# Test cache effectiveness (compare warm vs cold)
time git --version  # Warm cache
SHIM_CACHE_BUST=1 time git --version  # Cold cache
```

### Updates and Upgrades

```bash
# 1. Build new binaries
cargo build --release

# 2. Backup current installation
cp -r ~/.substrate/shims ~/.substrate/shims.backup.$(date +%Y%m%d)

# 3. Deploy new version
./scripts/stage_shims.sh

# 4. Verify installation
which git  # Should show shim path
git --version  # Should work normally
tail -1 ~/.trace_shell.jsonl | jq '.build'  # Check version
```

### Reedline Fork Maintenance

The system uses a patched reedline version:

```bash
# Check reedline integration
grep -A 3 "\[patch.crates-io\]" Cargo.toml

# Update reedline fork if needed
cd third_party/reedline
git pull upstream main  # If upstream updates needed
cd ../..
cargo build --release  # Rebuild with updated reedline
```

## Emergency Procedures

### Emergency Rollback

If shims cause system instability:

```bash
# Use provided rollback script
./scripts/rollback.sh

# Manual rollback if script fails
mv ~/.substrate/shims ~/.substrate/shims.disabled
export PATH="$SHIM_ORIGINAL_PATH"
hash -r
```

### Complete Bypass

For critical situations:

```bash
# Temporary bypass for single command
SHIM_BYPASS=1 git status

# Global bypass (disables all tracing)
export SHIM_BYPASS=1
```

### Recovery from Broken Environment

```bash
# If PATH is completely broken
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

# If you can't access the rollback script
rm -rf ~/.substrate/shims
unset SHIM_ORIGINAL_PATH SHIM_TRACE_LOG BASH_ENV
hash -r
```

## Integration Monitoring

### Claude Code Integration

```bash
# Verify BASH_ENV setup
echo $BASH_ENV
test -f "$BASH_ENV" && echo "BASH_ENV file exists"

# Test non-interactive execution
bash -c 'which git; git --version'

# Check integration logging
jq 'select(.component == "shim")' ~/.trace_shell.jsonl | tail -5
```

### Hash Pinning Verification

```bash
# Check hash table state
hash -l | grep -E "(git|npm|node)"

# Verify shim resolution
type git npm node
which -a git npm node

# Test hash pinning
hash -p "$HOME/.substrate/shims/git" git
type git  # Should show pinned path
```

## Monitoring Integration

### Structured Logging Analysis

Current JSONL format supports various log analysis tools:

```bash
# Event type distribution
jq -r '.event_type' ~/.trace_shell.jsonl | sort | uniq -c

# Component breakdown
jq -r '.component' ~/.trace_shell.jsonl | sort | uniq -c

# Command frequency analysis
jq -r '.command' ~/.trace_shell.jsonl | sort | uniq -c | sort -nr | head -20

# Session duration analysis
jq 'select(.event_type == "command_complete") | .duration_ms' ~/.trace_shell.jsonl | sort -n
```

### Performance Analysis

```bash
# Commands by execution time
jq 'select(.duration_ms != null) | {command, duration_ms}' ~/.trace_shell.jsonl | sort -k2 -nr | head -10

# Cache hit analysis (compare early vs late executions)
jq '.duration_ms // empty' ~/.trace_shell.jsonl | head -20 > /tmp/early.txt
jq '.duration_ms // empty' ~/.trace_shell.jsonl | tail -20 > /tmp/late.txt
echo "Early avg:" $(awk '{sum+=$1} END {print sum/NR}' /tmp/early.txt)
echo "Late avg:" $(awk '{sum+=$1} END {print sum/NR}' /tmp/late.txt)
```

## Support and Debugging

### Debug Logging

```bash
# Enable comprehensive debug logging
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
export SUBSTRATE_PTY_DEBUG=1

# Test command with full debugging
substrate -c "git status"
```

### Diagnostic Information Collection

```bash
# System state dump
echo "=== Environment ==="
echo "PATH: $PATH"
echo "SHIM_ORIGINAL_PATH: $SHIM_ORIGINAL_PATH"
echo "SHIM_TRACE_LOG: $SHIM_TRACE_LOG"
echo "BASH_ENV: $BASH_ENV"

echo "=== Shim Installation ==="
ls -la ~/.substrate/shims/ | head -10
file ~/.substrate/shims/.shimbin

echo "=== Recent Activity ==="
tail -5 ~/.trace_shell.jsonl | jq '.'

echo "=== Performance ==="
jq '.duration_ms // empty' ~/.trace_shell.jsonl | tail -20 | awk '{sum+=$1} END {print "Recent avg:", sum/NR "ms"}'
```

### Known Limitations

1. **Absolute path bypass**: Commands invoked with absolute paths skip shimming
2. **Shell builtin limitation**: Built-ins in non-substrate shells aren't captured
3. **Windows PTY**: Live resize not yet supported on Windows ConPTY
4. **Log interleaving**: Large entries may interleave in high-concurrency scenarios
5. **Reedline dependency**: Requires maintained fork for shell functionality

For issues not covered in this guide, collect diagnostic information and refer to the codebase or OLD_OPS.md for planned enhancements.