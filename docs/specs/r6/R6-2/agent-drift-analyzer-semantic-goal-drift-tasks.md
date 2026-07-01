# Tasks: Agent Drift Analyzer Semantic Goal Drift From Kickoff/Plan/Docs (R6-2)

Status: task ledger created on 2026-06-27 from the `R6-2` SPEC/PLAN in this directory. `R6-2.1.1`
landed on 2026-07-01; remaining tasks stay open. Sequenced after `R6-1`. This ledger is the closeout
record as tasks land.

Packet prerequisite rule: this packet names `R5.75-1` (structured sidecar + `comparison_key_from_structured`)
and `R6-1` (dead_end_thrash cutover) as landed. Verify both in live code/tests before editing. If a named
prerequisite is missing, stop and report it instead of compensating inside this packet.

## R6-2.0: Docs Lock

- [ ] Task R6-2.0.1: Lock the SPEC/PLAN/TASKS family and packet-prompts artifact.
  - Acceptance: `docs/specs/r6/R6-2/` contains the spec, plan, this tasks ledger, and the packet-prompts
    artifact, and they record the structured-state-only drift-read contract, the current-goal three-state
    sidecar-presence guard plus the separate anchor-presence guard, the sanctioned-replan exclusion via
    `analysis.sanctioned_replan`, and the no-`progress.rs`-migration boundary. The packet-prompts artifact
    uses the live packet numbering (`R6-2.0` through `R6-2.5`) and matches those source-of-truth
    constraints.
  - Verify: manual review against the `R6` MAP, the DESIGN doc, and live `context/objective.rs`.
  - Files:
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md`
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md`
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`
    - `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-packet-prompts.md`

## R6-2.1: Capture And Thread The Kickoff Anchor

- [x] Task R6-2.1.1: Capture the session kickoff structured-goal anchor (access path settled — threaded).
  - Acceptance: (a) a minimal, committed helper captures the kickoff anchor from the existing per-checkpoint
    `structured_objective` (the first confident `TaskStatement` goal — the only concrete source; the
    session-level kickoff-signal hook is disabled; the Open Question 1 corpus check confirms it holds), read once and reused, not
    recomputing objective extraction; and (b) the anchor is threaded into `score_session` via the
    per-session analyze loop (settled — mirroring `previous_truth_grounding_gap` at `lib.rs`);
    `session_kickoff_anchor(analysis)` is not viable and the current goal stays read from `analysis.current`.
    Also: (c) record the
    confidence-bar corpus check (`High`-anchor frequency; current `Medium`-vs-`High`), confirming the
    resolved bar (anchor `High` + current `Medium+`) is not dormant — or relaxing the anchor bar to
    `Medium+` if `High` anchors are scarce (SPEC Resolved Decision 5); and (d) add a
    `CheckpointAnalysis.sanctioned_replan` field computed at analysis-assembly time from steer-row evidence
    (SPEC Resolved Decision 4), with every construction site (incl. test fixtures) setting it. Resolves SPEC
    Open Question 1 and sizes R6-2.3.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (anchor capture + `sanctioned_replan` field)
    - `crates/agent-drift-analyzer/src/lib.rs` (thread the anchor into `score_session`)
    - `crates/agent-drift-analyzer/src/scoring/mod.rs` (`score_session` additive anchor input)
    - (read-only) `crates/agent-drift-analyzer/src/context/objective.rs`
  - Finding: Verified live prerequisites before editing: `R5.75-1` is promoted/closed in
    `docs/specs/r5/R5_75/MAP.md` and live code still exports the structured sidecar /
    `comparison_key_from_structured`; `R6-1` is promoted history in `docs/specs/r6/MAP.md` and the
    `R6-1` tasks ledger. Anchor source/access path: the only live kickoff source remains the per-checkpoint
    `structured_objective` sidecar (`ENABLE_KICKOFF_PRIORS` is still disabled), so
    `session_kickoff_structured_goal_anchor(&analyses)` now captures the **first**
    `current.context.objective.structured` that is `TaskStatement` + empty `unknowns` + `Confidence::High`,
    clones it once, and `analyze_loaded_bundle` threads `Option<&StructuredObjective>` into `score_session`
    alongside `previous_truth_grounding_gap`; no `session_kickoff_anchor(analysis)` helper was introduced
    and nothing was stamped onto every `CheckpointAnalysis`. Confidence-bar corpus check (committed bundle
    fixtures under `tests/fixtures/acceptance/` and `tests/fixtures/progress_acceptance/`): 18 fixture
    directories produced 120 checkpoints; 9 sessions exposed a qualifying kickoff anchor, all `High`
    confidence (8 first appeared at ordinal 1, 1 at ordinal 3). The current-goal side of the bar was also
    non-dormant: 32 checkpoints had `TaskStatement` + empty `unknowns`, and all 32 were `High` (0
    `Medium`). The resolved bar therefore stays anchor `High` + current `Medium+`; no relaxation to
    `Medium+` anchor was needed. `sanctioned_replan` derivation: `CheckpointAnalysis.sanctioned_replan` is
    now checkpoint-local assembly metadata, set by scanning the current checkpoint window for
    `UserMessageRole::Steer` rows whose normalized text matches the existing explicit-pivot phrase set
    (`replan`, `pivot`, `instead of`, `instead`, `new objective`, `change objective`, `change the
    objective`, `change scope`, `change the scope`, `different objective`). That keeps the signal sourced
    from steer-row evidence, not the private `progress.rs` string heuristic, and makes it available for the
    later semantic-drift scorer without threading another session-running bool.

## R6-2.2: Confirm The Variant Blast Radius And Schema Decision (Impact-Gated, No Scorer Code)

- [x] Task R6-2.2.1: Confirm the `SemanticGoalDrift` variant blast radius and the `schema_version` decision.
  - Acceptance: the variant is the decided shape (SPEC Resolved Decision 6). Run `gitnexus_impact` on
    `score_session` and the `SemanticGoalDrift` variant; record the full blast radius (schema `DriftClass`,
    analyzer `export.rs` class lists/labels, analyzer sort order, sentinel `operator_surface.rs` mappings —
    `drift_class_name` / `historical_reason_prefixes` / `checkpoint_had_active_class`). From that report,
    decide the `schema_version` `v0.7` bump (honest labeling + sentinel allowlist/gate updates, not
    compat protection — old readers break at deserialization regardless). Record the decision.
  - Verify: impact report recorded in this ledger; no scorer code committed from this task.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/checkpoint/schema.rs`,
      `crates/agent-drift-analyzer/src/checkpoint/export.rs`,
      `crates/agent-drift-sentinel/src/operator_surface.rs`
  - Finding: Live prerequisite check passed before this ledger update: `R6-2.0` is landed in repo truth
    via the docs-lock commit (`085b5193c`) plus the present spec/plan/tasks/prompt artifact set under
    `docs/specs/r6/R6-2/`, and `R6-2.1.1` is landed in both this ledger and live code
    (`c53ba7a44`, `1c2e1da8a`; `score_session` now accepts `kickoff_anchor: Option<&StructuredObjective>`).
    GitNexus impact on `score_session` (`npx gitnexus impact score_session --repo 97a0-substrate --direction upstream --depth 4`)
    came back **LOW** risk with one direct caller (`analyze_loaded_bundle`), one affected process
    (`analyze_loaded_bundle`, 9 hits / earliest broken step 1), two affected modules (`Checkpoint`
    direct, `Tests` indirect), and the upstream chain `analyze_loaded_bundle` -> `analyze_bundle` ->
    CLI `run`. GitNexus could not resolve `SemanticGoalDrift` yet because the variant does **not** exist
    at HEAD, so the honest proxy was the live `DriftClass` surface plus direct code reads: adding the
    variant will require lockstep edits in `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    (the `DriftClass` enum itself), `crates/agent-drift-analyzer/src/scoring/mod.rs`
    (`score_session` exhaustive sort order), `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    (`drift_classes()` inventory and `drift_class_label()`), and
    `crates/agent-drift-sentinel/src/operator_surface.rs`
    (`drift_class_name`, `historical_reason_prefixes`, and the posture path that depends on
    `checkpoint_had_active_class`; the helper body is generic, but the new variant only becomes
    sentinel-visible once the explicit name/prefix surfaces are taught about it). Schema blast radius is
    likewise broader than the enum line itself: analyzer checkpoints are still emitted as
    `schema_version: "v0.6"` in `checkpoint/mod.rs`; analyzer validation currently treats
    `session_archetype` as required only for `v0.5 | v0.6` and `session_progress` as required only for
    `v0.6` in `checkpoint/schema.rs`; sentinel replay input hard-codes support only through `v0.6` in
    `crates/agent-drift-sentinel/src/input.rs`
    (`SUPPORTED_ANALYZER_CHECKPOINT_SCHEMAS` / `SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA_DESCRIPTION`, plus the
    `validate_checkpoint_contract` branches that enforce the `v0.6` contract fields); sentinel live input
    does the same in `crates/agent-drift-sentinel/src/live_input.rs`
    (`SUPPORTED_ANALYZER_CHECKPOINT_SCHEMAS` / `SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA_DESCRIPTION`, plus the
    live-fixture compatibility checks against the `v0.6` analyzer contract); and sentinel explicit-state
    gating currently admits only `v0.3 | v0.4 | v0.5 | v0.6` in `operator_surface.rs`. Those
    `input.rs` / `live_input.rs` gates are therefore part of the real `v0.7` fallout now, not merely a
    later `R6-2.3` handoff concern. Decision: **bump checkpoint `schema_version` to `v0.7` when
    `SemanticGoalDrift` lands.** This is an honest-labeling + sentinel-gate decision, not a compatibility
    shield: old readers still fail at enum deserialization before any version gate if the new variant
    appears. The bump is still warranted because the active drift-class surface, analyzer export
    labels/lists, sentinel replay/live schema allowlists and contract gates, and sentinel explicit-state
    allowlist all change together, so shipping the variant under `v0.6` would mislabel a materially
    different checkpoint contract. Operationally this remains a **lockstep deploy**: land the variant with
    the analyzer `export.rs` updates, the sentinel `input.rs` / `live_input.rs` / `operator_surface.rs`
    updates, the `v0.7` writer/gate changes, and the full analyzer + sentinel walls in the same packet;
    do not rely on versioning to protect mixed old/new binaries.

## R6-2.3: Semantic-Goal-Drift Scorer + Presence Guard

- [ ] Task R6-2.3.1: Implement the rule-based scorer with the two presence guards.
  - Acceptance: a new `scoring/semantic_goal_drift.rs` first applies the current-goal three-state guard
    (present+confident → score; absent → no claim; present-but-unknown → no claim) and the separate anchor
    guard, at the resolved confidence bar (anchor `High` + current `Medium+`, both `TaskStatement`, empty
    `unknowns` — SPEC Resolved Decision 5), then compares the current goal (from `analysis.current`) to the
    kickoff anchor (from the R6-2.1 threaded `score_session` input, not from `analysis`) over
    `comparison_key`/structured terms, excluding sanctioned explicit replans **via `analysis.sanctioned_replan`**
    (SPEC Resolved Decision 4; the field added in R6-2.1 — the private `progress.rs` detectors are not
    reachable from `score_session`), and attaches named evidence (anchor + drifted goal). It emits the new
    `DriftClass::SemanticGoalDrift` (SPEC Resolved Decision 6), wired into `score_session` per the R6-2.1
    access path. Per the Packet `R6-2.2` blast-radius decision, this same packet also owns the lockstep
    `v0.7` fallout: bump the checkpoint writer in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`,
    extend the analyzer/sentinel schema gates in `crates/agent-drift-analyzer/src/checkpoint/schema.rs`,
    `crates/agent-drift-sentinel/src/input.rs`, `crates/agent-drift-sentinel/src/live_input.rs`, and
    `crates/agent-drift-sentinel/src/operator_surface.rs`, and move the class-list / schema-version test
    surfaces with the variant in the same commit. The drift signal reads structured state only — never
    `task_frame.objective`. Add the minimal presence-guard + drift-vs-replan proof here (TDD); the full
    matrix is R6-2.4.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - targeted fallout review/update for the class-list / schema-version surfaces named below so `v0.7`
      writer + gate changes do not strand stale coverage
  - Files:
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (new)
    - `crates/agent-drift-analyzer/src/scoring/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (`schema_version` writer bump to `v0.7`)
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs` (`SemanticGoalDrift` variant + analyzer gate)
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs` (class lists/labels, lockstep)
    - `crates/agent-drift-sentinel/src/input.rs` (replay schema allowlist/gate, lockstep)
    - `crates/agent-drift-sentinel/src/live_input.rs` (live schema allowlist/gate, lockstep)
    - `crates/agent-drift-sentinel/src/operator_surface.rs` (mappings + explicit-state gate, lockstep)
    - `crates/agent-drift-analyzer/tests/end_to_end.rs` and
      `crates/agent-drift-analyzer/tests/export_bundle.rs` (class-list / schema-version fallout)
    - `crates/agent-drift-sentinel/tests/replay_input.rs`,
      `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`,
      `crates/agent-drift-sentinel/tests/live_end_to_end.rs`, and
      `crates/agent-drift-sentinel/tests/operator_surface.rs` (schema-version / operator-surface fallout)
    - `crates/agent-drift-analyzer/tests/...` (minimal proof only)

## R6-2.4: Regressions And Acceptance Fixture

- [ ] Task R6-2.4.1: Complete the presence-guard + drift matrix (do not re-add R6-2.3's minimal proof).
  - Acceptance: tests assert all three presence-guard states (score / absent-no-claim / unknown-no-claim);
    an unauthorized pivot → flagged with anchor-naming evidence; a sanctioned explicit replan → not
    flagged; and a structured-source proof where the bridge-patched display string and the structured goal
    disagree scores off the structured goal.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/...`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R6-2.4.2: Commit a kickoff-anchored drift acceptance fixture + assert non-regression.
  - Acceptance: a committed acceptance fixture (locked like `objective_acceptance`) proves the
    kickoff-anchored drift case end to end; `R5.75-3`/`R5.75-4` witnesses and the `R6-1` `dead_end_thrash`
    posture are asserted unchanged.
  - Verify: `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/` (new acceptance fixture + harness wiring)
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

## R6-2.5: Smoke And Closeout

- [ ] Task R6-2.5.1: Full (+ sentinel) walls, the `R6-4` open/defer decision, and MAP status update.
  - Acceptance: the full analyzer wall and the full sentinel walls (the `SemanticGoalDrift` variant touches
    the sentinel surface) are green; the closeout records whether any `R6-1`/`R6-2` replay evidence showed a `progress.rs` reset
    error caused by objective-string quality — if yes, route to the conditional `R6-4`; if no, close `R6`
    with `R6-4` deferred to the later full-migration phase. The `R6-2` entry in `docs/specs/r6/MAP.md` is
    updated with status, routing, and the `R6-4` decision.
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel -- --nocapture` (the `SemanticGoalDrift` variant touches
      `operator_surface.rs`, so this wall is required)
  - Files:
    - `docs/specs/r6/MAP.md`

## Deferred / Ask-First

- [ ] Task R6-2.X.1: Open the conditional `R6-4` (progress.rs reset onto `comparison_key`).
  - Acceptance: only if R6-1/R6-2 replay evidence shows reset/continuity errors caused by objective-string
    quality; migrate `explicit_replan_boundary` / `material_objective_delta` onto `comparison_key` with a
    sidecar-presence guard, preserving `R5.75-3`/`R5.75-4`. Otherwise this folds into the later
    full-migration phase and is not written here.
  - Verify: to be defined when (and if) opened, with its own SPEC/PLAN/TASKS.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
