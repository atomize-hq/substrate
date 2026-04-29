# Milestone 2.1: Remove Extra Listeners and Tighten Agent Surface

## Status

Draft

## Purpose / outcome

Remove the default guest TCP listener posture from macOS/Lima so the hardened default is one world-agent transport contract: the Unix domain socket at `/run/substrate.sock`, preferably inherited through socket activation.

## Why this milestone exists

The current macOS warm flow writes `Environment=SUBSTRATE_AGENT_TCP_PORT=61337` into the guest service in [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh). In [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs), that environment variable enables a loopback TCP listener whenever one was not inherited from socket activation. That widens the guest attack surface even though the documented contract centers on `/run/substrate.sock`.

This milestone exists to make the listener surface match the intended contract before any lifecycle tooling is built on top of it.

## In-scope

- Remove the default `SUBSTRATE_AGENT_TCP_PORT=61337` injection from the macOS guest service path.
- Freeze the intended macOS listener contract as “UDS required; any retained
  raw TCP path is breakglass/unsupported and not part of the supported or
  degraded-but-supported runtime contract.”
- Update doctor, smoke, and docs so they validate or describe the hardened listener posture instead of silently tolerating the old one.
- Identify any transport code paths that still assume TCP fallback is always present.

## Out-of-scope

- Reworking the full host-to-guest forwarding implementation.
- Removing TCP support from `world-agent` globally or for Windows/WSL.
- Solving mount breadth or service-unit duplication beyond what is required to remove the extra listener default.

## Architectural approach

- Keep `world-agent` support for optional TCP listeners in shared runtime code, but stop enabling it in the macOS hardened default.
- Treat TCP on macOS as a breakglass-only exception, not as background service
  behavior or a supported compatibility mode.
- Align the warm script, doctor checks, smoke coverage, and operator docs around one default listener posture so later Phase 3 commands do not have to explain mixed semantics.

## Dependencies / sequencing

- This milestone should land first in Phase 2.
- Milestone 2.3 must encode the listener decision in the unified unit definition after this milestone freezes the contract.
- If host forwarding still depends on the guest TCP port for any path, that dependency must be identified here and either removed or formally carried as an exception.

## Concrete repo surfaces and file pointers

- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
  - remove the service-level `SUBSTRATE_AGENT_TCP_PORT=61337` default
  - update check-only output if it currently assumes that env exists
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
  - confirm the runtime behavior when the TCP env var is absent
  - preserve explicit opt-in semantics if shared platforms still need TCP
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
  - update macOS transport text so TCP is no longer presented as part of the default guest listener surface
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)
  - remove or reframe guidance that implies the guest service normally exposes a TCP listener
- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
  - add or refine checks that show the UDS contract is healthy without relying on TCP
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
  - ensure the smoke path proves the macOS backend still works after the TCP default is removed

## Deliverables

- A frozen listener contract for hardened same-user Lima.
- An implementation plan that removes the default guest TCP listener on macOS.
- Updated diagnostic and smoke expectations showing how to verify UDS-only operation.
- Doc changes that stop normalizing TCP as part of the hardened default.

## Acceptance criteria

- The macOS warm/provision flow no longer injects `SUBSTRATE_AGENT_TCP_PORT=61337` by default.
- A fresh or repaired Lima guest starts `world-agent` successfully with only `/run/substrate.sock` exposed by default.
- macOS doctor and smoke evidence remains green without depending on the guest TCP listener.
- Any remaining raw TCP path on macOS is explicitly labeled
  breakglass/unsupported rather than supported, degraded-but-supported, or
  compatibility mode.

## Validation / evidence plan

- Capture the rendered guest service unit before and after the change and prove the TCP env line is absent.
- Run `scripts/mac/lima-doctor.sh` and confirm health is established through the socket path and systemd/socket-activation state, not a TCP port probe.
- Run `scripts/mac/smoke.sh` and any transport-specific smoke needed to prove replay, PTY, and gateway flows still work.
- Inspect `world-agent` startup logs for `listener_kind = "tcp"` and `listener_mode` changes to prove the hardened default no longer enables direct-bind TCP.

## Risks / open questions

- Some SSH forwarding or gateway flows may still assume a guest TCP bridge exists even if the documented contract does not.
- There may be macOS-local debugging workflows that currently rely on port `61337`; those need explicit migration guidance instead of quiet breakage.
- The shared `world-agent` runtime supports TCP for valid cross-platform reasons, so the macOS-specific hardening must avoid regressing WSL or other callers.
