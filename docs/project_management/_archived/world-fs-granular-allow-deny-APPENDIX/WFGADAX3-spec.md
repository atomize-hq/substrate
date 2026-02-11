# WFGADAX3 — REPL exit transparency + `repl.exit_cwd` (Spec)

## Goal
- Implement Appendix B REPL exit note and `repl.exit_cwd` behavior.

## Acceptance criteria
- On exit, when `world_cwd != entered_cwd`, Substrate prints:
  - `substrate: note: returning to host cwd: <path>`
- `repl.exit_cwd` selects the exit target and rejects unsupported modes as hard errors (exit `2`).

