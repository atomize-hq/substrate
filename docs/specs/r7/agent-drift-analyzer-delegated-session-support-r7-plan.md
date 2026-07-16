# Implementation Plan: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`

Status: **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-3 COMPLETE /
CHECKPOINT-DOC COMMIT `931c2701c` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN` /
`CTX-R7-04` PROVEN / R7-4 SOLE ACTIVE PHASE AT ENTRY ONLY / ACTIVE PACKET NONE / TRANSITION/FIX SERIES `e077de489` + `3dd5ba943` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN`; FIRST REVIEW'S TWO P2 STALE-STATUS DEFECTS CORRECTED / R7-4.1 NEXT, UNCHECKED, AND UNSTARTED /
R7-4 PRODUCTION/SCORER WORK UNSTARTED / R7-5..R7-6 AND R8 BLOCKED / PROMPT 1 SELECTORS
`PHASE_ID: R7-4` / `ACTIVE_PACKET: none` PREPARED AND ELIGIBLE BUT NOT INVOKED**

This implementation plan is reconciled and implementation-ready. R6 is `CLOSED`, and promotion
series `455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`, so
`R7-PROMOTE` is complete. Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` +
`faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` are fresh independent built-in `default`
`REVIEW CLEAN`, completing `R7-0`. Transition/fix series `339744dff` + `d20cac6a9` also received
fresh independent built-in `default` `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`,
`4d122cd9f`, and `e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. R7-1
checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`,
plus R7-2.3 series `7af2ae517` + `75a353e46`, received fresh independent built-in `default`
`REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Only R7-4 is active at
entry with packet `none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both.
`R7-4.1` is next, unchecked, and unstarted; no R7-4 production/scorer work has started.
`R7-5..R7-6` and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are
prepared and eligible but have not been invoked.

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
7. **Existing scorer taxonomy is sufficient for R7-4 evidence.** Existing scorers run per
   trajectory and typed delegation context preserves ownership. Any future distinct,
   unexpressible delegated failure mode requires a separate evidence-backed reviewed packet.
8. **R7-compatible sentinel only.** Multi-session cursor and presentation support may change, but
   broad replay/live interpretation refactoring remains R8.
9. **R6 closure boundary stands.** Do not reopen `R6-3.X.3`, conditional `R6-4`, or closed
   semantic-goal-drift packets without new evidence, and do not make R7 absorb the ordinary
   single-session acceptance gaps named by the R6 closure finding.

## Dependency Graph

The R6 closure entry gate is satisfied: applicability audit complete; every material scoring
surface assigned exactly one terminal disposition; broad acceptance proven or narrowed honestly;
named controls resolved; and the R6 finding plus authority stack updated to `CLOSED`. Promotion
series `455d0ed90` + `876ac55de` completed the family content/gate audit and received fresh
independent built-in `default` `REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` +
`faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` are fresh independent built-in `default`
`REVIEW CLEAN`, completing `R7-0`. Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430`
remains the review-clean `R7-0` entry receipt. Transition/fix series `339744dff` + `d20cac6a9` is
the review-clean `R7-1` entry receipt. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and
`e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`; the R7-1 checkpoint is
complete. Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Only R7-4 is active at
entry with packet `none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both.
`R7-4.1` is next, unchecked, and unstarted; no R7-4 production/scorer work has started.
`R7-5..R7-6` and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are
prepared and eligible but have not been invoked.

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
- Record that the `dead_end_thrash` progress-aware core is landed and R6 is closed while preserving
  the separate R7 phase-transition boundary.
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
- Receipt: commit `c60d05f77` is fresh independent built-in `default` `REVIEW CLEAN`; input proof is
  `16 / 16`, and staged GitNexus reported LOW / `0` affected processes.

### R7-2.2 Promote delegation to checkpoint v0.8

- Move/promote topology and visibility types to the public schema.
- Add `parent_session_id`, ordered `child_session_ids`, confidence, and evidence.
- Add `ChildWorkVisibility::Linked`.
- Require the field for v0.8 while preserving v0.7 deserialization.
- Receipt: after operator decision `R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A`, commit `9403c8a24`
  received fresh independent built-in `default` `REVIEW CLEAN`. Public v0.8, readable v0.7,
  `Linked`, role ids, and deterministic `RowRef` evidence are proven; staged GitNexus reported
  MEDIUM / `1` affected process.

### R7-2.3 Derive trajectory roles from verified graph truth

- Populate `DelegatingParent` and `DelegatedChild` from verified links.
- Use existing heuristic markers only as conservative fallback when graph truth is missing.
- Resolve mixed verified/unverified children to `Partial`.
- Resolve conflicts to `MixedOrAmbiguous` or `Opaque` without semantic import.
- Receipt: series `7af2ae517` + `75a353e46` received fresh independent built-in `default` `REVIEW
  CLEAN` after the fix reconciled the summary-vs-checkpoint blocker. Graph-derived roles and ids,
  fail-closed conflicts, JSON-summary parity, and separate trajectories are proven; staged GitNexus
  reported HIGH / `9` affected processes within the authorized seam and MEDIUM / `2` for the fix.

Checkpoint:

```bash
cargo test -p agent-drift-analyzer delegation -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

R7-2 behavior/static checkpoint receipt: at implementation HEAD `75a353e46`, input passes `16 / 16`;
delegation matches pass `39` total (`25` library + `4` checkpoint + `8` delegation-context + `2`
export); checkpoint matches pass `172` total (`36` library + `134` checkpoints + `1` export + `1`
truth-grounding); full analyzer passes `417 / 417`; formatting, `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and
diff checks are green. No R7-3, R7-4, sentinel, or R8 work leaked in. Checkpoint-doc receipt
`78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, so R7-2 is `COMPLETE` and
`CTX-R7-03` is `PROVEN`. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` are fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 and R7-3.2 are
complete at fresh-review-clean test-only commits `f8dd04549` and `c7c6f35b8`. Checkpoint-doc commit
`931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-3 exit
gate and proving `CTX-R7-04`. Only R7-4 is active at entry with `ACTIVE_PACKET: none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both. No R7-4 work has started.

## Phase 3: Separate Parent And Child Progress

### R7-3.1 Keep progress trajectory-local

- Analyze every included child session through the existing checkpoint/progress pipeline.
- Keep parent checkpoints on `ParentVisibleOrchestration` when that is the visible dimension.
- Never substitute a child checkpoint's `SessionProgress` into a parent checkpoint.
- Receipt: test-only commit `f8dd04549` received fresh independent built-in `default` `REVIEW
  CLEAN`. Its placeholder acceptance scaffold intentionally produced test-scaffold RED `0 / 1`;
  after replacement with the real acceptance assertion, the exact target passed `1 / 1`, and full
  `progress_acceptance` passed `4 / 4`. Parent and child progress/evidence remain trajectory-local;
  staged GitNexus was LOW / `0` affected processes.

### R7-3.2 Pin parent/child semantic separation

- Parent wait plus child advancement: parent remains bounded; child may be `Advancing` from child
  evidence.
- Parent clean orchestration plus child stall: child may be `Stalled`; parent does not inherit it.
- Missing child: parent stays `Partial`/`Opaque`; no child trajectory is fabricated.
- Receipt: test-only commit `c7c6f35b8` received fresh independent built-in `default` `REVIEW
  CLEAN`. Its placeholder acceptance scaffolds intentionally produced test-scaffold RED `3 / 3`;
  after replacement with the real acceptance assertions, exact `3 / 3` passed, and the checkpoint
  filter passed `175` matched tests across targets. Staged GitNexus was LOW / `0` affected processes.

Checkpoint:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

R7-3 behavior/static checkpoint receipt: at implementation HEAD `c7c6f35b8`, R7-3.1's placeholder
acceptance scaffold intentionally produced test-scaffold RED `0 / 1`; after replacement with the
real acceptance assertion, exact `1 / 1` and full `progress_acceptance` `4 / 4` passed. R7-3.2's
placeholder acceptance scaffolds intentionally produced test-scaffold RED `3 / 3`; after
replacement with the real acceptance assertions, exact `3 / 3` passed. Checkpoint matches passed
`175` across targets, and
`cargo test -p agent-drift-analyzer -- --nocapture` passed `421 / 421` aggregate. Formatting,
`cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and diff checks are green.
`cargo clippy --workspace --all-targets -- -D warnings` remains RED only in R7-6-owned sentinel test
constructors missing
`Checkpoint.delegation` at `crates/agent-drift-sentinel/tests/support/mod.rs:81`,
`crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs:48`, and
`crates/agent-drift-sentinel/tests/replay_input.rs:79`; route that witness to already-planned R7-6.1
and do not claim workspace clippy green. Checkpoint-doc commit `931c2701c` received fresh independent
built-in `default` `REVIEW CLEAN`, satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is
complete. Only R7-4 is active at entry with `ACTIVE_PACKET: none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both. `R7-4.1` is next, unchecked, and unstarted; no R7-4 production/
scorer work has started. Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are prepared
and eligible but uninvoked.

## Phase 4: Delegated Scorer Guardrails

### R7-4.1 Apply scores only to the observed trajectory

- Keep `score_session` per `BundleSession`.
- Prevent parent-side waits or orchestration loops from becoming claims about child thrash.
- Preserve child scorer results on child checkpoints only.
- Keep semantic-goal-drift opaque-parent guardrails intact.

### R7-4.2 Decide taxonomy from evidence

- Decision: existing `DriftClass` values plus typed delegation context are sufficient for the live
  R7-4.1 evidence. The production-shaped linked witness keeps parent waits cleared, preserves
  child-local repetition as active `dead_end_thrash`, and identifies each trajectory through
  topology and child-work visibility.
- No new `DriftClass` variant was added. If future evidence demonstrates a distinct delegated
  failure mode that cannot be expressed through trajectory-local scores plus delegation context,
  stop and open a separate evidence-backed reviewed packet before schema or compatibility edits.
- Upstream GitNexus impact for `DriftClass` was LOW with `0` direct callers, `0` affected
  processes, and `0` affected modules; this manual boundary review did not authorize an enum edit.

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

No unresolved design question emerged from R7-2 implementation. The R7-1 task series `e65127720` +
`685cf843b`, `4d122cd9f`, and `e865eee13` are fresh independent `REVIEW CLEAN`, and the R7-1
checkpoint is complete. Checkpoint-doc commit `1cae7d693` received fresh independent built-in
`default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits
`c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` + `75a353e46`, received fresh
independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Only R7-4 is active at
entry with packet `none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both.
`R7-4.1` is next, unchecked, and unstarted; no R7-4 production/scorer work has started.
`R7-5..R7-6` and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are
prepared and eligible but have not been invoked.
Default-on linked closure and recursive depth remain evidence-gated decisions for later packets.
The R7-4 evidence requires no new drift taxonomy; any future distinct, unexpressible delegated
failure mode requires a separate evidence-backed reviewed packet.
