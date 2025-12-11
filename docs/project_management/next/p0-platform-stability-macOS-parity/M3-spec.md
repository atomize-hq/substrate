# M3-spec – Backend & Doctor Parity

## Scope
- Propagate policy-driven world fs mode and P0 shell/replay behaviors to mac:
  - `MacLimaBackend` must honor `WorldSpec.fs_mode` (read-only vs writable), not just env overrides, across PTY and non-PTY exec/replay.
  - Preserve caging/anchor parity and transport selection used in P0.
- Fix readiness/forwarding order and socket access expectations:
  - Establish forwarding before probing the agent; avoid UDS checks that fail prior to forwarding.
  - Document/adjust socket ownership/group model so mac matches Linux intent or clearly states the divergence.
- Align CLI surfacing with Linux P0:
  - `substrate --shim-status` / `--shim-status-json` on mac should report socket activation state and fs_mode in parity with Linux (noting any platform-specific caveats).
  - `substrate health` (text/JSON) should reflect manager parity decisions and socket state the same way Linux does, or explicitly document mac differences.
- Align world doctor/manual flows with P0 outputs:
  - Ensure doctor reflects the socket-activation state and fs_mode.
  - Update manual playbooks/cross-platform docs for the mac flow (host vs in-VM doctor, reprovision steps).
- Add tests/fixtures for mac pathways where feasible (unit-level and doctored JSON/CLI output where platform-agnostic).

## Acceptance Criteria
- mac executions/replays respect policy `fs_mode` the same way Linux/WSL do, with trace/doctor visibility.
- Agent readiness no longer fails due to pre-forwarding UDS probes; forwarding selection remains automatic (vsock/ssh).
- Doctor/shim/health output and docs reflect socket-activation status, fs_mode, and manager parity guidance on mac (or clearly documented divergences); manual playbook updated.
- Shim status (text/JSON) on mac emits socket activation state and fs_mode similar to Linux.
- Health CLI (text/JSON) mirrors Linux manager parity thresholds and socket reporting on mac, or documents any mac-only deviations.
- Tests cover fs_mode propagation plus doctor/shim-status/health JSON/text outputs (platform-agnostic portions), and integration gating runs remain green.

## Out of Scope
- Installer/provisioning mechanics (M1/M2).
- New replay features beyond parity with P0 behavior.
