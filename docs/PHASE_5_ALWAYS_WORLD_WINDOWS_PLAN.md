# Phase 5 — Windows "Always World" Implementation Plan

Status: Draft (awaiting review)  
Owner: Substrate Core  
Scope: Deliver Windows parity with the Linux/macOS "Always World" stack. Every entry point (`substrate`, PTY, replay, shim, agent API) must execute inside a WSL2-hosted Linux world and return identical telemetry, isolation semantics, and user experience. This document assumes no prior knowledge of the repository—follow each step exactly, gather evidence for every sanity check, and escalate if anything diverges from expectations.

> **Execution Guardrails**
> 1. Read this plan end-to-end before running any command.
> 2. Maintain an evidence log at `docs/project_management/logs/windows_always_world.md` capturing timestamps, commands, outputs, sanity-check results, remediation steps, and reviewer sign-off. Append entries chronologically.
> 3. After each numbered step, complete the associated sanity check immediately and paste evidence (command output, screenshots, log snippet) into the log before advancing.
> 4. If any command fails or produces unexpected output, STOP. Diagnose using the troubleshooting subsections, record the root cause and corrective action, then re-run. Do not skip steps or reorder tasks.
> 5. Do not substitute tools, change script names/paths, or introduce alternative workflows without explicit plan-owner approval.

---

## Table of Contents

1. Background & Rationale  
   1.1 Current Platform Comparison  
   1.2 WSL2 Constraints & Assumptions  
2. Host Prerequisites & Environment Setup  
   2.1 Platform Support Matrix  
   2.2 Required Windows Features  
   2.3 Tooling & Accounts  
   2.4 Repository Checkout  
3. Architecture Overview (Target State)  
   3.1 Component Diagram  
   3.2 Invariants & Cross-Platform Parity Goals  
4. Phase 0 — Bootstrap Tooling (Scripts + Docs)  
   P0.1 Directory & Log Preparation  
   P0.2 Provisioning Script (`provision.sh`)  
   P0.3 PowerShell Helper Scripts  
   P0.4 Operator Documentation (`wsl_world_setup.md`)  
   P0.5 Troubleshooting Catalogue  
5. Phase 1 — Forwarder Service (Rust)  
   P1.1 Crate Skeleton  
   P1.2 Forwarder Implementation  
   P1.3 Integration with Scripts  
   P1.4 Forwarder Logging & Rotation  
   P1.5 Troubleshooting  
6. Phase 2 — World Backend Integration  
   P2.1 `world-windows-wsl` Crate  
   P2.2 Backend Factory Wiring  
   P2.3 Shell Non-PTY Integration  
   P2.4 Telemetry & Span Alignment  
   P2.5 Troubleshooting  
7. Phase 3 — PTY via ConPTY + WebSocket  
   P3.1 ConPTY Wrapper  
   P3.2 WS Bridge & Reedline Guard  
   P3.3 Signal Handling  
   P3.4 Troubleshooting  
8. Phase 4 — Path Translation & Replay Parity  
   P4.1 Path Utilities  
   P4.2 Replay Alignment  
   P4.3 Span Schema Update  
   P4.4 Troubleshooting  
9. Phase 5 — Validation Matrix, CI, & Releases  
   P5.1 Smoke Script  
   P5.2 CI Workflow  
   P5.3 Documentation Refresh  
   P5.4 Release Checklist  
10. Appendices  
   A. Provisioning Script (Full listing)  
   B. PowerShell Scripts (Full listings)  
   C. Forwarder Source (Main + Support)  
   D. Backend Source Skeleton  
   E. ConPTY Implementation  
   F. Path Utilities  
   G. Smoke Script  
   H. CI Workflow  
   I. Doctor Output Samples  
   J. Troubleshooting Catalogue (Expanded)  
   K. Evidence Log Template  
11. Glossary  
12. Revision History

---

## 1. Background & Rationale

### 1.1 Current Platform Comparison

| Capability | Linux (Phase 4.5) | macOS (Phase 4.5) | Windows (Current) | Target (Phase 5) |
|------------|-------------------|-------------------|-------------------|------------------|
| Default world enforcement | ✅ Shell auto-starts world, PTY + non-PTY routed through `world-agent` | ✅ Shell ensures Lima VM + WLS transport | ❌ Host execution fallback | ✅ Shell ensures WSL world on startup |
| Agent transport | Unix socket (`/run/substrate.sock`) | VSock → UDS → TCP fallback | n/a | Named pipe ↔ UDS, optional TCP bridge |
| Telemetry (trace.jsonl) | `world_id`, `fs_diff`, `scopes_used` | Same as Linux | Partial (no world info) | Identical JSON schema with Windows path supplement |
| Replay | Uses `world-api` overlay, includes `fs_diff` | Same | Host re-exec only | Uses WSL backend, dual path display |
| Tooling | Podman docs + shell scripts | Lima scripts + doctor + smoke | Absent | PowerShell warm/stop/doctor/smoke |
| PTY support | PTY inside world via WS | PTY inside Lima via WS | Partial (no input) | ConPTY + WS parity |

### 1.2 WSL2 Constraints & Assumptions
- Windows host must support WSL2 with virtualization enabled.
- WSL distro `substrate-wsl` based on Ubuntu 24.04 LTS.
- Shared filesystem via `/mnt/c` (9P) — acceptable for dev workflows (performance considerations noted in Appendix J).
- `substrate-world-agent` binaries for Linux reused inside WSL2.
- Named pipe bridging (or TCP fallback) used to communicate with Unix socket.
- Operators can run PowerShell scripts with `RemoteSigned` policy.

---

## 2. Host Prerequisites & Environment Setup

### 2.1 Platform Support Matrix
| Requirement | Minimum | Validation Command | Evidence to capture |
|-------------|---------|--------------------|---------------------|
| Windows version | Windows 11 22H2 (build ≥ 22621) or Windows 10 22H2 | `winver` (GUI) or `Get-ComputerInfo | Select OsName, OsVersion` | Screenshot or text in log |
| Virtualization | Enabled in BIOS/UEFI | `systeminfo | Select-String "Virtualization"` | Copy line showing `Yes` |
| WSL feature | Enabled | `Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux` | Command output |
| Virtual Machine Platform | Enabled | `Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform` | Command output |
| WSL2 default | Set to 2 | `wsl --status` | Output showing default version |
| Disk space | ≥ 20 GB free on system drive | `Get-PSDrive C` | Output snippet |
| User privileges | Local admin for feature installs | N/A | Mention in log |

### 2.2 Required Windows Features (Detailed Procedure)
1. **Enable features**
   ```powershell
   Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux -NoRestart
   Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform -NoRestart
   Restart-Computer
   ```
   > Troubleshooting: If `Enable-WindowsOptionalFeature` fails with DISM error 0x800f080c, ensure Windows Update service is running and try again. Document issue in log.

2. **Install WSL2 kernel update** (if not already on latest):
   ```powershell
   wsl --update
   ```
   Sanity check: `wsl --status` prints `Kernel version: 5.x` or higher.

### 2.3 Tooling & Accounts
- Install Git, Rust toolchain, and latest PowerShell:
  ```powershell
  winget install --id Git.Git -e
  winget install --id Rustlang.Rustup -e
  winget install --id Microsoft.PowerShell -e
  ```
- Configure execution policy for current user:
  ```powershell
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
  ```
- Verify installations:
  ```powershell
  git --version
  rustup show
  pwsh --version
  ```
  Sanity check: capture version outputs.

### 2.4 Repository Checkout
1. Choose workspace directory without spaces (recommended `C:\workspace`).
2. Clone repository:
   ```powershell
   Set-Location C:\workspace
   git clone https://github.com/atomize-hq/substrate.git
   ```
3. Set branch (if not already on feature branch):
   ```powershell
   Set-Location C:\workspace\substrate
   git checkout feature/world-isolation
   ```
4. Sanity check: `Test-Path Cargo.toml` returns `True`; log root commit hash (`git rev-parse HEAD`).

---

## 3. Architecture Overview (Target State)

### 3.1 Component Diagram
```
┌─────────────────────────────────────────────────────────────────────────┐
│                             Windows Host                                │
│                                                                         │
│  substrate.exe (Rust)                                                   │
│  ├── cli/main                                                           │
│  ├── platform_world::windows                                            │
│  │     ├── ensure_world_ready()                                         │
│  │     └── translate_path()                                             │
│  ├── Non-PTY executor → HTTP POST /v1/execute                           │
│  ├── PTY executor → WebSocket /v1/stream                                │
│  └── span builder → trace.jsonl                                         │
│                                                                         │
│  substrate-forwarder.exe (Rust)                                         │
│  ├── Named pipe listener (\\.\pipe\substrate-agent)                    │
│  ├── Optional TCP (127.0.0.1:17788)                                     │
│  └── Unix socket client (/run/substrate.sock via WSL interop)           │
│                                                                         │
│  PowerShell tooling (warm/stop/doctor/smoke)                            │
└─────────────────────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│          WSL2 Distro: substrate-wsl (Ubuntu 24.04 LTS)                  │
│                                                                         │
│  /usr/local/bin/substrate-world-agent (systemd service)                 │
│  ├── Unix socket: /run/substrate.sock                                   │
│  ├── Isolation stack: namespaces, cgroups v2, nftables, overlayfs       │
│  ├── Telemetry: fs_diff, scopes_used, span IDs                           │
│  └── Replay support                                                     │
│                                                                         │
│  Shared project mount: /mnt/c/workspace/substrate                       │
│  Logs: journalctl -u substrate-world-agent                              │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Invariants & Parity Goals
- `SUBSTRATE_WORLD` defaults to `enabled` on Windows, matching Linux/mac behavior.
- Fallback on failure prints exactly one WARN per shell invocation (`substrate: warn: world unavailable (observe-only): …`).
- `trace.jsonl` entries share schema across platforms; Windows adds `display_path` supplement where applicable.
- Doctor and smoke scripts exist for Windows analogous to mac’s Lima toolkit.
- Replay uses the same `WorldSpec` fields (`reuse_session`, `always_isolate`, etc.).

---

## 4. Phase 0 — Bootstrap Tooling (Scripts + Docs)

**Objective**: Provision reproducible WSL environment, helper scripts, and operator documentation. No Rust or binary changes yet.

### P0.1 Directory & Log Preparation
1. Create directories and evidence log (idempotent):
   ```powershell
   Set-Location C:\workspace\substrate
   New-Item -ItemType Directory -Force scripts\windows | Out-Null
   New-Item -ItemType Directory -Force docs\dev\wsl | Out-Null
   New-Item -ItemType Directory -Force docs\project_management\logs | Out-Null
   if (-not (Test-Path docs\project_management\logs\windows_always_world.md)) {
       New-Item -ItemType File docs\project_management\logs\windows_always_world.md -Force | Out-Null
   }
   ```
2. Sanity check: `Get-ChildItem docs\dev\wsl`, `Get-Content docs/project_management/logs/windows_always_world.md` (should be empty). Record in log.

### P0.2 Provisioning Script (`docs/dev/wsl/provision.sh`)
1. Copy Appendix A contents exactly into the file (use editor or PowerShell here-string). Ensure Unix line endings (LF). Example command:
   ```powershell
   @'
   #!/usr/bin/env bash
   set -euo pipefail

   export DEBIAN_FRONTEND=noninteractive
   apt-get update
   apt-get install -y \
       nftables \
       iproute2 \
       libseccomp-dev \
       curl \
       jq \
       git \
       python3 \
       python3-pip \
       build-essential \
       dnsmasq \
       openssh-server \
       unzip \
       ca-certificates

   install -d -m 0700 /run/substrate
   install -d -m 0755 /etc/substrate
   install -d -m 0755 /var/log/substrate
   install -d -m 0755 /var/lib/substrate

   cat <<'UNIT' >/etc/systemd/system/substrate-world-agent.service
   [Unit]
   Description=Substrate World Agent
   After=network.target

   [Service]
   Type=simple
   ExecStart=/usr/local/bin/substrate-world-agent --socket /run/substrate.sock
   Restart=always
   User=root
   Group=root
   RuntimeDirectory=substrate
   RuntimeDirectoryMode=0700
   StandardOutput=journal
   StandardError=journal

   [Install]
   WantedBy=multi-user.target
   UNIT

   systemctl daemon-reload
   systemctl enable substrate-world-agent.service
   '@ | Set-Content -Path docs/dev/wsl/provision.sh -NoNewline
   ```
2. Validate formatting: `wsl --system -- bash -lc "cd /mnt/c/workspace/substrate && bash -n docs/dev/wsl/provision.sh"` (requires WSL base distro). If WSL not yet installed, skip and note in log.
3. Sanity check: `Get-Content docs/dev/wsl/provision.sh` matches Appendix A exactly.

### P0.3 PowerShell Helper Scripts (Full Listings)
Create the following files using the appendices:
- `scripts/windows/wsl-warm.ps1`
- `scripts/windows/wsl-stop.ps1`
- `scripts/windows/wsl-doctor.ps1`
- `scripts/windows/wsl-smoke.ps1` (placeholder until Phase 5, but create file with header and TODO comment)

Each script should match Appendix B exactly. Recommended approach is to copy the relevant code block from Appendix B into the target path using a here-string or editor of your choice, for example:
```powershell
@'
<contents from Appendix B>
'@ | Set-Content -Path scripts/windows/wsl-warm.ps1 -NoNewline
# Repeat for other scripts
```
After creation:
```powershell
Get-ChildItem scripts/windows/*.ps1 | ForEach-Object { Write-Output "Script: $($_.Name)"; Get-Content $_ | Select-Object -First 5 }
```
Sanity checks:
- Run dry-run warm script: `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .) -WhatIf`
- Run doctor script (will warn about missing distro/forwarder): `pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName substrate-wsl -Verbose`
Capture outputs in log.

### P0.4 Operator Documentation (`docs/dev/wsl_world_setup.md`)
1. Create Markdown document mirroring structure of mac Lima guide. Include sections:
   - Prerequisites (with validation commands)
   - First-time setup (warm script, sample outputs)
   - Updating agent binary
   - Running doctor script (include sample PASS output from Appendix I)
   - Smoke test (once implemented)
   - Troubleshooting table (see Appendix J)
2. Use `npx markdownlint-cli docs/dev/wsl_world_setup.md` to ensure formatting.
3. Sanity check: attach PDF or screenshot of rendered doc for review (optional). Note in evidence log.

### P0.5 Troubleshooting Catalogue
Create `docs/dev/wsl_world_troubleshooting.md` with extended scenarios (from Appendix J). Cross-link from setup doc. Sanity check: file exists, lint passes.

---

## 5. Phase 1 — Forwarder Service (Rust)

**Objective**: Implement host-side `substrate-forwarder` bridging Windows named pipe/TCP to WSL Unix socket. This must be functional before backend integration.

### P1.1 Crate Skeleton
1. Create crate:
   ```powershell
   cargo new crates/forwarder --bin
   ```
2. Replace `Cargo.toml` with content below:
```toml
[package]
name = "substrate-forwarder"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
bytes = "1"
clap = { version = "4", features = ["derive"] }
ctrlc = "3"
futures = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal", "net"] }
tokio-util = "0.7"
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
uds_windows = "1"
```
3. Add dependency entry to workspace `Cargo.toml` if necessary. Sanity check: `cargo check -p substrate-forwarder`.

### P1.2 Forwarder Implementation
Overwrite `crates/forwarder/src/main.rs` with Appendix C. Ensure supporting modules (`bridge.rs`, `logging.rs`, `pipe.rs`, `tcp.rs`, `wsl.rs`) copied exactly. Important behaviors:
- CLI arguments: `--distro`, `--pipe`, `--tcp-bridge`, `--log-dir`, `--run-as-service`.
- Named pipe creation with proper security descriptor allowing current user.
- WSL socket connection via `uds_windows::UnixSeqpacketConn::connect_to_path`. Path pattern: `\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopyXYZ\...` not accessible; instead, execute `wsl.exe -d <distro> -- cat` bridging is used (see Appendix C). Implementation uses asynchronous tasks to forward bytes via `tokio::io::copy_bidirectional`.
- JSON structured logs with rotation at 10 MB using `tracing-appender`.
- Graceful shutdown on Ctrl+C (`ctrlc::set_handler`).

Sanity checks:
1. `cargo test -p substrate-forwarder` (Appendix C includes mock tests verifying CLI parsing and pipe creation).
2. Run forwarder help:
   ```powershell
   cargo run -p substrate-forwarder -- --help
   ```
   Capture usage output.
3. Dry-run bridging (without WSL running) to ensure informative error message. Document in log.

### P1.3 Integration with Scripts
1. Update `scripts/windows/wsl-warm.ps1` to build and launch forwarder:
   - After provisioning, run `cargo build -p substrate-forwarder --release` if binary absent.
   - `Start-Process` forwarding binary with configured arguments (Appendix B details). Store PID in `%LOCALAPPDATA%\Substrate\forwarder.pid`.
   - Wait for pipe availability with timeout (30s). On timeout, print remediation instructions.
2. Update `scripts/windows/wsl-stop.ps1` to terminate forwarder gracefully (read PID, `Stop-Process`, remove PID file).
3. Update `scripts/windows/wsl-doctor.ps1` to check pipe health by attempting connection (use `Test-Path` and custom `Test-NamedPipe` function from Appendix B) and verify forwarder log freshness.
4. Sanity check: run `pwsh -File scripts/windows/wsl-warm.ps1 -Verbose`. On success, log tail of `%LOCALAPPDATA%\Substrate\logs\forwarder.log` showing `Listening on named pipe`. Follow with doctor script (should report PASS for forwarder).

### P1.4 Forwarder Logging & Rotation
- Ensure log directory creation (Appendix B scripts). Confirm rotation via `tracing-appender` configuration (append `max_log_files = 5`, `rotation = Daily`). Log snippet in evidence file.

### P1.5 Troubleshooting
Document scenarios in Appendix J (e.g., `Access is denied` due to pipe permission, WSL not running, pipe in use). Update doctor script to detect and display targeted remediation.

---

## 6. Phase 2 — World Backend Integration

**Objective**: Implement `world-windows-wsl` backend mirroring Linux/mac behavior and integrate with shell for non-PTY commands.

### P2.1 `world-windows-wsl` Crate
1. Create crate: `cargo new crates/world-windows-wsl --lib`.
2. Use Cargo template from Appendix D. Key features:
   - `WindowsWslBackend::new()` reads distro name from env (`SUBSTRATE_WSL_DISTRO`, default `substrate-wsl`), sets up `AgentClient` pointing at named pipe (`Transport::Pipe`).
   - `ensure_session`: checks cache, invokes warm script when needed using `Command::new("pwsh")` (Appendix D shows `WarmCmd` helper). On success, returns `WorldHandle { id: world_id }` and caches it.
   - `exec`: uses `tokio::runtime::Handle::current().block_on(self.agent.execute(request))` to bridge async call. Build `ExecRequest` with stringified command, environment, `span_id` (if provided).
   - `fs_diff`: calls `self.agent.get_trace` and converts into `FsDiff`, adding `display_path` via `paths_windows::to_windows_display`.
   - `apply_policy`: stub to forward restrictions (wired once policy support extended; currently `Ok(())`).
   - Logging: use `tracing` to log warm script invocation, command success/failure, diff translation.
3. Add integration tests if possible (mock AgentClient stub returning canned responses). See Appendix D for test harness using `MockAgent` implementing trait with `Mutex<Vec<Request>>`.
4. Sanity check: `cargo test -p world-windows-wsl` (should pass with mocks).

### P2.2 Backend Factory Wiring
1. Modify `crates/world-backend-factory/Cargo.toml` to add dependency: `world-windows-wsl = { path = "../world-windows-wsl" }` under `[target.'cfg(target_os = "windows")'.dependencies]`.
2. In `src/lib.rs`, add Windows branch:
   ```rust
   #[cfg(target_os = "windows")]
   {
       let backend = world_windows_wsl::WindowsWslBackend::new()?;
       return Ok(Arc::new(backend));
   }
   ```
3. Add test `#[cfg(target_os = "windows")] #[test] fn factory_returns_backend()` verifying success. For non-Windows builds, mark with `#[cfg_attr(not(target_os = "windows"), ignore)]`.
4. Sanity check: `cargo check` on Windows host.

### P2.3 Shell Non-PTY Integration
1. Create `crates/shell/src/platform_world/windows.rs` using Appendix D guidance. Functions:
   - `ensure_world_ready(args: &Cli) -> Result<Option<String>>` (handles `--no-world`, env overrides, calls backend `ensure_session`, sets env vars).
   - `get_backend() -> Result<Arc<dyn WorldBackend>>` (memoized using `lazy_static` or `OnceLock`).
   - `to_exec_request(...)` converting command, env, cwd using `paths_windows::to_wsl_path`.
2. Update `crates/shell/src/platform_world/mod.rs` to include Windows module behind `#[cfg(target_os = "windows")]`.
3. In `crates/shell/src/lib.rs`, adjust startup sequence (similar to Linux path) to call `platform_world::windows::ensure_world_ready(&cli)` and to route non-PTY commands through backend, handling fallback with warning if `ExecResult` fails.
4. Ensure span builder includes `fs_diff` from `ExecResult` directly; only call fallback `collect_world_telemetry` when `fs_diff.is_none()`.
5. Add tests under `#[cfg(target_os = "windows")]` verifying command execution path (use mocks). For manual testing, run `cargo test --target x86_64-pc-windows-msvc`.
6. Sanity check: manual `substrate -c "echo hello"` prints once, `trace.jsonl` latest entry includes `"world_id"` and `"component": "shell"`. Log snippet.

### P2.4 Telemetry & Span Alignment
- Update `crates/trace/src/lib.rs` (if necessary) to handle optional `display_path` field (document in schema). Ensure Windows path conversion occurs before writing trace.
- Confirm `fs_diff` structure remains backward-compatible (fields optional). Update docs in Appendices as necessary.

### P2.5 Troubleshooting
Add Windows-specific troubleshooting to Appendix J (e.g., warm script fails due to `wsl.exe` not found, path translation errors). Update doctor script to detect and surface these cases.

---

## 7. Phase 3 — PTY via ConPTY + WebSocket

**Objective**: Achieve full interactive parity using Windows ConPTY front-end and agent `/v1/stream` WebSocket backend.

### P3.1 ConPTY Wrapper
1. Add dependency to `crates/shell/Cargo.toml` under Windows target:
```toml
[target."cfg(target_os = \"windows\")".dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_Console",
    "Win32_System_Threading",
    "Win32_System_Pipes",
    "Win32_System_Diagnostics_Debug",
    "Win32_Storage_FileSystem"
] }
```
2. Create file `crates/shell/src/windows_pty.rs` with implementation from Appendix E. Key steps:
   - Create pipes (`CreatePipe`) for input/output.
   - Initialize ConPTY via `CreatePseudoConsole`.
   - Spawn child process using `CreateProcessW` with `STARTUPINFOEXW`, hooking up ConPTY handles.
   - Wrap handles in `tokio::io::unix::AsyncFd`-like facility (use `tokio::io::windows::named_pipe`) or custom asynchronous polling with `mio` integration.
   - Provide `write`, `read`, and `resize` methods.
3. Add tests (if feasible) using simple command `cmd.exe /C "echo conpty"` (requires asynchronous reading; tests may need to be `#[cfg(test)]` and use blocking read).
4. Sanity check: run `cargo test -p substrate-shell windows_pty::tests::test_conpty_echo`.

### P3.2 WebSocket Bridge & Reedline Integration
1. In shell PTY execution path (`execute_command` or `run_pty_command`), detect Windows target and instantiate `WindowsPtySession`.
2. Connect to `world-agent` via forwarder:
   - If TCP bridge enabled (`SUBSTRATE_FORWARDER_TCP=1`), connect to `ws://127.0.0.1:17788/v1/stream` using `tokio_tungstenite`.
   - Otherwise, use named pipe stream with `tokio::net::UnixStream` via `uds_windows` wrappers (Appendix E includes helper `PipeStream::connect`).
3. Forward ConPTY output to WebSocket frames and vice versa. Map messages according to agent API (`start`, `stdin`, `stdout`, `resize`, `signal`).
4. Integrate with REPL using `reedline::Signal::Success` and `suspend_guard` to avoid prompt corruption (same pattern as Linux/mac).
5. Ensure span builder receives exit code and any `scopes_used` from agent message, finishing span accordingly.
6. Sanity check: manual tests
   - `substrate --pty -c "bash -lc 'printf "TYPE> "; read line; echo OK:$line'"` → type input, expect echo.
   - `substrate` REPL → `:pty python -q` → interactive session works, `Ctrl+C` interrupts.
   Record outputs and confirm no double prompts.

### P3.3 Signal Handling
- Implement Windows console handler (`SetConsoleCtrlHandler`) in shell to forward `CTRL_C_EVENT` and `CTRL_BREAK_EVENT` to agent via WebSocket (`{"type":"signal","sig":"INT"}` or `"TERM"`). Add tests by running long command and pressing Ctrl+C.
- Ensure forwarder handles abrupt disconnect gracefully (log WARN, attempt cleanup).

### P3.4 Troubleshooting
Document issues such as ConPTY creation failure (`ERROR_ACCESS_DENIED`), WebSocket handshake failure, double echo. Add to Appendix J and doctor script (e.g., check if forwarder socket is reachable). Update instructions for enabling `UseUnicodeOutput` if necessary.

---

## 8. Phase 4 — Path Translation & Replay Parity

**Objective**: Ensure Windows hosts provide dual-path representations in spans and that replay uses the WSL backend.

### P4.1 Path Utilities
1. Add file `crates/common/src/paths_windows.rs` with functions from Appendix F. Implementation details:
   - Drive letters: `C:\path` → `/mnt/c/path`
   - UNC paths: `\\server\share\dir` → `/mnt/unc/server/share/dir`
   - Use `std::path::Component` iteration to build normalized POSIX path.
   - Provide reverse function `to_windows_display` converting `/mnt/c/path` back to `C:\path` and `/mnt/unc/server/...` to UNC path.
2. Export functions in `crates/common/src/paths.rs` using `#[cfg(target_os = "windows")] pub use paths_windows::{...};`.
3. Write comprehensive tests (Appendix F includes `#[cfg(test)] mod tests` with several scenarios). Run `cargo test -p substrate-common paths_windows`.

### P4.2 Replay Alignment
1. Modify `crates/replay/src/replay.rs` to use backend factory on Windows, similar to Linux path:
   - Build `WorldSpec` with `always_isolate = true`, `reuse_session = true`.
   - Convert `cwd` and env to WSL paths using utilities from previous step.
   - Use returned `ExecResult` to update `fs_diff`; add Windows display path when printing.
2. Ensure `collect_world_telemetry` fallback works for PTY replays (if agent doesn’t return diff).
3. Sanity check: capture span by running `substrate -c "python - <<'PY'..."` that writes file, then run `substrate replay <span>`. Verify output shows both `/mnt/c/...` and `C:\...` paths.

### P4.3 Span Schema Update
- Update `docs/TRACE.md` to mention optional `fs_diff.display_path` for Windows.
- Ensure JSON schema or TypeScript typings (if any) reflect optional field.
- Add tests verifying JSON serialization includes new field only on Windows.

### P4.4 Troubleshooting
Add to Appendix J cases like incorrect path translation (document expected vs actual). Doctor script should include sample translation check (`pwsh -File scripts/windows/wsl-doctor.ps1` prints sample `C:\ -> /mnt/c` mapping).

---

## 9. Phase 5 — Validation Matrix, CI, & Releases

### P5.1 Smoke Script (`scripts/windows/wsl-smoke.ps1`)
1. Replace placeholder with full script from Appendix G (performing doctor run, non-PTY/PTY/replay tests, forwarder restart). Ensure script writes summary table and exits non-zero on failure.
2. Run smoke script on fresh warm environment:
   ```powershell
   pwsh -File scripts/windows/wsl-smoke.ps1 -Verbose
   ```
   Capture output and attach to evidence log. Confirm created files cleaned up afterward.

### P5.2 CI Workflow (`.github/workflows/ci-windows-world.yml`)
1. Create workflow from Appendix H. Ensure steps include checkout, Rust toolchain install, caching, `cargo fmt --check`, `cargo test`, `wsl-warm.ps1`, `wsl-smoke.ps1`.
2. Configure runner (self-hosted or GitHub) with WSL support and required permissions. Document instructions in release checklist.
3. Sanity check: run workflow manually via GitHub UI. Attach logs in evidence.

### P5.3 Documentation Refresh
- Update `docs/WORLD.md` Windows section to summarize architecture and tooling.
- Update `docs/TRACE.md` and `docs/TELEMETRY.md` with Windows notes.
- Add quick-start steps to `README.md` under Windows section.
- Ensure `docs/BACKLOG.md` updates reflect completion of Windows parity tasks and note any follow-up items.
- Sanity check: run `npx markdownlint-cli docs/WORLD.md docs/TRACE.md README.md`.

### P5.4 Release Checklist
1. Build release binaries (`cargo build --release`) for Windows host and WSL agent.
2. Package forwarder, shell, world-agent, and scripts into installer or zip.
3. Update release notes summarizing new Windows support, prerequisites, known limitations.
4. Create acceptance test matrix (table of features vs Windows/mac/Linux).
5. Obtain approvals from security and ops stakeholders.
6. Tag release once all validations pass.

---

## 10. Appendices

### Appendix A — Provisioning Script (`docs/dev/wsl/provision.sh`)
```bash
#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y \
    nftables \
    iproute2 \
    libseccomp-dev \
    curl \
    jq \
    git \
    python3 \
    python3-pip \
    build-essential \
    dnsmasq \
    openssh-server \
    unzip \
    ca-certificates

install -d -m 0700 /run/substrate
install -d -m 0755 /etc/substrate
install -d -m 0755 /var/log/substrate
install -d -m 0755 /var/lib/substrate

cat <<'UNIT' >/etc/systemd/system/substrate-world-agent.service
[Unit]
Description=Substrate World Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-agent --socket /run/substrate.sock
Restart=always
User=root
Group=root
RuntimeDirectory=substrate
RuntimeDirectoryMode=0700
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable substrate-world-agent.service
```

### Appendix B — PowerShell Scripts (Full Listings)

**`scripts/windows/wsl-warm.ps1`**
```powershell
#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\..' | Select-Object -ExpandProperty Path),
    [switch]$WhatIf
)

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit 1 }

Write-Info "Starting wsl-warm for distro '$DistroName'"

$projectPath = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
Write-Info "Project path: $projectPath"

if (-not (Test-Path (Join-Path $projectPath 'Cargo.toml'))) {
    Write-ErrorAndExit "Project path does not contain Cargo.toml"
}

# Ensure WSL installed
$wslStatus = & wsl --status 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "WSL not available. Run 'wsl --install' first."
}

# Import distro if missing
$distroList = & wsl -l -v | Out-String
if ($distroList -notmatch [regex]::Escape($DistroName)) {
    Write-Info "Importing distro '$DistroName'"
    $tempTar = Join-Path $env:TEMP "ubuntu-noble-wsl-amd64.tar.gz"
    Write-Info "Downloading Ubuntu rootfs..."
    Invoke-WebRequest -Uri "https://cloud-images.ubuntu.com/wsl/noble/current/ubuntu-noble-wsl-amd64-wsl.rootfs.tar.gz" -OutFile $tempTar
    $installDir = Join-Path $env:LOCALAPPDATA "substrate\wsl"
    New-Item -ItemType Directory -Force $installDir | Out-Null
    & wsl --import $DistroName $installDir $tempTar --version 2
    Remove-Item $tempTar
}

if ($WhatIf) {
    Write-Warn "WhatIf mode enabled - skipping provisioning"
    exit 0
}

# Copy provisioning script and run
$hostProvisionPath = Join-Path $projectPath 'docs\dev\wsl\provision.sh'
if (-not (Test-Path $hostProvisionPath)) {
    Write-ErrorAndExit "Provisioning script not found at $hostProvisionPath"
}

Write-Info "Updating package cache and running provision script"
& wsl -d $DistroName -- bash -lc "set -euo pipefail; cp /mnt/c/$(($projectPath -replace ':', '') -replace '\\','/')/docs/dev/wsl/provision.sh /tmp/provision.sh && chmod +x /tmp/provision.sh && sudo /tmp/provision.sh"
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Provision script failed"
}

# Build world-agent if absent
$agentHostPath = Join-Path $projectPath 'target\release\world-agent.exe'
if (-not (Test-Path $agentHostPath)) {
    Write-Info "Building world-agent (release)"
    cargo build -p world-agent --release
}

# Copy agent binary into WSL
Write-Info "Copying world-agent into WSL"
$agentUnixPath = ($projectPath -replace ':', '') -replace '\\','/'
& wsl -d $DistroName -- bash -lc "set -euo pipefail; sudo cp /mnt/c/$agentUnixPath/target/release/world-agent.exe /usr/local/bin/substrate-world-agent && sudo chmod 755 /usr/local/bin/substrate-world-agent"

# Restart service
Write-Info "Restarting substrate-world-agent service"
& wsl -d $DistroName -- bash -lc "sudo systemctl restart substrate-world-agent.service"
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Failed to restart agent service"
}

# Build forwarder if needed
$forwarderHostPath = Join-Path $projectPath 'target\release\substrate-forwarder.exe'
if (-not (Test-Path $forwarderHostPath)) {
    Write-Info "Building substrate-forwarder (release)"
    cargo build -p substrate-forwarder --release
}

# Launch forwarder
Write-Info "Launching forwarder"
$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\logs'
New-Item -ItemType Directory -Force $logDir | Out-Null
$pipePath = "\\.\pipe\substrate-agent"
$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Warn "Forwarder PID file exists; attempting cleanup"
    $existingPid = Get-Content $pidFile
    Stop-Process -Id $existingPid -ErrorAction SilentlyContinue
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}
$forwarderProcess = Start-Process -FilePath $forwarderHostPath -ArgumentList "--distro", $DistroName, "--pipe", $pipePath, "--log-dir", $logDir -WindowStyle Hidden -PassThru
Set-Content $pidFile -Value $forwarderProcess.Id

# Wait for pipe
Write-Info "Waiting for forwarder pipe $pipePath"
$timeout = [DateTime]::UtcNow.AddSeconds(30)
while (-not (Test-Path $pipePath)) {
    if ([DateTime]::UtcNow -gt $timeout) {
        Write-ErrorAndExit "Forwarder pipe not available after 30s"
    }
    Start-Sleep -Seconds 1
}
Write-Info "Forwarder pipe ready"

Write-Info "Warm complete"
```
**`scripts/windows/wsl-stop.ps1`**
```powershell
#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

param(
    [string]$DistroName = 'substrate-wsl'
)

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit 1 }

Write-Info "Stopping forwarder and WSL distro '$DistroName'"

$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Info "Attempting to stop forwarder recorded in PID file"
    try {
        $pid = [int](Get-Content $pidFile)
        Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        Write-Info "Forwarder PID $pid terminated"
    } catch {
        Write-Warn "Unable to terminate PID in $pidFile: $_"
    }
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}

# Clean any stray forwarder processes owned by user
Get-Process -Name substrate-forwarder -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*$($env:LOCALAPPDATA)*substrate-forwarder.exe" } |
    ForEach-Object {
        Write-Warn "Stopping stray forwarder process Id=$($_.Id)"
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }

$pipePath = "\\.\pipe\substrate-agent"
if (Test-Path $pipePath) {
    Write-Warn "Named pipe $pipePath still exists; it should disappear after forwarder shutdown"
}

Write-Info "Terminating WSL distro $DistroName"
& wsl --terminate $DistroName 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Warn "wsl --terminate returned non-zero (distro may already be stopped)"
}

# Wait for distro to report Stopped
$timeout = [DateTime]::UtcNow.AddSeconds(60)
while ([DateTime]::UtcNow -lt $timeout) {
    $listing = & wsl -l -v | Out-String
    if ($listing -notmatch [regex]::Escape($DistroName)) {
        Write-Info "Distro $DistroName no longer listed"
        break
    }
    if ($listing -match "$DistroName\s+\d+\s+Stopped") {
        Write-Info "Distro state is Stopped"
        break
    }
    Start-Sleep -Seconds 2
}

if (Test-Path $pipePath) {
    Write-Warn "Pipe $pipePath still present. If subsequent runs fail, delete pipe or reboot."
}

Write-Info "WSL stop complete"
```

**`scripts/windows/wsl-doctor.ps1`**
```powershell
#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

param(
    [string]$DistroName = 'substrate-wsl',
    [switch]$Json
)

function New-Result($Name, $Status, $Detail, $Remediation) {
    [PSCustomObject]@{
        Name        = $Name
        Status      = $Status
        Detail      = $Detail
        Remediation = $Remediation
    }
}

function Invoke-Check {
    param(
        [string]$Name,
        [scriptblock]$Probe,
        [string]$Remediation
    )
    try {
        $detail = & $Probe
        if ($detail -is [System.Array]) { $detail = ($detail | Out-String).Trim() }
        elseif ($detail -isnot [string]) { $detail = [string]$detail }
        New-Result $Name 'PASS' ($detail.Trim()) $Remediation
    } catch {
        New-Result $Name 'FAIL' ($_.Exception.Message.Trim()) $Remediation
    }
}

$results = @()

$results += Invoke-Check 'Virtualization' {
    $line = systeminfo | Select-String 'Virtualization'
    if (-not $line) { throw 'Virtualization status not reported' }
    if ($line.ToString() -notmatch 'Yes') { throw "Virtualization disabled: $line" }
    $line.ToString().Trim()
} 'Enable VT-x/AMD-V in BIOS/UEFI'

$results += Invoke-Check 'WSL Feature' {
    $feature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
    if ($feature.State -ne 'Enabled') { throw "State: $($feature.State)" }
    "Microsoft-Windows-Subsystem-Linux: $($feature.State)"
} 'Enable Windows Subsystem for Linux feature and reboot'

$results += Invoke-Check 'VirtualMachinePlatform Feature' {
    $feature = Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
    if ($feature.State -ne 'Enabled') { throw "State: $($feature.State)" }
    "VirtualMachinePlatform: $($feature.State)"
} 'Enable VirtualMachinePlatform feature and reboot'

$results += Invoke-Check 'WSL Status' {
    $status = & wsl --status 2>&1
    if ($LASTEXITCODE -ne 0) { throw $status }
    $status
} 'Run "wsl --install" or repair WSL'

$results += Invoke-Check "Distro $DistroName" {
    $listing = & wsl -l -v | Out-String
    if ($LASTEXITCODE -ne 0) { throw $listing }
    if ($listing -notmatch [regex]::Escape($DistroName)) { throw 'Distro not found' }
    ($listing -split "`n") | Where-Object { $_ -match $DistroName } | ForEach-Object { $_.Trim() }
} 'Import distro via scripts/windows/wsl-warm.ps1'

$results += Invoke-Check 'Forwarder PID' {
    $pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'
    if (-not (Test-Path $pidFile)) { throw 'PID file not found' }
    $pid = [int](Get-Content $pidFile)
    $proc = Get-Process -Id $pid -ErrorAction Stop
    "PID $pid ($($proc.Path))"
} 'Run wsl-warm.ps1 to launch forwarder'

$results += Invoke-Check 'Forwarder Pipe' {
    $pipePath = "\\.\pipe\substrate-agent"
    if (-not (Test-Path $pipePath)) { throw 'Pipe not available' }
    $pipePath
} 'Restart forwarder via wsl-warm.ps1'

$results += Invoke-Check 'Agent Socket' {
    & wsl -d $DistroName -- bash -lc 'test -S /run/substrate.sock'
    if ($LASTEXITCODE -ne 0) { throw 'Socket /run/substrate.sock missing' }
    '/run/substrate.sock present'
} 'Verify substrate-world-agent systemd service is running'

$results += Invoke-Check 'Agent Capabilities' {
    $output = & wsl -d $DistroName -- bash -lc "curl --unix-socket /run/substrate.sock -s http://localhost/v1/capabilities"
    if ($LASTEXITCODE -ne 0) { throw $output }
    $json = $output | ConvertFrom-Json
    "version=$($json.version) features=$($json.features -join ',')"
} 'Inspect agent logs via journalctl -u substrate-world-agent'

$results += Invoke-Check 'nftables' {
    $output = & wsl -d $DistroName -- bash -lc 'nft list tables'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -First 5) -join '; '
} 'Install nftables package inside WSL distro'

$results += Invoke-Check 'Disk (/)' {
    $output = & wsl -d $DistroName -- bash -lc 'df -h /'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -Last 1).Trim()
} 'Free disk space or expand WSL virtual disk'

$results += Invoke-Check 'Agent Logs' {
    & wsl -d $DistroName -- bash -lc 'journalctl -u substrate-world-agent -n 20 --no-pager'
} 'Investigate errors shown in journal'

if ($Json) {
    $results | ConvertTo-Json -Depth 3
} else {
    $results | Format-Table -AutoSize
}

if ($results.Status -contains 'FAIL') {
    Write-Host "One or more checks FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "All checks PASS" -ForegroundColor Green
}
```

**`scripts/windows/wsl-smoke.ps1`**
```powershell
#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\..' | Select-Object -ExpandProperty Path),
    [switch]$SkipWarm
)

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Block
    )
    Write-Host "[STEP] $Name" -ForegroundColor Cyan
    try {
        & $Block
        Write-Host "[PASS] $Name" -ForegroundColor Green
    } catch {
        Write-Host "[FAIL] $Name - $_" -ForegroundColor Red
        throw
    }
}

if (-not $SkipWarm) {
    Invoke-Step "Warm environment" {
        pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    }
}

Invoke-Step "Doctor checks" {
    pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName $DistroName | Out-Host
}

$tracePath = Join-Path $env:USERPROFILE '.substrate\trace.jsonl'
if (-not (Test-Path $tracePath)) {
    New-Item -ItemType File -Force $tracePath | Out-Null
}

Invoke-Step "Non-PTY command produces world span" {
    $marker = [guid]::NewGuid().ToString()
    substrate -c "python - <<'PY'\nimport pathlib\npathlib.Path('win_smoke.txt').write_text('$marker')\nPY" | Out-Host
    $entry = Get-Content $tracePath | Select-Object -Last 1 | ConvertFrom-Json
    if (-not $entry.world_id) { throw 'world_id missing from span' }
    if (-not ($entry.fs_diff.writes -match 'win_smoke.txt')) { throw 'fs_diff does not mention win_smoke.txt' }
}

Invoke-Step "PTY command" {
    $output = substrate --pty -c "bash -lc 'echo pty-smoke'"
    if ($output -notmatch 'pty-smoke') { throw 'PTY output missing expected text' }
}

Invoke-Step "Replay" {
    $last = Get-Content $tracePath | Select-Object -Last 1 | ConvertFrom-Json
    if (-not $last.span_id) { throw 'span_id missing' }
    $replay = substrate replay $last.span_id 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Replay failed: $replay" }
}

Invoke-Step "Forwarder restart resilience" {
    pwsh -File scripts/windows/wsl-stop.ps1 -DistroName $DistroName | Out-Host
    pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    substrate -c "echo restart-smoke" | Out-Host
}

Write-Host "Smoke suite completed successfully" -ForegroundColor Green
```

**`scripts/windows/wsl-stop.ps1`**, **`scripts/windows/wsl-doctor.ps1`**, and **`scripts/windows/wsl-smoke.ps1`** are provided in full in Appendix B (omitted here for brevity due to length; ensure copies exactly match there).

### Appendix C — Forwarder Source (Main + Support)
Full source files for `crates/forwarder/` including `main.rs`, `bridge.rs`, `logging.rs`, `pipe.rs`, `tcp.rs`, `wsl.rs`, and tests. (Due to length, refer to repository path `docs/appendices/windows_forwarder/` for raw files created during plan execution.)

### Appendix D — Backend Source Skeleton
Includes `lib.rs`, `agent_mock.rs`, and tests. Provided as reference for implementation; actual code resides under `crates/world-windows-wsl/` once created.

### Appendix E — ConPTY Implementation
Full Windows PTY module with FFI definitions and helpers.

### Appendix F — Path Utilities
Complete module with conversion functions and tests.

### Appendix G — Smoke Script
Full content in `scripts/windows/wsl-smoke.ps1` once implemented.

### Appendix H — CI Workflow
Full YAML for GitHub Actions workflow.

### Appendix I — Doctor Output Samples
Example PASS output:
```
[substrate/windows doctor]
Virtualization: PASS (Enabled)
WSL status: PASS (Default version 2, kernel 5.15.133.1)
Distro substrate-wsl: PASS (Running)
Forwarder pipe: PASS (\\.\pipe\substrate-agent)
Agent socket: PASS (/run/substrate.sock exists)
Agent capabilities: PASS (features: execute, pty_streaming, trace_retrieval)
Nftables: PASS
Logs: PASS (no errors in last 100 lines)
```

### Appendix J — Troubleshooting Catalogue
Detailed table covering issues such as feature enable failures, WSL import errors, forwarder permission issues, ConPTY failures, path translation edge cases. Include symptoms, likely causes, remediation steps, and commands to verify resolution.

### Appendix K — Evidence Log Template
Provide Markdown template for entries (timestamp, step, command, output, sanity result, reviewer signature).

---

## 11. Glossary
- **WSL2**: Windows Subsystem for Linux v2.
- **Forwarder**: Windows binary bridging named pipe/TCP to agent Unix socket.
- **ConPTY**: Windows pseudo-terminal API enabling interactive shells.
- **World**: Isolated execution environment managed by `substrate-world-agent`.
- **FsDiff**: Filesystem diff structure returned by agent.
- **Doctor script**: Diagnostic PowerShell script verifying environment health.
- **Smoke script**: End-to-end validation script ensuring parity features run correctly.

---

## 12. Revision History
| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 0.1 | 2025-??-?? | TBD | Initial draft (expanded to match mac plan detail) |
| ... | ... | ... | ... |
