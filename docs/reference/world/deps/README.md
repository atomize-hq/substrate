# World Deps (Operator Reference)

`substrate world deps` is Substrate’s operator-facing mechanism for making toolchains available **inside the world** in a
deterministic, policy-compatible way.

The contract is:
- You define **packages** and **bundles** in an inventory (YAML).
- You enable packages/bundles via config patches (global/workspace).
- You apply the effective enabled set to the world via `substrate world deps current sync`.

## Mental model

### Inventory vs enabled vs applied
- **Inventory**: definitions of packages/bundles (what exists and how it is installed).
- **Enabled**: which names should be present for a given scope (global/workspace).
- **Applied**: world-backed status (`present|missing|blocked`) for enabled or queried items.

### Where inventory lives
Inventory is discovered from multiple layers:
- Built-ins (shipped with Substrate)
- Global inventory: `$SUBSTRATE_HOME/deps/`
- Workspace inventory: `<workspace_root>/.substrate/deps/` (and a workspace chain, if applicable)

### Where enabled lists live
Enabled lists are patch keys (not separate “selection files”):
- Global: `$SUBSTRATE_HOME/config.yaml` (`world.deps.enabled`)
- Workspace: `<workspace_root>/.substrate/workspace.yaml` (`world.deps.enabled`)

## Hardening constraints (most important)

In hardened worlds (notably macOS Lima and Windows WSL guests), Substrate intentionally restricts writes during world
execution.

You should assume:
- The world filesystem is effectively **read-only** outside Substrate-managed writable surfaces.
- **Do not** install to `$HOME` / dotfiles / system paths.

### Writable surfaces you can rely on
At runtime (during `current sync` / `current install`):
- `/var/lib/substrate/world-deps` (Substrate-managed prefix; stable)
- `/tmp` (scratch)

### Common non-writable paths
Avoid writing to:
- `$HOME` (in-world default is typically `/root`, and it may be read-only under hardening)
- `/usr`, `/etc`, `/var/lib/dpkg`, `/var/log`, `/var/cache`

If a third-party installer “insists” on writing to `$HOME`, set its tool-specific home/dir variables to live under
`/var/lib/substrate/world-deps` (example: `NVM_DIR=/var/lib/substrate/world-deps/nvm`).

## Authoring inventory items

- Package authoring guide: `docs/reference/world/deps/authoring_packages.md`
- Bundle authoring guide: `docs/reference/world/deps/authoring_bundles.md`
- Copy/pasteable inventory examples: `docs/reference/world/deps/examples/README.md`

## Commands you will use

- See inventory: `substrate world deps current list available`
- See enabled: `substrate world deps current list enabled`
- Enable globally: `substrate world deps global add <name...>`
- Enable for a workspace: `substrate world deps workspace add <name...>`
- Apply enabled set: `substrate world deps current sync`
- Debug an item: `substrate world deps current show <name> --explain`

## APT packages (current limitation in hardened worlds)

If a package uses `install.method=apt`, installation may fail in hardened worlds because `apt/dpkg` writes outside
`/var/lib/substrate/world-deps`.

Preferred approaches:
- Model the tool as a **user-space script install** under `/var/lib/substrate/world-deps`, or
- Use an explicit **provisioning** workflow that runs outside the hardened runtime sandbox (when available).

Internal details and rationale: `docs/internals/world/deps.md`.

