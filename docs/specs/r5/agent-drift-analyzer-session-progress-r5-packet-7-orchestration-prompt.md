/goal Land Packet `R5-7` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5-7` only, assuming Packets `R5-0` through `R5-6` are already landed.

Packet authority:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`
- `AGENTS.md`

Packet `R5-7` scope only:

- add the dedicated progress acceptance harness and committed fixture corpus
- add the fixture README and expected-case docs
- prove a bounded semantic corpus across the core dimensions
- keep the legacy R2 acceptance wall stable unless explicitly widened
- run the full analyzer and sentinel verification walls
- update the R5 docs with final implemented fixture coverage and status

Out of scope:

- new progress semantics beyond what earlier packets already defined
- scorer retuning
- scheduler changes
- any new post-R5 follow-on packet work

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R5-7` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not widen beyond the bounded R5 acceptance wall.

Required verification wall for Packet `R5-7`:

```bash
cargo fmt --all -- --check
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5-7 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5-0 through R5-6 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5-7`:
- add the dedicated progress acceptance harness and committed fixture corpus
- add the fixture README and expected-case docs
- prove a bounded semantic corpus across the core dimensions
- keep the legacy R2 acceptance wall stable unless explicitly widened
- run the full analyzer and sentinel verification walls
- update the R5 docs with final implemented fixture coverage and status

Read first:
- `AGENTS.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

Live-code and artifact files to inspect before editing:
- `crates/agent-drift-analyzer/tests/support/mod.rs`
- `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
- `crates/agent-drift-analyzer/tests/fixtures/acceptance/README.md`
- any new `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- any new `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5-7`
- keep the new acceptance corpus committed, stable, and independent of mutable `target/` or `~/.codex` state at test time
- preserve the legacy R2 acceptance wall unless the packet explicitly and intentionally widens it
- do not change scorer or scheduler behavior
- run the full verification wall

Return with: changed files, tests run, corpus shape, any deferred cases, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5-7 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, tasks, fixtures, and validation protocol, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5-7` from:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

Focus:
- correctness and stability of the acceptance harness and committed corpus
- adequacy of fixture coverage relative to the fixture manifest
- preservation of the legacy R2 acceptance wall
- correctness of final docs alignment and verification story
- refusal to smuggle in scorer or scheduler changes

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5-7 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5-7 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5-7` issues
- do not broaden beyond the bounded acceptance-wall packet
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: add r5 progress acceptance wall`
- fix commit message example: `fix: address r5-7 review findings`

Your job is done only when Packet `R5-7` is committed and a fresh review subagent reports it review-clean.
