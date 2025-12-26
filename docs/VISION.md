# Vision & Roadmap

Substrate's evolution from command tracing to secure AI agent collaboration platform.

This document is aspirational. For the current, enforced behavior of world isolation, see
`docs/WORLD.md`. For the current policy schema (including `world_fs`), see `docs/CONFIGURATION.md`.

## Long-Term Vision

Substrate is becoming the **foundational platform for secure AI-assisted development**, enabling:

- **Secure Agent Collaboration**: AI assistants execute commands safely with policy enforcement and resource budgets
- **World-Based Isolation**: Untrusted code runs in isolated environments with comprehensive controls
- **Intelligent Analysis**: Graph-based understanding of command relationships and dependencies
- **Cross-Platform Security**: Consistent isolation across Linux, macOS, and Windows

## Security & Agent Integration

### Security Worlds

Isolated execution environments with comprehensive controls:

```bash
# World routing is enabled by default on Linux/macOS.
# Use --no-world (or SUBSTRATE_WORLD=disabled) to opt out for a single run.
substrate -c 'echo hello from world'
```

**Features**:

- Filesystem isolation with overlayfs + mount namespaces (Linux)
- Optional full cage mode (`world_fs.cage=full`) for strong host-path isolation (Linux)
- Network filtering via nftables (best-effort)
- Resource limits via cgroups (best-effort)
- Process isolation via namespaces (Linux)

### AI Agent API

RESTful interface for safe AI assistant integration:

```bash
# Agent API endpoint
curl --unix-socket ~/.substrate/sock/agent.sock \
  -X POST http://localhost/v1/execute \
  -d '{"cmd": "npm test", "agent_id": "claude", "budget": {"max_execs": 10}}'
```

**Capabilities**:

- Per-agent execution budgets
- Scope-based permission tokens
- PTY streaming for interactive commands
- Automatic resource tracking

### Policy Engine

YAML-based policies for comprehensive control:

```yaml
# ~/.substrate/policy.yaml (or per-project .substrate-profile)
id: default
name: Development Policy

world_fs:
  mode: writable            # writable | read_only
  cage: project             # project | full
  require_world: false      # true = no host fallback when world is unavailable
  read_allowlist:
    - "*"                   # required, must be non-empty
  write_allowlist:
    - "./dist/*"            # allow project writes under dist

cmd_allowed: []
cmd_denied:
  - "sudo *"
  - "rm -rf /"
cmd_isolated:
  - "pip install *"
  - "npm install *"

net_allowed:
  - "github.com"
  - "npmjs.org"
  - "crates.io"
limits:
  max_egress_bytes: 1073741824
```

### Graph Intelligence

Kuzu database for command relationship analysis:

```bash
# Inspect graph status
substrate graph status

# Show files touched by a span
substrate graph what-changed <SPAN_ID> --limit 25

# Replay command sequences
substrate replay span_abc123
```

## Cross-Platform Strategy

### Linux (Native)

Full isolation using Linux kernel features:

- Namespaces for process/filesystem/network isolation
- Cgroups v2 for resource limits
- Seccomp for syscall filtering
- nftables for network controls

### macOS (Lima Integration)

Consistent experience via lightweight Linux VM:

- Lima VM with virtiofs for fast file access
- All isolation happens inside VM using Linux stack
- Transparent integration with host development workflow

### Windows (WSL2 Integration)

Future support via WSL2 integration:

- WSL2 distro as isolation backend
- ConPTY for terminal handling
- Reuse Linux isolation code inside WSL2

## Future Capabilities

### Deferred Features

Advanced capabilities planned for later phases:

**Enhanced Security**:

- Advanced seccomp policy tuning for granular syscall filtering
- Sophisticated domain/SNI-based network egress controls
- Complex network filtering with deep packet inspection
- High-isolation microVMs for maximum security scenarios

**AI/ML Integration**:

- Human-readable command training datasets for ML applications
- Intelligent command suggestion and completion
- Automated policy generation from usage patterns
- Predictive resource allocation

**Platform Extensions**:

- Firecracker integration for Linux high-isolation scenarios
- Advanced Windows integration beyond WSL2
- Container runtime integration (Docker, Podman)
- Kubernetes operator for multi-node scenarios

## Development Principles

### Security by Design

- Default-deny policies with explicit allowlists
- Comprehensive audit logging with tamper detection
- Privacy-aware data collection and analysis
- Emergency bypass mechanisms for critical situations

### AI-First Architecture

- APIs designed for programmatic agent interaction
- Structured data formats for machine consumption
- Predictable behavior for automated workflows
- Clear error messages and status reporting

### Cross-Platform Consistency

- Identical policy semantics across platforms
- Unified API surface regardless of backend
- Consistent security guarantees
- Platform-specific optimizations where beneficial

## Migration Strategy

### Gradual Adoption

1. **Phase 1**: Current command tracing (available now)
2. **Phase 2**: Enable worlds with observe-only policies
3. **Phase 3**: Gradually tighten policies from observe to enforce
4. **Phase 4**: Full agent integration with budgets and scopes

### Backward Compatibility

- Existing functionality remains unchanged
- New features opt-in via environment variables
- SHIM_BYPASS escape hatch always available
- Graceful degradation when features unavailable

## Success Metrics

### Performance Targets

- Policy evaluation under 10ms
- Agent API response under 50ms
- Session world reuse over 90%
- Cross-platform overhead under 100ms

### Adoption Goals

- Zero breaking changes to existing workflows
- Seamless AI agent integration
- Comprehensive security coverage
- Intuitive policy authoring experience

For current architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).
For implementation progress, see project management docs in `docs/project_management/`.
