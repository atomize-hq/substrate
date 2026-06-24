# Adapted External Robustness Fixture Family (R5.75-5) Packet Prompts

Status: orchestration prompts created on 2026-06-23 against the live
`docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md`
ledger. Use `docs/specs/r5/R5_75/MAP.md` plus the current SPEC/PLAN/TASKS ledger as the live
execution authority before starting any packet.

Each prompt below is self-contained: paste one into a fresh session to land exactly one `R5.75-5`
packet through a GPT-5.4 high implementation -> commit -> review -> fix -> commit loop until
review-clean.

These prompts map:

- Packet 0 → Task `R5.75-5.0`
- Packet 1 → Task `R5.75-5.1`
- Packet 2 → Task `R5.75-5.2`
- Packet 3 → Task `R5.75-5.3`
- Packet 4 → Task `R5.75-5.4`
- Packet 5 → Task `R5.75-5.5`
- Packet 6 → Task `R5.75-5.6`
- Packet 7 → Task `R5.75-5.7`

Shared authority for all packets:

- `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md`
- `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md`
- `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md`
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

## Packet 0 Prompt — Task R5.75-5.0

````text
/goal Land Packet `R5.75-5.0` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.0` only.

Packet `R5.75-5.0` scope only:
- commit the `R5.75-5` SPEC/PLAN/TASKS triplet
- ensure the docs record the native-primary / adapted-secondary boundary
- ensure the docs record the adapted-session routing rule, stretch-external net-new-signal rule, missing fixture-manifest restoration requirement, and prompt-out-of-scope note

Out of scope:
- any fixture commits
- any test harness changes
- any smoke execution
- Packets `R5.75-5.1+`

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
11. Do not start Packet `R5.75-5.1`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: finalize and commit the `R5.75-5` SPEC/PLAN/TASKS docs lock.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- verify the triplet matches Task `R5.75-5.0.1`
- make only doc fixes needed to satisfy the packet acceptance
- keep scope strictly docs-only

Return with: changed files, any doc fixes made, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-5.0` docs lock in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-5.0` against:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Focus:
- whether the primary-vs-secondary authority boundary is explicit and honest
- whether the adapted routing and stretch-external rules are clear enough to guide later packets
- whether the missing fixture-manifest restoration work is made explicit
- whether the docs avoid silently absorbing later packet work

State clearly whether Packet `R5.75-5.0` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-5.0` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.0` doc issues
- keep it docs-only
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: add R5.75-5 adapted robustness triplet`
- fix commit message example: `fix: address R5.75-5.0 review findings`
````

---

## Packet 1 Prompt — Task R5.75-5.1

````text
/goal Land Packet `R5.75-5.1` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.1` only, assuming Packet `R5.75-5.0` is already landed.

Packet `R5.75-5.1` scope only:
- re-read the four adopted adapted sample ids
- record the correct home for each one
- update the tasks ledger with the routing outcome and the packet-owned expectation each adapted case must encode
- explicitly decide whether `05a56cc51632982b` adds net-new objective signal beyond the existing locked corpus

Out of scope:
- fixture commits
- test harness changes
- MAP promotion changes
- Packets `R5.75-5.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the recorded routing/evidence yourself.
5. Commit the tasks-ledger evidence before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.2`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: audit the adopted adapted sessions and record their correct homes plus packet-owned expectations in the tasks ledger.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/R5_75/MAP.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md
- AGENTS.md

Do this:
- inspect the adapted source artifacts for:
  - `05a56cc51632982b`
  - `f47b81f39f2495dd`
  - `097d97e914ca220f`
  - `da59436e63915185`
- inspect any already-saved smoke outputs cited by `R5.75-1` through `R5.75-4`
- update Task `R5.75-5.1.1` with the exact routing outcome for each case
- explicitly record whether `05a56cc51632982b` adds objective signal beyond the locked `R5.75-1` corpus
- keep scope ledger/evidence-only: do not commit fixtures or harness changes yet

Precondition check:
- confirm Packet `R5.75-5.0` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: the recorded routing decisions, exact evidence paths used, changed files, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-5.1` routing audit in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether all four adapted sample ids were audited against live evidence
- whether the correct home for each case is explicit and justified
- whether the `05a56cc51632982b` decision is grounded in the locked objective corpus rather than hand-waving
- whether the packet stayed bounded to routing/audit work and did not drift into fixture implementation

State clearly whether Packet `R5.75-5.1` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-5.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.1` routing/evidence/wording issues
- keep it ledger/evidence-only
- rerun only the inspections needed to correct the findings
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, any refreshed evidence, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: record R5.75-5.1 adapted routing audit`
- fix commit message example: `fix: address R5.75-5.1 review findings`
````

---

## Packet 2 Prompt — Task R5.75-5.2

````text
/goal Land Packet `R5.75-5.2` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.2` only, assuming Packet `R5.75-5.1` is already landed.

Packet `R5.75-5.2` scope only:
- restore or create `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- update the fixture README contract(s) so native primary authority vs adapted secondary robustness is explicit
- keep objective `stretch-external/` wording honest if it remains placeholder-only

Out of scope:
- progress fixture commits
- objective case commits
- analyzer semantic changes
- Packets `R5.75-5.3+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the doc contract changes yourself.
5. Commit the doc contract packet before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.3`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: restore the live fixture authority contract for R5 progress fixtures and adapt it to the new secondary adapted lane.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/design-arch/DESIGN-r5-validation-and-rollout-protocol.md
- docs/specs/archived/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
- crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md
- AGENTS.md

Do this:
- create or restore `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` in the live docs tree
- update the progress and objective fixture README contract wording so native primary authority vs adapted secondary robustness is explicit
- keep scope docs/README-only

Precondition check:
- confirm Packet `R5.75-5.1` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: changed files, the contract decisions you recorded, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-5.2` fixture authority contract updates in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether they are sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the live fixture-manifest doc was restored in the correct location
- whether native primary authority vs adapted secondary robustness is explicit and consistent across docs/READMEs
- whether the contract stays bounded to the adopted repro classes
- whether objective `stretch-external/` wording remains honest

State clearly whether Packet `R5.75-5.2` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-5.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.2` docs/README issues
- keep it docs-only
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: restore R5 progress fixture authority contract`
- fix commit message example: `fix: address R5.75-5.2 review findings`
````

---

## Packet 3 Prompt — Task R5.75-5.3

````text
/goal Land Packet `R5.75-5.3` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.3` only, assuming Packet `R5.75-5.2` is already landed.

Packet `R5.75-5.3` scope only:
- add the explicit adapted-external secondary lane to `progress_acceptance`
- commit the three adapted progress fixtures for:
  - `f47b81f39f2495dd`
  - `097d97e914ca220f`
  - `da59436e63915185`
- update the fixture authority doc and progress README to reflect the committed secondary lane

Out of scope:
- objective `stretch-external/` decisions
- full native/adapted smoke rerun packet
- MAP promotion packet
- Packets `R5.75-5.4+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the committed fixture set, harness diff, and verification output yourself.
5. Commit the packet before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.4`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: commit the adapted-external secondary progress lane and the three bounded adapted progress fixtures.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
- crates/agent-drift-analyzer/tests/progress_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
- target/ranga-validation/runs/f47b81f39f2495dd/compactor/**
- target/ranga-validation/runs/097d97e914ca220f/compactor/**
- target/ranga-validation/runs/da59436e63915185/compactor/**
- AGENTS.md

Do this:
- implement Task `R5.75-5.3.1`
- implement Task `R5.75-5.3.2`
- implement Task `R5.75-5.3.3`
- implement Task `R5.75-5.3.4`
- keep the packet bounded to the progress secondary lane only
- run `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`

Precondition checks:
- confirm Packet `R5.75-5.2` is already landed
- before modifying any indexed Rust symbol in `progress_acceptance.rs`, run GitNexus impact analysis and report any HIGH or CRITICAL blast radius
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: changed files, committed fixture ids, exact verification commands run, any GitNexus impact findings, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-5.3` progress secondary lane in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the harness clearly distinguishes native authority, adapted secondary cases, and synthetic support cases
- whether each adapted fixture encodes the correct packet-owned expectation without smuggling in new semantics
- whether the fixture corpus remains deterministic and bounded
- whether the README and fixture-manifest docs match the actual committed corpus

State clearly whether Packet `R5.75-5.3` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-5.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.3` issues
- stay scoped to the progress secondary lane and its supporting docs
- rerun only the minimum verification needed after each fix batch
- run `gitnexus_detect_changes()` before handing back for commit
- before modifying any indexed Rust symbol in the fix path, rerun GitNexus impact analysis

Return with: exact fixes made, exact verification reruns, any GitNexus impact findings, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add R5.75-5 adapted progress robustness fixtures`
- fix commit message example: `fix: address R5.75-5.3 review findings`
````

---

## Packet 4 Prompt — Task R5.75-5.4

````text
/goal Land Packet `R5.75-5.4` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.4` only, assuming Packet `R5.75-5.3` is already landed.

Packet `R5.75-5.4` scope only:
- decide whether `05a56cc51632982b` earns one bounded `stretch-external` objective case
- if yes, add exactly one bounded `stretch-external` case plus supporting harness/docs updates
- if no, keep `stretch-external/` placeholder-only and record the explicit no-op rationale

Out of scope:
- progress secondary lane changes
- full smoke rerun packet
- MAP promotion packet
- Packets `R5.75-5.5+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the decision and supporting evidence yourself.
5. Commit the packet if it produced non-empty changes before dispatching review. If the packet is an honest no-op with no file changes, do not fabricate an empty commit.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.5`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: decide honestly whether adapted objective case `05a56cc51632982b` adds net-new signal beyond the locked `R5.75-1` corpus and either land one bounded stretch-external case or record the no-op rationale.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/orchestration-evaluate-ask-anchor/**
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/orchestration-marker-free-boilerplate-exclusion/**
- target/ranga-validation/runs/05a56cc51632982b/compactor/**
- AGENTS.md

Do this:
- audit whether `05a56cc51632982b` adds net-new signal beyond the locked objective corpus
- if yes, land exactly one bounded `stretch-external` case and the minimum supporting harness/docs changes
- if no, keep `stretch-external/` placeholder-only and record the explicit cross-reference/no-op rationale
- run `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`

Precondition checks:
- confirm Packet `R5.75-5.3` is already landed
- before modifying any indexed Rust symbol in `objective_acceptance.rs`, run GitNexus impact analysis and report any HIGH or CRITICAL blast radius
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: the yes/no decision, exact evidence used, changed files (or explicit no-op), exact verification commands run, any GitNexus impact findings, and the exact commit message you recommend if a commit is needed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-5.4` objective stretch-external decision in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the add-one-case vs honest-no-op decision is actually grounded in the locked objective corpus
- whether any landed stretch-external case is truly bounded and non-duplicative
- whether placeholder-only wording remains honest if no case landed
- whether the packet stayed scoped to objective robustness only

State clearly whether Packet `R5.75-5.4` is review-clean or requires changes. If the packet was a no-op, say so explicitly and review the reasoning rather than demanding an empty commit.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-5.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.4` issues
- stay scoped to the stretch-external decision and its minimal supporting files
- rerun only the minimum verification needed after each fix batch
- run `gitnexus_detect_changes()` before handing back for commit
- before modifying any indexed Rust symbol in the fix path, rerun GitNexus impact analysis

Return with: exact fixes made, exact verification reruns, any GitNexus impact findings, and the exact commit message you recommend if a commit is needed.
```

Commit guidance:
- implementation commit message example: `test: add R5.75-5 stretch-external objective case`
- no-op outcome: do not create an empty commit; return the explicit no-op rationale for review instead
- fix commit message example: `fix: address R5.75-5.4 review findings`
````

---

## Packet 5 Prompt — Task R5.75-5.5

````text
/goal Land Packet `R5.75-5.5` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped verification -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.5` only, assuming Packet `R5.75-5.4` is already landed.

Packet `R5.75-5.5` scope only:
- run the focused objective/progress acceptance gates and the full analyzer wall
- record the verification outcome honestly
- make no unrelated changes

Out of scope:
- new fixture design work
- smoke rerun packet
- MAP promotion packet
- Packets `R5.75-5.6+`

Hard rules:
1. Spawn a fresh implementation/verification subagent first. Use `GPT-5.4` on `high`.
2. The implementation/verification subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the verification subagent finishes, inspect the verification evidence yourself.
5. If the packet required a ledger/doc update, commit it before dispatching review. If the packet is an honest no-op with no file changes, do not fabricate an empty commit.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.6`.

Implementation/verification subagent prompt to send:

```text
/goal Land Packet `R5.75-5.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: run the packet's focused acceptance gates and full analyzer wall, then record the result honestly.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- AGENTS.md

Do this:
- run `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
- run `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
- run `cargo test -p agent-drift-analyzer -- --nocapture`
- if the packet needs a ledger/doc note to capture the verification outcome, add only that minimal update
- if no files need to change, report the packet as an honest no-op verification step

Precondition check:
- confirm Packet `R5.75-5.4` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: exact commands run, their results, any changed files (or explicit no-op), and the exact commit message you recommend if a commit is needed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-5.5` verification outcome in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the required acceptance/analyzer commands were actually run
- whether the reported outcomes are accurate and complete
- whether any doc/ledger note stays strictly packet-scoped
- whether a no-op outcome, if reported, is honest and justified

State clearly whether Packet `R5.75-5.5` is review-clean or requires changes. If the packet was a no-op, review the verification story rather than demanding an empty commit.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-5.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.5` verification-story issues
- rerun only the minimum commands needed to correct the findings
- keep any file edits minimal and packet-scoped
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, exact verification reruns, and the exact commit message you recommend if a commit is needed.
```

Commit guidance:
- implementation commit message example: `docs: record R5.75-5.5 acceptance wall rerun`
- no-op outcome: do not create an empty commit; return the explicit verification result for review instead
- fix commit message example: `fix: address R5.75-5.5 review findings`
````

---

## Packet 6 Prompt — Task R5.75-5.6

````text
/goal Land Packet `R5.75-5.6` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit-if-needed -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.6` only, assuming Packet `R5.75-5.5` is already landed.

Packet `R5.75-5.6` scope only:
- rerun the full named native + adapted smoke set
- inspect `summary.md` and relevant `checkpoints.jsonl` rows for every session
- record the smoke outcome honestly in the packet evidence/ledger

Out of scope:
- fixture redesign
- MAP promotion packet
- Packets `R5.75-5.7+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the smoke outputs yourself before proceeding.
5. If the packet produced a ledger/doc update, commit it before dispatching review. If it produced no file changes, do not fabricate an empty commit.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not start Packet `R5.75-5.7`.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: rerun the full named native + adapted smoke set, inspect the outputs, and record the outcome honestly.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- rerun the full named native smoke set from the companion spec
- rerun the full named adapted smoke set from the companion spec
- inspect `summary.md` and relevant `checkpoints.jsonl` rows for every session
- update the packet evidence/ledger only as needed to record the result honestly
- if no files need to change, report the packet as an honest no-op smoke-evidence step

Precondition check:
- confirm Packet `R5.75-5.5` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: exact smoke commands run, exact output paths inspected, the verdict for each named session, any changed files (or explicit no-op), and the exact commit message you recommend if a commit is needed.
```

Review subagent prompt:

```text
/goal Review the Packet `R5.75-5.6` full smoke rerun in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether the full named native and adapted smoke set was actually rerun
- whether the recorded outcome inspects real `summary.md` / `checkpoints.jsonl` evidence instead of only exit codes
- whether earlier packet expectations are preserved honestly across the full smoke set
- whether adapted sessions remain explicitly secondary robustness proof

State clearly whether Packet `R5.75-5.6` is review-clean or requires changes. If the packet was a no-op, review the smoke evidence story rather than demanding an empty commit.
```

Fix subagent prompt template:

```text
/goal Address the review findings for Packet `R5.75-5.6` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.6` smoke-evidence issues
- rerun only the minimum smoke steps needed to correct the findings
- keep any file edits minimal and packet-scoped
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, exact smoke reruns, and the exact commit message you recommend if a commit is needed.
```

Commit guidance:
- implementation commit message example: `docs: record R5.75-5.6 full smoke rerun`
- no-op outcome: do not create an empty commit; return the explicit smoke-evidence result for review instead
- fix commit message example: `fix: address R5.75-5.6 review findings`
````

---

## Packet 7 Prompt — Task R5.75-5.7

````text
/goal Land Packet `R5.75-5.7` from `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-5.7` only, assuming Packet `R5.75-5.6` is already landed.

Packet `R5.75-5.7` scope only:
- update `docs/specs/r5/R5_75/MAP.md` after the packet is honestly closed
- mark `R5.75-5` as promoted history
- route `R5.75-6` as the next active seam only if the promotion gate is truly met

Out of scope:
- any new fixture work
- any fresh smoke or harness redesign
- any `R5.75-6` implementation

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and explicitly use `$incremental-implementation`.
3. The orchestration agent must not implement locally unless delegated execution is unavailable.
4. After the implementation subagent finishes, inspect the MAP diff yourself.
5. Commit the routing update before dispatching review.
6. Then spawn a fresh review subagent on `GPT-5.4` `high`.
7. The review subagent prompt must start with `/goal ` and explicitly use `$code-review-and-quality`.
8. If review finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
9. Commit every non-empty fix batch before sending a fresh review subagent back through the loop.
10. Run `gitnexus_detect_changes()` before every real commit.
11. Do not begin `R5.75-6` inside this packet.

Implementation subagent prompt to send:

```text
/goal Land Packet `R5.75-5.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`: update the main R5.75 routing hub after the adapted external robustness fixture family has actually closed.

Use the `$incremental-implementation` skill.

Read first:
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
- docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Do this:
- verify Tasks `R5.75-5.1` through `R5.75-5.6` are honestly complete first
- update `docs/specs/r5/R5_75/MAP.md` only if the packet is truly promoted
- record the adapted fixture-family outcome honestly and route `R5.75-6` next only if warranted
- keep scope strictly to the MAP/routing update

Precondition check:
- confirm Packet `R5.75-5.6` is already landed
- if the prerequisite is missing, stop and report it instead of compensating here

Return with: changed files, the routing update made, and the exact commit message you recommend.
```

Review subagent prompt:

```text
/goal Review the already-landed Packet `R5.75-5.7` routing update in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and ready to keep.

Use the `$code-review-and-quality` skill.

Focus:
- whether `MAP.md` was updated only after the packet truly closed
- whether `R5.75-5` is recorded honestly as promoted history
- whether `R5.75-6` is routed next only because the promotion gate was actually met
- whether the update stays scoped to routing authority rather than reopening implementation work

State clearly whether Packet `R5.75-5.7` is review-clean or requires changes. Keep any required changes packet-scoped.
```

Fix subagent prompt template:

```text
/goal Address the review findings for already-landed Packet `R5.75-5.7` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only Packet `R5.75-5.7` routing-authority issues
- keep it docs-only
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: promote R5.75-5 and route R5.75-6`
- fix commit message example: `fix: address R5.75-5.7 review findings`
````
