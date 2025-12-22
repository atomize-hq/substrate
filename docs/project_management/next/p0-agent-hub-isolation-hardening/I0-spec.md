# I0-spec: Strict Policy Schema (world_fs v1)

## Scope
- Introduce a strict, versioned policy schema for filesystem + caging under a single `world_fs` block:
  - `world_fs.mode`: `writable | read_only`
  - `world_fs.cage`: `project | full`
  - `world_fs.read_allowlist`: glob/pattern list (required; may default to `["**"]` if explicitly omitted)
  - `world_fs.write_allowlist`: glob/pattern list (required; may default to `[]` if explicitly omitted)
- Remove reliance on legacy keys in docs and implementation (breaking change acceptable; greenfield).
  - Existing keys like `world_fs_mode`, `fs_read`, `fs_write` are either:
    - rejected with clear error messages, or
    - accepted only if the spec explicitly says so (default: reject).
- Add strong validation with actionable diagnostics:
  - Missing `world_fs` → error explaining required fields and providing an example.
  - Invalid values → error listing allowed values.
  - Inconsistent fields (e.g., `cage=full` but empty allowlists) → error or explicit warning per spec.
- Broker output must include enough information for the shell/world backend to enforce policy decisions:
  - “world required” vs “may fall back to host”
  - desired `world_fs` parameters

## Acceptance
- A minimal policy/profile containing `world_fs` parses and applies.
- Policies missing `world_fs` fail with a clear diagnostic (no silent fallback to permissive behavior).
- Invalid `world_fs.mode` or `world_fs.cage` fails with a clear diagnostic.
- Docs/examples in this track use `world_fs` only.

## Out of Scope
- Implementing full caging mechanics (pivot_root/Landlock) — handled in I2/I4.
- Changing runtime config file formats (YAML vs TOML) — separate decision track.

