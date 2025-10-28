# Phase 4 Concurrent Output Design

## Problem Statement

The original ExternalPrinter implementation for concurrent output caused 2.4% idle CPU usage due to polling every 100ms, even when no messages were being sent. This was discovered and fixed in Phase 3.75.

### Field Evidence (macOS 15.6.1)

- Running `substrate` headless on macOS with no TTY attached pegs one core at ~100% (Activity Monitor and `powermetrics --samplers tasks -i 5` show ~1000 ms/s CPU usage).
- `sample` traces show the main thread cycling through `reedline::engine::Reedline::read_line → crossterm::event::read → read()` with no blocking, indicating a busy loop.
- RSS remains ~6 MB, so it is purely a CPU/energy issue.
- Immediate mitigations include: only starting the REPL when stdin is a TTY, adding backoff to the poll loop, or moving to an async event stream.
- This real-world observation reinforces the need to implement the async/concurrent output design.

 

Host-proxy context: Agents typically connect to a host UDS/TCP endpoint exposed by the host-proxy, which forwards to world-agent inside the world/VM. This does not change REPL logic; agent messages arrive over an async channel.

Cross-reference: For the broader Phase 4 plan, workspace changes, and broker/world integration points, see `docs/project_management/future/implementation_phase4_merged.md`.

## Root Cause Analysis

- ExternalPrinter feature in Reedline uses `event::poll()` with 100ms timeout
- This causes 10 polls/second, each consuming ~2.4ms of CPU time
- Results in constant 2.4% CPU usage even when completely idle
- The feature was added speculatively for Phase 4 but never actually used

## Recommended Solution: Async REPL

Since Phase 4 already uses tokio for the agent API server, convert the REPL to async:

### Benefits
- **Zero CPU usage when idle** - Pure event-driven I/O
- **Scales to multiple agents** - Can handle many concurrent connections
- **Clean architecture** - Aligns with existing Phase 4 tokio usage
- **No polling overhead** - Eliminates the CPU waste issue entirely

### Implementation Example

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::net::UnixListener;

struct AsyncRepl {
    agent_rx: mpsc::Receiver<AgentMessage>,
    stdin: BufReader<tokio::io::Stdin>,
}

impl AsyncRepl {
    async fn run(&mut self) {
        let mut lines = self.stdin.lines();
        
        loop {
            tokio::select! {
                // User input - blocks until data arrives
                Ok(Some(line)) = lines.next_line() => {
                    self.handle_user_command(line).await;
                }
                
                // Agent messages - blocks until message arrives
                Some(msg) = self.agent_rx.recv() => {
                    self.print_agent_message(msg);
                }
                
                // Could add more channels here (e.g., signals)
            }
        }
    }
    
    fn print_agent_message(&self, msg: AgentMessage) {
        // Use ANSI codes to preserve prompt position
        print!("\r\x1b[K"); // Clear current line
        println!("[{}] {}", msg.agent_id, msg.content);
        // Reprint prompt
        self.print_prompt();
    }
}
```

### CLI & Integration

Add an opt‑in flag and branch to the async REPL when enabled.

Add to `crates/shell/src/lib.rs` in `Cli`:
```rust
#[arg(long = "async-repl")]
pub async_repl: bool,
```

Use the flag in interactive mode selection (simplified):
```rust
match &config.mode {
    ShellMode::Interactive { .. } if cli.async_repl => {
        run_async_repl(&config).await
    }
    ShellMode::Interactive { .. } => run_interactive_shell(&config),
    _ => { /* other modes unchanged */ }
}
```

Minimal `run_async_repl` skeleton:
```rust
pub async fn run_async_repl(cfg: &ShellConfig) -> anyhow::Result<i32> {
    use std::io::Write;
    use tokio::io::{stdin, AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(stdin()).lines();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<AgentMessage>(1024);

    // Spawn agent listener (UDS/TCP depending on backend)
    tokio::spawn(agent_listen_and_forward(tx));

    print_prompt();
    std::io::stdout().flush().ok();
    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line? { Some(cmd) => handle_user_command(cfg, cmd).await?, None => break }
                print_prompt();
                std::io::stdout().flush().ok();
            }
            Some(msg) = rx.recv() => {
                print!("\r\x1b[K");
                println!("[{}] {}", msg.agent_id, msg.content);
                print_prompt();
                std::io::stdout().flush().ok();
            }
        }
    }
    Ok(0)
}
```

Note: In `--async-repl` mode we temporarily bypass Reedline’s advanced line-editing (history/completions/multiline). If these features are required during transition, use the select/condvar fallback with `Reedline::suspend_guard()` feeding a queue until full async integration restores parity.

## Migration Strategy

1. **Phase 3.75** (Current)
   - ExternalPrinter removed ✅
   - Sync REPL with 0% idle CPU usage ✅

2. **Phase 4 Initial**
   - Add async REPL behind feature flag
   - `substrate --async-repl` to opt-in
   - Both REPLs coexist during transition

3. **Phase 4 Stable**
   - Make async REPL the default
   - Remove sync REPL code
   - Full integration with agent API

Feature gating:
- Keep a cargo feature `async-repl` (compile-time) to allow building without tokio in minimal environments; default ON for dev builds. The runtime CLI flag `--async-repl` selects the async REPL at runtime.

## Alternative Approaches (If Async Not Viable)

### 1. Select-Based I/O (Unix only)
```rust
use nix::sys::select::{select, FdSet};

fn read_with_agents(stdin_fd: i32, agent_fd: i32) -> Result<Input> {
    let mut read_fds = FdSet::new();
    read_fds.insert(stdin_fd);
    read_fds.insert(agent_fd);
    
    // Blocks until one fd has data
    select(None, &mut read_fds, None, None, None)?;
    
    if read_fds.contains(stdin_fd) {
        Ok(Input::UserLine(read_stdin()?))
    } else {
        Ok(Input::AgentMessage(read_agent()?))
    }
}
```

### 2. Thread with Condition Variable
```rust
struct MessageQueue {
    messages: Mutex<VecDeque<AgentMessage>>,
    condvar: Condvar,
}

// Agent thread pushes messages and signals
// REPL thread waits on condvar with timeout
```

### 3. Modified Reedline with Event Integration
Fork Reedline to add agent fd to its event loop, similar to how it handles terminal resize events.

Minimal fallback for Unix `select()` can be combined with `Reedline::suspend_guard()` for PTY commands and a background thread feeding a queue.

## Design Principles

1. **No Polling** - All I/O must be event-driven
2. **Zero Overhead** - Idle REPL uses 0% CPU
3. **Scalable** - Support multiple concurrent agents
4. **Clean Integration** - Align with Phase 4 architecture
5. **No Blocking in Async** - Use `spawn_blocking` or `AsyncFd` for PTY and other sync I/O
6. **Structured Tracing** - Use `tracing` spans for REPL events and agent messages

## Testing Strategy

1. Verify 0% CPU usage when idle with agents connected
2. Test concurrent output doesn't corrupt terminal state
3. Ensure prompt remains responsive during agent output
4. Validate clean shutdown with active connections

Suggested commands:
- Linux: `pidstat -p $(pgrep substrate) 1 5` or `top -b -p $(pgrep substrate) -n 5`
- macOS: `top -l 3 -stats pid,command,cpu -pid $(pgrep substrate)`
- Cross-check by running `substrate --async-repl`, connecting an agent that sends bursts, then idling.

Idle CPU target:
- Aim for < 0.1% CPU at idle on Linux/macOS when agents are connected but quiescent.

## Standards Alignment

- Errors: No `unwrap`/`expect` in REPL path; return `Result` and surface errors cleanly.
- Async: Do not block tokio executor; PTY and other sync I/O use `spawn_blocking` or `AsyncFd`.
- Observability: Add `tracing` spans around read_line, command execution, and agent message printing; propagate correlation IDs if present.
- Unsafe: Avoid in REPL. If any FD tricks require `unsafe`, add `// SAFETY:` comments and encapsulate.

## Conclusion

The async REPL approach provides the best solution for Phase 4's concurrent output needs. It eliminates the CPU waste issue while providing a clean, scalable architecture for AI agent integration.
