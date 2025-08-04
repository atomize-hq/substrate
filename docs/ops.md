# Substrate Operations Guide

## Overview

This guide covers operational aspects of the Substrate command tracing system, including deployment, monitoring, troubleshooting, and maintenance procedures.

## Deployment

### Phase 1: Basic Shim Deployment

1. **Build the shim binary**:
   ```bash
   cargo build --release -p substrate-shim
   ```

2. **Stage shims using the deployment script**:
   ```bash
   scripts/stage_shims.sh
   ```

3. **Activate shims in your shell**:
   ```bash
   # Create clean ORIGINAL_PATH (strips existing shim directory)
   ORIGINAL_PATH=$(python3 -c "import os; sd='$HOME/.cmdshim_rust'; print(':'.join(p for p in os.environ.get('PATH','').split(':') if p and p.rstrip('/')!=sd.rstrip('/')))")
   export ORIGINAL_PATH
   export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
   hash -r
   ```

4. **Verify deployment**:
   ```bash
   which -a git  # Should show shim first
   type -a git   # Should show shim first
   TRACE_LOG_FILE=/tmp/test.jsonl git --version
   cat /tmp/test.jsonl  # Should contain JSON log entry
   ```

### Phase 2: Non-Interactive Shell Support

1. **Create BASH_ENV file**:
   ```bash
   scripts/create_bashenv.sh
   ```

2. **Test with non-interactive shells**:
   ```bash
   BASH_ENV="$HOME/.substrate_bashenv" bash -c 'which git; git --version'
   ```

## Performance Monitoring

### Key Metrics

Monitor these performance indicators:

```bash
# Average shim overhead
jq '.duration_ms' ~/.trace_shell.jsonl | awk '{sum+=$1; count++} END {print "avg:", sum/count "ms"}'

# 95th percentile latency
jq -r '.duration_ms' ~/.trace_shell.jsonl | sort -n | awk 'NR==int(NR*0.95)'

# Command execution frequency
jq -r '.command' ~/.trace_shell.jsonl | sort | uniq -c | sort -nr | head -10

# Cache effectiveness (compare early vs late execution times)
jq '.duration_ms' ~/.trace_shell.jsonl | head -100 > /tmp/cold_cache.txt
jq '.duration_ms' ~/.trace_shell.jsonl | tail -100 > /tmp/warm_cache.txt
```

### Performance Targets

- **Shim overhead**: < 5ms per execution on macOS/Linux
- **Memory usage**: < 1MB resident per shim process  
- **Cache hit rate**: ~40% reduction in stat() calls after warmup
- **Log file growth**: Bounded by external rotation

## Security Operations

### Log File Security

Substrate automatically creates log files with user-only permissions (0o600):

```bash
# Verify log file permissions
ls -la ~/.trace_shell.jsonl
# Should show: -rw------- (user read/write only)
```

### Credential Redaction

Substrate automatically redacts sensitive information:

- **Key-value patterns**: `token=secret`, `password=mypass`, `SECRET=value`
- **Flag-value patterns**: `--token secret`, `-p password`, `--apikey value`

To disable redaction for debugging:
```bash
export SHIM_LOG_OPTS=raw
```

### Security Monitoring

```bash
# Check for potential credential leaks (when redaction is disabled)
grep -i "token\|password\|secret\|key" ~/.trace_shell.jsonl

# Monitor failed executions
jq 'select(.exit_code != 0) | {command, exit_code, argv}' ~/.trace_shell.jsonl

# Track signal terminations (potential security issues)
jq 'select(.term_signal != null) | {command, term_signal, argv}' ~/.trace_shell.jsonl
```

## Error Monitoring and Troubleshooting

### Common Issues

#### 1. Shim Not Found in PATH

**Symptoms**: Commands execute without logging, `which` shows system binary

**Diagnosis**:
```bash
echo $PATH  # Check if shim directory is first
which -a git  # Should show shim first
hash -r  # Clear bash hash table
```

**Solution**:
```bash
# Reactivate shims
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
hash -r
```

#### 2. Recursion Detection

**Symptoms**: Commands fail with "recursive shim detection" errors

**Diagnosis**:
```bash
echo $SHIM_ACTIVE  # Should be unset in normal shells
```

**Solution**:
```bash
unset SHIM_ACTIVE
```

#### 3. Permission Denied Errors

**Symptoms**: Commands fail to execute

**Diagnosis**:
```bash
ls -la ~/.cmdshim_rust/  # Check shim permissions
file ~/.cmdshim_rust/git  # Verify binary type
```

**Solution**:
```bash
chmod +x ~/.cmdshim_rust/*
```

#### 4. macOS Bash Compatibility Issues

**Symptoms**: BASH_ENV errors on macOS

**Diagnosis**:
```bash
echo $BASH_VERSION  # Check bash version
```

**Solution**: The create_bashenv.sh script automatically handles bash 3.2 compatibility.

### Diagnostic Commands

```bash
# Check shim binary integrity
file ~/.cmdshim_rust/.shimbin
md5sum ~/.cmdshim_rust/.shimbin ~/.cmdshim_rust/*

# Verify path resolution
SHIM_CACHE_BUST=1 TRACE_LOG_FILE=/tmp/debug.jsonl git --version
jq '.duration_ms' /tmp/debug.jsonl  # Should be slower without cache

# Test signal handling (Unix)
timeout 2s yes > /dev/null  # Should log SIGTERM
jq 'select(.term_signal != null)' ~/.trace_shell.jsonl | tail -1
```

## Maintenance Procedures

### Log Rotation

Implement external log rotation to prevent unbounded growth:

```bash
# Example logrotate configuration
cat > /etc/logrotate.d/substrate <<EOF
/home/*/.trace_shell.jsonl {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0600
}
EOF
```

### Cache Management

The resolution cache is automatically managed, but can be controlled:

```bash
# Disable cache for debugging
export SHIM_CACHE_BUST=1

# Monitor cache effectiveness
# (Compare execution times before/after cache warmup)
```

### Updates and Upgrades

1. **Build new shim binary**:
   ```bash
   cargo build --release -p substrate-shim
   ```

2. **Update shims atomically**:
   ```bash
   # Backup current shims
   cp -r ~/.cmdshim_rust ~/.cmdshim_rust.backup
   
   # Deploy new version
   scripts/stage_shims.sh
   ```

3. **Verify update**:
   ```bash
   ~/.cmdshim_rust/.shimbin --version 2>/dev/null || echo "Shim binary updated"
   ```

## Emergency Procedures

### Emergency Rollback

If shims cause system instability:

```bash
# Immediate rollback
scripts/rollback.sh

# Manual rollback if script fails
mv ~/.cmdshim_rust ~/.cmdshim_rust.disabled
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
hash -r
```

### Disable Immutable Shims

If shims were made immutable for security:

```bash
# Remove immutable flags (macOS)
chflags nouchg ~/.cmdshim_rust/*

# Then proceed with normal rollback
scripts/rollback.sh
```

## Known Limitations

### Absolute Path Commands

Commands invoked with absolute paths cannot be intercepted:

```bash
/usr/bin/git status  # Will NOT be logged
git status           # Will be logged (uses PATH resolution)
```

**Workaround**: Use relative command names when possible.

### Windows Signal Handling

Signal capture is limited on Windows:
- SIGINT/SIGTERM mapping is basic
- Some termination methods may not be captured

### Large Log Entries

Log entries exceeding ~8MB may have interleaved writes, though this is rare in practice.

### Set-uid Binaries

Shims should not be used with set-uid binaries or privilege elevation scenarios.

## Monitoring Integration

### Structured Logging

Substrate produces JSONL logs suitable for ingestion by:
- **ELK Stack**: Logstash can parse JSONL directly
- **Splunk**: Universal Forwarder with JSON parsing
- **Prometheus**: Custom exporters can parse logs for metrics

### Sample Queries

```bash
# Commands by user (multi-user systems)
jq -r '.user' ~/.trace_shell.jsonl | sort | uniq -c

# Execution patterns by time of day
jq -r '.ts' ~/.trace_shell.jsonl | cut -dT -f2 | cut -d: -f1 | sort | uniq -c

# Failed commands with context
jq 'select(.exit_code != 0) | {ts, command, argv, cwd, exit_code}' ~/.trace_shell.jsonl
```

## Support and Debugging

For issues not covered in this guide:

1. **Enable debug logging**:
   ```bash
   export SHIM_LOG_OPTS=raw
   export TRACE_LOG_FILE=/tmp/substrate_debug.jsonl
   ```

2. **Collect diagnostic information**:
   ```bash
   echo "PATH: $PATH"
   echo "ORIGINAL_PATH: $ORIGINAL_PATH"
   echo "SHIM_ACTIVE: $SHIM_ACTIVE"
   ls -la ~/.cmdshim_rust/
   tail -10 ~/.trace_shell.jsonl
   ```

3. **Test minimal reproduction case**:
   ```bash
   TRACE_LOG_FILE=/tmp/repro.jsonl git --version
   cat /tmp/repro.jsonl
   ```