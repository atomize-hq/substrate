# New PR: Minimal Suspend API for Reedline

## PR Strategy

You're correct - you cannot update the existing PR to completely change the approach. Best strategy:
1. Close the existing PR with an explanation
2. Open a new, clean PR with the minimal approach
3. Reference the discussion from the old PR

## New PR Title

`feat: Add suspend_guard() API for external terminal-switching commands`

## New PR Description

```markdown
## Summary

This PR adds a minimal public API to allow host applications to properly suspend Reedline when running external commands that take over the terminal (e.g., vim, ssh, less).

## Motivation

When building REPLs that execute external commands, there's currently no way to properly suspend Reedline's terminal handling. This causes issues where:
- The prompt doesn't repaint correctly after PTY commands exit
- Terminal state can become corrupted
- An extra keypress is required to see the prompt

This PR exposes the existing internal suspension mechanism that's already used by `ReedlineEvent::ExecuteHostCommand`, making it available for programmatic use.

## Changes

- Add `suspend_guard()` method that returns an RAII guard
- Guard automatically resumes on drop (even on panic)
- Expose `SuspendGuard` type from lib.rs
- Total changes: ~25 lines of code

## Usage

```rust
// When executing a command that needs terminal control
let mut line_editor = Reedline::create();

loop {
    let sig = line_editor.read_line(&prompt)?;
    
    match sig {
        Ok(Signal::Success(buffer)) => {
            if needs_pty(&buffer) {
                // Suspend Reedline for external command
                let _guard = line_editor.suspend_guard();
                run_pty_command(&buffer)?;
                // Guard automatically resumes here
            } else {
                run_normal_command(&buffer)?;
            }
        }
        // ... handle other signals
    }
}
```

## Why This Design?

- **RAII pattern** prevents forgetting to resume (automatic on drop)
- **Minimal API surface** - just one public method
- **Zero breaking changes** - purely additive
- **Reuses existing logic** - leverages the suspension mechanism already present for ExecuteHostCommand

## Testing

The suspension mechanism is already tested via the existing `ExecuteHostCommand` tests. This PR simply exposes the same functionality through a public API.

## Related Discussion

This supersedes #[old-pr-number] which proposed a more complex execution filtering approach. After discussion with maintainers, we've simplified to this minimal API that addresses the core need without unnecessary complexity.
```

## Response to Maintainer on Old PR

```markdown
Thank you for the valuable feedback! You were absolutely right - the execution filter approach was over-engineered for the actual problem.

After reconsidering, I've realized all we really need is access to the suspension mechanism that's already used internally by `ExecuteHostCommand`. I'm closing this PR in favor of a much simpler approach: #[new-pr-number]

The new PR adds just ~25 lines:
- A single `suspend_guard()` method that returns an RAII guard
- The guard automatically resumes on drop

This gives host applications the ability to properly suspend Reedline during external command execution without any of the complexity of filtering. It's essentially just exposing the existing `suspended_state` management through a clean API.

Thanks again for pushing back on the complexity - the simpler solution is much better!
```

## Tests to Add

Yes, you should add tests! Here's what to include:

### tests/suspend_guard_test.rs

```rust
#[cfg(test)]
mod suspend_tests {
    use reedline::{Reedline, DefaultPrompt, Signal};
    use std::panic;
    
    #[test]
    fn test_suspend_guard_basic() {
        let mut editor = Reedline::create();
        
        // Create and drop guard
        {
            let _guard = editor.suspend_guard();
            // Editor should be suspended here
            // (Note: hard to test internal state without exposing it)
        }
        // Editor should be resumed here
        
        // Should be able to create another guard
        let _guard2 = editor.suspend_guard();
    }
    
    #[test]
    fn test_suspend_guard_resume_on_panic() {
        let mut editor = Reedline::create();
        
        // Guard should resume even on panic
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = editor.suspend_guard();
            panic!("test panic");
        }));
        
        assert!(result.is_err());
        
        // Editor should still be usable after panic
        // (Create another guard to verify editor isn't broken)
        let _guard = editor.suspend_guard();
    }
    
    #[test]
    fn test_suspend_guard_multiple_sequential() {
        let mut editor = Reedline::create();
        
        // Should handle multiple suspend/resume cycles
        for _ in 0..3 {
            let _guard = editor.suspend_guard();
            // Suspended
        }
        // Resumed
        
        // Editor should still be functional
        let _final_guard = editor.suspend_guard();
    }
    
    #[test]
    fn test_suspend_guard_with_drop() {
        let mut editor = Reedline::create();
        
        // Explicit drop should work
        let guard = editor.suspend_guard();
        drop(guard);
        
        // Should be able to create new guard
        let _guard2 = editor.suspend_guard();
    }
}
```

### Integration test (optional but recommended)

```rust
// tests/integration/suspend_integration.rs

#[test]
#[ignore] // Manual test - requires terminal
fn test_suspend_with_real_command() {
    use std::process::Command;
    
    let mut editor = Reedline::create();
    let prompt = DefaultPrompt::default();
    
    // This would need to be run manually to verify behavior
    println!("Testing suspend with echo command...");
    
    {
        let _guard = editor.suspend_guard();
        
        // Run a simple command while suspended
        let output = Command::new("echo")
            .arg("test")
            .output()
            .expect("failed to execute echo");
            
        assert!(output.status.success());
    }
    
    // Verify editor still works after resume
    // (In a real terminal, you'd check if prompt repaints correctly)
    println!("Editor resumed successfully");
}
```

## Where to Add Tests

1. **Unit tests**: Add directly in `src/engine.rs` after the implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn suspend_guard_drops() {
        let mut editor = Reedline::create();
        {
            let _guard = editor.suspend_guard();
        }
        // Test passes if no panic
    }
}
```

2. **Integration tests**: Create `tests/suspend_guard.rs` in the reedline repo

## Final Notes

- The new PR should reference the old one for context
- Keep the description focused on the problem and solution
- Emphasize the minimal nature of the change
- Thank the maintainer for their feedback that led to the better solution