# WFGAD0 — spec — broker policy schema v2 (breaking)

## Scope (explicit)
- Implement the breaking policy schema v2 validation rules in `crates/broker` per the authoritative spec pack.
- This slice is Linux full isolation only (`world_fs.isolation=full`).
- This slice enforces hard errors for:
  - legacy keys (no compatibility), and
  - invalid path patterns (no silent ignore).

## Acceptance (explicit)
- Implements requirements: R-001, R-002, R-004, R-005, R-018, R-019, R-016.
- Validation:
  - unit tests cover all acceptance requirements for this slice.

## Out of scope (explicit)
- Any world-agent changes.
- Any shell changes.
- Any deny masking or strict lockdown implementation.
