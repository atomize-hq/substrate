# Windows Host Transport Integration Plan

Status: Draft (Phase W addendum)
Owner: Windows Transport Strike Team
Last Updated: 2025-09-29T00:00:00Z
Related Docs:

- docs/dev/transport_parity_design.md
- docs/SPIKE_TRANSPORT_PARITY_PLAN.md
- docs/project_management/logs/windows_always_world.md

---

## 1. Purpose & Scope

This addendum augments the transport parity spike with the concrete tasks
required to finish Windows host integration. It fills the gap between the
architecture overview and the day-to-day work required to:

- Expose a supported Agent transport from `world-windows-wsl`.
- Wire `substrate-shell`, `host-proxy`, and related host binaries to the
  named-pipe transport while keeping Unix behaviour intact.
- Verify the forwarder plus agent pipeline end to end on Windows.
- Document Windows-only guardrails and evidence expectations.

All work items map to Step W6 and Step W8 of the spike plan. Use this plan
alongside the main spike document.

---

## 2. Preconditions

- Phase W prerequisites are complete (WSL distro provisioned, warm scripts
  available, evidence logging in place).
- `/etc/wsl.conf` inside `substrate-wsl` contains the systemd bootstrap:

  ```ini
  [boot]
  systemd=true
  ```

- The host has run `wsl --shutdown` since editing the file, and
  `systemctl is-system-running` inside the distro reports `running` or
  `degraded` (doctor should warn—but not fail—if systemd is unavailable).
- Branch `feature/world-isolation` is checked out and rebased on the latest
  integration commit.
- `%LOCALAPPDATA%\Substrate\logs` is accessible for forwarder inspection.

---

## 3. Task Breakdown

### 3.1 Expose Windows Agent Transport

1. Build the Linux agent inside WSL and install it.
   - Run `cargo build -p world-agent --release` within `substrate-wsl`.
   - Install the ELF into `/usr/local/bin/substrate-world-agent` with
     `sudo install`, then restart the service via `systemctl restart`.
   - Update `scripts/windows/wsl-warm.ps1` to perform this flow if it still
     copies a Windows `.exe` into WSL.
2. Add a public accessor in `world-windows-wsl`.
   - Provide `pub fn agent_transport(&self) -> Transport` or
     `pub fn build_agent_client(&self) -> AgentClient`.
   - Ensure the API returns the same decision the backend already makes (named
     pipe when the forwarder exposes one, TCP otherwise).
   - Add unit tests for the env-driven branches
     (`SUBSTRATE_FORWARDER_TCP_ADDR` and `SUBSTRATE_FORWARDER_TCP`).
3. Document expectations around security (pipe ACL, TCP loopback only).
4. Evidence: capture `cargo fmt`, `cargo check -p world-windows-wsl`, and
   `cargo test -p world-windows-wsl` in the Windows log.

### 3.2 Integrate With `platform_world::windows`

1. Replace ad-hoc client construction with the new accessor.
2. Ensure `WorldTransport::NamedPipe(PathBuf)` is stored and its `Display`
   implementation sanitises the path.
3. Cache the resolved transport in the global context for reuse.
4. Surface actionable errors that reference forwarder logs when the pipe is
   missing.
5. Evidence: note `cargo fmt`, `cargo check -p substrate-shell`, and any
   diagnostic logs.

### 3.3 Shell Command Execution Paths

Telemetry spans must populate `transport.mode` and `transport.endpoint`
consistently. Expect `named_pipe` on Windows and `unix` on Unix hosts.

1. Non-PTY execution.
   - Keep mac-specific logic under `cfg(target_os = "macos")`.
   - Add a Windows helper that fetches the context, ensures the session, and
     uses the prebuilt `AgentClient` (or `AgentClient::named_pipe`).
   - Leave the Unix fallback unchanged.
2. PTY and WebSocket execution.
   - Guard the macOS websocket helper behind `cfg(target_os = "macos")`.
   - Implement a Windows helper that upgrades HTTP to WS via the named-pipe
     connector. Until parity is complete, warn once and fall back to the host
     PTY when pipe operations fail.
3. Build-system hygiene: keep `hyperlocal` under
   `[target.'cfg(unix)'.dependencies]` only.
4. Evidence: record `cargo fmt`, `cargo check -p substrate-shell`, and any
   telemetry assertions.

### 3.4 Host Proxy Alignment

1. Default `AgentTransportConfig` to the named pipe on Windows while keeping
   Unix defaults intact.
2. Ensure CLI and environment overrides (for example,
   `SUBSTRATE_AGENT_TRANSPORT=named-pipe://.`) produce the named-pipe variant.
3. Add tests that serialise and deserialise the named pipe configuration.
4. Evidence: capture `cargo check -p host-proxy` and (where feasible)
   `cargo test -p host-proxy`.

### 3.5 Forwarder and Warm Script Verification

1. Run the warm script end to end and confirm the Linux agent was rebuilt and
   installed inside WSL.

   ```pwsh
   pwsh -File scripts/windows/wsl-warm.ps1 `
     -DistroName substrate-wsl `
     -ProjectPath (Resolve-Path .)
   ```

2. Inspect `%LOCALAPPDATA%\Substrate\logs` for `forwarder.target=<mode>` and
   `pipe_ready=true` entries.
3. Verify the pipe exists.

   ```pwsh
   Get-ChildItem -Path "\\\\.\\pipe\\" |
     Where-Object { $_.Name -eq 'substrate-agent' }
   ```

4. Run a world-enabled shell command and note the transport metadata in the
   resulting trace.
5. Evidence: include command transcripts, `systemctl status`, forwarder log
   excerpts, and remediation notes.

### 3.6 Regression Checks

1. `cargo fmt`
2. `cargo check -p substrate-shell`
3. `cargo check -p host-proxy`
4. `cargo check -p world-backend-factory`
5. `cargo test -p world-windows-wsl`
6. `npx markdownlint-cli docs/dev/windows_host_transport_plan.md`

Document all results per the guardrails.

---

## 4. Risk Notes and Mitigations

- **Forwarder fails to publish the named pipe** – Inspect logs, verify the warm
  script reinstalled the Linux binary, and restart the service.
- **Mac-only code compiles on Windows** – Guard with `cfg` annotations and run
  checks on both Windows and WSL.
- **Telemetry gaps** – Ensure `AgentClient::transport_mode()` propagates the
  named-pipe mode and add telemetry assertions in smoke tests.
- **Documentation drift** – Run markdown linting and cross-link updates back to
  the main design doc.

---

## 5. Deliverables

- Public API in `world-windows-wsl` exposing the agent transport or a ready
  `AgentClient`.
- Updated `platform_world::windows` glue and shell command execution paths that
  honour named pipes.
- Host proxy configuration defaults and overrides that support named pipes.
- Evidence log entries for each subtask.
- Linted addendum stored as `docs/dev/windows_host_transport_plan.md`.
- Optional PR checklist referencing this plan.

---

## 6. Handoff Checklist

Use this when pausing work:

- [ ] Evidence log updated with a timestamped entry referencing this plan.
- [ ] Forwarder log snippet attached if issues occurred.
- [ ] Outstanding TODOs noted under "Next Actions" in the evidence log.
- [ ] Phase Status Matrix left unchanged unless Step W6 or W8 is complete.
