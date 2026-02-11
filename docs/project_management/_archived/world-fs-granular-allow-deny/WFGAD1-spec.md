# WFGAD1 — spec — PolicySnapshotV2 + request/response model v2

## Scope (explicit)
- Introduce PolicySnapshotV2 in request/response models and enforce world-agent rejection of PolicySnapshotV1.
- This slice is Linux full isolation only (`world_fs.isolation=full`).
- This slice is breaking (no backwards compatibility):
  - `PolicySnapshotV1` is rejected.
  - unknown fields and wrong schema_version are rejected.

## Acceptance (explicit)
- Implements requirements: R-003, R-011, R-012, R-017.
- Validation:
  - tests cover HTTP `/v1/execute` and WS `start_session` rejection behavior for invalid snapshots exactly as specified.

## Out of scope (explicit)
- Any helper-side deny masking implementation.
- Any strict-mode lockdown implementation.
