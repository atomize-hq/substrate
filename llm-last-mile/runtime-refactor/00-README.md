# Runtime Refactor Control Pack

**Status:** canonical control pack for future runtime-refactor slices
**Scope:** planning, contracts, sequencing, and proof gates; not implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-07-09; re-check live code before every slice

## Canonical repo location

This pack's canonical location is:

```text
llm-last-mile/runtime-refactor/
```

The `../../...` links to repo-root directives and debug memos assume that placement. If this directory moves, update every affected relative link in the same PR.

## Purpose

This directory compresses the runtime-refactor directive into a selective-load control surface. It exists to prevent a recurring failure mode:

> An artifact in the tree is not evidence that its architecture seam has landed.

A seam is landed only when **all four** conditions are true:

1. the correct authority boundary owns the decision;
2. the real production call path routes through that boundary;
3. the intended policy is enforced at that boundary; and
4. smoke/e2e/regression proof exercises that exact path.

Unit tests, type names, persisted rows, helper functions, process liveness, socket reachability, and successful launch are useful evidence. None is sufficient by itself.

## Control-pack map

| File | Load when | Canonical content |
|---|---|---|
| [`01-target-architecture.md`](01-target-architecture.md) | deciding ownership or reviewing a boundary | target layers, authority map, non-negotiable invariants |
| [`02-seam-crosswalk.md`](02-seam-crosswalk.md) | scoping a slice or assessing current landing status | current artifacts, semantic classification, required action, sibling context |
| [`03-phase-slice-map.md`](03-phase-slice-map.md) | planning or executing a slice | five tracks, bounded slices, allowed areas, non-goals, exit and regression gates |
| [`04-contracts-and-gates.md`](04-contracts-and-gates.md) | changing schemas, receipts, supervisor behavior, policy, or UAA execution | concrete V1 contracts and acceptance rules |
| [`05-debug-regression-ledger.md`](05-debug-regression-ledger.md) | writing tests, smoke plans, or closeout evidence | resolved baselines, open debug seams, permanent regression gates |

Do not load the full historical design/debug stack by default. Start with the applicable crosswalk row, slice row, contract section, and regression row. Follow only the named must-read links. Slice A0 records its authority-leak inventory inside `02-seam-crosswalk.md`; it must not create an untracked seventh control-pack file.

## Semantic status labels

These labels describe the **target seam as a whole**, not the quality of individual functions.

| Label | Meaning |
|---|---|
| `ContractCorrectAndProven` | Correct owner, real path, enforcement point, and runtime proof all exist. |
| `UsefulFootholdButWrongBoundary` | Reusable logic/data exists, but ownership or call-path placement is wrong. |
| `DefensiveScaffoldingOnly` | The artifact reduces risk or enables transition, but does not implement the target authority contract. |
| `MislandedWrongModel` | The implementation encodes semantics that conflict with the target model and must be replaced or inverted. |
| `MissingSeam` | No meaningful implementation of the target boundary exists, even if neighboring primitives do. |

Promotion to `ContractCorrectAndProven` requires explicit evidence for all four landing conditions. A component test cannot promote a seam whose production path bypasses it.

## Authority vocabulary

- **Authority:** decides durable meaning and validates state transitions.
- **Persistence:** stores authority decisions; it does not invent them.
- **Transport:** delivers requests/events; reachability is a signal, not durable truth.
- **Projection:** derives a view or runtime-native artifact from canonical truth.
- **Enforcement:** makes the policy unavoidable on the side-effecting path.
- **Receipt:** durable accepted-work identity returned before terminal completion.
- **Supervisor:** restart-safe owner of post-acceptance observation and closeout.
- **Secret handoff:** one-time secure-FD delivery from host credential authority to the in-world Substrate gateway; never a UAA-native credential file projection.
- **Runtime-family adapter:** provider mechanics only; never Substrate lifecycle or policy semantics.

## Reading and update rules

1. Treat this pack as canonical for refactor intent, slice boundaries, contracts, and gates.
2. Treat live code plus fresh runtime evidence as canonical for current artifact truth.
3. If code truth changes, update the affected crosswalk row and regression row in the same implementation PR.
4. Keep a seam below `ContractCorrectAndProven` until its actual production path is proven.
5. Do not use helper/PID/socket liveness as authority in new contracts.
6. Do not create one crate per named seam. A seam may be a module, facade, trait, type, or extracted function set.
7. Keep slice boundaries hard. Adjacent sibling seams stay in context but are not implicit scope.
8. Preserve resolved debug behavior while replacing the model that produced it.

## Current control conclusion

The current tree contains important constraints and footholds, but this pack does not classify any required seam as `ContractCorrectAndProven`. That is intentional. A0 should inventory the authority leaks without changing behavior. The first implementation PR after A0 should make one authority boundary true and prove it; it should not claim the architecture landed because similarly named artifacts already exist.
