# Zero-Verifier Anti-Flap Gate (R5.75-4) Packet Prompts

Status: orchestration prompts created on 2026-06-23 against the live
`docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md`
ledger. Use `docs/specs/r5/R5_75/MAP.md` plus the current SPEC/PLAN/TASKS ledger as the live
execution authority before starting any packet.

Each prompt below is self-contained: paste one into a fresh session to land exactly one `R5.75-4`
packet through a GPT-5.4 high implementation -> commit -> review -> fix -> commit loop until
review-clean.

These prompts map:

- Packet 0 → Task `R5.75-4.0`
- Packet 1 → Task `R5.75-4.1`
- Packet 2 → Task `R5.75-4.2`
- Packet 3 → Task `R5.75-4.3`
- Packet 4 → Task `R5.75-4.4`
- Packet 5 → Task `R5.75-4.5`
- Packet 6 → Task `R5.75-4.6`
- Packet 7 → Task `R5.75-4.7`

Shared authority for all packets:

- `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md`
- `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md`
- `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md`
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/structured-objective-bug-map.md`
- `AGENTS.md`

Working directory for every prompt:
`/Users/spensermcconnell/.codex/worktrees/97a0/substrate`

Global rules for every packet prompt below:

1. The parent session is the orchestration agent; it should not do the packet work locally unless
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
   through the loop. Do not fabricate empty commits for verification-only packets that produce no file
   changes.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Before modifying any indexed Rust symbol, run GitNexus impact analysis first and report any HIGH
    or CRITICAL blast radius before proceeding.
11. Do not advance to the next packet until the current packet is committed and a fresh review
    subagent reports it review-clean, or for verification-only/no-op packets, explicitly reports there
    was nothing to commit.

---

## Packet 0 Prompt — Task R5.75-4.0

````text
/goal Land Packet `R5.75-4.0` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.0` only.

Packet `R5.75-4.0` scope only:
- commit the `R5.75-4` SPEC/PLAN/TASKS triplet
- ensure the docs record the analyzer-local packet boundary, adapted smoke witnesses, observational-only structured intent rule, non-widening boundary, and prompt-out-of-scope note

Out of scope:
- any production code
- smoke execution
- regression changes
- Packets `R5.75-4.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the doc diff yourself.
5. Commit the doc packet before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-4.1`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-4.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: finalize and commit the `R5.75-4` SPEC/PLAN/TASKS docs lock.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- verify the triplet matches Task `R5.75-4.0.1`
- make only doc fixes needed to satisfy the packet acceptance
- keep scope strictly docs-only

Return with: changed files, any doc fixes made, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-4.0` docs lock in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-4.0` against:
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Focus:
- whether the packet boundary is explicit and honest
- whether the adapted smoke witnesses are named correctly
- whether the packet avoids silently absorbing `R5.75-5`, `R5.75-6`, or `R6`
- whether the docs are sufficient to guide later packets

State clearly whether Packet `R5.75-4.0` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-4.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-4.0` doc issues
- keep it docs-only
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: add R5.75-4 zero-verifier anti-flap triplet`
- fix commit message example: `fix: address R5.75-4.0 review findings`
````

---

## Packet 1 Prompt — Task R5.75-4.1

````text
/goal Land Packet `R5.75-4.1` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.1` only, assuming Packet `R5.75-4.0` is already landed.

Packet `R5.75-4.1` scope only:
- run the adapted smoke witnesses
- record exact packet-local expectations for:
  - `097d97e914ca220f`
  - `da59436e63915185`
- update the tasks ledger under Task `R5.75-4.1.1`
- explicitly distinguish `097d97e914ca220f` as the canonical zero-verifier exploratory witness and `da59436e63915185` as the mixed delegated/exploratory witness that must preserve the conservative `R5.75-3` parent-visible bar while losing unrelated overclaim

Out of scope:
- production-source edits
- progress logic changes
- acceptance corpus changes
- Packets `R5.75-4.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the recorded findings and smoke evidence yourself.
5. Commit the tasks-ledger evidence before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-4.2`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-4.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: characterize the named exploratory adapted repros and record the exact packet-local expectations in the tasks ledger.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- run the adapted smoke harness for `097d97e914ca220f`
- run the adapted smoke harness for `da59436e63915185`
- inspect `summary.md` and the relevant `checkpoints.jsonl` rows for each
- update the tasks ledger under Task `R5.75-4.1.1` with exact observed packet-local expectations: lane, status, confidence, and decisive/counter-evidence
- keep scope docs-only: no production code, no acceptance fixture changes

Precondition check:
- confirm Packet `R5.75-4.0` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: the recorded expectations, the exact smoke outputs referenced, changed files, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-4.1` characterization in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether both adapted smoke witnesses were actually run
- whether the recorded expectations are concrete enough to guide later packets
- whether the `R5.75-3` parent-visible boundary for `da59436e63915185` is preserved honestly
- whether the packet stayed docs-only and did not drift into implementation

State clearly whether Packet `R5.75-4.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-4.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-4.1` evidence/wording issues
- keep it docs-only
- rerun only the smoke steps needed to correct the findings
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, any rerun evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R5.75-4.1 exploratory adapted characterization`
- fix commit message example: `fix: address R5.75-4.1 review findings`
````

---

## Packet 2 Prompt — Task R5.75-4.2

````text
/goal Land Packet `R5.75-4.2` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.2` only, assuming Packets `R5.75-4.0` and `R5.75-4.1` are already landed.

Packet `R5.75-4.2` scope only:
- in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`, land the minimal analyzer-local anti-flap rule
- prefer conservative `planning_convergence` / `insufficient_evidence` posture for long browse/read/tool-output-heavy intervals when verifier attempts, concrete source-edit progress, and explicit failure evidence are absent
- preserve conservative `parent_visible_orchestration` behavior for `da59436e63915185` where delegation evidence still justifies it
- do not wire structured `primary_intent` into the decision

Primary files:
- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

Out of scope:
- scorer or `dead_end_thrash` redesign
- adapted fixture-family work
- structured-goal consumer wiring
- Packets `R5.75-4.3+`

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
12. Do not start Packet `R5.75-4.3`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-4.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: land the minimal analyzer-local anti-flap rule for zero-verifier exploratory sessions.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- inspect `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- land the narrow anti-flap guard so long exploratory zero-verifier intervals stay conservative when decisive evidence is absent
- preserve `R5.75-3` parent-visible delegated behavior where justified
- keep the change additive and packet-local
- add only the minimum `tests/checkpoints.rs` support needed to prove this packet's behavior

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- confirm Packets `R5.75-4.0` and `R5.75-4.1` are landed
- if a prerequisite is missing, stop and report it

Run:
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-4.2` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether zero-verifier exploratory intervals now stay conservative when decisive signals are absent
- whether real troubleshooting signals remain possible
- whether delegated parent-visible behavior from `R5.75-3` was preserved where appropriate
- whether the packet stayed narrowly inside `progress.rs` plus matching checkpoint regressions
- whether the verification story is sufficient

State clearly whether Packet `R5.75-4.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-4.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R5.75-4.2` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-4.2` issues
- keep the work narrow
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run GitNexus impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: add R5.75-4 zero-verifier anti-flap guard`
- fix commit message example: `fix: address R5.75-4.2 review findings`
````

---

## Packet 3 Prompt — Task R5.75-4.3

````text
/goal Land Packet `R5.75-4.3` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.3` only, assuming Packets `R5.75-4.0` through `R5.75-4.2` are already landed.

Packet `R5.75-4.3` scope only:
- add or refresh fast zero-verifier anti-flap regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove:
  - broad-scan/read-output exploratory sessions stay low-confidence and conservative
  - real verifier-backed or explicit-failure-backed troubleshooting still advances when it should
  - delegated parent-visible semantics from `R5.75-3` are not erased
  - the anti-flap rule explains missing proof through limiting/counter-evidence instead of merely muting output

Out of scope:
- acceptance corpus work unless strictly required by this packet
- scorer redesign
- adapted committed fixture-family expansion
- Packets `R5.75-4.4+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed Rust symbol.
4. Commit the landed regression work before dispatching review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-4.4`.

Required verification:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-4.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: add or refresh fast checkpoint regressions for zero-verifier anti-flap behavior.

Use the `$incremental-implementation` skill.

Read first:
- the `R5.75-4` spec/plan/tasks docs
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- update `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove the behavior locked by Packet `R5.75-4.2`
- keep the assertions conservative and packet-scoped
- preserve delegated parent-visible semantics from `R5.75-3`
- do not broaden into fixture work or scorer redesign

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
/goal Review the already-landed Packet `R5.75-4.3` regression work in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the new or updated checkpoint regressions actually lock the intended anti-flap behavior
- whether they still allow real troubleshooting advancement where warranted
- whether they preserve delegated parent-visible semantics from `R5.75-3`
- whether they are packet-scoped and conservative
- whether the verification story is sufficient

State clearly whether Packet `R5.75-4.3` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-4.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet `R5.75-4.3` verification command.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-4.3` issues
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock R5.75-4 anti-flap regressions`
- fix commit message example: `fix: address R5.75-4.3 review findings`
````

---

## Packet 4 Prompt — Task R5.75-4.4

````text
/goal Land Packet `R5.75-4.4` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.4` only, assuming Packets `R5.75-4.0` through `R5.75-4.3` are already landed.

Packet `R5.75-4.4` scope only:
- determine whether a bounded `progress_acceptance` proof is actually needed
- if needed, land one bounded semantic proof without starting `R5.75-5`
- if not needed, report that honestly and do not fabricate a commit

Out of scope:
- adapted committed fixture-family expansion
- corpus-shape drift that is not strictly required by this packet
- any scorer or structured-goal wiring
- Packets `R5.75-4.5+`

Hard rules:
1. Spawn a fresh implementation subagent first on `GPT-5.4` `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. Run GitNexus impact analysis before editing any indexed symbol.
4. If the implementation subagent reports no code/doc change was needed, do not fabricate an empty commit; go straight to review of that no-op conclusion.
5. If a real update lands, commit it before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
7. If review flags issues, spawn a fresh fix subagent on `GPT-5.4` `high` with a `/goal ` prompt using `$incremental-implementation`, commit the fix, then send a fresh review subagent.
8. Run `gitnexus_detect_changes()` before every real commit.
9. Do not start Packet `R5.75-4.5`.

Required verification if a change lands:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Execute Packet `R5.75-4.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: determine whether a bounded `progress_acceptance` proof is still needed, and land it only if the packet's condition is met.

Use the `$incremental-implementation` skill.

Read first:
- the `R5.75-4` spec/plan/tasks docs
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- inspect the current `progress_acceptance` wall and Packet `R5.75-4.3` regression proof
- determine whether the packet still needs one bounded semantic proof
- if yes, add or refine exactly one bounded packet-local proof without starting adapted fixture-family work
- if no update is needed, report that honestly and make no edit

GitNexus requirements:
- run impact analysis before editing any indexed symbol
- report HIGH/CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for any real commit

If a change lands, run:
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`

Return with: whether the proof was needed, exact changes made if any, verification run, residual risks, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-4.4` outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the no-op or landed `progress_acceptance` result is correct and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the packet correctly determined if a bounded semantic proof was needed
- if a proof landed, whether it stayed packet-local and did not start `R5.75-5`
- whether corpus counts/exclusions stayed honest
- whether the verification story is sufficient

State clearly whether Packet `R5.75-4.4` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-4.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay strictly within the packet's bounded `progress_acceptance` boundary
- if a code/doc fix is needed, rerun the packet verification command
- if the correct outcome is `no change needed`, preserve that honestly
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, verification run, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add bounded R5.75-4 progress acceptance proof`
- no-op outcome example: `no commit; Packet R5.75-4.4 not needed`
- fix commit message example: `fix: address R5.75-4.4 review findings`
````

---

## Packet 5 Prompt — Task R5.75-4.5

````text
/goal Land Packet `R5.75-4.5` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.5` only, assuming Packets `R5.75-4.0` through `R5.75-4.4` are already landed.

Packet `R5.75-4.5` scope only:
- run the packet's automated validation wall
- if the wall reveals a real packet-scoped defect in already-landed `R5.75-4.2` through `R5.75-4.4` work, spawn a fix subagent to repair only that defect, commit it, and rerun the wall
- if the wall is already green and no files change, do not fabricate a commit

Hard rules:
1. Spawn a fresh implementation/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If validation reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the verification story or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-4.6`.

Verification wall:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-4.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the automated validation wall for already-landed R5.75-4 work and repair only packet-scoped defects if the wall exposes them.

Use the `$incremental-implementation` skill.

Do this:
- run the three Packet `R5.75-4.5` validation commands
- if all are green and no files need changing, report that honestly
- if a packet-scoped defect in already-landed `R5.75-4.2` through `R5.75-4.4` work is exposed, fix only that defect, rerun the necessary validation commands, and hand back a focused diff

Rules:
- do not widen beyond already-landed `R5.75-4` code/test/fixture scope
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: commands run, results, whether any fix was needed, changed files if any, and the exact commit message you recommend if a real fix landed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-4.5` automated verification outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the green wall or any resulting fix is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the automated wall was run honestly
- whether any resulting fix stayed packet-scoped
- whether the verification story is sufficient and clearly documented

State clearly whether Packet `R5.75-4.5` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-4.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed `R5.75-4` code/test/fixture scope
- rerun only the necessary validation commands after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, commands rerun, and the exact commit message you recommend.
```

Commit guidance:
- fix commit message example: `fix: address R5.75-4 automated validation findings`
- no-op outcome example: `no commit; Packet R5.75-4.5 validation already green`
````

---

## Packet 6 Prompt — Task R5.75-4.6

````text
/goal Land Packet `R5.75-4.6` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a smoke -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.6` only, assuming Packets `R5.75-4.0` through `R5.75-4.5` are already landed.

Packet `R5.75-4.6` scope only:
- rerun the two adapted exploratory witnesses
- inspect packet-owned outputs manually
- if smoke reveals a real packet-scoped defect in already-landed `R5.75-4.2` through `R5.75-4.4` work, spawn a fix subagent to repair only that defect, commit it, and rerun the smoke
- if smoke is clean and no files change, do not fabricate a commit

Hard rules:
1. Spawn a fresh smoke/verification subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If smoke reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
4. Commit every non-empty fix batch before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the smoke evidence or a resulting fix, spawn a fresh fix subagent and repeat the loop.
7. Run `gitnexus_detect_changes()` before every real commit.
8. Do not start Packet `R5.75-4.7`.

Smoke/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-4.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun the two adapted exploratory witnesses, inspect the packet-owned outputs manually, and repair only packet-scoped defects if smoke exposes them.

Use the `$incremental-implementation` skill.

Do this:
- run the adapted smoke harness for `097d97e914ca220f`
- run the adapted smoke harness for `da59436e63915185`
- inspect `summary.md` and the relevant `checkpoints.jsonl` rows for each
- compare the observed outputs against the expectations locked in Packet `R5.75-4.1`
- if smoke is clean, report that honestly with no code change
- if a packet-scoped defect in already-landed `R5.75-4` work is exposed, fix only that defect, rerun the affected smoke, and hand back a focused diff

Rules:
- do not widen beyond already-landed `R5.75-4` code/test/fixture scope
- preserve the conservative delegated parent-visible bar for `da59436e63915185`
- run impact analysis before editing any affected indexed symbol
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: smoke outputs, whether any fix was needed, changed files if any, and the exact commit message you recommend if a real fix landed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-4.6` adapted smoke outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the evidence or any resulting fix is ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the adapted smoke was run honestly
- whether `097d97e914ca220f` stayed boring/conservative without unsupported troubleshooting overclaim
- whether `da59436e63915185` preserved the conservative delegated parent-visible bar while losing unrelated stronger overclaim
- whether any resulting fix stayed packet-scoped

State clearly whether Packet `R5.75-4.6` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-4.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- stay inside already-landed `R5.75-4` scope
- rerun only the necessary adapted smoke after any code change
- run impact analysis before editing affected indexed symbols
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made, smoke rerun results, and the exact commit message you recommend.
```

Commit guidance:
- fix commit message example: `fix: address R5.75-4 adapted smoke findings`
- no-op outcome example: `no commit; Packet R5.75-4.6 adapted smoke already clean`
````

---

## Packet 7 Prompt — Task R5.75-4.7

````text
/goal Land Packet `R5.75-4.7` from `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a docs-closeout -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-4.7` only, assuming Packets `R5.75-4.0` through `R5.75-4.6` are already landed.

Packet `R5.75-4.7` scope only:
- update `docs/specs/r5/R5_75/MAP.md` only after the packet has actually closed
- record `R5.75-4` promoted history, the adapted smoke witnesses used for promotion, the packet-local anti-flap result, any bounded `progress_acceptance` proof that landed, and `R5.75-5` as the active next seam

Out of scope:
- any new analyzer behavior changes
- any fresh fixture-family work
- any smoke reruns not strictly needed to support the MAP wording
- opening `R5.75-5` implementation work

Hard rules:
1. Spawn a fresh implementation/doc-closeout subagent first on `GPT-5.4` `high`.
2. Its prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. If the subagent finds that `R5.75-4` is not honestly ready for promotion, stop and report that instead of editing `MAP.md` optimistically.
4. If a real MAP update lands, commit it before review.
5. Then spawn a fresh review subagent on `GPT-5.4` `high` using a `/goal ` prompt with `$code-review-and-quality`.
6. If review flags issues with the closeout wording or packet truthfulness, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
7. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
8. Run `gitnexus_detect_changes()` before every real commit.
9. Do not start `R5.75-5` implementation in this packet.

Implementation/doc-closeout subagent prompt to send:

```text
/goal Execute Packet `R5.75-4.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: update the main `R5.75` routing hub only if Packet `R5.75-4` is actually closed and promoted.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
- docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- confirm Tasks `R5.75-4.1` through `R5.75-4.6` are actually complete
- if the packet is honestly ready, update `docs/specs/r5/R5_75/MAP.md` to record `R5.75-4` promotion and `R5.75-5` as next
- if the packet is not honestly ready, stop and report the blocker instead of editing the map
- keep the change docs-only

Return with: whether the packet was honestly promotable, changed files if any, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-4.7` MAP closeout outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the routing update is truthful and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether `MAP.md` was updated only after the packet actually closed
- whether the smoke witnesses and anti-flap result are summarized honestly
- whether any bounded `progress_acceptance` result is represented correctly
- whether `R5.75-5` is named as next without starting it here

State clearly whether Packet `R5.75-4.7` is review-clean or requires changes.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-4.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- keep the work docs-only
- do not start `R5.75-5`
- run `gitnexus_detect_changes()` before handing back for any real commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: promote R5.75-4 and route to R5.75-5`
- no-op outcome example: `no commit; Packet R5.75-4 not ready for MAP promotion`
- fix commit message example: `fix: address R5.75-4.7 review findings`
````
