# I3-spec: Full Cage (PTY) Parity

## Scope
- Extend full caging (I2) to PTY execution paths:
  - PTY child spawn must enter the same full cage as non-PTY, before executing the user command.
  - Ensure we avoid inode/cwd escapes (do not start the child in the project directory before the cage is active).
- Keep behavior consistent across:
  - REPL PTY runs
  - world-agent `/v1/stream` path
- Ensure signal forwarding and resizing continue to work.

## Acceptance
- PTY commands respect full cage constraints (cannot touch host paths outside the cage).
- `world_fs.mode=read_only` prevents project writes from PTY commands.
- No regressions to PTY streaming and signal forwarding.

## Out of Scope
- Landlock allowlist enforcement — I4.

