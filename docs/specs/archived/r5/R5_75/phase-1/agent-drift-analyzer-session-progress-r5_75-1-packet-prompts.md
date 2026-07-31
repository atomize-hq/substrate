# R5.75-1 Packet Prompts

Status: draft orchestration prompts created on 2026-06-12 for the active `R5.75-1` packet family.

These prompts treat each task in
`docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
as one sequential packet prompt:

- Packet 1 → Task `R5.75-1.1`
- Packet 2 → Task `R5.75-1.2`
- Packet 3 → Task `R5.75-1.3`
- Packet 4 → Task `R5.75-1.4`
- Packet 5 → Task `R5.75-1.5`
- Packet 6 → Task `R5.75-1.6`

Global rules for every packet prompt below:

1. The parent session is the orchestration agent; it should not do the packet work locally unless
   delegated execution is unavailable.
2. The orchestration agent must spawn a fresh `GPT-5.4` subagent on `high` for implementation
   first.
3. Every implementation or fix subagent prompt must start with `/goal ` and must explicitly tell
   the subagent to use the `$incremental-implementation` skill.
4. After the implementation work is landed, the orchestration agent must commit before dispatching
   review.
5. The orchestration agent must then spawn a fresh `GPT-5.4` subagent on `high` for review.
6. Every review subagent prompt must start with `/goal ` and must explicitly tell the subagent to
   use the `$code-review-and-quality` skill.
7. If the review subagent flags issues, the orchestration agent must spawn a fresh `GPT-5.4`
   `high` fix subagent whose prompt starts with `/goal ` and explicitly uses the
   `$incremental-implementation` skill.
8. Commit every non-empty implementation/fix batch before sending a fresh review subagent back
   through the loop. Do not fabricate empty commits for pure verification packets that produce no
   file changes.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not advance to the next packet until the current packet is committed and a fresh review
    subagent reports it review-clean, or for verification-only packets with no file changes,
    explicitly reports there was nothing to commit.

Shared authority for all packets:

- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

---

## Packet 1 Prompt — Task R5.75-1.1

````text
/goal Land Packet `R5.75-1.1` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.1` only, assuming `R5.75-0` is already landed and no later `R5.75-1.2+` packet has started.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.1` scope only:
- add regressions for giant pasted user prompts whose concrete ask appears inside the body
- keep the work focused on `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove those long prompt/scaffold bodies resolve to the concrete task ask rather than the entire pasted body

Out of scope:
- preserved boilerplate-target regressions beyond what this packet strictly needs
- objective-candidate ordering changes in `checkpoint/mod.rs`
- chosen-row condensation logic
- automated full-suite closeout
- named native/adapted smoke sessions
- Packet `R5.75-1.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust test function or helper, the implementation subagent must honor the repo GitNexus rule: run impact analysis before modifying any function/method/symbol it plans to touch, and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent should not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `R5.75-1.1` implementation before dispatching review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-1.2`.

Required verification wall for Packet `R5.75-1.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-1.1` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `R5.75-0` is already landed and no later `R5.75-1.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.1`:
- add regressions for giant pasted user prompts whose concrete ask appears inside the body
- keep the scope focused on `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove those long pasted prompt/scaffold bodies resolve to the concrete task ask rather than the full pasted body

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Files to inspect before editing:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs` for current objective behavior only; do not edit it in this packet unless the task docs prove it is strictly required

GitNexus requirements:
- before modifying any Rust function or helper, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.1`
- keep the change reviewable and regression-focused
- do not broaden into candidate ordering or condensation logic
- run `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, any residual risk, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-1.1` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.1` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- whether the new regressions capture giant pasted user prompts whose real ask appears later in the body
- whether the tests are scoped to Packet `R5.75-1.1` only
- whether the tests would actually fail if the concrete-task condensation behavior regressed
- whether the packet stayed out of `checkpoint/mod.rs` behavior changes

Review the tests first, then the verification story.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-1.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `R5.75-1.1` verification command.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.75-1.1` issues
- keep the work regression-focused
- do not broaden into Packet `R5.75-1.2+`
- run impact analysis before editing affected Rust test functions/helpers
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add r5.75 giant-prompt objective regressions`
- fix commit message example: `fix: address r5.75-1.1 review findings`

Your job is done only when Packet `R5.75-1.1` is committed and a fresh review subagent reports it review-clean.
````

---

## Packet 2 Prompt — Task R5.75-1.2

````text
/goal Land Packet `R5.75-1.2` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.2` only, assuming Packet `R5.75-1.1` is already landed.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.2` scope only:
- add regressions proving preserved boilerplate-target requests still survive condensation
- keep the work focused on `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove requests to analyze, compare, explain, or edit `AGENTS.md`, `<skill>`, `Available skills`, and tooling scaffolds remain intact

Out of scope:
- candidate ordering changes
- condensation helper implementation changes
- full-suite validation and smoke sessions
- Packet `R5.75-1.3+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust test function or helper, the implementation subagent must run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent should not implement locally unless the subagent path is unavailable.
5. Inspect the implementation diff and verification results yourself before proceeding.
6. Commit the landed Packet `R5.75-1.2` implementation before review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` using a `/goal ` prompt and the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before the next fresh review.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-1.3`.

Required verification wall for Packet `R5.75-1.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-1.2` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet `R5.75-1.1` is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.2`:
- add regressions proving preserved boilerplate-target requests still survive condensation
- keep the work focused on `crates/agent-drift-analyzer/tests/checkpoints.rs`
- prove the analyzer preserves real requests to analyze, compare, explain, or edit `AGENTS.md`, `<skill>`, `Available skills`, and tooling scaffolds

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Files to inspect before editing:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs` for preservation behavior context only; do not edit it in this packet unless the task docs prove it is strictly required

GitNexus requirements:
- before modifying any Rust function or helper, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.2`
- preserve the deliberate boilerplate-target correctness wall
- do not broaden into candidate ordering or condensation logic
- run `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`

Return with: changed files, tests run, any residual risk, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-1.2` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.2` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- whether the regressions truly preserve real boilerplate-target requests
- whether the tests would catch accidental shortening of those requests into misleading fragments
- whether the packet stayed limited to regression coverage only

Review the tests first, then the verification story.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-1.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `R5.75-1.2` verification command.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.75-1.2` issues
- keep the work limited to preservation regressions
- do not broaden into Packet `R5.75-1.3+`
- run impact analysis before editing affected Rust test functions/helpers
- rerun `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: preserve r5.75 boilerplate-target objective cases`
- fix commit message example: `fix: address r5.75-1.2 review findings`

Your job is done only when Packet `R5.75-1.2` is committed and a fresh review subagent reports it review-clean.
````

---

## Packet 3 Prompt — Task R5.75-1.3

````text
/goal Land Packet `R5.75-1.3` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.3` only, assuming Packets `R5.75-1.1` and `R5.75-1.2` are already landed.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.3` scope only:
- refine objective-candidate ordering so longer same-priority bodies do not win by length
- keep `/goal`, short imperative asks, steer pivots, and concrete workspace/action-target phrases preferred
- implement the narrow ordering fix in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- update or add the needed regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`

Out of scope:
- chosen-row condensation beyond what ordering alone requires
- packet-wide smoke sessions
- Packet `R5.75-1.4+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct in `checkpoint/mod.rs`, the implementation subagent must run GitNexus impact analysis on the target symbol and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent should not implement locally unless the subagent path is unavailable.
5. Inspect the implementation diff and verification results yourself before proceeding.
6. Commit the landed Packet `R5.75-1.3` implementation before review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` using a `/goal ` prompt and the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before the next fresh review.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-1.4`.

Required verification wall for Packet `R5.75-1.3`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-1.3` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `R5.75-1.1` and `R5.75-1.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.3`:
- refine objective-candidate ordering so longer same-priority bodies do not win by length
- keep `/goal`, short imperative asks, steer pivots, and concrete workspace/action-target phrases preferred
- implement the narrow ordering fix in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- update or add the needed regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.3`
- keep the change narrow to candidate ordering
- do not implement the chosen-row condensation logic yet unless the packet authority proves it is unavoidable and you call that out explicitly
- run:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`

Return with: changed files, tests run, any residual risk, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-1.3` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.3` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- correctness of the ordering change
- proof that the longer same-priority body no longer wins
- proof that `/goal`, short imperative asks, and steer pivots remain preferred
- packet-scope discipline: no early condensation logic and no widening into later packets

Review the tests first, then the implementation and verification story.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-1.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `R5.75-1.3` verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.75-1.3` issues
- do not broaden into Packet `R5.75-1.4+`
- run impact analysis before editing affected symbols
- rerun:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: prefer shorter concrete objectives in r5.75`
- fix commit message example: `fix: address r5.75-1.3 review findings`

Your job is done only when Packet `R5.75-1.3` is committed and a fresh review subagent reports it review-clean.
````

---

## Packet 4 Prompt — Task R5.75-1.4

````text
/goal Land Packet `R5.75-1.4` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.4` only, assuming Packets `R5.75-1.1` through `R5.75-1.3` are already landed.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.4` scope only:
- condense the chosen objective row down to the true concrete ask when possible
- keep the stored checkpoint objective reduced to the shortest concrete task ask that preserves the real target
- implement the chosen-row condensation in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- update/add the needed regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`

Out of scope:
- redoing Packet `R5.75-1.3` candidate ordering unless strictly required to finish this packet cleanly
- manual smoke sessions
- Packet `R5.75-1.5+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct in `checkpoint/mod.rs`, the implementation subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent should not implement locally unless the subagent path is unavailable.
5. Inspect the implementation diff and verification results yourself before proceeding.
6. Commit the landed Packet `R5.75-1.4` implementation before review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high` using a `/goal ` prompt and the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before the next fresh review.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-1.5`.

Required verification wall for Packet `R5.75-1.4`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-1.4` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `R5.75-1.1` through `R5.75-1.3` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.4`:
- condense the chosen objective row down to the true concrete ask when possible
- keep the stored checkpoint objective reduced to the shortest concrete task ask that preserves the real target
- implement the chosen-row condensation in `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- update/add the needed regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.4`
- preserve deliberate boilerplate-target requests while condensing large chosen rows
- do not widen into manual smoke or adapted fixture-family work
- run:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`

Return with: changed files, tests run, any residual risk, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-1.4` implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.4` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- correctness of the chosen-row condensation behavior
- proof that the stored objective becomes the concrete ask instead of the full pasted body
- proof that preserved boilerplate-target requests remain intact
- packet-scope adherence

Review the tests first, then the implementation and verification story.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-1.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `R5.75-1.4` verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.75-1.4` issues
- do not broaden into Packet `R5.75-1.5+`
- run impact analysis before editing affected symbols
- rerun:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: condense chosen objective rows for r5.75`
- fix commit message example: `fix: address r5.75-1.4 review findings`

Your job is done only when Packet `R5.75-1.4` is committed and a fresh review subagent reports it review-clean.
````

---

## Packet 5 Prompt — Task R5.75-1.5

````text
/goal Land Packet `R5.75-1.5` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation/verification -> review -> fix loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.5` only, assuming Packets `R5.75-1.1` through `R5.75-1.4` are already landed.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.5` scope only:
- run the packet’s automated validation gates
- confirm:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- do not widen into new code or test changes unless a failing validation requires a packet-scoped fix

Out of scope:
- native/adapted smoke sessions
- new features or refactors
- Packet `R5.75-1.6`

Hard rules:
1. Spawn a fresh implementation/verification subagent first. Use `GPT-5.4` on `high`.
2. The implementation/verification subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. The orchestration agent should not fabricate code changes just to create a commit. If the verification packet produces no file changes, record that honestly.
4. If validation reveals a real packet-scoped defect in the already-landed Packet `R5.75-1.1` through `R5.75-1.4` changes, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
5. Commit any non-empty fix batch before review.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If review flags issues with the verification story or a resulting fix, spawn a fresh fix subagent and repeat the loop.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not start Packet `R5.75-1.6`.

Required verification wall for Packet `R5.75-1.5`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-1.5` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `R5.75-1.1` through `R5.75-1.4` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.5`:
- run the automated validation gates
- confirm:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- do not widen into new changes unless a failing validation requires a packet-scoped fix

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.5`
- if both commands pass and no file changes are needed, say so explicitly and recommend no commit
- if a failing test requires a fix, keep it packet-scoped to the already-landed `R5.75-1.*` objective work and call out exactly which packet it corrects
- if you do edit code, run impact analysis before modifying affected symbols and run `gitnexus_detect_changes()` before handing back for commit

Return with: whether changes were needed, tests run, any fixes made, residual risk, and the exact commit message you recommend if there was a non-empty change batch.
```

Review subagent prompt to send after the implementation/fix commit, or after a clean no-change validation pass:

```text
/goal Review the Packet `R5.75-1.5` verification result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether the automated gate story is sufficient and honest.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.5` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- whether both required analyzer test gates were actually run
- whether any resulting fix stayed packet-scoped
- whether the orchestrator honestly reported a no-change verification packet if nothing needed committing

List findings by severity.
State clearly whether Packet `R5.75-1.5` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues or the verification run surfaces a defect:

```text
/goal Address the Packet `R5.75-1.5` review findings or validation failures only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant analyzer gate commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings or failing-gate details:
- [PASTE REVIEW FINDINGS OR FAILING COMMAND DETAILS HERE]

Rules:
- fix only the packet-scoped issue revealed by automated validation
- if code changes are required, run impact analysis before editing affected symbols
- rerun:
  - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- run `gitnexus_detect_changes()` before handing back for commit if there are file changes

Return with: exact fixes made, tests rerun, residual risks if any, and the exact commit message you recommend if there was a non-empty change batch.
```

Commit guidance:
- implementation/fix commit message example: `fix: address r5.75 automated gate findings`
- if no files changed, do not invent a no-op commit; report the packet as verification-only with no commit required

Your job is done only when Packet `R5.75-1.5` is review-clean and the automated gate story is complete and honest.
````

---

## Packet 6 Prompt — Task R5.75-1.6

````text
/goal Land Packet `R5.75-1.6` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped smoke/verification -> review -> fix loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-1.6` only, assuming Packets `R5.75-1.1` through `R5.75-1.5` are already landed and the automated gates are already green.

Packet authority:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Packet `R5.75-1.6` scope only:
- run the named native smoke sessions:
  - `019eb430-6f9a-7a03-9a63-cb451b654795`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
  - `019eb98e-3c16-7ba0-92f9-0085654b470c`
- run the adapted smoke session:
  - `05a56cc51632982b`
- inspect `summary.md` and the first `checkpoints.jsonl` rows
- prove the first checkpoint objective resolves to the concrete task ask rather than the pasted scaffold body

Out of scope:
- adapted fixture-family landing work
- later packet families
- unrelated analyzer cleanup

Hard rules:
1. Spawn a fresh smoke/verification subagent first. Use `GPT-5.4` on `high`.
2. The smoke/verification subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. The orchestration agent should not fabricate code changes just to create a commit. If the smoke packet produces no file changes, record that honestly.
4. If smoke reveals a real packet-scoped defect, spawn a fresh fix subagent on `GPT-5.4` `high` whose prompt starts with `/goal ` and explicitly uses `$incremental-implementation`.
5. Commit any non-empty fix batch before review.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If review flags issues with the smoke evidence or a resulting fix, spawn a fresh fix subagent and repeat the loop.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Do not declare `R5.75-1` complete unless this packet proves the hard gate from the spec/plan/tasks docs.

Required smoke wall for Packet `R5.75-1.6`:

Use the native/adapted command blocks already recorded in:
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`

Inspect at minimum:
- `sed -n '1,80p' "$ANALYZER_OUT/summary.md"`
- `sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"`

Smoke/verification subagent prompt to send:

```text
/goal Execute Packet `R5.75-1.6` only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `R5.75-1.1` through `R5.75-1.5` are already landed and the automated gates are green.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-1.6`:
- run the named native smoke sessions:
  - `019eb430-6f9a-7a03-9a63-cb451b654795`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
  - `019eb98e-3c16-7ba0-92f9-0085654b470c`
- run the adapted smoke session:
  - `05a56cc51632982b`
- inspect `summary.md` and the first `checkpoints.jsonl` rows
- prove the first checkpoint objective resolves to the concrete task ask rather than the pasted scaffold body

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `R5.75-1.6`
- run the native/adapted command blocks from the spec exactly as written unless an environment issue forces a narrow adjustment that you explain
- inspect `summary.md` and the first `checkpoints.jsonl` rows for each session
- if the smoke passes and no file changes are required, say so explicitly and recommend no commit
- if smoke exposes a defect that requires a fix, keep it packet-scoped to the `R5.75-1` objective work; run impact analysis before editing affected symbols and run `gitnexus_detect_changes()` before handing back for commit

Return with: smoke results per session, whether changes were needed, any fixes made, residual risks, and the exact commit message you recommend if there was a non-empty change batch.
```

Review subagent prompt to send after the smoke/fix pass:

```text
/goal Review the Packet `R5.75-1.6` smoke and hard-gate evidence in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether `R5.75-1` is actually ready to close.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-1.6` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Focus:
- whether all named native/adapted sessions were actually run
- whether the first checkpoint objective resolves to the concrete task ask for each named session
- whether preserved boilerplate-target behavior still appears intact alongside the smoke story
- whether any resulting fix stayed packet-scoped and honest
- whether the packet satisfies the hard gate for closing `R5.75-1`

List findings by severity.
State clearly whether Packet `R5.75-1.6` is review-clean and whether the overall `R5.75-1` hard gate is satisfied.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues or smoke exposes a defect:

```text
/goal Address the Packet `R5.75-1.6` smoke/review findings only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant named smoke sessions and inspections.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md`
- `AGENTS.md`

Review findings or smoke failures to fix:
- [PASTE REVIEW FINDINGS OR FAILING SESSION DETAILS HERE]

Rules:
- fix only the packet-scoped issue revealed by smoke/review
- if code changes are required, run impact analysis before editing affected symbols
- rerun the relevant named smoke sessions and inspect `summary.md` plus the first `checkpoints.jsonl` rows again
- run `gitnexus_detect_changes()` before handing back for commit if there are file changes

Return with: exact fixes made, smoke sessions rerun, residual risks if any, and the exact commit message you recommend if there was a non-empty change batch.
```

Commit guidance:
- implementation/fix commit message example: `fix: address r5.75 smoke-gate findings`
- if no files changed, do not invent a no-op commit; report the packet as smoke-only with no commit required

Your job is done only when Packet `R5.75-1.6` is review-clean and the overall `R5.75-1` hard gate is satisfied.
````
