# WDP3-spec — Install/sync planning + dry-run behavior

## Scope
- Implement planning-only install/sync behavior:
  - `substrate world deps current install ... --dry-run`
  - `substrate world deps current sync --dry-run`
- Implement install plan computation as specified in the contract doc:
  - bundle expansion to packages
  - apt-first, script-second ordering
  - explicit `manual` blocked classification

## Behavior (authoritative)
All behavior for commands in scope is defined by the contract doc:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

Constraints enforced in this slice:
- `--dry-run` produces no side effects.
- The printed plan includes apt package list and script package list and is deterministic.

## Acceptance criteria
- `current install <item...> --dry-run` exits `0` and prints the computed plan.
- `current sync --dry-run` exits `0` and prints the computed plan for the effective enabled set.

## Out of scope
- Any mutation (install execution) (WDP4/WDP5).

