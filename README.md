# Substrate Command Tracing Shim

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-passing-green.svg)](#testing)

> **Production-ready command execution shim for comprehensive observability and debugging of shell command chains. Designed for seamless integration with AI-assisted development environments like Claude Code.**

## Overview

Substrate provides transparent command interception through binary shimming, enabling developers and AI assistants to maintain complete visibility into command execution patterns without disrupting normal workflows.

### Key Features

- 🔍 **Structured Logging**: JSONL format with rich metadata
- 🔗 **Session Correlation**: Track command chains across nested executions  
- 🔒 **Security First**: Automatic credential redaction for API keys, tokens, passwords
- ⚡ **High Performance**: <5ms overhead with intelligent caching
- 🛡️ **Binary Integrity**: SHA-256 fingerprinting for security verification
- 🌐 **Cross-Platform**: Unix/Windows with platform-specific optimizations
- 🚨 **Emergency Bypass**: Built-in escape hatch for troubleshooting

## Quick Start

### 1. Installation

```bash
# Clone and build
git clone <repository-url>
cd substrate
cargo build --release --bin shim
```

### 2. Deploy Shims

```bash
# Stage shims for common development commands
./scripts/stage_shims.sh target/release/shim

# Verify deployment
ls -la ~/.cmdshim_rust/
```

### 3. Activate Tracing

```bash
# Set up environment
export ORIGINAL_PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH" 
export TRACE_LOG_FILE="$HOME/.trace_shell.jsonl"

# Clear command cache
hash -r
```

### 4. Use Commands Normally

```bash
git status              # ✅ Intercepted and logged
npm install            # ✅ Intercepted and logged  
curl -H "Authorization: Bearer secret" api.com  # ✅ Credentials redacted
```

## Integration Examples

### Claude Code Integration

The proven pattern for AI assistant integration:

```bash
# 1. Set up non-interactive shell environment
./scripts/create_bashenv.sh
export BASH_ENV="$HOME/.substrate_bashenv"

# 2. Use hash pinning for reliable resolution
hash -r
hash -p "$HOME/.cmdshim_rust/git" git
hash -p "$HOME/.cmdshim_rust/npm" npm

# 3. Verify integration works
which git              # Should show: /Users/you/.cmdshim_rust/git
git --version          # Should work normally with background logging
```

### Manual Hash Pinning (Advanced)

For maximum control in complex shell environments:

```bash
# Pin specific commands to shims
hash -r
hash -p "$HOME/.cmdshim_rust/git" git
hash -p "$HOME/.cmdshim_rust/docker" docker
hash -p "$HOME/.cmdshim_rust/kubectl" kubectl

# Verify pinning
type git docker kubectl
```

## Architecture

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Command       │    │   Shim Binary   │    │  Real Binary    │
│   Invocation    │───▶│   (Intercept)   │───▶│   (Execute)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                               ▼
                       ┌─────────────────┐
                       │  Structured     │
                       │  Logging        │
                       │  (JSONL)        │
                       └─────────────────┘
```

### Core Components

- **`crates/shim/src/context.rs`**: Environment detection and configuration
- **`crates/shim/src/resolver.rs`**: Binary path resolution with caching
- **`crates/shim/src/logger.rs`**: Structured logging with credential redaction
- **`crates/shim/src/exec.rs`**: Cross-platform command execution
- **`scripts/`**: Deployment and management utilities

## Configuration

All configuration is done through environment variables:

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `ORIGINAL_PATH` | Clean PATH for binary resolution | *none* | `/usr/bin:/bin` |
| `TRACE_LOG_FILE` | Log output destination | *none* | `~/.trace_shell.jsonl` |
| `SHIM_SESSION_ID` | Session correlation ID | auto-generated | uuid-v7-string |
| `SHIM_DEPTH` | Nesting level tracking | `0` | `0`, `1`, `2`... |
| `SHIM_BYPASS` | Emergency bypass mode | *none* | `1` |
| `SHIM_LOG_OPTS` | Logging options | *none* | `raw` |
| `SHIM_FSYNC` | Force disk sync | *none* | `1` |

## Log Format

Commands are logged in structured JSONL format:

```json
{
  "ts": "2024-01-15T10:30:45.123Z",
  "command": "git",
  "argv": ["git", "commit", "-m", "feat: add new feature"],
  "cwd": "/Users/dev/project",
  "exit_code": 0,
  "duration_ms": 234,
  "pid": 12345,
  "hostname": "dev-machine",
  "platform": "darwin-aarch64",
  "depth": 0,
  "session_id": "018d1234-5678-7abc-def0-123456789abc",
  "resolved_path": "/opt/homebrew/bin/git",
  "shim_fingerprint": "sha256:abc123def456...",
  "isatty_stdin": true,
  "isatty_stdout": true,
  "isatty_stderr": true
}
```

### Credential Redaction

Sensitive information is automatically redacted:

```bash
# Original command
curl -H "Authorization: Bearer secret123" -H "X-API-Key: key456" api.example.com

# Logged as
["curl", "-H", "Authorization: ***", "-H", "X-API-Key: ***", "api.example.com"]
```

Redaction patterns include:
- Authorization headers
- API keys and tokens
- Passwords and secrets
- Cookie values
- Custom sensitive patterns

## Performance

- **Startup Overhead**: 2-5ms typical (with binary resolution caching)
- **Memory Usage**: 1-2MB RSS per shim process
- **Cache Performance**: ~40% reduction in stat() calls after warmup
- **Log Throughput**: >10,000 commands/second sustained
- **Binary Size**: ~1.5MB release build

## Security

### Built-in Security Features

- **Credential Redaction**: Comprehensive pattern matching for sensitive data
- **Binary Integrity**: SHA-256 fingerprinting of executables
- **Secure Permissions**: Log files created with 0o600 (user-only access)
- **Path Sanitization**: Prevents PATH injection attacks
- **Emergency Bypass**: `SHIM_BYPASS=1` for critical situations

### Security Considerations

- Log files may contain sensitive information despite redaction efforts
- Shim binaries should be protected from unauthorized modification
- The `ORIGINAL_PATH` should not include untrusted directories
- Regular integrity verification is recommended for production use

## Testing

Comprehensive test suite covering unit, integration, and scenario testing:

```bash
# Run all tests
cargo test

# Run with output for debugging
cargo test -- --nocapture

# Run specific test suites
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests
```

### Test Coverage

- ✅ Unit tests for all core modules
- ✅ Integration tests for complete workflow
- ✅ Claude Code integration scenarios
- ✅ Cross-platform compatibility tests
- ✅ Security and redaction validation
- ✅ Performance and caching tests

## Troubleshooting

### Emergency Bypass

If shimming breaks your environment:

```bash
# Immediate bypass for current command
SHIM_BYPASS=1 git status

# Complete rollback to restore original environment
./scripts/rollback.sh
```

### Debug Mode

```bash
# Raw logging (disables credential redaction)
SHIM_LOG_OPTS=raw git clone https://github.com/user/repo.git

# Force fsync for debugging lost logs
SHIM_FSYNC=1 git push

# Verbose execution tracing
RUST_LOG=debug git status
```

### Common Issues

| Problem | Symptom | Solution |
|---------|---------|----------|
| Command not found | `bash: git: command not found` | Check `ORIGINAL_PATH` includes system directories |
| Infinite loops | Commands hang indefinitely | Ensure shim directory not in `ORIGINAL_PATH` |
| Permission denied | Cannot write to log file | Check log file permissions and directory access |
| Hash conflicts | Wrong binary executed | Run `hash -r` to clear shell command cache |
| Integration issues | AI assistant can't see commands | Verify `BASH_ENV` and hash pinning setup |

## Development

### Building from Source

```bash
# Development build
cargo build

# Release build (recommended for deployment)
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

### Project Structure

```
substrate/
├── Cargo.toml                 # Workspace configuration
├── crates/shim/               # Main shim implementation  
│   ├── Cargo.toml            # Crate configuration
│   ├── src/                  # Source code
│   └── tests/                # Integration tests
├── scripts/                  # Deployment and management scripts
├── docs/                     # Additional documentation
└── target/                   # Build artifacts
```

### Architecture Decisions

This implementation follows Rust best practices:

1. **Thin Binary, Thick Library**: Minimal `main.rs`, comprehensive `lib.rs`
2. **Result Everywhere**: No panics in library code, comprehensive error handling
3. **Structured Observability**: Rich logging with structured fields
4. **Security by Design**: Credential redaction and integrity verification
5. **Performance Focus**: Caching and minimal allocations

## Compatibility

- **Rust MSRV**: 1.70+ (enforced in CI)
- **Platforms**: macOS, Linux, Windows
- **Shells**: bash, zsh, fish, PowerShell
- **Commands**: Any executable (git, npm, python, docker, kubectl, etc.)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Fix all linter warnings: `cargo clippy`
- Maintain comprehensive test coverage
- Update documentation for public APIs

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: Report bugs and feature requests via GitHub Issues
- **Discussions**: Technical discussions via GitHub Discussions
- **Security**: Report security vulnerabilities privately via email

---

**Built with Rust 🦀 for reliable, high-performance command tracing**