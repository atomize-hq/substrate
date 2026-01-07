# Plan — Doctor scope split (host vs world)

This plan is driven by `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`.

High-level steps:
- Add `substrate host doctor` CLI surface and route current host checks into it.
- Extend the world-agent API with a “world doctor” report (guest-kernel facts + enforcement readiness).
- Make `substrate world doctor` consume the agent report and render a scoped Host/World output (text + JSON).
- Update docs (`docs/COMMANDS.md`, `docs/WORLD.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/ISOLATION_SUPPORT_MATRIX.md`).

