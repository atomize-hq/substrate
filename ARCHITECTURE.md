# Substrate Architecture Documentation

## Overview

This document provides in-depth technical details about the Substrate command tracing system architecture, implementation decisions, and development guidelines.

## System Architecture

### Component Interaction Flow

```
User Command → Shell Resolution → PATH Lookup → Shim Intercept → Logging → Real Binary Execution
                                        ↓
                                  Substrate Shell
                                        ↓
                              (Builtin or External Command)
```

### Crate Dependencies

```
substrate-common (base utilities)
    ↑
    ├── substrate-shim (command interception)
    ├── substrate-shell (custom shell)
    └── substrate-supervisor (process management)
```

## Core Components

### 1. Shim (`crates/shim/`)

**Purpose**: Transparent command interception via binary replacement in PATH.

**Key Design Decisions**:
- **Copy-based deployment**: Each command gets its own shim copy (not symlinks) to ensure argv[0] preservation
- **Cache strategy**: LRU cache for binary resolution, keyed by command + search paths (not CWD)
- **Depth tracking**: Uses `SHIM_DEPTH` environment variable to track nested execution levels
- **Session correlation**: UUIDv7-based session IDs for command chain tracking
- **Bypass mode**: Nested shims (SHIM_ACTIVE set) bypass to real binary to prevent recursion
- **Call stack tracking**: Maintains SHIM_CALL_STACK (max 8 items, deduped) for debugging
- **Parent correlation**: Links to shell cmd_id via SHIM_PARENT_CMD_ID

**Critical Code Paths**:
```rust
main() 
  → detect_context()          // Environment setup
  → should_skip_shimming()    // Check SHIM_ACTIVE for bypass
    ↓ (if set)
  → execute_real_binary_bypass() // Direct execution, bypass logging
    ↓ (if not set)
  → setup_execution_env()     // Set SHIM_ACTIVE, CALLER, CALL_STACK
  → resolve_real_binary()     // Path resolution with caching
  → log_command_start()       // Pre-execution logging
  → execute_command()         // Fork/exec with signal forwarding
  → log_command_complete()    // Post-execution logging with exit status
```

**Performance Optimizations**:
- Binary resolution cache reduces filesystem stat() calls
- Lazy initialization of log file handles
- Optional resolved_path computation (`SHIM_LOG_OPTS=resolve`)

### 2. Shell (`crates/shell/`)

**Purpose**: Custom shell providing controlled execution environment with comprehensive tracing.

**Execution Modes**:
```rust
pub enum ShellMode {
    Interactive { use_pty: bool },  // REPL with optional PTY
    Wrap(String),                   // Single command (-c)
    Script(PathBuf),                // Script file (-f)
    Pipe,                           // Stdin commands
}
```

**Built-in Commands**:
- Handled directly without spawning external processes
- State changes (cd, export) modify shell environment
- Logged as `builtin_command` events

**PTY Support** (Unix only):
- Uses `libc::openpty()` for terminal allocation
- Handles window resize via SIGWINCH
- Maintains terminal state restoration on exit

### 3. Common (`crates/common/`)

**Shared Utilities**:
- `dedupe_path()`: PATH deduplication while preserving order
- `redact_sensitive()`: Credential redaction logic
- `log_schema`: Standardized field names for JSON logs

### 4. Supervisor (`crates/supervisor/`)

**Status**: Partially implemented

**Planned Features**:
- Process lifecycle management
- Resource monitoring
- Automatic restart policies
- Health checking

## Logging Architecture

### Log Schema

All components use consistent JSONL format with these core fields:

```typescript
interface LogEntry {
  ts: string;                  // ISO 8601 timestamp
  event_type: string;          // Event classification
  session_id: string;          // UUIDv7 for correlation
  cmd_id?: string;             // Command-specific UUID
  component: "shim" | "shell" | "supervisor";
  // ... additional fields per event type
}
```

### Log Rotation

- Automatic rotation at 50MB (configurable via `TRACE_LOG_MAX_MB`)
- Best-effort: existing file handles continue writing to rotated file
- New processes create fresh log files post-rotation

### Credential Redaction

**Token-aware patterns**:
- Flag + value: `-u user:pass` → `-u *** ***`
- Headers: `-H "Authorization: Bearer X"` → `-H "Authorization: ***"`
- Environment: `token=value` → `token=***`

**Bypass**: `SHIM_LOG_OPTS=raw` disables all redaction

## Signal Handling

### Unix Signal Management

**SIGINT/SIGTERM Propagation**:
```c
// Child processes become group leaders
setpgid(0, 0)  // Before exec

// Parent forwards signals to entire group
killpg(child_pgid, signal)
```

**Exit Code Convention**:
- Normal exit: Process exit code
- Signal termination: 128 + signal number (POSIX convention)

### Shell Signal Handling

Interactive mode installs handlers for:
- SIGINT: Forward to running child, continue REPL if no child
- SIGTERM: Forward to child process group
- SIGWINCH (PTY mode): Resize PTY window

## Security Considerations

### Path Resolution Security

1. **SHIM_ORIGINAL_PATH validation**: Must not contain shim directory
2. **Binary fingerprinting**: SHA-256 hash of resolved binary
3. **Permission checks**: Executable bit verification

### Log Security

- Files created with 0600 permissions (user-only)
- Sensitive data redaction (can be bypassed for debugging)
- No logging of file contents, only command metadata

### Emergency Bypass

`SHIM_BYPASS=1` environment variable skips all shimming logic for recovery scenarios.

### Environment Variables

All shim-related environment variables use the `SHIM_` prefix for consistency:

- `SHIM_ACTIVE`: Signals nested shim call (triggers bypass mode)
- `SHIM_DEPTH`: Tracks nesting depth (0-based)
- `SHIM_SESSION_ID`: UUIDv7 for command chain correlation
- `SHIM_ORIGINAL_PATH`: Clean PATH without shim directory
- `SHIM_TRACE_LOG`: Path to JSONL trace log
- `SHIM_CALLER`: First shim in the call chain
- `SHIM_CALL_STACK`: Comma-separated chain (capped at 8, deduped)
- `SHIM_PARENT_CMD_ID`: Links to substrate shell cmd_id
- `SHIM_BYPASS`: Emergency bypass mode (1 = skip all tracing)
- `SHIM_CACHE_BUST`: Force cache invalidation

## Development Guidelines

### Adding New Features

1. **Shared code** goes in `substrate-common`
2. **Logging changes** must maintain schema compatibility
3. **New environment variables** should be prefixed with `SHIM_` or `SUBSTRATE_`
4. **Cross-platform code** should use `cfg!` attributes appropriately

### Testing Strategy

**Unit Tests**: Internal logic, no external dependencies
```bash
cargo test --lib
```

**Integration Tests**: Full command execution paths
```bash
cargo test --test integration
```

**Manual Testing Checklist**:
- [ ] Basic command execution
- [ ] Nested command execution (check depth tracking)
- [ ] Signal handling (Ctrl+C during long-running command)
- [ ] Log rotation at boundary
- [ ] Credential redaction for various patterns
- [ ] PTY mode with interactive programs (vim, less)
- [ ] CI mode error propagation

### Performance Profiling

**Key Metrics to Monitor**:
- Shim startup time (target: <5ms)
- Memory usage per shim (target: <3MB)
- Log write throughput
- Cache hit rate for path resolution

**Profiling Commands**:
```bash
# Time overhead
time -p target/release/shim echo test

# Memory usage
/usr/bin/time -l target/release/shim echo test

# Strace for syscall analysis (Linux)
strace -c target/release/shim echo test
```

### Known Limitations

1. **Absolute path bypass**: Commands invoked via absolute paths skip shimming
2. **Builtin shell commands**: Shell builtins in non-substrate shells aren't captured
3. **Windows support**: PTY functionality requires ConPTY (not yet implemented)
4. **Log atomicity**: Large entries (>8MB) may interleave in multi-process scenarios

## Future Enhancements

### Planned Features

1. **Enhanced Supervisor**
   - Process tree visualization
   - Resource limit enforcement
   - Dependency management

2. **Windows Full Support**
   - ConPTY integration for terminal emulation
   - PowerShell provider for better integration
   - Windows service mode

3. **Advanced Shell Features**
   - Job control (fg/bg/jobs)
   - Command aliases
   - Programmable completion

4. **Observability Improvements**
   - Metrics export (Prometheus/OpenTelemetry)
   - Real-time streaming API
   - Query language for log analysis

### Extension Points

**Custom Redaction Rules**:
```rust
// Add to substrate-common/src/lib.rs
pub trait RedactionRule {
    fn should_redact(&self, arg: &str) -> bool;
    fn redact(&self, arg: &str) -> String;
}
```

**Shell Command Plugins**:
```rust
// Potential plugin interface
pub trait ShellPlugin {
    fn name(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<i32>;
}
```

## Debugging Guide

### Common Issues

**Issue**: Commands not being intercepted
- Check: Is shim directory first in PATH?
- Check: Run `hash -r` to clear shell cache
- Check: Verify shim binary exists and is executable

**Issue**: High memory usage
- Check: Log file size (rotation working?)
- Check: Cache size limits
- Check: File descriptor leaks

**Issue**: Signals not handled properly
- Check: Process group setup (`ps -o pid,pgid,cmd`)
- Check: Signal mask inheritance
- Check: PTY vs non-PTY mode differences

### Debug Environment Variables

```bash
# Maximum verbosity
export RUST_LOG=trace
export SHIM_LOG_OPTS=raw,resolve
export SHIM_FSYNC=1

# Analyze specific session
export SHIM_SESSION_ID=debug-session-1
grep debug-session-1 ~/.trace_shell.jsonl | jq .
```

## Code Organization

### Module Responsibilities

```
substrate-shim/
├── main.rs           # Entry point, minimal logic
├── lib.rs            # Public API, orchestration
├── context.rs        # Environment detection
├── resolver.rs       # Binary path resolution
├── cache.rs          # LRU cache implementation
├── logger.rs         # Structured logging
├── exec.rs           # Process execution
└── redact.rs         # Credential redaction

substrate-shell/
├── main.rs           # Entry point
├── lib.rs            # Shell modes, command routing
├── pty.rs            # PTY management (Unix)
├── builtins.rs       # Built-in command handlers
└── repl.rs           # Interactive mode
```

### Error Handling Strategy

- **Library code**: Return `Result<T, anyhow::Error>`
- **Binary code**: Handle errors, set appropriate exit codes
- **Logging errors**: Best-effort, never fail the primary operation
- **Critical errors**: Log to stderr, return non-zero exit code

## Contributing

### Pull Request Checklist

- [ ] Tests pass: `cargo test`
- [ ] Formatting: `cargo fmt`
- [ ] Linting: `cargo clippy -- -D warnings`
- [ ] Documentation updated
- [ ] CHANGELOG entry added
- [ ] Performance impact assessed
- [ ] Security implications reviewed

### Review Focus Areas

1. **Security**: Path traversal, command injection, privilege escalation
2. **Performance**: Startup overhead, memory usage, I/O patterns
3. **Compatibility**: Shell differences, platform variations
4. **Reliability**: Error handling, signal safety, race conditions