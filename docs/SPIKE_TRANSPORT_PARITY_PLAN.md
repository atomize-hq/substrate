# SPIKE: Cross-Platform Agent Transport Parity

Status: Draft (awaiting review)  
Owner: Substrate Core (Windows strike team lead TBD)  
Created: 2025-09-23  
Related Plans: docs/PHASE_5_ALWAYS_WORLD_WINDOWS_PLAN.md  
Tracking IDs: T-011, T-012, (new) T-013 once filed

---

## Phase Status Matrix
| Phase | Host Platform | Status | Last Updated | Reviewer | Evidence Log Anchor |
|-------|---------------|--------|--------------|----------|----------------------|
| W | Windows (WSL2) | Pending | _tbd_ | _tbd_ | windows_always_world.md#W |
| M | macOS (Lima) | Pending | _tbd_ | _tbd_ | macos_always_world.md#M |
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
3. After each numbered step, capture: timestamp, command(s), exit code, key output, sanity-check result, remediation, reviewer signature placeholder, branch/commit, and handoff notes.
4. Stop immediately if a command fails. Diagnose using the troubleshooting catalogue, record remediation, and only then retry.
5. Do not reorder tasks, substitute tooling, or change script names without plan-owner approval.
6. Keep file encoding ASCII unless a file already uses UTF-8 with BOM; preserve line endings appropriate to the file.

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
- Rust toolchain 1.79+ with `cargo`, `rustup`, and target toolchains for each platform.
- Node.js 18+ for documentation linting.
- `markdownlint-cli` installed globally (`npm install -g markdownlint-cli`) or available via `npx`.
- Shell utilities: `pwsh` 7.x on Windows, `bash`/`zsh` on macOS/Linux, `rg` (ripgrep) on all hosts.
- Ability to install or verify virtualization components (WSL2 on Windows, Lima on macOS, systemd-based distro on Linux).
- Credentials to access required package repositories (Canonical WSL images, Homebrew taps, apt repositories).

Before beginning any phase, ensure the repository is synchronized with the main integration branch and that outstanding changes are either committed or stashed. Example commands (adjust branch names as needed):

```pwsh
# Windows PowerShell
cd C:\Users\<user>\Documents\__Project_Code\substrate
git status
# Ensure clean or stash user changes before proceeding
```

```bash
# macOS/Linux
cd ~/Documents/__Project_Code/substrate
git status
```

Record the `git status` output in the appropriate evidence log.

---

## 3. Terminology Snapshot
- **Forwarder**: Windows-side bridge that proxies host I/O into the WSL world via Named Pipe/TCP.
- **Transport Abstraction**: New layer in `agent-api-client` providing platform-neutral HTTP connectivity to `world-agent`.
- **Evidence Log**: Markdown file capturing commands, outputs, sanity results, branch/commit, and handoff notes for auditing.
- **Smoke Suite**: Platform-specific automation that validates world warmup, doctor checks, PTY, replay, and resilience.

---

## 4. Phase W – Windows Transport Layer Refactor (Session 1)

### Entry Criteria Checklist
- Windows 11 Pro/Enterprise host with virtualization enabled.
- WSL2 feature installed with distro `substrate-wsl` provisioned per `docs/dev/wsl_world_setup.md`.
- PowerShell 7.x available as `pwsh`.
- Evidence log `docs/project_management/logs/windows_always_world.md` exists.

Document each checklist item with commands below and store results in the Windows evidence log. When complete, update the Phase Status Matrix (Phase W → In Progress, record timestamp).

#### Step W0 – Host Validation
1. Verify virtualization and optional features:
   ```pwsh
   systeminfo | Select-String "Virtualization"
   Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
   Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
   ```
   Expect `Enabled` for both optional features.
2. Confirm WSL status and distro:
   ```pwsh
   wsl --status
   wsl -l -v
   ```
   Ensure `Default Version: 2` and `substrate-wsl` listed with version 2.

Log outputs and mark sanity as PASS/FAIL.

#### Step W1 – Repository Sync & Baseline Failure Capture
1. Synchronize repository:
   ```pwsh
   git pull --rebase
   ```
   Resolve conflicts if any; record actions.
2. Capture the current failure to establish baseline evidence:
   ```pwsh
   cargo check
   ```
   Expect failure referencing `tokio::net::UnixStream/UnixListener` from hyperlocal. Paste the failing section into the log under entry `W1`.

#### Step W2 – Architecture Preparation
1. Create an architecture sketch summarizing the intended transport flow. Store the diagram source (markdown or ASCII) at `docs/dev/transport_parity_design.md`.
   - Outline components: host binaries → `agent-api-client` transport → forwarder (Windows) → TCP/UDS bridge → `world-agent` inside WSL.
   - Include state transitions for Named Pipe, TCP, Unix socket.
2. Run `npx markdownlint-cli docs/dev/transport_parity_design.md` and log the result.

#### Step W3 – Implement Transport Abstraction in `agent-api-client`
... (rest of existing steps remain unchanged)
...

#### Step W9 – Phase Exit Checklist
- [ ] Workspace builds/tests passing (`cargo check`, `cargo test`).
- [ ] Smoke suite passes with updated transport (`scripts/windows/wsl-smoke.ps1`).
- [ ] Telemetry inspected (`transport.mode` observed in trace).
- [ ] Docs & troubleshooting updated and linted.
- [ ] Evidence log entry completed with branch/commit hash and handoff notes.
- [ ] Code pushed to shared branch (`git push`).
- [ ] Phase Status Matrix updated (Phase W → Complete with timestamp & reviewer).
- [ ] Next session kickoff prompt prepared (see Prompt Templates section) and attached to evidence log.

---

## 5. Phase M – macOS Transport Validation (Session 2)

### Entry Criteria
- Access to macOS Ventura/Sonoma workstation with admin rights.
- Lima VM configured per Phase 4.5 documentation (`lima/substrate.yaml`).
- Changes from Phase W merged or rebased onto working branch.
- Evidence log `docs/project_management/logs/macos_always_world.md` available.
- Kickoff prompt from Windows session reviewed.

Update Phase Status Matrix (Phase M → In Progress) when starting.

... (Phase M steps remain; add exit checklist analogous to Phase W)

#### Step M7 – Phase Exit Checklist
- [ ] Workspace builds/tests passing on macOS.
- [ ] Lima smoke suite passes.
- [ ] Trace shows `transport.mode = "unix"`.
- [ ] Docs updated if mac-specific changes introduced; lint passes.
- [ ] Evidence log entry complete with branch/commit & handoff notes.
- [ ] Code pushed; matrix updated (Phase M → Complete, timestamp & reviewer).
- [ ] Next session kickoff prompt prepared and logged.

---

## 6. Phase L – Linux Transport Validation (Session 3)

### Entry Criteria
- Native Linux workstation (Ubuntu 24.04 LTS recommended) or VM with systemd.
- Access to repository with Phase W/M changes applied.
- Evidence log `docs/project_management/logs/linux_always_world.md` created.
- Kickoff prompt from macOS session reviewed.

Update Phase Status Matrix (Phase L → In Progress) when starting.

... (Phase L steps remain; add exit checklist)

#### Step L6 – Phase Exit Checklist
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
- Forwarder, world agent, and host tooling all using new interface.
- Documentation updates (`docs/dev/*`, troubleshooting catalogue, design doc).
- Evidence logs for Windows, macOS, Linux with complete command/output history, handoff notes, and kickoff prompts.
- Final summary entry in `docs/project_management/logs/windows_always_world.md` referencing macOS/Linux logs and confirming multi-platform PASS.
- Pull request or change request containing:
  - Implementation commits.
  - Documentation updates.
  - Links to evidence log sections.
  - Reviewer checklist covering transport behavior, security, and telemetry parity.

---

## 9. Risk Register & Mitigations
- **Transport Abstraction Bugs**: Unit/integration tests per connector, CI coverage on Windows.
- **Security Exposure via TCP Listener**: Enforce localhost binding with automated tests; document firewall expectations.
- **Environment Drift Between Sessions**: Mandatory handoff notes + kickoff prompts ensure continuity.
- **Trace Schema Changes**: Coordinate with telemetry team before altering span fields; update docs & tests.

---

## 10. Open Questions
1. Should the world agent support vsock (for future hypervisor integrations) alongside TCP? Track in follow-up RFC.
2. Do we need automated rollback procedures if transport abstraction fails in production? Determine during design review.
3. How will CI enforce transport parity going forward? Engage DevInfra to extend test matrix.

---

## 11. Next Actions Checklist
- [ ] Schedule design review covering transport abstraction, forwarder changes, and security posture.
- [ ] Assign platform owners for Windows, macOS, Linux sessions.
- [ ] Prepare macOS and Linux hosts (toolchain, access, documentation) before Phase W exit to minimize downtime.
- [ ] Merge spike findings back into Phase 5 plan once implementation timeline approved.

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

Append entries sequentially; include references back to Phase Status Matrix updates.

---

## Prompt Templates

### Phase Kickoff Prompt Template
Use this template at the end of each phase to generate context for the next session/operator. Paste into evidence log and share with the next agent.

```
You are the next operator continuing the Substrate transport parity spike.

Context Summary:
- Phase just completed: <W/M/L>
- Branch/Commit: <branch>@<commit>
- Evidence log anchor: <file#Lanchor>
- Transport status: <named_pipe|unix|tcp verified>
- Outstanding issues: <open bugs or TODOs>
- Next required phase: <M/L/Final>
- Entry criteria already satisfied: <list>
- Remaining entry prep: <steps still needed>
- Pending risks or watch items: <list>

Instructions:
- Review the linked evidence log entry.
- Confirm repository is synced to the branch/commit above.
- Start at Step <X> in docs/SPIKE_TRANSPORT_PARITY_PLAN.md for Phase <next>.
```

### Post-Phase Prompt Usage
1. After completing the exit checklist, fill placeholders in the template.
2. Provide the resulting prompt to the next operator (or future self) to kick off the following session.
3. Record the prompt under the “Next Actions / Handoff Notes” section in the evidence log.

---

Append evidence log templates (copy into respective files if missing):

```markdown
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
```
