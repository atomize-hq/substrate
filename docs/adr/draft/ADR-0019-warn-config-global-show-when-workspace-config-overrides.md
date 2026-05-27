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

## Folded Contract Detail

When this queued work lands, the stable command-surface intent is:

- `substrate config global show` still prints only the global patch on stdout
- when a workspace override is active, `substrate config global show` emits exactly one stderr note
  explaining that workspace config overrides the global patch in the current directory
- unreadable or invalid workspace config must not make `config global show` fail if the global
  patch is otherwise readable
- implicit `substrate config set <key>=<value>` remains workspace-scoped and emits an explicit
  write-target note on stderr while keeping stdout as the effective merged config

## Why Queued

This remains active UX/input-surface work, but it is not landed and should not be treated as a
stable contract yet.

When implementation is ready, it should be restated against:

- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0005-workspace-config-precedence-over-env.md`

## Draft Note

This curated draft now carries the queued contract summary that previously lived only in the pack
contract file. Keep the project-management ADR as the planning-rich historical source.
