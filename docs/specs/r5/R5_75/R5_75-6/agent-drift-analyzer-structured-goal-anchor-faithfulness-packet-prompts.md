# Structured Goal Anchor Faithfulness (R5.75-6) Packet Prompts

Status: orchestration prompts created on 2026-06-24 against the live
`docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md`
ledger. Use `docs/specs/r5/R5_75/MAP.md` plus the current SPEC/PLAN/TASKS ledger as the live
execution authority before starting any packet.

Each prompt below is self-contained: paste one into a fresh session to land exactly one `R5.75-6`
packet through a GPT-5.4 high implementation -> commit -> review -> fix -> commit loop until
review-clean.

These prompts map:

- Packet 0 → Task `R5.75-6.0`
- Packet 1 → Task `R5.75-6.1`
- Packet 2 → Task `R5.75-6.2`
- Packet 3 → Task `R5.75-6.3`
- Packet 4 → Task `R5.75-6.4`

Shared authority for all packets:

- `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md`
- `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md`
- `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md`
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/structured-objective-bug-map.md`
- `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`
- `AGENTS.md`

Working directory for every prompt:
`/Users/spensermcconnell/.codex/worktrees/97a0/substrate`

Global rules for every packet prompt below:

1. The parent session is the orchestration agent; it should not do packet work locally unless
   delegated execution is unavailable.
2. The orchestration agent must spawn a fresh `GPT-5.4` subagent on `high` for implementation or
   verification first.
3. Every implementation or fix subagent prompt must start with `/goal ` and must explicitly tell the
   subagent to use the `$incremental-implementation` skill.
4. After implementation work lands, the orchestration agent must commit before dispatching review.
5. The orchestration agent must then spawn a fresh `GPT-5.4` subagent on `high` for review.
6. Every review subagent prompt must start with `/goal ` and must explicitly tell the subagent to use
   the `$code-review-and-quality` skill.
7. If the review subagent flags issues, the orchestration agent must spawn a fresh `GPT-5.4` `high`
   fix subagent whose prompt starts with `/goal ` and explicitly uses the
   `$incremental-implementation` skill.
8. Commit every non-empty implementation/fix batch before sending a fresh review subagent back
   through the loop. Do not fabricate empty commits for verification-only packets that produce no
   file changes.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Before modifying any indexed Rust symbol, run GitNexus impact analysis first and report any HIGH
    or CRITICAL blast radius before proceeding.
11. Do not advance to the next packet until the current packet is committed and a fresh review
    subagent reports it review-clean, or for verification-only/no-op packets, explicitly reports there
    was nothing to commit.

---

## Packet 0 Prompt — Task R5.75-6.0

````text
/goal Land Packet `R5.75-6.0` from `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-6.0` only.

Packet `R5.75-6.0` scope only:
- commit the `R5.75-6` SPEC/PLAN/TASKS triplet
- ensure the docs record the bounded bridge-fidelity scope
- ensure the docs record the named wrong-imperative repro and anchored-review witness
- ensure the docs record the non-widening boundary against the deferred structured-native migration
- ensure the docs record the requirement to rerun the full named `R5.75-5` smoke set before promotion

Out of scope:
- production code
- regression/test changes
- smoke execution
- `MAP.md` promotion updates
- Packets `R5.75-6.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the doc diff yourself.
5. Commit the docs packet before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-6.1`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-6.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: finalize and commit the `R5.75-6` SPEC/PLAN/TASKS docs lock.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- AGENTS.md

Do this:
- verify the triplet matches Task `R5.75-6.0.1`
- make only doc fixes needed to satisfy the packet acceptance
- keep scope strictly docs-only

Return with: changed files, any doc fixes made, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-6.0` docs lock in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-6.0` against:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- AGENTS.md

Focus:
- whether the packet boundary is explicit and honest
- whether the named repros and witnesses are captured correctly
- whether the docs avoid silently absorbing the later structured-native migration or `R6`
- whether the docs are sufficient to guide Packets `R5.75-6.1` through `R5.75-6.4`

State clearly whether Packet `R5.75-6.0` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-6.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-6.0` doc issues
- keep it docs-only
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: add R5.75-6 structured goal anchor triplet`
- fix commit message example: `fix: address R5.75-6.0 review findings`
````

---

## Packet 1 Prompt — Task R5.75-6.1

````text
/goal Land Packet `R5.75-6.1` from `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-6.1` only, assuming Packet `R5.75-6.0` is already landed.

Packet `R5.75-6.1` scope only:
- make the checkpoint compatibility bridge defer to the grounded structured goal anchor when it exists
- prevent optional reviewer nits / closeout bullets from becoming the effective objective after the real ask is already anchored
- keep legacy narrowing available as fallback when structured grounding is absent or weak
- keep the change bounded to the checkpoint/inference bridge without widening into the full structured-native migration

Primary files:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- optionally `crates/agent-drift-analyzer/src/context/objective.rs` only if a shared grounding helper is strictly required

Out of scope:
- broad `TaskFrame` schema migration
- `context/working_set.rs` migration
- `checkpoint/progress.rs` comparison redesign
- new adapted fixtures
- Packets `R5.75-6.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the diff and verification results yourself.
6. Commit the landed implementation before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
11. Run `gitnexus_detect_changes()` before every real commit.
12. Do not start Packet `R5.75-6.2`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-6.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: repair the bounded checkpoint compatibility bridge so the effective objective stays faithful to the grounded structured goal anchor.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- AGENTS.md

Do this:
- inspect the live bridge between `assemble_context(...)`, `narrowed_objective_summary(...)`, and `infer_task_frame(...)`
- land the narrowest fix that ensures a grounded structured goal anchor wins over optional-nit / closeout imperative lines
- preserve fallback behavior when structured grounding is absent or weak
- do not widen into the deferred full structured-native migration

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- confirm Packet `R5.75-6.0` is already landed
- if the prerequisite is missing, stop and report it

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-6.1` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-6.1` against:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- AGENTS.md

Focus:
- whether the effective objective now defers to the grounded structured goal anchor when present
- whether optional reviewer nits / closeout bullets can still incorrectly win
- whether fallback behavior for non-structured cases remains honest
- whether the change stayed bounded to the checkpoint/inference bridge without widening into the deferred full migration
- whether verification is adequate for this packet

State clearly whether Packet `R5.75-6.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-6.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-6.1` bridge issues
- keep the repair bounded to the checkpoint/inference seam unless the review explicitly proves a tiny shared helper is required
- rerun the minimum packet-local tests needed after each fix batch
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests rerun, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: keep R5.75-6 objective bridge faithful to structured goals`
- fix commit message example: `fix: address R5.75-6.1 review findings`
````

---

## Packet 2 Prompt — Task R5.75-6.2

````text
/goal Land Packet `R5.75-6.2` from `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-6.2` only, assuming Packets `R5.75-6.0` and `R5.75-6.1` are already landed.

Packet `R5.75-6.2` scope only:
- add a minimized checkpoint regression for the wrong-imperative follow-up shape modeled on `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
- assert that the effective objective matches the validate/readiness ask rather than the reviewer nit
- add or preserve a fallback guard so the packet does not over-disable legacy narrowing when structured grounding is absent

Primary files:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- optionally packet-local helper touch-ups in `crates/agent-drift-analyzer/src/checkpoint/mod.rs` only if strictly required for the regression to express real behavior

Out of scope:
- broad source changes beyond the packet-owned regression/fallback guard
- progress acceptance edits
- smoke reruns beyond what is minimally needed to validate the regression
- Packets `R5.75-6.3+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. After the implementation subagent finishes, inspect the diff and test results yourself.
6. Commit the landed regression packet before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
11. Run `gitnexus_detect_changes()` before every real commit.
12. Do not start Packet `R5.75-6.3`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-6.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: add durable checkpoint regression coverage for the wrong-imperative follow-up shape and preserve honest fallback behavior.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- AGENTS.md

Do this:
- add a minimized regression in `crates/agent-drift-analyzer/tests/checkpoints.rs` modeled on `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
- assert the effective objective matches the validate/readiness ask instead of `add extra task-local grep checks for transition routing / outcome/final-marker meaning`
- assert the exported structured objective remains aligned to the same real ask
- add or preserve a fallback guard so legitimate non-structured compatibility behavior is not lost
- keep the packet tightly scoped to regression coverage

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- confirm Packets `R5.75-6.0` and `R5.75-6.1` are already landed
- if a prerequisite is missing, stop and report it

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, what the new regression proves, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-6.2` regression coverage in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the new regression actually models the wrong-imperative failure instead of a weaker synthetic variant
- whether it proves both the effective objective string and the structured objective stay aligned to the real ask
- whether the fallback guard is sufficient to prevent over-disabling legitimate narrowing
- whether the packet stayed narrowly scoped to regression coverage

State clearly whether Packet `R5.75-6.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-6.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-6.2` regression/fallback-guard issues
- keep the scope in checkpoint tests unless the review proves a tiny packet-local source adjustment is required
- rerun the packet checkpoint suite after each fix batch
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests rerun, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock R5.75-6 wrong-imperative regression`
- fix commit message example: `fix: address R5.75-6.2 review findings`
````

---

## Packet 3 Prompt — Task R5.75-6.3

````text
/goal Land Packet `R5.75-6.3` from `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-6.3` only, assuming Packets `R5.75-6.0` through `R5.75-6.2` are already landed.

Packet `R5.75-6.3` scope only:
- rerun the progress acceptance wall after the bridge repair
- rerun the full analyzer wall and sentinel spot-checks
- if these suites expose bounded packet-owned fallout from `R5.75-6`, fix only that fallout
- if the failures instead prove broader migration pressure or out-of-scope regressions, stop and report rather than widening the packet

Out of scope:
- manual smoke promotion work
- `MAP.md` promotion updates
- broad consumer migration
- Packets `R5.75-6.4+`

Hard rules:
1. Spawn a fresh implementation/verification subagent first. Use `GPT-5.4` on `high`.
2. The implementation/verification subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. If the subagent finds no packet-owned fallout and makes no file changes, do not fabricate a commit; explicitly record that the packet is verification-only and nothing changed.
6. If the subagent lands bounded fixes, inspect the diff and verification results yourself, then commit before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues in a landed fix batch, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
11. Run `gitnexus_detect_changes()` before every real commit.
12. Do not start Packet `R5.75-6.4`.

Required verification:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Land Packet `R5.75-6.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun the downstream automated walls after `R5.75-6` and fix only bounded packet-owned fallout if it appears.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- run:
  - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
  - `cargo test -p agent-drift-sentinel warning_policy -- --nocapture`
  - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
- if everything passes and no source/doc changes are needed, stop with an explicit no-op report and recommend no commit
- if failures expose bounded packet-owned fallout from the `R5.75-6` objective-faithfulness work, fix only that fallout, rerun the minimum required suites, and prepare the changes for commit
- if the failures instead indicate broader out-of-scope migration pressure, stop and report rather than widening the packet

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for any real commit

Precondition check:
- confirm Packets `R5.75-6.0` through `R5.75-6.2` are already landed
- if a prerequisite is missing, stop and report it

Return with: the exact commands run, pass/fail outcomes, whether this packet is a no-op or landed bounded fixes, any changed files, and the exact commit message you recommend if a real commit is needed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-6.3` downstream verification result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether all required automated walls were actually run
- whether any fixes stayed bounded to packet-owned fallout instead of drifting into broader migration work
- whether a no-op outcome is justified if no files changed
- whether the verification story is sufficient to support moving to manual smoke

State clearly whether Packet `R5.75-6.3` is review-clean, requires changes, or is a justified no-op verification packet. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues in a landed fix batch:

```text
/goal Address the review findings for Packet `R5.75-6.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only bounded packet-owned fallout or verification-story issues for `R5.75-6.3`
- do not widen into manual smoke or broader migration work
- rerun the minimum required suites after each fix batch
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, suites rerun, and the exact commit message you recommend.
```

Commit guidance:
- if a real implementation/fix batch lands: `fix: address R5.75-6 downstream verification fallout`
- if the packet is verification-only and no files changed: no commit
````

---

## Packet 4 Prompt — Task R5.75-6.4

````text
/goal Land Packet `R5.75-6.4` from `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-6.4` only, assuming Packets `R5.75-6.0` through `R5.75-6.3` are already landed.

Packet `R5.75-6.4` scope only:
- rerun native smoke for:
  - `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
- rerun the full named native + adapted smoke set carried forward from `R5.75-5`
- inspect `summary.md` and the first `checkpoints.jsonl` rows for each witness
- update `docs/specs/r5/R5_75/MAP.md` only after the smoke wall proves `R5.75-6` is honestly promoted and `R5.75` is truly complete

Out of scope:
- new semantic implementation beyond bounded packet-owned fallout discovered during smoke
- broader migration work
- `R6` packet creation or execution

Hard rules:
1. Spawn a fresh implementation/verification subagent first. Use `GPT-5.4` on `high`.
2. The implementation/verification subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless delegated execution is unavailable.
5. If smoke reveals no packet-owned failures and only `MAP.md` needs the honest promotion update, commit that docs closeout before dispatching review.
6. If smoke reveals bounded packet-owned fallout, fix only that fallout, rerun the necessary smoke/tests, then commit before dispatching review.
7. Then spawn a fresh review subagent on `GPT-5.4` `high`.
8. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
9. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
11. Run `gitnexus_detect_changes()` before every real commit.
12. Do not open `R6` inside this packet; finish with an honest `R5.75` closeout only.

Implementation/verification subagent prompt to send:

```text
/goal Land Packet `R5.75-6.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun the named smoke wall for `R5.75-6`, confirm no earlier `R5.75` issue regresses, and update `MAP.md` only if the family is honestly complete.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
- docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/structured-objective-bug-map.md
- AGENTS.md

Do this:
- rerun native smoke for:
  - `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
- rerun the full named native + adapted smoke set carried forward from `R5.75-5` under `target/r5_75-smoke/R5.75-6/`
- inspect `summary.md` and the first `checkpoints.jsonl` rows for every named witness
- if everything holds, update `docs/specs/r5/R5_75/MAP.md` honestly to promote `R5.75-6` and close `R5.75`
- if smoke exposes bounded packet-owned fallout, fix only that fallout, rerun the necessary test/smoke wall, and then update `MAP.md` only if the closeout is truly earned
- if smoke exposes broader out-of-scope migration pressure, stop and report rather than widening the packet

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for any real commit

Precondition check:
- confirm Packets `R5.75-6.0` through `R5.75-6.3` are already landed
- if a prerequisite is missing, stop and report it

Return with: exact smoke commands run, exact witnesses inspected, whether the packet landed via pure closeout or bounded fallout fixes, changed files, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-6.4` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether `R5.75` was closed honestly.

Use the `$code-review-and-quality` skill.

Focus:
- whether the named wrong-imperative repro and anchored-review witness were actually rerun and inspected
- whether the full carried-forward native + adapted smoke set was rerun
- whether `MAP.md` promotion wording matches the live smoke/test evidence
- whether any packet-owned fallout fix stayed bounded and fully revalidated
- whether the packet avoided prematurely opening `R6`

State clearly whether Packet `R5.75-6.4` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-6.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-6.4` closeout issues
- if smoke or tests must be rerun to address findings, rerun only the necessary packet-local wall and report it explicitly
- do not widen into `R6` work
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, any rerun evidence, and the exact commit message you recommend.
```

Commit guidance:
- closeout-without-code-fix commit message example: `docs: close R5.75 after R5.75-6 smoke promotion`
- bounded-fallout-fix commit message example: `fix: address R5.75-6 closeout smoke findings`
- fix-after-review commit message example: `fix: address R5.75-6.4 review findings`
````
