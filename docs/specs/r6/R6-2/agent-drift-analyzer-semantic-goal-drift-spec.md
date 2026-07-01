# Spec: Agent Drift Analyzer Semantic Goal Drift From Kickoff/Plan/Docs (R6-2)

Status: draft spec created on 2026-06-27 after `R5.75` closed and `Decision Gate 0` resolved to scoped
Option C in `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`. This spec is the
implementation authority for the `R6-2` packet defined in `docs/specs/r6/MAP.md`. `R6-2` is the
**structured-objective integration** half of Option C: it gives `comparison_key` its first live consumer.

Authority order for this packet:
`docs/specs/r6/MAP.md` owns landing order and the `R6` family routing;
`docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md` owns the resolved
objective-consumption boundary and the new-consumer architecture this packet implements;
this SPEC/PLAN/TASKS family owns the implementation contract; the live crate
(`crates/agent-drift-analyzer/src/context/objective.rs` for the structured sidecar,
`src/checkpoint/schema.rs`/`mod.rs` for the per-checkpoint `structured_objective`,
`src/scoring/`) is the ground truth for current behavior.

## Assumptions I'm Making

1. **`R6-2` reads the structured sidecar directly; it does not read the `R5.75-6` bridge-patched display
   string and does not migrate `TaskFrame`.** The whole point of resolving Gate 0 to Option C is that
   semantic goal drift is computed from typed structured state — the `StructuredObjective` goal anchor and
   the already-computed `ObjectiveSummary.comparison_key` (the private `comparison_key_from_structured` is
   its producer inside `context/objective.rs`; the scorer reads the computed `comparison_key`, it does not
   call the producer) — never the heuristic-patched `task_frame.objective`. The full `TaskFrame` Phase-2
   migration stays a later phase.
2. **The kickoff anchor is the session's anchoring structured goal, captured once and threaded into
   scoring.** Drift is measured between the current checkpoint's structured goal and the session's
   kickoff/anchor structured goal. The anchor is read from the first checkpoint whose structured objective
   is a confident `TaskStatement` (reusing the `R5.75-1` grounding), not recomputed. This "first confident
   `TaskStatement` checkpoint" is the **only concrete anchor source today**: the alternative "session-level
   kickoff signal" (`R4` kickoff priors) is a *disabled* hook (`checkpoint/mod.rs`), so it is noted for
   future work, not a live peer option. **Reachability (do not
   repeat the `R6-1` mistake):** the scorer receives only `CheckpointAnalysis` (current + one `previous`)
   plus `previous_truth_grounding_gap`; the kickoff anchor is *session-level* and is **not** a field on
   `CheckpointAnalysis`. So the anchor must be threaded into `score_session` through the per-session analyze
   loop — mirroring how `previous_truth_grounding_gap` is already threaded (`lib.rs`) — or captured onto
   `CheckpointAnalysis`. It **cannot** be read via `session_kickoff_anchor(analysis)`. The *current* goal,
   by contrast, is checkpoint-local and is reachable directly at `analysis.current.context.objective.structured`.
   The access path is resolved in `R6-2.1` (Open Question 1b).
3. **Drift is scored only when the sidecar is present and confident (mandatory presence guards).** Two
   distinct guards, not one four-state guard: (i) the three-state guard on the **current** goal's sidecar —
   present and confident → eligible to score; sidecar absent → no drift claim (conservative); sidecar
   present but key fields unknown (`ObjectiveUnknown` / `objective_class != TaskStatement`) → no drift
   claim; and (ii) a **separate** anchor-presence guard (no confident anchor captured yet → no claim).
   **Confidence bar (resolved — see Resolved Decision 5):** the anchor must be `High`, the current goal
   `Medium`-or-`High`, both `TaskStatement` with empty `unknowns`. `High`-only-both-sides would be dormant
   because the extractor is structurally `Medium`-heavy (any goal clause resolves to `Medium` unless it is
   on `Scope`/`Mission` with goal evidence — `context/objective.rs`); the `High` anchor + `Medium+` current
   bar keeps the origin confident while still firing on the common case. Silent fallback to brittle string
   assumptions must be visible in tests.
4. **Semantic goal drift is surfaced as a new `DriftClass::SemanticGoalDrift` variant (resolved — see
   Resolved Decision 6).** Burying it as evidence under an existing class conflates failure modes and pays
   the variant cost later anyway plus a migration. The variant needs lockstep exhaustive-match updates in:
   analyzer `score_session` sort order (`scoring/mod.rs`), analyzer export class lists/labels
   (`checkpoint/export.rs`), and the sentinel `operator_surface.rs` mappings (`drift_class_name`, the
   historical-fallback / `historical_reason_prefixes`, `checkpoint_had_active_class`). `DriftClass` is a
   **closed serde enum**, so adding a variant is a hard forward-compat break for any older reader — it fails
   at checkpoint deserialization *before* any `schema_version` gate, so a version bump does **not** protect
   old binaries; lockstep deploy is the only mitigation. Whether to bump `schema_version` to `v0.7` (honest
   labeling + sentinel allowlist/gate updates, not compat protection) is decided in `R6-2.2` with
   `gitnexus_impact`. Whether `R6-3` rolling drift reuses this variant or gets its own is **not** pre-decided
   here — deferred to the `R6-3` spec (one merged operator family vs two independently surfaced signals is a
   real tradeoff given the per-class fingerprint/threshold surfaces).
5. **Rule-based, interpretable first cut.** Semantic distance is a deterministic comparison of structured
   goal anchors (`comparison_key` / structured terms), not a learned monitor. Learned/hybrid scoring stays
   deferred.
6. **`R5.75` behavior is preserved.** This packet is additive; it must not change the progress lanes or
   regress delegated-stability (`R5.75-3`) or zero-verifier anti-flap (`R5.75-4`).
7. **Drift ≠ legitimate replan; the sanctioned-replan signal is a new `CheckpointAnalysis` field (resolved
   — see Resolved Decision 4).** A sanctioned/authorized pivot must not be flagged. The existing replan
   detectors are private and not reachable from `score_session`, and they are *weak heuristics*
   (`explicit_replan_boundary` is objective-string delta + a keyword, `progress.rs`;
   `objective_candidate_is_explicit_replan_pivot` keys off a `UserMessageRole::Steer` row + a phrase list,
   `checkpoint/mod.rs`). Because "was this checkpoint an authorized replan?" is *checkpoint-local derived
   metadata* (like `turn_context` / `recovery` / `delegation`), it is added as a field on
   `CheckpointAnalysis` computed at analysis-assembly time, sourced primarily from **steer-row evidence**,
   not the `progress.rs` string heuristic. The scorer reads `analysis.sanctioned_replan`; nothing is
   threaded for replan (only the session-level anchor is threaded).

If any of these assumptions drift, update this spec before implementation.

## Objective

Add a `semantic drift from kickoff/plan/docs` drift dimension that flags when a session's current
effective goal has drifted away from its anchored kickoff goal, computed from the **structured objective
sidecar** (`comparison_key` + typed goal anchor) with a mandatory sidecar-presence guard, never from the
bridge-patched display string. This is the first live consumer of `comparison_key`.

Primary users:

1. operators who need to see "the agent stopped working the stated goal" as an explicit drift signal,
   distinct from thrash or out-of-scope edits;
2. the sentinel/replay path, which should receive an honest semantic-drift posture only when the
   structured objective is confident;
3. the later full structured-native migration, which gains a proven first consumer of `comparison_key`.

This packet succeeds when:

1. a session that pivots away from its kickoff structured goal (without an authorized replan) scores
   semantic goal drift, with evidence naming the anchor and the drifted goal;
2. drift is scored **only** when the sidecar is present and confident; an absent or unknown sidecar
   yields no drift claim (conservative), proven by tests;
3. a sanctioned explicit replan/pivot is **not** flagged as drift;
4. the signal is derived from `comparison_key` / structured terms, not `task_frame.objective`;
5. `R5.75-3`/`R5.75-4` outcomes and the `R6-1` `dead_end_thrash` posture do not regress;
6. any new `DriftClass` variant is additive and the sentinel mapping is updated in lockstep, with no
   schema version bump beyond the additive variant.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer` (+ coordinated additive change in `agent-drift-sentinel` if a new
  `DriftClass` variant lands)
- Live code seams for this packet:
  - `crates/agent-drift-analyzer/src/context/objective.rs` (read: `StructuredObjective`,
    `comparison_key_from_structured`, the grounded goal anchor authority)
  - `crates/agent-drift-analyzer/src/checkpoint/{schema.rs,mod.rs}` (the per-checkpoint
    `structured_objective` already exported by `R5.75-1`; the kickoff anchor capture)
  - `crates/agent-drift-analyzer/src/scoring/` (a new `semantic_goal_drift.rs` module + `scoring/mod.rs`)
  - `crates/agent-drift-sentinel/src/operator_surface.rs` (the `DriftClass` mapping, if a variant is added)
  - `crates/agent-drift-analyzer/tests/` (a new acceptance + scorer regression surface)
- This packet does **not** touch `progress.rs` comparability/reset (that is the conditional `R6-4`).

## Commands

Focused scorer + structured-objective regressions:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

Full analyzer wall + sentinel walls for closeout (the `SemanticGoalDrift` variant touches the sentinel surface):

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

## Project Structure

```text
docs/specs/r6/MAP.md
  Landing-order authority for the R6 family.
docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
  Resolved Decision Gate 0; the new-consumer architecture this packet implements.
docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/context/objective.rs
  StructuredObjective + comparison_key_from_structured: the typed state R6-2 consumes (read-only here).
crates/agent-drift-analyzer/src/checkpoint/{schema.rs,mod.rs}
  Per-checkpoint structured_objective (from R5.75-1); kickoff-anchor capture; the sanctioned_replan field; the new SemanticGoalDrift DriftClass variant.
crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs (new)
  The new rule-based scorer with the two presence guards.
crates/agent-drift-analyzer/src/checkpoint/export.rs
  DriftClass class lists/labels for the new variant (lockstep).
crates/agent-drift-sentinel/src/operator_surface.rs
  DriftClass mapping update for the new variant (lockstep).
```

## Code Style

Make the presence guards the first thing the scorer does, and keep the states explicit and test-visible.
Note the live types: the goal is a `StructuredObjective` (`schema.rs`), reached via
`analysis.current.context.objective` (an `ObjectiveSummary` carrying `structured` + `comparison_key`).
`confident_structured_goal` below is a **helper to define** from `objective_class == TaskStatement`, empty
`unknowns`, and the resolved confidence bar (anchor `High`, current `Medium`-or-`High` — Resolved
Decision 5). `sanctioned_replan` is a new `CheckpointAnalysis` field (Resolved Decision 4), read as
`analysis.sanctioned_replan`. Only the session-level `anchor` is threaded.

```rust
// `anchor: Option<&StructuredObjective>` is threaded into score_session from the per-session analyze loop
// (like previous_truth_grounding_gap) — None until the first High-confidence anchor is captured. NOT on `analysis`.
// `analysis.sanctioned_replan` is a CheckpointAnalysis field (Resolved Decision 4), derived at
// analysis-assembly time from steer-row evidence — NOT read from a private progress.rs helper.

// Current-goal guard (three states) at the resolved bar (Medium-or-High): absent OR unknown → no drift claim.
let Some(current_goal) = confident_structured_goal(&analysis.current) else {
    return no_drift_claim(SidecarState::AbsentOrUnknown); // conservative, test-visible
};
// Separate anchor-presence guard: no confident (High) kickoff anchor captured yet → no drift claim.
let Some(anchor_goal) = anchor else {
    return no_drift_claim(SidecarState::NoAnchor);
};

// Compare structured goal anchors (comparison_key / structured terms), never the bridge-patched display string.
if comparison_key_diverged(current_goal, anchor_goal) && !analysis.sanctioned_replan {
    // flagged semantic goal drift, evidence names the anchor + the drifted goal
}
```

Conventions for this packet:

- Read `comparison_key` / structured goal anchors; never read `task_frame.objective` for the drift signal.
- Thread the kickoff anchor into `score_session` as session-running input (mirroring
  `previous_truth_grounding_gap`); read the current goal from `analysis.current`. Never assume the anchor
  is on a single `CheckpointAnalysis`.
- The current-goal guard's three outcomes (score / absent / unknown) and the separate anchor guard are
  each asserted in tests.
- Exclude sanctioned explicit replans from drift via the `analysis.sanctioned_replan` field (Resolved
  Decision 4), not private `progress.rs` helpers.
- Keep the scorer rule-based and evidence-first; no learned monitor.

## Testing Strategy

1. **Sidecar-presence guard regressions** (new `tests/...` + `tests/checkpoints.rs`): present+confident →
   scores; absent → no claim; present-but-unknown (`ObjectiveUnknown`) → no claim. Each state asserted.
2. **Drift-vs-replan regressions**: a goal pivot away from the kickoff anchor without authorization →
   flagged; a sanctioned explicit replan → not flagged.
3. **Structured-source proof**: the signal changes with `comparison_key`/structured state and is
   independent of the bridge-patched display string (a session where the patched string and the structured
   goal disagree scores off the structured goal).
4. **Acceptance fixture(s)**: a committed kickoff-anchored session that drifts, locked like the
   `objective_acceptance` corpus.
5. **Non-regression**: `R5.75-3`/`R5.75-4` witnesses and the `R6-1` `dead_end_thrash` posture unchanged;
   the sentinel walls stay green after the `SemanticGoalDrift` variant + `operator_surface.rs`/`export.rs`
   updates.

## Boundaries

- **Always:**
  - verify `R5.75-1` (structured sidecar + `comparison_key_from_structured`) and `R6-1` are landed before
    editing; if a named prerequisite is missing, stop and report rather than compensating here;
  - read the drift signal from structured state only;
  - reach the kickoff anchor by threading it into `score_session` as additive session-running input
    (mirroring `previous_truth_grounding_gap`), not by reading it off a single `CheckpointAnalysis`; read
    the current goal from `analysis.current`;
  - implement the three-state sidecar-presence guard and assert each state;
  - run `gitnexus_impact` on `score_session` and the new `SemanticGoalDrift` `DriftClass` variant before
    editing, and report the blast radius — especially the sentinel `operator_surface.rs` coupling and the
    analyzer `export.rs` class lists;
  - keep `DriftScore`'s output shape additive and preserve `R5.75`/`R6-1` behavior; the `DriftClass`
    variant lands only with its lockstep sentinel + `export.rs` updates.
- **Ask first:**
  - the `schema_version` `v0.7` bump decision (honest labeling + sentinel allowlist/gate updates, not
    compat protection) — resolve in `R6-2.2` from the `gitnexus_impact` blast-radius report;
  - the exact semantic-distance threshold over `comparison_key`/structured terms (Open Question 2).
- **Never:**
  - read `task_frame.objective` (the bridge-patched display string) for the drift signal;
  - migrate `progress.rs` comparability/reset here (that is the conditional `R6-4`);
  - flag a sanctioned explicit replan as drift;
  - score drift when the sidecar is absent or its goal fields are unknown;
  - add a learned/hybrid scorer, or ship the `DriftClass` variant without its lockstep sentinel /
    `export.rs` updates.

## Success Criteria

1. Semantic goal drift is flagged only on an unauthorized pivot away from the confident kickoff structured
   goal, with anchor-naming evidence.
2. The three sidecar-presence states are each asserted: score / absent-no-claim / unknown-no-claim.
3. The signal is provably derived from `comparison_key`/structured state, not the patched display string.
4. A sanctioned explicit replan is not flagged.
5. `R5.75-3`/`R5.75-4` and `R6-1` postures do not regress; the `SemanticGoalDrift` `DriftClass` variant
   lands with its lockstep sentinel (`operator_surface.rs`) and analyzer (`export.rs`) updates — a
   forward-compat break for old readers, mitigated only by lockstep deploy; `cargo test -p
   agent-drift-analyzer -- --nocapture` and the sentinel walls are green.

## Resolved Decisions

1. **Resolved (2026-06-27, from DESIGN Gate 0): the drift signal reads structured state, not the bridge
   string.** This is the reason Option C exists — the new dimension cannot be honest on the heuristic
   `task_frame.objective`. `comparison_key`/structured goal anchors are the authority.
2. **Resolved (2026-06-27): `progress.rs` comparability/reset is not migrated in this packet.** Any
   structured-state migration of the reset surface is the conditional `R6-4`, evidence-gated, to honor
   Guardrail 4 (do not move progress reasoning first).
3. **Resolved (2026-06-30): `R6-2` ships the kickoff-anchored first cut only.** Drift is measured between
   the current checkpoint's structured goal and the session's kickoff/anchor goal (the "still on the
   original ask?" question). Rolling / previous-checkpoint drift — the current checkpoint's structured
   goal vs the *immediately-previous* checkpoint's (the "did we lurch this checkpoint?" question) — is the
   committed follow-up packet `R6-3` (MAP "Packet Documents"; DESIGN "Packet Decomposition"), **not** this
   packet. The two are complementary, not redundant: kickoff-anchored is the *cumulative* measure (it
   catches gradual drift from the original ask), and rolling is the *step-size* measure (it catches abrupt
   single-checkpoint pivots cheaply but on its own misses slow cumulative drift). The plumbing asymmetry
   also motivates the split: the previous-checkpoint goal is reachable via
   `analysis.previous.context.objective.structured` with no new plumbing, whereas this packet's kickoff
   anchor is session-level and must be threaded into scoring (see Open Question 1 / `R6-2.1`).
4. **Resolved (2026-06-30, codex-adjusted): the sanctioned-replan signal is a new `CheckpointAnalysis`
   field.** "Was this checkpoint an authorized replan?" is checkpoint-local derived metadata (like
   `turn_context` / `recovery` / `delegation`), so it is a `CheckpointAnalysis` field computed at
   analysis-assembly time from **steer-row evidence** — **not** a bool threaded into `score_session` (that
   pattern is for session-running state like the anchor), and **not** the weak private `progress.rs`
   string heuristic. The scorer reads `analysis.sanctioned_replan`. Dropping the exclusion was rejected
   (it ships a known false positive, violating Success Criterion 4).
5. **Resolved (2026-06-30, codex-adjusted): the confidence bar is anchor `High` + current `Medium`-or-`High`.**
   Both must be `TaskStatement` with empty `unknowns`. `High`-only-both-sides was rejected as dormant: the
   extractor is structurally `Medium`-heavy (`context/objective.rs`), so `High/High` would rarely fire. The
   `High` anchor keeps the origin confident; `Medium+` current fires on the common case. The empirical
   corpus check in `R6-2.1` (Open Question 1) must confirm `High` anchors actually occur often enough; if
   they are scarce, relax the anchor bar to `Medium+` and lean harder on the distance threshold.
6. **Resolved (2026-06-30, codex-adjusted): surface drift as a new `DriftClass::SemanticGoalDrift` variant.**
   Evidence-within-an-existing-class was rejected (conflates failure modes; pays the variant cost later
   plus a migration). The variant lands with lockstep exhaustive-match updates in analyzer sort order
   (`scoring/mod.rs`), analyzer `export.rs` class lists/labels, and sentinel `operator_surface.rs`
   mappings. Adding it is a hard forward-compat break (old readers fail at deserialization before any
   `schema_version` gate), so a version bump does not protect them — lockstep deploy does. The
   `schema_version` `v0.7` decision and the exact blast radius are confirmed in `R6-2.2` via
   `gitnexus_impact`. Whether `R6-3` reuses this variant is deferred to the `R6-3` spec.

## Open Questions

1. **Empirical anchor + confidence validation (resolve in `R6-2.1`).** Confirm across the fixture corpus
   that (a) the first-confident-`TaskStatement` anchor source holds (the `R4` session-level kickoff-signal
   hook is disabled, so it is the only concrete source), and (b) `High`-confidence anchors occur often
   enough for the anchor-`High` bar (Resolved Decision 5) not to leave the signal dormant — if `High`
   anchors are scarce, relax the anchor bar to `Medium+`. The access path is settled: thread the anchor
   into `score_session` (Resolved Decisions / Assumption 2); `session_kickoff_anchor(analysis)` is not
   viable.
2. What `comparison_key`/structured-term distance threshold cleanly separates drift from normal goal
   refinement without flagging sanctioned replans? (Resolve in `R6-2.3` against the acceptance fixtures.)
