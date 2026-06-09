# Design: R5 Validation And Rollout Protocol

Status: canonical design authority locked in Packet R5-0 on 2026-06-09.

## Why This Doc Exists

R5 introduces semantic progress labels. Green unit tests that only assert "a field exists" are not
enough. The repo's landing-order authority already warns not to validate future drift semantics
only through synthetic fixtures. DoVer strengthens that warning: a failure/progress explanation is
not reliable unless it can be validated, refuted, or marked inconclusive.

This doc defines the validation protocol for R5.

## Validation Philosophy

R5 should be validated by three layers:

1. **Synthetic focused fixtures** for exact deterministic behavior.
2. **Bundle-shaped analyzer fixtures** for realistic checkpoint assembly and summary output.
3. **Bounded real-rollout annotation** for semantic honesty on actual traces.

Optional but useful:

4. **DoVer-inspired counterfactual replay study** for a small troubleshooting subset after the base
   R5 field is stable.

Packet R5-0 locks the family verification story:

- `R5-0` is manual doc review only,
- `R5-1` through `R5-6` prove schema, rule, summary, and replay/live compatibility behavior,
- `R5-7` is the first packet allowed to claim bounded semantic acceptance on a dedicated progress
  corpus.

## What Counts As A Correct R5 Output

A checkpoint progress label is correct only if it satisfies all of these:

1. the status and dimension match the documented fixture expectation,
2. confidence is within the fixture's ceiling/floor,
3. supporting evidence names the observed signal,
4. delegated cases include limiting counter-evidence, and ambiguous or negative cases include
   counter-evidence when meaningful contradictory or limiting evidence exists,
5. the label does not claim progress for an unexercised target,
6. the label does not infer opaque child work,
7. the output remains deterministic across reruns.

## Fixture Manifest

R5 should add:

```text
docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
```

Each fixture entry should record:

```text
Fixture id:
Archetype:
Expected dimension:
Expected status:
Expected confidence:
Decisive evidence:
Counter-evidence:
Comparable attempts:
Window/reset expectation:
Why competing statuses lose:
Implementation fixture location:
```

This lets review debate the documented semantic boundary instead of taste.

## Required Synthetic Matrix

The first R5 implementation should cover at least these cases:

| ID | Case | Expected |
|---|---|---|
| `r5_troubleshooting_frontier_advanced_compile_to_test` | compile/build failure becomes focused test/assertion failure on overlapping target | `advancing / troubleshooting_frontier` |
| `r5_troubleshooting_failure_count_reduced` | many failing tests become fewer comparable failing tests | `advancing / troubleshooting_frontier` |
| `r5_troubleshooting_same_signature_no_edit` | same diagnostic signature repeats with no overlapping edit | `stalled / troubleshooting_frontier` |
| `r5_troubleshooting_previous_clean_broken` | previously clean focused verifier later breaks on same scope | `regressing / troubleshooting_frontier` for `troubleshooting`, or `regressing / implementation_verification_wall` for `autonomous_implementation` |
| `r5_planning_candidate_set_narrows_to_spec` | broad scan narrows into spec/plan/tasks artifact | `advancing / planning_convergence` |
| `r5_planning_broad_scan_meanders` | repeated broad reads/searches expand candidate set with no artifact | `stalled` or low-confidence `mixed / planning_convergence` |
| `r5_implementation_wall_advances` | source edits plus aligned tests move verifier from focused to broader proof | `advancing / implementation_verification_wall` |
| `r5_implementation_same_failure_unrelated_edits` | same verifier failure repeats after unrelated edits | `stalled / implementation_verification_wall` |
| `r5_closeout_clean_proof_no_source_churn` | proof commands pass, residual scope shrinks, no source churn | `advancing / verification_closeout_narrowing` |
| `r5_closeout_scope_narrows_to_residual` | closeout proof narrows the remaining residual scope and emits `VerificationScopeNarrowed` plus `ResidualScopeShrank` | `advancing / verification_closeout_narrowing` |
| `r5_closeout_reopens_source_churn` | closeout-labeled checkpoint introduces new source changes and breaks proof | `regressing` or `mixed / verification_closeout_narrowing` |
| `r5_sparse_no_comparable_attempt` | archetype exists but no comparable progress evidence | `insufficient_evidence` |
| `r5_delegated_parent_opaque` | parent-visible orchestration with opaque child work | low-confidence `insufficient_evidence / parent_visible_orchestration`, `stalled / parent_visible_orchestration`, or `mixed / parent_visible_orchestration` |
| `r5_legacy_v0_5_still_loads` | v0.5 checkpoint without session_progress remains compatible | legacy load succeeds |
| `r5_v0_6_requires_progress` | v0.6 checkpoint missing session_progress fails closed | contract error |
| `r5_replay_live_v0_6_parity` | same v0.6 checkpoint renders same compact progress line | replay/live parity |

## Real-Rollout Annotation

R5 should add a small bounded annotation corpus before R6 consumes the field. It does not need to be
large. It should be reviewable.

Recommended shape:

```text
crates/agent-drift-analyzer/tests/progress_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<case-id>/manifest.json
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<case-id>/rows.archival.jsonl
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<case-id>/rows.compact.jsonl
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<case-id>/dedupe-audit.jsonl
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<case-id>/expected.json
```

This matches the current analyzer fixture helpers and bundle shape more closely than an invented
single-file `bundle.json`.

The existing `crates/agent-drift-analyzer/tests/fixtures/acceptance` corpus is a frozen R2 wall
with a fixed harness contract. R5 should prefer a dedicated progress corpus/harness instead of
quietly mutating that older acceptance surface unless the packet explicitly chooses to widen it.

Recommended first corpus:

1. one recovered/sticky non-subagent session from the existing success-tail family,
2. one active failure or troubleshooting session with nonzero exits,
3. one planning/spec-development session,
4. one closeout/proof session,
5. one delegated/opaque parent session that should not make child progress claims.

Each annotated checkpoint should include only two or three decision points per session. Do not
attempt to label every checkpoint in a long run at first.

## DoVer-Inspired Offline Checks

R5 should not add online interventions. But an optional offline experiment can borrow DoVer's
validation pattern:

1. choose a small set of troubleshooting checkpoints labeled `advancing` or `stalled`,
2. inspect later replay tails or known alternate continuations,
3. ask whether `advancing` is enriched in recoveries and `stalled/regressing` is enriched in
   persistent failures,
4. record mismatches as fixture improvements or R6 scoring caveats.

This is not a blocker for the first R5 implementation, but it is useful before R6 scorer cutover.

## Test Placement

Analyzer tests:

```text
crates/agent-drift-analyzer/tests/checkpoints.rs
  schema v0.6, requiredness, builder-level progress cases.

crates/agent-drift-analyzer/tests/export_bundle.rs
  summary progress distribution and compact per-checkpoint progress lines.

crates/agent-drift-analyzer/tests/end_to_end.rs
  full bundle rerun produces deterministic v0.6 checkpoints.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  dedicated real-rollout/frozen acceptance wall for progress semantics.

crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
  legacy R2 acceptance wall that should remain stable unless intentionally widened.
```

Sentinel tests:

```text
crates/agent-drift-sentinel/tests/replay_input.rs
  v0.6 support and requiredness.

crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
  v0.6 live compatibility and requiredness.

crates/agent-drift-sentinel/tests/operator_surface.rs
  compact Progress rendering.

crates/agent-drift-sentinel/tests/live_end_to_end.rs
  replay/live parity for v0.6 checkpoint surfaces.
```

## Acceptance Metrics

R5 should report or manually verify:

1. synthetic exact-match rate for `(status, dimension)`,
2. evidence completeness for non-`insufficient_evidence` statuses,
3. confidence cap compliance for delegated/ambiguous cases,
4. adjacent-checkpoint stability when no comparable new attempt appears,
5. replay/live v0.6 compatibility parity,
6. legacy v0.2-v0.5 compatibility preservation,
7. real-rollout rubric agreement for annotated decision points.

## Review Protocol

Before R5 is called review-clean, the packet should show:

1. the DESIGN docs match the implemented public DTO and internal seams,
2. the spec/plan/tasks docs reflect the final file touch set,
3. schema `v0.6` requiredness is enforced in analyzer serde and sentinel replay/live validation,
4. analyzer summary renders progress distribution and checkpoint-local progress,
5. operator surface renders a compact `Progress:` line,
6. synthetic fixtures cover all required matrix rows or explicitly defer a row with rationale,
7. at least one bounded real-rollout acceptance pass was performed in `R5-7`, or—if the family has
   not reached `R5-7` yet—the docs explicitly preserve that deferral and do not overclaim semantic
   acceptance earlier,
8. no scorer behavior was retuned.

## Non-Goals

This validation protocol does not require:

1. dataset-scale accuracy claims,
2. learned reward-model evaluation,
3. external benchmark integration,
4. full DoVer intervention execution,
5. full R7 parent/child trace stitching.

## Locked Decisions After Packet R5-0

1. The real-rollout acceptance wall ships in `R5-7`; there is no `R5.5`, and semantic acceptance
   is not pulled forward into `R5-6`.
2. `progress_debug.jsonl` is recommended only; it is not required for review-clean status.
3. The first dedicated progress corpus includes a delegated parent opaque guardrail case, but does
   not attempt positive child-progress semantics before R7.
