# Phase 4 Concurrent Output Design

## Problem Statement

The original ExternalPrinter implementation for concurrent output caused 2.4% idle CPU usage due to polling every 100ms, even when no messages were being sent. This was discovered and fixed in Phase 3.75.

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

## Design Principles

1. **No Polling** - All I/O must be event-driven
2. **Zero Overhead** - Idle REPL uses 0% CPU
3. **Scalable** - Support multiple concurrent agents
4. **Clean Integration** - Align with Phase 4 architecture

## Testing Strategy

1. Verify 0% CPU usage when idle with agents connected
2. Test concurrent output doesn't corrupt terminal state
3. Ensure prompt remains responsive during agent output
4. Validate clean shutdown with active connections

## Conclusion

The async REPL approach provides the best solution for Phase 4's concurrent output needs. It eliminates the CPU waste issue while providing a clean, scalable architecture for AI agent integration.