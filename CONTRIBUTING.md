# Contributing to Substrate

Thank you for your interest in contributing to Substrate! This guide outlines how to contribute effectively.

## Quick Start

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Create** a feature branch: `git checkout -b feature-name`
4. **Make** your changes with tests
5. **Test** thoroughly: `cargo test && cargo clippy`
6. **Submit** a pull request

## Development Setup

```bash
# Clone and setup
git clone https://github.com/your-username/substrate.git
cd substrate
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings
```

## Code Standards

### Rust Guidelines

- **MSRV**: Rust 1.74+ required
- **Edition**: Use Rust 2021 edition
- **Formatting**: Use `cargo fmt` with default settings
- **Linting**: Address all `cargo clippy` warnings
- **Documentation**: Document public APIs with examples

### Code Style

- **Error Handling**: Use `Result<T, anyhow::Error>` in library code
- **No Panics**: Library code must not panic, return errors instead  
- **Structured Logging**: Use consistent field names from `log_schema`
- **Security**: Never log or expose sensitive information
- **Performance**: Consider startup time and memory usage

## Testing Requirements

### Required Tests

- **Unit Tests**: All new functions and modules
- **Integration Tests**: End-to-end workflow testing
- **Documentation Tests**: Ensure examples compile and work

### Test Categories

```bash
cargo test --lib        # Unit tests
cargo test --test integration  # Integration tests
cargo test --doc        # Documentation examples
```

### Platform Testing

Test on multiple platforms when possible:
- Linux (preferred for full feature testing)
- macOS (PTY and shell functionality)
- Windows (basic compatibility)

## Pull Request Guidelines

### Before Submitting

- [ ] All tests pass: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] No lint warnings: `cargo clippy -- -D warnings`
- [ ] Documentation updated for public APIs
- [ ] Integration tests added for new features
- [ ] Performance impact assessed

### PR Description

Include in your pull request:
- **Summary**: What does this change do?
- **Motivation**: Why is this change needed?
- **Testing**: How was this tested?
- **Breaking Changes**: Any compatibility impacts?

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: Maintainer review required
3. **Testing**: Verify on multiple platforms when possible
4. **Documentation**: Ensure docs are updated
5. **Merge**: Squash merge preferred for clean history

## Areas for Contribution

### Current Priorities

- **Performance Optimization**: Reduce startup overhead
- **Platform Support**: Windows ConPTY improvements
- **Documentation**: Usage examples and guides
- **Testing**: Increase test coverage

### Future Features

- **Security Worlds**: Isolation and policy engine (Phase 4)
- **Agent API**: RESTful interface for AI assistants
- **Graph Analysis**: Command relationship tracking
- **Cross-Platform**: Lima and WSL2 integration

## Bug Reports

### Good Bug Reports Include

- **Environment**: OS, Rust version, Substrate version
- **Reproduction**: Minimal steps to reproduce
- **Expected vs Actual**: What should happen vs what happens
- **Logs**: Relevant log output (with credentials redacted)

### Bug Report Template

```markdown
**Environment**
- OS: macOS 14.0
- Rust: 1.74.0
- Substrate: v0.1.0

**Steps to Reproduce**
1. Run `substrate -c "git status"`
2. Observe output

**Expected**: Git status output
**Actual**: Command hangs

**Logs**
```
[paste relevant logs]
```
```

## Feature Requests

### Good Feature Requests Include

- **Use Case**: What problem does this solve?
- **Proposal**: How should it work?
- **Alternatives**: Other approaches considered?
- **Implementation**: Any implementation ideas?

## Code Review Focus

### Security Considerations

- Path traversal prevention
- Command injection prevention
- Credential exposure prevention
- Privilege escalation prevention

### Performance Considerations

- Startup time impact
- Memory usage patterns
- Filesystem operation efficiency
- Cache effectiveness

### Compatibility Considerations

- Cross-platform behavior
- Shell compatibility
- Backward compatibility
- Integration reliability

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and design discussions
- **Code Review**: Ask questions in pull request comments

## Recognition

Contributors are recognized in:
- Git commit history
- CHANGELOG.md acknowledgments
- GitHub contributor statistics

Thank you for helping make Substrate better!