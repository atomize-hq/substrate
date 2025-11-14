# PTY Prompt Fix Implementation

> **Historical Note**: This document describes the custom fork housed under `third_party/reedline` during the PTY spike. The fork has been retired in favor of the upstream crate; the content below is preserved for future reference.

## Problem Statement

After exiting PTY commands (vim, python REPL, claude, etc.) in the Substrate shell, users had to press an additional key to see the prompt again. This was a persistent issue that resisted multiple solution attempts.

## Root Cause

The issue had **two root causes**:

1. **Reedline's suspended state**: When Reedline didn't know a command would be executed in a PTY, it would remain in an active state, potentially interfering with terminal restoration.

2. **stdin forwarding thread race condition**: After the PTY child process exited, the stdin forwarding thread would perform one more blocking `read()` on stdin, stealing the next keystroke that was meant to trigger Reedline's prompt display.

## Solution Overview

The fix required two complementary approaches:

### 1. Reedline Fork with ExecuteHostCommand Signal

We forked Reedline to add support for PTY command detection and proper suspension. This ensures Reedline knows when to suspend itself for PTY commands.

### 2. stdin Thread Fix with select()

We fixed the race condition in the stdin forwarding thread by using `select()` with a timeout instead of blocking reads, preventing input from being stolen after PTY exit.

## Implementation Details

### Part 1: Reedline Fork

**Yes, we still need the Reedline fork.** The fork is essential for proper PTY command handling.

#### Files Modified in Reedline Fork (`third_party/reedline/`):

1. **Cargo.toml** - Added two feature flags:
   ```toml
   substrate_api = []         # exposes suspend + repaint APIs
   substrate_host_hook = []   # adds commit-time host hook + signal
   ```

2. **src/enums.rs** - Added `ExecuteHostCommand` variant to Signal enum:
   ```rust
   pub enum Signal {
       Success(String),
       CtrlC,
       CtrlD,
       #[cfg(feature = "substrate_host_hook")]
       ExecuteHostCommand(String),
   }
   ```

3. **src/host_hook.rs** (new file) - Defined the host command decision interface:
   ```rust
   pub enum ExecDecision {
       Success(String),
       ExecuteHostCommand(String),
   }
   
   pub trait HostCommandDecider: Send + Sync {
       fn decide(&self, line: &str) -> ExecDecision;
   }
   ```

4. **src/engine.rs** - Added host decider integration:
   - Added `host_decider` field to Reedline struct
   - Added `set_host_decider()` method
   - Modified `submit_buffer()` to check with host decider before execution
   - Added `substrate_api` methods: `set_suspended()` and `repaint_now()`

5. **src/lib.rs** - Added module exports for host_hook

### Part 2: Substrate Implementation

#### Files Modified in Substrate:

1. **crates/shell/src/host_decider.rs** (new file) - Implements the decider:
   ```rust
   pub struct SubstrateHostDecider;
   
   impl HostCommandDecider for SubstrateHostDecider {
       fn decide(&self, line: &str) -> ExecDecision {
           if crate::needs_pty(line) {
               ExecDecision::ExecuteHostCommand(line.to_string())
           } else {
               ExecDecision::Success(line.to_string())
           }
       }
   }
   ```

2. **crates/shell/src/lib.rs** - Shell loop modifications:
   - Sets up host decider on Reedline
   - Handles `ExecuteHostCommand` signal
   - Adds newline before PTY execution for clean TUI display

3. **crates/shell/src/pty_exec.rs** - Fixed stdin forwarding race condition:
   - Uses `select()` with 100ms timeout instead of blocking reads
   - Properly checks `done` flag between operations
   - Prevents stdin thread from stealing input after PTY exit

4. **crates/shell/Cargo.toml** - Updated dependencies:
   ```toml
   reedline = { version = "0.41", features = ["external_printer", "substrate_api", "substrate_host_hook"] }
   nix = { version = "0.29", features = ["process", "signal", "term", "fs", "time", "poll"] }
   ```

## Why the Reedline Fork is Required

The Reedline fork is **absolutely necessary** for the following reasons:

1. **Signal Flow**: The `ExecuteHostCommand` signal allows Reedline to know when a command should be executed in a PTY, enabling it to properly suspend itself.

2. **Host Decision Hook**: The `HostCommandDecider` trait allows Substrate to decide at runtime which commands need PTY execution, without hardcoding this logic into Reedline.

3. **Suspended State Management**: The `set_suspended()` API ensures Reedline properly manages its internal state when PTY commands are running.

4. **Clean Separation**: This design keeps PTY detection logic in Substrate while giving Reedline the hooks it needs to cooperate.

## Alternative Approaches That Failed

Before arriving at this solution, we tried many approaches that didn't work:

1. **Terminal state restoration with tcsetattr** - Didn't solve the input stealing issue
2. **Repainting before entering raw mode** - The problem was after exit, not before entry
3. **Terminal escape sequences** - Couldn't fix the fundamental race condition
4. **Manual prompt printing** - The stdin thread would still steal input
5. **Non-blocking I/O on stdin directly** - Interfered with programs like claude

## Testing

The fix has been validated with:
- Python REPL (`python3`)
- Vim editor
- Claude interactive mode
- SSH sessions
- Other PTY commands (htop, less, etc.)

All commands now return to the prompt immediately without requiring an extra keypress.

## Future Considerations

### Upstreaming to Reedline

The changes we made to Reedline are general-purpose and could benefit other users. Consider:

1. Opening a PR to upstream Reedline with the `substrate_host_hook` feature
2. The feature is already behind a feature flag, so it won't affect existing users
3. The API is clean and well-separated

### Maintaining the Fork

Until/unless the changes are upstreamed:

1. Keep the fork in `third_party/reedline/`
2. Track upstream Reedline releases and rebase as needed
3. The modifications are minimal and well-contained, making maintenance straightforward

## Complete Reedline Fork Diffs

Here are all the changes made to the Reedline fork in `third_party/reedline/`:

### 1. Cargo.toml (2 lines added)
```diff
@@ -47,6 +47,8 @@ external_printer = ["crossbeam"]
 sqlite = ["rusqlite/bundled", "serde_json"]
 sqlite-dynlib = ["rusqlite", "serde_json"]
 system_clipboard = ["arboard"]
+substrate_api = []         # exposes suspend + repaint APIs
+substrate_host_hook = []   # adds commit-time host hook + signal
 
 [[example]]
 name = "cwd_aware_hinter"
```

### 2. src/enums.rs (3 lines added)
```diff
@@ -12,6 +12,9 @@ pub enum Signal {
     CtrlC, // Interrupt current editing
     /// Abort with `Ctrl+D` signalling `EOF` or abort of a whole interactive session
     CtrlD, // End terminal session
+    /// Execute command with host (for PTY/TUI commands)
+    #[cfg(feature = "substrate_host_hook")]
+    ExecuteHostCommand(String),
 }
 
 /// Editing actions which can be mapped to key bindings.
```

### 3. src/lib.rs (5 lines added)
```diff
@@ -287,6 +287,11 @@ pub use terminal_extensions::kitty_protocol_available;
 mod utils;
 
 mod external_printer;
+
+#[cfg(feature = "substrate_host_hook")]
+mod host_hook;
+#[cfg(feature = "substrate_host_hook")]
+pub use host_hook::{ExecDecision, HostCommandDecider};
 pub use utils::{
     get_reedline_default_keybindings, get_reedline_edit_commands,
     get_reedline_keybinding_modifiers, get_reedline_keycodes, get_reedline_prompt_edit_modes,
```

### 4. src/host_hook.rs (new file, 17 lines)
```rust
/// Decision about how to execute a command
#[cfg(feature = "substrate_host_hook")]
#[derive(Debug, Clone)]
pub enum ExecDecision {
    /// Normal REPL submission
    Success(String),
    /// Host should run this command out-of-band (e.g., PTY)
    ExecuteHostCommand(String),
}

/// Trait for deciding whether a command should be executed by the host
#[cfg(feature = "substrate_host_hook")]
pub trait HostCommandDecider: Send + Sync {
    /// Decide how to handle the given command line
    fn decide(&self, line: &str) -> ExecDecision;
}
```

### 5. src/engine.rs (approximately 65 lines modified/added)

Key changes in engine.rs:

**Added field to Reedline struct (line 173-175):**
```diff
     #[cfg(feature = "external_printer")]
     external_printer: Option<ExternalPrinter<String>>,
+    
+    #[cfg(feature = "substrate_host_hook")]
+    host_decider: Option<std::sync::Arc<dyn crate::HostCommandDecider>>,
 }
```

**Initialize in constructor (line 250-251):**
```diff
             #[cfg(feature = "external_printer")]
             external_printer: None,
+            #[cfg(feature = "substrate_host_hook")]
+            host_decider: None,
         }
```

**Added public methods (lines 631-653):**
```rust
#[cfg(feature = "substrate_api")]
/// Mark editor as suspended or not. When true, the next `read_line` is a "resume".
pub fn set_suspended(&mut self, suspended: bool) {
    if suspended {
        self.suspended_state = Some(self.painter.state_before_suspension());
    } else {
        self.suspended_state = None;
    }
}

#[cfg(feature = "substrate_api")]
/// Force an immediate repaint right now.
pub fn repaint_now(&mut self, prompt: &dyn Prompt) -> Result<()> {
    // Call the same internal render path used when resuming from suspend
    self.repaint(prompt)?;
    Ok(())
}

#[cfg(feature = "substrate_host_hook")]
/// Set the host command decider for determining when to use ExecuteHostCommand
pub fn set_host_decider(&mut self, decider: std::sync::Arc<dyn crate::HostCommandDecider>) {
    self.host_decider = Some(decider);
}
```

**Modified read_line_helper (lines 730-741):**
```diff
-        self.painter
-            .initialize_prompt_position(self.suspended_state.as_ref())?;
+        // Only initialize and repaint if not already done in read_line
         if self.suspended_state.is_some() {
-            // Last editor was suspended to run a ExecuteHostCommand event,
-            // we are resuming operation now.
+            self.painter
+                .initialize_prompt_position(self.suspended_state.as_ref())?;
             self.suspended_state = None;
+            self.hide_hints = false;
+            self.repaint(prompt)?;
+        } else {
+            self.painter.initialize_prompt_position(None)?;
+            self.hide_hints = false;
+            self.repaint(prompt)?;
         }
-        self.hide_hints = false;
-
-        self.repaint(prompt)?;
```

**Modified submit_buffer (lines 1943-1957):**
```diff
         self.run_edit_commands(&[EditCommand::Clear]);
         self.editor.reset_undo_stack();
 
+        #[cfg(feature = "substrate_host_hook")]
+        if let Some(ref decider) = self.host_decider {
+            match decider.decide(&buffer) {
+                crate::ExecDecision::ExecuteHostCommand(cmd) => {
+                    // Mark suspended so the next read_line becomes a resume
+                    self.suspended_state = Some(self.painter.state_before_suspension());
+                    return Ok(EventStatus::Exits(Signal::ExecuteHostCommand(cmd)));
+                }
+                crate::ExecDecision::Success(line) => {
+                    return Ok(EventStatus::Exits(Signal::Success(line)));
+                }
+            }
+        }
+
         Ok(EventStatus::Exits(Signal::Success(buffer)))
     }
 }
```

## Summary Statistics

- **Files Modified**: 4 existing files (Cargo.toml, src/enums.rs, src/lib.rs, src/engine.rs)
- **Files Added**: 1 new file (src/host_hook.rs)
- **Total Lines Added**: Approximately 92 lines
- **Total Lines Modified**: Approximately 15 lines

The changes are minimal, well-contained behind feature flags, and don't affect the default behavior of Reedline. This makes the fork maintainable and potentially suitable for upstreaming.

## Summary

The PTY prompt fix required both:
1. **A Reedline fork** to add the ExecuteHostCommand signal and host decision hook
2. **A stdin thread fix** using select() with timeout to prevent input stealing

Both parts are essential. The Reedline fork cannot be replaced with the stock library as it lacks the necessary hooks for PTY command detection and proper suspension management.
