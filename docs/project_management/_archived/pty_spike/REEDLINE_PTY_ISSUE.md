# Reedline PTY Prompt Issue – Investigation & Current Status

## Background

Substrate is a command-line shell that was migrated from Rustyline to Reedline (the line editor used by Nushell) to gain better features and maintenance. However, after migration, we discovered that the PTY prompt issue persists.

## The Problem

### Original Issue (with Rustyline)

```text
substrate> python
>>> exit()
[cursor here, no prompt visible]
# User must press Enter to see prompt
substrate>
```

### Current Issue (with Reedline)

```text
substrate> vim myfile.txt
# [vim opens, user edits, then exits]
[cursor here, no prompt visible]
# User must press Enter or any key to see prompt
substrate>
```

The same behavior occurs with all PTY commands: `python`, `vim`, `claude`, `ssh`, etc.

## Root Cause Analysis

### How Reedline is Supposed to Work (Nushell's approach)

1. User types command and presses Enter
2. Reedline's EditMode parses input and generates `ReedlineEvent::ExecuteHostCommand`
3. Reedline sets internal `suspended_state` to save painter state
4. Reedline returns `Signal::Success(host_command)` to host application
5. Host executes the external command
6. Host calls `read_line()` again
7. Reedline detects `suspended_state`, clears it, and repaints prompt immediately
8. Prompt appears without requiring user input

### How Substrate Currently Works

1. User types command and presses Enter
2. Reedline returns `Signal::Success(line)` with the command text
3. Substrate executes command directly via `execute_command()` and PTY handling
4. After command completes, Substrate calls `read_line()` again
5. Reedline paints prompt internally but then immediately blocks on `crossterm::event::read()`
6. **Prompt is painted but not visible to user until an event arrives**
7. User must press a key to trigger an event, making the prompt visible

### The Critical Difference

- **Nushell**: Uses `ExecuteHostCommand` event → Reedline manages suspend/resume
- **Substrate**: Executes commands outside Reedline → No suspend/resume mechanism

## Investigation via DeepWiki MCP

Through extensive investigation using the DeepWiki MCP server to query the Reedline codebase, we discovered:

1. **`repaint()` is actually private** in Reedline 0.41 (contrary to some documentation)
2. **`suspended_state` is private** and can only be set via `ExecuteHostCommand` event
3. **The prompt IS painted before blocking** but isn't visible until an event arrives
4. **`ExecuteHostCommand` requires EditMode changes** which don't have buffer access

## Attempted Solutions

### 1. TIOCSTI Character Injection

- **Approaches tried:**
  - Inject newline (`\n`) after PTY command
  - Inject Ctrl+L to trigger `ClearScreen` event
  - Inject null character (`\0`)
- **Result:** Characters are processed by Reedline but `read_line()` continues blocking
- **Why it fails:** Injected characters don't make the already-painted prompt visible

### 2. Terminal State Restoration

- **Approaches tried:**
  - Reset ANSI attributes (`\x1b[0m`)
  - Ensure cursor visibility (`\x1b[?25h`)
  - Run `stty sane` to reset terminal
  - Ensure cursor at column 0
- **Result:** Terminal state is clean but prompt still not visible
- **Why it fails:** Terminal state isn't the root cause; prompt is painted but hidden

### 3. ExternalPrinter

- **Approach:** Send messages to ExternalPrinter to trigger repaint
- **Result:** Only processes messages while `read_line()` is actively polling
- **Why it fails:** Can't trigger repaint before `read_line()` starts; messages sent before are ignored

### 4. Manual Prompt Printing

- **Approach:** Print "substrate> " manually before calling `read_line()`
- **Result:** Causes duplicate prompts
- **Why it fails:** Reedline doesn't detect manually printed text as its prompt; creates new prompt on next line

### 5. Cursor Positioning

- **Approach:** Position cursor at specific locations to influence Reedline's prompt placement
- **Result:** Reedline still creates new prompt based on its logic
- **Why it fails:** Doesn't solve the visibility issue; prompt is still painted but not visible

### 6. Custom EditMode Implementation

- **Approach:** Create custom EditMode to emit `ExecuteHostCommand` for PTY commands
- **Result:** Not viable
- **Why it fails:** EditMode's `parse_event()` doesn't have access to line buffer content

## Why This is Hard to Fix

1. **Architectural Mismatch**: Substrate determines which commands need PTY _after_ receiving the line, but Reedline needs to know _before_ returning the line to use `ExecuteHostCommand`

2. **Private APIs**: Key mechanisms (`repaint()`, `suspended_state`) are private in Reedline

3. **Blocking Behavior**: `read_line()` blocks immediately after painting, and the painted content isn't visible until an event arrives (likely a terminal buffering issue)

## Possible Solutions (Not Implemented)

### Option 1: Fork Reedline

- Make `repaint()` public
- Add API to manually set `suspended_state`
- **Downside:** Maintenance burden, divergence from upstream

### Option 2: Restructure Command Execution

- Parse commands before execution to detect PTY needs
- Use different handling for PTY vs non-PTY commands
- **Downside:** Major architectural change, still wouldn't fully solve the issue

### Option 3: Use ExecuteHostCommand Pattern

- Implement command detection in EditMode somehow
- Return commands as `ExecuteHostCommand` events
- **Downside:** EditMode can't access buffer; would need Reedline changes

## Current Status

The migration to Reedline is complete and functional, but the PTY prompt issue persists as a known limitation. Users must press Enter (or any key) after PTY commands to see the prompt.

### Workaround

After PTY commands complete, we:

1. Ensure terminal is on a fresh line
2. Reset terminal state
3. Flush output buffers

This ensures the terminal is in a good state, but the prompt visibility issue remains.

## Recommendation

Accept this as a known limitation for now. The functionality works correctly (just requires an extra keypress), and a proper fix would require either:

1. **Upstream Reedline changes** to expose necessary APIs
2. **Major architectural changes** to Substrate's command handling
3. **Custom terminal handling** that bypasses Reedline for PTY commands

Given that this is a minor UX issue (one extra keypress) and the complexity of a proper fix, it's reasonable to document this as a known issue and revisit if/when Reedline adds better support for this use case.

## Files Modified During Investigation

- `crates/shell/Cargo.toml` - Migrated from Rustyline to Reedline
- `crates/shell/src/lib.rs` - Rewrote `run_interactive_shell()` for Reedline
- `crates/shell/src/pty_exec.rs` - Attempted various workarounds

---

_Last updated: 2025-08-13_
_Investigation used DeepWiki MCP to query nushell/reedline repository_

---

Analysis Complete: PTY Prompt Issue Root Cause Identified

THE EXACT ISSUE

When read_line() is called after a PTY command:

1. Reedline enables raw terminal mode via crossterm
2. Paints the prompt to the terminal buffer
3. Immediately blocks on crossterm::event::read()
4. In raw mode, the painted prompt isn't automatically visible - it requires an input event to trigger
   display

Why Current Solutions Don't Work

- stty sane: Executes BEFORE read_line(), but read_line() immediately re-enables raw mode
- TIOCSTI injection: Characters are processed but don't make the already-painted prompt visible
- repaint(): Method is PRIVATE in Reedline 0.41, cannot be called
- Terminal restoration: Doesn't help because read_line() resets terminal to raw mode

Viable Solutions

Solution 1: Pre-print Prompt with Duplicate Handling

// After PTY command completes
if was_pty_command {
// Print prompt manually
print!("substrate> ");
io::stdout().flush()?;

      // Track that we pre-printed
      let pre_printed = true;

      // When read_line returns, check if we need to clear duplicate
      // This requires detecting if Reedline printed its own prompt

}
Challenge: Detecting and handling duplicate prompts

Solution 2: Background Thread with Timing

// Start read_line in one thread, send ExternalPrinter message after delay
if was_pty_command {
let printer = external_printer.clone();
thread::spawn(move || {
thread::sleep(Duration::from_millis(50)); // After read_line starts
printer.print("".to_string()); // Trigger repaint
});
}
Challenge: Race condition - timing is critical

Solution 3: Restructure for ExecuteHostCommand

Completely redesign command execution to detect PTY commands BEFORE returning from read_line():
// Parse command to detect PTY needs
// Return ExecuteHostCommand event instead of Success
// Let Reedline manage suspend/resume
Challenge: Major architectural change

Solution 4: Fork/Patch Reedline

- Make repaint() public
- Add API to set suspended_state manually
- Submit PR upstream

Challenge: Maintenance burden

Solution 5: Terminal Control Workaround

Force terminal to display buffered content using escape sequences:
if was_pty_command {
// Force terminal to show buffered content
print!("\x1b[6n"); // Request cursor position (forces flush in some terminals)
io::stdout().flush()?;
}
Challenge: Terminal-specific behavior

RECOMMENDED SOLUTION

Given the constraints, I recommend Solution 2 (Background Thread) with improvements:

// In lib.rs after PTY command
if was_pty_command {
// Clean terminal state
println!();
io::stdout().flush()?;

      // Start a background thread to trigger repaint via ExternalPrinter
      let printer = external_printer.clone();
      thread::spawn(move || {
          // Wait for read_line to start and paint initial prompt
          thread::sleep(Duration::from_millis(100));

          // Send empty message to trigger repaint
          // This will cause read_line's event loop to process the message
          // and repaint the prompt, making it visible
          let _ = printer.print("".to_string());
      });

}

This exploits the fact that ExternalPrinter messages are checked periodically by read_line() and
trigger a repaint when processed.

Alternative: Accept and Document

If the above doesn't work reliably, the pragmatic approach is to:

1. Document this as a known limitation
2. Add a message after PTY commands: "Press any key to continue..."
3. Wait for Reedline to add proper APIs
4. Consider this a minor UX issue (one keypress) vs the complexity of fixing it

The fundamental issue is an architectural mismatch between Substrate's command execution model and
Reedline's design expectations. A proper fix requires either Reedline changes or significant Substrate
refactoring.
