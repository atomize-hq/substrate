# ADR-0011 — World Deps Packages/Bundles Inventory + Enabled Contract

> NOTICE (2026-01-24)
>
> This ADR is aligned to ADR-0016 interactive REPL semantics:
> - Non-interactive world execution uses `/bin/sh -c`.
> - Interactive `substrate>` REPL evaluation uses a bash evaluator with no rcfiles (`/bin/bash --noprofile --norc`).

## Status
- Status: Approved
- Date (UTC): 2026-01-13
- Owner(s): Shell / World maintainers

## Scope
- Feature directories (impacted):
  - `docs/project_management/_archived/next/` (this ADR; cross-cutting contract)
  - `docs/project_management/_archived/world_deps_selection_layer/` (related work; see notes under Sequencing)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)

## Related Docs
- Source contract doc (must remain in parity with this ADR’s contract section):
  - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`
- Planning Pack (execution v4; this feature directory):
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/plan.md`
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/tasks.json`
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/spec_manifest.md`
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/decision_register.md`
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/impact_map.md`
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/manual_testing_playbook.md`
- Existing world-deps work (may be superseded / requires reconciliation if this ADR is Accepted):
  - `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
  - `docs/project_management/_archived/world_deps_selection_layer/plan.md`
  - `docs/project_management/_archived/world_deps_selection_layer/decision_register.md`
- Patch-file (scope/current/global/workspace) mental model:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Per-key merge strategies + multi-source provenance (for `world.deps.*` keys):
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Exit codes:
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 2b1783166231cc55d91de12770705f87fd3ea92737d1611e9a6e39a1fa3cdb15
### Changes (operator-facing)
- World deps becomes “inventory + enabled patches”
  - Existing: world-deps behavior is anchored on legacy manifest/overlay/selection files (`manager_hooks.yaml`, `world-deps.yaml`, `world-deps.selection.yaml`) with semantics that are easy to misread and hard to reason about across scopes.
  - New: `substrate world deps` is driven by an inventory directory model (`$SUBSTRATE_HOME/deps/`, `<workspace_root>/.substrate/deps/`) plus enabled patch keys in YAML (`$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`), with explicit `current|global|workspace` CLI scopes.
  - Why: makes “what exists / what you want / what is applied” explicit and scriptable; removes misleading “looks like a CLI but isn’t” cases.
  - Links:
    - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`
    - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
    - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`

- Legacy file paths are removed from world-deps plumbing
  - Existing: world-deps behavior can be influenced by multiple historical file locations (including overlay and selection files).
  - New: `world deps` MUST NOT read (or be influenced by) any legacy world-deps paths; inventory/enabled sources are limited to the new directories/patch files plus built-in defaults.
  - Why: prevents silent drift, hidden overrides, and “it works on one machine” confusion; makes tests enforce the end state.
  - Links:
    - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

## Problem / Context
- `substrate world deps` needs a stable, predictable, scope-aware contract for declaring and applying in-world dependencies.
- The current/legacy manifest + overlay + selection semantics make it too easy to confuse “available” tools with “enabled” tools, and “enabled” tools with “installed/applied” tools.
- Substrate already has a strong “scope + patch file” mental model (ADR-0008); world-deps should read like the rest of Substrate, not like a one-off subsystem with bespoke file semantics.

## Goals
- Make world deps a predictable way to declare and apply **in-world** dependencies.
- Unify the mental model around **inventory** (what exists) → **enabled** (what you want applied) → **applied** (what is present in the world).
- Avoid misleading names (no more `pyenv`-style “looks like a CLI but isn’t”).
- Remove legacy world-deps files from plumbing so the end state is enforceable (tests fail if legacy paths are still read).

## Non-Goals
- Designing a general multi-distro package manager abstraction (beyond the `apt` contract explicitly specified here).
- Introducing migrations/back-compat layers for legacy world-deps files (default policy is greenfield/breaking unless explicitly required).
- Defining Substrate’s full world backend capability contract (see `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`).
- Guaranteeing that every package/bundle is supported on every platform; platform filtering and “unsupported/manual” flows are explicit parts of the contract.

## User Contract (Authoritative)

This section is a direct, parity-preserving conversion of:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

It is authoritative; other sections in this ADR must not contradict it.

This contract defines the **user-facing contract** for `substrate world deps` using Substrate’s **host/world** terminology (no “guest” language). It is intentionally concise so we can iterate.

Global paths in this contract are rooted at `$SUBSTRATE_HOME` (defaults to `~/.substrate` when `SUBSTRATE_HOME` is unset).

### Goals
- Make world deps a predictable way to declare and apply **in-world** dependencies.
- Unify the mental model around **inventory** (what exists) → **enabled** (what you want applied) → **applied** (what is present in the world).
- Avoid misleading names (no more `pyenv`-style “looks like a CLI but isn’t”).

### What “Replace” Means (End State)
- No user-facing `manager_hooks.yaml` semantics.
- No `world-deps.yaml` overlay mechanism (and no local variants).
- Inventory sources become:
  - Built-in defaults (shipped with Substrate; not user-edited)
  - `$SUBSTRATE_HOME/deps/` (global inventory directory)
  - `<workspace_root>/.substrate/deps/` (workspace inventory directory)
- Enabled sources become:
  - `$SUBSTRATE_HOME/config.yaml` (global enabled patch)
  - `<workspace_root>/.substrate/workspace.yaml` (workspace enabled patch)

#### Legacy files removed from plumbing
In this end state, `world deps` MUST NOT read (or be influenced by) any of:
- `config/manager_hooks.yaml`
- `scripts/substrate/world-deps.yaml`
- `$SUBSTRATE_HOME/manager_hooks.local.yaml`
- `$SUBSTRATE_HOME/world-deps.local.yaml`
- `.substrate/world-deps.selection.yaml`
- `$SUBSTRATE_HOME/world-deps.selection.yaml`
Replacement completeness requirement:
- Tests that previously validated legacy file loading MUST be updated to validate the new inventory/enabled files, and MUST fail if `world deps` still reads any legacy file paths.

### Key Terms
- **Host**: the developer workstation environment running `substrate`.
- **World**: the isolated execution environment behind world-agent (Linux host, macOS Lima VM, Windows WSL).
- **Inventory**: definitions of available **packages** and **bundles**.
- **Enabled**: the desired set of inventory items for the **current directory** (resolved via sparse config merge).
- **World image** install: mutates OS-managed state in the world (e.g. apt/dpkg under `/usr`, `/var/lib/dpkg`).
- **World deps prefix** install: installs under `/var/lib/substrate/world-deps` and exposes entrypoints under `/var/lib/substrate/world-deps/bin`.

### World Shell Contract (Why `nvm` Needs a Wrapper)
Substrate world execution is intentionally conservative and does not behave like an interactive login shell.

Contract:
- World commands executed via non-interactive pathways (e.g., `substrate -c`, automation, world-agent `/v1/execute`) execute under `/bin/sh -c` in the world, with no user shell rc sourcing.
- Interactive REPL sessions (`substrate>`) execute under the world-first persistent-session model and evaluate submissions under `/bin/bash --noprofile --norc -c` (still no user rc sourcing).
- Therefore, runnable deps MUST expose real executable entrypoints (files) and MUST NOT rely on shell functions, aliases, or `~/.bashrc`-style initialization. If a tool requires shell init, it MUST be made runnable via a generated wrapper entrypoint (e.g., `bash_function` / `bash_source_exec` wrappers).

Install-time note:
- Script-based installs (`install.method=script`) MAY run under `bash -lc` for compatibility with common installer recipes, but that does not change the runtime execution contract above.

Implication for `nvm`-style deps:
- `nvm` is a shell function defined by sourcing `nvm.sh`. It is not invokable under `/bin/sh -c` unless we provide a wrapper.
- If we ship a runnable package named `nvm`, its `entrypoints: ["nvm"]` MUST resolve to a real executable in-world (typically a wrapper placed under `/var/lib/substrate/world-deps/bin/nvm`) that:
  - invokes `bash -lc ...` internally,
  - sources the installed `nvm.sh`,
  - then runs `nvm "$@"`,
  - and fails with an actionable error if `bash` is unavailable.

### Notes (ADR-0016 impact)
- ADR-0016 introduces a persistent in-world REPL session that uses `bash --noprofile --norc` (no rcfiles) for interactive `substrate>` sessions.
- This ADR’s world-deps contract should remain compatible with non-interactive world execution (`/bin/sh -c`) and must not rely on shell init behavior for “runnable” packages.
- For consistency, the world-first REPL session environment should include `/var/lib/substrate/world-deps/bin` in `PATH` so enabled deps are runnable without requiring manual PATH edits.

### Inventory Model

#### Item types
- **Package**: an installable unit (via apt or via a script).
  - Install methods:
    - **apt** (world image install)
    - **script** (world deps prefix install; must create/ensure an entrypoint under `/var/lib/substrate/world-deps/bin`)
- **Bundle**: named group of packages (no installer; expands to packages).

#### Naming rules (avoid collisions)
- Package and bundle names are bare (e.g. `bun`, `node-runtime`).
- A name MUST NOT exist in both `packages` and `bundles` after inventory merge (enabled names must be unambiguous).
- Runnable CLIs MUST only be represented as **packages** (never bundles).
- Non-runnable packages MUST be named like prerequisites (e.g. `python-build-deps`, `node-toolchain-deps`) and MUST NOT reuse well-known CLI names.

#### Package “runnable” requirement (prevents `pyenv` confusion)
Every package MUST declare whether it is runnable in-world:
- `runnable: true` means the user is expected to invoke a CLI entrypoint in the world.
- `runnable: false` means the package exists only to satisfy prerequisites (it is not a user-facing CLI).

#### Inventory schema (`deps/`)
Inventory is a directory, not a single file.

Layout (per scope):
- Packages: `<scope>/.substrate/deps/packages/<dep_name>.yaml`
- Bundles: `<scope>/.substrate/deps/bundles/<dep_name>.yaml`

The item name is the filename without `.yaml` (e.g. `packages/bun.yaml` defines package `bun`).
For safety, the YAML inside the file MUST also declare `name: ...`, and it MUST match the filename-derived name exactly.

##### Package file schema (`deps/packages/<dep_name>.yaml`)
```yaml
version: 1
name: <package_name>                 # required; MUST match the filename (<dep_name>.yaml)
description: <string optional>
runnable: <bool>                      # required
entrypoints: [<string>...]            # required when runnable=true (e.g. ["bun"])
wrappers:                             # optional; used for function/rc-style tools (e.g. nvm)
  # Each wrapper declares how Substrate should generate a runnable entrypoint file under:
  #   /var/lib/substrate/world-deps/bin/<name>
  #
  # Use wrappers when the “real” tool is not a stable executable (e.g. it is a shell function),
  # or when the tool requires sourcing an env script before invocation.
  - name: <entrypoint_name>           # required; MUST be listed in entrypoints[]
    kind: bash_function | bash_source_exec | sh_env_exec
    # bash_function:
    #   - For tools that are defined as bash functions after sourcing a script (e.g. nvm).
    #   - Requires bash in-world.
    bash_source: <string>             # required for bash_* kinds; e.g. "$HOME/.nvm/nvm.sh"
    function: <string>                # required for kind=bash_function; e.g. "nvm"
    # bash_source_exec:
    #   - Source bash_source, then exec a command (useful for env scripts that define PATH).
    #   - Requires bash in-world.
    exec: <string>                    # required for kind=bash_source_exec; e.g. "node" or "python"
    # sh_env_exec:
    #   - Set env vars, then exec a command. Does not require bash.
    env:                              # required for kind=sh_env_exec
      <KEY>: <VALUE>
    exec: <string>                    # required for kind=sh_env_exec; e.g. "foo"
platforms: [linux|macos|windows]      # optional allowlist; default: all (host platform)
install:                              # required
  method: apt | script | manual
  apt:                                # required iff method=apt
    - name: <apt_package_name>
      version: <string optional>      # when omitted, installs the default candidate/latest
  script_path: <string>               # recommended iff method=script (see deps/scripts/ below)
  script: |                           # allowed iff method=script (inline fallback)
    <sh script>                       # used only when script_path is omitted
  manual_instructions: |              # required iff method=manual
    <text>
probe:                                # optional; overrides default presence checks
  command: <string>                   # run inside the world; present iff exit 0
```

##### Bundle file schema (`deps/bundles/<dep_name>.yaml`)
```yaml
version: 1
name: <bundle_name>                   # required; MUST match the filename (<dep_name>.yaml)
description: <string optional>
platforms: [linux|macos|windows]      # optional allowlist; default: all (host platform)
packages: [<package_name>...]
```

#### APT install notes (`install.method=apt`)
- Each `install.apt[]` entry MAY specify `version`.
- If `version` is omitted, Substrate installs the default candidate (equivalent to `apt-get install <name>`).
- If `version` is specified, Substrate installs exactly that version (equivalent to `apt-get install <name>=<version>`); if unavailable, the install MUST fail with an actionable error.

#### Wrapper generation (`wrappers[]`)
`wrappers[]` is an optional declarative mechanism to make function/rc-style tools runnable under the world shell contract.

Contract:
- For each `wrappers[]` entry, Substrate MUST generate an executable entrypoint at:
  - `/var/lib/substrate/world-deps/bin/<name>`
- Wrapper generation MUST be deterministic:
  - The wrapper path is fixed by `<name>`.
  - The wrapper contents MUST be a stable rendering of the package definition (no timestamps/randomness).
  - Wrapper generation MUST be idempotent (re-running `sync` does not change wrapper contents unless the definition changes).
- Wrapper kinds:
  - `bash_function`:
    - The wrapper MUST execute `bash -lc ...` (not `sh`) so it can `source` bash scripts and invoke the function.
    - The wrapper MUST source `bash_source`, then invoke `<function> "$@"`.
    - If `bash` is unavailable, the wrapper MUST fail with an actionable error (`bash is required for <name>; install bash in the world`).
  - `bash_source_exec`:
    - The wrapper MUST execute `bash -lc ...`, source `bash_source`, then `exec <exec> "$@"`.
    - If `bash` is unavailable, it MUST fail with an actionable error.
  - `sh_env_exec`:
    - The wrapper MUST be a POSIX `sh` script that exports each `env` entry, then `exec <exec> "$@"`.

Observability requirements:
- On wrapper failure, stderr MUST include:
  - the wrapper kind (`bash_function|bash_source_exec|sh_env_exec`)
  - the resolved `bash_source` path when applicable
  - whether `bash` was found when applicable
  - a single-line next step (e.g. install `bash`, fix env var, or run `substrate world deps current show <name> --explain`)
- `substrate world deps current show <name> --explain` MUST surface wrapper details:
  - wrapper kind and key fields (`bash_source`, `function`/`exec`, env keys)
  - the exact invocation shape that will be used (e.g. `bash -lc 'source ...; ...'`)

#### Script install sources (`deps/scripts/`)
For `method: script`, inventory MAY embed scripts inline, but SHOULD use a script path for maintainability.

Recommended layout (mirrors scope):
- Global: `$SUBSTRATE_HOME/deps/scripts/`
- Workspace: `<workspace_root>/.substrate/deps/scripts/`

Script path resolution:
- If `install.script_path` is relative, it is resolved relative to the package definition file that declared it.
  - Example (from `deps/packages/bun.yaml`): `script_path: ../scripts/bun.sh`
- If `install.script_path` is absolute, it is used as-is.
- If both `install.script_path` and inline `install.script` are provided, `script_path` MUST take precedence.

Default probe behavior when `probe.command` is omitted:
- For `runnable: true`: present iff every `entrypoints[]` is invokable via `command -v <entrypoint>` in the world.
- For `runnable: false`: present iff the package’s `install` requirements are satisfied (implementation-defined; non-runnable packages SHOULD provide an explicit `probe.command` to keep status deterministic).

#### Inventory sources and merge order
Inventory is resolved by merging these sources (later layers override earlier):
1) **Built-in defaults** shipped with Substrate (configurable to hide/disable; not edited directly).
2) **Global user inventory**: `$SUBSTRATE_HOME/deps/`
3) **Workspace inventory chain**: from the current directory upward, merge any `<dir>/.substrate/deps/` found (nearest overrides earlier ancestors).

Workspace inventories **extend** global/built-ins by default. A workspace may opt into **workspace-only inventory** via enabled config (see below).

Merge rules:
- Inventory is merged by item name and kind:
  - `packages/<name>.yaml` defines package `<name>`.
  - `bundles/<name>.yaml` defines bundle `<name>`.
- When the same `<name>` is defined in multiple inventory layers, the closest layer to `cwd` MUST replace the definition (full replacement; not per-field merge).
- It is an error if, after merge, the same name exists in both packages and bundles.
- Inventory is filtered by platform:
  - If an item declares `platforms`, it is visible only when the host platform is in the list.
  - Non-matching items are treated as non-existent (they do not appear in `available` and cannot be enabled).

### Enabled Model (Sparse YAML)
Enabled deps are resolved from sparse YAML config merged by current working directory:
- **Global enabled defaults**: `$SUBSTRATE_HOME/config.yaml`
- **Workspace enabled**: `<workspace_root>/.substrate/workspace.yaml`

Enabled deps are an ordered list of inventory item names (packages and bundles).

#### Enabled schema (patch keys)
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
    builtins: enabled | disabled
```

Rules:
- `world.deps.enabled` MUST be a YAML list of non-empty strings when present.
- Ordering is preserved (the order in `enabled` is the order applied/printed).
- Duplicate names MUST be ignored after the first occurrence.
- Every enabled name MUST exist in the effective available inventory view for `cwd`; otherwise it is a configuration error (exit `2`).
- `world.deps.builtins` controls whether Substrate-shipped inventory defaults are visible:
  - `enabled` (default): built-ins participate in inventory merge.
  - `disabled`: built-ins are excluded from inventory merge (only user-provided inventory directories apply).
- Enabled list merge:
  - The effective enabled list for `cwd` is computed by concatenating enabled lists from applicable scopes, then de-duplicating in-order:
    1) global enabled list (`$SUBSTRATE_HOME/config.yaml`), then
    2) workspace enabled list (`<workspace_root>/.substrate/workspace.yaml`, when a workspace exists and is enabled).
  - A scope can “contribute nothing” by omitting `world.deps.enabled` (inherit-only); it can “contribute an explicit empty list” by setting `world.deps.enabled: []`.

### Implementation dependency: shared config model (Phase C)
`world deps` reads and edits the same config patch files defined by `ADR-0008`, and it MUST be implemented against the shared config schema/merge/editor model (`ADR-0008` + `ADR-0012`), not bespoke YAML patch handling.

Requirements:
- `world deps ... add|remove|reset` MUST mutate patch files via the shared config editor so allowlisting, type validation, per-key merge semantics, and comment-header preservation are consistent across the CLI surface.
- `world deps current` MUST read effective `world.deps.*` keys via the shared per-key merge engine, with `world.deps.enabled` using the concat-then-in-order-dedupe semantics in this contract (`ADR-0012`: `concat_dedupe_ordered_set`).

### Patch File Comment Headers (Examples)

World deps uses the same “patch file” concept as `ADR-0008`: the file at a scope contains only overrides for that scope, and commands MUST preserve any existing comment header.
You MAY also edit these files directly; the CLI is a convenience layer over YAML patches (invalid YAML is an actionable user error).

#### Global enabled patch (`$SUBSTRATE_HOME/config.yaml`)
```yaml
# Substrate world deps enabled patch (global scope).
# - Update via:
#   - `substrate world deps global add ...`
#   - `substrate world deps global remove ...`
#   - `substrate world deps global reset`
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
    builtins: enabled
```

#### Workspace enabled patch (`<workspace_root>/.substrate/workspace.yaml`)
```yaml
# Substrate world deps enabled patch (workspace scope).
# - Update via:
#   - `substrate world deps workspace add ...`
#   - `substrate world deps workspace remove ...`
#   - `substrate world deps workspace reset`
# - Or edit this file directly (YAML).
# - Changes do not affect the world until you run:
#   - `substrate world deps current sync`
world:
  deps:
    enabled:
      - "python-build-deps"
```

#### Inventory directory (`$SUBSTRATE_HOME/deps/` or `<workspace_root>/.substrate/deps/`)
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

#### Workspace-only lever
`world.deps.inventory_mode`:
- `merged` (default): built-ins + global + workspace inventories are visible.
- `workspace_only`: only inventories from the workspace chain are visible (built-ins/global hidden).

### CLI

This section mirrors the **scope and “current vs patch”** style used by `ADR-0008` so `world deps` reads like the rest of Substrate.

#### `substrate world deps current list [available|enabled|applied] [--all] [--json]`
- Purpose: show the **effective** (merged) deps views for the current directory.
- `available` (default):
  - Prints the **current inventory view** visible from `cwd` (after inventory merge + `world.deps.inventory_mode`).
  - Output SHOULD be a table.
  - Table columns MUST include: `source`, `kind`, `name`, `runnable`, `method`, `entrypoints`, `platforms`, `description`.
    - `source` MUST be one of: `builtin`, `global`, `workspace` and indicates which scope contributed the **effective definition** after inventory merge + platform filtering + `world.deps.inventory_mode` (full-replace by item name).
  - It MUST NOT make world-agent calls.
  - Hints (stderr, only if empty):
    - `substrate: note: no deps inventory items visible for this directory; add definitions under $SUBSTRATE_HOME/deps/ or <workspace_root>/.substrate/deps/`
- `enabled`:
  - Prints the **current enabled list** (effective merged enabled list for `cwd`) without querying world-agent.
  - If any enabled name does not exist in the effective available inventory view, it MUST fail with exit `2` and list the unknown names.
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
  - For `available`: `source=builtin|global|workspace` (effective inventory provenance after merge; required in table output and `--json`)
  - For `enabled`: list items are ordered and MUST match the effective `world.deps.enabled` list; `enabled=true` is implied.
  - For `applied`: `enabled=true|false` (enabled in the effective enabled list)
  - For `applied`: `world=present|missing|blocked`
  - Optional (only for `applied`): `remediation=<one-line remediation or empty>`
- Exit codes:
  - `0` success
  - `2` invalid YAML / unknown item name / invalid args
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
  - `2` unknown item name / invalid inventory YAML
  - `3` world backend unavailable (only when `--explain` needs world status)
  - `1` unexpected

#### `substrate world deps global add <item_name...> [--json]`
- Applies a **global enabled patch** update (does not install).
- It MUST:
  - Validate item names exist in the **global available inventory view** (built-ins + `$SUBSTRATE_HOME/deps/`; never workspace inventory).
  - Write only `$SUBSTRATE_HOME/config.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (global): added: <csv>`
  - `substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps global remove <item_name...> [--json]`
- Removes items from the global enabled patch (does not install).
- It MUST:
  - Not require names to exist in inventory (supports removing unknown names after manual edits).
  - Write only `$SUBSTRATE_HOME/config.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (global): removed: <csv>`
  - `substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply`
  - If a workspace is active for the current `cwd` and any removed item remains enabled via the workspace enabled list, it MUST also print:
    - `substrate: note: '<item>' was removed from global enabled deps but is still enabled via workspace; run 'substrate world deps workspace remove <item>' to fully disable it for this workspace`
- Exit codes: `0` success (including no-op); `2` invalid args / invalid YAML; `1` unexpected

#### `substrate world deps global reset [--json]`
- Resets the global enabled deps patch to inherited defaults by editing only `$SUBSTRATE_HOME/config.yaml`.
- It MUST remove the `world.deps.enabled` key from the global patch (inherit-only).
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Enabled deps reset (global)`
  - `substrate: note: run 'substrate world deps current sync' to apply enabled deps changes`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps workspace add <item_name...> [--json]`
- Applies a **workspace enabled patch** update (does not install).
- Requires `cwd` is within an enabled workspace (workspace root discovered from `cwd`).
- It MUST:
  - Validate item names exist in the **current available inventory view** for `cwd`.
  - Write only `<workspace_root>/.substrate/workspace.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (workspace): added: <csv>`
  - `substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'`
- Exit codes: `0` success (including no-op); `2` no workspace root / unknown item name / invalid YAML; `1` unexpected

#### `substrate world deps workspace remove <item_name...> [--json]`
- Removes items from the workspace enabled patch (does not install).
- It MUST:
  - Not require names to exist in inventory (supports removing unknown names after manual edits).
  - Write only `<workspace_root>/.substrate/workspace.yaml` (patch semantics; preserve comment header).
- On success, it MUST print:
  - `Enabled deps updated (workspace): removed: <csv>`
  - `substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply`
  - If any removed item remains enabled via the global enabled list, it MUST also print:
    - `substrate: note: '<item>' was removed from workspace enabled deps but is still enabled via global; run 'substrate world deps global remove <item>' to fully disable it`
- Exit codes: `0` success (including no-op); `2` no workspace root / invalid args / invalid YAML; `1` unexpected

#### `substrate world deps workspace reset [--json]`
- Resets workspace enabled deps back to inherited defaults by editing only `<workspace_root>/.substrate/workspace.yaml`.
- Requires `cwd` is within an enabled workspace.
- It MUST remove the `world.deps.enabled` key from the workspace patch (inherit-only).
- It MUST preserve any comment header in the patch file.
- On success, it MUST print:
  - `Enabled deps reset (workspace)`
  - `substrate: note: run 'substrate world deps current sync' to apply enabled deps changes`
- Exit codes: `0` success (including no-op); `2` actionable user error; `1` unexpected

#### `substrate world deps current install <item_name...> [--dry-run] [--verbose]`
- Applies items immediately without modifying the enabled list.
- It MUST:
  1) Expand bundles → packages.
  2) Apply **world image** installs first (apt).
  3) Apply **world deps prefix** installs second (scripts + entrypoints under `/var/lib/substrate/world-deps/bin`).
  4) Never execute `manual` installs; instead print `manual_instructions` and exit `4`.
- `--dry-run`:
  - MUST print the computed plan (apt list + script package list) and exit `0` without side effects.
- On success, it MUST print:
  - A short summary of what was applied (world image + world deps prefix), then:
  - `substrate: note: this updates the world only (enabled list not modified)`
  - `substrate: hint: run 'substrate world deps current list applied' to verify`
- Guarantee (runnable packages):
  - After success, runnable package entrypoints are invokable in-world via the standard world execution path (interactive `substrate>` and non-interactive runs) without requiring shell RC sourcing.
- Exit codes:
  - `0` success
  - `2` unknown item name / invalid YAML / invalid inventory
  - `3` world backend unavailable
  - `4` unmet prerequisites (e.g. manual install required, platform unsupported)
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
  - Prints the **global inventory patch view** from `$SUBSTRATE_HOME/deps/` (or empty if missing) as a table.
  - Table columns SHOULD include: `kind`, `name`, `runnable`, `method`, `entrypoints`, `platforms`, `description`.
  - It MUST NOT incorporate workspace inventory.
  - It MUST NOT print built-in defaults; use `deps current list available` for the merged view.
- `enabled`:
  - Prints the **global enabled deps patch** at `$SUBSTRATE_HOME/config.yaml` (or `{}` if missing).
- Exit codes: `0` success; `2` invalid YAML; `1` unexpected.

#### `substrate world deps workspace list [available|enabled] [--json]`
- Requires `cwd` is within an enabled workspace.
- `available` (default):
  - Prints the **workspace inventory patch view** from `<workspace_root>/.substrate/deps/` (or empty if missing) as a table.
  - Table columns SHOULD include: `kind`, `name`, `runnable`, `method`, `entrypoints`, `platforms`, `description`.
  - It MUST NOT incorporate global inventory.
  - It MUST NOT print built-in defaults; use `deps current list available` for the merged view.
- `enabled`:
  - Prints the **workspace enabled deps patch** at `<workspace_root>/.substrate/workspace.yaml` (patch view; not merged).
- Exit codes: `0` success; `2` no workspace root / invalid YAML; `1` unexpected.

### World Status Semantics
`present/missing/blocked` is always from the world perspective.
- For **runnable packages**, `present` means the package entrypoint is invokable in-world via the standard world shell execution path.
- For **bundles**, `present` means all constituent packages are `present` (bundles are never invoked directly).
- For packages with `install.method=manual`:
  - `present` means the package’s probe succeeds (via `probe.command`, or via `entrypoints[]` when runnable).
  - `blocked` means the package is not present and Substrate will not install it automatically; `remediation` SHOULD be: `manual install required; run 'substrate world deps current show <name> --explain'`.

### Notes / Known follow-ups
- Some packages are “managers” (e.g. `nvm`) and are not the same as their runtimes (e.g. `node`). A manager package being `present` does not imply a runtime exists unless a runtime package/bundle is enabled/installed.

## Architecture Shape
- CLI entrypoint (host): `crates/shell/src/builtins/world_deps/` becomes responsible for:
  - resolving “current directory” scope (workspace root discovery),
  - reading/writing YAML patch files (`$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`),
  - resolving inventory directories (built-ins + global + workspace chain),
  - enforcing merge rules, collision rules, and platform filters,
  - routing world-agent-backed operations for `applied`, `install`, `sync`.
- Shared models/parsing:
  - Replace/extend `crates/common/src/world_deps_manifest.rs` (currently manager-manifest-backed) with package/bundle inventory parsing and validation as specified in the contract.
- World execution (in-world):
  - `crates/world-agent/` owns in-world probes and installs (apt + script execution + `manual` blocked behavior).
- Legacy plumbing removal (host + installer):
  - `crates/shim/src/exec/logging.rs` and install scripts that read/copy `manager_hooks.yaml` / `world-deps.yaml` / selection files must be updated so `world deps` is not influenced by any legacy paths.

## Sequencing / Dependencies
- Alignment target: `docs/project_management/packs/sequencing.json` (this ADR introduces a contract that is not yet represented as a sprint entry).
- Hard dependencies (contract-level):
  - Patch-file scope semantics from `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md` (global/workspace patch files; preserve comment headers).
  - World backend availability semantics for commands that query/apply in-world state (exit `3` when backend is unavailable).
- Coexistence note (separate bodies of work):
  - `docs/project_management/_archived/world_deps_selection_layer/` defines a selection-file-driven contract that conflicts with this ADR’s “enabled patch” model and “legacy selection file paths removed” requirement.
  - This ADR is a separate/add-on body of work and does not update that planning-pack directory.
  - If/when we unify the world-deps operator contract, do it via a new planning pack and/or a new cross-cutting ADR under `docs/project_management/_archived/next/` (and update `docs/project_management/packs/sequencing.json`), without authoring changes inside existing planning-pack directories.

## Security / Safety Posture
- Fail-closed vs degrade:
  - World-backed operations (`applied`, `install`, `sync`) fail with exit `3` if the world backend is unavailable (no silent host fallback).
  - “Manual” installs are never executed automatically; they are surfaced as `blocked`/exit `4` with actionable instructions.
- Protected paths / invariants:
  - World-deps prefix installs are constrained to `/var/lib/substrate/world-deps` (scripts + entrypoints) as the allowed writable surface in hardened worlds.
  - If the install plan requires broader writes than allowed (hardening conflict), the operation fails with exit `5` (explicit, non-degrading).
- Observability:
  - World-agent-backed commands must surface `present/missing/blocked` consistently and provide a one-line remediation when applicable.

## Validation Plan (Authoritative)
- Unit tests (contract invariants):
  - Inventory parsing rejects schema violations (filename/name mismatch; missing required fields; invalid method shapes).
  - Inventory merge semantics are full-replace per item name; collision (same name in packages and bundles) is an error.
  - Enabled list validation enforces “must exist in effective inventory” and de-duplicates preserving order.
  - Platform filtering hides non-matching items (treated as non-existent).
- Integration tests (CLI contract):
  - `current list available|enabled` makes no world-agent calls.
  - World-backend-unavailable paths return exit `3` only where allowed by contract.
  - Replacement completeness: tests MUST fail if any legacy world-deps file path influences `world deps` behavior.
- Manual playbook and smoke:
  - If this ADR is executed as a feature sprint, it requires a feature-local `manual_testing_playbook.md` and platform smoke scripts validating `list/show/install/sync` on Linux/macOS/Windows.

## Rollout / Backwards Compatibility
- Greenfield breaking is allowed: the contract explicitly removes legacy file semantics and requires tests to enforce that the legacy paths no longer influence `world deps`.
- No backwards-compat layer is provided unless a future Accepted revision explicitly defines a compat policy and end condition.

## Decision Summary
- Decision register:
  - `docs/project_management/_archived/next/world-deps-packages-bundles-contract/decision_register.md`

## Appendix A — Host-visible worlds: preventing “host deps” from satisfying world-deps

This ADR’s core contract assumes that “runnable” package entrypoints are deterministically resolved in-world via the world-deps prefix (`/var/lib/substrate/world-deps/bin`) and that host toolchains do not “accidentally” become usable just because the workspace is visible.

In practice, `world_fs.host_visible=true` (see ADR-0018) makes the host filesystem nameable from within the world. If the in-world `PATH` contains host user toolchain directories (e.g. `$HOME/.config/nvm/.../bin`), then `which npm` (and normal command resolution) can find host-provided tools even when no world-deps are enabled/applied. This violates the intended “no host deps” property for host-visible worlds.

This appendix tightens the contract and enumerates the additional work required to make host-visible worlds behave like host-hidden worlds with respect to toolchain availability.

### A.1 Clarifications (what is and is not a host leak)

- `substrate world deps current list available` MAY show items even when `$SUBSTRATE_HOME/deps/` does not exist. Those “available” items can come from **built-in inventory defaults** shipped with Substrate (compiled-in / embedded), not from the host.
- To make provenance obvious, `substrate world deps current list available` MUST include a `source` column (and `--json` field) with one of: `builtin`, `global`, `workspace` indicating where the **effective** definition came from after inventory merge + platform filtering + `world.deps.inventory_mode`.
- The **shim** remains a host-side interception layer. It MUST NOT be assumed to be “the first PATH entry” inside the world:
  - World execution is performed by the world backend/world-agent, and the in-world environment must be built explicitly.
  - The hardening lever inside the world is **environment construction** (especially `PATH`) plus **world-deps wrappers** under `/var/lib/substrate/world-deps/bin`, not the host shim.

### A.2 Contract tightening: command resolution must not depend on host PATH

When running with `--world`, Substrate MUST ensure that:

1. `/var/lib/substrate/world-deps/bin` is the first `PATH` entry (already required by this ADR).
2. Host user toolchain PATH segments (e.g. anything under a host-mounted `$HOME`, `~/.nvm`, `~/.pyenv`, `~/.cargo`, `~/.local/bin`, etc.) MUST NOT be allowed to satisfy:
   - “runnable” entrypoint presence checks (`present/missing/blocked`), or
   - user invocations that rely on PATH lookup (e.g. `npm`, `node`, `bun`).
3. If a runnable dep is **not enabled/applied**, invoking its entrypoint in-world SHOULD fail with “command not found” even if a host-provided binary would otherwise be discoverable.

Stated differently: “host-visible” is a filesystem capability, not a toolchain capability. Toolchain resolution remains a world-deps-controlled surface.

### A.3 Required implementation work (to achieve the hardened behavior)

This is the additional work that must be implemented beyond the base packages/bundles contract to ensure no host toolchain leakage in host-visible worlds.

#### A.3.1 Normalize the in-world execution environment (especially `PATH`)

For all `--world` executions (PTY and non-PTY), Substrate should construct a deterministic in-world environment rather than inheriting the host environment:

- Build `PATH` from a known-safe baseline (platform/world-backend specific) plus the world-deps bin dir first, for example:
  - `/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
- Do not include host user toolchain segments by default.
- Ensure `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=/var/lib/substrate/world-deps/bin` is set in-world to make the chosen prefix explicit and debuggable.

Notes:
- This normalization is required even when `world_fs.host_visible=true`; visibility of the workspace does not imply inheriting the host process environment.
- Any “opt-in” to inheriting host PATH should be an explicit, audited toggle (policy/config), not a default.

#### A.3.2 Generate entrypoint wrappers for *all runnable packages*, including `apt` installs

This ADR already requires that runnable packages be invokable in-world. To make that property robust in host-visible worlds, the resolution must be anchored in the world-deps prefix:

- For each runnable package, for each `entrypoints[]` name, Substrate MUST ensure there is an executable at:
  - `/var/lib/substrate/world-deps/bin/<entrypoint>`
- For `install.method=script`, the script installer already owns wrapper/entrypoint creation.
- For `install.method=apt`, additional work is required:
  - After `apt` install completes, generate a wrapper (or symlink, where safe) under `/var/lib/substrate/world-deps/bin/` for each entrypoint.
  - The wrapper MUST exec the world-provided binary (typically under `/usr/bin/...`) resolved using the normalized PATH (not the inherited host PATH).

This makes `which npm` deterministic: if `npm` is enabled/applied, it resolves to `/var/lib/substrate/world-deps/bin/npm`; if not, it does not resolve via host PATH.

#### A.3.3 Redefine “present” probes to be host-path-independent

For `present/missing/blocked` to be meaningful in host-visible worlds:

- For runnable packages, default “present” SHOULD be determined by:
  - wrapper existence + executability under `/var/lib/substrate/world-deps/bin`, or
  - an explicit `probe.command` executed with the normalized environment (sanitized PATH).
- The presence logic MUST NOT accept “it is somewhere on PATH” if that PATH can include host user toolchains.

#### A.3.4 Optional but recommended: execution-time guardrails against explicit host binary execution

The measures above prevent *accidental* host toolchain usage via PATH lookup. They do not prevent an actor from explicitly executing a host-visible binary path (e.g. `/home/<user>/.config/nvm/.../npm`).

If the desired security posture is “no host deps may execute inside the world”, then add explicit guardrails:

- World-agent (or the world backend) SHOULD reject process execution when the resolved executable path is under host-mounted locations (e.g. host `$HOME`) unless explicitly allowed by policy.
- Provide an actionable error indicating that host toolchains are disallowed and that the user should enable/install a world-deps package instead.

This should be treated as a policy-controlled hardening feature to avoid surprising advanced workflows.

### A.4 Installer scaffolding: create `$SUBSTRATE_HOME/deps/` with examples

To reduce confusion and make customization discoverable, the installer (or first-run init) SHOULD scaffold the global deps inventory directory under `$SUBSTRATE_HOME` even if the user has not created any custom deps yet.

Recommended global layout:
```text
~/.substrate/
  deps/
    README.md
    packages/
      example-manual.yaml
      example-script.yaml
      example-apt.yaml
    bundles/
      example-bundle.yaml
    scripts/
      example-install.sh
```

Example `packages/example-apt.yaml` (shape only; names illustrative):
```yaml
version: 1
name: example-apt
description: Example apt package that provides a runnable CLI.
runnable: true
entrypoints: ["example"]
install:
  method: apt
  apt:
    - name: example
```

Example `packages/example-script.yaml`:
```yaml
version: 1
name: example-script
description: Example script-installed package.
runnable: true
entrypoints: ["example-script"]
install:
  method: script
  script_path: deps/scripts/example-install.sh
```

Example `bundles/example-bundle.yaml`:
```yaml
version: 1
name: example-bundle
description: Example bundle of packages.
packages: ["example-apt", "example-script"]
```

Scaffolding notes:
- These examples MUST NOT auto-enable anything; they are purely illustrative.
- The scaffold should clearly explain precedence/merge behavior (built-ins + global inventory + workspace inventory) and point users to `substrate world deps current ...` to see the effective view.
- If a legacy world-deps file is still installed for backwards reasons (e.g. `world-deps.yaml`), the scaffold should explicitly note that this contract ignores it.

### A.5 Acceptance criteria for host-visible hardening

The following scenarios should be added to the authoritative validation plan once the above work is implemented:

- With `world_fs.host_visible=true` and with no deps enabled/applied, `substrate --world -c 'which npm'` MUST fail (no host PATH satisfaction).
- After enabling and syncing `node`/`npm`, `substrate --world -c 'which npm'` MUST return `/var/lib/substrate/world-deps/bin/npm` and invoking `npm` must execute the world-provided toolchain.
- For any runnable package, `present` MUST reflect the wrapper/probe outcome under the normalized environment, independent of host PATH.
