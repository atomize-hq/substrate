# Spec Scaffold: `macos-hardened-same-user-lima`

Status: draft scaffold  
Last updated: 2026-06-11

## Purpose

Provide the execution-layer document structure for landing
`macos-hardened-same-user-lima` in repo-native spec-driven form.

The existing feature and phase docs remain the program-level authority. This
directory is where the bounded execution artifacts should live:

- execution guidance in [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md),
- shared `DESIGN-*` inputs,
- numbered `SPEC-*` slices,
- companion `PLAN-*` and `TASKS-*` docs.

## Directory layout

```text
spec/
├── README.md
└── design/
    ├── README.md
    ├── DESIGN-supported-mode-and-breakglass-taxonomy.md
    ├── DESIGN-macos-lima-transport-contract.md
    ├── DESIGN-macos-policy-input-parity.md
    ├── DESIGN-macos-ingress-and-mount-contract.md
    ├── DESIGN-macos-guest-unit-source-of-truth.md
    └── DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md
```

Future slice docs should also live here:

```text
SPEC-01-...
PLAN-01.md
TASKS-01.md
SPEC-02-...
PLAN-02.md
TASKS-02.md
...
```

## Roles

### `design/`

Cross-slice contract documents. Use these when a decision affects multiple
future slices or multiple phase/milestone areas.

### `SPEC-*`

One honest implementation seam per spec. A spec should be specific enough to
implement but narrow enough to review without reopening the entire program.

### `PLAN-*`

Packet sequencing, dependency order, drift-resolution decisions, and
verification checkpoints for the corresponding `SPEC-*`.

### `TASKS-*`

Session-sized execution checklists for the corresponding `PLAN-*`.

## Proposed slice order

This scaffold currently assumes the roadmap sequence documented in
[`../ROADMAP.md`](../ROADMAP.md):

1. Slice `01`: supported mode and support taxonomy
2. Slice `02`: Lima version floor and breakglass contract
3. Slice `03`: canonical guest endpoint and transport contract
4. Slice `04`: routed consumer parity for PTY, non-PTY, doctor, and readiness
5. Slice `05`: backend policy input parity
6. Slice `06`: routed-path-first readiness/doctor/smoke truth
7. Slice `07`: remove default extra listener surface
8. Slice `08`: ingress inventory and narrowed mount contract
9. Slice `09`: ingress implementation and/or Substrate-managed sync path
10. Slice `10`: guest unit/service source of truth and sandbox unification
11. Slice `11`: Substrate-owned lifecycle and diagnostics contract
12. Slice `12`: breakglass reclassification and docs cutover

## Phase gate

Before writing the first `SPEC-*`:

1. review the existing phase/milestone docs,
2. review [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md),
3. review the initial `DESIGN-*` docs in this directory,
4. confirm whether any proposed slice needs to be split further based on live
   repo truth.

## Related docs

- [Execution Rubric](../EXECUTION-RUBRIC.md)
- [Roadmap](../ROADMAP.md)
- [Design docs](./design/README.md)
