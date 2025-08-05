# Substrate Command Tracing System

[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/badge/build-passing-green.svg)](#testing)

> **Production-ready command execution tracing system with custom shell, binary shimming, and comprehensive observability for debugging command chains and AI-assisted development workflows.**

## Overview

Substrate is a complete command tracing ecosystem that provides transparent command interception, a custom shell with multiple execution modes, and structured logging for full visibility into command execution patterns.

### Key Features

- **Custom Shell**: Interactive REPL, wrap mode, script execution, and pipe processing
- **Structured Logging**: JSONL format with rich metadata and automatic log rotation
- **Session Correlation**: Track command chains across nested executions with UUIDv7
- **Advanced Security**: Comprehensive credential redaction including headers and tokens
- **High Performance**: <5ms shim overhead with intelligent caching
- **Binary Integrity**: SHA-256 fingerprinting for forensics and compliance
- **Cross-Platform**: Unix/macOS support, Windows prepared
- **Emergency Bypass**: `SHIM_BYPASS=1` escape hatch for troubleshooting
- **PTY Support**: Full terminal emulation for interactive sessions (Unix)
- **CI/CD Ready**: Strict mode, error handling, and automation features

## Quick Start

### 1. Installation

```bash
# Clone and build all components
git clone <repository-url>
cd substrate
cargo build --release

# Binaries built:
# - target/release/shim       # Command interception shim
# - target/release/substrate  # Custom shell with tracing
```

### 2. Using the Custom Shell

```bash
# Interactive REPL mode
./target/release/substrate

# Execute single command
substrate -c "git status && npm test"

# Run a script
substrate -f deploy.sh

# Pipe commands
echo "date" | substrate

# CI/CD mode with strict error handling
substrate --ci -c "make test"

# PTY mode for full terminal emulation (Unix)
substrate --pty
```

### 3. Deploy Binary Shims

```bash
# Stage shims for command interception
./scripts/stage_shims.sh target/release/shim

# Set up environment
export ORIGINAL_PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH" 
export TRACE_LOG_FILE="$HOME/.trace_shell.jsonl"

# Clear command cache
hash -r
```

### 4. Combined Usage

```bash
# Use substrate shell with shims active
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
substrate -c "git commit -m 'test' && npm run build"

# All commands are traced through both shell and shims
tail -f ~/.trace_shell.jsonl | jq '.event_type'
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
┌─────────────────┐           ┌─────────────────┐
│  Substrate      │           │   User/Script   │
│  Custom Shell   │◀──────────│   Invocation    │
└─────────────────┘           └─────────────────┘
         │
         ├─── Interactive REPL Mode
         ├─── Wrap Mode (-c "command")
         ├─── Script Mode (-f script.sh)
         └─── Pipe Mode (stdin)
         
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

- **`crates/shell/`**: Custom shell with multiple execution modes
  - Interactive REPL with command history
  - Built-in command support (cd, pwd, export, unset)
  - PTY terminal emulation for Unix systems
  - Signal handling and process management
- **`crates/shim/`**: Binary shimming for command interception
  - Path resolution with intelligent caching
  - Session and depth tracking
  - Comprehensive credential redaction
- **`crates/common/`**: Shared utilities and log schema
- **`scripts/`**: Deployment and management utilities

## Configuration

All configuration is done through environment variables:

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `ORIGINAL_PATH` | Clean PATH for binary resolution | *none* | `/usr/bin:/bin` |
| `TRACE_LOG_FILE` | Log output destination | `~/.trace_shell.jsonl` | `/tmp/trace.jsonl` |
| `SHIM_SESSION_ID` | Session correlation ID | auto-generated | uuid-v7-string |
| `SHIM_DEPTH` | Nesting level tracking | `0` | `0`, `1`, `2`... |
| `SHIM_BYPASS` | Emergency bypass mode | *none* | `1` |
| `SHIM_LOG_OPTS` | Logging options | *none* | `raw`, `resolve` |
| `SHIM_FSYNC` | Force disk sync | *none* | `1` |
| `TRACE_LOG_MAX_MB` | Log rotation size limit | `50` | `100` |
| `BASH_ENV` | Bash startup script | *none* | `~/.substrate_bashenv` |

## Log Format

Commands are logged in structured JSONL format:

```json
{
  "ts": "2024-01-15T10:30:45.123Z",
  "event_type": "command_complete",
  "command": "git commit -m 'feat: add new feature'",
  "cmd_id": "018d5678-9abc-7def-0123-456789abcdef",
  "session_id": "018d1234-5678-7abc-def0-123456789abc",
  "component": "shell",
  "mode": "wrap",
  "argv": ["git", "commit", "-m", "feat: add new feature"],
  "cwd": "/Users/dev/project",
  "exit_code": 0,
  "duration_ms": 234,
  "pid": 12345,
  "ppid": 12344,
  "hostname": "dev-machine",
  "shell": "/bin/bash",
  "depth": 0,
  "resolved_path": "/opt/homebrew/bin/git",
  "shim_fingerprint": "sha256:abc123def456...",
  "isatty_stdin": true,
  "isatty_stdout": true,
  "isatty_stderr": true,
  "pty": false,
  "build": "v0.1.0"
}
```

### Event Types

- `command_start`: Command execution begins
- `command_complete`: Command execution finishes with exit code
- `builtin_command`: Shell built-in command executed
- `pty_session_start`: PTY terminal session initiated
- `pty_session_end`: PTY terminal session terminated

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

- **Startup Overhead**: <5ms typical (first run ~19ms with cache warmup, subsequent <1ms)
- **Memory Usage**: ~2.5-2.7MB RSS per shim process
- **Cache Performance**: Reduces stat() calls after warmup (exact percentage varies by workload)
- **Binary Size**: 
  - Shim: ~684KB release build
  - Substrate shell: ~1.8MB release build

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

## Substrate Shell Usage

The substrate shell provides multiple execution modes for different use cases:

### Interactive Mode
```bash
substrate
# Substrate v0.1.0
# Session ID: 018d1234-5678-7abc-def0-123456789abc
# Logging to: /Users/you/.trace_shell.jsonl
substrate> git status
substrate> npm test
substrate> exit
```

### CI/CD Integration
```bash
# Strict error handling for CI
substrate --ci -c "npm test && npm build"

# Continue on error
substrate --ci --no-exit-on-error -f integration-tests.sh

# Specify shell explicitly
substrate --shell /bin/bash --ci -c "make test"
```

### Script Execution
```bash
# Execute script with state preservation
substrate -f deploy.sh

# Scripts maintain state (cd, export, etc.)
echo -e "cd /tmp\npwd\nexport FOO=bar\necho \$FOO" > test.sh
substrate -f test.sh
# Output: /tmp
#         bar
```

### Built-in Commands

The shell includes built-in commands that work across all platforms:
- `cd [dir]` - Change directory (supports `cd -` for previous directory)
- `pwd` - Print working directory
- `export VAR=value` - Set environment variables
- `unset VAR` - Remove environment variables
- `exit` / `quit` - Exit the shell

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
cargo test -p substrate-shell       # Shell-specific tests
cargo test -p substrate-shim        # Shim-specific tests
```

### Test Coverage

- Unit tests for all core modules
- Integration tests for complete workflow
- Shell mode testing (interactive, wrap, script, pipe)
- Signal handling and PTY tests
- Cross-platform compatibility tests
- Security and redaction validation
- Performance and caching tests
- CI/CD mode validation

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

For detailed architectural information and development guidelines, see [ARCHITECTURE.md](ARCHITECTURE.md).

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
├── crates/
│   ├── common/               # Shared utilities and log schema
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── shell/                # Custom shell implementation
│   │   ├── Cargo.toml
│   │   ├── build.rs          # Version tracking
│   │   ├── src/
│   │   │   ├── main.rs       # Shell entry point
│   │   │   ├── lib.rs        # Shell modes and execution
│   │   │   └── pty.rs        # PTY terminal emulation
│   │   └── tests/
│   ├── shim/                 # Binary shimming implementation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   └── supervisor/           # Process supervision (partially implemented)
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

- **Rust MSRV**: 1.74+ (enforced via rust-version)
- **Platforms**: macOS, Linux, Windows (prepared)
- **Shells**: bash, zsh, sh, fish, PowerShell, cmd.exe
- **Commands**: Any executable (git, npm, python, docker, kubectl, etc.)
- **Terminal**: PTY support for Unix, ConPTY prepared for Windows

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

This project is licensed under the MIT License.

## Support

- **Issues**: Report bugs and feature requests via GitHub Issues
- **Discussions**: Technical discussions via GitHub Discussions
- **Security**: Report security vulnerabilities privately via email

---

**Built with Rust 🦀 for reliable, high-performance command tracing**