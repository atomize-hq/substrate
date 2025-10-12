# Phase 4.5 — Always-in-World Execution Plan (Linux-first)

Purpose: ensure that when running the `substrate` CLI, command execution is always inside an isolated “world” on Linux, align replay with world‑api types, and standardize the netfilter log prefix across docs and implementation. This document provides a phased plan with concrete steps, code sketches, and acceptance tests.

Status: Proposed plan
Owner: Substrate Core
Timebox: 5–8 working days (Phase 1–2); PTY integration may extend to 8–12

Non‑Goals
- Optional FsDiff small‑file hashing and content privacy filters (deferred).

## High‑Level Objectives
- Shell always ensures a session world on Linux and runs commands in that world (or captures post‑hoc fs_diff) with clear fallbacks elsewhere.
- Replay uses world‑api `ExecRequest`/`ExecResult` for isolation semantics and telemetry alignment.
- Netfilter LOG prefix standardized and docs in sync with implementation.

## Terminology
- “World”: Linux world backed by namespaces/cgroups and optional overlay/fuse copy‑on‑write isolation.
- “Observe‑only”: Print helpful warnings and proceed without isolation when unsupported/unprivileged; still trace spans.

---

## Phase 1 — Always Ensure World in Shell (Default‑On) + Prefix Alignment (docs)

Scope
- Linux‑only shell path: ensure session world at startup by default (no installer env required). Set `SUBSTRATE_WORLD_ID`. Route non‑PTY execs through world backend and pass `span_id` so `fs_diff` is immediate. Keep PTY integration for Phase 1B (below).
- Decide and document netfilter log prefix to keep code+docs aligned.

Changes
1) Shell initialization: default‑on world, ensure session, set env
   - File: `crates/shell/src/lib.rs`
   - When running on Linux, at startup (inside `run_shell()`), unless explicitly disabled (CLI `--no-world` or `SUBSTRATE_WORLD=disabled`), build a `world_api::WorldSpec` from policy + CWD and ensure a session via `world::LinuxLocalBackend::ensure_session(&spec)`. Set `SUBSTRATE_WORLD_ID` env var with the returned handle. Internally treat absence of `SUBSTRATE_WORLD` as enabled.
   - Keep graceful fallback (warnings) when ensure fails.

   Code sketch:
   ```rust
   // near start of run_shell(), after init_trace and shims setup
   #[cfg(target_os = "linux")]
   {
       use world::{LinuxLocalBackend};
       use world_api::{WorldBackend, WorldSpec, ResourceLimits};
       let mut world_id_opt: Option<String> = None;
       let world_disabled = std::env::var("SUBSTRATE_WORLD").map(|v| v=="disabled").unwrap_or(false)
           || std::env::args().any(|a| a=="--no-world");
       if !world_disabled {
           let spec = WorldSpec {
           reuse_session: true,
           isolate_network: true,
           limits: ResourceLimits::default(),
           enable_preload: false,
           allowed_domains: substrate_broker::allowed_domains(),
           project_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
           };
           let backend = LinuxLocalBackend::new();
           match backend.ensure_session(&spec) {
               Ok(handle) => {
                   std::env::set_var("SUBSTRATE_WORLD", "enabled");
                   std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                   world_id_opt = Some(handle.id);
               }
               Err(e) => {
                   eprintln!("substrate: warn: world isolation unavailable (observe-only): {}", e);
               }
           }
       }
   }
   ```

2) Route non‑PTY to world backend and pass span_id
   - File: `crates/shell/src/lib.rs`
   - In `execute_command(...)` when not using PTY, create `ActiveSpan` first, then build `ExecRequest { cmd: <redacted/raw>, cwd, env, pty: false, span_id: Some(active_span.get_span_id()) }`, call `backend.exec(&handle, req)`, and use the returned `ExecResult { exit, stdout/stderr, scopes_used, fs_diff }` to complete the span.
   - Preserve current shell‑vs‑no‑shell logic by passing the same `trimmed` command string to `sh -c` on the world side (the Linux backend already does that within overlay path). For a minimal change, leverage the existing `SessionWorld::execute` behavior.

   Code sketch (high‑level, eliding surrounding logic):
   ```rust
   // inside execute_command(), after we construct `span` via create_span_builder()
   #[cfg(target_os = "linux")]
   if std::env::var("SUBSTRATE_WORLD_ID").is_ok() {
       use world::{LinuxLocalBackend};
       use world_api::{WorldBackend, ExecRequest};
       let backend = LinuxLocalBackend::new();
       let handle = world_api::WorldHandle { id: std::env::var("SUBSTRATE_WORLD_ID").unwrap() };
       let req = ExecRequest {
           cmd: trimmed.to_string(),
           cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
           env: std::env::vars().collect(),
           pty: false,
           span_id: span.as_ref().map(|s| s.get_span_id()).or_else(|| None),
       };
       match backend.exec(&handle, req) {
           Ok(res) => {
               // print stdout/stderr similarly to current behavior when appropriate
               let exit_code = res.exit;
               let scopes_used = res.scopes_used;
               let fsd = res.fs_diff;
               if let Some(active_span) = span { let _ = active_span.finish(exit_code, scopes_used, fsd); }
               // convert to ExitStatus and return
           }
           Err(e) => {
               eprintln!("substrate: warn: world exec failed, running direct: {}", e);
               // fall back to existing external execution path, then collect_world_telemetry()
           }
       }
   }
   ```

   Notes:
   - Keep the current PTY path unchanged in Phase 1.
   - For direct fallback, retain `collect_world_telemetry(span_id)` to avoid losing fs_diff when the world side had stored it.

3) Prefer immediate `ExecResult.fs_diff` over post‑hoc fetch
   - File: `crates/shell/src/lib.rs`
   - When the world `exec` returns `fs_diff`, pass it into `active_span.finish(...)` directly, bypassing `collect_world_telemetry`.
   - Retain `collect_world_telemetry` only when `ExecResult.fs_diff` is `None`.

4) Netfilter LOG prefix — standardize
   - Recommendation: keep `substrate-dropped-<WORLD_ID>:` (matches current code and parsers). Update docs and any plan references to “drop” to use “dropped”.
   - Action now: update docs only (this plan and future hardening docs) to say `substrate-dropped-<WORLD_ID>:`. Code alignment NOT required in Phase 1 since it already uses this form.

Acceptance (Phase 1)
- Linux host: running `substrate -c "echo hi"` ensures a world, sets `SUBSTRATE_WORLD_ID`, executes via backend, and records `fs_diff` in the completion span (empty diff allowed for read‑only commands).
- Non‑Linux: prints a single friendly notice and runs direct; no crashes.
- Trace JSONL entries contain `world_id` and, when applicable, a non‑empty `fs_diff`.
- Docs and any user output that reference the nftables LOG prefix show `substrate-dropped-<WORLD_ID>:`.

---

## Phase 1B — PTY Execution Inside World (Linux) aligned with Path A

Context
- See PRE_PHASE_4_5_HARDENING_PLAN.md section “Concurrent Output: Path A vs Path B” and future/PHASE_4_5_ADVANCED_FEATURES_PLAN.md Part A. We align with Path A (non‑polling renderer; Agent Hub later), while enabling in‑world PTY now without fighting that direction.

Scope
- Implement world‑agent PTY WebSocket stream (`/v1/stream`) as the transport for PTY sessions inside the world.
- Shell PTY path: on Linux with `SUBSTRATE_WORLD` enabled, route PTY commands to world‑agent stream with `span_id`, render using Reedline’s `suspend_guard` to avoid prompt corruption (matches Path A renderer philosophy). Later, the Agent Hub can subscribe/forward `PtyData` events without breaking this transport.
- Fallbacks: if agent/socket unavailable, warn and fallback to host PTY path, then attempt post‑hoc telemetry.

Changes
1) world-agent: implement `/v1/stream` WebSocket with PTY attach
   - Files: `crates/world-agent/src/pty.rs`, `handlers.rs`
   - Create/attach PTY inside world, spawn child shell (`sh -lc <cmd>`), forward I/O frames over WS, carry `span_id`.

2) shell: route PTY via agent
   - File: `crates/shell/src/lib.rs`
   - When `needs_pty()` and Linux & world enabled: open WS to agent, connect stdin/stdout with a prompt-safe printer guarded by `suspend_guard`, set `span_id` and on close finalize span (collect scopes/fs_diff via backend if not provided inline).

Acceptance (Phase 1B)
- Interactive commands run inside the world with correct terminal behavior.
- Idle CPU ~0% when idle; prompt not corrupted under bursts.
- Span completion includes `fs_diff` when applicable.

---

## Phase 2 — Replay Alignment with world‑api + Session Overlay Policy

Scope
- Replace bespoke replay isolation logic with world‑api alignment to consolidate world semantics.
- Add a `WorldSpec` flag to force overlay isolation for “always‑in‑world” replay (no heuristics).

Changes
1) Add `always_isolate: bool` to `WorldSpec` (Linux path only for now)
   - Files:
     - `crates/world-api/src/lib.rs`: add field with default false.
     - `crates/world/src/session.rs`: in `should_isolate_command`, return `true` when `self.spec.always_isolate` else use current heuristics.
   - Code sketch:
   ```rust
   // world-api/src/lib.rs
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct WorldSpec {
       pub reuse_session: bool,
       pub isolate_network: bool,
       pub limits: ResourceLimits,
       pub enable_preload: bool,
       pub allowed_domains: Vec<String>,
       pub project_dir: PathBuf,
       pub always_isolate: bool, // NEW
   }
   impl Default for WorldSpec { fn default() -> Self { Self { /*…*/ always_isolate: false, }}}

   // world/src/session.rs
   fn should_isolate_command(&self, _cmd: &str) -> bool {
       if self.spec.always_isolate { return true; }
       // existing patterns…
   }
   ```

2) Replay uses world‑api `ExecRequest`/`ExecResult`
   - File: `crates/replay/src/replay.rs`
   - Build a `WorldSpec` using policy‐derived allowed domains, `always_isolate: true`, and `project_dir = state.cwd`.
   - Ensure session and call `backend.exec` with `ExecRequest { cmd: format!("bash -lc '{}'", raw), cwd: state.cwd, env: state.env, pty: false, span_id: Some(state.span_id.clone()) }`.
   - Remove (or guard) bespoke cgroup/netns/nftables setup if duplication with world init is not desired.

   Code sketch:
   ```rust
   #[cfg(target_os = "linux")]
   {
       use world::{LinuxLocalBackend};
       use world_api::{WorldBackend, WorldSpec, ExecRequest, ResourceLimits};
       let spec = WorldSpec {
           reuse_session: true,
           isolate_network: true,
           limits: ResourceLimits::default(),
           enable_preload: false,
           allowed_domains: substrate_broker::allowed_domains(),
           project_dir: state.cwd.clone(),
           always_isolate: true,
       };
       let backend = LinuxLocalBackend::new();
       let handle = backend.ensure_session(&spec)?;
       let req = ExecRequest {
           cmd: format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''")),
           cwd: state.cwd.clone(),
           env: state.env.clone(),
           pty: false,
           span_id: Some(state.span_id.clone()),
       };
       let res = backend.exec(&world_api::WorldHandle { id: handle.id.clone() }, req)?;
       return Ok(ExecutionResult {
           exit_code: res.exit,
           stdout: res.stdout,
           stderr: res.stderr,
           fs_diff: res.fs_diff,
           scopes_used: res.scopes_used,
           duration_ms: start.elapsed().as_millis() as u64,
       });
   }
   ```

3) Update docs — Replay
   - Reflect that replay now goes through the same world backend and returns `fs_diff` via `ExecResult`.

Acceptance (Phase 2)
- `substrate --replay-verbose --replay <SPAN>` prints `strategy=overlay|fuse|copy-diff` from the backend and includes `fs_diff` and `scopes_used` aligned with shell semantics.
- Removing bespoke replay isolation code does not regress behavior; netns/cgroup limits are in effect through the world backend.

---

## Netfilter LOG Prefix Decision

Recommendation: keep `substrate-dropped-<WORLD_ID>:`
- Rationale: matches current implementation (`crates/world/src/netfilter.rs`) and parser (`parse_dropped_packets` looks for `substrate-dropped-...`). The suffix describes the event (“dropped”) and avoids bikeshedding.
- Actions:
  - Update any docs/plans to use `substrate-dropped-<WORLD_ID>:`.
  - If future consolidation is needed, keep a single constant in code and reference it from parsers and docs.

---

## Developer Notes & Pitfalls
- Set `SUBSTRATE_WORLD_ID` before spawning child processes so spans include world_id and post‑hoc telemetry works.
- When routing via world backend, prefer immediate `ExecResult.fs_diff` and avoid re‑reading via `fs_diff(span_id)` except as fallback.
- Keep non‑Linux behavior unchanged: print a single friendly notice and proceed direct.
- Do not regress PATH/shim semantics; the shell’s shim deployment remains intact.

---

## Validation Matrix
- Linux native host (root and non‑root):
  - shell non‑PTY commands get fs_diff through backend.
  - PTY commands (Phase 3) run via world‑agent.
  - netfilter logs show `substrate-dropped-<WORLD_ID>:` when egress is blocked.
- macOS/Windows: 
  - No isolation (observe‑only), no crashes, clear notices; replay still works in direct mode.

---

## Work Items (Checklist)

- [ ] Phase 1: ensure session world at shell startup; set `SUBSTRATE_WORLD_ID`.
- [ ] Phase 1: route non‑PTY through world backend with `span_id`, prefer `ExecResult.fs_diff`.
- [ ] Phase 1: update docs to standardize `substrate-dropped-<WORLD_ID>:` prefix.
- [ ] Phase 2: add `always_isolate` to `WorldSpec`; make session overlay unconditional when set.
- [ ] Phase 2: refactor replay to use `world_api::{WorldSpec, ExecRequest, ExecResult}`.
- [ ] Phase 3: wire PTY path to `world-agent` with `pty=true` and `span_id`.
- [ ] Phase 3: fallbacks and telemetry when agent not available.
- [ ] Tests: unit + privileged integration for overlay/netfilter; update docs/USAGE and REPLAY.

---

## Quick Commands (for local validation)

- Ensure environment
  - `export SUBSTRATE_WORLD=enabled`
  - `cargo build`

- Shell (non‑PTY)
  - `target/debug/substrate -c "echo hello > out.txt"`
  - Expect: world ensured on Linux; span contains `fs_diff.writes: ["out.txt"]`.

- Replay
  - Capture a span ID from `~/.substrate/trace.jsonl` (command_complete).
  - `target/debug/substrate --replay-verbose --replay "$SPAN"`
  - Expect: `fs_diff` present; verbose prints strategy; logs may include `substrate-dropped-<WORLD_ID>:` when egress blocked.

---

## Podman Testing (Incremental, Exact Commands)

Use the provided Podman scripts for a privileged Linux testbed. These steps are incremental to catch build/config issues early.

0) One‑time VM setup (host)
- `podman machine set --rootful`
- `podman machine stop; podman machine start`
- `podman system connection default podman-machine-default-root`
- Kernel prerequisites inside VM: `bash scripts/podman/setup-machine.sh`
- Quick validate (optional): `bash scripts/podman/validate.sh`

1) Build image (host)
- Default: `bash scripts/podman/build.sh`
- Force x86_64: `BUILD_ARCH=amd64 bash scripts/podman/build.sh`

2) Run container (host)
- `bash scripts/podman/run.sh` (mounts repo at `/src`, starts privileged container)

3) Inside container — prereqs and build
- `bash scripts/check-container-prereqs.sh`
- `cargo build`
- Doctor: `target/debug/substrate world doctor --json && echo OK || echo FAIL`

4) Inside container — unit/privileged tests
- `RUST_LOG=info cargo test -p world -- --nocapture`
- Optional (root‑gated): `RUST_LOG=info cargo test -p world -- --nocapture test_nftables_rules`

5) Phase 1A check — default‑on world non‑PTY
- `mkdir -p /tmp/p1 && cd /tmp/p1`
- `target/debug/substrate -c "bash -lc 'echo data > out.txt'"`
- `tail -n 200 ~/.substrate/trace.jsonl | jq -r 'select(.event_type=="command_complete") | [.span_id, (.fs_diff.writes|tostring)] | @tsv' | tail -n1`
- Expect last line shows span id and writes containing `out.txt`.

6) Netfilter prefix sanity
- Blocked egress example and dmesg check:
  - `target/debug/substrate -c "bash -lc 'curl -m2 http://example.com || true'"`
  - `SPAN=$(tail -n 200 ~/.substrate/trace.jsonl | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n1)`
  - `export SUBSTRATE_REPLAY_USE_WORLD=1`
  - `target/debug/substrate --replay-verbose --replay "$SPAN"`
  - `dmesg -T | rg 'substrate-dropped-' | tail -n5 || true`
- Expect prefix `substrate-dropped-<WORLD_ID>:` when nft/dmesg are active; warn and proceed when constrained.

7) Replay alignment (after Phase 2)
- `mkdir -p /tmp/p2 && cd /tmp/p2 && target/debug/substrate -c "bash -lc 'mkdir demo && echo x > demo/a.txt'"`
- `SPAN=$(tail -n 200 ~/.substrate/trace.jsonl | jq -r 'select(.event_type=="command_complete") | .span_id' | tail -n1)`
- `target/debug/substrate --replay-verbose --replay "$SPAN"`
- Expect printed strategy (overlay|fuse|copy-diff) and non‑empty fs_diff including `demo/` and `demo/a.txt`.

Notes
- Container is privileged with `/dev/fuse` and required CAPs; overlay/netns/nft tests should work.
- If LOGs don’t appear due to missing route in netns, this is acceptable for isolation validation. The presence of warnings proves graceful fallbacks.

---

## Implementation Details (Agent‑Executable)

This appendix enumerates the precise file touchpoints, concrete code sketches, message protocols, and test guidance to enable a smooth implementation.

### A) CLI: Default‑On World + Opt‑Out Lever

- File: `crates/shell/src/lib.rs`
- Add flag to `Cli`:
```rust
#[derive(Parser, Debug)]
pub struct Cli {
    // ...existing args
    /// Disable world isolation (Linux only)
    #[arg(long = "no-world")]
    pub no_world: bool,
}
```
- In `run_shell()`, compute disabled state and ensure session world (Linux only):
```rust
#[cfg(target_os = "linux")]
{
    use world::{LinuxLocalBackend};
    use world_api::{WorldBackend, WorldSpec, ResourceLimits};
    let world_disabled = std::env::var("SUBSTRATE_WORLD").map(|v| v=="disabled").unwrap_or(false)
        || Cli::parse_from(std::env::args()).no_world; // or plumb `cli` in
    if !world_disabled {
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            // Phase 2 adds always_isolate
            ..WorldSpec::default()
        };
        let backend = LinuxLocalBackend::new();
        if let Ok(handle) = backend.ensure_session(&spec) {
            std::env::set_var("SUBSTRATE_WORLD", "enabled");
            std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
        } else {
            eprintln!("substrate: warn: world isolation unavailable (observe-only)");
        }
    }
}
```

CLI plumbing note (no double-parse)
- Prefer not to call `Cli::parse_from` again inside `run_shell()`. Instead:
  - Add `no_world: bool` to `Cli` and propagate it into `ShellConfig` (e.g., `ShellConfig { no_world: cli.no_world, .. }`).
  - Read `config.no_world` inside `run_shell()` to determine `world_disabled`.
  - This prevents surprising behavior if arguments are modified by wrappers or tests.

### B) Non‑PTY Routing Through World Backend

- File: `crates/shell/src/lib.rs` (`execute_command`)
- Create span first, then if Linux and `SUBSTRATE_WORLD_ID` set, route:
```rust
#[cfg(target_os = "linux")]
if std::env::var("SUBSTRATE_WORLD_ID").is_ok() {
    use world::{LinuxLocalBackend};
    use world_api::{WorldBackend, ExecRequest};
    let backend = LinuxLocalBackend::new();
    let handle = world_api::WorldHandle { id: std::env::var("SUBSTRATE_WORLD_ID").unwrap() };
    let req = ExecRequest {
        cmd: trimmed.to_string(),
        cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        env: std::env::vars().collect(),
        pty: false,
        span_id: span.as_ref().map(|s| s.get_span_id()),
    };
    match backend.exec(&handle, req) {
        Ok(res) => {
            use std::io::{self, Write};
            let _ = io::stdout().write_all(&res.stdout);
            let _ = io::stderr().write_all(&res.stderr);
            let exit_code = res.exit;
            let scopes_used = res.scopes_used;
            let fsd = res.fs_diff;
            if let Some(active_span) = span { let _ = active_span.finish(exit_code, scopes_used, fsd); }
            #[cfg(unix)] { use std::os::unix::process::ExitStatusExt; return Ok(std::process::ExitStatus::from_raw((exit_code & 0xff) << 8)); }
            #[cfg(windows)] { use std::os::windows::process::ExitStatusExt; return Ok(std::process::ExitStatus::from_raw(exit_code as u32)); }
        }
        Err(e) => {
            eprintln!("substrate: warn: world exec failed, running direct: {}", e);
            // fall back to existing path
        }
    }
}
```

### C) PTY World Execution via WebSocket (Phase 1B)

Message Protocol (text JSON frames)
- Client → Server
  - `{"type":"start","cmd":"bash -lc '<raw>'","cwd":"/path","env":{...},"span_id":"spn_...","cols":<u16>,"rows":<u16>}`
  - `{"type":"stdin","data_b64":"..."}` (base64 bytes)
  - `{"type":"resize","cols":<u16>,"rows":<u16>}`
  - `{"type":"signal","sig":"INT"}`
- Server → Client
  - `{"type":"stdout","data_b64":"..."}`
  - `{"type":"exit","code":0}`
  - `{"type":"error","message":"..."}`

Agent: `crates/world-agent/src/pty.rs`
```rust
pub async fn handle_ws_pty(ws: WebSocket) -> anyhow::Result<()> {
    use axum::extract::ws::{Message};
    use futures::{StreamExt, SinkExt};
    use portable_pty::*;
    let (mut tx, mut rx) = ws.split();
    // Wait for start
    let mut cmd = String::new();
    let mut cols = 80u16; let mut rows = 24u16;
    // read first message, parse JSON for cmd/cwd/env/span_id/size
    if let Some(Ok(Message::Text(s))) = rx.next().await {
        let v: serde_json::Value = serde_json::from_str(&s)?;
        cmd = v["cmd"].as_str().unwrap_or("sh -lc true").to_string();
        cols = v["cols"].as_u64().unwrap_or(80) as u16;
        rows = v["rows"].as_u64().unwrap_or(24) as u16;
        // TODO: cwd/env/span_id
    }
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
    let mut cmd_builder = CommandBuilder::new("sh");
    cmd_builder.args(["-lc", &cmd]);
    let mut child = pair.slave.spawn_command(cmd_builder)?;
    drop(pair.slave);
    // Reader task: forward stdout as base64
    let mut reader = pair.master.try_clone_reader()?;
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buf = [0u8; 8192];
        let mut writer = tx;
        let mut async_reader = tokio::io::unix::AsyncFd::new(reader).unwrap();
        loop {
            // simplified; real impl should use AsyncRead on a pipe
            break;
        }
    });
    // Main loop: handle stdin/resize
    while let Some(msg) = rx.next().await { /* ... */ }
    // On exit
    let status = child.wait()?;
    // send exit frame
    Ok(())
}
```

Shell PTY route (summary)
- In PTY branch, when Linux and `SUBSTRATE_WORLD` enabled:
  - Open WS `unix://run/substrate.sock/v1/stream`
  - Send `start` frame with `cmd`, `cwd`, `env`, `span_id`, terminal size.
  - Forward user input to `stdin` frames, forward server `stdout` frames to printer guarded by `reedline::suspend_guard()`.
  - On `exit` frame, finalize span and fetch fs_diff via backend if needed.

Agent lifecycle & client stack (Phase 1B)
- Ensuring world-agent is running:
  - If `/run/substrate.sock` is absent, the shell should attempt to spawn the agent binary (e.g., `target/debug/world-agent` in dev or `substrate-world-agent` if installed).
  - Wait for readiness: poll Unix socket existence and attempt an HTTP GET to `/v1/capabilities` over the Unix socket (hyperlocal) with a timeout (e.g., 1s). If not ready in time, print a warning and fall back to host PTY path for this session.
  - On shell shutdown, best-effort terminate the agent child process.
- Server dependency:
  - Ensure `axum` in world-agent has the `ws` feature enabled in Cargo.toml: `axum = { version = "*", features = ["ws"] }`.
- Client dependencies (shell):
  - Add `tokio-tungstenite` for WebSocket client.
  - Use `hyperlocal` or `uds` connector to build a WebSocket stream over a Unix domain socket.
  - Required crates: `tokio`, `tokio-tungstenite`, `hyperlocal`, `futures`.
  - Keep implementation minimal and aligned with Path A; the future Agent Hub can subscribe/bridge PtyData without changing this transport.

### D) Replay World‑API Alignment (Phase 2)

- File: `crates/world-api/src/lib.rs`: add `always_isolate: bool` + default.
- File: `crates/world/src/session.rs`: honor `spec.always_isolate` in `should_isolate_command`.
- File: `crates/replay/src/replay.rs`: replace bespoke isolation with `ensure_session` + `exec` path shown earlier.

### E) Tests & CI

- Unit tests
  - shell: flag parsing for `--no-world`, routing guard behavior when SUBSTRATE_WORLD_ID set/unset.
  - world-api: `WorldSpec` default + new field.
  - world: `should_isolate_command` respects `always_isolate`.
  - netfilter: existing nft rules tests (root-gated).
- Privileged integration tests (Linux‑only)
  - overlay mount/unmount (skips when unsupported), copy‑diff fallback path.
  - world-agent PTY: WS round‑trip loopback test (spawn echo under PTY, assert output).
- CI notes
  - Provide a GitHub Actions workflow segment (privileged Linux) mirroring current test gating patterns.
