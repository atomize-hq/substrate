# World Deps — Packages/Bundles Contract (Draft)

This document defines the **user-facing contract** for `substrate world deps` using Substrate’s **host/world** terminology (no “guest” language). It is intentionally concise so we can iterate.

## Goals
- Make world deps a predictable way to declare and apply **in-world** dependencies.
- Unify the mental model around **inventory** (what exists) → **enabled** (what you want applied) → **applied** (what is present in the world).
- Avoid misleading names (no more `pyenv`-style “looks like a CLI but isn’t”).

## What “Replace” Means (End State)
- No user-facing `manager_hooks.yaml` semantics.
- No `world-deps.yaml` overlay mechanism (and no local variants).
- Inventory sources become:
  - Built-in defaults (shipped with Substrate; not user-edited)
  - `~/.substrate/deps/` (global inventory directory)
  - `<workspace_root>/.substrate/deps/` (workspace inventory directory)
- Enabled sources become:
  - `~/.substrate/config.yaml` (global enabled patch)
  - `<workspace_root>/.substrate/workspace.yaml` (workspace enabled patch)

### Legacy files removed from plumbing
In this end state, `world deps` MUST NOT read (or be influenced by) any of:
- `config/manager_hooks.yaml`
- `scripts/substrate/world-deps.yaml`
- `~/.substrate/manager_hooks.local.yaml`
- `~/.substrate/world-deps.local.yaml`
- `.substrate/world-deps.selection.yaml`
- `~/.substrate/world-deps.selection.yaml`
Replacement completeness requirement:
- Tests that previously validated legacy file loading MUST be updated to validate the new inventory/selection files, and MUST fail if `world deps` still reads any legacy file paths.

## Key Terms
- **Host**: the developer workstation environment running `substrate`.
- **World**: the isolated execution environment behind world-agent (Linux host, macOS Lima VM, Windows WSL).
- **Inventory**: definitions of available **packages** and **bundles**.
- **Enabled**: the desired set of inventory items for the **current directory** (resolved via sparse config merge).
- **World image** install: mutates OS-managed state in the world (e.g. apt/dpkg under `/usr`, `/var/lib/dpkg`).
- **World deps prefix** install: installs under `/var/lib/substrate/world-deps` and exposes entrypoints under `/var/lib/substrate/world-deps/bin`.

## Inventory Model
### Item types
- **Package**: an installable unit (via apt or via a script).
  - Install methods:
    - **apt** (world image install)
    - **script** (world deps prefix install; must create/ensure an entrypoint under `/var/lib/substrate/world-deps/bin`)
- **Bundle**: named group of packages (no installer; expands to packages).

### Naming rules (avoid collisions)
- Package and bundle names are bare (e.g. `bun`, `node-runtime`).
- A name MUST NOT exist in both `packages` and `bundles` after inventory merge (selection must be unambiguous).
- Runnable CLIs MUST only be represented as **packages** (never bundles).
- Non-runnable packages MUST be named like prerequisites (e.g. `python-build-deps`, `node-toolchain-deps`) and MUST NOT reuse well-known CLI names.

### Package “runnable” requirement (prevents `pyenv` confusion)
Every package MUST declare whether it is runnable in-world:
- `runnable: true` means the user is expected to invoke a CLI entrypoint in the world.
- `runnable: false` means the package exists only to satisfy prerequisites (it is not a user-facing CLI).

### Inventory schema (`deps/`)
Inventory is a directory, not a single file.

Layout (per scope):
- Packages: `<scope>/.substrate/deps/packages/<dep_name>.yaml`
- Bundles: `<scope>/.substrate/deps/bundles/<dep_name>.yaml`

The item name is the filename without `.yaml` (e.g. `packages/bun.yaml` defines package `bun`).
For safety, the YAML inside the file MUST also declare `name: ...`, and it MUST match the filename-derived name exactly.

#### Package file schema (`deps/packages/<dep_name>.yaml`)
```yaml
version: 1
name: <package_name>                 # required; MUST match the filename (<dep_name>.yaml)
description: <string optional>
runnable: <bool>                      # required
entrypoints: [<string>...]            # required when runnable=true (e.g. ["bun"])
platforms: [linux|macos|windows]      # optional allowlist; default: all
install:                              # required
  method: apt | script | manual
  apt: [<apt_pkg>...]                 # required iff method=apt
  script_path: <string>               # recommended iff method=script (see deps/scripts/ below)
  script: |                           # allowed iff method=script (inline fallback)
    <sh script>                       # used only when script_path is omitted
  manual_instructions: |              # required iff method=manual
    <text>
probe:                                # optional; overrides default presence checks
  command: <string>                   # run inside the world; present iff exit 0
```

#### Bundle file schema (`deps/bundles/<dep_name>.yaml`)
```yaml
version: 1
name: <bundle_name>                   # required; MUST match the filename (<dep_name>.yaml)
description: <string optional>
platforms: [linux|macos|windows]      # optional allowlist; default: all
packages: [<package_name>...]
```

### Script install sources (`deps/scripts/`)
For `method: script`, inventory MAY embed scripts inline, but SHOULD use a script path for maintainability.

Recommended layout (mirrors scope):
- Global: `~/.substrate/deps/scripts/`
- Workspace: `<workspace_root>/.substrate/deps/scripts/`

Script path resolution:
- If `install.script_path` is relative, it is resolved relative to the package definition file that declared it.
  - Example (from `deps/packages/bun.yaml`): `script_path: ../scripts/bun.sh`
- If `install.script_path` is absolute, it is used as-is.
- If both `install.script_path` and inline `install.script` are provided, `script_path` MUST take precedence.

Default probe behavior when `probe.command` is omitted:
- For `runnable: true`: present iff every `entrypoints[]` is invokable via `command -v <entrypoint>` in the world.
- For `runnable: false`: present iff the package’s `install` requirements are satisfied (implementation-defined; non-runnable packages SHOULD provide an explicit `probe.command` to keep status deterministic).

### Inventory sources and merge order
Inventory is resolved by merging these sources (later layers override earlier):
1) **Built-in defaults** shipped with Substrate (configurable to hide/disable; not edited directly).
2) **Global user inventory**: `~/.substrate/deps/`
3) **Workspace inventory chain**: from the current directory upward, merge any `<dir>/.substrate/deps/` found (nearest overrides earlier ancestors).

Workspace inventories **extend** global/built-ins by default. A workspace may opt into **workspace-only inventory** via enabled config (see below).

## Enabled Model (Sparse YAML)
Enabled deps are resolved from sparse YAML config merged by current working directory:
- **Global enabled defaults**: `~/.substrate/config.yaml`
- **Workspace enabled**: `<workspace_root>/.substrate/workspace.yaml`

Enabled deps are an ordered list of inventory item names (packages and bundles).

### Enabled schema (patch keys)
Enabled deps live under `world.deps`:
```yaml
world:
  deps:
    # Canonical shape (multi-line YAML list).
    enabled:
      - <item_name>
      - <item_name>
      - <item_name>
    inventory_mode: merged | workspace_only
```

Rules:
- `world.deps.enabled` MUST be a YAML list of non-empty strings when present.
- Ordering is preserved (the order in `enabled` is the order applied/printed).
- Duplicate names MUST be ignored after the first occurrence.

## Patch File Comment Headers (Examples)

World deps uses the same “patch file” concept as `ADR-0008`: the file at a scope contains only overrides for that scope, and commands MUST preserve any existing comment header.
You MAY also edit these files directly; the CLI is a convenience layer over YAML patches (invalid YAML is an actionable user error).

### Global enabled patch (`~/.substrate/config.yaml`)
```yaml
# Substrate world deps enabled patch (global scope).
# - Update via:
#   - `substrate world deps global add ...`
#   - `substrate world deps global remove ...`
#   - `substrate world deps global reset ...`
# - Or edit this file directly (YAML).
# - Changes do not affect the world until you run:
#   - `substrate world deps current sync`
# - Inspect the effective view for your current directory:
#   - `substrate world deps current list enabled`
#   - `substrate world deps current list applied`
world:
  deps:
    enabled:
      - "bun"
      - "node-runtime"
    inventory_mode: merged
```

### Workspace enabled patch (`<workspace_root>/.substrate/workspace.yaml`)
```yaml
# Substrate world deps enabled patch (workspace scope).
# - Update via:
#   - `substrate world deps workspace add ...`
#   - `substrate world deps workspace remove ...`
#   - `substrate world deps workspace reset ...`
# - Or edit this file directly (YAML).
# - Changes do not affect the world until you run:
#   - `substrate world deps current sync`
world:
  deps:
    enabled:
      - "python-build-deps"
```

### Inventory directory (`~/.substrate/deps/` or `<workspace_root>/.substrate/deps/`)
Per-item files live at:
- `deps/packages/<dep_name>.yaml`
- `deps/bundles/<dep_name>.yaml`
```yaml
# Inventory is a directory:
#   <scope>/.substrate/deps/
#
# Example package file:
#   <scope>/.substrate/deps/packages/<dep_name>.yaml
version: 1
name: bun
description: Bun runtime
runnable: true
entrypoints: ["bun"]
install:
  method: script
  script_path: ../scripts/bun.sh
probe:
  command: "bun --version"
```

```yaml
# Example bundle file:
#   <scope>/.substrate/deps/bundles/<dep_name>.yaml
version: 1
name: node-runtime
packages: ["node", "npm"]
```

### Workspace-only lever
`world.deps.inventory_mode`:
- `merged` (default): built-ins + global + workspace inventories are visible.
- `workspace_only`: only inventories from the workspace chain are visible (built-ins/global hidden).

## User Contract (Authoritative)

This section mirrors the **scope and “current vs patch”** style used by `ADR-0008` so `world deps` reads like the rest of Substrate.

### CLI

#### `substrate world deps current list [available|enabled|applied] [--all] [--json]`
- Purpose: show the **effective** (merged) deps views for the current directory.
- `available` (default):
  - Prints the **current inventory view** visible from `cwd` (after inventory merge + `world.deps.inventory_mode`).
  - It MUST NOT make world-agent calls.
  - Hints (stderr, only if empty):
    - `substrate: note: no deps inventory items visible for this directory; add definitions under ~/.substrate/deps/ or <workspace>/.substrate/deps/`
- `enabled`:
  - Prints the **current enabled list** (effective merged enabled list for `cwd`) without querying world-agent.
  - Stderr (always):
    - `substrate: note: showing current effective enabled deps list for this directory`
  - Hints (stderr, when empty):
    - `substrate: hint: add deps with 'substrate world deps workspace add ...' (or '... global add ...') then apply with 'substrate world deps current sync'`
- `applied`:
  - Prints world-agent-backed status for items.
  - Default scope: the current enabled set.
  - `--all`: include every currently available inventory item (debug/bring-up only). Valid only with `applied`.
  - Stderr (always):
    - `substrate: note: showing current world deps status for this directory`
- Output MUST include, for each item (view-dependent):
  - Always: `name` (string) and `kind=package|bundle`
  - For `enabled` and `applied`: `enabled=true|false` (enabled in current enabled list)
  - For `applied`: `world=present|missing|blocked`
  - Optional (only for `applied`): `remediation=<one-line remediation or empty>`
- Exit codes:
  - `0` success
  - `2` invalid YAML / unknown ids / invalid args
  - `3` world backend unavailable (only for `applied`)
  - `1` unexpected

#### `substrate world deps current show <item_name> [--json] [--explain]`
- Prints the **current resolved definition** for `<item_name>` after inventory merges (same inventory view as `deps current list available`).
- `--explain` adds:
  - Whether the item is enabled in the current enabled list (and whether it is enabled via global/workspace patch).
  - If the item is not satisfied in-world, it MUST print a single-line “why” plus the exact next command.
    - Example (direct): `substrate: hint: run 'substrate world deps current install <item_name>'`
    - Example (persist): `substrate: hint: run 'substrate world deps workspace add <item_name>' then 'substrate world deps current sync'`
- Exit codes:
  - `0` success
  - `2` unknown item id / invalid inventory YAML
  - `3` world backend unavailable (only when `--explain` needs world status)
  - `1` unexpected

#### `substrate world deps global add <item_name...> [--json]`
- Applies a **global enabled patch** update (does not install).
- It MUST:
  - Validate item names exist in the **current available inventory view** for `cwd`.
  - Write only `~/.substrate/config.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (global): added: <csv>`
  - `substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps global remove <item_name...> [--json]`
- Same as `global add`, but removes items from the global enabled patch.
- On success, it MUST print:
  - `Enabled deps updated (global): removed: <csv>`
  - `substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply`
- Exit codes match `global add`.

#### `substrate world deps global reset [item_name ...] [--json]`
- Resets global enabled deps back to defaults by editing only `~/.substrate/config.yaml`.
- If no `item_name` arguments are provided:
  - Resets the global enabled deps patch to “unset” (inherit from defaults).
- If one or more `item_name` arguments are provided:
  - Removes only those names from the global enabled deps patch.
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Enabled deps reset (global)`
  - `substrate: note: run 'substrate world deps current sync' to apply enabled deps changes`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps workspace add <item_name...> [--json]`
- Applies a **workspace enabled patch** update (does not install).
- Requires `cwd` is within an enabled workspace (workspace root discovered from `cwd`).
- It MUST:
  - Validate item names exist in the current available inventory view for `cwd`.
  - Write only `<workspace_root>/.substrate/workspace.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (workspace): added: <csv>`
  - `substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'`
- Exit codes: `0` success (including no-op); `2` no workspace root / unknown ids / invalid YAML; `1` unexpected

#### `substrate world deps workspace remove <item_name...> [--json]`
- Same as `workspace add`, but removes items from the workspace enabled patch.
- On success, it MUST print:
  - `Enabled deps updated (workspace): removed: <csv>`
  - `substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply`
- Exit codes match `workspace add`.

#### `substrate world deps workspace reset [item_name ...] [--json]`
- Resets workspace enabled deps back to inherited defaults by editing only `<workspace_root>/.substrate/workspace.yaml`.
- Requires `cwd` is within an enabled workspace.
- If no `item_name` arguments are provided:
  - Resets the workspace enabled deps patch to “unset” (inherit from global/defaults).
- If one or more `item_name` arguments are provided:
  - Removes only those names from the workspace enabled deps patch.
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Enabled deps reset (workspace)`
  - `substrate: note: run 'substrate world deps current sync' to apply enabled deps changes`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps current install <item_name...> [--dry-run] [--verbose]`
- Applies items immediately without modifying selection.
- It MUST:
  1) Expand bundles → packages.
  2) Apply **world image** installs first (apt).
  3) Apply **world deps prefix** installs second (scripts + entrypoints under `/var/lib/substrate/world-deps/bin`).
- `--dry-run`:
  - MUST print the computed plan (apt list + script package list) and exit `0` without side effects.
- On success, it MUST print:
  - A short summary of what was applied (world image + world deps prefix), then:
  - `substrate: note: this updates the world only (selection not modified)`
  - `substrate: hint: run 'substrate world deps current list applied' to verify`
- Guarantee (runnable packages):
  - After success, runnable package entrypoints are invokable in-world via the standard world execution path (interactive `substrate>` and non-interactive runs) without requiring shell RC sourcing.
- Exit codes:
  - `0` success
  - `2` unknown ids / invalid YAML / invalid inventory
  - `3` world backend unavailable
  - `5` hardening conflict (world is writable only under `/var/lib/substrate/world-deps` but the install needs broader writes)
  - `1` unexpected

#### `substrate world deps current sync [--dry-run] [--verbose] [--all]`
- Applies the **current enabled list** (effective for `cwd`) using the same engine as `deps current install`.
- `--all`:
  - Ignores enabled list and applies every visible inventory item (debug/bring-up only).
- On success, it MUST print:
  - A one-line confirmation plus:
  - `substrate: note: applied effective enabled deps list for this directory (sources: workspace, global, defaults as applicable)`
  - `substrate: hint: run 'substrate world deps current list applied' to verify`
- Exit codes match `deps current install`.

#### `substrate world deps global list [available|enabled] [--json]`
- `available` (default):
  - Prints the **global inventory patch view** from `~/.substrate/deps/` (or empty if missing).
  - It MUST NOT incorporate workspace inventory.
  - It MUST NOT print built-in defaults; use `deps current list available` for the merged view.
- `enabled`:
  - Prints the **global enabled deps patch** at `~/.substrate/config.yaml` (or `{}` if missing).
- Exit codes: `0` success; `2` invalid YAML; `1` unexpected.

#### `substrate world deps workspace list [available|enabled] [--json]`
- Requires `cwd` is within an enabled workspace.
- `available` (default):
  - Prints the **workspace inventory patch view** from `<workspace_root>/.substrate/deps/` (or empty if missing).
  - It MUST NOT incorporate global inventory.
  - It MUST NOT print built-in defaults; use `deps current list available` for the merged view.
- `enabled`:
  - Prints the **workspace enabled deps patch** at `<workspace_root>/.substrate/workspace.yaml` (patch view; not merged).
- Exit codes: `0` success; `2` no workspace root / invalid YAML; `1` unexpected.

## World Status Semantics
`present/missing/blocked` is always from the world perspective.
- For **runnable packages**, `present` means the package entrypoint is invokable in-world via the standard world shell execution path.
- For **bundles**, `present` means all constituent packages are `present` (bundles are never invoked directly).

## Notes / Known follow-ups
- Some packages are “managers” (e.g. `nvm`) and are not the same as their runtimes (e.g. `node`). A manager package being `present` does not imply a runtime exists unless a runtime package/bundle is enabled/installed.
