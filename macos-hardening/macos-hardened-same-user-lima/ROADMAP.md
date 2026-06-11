# Roadmap: Landing `macos-hardened-same-user-lima`

Status: draft roadmap  
Last updated: 2026-06-11

## Purpose

Turn the existing `macos-hardened-same-user-lima` phase and milestone docs into
an execution-ready spec-driven workflow without discarding the work already
captured in this feature directory.

This roadmap freezes the document strategy for landing the macOS hardening work:

1. keep the existing phases as the program-level sequencing authority,
2. add cross-cutting `DESIGN-*` docs for contracts that span multiple slices,
3. land implementation through bounded numbered `SPEC-*`, `PLAN-*`, and
   `TASKS-*` slices under a new `spec/` directory.

## Why keep the existing phases

The current phase structure already captures real dependency order:

1. **Phase 0** freezes the support and security contract.
2. **Phase 1** converges runtime parity around already-landed CLI/operator
   surfaces.
3. **Phase 2** narrows the same-user Lima runtime and guest attack surface.
4. **Phase 3** consolidates Substrate-owned operator workflows and completes the
   docs cutover.

That sequencing is useful and should remain the high-level program map.

What is still missing is the repo’s usual implementation granularity:

- one honest seam per spec,
- reviewable plan packets,
- session-sized tasks,
- explicit contract docs for cross-slice decisions.

## Document layering

The landing stack for this feature should be:

1. **Feature overview + phase/milestone docs**
   - keep the existing `README.md` plus `phase-*` READMEs and milestone SOWs
   - treat them as program truth, scope framing, and milestone sequencing
2. **Roadmap**
   - this file
   - explains how the current program docs translate into repo-native execution
3. **Execution rubric**
   - [`EXECUTION-RUBRIC.md`](./EXECUTION-RUBRIC.md)
   - defines how future short prompts should identify the next slice, required
     skills, and required official sources
4. **Design docs**
   - live under `spec/design/`
   - freeze cross-slice contracts and architectural direction
5. **Slice docs**
   - future `SPEC-*`, `PLAN-*`, and `TASKS-*` files under `spec/`
   - each slice should own one bounded implementation seam

## Design docs to establish first

These are the first cross-cutting design inputs to write before drafting the
initial implementation slices:

1. `spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`
2. `spec/design/DESIGN-macos-lima-transport-contract.md`
3. `spec/design/DESIGN-macos-policy-input-parity.md`
4. `spec/design/DESIGN-macos-ingress-and-mount-contract.md`
5. `spec/design/DESIGN-macos-guest-unit-source-of-truth.md`
6. `spec/design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`

These are design inputs, not implementation specs. They exist to prevent the
same contract from being redefined in multiple `SPEC-*` files.

## Proposed numbered slice map

The current milestones are useful epics, but not every milestone should map
1:1 to a single implementation spec. The recommended initial slice map is:

Slice `01` is also the terminology freeze point for the feature-local support
taxonomy. Later slices should keep reusing the exact labels `supported`,
`degraded-but-supported`, and `breakglass` rather than restating them with new
categories.

| Slice | Focus | Primary phase alignment |
| --- | --- | --- |
| 01 | supported mode and support taxonomy | Phase 0 |
| 02 | Lima version floor and breakglass contract | Phase 0 |
| 03 | canonical guest endpoint and transport contract | Phase 1 |
| 04 | PTY, non-PTY, doctor, and readiness transport convergence | Phase 1 |
| 05 | `MacLimaBackend` policy input parity | Phase 1 |
| 06 | routed-path-first doctor/smoke/readiness truth | Phase 1 |
| 07 | remove default extra listener surface | Phase 2 |
| 08 | ingress inventory and narrowed mount contract | Phase 2 |
| 09 | ingress implementation and/or Substrate-managed sync path | Phase 2 / 3 boundary |
| 10 | guest unit/service source of truth and sandbox unification | Phase 2 |
| 11 | Substrate-owned lifecycle and diagnostics contract | Phase 3 |
| 12 | breakglass reclassification and docs cutover | Phase 3 |

This numbering is intentionally local to this feature and should stay stable
even if milestone wording evolves.

## Mapping from milestones to slices

### Phase 0

- milestone `0.1` maps most directly to Slice `01`
- milestone `0.2` maps most directly to Slice `02`
- milestone `0.1` should leave later docs with one stable vocabulary for
  `supported`, `degraded-but-supported`, and `breakglass`

### Phase 1

- milestone `1.1` likely spans Slices `03` and `04`
- milestone `1.2` maps most directly to Slice `05`
- milestone `1.3` maps most directly to Slice `06`

### Phase 2

- milestone `2.1` maps most directly to Slice `07`
- milestone `2.2` is likely too broad for one slice and should begin with Slice
  `08`, then continue into Slice `09`
- milestone `2.3` maps most directly to Slice `10`

### Phase 3

- milestone `3.1` is likely too broad for one slice and should begin with Slice
  `11`, with any ingress/sync cutover work coordinated with Slice `09`
- milestone `3.2` maps most directly to Slice `12`

## Execution rules for future `SPEC-*` / `PLAN-*` / `TASKS-*`

### Spec rules

Each `SPEC-*` should:

1. own one honest seam,
2. list assumptions at the top,
3. point back to the relevant phase/milestone docs and any `DESIGN-*` inputs,
4. define commands, touched repo surfaces, testing expectations, and
   boundaries,
5. state clear success criteria and explicit non-goals.

### Plan rules

Each `PLAN-*` should:

1. identify packet sequencing and dependencies,
2. call out risks and drift-resolution decisions,
3. name what is parallelizable and what is not,
4. define verification checkpoints between packets.

### Tasks rules

Each `TASKS-*` should:

1. break the slice into session-sized work packets,
2. include acceptance and verification per task,
3. keep touched file sets narrow,
4. avoid silently widening into adjacent milestones.

## Commands and evidence surfaces to keep central

As slices are created, keep these evidence surfaces explicit whenever they are
relevant to the seam:

```bash
cargo test -p world-mac-lima
substrate host doctor --json
substrate world doctor --json
substrate world gateway status --json
scripts/mac/lima-doctor.sh
scripts/mac/smoke.sh
scripts/mac/orchestration-smoke.sh
```

Additional targeted test commands should be named inside each slice based on
the actual touched code paths.

## Boundaries

### Always

- Keep the existing phase and milestone docs as the high-level authority unless
  repo truth forces a correction.
- Use `DESIGN-*` docs only for decisions that span multiple future slices.
- Keep each implementation slice narrow and dependency-ordered.
- Preserve the explicit same-user limitation as a first-class constraint.

### Ask first

- Renumbering the proposed slice map once `SPEC-*` files begin landing.
- Collapsing multiple proposed slices into one if the live code says the seam is
  smaller than expected.
- Broadening Phase 3 into a larger CLI redesign beyond the macOS hardening
  scope.

### Never

- Treat the current phase READMEs or milestone SOWs as implementation-complete
  specs by themselves.
- Claim Linux-equivalent host ownership isolation for same-user Lima.
- Let a single `SPEC-*` silently absorb multiple milestone-scale seams.

## Immediate next steps

1. Create the `spec/` scaffold for this feature.
2. Draft the six initial `DESIGN-*` docs under `spec/design/`.
3. Start with `SPEC-01`, `PLAN-01`, and `TASKS-01` for supported mode and
   support taxonomy once the design inputs are reviewed.
4. Follow with `SPEC-02`, `PLAN-02`, and `TASKS-02` for the version-floor and
   breakglass contract.

## Related docs

- [Feature overview](./README.md)
- [Phase 0: Security Contract and Scope](./phase-0-security-contract-and-scope/README.md)
- [Phase 1: Runtime Parity Foundation](./phase-1-runtime-parity-foundation/README.md)
- [Phase 2: Same-User Hardening](./phase-2-same-user-hardening/README.md)
- [Phase 3: Substrate-Owned Operations](./phase-3-substrate-owned-operations/README.md)
- [Execution Rubric](./EXECUTION-RUBRIC.md)
- [Spec directory](./spec/README.md)
