# Substrate Windows Transport Parity Overview (External Briefing)

## 1. System Primer

- **Mission**: Substrate isolates untrusted automation by forcing every command
  through controlled "worlds" that enforce policy, collect telemetry, and emit
  reproducible filesystem diffs.
- **World lifecycle**:
  - Host tool (`substrate-shell`, `host-proxy`, replay, automation APIs) asks a
    backend to prepare a world.
  - `world-agent` inside the world executes the request, returning stdout,
    stderr, exit status, telemetry spans, and a structured fs diff.
  - The host persists span metadata, diffs, and policy verdicts for replay or
    audit.
- **Platform matrix**:
  - Linux: native namespaces + cgroups.
  - macOS: Lima VM with vsock transport.
  - Windows: WSL2 distro + Windows named pipe <-> Unix socket forwarder.
- **Windows data path**:
  - `substrate-shell` resolves the agent transport via `world-windows-wsl`,
    and communicates over a host named pipe to the forwarder by default.
    The forwarder then targets loopback TCP inside WSL (default
    `127.0.0.1:61337`), with a Unix-socket fallback when required.
  - `host-proxy` deserialises `TransportConfig` (named pipe vs TCP) and dials
    the forwarder, emitting telemetry for the selected mode.
  - `substrate-forwarder` exposes `\\\\.\\pipe\\substrate-agent`, accepts the host
    connection, spawns `wsl.exe` helpers that reach the agent over its Unix
    socket, and streams traffic bidirectionally.
  - `world-agent` executes commands under systemd supervision and returns
    structured results.

```text
┌─────────────┐    transport    ┌────────────────┐    exec    ┌──────────────┐
│ Host Client │───────────────▶│ Windows Pipe    │──────────▶│ WSL world     │
│ (shell/CI)  │                │ Forwarder       │           │ agent + exec │
└─────────────┘◀───────────────┴─────────────────┴───────────┴──────────────┘
        ▲              spans + fs diff ▲        pipe logs / ACLs↓
        │                               │             systemd status
```

- **Audit surfaces**: Forwarder structured logs (`%LOCALAPPDATA%\\Substrate`),
  agent logs (`/var/log/substrate-world-agent.log`), and evidence markdown
  history under `docs/project_management/logs`.

## 2. Chronology of Plans (Why We Are Here)

1. **Phase 4.5 Always-in-World (Linux)**
   - Enforced auto-world creation in `substrate-shell`.
   - Switched replay to `world-api::ExecResult`.
   - Standardised netfilter telemetry and introduced evidence logging rules.
2. **Phase 4.5 macOS Plan**
   - Brought Lima VM workflow, host tooling, smoke tests, and documentation up
     to parity with Linux.
   - Documented operator guardrails (no manual VM editing, require `lima sudo`).
3. **Phase 5 Windows Plan**
   - Added PowerShell helpers (`wsl-warm`, `wsl-doctor`, `wsl-smoke`), forwarder
     crate requirements, security expectations (pipe ACLs, service accounts),
     and end-to-end validation matrix.
4. **Transport Parity Spike**
   - Responded to Windows build failures caused by Unix-only transports.
   - Broke work into phases W/M/L/Final with status matrix and mandatory
     evidence logs.
   - Introduced cross-platform transport trait in `agent-api-client` and
     telemetry requirements (`transport.mode`, `transport.endpoint`).
5. **Windows Host Transport Addendum**
   - Provides day-by-day instructions for Spike Step W6 (host integration) and
     Step W8 (validation).
   - Calls out named-pipe forwarder tuning, telemetry verification, and warm
     workflow acceptance criteria.

## 3. Architectural State (2025-09-29)

- **Branch**: `feature/world-isolation` (HEAD
  `5053f6160b68a0c1fe4fd506835d073b18db416a`).
- **host-proxy**:
  - Parses `TransportConfig` from config/env.
  - Named-pipe serde path covered by unit + integration tests.
  - `cargo check/test -p host-proxy` passes on Windows.
- **substrate-shell**:
  - Telemetry spans set `transport.mode` for PTY and non-PTY execution.
  - Windows-specific unit test ensures named-pipe metadata is emitted.
- **world-windows-wsl**:
  - Exposes `build_agent_client()` to share transport logic with host crates.
  - Provides config-driven overrides to force Unix sockets when TCP is disabled.
- **agent-api-client**:
  - Houses connector trait with Unix, TCP, and named-pipe backends.
  - Removes `hyperlocal` dependency from Windows path.
- **Forwarder (current work)**:
  - Implements `PipeListener::create_listening_instance` and `accept_with`.
  - Accept loop tracks sessions via `JoinSet`, logging each client.
  - Unit test exercises new accept workflow.
- **Docs**: Setup, troubleshooting, and host-transport addendum live in
  `docs/dev`. Markdown linting passes.

## 4. Evidence Trail (Key Entries)

- **2025-09-22T16:52–21:52 (P0.x tooling/docs)**
  - PASS. Created warm/doctor scripts and authored Windows setup guides.
- **2025-09-23T16:22–16:24 (P2.4/P2.5 telemetry/troubleshooting)**
  - PASS. Added span display-path support and new doctor checks (T-011/T-012).
- **2025-09-24T10:51–11:07 (Phase W0–W3)**
  - PASS. Validated host prerequisites, documented baseline hyperlocal failure,
    and landed the initial transport abstraction.
- **2025-09-29T14:12 (Step W6 shell integration)**
  - PASS. `cargo check -p substrate-shell` recorded transport telemetry metadata.
- **2025-09-29T14:52 (Step W6 host proxy + warm)**
  - PARTIAL. Warm script timed out; forwarder log emitted `ERROR_PIPE_BUSY`.
- **2025-09-29T23:05–23:07 (Forwarder refactor attempt)**
  - FAIL. Refactored accept loop but warm still fails; log shows a single
    named pipe instance ready entry followed by repeated busy errors.

Full log: `docs/project_management/logs/windows_always_world.md` (see the
2025-09-29T14:52 anchor and subsequent notes).

## 5. Current Failure Picture

- `scripts/windows/wsl-warm.ps1` rebuilds the Linux agent inside WSL, restarts
  the service, launches the forwarder, and polls `Test-Path \\\\.\\pipe\\substrate-`
  `agent` for 30 seconds.
- Forwarder logs after refactor:
  - `INFO named pipe instance ready` (only once per run).
  - Continuous `ERROR failed to accept pipe connection: All pipe instances are
    busy. (os error 231)` until warm aborts.
- Manual execution with incorrect pipe path proves validation works and the
  process exits cleanly.
- Killing the lingering forwarder process is required before rebuilding or
  rerunning warm.

## 6. Environment Snapshot

- **Host OS**: Windows 11 Pro; virtualization enabled (VBS running).
- **Shell**: PowerShell 7.2.6 accessible as `pwsh` (required by scripts).
- **WSL**: Ubuntu 24.04 (`substrate-wsl`), systemd enabled via `/etc/wsl.conf`.
- **Agent**: Built inside WSL (`cargo build -p world-agent --release`), service
  managed with `systemctl restart substrate-world-agent`.
- **Logs**: `%LOCALAPPDATA%\\Substrate\\logs\\forwarder.YYYY-MM-DD` accumulate
  structured JSON.
- **Tooling prerequisites**: `winget` installed PowerShell, `npx
  markdownlint-cli`, `cargo.exe` on PATH for Windows-hosted builds.

## 7. Forwarder Code (Current Behaviour)

- `PipeListener::create_listening_instance` wraps `ServerOptions::new()` with
  ACLs (`D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)`), logs when a handle is ready.
- `accept_with` awaits `NamedPipeServer::connect()` and simultaneously prepares
  the next server instance.
- `serve` starts with one pre-created instance, stores a pinned future, and on
  each success spawns `bridge::run_pipe_session` (which in turn spawns WSL
  bridge processes).
- When `accept_with` returns an error, the loop sleeps 200 ms before creating a
  new instance.
- Unit test now creates a pending server, calls `accept_with`, and validates a
  basic round trip.

## 8. Working Hypotheses

1. **Missing `DisconnectNamedPipe`**: Dropping the `NamedPipeServer` at session
   completion might not disconnect promptly, leaving the kernel counting the
   instance as busy.
2. **Tokio IOCP semantics**: Creating the next server inside `accept_with` may
   still be too late; the recommended pattern creates a new server *before*
   awaiting clients and calls `ServerOptions::create` (without custom wrapper)
   in the loop.
3. **Readiness check mismatch**: `Test-Path` might not see the pipe until a
   client opens it. Without a probe client, the host never triggers the accept
   future, so the initial instance never transitions to "listening".
4. **Security descriptor nuance**: Though ACL grants interactive users access,
   the forwarder runs under the current user account. Verification with
   Sysinternals `accesschk` may be necessary.
5. **Lingering PID file**: Warm script removes the PID file before relaunch, but
   a crashed forwarder can leave orphaned handles.

## 9. Questions for External Review

1. What is the canonical async accept pattern for Windows named pipes under
   Tokio 1.47+? Are we expected to call `disconnect()` explicitly after each
   session?
2. Should the warm workflow switch to an active probe (open pipe, close) rather
   than `Test-Path`? If yes, what is the minimal, race-free health check?
3. Does Windows require multiple pre-created instances (equal to
   `max_instances`) to avoid `ERROR_PIPE_BUSY` when no client is connected?
4. Could WSL integration (spawning `wsl.exe` per session) hold the pipe open
   until the child process finishes, and do we need to flush handles manually?

## 10. Suggested Experiments Before Engaging Guru

- **Standalone repro**: Implement the same loop in a minimal binary, run it
  without the WSL bridge, and confirm whether the pipe ever becomes observable.
- **Session teardown logging**: Instrument `bridge::run_pipe_session` to confirm
  when the `NamedPipeServer` is dropped and whether a manual
  `DisconnectNamedPipe` helps.
- **Host probe**: Modify `wsl-warm.ps1` temporarily to attempt
  `[System.IO.Pipes.NamedPipeClientStream]::new('.', 'substrate-agent', 'InOut')`
  and close it, measuring whether the forwarder accepts once.

### Recommended Windows Pipe Probe (Canonical)

- Use the status-line-only helper for a race-free health check of the forwarder’s named pipe. This avoids PowerShell `StreamReader.ReadLine()` quirks and body reads.

```powershell
pwsh -File scripts/windows/pipe-status.ps1 `
  -PipePath '\\.\pipe\substrate-agent' `
  -TimeoutSeconds 8 `
  -ExpectStatus 200
```

- Acceptance: prints `HTTP/1.1 200 OK` within ≤ 8 seconds.
- Always show the canonical, quoted PipePath `'\\.\pipe\substrate-agent'` in examples.
- **Handle inspection**: Use Sysinternals `handle.exe` or PowerShell
  `Get-ChildItem \\\\.\\pipe\\` while the forwarder waits to verify the instance
  exists and identify owning PIDs.
- **ACL validation**: Run `icacls \\\\.\\pipe\\substrate-agent` (or equivalent) to
  confirm interactive users have `GA` permissions.

## 11. Guardrails & Evidence Expectations

- Do not reorder Spike steps. Every command run must be appended to
  `windows_always_world.md` with timestamp, output, and remediation notes.
- When running the forwarder manually, launch it via ``scripts/windows/start-forwarder.ps1``
  so a five-minute timeout prevents hung sessions.
- Update the Phase Status Matrix in `docs/SPIKE_TRANSPORT_PARITY_PLAN.md` only
  when Step W6 and Step W8 meet exit criteria.
- Keep `/etc/wsl.conf` stable (`systemd=true`). Build the agent inside WSL;
  never copy Windows binaries into the distro.
- Maintain ASCII for scripts/docs and lint with `markdownlint-cli`.
- Preserve warm/doctor script behaviour unless explicitly approved otherwise.

## 12. Reference Packet for External Reviewer

- Plans: `docs/PHASE_4_5_ALWAYS_WORLD_IMPLEMENTATION_PLAN.md`,
  `docs/PHASE_4_5_ALWAYS_WORLD_MAC_PLAN.md`,
  `docs/PHASE_5_ALWAYS_WORLD_WINDOWS_PLAN.md`,
  `docs/SPIKE_TRANSPORT_PARITY_PLAN.md`,
  `docs/dev/transport_parity_design.md`,
  `docs/dev/windows_host_transport_plan.md`.
- Setup/Troubleshooting: `docs/dev/wsl_world_setup.md`,
  `docs/dev/wsl_world_troubleshooting.md`, `scripts/windows/wsl-warm.ps1`,
  `scripts/windows/wsl-doctor.ps1`, `scripts/windows/wsl-smoke.ps1`,
  `scripts/windows/wsl-stop.ps1`.
- Evidence: `docs/project_management/logs/windows_always_world.md` (focus on
  2025-09-29 entries).
- Source hotspots: `crates/forwarder/src/pipe.rs`,
  `crates/forwarder/src/bridge.rs`, `crates/forwarder/Cargo.toml`,
  `crates/world-windows-wsl/src/lib.rs`, `crates/host-proxy/Cargo.toml`,
  `crates/host-proxy/src/lib.rs`, `crates/host-proxy/src/main.rs`,
  `crates/shell/src/lib.rs`, `crates/shell/src/platform_world/windows.rs`,
  `crates/agent-api-client/src/transport/*`.
- Supplemental note: `readthis.md` contains GPT-5 PRO feedback on plan
  alignment (build artifact handling, vsock vs pipe, telemetry requirements).
Transport defaults (Windows)
- Host → forwarder: named pipe by default (`\\.\pipe\substrate-agent`).
- Forwarder → agent (inside WSL): loopback TCP (`127.0.0.1:61337`) by default.
- To opt into host TCP (host client → forwarder via TCP) instead of named pipe,
  set `SUBSTRATE_FORWARDER_TCP=1` or provide an explicit
  `SUBSTRATE_FORWARDER_TCP_ADDR=host:port`. If not set, the backend uses the
  named pipe.
