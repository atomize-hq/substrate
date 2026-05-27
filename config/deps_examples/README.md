# World Deps `deps/` Examples

These files are examples of the **per-item inventory** format described in:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`

They are intended to be copied into a scope’s inventory directory:
- Global: `~/.substrate/deps/`
- Workspace: `<workspace_root>/.substrate/deps/`

Layout mirrors the canonical on-disk format:
- `packages/<dep_name>.yaml`
- `bundles/<dep_name>.yaml`
- `scripts/*.sh` (referenced via `script_path`)

