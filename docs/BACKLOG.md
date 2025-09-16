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

- macOS backend enablement (Lima)
  - Implement `world-mac-lima` agent calls via `agent-api-client` over SSH-forwarded UDS.
  - Add shell macOS routing (ensure Lima ready, forward socket, use MacLimaBackend).
  - Acceptance: `substrate -c` and replay work end-to-end on macOS using Lima world-agent; docs updated.
