# ADR-0032 — Persist Linux Distro and Package-Manager Detection in `install_state.json`

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Installer and host-provisioning maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Successful Linux installs must persist platform detection metadata into the canonical installer
state file.

The stable decision is:

- Linux installs write or update `$SUBSTRATE_HOME/install_state.json` even when no separate
  host-state event occurred
- persisted metadata includes os-release identity and selected package-manager provenance
- the file remains additive and schema-compatible for older readers
- persisted metadata is diagnostic-only and does not itself redefine provisioning behavior

## Stable Owned Surface

This ADR owns the stable Linux install-state metadata contract documented in:

- `docs/INSTALLATION.md`
- `docs/contracts/install-state-schema.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `tests/installers/install_state_smoke.sh`

## Related ADRs

- `docs/adr/implemented/ADR-0031-best-effort-linux-distro-package-manager-discovery-during-install.md`
- `docs/adr/draft/ADR-0039-persist-macos-host-os-details-in-install-state.md`

## Historical Note

The original ADR captured the persistence rollout details and schema decisions for installer
metadata. The stable Linux install-state contract now lives here and in the installation guide.
