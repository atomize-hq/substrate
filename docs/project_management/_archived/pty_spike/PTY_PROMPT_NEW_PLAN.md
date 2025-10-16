Here’s a drop-in implementation spec you can hand to Claude. It includes the exact changes for a **minimal Reedline fork**, the **host commit-time hook**, and the **Substrate integration** with PTY + full tracing. It’s written so an engineer can implement without hunting context.

---

# Substrate: Instant Repaint After PTY Exit + Commit-Time Host Hook

**Scope:** Minimal Reedline fork (vendored) + Substrate changes
**Goals:**

- Prompt is visible **immediately** after any PTY TUI exits, **no keypress**.
- Keep **byte-for-byte PTY capture** for audit/training.
- Add a **commit-time hook** so Reedline can emit `ExecuteHostCommand` for interactive lines.
- Keep diffs **small and feature-gated** so we can PR upstream later.

---

## 0) Repo wiring: “vendored fork” options

Pick one:

### Option A: Git fork (recommended)

1. Fork `reedline` into `github.com/<org>/reedline-substrate`.
2. Implement the small changes below on a branch, e.g. `substrate/hooks-and-repaint`.
3. In Substrate **workspace root** `Cargo.toml`:

```toml
[patch.crates-io]
reedline = { git = "https://github.com/<org>/reedline-substrate", rev = "<commit-sha>" }
```

### Option B: Path vendoring (local copy)

1. Copy the Reedline crate into `third_party/reedline/`.
2. Patch it locally.
3. In Substrate **workspace root** `Cargo.toml`:

```toml
[patch.crates-io]
reedline = { path = "third_party/reedline" }
```

Both are “vendoring.” We point Cargo at **our** reedline instead of crates.io.

---

## 1) Reedline fork: feature flags

In the fork’s `Cargo.toml` add:

```toml
[features]
substrate_api = []         # exposes suspend + repaint APIs
substrate_host_hook = []   # adds commit-time host hook + signal
default = []               # keep disabled by default for upstream PR friendliness
```

---

## 2) Reedline fork: Option 2 (deterministic repaint)

### 2.1 Public API on the editor

**File:** `reedline/src/editor.rs` (or wherever `Reedline`/`Editor` impl lives)

Add two public methods behind the feature:

```rust
#[cfg(feature = "substrate_api")]
impl Reedline {
    /// Mark editor as suspended or not. When true, the next `read_line` is a "resume".
    pub fn set_suspended(&mut self, on: bool) {
        // Wrap whatever internal field currently tracks suspended state.
        // Example: self.state.suspended = on;
        self.state.set_suspended(on);
    }

    /// Force an immediate repaint right now.
    pub fn repaint_now(&mut self) -> crate::Result<()> {
        // Call the same internal render path used when resuming from suspend
        // (this is the key path Nushell relies on). Depending on version,
        // this may be named `render()`, `refresh_prompt()`, or similar.
        self.render_immediately()
    }
}
```

**Notes for implementer:**

- Locate the internal method Reedline uses to repaint when it resumes after an external command. Call that. Do **not** re-implement painting here.
- If there is a guard that only allows repaint while idle or resuming, keep it. Return an error if called at the wrong time.
- Keep names minimal and doc comments clear for upstream.

---

## 3) Reedline fork: Option 3 (commit-time host hook)

### 3.1 Add a hook trait and decision enum

**New file:** `reedline/src/host_hook.rs` (or inline near editor)

```rust
#[cfg(feature = "substrate_host_hook")]
pub enum ExecDecision {
    Success(String),               // normal REPL submission
    ExecuteHostCommand(String),    // host should run this command out-of-band (e.g., PTY)
}

#[cfg(feature = "substrate_host_hook")]
pub trait HostCommandDecider: Send + Sync {
    fn decide(&self, line: &str) -> ExecDecision;
}
```

### 3.2 Wire the hook into the Enter path

Find where Reedline finalizes the buffer when Enter is pressed (the “submit” branch in the read loop). Before returning the final line, add:

```rust
#[cfg(feature = "substrate_host_hook")]
if let Some(decider) = &self.host_decider {
    match decider.decide(&final_line) {
        ExecDecision::ExecuteHostCommand(cmd) => {
            // Mark suspended so the next `read_line` becomes a resume
            self.state.set_suspended(true);
            return Ok(Signal::ExecuteHostCommand(cmd)); // add this variant
        }
        ExecDecision::Success(line) => {
            return Ok(Signal::Success(line));
        }
    }
}
```

### 3.3 Add storage + setter on the editor

In the editor struct:

```rust
#[cfg(feature = "substrate_host_hook")]
host_decider: Option<std::sync::Arc<dyn HostCommandDecider>>,
```

Add a setter:

```rust
#[cfg(feature = "substrate_host_hook")]
impl Reedline {
    pub fn set_host_decider(&mut self, decider: std::sync::Arc<dyn HostCommandDecider>) {
        self.host_decider = Some(decider);
    }
}
```

### 3.4 Add the signal enum variant

Where `Signal` (or equivalent) is defined:

```rust
#[derive(Debug)]
pub enum Signal {
    // existing…
    Success(String),
    // …
    #[cfg(feature = "substrate_host_hook")]
    ExecuteHostCommand(String),
}
```

**Behavior expectation:**

- On `ExecuteHostCommand`, the host runs the command “outside” the editor (PTY in our case), then calls back into `read_line()`. Because `suspended==true`, Reedline **repaints immediately** before blocking for input. That returns the instant prompt UX.

---

## 4) Substrate integration

### 4.1 Cargo wiring

In Substrate `Cargo.toml`, make sure we **enable** the fork features:

```toml
[dependencies]
reedline = { version = "*", features = ["substrate_api", "substrate_host_hook"] }
```

Plus the `[patch.crates-io]` section you chose earlier.

### 4.2 Provide the HostCommandDecider

Create `substrate/src/shell/host_decider.rs`:

```rust
use reedline::{ExecDecision, HostCommandDecider};
use std::sync::Arc;

pub struct SubstrateHostDecider {
    // Inject config/allowlists here if needed
}

impl SubstrateHostDecider {
    pub fn new() -> Arc<Self> { Arc::new(Self {}) }
}

impl HostCommandDecider for SubstrateHostDecider {
    fn decide(&self, line: &str) -> ExecDecision {
        // 1) Hard allowlist for known TUIs/REPLs
        const TUI_BINARIES: &[&str] = &[
            "vim", "nvim", "less", "more", "man", "tig",
            "top", "htop", "btop", "watch",
            "python", "python3", "node", "ipython", "irb", "psql", "mysql",
            "fzf", "ssh", "sftp"
        ];

        // naive parse: first token is the bin
        let bin = line.split_whitespace().next().unwrap_or_default();

        // 2) Switch to PTY when stdout is a TTY and bin is known interactive
        if atty::is(atty::Stream::Stdout) && TUI_BINARIES.contains(&bin) {
            return ExecDecision::ExecuteHostCommand(line.to_string());
        }

        // 3) Optional heuristic: if user opted-in to record everything
        if std::env::var_os("SUBSTRATE_RECORD_ALL").is_some() {
            return ExecDecision::ExecuteHostCommand(line.to_string());
        }

        ExecDecision::Success(line.to_string())
    }
}
```

Wire it up where you construct the editor:

```rust
let mut editor = reedline::Reedline::create()?;
// …
editor.set_host_decider(SubstrateHostDecider::new());
```

### 4.3 Main REPL loop handling the new Signal

**File:** wherever your shell loop is (e.g. `crates/shell/src/lib.rs`)

```rust
loop {
    match editor.read_line(&prompt) {
        Ok(reedline::Signal::ExecuteHostCommand(cmd)) => {
            // We are marked suspended=true inside Reedline. Run the command via PTY with full tracing.
            run_pty_with_trace(&cmd)?;
            // Drain/close PTY completely before we repaint to avoid late bytes overwriting the prompt.
            editor.set_suspended(false);      // tell editor we are resuming
            editor.repaint_now()?;            // show prompt instantly (no keypress)
            continue;                         // next iteration calls read_line() again
        }

        Ok(reedline::Signal::Success(line)) => {
            eval_line(&line)?;
        }

        Ok(other) => handle_signal(other)?,
        Err(e) => handle_err(e)?,
    }
}
```

### 4.4 PTY execution with tee + tracing

Use your existing PTY runner, but ensure:

- **Tee** PTY master output to the user **and** to a JSONL trace file.
- Also write **stdin keystrokes** to trace.
- Record **winsize** events and apply them to the PTY slave on host resize.
- Mask `pty_in` frames while ECHO is off if you have visibility. If not, use policy guards around known prompts (e.g. detect “password: ” line) or give users a toggle to redact input.

Example trace schema (stable and simple):

```json
{
  "ts": "2025-08-13T21:12:34.567Z",
  "session_id": "uuid",
  "event": "pty_out|pty_in|winsize|exit",
  "data": "<base64 or utf8>",
  "meta": {
    "rows": 48,
    "cols": 160,
    "echo": true,
    "alt_screen": false,
    "code": 0
  }
}
```

**Order on exit:**

1. Stop stdin → PTY writer.
2. Drain PTY reader until EOF.
3. Close master.
4. Reap child (`waitpid`).
5. Only then call `editor.set_suspended(false); editor.repaint_now()?`.

This ordering prevents the “late byte overwrote my prompt” bug.

---

## 5) Tests (must pass before merge)

Add an integration test harness that spawns the shell in a PTY (use `portable-pty` or similar). The tests should assert the prompt is visible within a small timeout after exit and that no duplicate prompts appear.

**Cases:**

1. `vim -u NONE -c q` → prompt visible < 100 ms after exit.
2. `less Cargo.toml` → send `q` → prompt visible instantly.
3. `python -c "print('x')"` then `exit()` → instant prompt.
4. Resize storm while TUI is open, then quit → prompt visible, no artifacts.
5. Rapid open/close loop (e.g., 10× `nvim -u NONE -c q`) → no duplicate prompts, no missing prompts.
6. Pipe case (non-TTY): `echo foo | less` should **not** trigger PTY path.
7. Trace validation: for one run, trace contains `pty_in`, `pty_out`, `winsize`, and `exit` in correct order.

**Metrics:** log `ms_to_prompt` from child exit to repaint. Track in CI and alert if it regresses.

---

## 6) Rollout + flags

- `SUBSTRATE_RECORD_ALL=1` → force PTY for every command for testing.
- `SUBSTRATE_REPAINT_DEBUG=1` → log when we call `set_suspended()/repaint_now()`.
- Keep Reedline features `substrate_api` and `substrate_host_hook` **enabled** in Substrate’s dependency stanza. Do **not** enable by default in the fork’s `Cargo.toml`.

---

## 7) Upstream PR plan (so we can drop the fork)

Split into two tiny PRs:

1. **Commit-time host hook PR**

   - Adds `HostCommandDecider`, `ExecDecision`, new `Signal::ExecuteHostCommand`.
   - Keeps behavior identical when the hook is not set.
   - Docs: one paragraph + example host loop.

2. **Public “resume & repaint” PR**

   - Adds `set_suspended(bool)` and `repaint_now()` as safe public methods.
   - Guards: callable only when editor is idle or resuming. Return an error otherwise.
   - Docs: advise hosts to call after external commands run outside Reedline.

Keep both PRs independent. Link them in descriptions. Add simple unit tests for the hook path.

---

## 8) Known edge cases and fixes

- **Cursor hidden after TUI**
  Some TUIs leave cursor hidden. After PTY exit, always send `\x1b[?25h` (show cursor) before `repaint_now()`.

- **Alt-screen left enabled**
  If final PTY output did not restore the main screen, send `\x1b[?1049l` once. Track this in trace as `alt_screen=false`.

- **macOS ConHost quirks**
  Always open `/dev/tty` for real TTY ioctls. Avoid using stdout fd for `TIOCSWINSZ` unless it is a TTY.

- **Windows**
  Use ConPTY for interactive capture. The same host loop applies. Replace winsize ioctls with ConPTY APIs.

---

## 9) Acceptance criteria

- After any PTY TUI or REPL exits, the Substrate prompt is visible **without** any keypress.
- No duplicate prompts, no lingering artifacts, and the cursor is visible.
- Full PTY trace (in/out/winsize/exit) is captured for those sessions.
- Non-interactive pipelines do not switch to PTY.
- All integration tests pass on macOS and Linux.

---

## 10) Drop-in checklist for the implementer

1. Create Reedline fork and add `substrate_api` + `substrate_host_hook` features.
2. Implement `set_suspended(bool)` and `repaint_now()` calling the internal resume repaint path.
3. Add `HostCommandDecider`, `ExecDecision`, and `Signal::ExecuteHostCommand`. Wire hook into Enter submit path.
4. Add `set_host_decider` on the editor.
5. In Substrate, patch Cargo to use the fork and enable both features.
6. Implement `SubstrateHostDecider` and set it on the editor.
7. Update the shell loop to handle `ExecuteHostCommand` and call the PTY runner, then `set_suspended(false)` + `repaint_now()`.
8. Ensure PTY runner order: stop stdin → drain → close → reap → repaint.
9. Add integration tests and CI job.
10. Ship behind no user-visible flags. Prepare upstream PRs.

---

If you want this as a repo PR template, say the word and I’ll rewrite the above into commit messages and a PR description.
