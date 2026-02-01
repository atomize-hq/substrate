# WFGAD3 — spec — deny masking (allow/deny semantics + wildcard snapshot)

## Scope (explicit)
- Implement deny-overrides-allow semantics via mount masking inside the per-command mount namespace, before user code runs.
- Implement wildcard deny snapshot scanning semantics and ensure it does not follow symlinks.
- Enforce deterministic errno behavior for denied operations.

## Acceptance (explicit)
- Implements requirements: R-006, R-008, R-010, R-020, R-021.
- Validation:
  - integration validation covers the manual playbook cases that exercise these requirements.

## Out of scope (explicit)
- Any strict-mode lockdown implementation.
- Any discover/read split implementation.
