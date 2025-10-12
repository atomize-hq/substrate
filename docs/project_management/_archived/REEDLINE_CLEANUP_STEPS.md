# Reedline PR Preparation Steps

## Phase 1: Setup Clean Environment

```bash
# 1. Create a separate directory for the clean fork
cd ~/repos  # or wherever you keep projects
git clone https://github.com/nushell/reedline reedline-upstream
cd reedline-upstream

# 2. Add your fork as a remote
git remote add myfork git@github.com:YOUR_USERNAME/reedline.git

# 3. Create feature branch
git checkout -b feature/execution-filter
```

## Phase 2: Apply Changes with Clean Names

### File: Cargo.toml
```toml
[features]
# Add these feature flags
execution_filter = []    # Content-based command filtering
suspend_control = []      # Terminal state management APIs
```

### File: src/enums.rs
```rust
// Add to Signal enum
#[cfg(feature = "execution_filter")]
ExecuteHostCommand(String),

// Add new enums
#[cfg(feature = "execution_filter")]
#[derive(Debug)]
pub enum FilterDecision {
    Execute(String),      // Normal execution
    Delegate(String),     // Delegate to host
}

#[cfg(feature = "execution_filter")]
pub trait ExecutionFilter: Send + Sync {
    fn filter(&self, command: &str) -> FilterDecision;
}
```

### File: src/engine.rs
```rust
// Add fields to Reedline struct
#[cfg(feature = "execution_filter")]
execution_filter: Option<Arc<dyn ExecutionFilter>>,

#[cfg(feature = "suspend_control")]
suspended_state: Option<PainterState>,

// Add methods
#[cfg(feature = "execution_filter")]
pub fn set_execution_filter(&mut self, filter: Arc<dyn ExecutionFilter>) {
    self.execution_filter = Some(filter);
}

#[cfg(feature = "suspend_control")]
pub fn suspend(&mut self) {
    self.suspended_state = Some(self.painter.state_before_suspension());
}

#[cfg(feature = "suspend_control")]
pub fn resume(&mut self) -> Result<()> {
    if let Some(state) = self.suspended_state.take() {
        self.painter.restore_state(state)?;
    }
    Ok(())
}

#[cfg(feature = "suspend_control")]
pub fn force_repaint(&mut self, prompt: &dyn Prompt) -> Result<()> {
    self.painter.repaint_now(prompt, &self.editor, None)?;
    Ok(())
}
```

## Phase 3: Add Tests

### File: src/engine.rs (test module)
```rust
#[cfg(test)]
mod execution_filter_tests {
    use super::*;

    #[test]
    #[cfg(feature = "execution_filter")]
    fn test_filter_delegation() {
        struct TestFilter;
        impl ExecutionFilter for TestFilter {
            fn filter(&self, cmd: &str) -> FilterDecision {
                if cmd.starts_with("vim") {
                    FilterDecision::Delegate(cmd.to_string())
                } else {
                    FilterDecision::Execute(cmd.to_string())
                }
            }
        }

        let mut rl = Reedline::create();
        rl.set_execution_filter(Arc::new(TestFilter));
        // Test that vim commands get delegated
    }

    #[test]
    #[cfg(feature = "suspend_control")]
    fn test_suspend_resume() {
        let mut rl = Reedline::create();
        rl.suspend();
        assert!(rl.suspended_state.is_some());
        rl.resume().unwrap();
        assert!(rl.suspended_state.is_none());
    }
}
```

## Phase 4: Create Example

### File: examples/execution_filter.rs
```rust
use reedline::{Reedline, ExecutionFilter, FilterDecision, Signal};
use std::sync::Arc;

/// Example filter that delegates interactive commands to PTY
struct PtyFilter;

impl ExecutionFilter for PtyFilter {
    fn filter(&self, command: &str) -> FilterDecision {
        let needs_pty = ["vim", "nano", "ssh", "htop", "top"]
            .iter()
            .any(|&cmd| command.starts_with(cmd));
        
        if needs_pty {
            println!("Delegating '{}' to PTY handler", command);
            FilterDecision::Delegate(command.to_string())
        } else {
            FilterDecision::Execute(command.to_string())
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut rl = Reedline::create();
    rl.set_execution_filter(Arc::new(PtyFilter));

    loop {
        match rl.read_line(&DefaultPrompt)? {
            Signal::Success(line) => {
                println!("Executing normally: {}", line);
            }
            Signal::ExecuteHostCommand(cmd) => {
                // Suspend reedline
                rl.suspend();
                
                // Run command in PTY
                println!("Running in PTY: {}", cmd);
                // ... actual PTY execution ...
                
                // Resume reedline
                rl.resume()?;
                rl.force_repaint(&DefaultPrompt)?;
            }
            Signal::CtrlC | Signal::CtrlD => break,
        }
    }
    Ok(())
}
```

## Phase 5: PR Description

```markdown
# Add execution filtering with terminal state management

## Summary
This PR adds optional execution filtering that allows REPL applications to intercept commands and decide whether to execute them normally or delegate to external handlers, with proper terminal state management.

## Problem
Many REPL applications need to handle different types of commands differently:
- Interactive commands (vim, ssh, htop) require PTY allocation
- Some commands need container/VM execution
- Certain commands require special security contexts

Currently, reedline executes all commands the same way, forcing applications to either:
1. Break interactive commands
2. Require users to use special keybindings (like the existing ExecuteHostCommand)

## Solution
This PR adds two complementary features (both optional via feature flags):

1. **`execution_filter`**: Allows applications to filter commands at commit-time
2. **`suspend_control`**: APIs for proper terminal state management during delegation

## Example Use Case
```rust
// Automatically detect and handle interactive commands
struct PtyFilter;
impl ExecutionFilter for PtyFilter {
    fn filter(&self, cmd: &str) -> FilterDecision {
        if needs_pty(cmd) {
            FilterDecision::Delegate(cmd.to_string())
        } else {
            FilterDecision::Execute(cmd.to_string())
        }
    }
}
```

## Impact
- **Zero breaking changes** - Everything is behind feature flags
- **Small footprint** - ~100 lines of code
- **Complements existing features** - Works alongside ExecuteHostCommand keybindings

## Testing
- Unit tests for filtering logic
- Unit tests for suspend/resume
- Example demonstrating PTY delegation
- All existing tests pass

## Documentation
- Comprehensive rustdoc comments
- Working example in examples/
- Clear use cases documented

Fixes #[issue_number]
```

## Git Commands Summary

```bash
# After implementing all changes
git add -A
git commit -m "Add execution filtering with terminal state management

- Add execution_filter feature for content-based command filtering
- Add suspend_control feature for terminal state management
- Include tests and examples
- No breaking changes (feature-gated)"

# Push to your fork
git push myfork feature/execution-filter

# Then create PR via GitHub UI
```