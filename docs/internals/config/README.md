# Config Internals

Developer-facing notes for how Substrate resolves configuration and turns it into runtime behavior.

This section complements the operator-facing config contract in `docs/reference/config/`.

## Documents

- `docs/internals/config/world_root_and_caging.md`: how `world.anchor_mode`, `world.anchor_path`, and `world.caged` flow
  through the shell → world-agent → world backend (including persistent-session behavior).

