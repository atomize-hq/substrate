# Development Guide

Building, testing, and contributing to Substrate.

## Prerequisites

- **Rust 1.74+** - Required for uuid v7 features
- **Git** - For version control and submodules
- **Platform tools** - Platform-specific development dependencies

## Project Structure

```
substrate/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── common/               # Shared utilities and log schema
│   ├── shell/                # Custom shell implementation
│   ├── shim/                 # Binary shimming implementation
├── third_party/reedline/     # Custom Reedline fork
├── scripts/                  # Deployment and management scripts
└── docs/                     # Documentation
```

## Building

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Specific Components

```bash
# Build individual components
cargo build --bin substrate
cargo build --bin substrate-shim

# Build with features
cargo build --release --features production
```

## Testing

### Complete Test Suite

```bash
cargo test
```

### Specific Test Types

```bash
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests
cargo test -p substrate             # Shell-specific tests
cargo test -p substrate-shim        # Shim-specific tests
cargo test --test shim_deployment   # Shim deployment tests
```

### Test with Output

```bash
cargo test -- --nocapture
```

### Performance Benchmarks

```bash
cargo bench
```

## Code Quality

### Formatting

```bash
cargo fmt
cargo fmt -- --check  # Check without changes
```

### Linting

```bash
cargo clippy
cargo clippy -- -D warnings  # Fail on warnings
cargo clippy --fix           # Auto-fix issues
```

### Feature flags and heavy backends (Kuzu)

- Avoid running `--all-features` at the workspace level. It enables the optional Kuzu graph backend, which triggers a heavy native build (or requires a system lib), and can stall local builds.
- Recommended workspace commands (no Kuzu):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace -- --nocapture
```

- Only enable Kuzu when working on the graph crate:

```bash
# Dynamically link to a system Kuzu (fast if installed)
cargo build -p substrate-graph --features kuzu-dylib

# Statically build Kuzu (slow; requires cmake and a long native build)
cargo build -p substrate-graph --features kuzu-static
```

Use `cargo tree -p substrate-graph -e features` to inspect which features are active.

### Documentation

```bash
cargo doc --open
cargo doc --no-deps  # Skip dependencies
```

## Architecture

### Crate Dependencies

```
substrate-common (base utilities)
    ↑
    ├── substrate-shim (command interception)
    └── substrate (custom shell)
```

### Design Principles

1. **Thin Binary, Thick Library**: Minimal main.rs, comprehensive lib.rs
2. **Result Everywhere**: No panics in library code
3. **Structured Observability**: Rich logging with structured fields
4. **Security by Design**: Credential redaction and integrity verification
5. **Performance Focus**: Caching and minimal allocations

### Module Organization

**substrate-shim**:

- `context.rs`: Environment detection
- `resolver.rs`: Binary path resolution with caching
- `logger.rs`: Structured logging
- `exec.rs`: Process execution

**substrate (shell)**:

- `lib.rs`: Shell modes and built-in commands
- `pty_exec.rs`: PTY management and terminal emulation

**substrate-common**:

- Shared utilities (path handling, redaction)
- Cross-component constants and types

## Reedline Integration

Substrate uses a custom fork of Reedline:

### Working with the Fork

```bash
# Update reedline fork
cd third_party/reedline
git pull upstream main
cd ../..
cargo build --release
```

### Patch Configuration

```toml
[patch.crates-io]
reedline = { path = "third_party/reedline" }
```

## Adding Features

### New Environment Variables

- Prefix with `SHIM_` for shim-related features
- Prefix with `SUBSTRATE_` for shell-related features
- Document in [CONFIGURATION.md](CONFIGURATION.md)

### New Built-in Commands

Add to `execute_builtin()` function in `crates/shell/src/lib.rs`:

```rust
match command {
    "cd" => handle_cd(args, config),
    "pwd" => handle_pwd(),
    "your_command" => handle_your_command(args),
    // ...
}
```

### New Logging Events

1. Add event type constant to `substrate-common/src/lib.rs`
2. Use consistent field names from `log_schema` module
3. Maintain backward compatibility

## Testing Strategy

### Unit Tests

- Internal logic, no external dependencies
- Located in `#[cfg(test)]` modules within lib.rs files

### Integration Tests

- Full command execution paths
- Located in `tests/` directories
- Test real binary execution and logging

### Manual Testing Checklist

- Basic command execution
- Nested command execution (depth tracking)
- Signal handling (Ctrl+C during long-running command)
- PTY mode with interactive programs
- Credential redaction patterns
- Emergency bypass functionality

## Debugging

### Environment Setup

```bash
export RUST_LOG=debug
export SHIM_LOG_OPTS=raw,resolve
export SUBSTRATE_PTY_DEBUG=1
```

### Common Debug Tasks

```bash
# Test shim deployment
substrate --shim-status              # Check deployment status
substrate --shim-deploy              # Force redeployment
substrate --shim-remove              # Clean up shims
SUBSTRATE_NO_SHIMS=1 substrate      # Skip auto-deployment

# Test shim execution
SHIM_TRACE_LOG=/tmp/debug.jsonl git --version
cat /tmp/debug.jsonl

# Test PTY functionality
SUBSTRATE_PTY_DEBUG=1 substrate -c "vim test.txt"

# Test cache behavior
SHIM_CACHE_BUST=1 time git --version  # Cold cache
time git --version                    # Warm cache
```

## Release Process

### Version Updates

1. Update version in relevant Cargo.toml files
2. Update CHANGELOG.md
3. Tag release: `git tag v0.x.x`
4. Build release artifacts: `cargo build --release`

### Binary Distribution

```bash
# Create release binaries
cargo build --release
tar -czf substrate-v0.x.x-$(uname -m).tar.gz -C target/release substrate substrate-shim
```

For contributing guidelines, see [CONTRIBUTING.md](../CONTRIBUTING.md).
For current architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).
