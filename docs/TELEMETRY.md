# Substrate Telemetry Library

## Overview

The Substrate Telemetry Library (`crates/telemetry-lib/`) provides syscall-level interception for comprehensive command execution tracking within isolated environments. It uses Linux's `LD_PRELOAD` mechanism to intercept system calls and log them to the trace.jsonl format.

## Architecture

### Components

1. **Core Library** (`src/lib.rs`)
   - Lazy initialization on first syscall
   - JSON event serialization
   - Thread-safe trace file handling

2. **Correlation Module** (`src/correlation.rs`)
   - Session ID management
   - Parent span tracking
   - Environment variable inheritance

3. **Interception Modules**
   - **Exec** (`src/exec.rs`): `execve`, `execvp`, `system`, `popen`
   - **File** (`src/file.rs`): `open`, `openat`, `creat`, `unlink`, `rename`
   - **Network** (`src/network.rs`): `connect`, `bind`, `accept`, `getaddrinfo`

### Design Principles

- **Telemetry Only**: Never enforces policy, only observes
- **Minimal Overhead**: <10ms per intercepted call
- **Correlation Preserved**: Session/span IDs flow through fork/exec
- **Safe Defaults**: Skips system paths to avoid recursion

## Usage

### Environment Variables

```bash
# Required for correlation
export SUBSTRATE_SESSION_ID="unique_session_id"
export SUBSTRATE_TRACE_LOG="/path/to/trace.jsonl"

# Optional
export SUBSTRATE_PARENT_SPAN="parent_span_id"
export SUBSTRATE_WORLD_ID="world_instance_id"
export SUBSTRATE_AGENT_ID="agent_name"
export SUBSTRATE_POLICY_ID="policy_name"
```

### Linux Deployment

```bash
# Build the library
cargo build -p substrate-telemetry --release

# Use with LD_PRELOAD
LD_PRELOAD=/path/to/libsubstrate_telemetry.so command_to_trace
```

### Docker/Container Deployment

```dockerfile
# Copy library into container
COPY libsubstrate_telemetry.so /usr/lib/substrate/telemetry.so

# Set in environment
ENV LD_PRELOAD=/usr/lib/substrate/telemetry.so
```

## Platform Support

### Linux
- **Status**: ✅ Fully Supported
- **Mechanism**: `LD_PRELOAD` with `dlsym(RTLD_NEXT, ...)`
- **Testing**: Native and containerized environments

### macOS
- **Status**: ⚠️ Limited Support
- **Issue**: System Integrity Protection (SIP) blocks `DYLD_INSERT_LIBRARIES` for system binaries
- **Solution**: Use Docker containers or Lima VMs for testing
- **Production**: Run inside Lima VM (MacLima backend)

### Windows
- **Status**: ✅ Supported inside the `substrate-wsl` world (the library runs in the Linux guest). Native Win32 interception is not provided.
- **Mechanism**: Injected automatically by the world backend when telemetry is enabled, identical to the Linux flow.
- **Notes**: Use the PowerShell warm/doctors in `docs/cross-platform/wsl_world_setup.md` to provision the WSL distro before relying on telemetry.

## Event Format

Events are logged as JSONL to the trace file:

```json
{
  "ts": "2025-09-03T13:55:14.921517011+00:00",
  "event_type": "syscall",
  "session_id": "test_1756907714",
  "span_id": "spn_01990fdc-3969-7231-b9ba-581feadd9a70",
  "parent_span": null,
  "component": "telemetry",
  "syscall": "execve",
  "args": ["/usr/bin/cat", "[\"cat\", \"/tmp/test.txt\"]"],
  "result": null,
  "elapsed_us": 2
}
```

## Integration with Substrate

The telemetry library is designed to be injected when worlds are created:

```rust
// In world backend
if world_spec.enable_telemetry {
    env.insert("LD_PRELOAD", "/usr/lib/substrate/telemetry.so");
    env.insert("SUBSTRATE_SESSION_ID", session_id);
    env.insert("SUBSTRATE_PARENT_SPAN", current_span_id);
}
```

## Testing

### Docker Test Container

```bash
# Build test container (includes library compilation)
docker build -t telemetry-test -f crates/telemetry-lib/Dockerfile.standalone .

# Run tests
docker run --rm telemetry-test
```

### Expected Output

```
=== Substrate Telemetry Library Test ===
Session: test_1756907714
Trace: /tmp/trace.jsonl

Test 1: Check library loads
Library loaded successfully

Test 2: Simple file operation
test

Test 3: System call
adduser.conf
alternatives
apt

=== Results ===
Events captured: 5
First 3 events: [JSON events displayed]
```

## Security Considerations

1. **No Enforcement**: The library only observes, never blocks
2. **Credential Safety**: Doesn't log sensitive data from intercepted calls
3. **Path Filtering**: Skips /proc, /sys, /dev to avoid system interference
4. **Container Isolation**: Runs inside worlds/VMs, not on host

## Performance

- **Initialization**: ~1ms on first syscall
- **Per-syscall overhead**: <10ms including JSON serialization
- **File I/O**: Buffered writes with periodic flush
- **Memory**: Minimal - only session info cached

## Limitations

1. **Variadic Functions**: `open()` and `openat()` mode parameter handling is simplified
2. **Signal Safety**: Not async-signal-safe (avoid in signal handlers)
3. **Static Linking**: Won't intercept statically linked binaries
4. **macOS Host**: Requires Docker/VM due to SIP restrictions

## Future Enhancements

- [ ] Add `read`/`write` syscall interception for data flow tracking
- [ ] Implement `fork`/`clone` tracking for process tree visualization
- [ ] Add configuration file support for selective interception
- [ ] Support for `LD_AUDIT` interface (more robust than `LD_PRELOAD`)
- [ ] Windows support via Detours or API hooking
