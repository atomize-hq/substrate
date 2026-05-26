# ADR-0019 — Warn on Config Global Show When Workspace Config Overrides

## Status

- Status: Draft
- Queue state: Queued
- Original date (UTC): 2026-01-30
- Owner(s): Substrate maintainers

## Curated From

- Planning ADR:
  - `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`

The project-management ADR remains the planning-rich source retained for compatibility while
`docs/project_management/**` is retired.

## Queued Direction

Substrate should emit a high-signal stderr note when `config global show` is likely to be
misread inside a workspace with active workspace overrides, while preserving stdout and exit-code
stability.

The queued direction that still matters is:

- warn only when workspace override conditions actually apply
- keep stdout patch-only and script-safe
- emit explicit scope/write-target guidance for implicit-scope `config set`

## Why Queued

This remains active UX/input-surface work, but it is not landed and should not be treated as a
stable contract yet.

When implementation is ready, it should be restated against:

- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`

## Draft Note

Keep the project-management ADR for the original operator-contract detail, but treat this curated
draft as the queued warning-behavior placeholder.
