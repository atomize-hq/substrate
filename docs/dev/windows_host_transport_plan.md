# Windows Host Transport Integration Plan

Status: Draft (Phase W addendum)
Owner: Windows Transport Strike Team
Last Updated: 2025-09-30T14:35:00Z
Related Docs:

- docs/dev/transport_parity_design.md
- docs/SPIKE_TRANSPORT_PARITY_PLAN.md
- docs/dev/windows_transport_external_overview.md
- docs/project_management/logs/windows_always_world.md

---

## 1. Purpose & Scope

This addendum augments the transport parity spike with the concrete tasks
required to finish Windows host integration. It now reflects the updated
strategy recommended by the guru review:

- Adopt Tokio's canonical named-pipe accept pattern (pre-create next instance,
  explicit disconnect).
- Switch the warm workflow to an active pipe probe.
- Prefer loopback TCP from the forwarder into WSL, with a Unix-socket fallback
  staged separately.
- Reconfirm telemetry and host-crate wiring against the new transport modes.
- Document guardrails, evidence, and follow-up work.

All work items still map to Spike Step W6 (implementation) and Step W8
(validation). Follow this plan alongside the main spike document.

---

## 2. Preconditions

- Phase W prerequisites and evidence logging are in place (`docs/project_management/logs/windows_always_world.md`).
- `/etc/wsl.conf` inside `substrate-wsl` contains `systemd=true`, and `wsl --shutdown`
  has been run since editing the file.
- `systemctl is-system-running` inside `substrate-wsl` reports `running` or
  `degraded`.
- Branch `feature/world-isolation` is up to date with the latest integration base.
- `%LOCALAPPDATA%\Substrate\logs` is available for forwarder inspection.
- `cargo`, `pwsh`, `npx markdownlint-cli`, and `handle.exe` (optional) are present.

---

## 3. Task Breakdown

### 3.1 Rework Forwarder Accept Loop

1. Update `crates/forwarder/src/pipe.rs`:
   - Use `ServerOptions::new().first_pipe_instance(true).reject_remote_clients(true)`.
   - Pre-create the next server instance **before** spawning the task for the
     connected client.
   - On session completion call `FlushFileBuffers` (optional but recommended)
     followed by `NamedPipeServer::disconnect()` before dropping the handle.
2. Instrument `bridge::run_pipe_session` to log session start/end, bytes, and
   disconnect status.
3. Extend tests:
   - Unit test covers multiple sequential clients, asserting `disconnect()` was
     invoked.
   - Optional async test to open two clients in sequence using
     `NamedPipeClientStream`.
4. Evidence: capture diffs, `cargo fmt`, `cargo check -p substrate-forwarder`,
   and `cargo test -p substrate-forwarder`.

> When running the forwarder manually during debugging, use
> `scripts/windows/start-forwarder.ps1`. It applies a five-minute timeout so
> hung sessions return control to the operator. Example:
>
> ```powershell
> pwsh -File scripts/windows/start-forwarder.ps1 -DistroName substrate-wsl
> ```

### 3.2 Warm Workflow & Health Probe

1. Modify `scripts/windows/wsl-warm.ps1`:
   - After launching the forwarder, replace `Test-Path` with a
     `WaitNamedPipe` + client open/close probe (PowerShell or .NET).
   - Log probe success/failure and the elapsed time.
   - Ensure lingering PID files are removed before relaunch.
2. Update docs (`docs/dev/wsl_world_setup.md`, troubleshooting catalogue) to
   describe the new probe.
3. Evidence: include the probe output in the evidence log and forwarder logs
   showing sessions created by the probe.

Note: The warm script now accepts a `-PipePath` override. Use this to avoid
conflicts if another process already owns the default
`\\.\pipe\substrate-agent` during investigation (e.g.,
`-PipePath \\.\pipe\substrate-agent-$env:USERNAME`). All manual forwarder runs
must still use `scripts/windows/start-forwarder.ps1` for the timeout guard.

Quick-Return Launcher (manual runs)
- Always start the forwarder with quick return (do not wait for process exit):

  ```powershell
  pwsh -File scripts/windows/start-forwarder.ps1 `
    -DistroName substrate-wsl `
    -PipePath \\.
    \pipe\substrate-agent `
    -ReadyTimeoutSeconds 20
  ```

- Only use `-WaitForExit` in CI/service contexts. Manual runs should never wait
  for the process to exit; rely on the readiness window and then continue.
- For end-to-end checks, use the helper to probe over the pipe:

  ```powershell
  pwsh -File scripts/windows/pipe-http.ps1 `
    -PipePath \\.
    \pipe\substrate-agent `
    -TimeoutSeconds 8 `
    -ExpectStatus 200
  ```

### 3.3 Downstream Target: TCP-First Bridge

1. Enable the agent's loopback TCP listener inside WSL (default
   `127.0.0.1:61337`, configurable via env).
2. Update forwarder config to target the TCP endpoint by default. Retain the
   Unix-socket path behind a feature flag or configuration option.
3. Add configuration validation and logging that records the downstream target
   (`target_mode=tcp`, `target=127.0.0.1:61337`).
4. Evidence: `systemctl status substrate-world-agent` showing TCP listener,
   forwarder log confirming TCP mode, and a successful warm run.

### 3.4 Unix-Socket Fallback (Stage 2)

_This remains follow-up work but document the approach now so it can be picked up
immediately after TCP parity is achieved._

1. Draft a helper (or documented command) that spawns `wsl.exe -- socat` to
   bridge the Unix socket to a loopback TCP port when the agent requires UDS.
2. Keep the fallback behind a configuration flag (for example,
   `SUBSTRATE_FORWARDER_TARGET=uds`).
3. Ensure teardown logic kills the `socat` process when sessions end.
4. Evidence: leave TODO notes in the evidence log when this stage is picked up.

### 3.5 Host Crate Alignment

1. `world-windows-wsl`:
   - Keep `build_agent_client()` returning the chosen transport.
   - Add methods to surface TCP vs. named-pipe selection in telemetry.
2. `platform_world::windows` and `substrate-shell`:
   - Continue using the shared builder but ensure telemetry reflects new modes
     (`tcp`, `named_pipe`).
   - Update PTY/WebSocket flows to rely on Hyper/H1 + tungstenite over the
     chosen transport.
3. `host-proxy`:
   - Default to named-pipe URI but accept TCP overrides.
   - Extend serde tests for `named-pipe://.` and `tcp://127.0.0.1:7777`.
4. Evidence: document `cargo fmt`, `cargo check` for all touched crates plus
   targeted tests.

### 3.6 Regression & Verification Checklist

1. `cargo fmt` (workspace or targeted crates).
2. `cargo check -p substrate-forwarder`
3. `cargo check -p host-proxy`
4. `cargo check -p world-backend-factory`
5. `cargo check -p substrate-shell`
6. `cargo test -p substrate-forwarder`
7. `cargo test -p world-windows-wsl`
8. `npx markdownlint-cli docs/dev/windows_host_transport_plan.md`
9. Manual warm run with evidence collection (`scripts/windows/wsl-warm.ps1`).

---

## 4. Risk Notes & Mitigations

- **Pipe busy / race conditions**: avoided by the new accept loop and explicit
  disconnects; verify with the probe before running smoke tests.
- **Forwarder targeting wrong endpoint**: logs now emit `target_mode`/`target`;
  add guardrails in the script to fail fast if neither TCP nor named pipe is
  configured.
- **Telemetry drift**: update unit tests (e.g., `transport_meta_named_pipe_mode`)
  to cover the TCP case and run smoke tests on both transports if possible.
- **Fallback parity**: treat the Unix-socket fallback as a separate task with
  clear TODOs to avoid overloading Step W6; document what remains at handoff.

---

## 5. Deliverables

- Updated forwarder implementation with canonical accept loop and instrumentation.
- Warm script using an active `WaitNamedPipe`/client probe.
- Forwarder defaulting to loopback TCP downstream with documented configuration.
- Host crates and telemetry reflecting the new transport modes.
- Updated documentation (this plan, troubleshooting, setup, external overview).
- Evidence log entries for each major change, including successful warm run.

---

## 6. Handoff Checklist

Use this when pausing work:

- [ ] Evidence log updated with timestamps, commands, outputs, and remediation.
- [ ] Forwarder log snippets attached for each attempted probe/run.
- [ ] Outstanding TODOs (especially the Unix-socket fallback) captured in the
      evidence log under "Next Actions".
- [ ] Phase Status Matrix left unchanged unless Step W6 or W8 exit criteria are
      satisfied.
