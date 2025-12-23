# Y0-spec: Migrate Settings/Config Stack from TOML to YAML

## Scope
- Replace TOML-based settings stack with YAML equivalents:
  - `~/.substrate/config.yaml` (new) replacing `~/.substrate/config.toml`
  - `.substrate/settings.yaml` (new) replacing `.substrate/settings.toml`
- Update config command surface to read/write YAML:
  - `substrate config init/show/set`
- Update settings loader/builder and any callsites that assume TOML paths/names.
- Update docs and tests that reference TOML paths.

Migration policy (greenfield, no dual-format support):
- No dual-format parsing long-term.
- Provide a clear failure message if TOML files exist but YAML is required (and a suggested one-liner migration path).

## Acceptance
- All existing commands that read settings work with YAML equivalents.
- `substrate config init` produces YAML and no longer writes TOML.
- CI passes on Linux/macOS/Windows (fmt/clippy/tests).
- Docs references updated (CONFIGURATION/USAGE as needed).

## Out of Scope
- Converting YAML manifests/policies to TOML (we are standardizing on YAML).
- Adding “support both formats forever”.

