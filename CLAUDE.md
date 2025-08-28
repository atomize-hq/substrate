# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Substrate is a command tracing ecosystem written in Rust that provides transparent command interception, a custom shell with multiple execution modes, and structured logging for full visibility into command execution patterns. It consists of two main binaries:
- `substrate`: Custom shell with interactive REPL, wrap mode, script execution, and pipe processing
- `substrate-shim`: Binary shimming tool for command interception

## Development Commands

### Building
```bash
# Development build
cargo build

# Release build (recommended for deployment)
cargo build --release

# Build artifacts are located in:
# - target/debug/substrate & target/debug/substrate-shim (dev)
# - target/release/substrate & target/release/substrate-shim (release)
```

### Testing
```bash
# Run all tests
cargo test

# Run with output for debugging
cargo test -- --nocapture

# Run specific test suites
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests
cargo test -p substrate             # Shell-specific tests
cargo test -p substrate-shim        # Shim-specific tests

# Run integration tests (requires workspace dir for binary discovery)
CARGO_WORKSPACE_DIR=$PWD cargo test --test integration

# Run a single test
cargo test test_name -- --exact
```

### Linting & Formatting
```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Fix clippy warnings automatically
cargo clippy --fix
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs without dependencies
cargo doc --no-deps
```

## High-Level Architecture

### Workspace Structure
The project uses a Cargo workspace with four main crates:

1. **`crates/common/`**: Shared utilities and log schema
   - `src/lib.rs`: Core utilities including path deduplication and credential redaction
   - `src/paths.rs`: Cross-platform path handling for substrate directories
   - Log schema constants for consistent structured logging
   - Used by both shell and shim components

2. **`crates/shell/`**: Custom shell implementation (`substrate` binary)
   - `src/lib.rs`: Core shell modes (interactive, wrap, script, pipe)
   - `src/pty_exec.rs`: PTY terminal emulation and signal handling for Unix systems
   - `src/shim_deploy.rs`: Automatic shim deployment with version tracking
   - `src/lock.rs`: Process locking for safe multi-instance operations
   - Built-in commands: cd, pwd, export, unset, exit
   - Session and command ID generation using UUIDv7
   - Structured JSONL logging with automatic rotation
   - Integration with patched Reedline for interactive REPL

3. **`crates/shim/`**: Binary shimming implementation (`substrate-shim` binary)
   - `src/lib.rs`: Core shimming logic and main entry point
   - `src/context.rs`: Environment detection and configuration management
   - `src/resolver.rs`: Binary path resolution with intelligent caching
   - `src/logger.rs`: Structured JSONL logging with credential redaction
   - `src/exec.rs`: Cross-platform command execution with signal handling
   - Session correlation and depth tracking
   - Binary fingerprinting (SHA-256)
   - Comprehensive credential redaction

4. **`src/`**: Thin binary entry points (workspace root)
   - `main.rs`: Entry point for `substrate` binary - delegates to `substrate_shell::run_shell()`
   - `shim_main.rs`: Entry point for `substrate-shim` binary - delegates to `substrate_shim::run_shim()`
   - These create the two main binaries: `target/debug/substrate` and `target/debug/substrate-shim`


### Key Design Patterns

1. **Thin Binary, Thick Library**: Main binaries (`main.rs`) are minimal, with core logic in library modules (`lib.rs`)

2. **Session Correlation**: Uses UUIDv7 for tracking command chains across nested executions via environment variables:
   - `SHIM_SESSION_ID`: Correlates all commands in a session
   - `SHIM_PARENT_CMD_ID`: Links shim commands to parent shell commands
   - `SHIM_DEPTH`: Tracks nesting level

3. **PTY Detection**: Automatically detects when to use PTY based on command patterns (interactive shells, TUI apps, REPLs)

4. **Credential Redaction**: Comprehensive pattern matching in `substrate_common::redact_sensitive()` for security

5. **Emergency Bypass**: `SHIM_BYPASS=1` environment variable for critical situations

6. **Automatic Shim Deployment**: Shims deploy automatically on first run with version tracking

### Directory Structure

Substrate uses `~/.substrate/` for all its data:
- `~/.substrate/shims/` - Command interception symlinks/binaries
- `~/.substrate/.substrate.lock` - Multi-instance lock file  
- `~/.substrate/shims/.version` - Version tracking with metadata
- `~/.substrate/logs/` - Future: centralized logging
- `~/.substrate/cache/` - Future: command cache

### Important Environment Variables

Critical for operation:
- `SHIM_ORIGINAL_PATH`: Clean PATH for binary resolution (required for shim)
- `SHIM_TRACE_LOG`: Log output destination (default: `~/.trace_shell.jsonl`)
- `SHIM_SESSION_ID`: Session correlation ID (auto-generated if not set)
- `SHIM_BYPASS`: Emergency bypass mode (set to `1` to disable tracing)
- `SUBSTRATE_NO_SHIMS`: Set to `1` to disable automatic shim deployment

For debugging and development:
- `SHIM_LOG_OPTS`: Logging options (`raw`, `resolve`, or `raw,resolve`)
  - `raw`: Disables credential redaction (security sensitive)
  - `resolve`: Includes binary resolution paths in logs
- `SHIM_CACHE_BUST`: Set to `1` to force cache invalidation for testing
- `SHIM_FSYNC`: Set to `1` to force filesystem sync for maximum log durability
- `RUST_LOG`: Standard Rust logging (set to `debug` for detailed output)
- `SUBSTRATE_PTY_DEBUG`: Set to `1` to enable PTY debugging output

## Testing Strategy

When modifying the codebase:
1. **Unit tests**: Located in `src/lib.rs` files as `#[cfg(test)]` modules
2. **Integration tests**: Located in `tests/` directories - require `CARGO_WORKSPACE_DIR=$PWD` environment variable for binary discovery
3. **Shell mode testing**: Covers interactive, wrap (-c), script (-f), pipe modes
4. **PTY tests**: Validate terminal emulation (Unix only)
5. **Security tests**: Verify credential redaction patterns

### Integration Test Requirements
Integration tests need to locate the built binaries (`target/debug/substrate` and `target/debug/substrate-shim`). They use the `CARGO_WORKSPACE_DIR` environment variable to find the workspace root, falling back to relative paths if not set. Always run integration tests with:
```bash
CARGO_WORKSPACE_DIR=$PWD cargo test --test integration
```

## Shim Deployment Implementation

### Automatic Deployment
- Uses symlinks on Unix for efficiency
- Falls back to file copies on Windows
- Version checking via `env!("CARGO_PKG_VERSION")`
- Atomic deployment using tempfile crate
- Process locking with 5-second timeout


## Deployment and Management

### CLI Commands for Shim Management
- `substrate --shim-status`: Check deployment status and version
- `substrate --shim-deploy`: Force redeployment of shims
- `substrate --shim-remove`: Remove all deployed shims
- `substrate --shim-skip`: Skip automatic deployment for this run

### Shell Execution Modes
- **Interactive mode**: `substrate` - Full REPL with history and completion
- **Wrap mode**: `substrate -c "command"` - Execute single command with tracing
- **Script mode**: `substrate -f script.sh` - Execute script file with tracing
- **Pipe mode**: `echo "command" | substrate` - Process commands from stdin

## Third-Party Dependencies

The project includes a patched version of `reedline` in `third_party/reedline/` for the interactive shell REPL functionality. This is configured via `[patch.crates-io]` in the root `Cargo.toml`.

## Common Development Tasks

### Adding a New Built-in Shell Command
Built-in commands are handled in `crates/shell/src/lib.rs` in the `execute_builtin()` function. Add new cases to the match statement.

### Modifying Credential Redaction
Update the `redact_sensitive()` function in `crates/common/src/lib.rs` to add new patterns.

### Adding PTY Support for a New Command
Modify the `should_use_pty()` function in `crates/shell/src/lib.rs` to add detection for new interactive commands.

### Debugging Path Resolution
Set `SHIM_LOG_OPTS=resolve` to log path resolution details, or use `SHIM_CACHE_BUST=1` to force cache invalidation.

### Environment Setup for Comprehensive Debugging
```bash
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
export SUBSTRATE_PTY_DEBUG=1
export SHIM_CACHE_BUST=1  # Force cache invalidation
```