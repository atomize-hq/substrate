# Substrate Backlog

Status: living document capturing near-term and upcoming work. Keep concise, actionable, and security-focused.

## Near-Term (Next 1–2 sprints)

- Auto-start world-agent on shell startup (Linux)
  - Goal: “Always World” UX without manual agent management.
  - Behavior: On shell start, if `SUBSTRATE_WORLD` is not disabled and `/run/substrate.sock` is absent or stale, spawn `world-agent` and wait for readiness; degrade only on explicit `--no-world` or env disable.
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
  - Linux: Add systemd units (socket + service) with `socat` to expose `127.0.0.1:17788` → `/run/substrate.sock`. Disabled by default.
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
