# WFGAD2 — spec — host snapshot emission + enforcement plan env contract

## Scope (explicit)
- Emit PolicySnapshotV2 from the host (shell) and ingest PolicySnapshotV2 in world-agent request handlers.
- Build and validate the helper-side enforcement plan env input schema (base64 JSON) exactly as defined in `ENV.md`.
- Ensure helper invocation is mandatory whenever helper-side enforcement is required.

## Acceptance (explicit)
- Implements requirements: R-013, R-022.
- Validation:
  - tests cover enforcement plan env parsing/validation failures (fail closed) and the helper invocation gating behavior.

## Out of scope (explicit)
- Any deny masking implementation.
- Any strict-mode lockdown implementation.
