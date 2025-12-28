# I8-spec: I1 Noise Reduction (Single Warning / Single Error)

## Scope
- Bring the implementation in line with `I1-spec.md` “not noisy” requirements:
  - If world is unavailable and `world_fs.require_world=false`:
    - Print **exactly one** warning about world backend unavailability (per process/session), then run on host.
  - If world is unavailable and world is required (`world_fs.require_world=true`):
    - Print **exactly one** clear error and stop (no host fallback and no extra “fallback failed” noise).
- Applies to both:
  - `substrate -c ...` (wrap mode)
  - Interactive/REPL paths (per-command execution)

### Constraints / notes
- This triad is about message emission and control flow; it must not weaken enforcement:
  - No host fallback when required.
  - No silent degrade when full cage/read_only require a world backend.
- Keep existing actionable hints (e.g., “run `substrate world doctor --json`”, systemd socket checks),
  but avoid duplicating them across multiple lines.

## Acceptance
- When world is unavailable and fallback is allowed:
  - Exactly one warning is emitted, and the command runs on host.
- When world is unavailable and world is required:
  - Exactly one error is emitted, and the command does not run.
- Tests cover both cases (fixture-based where possible) and assert on warning/error line counts.

## Out of Scope
- Changing exit code mappings (handle in a dedicated exit-code triad if needed).
- Any changes to isolation semantics beyond message/noise behavior.
