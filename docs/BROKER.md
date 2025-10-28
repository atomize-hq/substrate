# Substrate Policy Broker

## Overview

The Substrate Policy Broker (`crates/broker`) provides security policy evaluation for command execution in the Substrate ecosystem. It acts as a gatekeeper, evaluating commands against configurable policies before they execute, supporting both observation and enforcement modes.

### Key Features

- **Dual-mode operation**: Observe (log violations) or Enforce (block execution)
- **Two evaluation paths**: `quick_check()` for shims (fast), `evaluate()` for shell (comprehensive)
- **Profile auto-detection**: Discovers `.substrate-profile` files by walking up directory tree
- **Interactive approvals (enforce mode)**: Risk-assessed approval prompts with caching when the broker is switched out of observe-only operation
- **Optional policy watcher**: File-system hot-reload available behind the `policy-watcher` Cargo feature or in tests
- **Restriction hints**: Suggests isolation requirements for world backend

## Quick Usage Guide

### 1. Enable the Broker

```bash
# Enable policy evaluation (shell defaults to observe-only mode)
export SUBSTRATE_WORLD=enabled

# Run commands through substrate
substrate -c "echo hello"  # Evaluated by broker (violations logged)
```

> **Note:** Moving from observe to enforce mode requires calling
> `substrate_broker::set_observe_only(false)` (or equivalent initialization) in
> the host application. The current `substrate` binary keeps enforcement off by default.

### 2. Create a Policy Profile

Create `.substrate-profile` in your project root:

```yaml
id: my-project
name: Project Security Policy

# Filesystem permissions
fs_read: ["*"]                    # Allow all reads
fs_write: ["/tmp/*", "./dist/*"]  # Restrict writes

# Network permissions  
net_allowed: ["github.com", "*.npmjs.org"]

# Command policies
cmd_denied:                       # Always block these
  - "rm -rf /"
  - "curl * | bash"
  
cmd_isolated:                      # Run in ephemeral world
  - "npm install"
  - "pip install"

require_approval: false            # Interactive approval
allow_shell_operators: true        # Pipes, redirects, etc.
```

### 3. Integration Points

The broker integrates at two levels:

**Shim Integration** (fast path):
```rust
// In shim execution (crates/shim/src/exec.rs)
match quick_check(&argv, cwd) {
    Ok(Decision::Deny(reason)) => return Ok(126),
    _ => continue_execution()
}
```

**Shell Integration** (full evaluation):
```rust
// In shell execution (crates/shell/src/lib.rs)
detect_profile(&cwd);  // Auto-load .substrate-profile
match evaluate(cmd, cwd, world_id) {
    Ok(Decision::AllowWithRestrictions(r)) => apply_restrictions(r),
    Ok(Decision::Deny(reason)) => return blocked(),
    _ => continue_execution()
}
```

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Substrate Shell                       │
│  ┌──────────────┐                 ┌──────────────┐     │
│  │   Command    │────evaluate()──▶│    Broker    │     │
│  │   Execution  │                 │              │     │
│  └──────────────┘                 └──────▲───────┘     │
│                                          │              │
│  ┌──────────────┐                       │              │
│  │     Shim     │───quick_check()───────┘              │
│  │  (Intercept) │                                       │
│  └──────────────┘                 ┌──────────────┐     │
│                                   │   Profile    │     │
│                                   │   Detector   │     │
│                                   └──────────────┘     │
└─────────────────────────────────────────────────────────┘

                           │
                           ▼
                    ┌──────────────┐
                    │   Decision   │
                    ├──────────────┤
                    │ • Allow      │
                    │ • Restrict   │
                    │ • Deny       │
                    └──────────────┘
```

### Decision Flow

```
Command Input
     │
     ▼
[Profile Detection]──────▶ .substrate-profile
     │                           │
     ▼                           ▼
[Policy Load]◀───────────[Profile Found?]
     │
     ▼
[Deny List Check]────────▶ DENY ──▶ Block (exit 126)
     │
     ▼
[Allow List Check]───────▶ NOT IN LIST ──▶ Deny/Observe
     │
     ▼
[Isolation Check]────────▶ MATCH ──▶ AllowWithRestrictions
     │                                    │
     ▼                                    ▼
[Approval Check]                    [World Backend]
     │                               
     ▼
[ALLOW] ──▶ Execute
```

### Module Structure

```
crates/broker/
├── src/
│   ├── lib.rs          # Main broker logic, singleton pattern
│   ├── policy.rs       # Policy structures and rules
│   ├── approval.rs     # Interactive approval system
│   ├── profile.rs      # .substrate-profile detection
│   └── watcher.rs      # Hot-reload file watching
└── Cargo.toml
```

## Architecture Decisions

### 1. Singleton Pattern with Global State

**Decision**: Use a global `GLOBAL_BROKER` singleton with `RwLock` protection.

**Rationale**: 
- Single policy instance across entire process
- Avoids passing broker through deep call stacks
- Enables hot-reload without restart
- Thread-safe for concurrent access

**Trade-off**: Global state makes testing harder, but isolation needs outweigh this.

### 2. Two-Path Evaluation

**Decision**: Separate `quick_check()` (shim) and `evaluate()` (shell) functions.

**Rationale**:
- Shims need minimal latency (< 1ms overhead)
- Shell can afford comprehensive evaluation
- Different contexts have different information available

**Implementation**:
- `quick_check()`: Deny-list only, no file I/O
- `evaluate()`: Full policy, profile detection, approvals

### 3. Observe-by-Default

**Decision**: Start in observe mode unless `SUBSTRATE_WORLD=enabled`.

**Rationale**:
- Safe rollout without breaking workflows
- Gather telemetry before enforcement
- Gradual migration path
- Emergency bypass available (`SHIM_BYPASS=1`)

### 4. Profile Discovery

**Decision**: Walk up directory tree from CWD to find `.substrate-profile`.

**Rationale**:
- Project-specific policies
- No global configuration needed
- Works with monorepos (nearest profile wins)
- Cached for performance

**Limits**: Max 10 directories up, stop at home/root.

### 5. Decision Types

**Decision**: Three-state decision enum instead of boolean.

```rust
enum Decision {
    Allow,
    AllowWithRestrictions(Vec<Restriction>),
    Deny(String),
}
```

**Rationale**:
- Communicate isolation requirements to world backend
- Preserve intent for telemetry
- Enable gradual restriction application
- Future: Apply restrictions via world backend

### 6. Interactive Approvals

**Decision**: Terminal UI with risk assessment and caching.

**Rationale**:
- Human-in-the-loop for sensitive operations
- Risk levels guide decision making
- Session/permanent approval reduces friction
- Cache prevents repeated prompts

**Risk Levels**:
- **Critical**: `rm -rf`, format commands, fork bombs
- **High**: sudo, chmod 777, eval, curl|bash
- **Medium**: package installs, network fetches
- **Low**: Everything else

## Integration with Phase 4

The broker is designed to work with other Phase 4 components:

### World Backend Integration (TODO)
```rust
// When broker returns AllowWithRestrictions
match decision {
    AllowWithRestrictions(restrictions) => {
        for restriction in restrictions {
            match restriction.type_ {
                IsolatedWorld => world.set_ephemeral(),
                OverlayFS => world.enable_overlay(),
                NetworkFilter => world.restrict_network(&restriction.value),
                ResourceLimit => world.set_limits(&restriction.value),
            }
        }
    }
}
```

### Agent API Integration (TODO)
```rust
// Agent commands include agent_id for tracking
let decision = evaluate(&request.cmd, &request.cwd, Some(&request.agent_id))?;

// Apply budget limits
if let Some(budget) = request.budget {
    broker.apply_budget_limits(&request.agent_id, budget)?;
}
```

### Trace Integration (TODO)
```rust
// Log policy decisions to span
trace::policy_decision(&cmd, &decision, &policy_id)?;

// Track violations
if matches!(decision, Decision::Deny(_)) {
    trace::policy_violation(&cmd, &reason)?;
}
```

## Testing

### Unit Tests
```bash
cargo test -p substrate-broker
# 17 tests pass
```

### Integration Test
```bash
# Create test profile
cat > .substrate-profile <<EOF
cmd_denied: ["rm -rf /"]
EOF

# Test enforcement
SUBSTRATE_WORLD=enabled substrate -c "rm -rf /"
# Output: substrate: command denied by policy

# Test observe mode (default)
substrate -c "echo test"
# Output: test (no policy evaluation)
```

## Future Enhancements

1. **Policy Composition**: Merge multiple profiles (user + project + system)
2. **Dynamic Rules**: Time-based, context-aware policies
3. **Telemetry**: Send violations to observability backend
4. **AI Suggestions**: Learn patterns and suggest policy updates
5. **Scope Tokens**: Cryptographic proof of allowed operations

## References

- Implementation: `crates/broker/`
- Integration: `crates/shim/src/exec.rs`, `crates/shell/src/lib.rs`
- Phase 4 Plan: `docs/project_management/future/implementation_phase4_merged.md`
