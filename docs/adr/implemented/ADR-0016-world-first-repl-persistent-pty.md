# ADR-0016 — World-First REPL With Persistent World PTY

## Status

- Status: Implemented
- Original date (UTC): 2026-01-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Substrate maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

When world execution is enabled, the interactive REPL must behave like a world-first shell backed
by a persistent world PTY session rather than mixing world-backed command execution with host-only
builtin semantics.

The stable decision is:

- unprefixed interactive REPL commands run against a persistent in-world session
- `cd`, `pwd`, `export`, and `unset` follow in-world shell semantics when world mode is active
- `:host` remains an explicit gated escape hatch rather than an implicit fallback
- `-c/--command` must stay world-consistent when world mode is enabled
- PTY output handling and completion ordering must preserve traceability and operator clarity

## Stable Owned Surface

This ADR owns the stable world-first REPL behavior surfaced through:

- interactive `substrate` REPL behavior when world execution is enabled
- `:host` and `:pty` interactive escape/forcing behavior
- world-consistent `-c/--command` semantics

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/invocation/runtime.rs`
- `crates/shell/src/execution/routing/dispatch/exec.rs`
- `crates/shell/src/execution/routing/builtin/utility.rs`
- `crates/shell/src/execution/routing/builtin/world_deps.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/draft/ADR-0020-profiles-config-policy-snapshots.md`

## Historical Note

The original ADR captures detailed protocol, state-machine, and rollout context. The stable
world-first REPL contract now lives here.
