# Dead-End-Thrash Cutover (R6-1) Packet Prompts

Status: orchestration prompts created on 2026-06-27 against the live
`docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` ledger. Each prompt below is
self-contained: paste one into a fresh session to land exactly one `R6-1` sub-packet through a
`GPT-5.4` high implementation -> commit -> review -> fix -> commit loop until review-clean.

Packet `R6-1.0` (docs lock) is committing the `R6-1` SPEC/PLAN/TASKS family plus this packet-prompts
file. It needs no subagent loop — the orchestration agent commits those docs first (suggested message
`docs: lock R6-1 dead-end-thrash cutover spec/plan/tasks`), runs `gitnexus_detect_changes()` before the
commit, then starts Packet 1. These prompts map:

- Packet 1 → Task `R6-1.1` (characterize the frontier predicate; investigation/docs-only)
- Packet 2 → Task `R6-1.2` (dead-end-thrash cutover; the core scorer change)
- Packet 3 → Task `R6-1.3` (churn-vs-stall regressions + frozen-corpus/`R5.75` invariance)
- Packet 4 → Task `R6-1.4` (Guardrail-5 bridge coverage)
- Packet 5 → Task `R6-1.5` (full + sentinel walls + MAP status update; closeout)

Shared authority for all packets:

- `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-spec.md`
- `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-plan.md`
- `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md`
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
   report any HIGH or CRITICAL blast radius before proceeding. Run `gitnexus_detect_changes()` before
   every real commit.
10. `R6-1` is objective-independent: no subagent may read the structured objective (that is `R6-2`),
    migrate the reset/comparability surface onto `comparison_key` (that is the conditional `R6-3`), edit
    the `progress.rs` frontier *computation* (read-only here; exposing/plumbing the already-computed
    signal per `R6-1.1` is allowed, editing the model is ask-first), add a `DriftClass` variant, or bump
    the schema.
11. Do not advance to the next packet until the current packet is committed and a fresh review subagent
    reports it review-clean, or for verification-only packets with no file changes, explicitly reports
    there was nothing to commit.

---

## Packet 1 Prompt — Task R6-1.1

````text
/goal Land Packet `R6-1.1` from `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-1.1` only, assuming `R6-1.0` (docs lock) is already committed and no later `R6-1.2+` packet has started.

Packet `R6-1.1` scope only:
- determine, by read-only probing, which existing `CheckpointAnalysis` / `checkpoint/progress.rs` signal(s) cleanly express "frontier advanced in this interval" vs "no frontier movement" (e.g. the existing `frontier_advanced` / `frontier_rank` / troubleshooting-frontier signals)
- confirm the candidate predicate leaves the frozen `dead_end_thrash` replay corpus unmoved: `019e93fa-60d4-73d1-9092-014130b60e14`, `019e940c-a91b-7fe0-a967-b0bdd595b581`, `019e943c-668e-7a03-992b-6a98cf3055da` stay cleared/`raw_score=0`; `019e894a-86c9-71e3-b57b-e3d3285f0988` stays recovered/`raw_score=20`/`flagged=false`
- confirm the `R5.75-3` and `R5.75-4` witnesses are unaffected
- record the chosen predicate and the confirmation in the tasks ledger under Task `R6-1.1.1`

Out of scope:
- any production scorer edit (that is Packet `R6-1.2`)
- any structured-objective read
- any change to `progress.rs` frontier computation
- Packets `R6-1.2+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the recorded finding yourself.
5. Commit the tasks-ledger finding before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not start Packet `R6-1.2`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R6-1.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: characterize the frontier-movement predicate the dead-end-thrash cutover will use, and record it in the tasks ledger.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-spec.md
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-plan.md
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md
- docs/specs/r6/MAP.md
- AGENTS.md

Do this (read-only investigation; no production code):
- inspect `crates/agent-drift-analyzer/src/checkpoint/progress.rs` and `src/scoring/dead_end_thrash.rs`
- identify the existing signal(s) on `CheckpointAnalysis` that express "frontier advanced" vs "no frontier movement"
- confirm the candidate predicate leaves the frozen `dead_end_thrash` corpus posture unchanged (019e93fa / 019e940c / 019e943c cleared/0; 019e894a recovered/20/unflagged) and the R5.75-3 / R5.75-4 witnesses unaffected
- record the chosen predicate, the signals it reads, and the confirmation under Task R6-1.1.1 in the tasks ledger

Precondition check:
- confirm R6-1.0 docs lock is committed; if a prerequisite is missing, stop and report it instead of compensating here

Return with: the chosen frontier predicate, the signals it reads, the corpus/witness confirmation, changed files (tasks ledger only), and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R6-1.1` frontier-predicate characterization in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R6-1.1` against:
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-spec.md
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md
- docs/specs/r6/MAP.md
- AGENTS.md

Focus:
- whether the chosen frontier predicate is grounded in real existing signals, not invented
- whether the corpus-invariance claim is actually checked, not asserted
- whether the packet stayed docs-only (no production scorer edit, no objective read, no progress.rs change)

State clearly whether Packet `R6-1.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R6-1.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-1.1` evidence/wording issues; keep it docs-only
- re-confirm the corpus/witness invariance if a finding questions it
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, any re-confirmation evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R6-1.1 frontier-predicate characterization`
- fix commit message example: `fix: address R6-1.1 review findings`
````

---

## Packet 2 Prompt — Task R6-1.2

````text
/goal Land Packet `R6-1.2` from `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-1.2` only, assuming Packets `R6-1.0` and `R6-1.1` are already landed.

Packet `R6-1.2` scope only:
- re-score `score_dead_end_thrash` in `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs` to be frontier-aware (decisive-step), using the frontier predicate characterized in Packet `R6-1.1`
- repeated activity with an advancing frontier yields `flagged=false` (historical context) with churn-with-progress evidence
- repeated activity with no frontier movement flags with stall-named evidence, scored by decisiveness rather than streak length alone
- keep `DriftScore` / `DriftClass` / `score_session` shapes unchanged; attach a named `EvidenceRef` to every decision
- add the minimal churn-vs-stall proof in `tests/dead_end_thrash.rs` (the full matrix is Packet `R6-1.3`)

Primary files:
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- `crates/agent-drift-analyzer/tests/dead_end_thrash.rs` (minimal proof only)

Out of scope:
- any structured-objective read; any `progress.rs` frontier change (read-only consumer)
- a new `DriftClass` variant or schema bump
- the full regression matrix and corpus-invariance assertions (Packet `R6-1.3`)
- Packets `R6-1.3+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying `score_dead_end_thrash` (an indexed symbol), the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the diff and verification results yourself.
6. Commit the landed implementation before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
8. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not start Packet `R6-1.3`.

Required verification:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-1.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: re-score dead_end_thrash to be frontier-aware (decisive-step, not streak-length).

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-spec.md
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-plan.md
- docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md (note the Packet R6-1.1 frontier-predicate finding)
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- run GitNexus impact analysis on `score_dead_end_thrash` and report HIGH/CRITICAL blast radius before editing
- in `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`, use the Packet R6-1.1 frontier predicate so an advancing frontier suppresses the dead-end flag (flagged=false, historical context, churn-with-progress evidence) and a stall (no frontier movement) flags with stall-named evidence scored by decisiveness
- keep DriftScore / DriftClass / score_session shapes unchanged; attach a named EvidenceRef to every decision
- add the minimal churn-vs-stall proof in `tests/dead_end_thrash.rs`; do not read the structured objective or change progress.rs

Precondition check:
- confirm Packets R6-1.0 and R6-1.1 are landed; if a prerequisite is missing, stop and report it

Run:
- `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
- `cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture`
- `gitnexus_detect_changes()` before handing back for commit

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-1.2` dead-end-thrash cutover in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether an advancing frontier correctly suppresses the dead-end flag and a stall correctly flags with stall-named evidence
- whether scoring moved from streak length toward decisiveness without losing interpretability
- whether DriftScore / DriftClass / score_session shapes are unchanged and no objective read / progress.rs change crept in
- whether the frozen `acceptance_fixtures` corpus still passes
- whether the verification story is sufficient

Review against the R6-1 spec/plan/tasks docs, docs/specs/r6/MAP.md, and AGENTS.md.

State clearly whether Packet `R6-1.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-1.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-1.2` verification commands.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-1.2` issues; keep the work narrow and objective-independent
- run GitNexus impact analysis before editing any affected indexed symbol
- rerun `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture` and `cargo test -p agent-drift-analyzer --test acceptance_fixtures -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: re-score dead_end_thrash on frontier movement`
- fix commit message example: `fix: address R6-1.2 review findings`
````

---

## Packet 3 Prompt — Task R6-1.3

````text
/goal Land Packet `R6-1.3` from `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-1.3` only (Tasks `R6-1.3.1` and `R6-1.3.2`), assuming Packets `R6-1.0` through `R6-1.2` are already landed.

Packet `R6-1.3` scope only:
- complete the churn-vs-stall matrix in `tests/dead_end_thrash.rs` (do not re-add Packet R6-1.2's minimal proof): advancing-frontier-with-repeated-failures -> flagged=false; repeated-activity-no-movement -> flagged with stall evidence; a single decisive stuck step scored as decisive
- assert frozen-corpus invariance in `tests/acceptance_fixtures.rs` (controls cleared/0; sticky recovered/20/unflagged)
- assert `R5.75-3` / `R5.75-4` witnesses unchanged in `tests/checkpoints.rs` / `tests/progress_acceptance.rs`

Primary files:
- `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
- `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

Out of scope:
- production-logic changes beyond minimal test-support adjustments
- objective reads / progress.rs changes / new DriftClass variant
- Packets `R6-1.4+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed symbol.
4. Commit the landed regression work before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R6-1.4`.

Required verification:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-1.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: complete the churn-vs-stall regression matrix and assert frozen-corpus + R5.75 invariance.

Use the `$incremental-implementation` skill.

Read first:
- the R6-1 spec/plan/tasks docs
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- in `tests/dead_end_thrash.rs`, complete the matrix (advancing-frontier-not-flagged; no-movement-flagged with stall evidence; single decisive stuck step scored as decisive) without re-adding Packet R6-1.2's minimal proof
- in `tests/acceptance_fixtures.rs`, assert the frozen dead_end_thrash corpus keeps posture (019e93fa / 019e940c / 019e943c cleared/0; 019e894a recovered/20/unflagged)
- in `tests/checkpoints.rs` and `tests/progress_acceptance.rs`, assert the R5.75-3 / R5.75-4 witnesses are unchanged
- keep assertions conservative and packet-scoped; no production-logic change beyond minimal test support

GitNexus requirements:
- run impact analysis before editing any indexed symbol; report HIGH/CRITICAL blast radius
- run `gitnexus_detect_changes()` before handing back for commit

Run:
- `cargo test -p agent-drift-analyzer -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-1.3` regression work in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the churn-vs-stall matrix actually locks the intended frontier-aware behavior
- whether the frozen corpus and R5.75-3 / R5.75-4 invariance assertions are real and would catch a regression
- whether the work stayed test-scoped and conservative
- whether the verification story is sufficient

State clearly whether Packet `R6-1.3` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-1.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-1.3` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-1.3` issues; keep it test-scoped
- run impact analysis before editing affected indexed symbols
- rerun `cargo test -p agent-drift-analyzer -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock churn-vs-stall + corpus invariance for dead_end_thrash`
- fix commit message example: `fix: address R6-1.3 review findings`
````

---

## Packet 4 Prompt — Task R6-1.4

````text
/goal Land Packet `R6-1.4` from `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-1.4` only, assuming Packets `R6-1.0` through `R6-1.3` are already landed.

Packet `R6-1.4` scope only:
- add regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs` that exercise the `R5.75-6` bridge needle lists (`anchor_text_looks_grounded_goal`, `narrowed_objective_looks_subordinate`) so a phrasing the bridge misses fails the wall (caught as a bridge gap, not silently mis-scored)
- record, in the packet closeout note, that legacy-surface string-truth is explicit migration debt, not target state (Guardrail 5)

Primary files:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

Out of scope:
- changing the bridge logic itself in `checkpoint/mod.rs` (this packet adds coverage, it does not rewrite the bridge)
- objective reads in the scorer / progress.rs changes / new DriftClass variant
- Packet `R6-1.5`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed symbol.
4. Commit the landed coverage before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R6-1.5`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R6-1.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: add Guardrail-5 acceptance coverage for the R5.75-6 objective bridge.

Use the `$incremental-implementation` skill.

Read first:
- the R6-1 spec/plan/tasks docs
- docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md (Guardrail 5)
- the R5.75-6 bridge in `crates/agent-drift-analyzer/src/checkpoint/mod.rs` (anchor_text_looks_grounded_goal / narrowed_objective_looks_subordinate)
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- in `tests/checkpoints.rs`, add regressions that exercise the bridge needle lists so a novel phrasing the bridge misses fails the wall as a bridge gap
- do not rewrite the bridge logic; this packet adds coverage only
- record in the R6-1 tasks closeout that legacy-surface string-truth is explicit migration debt, not target state

GitNexus requirements:
- run impact analysis before editing any indexed symbol; report HIGH/CRITICAL blast radius
- run `gitnexus_detect_changes()` before handing back for commit

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R6-1.4` bridge coverage in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the new regressions actually exercise the R5.75-6 bridge needle lists and would fail on a phrasing the bridge misses
- whether the packet added coverage only (did not rewrite the bridge)
- whether the migration-debt note is recorded honestly
- whether the verification story is sufficient

State clearly whether Packet `R6-1.4` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R6-1.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R6-1.4` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R6-1.4` issues; keep it coverage-only
- run impact analysis before editing affected indexed symbols
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: cover R5.75-6 objective-bridge needle lists`
- fix commit message example: `fix: address R6-1.4 review findings`
````

---

## Packet 5 Prompt — Task R6-1.5

````text
/goal Land Packet `R6-1.5` from `docs/specs/r6/R6-1/agent-drift-analyzer-dead-end-thrash-cutover-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R6-1.5` only (closeout), assuming Packets `R6-1.0` through `R6-1.4` are already landed.

Packet `R6-1.5` scope only:
- run the full analyzer wall and the touched sentinel spot-checks
- if the walls reveal a real packet-scoped defect in already-landed `R6-1.2` through `R6-1.4` work, spawn a fix subagent to repair only that defect, commit it, and rerun the walls
- update the `R6-1` entry in `docs/specs/r6/MAP.md` with status and a routing note pointing to `R6-2` as the next active seam
- if the walls are green and only the MAP doc changes, commit the MAP update; do not fabricate other commits

Primary files:
- `docs/specs/r6/MAP.md` (status/routing at closeout)

Hard rules:
1. Spawn a fresh implementation/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If validation reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch (and the MAP status update) before review; do not fabricate empty commits.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the verification story, the MAP update, or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. This packet closes `R6-1`; do not start `R6-2`.

Verification wall:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Execute Packet `R6-1.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the full analyzer wall + touched sentinel spot-checks, repair only packet-scoped defects if exposed, and update the MAP R6-1 status.

Use the `$incremental-implementation` skill.

Read first:
- the R6-1 spec/plan/tasks docs
- docs/specs/r6/MAP.md
- AGENTS.md

Do this:
- run the three verification commands (full analyzer wall + sentinel warning_policy + live_end_to_end)
- if all are green and no production defect is exposed, make no code change
- if a packet-scoped defect in already-landed R6-1.2 through R6-1.4 work is exposed, fix only that defect, rerun the affected commands, and hand back a focused diff
- update the R6-1 entry in `docs/specs/r6/MAP.md` with promotion status and a routing note to R6-2

Rules:
- do not widen beyond already-landed R6-1 code/test scope; no objective read, no progress.rs change, no DriftClass variant
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: commands run, results, whether any fix was needed, changed files, and the exact commit message(s) you recommend.
```

Review subagent prompt:

```text
/goal Review the Packet `R6-1.5` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the green walls, any resulting fix, and the MAP update are ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the full analyzer wall and the touched sentinel spot-checks were run honestly and are green
- whether any resulting fix stayed packet-scoped
- whether the MAP R6-1 status + R6-2 routing note are accurate
- whether the verification story is sufficient to promote R6-1

State clearly whether Packet `R6-1.5` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R6-1.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed R6-1 code/test scope and the MAP status update
- rerun only the necessary verification commands after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, commands rerun, and the exact commit message you recommend.
```

Commit guidance:
- MAP status commit message example: `docs: promote R6-1 and route to R6-2`
- fix commit message example: `fix: address R6-1.5 review findings`
- no-op outcome example: `no commit; Packet R6-1.5 walls already green` (still commit the MAP status update separately)
````
