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
- Provision APT-backed system packages: `substrate world enable --provision-deps`
- Apply enabled set: `substrate world deps current sync`
- Debug an item: `substrate world deps current show <name> --explain`

## APT packages (current limitation in hardened worlds)

If a package uses `install.method=apt`, runtime `substrate world deps current sync` and
`substrate world deps current install ...` never invoke `apt`, `apt-get`, or mutating `dpkg`.
Instead, Substrate derives the normalized APT requirement set, probes it read-only with
`dpkg-query`, and fails early with remediation when any requirement is missing.

Operator workflow:
- Run `substrate world enable --provision-deps` before runtime `world deps current ...` commands.
- On Linux host-native, Substrate will not mutate the host OS at runtime.
- On Windows, provisioning-time APT is unsupported on Windows.

Contract source:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`

Internal details and rationale:
- `docs/internals/world/deps.md`
