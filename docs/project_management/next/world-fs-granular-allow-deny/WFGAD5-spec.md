# WFGAD5 — spec — strict deny lockdown (bypass prevention)

## Scope (explicit)
- Implement strict-mode lockdown so deny masking is a security boundary:
  - the workload cannot undo deny masks via mount/umount/remount,
  - syscall blocks return deterministic `EPERM` without killing the process.

## Acceptance (explicit)
- Implements requirements: R-009, R-014, R-015, R-021.
- Validation:
  - integration validation covers manual playbook Case 2.

## Out of scope (explicit)
- Any cross-platform work.
