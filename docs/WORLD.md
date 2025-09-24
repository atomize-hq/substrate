# Substrate World: Architecture, Behavior, and Operations (Linux & macOS)

This document describes the “world” execution model in Substrate — what it is, how the shell and world‑agent cooperate, isolation boundaries (netns/cgroup/overlay), protocols, environment toggles, validation, troubleshooting, and planned housekeeping (netns GC).

Status: Linux and macOS default to “always‑in‑world” execution. Windows degrades gracefully.

---

## 1) Overview

A “world” is a reusable Linux execution context providing:
- Process isolation and resource controls (per‑world cgroup v2)
- Network scoping and logging (nftables in a per‑world named netns)
- Filesystem isolation for non‑PTY execs (overlay/copy‑diff)
- A stable API surface for the Substrate shell and agents

Two cooperating components:
- `substrate` (shell): orchestrates execution, tracing, and routing (non‑PTY via REST, PTY via WS)
- `world-agent`: runs inside the target Linux environment and exposes a small API over a Unix domain socket (`/run/substrate.sock`)

On Linux the agent runs directly on the host. On macOS the agent runs inside a Lima VM; the shell ensures transport forwarding (VSock → SSH UDS → SSH TCP) back to the guest socket.

Helper scripts (`scripts/mac/lima-*.sh`, `scripts/mac/smoke.sh`) keep the Lima environment reproducible.

`/tmp` is included in the guest unit’s `ReadWritePaths` list so replay and shim flows can surface temp‑file diffs on both platforms. The provisioning script embeds this setting in the unit automatically—no manual tuning needed.

---

## 2) Execution Paths (Linux & macOS)

Always‑on by default (unless disabled via `SUBSTRATE_WORLD=disabled`):

- Non‑PTY commands
  - Shell POSTs to `world-agent /v1/execute` over a Unix socket (Linux) or forwarded socket (macOS). The shell auto‑starts or ensures availability before sending the request.
  - Response returns `exit`, `stdout_b64`, `stderr_b64`, `scopes_used`, and `fs_diff` (when available). The shell attaches `fs_diff` to the span on both platforms.

- PTY commands (interactive/TUIs)
  - Shell connects a WebSocket to `world-agent /v1/stream` over the active transport.
  - The agent allocates a PTY and spawns the child inside the session world, forwarding I/O, window resize, and signals.

- REPL integration
  - The Substrate REPL can force PTY per‑line using the `:pty ` prefix. The shell strips this prefix before sending the command to the agent.

- Fallback
  - If the agent/socket is unavailable or a transport handshake fails, the shell prints exactly one warning and runs on the host path for that command. Subsequent commands continue to attempt world routing.

Windows currently degrades to host execution with a friendly notice.

---

## 3) macOS Architecture (Lima)

Substrate on macOS uses a Lima VM (“substrate”) to host the world-agent. The shell guarantees the VM, agent, and forwarding layer are ready before routing commands.

- Provisioning & lifecycle
  - `scripts/mac/lima-warm.sh` starts or creates the VM from `docs/dev/lima/substrate.yaml`, installs required packages, and ensures the systemd unit writes to `/run/substrate.sock` with `/tmp` included in `ReadWritePaths`.
  - `scripts/mac/lima-stop.sh` shuts the VM down cleanly; `scripts/mac/lima-doctor.sh` reports health (virtualization, agent socket, service status, forwarding tools).
  - The helper scripts substitute the active project path so `/src` inside the VM mirrors the host repo checkout.

- Transport selection (host ⇄ guest)
  1. VSock via `vsock-proxy` (preferred when Virtualization.framework exposes VSock)
  2. SSH Unix domain socket forwarding (`~/.substrate/sock/agent.sock`)
  3. SSH TCP forwarding (`127.0.0.1:<port>`)
  - The backend attempts transports in that order; failure logs include remediation hints and the shell degrades to host execution after a single warning if all transports fail.

- Logs & diagnostics
  - Agent logs live in the guest: `substrate sudo journalctl -u substrate-world-agent -n 200` (the CLI shells into Lima automatically) or manually via `limactl shell substrate sudo journalctl -u substrate-world-agent -n 200`.
  - Forwarding issues surface in shell `DEBUG` logs with the selected transport. `scripts/mac/lima-doctor.sh` mirrors doctor CLI checks.

- Validation
  - `scripts/mac/smoke.sh` exercises non‑PTY, PTY, and replay flows on macOS and asserts that the replay `fs_diff` contains project paths.

## 4) Isolation Details (Linux)

Per session world (identified by `WORLD_ID`, e.g., `wld_01994…`):
- Named network namespace: `substrate-<WORLD_ID>`
  - Created best‑effort on world setup; `lo` brought up
  - Netfilter rules installed in this netns when available
- Netfilter rules (nftables)
  - Table: `substrate_<WORLD_ID>` (inet)
  - Allows DNS and allowed domains (from policy), logs drops with prefix `substrate-dropped-<WORLD_ID>:`
- Cgroup v2 path: `/sys/fs/cgroup/substrate/<WORLD_ID>`
  - Resource limits applied best‑effort; PTY children are attached to this cgroup
- Filesystem isolation
  - Non‑PTY: overlay/copy‑diff is used by the world backend (fs_diff returned in ExecResult)
  - PTY: overlay not used in this phase; fs_diff collection for PTY is intentionally skipped

---

## 5) Agent API (over UDS)

Socket: `/run/substrate.sock`

- `GET /v1/capabilities`
  - Version and feature list for readiness probes
- `POST /v1/execute` (non‑PTY)
  - Body: `{ cmd, cwd, env, pty: false, agent_id, budget? }`
  - Returns: `{ exit, span_id, stdout_b64, stderr_b64, scopes_used }`
- `GET /v1/stream` (WebSocket, PTY)
  - Client → Server frames (text JSON):
    - `{"type":"start","cmd":"bash -lc '<raw>'","cwd":"/path","env":{...},"span_id":"spn_...","cols":<u16>,"rows":<u16>}`
    - `{"type":"stdin","data_b64":"..."}`
    - `{"type":"resize","cols":<u16>,"rows":<u16>}`
    - `{"type":"signal","sig":"INT|TERM|HUP|QUIT"}`
  - Server → Client frames:
    - `{"type":"stdout","data_b64":"..."}`
    - `{"type":"exit","code":0}`
    - `{"type":"error","message":"..."}`
- `GET /v1/trace/:span_id` (placeholder)
- `POST /v1/request_scopes` (placeholder)
- `POST /v1/gc` (netns GC; see §10)

Notes
- Protocol is stable; no TLS by design (UDS only).

---

## 6) Shell Behavior

- Default‑on world (Linux & macOS)
  - On startup, the shell ensures a session world and sets `SUBSTRATE_WORLD=enabled` plus `SUBSTRATE_WORLD_ID`.
  - macOS builds warm the Lima VM, establish forwarding, and reuse the same backend factory used on Linux.
- Routing
  - Non‑PTY: POST to `/v1/execute` over UDS (Linux) or the forwarded socket/port (macOS).
  - PTY: use WS to `/v1/stream` over the active transport; fallback to host PTY otherwise.
- Prompt safety
  - The REPL wraps PTY runs in `reedline::suspend_guard()` to avoid prompt corruption during external output.
- Readiness & auto‑spawn
  - The shell probes `/v1/capabilities`; if stale socket is found, it removes it.
  - If the agent isn’t running, the shell attempts to spawn it (Linux dev flow: `target/debug/world-agent`).
  - On macOS the shell invokes the Lima backend ensure path, which starts the VM if needed and establishes the first working transport before proceeding.
- Fallback
  - Exactly one warning is printed if the WS/agent path fails, then host execution proceeds.

---

## 7) Logging & Telemetry

- Windows spans include an optional `fs_diff.display_path` map pairing canonical WSL paths with native Windows paths for telemetry consumers.

- world-agent (PTY)
  - Logs: client connected; `start cmd=… cwd=… span_id=… cols=… rows=…`;
  - In‑world details: `world_id=… ns=… cgroup=… in_world=true` on PTY start
  - Forwarded signals: `ws_pty: forwarded signal <SIG> to pid <PID>`
  - Exit: `exit exit_code=…`; `session closed`
- Netfilter logs
  - Dropped packets are logged with prefix `substrate-dropped-<WORLD_ID>:` (subject to kernel/syslog visibility)

---

## 8) Environment Variables

- Core
  - `SUBSTRATE_WORLD=enabled|disabled` (default: enabled on Linux & macOS)
  - `SUBSTRATE_WORLD_ID` (set by the shell on ensure)
  - `SUBSTRATE_AGENT_ID` (tracing/attribution; defaults to "human")
- PTY/WS
  - `SUBSTRATE_WS_DEBUG=1` (prints “using world-agent PTY WS” on connect)
  - `SUBSTRATE_FORCE_PTY` / `SUBSTRATE_DISABLE_PTY` (developer toggles)
- GC Configuration
  - `SUBSTRATE_NETNS_GC_INTERVAL_SECS` (default 600; 0 disables periodic)
  - `SUBSTRATE_NETNS_GC_TTL_SECS` (default 0 meaning disabled)

---

## 9) Validation (Quick Start)

- Build & run agent
  - `cargo build && cargo build -p world-agent`
  - `RUST_LOG=info target/debug/world-agent &`
  - `ls -l /run/substrate.sock`

- Non‑interactive PTY over WS
  - `target/debug/substrate --pty -c "bash -lc 'echo hello && sleep 1 && echo done'"`

- Interactive PTY (robust echo)
  - `SUBSTRATE_WS_DEBUG=1 target/debug/substrate --pty -c "bash -lc 'printf \"TYPE> \"; IFS= read -r line; printf \"OK: %s\\n\" \"\$line\"'"`

- Signals
  - While running the above (waiting for input): `kill -INT $(pgrep -n substrate)`

- REPL PTY
  - `target/debug/substrate`
  - `substrate> :pty bash -lc 'printf "TYPE> "; IFS= read -r line; printf "OK: %s
" "$line"'`

- Isolation checks
  - `ip netns list | grep substrate-`
  - From agent logs: use `world_id` to inspect `/sys/fs/cgroup/substrate/<WORLD_ID>`

- macOS quick validation
  - `scripts/mac/lima-doctor.sh`
  - `PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh` (non‑PTY, PTY, replay + fs_diff assertion)
  - `substrate sudo journalctl -u substrate-world-agent -n 200` (or `limactl shell substrate sudo journalctl -u substrate-world-agent -n 200`) to review guest logs

---

## 10) Troubleshooting

- No WS debug line / no agent logs
  - The shell fell back to host PTY. Ensure `SUBSTRATE_WORLD=enabled` or start the agent (`/run/substrate.sock` must exist) and re‑run.
- “Exec format error” running `world-agent`
  - You’re trying to run a binary built for a different arch/OS. Build within the container/host you’re running on.
- `stty -a` fails in REPL without PTY
  - Use the PTY path (`:pty …` in REPL or `--pty -c` on CLI). `stty` requires a real TTY.
- Quoting one‑liners
  - Prefer the single‑quote trick shown above; unescaped inner double quotes will break `-c` arguments.
- macOS fallback warnings every command
  - Run `scripts/mac/lima-doctor.sh` to confirm virtualization support, VM status, agent socket, and forwarding binaries. Address any `[FAIL]` entries before re-running the shell.
- macOS vsock unavailable
  - The backend automatically falls back to SSH UDS/TCP. If VSock is expected, confirm `vsock-proxy` exists in `$PATH` and Lima is running with `vmType: "vz"`.

---

## 11) Netns GC (Implemented)

Goal: Safely garbage‑collect orphaned `substrate-<WORLD_ID>` netns and best‑effort teardown of related nftables and empty cgroup dirs.

Implemented features:
- Triggers
  - Startup sweep (once), periodic sweep (default every 10 minutes; configurable), and `POST /v1/gc` endpoint for on‑demand GC.
- Conservative delete criteria
  - Name matches `substrate-wld_`
  - `ip netns pids <ns>` has no PIDs
  - `/sys/fs/cgroup/substrate/<WORLD_ID>/cgroup.procs` empty or missing
  - Optional TTL guard via `SUBSTRATE_NETNS_GC_TTL_SECS`
- Actions (best‑effort)
  - `ip netns exec <ns> nft delete table inet substrate_<WORLD_ID>` (ignore errors)
  - `ip netns delete <ns>`
  - Remove empty cgroup dir
- Reporting & logs
  - JSON report: `{ removed: [..], kept: [{name,reason}], errors: [{name,message}] }`
  - INFO summary per sweep; DEBUG reasons per namespace

---

## 12) Limitations & Next Steps

- PTY overlay fs_diff is intentionally deferred; non‑PTY continues to provide `fs_diff`.
- macOS/Windows support is "observe‑only" for worlds.
- Next steps
  - Consider PTY overlay or post‑exit diff per span as a follow‑up phase

---

## 13) Naming Conventions

- World ID: `wld_<UUIDv7>`
- Netns: `substrate-<WORLD_ID>`
- Netfilter table: `substrate_<WORLD_ID>` (inet)
- Netfilter log prefix: `substrate-dropped-<WORLD_ID>:`
- Cgroup: `/sys/fs/cgroup/substrate/<WORLD_ID>`
- Socket: `/run/substrate.sock`

---

## 14) Pointers & Related Docs

- Implementation plan (phase): `docs/ALWAYS_IN_WORLD_PTY_EXECUTION_PLAN.md`
- Earlier roadmap: `docs/PHASE_4_5_ALWAYS_WORLD_IMPLEMENTATION_PLAN.md`
- Architecture: `docs/ARCHITECTURE.md`
- Policy broker: `docs/BROKER.md`
- Telemetry: `docs/TELEMETRY.md`, `docs/TRACE.md`
