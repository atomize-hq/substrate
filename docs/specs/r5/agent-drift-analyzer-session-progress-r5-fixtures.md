# Fixture Manifest: Agent Drift Analyzer Session Progress R5

Status: draft fixture authority for R5.

## Purpose

This document defines the semantic fixture matrix for R5. Each fixture should prove a specific
progress boundary, not merely that `session_progress` exists.

The implementation should update the `Implementation fixture location` fields as concrete tests or
committed fixtures land.

## Labeling Rules

Each fixture expectation must include:

1. expected archetype,
2. expected progress dimension,
3. expected progress status,
4. expected confidence floor or ceiling,
5. decisive supporting evidence,
6. required counter-evidence when ambiguous/negative/delegated,
7. why nearby statuses lose.

Public labels use serialized snake_case values:

```text
status:
  advancing
  mixed
  stalled
  regressing
  insufficient_evidence

dimension:
  troubleshooting_frontier
  planning_convergence
  implementation_verification_wall
  verification_closeout_narrowing
  parent_visible_orchestration
```

## Required Fixture Matrix

## Fixture: r5_troubleshooting_frontier_advanced_compile_to_test

- Archetype: `troubleshooting`
- Expected dimension: `troubleshooting_frontier`
- Expected status: `advancing`
- Expected confidence: `high` if diagnostic parser gets same/overlapping target; otherwise `medium`
- Setup:
  1. `cargo test parser::roundtrip` fails before target due to compile/type error in `src/parser.rs`.
  2. `apply_patch` edits `src/parser.rs` or related test source.
  3. `cargo test parser::roundtrip` reaches a focused test/assertion failure.
- Decisive evidence:
  - `FailureFrontierAdvanced`
  - `FailingScopeEdited`
- Counter-evidence:
  - none required unless parser confidence is low
- Why competing statuses lose:
  - not `stalled`: failure class changed along a comparable target frontier
  - not `regressing`: later failure is deeper/later than compile failure
  - not `insufficient_evidence`: target was exercised by the second attempt
- Implementation fixture location: `TBD: crates/agent-drift-analyzer/tests/checkpoints.rs`

## Fixture: r5_troubleshooting_failure_count_reduced

- Archetype: `troubleshooting`
- Expected dimension: `troubleshooting_frontier`
- Expected status: `advancing`
- Expected confidence: `medium` or `high`
- Setup:
  1. first comparable test attempt reports many failing tests,
  2. intervening edit overlaps failing test/source scope,
  3. later comparable test attempt reports fewer failing tests.
- Decisive evidence:
  - `FailureCountReduced`
  - `FailingScopeEdited`
- Counter-evidence:
  - include parser-confidence counter if fail-count extraction is partial
- Why competing statuses lose:
  - not `stalled`: quantitative failing frontier improved
  - not `regressing`: failure count did not grow
- Implementation fixture location: `TBD`

## Fixture: r5_troubleshooting_same_signature_no_edit

- Archetype: `troubleshooting`
- Expected dimension: `troubleshooting_frontier`
- Expected status: `stalled`
- Expected confidence: `high` if exact normalized signature repeats; otherwise `medium`
- Setup:
  1. same verification command fails,
  2. no overlapping edit occurs,
  3. same normalized diagnostic signature repeats.
- Decisive evidence:
  - `FailureSignatureRepeated`
  - `FailingScopeUnchanged`
- Counter-evidence:
  - none required unless unrelated edits occurred
- Why competing statuses lose:
  - not `advancing`: no frontier movement or edit overlap
  - not `regressing`: no prior cleaner state was broken
- Implementation fixture location: `TBD`

## Fixture: r5_troubleshooting_previous_clean_broken

- Archetype: `troubleshooting` or `autonomous_implementation`
- Expected dimension: `troubleshooting_frontier` or `implementation_verification_wall` based on R4
  archetype in fixture
- Expected status: `regressing`
- Expected confidence: `medium` or `high`
- Setup:
  1. focused verifier is clean,
  2. later edits touch overlapping source/test scope,
  3. same focused verifier fails or earlier compile/build failure appears.
- Decisive evidence:
  - `PreviouslyCleanScopeBroken`
  - `FailureCountIncreased` when available
- Counter-evidence:
  - include any positive signal if another unrelated verifier passed
- Why competing statuses lose:
  - not `stalled`: state moved backward from clean to failed
  - not `advancing`: a previous best frontier was lost
- Implementation fixture location: `TBD`

## Fixture: r5_planning_candidate_set_narrows_to_spec

- Archetype: `planning`
- Expected dimension: `planning_convergence`
- Expected status: `advancing`
- Expected confidence: `medium` max for first R5 landing unless multiple structural signals exist
- Setup:
  1. broad read/search across several docs or source paths,
  2. working set narrows,
  3. `DESIGN-*`, `*-spec.md`, `*-plan.md`, or `*-tasks.md` artifact is created/refined.
- Decisive evidence:
  - `CandidateSetNarrowed`
  - `PlanArtifactCreated` or `PlanArtifactRefined`
  - `WorkingSetConcentrated`
- Counter-evidence:
  - none required unless broad searches continue after artifact creation
- Why competing statuses lose:
  - not `stalled`: artifact and narrowed set show structural convergence
  - not `implementation_verification_wall`: no source/test implementation loop dominates
- Implementation fixture location: `TBD`

## Fixture: r5_planning_broad_scan_meanders

- Archetype: `planning`
- Expected dimension: `planning_convergence`
- Expected status: `stalled` or low-confidence `mixed`
- Expected confidence: `low` or `medium`
- Setup:
  1. repeated broad `rg`/`find`/`sed`/`cat` style scans,
  2. working set expands or remains diffuse,
  3. no plan/spec/tasks/design artifact is created or refined.
- Decisive evidence:
  - `CandidateSetExpanded`
  - `WorkingSetDiffused`
- Counter-evidence:
  - any narrowed truth artifact or candidate should appear as counter-evidence
- Why competing statuses lose:
  - not `advancing`: no convergence artifact or narrowing
  - not `insufficient_evidence`: repeated broad scans are enough evidence of meander/stall
- Implementation fixture location: `TBD`

## Fixture: r5_implementation_wall_advances

- Archetype: `autonomous_implementation`
- Expected dimension: `implementation_verification_wall`
- Expected status: `advancing`
- Expected confidence: `medium` or `high`
- Setup:
  1. source edit in stable working set,
  2. related test/golden edit or focused verification,
  3. verifier moves from compile/build failure to focused test failure or clean focused proof,
  4. optional broader verifier follows.
- Decisive evidence:
  - `WorkingSetConcentrated`
  - `FailureFrontierAdvanced` or `VerificationClean`
  - `VerificationScopeBroadened` when broader proof follows focused proof
- Counter-evidence:
  - none required unless unrelated churn appears
- Why competing statuses lose:
  - not `stalled`: verifier moved forward
  - not `verification_closeout_narrowing`: implementation edits still dominate the checkpoint
- Implementation fixture location: `TBD`

## Fixture: r5_implementation_same_failure_unrelated_edits

- Archetype: `autonomous_implementation`
- Expected dimension: `implementation_verification_wall`
- Expected status: `stalled` or `mixed`
- Expected confidence: `medium`
- Setup:
  1. verifier fails with stable signature,
  2. intervening edits are unrelated to failing path/test/symbol,
  3. verifier repeats same failure.
- Decisive evidence:
  - `FailureSignatureRepeated`
  - `FailingScopeUnchanged`
- Counter-evidence:
  - unrelated edit activity should be recorded as counter-evidence or mixed context
- Why competing statuses lose:
  - not `advancing`: edits did not overlap the failing scope and diagnostics did not improve
  - not `regressing`: no previous clean/later frontier was broken
- Implementation fixture location: `TBD`

## Fixture: r5_closeout_clean_proof_no_source_churn

- Archetype: `verification_closeout`
- Expected dimension: `verification_closeout_narrowing`
- Expected status: `advancing`
- Expected confidence: `high` if proof is direct and source churn is absent
- Setup:
  1. little/no source edit activity,
  2. proof-oriented verifier(s) pass,
  3. summary/handoff/spec fixture artifact is refined,
  4. residual scope shrinks or expected next step moves toward completion.
- Decisive evidence:
  - `VerificationClean`
  - `ResidualScopeShrank`
  - `PlanArtifactRefined` when summary/handoff artifact is written
- Counter-evidence:
  - none required unless new source edit is present
- Why competing statuses lose:
  - not `implementation_verification_wall`: no meaningful source churn
  - not `stalled`: clean proof and residual narrowing add new information
- Implementation fixture location: `TBD`

## Fixture: r5_closeout_reopens_source_churn

- Archetype: `verification_closeout`
- Expected dimension: `verification_closeout_narrowing`
- Expected status: `regressing` when prior clean proof is broken, otherwise `mixed`
- Expected confidence: `medium`
- Setup:
  1. closeout checkpoint previously had clean proof,
  2. new source edit reopens scope,
  3. verifier fails or residual target list expands.
- Decisive evidence:
  - `ResidualScopeReopened`
  - `PreviouslyCleanScopeBroken` when applicable
  - `WorkingSetDiffused` if scope expands
- Counter-evidence:
  - clean unrelated proof should be counter-evidence, not decisive support
- Why competing statuses lose:
  - not `advancing`: closeout scope reopened
  - not `stalled`: the state changed materially
- Implementation fixture location: `TBD`

## Fixture: r5_sparse_no_comparable_attempt

- Archetype: any
- Expected dimension: default from archetype unless delegation overrides
- Expected status: `insufficient_evidence`
- Expected confidence: `low`
- Setup:
  1. checkpoint has R4 archetype,
  2. no comparable verification, artifact, narrowing, or edit-overlap evidence exists.
- Decisive evidence:
  - `TargetNotExercised` when a verifier was intended but did not exercise target
  - otherwise no signal or weak limiting signal only
- Counter-evidence:
  - evidence reason should name sparse evidence rather than forcing progress
- Why competing statuses lose:
  - none of `advancing`, `stalled`, or `regressing` has enough comparable evidence
- Implementation fixture location: `TBD`

## Fixture: r5_delegated_parent_opaque

- Archetype: any R4 label, usually `planning` or `troubleshooting`
- Expected dimension: `parent_visible_orchestration` or archetype default with limiting signal
- Expected status: `insufficient_evidence` or low-confidence `mixed/stalled` for parent-visible
  orchestration only
- Expected confidence: `low`
- Setup:
  1. parent shows `spawn_agent`, `wait_agent`, or `close_agent`,
  2. child work is opaque,
  3. no visible child diagnostics or implementation output exists.
- Decisive evidence:
  - `DelegationVisibilityLimited`
- Counter-evidence:
  - child-opaque evidence required
- Why competing statuses lose:
  - no child progress claim is allowed
  - parent waiting is not child implementation stall
- Implementation fixture location: `TBD`

## Schema / Compatibility Fixtures

## Fixture: r5_legacy_v0_5_still_loads

- Expected: v0.5 checkpoint without `session_progress` remains replay/live compatible.
- Required tests:
  - analyzer DTO legacy deserialization
  - sentinel replay load
  - sentinel live compatibility
- Implementation fixture location: `TBD`

## Fixture: r5_v0_6_requires_progress

- Expected: v0.6 checkpoint without `session_progress` fails closed.
- Required tests:
  - analyzer serde requiredness
  - sentinel replay contract gap
  - sentinel live contract gap
- Implementation fixture location: `TBD`

## Fixture: r5_replay_live_v0_6_parity

- Expected: same v0.6 checkpoint renders same compact progress line in replay and live.
- Required tests:
  - operator surface replay block
  - live end-to-end block
- Implementation fixture location: `TBD`

## Acceptance Wall Notes

The R5 implementation should not claim real semantic acceptance until at least a small committed
corpus exists. Synthetic fixtures can prove deterministic rules, but a bounded real-rollout or
bundle-shaped corpus should prove the labels remain honest on realistic traces.

Recommended first real/bundle-shaped cases:

1. non-subagent recovered sticky session from the R1E/R2 family,
2. non-subagent active failure with repeated non-zero verification,
3. planning/spec-development session,
4. closeout/proof session,
5. delegated parent opaque session used only for guardrail proof.

## Deferred Fixture Ideas

These are useful but not required for the first R5 landing:

1. language-server style diagnostics,
2. exact symbol overlap from AST selectors,
3. multi-session parent/child stitched progress,
4. learned/LLM milestone extraction,
5. reference-patch/process-template alignment.
