# Always-In-World PTY Execution (Linux) — End-to-End Implementation Guide

Purpose
- Make all substrate shell commands run “in the world” by default on Linux — both non-PTY and PTY — while keeping transport and protocol stable, preserving prompt safety, and maintaining graceful fallbacks.
- This document is self-contained. Follow it top-to-bottom to implement the remaining work with no additional context.

Current State (baseline)
- Non-PTY: Already routed through the world backend with `span_id` and immediate `fs_diff`.
- PTY over WS: Implemented client/server; raw stdin, resize (SIGWINCH), readiness probe, and signal forwarding (INT/TERM/HUP/QUIT) are working.
- REPL PTY: Routes through the same `execute_command()` path (protected by `reedline::suspend_guard()`), so when the world is enabled it uses the WS PTY path with the same fallbacks.
- Remaining work: Ensure the PTY child process runs “inside the world session” (namespaces + cgroup) on the agent.

Key Constraints
- Keep the JSON protocol and WS transport unchanged.
- Keep `world_api` trait stable in this phase (no new streaming PTY trait).
- Maintain graceful fallbacks and exactly one warning on fallback.

Repository Structure (relevant parts)
- `crates/shell/` — substrate shell (routing, WS client, prompt safety)
- `crates/world-agent/` — agent providing `/v1/execute` and `/v1/stream`
- `crates/world/` — Linux “world” backend (session, netfilter, cgroups)
- `crates/world-api/` — API types and trait (`WorldSpec`, `ExecRequest`, `ExecResult`, `WorldBackend`)
- `crates/broker/` — policy engine; provides `allowed_domains()` to scope egress

Environment & Preflight (Linux)
- Requirements:
  - Kernel with cgroup v2 and user namespaces (best-effort)
  - `iproute2` (`ip` command) for named netns
  - `nftables` (`nft` command) for network filtering
  - Build toolchain: Rust stable, cargo
- Install (Debian/Ubuntu):
  - `apt-get update && apt-get install -y iproute2 nftables`
- Sanity checks:
  - `command -v ip && command -v nft` should both be present
  - `/sys/fs/cgroup/cgroup.controllers` should exist (otherwise cgroup attach will be skipped with a warning)

Message Protocol (reference)
- Client → Server:
  - `{"type":"start","cmd":"bash -lc '<raw>'","cwd":"/path","env":{...},"span_id":"spn_...","cols":<u16>,"rows":<u16>}`
  - `{"type":"stdin","data_b64":"..."}`
  - `{"type":"resize","cols":<u16>,"rows":<u16>}`
  - `{"type":"signal","sig":"INT|TERM|HUP|QUIT"}`
- Server → Client:
  - `{"type":"stdout","data_b64":"..."}`
  - `{"type":"exit","code":0}`
  - `{"type":"error","message":"..."}`

Design Choice (Phase 1B)
- For PTY sessions: use a named network namespace per world and attach the PTY child to the world’s cgroup. Avoid `unshare()`/`pivot_root` in the agent process. Non-PTY overlayfs/isolation remains as-is via backend.exec.

Implementation Steps (copy-paste patches)

1) world: Create/record a named netns per SessionWorld and scope NetFilter to it (Linux)
- File: `crates/world/src/session.rs`
- Goal: Create `substrate-<WORLD_ID>` netns, bring `lo` up, and scope NetFilter to that netns. Avoid calling heavy unshare/pivot_root in the agent path.

Patch A — add lightweight netns setup during SessionWorld::setup()
```rust
// crates/world/src/session.rs (inside impl SessionWorld)
// Add near the end of setup() after create_directories():
let ns_name = format!("substrate-{}", self.id);
if crate::netns::NetNs::ip_available() {
    let mut ns = crate::netns::NetNs::new(&ns_name);
    if ns.add().is_ok() {
        let _ = ns.lo_up();
        self.net_namespace = Some(ns_name.clone());
    }
}

// Initialize NetFilter scoped to the netns (if available)
let mut filter = crate::netfilter::NetFilter::new(&self.id, self.spec.allowed_domains.clone())?;
filter.set_namespace(self.net_namespace.clone());
filter.resolve_domains()?;
filter.install_rules()?;
self.network_filter = Some(filter);
```

Patch B — make setup_linux_isolation() lightweight
```rust
// crates/world/src/session.rs
#[cfg(target_os = "linux")]
fn setup_linux_isolation(&self) -> Result<()> {
    // Lightweight for PTY phase: avoid unsharing or pivot_root in the caller process.
    // Non-PTY overlayfs remains handled by overlayfs::execute_with_overlay().
    Ok(())
}
```

Notes
- We intentionally avoid `LinuxIsolation::apply()` here to prevent mutating the agent process namespaces. This keeps the agent stable while still giving PTY children a netns to enter.
- On failure (missing `ip` or privileges), `self.net_namespace` remains `None` and NetFilter is installed in the default namespace (best-effort). Continue gracefully.

2) world-agent: Run PTY child inside world session (netns + cgroup)
- Files:
  - `crates/world-agent/src/handlers.rs`
  - `crates/world-agent/src/pty.rs`

Patch C — stream handler passes WorldAgentService to PTY
```rust
// crates/world-agent/src/handlers.rs
use crate::service::WorldAgentService;
use axum::extract::{State};

pub async fn stream(
    State(service): State<WorldAgentService>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| async move {
        crate::pty::handle_ws_pty(service, socket).await;
    })
}
```

Router (already correct):
```rust
// crates/world-agent/src/main.rs (excerpt)
let app = Router::new()
    .route("/v1/capabilities", get(handlers::capabilities))
    .route("/v1/execute", post(handlers::execute))
    .route("/v1/stream", get(handlers::stream))
    .route("/v1/trace/:span_id", get(handlers::get_trace))
    .route("/v1/request_scopes", post(handlers::request_scopes))
    .with_state(service);
```

Patch D — PTY handler: ensure session; ip netns exec; cgroup attach
```rust
// crates/world-agent/src/pty.rs
use crate::service::WorldAgentService;
use axum::extract::ws::WebSocket;

pub async fn handle_ws_pty(service: WorldAgentService, ws: WebSocket) {
    // ... parse ClientMessage::Start { cmd, cwd, env, span_id, cols, rows } as before

    // Ensure session world with same shape as /v1/execute
    let spec = world_api::WorldSpec {
        reuse_session: true,
        isolate_network: true,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: substrate_broker::allowed_domains(),
        project_dir: cwd.clone(),
    };
    let world = match service.backend.ensure_session(&spec) {
        Ok(w) => w,
        Err(e) => {
            tracing::info!(error = %e, "ws_pty: ensure_session failed; running non-isolated");
            // Continue with non-isolated PTY below (keep transport)
            world_api::WorldHandle { id: "-".to_string() }
        }
    };

    let ns_name = format!("substrate-{}", world.id);
    let ns_path = format!("/var/run/netns/{}", ns_name);
    let cgroup_path = std::path::PathBuf::from("/sys/fs/cgroup/substrate").join(&world.id);

    // Prepare command under netns when available
    let mut cmd_builder = portable_pty::CommandBuilder::new("sh");
    if std::path::Path::new(&ns_path).exists() {
        cmd_builder = portable_pty::CommandBuilder::new("ip");
        cmd_builder.args(["netns", "exec", &ns_name, "sh", "-lc", &cmd]);
    } else {
        cmd_builder.args(["-lc", &cmd]);
    }
    cmd_builder.cwd(cwd.clone());
    for (k, v) in env.iter() { cmd_builder.env(k, v); }

    // Create PTY and spawn
    let pty_system = portable_pty::native_pty_system();
    let pair = match pty_system.openpty(portable_pty::PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.lock().await.send(Message::Text(serde_json::to_string(&ServerMessage::Error{message: format!("Failed to create PTY: {}", e)}).unwrap())).await;
            return;
        }
    };
    let mut child = match pair.slave.spawn_command(cmd_builder) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.lock().await.send(Message::Text(serde_json::to_string(&ServerMessage::Error{message: format!("Failed to spawn command: {}", e)}).unwrap())).await;
            return;
        }
    };
    drop(pair.slave);

    // Attach child to world cgroup (best-effort)
    if let Some(pid) = child.process_id() {
        let _ = std::fs::create_dir_all(&cgroup_path);
        let _ = std::fs::write(cgroup_path.join("cgroup.procs"), pid.to_string());
    }

    tracing::info!(
        world_id = %world.id,
        ns = %ns_name,
        cgroup = %cgroup_path.display(),
        cols = cols, rows = rows,
        "ws_pty: start (in_world=true)"
    );

    // ... keep reader/writer, resize, and signal forwarding loops unchanged
}
```

Notes
- Signal forwarding is already implemented: client sends `Signal` frames; server delivers via `kill(pid, signo)`.
- Resize events via SIGWINCH → `resize` frames are already implemented.
- If both netns and cgroup attach fail, we still run the PTY transport without isolation.

3) Shell (no code changes in this phase)
- Non-PTY remains routed via `world_api::WorldBackend::exec` and finishes span with immediate `fs_diff`.
- PTY (REPL and `--pty -c`) uses the WS PTY path when world is enabled or socket exists; prompt protected by `reedline::suspend_guard()`.
- Debug toggle: `SUBSTRATE_WS_DEBUG=1` prints “using world-agent PTY WS” upon connect.

Validation (copy/paste)
1. Build
- `cargo build`
- `cargo build -p world-agent`

2. Start agent (or rely on auto-spawn)
- `RUST_LOG=info target/debug/world-agent &`
- `ls -l /run/substrate.sock`

3. Non-interactive WS PTY
- `target/debug/substrate --pty -c "bash -lc 'echo hello && sleep 1 && echo done'"`
- Expect: single echo; exit code 0. Agent logs include `ws_pty: start (in_world=true)` with `world_id`, `ns`, `cgroup`.

4. Interactive WS PTY
- `SUBSTRATE_WS_DEBUG=1 target/debug/substrate --pty -c "bash -lc 'stty -a; echo type; read x; echo X:\$x'"`
- Type and press Enter → expect `X:<text>`. Resize terminal and confirm `stty -a` reflects updated cols/rows.

5. Signals
- In another shell: `pkill -INT substrate` (or `kill -INT <substrate-pid>`) while a WS PTY command is running
- Expect: remote child receives SIGINT (e.g., breaks `sleep`); agent logs signal forwarded.

6. Fallback
- Stop agent: `pkill world-agent`
- Rerun the command → exactly one warning; host PTY execution proceeds.

7. Non-PTY (world backend)
- `target/debug/substrate -c "echo hello > out.txt"`
- Expect: span completed with immediate `fs_diff` containing `out.txt`.

Verify isolation artifacts
- Netns exists: `ip netns list | grep substrate-`
- Inspect inside netns (root): `ip netns exec substrate-<WORLD_ID> ip a`
- Cgroup attach: `cat /sys/fs/cgroup/substrate/<WORLD_ID>/cgroup.procs | grep <child-pid>`

Naming & Conventions
- World ID: `wld_<UUIDv7>`
- Named netns: `substrate-<WORLD_ID>` (e.g., `substrate-wld_...`)
- Netfilter LOG prefix: `substrate-dropped-<WORLD_ID>:`
- UDS path: `/run/substrate.sock`
- Env toggles:
  - `SUBSTRATE_WORLD=enabled|disabled`
  - `SUBSTRATE_WORLD_ID`
  - `SUBSTRATE_WS_DEBUG=1`

Troubleshooting
- ip/nft missing: Install `iproute2` and `nftables`; log will show best-effort warnings.
- cgroup v2 missing: `/sys/fs/cgroup/cgroup.controllers` not present → cgroup attach skipped; PTY still runs.
- WS handshake fails: Shell prints exactly one warning and falls back to host PTY.
- Double echo: Ensure raw-mode guard is active (already implemented); verify you’re on the WS PTY path (set `SUBSTRATE_WS_DEBUG=1`).

Security Notes
- Signal forwarding: only forwards OS-delivered signals (INT/TERM/HUP/QUIT). User-typed `^C` in the terminal is already delivered by the PTY as 0x03 and handled by the remote shell.
- Netfilter rules are scoped to the named netns when available; otherwise installed globally (best-effort). LOG prefix remains `substrate-dropped-<WORLD_ID>:`.

File Touchpoints Summary
- world (Linux)
  - `crates/world/src/session.rs`: add named netns creation; scope `NetFilter` to netns; lighten `setup_linux_isolation()`.
- world-agent
  - `crates/world-agent/src/handlers.rs`: pass `State(WorldAgentService)` into `stream` handler.
  - `crates/world-agent/src/pty.rs`: ensure session; spawn PTY child via `ip netns exec`; attach PID to cgroup; log `in_world=true`.
- shell
  - Already aligned (routing + prompt safety + readiness + signals + resize).

Checklist
- [ ] SessionWorld creates/records named netns and scopes NetFilter to it.
- [ ] `setup_linux_isolation()` is lightweight (no unshare/pivot_root in agent path).
- [ ] Agent WS PTY spawns under `ip netns exec` when available and attaches child to world cgroup.
- [ ] Logging includes `in_world=true`, `world_id`, `ns`, `cgroup` at PTY start.
- [ ] Fallbacks produce exactly one warning; host PTY still works.
- [ ] Non-PTY path unchanged and continues to provide immediate `fs_diff`.

