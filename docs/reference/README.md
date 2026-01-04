# Reference Docs

This directory is scaffolding: it is intended to hold stable, operator-facing contracts (supported interfaces) for Substrate.

What belongs here:
- User-visible CLI contract (commands, flags, exit codes, error shapes)
- Config and policy contract (file locations, precedence, schema expectations)
- Supported environment variable contract (inputs only; clear stability)
- Path/layout contract (where Substrate reads/writes files)

What does not belong here:
- Exhaustive inventories of internal/test knobs
- Implementation details tied to specific crates/modules (put those in `docs/internals/`)

