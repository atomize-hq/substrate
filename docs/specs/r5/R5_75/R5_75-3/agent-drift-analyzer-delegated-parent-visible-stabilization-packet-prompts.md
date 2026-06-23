# Delegated Parent-Visible Stabilization (R5.75-3) Packet Prompts

Status: orchestration prompts created on 2026-06-22 against the live
`docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md`
ledger.

Each prompt below is self-contained: paste one into a fresh session to land exactly one `R5.75-3`
packet through a GPT-5.4 high implementation -> commit -> review -> fix -> commit loop until
review-clean.

Packet `R5.75-3.0` (docs lock) is already landed in this session and is excluded. These prompts map:

- Packet 1 → Task `R5.75-3.1`
- Packet 2 → Task `R5.75-3.2`
- Packet 3 → Task `R5.75-3.3`
- Packet 4 → Task `R5.75-3.4`
- Packet 5 → Task `R5.75-3.5`
- Packet 6 → Task `R5.75-3.6`
- Packet 7 → Task `R5.75-3.7`
- Packet 8 → Task `R5.75-3.8`

Shared authority for all packets:

- `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md`
- `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md`
- `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md`
- `docs/specs/r5/R5_75/MAP.md`
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
   fix subagent whose prompt starts with `/goal ` and explicitly uses the
   `$incremental-implementation` skill.
8. Commit every non-empty implementation/fix batch before sending a fresh review subagent back through
   the loop. Do not fabricate empty commits for verification-only packets that produce no file changes.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not advance to the next packet until the current packet is committed and a fresh review
    subagent reports it review-clean, or for verification-only packets with no file changes,
    explicitly reports there was nothing to commit.

---

## Packet 1 Prompt — Task R5.75-3.1

````text
/goal Land Packet `R5.75-3.1` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.1` only, assuming `R5.75-3.0` is already landed and no later `R5.75-3.2+` packet has started.

Packet `R5.75-3.1` scope only:
- run the named native/adapted delegated smoke sessions
- record the current/expected lane, status, confidence, and limiting/supporting evidence expectations for:
  - `019eb907-95c4-73e1-843e-e337d1e93cb9`
  - `019eb917-9531-74e0-897d-ad8d362138ec`
  - `019eb970-3543-7ab1-a5d6-2a62c00c7185`
  - `da59436e63915185`
- record those expectations in the tasks ledger under Task `R5.75-3.1.1`
- explicitly distinguish `019eb970-3543-7ab1-a5d6-2a62c00c7185` as the positive-proof case, the two unstable native sessions as conservative delegated-parent witnesses, and `da59436e63915185` as the adapted smoke witness whose anti-flap concerns beyond this bar remain owned by `R5.75-4`

Out of scope:
- production-source edits
- fixture promotion
- planning-artifact stabilization
- comparability changes
- Packets `R5.75-3.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the recorded findings and smoke evidence yourself.
5. Commit the tasks-ledger evidence before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues (bad evidence, unclear expectations, wrong packet boundary), spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-3.2`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: characterize the named delegated repros and record the exact packet-local expectations in the tasks ledger.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- run the native smoke harness for:
  - 019eb907-95c4-73e1-843e-e337d1e93cb9
  - 019eb917-9531-74e0-897d-ad8d362138ec
  - 019eb970-3543-7ab1-a5d6-2a62c00c7185
- run the adapted smoke harness for:
  - da59436e63915185
- inspect summary.md and the first checkpoints.jsonl rows for each
- update the tasks ledger under Task R5.75-3.1.1 with exact observed packet-local expectations: lane, status, confidence, and limiting/supporting evidence
- keep scope docs-only: no production code, no fixture changes

Precondition check:
- confirm R5.75-3.0 is already landed
- if a prerequisite is missing, stop and report it instead of compensating here

Return with: the recorded expectations, the exact smoke outputs referenced, changed files, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-3.1` characterization in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-3.1` against:
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Focus:
- whether the four named sessions were actually run
- whether the recorded expectations are concrete enough to guide later packets
- whether packet boundaries are honest (`da59436e63915185` smoke-only; R5.75-4 ownership preserved)
- whether the positive-proof vs conservative-witness distinction is clearly documented

State clearly whether Packet `R5.75-3.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-3.1` evidence/wording issues
- keep it docs-only
- rerun only the smoke steps needed to correct the findings
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, any rerun evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R5.75-3.1 delegated repro characterization`
- fix commit message example: `fix: address R5.75-3.1 review findings`
````

---

## Packet 2 Prompt — Task R5.75-3.2

````text
/goal Land Packet `R5.75-3.2` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.2` only, assuming Packets `R5.75-3.0` and `R5.75-3.1` are already landed.

Packet `R5.75-3.2` scope only:
- in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`, stop returning to generic planning-only handling solely because `plan_artifact_edits(analysis)` is non-empty when delegation markers and parent-visible synthesis/orchestration evidence are otherwise strong
- keep child-visibility limits explicit and conservative
- do not fabricate positive opaque-child progress

Primary files:
- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

Out of scope:
- committed fixture promotion
- comparability-reset follow-on unless the packet docs prove it is strictly required here
- adapted committed fixtures
- Packets `R5.75-3.3+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
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
12. Do not start Packet `R5.75-3.3`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: stabilize the delegated parent-visible path when the parent edits planning/spec/handoff artifacts.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md
- docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- inspect `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- land the narrow fix so plan/spec/handoff edits do not eject strong delegated-parent evidence out of the parent-visible lane
- keep child-visibility limitations explicit and conservative
- add only the minimum `tests/checkpoints.rs` coverage needed to prove this packet's behavior

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- confirm Packets `R5.75-3.0` and `R5.75-3.1` are landed
- if a prerequisite is missing, stop and report it

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-3.2` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether planning/spec/handoff edits no longer erase the delegated parent-visible lane when strong delegation evidence exists
- whether child-visibility limits remain explicit and conservative
- whether the packet stayed narrowly inside `progress.rs` + matching checkpoint regressions
- whether the verification story is sufficient

Review against:
- the `R5.75-3` spec/plan/tasks docs
- `docs/specs/r5/R5_75/MAP.md`
- AGENTS.md

State clearly whether Packet `R5.75-3.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R5.75-3.2` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-3.2` issues
- keep the work narrow
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run GitNexus impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: stabilize delegated parent-visible planning edits`
- fix commit message example: `fix: address R5.75-3.2 review findings`
````

---

## Packet 3 Prompt — Task R5.75-3.3

````text
/goal Land Packet `R5.75-3.3` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.3` only, assuming Packets `R5.75-3.0` through `R5.75-3.2` are already landed.

Packet `R5.75-3.3` scope only:
- add or refresh fast delegated-parent regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove:
  - planning/spec/handoff edits do not erase the delegated parent-visible lane when delegation evidence is otherwise strong
  - limited child visibility remains visible as limiting/counter evidence
  - the packet preserves conservative parent-visible interpretation instead of overclaiming child execution progress
  - any later comparability tweak stays narrow and additive

Out of scope:
- production logic changes beyond minimal test-support adjustments required by this packet
- committed fixture promotion
- comparability follow-on
- Packets `R5.75-3.4+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed Rust symbol.
4. Commit the landed regression work before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-3.4`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-3.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: add or refresh fast checkpoint regressions for delegated parent-visible stability.

Use the `$incremental-implementation` skill.

Read first:
- the `R5.75-3` spec/plan/tasks docs
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- update `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove the delegated parent-visible behavior locked by Packet `R5.75-3.2`
- keep the assertions conservative and packet-scoped
- do not broaden into fixture work or comparability redesign

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-3.3` regression work in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the new or updated checkpoint regressions actually lock the intended delegated-parent behavior
- whether they are packet-scoped and conservative
- whether they would catch regressions in planning-artifact handling and visibility-limiting evidence
- whether the verification story is sufficient

State clearly whether Packet `R5.75-3.3` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-3.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R5.75-3.3` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-3.3` issues
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock delegated parent-visible stability regressions`
- fix commit message example: `fix: address R5.75-3.3 review findings`
````

---

## Packet 4 Prompt — Task R5.75-3.4

````text
/goal Land Packet `R5.75-3.4` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.4` only, assuming Packets `R5.75-3.0` through `R5.75-3.3` are already landed.

Packet `R5.75-3.4` scope only:
- promote the first native delegated real-rollout proof into committed `progress_acceptance` coverage
- by default, the first committed case is `019eb970-3543-7ab1-a5d6-2a62c00c7185`
- update:
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
  - the new delegated case directory
- keep delegated cases guardrail-only in `R5`
- retain the synthetic delegated guardrail proof unless there is an explicitly documented reason to replace it

Out of scope:
- adapted committed fixture-family work
- broader corpus redesign
- comparability follow-on
- Packets `R5.75-3.5+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed symbol.
4. Commit the landed fixture/corpus update before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-3.5`.

Required verification:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-3.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: admit the first native delegated real-rollout proof into the committed progress_acceptance corpus.

Use the `$incremental-implementation` skill.

Read first:
- the `R5.75-3` spec/plan/tasks docs
- crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
- crates/agent-drift-analyzer/tests/progress_acceptance.rs
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- by default, promote `019eb970-3543-7ab1-a5d6-2a62c00c7185` as the first committed delegated real-rollout case
- update README, corpus-count/exclusion assertions, and case fixtures coherently
- keep delegated cases guardrail-only in `R5`
- retain the synthetic delegated guardrail proof unless the packet docs justify replacing it
- stay strictly inside Packet `R5.75-3.4`

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Run:
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`

Return with: changed files, tests run, any residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-3.4` corpus update in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the new delegated real-rollout fixture is correctly bounded and annotated
- whether README and `progress_acceptance.rs` were updated honestly
- whether delegated cases remain guardrail-only in `R5`
- whether the synthetic delegated guardrail proof was preserved unless there was an explicitly justified replacement
- whether the verification story is sufficient

State clearly whether Packet `R5.75-3.4` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-3.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R5.75-3.4` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-3.4` issues
- rerun `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add delegated real-rollout progress acceptance proof`
- fix commit message example: `fix: address R5.75-3.4 review findings`
````

---

## Packet 5 Prompt — Task R5.75-3.5

````text
/goal Land Packet `R5.75-3.5` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean, or honestly report that no change was needed.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.5` only, assuming Packets `R5.75-3.0` through `R5.75-3.4` are already landed.

Packet `R5.75-3.5` scope only:
- apply one narrow parent-visible comparability-reset tweak only if the post-stabilization rerun proves the remaining failure is specifically over-broad reset behavior
- keep any tweak additive and limited to `parent_visible_comparability_fingerprint(...)` on the legacy objective/delegated-surface/working-set inputs
- if no tweak is needed, report that honestly and do not fabricate a commit
- if the needed change is broader, stop and report that a new packet is required

Out of scope:
- generalized fingerprint redesign
- structured-state comparability migration
- any change not strictly required by the packet's conditional rule

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed symbol.
4. If the implementation subagent reports no code/doc change was needed, do not fabricate an empty commit; go straight to review of that no-op conclusion.
5. If a real fix lands, commit it before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
7. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
8. Run `gitnexus_detect_changes()` before every real commit.
9. Do not start Packet `R5.75-3.6`.

Required verification if a change lands:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Execute Packet `R5.75-3.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: determine whether a narrow parent-visible comparability-reset tweak is still needed, and land it only if the packet's condition is met.

Use the `$incremental-implementation` skill.

Read first:
- the `R5.75-3` spec/plan/tasks docs
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- rerun the named delegated repro evidence after Packets `R5.75-3.2` through `R5.75-3.4`
- determine whether the remaining failure, if any, is specifically over-broad `parent_visible_comparability_fingerprint(...)` reset behavior
- if yes, apply one narrow additive tweak and verify it
- if no change is needed, report that honestly and make no edit
- if the needed change is broader than the packet allows, stop and report that instead of widening scope

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for any real commit

If a change lands, run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`

Return with: whether the tweak was needed, exact changes made if any, verification run, residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-3.5` outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the no-op or landed comparability result is correct and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the packet correctly determined if a comparability tweak was needed
- if a tweak landed, whether it stayed narrow and additive on the legacy surface
- whether the packet avoided widening into generalized redesign
- whether the verification story is sufficient

State clearly whether Packet `R5.75-3.5` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-3.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay strictly within the packet's conditional comparability boundary
- if a code fix is needed, rerun the packet verification commands
- if the correct outcome is "no change needed," preserve that honestly
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, verification run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: narrow delegated parent-visible comparability reset`
- no-op outcome example: `no commit; Packet R5.75-3.5 not needed`
- fix commit message example: `fix: address R5.75-3.5 review findings`
````

---

## Packet 6 Prompt — Task R5.75-3.6

````text
/goal Land Packet `R5.75-3.6` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.6` only, assuming Packets `R5.75-3.0` through `R5.75-3.5` are already landed.

Packet `R5.75-3.6` scope only:
- run the packet's automated validation wall
- if the wall reveals a real packet-scoped defect in already-landed `R5.75-3.2` through `R5.75-3.5` work, spawn a fix subagent to repair only that defect, commit it, and rerun the wall
- if the wall is already green and no files change, do not fabricate a commit

Hard rules:
1. Spawn a fresh implementation/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If validation reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the verification story or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-3.7`.

Verification wall:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-3.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the automated validation wall for already-landed R5.75-3 work and repair only packet-scoped defects if the wall exposes them.

Use the `$incremental-implementation` skill.

Do this:
- run the three Packet `R5.75-3.6` validation commands
- if all are green and no files need changing, report that honestly
- if a packet-scoped defect in already-landed `R5.75-3.2` through `R5.75-3.5` work is exposed, fix only that defect, rerun the necessary validation commands, and hand back a focused diff

Rules:
- do not widen beyond already-landed `R5.75-3` code/test/fixture scope
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: commands run, results, whether any fix was needed, changed files if any, and the exact commit message you recommend if a real fix landed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-3.6` automated verification outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the green wall or any resulting fix is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the automated wall was run honestly
- whether any resulting fix stayed packet-scoped
- whether the verification story is sufficient and clearly documented

State clearly whether Packet `R5.75-3.6` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-3.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed `R5.75-3` code/test/fixture scope
- rerun only the necessary validation commands after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, commands rerun, and the exact commit message you recommend.
```

Commit guidance:
- fix commit message example: `fix: address R5.75-3 automated validation findings`
- no-op outcome example: `no commit; Packet R5.75-3.6 validation already green`
````

---

## Packet 7 Prompt — Task R5.75-3.7

````text
/goal Land Packet `R5.75-3.7` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a smoke -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.7` only, assuming Packets `R5.75-3.0` through `R5.75-3.6` are already landed.

Packet `R5.75-3.7` scope only:
- rerun the three named native delegated sessions
- inspect packet-owned outputs manually
- if smoke reveals a real packet-scoped defect in already-landed `R5.75-3.2` through `R5.75-3.5` work, spawn a fix subagent to repair only that defect, commit it, and rerun the smoke
- if smoke is clean and no files change, do not fabricate a commit

Hard rules:
1. Spawn a fresh smoke/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If smoke reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the smoke evidence or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-3.8`.

Smoke/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-3.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun the three named native delegated sessions, inspect the packet-owned outputs manually, and repair only packet-scoped defects if smoke exposes them.

Use the `$incremental-implementation` skill.

Do this:
- run the native smoke harness for:
  - 019eb907-95c4-73e1-843e-e337d1e93cb9
  - 019eb917-9531-74e0-897d-ad8d362138ec
  - 019eb970-3543-7ab1-a5d6-2a62c00c7185
- inspect summary.md and the first checkpoints.jsonl rows for each
- compare the observed outputs against the expectations locked in Packet `R5.75-3.1`
- if smoke is clean, report that honestly with no code change
- if a packet-scoped defect in already-landed `R5.75-3` work is exposed, fix only that defect, rerun the affected smoke, and hand back a focused diff

Rules:
- do not widen beyond already-landed `R5.75-3` code/test/fixture scope
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: smoke outputs, whether any fix was needed, changed files if any, and the exact commit message you recommend if a real fix landed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-3.7` native smoke outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the evidence or any resulting fix is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the named native smoke was run honestly
- whether the outputs actually match the expectations recorded in Packet `R5.75-3.1`
- whether any resulting fix stayed packet-scoped
- whether the smoke evidence is sufficient to keep

State clearly whether Packet `R5.75-3.7` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-3.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed `R5.75-3` scope
- rerun only the necessary native smoke after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, smoke rerun results, and the exact commit message you recommend.
```

Commit guidance:
- fix commit message example: `fix: address R5.75-3 native smoke findings`
- no-op outcome example: `no commit; Packet R5.75-3.7 native smoke already clean`
````

---

## Packet 8 Prompt — Task R5.75-3.8

````text
/goal Land Packet `R5.75-3.8` from `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a smoke -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-3.8` only, assuming Packets `R5.75-3.0` through `R5.75-3.7` are already landed.

Packet `R5.75-3.8` scope only:
- rerun adapted delegated witness `da59436e63915185`
- confirm the packet boundary stays honest
- if smoke reveals a real packet-scoped defect in already-landed `R5.75-3.2` through `R5.75-3.5` work, spawn a fix subagent to repair only that defect, commit it, and rerun the smoke
- if smoke is clean and no files change, do not fabricate a commit

Hard rules:
1. Spawn a fresh smoke/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If smoke reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the smoke evidence, packet boundary, or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. This packet closes the `R5.75-3` smoke review; do not widen into `R5.75-4`.

Smoke/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-3.8` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun adapted delegated witness `da59436e63915185`, confirm the packet boundary stays honest, and repair only packet-scoped defects if smoke exposes them.

Use the `$incremental-implementation` skill.

Do this:
- run the adapted smoke harness for `da59436e63915185`
- inspect summary.md and the first checkpoints.jsonl rows
- compare the observed output against the expectation locked in Packet `R5.75-3.1`
- confirm any residual anti-flap or zero-verifier concerns beyond the parent-visible stability bar are explicitly left to `R5.75-4`
- if smoke is clean, report that honestly with no code change
- if a packet-scoped defect in already-landed `R5.75-3` work is exposed, fix only that defect, rerun the smoke, and hand back a focused diff

Rules:
- do not widen beyond already-landed `R5.75-3` code/test/fixture scope
- do not convert this adapted witness into a committed adapted fixture family
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: smoke outputs, whether any fix was needed, changed files if any, and the exact commit message you recommend if a real fix landed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-3.8` adapted smoke outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the evidence or any resulting fix is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the adapted smoke was run honestly
- whether the output matches the expectation recorded in Packet `R5.75-3.1`
- whether the packet boundary stayed honest (`R5.75-4` ownership preserved; no adapted committed fixture-family widening)
- whether any resulting fix stayed packet-scoped

State clearly whether Packet `R5.75-3.8` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-3.8` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed `R5.75-3` scope
- preserve the smoke-only status of `da59436e63915185`
- rerun only the necessary adapted smoke after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, smoke rerun results, and the exact commit message you recommend.
```

Commit guidance:
- fix commit message example: `fix: address R5.75-3 adapted smoke findings`
- no-op outcome example: `no commit; Packet R5.75-3.8 adapted smoke already clean`
````
