# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Substrate is a secure execution layer for AI agent development that provides isolation, audit trails, and centralized policy control. It implements a "world-based" execution model with comprehensive command tracing, resource controls, and network isolation.

Core binaries:
- `substrate`: Custom shell with world integration, interactive REPL, wrap mode, and structured logging
- `substrate-shim`: Binary shimming tool for transparent command interception
- `world-agent`: API service providing isolated execution environments (Linux)

## Development Commands

### Building
```bash
# Build entire workspace
cargo build --workspace

# Release build (recommended for deployment)
cargo build --release

# Build specific components
cargo build -p substrate-shell
cargo build -p substrate-shim
cargo build -p world-agent

# Build artifacts location:
# - target/debug/* (development)
# - target/release/* (production)
```

### Testing
```bash
# Run all tests
cargo test

# Run with output for debugging
cargo test -- --nocapture

# Run specific crate tests
cargo test -p substrate-shell       # Shell tests
cargo test -p substrate-shim        # Shim tests
cargo test -p world                 # World isolation tests (requires Linux)
cargo test -p substrate-broker      # Policy broker tests
cargo test -p substrate-replay      # Replay functionality tests

# Run single test
cargo test test_name -- --exact

# Run module tests (e.g., lock tests)
cargo test -p substrate-shell --lib lock::

# Privileged tests (Linux, requires sudo)
sudo -E RUST_LOG=info cargo test -p world -- --nocapture
```

### Linting & Formatting
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Run clippy with warnings as errors
cargo clippy --workspace -- -D warnings

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
The project uses a Cargo workspace with multiple specialized crates:

#### Core Execution Layer
1. **`crates/shell/`**: Custom shell (`substrate` binary)
   - `src/lib.rs`: Core shell modes with world integration
   - `src/pty_exec.rs`: PTY terminal emulation
   - `src/shim_deploy.rs`: Automatic shim deployment
   - Integrates with world-agent for isolated execution
   - Built-in commands: cd, pwd, export, unset, exit

2. **`crates/shim/`**: Binary shimming (`substrate-shim` binary)
   - Transparent command interception via PATH manipulation
   - Session correlation with UUIDv7
   - Binary fingerprinting (SHA-256)
   - Credential redaction

3. **`crates/common/`**: Shared utilities
   - Log schema constants
   - Path deduplication
   - Credential redaction patterns

#### World Isolation System (Linux)
4. **`crates/world/`**: Linux isolation backends
   - Network namespace management with nftables
   - Cgroup v2 resource controls
   - OverlayFS/copy-diff filesystem isolation
   - Session-based world lifecycle
   - Automatic netns garbage collection

5. **`crates/world-agent/`**: Execution API service
   - REST API over Unix domain socket (`/run/substrate.sock`)
   - WebSocket support for PTY/interactive commands
   - Manages world creation and command execution
   - Filesystem diff tracking

6. **`crates/world-api/`**: World abstraction layer
   - Platform-agnostic API definitions
   - Session management interfaces

#### Policy & Security
7. **`crates/broker/`**: Policy enforcement engine
   - YAML-based policy definitions
   - Command allowlists/denylists
   - Resource budgets and limits
   - Approval workflows

8. **`crates/trace/`**: Tracing aggregation
   - Centralized log collection
   - Session correlation

#### Agent Integration
9. **`crates/agent-api-*`**: AI agent interfaces
   - `agent-api-types`: Shared type definitions
   - `agent-api-core`: Core API traits
   - `agent-api-client`: Client implementations
   - REST endpoints for agent command execution

10. **`crates/host-proxy/`**: Host-to-agent proxy
    - Bridges host and isolated environments
    - Request routing and validation

#### Analysis & Intelligence
11. **`crates/telemetry-lib/`**: Telemetry collection
    - File access tracking
    - Network connection monitoring
    - Execution pattern analysis

12. **`crates/replay/`**: Command replay system
    - Session replay from logs
    - Testing and debugging support

13. **`crates/substrate-graph/`**: Graph database integration
    - Command relationship tracking (planned)
    - Dependency analysis (planned)


### Key Design Patterns

1. **World-Based Isolation**: Commands execute in isolated Linux environments with network namespaces, cgroups, and filesystem isolation

2. **Thin Binary, Thick Library**: Main binaries (`main.rs`) are minimal, with core logic in library modules (`lib.rs`)

3. **Session Correlation**: Uses UUIDv7 for tracking command chains across nested executions:
   - `SHIM_SESSION_ID`: Correlates all commands in a session
   - `SHIM_PARENT_CMD_ID`: Links shim commands to parent shell commands
   - `SHIM_DEPTH`: Tracks nesting level
   - `SUBSTRATE_SESSION_ID`: World session identifier

4. **Agent Communication**: Shell communicates with world-agent via Unix domain socket:
   - Non-PTY: REST API over `/run/substrate.sock`
   - PTY/Interactive: WebSocket for bidirectional streaming

5. **PTY Detection**: Automatically detects when to use PTY based on command patterns (interactive shells, TUI apps, REPLs)

6. **Credential Redaction**: Comprehensive pattern matching in `substrate_common::redact_sensitive()` for security

7. **Emergency Bypass**: `SHIM_BYPASS=1` environment variable for critical situations

8. **Automatic Shim Deployment**: Shims deploy automatically on first run with version tracking

### Directory Structure

Substrate uses `~/.substrate/` for all its data:
- `~/.substrate/shims/` - Command interception symlinks/binaries
- `~/.substrate/.substrate.lock` - Multi-instance lock file
- `~/.substrate/shims/.version` - Version tracking with metadata
- `~/.substrate/logs/` - Centralized logging (when enabled)
- `~/.substrate/cache/` - Command cache (future)
- `/run/substrate.sock` - World-agent Unix domain socket (Linux)
- `/tmp/substrate-worlds/` - World runtime data (Linux)

### Important Environment Variables

Critical for operation:
- `SHIM_ORIGINAL_PATH`: Clean PATH for binary resolution (required for shim)
- `SHIM_TRACE_LOG`: Log output destination (default: `~/.substrate/trace.jsonl`)
- `SHIM_SESSION_ID`: Session correlation ID (auto-generated if not set)
- `SHIM_BYPASS`: Emergency bypass mode (set to `1` to disable tracing)
- `SUBSTRATE_NO_SHIMS`: Set to `1` to disable automatic shim deployment
- `SUBSTRATE_WORLD`: Set to `enabled` to enable world isolation (Linux only)
- `SUBSTRATE_SESSION_ID`: World session identifier for isolation

For debugging and development:
- `SHIM_LOG_OPTS`: Logging options (`raw`, `resolve`, or `raw,resolve`)
  - `raw`: Disables credential redaction (security sensitive)
  - `resolve`: Includes binary resolution paths in logs
- `SHIM_CACHE_BUST`: Set to `1` to force cache invalidation for testing
- `SHIM_FSYNC`: Set to `1` to force filesystem sync for maximum log durability
- `RUST_LOG`: Standard Rust logging (set to `debug` for detailed output)
- `SUBSTRATE_PTY_DEBUG`: Set to `1` to enable PTY debugging output
- `WORLD_AGENT_SOCKET`: Override world-agent socket path (default: `/run/substrate.sock`)

## Testing Strategy

When modifying the codebase:
1. Unit tests are in `src/lib.rs` files as `#[cfg(test)]` modules
2. Integration tests are in `tests/` directories
3. Shell mode testing covers: interactive, wrap (-c), script (-f), pipe modes
4. PTY tests validate terminal emulation (Unix only)
5. World isolation tests require Linux and privileged access
6. Security tests verify credential redaction
7. Broker tests validate policy enforcement

### CI/CD Configuration
- **Platforms**: Linux (ubuntu-24.04) and macOS (macos-14)
- **Rust version**: 1.89.0
- **Windows support**: Deferred (removed from CI pipeline)
- **Known issues**: Lock tests may be timing-sensitive in CI environments

## World Isolation System

### Linux World Features
- **Network Isolation**: Per-world network namespaces with nftables rules
- **Resource Controls**: Cgroup v2 for memory/CPU limits
- **Filesystem Isolation**: OverlayFS or copy-diff for non-PTY commands
- **Session Management**: Automatic world lifecycle tied to shell sessions
- **Garbage Collection**: Automatic cleanup of orphaned network namespaces

### World-Agent API
- REST endpoints for command execution
- WebSocket for PTY/interactive sessions
- Filesystem diff tracking and reporting
- Automatic agent startup when `SUBSTRATE_WORLD=enabled`

## Shim Deployment Implementation

### Automatic Deployment
- Uses symlinks on Unix for efficiency
- Falls back to file copies on Windows
- Version checking via `env!("CARGO_PKG_VERSION")`
- Atomic deployment using tempfile crate
- Process locking with 5-second timeout

### CLI Commands
- `substrate --shim-status`: Check deployment status and version
- `substrate --shim-deploy`: Force redeployment of shims
- `substrate --shim-remove`: Remove all deployed shims
- `substrate --shim-skip`: Skip automatic deployment for this run

## Deployment Scripts

Located in `scripts/`:
- `stage_shims.sh`: Creates shimmed binaries in `~/.substrate/shims/`
- `create_bashenv.sh`: Sets up non-interactive shell environment
- `rollback.sh`: Emergency rollback to restore original environment

## Third-Party Dependencies

- Patched `reedline` from `https://github.com/atomize-hq/reedline` for REPL functionality
- Configured via `[patch.crates-io]` in root `Cargo.toml`

## Common Development Tasks

### Adding a New Built-in Shell Command
Built-in commands are handled in `crates/shell/src/lib.rs` in the `execute_builtin()` function. Add new cases to the match statement.

### Working with World Isolation
- Enable worlds: Set `SUBSTRATE_WORLD=enabled` before running substrate
- Debug world-agent: Check `/run/substrate.sock` socket presence
- View world status: Check `/tmp/substrate-worlds/` for active sessions

### Modifying Credential Redaction
Update the `redact_sensitive()` function in `crates/common/src/lib.rs` to add new patterns.

### Adding PTY Support for a New Command
Modify the `should_use_pty()` function in `crates/shell/src/lib.rs` to add detection for new interactive commands.

### Debugging Path Resolution
Set `SHIM_LOG_OPTS=resolve` to log path resolution details, or use `SHIM_CACHE_BUST=1` to force cache invalidation.

### Testing World Features (Linux)
```bash
# Enable world isolation
export SUBSTRATE_WORLD=enabled

# Run privileged tests
sudo -E RUST_LOG=info cargo test -p world -- --nocapture

# Test specific isolation features
cargo test -p world -- test_nftables_rules
cargo test -p world -- test_cgroup_creation
```

### Environment Setup for Comprehensive Debugging
```bash
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
export SUBSTRATE_PTY_DEBUG=1
export SHIM_CACHE_BUST=1  # Force cache invalidation
export SUBSTRATE_WORLD=enabled  # Enable world isolation (Linux)
```

## Troubleshooting

### Lock Test Failures
If lock tests fail with timing assertions, the issue is likely due to slow CI environments. The tests use `std::sync::Barrier` for synchronization and have lenient timing bounds (50-500ms).

### World-Agent Connection Issues
If world-agent fails to connect:
1. Check if socket exists: `ls -la /run/substrate.sock`
2. Verify world-agent is running: `ps aux | grep world-agent`
3. Check logs with `RUST_LOG=debug`
4. Ensure `SUBSTRATE_WORLD=enabled` is set

### Path Resolution Issues
If shims aren't resolving correctly:
1. Check `SHIM_ORIGINAL_PATH` is set correctly
2. Use `SHIM_LOG_OPTS=resolve` for detailed path resolution logs
3. Set `SHIM_CACHE_BUST=1` to force cache invalidation
