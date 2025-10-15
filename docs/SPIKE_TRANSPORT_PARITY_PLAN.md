# SPIKE: Cross-Platform Agent Transport Parity

Status: Draft (awaiting review)  
Owner: Substrate Core (Windows strike team lead TBD)  
Created: 2025-09-23  
Related Plans: docs/PHASE_5_ALWAYS_WORLD_WINDOWS_PLAN.md  
Tracking IDs: T-011, T-012, (new) T-013 once filed
Additional Windows Addendum: docs/dev/windows_host_transport_plan.md

---

## Phase Status Matrix
| Phase | Host Platform | Status | Last Updated | Reviewer | Evidence Log Anchor |
|-------|---------------|--------|--------------|----------|----------------------|
| W | Windows (WSL2) | Complete | 2025-09-30T16:47:43-04:00 | @spenser | windows_always_world.md#W |
| M | macOS (Lima) | Complete | 2025-10-15T13:13:55Z | @spenser | macos_always_world.md#M |
| L | Linux (Native) | Pending | _tbd_ | _tbd_ | linux_always_world.md#L |
| Final Verification | All | Pending | _tbd_ | _tbd_ | windows_always_world.md#Final |

Update this table at the start and completion of each phase.

---

## Execution Guardrails
1. Read this spike end-to-end before running any command.
2. Maintain evidence logs for every platform:
   - Windows: `docs/project_management/logs/windows_always_world.md`
   - macOS: `docs/project_management/logs/macos_always_world.md`
   - Linux: `docs/project_management/logs/linux_always_world.md`
   Create the macOS/Linux logs if they do not exist.
3. After each numbered step, capture: timestamp, command(s), exit code, key output, sanity-check result, remediation, reviewer placeholder, branch/commit, and handoff notes.
4. Stop immediately if a command fails. Diagnose using the troubleshooting catalogue, record remediation, and only then retry.
5. Do not reorder tasks, substitute tooling, or change script names without plan-owner approval.
6. Keep file encoding ASCII unless the target file already uses UTF-8 with BOM; preserve native line endings.

---

## 1. Background & Objective
Windows workspace builds fail because several host crates import Unix-only transports (`hyperlocal`, `tokio::net::UnixStream`). Our goal is to design and implement a cross-platform transport layer that:
- Chooses Named Pipes on Windows hosts.
- Uses Unix domain sockets on Unix hosts (including WSL and Lima VMs).
- Preserves the world agent's Unix socket semantics inside Linux environments while allowing proxy access from all hosts.
- Removes platform-specific `cfg` guards that simply bypass functionality.

The spike delivers a linear, reproducible guide for an operator with zero project context to land the refactor across three consecutive sessions (Windows → macOS → Linux) with final cross-platform validation.

---

## 2. Global Prerequisites
- GitHub access to the `substrate` repository with write permissions.
- Rust toolchain 1.79+ with `cargo`, `rustup`, and platform targets installed.
- Node.js 18+ for documentation linting (`npx markdownlint-cli`).
- Shell utilities: `pwsh` 7.x on Windows, `bash`/`zsh` on macOS & Linux, `rg` (ripgrep) on all hosts.
- Ability to install/verify virtualization components (WSL2, Lima, systemd-based Linux distro).
- Credentials to access required package repositories (Canonical WSL images, Homebrew taps, apt repositories).

Before each phase, synchronize with the main integration branch and ensure the working tree is clean (or stash local work). Example commands:

```pwsh
# Windows PowerShell
cd C:\Users\<user>\Documents\__Project_Code\substrate
git status
```

```bash
# macOS/Linux
cd ~/Documents/__Project_Code/substrate
git status
```

Record the output in the relevant evidence log.

---

## 3. Terminology Snapshot
- **Forwarder**: Windows-side bridge that proxies host I/O into the WSL world via Named Pipe/TCP.
- **Transport Abstraction**: New layer in `agent-api-client` providing platform-neutral HTTP connectivity to `world-agent`.
- **Evidence Log**: Markdown file capturing commands, outputs, sanity results, branch/commit, and handoff notes for auditing.
- **Smoke Suite**: Platform-specific automation validating warmup, doctor checks, PTY, replay, and resilience.

---

## 4. Phase W - Windows Transport Layer Refactor (Session 1)

### Entry Criteria Checklist
- Windows 11 Pro/Enterprise host with virtualization enabled.
- WSL2 feature installed with distro `substrate-wsl` provisioned per `docs/dev/wsl_world_setup.md`.
- PowerShell 7.x available as `pwsh`.
- Evidence log `docs/project_management/logs/windows_always_world.md` exists.

Document each checklist item in the Windows evidence log. Update the Phase Status Matrix (Phase W → In Progress) with timestamp.

#### Step W0 - Host Validation
1. Verify virtualization and optional features:
   ```pwsh
   systeminfo | Select-String "Virtualization"
   Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
   Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
   ```
   Note: optional feature queries require elevation; document if not available.
2. Confirm WSL status and distro:
   ```pwsh
   wsl --status
   wsl -l -v
   ```
   Ensure `Default Version: 2` and `substrate-wsl` listed.

Record outputs and mark sanity PASS/FAIL.

#### Step W1 - Repository Sync & Baseline Failure Capture
1. Synchronize repository:
   ```pwsh
   git pull --rebase
   ```
   Resolve conflicts; note remediation.
2. Capture Windows baseline failure:
   ```pwsh
   cargo check
   ```
   Expect hyperlocal failure referencing `tokio::net::UnixStream`. Paste the failure snippet in the log.

#### Step W2 - Architecture Preparation
1. Author the architecture sketch at `docs/dev/transport_parity_design.md` outlining host connectors, forwarder targets, world-agent listeners, and telemetry expectations.
2. Run `npx markdownlint-cli docs/dev/transport_parity_design.md` and log the result.

#### Step W3 - Implement Transport Abstraction in `agent-api-client`
1. **Create connector module**
   - Replace `crates/agent-api-client/src/transport.rs` with a module directory (`src/transport/`).
   - Implement `Transport`, `TransportMode`, and an async `Connector` trait with concrete connectors for Unix sockets, TCP, and Windows named pipes.
   - Ensure each connector exposes `mode()`, `endpoint()`, `build_uri()`, `prepare_request()`, and `execute()`.
   - Add platform-specific unit tests covering descriptions, keepalive support, and endpoint formatting.
2. **Wire connector into `AgentClient`**
   - Update `crates/agent-api-client/src/lib.rs` to construct connectors via `build_connector` and store them behind an `Arc<dyn Connector>`.
   - Provide helper methods `transport_mode()`, `transport_endpoint()`, and `transport()` for telemetry.
   - Route `get`/`post` helpers through the connector trait; maintain existing constructors (`unix_socket`, `tcp`).
3. **Adjust dependencies**
   - Add `async-trait = "0.1"` to `crates/agent-api-client/Cargo.toml`.
   - Move `hyperlocal` under `[target."cfg(unix)".dependencies]` and run `cargo metadata` to confirm resolution.
4. **Format and test**
   ```pwsh
   cargo fmt
   cargo check -p agent-api-client
   cargo test -p agent-api-client
   ```
   Resolve warnings before proceeding.
5. **Capture telemetry requirements**
   - Note the need for `transport.mode` and optional endpoint metadata ahead of Step W7.

#### Step W4 - Forwarder Target Bridge
1. **Update configuration schema**
   - Extend `crates/forwarder/src/config.rs` to support `target.mode = "tcp" | "uds"`, optional TCP port, and UDS path.
   - Support environment override `SUBSTRATE_FORWARDER_TARGET`; log chosen mode at startup.
2. **Implement dual-target runtime**
   - Update `bridge`, `pipe`, and `tcp` modules to honor the selected target (Named Pipe → TCP or Named Pipe → UDS) with backoff and cancellation.
3. **Enhance logging/diagnostics**
   - Emit structured logs including `forwarder.target`, pipe path, and downstream endpoint.
   - Update `scripts/windows/wsl-doctor.ps1` expectations to surface target mode.
4. **Tests**
   ```pwsh
   cargo check -p forwarder
   cargo test -p forwarder
   ```
   Add/refresh integration tests for both target modes.

#### Step W5 - World Agent Dual Listener
1. **Enable loopback TCP endpoint**
   - Update `crates/world-agent/src/main.rs` to read `SUBSTRATE_AGENT_TCP_PORT` and start a TCP listener bound to `127.0.0.1:<port>` alongside the Unix socket.
   - Ensure graceful shutdown covers both listeners.
2. **Provisioning updates**
   - Adjust `docs/dev/wsl/provision.sh` (and systemd unit) to export the TCP port when running under Windows.
   - Document firewall requirements (localhost only) and verify nftables settings remain valid.
3. **Tests**
   ```pwsh
   cargo check -p world-agent
   cargo test -p world-agent
   ```
   Run inside WSL if runtime coverage is required.

#### Step W6 - Host Integration & Forwarder Alignment
1. **Forwarder loop & downstream target**
   - Adopt the Tokio named-pipe accept pattern (`first_pipe_instance(true)`, pre-create the next instance, explicit `disconnect()` / `FlushFileBuffers`).
   - Default the forwarder to bridge named pipe → TCP (`127.0.0.1:<port>` inside WSL); keep the Unix-socket path behind a feature flag for follow-up.
2. **Warm tooling updates**
   - Replace `Test-Path` with a `WaitNamedPipe` + client probe in `scripts/windows/wsl-warm.ps1` and document the behaviour in the setup/troubleshooting guides.
3. **Host crates (`substrate-shell`, `host-proxy`, factories)**
   - Consume the shared `AgentClient` builder, ensure telemetry reflects `named_pipe` or `tcp`, and gate Unix-only dependencies behind `cfg(unix)`.
4. **Checks**
   ```pwsh
   cargo check -p substrate-forwarder
   cargo test -p substrate-forwarder
   cargo check -p substrate-shell
   cargo check -p host-proxy
   cargo check -p world-backend-factory
   ```pwsh
   cargo check -p substrate-shell
   cargo check -p host-proxy
   cargo check -p world-backend-factory
   ```

#### Step W7 - Telemetry & Documentation
1. **Telemetry schema** - Update `substrate-trace` (and consumers) to include `transport.mode` plus optional endpoint metadata; add serialization tests.
2. **Doctor & smoke scripts**
   - Standardize the Windows pipe health check on `scripts/windows/pipe-status.ps1` (status-line only).
   - Acceptance: probe prints `HTTP/1.1 200 OK` within ≤ 8 seconds against the canonical pipe `\\.\pipe\substrate-agent`.
   - Update `scripts/windows/wsl-smoke.ps1` to use `pipe-status.ps1` for the HTTP 200 check and keep restart resilience.
3. **Documentation**
   - Use single-line, quoted canonical PipePath examples: `'\\.\pipe\substrate-agent'`.
   - Reference `scripts/windows/pipe-status.ps1` as the recommended probe in setup and troubleshooting docs.
   - Run `npx markdownlint-cli` across edited docs.

#### Step W8 - Validation & Evidence Capture
1. **Workspace verification**
   ```pwsh
   cargo fmt
   cargo check
   cargo test
   ```
2. **Runtime validation**
   ```pwsh
   pwsh -File scripts/windows/wsl-stop.ps1 -DistroName substrate-wsl
   pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
   pwsh -File scripts/windows/wsl-smoke.ps1 -DistroName substrate-wsl
   ```
3. **Telemetry inspection**
   ```pwsh
   Get-Content "$env:USERPROFILE\.substrate\trace.jsonl" | Select-Object -Last 3
   ```
   Confirm `transport.mode = "named_pipe"` and endpoint metadata.
4. **Evidence logging** – Append findings to `docs/project_management/logs/windows_always_world.md`, capture branch/commit, and prepare the Phase M kickoff prompt.

#### Step W9 - Phase Exit Checklist
- [ ] Workspace builds/tests passing (`cargo check`, `cargo test`).
- [ ] Smoke suite passes with updated transport (`scripts/windows/wsl-smoke.ps1`).
- [ ] Telemetry inspected (`transport.mode` observed in trace).
- [ ] Docs & troubleshooting updated and linted.
- [ ] Evidence log entry completed with branch/commit hash and handoff notes.
- [ ] Code pushed to shared branch (`git push`).
- [x] Phase Status Matrix updated (Phase W → Complete with timestamp & reviewer).
- [ ] Next session kickoff prompt prepared (see Prompt Templates section) and attached to evidence log.

---

## 5. Phase M - macOS Transport Validation (Session 2)

### Entry Criteria
- Access to a macOS Ventura/Sonoma workstation with admin rights.
- Lima VM configured per Phase 4.5 documentation (`lima/substrate.yaml`).
- Phase W changes merged or rebased onto the working branch.
- Evidence log `docs/project_management/logs/macos_always_world.md` available.
- Kickoff prompt from the Windows session reviewed.

Update the Phase Status Matrix (Phase M → In Progress) when starting and record the timestamp.

#### Step M0 - Host Preparation
1. Verify virtualization support (Apple Silicon or Intel VT-x):
   ```bash
   sysctl -n hw.optional.arm64 || sysctl -a | grep -i vmx
   ```
2. Confirm Lima tooling is installed:
   ```bash
   limactl ls
   ```
3. Install or update required tooling:
   ```bash
   brew bundle --file=docs/dev/macos/Brewfile
   ```

#### Step M1 - Repository Sync
1. Synchronize repository:
   ```bash
   cd ~/Documents/__Project_Code/substrate
   git status
   git pull --rebase
   ```

#### Step M2 - Environment Alignment
1. Start the Lima instance and verify mounts:
   ```bash
   limactl start substrate
   limactl shell substrate -- bash -lc "cd /Users/$(whoami)/Documents/__Project_Code/substrate && git status"
   mount | grep /Users
   ```
   Record Lima kernel version and mount output.

#### Step M3 - Build & Test on macOS Host
1. Run host-side checks:
   ```bash
   cargo fmt
   cargo check
   cargo test
   ```
2. Perform targeted package checks as needed:
   ```bash
   cargo check -p agent-api-client
   cargo check -p substrate-shell
   cargo check -p host-proxy
   ```

#### Step M4 - Lima Environment Smoke Validation
1. Warm the environment and execute diagnostics:
   ```bash
   ./scripts/mac/lima-warm.sh
   ./scripts/mac/lima-doctor.sh
   ```
2. Run the smoke suite:
   ```bash
   ./scripts/mac/smoke.sh
   ```
   Capture logs showing `transport.mode = "unix"` and PTY success.

#### Step M5 - Telemetry & Replay Verification
1. Inspect trace metadata:
   ```bash
   tail -n 3 ~/.substrate/trace.jsonl
   ```
   Confirm `transport.mode = "unix"`.
2. Replay the latest span:
   ```bash
   span=$(tail -n 1 ~/.substrate/trace.jsonl | jq -r '.span_id')
   substrate replay "$span"
   ```

#### Step M6 - Documentation & Handoff Prep
1. Update macOS docs if workflows changed (`docs/dev/macos_lima_setup.md`, etc.).
2. Run `npx markdownlint-cli` on edited docs.
3. Summarize issues, environment quirks, and tool versions in the evidence log.
4. Prepare the Phase L kickoff prompt with branch/commit, risks, and required reading.

#### Step M7 - Phase Exit Checklist
- [ ] Workspace builds/tests passing on macOS.
- [ ] Lima smoke suite passes.
- [ ] Trace shows `transport.mode = "unix"`.
- [ ] Docs updated if mac-specific changes introduced; lint passes.
- [ ] Evidence log entry complete with branch/commit & handoff notes.
- [ ] Code pushed; matrix updated (Phase M → Complete, timestamp & reviewer).
- [ ] Next session kickoff prompt prepared and logged.

---

## 6. Phase L - Linux Transport Validation (Session 3)

### Entry Criteria
- Native Linux workstation (Ubuntu 24.04 LTS recommended) or VM with systemd access.
- Phase W/M changes applied to the working branch.
- Evidence log `docs/project_management/logs/linux_always_world.md` created.
- Kickoff prompt from the macOS session reviewed.

Update the Phase Status Matrix (Phase L → In Progress) when starting and record the timestamp.

#### Step L0 - Host Preparation
1. Verify virtualization modules / CPU features:
   ```bash
   lscpu | grep -E 'Virtualization|Vendor ID'
   ```
2. Install required packages:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev libsystemd-dev
   ```

#### Step L1 - Repository Sync
1. Synchronize repository:
   ```bash
   cd ~/Documents/__Project_Code/substrate
   git status
   git pull --rebase
   ```

#### Step L2 - Build & Test on Linux Host
1. Run workspace checks:
   ```bash
   cargo fmt
   cargo check
   cargo test
   ```
2. Execute targeted package checks:
   ```bash
   cargo check -p agent-api-client
   cargo check -p world-backend-factory
   cargo check -p substrate-shell
   ```

#### Step L3 - World Agent Validation
1. Confirm world-agent service status:
   ```bash
   systemctl status substrate-world-agent
   ```
2. Run smoke validations (or equivalent script):
   ```bash
   substrate -c "echo linux-transport"
   substrate --pty -c "bash -lc 'echo linux-pty'"
   ```
   Capture outputs and ensure trace metadata shows `transport.mode = "unix"`.

#### Step L4 - Security Audit
1. Verify TCP listener binds only to loopback when enabled:
   ```bash
   ss -ltnp | grep substrate-world-agent
   ```
2. Review firewall/nftables rules:
   ```bash
   sudo nft list ruleset | grep substrate
   ```

#### Step L5 - Documentation & Handoff Prep
1. Update Linux-specific docs (`docs/dev/linux_world_setup.md`, troubleshooting) if steps changed.
2. Run `npx markdownlint-cli` on modified docs.
3. Summarize observations, risks, and outstanding TODOs in the evidence log.
4. Prepare the Final Verification kickoff prompt referencing Windows/macOS/Linux evidence anchors.

#### Step L6 - Phase Exit Checklist
- [ ] Workspace builds/tests passing on Linux.
- [ ] Smoke/telemetry checks captured with `transport.mode = "unix"`.
- [ ] Security audit notes logged (loopback bind confirmed).
- [ ] Evidence log entry complete with branch/commit & handoff notes.
- [ ] Code pushed; matrix updated (Phase L → Complete, timestamp & reviewer).
- [ ] Final verification kickoff prompt prepared.

---

## 7. Final Cross-Platform Verification
After Phase L, perform a final validation pass on all three platforms in order W → M → L using the finalized branch/tag.

#### Final Verification Exit Checklist
- [ ] Git sync done on all platforms; evidence logs updated.
- [ ] `cargo fmt`, `cargo check`, `cargo test` pass on each host.
- [ ] Smoke suites succeed with trace transport metadata confirmed.
- [ ] Replay verified on recent span for each platform.
- [ ] Phase Status Matrix row "Final Verification" set to Complete with timestamp & reviewer.
- [ ] Consolidated summary added to Windows evidence log referencing macOS/Linux sections.
- [ ] Pull request prepared with links to evidence logs.

---

## 8. Deliverables & Handoff
- Updated source code implementing cross-platform transport abstraction.
- Forwarder, world agent, and host tooling using the new interface.
- Documentation updates (`docs/dev/*`, troubleshooting catalogue, design doc).
- Evidence logs for Windows, macOS, Linux with complete command/output history, handoff notes, and kickoff prompts.
- Final summary entry in `docs/project_management/logs/windows_always_world.md` referencing macOS/Linux logs and confirming multi-platform PASS.
- Pull request containing implementation commits, documentation updates, evidence links, and a reviewer checklist covering transport behaviour, security, and telemetry parity.

---

## 9. Risk Register & Mitigations
- **Transport abstraction bugs** – Mitigate via unit/integration tests per connector and Windows CI coverage.
- **Security exposure via TCP listener** – Enforce localhost binding with automated tests; document firewall expectations.
- **Environment drift between sessions** – Mandatory handoff notes plus kickoff prompts ensure continuity.
- **Trace schema changes** – Coordinate with telemetry consumers before altering span fields; update docs/tests accordingly.

---

## 10. Open Questions
1. Should the world agent support vsock (for future hypervisor integrations) alongside TCP? Track in follow-up RFC.
2. Do we need automated rollback procedures if the transport abstraction fails in production? Determine during design review.
3. How will CI enforce transport parity going forward? Engage DevInfra to extend the test matrix.

---

## 11. Next Actions Checklist
- [ ] Schedule design review covering transport abstraction, forwarder changes, and security posture.
- [ ] Assign platform owners for Windows, macOS, Linux sessions.
- [ ] Prepare macOS and Linux hosts (toolchain, access, documentation) before Phase W exit to minimize downtime.
- [ ] Merge spike findings back into Phase 5 plan once the implementation timeline is approved.

---

## Evidence Log Template (All Platforms)
````markdown
## <ISO8601 Timestamp> - <Phase Step>
- Branch/Commit: <branch>@<commit>
- Command(s):
  ```
  <commands>
  ```
- Output Summary: <key lines>
- Sanity Check: <PASS/FAIL + notes>
- Remediation: <details or n/a>
- Next Actions / Handoff Notes: <what the next operator must do or watch>
- Reviewer: _pending_
````

---

## Prompt Templates

### Phase Handoff Prompt Template
Use this prompt after completing a phase to instruct the outgoing operator to craft a fully contextual handoff for the next session.

```
You have just completed Phase <W/M/L/Final> of the Substrate transport parity spike.
Prepare a detailed handoff message for the next operator so they can resume with zero prior context.

Your handoff message must include the following sections:

1. Phase Recap
   - Summarize the objective of Phase <phase> and confirm its completion status.
   - Mention the exact branch and commit you ended on.

2. Execution Evidence Highlights
   - Enumerate the most critical commands run (reference the evidence log entry) and their outcomes.
   - Note where full output is stored (e.g., `docs/project_management/logs/<platform>_always_world.md#<anchor>`).

3. Wildcards / Non-Documented Findings
   - List any unexpected behaviours, manual tweaks, or environment quirks not already covered in `docs/SPIKE_TRANSPORT_PARITY_PLAN.md`.
   - Include remediation steps taken and anything still unresolved.

4. Required Reading For The Next Session
   - Provide an ordered list of files/directories the next operator must read before touching the code.
   - Highlight diffs or config files that changed and need close inspection.

5. Guardrails To Honor
   - Restate the key guardrails from `docs/SPIKE_TRANSPORT_PARITY_PLAN.md`.
   - Call out any platform-specific constraints discovered during this phase.

6. Next Steps Checklist
   - Specify exactly which step number in `docs/SPIKE_TRANSPORT_PARITY_PLAN.md` the next operator should start at.
   - List outstanding TODOs, risks, or verification tasks that must happen first.
   - Confirm that the Phase Status Matrix was updated and the repository state (pushed/stashed).

7. Contact / Follow-up
   - State who to contact (or note "self" if you will resume later).
   - Include any timing commitments or gating approvals still required.

Make sure the resulting handoff reads like an operations runbook entry: concise headers, bullet lists where useful, no missing context.
```

### Post-Phase Prompt Usage
1. After completing the exit checklist, give the template above to the outgoing operator (or use it yourself).
2. Ensure the resulting prompt is stored under “Next Actions / Handoff Notes” in the evidence log.

---
