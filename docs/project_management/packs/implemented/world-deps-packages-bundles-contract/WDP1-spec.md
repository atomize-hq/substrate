# WDP1-spec — Enabled patch editing + effective enabled view

## Scope
- Implement enabled patch file editing via the shared config editor/merge engine:
  - `substrate world deps global add|remove|reset`
  - `substrate world deps workspace add|remove|reset`
- Implement non-world enabled views:
  - `substrate world deps current list enabled`
  - `substrate world deps global list enabled`
  - `substrate world deps workspace list enabled`

## Behavior (authoritative)
All behavior for commands in scope is defined by the contract doc:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

Constraints enforced in this slice:
- Patch edits preserve comment headers.
- Patch-only semantics and effective resolution are broker-canonical (no bespoke merge logic):
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
- `global add` validates names only against the global available inventory view (built-ins + `$SUBSTRATE_HOME/deps/`).
- `workspace add` validates names against the current available inventory view for `cwd` (subject to `world.deps.inventory_mode`).
- `current list enabled` MUST NOT make world-service calls.

## Acceptance criteria
- Global/workspace add/remove/reset mutate only their scoped patch file and preserve existing comment headers.
- `current list enabled` exits `2` when any enabled name is not in effective available inventory view for `cwd`.
- Patch-only list commands print patch view (not merged) per contract doc.

## Out of scope
- World-backed status (`applied`, `show --explain`) (WDP2).
- Install/sync mutation surfaces (WDP3+).
