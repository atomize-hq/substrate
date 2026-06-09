# Design: R5 Archetype Progress Rules

Status: draft canonical design input for `R5`.

## Why This Doc Exists

`session_archetype` tells the analyzer what kind of work the checkpoint is part of. R5 must then
measure progress using rules appropriate to that archetype. The same visible behavior can mean
opposite things in different archetypes: repeated failing verification may be expected debugging
motion in troubleshooting, but scope reopening in verification closeout.

This doc defines the archetype-specific progress semantics that the R5 implementation should use.

## Core Rule

Direct machine-observed progress beats contextual intent evidence.

R4's internal intent buckets and public `session_archetype` are useful context, but R5 should prefer
observable deltas:

1. diagnostic frontier movement,
2. fail-count deltas,
3. clean verification after prior failure,
4. edit overlap with failing scope,
5. working-set concentration/diffusion,
6. artifact creation/refinement,
7. residual scope shrinking or reopening.

## Status Vocabulary

Use one status per checkpoint:

| Status | Meaning |
|---|---|
| `advancing` | The relevant frontier moved forward or narrowed with supporting evidence. |
| `mixed` | Positive and negative signals both matter, or the frontier is partly comparable but noisy. |
| `stalled` | Comparable attempts repeat without material frontier movement. |
| `regressing` | A previously cleaner/later frontier deteriorated or scope reopened/destructively changed. |
| `insufficient_evidence` | The trace does not provide enough comparable or exercised evidence to make a progress claim. |

## Troubleshooting Frontier

Default dimension for `SessionArchetypeLabel::Troubleshooting`.

### Advancing Signals

1. Failure class advances on the same or overlapping target:
   - dependency/setup -> compile/build
   - compile/build -> test discovery/execution
   - test execution -> assertion/golden/replay mismatch
   - failure -> clean focused verification
2. Failing count decreases:
   - `8 failed` -> `2 failed`
   - multiple compile errors -> fewer compile errors on same scope
3. Intervening edits overlap failing path/symbol/test scope.
4. A previously blocked verifier reaches the target:
   - `cargo test foo` blocked by compile error -> `foo` test actually runs and fails.
5. Repeated command is still failing but the diagnostic payload is no longer the same and the
   changed signature is frontier-related.

### Stalled Signals

1. Exact or strong-fuzzy same diagnostic signature repeats.
2. Same verifier command repeats with same failure and no overlapping edits.
3. Fail count remains stable across comparable attempts.
4. Diagnostic changes are volatile-only: line numbers, durations, temp paths, timestamps.
5. Agent cycles through read/verify without narrowing target or editing failing scope.

### Regressing Signals

1. Previously clean verifier fails again on overlapping scope.
2. Later failure class moves earlier in the pipeline:
   - assertion failure -> compile failure
   - focused test failure -> dependency/setup failure due to unrelated changes
3. Failing count increases after edits.
4. Working set diffuses into unrelated areas while the original failure remains unresolved.
5. Previously narrowed target broadens because earlier progress was destroyed.

### Insufficient Evidence

1. No verification attempts.
2. Verification output cannot be paired to a command.
3. Failure parser can only produce `Unknown` and no stable repeated signature exists.
4. Delegated child work is opaque and parent only waits or orchestrates.

### Expected Signals

Use these public signal codes:

```text
FailureFrontierAdvanced
FailureSignatureRepeated
FailureCountReduced
FailureCountIncreased
FailingScopeEdited
FailingScopeUnchanged
VerificationClean
PreviouslyCleanScopeBroken
TargetNotExercised
```

## Planning Convergence

Default dimension for `SessionArchetypeLabel::Planning`.

Planning is structurally harder to infer than troubleshooting. The first R5 implementation must be
conservative and mostly low/medium confidence.

### Advancing Signals

1. Candidate set narrows:
   - fewer distinct working-set paths after broad read/search,
   - fewer tools or command families in the recent prefix,
   - search terms become more specific.
2. Objective sharpens:
   - task frame stabilizes after earlier transitions,
   - expected next step becomes more concrete.
3. Plan/spec/todo/design artifact is created or refined:
   - `docs/specs/...-spec.md`
   - `...-plan.md`
   - `...-tasks.md`
   - `DESIGN-*.md`
4. Open questions shrink or become assigned to explicit follow-up decisions.
5. Truth artifacts become focused rather than expanding unchecked.

### Stalled Signals

1. Repeated broad searches/reads without artifact creation or narrowed working set.
2. Candidate set expands for multiple checkpoints without a convergence artifact.
3. Task-frame transitions remain high and no stable objective emerges.
4. Same objective is restated repeatedly without a plan/spec/tasks artifact.

### Regressing Signals

1. Previously narrowed plan reopens into unrelated scope without user steer.
2. Newly created plan/spec is contradicted by later broad wandering.
3. Working set diffuses after an apparently settled plan.
4. The checkpoint returns to old candidate branches that were already ruled out.

### Insufficient Evidence

1. Only one directive row and no material activity.
2. Read-only context is too sparse to decide convergence.
3. Planning evidence is mostly commentary with no structural signal.

### Expected Signals

```text
PlanArtifactCreated
PlanArtifactRefined
CandidateSetNarrowed
CandidateSetExpanded
WorkingSetConcentrated
WorkingSetDiffused
```

## Implementation Verification Wall

Default dimension for `SessionArchetypeLabel::AutonomousImplementation`.

Implementation progress is not "more edits." It is movement against the verification wall while the
working set stays aligned to the objective.

### Advancing Signals

1. Source edits remain concentrated in the active working set.
2. Related tests/goldens/specs move with the changed source files.
3. Verifier target advances:
   - compile/check passes after source edits,
   - focused test reaches later failure,
   - focused verifier passes and broader verifier starts.
4. Diagnostic severity drops on the same scope.
5. Expected next step narrows from broad implementation to a concrete verification/fix target.

### Stalled Signals

1. Same verifier failure repeats after edit batches, with no overlap to failing scope.
2. Edits continue without verification for too long in an autonomous turn.
3. Source churn occurs across unrelated paths while the original verifier remains unchanged.
4. Dependency/config changes appear without local proof.

### Regressing Signals

1. Previously passing compile/check breaks after source edits.
2. Previously clean focused tests fail after unrelated edits.
3. Working set diffuses into unrelated implementation branches.
4. Test/golden changes mask source failure without proof-oriented verification.

### Insufficient Evidence

1. Only edit commands exist; no verifier or comparable diagnostic.
2. Verifier output is not paired to the edit scope.
3. Implementation is delegated to an opaque child.

### Expected Signals

```text
WorkingSetConcentrated
WorkingSetDiffused
VerificationScopeBroadened
VerificationScopeNarrowed
VerificationClean
FailureFrontierAdvanced
FailureSignatureRepeated
PreviouslyCleanScopeBroken
```

## Verification Closeout Narrowing

Default dimension for `SessionArchetypeLabel::VerificationCloseout`.

Closeout progress is proof and residual-scope narrowing. New source churn is counter-evidence.

### Advancing Signals

1. Little or no new source churn.
2. Proof commands run against the intended residual scope.
3. Focused proof passes, then broader smoke/replay/full-suite proof passes.
4. Summary, handoff, fixture, or acceptance artifact is created/refined after proof.
5. Residual target list shrinks.
6. Drift evidence recovers or becomes historical while proof remains clean.

### Stalled Signals

1. Same proof command repeats without new information.
2. Proof scope does not narrow or broaden meaningfully.
3. The checkpoint keeps summarizing without running required proof.
4. Residual failures remain identical.

### Regressing Signals

1. New source edits reopen implementation after closeout should be narrowing.
2. A previously clean proof fails.
3. Broad proof discovers unrelated failures caused by recent edits.
4. The session shifts back into troubleshooting without recognizing the closeout reset.

### Insufficient Evidence

1. Closeout-shaped prose with no proof commands.
2. Only summary artifact exists and no verification output is visible.
3. Delegated child proof is opaque.

### Expected Signals

```text
VerificationClean
VerificationScopeBroadened
VerificationScopeNarrowed
ResidualScopeShrank
ResidualScopeReopened
PreviouslyCleanScopeBroken
PlanArtifactRefined
```

## Parent Visible Orchestration

R5 should use `ParentVisibleOrchestration` only when delegation prevents honest archetype-native
progress claims.

### Advancing Signals

1. Parent spawns child with clear instructions.
2. Parent receives a visible child result link or summary.
3. Parent updates plan/handoff based on visible child output.
4. Parent closes child work with a visible result.

### Stalled Signals

1. Parent repeatedly waits without child-visible progress.
2. Parent retries orchestration commands with no new evidence.
3. Parent switches child instructions without visible output.

### Insufficient Evidence

1. Child work is opaque and parent only waits.
2. Separate child rollout is mentioned but not visible in this checkpoint.

### Required Limiting Signal

Opaque or partial cases should include:

```text
DelegationVisibilityLimited
```

## Status Precedence

When multiple signals are present:

1. `Regressing` wins over `Advancing` when direct machine evidence shows a previously clean/later
   frontier became worse.
2. `Advancing` wins over `Stalled` when a direct comparable diagnostic frontier moved forward.
3. `Mixed` wins when progress occurs in one dimension but regression occurs in another and neither
   dominates.
4. `InsufficientEvidence` wins when the trace does not contain comparable or exercised evidence.

## Evidence Language Examples

Good evidence reasons:

```text
"compile failure advanced to focused test failure on overlapping scope"
"same diagnostic signature repeated without an overlapping failing-scope edit"
"plan artifact was created after broad search narrowed to docs/specs"
"new source churn reopened closeout scope after proof-oriented checkpoint"
"delegating-parent with opaque child work limited progress certainty"
```

Avoid causal overclaims:

```text
"this patch fixed the compile error"
"child agent failed the task"
"the plan is correct"
"verification proves no remaining bugs"
```

## Non-Goals

This design does not:

1. define final drift-score effects,
2. require a learned stage classifier,
3. require perfect semantic text understanding for planning,
4. treat parent waiting as child progress,
5. eliminate R6 scorer work.

## Open Questions

1. Should `PlanningConvergence` support `High` confidence in the first landing, or cap planning at
   `Medium` until a real-rollout annotation wall exists?
2. Should closeout source churn immediately force `Regressing`, or first produce `Mixed` unless a
   clean proof was previously observed?
3. How much repeated no-verification implementation activity should be enough to call
   implementation `Stalled` versus `InsufficientEvidence`?
