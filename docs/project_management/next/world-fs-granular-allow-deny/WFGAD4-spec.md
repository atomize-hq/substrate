# WFGAD4 — spec — discover vs read dimension (directory visibility)

## Scope (explicit)
- Implement the optional `discover` dimension semantics:
  - when omitted it mirrors `read`.
  - when configured it can produce “visible but not readable” behavior per the manual playbook.
- Implement the required Landlock behavior changes to support discover/read split as specified.

## Acceptance (explicit)
- Implements requirements: R-007.
- Validation:
  - integration validation covers manual playbook Case 3.

## Out of scope (explicit)
- Any strict-mode lockdown implementation.
