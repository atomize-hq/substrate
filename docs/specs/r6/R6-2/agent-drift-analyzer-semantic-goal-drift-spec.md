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
3. **Drift is scored only when the sidecar is present and confident (mandatory presence guard).** This is
   the three-state guard on the **current** goal's sidecar: present and confident → eligible to score;
   sidecar absent → no drift claim (conservative); sidecar present but key fields unknown
   (`ObjectiveUnknown` / `objective_class != TaskStatement`) → no drift claim. There is a **separate**
   anchor-presence guard (anchor captured yet, or not → no claim); the two guards are distinct, not a
   single four-state guard. What counts as "confident" is not yet pinned: live extraction yields
   `Low`/`Medium`/`High` (`context/objective.rs`), and whether `TaskStatement + Medium` scores or is
   suppressed to `High`-only is an **open design decision** (see Open Questions). Silent fallback to
   brittle string assumptions must be visible in tests.
4. **A new `DriftClass` variant is the likely shape, and it has cross-crate blast radius plus a serde
   cost.** Surfacing semantic goal drift as a first-class drift signal needs a new `DriftClass` variant
   (e.g. `SemanticGoalDrift`), which requires updating the sentinel `operator_surface.rs` mapping
   (`drift_class_name`, `historical_reason_prefixes`, `checkpoint_had_active_class`) and `score_session`'s
   sort order. It is additive at the enum, but `DriftClass` is a **closed serde enum**, so adding a variant
   is a forward-compat break for any older reader deserializing checkpoints — acceptable only under in-repo
   lockstep, **not** a cost-free additive change. Whether to pay that (a real variant) or surface drift as
   evidence within an existing class is an open design decision (see Open Questions). This cross-crate
   coordination is called out as Ask-First (Boundaries) and gated on `gitnexus_impact`.
5. **Rule-based, interpretable first cut.** Semantic distance is a deterministic comparison of structured
   goal anchors (`comparison_key` / structured terms), not a learned monitor. Learned/hybrid scoring stays
   deferred.
6. **`R5.75` behavior is preserved.** This packet is additive; it must not change the progress lanes or
   regress delegated-stability (`R5.75-3`) or zero-verifier anti-flap (`R5.75-4`).
7. **Drift ≠ legitimate replan.** A goal change that is an explicit, user-authorized replan/pivot is not
   semantic drift; the scorer must not flag a sanctioned objective change. **But the existing replan
   detectors are not reachable from the scorer as written:** `explicit_replan_boundary` (`progress.rs`) and
   `objective_candidate_is_explicit_replan_pivot` (`checkpoint/mod.rs`) are private, and `CheckpointAnalysis`
   carries no sanctioned-replan bit — `score_session` gets only `CheckpointAnalysis` +
   `previous_truth_grounding_gap`. So "just cross-check the existing signals" is not implementable today;
   how to expose a sanctioned-replan signal to the scorer is an **open design decision** (see Open
   Questions).

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

Full analyzer wall + (if a `DriftClass` variant lands) sentinel walls for closeout:

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
  Per-checkpoint structured_objective (from R5.75-1); kickoff-anchor capture; the new DriftClass variant if added.
crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs (new)
  The new rule-based scorer with the sidecar-presence guard.
crates/agent-drift-sentinel/src/operator_surface.rs
  DriftClass mapping update, only if a variant lands (lockstep, additive).
```

## Code Style

Make the presence guards the first thing the scorer does, and keep the states explicit and test-visible.
Note the live types: the goal is a `StructuredObjective` (`schema.rs`), reached via
`analysis.current.context.objective` (which is an `ObjectiveSummary` carrying `structured` +
`comparison_key`). `confident_structured_goal` below is a **helper to define** in this packet from
`objective_class == TaskStatement`, `confidence`, and empty `unknowns` — the exact `confidence` bar is an
open design decision. `sanctioned_replan` is **not** derivable from `analysis` today (see Assumption 7 /
Open Questions); it is shown as a threaded/CheckpointAnalysis input, pending that decision.

```rust
// `anchor: Option<&StructuredObjective>` is threaded into score_session from the per-session analyze loop
// (like previous_truth_grounding_gap) — None until the first confident anchor is captured. NOT on `analysis`.
// `sanctioned_replan: bool` is likewise supplied to the scorer (threaded or as a CheckpointAnalysis field);
// it is NOT read from a private progress.rs helper — those are unreachable from score_session.

// Sidecar-presence guard on the CURRENT goal (three states): absent OR unknown → no drift claim.
let Some(current_goal) = confident_structured_goal(&analysis.current) else {
    return no_drift_claim(SidecarState::AbsentOrUnknown); // conservative, test-visible
};
// Separate anchor-presence guard: no confident kickoff anchor captured yet → no drift claim.
let Some(anchor_goal) = anchor else {
    return no_drift_claim(SidecarState::NoAnchor);
};

// Compare structured goal anchors (comparison_key / structured terms), never the bridge-patched display string.
if comparison_key_diverged(current_goal, anchor_goal) && !sanctioned_replan {
    // flagged semantic goal drift, evidence names the anchor + the drifted goal
}
```

Conventions for this packet:

- Read `comparison_key` / structured goal anchors; never read `task_frame.objective` for the drift signal.
- Thread the kickoff anchor into `score_session` as session-running input (mirroring
  `previous_truth_grounding_gap`); read the current goal from `analysis.current`. Never assume the anchor
  is on a single `CheckpointAnalysis`.
- The presence guard's three outcomes (score / absent / unknown) are each asserted in tests.
- Exclude sanctioned explicit replans from drift via the existing replan signals.
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
   if a `DriftClass` variant lands, the sentinel walls stay green.

## Boundaries

- **Always:**
  - verify `R5.75-1` (structured sidecar + `comparison_key_from_structured`) and `R6-1` are landed before
    editing; if a named prerequisite is missing, stop and report rather than compensating here;
  - read the drift signal from structured state only;
  - reach the kickoff anchor by threading it into `score_session` as additive session-running input
    (mirroring `previous_truth_grounding_gap`), not by reading it off a single `CheckpointAnalysis`; read
    the current goal from `analysis.current`;
  - implement the three-state sidecar-presence guard and assert each state;
  - run `gitnexus_impact` on `score_session` and (if added) the new `DriftClass` variant before editing,
    and report the blast radius — especially the sentinel `operator_surface.rs` coupling;
  - keep the change additive and preserve `R5.75`/`R6-1` behavior.
- **Ask first:**
  - adding the `SemanticGoalDrift` `DriftClass` variant (cross-crate: schema + sentinel mapping +
    `score_session` ordering) — confirm the variant vs. surfacing drift as evidence within an existing
    class before landing;
  - the exact semantic-distance threshold over `comparison_key`/structured terms;
  - whether the kickoff anchor is the first confident checkpoint or a session-level kickoff signal.
- **Never:**
  - read `task_frame.objective` (the bridge-patched display string) for the drift signal;
  - migrate `progress.rs` comparability/reset here (that is the conditional `R6-4`);
  - flag a sanctioned explicit replan as drift;
  - score drift when the sidecar is absent or its goal fields are unknown;
  - add a learned/hybrid scorer or bump the schema beyond the additive variant.

## Success Criteria

1. Semantic goal drift is flagged only on an unauthorized pivot away from the confident kickoff structured
   goal, with anchor-naming evidence.
2. The three sidecar-presence states are each asserted: score / absent-no-claim / unknown-no-claim.
3. The signal is provably derived from `comparison_key`/structured state, not the patched display string.
4. A sanctioned explicit replan is not flagged.
5. `R5.75-3`/`R5.75-4` and `R6-1` postures do not regress; any `DriftClass` variant is additive with the
   sentinel mapping updated in lockstep; `cargo test -p agent-drift-analyzer -- --nocapture` (and the
   sentinel walls if touched) are green.

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

## Open Questions

1. **Kickoff anchor — two coupled sub-questions, both resolved in PLAN step `R6-2.1`.** (a) *Source:* the
   anchor is the first confident `TaskStatement` checkpoint goal — the only concrete source today, since
   the `R4` session-level kickoff-signal hook is disabled (`checkpoint/mod.rs`), so it is not a live peer
   alternative; confirm this holds across the fixture corpus. (b) *Access path:* how does the session-level anchor
   reach the scorer, given `score_session` gets only `CheckpointAnalysis` + `previous_truth_grounding_gap`?
   Recommended: thread a running anchor through the per-session analyze loop into `score_session` (additive
   input, mirrors `previous_truth_grounding_gap` at `lib.rs`); the alternative is capturing it onto
   `CheckpointAnalysis`. `session_kickoff_anchor(analysis)` is **not** viable — the anchor is not on
   `analysis`. (Contrast `R6-1`: its frontier signal is checkpoint-local, so it could be derived inside
   `score_session` via `build_scoring_session_progress`; the anchor is session-level, so that pattern does
   not extend.)
2. Should semantic goal drift be a new `DriftClass` variant (cleaner operator semantics, cross-crate
   change) or evidence within an existing class (smaller blast radius, weaker surfacing)? (Resolve in
   `R6-2.2` with `gitnexus_impact`, default per Assumption 4 toward the additive variant pending the
   impact report.)
3. What `comparison_key`/structured-term distance threshold cleanly separates drift from normal goal
   refinement without flagging sanctioned replans? (Resolve in `R6-2.3` against the acceptance fixtures.)
4. **Sanctioned-replan reachability (blocking design decision).** The existing replan detectors are private
   and not reachable from the scorer. How is a sanctioned-replan signal exposed to `score_session` — thread
   a bool through the analyze loop (like the anchor / `previous_truth_grounding_gap`), add a field to
   `CheckpointAnalysis`, or drop replan-exclusion from the first cut and accept sanctioned pivots scoring as
   drift until a follow-up? (Resolve before `R6-2.3`.)
5. **Confidence bar for "confident" (blocking design decision).** Live extraction yields `Low`/`Medium`/`High`.
   Does `TaskStatement + Medium` (with no `unknowns`) score, or is scoring suppressed to `High`-only for the
   first cut? This sets how often the guard fires vs stays conservative. (Resolve in `R6-2.1`/`R6-2.3`.)
