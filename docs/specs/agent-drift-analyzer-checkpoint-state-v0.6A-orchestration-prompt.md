/goal Land Packet `v0.6A` from `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `v0.6A` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `AGENTS.md`

Packet `v0.6A` scope only:

- lock the checkpoint-state contract and packet boundary in repo docs
- add one analyzer-owned `DriftState` contract and state builder
- widen checkpoint export to `v0.3` with per-class state

Out of scope:

- Packet `v0.6B`
- sentinel cutover work except what is strictly required for compile/test compatibility from `v0.6A`
- typed outcome evidence / `ToolOutput` redesign
- reopening `v0.5A` or `v0.5B`
- unrelated cleanup or refactors

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `v0.6A` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `v0.6B`.

Required implementation verification wall for Packet `v0.6A`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet v0.6A only from `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `v0.6A`:
- lock the checkpoint-state contract and packet boundary in repo docs
- add one analyzer-owned `DriftState` contract and state builder
- widen checkpoint export to `v0.3` with per-class state

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-analyzer/src/scoring/mod.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/export_bundle.rs`
- `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs`
- `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `v0.6A`
- use minimal edits that satisfy the packet
- keep sentinel behavior changes out unless needed only for compile/test compatibility
- do not implement typed outcome evidence or Packet `v0.6B`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet v0.6A implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `v0.6A` from:
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`

Focus:
- correctness of analyzer-owned `DriftState`
- correctness of `v0.3` checkpoint export
- test adequacy for checkpoint state and export
- architecture fit with analyzer-owned semantics
- security/performance regressions if any
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet v0.6A only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet v0.6A verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `v0.6A` issues
- do not broaden into Packet `v0.6B` or typed outcome evidence
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `v0.6A` checks needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add analyzer checkpoint state v0.6a`
- fix commit message example: `fix: address packet v0.6a review findings`

Your job is not done when code lands. Your job is done only when Packet `v0.6A` is committed and a fresh review subagent reports it review-clean.
