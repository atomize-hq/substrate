# Substrate Architecture Documentation

## Overview

This document provides in-depth technical details about the Substrate command tracing system architecture, implementation decisions, and development guidelines based on the current implementation.

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
    ├── substrate (custom shell)
    └── substrate-supervisor (process management)
```

## Core Components

### 1. Shim (`crates/shim/`)

**Purpose**: Transparent command interception via binary replacement in PATH.

**Package**: `substrate-shim` (binary: `substrate-shim`)

**Key Design Decisions**:
- **Copy-based deployment**: Each command gets its own shim copy (not symlinks) to ensure argv[0] preservation
- **Path resolution with caching**: Intelligent caching for binary resolution performance
- **Depth tracking**: Uses `SHIM_DEPTH` environment variable to track nested execution levels
- **Session correlation**: UUIDv7-based session IDs for command chain tracking
- **Bypass mode**: `SHIM_ACTIVE` environment variable triggers bypass to prevent recursion
- **Call stack tracking**: Maintains call chain for debugging via `SHIM_CALL_STACK`
- **Parent correlation**: Links to shell cmd_id via `SHIM_PARENT_CMD_ID`

**Current Module Structure**:
```rust
substrate-shim/
├── main.rs           # Entry point, minimal logic
├── lib.rs            # Public API, orchestration
├── context.rs        # Environment detection and configuration
├── resolver.rs       # Binary path resolution with caching
├── exec.rs           # Process execution and signal handling
└── logger.rs         # Structured logging implementation
```

**Critical Code Paths**:
```rust
main() 
  → detect_context()          // Environment setup via context.rs
  → should_skip_shimming()    // Check SHIM_ACTIVE for bypass
    ↓ (if set)
  → execute_real_binary_bypass() // Direct execution, bypass logging
    ↓ (if not set)
  → setup_execution_env()     // Set SHIM_ACTIVE, CALLER, CALL_STACK
  → resolve_real_binary()     // Path resolution with caching (resolver.rs)
  → log_command_start()       // Pre-execution logging (logger.rs)
  → execute_command()         // Fork/exec with signal forwarding (exec.rs)
  → log_command_complete()    // Post-execution logging with exit status
```

### 2. Shell (`crates/shell/`)

**Purpose**: Custom shell providing controlled execution environment with comprehensive tracing.

**Package**: `substrate` (binary: `substrate`)

**Execution Modes**:
```rust
pub enum ShellMode {
    Interactive { use_pty: bool },  // REPL with optional PTY
    Wrap(String),                   // Single command (-c)
    Script(PathBuf),                // Script file (-f)
    Pipe,                           // Stdin commands
}
```

**Current Module Structure**:
```rust
substrate/
├── main.rs           # Entry point
├── lib.rs            # Shell modes, command routing, built-ins
└── pty_exec.rs       # PTY management and execution
```

**Built-in Commands**:
- Handled directly without spawning external processes in `execute_builtin()` function
- State changes (cd, export) modify shell environment
- Logged as `builtin_command` events
- Current built-ins: cd, pwd, export, unset, exit, quit

**PTY Support**:
- **Unix/macOS**: Full support using `portable-pty` crate
- **Windows**: ConPTY support via `portable-pty`
- Uses `libc::openpty()` for terminal allocation on Unix
- Handles window resize via SIGWINCH (Unix only)
- Maintains terminal state restoration on exit
- Global SIGWINCH handler using `signal-hook`

**Reedline Integration**:
- Uses patched version from `third_party/reedline/`
- Configured via `[patch.crates-io]` in root `Cargo.toml`
- Provides REPL functionality with history, completion, and editing

### 3. Common (`crates/common/`)

**Purpose**: Shared utilities and standardized logging schema.

**Package**: `substrate-common`

**Shared Utilities**:
- `dedupe_path()`: PATH deduplication while preserving order
- `redact_sensitive()`: Credential redaction logic with configurable patterns
- `log_schema`: Standardized field names for JSON logs

**Current Implementation**:
```rust
// Path deduplication
pub fn dedupe_path(path: &str) -> String

// Credential redaction (configurable via SHIM_LOG_OPTS=raw)
pub fn redact_sensitive(arg: &str) -> String

// Log schema constants
pub mod log_schema {
    pub const EVENT_TYPE: &str = "event_type";
    pub const SESSION_ID: &str = "session_id";
    // ... other constants
}
```

### 4. Supervisor (`crates/supervisor/`)

**Purpose**: Process supervision and environment management.

**Package**: `substrate-supervisor`

**Current Status**: Fully implemented with core functionality

**Key Features**:
- Process lifecycle management via `launch_supervised()`
- Environment setup and PATH management
- Session seeding and correlation
- Clean PATH construction with shim integration
- BASH_ENV setup for non-interactive shells

**API**:
```rust
pub struct SupervisorConfig {
    pub shim_dir: PathBuf,
    pub original_path: String,
    pub bash_env_file: Option<PathBuf>,
    pub target_command: Vec<String>,
    pub environment: HashMap<String, String>,
}

pub fn launch_supervised(config: SupervisorConfig) -> Result<()>
```

## Third-Party Dependencies

### Reedline Fork

**Location**: `third_party/reedline/`

**Integration**: Configured via `[patch.crates-io]` in root `Cargo.toml`:
```toml
[patch.crates-io]
reedline = { path = "third_party/reedline" }
```

**Purpose**: Provides interactive shell REPL functionality with:
- Command history and persistence
- Tab completion support
- Line editing with Emacs/Vi modes
- Syntax highlighting capabilities
- Custom prompt support

**Why Forked**: The fork contains minimal modifications needed for substrate-specific integration and behavior.

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

### Event Types

- `command_start`: Command execution begins
- `command_complete`: Command execution finishes with exit code
- `builtin_command`: Shell built-in command executed
- `pty_session_start`: PTY terminal session initiated
- `pty_session_end`: PTY terminal session terminated

### Log File Management

- Default location: `~/.trace_shell.jsonl`
- Automatic rotation at configurable size (default: environment controlled)
- Files created with 0o600 permissions (user-only access)
- Best-effort logging: failures don't interrupt command execution

### Credential Redaction

**Implementation**: `substrate_common::redact_sensitive()`

**Patterns**:
- Token/password patterns: `token=secret` → `token=***`
- Flag-based redaction: `--password`, `-p`, `-H` → `***`
- Header redaction for Authorization, API keys
- Environment variable patterns

**Bypass**: `SHIM_LOG_OPTS=raw` disables all redaction for debugging

## Signal Handling

### Unix Signal Management

**SIGINT/SIGTERM Propagation**:
```c
// Child processes become group leaders
setpgid(0, 0)  // Before exec

// Parent forwards signals to entire group
killpg(child_pgid, signal)
```

**PTY Signal Handling**:
- Global SIGWINCH handler using `signal-hook` crate
- Thread-safe PTY resizing via shared state
- Debug logging via `SUBSTRATE_PTY_DEBUG`

**Exit Code Convention**:
- Normal exit: Process exit code
- Signal termination: 128 + signal number (POSIX convention)

### Shell Signal Handling

Interactive mode installs handlers for:
- SIGINT: Forward to running child, continue REPL if no child
- SIGTERM: Forward to child process group
- SIGWINCH (PTY mode): Resize PTY window using global handler

## Environment Variables

All shim-related environment variables use the `SHIM_` prefix, shell-specific use `SUBSTRATE_`:

**Core Shim Variables**:
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
- `SHIM_LOG_OPTS`: Logging options (`raw`, `resolve`)

**Shell-Specific Variables**:
- `SUBSTRATE_FORCE_PTY`: Force PTY for all commands
- `SUBSTRATE_DISABLE_PTY`: Disable PTY globally (escape hatch)
- `SUBSTRATE_PTY_DEBUG`: Enable PTY debug logging
- `SUBSTRATE_PTY_PIPELINE_LAST`: PTY for last pipeline segment

**Integration Variables**:
- `BASH_ENV`: Bash startup script for non-interactive shells
- `TRACE_LOG_MAX_MB`: Log rotation size limit

## Security Considerations

### Path Resolution Security

1. **SHIM_ORIGINAL_PATH validation**: Must not contain shim directory
2. **Binary fingerprinting**: SHA-256 hash of resolved binary (implemented in resolver.rs)
3. **Permission checks**: Executable bit verification
4. **PATH sanitization**: Prevents injection attacks

### Log Security

- Files created with 0o600 permissions (user-only)
- Sensitive data redaction (can be bypassed for debugging)
- No logging of file contents, only command metadata
- Best-effort approach: logging failures don't break functionality

### Emergency Bypass

`SHIM_BYPASS=1` environment variable skips all shimming logic for recovery scenarios.

## Performance Characteristics

**Performance Characteristics**:
- **Startup Overhead**: Optimized for minimal latency with intelligent caching
- **Memory Usage**: Designed for efficient resource utilization  
- **Binary Size**: Compact Rust binaries optimized for performance
- **Performance Analysis**: Comprehensive benchmark suite available via `cargo bench`
- **Cache Performance**: Reduces filesystem stat() calls after warmup

**Optimization Strategies**:
- Binary resolution cache in resolver.rs
- Lazy initialization of log file handles
- Optional resolved_path computation (`SHIM_LOG_OPTS=resolve`)
- Efficient PATH deduplication

## Development Guidelines

### Rust Version Requirements

- **MSRV**: 1.74+ (required for uuid v7 features)
- **Edition**: 2021 throughout workspace

### Adding New Features

1. **Shared code** goes in `substrate-common`
2. **Logging changes** must maintain schema compatibility
3. **New environment variables** should be prefixed with `SHIM_` or `SUBSTRATE_`
4. **Cross-platform code** should use `cfg!` attributes appropriately
5. **Reedline changes** should be minimal and well-documented

### Module Responsibilities

**substrate-shim**:
- `context.rs`: Environment detection and configuration management
- `resolver.rs`: Binary path resolution with intelligent caching
- `logger.rs`: Structured JSONL logging with credential redaction
- `exec.rs`: Cross-platform command execution with signal handling

**substrate (shell)**:
- `lib.rs`: Shell modes, command routing, built-in commands
- `pty_exec.rs`: PTY management and terminal emulation

**substrate-common**:
- Shared utilities (path handling, redaction, logging schema)
- Cross-component constants and types

### Error Handling Strategy

- **Library code**: Return `Result<T, anyhow::Error>`
- **Binary code**: Handle errors, set appropriate exit codes
- **Logging errors**: Best-effort, never fail the primary operation
- **Critical errors**: Log to stderr, return non-zero exit code

## Known Limitations

1. **Absolute path bypass**: Commands invoked via absolute paths skip shimming
2. **Shell builtin commands**: Shell builtins in non-substrate shells aren't captured
3. **Windows SIGWINCH**: Live terminal resize not yet supported on Windows
4. **Log atomicity**: Large entries may interleave in multi-process scenarios
5. **Reedline dependency**: Requires maintained fork for shell functionality

## Future Architecture Considerations

### Planned Enhancements

The OLD_ARCHITECTURE.md file contains detailed plans for future enhancements including:
- Enhanced supervisor capabilities
- Advanced shell features (job control, aliases, completion)
- Windows full support improvements
- Observability improvements (metrics export, streaming API)
- Extension points for custom redaction and shell plugins

### Extension Points

**Current Architecture Supports**:
- Custom redaction rules via `substrate_common::redact_sensitive()`
- Environment-based configuration
- Pluggable logging backends via log file path configuration
- Shell mode extensions via command-line arguments

## Contributing

### Pull Request Checklist

- [ ] Tests pass: `cargo test`
- [ ] Formatting: `cargo fmt`
- [ ] Linting: `cargo clippy -- -D warnings`
- [ ] Documentation updated
- [ ] Performance impact assessed
- [ ] Security implications reviewed
- [ ] Reedline fork dependencies considered

### Review Focus Areas

1. **Security**: Path traversal, command injection, privilege escalation
2. **Performance**: Startup overhead, memory usage, I/O patterns
3. **Compatibility**: Shell differences, platform variations
4. **Reliability**: Error handling, signal safety, race conditions
5. **Maintainability**: Code organization, dependency management