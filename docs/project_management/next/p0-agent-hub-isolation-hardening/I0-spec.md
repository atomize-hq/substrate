# I0-spec: Strict Policy Schema (world_fs v1)

## Scope
- Introduce a strict, versioned policy schema for filesystem + caging under a single `world_fs` block:
  - `world_fs.mode`: `writable | read_only`
  - `world_fs.isolation`: `workspace | full`
  - `world_fs.require_world`: `true | false`
  - `world_fs.read_allowlist`: glob/pattern list (required; must be non-empty)
  - `world_fs.write_allowlist`: glob/pattern list (required; can be empty)
- Remove reliance on legacy keys in docs and implementation (breaking change acceptable; greenfield).
  - Existing keys like `world_fs_mode`, `fs_read`, `fs_write` are either:
    - rejected with clear error messages.
- Add strong validation with actionable diagnostics:
  - Missing `world_fs` → error explaining required fields and providing an example.
  - Invalid values → error listing allowed values.
    - Inconsistent fields → error:
    - `world_fs.mode=read_only` requires `world_fs.require_world=true`
    - `world_fs.isolation=full` requires `world_fs.require_world=true`
- Broker output must include enough information for the shell/world backend to enforce policy decisions:
  - `world_fs.require_world` (“world required” vs “host fallback allowed”)
  - desired `world_fs` parameters

## Acceptance
- A minimal policy/profile containing `world_fs` parses and applies.
- Policies missing `world_fs` fail with a clear diagnostic (no silent fallback to permissive behavior).
- Invalid `world_fs.mode` or `world_fs.isolation` fails with a clear diagnostic.
- Docs/examples in this track use `world_fs` only.

## Out of Scope
- Implementing full isolation mechanics (pivot_root/Landlock) — handled in I2/I4.
- Changing runtime config file formats — handled in Y0.
