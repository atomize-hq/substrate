# Installation Guide

Complete setup guide for Substrate command tracing and AI agent platform.

## Prerequisites

- **Rust 1.74+** - Required for uuid v7 features
- **Git** - For cloning the repository
- **Platform**: Linux, macOS, or Windows (via WSL2)

## Installation Options

### Option 1: Install from crates.io (Recommended)

```bash
# Install substrate (includes automatic shim deployment)
cargo install substrate

# That's it! Shims are deployed automatically on first run
substrate --version

# Verify installation
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

Substrate automatically deploys command shims on first run. No manual setup required!

```bash
# Automatic deployment happens on first run
substrate

# Check deployment status
substrate --shim-status

# Manual management (optional)
substrate --shim-deploy   # Force redeployment
substrate --shim-remove   # Remove all shims
substrate --shim-skip     # Skip deployment for this run

# To use shims for command interception, add to PATH:
export PATH="$HOME/.substrate/shims:$PATH"
export SHIM_ORIGINAL_PATH="$PATH"  # Save original PATH
export SHIM_TRACE_LOG="$HOME/.substrate/trace.jsonl"

# Clear command cache
hash -r

# Verify installation
which git  # Should show: ~/.substrate/shims/git
git --version  # Should work normally with logging
```

**Note**: Shims are deployed as:
- **Symlinks on Unix/macOS** (efficient, instant updates)
- **File copies on Windows** (for compatibility)

### Non-Interactive Shell Support

For AI agent integration (Claude Code, etc.):

```bash
# Create BASH_ENV file
./scripts/create_bashenv.sh
export BASH_ENV="$HOME/.substrate_bashenv"

# Test integration
bash -c 'which git; git --version'
tail -1 ~/.substrate/trace.jsonl | jq '.'
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

## Verification

Test your installation:

```bash
# Interactive shell
substrate
substrate> git status
substrate> exit

# Command tracing
git --version
tail -1 ~/.substrate/trace.jsonl
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
# Remove all shims using CLI
substrate --shim-remove

# Or manual cleanup
export PATH="$SHIM_ORIGINAL_PATH"
rm -rf ~/.substrate/shims

# Disable automatic deployment
export SUBSTRATE_NO_SHIMS=1
substrate
```

For detailed troubleshooting, see [USAGE.md](USAGE.md#troubleshooting).

## Next Steps

- **Usage Patterns**: See [USAGE.md](USAGE.md) for shell usage and integration examples
- **Configuration**: See [CONFIGURATION.md](CONFIGURATION.md) for environment variables and settings
- **Development**: See [DEVELOPMENT.md](DEVELOPMENT.md) for building and testing
