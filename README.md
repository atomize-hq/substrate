# Substrate

> Substrate is the secure execution layer that sits between AI agents and your computer - providing isolation, audit trails, and centralized policy control.

[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/badge/build-passing-green.svg)](docs/DEVELOPMENT.md)

## Vision

Substrate transforms development workflows by serving as the **secure middleware between AI agents and your computer**, enabling:

- **World-Based Isolation**: Run untrusted code in secure, policy-controlled environments
- **Centralized Security**: Single point of policy control across all AI agents and workflows
- **Comprehensive Audit Trails**: Complete logging for compliance, security auditing, and workflow optimization
- **Risk Mitigation**: Prevent AI agents from causing system damage through isolation and policy enforcement
- **Agent Workflow Intelligence**: AI-powered analysis to identify bottlenecks and optimize agent performance
- **Cross-Platform Foundation**: Consistent secure execution layer across Linux, macOS, and Windows

## Current State

Today, Substrate delivers a cross-platform **command tracing stack**:

- **Custom Shell**: Interactive REPL with PTY support and comprehensive logging
- **Binary Shimming**: Transparent command interception for full observability
- **Session Correlation**: Track command chains across nested executions
- **Advanced Security**: Credential redaction and binary integrity verification

## Quick Start

Substrate now builds its PATH + manager environment on the fly, so the host
shell remains untouched after install.

```bash
# 1. Run the installer (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash

# 2. Launch Substrate (no sourcing required)
substrate --shim-deploy          # ensures the shim dir exists
substrate shim doctor            # inspect manifest + PATH diagnostics
substrate                        # start the interactive shell

# Optional: compare PATHs
printf "host PATH  -> %s\n" "$PATH"
substrate -c 'printf "substrate PATH -> %s\n" "$PATH"'
```

> Need to test against a temp HOME? Override `HOME`, `USERPROFILE`, and
> `SUBSTRATE_MANAGER_MANIFEST` exactly like the integration tests do and the CLI
> will confine itself to the fixture directories.

### Windows Quick Start

On Windows 11, Substrate shells run inside the `substrate-wsl` distribution to match the Linux isolation stack. The WSL backend is **functional but experimental**, so expect occasional rough edges. Use the installer (PowerShell 7) to provision or refresh the distro:

```powershell
pwsh -File scripts\windows\install-substrate.ps1 -DistroName substrate-wsl

# Regenerate / warm the distro manually if you are iterating locally
pwsh -File scripts\windows\wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
```

After the warm step, run `substrate` from the workspace (or your PATH) and it will automatically connect to the world agent inside WSL. Validate the bridge, PTY, and replay flows with the smoke script:

```powershell
pwsh -File scripts\windows\wsl-smoke.ps1
```

Detailed setup guidance, doctor output, and troubleshooting live in [`docs/cross-platform/wsl_world_setup.md`](docs/cross-platform/wsl_world_setup.md).

> Windows installers follow the same pass-through model—host shells keep their
> PATH, and `substrate shim doctor` provides the canonical view of manager
> status plus repair hints.

### Pass-Through Shims & Manager Init

- The installer no longer edits `.bashrc`, `.zshrc`, or PowerShell profiles.
- Every `substrate` invocation prepends `~/.substrate/shims` to `PATH` in-memory,
  writes `SUBSTRATE_MANAGER_INIT`/`SUBSTRATE_MANAGER_ENV`, and sources the
  generated manager snippets before running your command.
- Legacy `.substrate_bashenv` is still honored so existing automation keeps
  working, but new installs do not touch it unless you explicitly run
  `substrate shim repair`.

Shim helpers remain scriptable:

```bash
substrate --shim-status   # report version + location
substrate --shim-deploy   # redeploy in-place
substrate --shim-remove   # delete ~/.substrate/shims
SUBSTRATE_NO_SHIMS=1 substrate   # skip deployment for this invocation
```

### Shim Doctor & Repair

`substrate shim doctor` surfaces manifest paths, PATH diagnostics, detected
managers, and the latest repair hints:

```bash
substrate shim doctor
substrate shim doctor --json | jq '.path'
```

When a manager needs to export a snippet into your host shell, apply it with:

```bash
substrate shim repair --manager nvm --yes
cat ~/.substrate_bashenv
```

The repair command appends (or replaces) a delimited block for the selected
manager and writes `~/.substrate_bashenv.bak` before mutating the file.

### Manager Manifest & Overlays

- Shipping manifest: `config/manager_hooks.yaml`
- User overlay: `~/.substrate/manager_hooks.local.yaml`
- Tests/automation can point at a custom manifest by exporting
  `SUBSTRATE_MANAGER_MANIFEST=/path/to/manager_hooks.yaml`.

Manifest entries control detection, init snippets, repair hints, and future
guest install recipes. The same data powers manager detection, shim doctor,
and shim hint logging.

### World Enable & Dependency Sync (Preview)

The B-phase roadmap delivers two CLI additions so operators can manage isolated
worlds without reinstalling:

```bash
substrate world enable          # re-run provisioning if --no-world was used
substrate world deps status     # inspect host vs world toolchains
substrate world deps install    # install missing dependencies
substrate world deps sync       # one-shot plan + install
```

Linux uses namespaces/cgroups, macOS rides Lima, and Windows leverages WSL. Use
`substrate world doctor` (already available) to confirm host readiness.

### Alternative: Build from Source (development)

```bash
git clone https://github.com/atomize-hq/substrate.git
cd substrate
cargo build --release
```

> **Note:** Building only the `substrate` binary does not provision the world
> agent, installer assets, or shims. To mirror a full install, run the platform
> installer (Linux/macOS script or Windows PowerShell) after rebuilding, or copy
> the auxiliary binaries and scripts into place manually.

## Ongoing Work

- **Security Worlds Enhancements**: Default-on worlds ship for Linux, macOS (via Lima), and Windows (via WSL **functional but experimental**); active work includes tightening isolation policies and expanding telemetry around world sessions.
- **Policy Engine Evolution**: The broker today enforces allow/deny/isolate rules in observe mode by default; future milestones include opt-in enforcement flows and richer restriction types.
- **Agent API Maturation**: `world-agent` already exposes REST/WebSocket endpoints for shell integration—planned work adds per-agent budgets and scope negotiation for third-party assistants.
- **Graph Intelligence Roadmap**: The `substrate-graph` crate ships with a mock backend; Kuzu-powered ingestion and query tooling remain active development areas.

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
- **[Windows World Setup](docs/cross-platform/wsl_world_setup.md)** - WSL2 provisioning, warm/doctor/smoke automation, and troubleshooting
- **[Transport Parity Design](docs/cross-platform/transport_parity_design.md)** - The cross-platform transport architecture that underpins Windows parity

### World Doctor & Host Readiness

Use the built-in doctor to validate host readiness for isolation and macOS Lima provisioning:

```bash
# Human-readable report
substrate world doctor

# JSON for CI
substrate world doctor --json | jq .
```

Linux report highlights:

- overlayfs kernel support (with best-effort `modprobe overlay` when privileged)
- FUSE availability: `/dev/fuse` and `fuse-overlayfs` in `PATH`
- cgroup v2 presence, nft availability, and `kernel.dmesg_restrict`
- Per-user runtime roots for overlay and copy-diff

macOS report highlights:

- Lima tooling (`limactl`), Virtualization.framework (`sysctl kern.hv_support`), and optional `vsock-proxy`
- Lima VM `substrate` status plus SSH connectivity
- Guest `substrate-world-agent` service, socket, and `/v1/capabilities` response
- nftables availability and root filesystem usage inside the guest
- JSON output nests these fields under `lima.{installed, vm_status, service_active, agent_socket, agent_caps_ok, vsock_proxy, ssh, nft, disk_usage}`

On Windows, run the PowerShell doctor to verify WSL, forwarder, and agent health:

```powershell
pwsh -File scripts\windows\wsl-doctor.ps1 -DistroName substrate-wsl -Verbose
```

Linux packaging note: we recommend installing `fuse-overlayfs` so Substrate can fall back to user-space overlay where kernel overlay mounts are unavailable. For example:

- Debian/Ubuntu: `apt-get install -y fuse-overlayfs fuse3`
- Fedora/RHEL/CentOS: `dnf install -y fuse-overlayfs`
- Arch/Manjaro: `pacman -S --needed fuse-overlayfs`

## Getting Help

- **Issues**: Report bugs via [GitHub Issues](https://github.com/your-org/substrate/issues)
- **Discussions**: Questions and ideas via [GitHub Discussions](https://github.com/your-org/substrate/discussions)
- **Security**: Report vulnerabilities privately (see [SECURITY.md](SECURITY.md))

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

---

Secure AI-assisted development, powered by Rust.
