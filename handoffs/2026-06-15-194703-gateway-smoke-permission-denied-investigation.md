# Handoff: Gateway Smoke Permission Denied Investigation

## Session Metadata
- Created: 2026-06-15 19:47:03
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: ~1 hour

### Recent Commits (for context)
  - 8deb75a4 fix: stabilize path tests with substrate home override
  - 2400b40e fix: restore linux world spec builders
  - 928ef583 Feat/macos hardening (#78)
  - 211332b2 Add Slice 57 packet orchestration prompts
  - 1d966a8a Add slice 57 host-global ingress coordination spec

## Handoff Chain

- **Continues from**: None (fresh start)
- **Supersedes**: None

> This is the first handoff for this task.

## Current State Summary

Investigated why `./scripts/substrate/dev-fresh-install-gateway-smoke.sh` fails on Linux with `substrate world gateway status` returning `Permission denied (os error 13)` on `/run/substrate.sock`. The current conclusion is that the failure is not caused by gateway config or the smoke helper itself. System provisioning appears correct: `/run/substrate.sock` exists as `root:substrate 0660`, the `substrate` group exists, and `spenser` is listed in that group in account data. The actual problem is that the live desktop/login session still has stale supplementary groups, so shells launched from that session cannot open the socket until group membership is refreshed. The unresolved product question is UX: first-time Linux install likely lands users in the same state, and current installer warnings may not be sufficient.

## Codebase Understanding

### Architecture Overview

The Linux world/gateway path is gated by the OS-level socket ACL on `/run/substrate.sock`. `substrate world gateway status` and `substrate world gateway sync` both talk to the world backend over that socket, so they fail before any higher-level gateway lifecycle logic if the calling process cannot open the socket. `substrate world doctor --json` is the authoritative diagnosis surface for this path: it reports socket ownership, mode, systemd socket/service state, probe result, and an explicit `permission_denied_help` message sourced from shell runtime code.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| `scripts/substrate/dev-fresh-install-gateway-smoke.sh` | Fresh-install helper for Codex host/world gateway smoke | Reproduced failure; helper is only surfacing the socket authorization problem |
| `scripts/substrate/dev-install-substrate.sh` | Linux/macOS dev installer | Already adds the invoking user to `substrate` and warns about `newgrp` / relogin |
| `scripts/linux/world-provision.sh` | Linux world socket/service provisioner | Creates/adds `substrate` group membership and prints same membership-refresh guidance |
| `crates/shell/src/execution/platform/linux.rs` | Linux doctor/probe logic | Contains `permission_denied_help` and confirms ACL boundary behavior |
| `docs/WORLD.md` | Linux world socket contract | Documents `/run/substrate.sock` as `root:substrate 0660` and group membership as access boundary |
| `AGENTS.md` | Repo operational guidance | Confirms provisioning expectation and post-provision group check |

### Key Patterns Discovered

- Linux world access is intentionally fail-closed at the socket boundary; permission failure is treated as expected diagnostics, not an ambiguous runtime error.
- Installer/provisioner scripts already separate two concerns: system setup and session refresh. They can mutate `/etc/group` and systemd units, but they cannot retroactively change supplementary groups of an already-running login session.
- `world doctor --json` is the fastest trustworthy way to distinguish misprovisioning from stale caller credentials.

## Work Completed

### Tasks Finished

- [x] Reproduced the gateway smoke failure from the helper script.
- [x] Verified direct `substrate world gateway status` and `sync` fail the same way outside the helper.
- [x] Collected authoritative `world doctor --json` output showing `PermissionDenied` on `/run/substrate.sock`.
- [x] Verified `/run/substrate.sock` ownership/mode are correct: `root:substrate 0660`.
- [x] Verified `spenser` is present in `getent group substrate`.
- [x] Verified current shell/session does not have `substrate` in supplementary groups.
- [x] Verified `newgrp substrate` immediately activates the correct group set in a subshell.
- [x] Reviewed installer/provisioner scripts and confirmed they already add the user to `substrate` and warn about relogin / `newgrp`.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| [no modified files detected] | | |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Treat the current gateway smoke failure as a session/group-refresh issue, not a gateway config bug | gateway lifecycle bug vs helper bug vs socket permission issue | `world doctor`, socket metadata, and `newgrp substrate` behavior all point to caller authorization state |
| Treat this as a real Linux first-install UX problem | dismiss as local machine quirk vs generalize to first-time install | Any user added to `substrate` after their login session starts will hit the same stale-group behavior |
| Do not start a code fix in this session | patch installer now vs stop at diagnosis | User explicitly asked for investigation and concrete issue without starting a fix |

## Pending Work

### Immediate Next Steps

1. Decide desired UX contract for first Linux install: is same-terminal immediate use required, or is explicit post-install shell handoff acceptable.
2. Design and validate an installer/user-shell handoff flow that makes the `substrate` group active for the terminal the user will actually use.
3. Add an early fail-closed preflight in the gateway smoke helper and/or first-run doctor path that explains stale group membership before attempting gateway lifecycle calls.

### Blockers/Open Questions

- [ ] What exact UX do we want on first Linux install: immediate current-terminal usability, or explicit re-entry command?
- [ ] Should installer reruns emit stronger warnings when account membership is correct but the current session still lacks `substrate`?
- [ ] Should the smoke helper preflight `id -nG` vs socket ACL and abort with a targeted remediation instead of letting `world gateway status` fail first?

### Deferred Items

- Implementing any installer or smoke-helper changes was deferred because the user asked for diagnosis only.
- Broader manual smoke doc fixes from earlier investigation were deferred; this handoff is focused on the gateway permission issue.

## Context for Resuming Agent

### Important Context

The most important fact is that the Linux system provisioning appears correct, but the live session credentials are stale. Evidence:

- `world doctor --json` reported `/run/substrate.sock` with `owner_user=root`, `group_name=substrate`, `mode_octal=0660`, `probe_ok=false`, `probe_error_kind=PermissionDenied`, and a help message saying the socket ACL is the authorization boundary.
- `ls -l /run/substrate.sock` showed `srw-rw---- 1 root substrate`.
- `getent group substrate` returned `substrate:x:954:spenser`.
- `id -nG` in the current shell did not include `substrate`.
- Running `newgrp substrate` caused `id -nG` to include `substrate` immediately in the spawned shell.
- `loginctl session-status` showed the desktop session is old (`Since: Mon 2026-06-08 ...`), so new terminals launched from it can still inherit stale supplementary groups.

This means the issue is not "gateway status is broken" and not "world socket is misprovisioned". The unresolved work is a Linux installer/session-handoff UX design problem.

### Assumptions Made

- The user’s observed “fresh shell” was likely another shell launched from the same desktop login session, not a full logout/login.
- The world socket/service installation under `/run/substrate.sock` is the intended Linux transport contract for this repo state.
- `newgrp substrate` is sufficient to prove the problem is stale supplementary groups, even though it does not itself solve desktop-wide session refresh.

### Potential Gotchas

- Do not confuse account database membership (`getent group substrate`) with active process credentials (`id -nG` in the current shell). The former can be correct while the latter is stale.
- A "new shell" is not necessarily a new login session. In this environment, terminals are children of an old XFCE session and inherit stale groups.
- The repo root contains `.codex` as an empty file, not a directory. The `session-handoff` skill's default `.codex/handoffs` path fails here; use `--dir handoffs`.
- There are many leftover test/helper processes in the desktop session from prior work; do not infer current user-session freshness from process noise.

## Environment State

### Tools/Services Used

- `./scripts/substrate/dev-fresh-install-gateway-smoke.sh --skip-sync`
- `~/.substrate/bin/substrate world doctor --json`
- `~/.substrate/bin/substrate world gateway status`
- `~/.substrate/bin/substrate world gateway sync`
- `ls -l /run/substrate.sock`
- `stat -c '%A %U %G %a %n' /run/substrate.sock`
- `id -nG`, `id`, `groups`
- `getent group substrate`
- `loginctl session-status`
- `newgrp substrate`

### Active Processes

- Desktop session `3` under LightDM/XFCE is still active and dates back to 2026-06-08.
- There are many leftover fake-codex/fake-claude and helper processes from prior test runs visible under the session. They were not modified in this investigation.

### Environment Variables

- `SUBSTRATE_HOME`
- `SUBSTRATE_WORLD_SOCKET`
- `SUDO_USER`
- `USER`

## Related Resources

- [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh)
- [scripts/substrate/dev-install-substrate.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-install-substrate.sh)
- [scripts/linux/world-provision.sh](/home/spenser/__Active_code/substrate/scripts/linux/world-provision.sh)
- [crates/shell/src/execution/platform/linux.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/platform/linux.rs)
- [docs/WORLD.md](/home/spenser/__Active_code/substrate/docs/WORLD.md)
- [AGENTS.md](/home/spenser/__Active_code/substrate/AGENTS.md)
- `/run/substrate.sock`
- `handoffs/2026-06-15-194703-gateway-smoke-permission-denied-investigation.md`

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
