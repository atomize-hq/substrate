# Implementation Plan: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`

Status: **DRAFT / BLOCKED ON R6 CLOSURE DECISION**

This implementation plan is inactive. Preserve it as design-ready draft work until
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md` reaches `CLOSED`; do not execute any
phase before that gate.

## Overview

Implement direct parent/child semantics as a typed pipeline:

```text
raw parent + child rollouts
        -> compactor reciprocal-link contract and explicit direct-child closure
        -> analyzer multi-session link graph and per-trajectory checkpoints
        -> trajectory-local progress/scoring
        -> minimal replay/live sentinel compatibility
```

The implementation deliberately does not flatten trajectories or use parent orchestration as a proxy
for child work.

## Architecture Decisions

1. **Compactor owns raw linkage.** It is already the raw rollout parser and must preserve structured
   `session_meta` source metadata plus parent spawn-result ids.
2. **Reciprocal direct links only.** Parent and child ids must agree. One-sided evidence remains
   diagnostic and non-semantic.
3. **Explicit closure.** Ordinary `--session-id` behavior stays stable; callers opt into direct linked
   children until acceptance evidence supports a default change.
4. **Additive compactor contract.** `delegation_links` is serde-defaulted on bundle `v0.2`.
5. **Required analyzer contract.** Checkpoint `v0.8` requires a public delegation context for every
   checkpoint, including `SingleAgent`.
6. **Separate trajectory truth.** Child progress stays on child checkpoints. Parent checkpoints hold
   parent-visible progress plus link references, never copied child status.
7. **No new scorer taxonomy by default.** Existing scorers run per trajectory; a new drift class
   requires later acceptance evidence.
8. **R7-compatible sentinel only.** Multi-session cursor and presentation support may change, but
   broad replay/live interpretation refactoring remains R8.
9. **R6 closure boundary stands.** Do not reopen `R6-3.X.3`, conditional `R6-4`, or closed
   semantic-goal-drift packets without new evidence, and do not make R7 absorb the ordinary
   single-session acceptance gaps named by the R6 closure finding.

## Dependency Graph

The entire graph is blocked on the R6 closure gate: applicability audit complete, every material
scorer classified, broad acceptance proven or narrowed honestly, named controls resolved, and the
R6 finding plus authority stack updated to `CLOSED`.

```text
R7-0 docs + sanitized evidence matrix
        |
        v
R7-1 compactor link extraction + direct-child closure
        |
        v
R7-2 analyzer link graph + checkpoint v0.8 delegation contract
        |
        v
R7-3 per-trajectory parent/child progress separation
        |
        v
R7-4 trajectory-local scorer guardrails
        |
        v
R7-5 delegated acceptance + real-corpus proof
        |
        v
R7-6 minimal sentinel replay/live compatibility
```

## Phase 0: Docs Lock And Evidence Matrix

### R7-0.1 Freeze the family contract

- Land `MAP`, `SPEC`, `PLAN`, and `TASKS` before implementation.
- Record that the `dead_end_thrash` progress-aware core is landed while the broad R6 closure audit
  remains partial.
- Freeze direct-only reciprocal linkage and parent/child progress separation.

Verification checkpoint:

```bash
rg -n "R6-1|reciprocal|direct child|Never infer child" docs/specs/r6/MAP.md docs/specs/r7
git diff --check
```

### R7-0.2 Build sanitized linkage fixtures

- Derive minimal raw parent and child fixtures from current rollout shapes.
- Include reciprocal, one-sided, conflict, multi-child, and nested-depth residue cases.
- Strip prompts, credentials, absolute private content, and unrelated tool output.

Checkpoint: fixture review proves only linkage-bearing fields remain.

## Phase 1: Compactor Link Contract

### R7-1.1 Preserve raw linkage metadata

- Extend ingestion to retain structured child-origin metadata without turning it into normal prose.
- Extract parent spawn-result child ids by matching function calls to function-call outputs.
- Keep row-level provenance for each side.

### R7-1.2 Validate reciprocal links

- Add typed `DelegationLink` and `DelegationLinkState`.
- Require exact parent/child agreement for `Verified`.
- Detect self-links, conflicting parents, duplicate child artifacts, and malformed ids.
- Sort and deduplicate links deterministically.

### R7-1.3 Add explicit direct-child closure

- Add library/CLI opt-in for linked direct children.
- Discover child artifacts by verified child id, then verify child metadata before inclusion.
- Record deeper descendants as residue without recursively importing them.
- Preserve ordinary single-session discovery behavior when the option is absent.

Checkpoint:

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-session-compactor --test end_to_end -- --nocapture
```

## Phase 2: Analyzer Link Graph And Public Contract

### R7-2.1 Load and validate delegation links

- Read serde-defaulted `manifest.delegation_links`.
- Confirm every verified link references sessions actually present in the bundle.
- Build a deterministic direct link graph keyed by session id.
- Keep legacy manifests valid with an empty graph.

### R7-2.2 Promote delegation to checkpoint v0.8

- Move/promote topology and visibility types to the public schema.
- Add `parent_session_id`, ordered `child_session_ids`, confidence, and evidence.
- Add `ChildWorkVisibility::Linked`.
- Require the field for v0.8 while preserving v0.7 deserialization.

### R7-2.3 Derive trajectory roles from verified graph truth

- Populate `DelegatingParent` and `DelegatedChild` from verified links.
- Use existing heuristic markers only as conservative fallback when graph truth is missing.
- Resolve mixed verified/unverified children to `Partial`.
- Resolve conflicts to `MixedOrAmbiguous` or `Opaque` without semantic import.

Checkpoint:

```bash
cargo test -p agent-drift-analyzer delegation -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## Phase 3: Separate Parent And Child Progress

### R7-3.1 Keep progress trajectory-local

- Analyze every included child session through the existing checkpoint/progress pipeline.
- Keep parent checkpoints on `ParentVisibleOrchestration` when that is the visible dimension.
- Never substitute a child checkpoint's `SessionProgress` into a parent checkpoint.

### R7-3.2 Pin parent/child semantic separation

- Parent wait plus child advancement: parent remains bounded; child may be `Advancing` from child
  evidence.
- Parent clean orchestration plus child stall: child may be `Stalled`; parent does not inherit it.
- Missing child: parent stays `Partial`/`Opaque`; no child trajectory is fabricated.

Checkpoint:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## Phase 4: Delegated Scorer Guardrails

### R7-4.1 Apply scores only to the observed trajectory

- Keep `score_session` per `BundleSession`.
- Prevent parent-side waits or orchestration loops from becoming claims about child thrash.
- Preserve child scorer results on child checkpoints only.
- Keep semantic-goal-drift opaque-parent guardrails intact.

### R7-4.2 Decide taxonomy from evidence

- Start with no new `DriftClass`.
- If the acceptance corpus demonstrates a distinct delegated failure mode that cannot be expressed
  through trajectory-local scores plus delegation context, stop and open a new reviewed packet.

Checkpoint:

```bash
cargo test -p agent-drift-analyzer --test dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## Phase 5: Delegated Acceptance And Corpus Proof

### R7-5.1 Land the bounded acceptance matrix

- Commit sanitized reciprocal, one-sided, conflict, multi-child, nested residue, and single-agent
  controls.
- Assert topology, visibility, link state, confidence, progress attribution, and scorer ownership.

### R7-5.2 Run real-session validation

- Re-run named R3.75/R5.75 delegated witnesses.
- Add a verified parent/child pair proof using sanitized derivatives of `019e93f8-...` and
  `019e93fa-...`.
- Report counts by topology/visibility stratum and manually audit fires/suppressions.
- Do not commit raw private rollouts.

Checkpoint: R7 cannot advance to sentinel live changes until linked vs opaque semantics are
review-clean in analyzer output.

## Phase 6: Minimal Sentinel Compatibility

### R7-6.1 Accept checkpoint v0.8

- Add v0.8 validation and legacy compatibility.
- Render compact topology/role/link visibility from the analyzer field.
- Keep the sentinel free of raw-link inference.

### R7-6.2 Support linked multi-session live output

- Replace the exact-single-session assertion with validation against the analyzer-owned verified
  direct closure.
- Persist a root session id plus per-session cursors.
- Preserve monotonic cursor checks independently per session.
- Keep scheduling/intervention semantics unchanged.

Checkpoint:

```bash
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

## Final Verification Wall

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
cargo test --workspace -- --nocapture
git diff --check
npx gitnexus detect-changes -r 97a0-substrate
```

Manual proof must confirm:

- a linked child is not mistaken for parent activity;
- an opaque parent does not claim child progress;
- single-agent outputs remain semantically unchanged;
- live/replay show the same analyzer-owned delegation role for matching checkpoints.

## Parallelization

Safe only after contracts are frozen:

- sanitized fixture authoring can proceed alongside compactor link implementation;
- sentinel v0.8 deserialization fixtures can be prepared after the analyzer schema is final.

Sequential requirements:

- reciprocal link extraction before analyzer graph semantics;
- analyzer graph semantics before progress/scorer changes;
- analyzer acceptance before multi-session live sentinel behavior.

## Risks And Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Parent/child links are inferred from weak prose | false child semantics | require reciprocal structured evidence |
| Child discovery silently widens every compaction | unstable outputs/performance | explicit opt-in until acceptance proves safe |
| Parent and child progress are flattened | misleading progress/drift | keep separate session checkpoints |
| Multi-session live cursors regress | duplicate or skipped events | persist/check cursors per session |
| Compactor schema breaks old bundles | compatibility regression | additive serde-defaulted v0.2 field |
| v0.8 leaks into R8 redesign | scope explosion | minimal compatibility/presentation only |
| Private rollout data enters fixtures | privacy/security issue | sanitized minimal derivatives and review gate |
| Nested delegation expands unboundedly | graph complexity | direct children only; deeper residue explicit |

## Open Questions

No blocker for R7-1. Default-on linked closure, new drift taxonomy, and recursive depth remain
evidence-gated decisions for later packets.
