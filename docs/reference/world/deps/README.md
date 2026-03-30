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

## System-package runtime fail-early

If a package uses `install.method=apt` or `install.method=pacman`, runtime
`substrate world deps current sync` and `substrate world deps current install ...` never invoke
`apt`, `apt-get`, mutating `dpkg`, or `pacman`. Instead, Substrate derives the normalized APT and
pacman requirement sets, probes them read-only with `dpkg-query` and `pacman -Q`, and fails early
with remediation when any requirement is missing.

Operator workflow:
- Run `substrate world enable --provision-deps` before runtime `world deps current ...` commands.
- On Linux host-native, Substrate will not mutate the host OS at runtime.
- On Windows, `substrate world enable --provision-deps` is unsupported on Windows for runtime system-package provisioning.
- Runtime `current sync` and `current install` exit `4` when required system packages are missing.
- Missing-package remediation includes `substrate world enable --provision-deps`.

### Provisioning contract

`substrate world enable --provision-deps` is the only operator-facing Substrate workflow that performs
APT package mutation for world deps, and only on supported guest backends.

Behavior:
- Supported on guest-backed worlds such as macOS Lima.
- Unsupported on Linux host-native; Substrate must not mutate the host OS.
- Unsupported on Windows.
- Uses the effective enabled world-deps set for the current directory.
- `--dry-run` prints the derived APT requirement set without mutating the world.
- `--verbose` additionally shows the provisioning request posture used for the world-agent call.

### Runtime contract

For APT-backed and pacman-backed items, `substrate world deps current sync` and
`substrate world deps current install` remain probe-only at runtime.

Behavior:
- They derive the in-scope APT and pacman requirement sets and probe them read-only with
  `dpkg-query` and `pacman -Q`.
- If any required system package is missing, they fail early with exit `4`.
- If all required system packages are already present, system-package items are treated as
  satisfied/no-op and Substrate continues with non-system-package work.
- Runtime execution never invokes `apt`, `apt-get`, mutating `dpkg`, or `pacman`.

Internal details and rationale:
- `docs/internals/world/deps.md`
- WDAP1 provisioning/remediation contract:
  `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
