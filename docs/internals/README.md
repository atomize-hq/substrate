# Internal Docs

This directory is scaffolding: it is intended to hold developer-facing implementation notes, inventories, and deep dives.

What belongs here:
- Exhaustive env var inventories and generation notes
- Trace/log schema details and internal invariants
- Shim/world/broker internals and debugging workflows
- REPL execution/routing internals and invariants
- Test-only toggles and harness documentation

What does not belong here:
- Operator-facing “supported interface” promises (put those in `docs/reference/`)

## Sections

- `docs/internals/broker/`
- `docs/internals/env/`
- `docs/internals/repl/`
- `docs/internals/replay/`
- `docs/internals/shim/`
- `docs/internals/telemetry/`
- `docs/internals/testing/`
- `docs/internals/trace/`
- `docs/internals/world/`
