# Installation Guide

Complete setup guide for Substrate command tracing and AI agent platform.

## Prerequisites

- **Rust 1.74+** - Required for uuid v7 features
- **Git** - For cloning the repository
- **Platform**: Linux, macOS, or Windows (via WSL2)

## Installation Options

### Option 1: Install from crates.io (Recommended)

```bash
# Install main substrate command
cargo install substrate

# Install supporting tools
cargo install substrate-shim
cargo install substrate-supervisor

# Verify installation
substrate --version
which substrate  # Should show ~/.cargo/bin/substrate
```

### Option 2: Build from Source

```bash
# Clone and build
git clone <repository-url>
cd substrate
cargo build --release

# Install to system PATH
sudo cp target/release/substrate* /usr/local/bin/

# Verify installation
substrate --version
which substrate  # Should show /usr/local/bin/substrate
```

## Component Installation

### Core Shell

The main `substrate` binary provides the interactive shell:

```bash
# Optional: Install to PATH
sudo cp target/release/substrate /usr/local/bin/
substrate --version
```

### Command Interception (Shimming)

Deploy binary shims for transparent command tracing:

```bash
# Deploy shims
./scripts/stage_shims.sh target/release/substrate-shim

# Configure environment
export SHIM_ORIGINAL_PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
export PATH="$HOME/.substrate/shims:$SHIM_ORIGINAL_PATH"
export SHIM_TRACE_LOG="$HOME/.trace_shell.jsonl"

# Clear command cache
hash -r

# Verify installation
which git  # Should show: ~/.substrate/shims/git
git --version  # Should work normally with logging
```

### Non-Interactive Shell Support

For AI agent integration (Claude Code, etc.):

```bash
# Create BASH_ENV file
./scripts/create_bashenv.sh
export BASH_ENV="$HOME/.substrate_bashenv"

# Test integration
bash -c 'which git; git --version'
tail -1 ~/.trace_shell.jsonl | jq '.'
```

## Platform-Specific Setup

### Linux

Full native support with all features available.

### macOS

Current: Full support except for some advanced isolation features.
Future: Lima VM integration for complete feature parity.

### Windows

Current: Basic support via ConPTY.
Future: WSL2 integration for full feature support.

## Process Supervisor

Optional process management utility:

```bash
# Direct usage
target/release/substrate-supervisor git status

# Or install to PATH
sudo cp target/release/substrate-supervisor /usr/local/bin/
```

## Verification

Test your installation:

```bash
# Interactive shell
substrate
substrate> git status
substrate> exit

# Command tracing
git --version
tail -1 ~/.trace_shell.jsonl

# Process supervision
substrate-supervisor echo "Hello, World!"
```

## Troubleshooting

### Common Issues

**Commands not intercepted**:
- Verify shim directory is first in PATH
- Run `hash -r` to clear shell cache
- Check `SHIM_ORIGINAL_PATH` excludes shim directory

**Permission errors**:
- Verify shim binaries are executable: `chmod +x ~/.substrate/shims/*`
- Check log file permissions and directory access

**Emergency recovery**:
```bash
# Complete rollback
./scripts/rollback.sh

# Or manual cleanup
export PATH="$SHIM_ORIGINAL_PATH"
rm -rf ~/.substrate/shims
```

For detailed troubleshooting, see [USAGE.md](USAGE.md#troubleshooting).

## Next Steps

- **Usage Patterns**: See [USAGE.md](USAGE.md) for shell usage and integration examples
- **Configuration**: See [CONFIGURATION.md](CONFIGURATION.md) for environment variables and settings
- **Development**: See [DEVELOPMENT.md](DEVELOPMENT.md) for building and testing