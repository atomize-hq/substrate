# World Deps — Packages/Bundles Contract (Draft)

This document defines the **user-facing contract** for `substrate world deps` using Substrate’s **host/world** terminology (no “guest” language). It is intentionally concise so we can iterate.

## Goals
- Make world deps a predictable way to declare and apply **in-world** dependencies.
- Unify the mental model around **inventory** (what exists) vs **selection** (what you want applied from the current directory).
- Avoid misleading names (no more `pyenv`-style “looks like a CLI but isn’t”).

## Key Terms
- **Host**: the developer workstation environment running `substrate`.
- **World**: the isolated execution environment behind world-agent (Linux host, macOS Lima VM, Windows WSL).
- **Inventory**: definitions of available **packages** and **bundles**.
- **Selection**: the desired set of inventory items for the **current directory** (resolved via sparse config merge).
- **World image** install: mutates OS-managed state in the world (e.g. apt/dpkg under `/usr`, `/var/lib/dpkg`).
- **World deps prefix** install: installs under `/var/lib/substrate/world-deps` and exposes entrypoints under `/var/lib/substrate/world-deps/bin`.

## Inventory Model
### Item types
- **Package** (`pkg:<name>`): an installable unit.
  - Install methods:
    - **apt** (world image install)
    - **script** (world deps prefix install; must create/ensure an entrypoint under `/var/lib/substrate/world-deps/bin`)
- **Bundle** (`bundle:<name>`): named group of packages (expands to packages).

### Naming rules (avoid collisions)
- Inventory IDs MUST use prefixes: `pkg:` and `bundle:` (no bare names).
- Bundle IDs MUST NOT be the same string as any package ID (prevents “`bundle:pyenv` that looks like `pkg:pyenv`”).
- Runnable CLIs MUST only be represented as **packages** (never bundles).

### Inventory sources and merge order
Inventory is resolved by merging these sources (later layers override earlier):
1) **Built-in defaults** shipped with Substrate (configurable to hide/disable; not edited directly).
2) **Global user inventory**: `~/.substrate/deps.yaml`
3) **Workspace inventory chain**: from the current directory upward, merge any `<dir>/.substrate/deps.yaml` found (nearest overrides earlier ancestors).

Workspace inventories **extend** global/built-ins by default. A workspace may opt into **workspace-only inventory** via selection config (see below).

## Selection Model (Sparse YAML)
Selection is resolved from sparse YAML config merged by current working directory:
- **Global selection defaults**: `~/.substrate/config.yaml`
- **Workspace selection**: `<workspace>/.substrate/workspace.yaml`

Selection is an ordered list of inventory item IDs (packages and bundles).

### Workspace-only lever
`world.deps.inventory_mode`:
- `merged` (default): built-ins + global + workspace inventories are visible.
- `workspace_only`: only inventories from the workspace chain are visible (built-ins/global hidden).

## User Contract (Authoritative)

This section mirrors the **scope and “current vs patch”** style used by `ADR-0008` so `world deps` reads like the rest of Substrate.

### CLI

#### `substrate world deps current list [--available|--selected|--applied] [--all] [--json]`
- Purpose: show the **effective** (merged) deps views for the current directory.
- `--available` (default):
  - Prints the **current inventory view** visible from `cwd` (after inventory merge + `world.deps.inventory_mode`).
  - It MUST NOT make world-agent calls.
  - Hints (stderr, only if empty):
    - `substrate: note: no deps inventory items visible for this directory; add definitions in ~/.substrate/deps.yaml or <workspace>/.substrate/deps.yaml`
- `--selected`:
  - Prints the **current selection** (effective merged selection for `cwd`) without querying world-agent.
  - Stderr (always):
    - `substrate: note: showing current effective deps selection for this directory`
  - Hints (stderr, when empty):
    - `substrate: hint: add deps with 'substrate world deps workspace add ...' (or '... global add ...') then apply with 'substrate world deps current sync'`
- `--applied`:
  - Prints world-agent-backed status for items.
  - Default scope: the current selected set.
  - `--all`: include every currently available inventory item (debug/bring-up only).
  - Stderr (always):
    - `substrate: note: showing current world deps status for this directory`
- Output MUST include, for each item:
  - `id` (`pkg:*` or `bundle:*`)
  - `selected=true|false` (selected in current selection)
  - `world=present|missing|blocked`
  - `next=<one-line remediation or empty>`
- Exit codes:
  - `0` success
  - `2` invalid YAML / unknown ids / invalid args
  - `3` world backend unavailable (only for `--applied`)
  - `1` unexpected

#### `substrate world deps current show <item_id> [--json] [--explain]`
- Prints the **current resolved definition** for `<item_id>` after inventory merges (same inventory view as `deps current list --available`).
- `--explain` adds:
  - Whether the item is selected in the current selection (and whether it is selected via global/workspace patch).
  - If the item is not satisfied in-world, it MUST print a single-line “why” plus the exact next command.
    - Example (direct): `Next: substrate world deps current install <item_id>`
    - Example (persist): `Next: substrate world deps workspace add <item_id> && substrate world deps current sync`
- Exit codes:
  - `0` success
  - `2` unknown item id / invalid inventory YAML
  - `3` world backend unavailable (only when `--explain` needs world status)
  - `1` unexpected

#### `substrate world deps global add <item_id...> [--json]`
- Applies a **global selection patch** update (does not install).
- It MUST:
  - Validate item IDs exist in the **current available inventory view** for `cwd`.
  - Write only `~/.substrate/config.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Selection updated (global): added: <csv>`
  - `substrate: note: selection changes apply to the world only after 'substrate world deps current sync'`
  - `Next: substrate world deps current sync`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps global remove <item_id...> [--json]`
- Same as `global add`, but removes items from the global selection patch.
- On success, it MUST print:
  - `Selection updated (global): removed: <csv>`
  - `substrate: note: selection changes apply to the world only after 'substrate world deps current sync'`
  - `substrate: note: this does not uninstall anything already present in the world`
  - `Next: substrate world deps current sync`
- Exit codes match `global add`.

#### `substrate world deps global reset [item_id ...] [--json]`
- Resets global deps selection back to defaults by editing only `~/.substrate/config.yaml`.
- If no `item_id` arguments are provided:
  - Resets the global deps selection patch to “unset” (inherit from defaults).
- If one or more `item_id` arguments are provided:
  - Removes only those IDs from the global selection patch.
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Selection reset (global)`
  - `Next: substrate world deps current sync`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps workspace add <item_id...> [--json]`
- Applies a **workspace selection patch** update (does not install).
- Requires `cwd` is within an enabled workspace (workspace root discovered from `cwd`).
- It MUST:
  - Validate item IDs exist in the current available inventory view for `cwd`.
  - Write only `<workspace_root>/.substrate/workspace.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Selection updated (workspace): added: <csv>`
  - `substrate: note: selection changes apply to the world only after 'substrate world deps current sync'`
  - `Next: substrate world deps current sync`
- Exit codes: `0` success (including no-op); `2` no workspace root / unknown ids / invalid YAML; `1` unexpected

#### `substrate world deps workspace remove <item_id...> [--json]`
- Same as `workspace add`, but removes items from the workspace selection patch.
- On success, it MUST print:
  - `Selection updated (workspace): removed: <csv>`
  - `substrate: note: selection changes apply to the world only after 'substrate world deps current sync'`
  - `substrate: note: this does not uninstall anything already present in the world`
  - `Next: substrate world deps current sync`
- Exit codes match `workspace add`.

#### `substrate world deps workspace reset [item_id ...] [--json]`
- Resets workspace deps selection back to inherited defaults by editing only `<workspace_root>/.substrate/workspace.yaml`.
- Requires `cwd` is within an enabled workspace.
- If no `item_id` arguments are provided:
  - Resets the workspace deps selection patch to “unset” (inherit from global/defaults).
- If one or more `item_id` arguments are provided:
  - Removes only those IDs from the workspace selection patch.
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Selection reset (workspace)`
  - `Next: substrate world deps current sync`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps current install <item_id...> [--dry-run] [--verbose]`
- Applies items immediately without modifying selection.
- It MUST:
  1) Expand bundles → packages.
  2) Apply **world image** installs first (apt).
  3) Apply **world deps prefix** installs second (scripts + entrypoints under `/var/lib/substrate/world-deps/bin`).
- `--dry-run`:
  - MUST print the computed plan (apt list + script package list) and exit `0` without side effects.
- On success, it MUST print:
  - A short summary of what was applied (world image + world deps prefix), then:
  - `Next: substrate world deps current list --applied`
- Guarantee (runnable packages):
  - After success, runnable package entrypoints are invokable in-world via the standard world execution path (interactive `substrate>` and non-interactive runs) without requiring shell RC sourcing.
- Exit codes:
  - `0` success
  - `2` unknown ids / invalid YAML / invalid inventory
  - `3` world backend unavailable
  - `5` hardening conflict (world is writable only under `/var/lib/substrate/world-deps` but the install needs broader writes)
  - `1` unexpected

#### `substrate world deps current sync [--dry-run] [--verbose] [--all]`
- Applies the **current selection** (effective for `cwd`) using the same engine as `deps current install`.
- `--all`:
  - Ignores selection and applies every visible inventory item (debug/bring-up only).
- On success, it MUST print:
  - A one-line confirmation plus:
  - `Next: substrate world deps current list --applied`
- Exit codes match `deps current install`.

#### `substrate world deps global list [--available|--selected] [--json]`
- `--available` (default):
  - Prints the **global inventory patch** at `~/.substrate/deps.yaml` (or `{}` if missing).
  - It MUST NOT incorporate workspace inventory.
  - It MUST NOT print built-in defaults; use `deps current list --available` for the merged view.
- `--selected`:
  - Prints the **global selection patch** at `~/.substrate/config.yaml` (or `{}` if missing).
- Exit codes: `0` success; `2` invalid YAML; `1` unexpected.

#### `substrate world deps workspace list [--available|--selected] [--json]`
- Requires `cwd` is within an enabled workspace.
- `--available` (default):
  - Prints the **workspace inventory patch** at `<workspace_root>/.substrate/deps.yaml` (or `{}` if missing).
  - It MUST NOT incorporate global inventory.
  - It MUST NOT print built-in defaults; use `deps current list --available` for the merged view.
- `--selected`:
  - Prints the **workspace selection patch** at `<workspace_root>/.substrate/workspace.yaml` (patch view; not merged).
- Exit codes: `0` success; `2` no workspace root / invalid YAML; `1` unexpected.

## World Status Semantics
`present/missing/blocked` is always from the world perspective.
- For **runnable packages**, `present` means the package entrypoint is invokable in-world via the standard world shell execution path.
- For **bundles**, `present` means all constituent packages are `present` (bundles are never invoked directly).

## Notes / Known follow-ups
- Some “managers” (e.g. `pkg:nvm`) are not the same as their runtimes (e.g. `pkg:node`). A manager package being `present` does not imply a runtime exists unless a runtime package/bundle is selected/installed.
