# Substrate Command Tracing System

[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/badge/build-passing-green.svg)](#testing)

> **Production-ready command execution tracing system with custom shell, binary shimming, and comprehensive observability for debugging command chains and AI-assisted development workflows.**

## Overview

Substrate is a complete command tracing ecosystem that provides transparent command interception, a custom shell with multiple execution modes, and structured logging for full visibility into command execution patterns.

### Key Features

- **Custom Shell**: Interactive REPL with PTY support, wrap mode, script execution, and pipe processing
- **Structured Logging**: JSONL format with rich metadata and session correlation
- **Session Correlation**: Track command chains across nested executions with UUIDv7
- **Advanced Security**: Comprehensive credential redaction including headers and tokens
- **High Performance**: Optimized shim overhead with intelligent caching (benchmarks available via `cargo bench`)
- **Binary Integrity**: SHA-256 fingerprinting for forensics and compliance
- **Cross-Platform**: Unix/macOS support with Windows ConPTY integration
- **Emergency Bypass**: `SHIM_BYPASS=1` escape hatch for troubleshooting
- **PTY Support**: Full terminal emulation for interactive sessions with live resizing
- **Reedline Integration**: Custom fork for enhanced REPL functionality

## Quick Start

### 1. Installation

```bash
# Clone and build all components
git clone <repository-url>
cd substrate
cargo build --release

# Binaries built:
# - target/release/substrate-shim           # Command interception shim
# - target/release/substrate                # Custom shell with tracing
# - target/release/substrate-supervisor     # Process supervisor (optional)
```

### 2. Using the Custom Shell

```bash
# Interactive REPL mode with PTY support
./target/release/substrate

# Execute single command
substrate -c "git status && npm test"

# Run a script with state preservation
substrate -f deploy.sh

# Pipe commands
echo "date" | substrate

# CI/CD mode with strict error handling
substrate --ci -c "make test"

# Force PTY for specific command
substrate -c ":pty vim file.txt"
```

### 3. Deploy Binary Shims

```bash
# Stage shims for command interception
./scripts/stage_shims.sh target/release/substrate-shim

# Set up environment (example paths - adjust for your system)
export SHIM_ORIGINAL_PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export PATH="$HOME/.cmdshim_rust:$SHIM_ORIGINAL_PATH"
export SHIM_TRACE_LOG="$HOME/.trace_shell.jsonl"

# Clear command cache
hash -r
```

### 4. Verify Installation

```bash
# Check shim resolution
which git              # Should show: ~/.cmdshim_rust/git
type git               # Should show shim first

# Test command execution and logging
git --version
tail -1 ~/.trace_shell.jsonl | jq '.'
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
which git              # Should show: ~/.cmdshim_rust/git
git --version          # Should work normally with background logging
```

### Supervisor Usage

```bash
# Using the process supervisor for managed execution
target/release/substrate-supervisor git status

# Supervisor automatically handles:
# - Environment setup
# - Session correlation
# - Clean PATH management
# - BASH_ENV configuration
```

## Architecture

```text
┌─────────────────┐           ┌─────────────────┐
│  Substrate      │           │   User/Script   │
│  Custom Shell   │◀──────────│   Invocation    │
└─────────────────┘           └─────────────────┘
         │
         ├─── Interactive REPL Mode (with Reedline)
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

- **`crates/shell/`**: Custom shell implementation (`substrate` binary)
  - Interactive REPL with Reedline integration
  - Built-in command support (cd, pwd, export, unset, exit, quit)
  - PTY terminal emulation for interactive sessions
  - Signal handling and process management
- **`crates/shim/`**: Binary shimming for command interception (`substrate-shim` binary)
  - Path resolution with intelligent caching
  - Session and depth tracking
  - Comprehensive credential redaction
  - SHA-256 binary fingerprinting
- **`crates/common/`**: Shared utilities and log schema
  - Path deduplication utilities
  - Credential redaction logic
  - Standardized logging constants
- **`crates/supervisor/`**: Process supervision and environment management
  - Process lifecycle management
  - Environment setup and PATH management
  - Session seeding and correlation
- **`scripts/`**: Deployment and management utilities
- **`third_party/reedline/`**: Custom fork for REPL functionality

## Reedline Integration

Substrate uses a minimal fork of the Reedline library for enhanced shell functionality:

**Location**: `third_party/reedline/`

**Integration**:

```toml
[patch.crates-io]
reedline = { path = "third_party/reedline" }
```

**Features**:

- Interactive command line editing
- Command history and persistence
- Tab completion support
- Syntax highlighting
- Emacs/Vi editing modes
- Custom prompt support

**Why Forked**: Contains minimal substrate-specific modifications for integration and behavior customization.

## Configuration

All configuration is done through environment variables:

| Variable                      | Purpose                             | Default                | Example                |
| ----------------------------- | ----------------------------------- | ---------------------- | ---------------------- |
| `SHIM_ORIGINAL_PATH`          | Clean PATH for binary resolution    | _none_                 | `/usr/bin:/bin`        |
| `SHIM_TRACE_LOG`              | Log output destination              | `~/.trace_shell.jsonl` | `/tmp/trace.jsonl`     |
| `SHIM_SESSION_ID`             | Session correlation ID              | auto-generated         | uuid-v7-string         |
| `SHIM_DEPTH`                  | Nesting level tracking              | `0`                    | `0`, `1`, `2`...       |
| `SHIM_BYPASS`                 | Emergency bypass mode (no tracing)  | _none_                 | `1`                    |
| `SHIM_CALLER`                 | First command in chain              | _none_                 | `npm`                  |
| `SHIM_CALL_STACK`             | Command chain (max 8, deduped)      | _none_                 | `npm,node`             |
| `SHIM_PARENT_CMD_ID`          | Links to shell command              | _none_                 | uuid-v7-string         |
| `SHIM_LOG_OPTS`               | Logging options                     | _none_                 | `raw`, `resolve`       |
| `SHIM_CACHE_BUST`             | Force cache invalidation            | _none_                 | `1`                    |
| `BASH_ENV`                    | Bash startup script                 | _none_                 | `~/.substrate_bashenv` |
| `SUBSTRATE_FORCE_PTY`         | Force PTY for all commands          | _none_                 | `1`                    |
| `SUBSTRATE_DISABLE_PTY`       | Disable PTY globally (escape hatch) | _none_                 | `1`                    |
| `SUBSTRATE_PTY_DEBUG`         | Enable PTY debug logging            | _none_                 | `1`                    |
| `SUBSTRATE_PTY_PIPELINE_LAST` | PTY for last pipeline segment       | _none_                 | `1`                    |

## PTY Support

Substrate includes comprehensive pseudo-terminal (PTY) support for running interactive commands and TUI applications with proper terminal emulation.

### Automatic PTY Detection

Substrate automatically uses PTY for commands that need terminal control:

**Interactive shells:**

- `bash`, `zsh`, `sh`, `fish` (without `-c` flag)
- `bash -i` (forced interactive mode)

**TUI applications:**

- Editors: `vim`, `vi`, `nvim`, `nano`, `emacs`
- Pagers: `less`, `more`, `most`
- System monitors: `top`, `htop`, `btop`, `glances`
- AI tools: `claude`, `chatgpt`
- Multiplexers: `tmux`, `screen`, `zellij`
- File managers: `ranger`, `yazi`
- Other TUIs: `fzf`, `lazygit`, `gitui`, `tig`, `k9s`, `nmtui`

**Interactive REPLs:**

- `python`, `python3`, `ipython`, `bpython` (without script)
- `node` (without script)
- `irb`, `pry` (Ruby)

**Container commands:**

- `docker run -it ubuntu`
- `docker exec -it container bash`
- `kubectl exec -it pod -- bash`

**SSH sessions:**

- `ssh host` (interactive login)
- `ssh -t host` (force PTY)

**Git interactive commands:**

- `git add -p`, `git add -i` (interactive staging)
- `git rebase -i` (interactive rebase)
- `git commit` (opens editor)

### Manual PTY Control

```bash
# Force PTY for any command
substrate -c ":pty ls -la"

# Force PTY globally
export SUBSTRATE_FORCE_PTY=1

# Disable PTY globally (escape hatch)
export SUBSTRATE_DISABLE_PTY=1

# Enable PTY debug logging
export SUBSTRATE_PTY_DEBUG=1
```

### PTY Features

- **Full terminal emulation** with proper control sequences
- **Live terminal resizing** via SIGWINCH (Unix/macOS)
- **Arrow keys and function keys** work correctly
- **Colors and formatting** preserved
- **Interactive password prompts** supported
- **Cross-platform**: Unix/Linux/macOS with Windows ConPTY support

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
  "depth": 0,
  "resolved_path": "/opt/homebrew/bin/git",
  "shim_fingerprint": "sha256:abc123def456...",
  "caller": "npm",
  "call_stack": "npm,node",
  "parent_cmd_id": "018d5678-9abc-7def-0123-456789abcdef",
  "bypass": false,
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

Redaction patterns include authorization headers, API keys, passwords, tokens, and cookie values.

## Performance

**Performance Characteristics**:

- **Optimized Design**: Intelligent caching reduces filesystem operations after warmup
- **Minimal Memory Footprint**: Designed for low resource usage
- **Compact Binaries**: Rust's efficient compilation produces small executables
- **Benchmark Suite**: Run `cargo bench` for detailed performance analysis
- **Cache Effectiveness**: Reduces filesystem stat() calls after warmup

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
- The `SHIM_ORIGINAL_PATH` should not include untrusted directories
- Regular integrity verification is recommended for production use

## Shell Usage

### Interactive Mode

```bash
substrate
# Substrate Shell v0.1.0
# Session ID: 018d1234-5678-7abc-def0-123456789abc
# Logging to: ~/.trace_shell.jsonl
substrate> git status
substrate> npm test
substrate> exit
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
cargo test -p substrate             # Shell-specific tests
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

# PTY debug logging
SUBSTRATE_PTY_DEBUG=1 substrate -c "vim"

# Verbose execution tracing
RUST_LOG=debug substrate -c "git status"
```

### Common Issues

| Problem           | Symptom                         | Solution                                               |
| ----------------- | ------------------------------- | ------------------------------------------------------ |
| Command not found | `bash: git: command not found`  | Check `SHIM_ORIGINAL_PATH` includes system directories |
| Infinite loops    | Commands hang indefinitely      | Ensure shim directory not in `SHIM_ORIGINAL_PATH`      |
| Permission denied | Cannot write to log file        | Check log file permissions and directory access        |
| Hash conflicts    | Wrong binary executed           | Run `hash -r` to clear shell command cache             |
| PTY issues        | Interactive commands don't work | Check `SUBSTRATE_DISABLE_PTY` or enable debug mode     |

## Development

For detailed architectural information and development guidelines, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

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
│   │   ├── build.rs          # Build-time configuration
│   │   ├── src/
│   │   │   ├── main.rs       # Shell entry point
│   │   │   ├── lib.rs        # Shell modes and execution
│   │   │   └── pty_exec.rs   # PTY terminal emulation
│   │   └── tests/
│   ├── shim/                 # Binary shimming implementation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs       # Shim entry point
│   │   │   ├── lib.rs        # Public API and orchestration
│   │   │   ├── context.rs    # Environment detection
│   │   │   ├── resolver.rs   # Path resolution with caching
│   │   │   ├── exec.rs       # Process execution
│   │   │   └── logger.rs     # Structured logging
│   │   └── tests/
│   └── supervisor/           # Process supervision
│       ├── Cargo.toml
│       └── src/
├── scripts/                  # Deployment and management scripts
├── docs/                     # Additional documentation
├── third_party/              # Third-party dependencies
│   └── reedline/            # Custom Reedline fork
└── target/                   # Build artifacts
```

### Architecture Decisions

This implementation follows Rust best practices:

1. **Thin Binary, Thick Library**: Minimal `main.rs`, comprehensive `lib.rs`
2. **Result Everywhere**: No panics in library code, comprehensive error handling
3. **Structured Observability**: Rich logging with structured fields
4. **Security by Design**: Credential redaction and integrity verification
5. **Performance Focus**: Caching and minimal allocations
6. **Cross-Platform Support**: Platform-specific optimizations where needed

## Compatibility

- **Rust MSRV**: 1.74+
- **Platforms**: macOS, Linux, Windows (ConPTY support)
- **Shells**: bash, zsh, sh, fish, PowerShell, cmd.exe
- **Commands**: Any executable (git, npm, python, docker, kubectl, etc.)
- **Terminal**: PTY support for Unix/Linux/macOS, ConPTY for Windows

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
- Consider impact on Reedline fork if modifying shell functionality

## License

This project is licensed under the MIT License.

## Support

- **Issues**: Report bugs and feature requests via GitHub Issues
- **Discussions**: Technical discussions via GitHub Discussions
- **Documentation**: See [ARCHITECTURE.md](docs/ARCHITECTURE.md) and [OPS.md](docs/OPS.md)
- **Security**: Report security vulnerabilities privately via email

---

**Built with Rust 🦀 for reliable, high-performance command tracing**
