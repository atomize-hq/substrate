# Fixture Manifest: Agent Drift Analyzer Session Progress R5

Status: Packet R5-0 fixture-manifest direction locked on 2026-06-09; concrete tests land later.

## Purpose

This document defines the semantic fixture matrix for R5. Each fixture should prove a specific
progress boundary, not merely that `session_progress` exists.

The implementation should update the `Implementation fixture location` fields as concrete tests or
committed fixtures land.

## Locked Implementation Direction

Packet `R5-0` locks the fixture-manifest direction as follows:

1. Synthetic analyzer semantic cases first land in
   `crates/agent-drift-analyzer/tests/checkpoints.rs` across `R5-1` through `R5-4`.
2. Analyzer summary/export rendering cases land in
   `crates/agent-drift-analyzer/tests/export_bundle.rs` and
   `crates/agent-drift-analyzer/tests/end_to_end.rs` during `R5-5`.
3. Sentinel compatibility and replay/live operator-parity cases land in
   `crates/agent-drift-sentinel/tests/replay_input.rs`,
   `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`,
   `crates/agent-drift-sentinel/tests/operator_surface.rs`, and
   `crates/agent-drift-sentinel/tests/live_end_to_end.rs` during `R5-6`.
4. Bounded semantic acceptance lands only in
   `crates/agent-drift-analyzer/tests/progress_acceptance.rs` plus
   `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**` during `R5-7`, and
   claiming it requires at least one annotated real-rollout case in that corpus.
5. The frozen R2 acceptance wall under
   `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs` and
   `crates/agent-drift-analyzer/tests/fixtures/acceptance/**` stays stable unless `R5-7`
   explicitly widens it.

## Labeling Rules

Each fixture expectation must include:

1. expected archetype,
2. expected progress dimension,
3. expected progress status,
4. expected confidence floor or ceiling,
5. decisive supporting evidence,
6. required limiting/counter-evidence for delegated cases, and counter-evidence for
   ambiguous/negative cases when meaningful contradictory or limiting evidence exists,
7. comparable attempts,
8. window/reset expectation,
9. why nearby statuses lose.

Each required row below therefore makes the comparability contract explicit. Schema/compatibility
rows still record `Comparable attempts` and `Window/reset expectation`, but those fields describe
payload/surface comparability rather than runtime progress-window ids. They are an explicit
template carve-out: semantic progress rows use the full archetype/dimension/status/confidence
shape, while schema/compatibility rows may instead use `Expected`, `Required tests`,
`Comparable attempts`, `Window/reset expectation`, and `Implementation fixture location` because
they validate payload/surface behavior rather than runtime progress labels.

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
- Comparable attempts:
  - compare the same failing command or equivalent focused verifier on the same parser target
    before/after an overlapping edit; unrelated commands or a different target file do not compare
- Window/reset expectation:
  - stays in one troubleshooting window while `parser::roundtrip` / `src/parser.rs` remains the
    target frontier; reset if the verifier target changes materially or a later clean proof closes
    this scope and subsequent work reopens a different target
- Why competing statuses lose:
  - not `stalled`: failure class changed along a comparable target frontier
  - not `regressing`: later failure is deeper/later than compile failure
  - not `insufficient_evidence`: target was exercised by the second attempt
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare the same package/filter test command, or an equivalent deterministic suite over the
    same failing scope, before and after an overlapping edit; different suites or filters do not
    compare
- Window/reset expectation:
  - stays in one troubleshooting window while the command/filter and failing scope stay materially
    the same; reset if the session switches to a different suite/target or a new objective rebuilds
    the failure set from another working set
- Why competing statuses lose:
  - not `stalled`: quantitative failing frontier improved
  - not `regressing`: failure count did not grow
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare repeats of the same verifier normalized to the same target signature when no
    overlapping edit intervenes; other commands or a different signature do not compare
- Window/reset expectation:
  - stays in one troubleshooting window while the same verifier/target repeats without overlapping
    edits; reset if an overlapping fix edit lands or the verifier target changes materially
- Why competing statuses lose:
  - not `advancing`: no frontier movement or edit overlap
  - not `regressing`: no prior cleaner state was broken
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare an earlier clean run and a later failing rerun of the same focused verifier on the same
    path/test scope after an overlapping edit; unrelated verifiers are only counter-evidence
- Window/reset expectation:
  - stays in one troubleshooting or implementation window while the same previously clean scope is
    being rechecked; reset if the verifier target changes materially or the session moves to a new
    objective/worktree
- Why competing statuses lose:
  - not `stalled`: state moved backward from clean to failed
  - not `advancing`: a previous best frontier was lost
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare planning checkpoints that pursue the same stated objective and truth-artifact family
    while the working set narrows from broad scan to specific `DESIGN-*`, spec, plan, or tasks
    files; implementation/test loops do not compare
- Window/reset expectation:
  - stays in one planning window while the objective and narrowed artifact family remain stable;
    reset if the truth artifact/worktree changes materially or the session pivots into a different
    implementation objective
- Why competing statuses lose:
  - not `stalled`: artifact and narrowed set show structural convergence
  - not `implementation_verification_wall`: no source/test implementation loop dominates
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare repeated broad planning scans for the same objective before any narrowing artifact is
    produced; once a concrete artifact becomes the anchor, the session exits this meander case
- Window/reset expectation:
  - stays in one planning window while the working set remains diffuse around the same objective;
    reset if a concrete spec/plan/tasks/design artifact becomes the target or the session switches
    archetypes
- Why competing statuses lose:
  - not `advancing`: no convergence artifact or narrowing
  - not `insufficient_evidence`: repeated broad scans are enough evidence of meander/stall
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare verification attempts against the same feature slice, symbol, or test scope after
    overlapping source edits; broader proof is comparable only when it follows the same focused
    target frontier
- Window/reset expectation:
  - stays in one implementation-verification window while the working set and target verifier remain
    materially the same; reset if the session changes slices/objectives or a different working set
    becomes dominant
- Why competing statuses lose:
  - not `stalled`: verifier moved forward
  - not `verification_closeout_narrowing`: implementation edits still dominate the checkpoint
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare repeats of the same verifier on the same failing path/test/symbol across unrelated
    edits; once an overlapping fix lands, later attempts no longer belong to this row
- Window/reset expectation:
  - stays in one implementation-verification window while unrelated edits continue and the verifier
    target stays fixed; reset if an overlapping fix edit lands or the command target changes
- Why competing statuses lose:
  - not `advancing`: edits did not overlap the failing scope and diagnostics did not improve
  - not `regressing`: no previous clean/later frontier was broken
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare proof attempts in the same closeout phase for the same residual checklist/target list
    once source churn has stopped; summary or handoff artifacts compare only when they describe that
    same residual scope
- Window/reset expectation:
  - stays in one closeout window while source files stay stable and verification continues to
    retire the same residual scope; reset if source edits resume or the proof target changes
    materially
- Why competing statuses lose:
  - not `implementation_verification_wall`: no meaningful source churn
  - not `stalled`: clean proof and residual narrowing add new information
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

## Fixture: r5_closeout_scope_narrows_to_residual

- Archetype: `verification_closeout`
- Expected dimension: `verification_closeout_narrowing`
- Expected status: `advancing`
- Expected confidence: `medium` or `high`
- Setup:
  1. source churn has stopped,
  2. a broader proof/checklist initially covers multiple verification surfaces or commands,
  3. later closeout narrows to a smaller residual command/target list on the same work product,
  4. the narrowed residual proof runs clean or the refined artifact records only that smaller
     residual scope.
- Decisive evidence:
  - `VerificationScopeNarrowed`
  - `ResidualScopeShrank`
  - `VerificationClean` when the narrowed residual proof passes
  - `PlanArtifactRefined` when handoff/checklist text records the focused residual
- Counter-evidence:
  - any new source edit, broadened unresolved failures, or reopened implementation work counts
    against this closeout claim
- Comparable attempts:
  - compare a broader closeout proof/checklist and a later focused residual proof/checklist when
    both address the same work product and the later step is a strict subset of the earlier proof
    scope
- Window/reset expectation:
  - stays in one closeout window while source files remain stable and the proof sequence keeps
    narrowing the same residual target list; reset if new source edits reopen implementation or the
    closeout objective/worktree changes materially
- Why competing statuses lose:
  - not `stalled`: proof scope became strictly smaller and more specific
  - not `implementation_verification_wall`: closeout proof, not source churn, dominates the
    checkpoint
  - not `insufficient_evidence`: the narrowed residual scope is explicit and comparable
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - compare a prior clean closeout proof and a later failing/reopened proof on the same residual
    target list after new source edits; unrelated proof commands are only counter-evidence
- Window/reset expectation:
  - stays in one closeout window only while the reopened work still concerns the same residual
    target list; reset if a new objective/worktree replaces that scope or the reopened work becomes
    a distinct implementation slice
- Why competing statuses lose:
  - not `advancing`: closeout scope reopened
  - not `stalled`: the state changed materially
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

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
- Comparable attempts:
  - comparable attempts are intentionally absent here; any single verifier/artifact that never
    returns to the same scope stays non-comparable and belongs in this sparse row
- Window/reset expectation:
  - this sparse case persists only until a second same-scope attempt or artifact appears; once
    comparable evidence exists, the fixture should move to a concrete archetype-specific row
- Why competing statuses lose:
  - none of `advancing`, `stalled`, or `regressing` has enough comparable evidence
- Implementation fixture location: `Synthetic: crates/agent-drift-analyzer/tests/checkpoints.rs; semantic re-proof subset: crates/agent-drift-analyzer/tests/progress_acceptance.rs`

## Fixture: r5_delegated_parent_opaque

- Archetype: any R4 label, usually `planning` or `troubleshooting`
- Expected dimension: `parent_visible_orchestration`
- Expected status: `insufficient_evidence`, or low-confidence `stalled`/`mixed` when direct
  parent-visible orchestration evidence exists
- Expected confidence: `low`
- Setup:
  1. parent shows `spawn_agent`, `wait_agent`, or `close_agent`,
  2. child work is opaque,
  3. no visible child diagnostics or implementation output exists.
- Decisive evidence:
  - `DelegationVisibilityLimited`
- Counter-evidence:
  - include child-opaque limiting evidence; add contradictory counter-evidence only when it exists
- Comparable attempts:
  - compare only parent-visible orchestration checkpoints for the same child objective while child
    outputs remain opaque; once child diagnostics/results become visible, the case exits this row
- Window/reset expectation:
  - stays in one parent-visible orchestration window while the same delegated objective remains
    opaque; reset if child visibility improves or the parent switches delegated objectives
- Why competing statuses lose:
  - no child progress claim is allowed
  - parent waiting is not child implementation stall
- Implementation fixture location: `Analyzer: crates/agent-drift-analyzer/tests/checkpoints.rs; replay: crates/agent-drift-sentinel/tests/replay_input.rs; live: crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

## Schema / Compatibility Fixtures

## Fixture: r5_legacy_v0_5_still_loads

- Expected: v0.5 checkpoint without `session_progress` remains replay/live compatible.
- Required tests:
  - analyzer DTO legacy deserialization
  - sentinel replay load
  - sentinel live compatibility
- Comparable attempts:
  - compare analyzer, replay, and live loading/rendering of the same serialized v0.5 payload;
    different schema versions or payload shapes are separate fixtures
- Window/reset expectation:
  - no runtime progress-window semantics apply; treat any `schema_version` or serialized payload
    mutation as a new case boundary
- Implementation fixture location: `Analyzer: crates/agent-drift-analyzer/tests/checkpoints.rs; replay: crates/agent-drift-sentinel/tests/replay_input.rs; live: crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

## Fixture: r5_v0_6_requires_progress

- Expected: v0.6 checkpoint without `session_progress` fails closed.
- Required tests:
  - analyzer serde requiredness
  - sentinel replay contract gap
  - sentinel live contract gap
- Comparable attempts:
  - compare analyzer, replay, and live handling of the same invalid v0.6 payload missing
    `session_progress`; payloads with different omissions or versions are distinct cases
- Window/reset expectation:
  - no runtime progress-window semantics apply; reset the case whenever schema version or missing
    field shape changes
- Implementation fixture location: `Analyzer: crates/agent-drift-analyzer/tests/checkpoints.rs; replay: crates/agent-drift-sentinel/tests/replay_input.rs; live: crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

## Fixture: r5_replay_live_v0_6_parity

- Expected: same v0.6 checkpoint renders same compact progress line in replay and live.
- Required tests:
  - operator surface replay block
  - live end-to-end block
- Comparable attempts:
  - compare replay and live rendering of the exact same v0.6 checkpoint payload and compact summary
    expectation; other payloads or formatting modes are separate fixtures
- Window/reset expectation:
  - no runtime progress-window semantics apply; reset whenever the payload, rendering contract, or
    expected compact line changes
- Implementation fixture location: `Replay surface: crates/agent-drift-sentinel/tests/operator_surface.rs; live parity: crates/agent-drift-sentinel/tests/live_end_to_end.rs`

## Acceptance Wall Notes

The R5 implementation should not claim real semantic acceptance until at least a small committed
corpus exists. Synthetic fixtures can prove deterministic rules, but bounded semantic acceptance
still requires at least one annotated real-rollout case; bundle-shaped cases can only provide
supporting coverage to show the labels remain honest on realistic traces.

Packet `R5-0` locks that this bounded semantic corpus belongs to `R5-7`; earlier packets can prove
deterministic behavior and compatibility, but must not overclaim semantic acceptance.

Recommended first annotated real-rollout / supporting bundle-shaped cases:

1. non-subagent recovered sticky session from the R1E/R2 family,
2. non-subagent active failure with repeated non-zero verification,
3. planning/spec-development session,
4. closeout/proof session that narrows from broad verification to a focused residual proof,
5. delegated parent opaque session used only for guardrail proof.

## Deferred Fixture Ideas

These are useful but not required for the first R5 landing:

1. language-server style diagnostics,
2. exact symbol overlap from AST selectors,
3. multi-session parent/child stitched progress,
4. learned/LLM milestone extraction,
5. reference-patch/process-template alignment.
