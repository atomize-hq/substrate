# WFGADAX2 — Policy-level caging requirement (Spec)

## Goal
- Implement Appendix B caging requirement as a policy boundary.

## Acceptance criteria
- When `world_fs.caged_required=true` is effective for entered cwd scope:
  - `world.caged=false` hard errors (exit `2`).
  - `world.anchor_mode=follow-cwd` hard errors (exit `2`).
  - REPL escape attempts are blocked with a human-readable note.

