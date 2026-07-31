# Semantic Goal Drift (R6-2) Packet Prompts

Status: orchestration prompts created on 2026-07-01 against the live
`docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` ledger. Each prompt below is
self-contained: paste one into a fresh session to land exactly one `R6-2` sub-packet through a
`GPT-5.4` high implementation -> commit -> review -> fix -> commit loop until review-clean.

These prompts map:

- Packet `R6-2.0` -> Task `R6-2.0.1` (docs lock)
- Packet `R6-2.1` -> Task `R6-2.1.1` (capture + thread the kickoff anchor; add `sanctioned_replan`)
- Packet `R6-2.2` -> Task `R6-2.2.1` (impact-gated variant blast radius + `schema_version` decision; no scorer code)
- Packet `R6-2.3` -> Task `R6-2.3.1` (semantic-goal-drift scorer + presence guards + `v0.7` writer/gate fallout + minimal proof)
- Packet `R6-2.4` -> Tasks `R6-2.4.1` and `R6-2.4.2` (regression matrix + acceptance fixture)
- Packet `R6-2.5` -> Task `R6-2.5.1` (full analyzer + sentinel walls, `R6-4` open/defer decision, MAP update)

Shared authority for all packets:

- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`
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
11. Packet prerequisite rule is strict: this family assumes `R5.75-1` and `R6-1` are landed before any
    editing. If a named prerequisite is missing, stop and report it instead of compensating inside the
    packet.
12. `R6-2` reads structured state only. No packet may read the bridge-patched `task_frame.objective` for
    the drift signal, migrate `progress.rs` comparability/reset (that is conditional `R6-4`), or flag a
    sanctioned explicit replan as drift. The drift contract stays: current-goal three-state guard,
    separate anchor-presence guard, and sanctioned-replan exclusion via `analysis.sanctioned_replan`.
13. Do not advance to the next packet until the current packet is committed and a fresh review subagent
    reports it review-clean, or for verification-only packets with no file changes, explicitly reports
    there was nothing to commit.

---

## Packet R6-2.0 Prompt — Task R6-2.0.1

````text
/goal Land Packet `R6-2.0` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.0` only. Do not start Packet `R6-2.1`.

Packet `R6-2.0` scope only:
- commit the bounded `R6-2` SPEC/PLAN/TASKS family and this packet-prompts file under `docs/specs/r6/R6-2/`
- ensure those docs explicitly record the structured-state-only drift-read contract, the current-goal three-state guard plus separate anchor guard, the sanctioned-replan exclusion via `analysis.sanctioned_replan`, and the no-`progress.rs`-migration boundary
- verify the docs against `docs/specs/r6/MAP.md`, `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`, and live `crates/agent-drift-analyzer/src/context/objective.rs`

Primary files:
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-packet-prompts.md`

Out of scope:
- any production code change
- any `R6-2.1+` packet work
- any `progress.rs` migration or scorer implementation

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the docs diff and manual-verification story yourself.
5. Commit the landed Packet `R6-2.0` docs before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run GitNexus detect-changes before every real commit.
10. Do not start Packet `R6-2.1`.

Required verification:

```bash
# manual review against:
# - docs/specs/r6/MAP.md
# - docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
# - crates/agent-drift-analyzer/src/context/objective.rs
```

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-2.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: lock the `R6-2` SPEC/PLAN/TASKS docs and the packet-prompts artifact so they accurately define the semantic-goal-drift packet before implementation begins.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-packet-prompts.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/context/objective.rs
- AGENTS.md

Do this:
- keep the work docs-only and Packet `R6-2.0`-scoped
- ensure the SPEC/PLAN/TASKS family clearly records: structured-state-only drift reads, the current-goal three-state guard plus separate anchor guard, the sanctioned-replan exclusion via `analysis.sanctioned_replan`, and the no-`progress.rs`-migration boundary
- ensure the packet-prompts artifact matches the live source-of-truth packet numbering and constraints
- do not start implementation for Packet `R6-2.1+`

Run:
- manual review against `docs/specs/r6/MAP.md`, `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`, and live `crates/agent-drift-analyzer/src/context/objective.rs`
- GitNexus detect-changes before handing back for commit

Return with: changed files, what doc truth was locked, any remaining ambiguity, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-2.0` docs lock in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R6-2.0` against:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-packet-prompts.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- crates/agent-drift-analyzer/src/context/objective.rs
- AGENTS.md

Focus:
- whether the docs accurately lock the packet contract before implementation
- whether the structured-state-only drift-read contract, the current-goal three-state guard plus separate anchor guard, the `analysis.sanctioned_replan` exclusion, and the no-`progress.rs` boundary are explicit and hard to miss
- whether the packet-prompts file now matches the source-of-truth packet numbering (`R6-2.0` through `R6-2.5`)
- whether the work stayed docs-only and Packet `R6-2.0`-scoped

State clearly whether Packet `R6-2.0` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-2.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-2.0` docs issues
- keep the work docs-only
- rerun the manual review against the design/MAP/live-objective sources if a finding questions source-of-truth alignment
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, any refreshed doc-alignment evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: lock R6-2 semantic goal drift spec family`
- fix commit message example: `fix: address R6-2.0 review findings`
````

---

## Packet R6-2.1 Prompt — Task R6-2.1.1

````text
/goal Land Packet `R6-2.1` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.1` only, assuming `R6-2.0` (docs lock) is already committed and no later `R6-2.2+` packet has started.

Packet `R6-2.1` scope only:
- capture the session kickoff structured-goal anchor from the first confident `TaskStatement` checkpoint goal, read once and reused, without recomputing objective extraction
- thread that anchor into `score_session` through the per-session analyze loop in the settled access path (mirroring `previous_truth_grounding_gap`), while the current goal remains read from `analysis.current`
- add `CheckpointAnalysis.sanctioned_replan`, computed at analysis-assembly time from steer-row evidence, and ensure every construction site including fixtures sets it
- record the anchor source, access path, confidence-bar corpus check, and `sanctioned_replan` derivation in the tasks ledger under Task `R6-2.1.1`

Primary files:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/src/scoring/mod.rs`
- `crates/agent-drift-analyzer/src/context/objective.rs` (read-only)
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`

Out of scope:
- adding the semantic-goal-drift scorer itself (that is Packet `R6-2.3`)
- deciding or landing the `SemanticGoalDrift` variant blast radius work beyond what is needed for this packet (that is Packet `R6-2.2`)
- migrating `progress.rs` onto `comparison_key`
- Packets `R6-2.2+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before editing any indexed Rust symbol, the subagent must run GitNexus impact analysis and report any HIGH or CRITICAL blast radius.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the recorded findings, diff, and verification yourself.
6. Commit the landed Packet `R6-2.1` work before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
10. Run GitNexus detect-changes before every real commit.
11. Do not start Packet `R6-2.2`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-2.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: capture and thread the kickoff structured-goal anchor, add `sanctioned_replan`, and record the required findings in the tasks ledger.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm `R5.75-1` and `R6-1` are already landed before editing; if not, stop and report
- run GitNexus impact analysis before editing the indexed symbols you touch and report HIGH/CRITICAL blast radius before proceeding
- in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, add the minimal helper that captures the kickoff anchor from the first confident `TaskStatement` `structured_objective`, read once and reused
- in `crates/agent-drift-analyzer/src/lib.rs` and `src/scoring/mod.rs`, thread that anchor into `score_session` through the per-session analyze loop, mirroring `previous_truth_grounding_gap`; do not invent `session_kickoff_anchor(analysis)` and do not stamp the session-level anchor onto every `CheckpointAnalysis`
- add `CheckpointAnalysis.sanctioned_replan`, derived at analysis-assembly time from steer-row evidence, and update every construction site including fixtures to set it
- perform the confidence-bar corpus check the packet requires and record the result in Task `R6-2.1.1` along with the anchor source, access path, and `sanctioned_replan` derivation
- keep the work additive and scoped to Packet `R6-2.1`; do not add the semantic-goal-drift scorer yet

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- GitNexus detect-changes before handing back for commit

Return with: changed files, the anchor-source/access-path finding, the confidence-bar corpus result, how `sanctioned_replan` is derived, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R6-2.1` kickoff-anchor and sanctioned-replan plumbing in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R6-2.1` against:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md
- docs/specs/r6/MAP.md
- AGENTS.md

Focus:
- whether the kickoff anchor truly comes from the first confident `TaskStatement` checkpoint goal and is read once rather than recomputed
- whether the access path is the settled session-level threading into `score_session`, not a fake `session_kickoff_anchor(analysis)` path or a stamped-per-checkpoint copy
- whether `CheckpointAnalysis.sanctioned_replan` is computed at analysis-assembly time from steer-row evidence and every constructor/fixture sets it
- whether the confidence-bar corpus check and packet findings are actually recorded in the tasks ledger
- whether the packet stayed scoped and did not start the scorer or `progress.rs` migration

State clearly whether Packet `R6-2.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R6-2.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-2.1` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-2.1` issues
- keep the session-level anchor threading and steer-row-derived `sanctioned_replan` contract intact
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: thread kickoff anchor into semantic drift scoring`
- fix commit message example: `fix: address R6-2.1 review findings`
````

---

## Packet R6-2.2 Prompt — Task R6-2.2.1

````text
/goal Land Packet `R6-2.2` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.2` only, assuming Packets `R6-2.0` and `R6-2.1` are already landed.

Packet `R6-2.2` scope only:
- run `gitnexus_impact` on `score_session` and the `SemanticGoalDrift` `DriftClass` variant
- record the full blast radius in the tasks ledger: schema `DriftClass`, analyzer `checkpoint/export.rs` class lists/labels, analyzer sort order, sentinel `operator_surface.rs` mappings, and any `schema_version` gate or lockstep deploy implications
- decide and record whether `schema_version` should bump to `v0.7` for honest labeling / sentinel gating, explicitly noting that the bump is not compat protection
- commit no scorer code from this packet

Primary files:
- `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs` (read-only unless a review finding later proves the packet wording is wrong)
- `crates/agent-drift-analyzer/src/checkpoint/export.rs` (read-only)
- `crates/agent-drift-sentinel/src/operator_surface.rs` (read-only)

Out of scope:
- implementing the scorer or any production logic
- landing the `SemanticGoalDrift` variant itself
- changing code outside packet-scoped evidence capture if the impact report can be recorded without it
- Packets `R6-2.3+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the recorded blast-radius report and schema decision yourself.
5. Commit the tasks-ledger update before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run GitNexus detect-changes before every real commit.
10. Do not start Packet `R6-2.3`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-2.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: confirm the `SemanticGoalDrift` variant blast radius and record the `schema_version` decision in the tasks ledger, with no scorer code.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm Packets `R6-2.0` and `R6-2.1` are landed; if not, stop and report
- run `gitnexus_impact` on `score_session` and on the `SemanticGoalDrift` `DriftClass` variant
- record the full blast radius under Task `R6-2.2.1`, explicitly covering schema `DriftClass`, analyzer sort order, `checkpoint/export.rs` class lists/labels, sentinel `operator_surface.rs` mappings (`drift_class_name`, `historical_reason_prefixes`, `checkpoint_had_active_class`), and the forward-compat / lockstep-deploy implications
- decide and record whether `schema_version` should bump to `v0.7`, making clear it is an honest-labeling / sentinel-gate decision rather than compat protection
- do not commit scorer code or other production logic from this packet

Run:
- GitNexus detect-changes before handing back for commit

Return with: the full blast-radius report, the `schema_version` decision and reasoning, changed files (ideally tasks ledger only), and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-2.2` impact report and schema-version decision in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the impact report actually covers `score_session`, the `SemanticGoalDrift` variant, `checkpoint/export.rs`, `operator_surface.rs`, analyzer sort order, and schema implications
- whether the `schema_version` decision is justified and honestly describes the compat story
- whether the packet stayed no-scorer-code and packet-scoped
- whether the tasks-ledger wording is precise enough for the downstream Packet `R6-2.3` agent to rely on

State clearly whether Packet `R6-2.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-2.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-2.2` evidence/wording issues
- keep the packet no-scorer-code unless a review finding proves a packet-scoped correction is absolutely required
- rerun the needed GitNexus impact call(s) if a finding questions the blast radius
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, any refreshed impact evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R6-2.2 semantic drift blast radius`
- fix commit message example: `fix: address R6-2.2 review findings`
````

---

## Packet R6-2.3 Prompt — Task R6-2.3.1

````text
/goal Land Packet `R6-2.3` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.3` only, assuming Packets `R6-2.0` through `R6-2.2` are already landed.

Packet `R6-2.3` scope only:
- add `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` with the two presence guards first: current-goal three-state guard and separate anchor-presence guard
- compare the current goal from `analysis.current` to the threaded kickoff anchor using `comparison_key` / structured terms only, excluding sanctioned explicit replans via `analysis.sanctioned_replan`
- emit the new `DriftClass::SemanticGoalDrift`, with named evidence for the anchor and drifted goal, and wire it into `score_session`
- land the lockstep analyzer/sentinel surfacing updates required by the Packet `R6-2.2` blast-radius decision, including the `crates/agent-drift-analyzer/src/checkpoint/mod.rs` `v0.7` writer bump and the analyzer/sentinel schema gates that must move with it
- add only the minimal presence-guard + drift-vs-replan proof here; the full regression matrix belongs to Packet `R6-2.4`

Primary files:
- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
- `crates/agent-drift-analyzer/src/scoring/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`
- `crates/agent-drift-analyzer/tests/export_bundle.rs`
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-analyzer/tests/...` (minimal proof only)

Out of scope:
- the full presence-guard matrix, structured-source proof, and acceptance fixture (that is Packet `R6-2.4`)
- `progress.rs` comparability/reset migration (`R6-4`)
- any fallback to `task_frame.objective`
- Packets `R6-2.4+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying `score_session`, the new scorer, the `DriftClass` enum, or any other indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the diff and verification results yourself.
6. Commit the landed implementation before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
8. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run GitNexus detect-changes before every real commit.
10. Do not start Packet `R6-2.4`.

Required verification:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-2.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: land the semantic-goal-drift scorer, the presence guards, the lockstep `SemanticGoalDrift` surfacing updates, and the minimal proof.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-spec.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-plan.md
- docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md (including the Packet `R6-2.1` and `R6-2.2` findings)
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm Packets `R6-2.0` through `R6-2.2` are landed; if not, stop and report
- run GitNexus impact analysis before editing `score_session`, `DriftClass`, or any other indexed symbol you touch, and report HIGH/CRITICAL blast radius before proceeding
- add `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` so the first thing it does is the current-goal three-state guard (present+confident -> score path eligible; absent -> no claim; present-but-unknown -> no claim) and the separate anchor guard (no confident anchor -> no claim)
- compare the current goal from `analysis.current` to the threaded kickoff anchor using `comparison_key` / structured terms only; never read `task_frame.objective`
- exclude sanctioned explicit replans via `analysis.sanctioned_replan`
- emit `DriftClass::SemanticGoalDrift`, attach named evidence for the anchor and drifted goal, and update `scoring/mod.rs`, `checkpoint/mod.rs`, `checkpoint/schema.rs`, `checkpoint/export.rs`, `agent-drift-sentinel/src/input.rs`, `agent-drift-sentinel/src/live_input.rs`, and `agent-drift-sentinel/src/operator_surface.rs` in lockstep per the Packet `R6-2.2` blast-radius decision
- bump the checkpoint writer to `schema_version: "v0.7"` and move the class-list / schema-version fallout surfaces with it: analyzer `tests/end_to_end.rs` + `tests/export_bundle.rs`, plus sentinel `tests/replay_input.rs`, `tests/live_checkpoint_compatibility.rs`, `tests/live_end_to_end.rs`, and `tests/operator_surface.rs`
- add only the minimal presence-guard + drift-vs-replan proof needed for this packet; leave the full matrix and acceptance fixture for Packet `R6-2.4`

Run:
- `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- GitNexus detect-changes before handing back for commit

Return with: changed files, impact-analysis summary, tests run, any residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-2.3` semantic-goal-drift scorer implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the scorer applies the current-goal three-state guard and separate anchor guard first, with no silent fallback to `task_frame.objective`
- whether the current goal comes from `analysis.current`, the anchor comes from the threaded session-level input, and sanctioned replans are excluded via `analysis.sanctioned_replan`
- whether the `SemanticGoalDrift` variant, `checkpoint/mod.rs` writer bump, `checkpoint/export.rs`, and sentinel schema/operator-surface updates are lockstep and consistent with Packet `R6-2.2`
- whether the class-list / schema-version fallout surfaces (`tests/end_to_end.rs`, `tests/export_bundle.rs`, `tests/replay_input.rs`, `tests/live_checkpoint_compatibility.rs`, `tests/live_end_to_end.rs`, `tests/operator_surface.rs`) moved with the variant
- whether the evidence naming is clear and the minimal proof is sufficient for this packet
- whether the work stayed packet-scoped and did not pull forward Packet `R6-2.4` / `R6-4`

State clearly whether Packet `R6-2.3` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-2.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-2.3` verification commands.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-2.3` issues; keep the full regression matrix and acceptance fixture out of scope
- if the finding touches the Packet `R6-2.2` blast-radius fallout, keep the `checkpoint/mod.rs` `v0.7` writer bump, analyzer/sentinel schema gates, and the named class-list / schema-version test surfaces in sync
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add semantic goal drift scorer`
- fix commit message example: `fix: address R6-2.3 review findings`
````

---

## Packet R6-2.4 Prompt — Tasks R6-2.4.1 and R6-2.4.2

````text
/goal Land Packet `R6-2.4` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.4` only (Tasks `R6-2.4.1` and `R6-2.4.2`), assuming Packets `R6-2.0` through `R6-2.3` are already landed.

Packet `R6-2.4` scope only:
- complete the presence-guard matrix: present+confident -> score path, sidecar absent -> no claim, present-but-unknown -> no claim, with tests asserting each state
- assert unauthorized pivot -> flagged with anchor-naming evidence, sanctioned explicit replan -> not flagged, and structured-source proof where patched display string and structured goal disagree still scores off the structured goal
- add the kickoff-anchored drift acceptance fixture and assert non-regression for the `R5.75-3` / `R5.75-4` witnesses and the `R6-1` `dead_end_thrash` posture
- do not re-add Packet `R6-2.3`'s minimal proof redundantly; extend the matrix and acceptance surface from it

Primary files:
- `crates/agent-drift-analyzer/tests/...`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

Out of scope:
- changing the scorer semantics beyond minimal test-support adjustments
- opening `R6-4` or migrating `progress.rs`
- Packet `R6-2.5`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before editing any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. Commit the landed regression + acceptance-fixture work before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run GitNexus detect-changes before every real commit.
8. Do not start Packet `R6-2.5`.

Required verification:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-2.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: complete the semantic-goal-drift regression matrix, add the kickoff-anchored acceptance fixture, and lock the required non-regressions.

Use the `$incremental-implementation` skill.

Read first:
- the R6-2 spec/plan/tasks docs
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm Packets `R6-2.0` through `R6-2.3` are landed; if not, stop and report
- run GitNexus impact analysis before editing any indexed symbol you touch and report HIGH/CRITICAL blast radius before proceeding
- complete the presence-guard matrix without re-adding Packet `R6-2.3`'s minimal proof redundantly
- add tests proving unauthorized pivot -> flagged with anchor-naming evidence; sanctioned explicit replan -> not flagged; structured-source proof uses structured goal / `comparison_key` rather than the bridge-patched display string
- commit the kickoff-anchored drift acceptance fixture and wire it into the harness, locked like `objective_acceptance`
- assert `R5.75-3` / `R5.75-4` witnesses and the `R6-1` `dead_end_thrash` posture are unchanged
- keep the work regression/fixture-scoped; do not open `R6-4`

Run:
- `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- GitNexus detect-changes before handing back for commit

Return with: changed files, tests run, what matrix/fixture coverage was added, any residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-2.4` regression matrix and acceptance fixture in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether all three presence-guard states are asserted and would catch regressions
- whether unauthorized pivot vs sanctioned replan behavior is locked correctly
- whether the structured-source proof truly ignores the bridge-patched display string
- whether the acceptance fixture is committed and the `R5.75-3` / `R5.75-4` + `R6-1` non-regression assertions are real
- whether the packet stayed regression/fixture-scoped and did not drift into `R6-4`

State clearly whether Packet `R6-2.4` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-2.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-2.4` verification commands.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-2.4` issues; keep the work regression/fixture-scoped
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock semantic goal drift matrix and acceptance fixture`
- fix commit message example: `fix: address R6-2.4 review findings`
````

---

## Packet R6-2.5 Prompt — Task R6-2.5.1

````text
/goal Land Packet `R6-2.5` from `docs/specs/r6/R6-2/agent-drift-analyzer-semantic-goal-drift-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-2.5` only (closeout), assuming Packets `R6-2.0` through `R6-2.4` are already landed.

Packet `R6-2.5` scope only:
- run the full analyzer wall and the full sentinel wall, because the `SemanticGoalDrift` variant touches the sentinel surface
- decide and record whether `R6-4` opens: if replay evidence from `R6-1` / `R6-2` shows `progress.rs` reset errors caused by objective-string quality, route to conditional `R6-4`; otherwise defer `R6-4` to the later full-migration phase
- update the `R6-2` entry in `docs/specs/r6/MAP.md` with status, routing, and the `R6-4` open/defer decision
- if the walls reveal a real packet-scoped defect in already-landed `R6-2.1` through `R6-2.4` work, spawn a fix subagent to repair only that defect, commit it, and rerun the walls

Primary files:
- `docs/specs/r6/MAP.md`

Hard rules:
1. Spawn a fresh implementation/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If validation reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch and the MAP status update before review; do not fabricate empty commits.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues in the MAP update or closeout routing, spawn a fresh fix subagent, commit, and re-review.
7. Run GitNexus detect-changes before every real commit.
8. Do not open `R6-4` unless the replay/reset evidence actually justifies it.

Required verification:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Land Packet `R6-2.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the full analyzer + sentinel walls, make the `R6-4` open/defer decision honestly, and update `docs/specs/r6/MAP.md` for R6-2 closeout.

Use the `$incremental-implementation` skill.

Read first:
- the R6-2 spec/plan/tasks docs
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- confirm Packets `R6-2.0` through `R6-2.4` are landed; if not, stop and report
- run the full analyzer and sentinel walls
- inspect whether any replay/reset evidence from `R6-1` / `R6-2` shows a `progress.rs` reset error caused by objective-string quality; only if yes should you recommend opening conditional `R6-4`, otherwise recommend deferring it to the later full-migration phase
- update the `R6-2` entry in `docs/specs/r6/MAP.md` with status, routing, and the `R6-4` decision
- if the walls expose a real packet-scoped defect in already-landed `R6-2` work, stop and report that defect clearly so the orchestration agent can decide whether to spawn a fix subagent before closeout review

Run:
- `cargo test -p agent-drift-analyzer -- --nocapture`
- `cargo test -p agent-drift-sentinel -- --nocapture`
- GitNexus detect-changes before handing back for commit if any files changed

Return with: whether the walls passed, the `R6-4` decision and evidence, whether any fix subagent is needed, changed files, and the exact commit message you recommend if a commit is needed.
```

Fix subagent prompt template if full-wall verification finds a packet-scoped defect:

```text
/goal Address the packet-scoped defect uncovered during Packet `R6-2.5` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the full Packet `R6-2.5` verification walls.

Use the `$incremental-implementation` skill.

Defect to fix:
- [PASTE DEFECT / REVIEW FINDING HERE]

Rules:
- fix only the concrete Packet `R6-2` defect the walls exposed
- if you must edit an indexed Rust symbol, run GitNexus impact analysis first and report HIGH/CRITICAL blast radius
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- rerun `cargo test -p agent-drift-sentinel -- --nocapture`
- run GitNexus detect-changes before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-2.5` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the full analyzer and sentinel walls actually ran and are reported accurately
- whether the `R6-4` open/defer decision is evidence-based and honest
- whether the `docs/specs/r6/MAP.md` update matches the real packet state and next routing
- whether any packet-scoped defects found during closeout were resolved and reverified before approval

State clearly whether Packet `R6-2.5` is review-clean or requires changes.
```

Commit guidance:
- closeout commit message example: `docs: close R6-2 semantic goal drift`
- fix commit message example: `fix: address R6-2.5 closeout findings`
````
