# ADR-0020 — Profiles Config and Policy Snapshots

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-01-30
- Curated into `docs/adr/draft/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0020-profiles-config-policy-snapshots.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Substrate may need explicit profiles that provide complete config and policy snapshots for specific
surfaces instead of relying entirely on layered defaults/global/workspace resolution.

The queued direction that still matters is:

- complete config and policy snapshots with no accidental layer leakage
- explicit surface scoping for profile application
- explainability about when profiles suppress workspace/global/default layers

## Why Queued

This remains active architectural input, but it is not landed and should not yet be treated as a
stable contract.

When implementation is ready, it should be restated against:

- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0016-world-first-repl-persistent-pty.md`

## Draft Note

Keep the project-management ADR for the fuller profile model and validation detail, but treat this
curated draft as the queued profile-snapshot placeholder.
