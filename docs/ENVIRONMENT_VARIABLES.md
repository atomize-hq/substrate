# Environment Variables (Entry Point)

This file is scaffolding: it is the stable entry point for environment variable documentation as the docs tree is reorganized.

## Where to look

- Supported operator contract: `docs/reference/env/contract.md`
- Exhaustive developer inventory (includes internal/test + standard env vars): `docs/internals/env/inventory.md`

## Override inputs vs exported state

- `SUBSTRATE_OVERRIDE_*` are the supported override-input environment variables (read during effective config resolution).
- `SUBSTRATE_*` are exported state variables written by Substrate and are not consulted as override inputs.
