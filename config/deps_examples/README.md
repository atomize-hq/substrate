# World Deps `deps/` Examples

These files are examples of the **per-item inventory** format described in:
- `docs/reference/world/deps/README.md`
- `docs/adr/implemented/ADR-0011-world-deps-packages-bundles-contract.md`

They are intended to be copied into a scope’s inventory directory:
- Global: `~/.substrate/deps/`
- Workspace: `<workspace_root>/.substrate/deps/`

Layout mirrors the canonical on-disk format:
- `packages/<dep_name>.yaml`
- `bundles/<dep_name>.yaml`
- `scripts/*.sh` (referenced via `script_path`)
