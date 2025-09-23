# Phase 4.5 — macOS "Always World" Implementation Plan

Status: Draft (awaiting review)  
Owner: Substrate Core  
Scope: Ship macOS parity with the Linux "Always World" stack. Every command (`substrate`, PTY, replay, shim) must run inside a Lima-hosted Linux world, returning identical telemetry, isolation, and UX.

This document is the canonical implementation guide. It assumes the executor is new to the project and has no context. Follow it linearly and run every check. Do not skip steps.

---

## Table of Contents

1. Background & Rationale
2. Prerequisites & Environment Setup
3. Architecture Overview
4. Phased Implementation (with sanity checks)
5. Incremental Validation Matrix
6. Observability & Logging Requirements
7. Risks & Mitigations
8. Rollout & Release Notes
9. Appendices (command snippets, scripts, configs)
10. Glossary & Quick Reference

---

## 1. Background & Rationale

| Capability | Linux (current) | macOS (current) | Target |
|------------|-----------------|-----------------|--------|
| Shell world execution | ✅ Auto agent spawn; PTY + non-PTY routed through world-agent | ❌ Host execution only | ✅ Same behavior via Lima VM |
| Replay isolation | ✅ uses world-api, returns `fs_diff` | ❌ host-only | ✅ identical world backend |
| Telemetry | ✅ spans include scopes + fs_diff | ❌ missing | ✅ same outputs |
| Docs/tooling | Podman doc, world doc, scripts | No official mac guide | Full Lima bootstrap, helper scripts, troubleshooting |
| Tests | Podman/VM smoke tests | None | Mac-specific smoke + validation scripts |

Design guardrails:
- Reuse Linux world-agent + policy code; mac path only handles transport/lifecycle.
- Always degrade gracefully with actionable warnings when world unavailable.
- Provide operators with turnkey scripts + docs.
- Instrument all mac-specific operations with `tracing` logs.
- Secure forwarded sockets (`0700` dirs, auto cleanup).

---

## 2. Prerequisites & Environment Setup

### 2.1 Host Requirements

- macOS 13.0+ (required for Virtualization.framework & VSock). Verify with:
  ```sh
  sw_vers
  sysctl hw.optional.arm64 # 1 = Apple Silicon, 0 = Intel
  sysctl kern.hv_support   # 1 = Apple hypervisor available
  ```
- Homebrew packages:
  ```sh
  brew install lima jq openssh coreutils gnused gnu-tar gettext
  ```
- Ensure `envsubst` (from gettext) is on PATH:
  ```sh
  echo 'export PATH="/opt/homebrew/opt/gettext/bin:$PATH"' >> ~/.zshrc
  source ~/.zshrc
  command -v envsubst
  ```
- Ensure Lima utilities (including `vsock-proxy`) are available:
  ```sh
  echo 'export PATH="$(brew --prefix lima)/bin:$PATH"' >> ~/.zshrc
  source ~/.zshrc
  command -v vsock-proxy
  ```
- Xcode Command Line Tools:
  ```sh
  xcode-select --install
  ```
- Rust toolchain on host (nightly not required):
  ```sh
  curl https://sh.rustup.rs -sSf | sh
  ```

### 2.2 Lima VM Expectations

Inside the guest we need:
- Ubuntu 24.04 image (arm64/x86 to match host). Lima handles download.
- Packages: `nftables iproute2 libseccomp-dev curl jq git python3 build-essential dnsmasq openssh-server unzip`. Provision script installs these.
- `substrate-world-agent` binary placed in `/usr/local/bin` (either copy from host `target/release/world-agent` or download released artifact).
- Systemd service `substrate-world-agent` enabled & running. Socket: `/run/substrate.sock`.

### 2.3 Transport Options (Host ↔ VM)

| Transport | Requirements | When Used | Notes |
|-----------|--------------|-----------|-------|
| VSock | macOS 13+, `vmType: "vz"`, `vsock-proxy` available | Primary | Use `vsock-proxy <port> unix:///run/substrate.sock` |
| SSH UDS | Working `ssh lima-substrate` | Fallback | Forward to `~/.substrate/sock/agent.sock`; ensure directory cleaned before binding |
| SSH TCP | Same SSH requirement | Last resort | Forward `127.0.0.1:17788` (or configurable). |

Implementation will auto-detect in that order.

### 2.4 Pre-Implementation Artifacts (must exist before coding)

1. **Lima profile** `docs/dev/lima/substrate.yaml` (see Appendix A).
2. **Helper scripts** under `scripts/mac/`:
   - `lima-warm.sh`: injects `$PROJECT` path, starts VM, waits for readiness.
   - `lima-stop.sh`: stops VM gracefully.
   - `lima-doctor.sh`: runs comprehensive health checks (see §5.1).
3. **Documentation** `docs/dev/mac_world_setup.md`: step-by-step guide, including provisioning logs, health checks, troubleshooting.
4. **Agent binary availability**: instructions on building (`cargo build -p world-agent --release`) and copying into VM.

Complete these before Phase 1.

---

## 3. Architecture Overview (Target State)

```
macOS Host
│
├── substrate CLI / REPL
│   ├── PlatformWorldContext (detect backend, socket, transport)
│   ├── ensure_world_agent_ready()
│   │     ├── macOS: ensure Lima VM + forwarding + agent
│   │     └── Linux: existing behavior (spawn agent binary)
│   ├── Non-PTY exec -> agent POST via agent-api-client
│   ├── PTY exec -> WebSocket via tokio-tungstenite
│   └── collect_world_telemetry (fs_diff, scopes)
│
├── Forwarding Manager (mac)
│   ├── VSock proxy or SSH tunnel
│   └── Manages child processes, cleans sockets on drop
│
└── Lima VM (Ubuntu)
    ├── substrate-world-agent (systemd)
    │    ├── Linux isolation stack (overlayfs, cgroups, netns, nftables)
    │    └── Listens on /run/substrate.sock
    ├── world-agent logs (journald)
    └── Provisioned directories (/var/lib/substrate, /run/substrate)
```

Failure handling at every hop (explicit logging + fallback).

---

## 4. Phased Implementation (with Sanity Checks)

Each phase contains numbered tasks. After each major task, run the sanity checks listed; do not proceed until all pass. Keep notes/logs for each check to ease debugging.

**Execution Guardrails (apply to every phase):**
- Follow tasks strictly in order; do not jump ahead even if a step feels familiar.
- Do not invent alternative commands, scripts, or tooling substitutions without explicit approval from the plan owner.
- Capture and store evidence for every sanity check (command output, log location, or file path) in the working log before advancing.
- If a sanity check fails, stop immediately, diagnose with the listed troubleshooting steps, and note remediation before retrying.
- When a guardrail conflicts with local intuition, the guardrail wins—escalate instead of improvising.

### Phase 0 — Provisioning & Bootstrap

**Objective**: Establish reproducible Lima environment + helper tooling. No Rust code changes yet.

**Phase Gate**: Begin only after confirming you have read the entire document end-to-end. Acknowledge in your work log that you will not modify commands or scripts beyond what is explicitly written.

#### P0.1 Create Lima Profile

- Create directory `docs/dev/lima/` (if missing).
- Write `substrate.yaml` (Appendix A) with:
  - Virtiofs mounts.
  - Provision script installing packages, enabling dnsmasq, user namespaces, creating directories, copying systemd unit.
  - `vmType: "vz"`; set `cpus`, `memory`, `disk` defaults (e.g., 4 CPUs, 4GiB, 20GiB).
- Validate YAML using `limactl validate docs/dev/lima/substrate.yaml`.

**Sanity Check (log evidence)**: `limactl validate` returns success.

#### P0.2 Build Helper Scripts

- `scripts/mac/lima-warm.sh`
  - Shebang `#!/usr/bin/env bash`.
  - `set -euo pipefail`.
  - Accept optional project path argument (defaults to current repo path).
  - Substitute `$PROJECT` in YAML by copying profile to temp file (use `envsubst`).
  - Start VM: `limactl start substrate --tty=false`.
  - Wait for status `Running` (poll `limactl list substrate --json`). Timeout 120s with helpful error.
- `scripts/mac/lima-stop.sh`
  - Stops VM if running; prints status if already stopped.
- `scripts/mac/lima-doctor.sh`
  - Checks: Lima version, VM status, virtualization sysctl, SSH connectivity (`limactl shell substrate uname -a`), agent capabilities (`curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities` inside VM), forwarded socket presence (later phases), systemd service status, nftables availability, `vsock-proxy` binary availability, disk usage.
- After writing the scripts, mark them executable:
  ```sh
  chmod +x scripts/mac/lima-*.sh
  ```

**Sanity Check (log evidence)**: Run each script with `bash -x` (dry-run). For `lima-doctor`, if VM not running yet, ensure it exits non-zero with guidance.

#### P0.3 Documentation

- `docs/dev/mac_world_setup.md` must include:
  1. Host prerequisites (brew packages, virtualization check).
  2. Steps to install Lima profile (copy file, run `scripts/mac/lima-warm.sh`).
  3. How to copy agent binary: `cargo build -p world-agent --release`, `limactl copy ./target/release/world-agent substrate:/usr/local/bin`, set permissions.
  4. Starting agent service via systemd.
  5. Running `lima-doctor` and interpreting output.
  6. Manual smoke check: inside VM run `substrate-world-agent --version`.
  7. Troubleshooting table (virtualization off, service failing, permission errors, port conflicts).

**Sanity Check (log evidence)**: Have a teammate follow the doc literally; ensure they reach a running agent.

#### P0.4 Manual Provision Test

- Clean environment: `limactl delete substrate || true`.
- Run `scripts/mac/lima-warm.sh`.
- Build agent on host: `cargo build -p world-agent --release`.
- Copy binary into VM: `limactl copy target/release/world-agent substrate:/usr/local/bin/` and `limactl shell substrate sudo chmod 755 /usr/local/bin/world-agent`.
- Ensure systemd unit present (Appendix C). If provisioning did not create it, copy from repo: `limactl copy scripts/mac/substrate-world-agent.service substrate:/tmp/` then move inside VM.
- SSH into VM: `limactl shell substrate`.
- Enable & start service: `sudo systemctl daemon-reload && sudo systemctl enable --now substrate-world-agent`.
- Run inside VM: `curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities` → should return JSON.
- Exit VM; run `scripts/mac/lima-doctor.sh` → all checks PASS.

**Record outputs** for reference.

### Phase 1 — Complete `MacLimaBackend`

**Objective**: Make `crates/world-mac-lima` fully functional.

**Phase Gate**: Do not start Phase 1 until every Phase 0 sanity check is logged with command output evidence. Confirm the Lima VM, systemd unit, and doctor script are functioning before writing Rust code.

#### P1.1 Runtime & Struct Setup

- Add field `runtime: tokio::runtime::Runtime` to `MacLimaBackend`.
- Create `fn new_runtime() -> anyhow::Result<Runtime>` using `Runtime::new()` with `Builder::new_current_thread().enable_all()`.
- Ensure runtime is dropped gracefully (Drop impl logs message).

**Sanity Check (log evidence)**: `cargo build -p world-mac-lima` passes.

#### P1.2 Transport Handling

- Create new module `forwarding.rs` with structs:
  ```rust
  pub enum ForwardingKind { Vsock { port: u16 }, SshUds { path: PathBuf }, SshTcp { port: u16 } }
  pub struct ForwardingHandle { child: std::process::Child, kind: ForwardingKind }
  impl Drop for ForwardingHandle { /* terminate child, remove sockets */ }
  ```
- Update `Transport::auto_select()` to return `ForwardingKind` and keep `ForwardingHandle` in backend state.
- Implement detection functions:
  - `fn vsock_supported() -> bool` (inspect Lima state file or attempt `vsock-proxy --help`). When available, spawn child: `vsock-proxy --vm substrate {port} unix:///run/substrate.sock`. Wait for health by curling `http://127.0.0.1:{port}/v1/capabilities`.
  - `fn ssh_available() -> bool` (use `which::which("ssh")`). For Unix fallback, run `ssh -F ~/.lima/substrate/ssh.config -o ExitOnForwardFailure=yes -o StreamLocalBindUnlink=yes -L ~/.substrate/sock/agent.sock:/run/substrate.sock lima-substrate -N`.
  - TCP fallback command: `ssh -F ~/.lima/substrate/ssh.config -o ExitOnForwardFailure=yes -L 127.0.0.1:{port}:/run/substrate.sock lima-substrate -N`.
  - `fn create_forwarding(kind) -> Result<ForwardingHandle>`: create staging directories (e.g., `~/.substrate/sock` with `0700`), spawn command, leak child's stdout/stderr to logs, block until socket reachable (use exponential backoff), and register cleanup in `Drop`.

**Sanity Check (log evidence)**: Unit tests (mock commands) ensure fallback order correct.

#### P1.3 Agent Client Integration

- Add helper `fn agent_client(&self) -> anyhow::Result<AgentClient>` that builds client based on current transport (provide `Transport::into_agent_transport()` returning `agent_api_client::Transport`).
- For each `WorldBackend` method:
  - Ensure VM running (`self.ensure_vm_running()` via `LimaVM`).
  - Ensure forwarding active (`self.ensure_forwarding()` returns `&ForwardingHandle`).
  - Use runtime to call agent methods.
  - Cache `WorldHandle` if `spec.reuse_session` true.
- Provide detailed error context (use `Context` trait). Example message: "Lima forwarding setup failed (VSock proxy exit 1). Run scripts/mac/lima-doctor.sh".

Notes:
- Build the world-agent binary inside the Lima guest to avoid architecture mismatches. If you see `Exec format error` from systemd, rebuild in-guest and redeploy: `limactl shell substrate` → `cargo build -p world-agent --release` → copy to `/usr/local/bin/substrate-world-agent`.
- SSH UDS forwarding can be blocked by SSH ControlMaster sessions. Disable it for stream-local forwards with `-o ControlMaster=no -o ControlPath=none`.
- The agent relaxes `/run/substrate.sock` permissions at runtime so non-root clients (SSH UDS) can connect.

**Sanity Check (log evidence)**:
1. Unit test with mocked `AgentClient` verifying request parameters.
2. Manual: with VM running, run small Rust snippet using backend to `ensure_session` and `exec` a command; verify agent logs (use `limactl shell substrate journalctl -u substrate-world-agent -n 20`).

#### P1.4 Logging & Metrics

- Use `tracing` macros:
  ```rust
  tracing::info!(transport=?self.transport, "mac world forwarding established");
  tracing::warn!(error=%e, "mac world forwarding failed; falling back");
  ```
- Consider metrics stubs (future). For now ensure logs sufficient for debugging.

**Sanity Check (log evidence)**: Run snippet with `RUST_LOG=info` on mac, confirm logs show transport choice.

### Phase 2 — Shell Integration & Auto-Start

**Objective**: Shell uses platform context and routes commands via backend on macOS.

**Phase Gate**: Enter Phase 2 only after Phase 1 unit/manual tests pass and logs are captured. Confirm forwarding commands are known-good before touching shell code; do not alter transport ordering or detection heuristics beyond this plan.

#### P2.1 Platform Context Module

- Create `crates/shell/src/platform_world.rs` with:
  ```rust
  pub struct PlatformWorldContext {
      pub backend: Arc<dyn WorldBackend>,
      pub transport: WorldTransport,
      pub socket_path: PathBuf,
      pub ensure_ready: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
  }
  pub fn detect() -> anyhow::Result<PlatformWorldContext> { /* cfg(target_os) logic */ }
  ```
- `WorldTransport` is enum { Unix(PathBuf), Tcp { host, port }, Vsock { port } }.
- On mac, `detect()` instantiates `MacLimaBackend`, sets `ensure_ready` to closure calling `backend.ensure_vm_running()` + forwarding + capability check.

**Sanity Check (log evidence)**: Unit tests under `#[cfg(test)]` create fake contexts (simulate mac vs linux) and ensure detection returns expected variant.

#### P2.2 Auto-Start Replacement

- Update `ensure_world_agent_ready()` in shell to:
  ```rust
  let ctx = platform_world::detect()?;
  (ctx.ensure_ready)()?;
  env::set_var("SUBSTRATE_WORLD_SOCKET", ctx.socket_path.display().to_string());
  env::set_var("SUBSTRATE_WORLD_TRANSPORT", ctx.transport.as_str());
  store_context_globally(ctx);
  ```
- For Linux path, reuse existing logic (auto spawn local agent), but pass through same context struct.

**Sanity Check (log evidence)**: On Linux, run `cargo test -p substrate-shell`; ensure no regressions.

#### P2.3 Non-PTY Execution Refactor

- Replace manual HTTP request in `exec_non_pty_via_agent` with:
  ```rust
  let ctx = get_context()?; // from globally stored context
  let mut req = ExecuteRequest { ... };
  let client = ctx.agent_client()?;
  let resp = client.execute(req).await?;
  ```
- Convert async call using runtime (like backend). Ensure env + cwd mirror Linux path. Maintain redaction logic.

**Sanity Check (log evidence)**: Run `substrate -c 'echo hello'` on mac (with VM/agent ready) → output `hello`, no warnings, agent logs show command.

#### P2.4 PTY Path Updates

- Modify WebSocket connect logic to support `ctx.transport` cases:
  - Unix: use `tokio::net::UnixStream` and `tokio_tungstenite::client_async` with `tungstenite::handshake::client::Request` built from path.
  - Tcp: `ws://{host}:{port}/v1/stream` via `connect_async`.
  - Vsock: If using `vsock-proxy`, treat as TCP (`127.0.0.1:port`).
- Ensure proper error messages when handshake fails.

**Sanity Check (log evidence)**: `substrate --pty -c 'printf hi\n'` on mac prints `hi`. Run again after killing the forwarding process (e.g., `pkill -f vsock-proxy || pkill -f 'ssh .*substrate.sock'`) to confirm fallback message appears once.
Notes:
- When connecting over Unix sockets, ensure the WebSocket handshake targets `/v1/stream` and that your client library accepts a dummy host header.

#### P2.5 CLI & Env Overrides

Removed for strict Linux parity. Linux does not expose transport overrides for world routing — world is default-on, and env is used only to disable (`SUBSTRATE_WORLD=disabled`). macOS mirrors Linux 1:1 with no extra CLI/env knobs.

Note: Potential overrides (transport/socket/host/port) are cataloged in the backlog under NEEDS EVALUATED for future cross‑platform consideration. Any reintroduction must be implemented uniformly across Linux/macOS/Windows.

#### P2.6 Tests & QA

- Unit tests covering new modules.
- Integration tests (mac only, optional) using mocked agent to ensure proper routing.
- `cargo fmt`, `cargo clippy --all-targets --all-features` must pass.

**Sanity Check (log evidence)**: On mac host after provisioning, run `cargo test` (full workspace). All tests should pass.

### Phase 3 — Replay, Shim, Telemetry Parity

**Objective**: Replay/shim use platform-aware backend; telemetry available on mac.

**Phase Gate**: Advance to Phase 3 only after shell integration smoke checks succeed (non-PTY + PTY) with evidence stored. Do not refactor replay/shim beyond the scoped changes described here.

#### P3.1 Backend Factory

- Introduce `world::backend::factory() -> anyhow::Result<Arc<dyn WorldBackend>>` returning Linux or mac backend.
- Use once_cell to cache instance per process.

**Sanity Check (log evidence)**: Unit test verifying `factory()` returns mac backend when `cfg(target_os = "macos")`.

#### P3.2 Replay Integration

- Replace `LinuxLocalBackend::new()` usage with factory.
- Update `world_isolation_available()` to check mac context (e.g., `MacLimaBackend::new()?.ensure_vm_running()` > log error).
- Ensure `WorldSpec` uses `always_isolate = true` as before.

**Sanity Check (log evidence)**: On mac, run command to create file, capture span, run `substrate --replay-verbose --replay` → `fs_diff` lists file, logs show world strategy overlay. On Linux, confirm unchanged behavior.

#### P3.3 Shim + Telemetry Helper

- Shim: use backend factory to execute commands when world enabled.
- Telemetry (`collect_world_telemetry`): call backend `fs_diff` regardless of platform; ensure runtime bridging if needed.

**Sanity Check (log evidence)**: On mac, run `substrate-shim --shell 'echo shim'` → agent logs show execution. Check trace JSON (`~/.substrate/trace.jsonl`) includes `fs_diff` entries after commands.

#### P3.4 Tests

- Unit tests verifying telemetry helper works when `SUBSTRATE_WORLD_ID` set.
- Possibly add integration test for shim path (mac only).

### Phase 4 — QA, Automation, Documentation, Rollout

**Objective**: Polish, document, and validate with smoke tests.

**Phase Gate**: Do not begin Phase 4 until replay, shim, and telemetry validations from Phase 3 are logged. Any remaining TODOs must be resolved or explicitly deferred with owner approval before starting this phase.

#### P4.1 Smoke Script

- `scripts/mac/smoke.sh` performing full flow:
  ```sh
  set -euo pipefail
  scripts/mac/lima-warm.sh
  substrate -c 'echo smoke-nonpty'
  substrate --pty -c 'printf smoke-pty\n'
  span=$(substrate -c 'bash -lc "mkdir -p /tmp/world-mac && echo data > /tmp/world-mac/file.txt"' --trace-id)
  substrate --replay-verbose --replay "$span" | tee /tmp/world-mac-replay.json
  jq '.fs_diff | map(.path)' /tmp/world-mac-replay.json | grep '/tmp/world-mac/file.txt'
  ```
- Script verifies outputs, ensures `fs_diff` includes `/tmp/world-mac/file.txt`.

**Sanity Check (log evidence)**: Run script manually, gather logs.

#### P4.2 Documentation Updates

- Update `docs/WORLD.md` with mac section: architecture, helper scripts, fallback semantics, troubleshooting.
- Update `docs/INSTALLATION.md` to reference mac setup doc and smoke script.
- Ensure `docs/BACKLOG.md` item marked complete (if desired).

**Sanity Check (log evidence)**: Have fresh developer follow docs to set up mac world from scratch.

#### P4.3 Release Notes

- Outline new requirements (install Lima, run helper script), new env vars/flags, smoke test commands, fallback behavior.

#### P4.4 Final QA Checklist

- Run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test` on mac + linux.
- Run smoke script twice (once with default, once forcing fallback by killing forwarding mid-run). Document outputs.
- Capture agent logs demonstrating PTY + non-PTY runs.

#### P4.5 Doctor CLI Parity (macOS + Linux)

Objective: Provide a unified `substrate world doctor` experience across Linux and macOS. On macOS, the doctor should validate the Lima world and agent, mirroring Linux output style and JSON schema.

Tasks:
- CLI integration:
  - File: `crates/shell/src/lib.rs` (world_doctor_main)
  - Replace the current non-Linux stub with a `cfg(target_os = "macos")` path that performs mac checks and prints human/JSON outputs consistent with Linux.
  - Keep Linux behavior unchanged; ensure both platforms support `--json`.
- Mac checks to implement (reuse logic from `scripts/mac/lima-doctor.sh`):
  - `limactl` presence; version optional.
  - VM status via `limactl list substrate --json` (object shape). Expect `status == "Running"`.
  - Inside VM: `systemctl is-active substrate-world-agent` must be `active`.
  - Inside VM: agent socket exists `/run/substrate.sock` and capabilities respond via `curl --unix-socket`.
  - Host: `vsock-proxy` availability (WARN if missing; not a failure).
  - Disk usage (INFO only) and optional nft presence (INFO).
- Output parity:
  - Human-readable: use the same PASS/WARN style as Linux (`PASS |`, `WARN |`, `INFO |`).
  - JSON schema: include common keys and mac-specific nested keys:
    - `platform: "macos"`
    - `lima: { installed, vm_status, service_active, agent_socket, agent_caps_ok, vsock_proxy }`
    - `ok`: true only if VM running, service active, and agent responding.
- Exit codes:
  - macOS: return 0 when `ok == true`; otherwise non-zero (2 recommended), mirroring Linux semantics.
- Tests:
  - Add unit tests with command-exec abstraction to simulate outputs (feature-gated for mac).
  - Add snapshot tests for human output formatting if feasible.
- Docs:
  - Update README “World Doctor” section to document macOS support and JSON fields.
  - Note that `scripts/mac/lima-doctor.sh` remains as a developer aid but `substrate world doctor` is the canonical command.

Sanity Check (log evidence):
- `substrate world doctor` on mac prints PASS lines for VM running, service active, and agent responding.
- `substrate world doctor --json | jq .ok` prints `true` on a healthy mac.
- On Linux, both commands behave unchanged.

---

## 5. Incremental Validation Matrix

Use this matrix after each major task (phase sub-steps). Record pass/fail and remediation.

| Phase.Task | Command(s) | Expected | Notes |
|------------|------------|----------|-------|
| P0.1 | `limactl validate docs/dev/lima/substrate.yaml` | "valid" | Fails if syntax error. |
| P0.2 | `bash scripts/mac/lima-warm.sh --project-path $(pwd)` | VM enters `Running` | Timeout → check virtualization, logs. |
| P0.2 | `bash scripts/mac/lima-stop.sh` | No errors | VM stops gracefully. |
| P0.2 | `bash scripts/mac/lima-doctor.sh` | Reports status; PASS/FAIL clearly labeled | Should fail with guidance if VM down. |
| P0.4 | `limactl shell substrate curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities` | JSON | Confirms agent running. |
| P1.* | `cargo test -p world-mac-lima` | All tests pass | Includes new mocks. |
| P1.* manual | `cargo run --example mac_backend_smoke` | Command executed in VM; logs visible | Use harness provided in Appendix D. |
| P2.* | `substrate -c 'echo mac-hello'` | Output + agent log | Requires backend integrated. |
| P2.* | `substrate --pty -c 'printf mac-pty\n'` | Output + WS log entries | Validate resizing/signals. |
| P2.* fallback | Kill forwarding process, rerun command | Single warning; host execution occurs | Ensure logs show fallback reason. |
| P3.* | Replay command as described | `fs_diff` present | Compare with Linux run. |
| P4.* | `scripts/mac/smoke.sh` | End-to-end success | Document outputs/logs. |
| P4.5 | `substrate world doctor` (mac) | PASS lines for VM/agent/service | Unified CLI doctor parity |
| P4.5 | `substrate world doctor --json | jq .ok` | `true` on mac | JSON parity with Linux |

Maintain a spreadsheet or log capturing dates/results for accountability.

---

## 6. Observability & Logging Requirements

- Mac backend must log:
  - VM start attempt/success/failure with elapsed time.
  - Transport selection (VSock/SSH UDS/TCP) + reason.
  - Forwarding failures with exit code & stdout/stderr.
  - Agent call latency (optional but recommended).
- Shell must log when world ready, when fallback occurs, and include transport in debug logs.
- `lima-doctor` script should print colored PASS/FAIL messages (use `tput` if available) to highlight issues.

---

## 7. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Async bridging misused | Deadlock/panic | Keep runtime dedicated; use `block_on` carefully; add integration tests. |
| VSock unavailable | Falls back to slower SSH | Auto-detect and log; document performance considerations. |
| Agent binary out of date inside VM | Inconsistent behavior | Provision script copies fresh binary each warm; `lima-doctor` compares versions (`world-agent --version`). |
| SSH prompts (host key) | Auto-start blocked | Use `StrictHostKeyChecking=no` and `/dev/null` known hosts; document security trade-offs. |
| Orphaned forwarding processes | Socket/port conflicts | Manage via `ForwardingHandle` Drop implementation. |
| CI cannot run Lima | Lack automated coverage | Provide manual smoke script; consider self-hosted runners later. |

---

## 8. Rollout & Release Notes

**Release Notes Draft**:
- macOS now ships with Always-World support via Lima VM.
- Requirements: macOS 13+, install Lima (see docs).
- Setup Steps: `scripts/mac/lima-warm.sh`, copy agent binary (`cargo build -p world-agent --release` + `limactl copy`), run `scripts/mac/lima-doctor.sh`.
- Smoke Test: `scripts/mac/smoke.sh` (non-PTY, PTY, replay).
- New Env Vars: `SUBSTRATE_WORLD_SOCKET`, `SUBSTRATE_WORLD_TRANSPORT`, `SUBSTRATE_WORLD_HOST/PORT`.
- Troubleshooting: See `docs/dev/mac_world_setup.md` & `scripts/mac/lima-doctor.sh` output.

Rollout steps:
1. Merge provisioning/doc changes (Phase 0) so early adopters can experiment.
2. Feature-flag mac backend integration while stabilizing.
3. After QA & smoke tests, remove flag, announce in release notes.
4. Monitor inbound bug reports; maintain fallback instructions.

---

## 9. Appendices

### Appendix A — Lima Profile (`docs/dev/lima/substrate.yaml`)

```yaml
# Lima configuration for Substrate world backend on macOS
# Copy this file to ~/.lima/substrate.yaml or use scripts/mac/lima-warm.sh to substitute PROJECT path.

vmType: "vz"
rosetta:
  enabled: true
cpus: 4
memory: "4GiB"
disk: "20GiB"
mounts:
  - location: "$HOME"
    writable: false
  - location: "$PROJECT"
    writable: true
    mountPoint: "/src"
provision:
  - mode: system
    script: |
      #!/bin/sh
      set -eux
      apt-get update
      DEBIAN_FRONTEND=noninteractive apt-get install -y \
        nftables iproute2 libseccomp-dev curl jq git python3 python3-pip \
        build-essential dnsmasq openssh-server tmux unzip
      systemctl disable --now systemd-resolved || true
      cat >/etc/dnsmasq.d/substrate.conf <<'EOF'
      port=53
      listen-address=127.0.0.53
      bind-interfaces
      no-resolv
      server=1.1.1.1
      server=1.0.0.1
      cache-size=1000
      EOF
      echo 'nameserver 127.0.0.53' > /etc/resolv.conf
      sysctl -w kernel.unprivileged_userns_clone=1 || true
      mkdir -p /var/lib/substrate/overlay /run/substrate
      cat >/etc/systemd/system/substrate-world-agent.service <<'EOF'
      [Unit]
      Description=Substrate World Agent
      After=network-online.target
      Wants=network-online.target

      [Service]
      Type=simple
      ExecStart=/usr/local/bin/substrate-world-agent
      Restart=always
      RestartSec=5
      Environment=RUST_LOG=info
      RuntimeDirectory=substrate
      RuntimeDirectoryMode=0750
      StateDirectory=substrate
      StateDirectoryMode=0750
      WorkingDirectory=/var/lib/substrate
      StandardOutput=journal
      StandardError=journal
      NoNewPrivileges=yes
      ProtectSystem=strict
      ProtectHome=read-only
      ReadWritePaths=/var/lib/substrate /run /run/substrate
      CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
      AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

      [Install]
      WantedBy=multi-user.target
      EOF
      chmod 644 /etc/systemd/system/substrate-world-agent.service
      systemctl daemon-reload
      # Binary copied later; leave service disabled until Phase 0.4
```

### Appendix B — Helper Script Skeletons

`scripts/mac/lima-warm.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail
PROJECT_PATH=${1:-$(pwd)}
PROFILE=docs/dev/lima/substrate.yaml
TMP_PROFILE=$(mktemp)
trap 'rm -f "$TMP_PROFILE"' EXIT
PROJECT="$PROJECT_PATH" envsubst < "$PROFILE" > "$TMP_PROFILE"
limactl list substrate >/dev/null 2>&1 || limactl start --tty=false --name substrate "$TMP_PROFILE"
# Wait until running
timeout=120
while [ $timeout -gt 0 ]; do
  status=$(limactl list substrate --json | jq -r '.status // ""')
  if [ "$status" = "Running" ]; then
    echo "Lima VM 'substrate' is running."
    exit 0
  fi
  sleep 2
  timeout=$((timeout-2))
done
echo "ERROR: Lima VM did not reach Running state" >&2
exit 1
```

`scripts/mac/lima-stop.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail
if limactl list substrate >/dev/null 2>&1; then
  limactl stop substrate
  echo "Lima VM 'substrate' stopped."
else
  echo "Lima VM 'substrate' not defined or already stopped."
fi
```

`scripts/mac/lima-doctor.sh` skeleton:
```bash
#!/usr/bin/env bash
set -euo pipefail

failures=0
check() {
  local name="$1"
  shift
  if "$@"; then
    printf '\033[32m[PASS]\033[0m %s\n' "$name"
  else
    printf '\033[31m[FAIL]\033[0m %s\n' "$name"
    failures=$((failures+1))
  fi
}

check "Lima installed" command -v limactl
check "Virtualization available" test "$(sysctl -n kern.hv_support)" -eq 1
check "VM running" limactl list substrate >/dev/null 2>&1
check "SSH connectivity" limactl shell substrate uname -a
check "Agent socket" limactl shell substrate sudo test -S /run/substrate.sock
check "Agent capabilities" limactl shell substrate curl --fail --unix-socket /run/substrate.sock http://localhost/v1/capabilities
check "vsock-proxy available" command -v vsock-proxy

if [ $failures -ne 0 ]; then
  echo "Doctor detected $failures issue(s). See above output for remediation." >&2
  exit 1
fi

echo "All checks passed."
```

### Appendix C — Systemd Unit

Create `/etc/systemd/system/substrate-world-agent.service` inside the Lima guest with the following content:

```ini
[Unit]
Description=Substrate World Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info
RuntimeDirectory=substrate
RuntimeDirectoryMode=0750
StateDirectory=substrate
StateDirectoryMode=0750
WorkingDirectory=/var/lib/substrate
StandardOutput=journal
StandardError=journal
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/substrate /run /run/substrate
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
```

After copying the unit:
1. Set permissions: `chmod 644 /etc/systemd/system/substrate-world-agent.service`.
2. Reload systemd: `systemctl daemon-reload`.
3. Enable and start the service: `systemctl enable --now substrate-world-agent`.
4. Verify status: `systemctl status substrate-world-agent` and tail logs via `journalctl -u substrate-world-agent -n 50`.

The `RuntimeDirectory` directive creates `/run/substrate` on boot with `0700` semantics, avoiding stale sockets. `StateDirectory` ensures `/var/lib/substrate` exists with the correct ownership before the agent starts.

### Appendix D — Mac Backend Test Harness

Create `examples/mac_backend_smoke.rs` to manually exercise backend:
```rust
fn main() -> anyhow::Result<()> {
    let backend = world_mac_lima::MacLimaBackend::new()?;
    let spec = world_api::WorldSpec::default();
    let handle = backend.ensure_session(&spec)?;
    let req = world_api::ExecRequest {
        cmd: "bash -lc 'echo from-mac-backend'".into(),
        cwd: std::env::current_dir()?,
        env: std::env::vars().collect(),
        pty: false,
        span_id: None,
    };
    let res = backend.exec(&handle, req)?;
    println!("exit={} stdout={} stderr={}", res.exit, String::from_utf8_lossy(&res.stdout), String::from_utf8_lossy(&res.stderr));
    Ok(())
}
```
Run with `cargo run --example mac_backend_smoke` (mac only; ensure VM ready).

### Appendix E — Troubleshooting Cheat Sheet

| Symptom | Diagnosis Steps | Fix |
|---------|-----------------|-----|
| `limactl start` fails | Check virtualization: `sysctl kern.hv_support` | Enable in System Settings → Privacy & Security → Developer Tools |
| `ssh lima-substrate` prompts for password | Ensure provisioning enables SSH key auth; run `limactl shell substrate` once to accept host key | Use `StrictHostKeyChecking=no`, copy key manually |
| `lima-doctor` reports agent capabilities fail | Inside VM: `systemctl status substrate-world-agent`, `journalctl -u substrate-world-agent` | Restart service, copy fresh binary |
| Shell fallback warnings persist | Inspect logs for forwarding errors, run `scripts/mac/lima-doctor.sh`, verify transport logs | Fix underlying issue (VM down, forwarding crash) |
| Replay missing `fs_diff` | Confirm backend factory used (check logs), ensure agent returning diff | Run smoke script, check trace JSON |

### Appendix F — Environment Variables

| Variable | Default | Description | When to Override |
|----------|---------|-------------|------------------|
| `SUBSTRATE_WORLD` | Set to `enabled` by the shell once world-ready | Gate for running commands inside the world and collecting telemetry | Force to `disabled` when debugging host-only regressions |
| `SUBSTRATE_WORLD_ID` | Assigned by shell | Correlates spans/telemetry with the active world session | Never set manually; inspect for debugging only |
| `SUBSTRATE_WORLD_AGENT_BIN` | Auto-discovered (`substrate-world-agent` in `$PATH`) | Explicit path to agent binary the shell should spawn on Linux | Point at a custom build or staging path |
| `SUBSTRATE_WORLD_SOCKET` | `/run/substrate.sock` inside world; host override resolved by platform context | Unix socket path used for agent API calls | When forwarding to a non-default socket (e.g., SSH UDS fallback) |
| `SUBSTRATE_WORLD_TRANSPORT` | `auto` | Transport selector: `unix`, `tcp`, or `vsock` | Force a specific transport while diagnosing forwarding issues |
| `SUBSTRATE_WORLD_HOST` | `127.0.0.1` | Hostname for TCP transport | Point at remote Lima instance or custom port proxy |
| `SUBSTRATE_WORLD_PORT` | `17788` | Port used for TCP transport | Avoid conflicts or align with custom firewall rules |
| `SUBSTRATE_NETNS_GC_INTERVAL_SECS` | `600` | Agent-side interval (seconds) between namespace GC sweeps | Increase for high churn workloads; set `0` to disable |
| `SUBSTRATE_NETNS_GC_TTL_SECS` | `0` (disabled) | Optional TTL before namespaces eligible for GC | Configure when worlds should persist briefly after exit |
| `SUBSTRATE_WS_DEBUG` | unset | Enables extra PTY WebSocket diagnostics in the shell | Toggle to debug PTY tunnel failures |
| `RUST_LOG` | `info` | Standard Rust tracing filter respected by agent and shell | Raise to `debug`/`trace` while troubleshooting |

### Appendix G — Command Reference

- Start VM: `scripts/mac/lima-warm.sh`
- Stop VM: `scripts/mac/lima-stop.sh`
- Doctor: `scripts/mac/lima-doctor.sh`
- Copy binary: `limactl copy target/release/world-agent substrate:/usr/local/bin/`
- Agent logs: `limactl shell substrate journalctl -u substrate-world-agent -n 50`
- Test agent: `limactl shell substrate curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities`

---

## 10. Glossary & Quick Reference

| Term | Definition |
|------|------------|
| World | Reusable Linux execution environment with isolation & telemetry. |
| Lima | Lightweight VM manager for macOS, used to host Linux world-agent. |
| world-agent | Substrate binary providing `/v1/execute`, `/v1/stream` APIs inside world. |
| Forwarding | Mechanism to expose guest `/run/substrate.sock` to host via VSock or SSH tunnels. |
| PlatformWorldContext | Shell abstraction describing backend + transport per OS. |
| `lima-doctor` | Health script checking VM, agent, forwarding, logs. |

---

## 11. Next Steps (Before Coding)

1. Review this plan with stakeholders (Core team, macOS users, QA). Capture feedback.
2. Spin up Lima VM using Phase 0 instructions to validate provisioning baseline.
3. Create Git issues/tasks aligned to each phase/subtask.
4. Only start implementation after sign-off.

*End of plan. Review required before execution.*

---

## 12. Post‑Launch Profile Tuning (Final Task)

Goal: Ship a slim default Lima profile for end users while providing a documented, heavier dev profile.

Tasks:
- Create and publish `docs/dev/lima/substrate-dev.yaml` with larger CPU/RAM/disk for development.
- Change the default runtime profile (`docs/dev/lima/substrate.yaml`) to a slim configuration (e.g., 2 CPUs, 2GiB RAM, 8GiB disk) once mac backend is stable.
- Update docs to make the slim profile the default in helper scripts and guides, and clearly recommend `substrate-dev.yaml` for development use cases (build/test heavy workflows).

Sanity Check (log evidence):
- Fresh VM with slim profile passes `substrate world doctor` on mac.
- Dev profile starts cleanly and `scripts/mac/lima-doctor.sh` reports all PASS.

---

## Backlog — NEEDS EVALUATED (Cross‑Platform Parity Required)

The following knobs were explored and intentionally removed to maintain strict parity with Linux. If added later, they must be implemented identically across Linux/macOS/Windows (1:1:1) with clear UX, docs, and diagnostics.

- Transport/env knobs
  - `SUBSTRATE_WORLD_SOCKET`: explicit UDS path on host
  - `SUBSTRATE_WORLD_TRANSPORT`: `unix` | `tcp` | `vsock`
  - `SUBSTRATE_WORLD_HOST`, `SUBSTRATE_WORLD_PORT`: TCP endpoint selection
- CLI knobs
  - `--world-socket <PATH>`
  - `--world-transport <unix|tcp|vsock>`
  - `--world-host <HOST>`, `--world-port <PORT>`

Advantages:
- Targeted diagnostics and recovery when auto‑detect/forwarding misbehave.
- Easier testing of alternate tunnels (e.g., custom proxies) without code changes.
- Can simplify dev workflows where a fixed endpoint is desired.

Disadvantages / Risks:
- Divergence from “it just works” UX; users may set stale overrides and break routing.
- Increases surface area for bugs and docs; support burden rises.
- Requires full parity across platforms; Linux currently assumes UDS and auto‑spawn semantics.
- Security: host/TCP endpoints may be exposed incorrectly if misconfigured.

Recommendation:
- Keep defaults simple (auto detect, default‑on world, graceful fallback) and avoid overrides unless we commit to cross‑platform support and robust diagnostics. If adopted, enhance `substrate world doctor` to detect/report overrides and mismatches.
