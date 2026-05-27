# ADR-0039 — Persist macOS Host OS Details in `install_state.json`

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-03-30
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Installer and host-provisioning maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

The installer metadata file may gain additive macOS host OS details so later diagnostics can reason
about the host platform that performed the install.

The queued direction that still matters is:

- keep `install_state.json` as the canonical installer metadata file
- add macOS host OS facts additively without changing provisioning behavior
- persist warning-only metadata rather than turning collection failures into install failures
- keep the schema and consumer posture compatible with existing installer metadata readers

## Why Queued

This is still active installer metadata work, but it is not landed and should not yet be treated
as a stable install contract.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0032-persist-linux-distro-package-manager-detection-in-install-state.md`
- `docs/INSTALLATION.md`

## Draft Note

Keep the project-management ADR for the original planning detail, but treat this curated draft as
the queued macOS install-state placeholder.
