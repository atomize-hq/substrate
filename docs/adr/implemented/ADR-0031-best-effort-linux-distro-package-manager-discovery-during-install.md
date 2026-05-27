# ADR-0031 — Best-Effort Linux Distro and Package-Manager Discovery During Install

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Substrate maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

The Linux installer must make host package-manager selection explicit, observable, and overrideable.

The stable decision is:

- Linux install uses a deterministic selection order: CLI flag, environment override, os-release
  mapping, then fixed PATH probe fallback
- the installer prints the detected distro and chosen package manager so prerequisite installation
  is observable
- invalid or unavailable explicit package-manager selections fail with stable, actionable exit
  behavior instead of silently probing something else
- macOS and Windows installer behavior are unchanged by this decision

## Stable Owned Surface

This ADR owns the stable Linux installer detection contract documented in:

- `docs/INSTALLATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `scripts/substrate/install-substrate.sh`
- `tests/installers/pkg_manager_detection_smoke.sh`
- `tests/installers/pkg_manager_container_smoke.sh`

## Related ADRs

- `docs/adr/implemented/ADR-0032-persist-linux-distro-package-manager-detection-in-install-state.md`
- `docs/adr/implemented/ADR-0034-stabilize-dev-install-helper-discovery-under-substrate-home.md`

## Historical Note

The original ADR captured the rollout detail for explicit Linux installer detection and override
precedence. The stable contract now lives here and in the installation guide.
