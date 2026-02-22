# WDP0-spec — Inventory parsing + merged available views

## Scope

- Implement the inventory directory model defined by:
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md` (Inventory schema + merge rules)
- Implement non-world, read-only CLI surfaces:
  - `substrate world deps current list available`
  - `substrate world deps current show <item_name>` (without `--explain`)
  - `substrate world deps global list available`
  - `substrate world deps workspace list available`

## Behavior (authoritative)

All behavior for the commands in scope is defined by the contract doc:

- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

Constraints enforced in this slice:

- Inventory parsing rejects invalid YAML and schema violations.
- Inventory merge rules are enforced (full-replace per item name; package/bundle name collisions are an error).
- `current list available` and `current show` MUST NOT make world-agent calls.

## Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Exit codes follow the contract doc sections for each command.

## Acceptance criteria

- `current list available` exits `0` when inventory is valid and includes visible built-ins + global + workspace inventory items (subject to `world.deps.inventory_mode`).
- `current show <item>` exits `0` for known items and exits `2` for unknown items.
- `global list available` and `workspace list available` behave per contract and exit `2` on invalid YAML.

## Out of scope

- Enabled patch editing (`add|remove|reset`) and effective enabled view (WDP1).
- World-backed status (`applied`, `--explain`) (WDP2).
- Install/sync mutation surfaces (WDP3+).
