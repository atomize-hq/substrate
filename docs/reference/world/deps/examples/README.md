# World-Deps Inventory Examples

This directory contains **copy/pasteable** examples you can use as a starting point for your own inventory under:
- Global: `$SUBSTRATE_HOME/deps/` (typically `~/.substrate/deps/`)
- Workspace: `<workspace_root>/.substrate/deps/`

## How to use

1) Copy the example package/bundle definitions into your inventory directory:

```text
~/.substrate/deps/
  packages/
  bundles/
  scripts/
```

2) Enable a package/bundle:
- Global: `substrate world deps global add <name...>`
- Workspace: `substrate world deps workspace add <name...>`

3) Apply to the world:
- `substrate world deps current sync`

4) Verify:
- `substrate world deps current list applied`
- `substrate --world -c '<entrypoint> --version'`

## Examples included

- `packages/nvm.yaml` + `scripts/nvm.sh`
  - Installs `nvm` into `/var/lib/substrate/world-deps/nvm` (not `$HOME`).
  - Provides a runnable `nvm` entrypoint via a Substrate-generated wrapper in `/var/lib/substrate/world-deps/bin/nvm`.

- `packages/hello.yaml` + `scripts/hello.sh`
  - Minimal example showing “install into prefix + create entrypoint”.

## Notes on hardening

In hardened worlds, treat the world filesystem as effectively read-only outside:
- `/var/lib/substrate/world-deps`
- `/tmp`

Avoid `$HOME` and system paths in your scripts.

