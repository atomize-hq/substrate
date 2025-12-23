# Y0-spec: Migrate Settings/Config Stack from TOML to YAML

## Scope
- Replace TOML-based settings stack with YAML equivalents:
  - `~/.substrate/config.yaml` (new) replacing `~/.substrate/config.toml`
  - `.substrate/settings.yaml` (new) replacing `.substrate/settings.toml`
- Update config command surface to read/write YAML:
  - `substrate config init/show/set`
- Update settings loader/builder and any callsites that assume TOML paths/names.
- Update docs and tests that reference TOML paths.

### YAML structure (required)

Keep the same logical structure as today’s TOML (minimal behavior change; only the serialization format changes).

Global config (`~/.substrate/config.yaml`) should be equivalent to the current `default_config_tables()`:
```yaml
install:
  world_enabled: true
world:
  anchor_mode: project   # project | follow-cwd | custom
  anchor_path: ""        # required when anchor_mode=custom
  root_mode: project     # legacy alias (may be accepted short-term; see below)
  root_path: ""          # legacy alias (may be accepted short-term; see below)
  caged: true
```

Per-directory settings (`.substrate/settings.yaml`) are the same schema but typically only override `world.*`:
```yaml
world:
  anchor_mode: follow-cwd
  caged: true
```

### Precedence (must remain unchanged)

`CLI > directory settings > global config > env vars > defaults`

This is the current `resolve_world_root()` behavior in `crates/shell/src/execution/settings/builder.rs` and must
remain consistent after the migration.

Migration policy (greenfield, no dual-format support):
- No dual-format parsing long-term.
- Provide a clear failure message if TOML files exist but YAML is required (and a suggested migration path).

Explicit requirements:
- If `config.toml` or `settings.toml` exist, treat them as **unsupported** and fail with an actionable message.
- Message must include at least:
  - the unsupported file path(s),
  - the new YAML path(s),
  - a suggested next step (e.g., delete old file(s) and run `substrate config init --force`, then reapply config via `substrate config set ...`).

Note: if we decide to support a one-time conversion command later, it must live behind an explicit command
(`substrate config migrate`) and not be “silent dual-format support”.

## Acceptance
- All existing commands that read settings work with YAML equivalents.
- `substrate config init` produces YAML and no longer writes TOML.
- CI passes on Linux/macOS/Windows (fmt/clippy/tests).
- Docs references updated (CONFIGURATION/USAGE as needed).

Concrete checks:
- `substrate config init` writes `~/.substrate/config.yaml` and refuses to write `config.toml`.
- `substrate config show` prints YAML (and `--json` still works).
- `substrate config set world.anchor_mode=follow-cwd` updates YAML and persists.
- `resolve_world_root()` uses `.substrate/settings.yaml` when present and ignores TOML.

## Out of Scope
- Converting YAML manifests/policies to TOML (we are standardizing on YAML).
- Adding “support both formats forever”.
