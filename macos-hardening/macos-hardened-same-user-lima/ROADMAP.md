# Roadmap: `macos-hardened-same-user-lima`

Status: superseded historical roadmap / current-state index
Last updated: 2026-06-22

## What this file is now

This file is **not** the live execution authority for creating a `spec/`
scaffold or starting `SPEC-01` / `SPEC-02`. That work already landed.

Use this file as:

1. the historical slice map for how the feature was sequenced, and
2. a pointer to the **current** authority stack.

## Current live authority stack

For execution or repo-truth questions, read these instead:

1. [`README.md`](./README.md) — feature-level current-state summary
2. [`spec/README.md`](./spec/README.md) — current slice index and authority map
3. [`EXECUTION-RUBRIC.md`](./EXECUTION-RUBRIC.md) — planning protocol
4. committed `SPEC-*`, `PLAN-*`, and `TASKS-*` files under [`spec/`](./spec/)
5. top-level operator docs:
   - [`../../docs/WORLD.md`](../../docs/WORLD.md)
   - [`../../docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
   - [`../../docs/USAGE.md`](../../docs/USAGE.md)

## Current state

The feature is no longer in scaffold/bootstrap mode.

Committed repo truth now includes:

1. `SPEC-01` through `SPEC-12`,
2. matching `PLAN-*` and `TASKS-*` documents,
3. landed runtime and docs cutover work for policy parity, listener removal,
   staged-workspace ingress, canonical unit authority, and routed-path-first
   macOS operator proof.

## Historical slice map

The original feature-local numbering still remains useful as a retrospective
map:

| Slice | Focus | Phase |
| --- | --- | --- |
| 01 | supported mode and support taxonomy | Phase 0 |
| 02 | Lima version floor and breakglass contract | Phase 0 |
| 03 | canonical guest endpoint and transport contract | Phase 1 |
| 04 | PTY, non-PTY, doctor, and readiness transport convergence | Phase 1 |
| 05 | backend policy input parity | Phase 1 |
| 06 | routed-path-first doctor/smoke/readiness truth | Phase 1 |
| 07 | remove default extra listener surface | Phase 2 |
| 08 | ingress inventory and narrowed mount contract | Phase 2 |
| 09 | ingress cutover and explicit staging path | Phase 2 / 3 boundary |
| 10 | guest unit/service source of truth and sandbox unification | Phase 2 |
| 11 | Substrate-owned lifecycle and diagnostics contract | Phase 3 |
| 12 | breakglass reclassification and docs cutover | Phase 3 |

## How to use this file going forward

- Use it to understand **why** the slice order exists.
- Do **not** use it as proof that scaffold creation or early-slice planning is
  still pending.
- If live code or docs disagree with this file, live repo truth wins and this
  file should be treated as historical framing only.

## Related docs

- [Feature overview](./README.md)
- [Spec README](./spec/README.md)
- [Execution Rubric](./EXECUTION-RUBRIC.md)
