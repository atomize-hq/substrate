# Spec Index: `macos-hardened-same-user-lima`

Status: current slice authority index
Last updated: 2026-06-22

## Purpose

Index the committed slice stack for `macos-hardened-same-user-lima` and make
the current authority order explicit.

The `spec/` scaffold is no longer hypothetical. `SPEC-01` through `SPEC-12`
already exist in this directory.

## Current authority order

For planning or closeout questions, use this order:

1. [`../README.md`](../README.md) for feature-level current truth
2. relevant `spec/design/DESIGN-*` docs for cross-slice contracts
3. the relevant committed `SPEC-*`
4. its paired `PLAN-*`
5. its paired `TASKS-*`
6. top-level operator docs when the question is about live user-facing posture

`../ROADMAP.md` is now historical / retrospective framing, not the live
instruction to create the scaffold.

## Directory contents

### Design docs

Cross-slice contract sources:

- `design/DESIGN-supported-mode-and-breakglass-taxonomy.md`
- `design/DESIGN-macos-lima-transport-contract.md`
- `design/DESIGN-macos-policy-input-parity.md`
- `design/DESIGN-macos-ingress-and-mount-contract.md`
- `design/DESIGN-macos-guest-unit-source-of-truth.md`
- `design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`

### Slice docs

Committed local slices:

1. `SPEC-01` / `PLAN-01` / `TASKS-01`
2. `SPEC-02` / `PLAN-02` / `TASKS-02`
3. `SPEC-03` / `PLAN-03` / `TASKS-03`
4. `SPEC-04` / `PLAN-04` / `TASKS-04`
5. `SPEC-05` / `PLAN-05` / `TASKS-05`
6. `SPEC-06` / `PLAN-06` / `TASKS-06`
7. `SPEC-07` / `PLAN-07` / `TASKS-07`
8. `SPEC-08` / `PLAN-08` / `TASKS-08`
9. `SPEC-09` / `PLAN-09` / `TASKS-09`
10. `SPEC-10` / `PLAN-10` / `TASKS-10`
11. `SPEC-11` / `PLAN-11` / `TASKS-11`
12. `SPEC-12` / `PLAN-12` / `TASKS-12`

## How to interpret slice truth

- The slice stack captures the intended dependency order and acceptance gates.
- If a slice doc overstates current implementation reality, live repo truth and
  updated closeout wording should win.
- Packet/prompt artifacts under `PROMPTS-*` are useful delegation helpers, but
  they are not the primary authority over the matching `SPEC/PLAN/TASKS`.

## Related docs

- [Feature overview](../README.md)
- [Execution Rubric](../EXECUTION-RUBRIC.md)
- [Roadmap](../ROADMAP.md)
