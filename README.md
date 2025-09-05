# Substrate

> Substrate is the secure execution layer that sits between AI agents and your computer - providing isolation, audit trails, and centralized policy control.

[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/badge/build-passing-green.svg)](#development)

## Vision

Substrate transforms development workflows by serving as the **secure middleware between AI agents and your computer**, enabling:

- **World-Based Isolation**: Run untrusted code in secure, policy-controlled environments
- **Centralized Security**: Single point of policy control across all AI agents and workflows
- **Comprehensive Audit Trails**: Complete logging for compliance, security auditing, and workflow optimization
- **Risk Mitigation**: Prevent AI agents from causing system damage through isolation and policy enforcement
- **Agent Workflow Intelligence**: AI-powered analysis to identify bottlenecks and optimize agent performance
- **Cross-Platform Foundation**: Consistent secure execution layer across Linux, macOS, and Windows

## Current State

Today, Substrate provides a production-ready **command tracing ecosystem**:

- **Custom Shell**: Interactive REPL with PTY support and comprehensive logging
- **Binary Shimming**: Transparent command interception for full observability
- **Session Correlation**: Track command chains across nested executions
- **Advanced Security**: Credential redaction and binary integrity verification

## Quick Start

```bash
# 1. Install from crates.io
cargo install substrate

# 2. Start the interactive shell
substrate

# That's it! Shims are deployed automatically on first run
```

### Shim Deployment

Substrate automatically deploys command shims on first run. The shims are:
- **Symlinks on Unix systems** (efficient, instant updates)
- **File copies on Windows systems** (for compatibility)
- **Version-tracked** to avoid unnecessary redeployment
- **Deployed to** `~/.substrate/shims/`

To manage shims manually:
```bash
# Check shim status
substrate --shim-status

# Force redeployment
substrate --shim-deploy

# Remove all shims
substrate --shim-remove

# Skip automatic deployment
export SUBSTRATE_NO_SHIMS=1
substrate
```

### Alternative: Build from Source

```bash
git clone <repository-url>
cd substrate
cargo build --release
sudo cp target/release/substrate* /usr/local/bin/
```

## What's Coming

- **Security Worlds**: Isolated execution environments with filesystem and network controls
- **Policy Engine**: YAML-based policies for command allowlists, resource limits, and approval workflows
- **Agent API**: REST endpoints for AI assistants to execute commands with budgets and scope controls
- **Graph Intelligence**: Kuzu database tracking command relationships and file dependencies
- **Cross-Platform**: macOS support via Lima VMs, Windows via WSL2

### Future Features Pipeline

Additional capabilities planned for later phases:

- Advanced seccomp policy tuning for granular syscall filtering
- Sophisticated domain/SNI-based network egress controls
- AI-powered analysis of agent logs to automatically identify workflow bottlenecks and failure patterns
- Centralized security policies and permissions for multi-agent environments
- Complex network filtering with deep packet inspection
- High-isolation microVMs for maximum security scenarios

## Architecture

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   AI Agents     │    │   Substrate     │    │   Secure Worlds │
│   (Claude, etc) │───▶│   Platform      │───▶│   (Isolation)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                               ▼
                       ┌─────────────────┐
                       │  Command Tracing│
                       │  & Analysis     │
                       └─────────────────┘
```

## Documentation

- **[Installation](docs/INSTALLATION.md)** - Complete setup guide and deployment
- **[Usage](docs/USAGE.md)** - Daily usage patterns and integration examples
- **[Configuration](docs/CONFIGURATION.md)** - Environment variables and advanced settings
- **[Vision](docs/VISION.md)** - Detailed roadmap and future capabilities
- **[Development](docs/DEVELOPMENT.md)** - Building, testing, and architecture
- **[Contributing](CONTRIBUTING.md)** - How to contribute to the project
- **[Security](SECURITY.md)** - Security model and vulnerability reporting
 - **[Replay](docs/REPLAY.md)** - Replaying traced commands (with Linux isolation option)
 - **[Graph](docs/GRAPH.md)** - Graph architecture and CLI (mock backend)
 - **[Privileged Tests](docs/HOWTO_PRIVILEGED_TESTS.md)** - Running isolation/netfilter tests on Linux

## Getting Help

- **Issues**: Report bugs via [GitHub Issues](https://github.com/your-org/substrate/issues)
- **Discussions**: Questions and ideas via [GitHub Discussions](https://github.com/your-org/substrate/discussions)
- **Security**: Report vulnerabilities privately (see [SECURITY.md](SECURITY.md))

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

---

**Secure AI-assisted development, powered by Rust**
