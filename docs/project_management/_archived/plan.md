Here’s a compact, end-to-end plan you can save and build from. It rolls up what we learned and lays out a clean Rust design to **trace everything a user (or an agent like Claude) runs**, without bricking shells or TUIs.

---

## Objectives

- **Interception coverage:** binaries (git, node, python, …), core utils (awk/sed/grep) when desired, pipelines, built-ins (cd/export), and agent-spawned commands (Claude, etc.).
- **Stability:** no login loops; TUIs work; minimal overhead.
- **Observability:** structured logs (JSONL), correct PIDs, CWD, argv, exit codes, timings. Optional keystroke-free “xtrace” for built-ins.
- **Portability:** macOS & Linux first; Windows later (PowerShell + CreateProcess quirks).

---

## Architecture (3 layers)

1. **Shim layer (per-tool launchers)**

   - Small Rust binary copied to `~/.cmdshim_rust/<tool>` (no symlinks).
   - For each intercepted command name, the file is the shim (a copy of the binary).
   - Looks up the **real** binary from a clean path (see below), logs, then `exec`s it.

2. **Launcher (supervisor/wrapper)**

   - Spawns target app (Claude or your own shell REPL) with a **deduped PATH** and clean `ORIGINAL_PATH`.
   - Optionally sets `BASH_ENV` to inject pins/trace for non-interactive bash sessions.

3. **Optional custom shell (Rust REPL + PTY)**

   - For “full control” mode: run a Rust interactive shell that parses commands, runs them in a PTY, captures built-ins via a thin bash layer, and streams logs.
   - Still keep shims to catch sub-processes that child programs spawn independently.

---

## Path & recursion rules (non-negotiable)

- **Never** shim `bash`, `sh`, `zsh`, `fish`. (This caused recursion crashes.)
- Maintain two paths:

  - `ORIGINAL_PATH` = “real world” (no shim dir).
  - `PATH` = `~/.cmdshim_rust:` + `ORIGINAL_PATH` (deduped).

- In shim resolution: **skip** any entries under `~/.cmdshim_rust`.
- Before `exec` in the shim: set `SHIM_ACTIVE=1`; if present on entry, skip logging to avoid double logs.
- If a tool isn’t found: exit `127` (pass-through semantics).

---

## Logging

- **Format:** JSON Lines (`~/.trace_shell.jsonl`).
- **Fields:** `ts` (RFC3339), `command`, `argv`, `cwd`, `exit_code`, `duration_ms`, `pid`, `ppid`, `user`, `host`, `platform`, `mode` (noninteractive/interactive), `tty` (bool), optional `env_diff` (keys changed), optional `stderr_bytes` (size only), optional `session_id`.
- Keep logs lightweight (no payload dumps); add redaction hooks later.

---

## Implementation plan (phased)

### Phase 1 — Intercept binaries (works today; you already proved it)

**Shim binary (Rust)**

- Parse `argv[0]` to learn the **requested command name** (e.g., “git”).
- Compute `shimdir = current_exe().parent()`.
- Build a search path:

  - If `ORIGINAL_PATH` present → use it; else use `PATH`.
  - Filter out any PATH entries that start with `shimdir`.

- Resolve the real binary (respect PATHEXT on Windows later).
- Start timer; set `SHIM_ACTIVE=1`; exec the real path with original `argv[1..]`.
- On error, return `127`. On success, log line with duration and exit code.

**Key snippet (pseudo-Rust)**

```rust
fn clean_search_path(shimdir: &Path, original_path: Option<String>) -> Vec<PathBuf> {
    let p = original_path.unwrap_or_else(|| std::env::var("PATH").unwrap_or_default());
    let sep = if cfg!(windows) { ';' } else { ':' };
    p.split(sep)
     .filter(|s| !s.is_empty())
     .map(PathBuf::from)
     .filter(|d| !d.starts_with(shimdir))
     .collect()
}

fn resolve(cmd: &str, path_dirs: &[PathBuf]) -> Option<PathBuf> { /* stat loop */ }

fn main() -> Result<()> {
    let exe = std::env::current_exe()?;
    let shimdir = exe.parent().unwrap().to_path_buf();
    let cmd = exe.file_name().unwrap().to_string_lossy().to_string();

    let original_path = std::env::var("ORIGINAL_PATH").ok();
    let dirs = clean_search_path(&shimdir, original_path);

    let real = resolve(&cmd, &dirs).ok_or_else(|| anyhow!("not found"))?;
    let ts = now_rfc3339();
    let start = Instant::now();

    // mark active to avoid nested logging
    std::env::set_var("SHIM_ACTIVE", "1");

    // exec with argv[1..]
    let argv: Vec<_> = std::env::args_os().collect();
    let status = exec_and_wait(&real, &argv[1..])?; // on Unix, use execvp or spawn+wait

    write_log(ts, &cmd, &argv, status, start.elapsed());
    std::process::exit(status.code().unwrap_or(1));
}
```

**Staging**

```bash
mkdir -p ~/.cmdshim_rust
install -m 0755 target/release/shim ~/.cmdshim_rust/.shimbin
# Create real files (no symlinks) for tools you want to trace:
for c in git npm npx node pnpm bun python python3 pip pip3 jq \
         curl wget tar unzip make go cargo deno docker kubectl; do
  install -m 0755 ~/.cmdshim_rust/.shimbin ~/.cmdshim_rust/$c
done
```

**Session env (manual)**

```bash
export ORIGINAL_PATH="$HOME/.nvm/versions/node/v22.16.0/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.bun/bin"
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
hash -r
```

**Test**

```bash
TRACE_LOG_FILE=/tmp/trace.jsonl git --version
tail -n1 /tmp/trace.jsonl
```

> Avoid shimming core utils at first. Once the shim is hardened, you can add them back.

---

### Phase 2 — Make ephemeral Bash sessions (Claude) honor the shim automatically

**Why:** Each `run …` in Claude spawns a fresh non-interactive bash. Use `BASH_ENV` to inject env + pins every time.

**Create `~/.claude_bashenv`**

```bash
cat > ~/.claude_bashenv <<'EOF'
export ORIGINAL_PATH="$HOME/.nvm/versions/node/v22.16.0/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.bun/bin"
PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"

# Clear and pin to shims (prevents bash hash from sticking to /usr/bin/*)
hash -r
for c in git npm npx node pnpm bun python python3 pip pip3 jq \
         curl wget tar unzip make go cargo deno docker kubectl; do
  [ -x "$HOME/.cmdshim_rust/$c" ] && hash -p "$HOME/.cmdshim_rust/$c" "$c"
done

# Optional: trace built-ins/functions
: "${TRACE_SHELL:=$HOME/.trace_shell.jsonl}"
exec {__xtrace_fd}>>"$TRACE_SHELL"
BASH_XTRACEFD=$__xtrace_fd
PS4='+CMD ${EPOCHREALTIME} ${BASH_SOURCE##*/}:${LINENO}: '
set -o xtrace
EOF
```

**Launch Claude**

```bash
BASH_ENV="$HOME/.claude_bashenv" claude code
# or via your Rust launcher with env -i, injecting BASH_ENV, PATH, ORIGINAL_PATH
```

**Verify inside Claude**

```
run bash -lc 'which -a git; type -a git; git --version'
```

---

### Phase 3 — Custom Rust shell (optional, for maximal control)

**Why:** When you want to own the UX (prompts, rich hints), ensure built-ins and pipelines are traced, and enforce policies.

**Core pieces**

- PTY management (`nix`/`libc` on Unix): allocate PTY, spawn `/bin/bash` as a child attached to PTY (not shimmed), feed it commands you parse or pass through.
- Line editor: `rustyline` or `reedline`.
- Built-ins: implement `cd`, `export`, `set`, `alias` in Rust; log them directly (JSON).
- Non-built-ins: delegate to child `/bin/bash -lc "...";` so expansions/pipelines work; your **shims** capture sub-processes, `BASH_ENV` captures built-ins (via `set -x`) if you want redundancy.
- Signals: forward SIGINT/SIGTERM/resize to PTY child.
- Policies: optional “allow/deny” filters, confirmations, dry-run.

**Tracing in shell**

- Emit a `session_id` once per shell start; include it in every JSON log from the shell layer and the shim (propagate via env).
- For TUI apps: run them in the PTY; output flows normally; you still get shim logs when they spawn child processes.

---

## Safety & rollback

- Never shim shells.
- Keep a “panic switch”:

  ```bash
  mv ~/.cmdshim_rust ~/.cmdshim_rust.DISABLED.$(date +%s)
  sed -i '' '/\.cmdshim_rust/d' ~/.bashrc ~/.bash_profile ~/.zshrc ~/.zprofile 2>/dev/null || true
  ```

- If files keep getting replaced, set immutability (macOS):

  ```bash
  chflags uchg ~/.cmdshim_rust/.shimbin ~/.cmdshim_rust/*
  # undo with: chflags nouchg …
  ```

---

## Testing matrix

- **Lookup correctness**

  - `which -a git` shows shim first; `type -a git` too (clear `hash -r`).
  - Claude `run`: `bash -lc 'which -a git; type -a git'` (with `BASH_ENV`).

- **Core dev tools**: `git`, `node`, `npm`, `python`, `pip`, `docker`—all log with correct `cwd` and exit codes.
- **Core utils (when enabled)**: `awk --version`, `grep --version`, `sed --version`; ensure agent startup (nvm) still works.
- **Pipelines**: `git status | grep foo` — see both shim logs (git/grep) + optional xtrace for the pipeline line.
- **TUIs**: `htop`/`fzf`/`npm init -y`/`python -i` — ensure no freeze; logs still record sub-spawns.
- **Performance**: measure average shim overhead (`duration_ms`), target < 2–5 ms per exec on macOS.

---

## Cross-platform notes

- macOS: `path_helper` affects login shells; prefer `BASH_ENV` for non-interactive.
- Linux: simpler startup; same `BASH_ENV` trick works.
- Windows (later): use a wrapper for `CreateProcess` and `%PATH%` resolution; PowerShell built-ins tracing is separate (script block logging).

---

## Deliverables checklist

- [ ] `crates/shim/` (Rust binary) with clean resolver, skip-shimdir, exec, JSONL logging.
- [ ] `scripts/stage_shims.sh` to install `.shimbin` and stubs (real files).
- [ ] `~/.claude_bashenv` to inject PATH + pins + optional xtrace.
- [ ] (Optional) `crates/supervisor/` to launch targets with deduped PATH and `BASH_ENV`.
- [ ] (Optional) `crates/shell/` REPL + PTY + built-ins + session_id propagation.
- [ ] `docs/ops.md` with rollback steps, safety notes, perf tips.

---

## Minimal “today” commands

```bash
# Build & stage shims
cargo build -p shim --release
mkdir -p ~/.cmdshim_rust
install -m 0755 target/release/shim ~/.cmdshim_rust/.shimbin
for c in git npm npx node pnpm bun python python3 pip pip3 jq curl wget tar unzip make go cargo deno docker kubectl; do
  install -m 0755 ~/.cmdshim_rust/.shimbin ~/.cmdshim_rust/$c
done

# Session env
export ORIGINAL_PATH="$HOME/.nvm/versions/node/v22.16.0/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.bun/bin"
export PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
hash -r

# Claude with per-shell injection
cat > ~/.claude_bashenv <<'EOF'
export ORIGINAL_PATH="$HOME/.nvm/versions/node/v22.16.0/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.bun/bin"
PATH="$HOME/.cmdshim_rust:$ORIGINAL_PATH"
hash -r
for c in git npm npx node pnpm bun python python3 pip pip3 jq curl wget tar unzip make go cargo deno docker kubectl; do
  [ -x "$HOME/.cmdshim_rust/$c" ] && hash -p "$HOME/.cmdshim_rust/$c" "$c"
done
: "${TRACE_SHELL:=$HOME/.trace_shell.jsonl}"
exec {__xtrace_fd}>>"$TRACE_SHELL"; BASH_XTRACEFD=$__xtrace_fd
PS4='+CMD ${EPOCHREALTIME} ${BASH_SOURCE##*/}:${LINENO}: '; set -o xtrace
EOF

BASH_ENV="$HOME/.claude_bashenv" claude code
```

That gives you **full traceability in Claude** (shim logs + optional built-in traces), with no shell recursion and minimal friction. When you’re ready, drop in the optional supervisor and then the Rust shell for even tighter control. If you want, I can draft the shim’s final `resolve_real_binary()` and POSIX `exec` code next.
