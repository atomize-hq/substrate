# Rolling / Previous-Checkpoint Semantic Goal Drift (R6-3) Packet Prompts

Status: orchestration prompts created on 2026-07-02 against the live
`docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` ledger. Each prompt below is
self-contained: paste one into a fresh session to land exactly one `R6-3` sub-packet through a
`GPT-5.4` high implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

These prompts map:

- Packet `R6-3.0` -> Task `R6-3.0.1` (docs lock), plus Task `R6-3.0.2` (this packet-prompts artifact — **optional parity** per the ledger, "not required for implementation to proceed"; produced and committed here for `R6-1`/`R6-2` four-file parity)
- Packet `R6-3.1` -> Task `R6-3.1.1` (previous-checkpoint reachability + analyzer-local confirmation + eligibility/threshold corpus check; hard stop/go gate)
- Packet `R6-3.2` -> Task `R6-3.2.1` (rolling scorer extension + rolling-tagged evidence; anchor early-return restructure + minimal proof)
- Packet `R6-3.3` -> Tasks `R6-3.3.1` (full rolling regression matrix + acceptance fixture), `R6-3.3.2` (sentinel evidence-rendering test, test-only), and `R6-3.3.3` (`R6-2`/`R5.75`/`R6-1` non-regression)
- Packet `R6-3.4` -> Task `R6-3.4.1` (full analyzer + sentinel walls, no-variant/no-schema/no-sentinel-source confirmation, `R6-4` open/defer decision, MAP update)

Shared authority for all packets:

- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-{spec,plan,tasks}.md` (the landed kickoff-anchored scorer this packet extends)
- `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
- `docs/specs/r6/MAP.md`
- `AGENTS.md`

Working directory for every prompt:
`/Users/spensermcconnell/.codex/worktrees/97a0/substrate`

Global rules for every packet prompt below:

1. The parent session is the orchestration agent; it should not do the packet work locally unless
   delegated execution is unavailable.
2. The orchestration agent must spawn a fresh `GPT-5.4` subagent on `high` for implementation first.
3. Every implementation or fix subagent prompt must start with `/goal ` and must explicitly tell the
   subagent to use the `$incremental-implementation` skill.
4. After implementation work lands, the orchestration agent must commit before dispatching review.
5. The orchestration agent must then spawn a fresh `GPT-5.4` subagent on `high` for review.
6. Every review subagent prompt must start with `/goal ` and must explicitly tell the subagent to use
   the `$code-review-and-quality` skill.
7. If the review subagent flags issues, the orchestration agent must spawn a fresh `GPT-5.4` `high`
   fix subagent whose prompt starts with `/goal ` and explicitly uses the `$incremental-implementation`
   skill.
8. Commit every non-empty implementation/fix batch before sending a fresh review subagent back through
   the loop. Do not fabricate empty commits for verification-only packets that produce no file changes.
9. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and
   report any HIGH or CRITICAL blast radius before proceeding.
10. Run GitNexus detect-changes before every real commit. If GitNexus CLI fallback is needed because the
    repo is ambiguous, use `npx gitnexus detect-changes --repo 97a0-substrate --scope staged`; if the
    index must be refreshed first, use `npx gitnexus analyze --name 97a0-substrate`.
11. Packet prerequisite rule is strict: `R6-3` assumes `R6-2` is landed before any editing. Verify in
    live code, before editing, all four named prerequisites: (a) the `score_semantic_goal_drift`
    scorer, (b) the `DriftClass::SemanticGoalDrift` variant, (c) the `CheckpointAnalysis.sanctioned_replan`
    field, and (d) the `analysis.previous` slice on `CheckpointAnalysis`. If any named prerequisite is
    missing, or if `score_semantic_goal_drift(analysis, kickoff_anchor)` does not match the SPEC's
    described signature (it must already receive `analysis`, which carries `.previous`), stop and report
    it instead of compensating inside the packet.
12. `R6-3` reads structured state only and is analyzer-local. No packet may read the bridge-patched
    `task_frame.objective` for the drift signal, migrate `progress.rs` comparability/reset (that is
    conditional `R6-4`), flag a sanctioned explicit replan as drift, add a new `DriftClass` variant, bump
    `schema_version`, or change `checkpoint/schema.rs`'s `DriftClass` enum, `checkpoint/export.rs`, or
    sentinel `operator_surface.rs` source. The rolling contract stays: symmetric eligibility bar on both
    adjacent goals; the disjoint-set distance primitive reused with **symmetric** extraction
    (`goal_specific_terms(.., Some(summary))` on both sides); the `analysis.sanctioned_replan` exclusion;
    kickoff-anchored and rolling computed **independently** so an absent or ineligible kickoff anchor does
    not short-circuit the rolling comparison; and distinct rolling reason prefixes.
13. Rolling drift is tagged evidence on the existing `SemanticGoalDrift` class, **not** a new variant, and
    there is **no** `schema_version` bump (SPEC Resolved Decisions 1-2). The three flip-conditions that
    would promote rolling to its own variant in a later `R6` iteration are recorded in the SPEC; none
    holds today. Do not reopen the variant question mid-implementation.
14. Do not advance to the next packet until the current packet is committed and a fresh review subagent
    reports it review-clean, or for verification-only packets with no file changes, explicitly reports
    there was nothing to commit.

---

## Packet R6-3.0 Prompt — Tasks R6-3.0.1 and R6-3.0.2

````text
/goal Land Packet `R6-3.0` from `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-3.0` only. Do not start Packet `R6-3.1`.

Packet `R6-3.0` scope only:
- commit the bounded `R6-3` SPEC/PLAN/TASKS family and this packet-prompts file under `docs/specs/r6/R6-3/` (the packet-prompts file is the **optional-parity** Task `R6-3.0.2` artifact; committing it here satisfies `R6-1`/`R6-2` four-file parity, but the ledger does not require it for implementation to proceed — do not treat its absence as a blocker on any later packet)
- ensure those docs explicitly record: the evidence-not-variant surfacing decision plus its three flip-conditions; the no-`schema_version`-bump / analyzer-local boundary; the symmetric eligibility bar (both adjacent goals at the current-goal bar); the symmetric-extraction reuse of the `R6-2` disjoint-set distance primitive; the `sanctioned_replan` reuse; and the independent-compute / co-fire contract (an absent/ineligible kickoff anchor must not short-circuit rolling)
- verify the docs against `docs/specs/r6/MAP.md` item 6, the DESIGN `R6-3` charter, the landed `R6-2` SPEC, and live `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (`analysis.previous`, `sanctioned_replan`) + `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`

Primary files:
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md`

Out of scope:
- any production code change
- any `R6-3.1+` packet work
- any scorer implementation, corpus check, or `progress.rs` migration

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the docs diff and manual-verification story yourself.
5. Commit the landed Packet `R6-3.0` docs before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run GitNexus detect-changes before every real commit.
10. Do not start Packet `R6-3.1`.

Required verification:

```bash
# manual review against:
# - docs/specs/r6/MAP.md (item 6)
# - docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md (R6-3 charter)
# - docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md (the landed scorer this extends)
# - crates/agent-drift-analyzer/src/checkpoint/mod.rs (analysis.previous, sanctioned_replan)
# - crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
```

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-3.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: lock the `R6-3` SPEC/PLAN/TASKS docs and this packet-prompts artifact so they accurately define the rolling / previous-checkpoint semantic-goal-drift packet before implementation begins.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- AGENTS.md

Do this:
- keep the work docs-only and Packet `R6-3.0`-scoped
- ensure the SPEC/PLAN/TASKS family clearly records: the evidence-not-variant surfacing decision plus its three flip-conditions; the no-`schema_version`-bump / analyzer-local boundary; the symmetric eligibility bar (both adjacent goals at the current-goal bar); the symmetric-extraction reuse of the `R6-2` disjoint-set primitive (`Some(summary)` on both sides); the `analysis.sanctioned_replan` reuse; and the independent-compute / co-fire contract, including that an absent or ineligible kickoff anchor must not short-circuit the rolling comparison
- ensure the packet-prompts artifact matches the live source-of-truth packet numbering (`R6-3.0` through `R6-3.4`) and constraints
- do not start implementation for Packet `R6-3.1+`

Run:
- manual review against `docs/specs/r6/MAP.md` item 6, the DESIGN `R6-3` charter, the landed `R6-2` SPEC, and live `crates/agent-drift-analyzer/src/checkpoint/mod.rs` + `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
- GitNexus detect-changes before handing back for commit

Return with: changed files, what doc truth was locked, any remaining ambiguity, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-3.0` docs lock in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R6-3.0` against:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-packet-prompts.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- AGENTS.md

Focus:
- whether the docs accurately lock the packet contract before implementation
- whether the evidence-not-variant decision + its three flip-conditions, the no-`schema_version`-bump / analyzer-local boundary, the symmetric eligibility bar, the symmetric-extraction reuse of the disjoint-set primitive, the `analysis.sanctioned_replan` reuse, and the independent-compute / co-fire contract (anchor-absent must not short-circuit rolling) are explicit and hard to miss
- whether the packet-prompts file now matches the source-of-truth packet numbering (`R6-3.0` through `R6-3.4`)
- whether the docs are consistent with the landed `R6-2` scorer they extend and with live `analysis.previous` / `sanctioned_replan`
- whether the work stayed docs-only and Packet `R6-3.0`-scoped

State clearly whether Packet `R6-3.0` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-3.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-3.0` docs issues
- keep the work docs-only
- rerun the manual review against the MAP / DESIGN / `R6-2` SPEC / live-scorer sources if a finding questions source-of-truth alignment
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, any refreshed doc-alignment evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: lock R6-3 rolling semantic goal drift spec family`
- fix commit message example: `fix: address R6-3.0 review findings`
````

---

## Packet R6-3.1 Prompt — Task R6-3.1.1

````text
/goal Land Packet `R6-3.1` from `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-3.1` only, assuming `R6-3.0` (docs lock) is already committed and no later `R6-3.2+` packet has started. This packet is a confirmation + corpus check; it writes NO scorer code.

Packet `R6-3.1` is a HARD STOP/GO GATE. Its corpus finding decides whether Packet `R6-3.2` may begin the scorer on the symmetric `Medium+` bar. Do not let it become "note-and-proceed": the explicit go/no-go decision must be recorded in the tasks ledger, and if the corpus shows over-fire, the bar must be retuned (tighten eligibility / hold `previous` to a higher bar) or the graduated-distance revisit opened first — do NOT green-light `R6-3.2` on an over-firing bar.

Packet `R6-3.1` scope only:
- (a) Reachability confirmation, from live code: confirm `CheckpointAnalysis` exposes `previous` and that the previous goal is reachable at `analysis.previous.<slice>.context.objective` with the same `ObjectiveSummary` shape the current goal uses, with NO new plumbing and NO signature change to `score_semantic_goal_drift`. Record the exact access path in the tasks ledger. Do not repeat the `R6-2` first-draft mistake of asserting a non-existent access path.
- (b) Analyzer-local confirmation: confirm extending the scorer + adding rolling evidence prefixes touches no `DriftClass` enum, no `export.rs` class list, and no sentinel `operator_surface.rs` source, and needs no `schema_version` bump. Record this as the `R6-3` analogue of `R6-2.2`, but note it is a confirmation, not an impact-gated variant decision.
- (c) Eligibility/threshold corpus check (SPEC Open Question 1): measure, across the fixture corpus, how often adjacent confident `TaskStatement` goals occur and how often they are fully disjoint under the reused distance primitive. Confirm the symmetric `Medium+` bar fires on real abrupt pivots without over-firing on ordinary evolution, OR record that a synthetic acceptance fixture is needed (as `R6-2.4` did) and/or that eligibility tightening / the graduated-distance revisit is warranted.
- record the finding AND the explicit go/no-go decision in the tasks ledger under Task `R6-3.1.1`

Primary files:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (read-only)
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (read-only)
- `crates/agent-drift-analyzer/src/context/objective.rs` (read-only)
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`

Out of scope:
- writing any scorer code (that is Packet `R6-3.2`)
- adding rolling evidence prefixes to source
- any `progress.rs` migration
- Packets `R6-3.2+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before reading/relying on any indexed Rust symbol you will later edit, prefer GitNexus context/impact to confirm the access path rather than grepping; this packet edits no production symbol, so there is no HIGH/CRITICAL edit gate, but the recorded access path must come from live code.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the recorded reachability path, analyzer-local confirmation, corpus finding, and the go/no-go decision yourself.
6. Commit the tasks-ledger update before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
10. Run GitNexus detect-changes before every real commit.
11. Do not start Packet `R6-3.2`. In particular, `R6-3.2` may only begin once this packet's recorded decision is `GO` or `GO-with-synthetic-fixture` on the (possibly retuned) bar; a `NO-GO` (over-fire) blocks `R6-3.2` until the bar is retuned or the graduated-distance revisit is opened. (`GO-with-synthetic-fixture` still proceeds to `R6-3.2`; the synthetic acceptance fixture is added in `R6-3.3`, not a blocker for the scorer.)

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: confirm the previous-checkpoint reachability and analyzer-local surfacing, run the eligibility/threshold corpus check, and record the finding plus an explicit go/no-go decision in the tasks ledger. Write NO scorer code.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- crates/agent-drift-analyzer/src/context/objective.rs
- AGENTS.md

Do this:
- confirm the four `R6-2` prerequisites are landed in live code before proceeding: the `score_semantic_goal_drift` scorer, the `DriftClass::SemanticGoalDrift` variant, `CheckpointAnalysis.sanctioned_replan`, and the `analysis.previous` slice. If any is missing, or `score_semantic_goal_drift(analysis, kickoff_anchor)` does not already receive `analysis` (which carries `.previous`), stop and report
- (a) confirm from live code that the previous goal is reachable at `analysis.previous.<slice>.context.objective` with the same `ObjectiveSummary` shape the current goal uses, with NO new plumbing and NO signature change; record the EXACT access path in the tasks ledger
- (b) confirm the change is analyzer-local: extending the scorer + adding rolling evidence prefixes touches no `DriftClass` enum, no `export.rs` class list, and no sentinel `operator_surface.rs` source, and needs no `schema_version` bump; record this as a confirmation (the `R6-3` analogue of `R6-2.2`), not an impact-gated variant decision
- (c) run the eligibility/threshold corpus check: measure how often adjacent confident `TaskStatement` goals occur across the fixture corpus and how often they are fully disjoint under the reused disjoint-set distance primitive; determine whether the symmetric `Medium+` bar fires on real abrupt pivots without over-firing on ordinary evolution
- record the corpus finding AND an explicit go/no-go decision in Task `R6-3.1.1`: GO (bar fires cleanly), GO-with-synthetic-fixture (abrupt pivots too rare, add a synthetic acceptance fixture in `R6-3.3` as `R6-2.4` did), or NO-GO (over-fire; retune the bar — tighten eligibility / hold `previous` higher — or open the graduated-distance revisit first)
- write NO scorer code and add no rolling evidence prefixes to source in this packet

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- GitNexus detect-changes before handing back for commit

Return with: the exact previous-goal access path, the analyzer-local confirmation, the corpus finding (adjacent-confident frequency + disjointness rate), the explicit go/no-go decision and its rationale, changed files (ideally tasks ledger only), tests run, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-3.1` reachability confirmation, analyzer-local confirmation, and eligibility/threshold corpus check in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and whether the go/no-go decision is trustworthy.

Use the `$code-review-and-quality` skill.

Review only Packet `R6-3.1` against:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- crates/agent-drift-analyzer/src/context/objective.rs
- AGENTS.md

Focus:
- whether the recorded previous-goal access path (`analysis.previous.<slice>.context.objective`) is real in live code, has the same `ObjectiveSummary` shape as the current goal, and needs no new plumbing or signature change
- whether the analyzer-local confirmation is honest: no `DriftClass` enum / `export.rs` / sentinel `operator_surface.rs` source touch and no `schema_version` bump
- whether the corpus check actually measured adjacent-confident frequency and disjointness rate rather than asserting a conclusion
- whether the go/no-go decision follows from the evidence, and whether a NO-GO or GO-with-synthetic-fixture path is correctly recorded if the bar over-fires or abrupt pivots are too rare
- whether the packet stayed no-scorer-code and did not start `R6-3.2`

State clearly whether Packet `R6-3.1` is review-clean or requires changes, and whether the recorded go/no-go decision is safe for Packet `R6-3.2` to rely on. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R6-3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-3.1` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-3.1` issues; keep the work confirmation + corpus-check scoped with no scorer code
- if a finding questions the access path or corpus result, re-derive it from live code / a real corpus run rather than restating it
- if a finding questions the go/no-go decision, correct the decision and its rationale in the tasks ledger
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, refreshed access-path/corpus/go-no-go evidence, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R6-3.1 rolling reachability and corpus go/no-go`
- fix commit message example: `fix: address R6-3.1 review findings`
````

---

## Packet R6-3.2 Prompt — Task R6-3.2.1

````text
/goal Land Packet `R6-3.2` from `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-3.2` only, assuming Packets `R6-3.0` and `R6-3.1` are already landed. Do NOT start Packet `R6-3.2` unless Packet `R6-3.1`'s recorded decision is `GO` or `GO-with-synthetic-fixture` on the (possibly retuned) bar; if it is `NO-GO`, stop and report. If it is `GO-with-synthetic-fixture`, proceed — the synthetic acceptance fixture is added later in Packet `R6-3.3`, not here.

Packet `R6-3.2` scope only:
- extend `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`: read the previous goal from `analysis.previous`, apply the landed `eligible_current_goal` bar to it (symmetric with the current goal), and compute a rolling divergence via the reused disjoint-set primitive with SYMMETRIC extraction (`goal_specific_terms(.., Some(summary))` on both sides)
- do NOT reuse the landed `semantic_goal_diverged` with the previous side passed as a bare `StructuredObjective` — that drops the previous side's `comparison_key` and silently reintroduces the `R6-2` asymmetry (SPEC Resolved Decision 4); use two `EligibleCurrentGoal`-like values or an equivalent symmetric helper
- exclude sanctioned replans via the existing `analysis.sanctioned_replan`
- compute kickoff-anchored and rolling INDEPENDENTLY: restructure the landed `eligible_anchor_goal` early return so that an absent or ineligible kickoff anchor does NOT short-circuit the rolling comparison (rolling must still be able to fire with no anchor)
- flag the single `SemanticGoalDrift` claim when either comparison fires; attach evidence tagged with distinct rolling reason prefixes (e.g. `ROLLING_CURRENT_REASON_PREFIX` / `ROLLING_PREVIOUS_REASON_PREFIX`) so operators can tell rolling from kickoff-anchored
- decide and assert the co-fire evidence ordering / current-goal de-dup (resolves SPEC Open Question 2)
- land the MINIMAL TDD proof only: rolling-flag, rolling-fires-when-kickoff-anchor-absent, first-checkpoint-`None` no-claim, and rolling-vs-replan. The full matrix + acceptance fixture belong to Packet `R6-3.3`

Non-negotiable invariants:
- no new `DriftClass` variant, no `schema_version` bump, no `scoring/mod.rs` signature change
- `DriftScore`'s posture / `raw_score` / `confidence` shape unchanged from `R6-2`
- never read `task_frame.objective` for the drift signal; do not migrate `progress.rs`

Primary files:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
- `crates/agent-drift-analyzer/tests/...` (minimal proof only)

Out of scope:
- the full rolling regression matrix, structured-source symmetric-extraction lock, and acceptance fixture (Packet `R6-3.3`)
- the sentinel evidence-rendering test (Packet `R6-3.3`, Task `R6-3.3.2`)
- `progress.rs` comparability/reset migration (`R6-4`)
- any new `DriftClass` variant or `schema_version` bump
- Packets `R6-3.3+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying `score_semantic_goal_drift`, `eligible_anchor_goal`, or any other indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the diff and verification results yourself — pay special attention that the anchor early return was actually restructured (rolling can fire with no anchor) and that the previous side carries its `ObjectiveSummary` (symmetric extraction).
6. Commit the landed implementation before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
8. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run GitNexus detect-changes before every real commit.
10. Do not start Packet `R6-3.3`.

Required verification:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: extend the semantic-goal-drift scorer with the rolling / previous-checkpoint comparison and rolling-tagged evidence, restructure the anchor early return so rolling is independent, and land the minimal proof.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md (esp. Resolved Decisions 3, 4, 6 and Open Question 2)
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md (including the Packet `R6-3.1` go/no-go finding)
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/context/objective.rs
- AGENTS.md

Do this:
- confirm Packets `R6-3.0` and `R6-3.1` are landed and that `R6-3.1`'s recorded decision is `GO` or `GO-with-synthetic-fixture` (a `NO-GO` blocks this packet); if it is `NO-GO` or the packets are not landed, stop and report
- run GitNexus impact analysis before editing `score_semantic_goal_drift`, `eligible_anchor_goal`, or any other indexed symbol you touch, and report HIGH/CRITICAL blast radius before proceeding
- read the previous goal from `analysis.previous` (no new plumbing, no signature change) and apply the landed `eligible_current_goal` bar to it, symmetric with the current goal; first checkpoint (`analysis.previous` is `None`) or an ineligible previous goal -> conservative no rolling claim
- compute the rolling divergence with SYMMETRIC extraction: `goal_specific_terms(structured, Some(summary))` on BOTH sides. Do NOT reuse `semantic_goal_diverged(cur, prev_structured)` with the previous side as a bare `StructuredObjective`; use two `EligibleCurrentGoal`-like values or an equivalent symmetric term-set helper so the previous side's `comparison_key` is not dropped
- restructure the landed `eligible_anchor_goal` early return so an absent/ineligible kickoff anchor does NOT short-circuit the rolling comparison; kickoff-anchored and rolling must be computed independently and either firing flags the class
- exclude sanctioned explicit replans via `analysis.sanctioned_replan` for both comparisons
- attach evidence under distinct rolling reason prefixes (e.g. `ROLLING_CURRENT_REASON_PREFIX` / `ROLLING_PREVIOUS_REASON_PREFIX`); keep the existing kickoff prefixes unchanged
- decide the co-fire evidence ordering / current-goal de-dup and encode it (resolves Open Question 2); assert it in the minimal proof
- keep `DriftScore`'s posture / `raw_score` / `confidence` shape unchanged; make NO `DriftClass` variant change, NO `schema_version` bump, NO `scoring/mod.rs` signature change; never read `task_frame.objective`
- add only the minimal TDD proof: rolling-flag, rolling-fires-when-kickoff-anchor-absent, first-checkpoint-`None` no-claim, and rolling-vs-replan; leave the full matrix + acceptance fixture for Packet `R6-3.3`

Run:
- `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- GitNexus detect-changes before handing back for commit

Return with: changed files, impact-analysis summary, how the anchor early return was restructured, how symmetric extraction was implemented, the co-fire ordering/de-dup decision, tests run, any residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-3.2` rolling scorer extension in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the previous goal is read from `analysis.previous` with the landed `eligible_current_goal` bar applied symmetrically, and first-checkpoint / ineligible-previous yields a conservative no rolling claim
- whether extraction is SYMMETRIC (`goal_specific_terms(.., Some(summary))` on both sides) and the previous side carries its `ObjectiveSummary` — i.e. the code does NOT reuse `semantic_goal_diverged(cur, prev_structured)` with the previous side as a bare `StructuredObjective` and drop its `comparison_key`
- whether the anchor early return was truly restructured so an absent/ineligible kickoff anchor does NOT short-circuit rolling — rolling must be able to fire with no anchor, and there is a test proving it
- whether kickoff-anchored and rolling are computed independently and either firing flags the single `SemanticGoalDrift` claim, with sanctioned replans excluded for both
- whether rolling evidence uses distinct reason prefixes and the co-fire evidence ordering / current-goal de-dup is decided and asserted
- whether the posture / `raw_score` / `confidence` shape is unchanged, no `DriftClass` variant / `schema_version` / `scoring/mod.rs` signature change landed, and `task_frame.objective` is never read
- whether the minimal proof (rolling-flag, anchor-absent-rolling-fires, first-checkpoint-None, rolling-vs-replan) is present and sufficient for this packet, without pulling forward the full `R6-3.3` matrix

State clearly whether Packet `R6-3.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-3.2` verification commands.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-3.2` issues; keep the full regression matrix and acceptance fixture out of scope
- preserve the load-bearing invariants: symmetric extraction with the previous side's `ObjectiveSummary`; the restructured anchor early return so rolling stays independent; distinct rolling prefixes; sanctioned-replan exclusion; unchanged posture/`raw_score`/`confidence`; no `DriftClass` variant / `schema_version` / `scoring/mod.rs` signature change; never read `task_frame.objective`
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add rolling previous-checkpoint semantic goal drift`
- fix commit message example: `fix: address R6-3.2 review findings`
````

---

## Packet R6-3.3 Prompt — Tasks R6-3.3.1, R6-3.3.2, and R6-3.3.3

````text
/goal Land Packet `R6-3.3` from `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-3.3` only (Tasks `R6-3.3.1`, `R6-3.3.2`, and `R6-3.3.3`), assuming Packets `R6-3.0` through `R6-3.2` are already landed.

Packet `R6-3.3` scope only:
- Task `R6-3.3.1` — complete the rolling regression matrix and commit the acceptance fixture:
  - all rolling guard states: flag / first-checkpoint-`None` / previous-present-but-unknown
  - step-size pivot (adjacent goals fully disjoint) -> flagged vs slow evolution (adjacent goals share one specific term) -> not flagged
  - rolling-vs-sanctioned-replan -> not flagged
  - independence/co-fire: kickoff-only checkpoint still flags with kickoff evidence and rolling silent (proving `R6-2` preserved); rolling-only; both tags co-fire on one claim; AND the load-bearing anchor-absent case — no eligible kickoff anchor (absent or below `High`) + an eligible, disjoint previous goal -> rolling still flags (locking that the anchor early return was restructured, not left short-circuiting rolling)
  - structured-source proof (patched display string and structured goal disagree -> score off the structured goal), INCLUDING the symmetric-extraction lock (SPEC Resolved Decision 4): a previous goal whose only distinguishing term lives in its `comparison_key` (not in `target`/constraints) still drives rolling divergence, so a naive `semantic_goal_diverged(cur, prev_structured)` reuse fails this regression
  - a rolling-drift acceptance fixture (an abrupt mid-session pivot) committed and locked like the `semantic_goal_drift_acceptance` corpus
- Task `R6-3.3.2` — add the sentinel evidence-rendering test (test-only, no sentinel source change): construct a checkpoint with a flagged rolling-tagged `SemanticGoalDrift` score and assert the operator surface renders the rolling evidence line(s) on the existing `SemanticGoalDrift` posture, mirroring the `R6-2` `operator_surface_renders_flagged_semantic_goal_drift_*` coverage. No `operator_surface.rs` source change.
- Task `R6-3.3.3` — assert `R6-2` / `R5.75` / `R6-1` non-regression: the `R6-2` kickoff-anchored acceptance witnesses, `R5.75-3` (delegated stability) / `R5.75-4` (zero-verifier anti-flap) witnesses, and the `R6-1` `dead_end_thrash` posture are unchanged. Any change to a prior witness is a regression, not a rebaseline.

Primary files:
- `crates/agent-drift-analyzer/tests/...` (new rolling acceptance + scorer regressions)
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs` (test-only)

Out of scope:
- changing the scorer semantics beyond minimal test-support adjustments (the scorer landed in Packet `R6-3.2`)
- any `operator_surface.rs` source change, `DriftClass` variant, or `schema_version` bump
- opening `R6-4` or migrating `progress.rs`
- Packet `R6-3.4`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before editing any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. Commit the landed regression + acceptance-fixture + sentinel-rendering + non-regression work before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
7. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
8. Run GitNexus detect-changes before every real commit.
9. Do not start Packet `R6-3.4`.

Required verification (union of Tasks `R6-3.3.1`, `R6-3.3.2`, `R6-3.3.3`; note `R6-3.3.3` requires the FULL sentinel wall, not just the focused `operator_surface` run):

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-3.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: complete the rolling regression matrix, commit the rolling acceptance fixture, add the test-only sentinel evidence-rendering test, and lock the `R6-2`/`R5.75`/`R6-1` non-regressions.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md (Testing Strategy items 1-8; Resolved Decision 4)
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
- crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
- crates/agent-drift-analyzer/tests/progress_acceptance.rs
- crates/agent-drift-sentinel/tests/operator_surface.rs
- AGENTS.md

Do this:
- confirm Packets `R6-3.0` through `R6-3.2` are landed; if not, stop and report
- run GitNexus impact analysis before editing any indexed symbol you touch and report HIGH/CRITICAL blast radius before proceeding
- Task `R6-3.3.1`: add regressions covering every rolling guard state (flag / first-checkpoint-`None` / previous-present-but-unknown); step-size pivot vs slow evolution; rolling-vs-sanctioned-replan; independence/co-fire including kickoff-only (rolling silent), rolling-only, both-co-fire, AND the load-bearing anchor-absent case (no eligible kickoff anchor + eligible disjoint previous goal -> rolling still flags); and the structured-source proof INCLUDING the symmetric-extraction lock (previous goal's only distinguishing term lives in its `comparison_key`, so a naive `semantic_goal_diverged(cur, prev_structured)` reuse fails). Commit a rolling-drift acceptance fixture (abrupt mid-session pivot) locked like `semantic_goal_drift_acceptance`
- Task `R6-3.3.2`: add a sentinel test that constructs a flagged rolling-tagged `SemanticGoalDrift` checkpoint and asserts the operator surface renders the rolling evidence line(s) on the existing `SemanticGoalDrift` posture, mirroring the `R6-2` `operator_surface_renders_flagged_semantic_goal_drift_*` coverage — test-only, NO `operator_surface.rs` source change
- Task `R6-3.3.3`: assert the `R6-2` kickoff-anchored acceptance witnesses, `R5.75-3` / `R5.75-4` witnesses, and the `R6-1` `dead_end_thrash` posture are unchanged; treat any change to a prior witness as a regression, not a rebaseline
- do not re-add Packet `R6-3.2`'s minimal proof redundantly; extend the matrix and acceptance surface from it
- make NO scorer-semantics change, NO `operator_surface.rs` source change, NO `DriftClass` variant / `schema_version` bump

Run:
- `cargo test -p agent-drift-analyzer -- --nocapture`
- `cargo test -p agent-drift-sentinel operator_surface -- --nocapture` (the focused Task `R6-3.3.2` command)
- `cargo test -p agent-drift-sentinel -- --nocapture` (the FULL sentinel wall required by Task `R6-3.3.3`)
- GitNexus detect-changes before handing back for commit

Return with: changed files, tests run, what matrix/fixture/sentinel/non-regression coverage was added (call out the anchor-absent case and the symmetric-extraction lock explicitly), any residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-3.3` rolling regression matrix, acceptance fixture, sentinel rendering test, and non-regression assertions in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether all rolling guard states (flag / first-checkpoint-`None` / previous-present-but-unknown) are asserted and would catch regressions
- whether step-size pivot vs slow evolution and rolling-vs-sanctioned-replan behavior is locked correctly
- whether the independence/co-fire coverage includes the load-bearing anchor-absent case (no eligible kickoff anchor + eligible disjoint previous -> rolling still flags), so a future edit that re-adds the anchor short-circuit would fail
- whether the structured-source proof truly ignores the bridge-patched display string AND the symmetric-extraction lock forces a case where the previous goal's only distinguishing term lives in its `comparison_key` (so a naive `semantic_goal_diverged(cur, prev_structured)` reuse fails)
- whether the rolling acceptance fixture is committed and locked like `semantic_goal_drift_acceptance`
- whether the sentinel rendering test is test-only (no `operator_surface.rs` source change) and asserts the rolling evidence renders on the existing `SemanticGoalDrift` posture
- whether the `R6-2` / `R5.75-3` / `R5.75-4` / `R6-1` non-regression assertions are real and unchanged, and whether the FULL sentinel wall (`cargo test -p agent-drift-sentinel -- --nocapture`) was run for Task `R6-3.3.3`, not just the focused `operator_surface` run
- whether the packet stayed regression/fixture/test-scoped and did not change scorer semantics or drift into `R6-4`

State clearly whether Packet `R6-3.3` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-3.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-3.3` verification commands.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-3.3` issues; keep the work regression/fixture/test-scoped and change no scorer semantics or sentinel source
- preserve the load-bearing coverage: the anchor-absent rolling-fires case and the `comparison_key`-only symmetric-extraction lock
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- rerun `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
- rerun `cargo test -p agent-drift-sentinel -- --nocapture` (the full sentinel wall for `R6-3.3.3` non-regression)
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock rolling semantic goal drift matrix and acceptance fixture`
- fix commit message example: `fix: address R6-3.3 review findings`
````

---

## Packet R6-3.4 Prompt — Task R6-3.4.1

````text
/goal Land Packet `R6-3.4` from `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-3.4` only (closeout), assuming Packets `R6-3.0` through `R6-3.3` are already landed.

Packet `R6-3.4` scope only:
- run the full analyzer wall and the full sentinel wall (the rolling evidence renders through the sentinel surface even though no sentinel source changed)
- confirm, with grep/diff evidence recorded in the tasks ledger, that NO new `DriftClass` variant, NO `schema_version` bump, and NO `export.rs` / sentinel `operator_surface.rs` source change landed
- decide and record whether `R6-4` opens: did any `R6-1` / `R6-2` / `R6-3` replay evidence show a `progress.rs` reset error caused by objective-string quality? If yes, route to conditional `R6-4`; if no, keep `R6-4` deferred to the later full-migration phase
- update the `R6-3` entry in `docs/specs/r6/MAP.md`: promote its status to landed and update the `R6-4` open/defer decision
- if the walls reveal a real packet-scoped defect in already-landed `R6-3.1` through `R6-3.3` work, spawn a fix subagent to repair only that defect, commit it, and rerun the walls

Primary files:
- `docs/specs/r6/MAP.md`

Hard rules:
1. Spawn a fresh implementation/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If validation reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch and the MAP status update before review; do not fabricate empty commits.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues in the MAP update, the no-change confirmation, or the closeout routing, spawn a fresh fix subagent, commit, and re-review.
7. Run GitNexus detect-changes before every real commit.
8. Do not open `R6-4` unless the replay/reset evidence actually justifies it. Do not promote the MAP `R6-3` status to landed until the walls are green and the no-variant/no-schema/no-sentinel-source confirmation holds.

Required verification:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
# and: git diff shows no DriftClass enum / schema_version / export.rs / operator_surface.rs source change
```

Implementation/verification subagent prompt to send:

```text
/goal Land Packet `R6-3.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the full analyzer + sentinel walls, confirm no variant/schema/sentinel-source change landed, make the `R6-4` open/defer decision honestly, and update `docs/specs/r6/MAP.md` for R6-3 closeout.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-spec.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-plan.md
- docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm Packets `R6-3.0` through `R6-3.3` are landed; if not, stop and report
- run the full analyzer and sentinel walls
- produce grep/diff evidence that no new `DriftClass` variant, no `schema_version` bump, and no `export.rs` / sentinel `operator_surface.rs` source change landed across `R6-3` (e.g. `git diff` of `crates/agent-drift-analyzer/src/checkpoint/schema.rs`, `checkpoint/export.rs`, and `crates/agent-drift-sentinel/src/operator_surface.rs` against the pre-`R6-3` base); record it in Task `R6-3.4.1`
- inspect whether any replay/reset evidence from `R6-1` / `R6-2` / `R6-3` shows a `progress.rs` reset error caused by objective-string quality; only if yes should you recommend opening conditional `R6-4`, otherwise recommend deferring it to the later full-migration phase
- update the `R6-3` entry in `docs/specs/r6/MAP.md`: promote status to landed and update the `R6-4` open/defer decision
- if the walls expose a real packet-scoped defect in already-landed `R6-3` work, stop and report that defect clearly so the orchestration agent can decide whether to spawn a fix subagent before closeout review

Run:
- `cargo test -p agent-drift-analyzer -- --nocapture`
- `cargo test -p agent-drift-sentinel -- --nocapture`
- GitNexus detect-changes before handing back for commit if any files changed

Return with: whether the walls passed, the no-variant/no-schema/no-sentinel-source grep/diff evidence, the `R6-4` decision and evidence, whether any fix subagent is needed, changed files, and the exact commit message you recommend if a commit is needed.
```

Fix subagent prompt template if full-wall verification finds a packet-scoped defect:

```text
/goal Address the packet-scoped defect uncovered during Packet `R6-3.4` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the full Packet `R6-3.4` verification walls.

Use the `$incremental-implementation` skill.

Defect to fix:
- [PASTE DEFECT / REVIEW FINDING HERE]

Rules:
- fix only the concrete `R6-3` defect the walls exposed
- do not introduce a `DriftClass` variant, `schema_version` bump, or `export.rs` / `operator_surface.rs` source change while fixing
- if you must edit an indexed Rust symbol, run GitNexus impact analysis first and report HIGH/CRITICAL blast radius
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- rerun `cargo test -p agent-drift-sentinel -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-3.4` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the full analyzer and sentinel walls actually ran and are reported accurately
- whether the no-variant / no-`schema_version` / no-`export.rs` / no-sentinel-`operator_surface.rs`-source confirmation is backed by real grep/diff evidence
- whether the `R6-4` open/defer decision is evidence-based and honest
- whether the `docs/specs/r6/MAP.md` update matches the real packet state, promotes `R6-3` to landed correctly, and records the next routing
- whether any packet-scoped defects found during closeout were resolved and reverified before approval

State clearly whether Packet `R6-3.4` is review-clean or requires changes.
```

Commit guidance:
- closeout commit message example: `docs: close R6-3 rolling semantic goal drift`
- fix commit message example: `fix: address R6-3.4 closeout findings`
````

---

## Deferred / Ask-First (not landed by these packets)

These are recorded in the tasks ledger's "Deferred / Ask-First" section and are **out of scope** for the
orchestration loop above. Do not spawn implementation subagents for them from this file:

- `R6-3.X.1` — promote rolling drift to its own `DriftClass::RollingSemanticGoalDrift` variant. Opened only
  if a SPEC Resolved Decision 1 flip-condition is met (different `raw_score`/threshold/debounce/warning
  policy; separate sentinel labels/actions; or acceptance evidence that aggregate `SemanticGoalDrift`
  reporting hides gradual-vs-abrupt operationally). If opened, it pays the same lockstep forward-compat
  break + `schema_version` bump (`v0.8`) that `R6-2` paid, is impact-gated via `gitnexus_impact`, and needs
  its own SPEC/PLAN/TASKS delta before any packet prompt is written.
- `R6-3.X.2` — graduated/weighted semantic distance (shared with the `R6-2` Resolved Decision 7 debt).
  Opened only if `R6-2`/`R6-3` acceptance evidence shows the binary disjoint-set rule under- or over-fires.
  Deferred to a later `R6` iteration; not written here.

If Packet `R6-3.1`'s corpus check returns NO-GO (over-fire), that is the trigger to consult the ledger on
whether to retune the bar in-packet or open `R6-3.X.2` first — it does not authorize writing `R6-3.2` on the
un-retuned bar.
