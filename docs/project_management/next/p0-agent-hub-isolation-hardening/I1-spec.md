# I1-spec: Fail-Closed Semantics (No Host Fallback When Required)

## Scope
- Define and implement “required isolation” semantics:
  - When policy requires world enforcement for a command/session, Substrate must **refuse** to execute on the host.
  - Required isolation must be visible in both:
    - human output (single clear error),
    - JSON output (structured error).
- Determine how “required” is derived (must be explicit in code and docs):
  - Either via an explicit policy knob (`require_world: true`), or
  - Derived from `world_fs.cage=full` and/or `world_fs.mode=read_only` (recommended to be explicit).
- Ensure enforcement covers:
  - Non-PTY command routing (`substrate -c`, non-interactive execution)
  - PTY/REPL command routing
- Ensure warning/diagnostic behavior is not noisy:
  - If world is unavailable and world is required → error and stop.
  - If world is unavailable and world is optional → single warning and host fallback (existing behavior).

## Acceptance
- With `require_world` (or equivalent), `substrate -c 'true'` fails if the world backend is unavailable.
- Without `require_world`, the current “warn once then host fallback” behavior remains.
- The REPL respects “required world” (cannot silently proceed on host).

## Out of Scope
- Full caging mechanics (pivot_root) — I2/I3.
- World-deps selection UX — separate track.

