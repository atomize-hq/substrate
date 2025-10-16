# Substrate Trace Module

## Overview

The Substrate Trace module (`crates/trace`) provides comprehensive span-based tracing for command execution across the Substrate ecosystem. It captures detailed execution context, policy decisions, and system state to enable command replay, security auditing, and graph-based analysis of command relationships.

### Key Features

- **Extended JSONL Schema**: Rich span format with policy decisions, graph edges, and replay context
- **Session Correlation**: Automatic parent-child span linking across nested executions
- **Policy Integration**: Captures broker decisions (allow/deny/restrict) in spans
- **Replay Context**: Preserves environment state for deterministic command replay
- **Graph Support**: Optional Kuzu database integration for relationship analysis
- **Component Attribution**: Tracks whether spans originate from shell, shim, or other components

## Quick Usage Guide

### Enabling Trace

Trace functionality is active by default whenever you launch the Substrate shell: the CLI calls `ensure_world_ready` and sets `SUBSTRATE_WORLD=enabled` on Linux, macOS (Lima), and Windows (WSL). If you need to emit spans from a custom wrapper or test harness, export the variables manually before launching the command:

```bash
export SUBSTRATE_WORLD=enabled
substrate -c "npm install"
```

### Trace Output Location

By default, traces are written to:
- `~/.substrate/trace.jsonl` (default)
- Or path specified in `SHIM_TRACE_LOG` environment variable

### Viewing Traces

```bash
# View latest span
tail -1 ~/.substrate/trace.jsonl | jq .

# Filter by session
jq 'select(.session_id == "ses_xxx")' ~/.substrate/trace.jsonl

# Find denied commands
jq 'select(.policy_decision.action == "deny")' ~/.substrate/trace.jsonl
```

### Span Schema

```json
{
  "ts": "2024-01-01T00:00:00Z",
  "event_type": "command_complete",
  "session_id": "ses_xxx",
  "span_id": "spn_xxx",
  "parent_span": "spn_yyy",
  "component": "shell|shim",
  "world_id": "wld_xxx",
  "policy_id": "default",
  "agent_id": "human|claude|cursor",
  "cwd": "/projects/foo",
  "cmd": "npm install",
  "exit": 0,
  "scopes_used": ["fs.write:/projects/foo/node_modules"],
  "fs_diff": {
    "writes": ["node_modules/..."],
    "mods": ["package-lock.json"],
    "deletes": [],
    "display_path": {
      "/mnt/c/projects/foo/node_modules": "C:\\projects\\foo\\node_modules"
    }
  },
  "replay_context": {
    "env_hash": "abc123...",
    "umask": 22,
    "locale": "en_US.UTF-8",
    "policy_commit": "def456..."
  },
  "policy_decision": {
    "action": "allow",
    "reason": null,
    "restrictions": null
  }
}
```

- Windows adds an optional `fs_diff.display_path` map that pairs canonical paths (e.g., `/mnt/c/...`) with native Windows representations; Linux and macOS omit this field. The map is populated by the `world-windows-wsl` backend and available whenever a diff is returned.

## Architecture

### Component Integration

```mermaid
graph TB
    subgraph "User Space"
        Shell[substrate shell]
        Shim[substrate-shim]
    end
    
    subgraph "Phase 4 Components"
        Broker[Policy Broker]
        Trace[Trace Module]
        World[World Backend]
        Telemetry[LD_PRELOAD Telemetry]
    end
    
    subgraph "Storage"
        JSONL[trace.jsonl]
        Kuzu[(Kuzu Graph DB)]
    end
    
    Shell -->|evaluate| Broker
    Shim -->|quick_check| Broker
    
    Shell -->|create_span| Trace
    Shim -->|create_span| Trace
    
    Broker -->|Decision| Trace
    World -->|scopes/diff| Trace
    Telemetry -->|syscalls| JSONL
    
    Trace -->|append| JSONL
    Trace -.->|ingest| Kuzu
    
    style Kuzu stroke-dasharray: 5 5
```

### Span Lifecycle

```mermaid
sequenceDiagram
    participant User
    participant Shell
    participant Broker
    participant Trace
    participant Shim
    participant Command
    
    User->>Shell: substrate -c "cmd"
    Shell->>Broker: evaluate(cmd)
    Broker-->>Shell: Decision
    
    Shell->>Trace: create_span_builder()
    Note over Trace: Generate span_id
    Shell->>Trace: .with_policy_decision()
    Shell->>Trace: .start()
    Note over Trace: Write command_start
    
    Shell->>Shim: exec(cmd)
    Note over Shim: SHIM_PARENT_SPAN set
    
    Shim->>Command: execve()
    Command-->>Shim: exit(0)
    
    Shim-->>Shell: exit status
    
    Shell->>Trace: span.finish(exit, scopes, diff)
    Note over Trace: Write command_complete
    Note over Trace: Capture replay_context
```

### Key Design Decisions

1. **Lazy Initialization**: Trace only initializes when `SUBSTRATE_WORLD=enabled`, keeping Phase 1-3 functionality unchanged.

2. **Environment-Based Correlation**: Parent span IDs are passed via `SHIM_PARENT_SPAN` environment variable, enabling correlation across process boundaries without IPC.

3. **Policy Decision Embedding**: Broker decisions are converted to trace-friendly format and embedded in spans for audit trails.

4. **Replay Context**: Captures sufficient environment state (PATH, env hash, umask, locale) to enable deterministic replay in future worlds.

5. **Feature-Gated Graph**: Kuzu integration is behind the `graph` feature flag to keep base dependencies minimal.

6. **Component Attribution**: Spans identify their origin (shell vs shim) via environment detection, crucial for understanding execution flow.

7. **World Integration Complete**: `scopes_used` and `fs_diff` are now populated via world backend integration (PR#10 ✅).

### Module Structure

```
crates/trace/
├── Cargo.toml          # Dependencies, kuzu feature flag
└── src/
    └── lib.rs          # Core implementation
        ├── Span        # JSONL schema structs
        ├── SpanBuilder # Fluent API for span creation
        ├── ActiveSpan  # In-flight span tracking
        ├── TraceOutput # JSONL file writer
        └── kuzu_integration # Feature-gated graph DB
```

### Integration Points

1. **Shell** (`crates/shell/src/lib.rs`):
   - Initializes trace in `run_shell()`
   - Creates spans in `execute_command()`
   - Captures policy decisions from broker
   - Sets `SHIM_PARENT_SPAN` for child processes

2. **Shim** (`crates/shim/src/exec.rs`):
   - Imports ready but not yet creating spans
   - Will create spans for direct shim executions
   - Inherits `SHIM_PARENT_SPAN` from parent

3. **Broker** (`crates/broker`):
   - Decisions are converted to `PolicyDecision` format
   - Restrictions are stringified for trace storage

4. **Telemetry Library** (`crates/telemetry-lib/`):
   - LD_PRELOAD syscall interception inside worlds/VMs
   - Writes syscall events directly to trace.jsonl
   - Maintains session correlation via environment variables
   - Complements span-level tracing with syscall-level detail

5. **Replay Module** (`crates/replay/`):
   - Consumes trace.jsonl for deterministic replay
   - Reconstructs environment from replay_context
   - Enables regression testing and debugging

## Recent Enhancements

### ✅ PR#10 Complete: Overlayfs & Network Filtering  
- `scopes_used` populated with actual filesystem/network access
- `fs_diff` captures overlayfs changes with smart truncation  
- Network scope tracking via nftables integration
- Unified FsDiff type in substrate-common

### ✅ PR#11 Complete: LD_PRELOAD Telemetry
- Syscall-level interception via `crates/telemetry-lib/`
- Intercepts exec*, file ops, network calls inside worlds/VMs
- Session correlation through fork/exec boundaries
- Docker-tested Linux compatibility

### ✅ PR#12 Complete: Replay Module
- Trace replay engine for regression testing (`crates/replay/`)
- Deterministic command replay with environment reconstruction
- Output comparison with non-deterministic element handling
- Batch testing and HTML regression reports

## Future Enhancements

### PR#13-14: Graph Intelligence
- Full Kuzu integration with query interface
- Graph-based security analysis
- Command dependency visualization
- Replay orchestration

## Testing

```bash
# Run unit tests
cargo test -p substrate-trace

# Test with Phase 4 enabled
SUBSTRATE_WORLD=enabled cargo run --bin substrate -- -c "echo test"

# Verify trace generation
tail -1 ~/.substrate/trace.jsonl | jq .

# Test with graph feature
cargo test -p substrate-trace --features graph
```

## Configuration

### Environment Variables

- `SUBSTRATE_WORLD=enabled` - Enable Phase 4 features including trace
- `SHIM_TRACE_LOG=/path/to/trace.jsonl` - Custom trace output location
- `SHIM_PARENT_SPAN=spn_xxx` - Parent span ID (set automatically)
- `SUBSTRATE_AGENT_ID=claude` - Identify AI agent (defaults to "human")
- `SHIM_FSYNC=1` - Force filesystem sync after each span write

### Performance Considerations

- Span creation overhead: < 1ms
- JSONL append: O(1) with buffered writes
- Graph ingestion: Async when implemented
- No performance impact when `SUBSTRATE_WORLD` is not enabled
