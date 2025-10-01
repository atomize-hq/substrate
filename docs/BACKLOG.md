# Substrate Backlog

Status: living document capturing near-term and upcoming work.
Keep concise, actionable, and security-focused.

## Near-Term (Next 1–2 sprints)

- Auto-start world-agent on shell startup (Linux)
  - Goal: “Always World” UX without manual agent management.
  - Behavior: On shell start, if `SUBSTRATE_WORLD` is not disabled and
    `/run/substrate.sock` is absent or stale, spawn `world-agent` and wait for
    readiness; degrade only on explicit `--no-world` or env disable.
  - Constraints: UDS-only, no TLS; idempotent start; single warning on failure; remove stale socket before bind.
  - Acceptance:
    - `substrate -c 'echo hi'` uses world by default on Linux with no manual steps.
    - One-time, clear log lines for: spawn attempt, readiness OK, or fallback.
    - Works in Podman privileged container and on native Linux.

- Non-PTY agent auto-start parity (shell)
  - Reuse the PTY `ensure_world_agent_ready()` flow for non-PTY world HTTP path before `exec_non_pty_via_agent`.
  - Acceptance: No more “world exec failed, running direct” when world is enabled and agent binary is available.

- Return fs_diff via agent HTTP execute
  - Extend `agent-api-types::ExecuteResponse` to include `fs_diff: Option<FsDiff>`; plumb through `world-agent` service.
  - Update shell to parse and attach fs_diff for agent-routed non-PTY commands.
  - Acceptance: `fs_diff` appears in completion spans for agent HTTP path.

- Replay verbose: print `scopes_used`
  - On `--replay-verbose`, include a concise “scopes:” line with any collected scopes.
  - Acceptance: visible alongside “world strategy: …”

- Install + Ops: ensure world-agent binary discoverable
  - Package/install both `substrate` and `substrate-world-agent` into PATH so auto-start works on fresh installs.
  - Consider systemd socket activation for `/run/substrate.sock` (units: world-agent.socket + world-agent.service).
  - Improve auto-start search: include `target/release/world-agent` in addition to debug path.
  - Optional helper: `substrate world install-agent` to copy/link the agent and a `world doctor` hint after install.
  - Container image: copy agent to `/usr/local/bin/substrate-world-agent` or set `SUBSTRATE_WORLD_AGENT_BIN` in entrypoint.

- Optional TCP bridge for agent (Cross-platform: Linux/macOS/Windows)
  - Goal: Provide an optional loopback TCP endpoint that bridges to the agent UDS for environments/tools requiring TCP.
  - Linux: Add systemd units (socket + service) with `socat` to expose
    `127.0.0.1:17788` → `/run/substrate.sock`. Disabled by default.
  - macOS: Mirror the same inside the Lima guest; shell keeps transport order (UDS/VSock preferred), uses TCP only when bridge is enabled and UDS unavailable.
  - Windows: When Windows backend exists (e.g., WSL2 or native service), provide an equivalent optional loopback TCP bridge to the local agent endpoint.
  - Security: Loopback only; document that it is a fallback, not a replacement. No TLS.
  - Acceptance:
    - `curl -sSf http://127.0.0.1:17788/v1/capabilities` works when bridge is enabled.
    - Shell auto-selects TCP only when VSock/UDS are not available and the bridge is active.

- macOS bootstrap automation
  - Build a signed installer script (or helper binary) that validates macOS version/virtualization support, installs required Homebrew packages (`lima`, `jq`, `openssh`, `coreutils`, `gnused`, `gnu-tar`, `gettext`), provisions Lima, deploys `substrate-world-agent`, and runs doctor checks automatically.
  - Acceptance: Fresh macOS host can run a single installer command and end up with the Lima VM, agent, and smoke test ready without manual steps.

- macOS binary distribution
  - Publish prebuilt `substrate` and `substrate-world-agent` artifacts (e.g., GitHub releases or Homebrew formula) so operators do not need a Rust toolchain to onboard.
  - Acceptance: Installer/bootstrap script can download versioned binaries; manual instructions reference the published artifacts instead of local builds.

## Hardening / Quality

- Differentiate shell vs replay world warnings
  - Make shell path messages clearly say “shell world-agent path”; keep replay messages with “[replay] …”.
- Integration tests: default-to-world replay
  - Validate default ON, `--no-world`, and env var opt-out.
- Document backend divergence
  - Clarify why shell uses world-agent and replay uses LinuxLocalBackend; outline future convergence plan.

- Heavy isolation wiring (optional)
  - Provide a mode to invoke `LinuxIsolation::apply()` for non-PTY when capabilities allow; keep current lightweight/PTy approach as default.
  - Acceptance: gated by capability checks; clear degrade messages.

- Docs consistency
  - Document non-PTY agent auto-start behavior and fallback; update REPLAY/WORLD runbooks accordingly.

## Later

- PTY world fs_diff strategy
  - Explore overlay or post-exit diff for interactive sessions.
- Policy/broker expansion
  - Richer allowlist/egress policy plumbing into world sessions.

- macOS backend enablement (Lima) — **DONE (2025-09)**
  - Implemented `world-mac-lima` agent calls via `agent-api-client` with VSock/SSH fallbacks.
  - Shell now routes macOS commands through Lima, ensures VM/forwarding, and mirrors Linux telemetry.
  - Acceptance met: `scripts/mac/smoke.sh` validates non-PTY, PTY, and replay; docs refreshed (`docs/WORLD.md`, `docs/dev/mac_world_setup.md`, `docs/INSTALLATION.md`).
---

## Windows Transport — Cross‑Cutting Backlog
User‑Scoped Named Pipe Default

Context: We standardize on a single host pipe
`\\.\\pipe\\substrate-agent`. On shared or multi‑session Windows systems this
name can be owned by a stray forwarder, an IT‑deployed service, or another
user session. While the “single‑instance guard + friendly error” mitigates
confusion at startup, adopting a user‑scoped default eliminates cross‑user
collisions by design.

Goal: Make the default forwarder pipe name user‑scoped (for example,
`\\.\\pipe\\substrate-agent-<SID>`), while preserving backward compatibility
and explicit overrides.

Design details
- Default name schema: `\\.\\pipe\\substrate-agent-<SID>` where `<SID>` is the
  current user’s Windows SID; fallback to `<USERNAME>` if SID retrieval fails.
  Keep `\\.\\pipe\\substrate-agent` as a documented legacy alias.
- Backward compatibility: If `SUBSTRATE_AGENT_PIPE` (or equivalent) is set to
  the legacy name, honor it. Doctor/warm should detect conflicts and print
  remediation.
- Display/telemetry: Sanitize endpoints in logs/telemetry (avoid leaking SID);
  use `substrate-agent-<user>` in human‑readable fields and keep the full path
  in debug logs only.
- Security: Maintain the current SDDL (SY/BA/IU GA) and
  `reject_remote_clients(true)`. Document that user‑scoped names reduce
  accidental cross‑session access but do not replace ACLs.

Code changes required (touch points)
- `crates/forwarder/src/main.rs`
  - Default CLI `--pipe` value to the user‑scoped name computed at runtime.
  - Emit both the effective path and a redacted display endpoint in logs.
- `crates/host-proxy/src/lib.rs`
  - Change `DEFAULT_AGENT_PIPE` to resolve at runtime via a helper that
    computes the user‑scoped name when no explicit config is supplied.
  - Update serde/display to accept both legacy and user‑scoped forms.
- `crates/world-windows-wsl/src/lib.rs`
  - Ensure `build_agent_client()` resolves the same default and exposes the
    chosen endpoint for telemetry.
- `crates/agent-api-client` (tests)
  - Update unit tests that assert the hard‑coded legacy pipe to accept the
    dynamic, user‑scoped default.
- `crates/shell` (tests/telemetry)
  - Adjust Windows tests that check `transport.endpoint` to tolerate the new
    default (or inject override during tests).

Scripts and docs
- `scripts/windows/wsl-warm.ps1`, `wsl-doctor.ps1`, `wsl-stop.ps1`
  - Default to the user‑scoped pipe; support `-PipePath` override and surface
    both the effective path and display value.
- Documentation updates
  - `docs/dev/windows_host_transport_plan.md`: call out the user‑scoped default
    and rationale.
  - `docs/dev/wsl_world_setup.md`: show the new default in examples and explain
    overrides.
  - `docs/dev/wsl_world_troubleshooting.md`: update entries for conflicts and
    how to identify the owning process (handle.exe), plus how to opt back to
    the legacy name for compatibility.
  - `docs/dev/windows_transport_external_overview.md`: update architecture
    diagrams and narrative to mention user‑scoped default.

Migration and compatibility strategy
- Soft roll‑out: keep the legacy name documented; scripts accept both and print
  a deprecation‑style hint when the legacy name is detected.
- Env/config knobs: add `SUBSTRATE_AGENT_PIPE` or forwarder TOML `pipe_path` to
  force legacy behavior for environments that depend on it.
- CI/tests: set `SUBSTRATE_AGENT_PIPE=\\.\\pipe\\substrate-agent-test` in CI to
  avoid SID dependencies in headless runners.

Acceptance criteria
- Fresh Windows setup uses the user‑scoped default end‑to‑end (forwarder,
  doctor/warm, shell, host‑proxy).
- Legacy name still works when explicitly configured; friendly warnings appear
  with migration hints.
- No collisions between different signed‑in users or sessions by default.
- Updated docs are markdownlint clean; evidence log captures a successful warm
  run using the user‑scoped default.

Risks / considerations
- Third‑party automations hard‑coding the legacy name will need overrides;
  mitigate via clear warnings and a deprecation window.
- Some admin workflows may prefer a machine‑wide pipe; document how to force
  the legacy name alongside hardening advice (service account, SDDL, audit).
