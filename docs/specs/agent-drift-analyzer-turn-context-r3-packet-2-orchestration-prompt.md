/goal Land Packet `R3-2` from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3-2` only, assuming Packet `R3-1` is already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R3-2` scope only:

- add analyzer-owned `TurnContext`, `TurnActivityMix`, and `TurnExecutionMode`
- export `turn_context` from `agent_drift_analyzer::Checkpoint`
- widen newly emitted checkpoints from `v0.3` to `v0.4`

Out of scope:

- current-turn derivation logic beyond the smallest scaffolding needed for compile/test progress
- summary rendering
- sentinel compatibility changes
- `R3-3+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3-2` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R3-3`.

Required implementation verification wall for Packet `R3-2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3-2 only from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet R3-1 is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3-2`:
- add analyzer-owned `TurnContext`, `TurnActivityMix`, and `TurnExecutionMode`
- export `turn_context` from `agent_drift_analyzer::Checkpoint`
- widen newly emitted checkpoints from `v0.3` to `v0.4`

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3-2`
- do not implement current-turn derivation, summary rendering, or sentinel compatibility beyond what is minimally required for compile/test continuity
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3-2 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3-2` from:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Focus:
- correctness of the analyzer-owned turn-context schema types
- correctness of the `v0.4` export cutover
- architecture fit with existing checkpoint DTO ownership
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3-2 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3-2 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3-2` issues
- do not broaden into Packet `R3-3+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add turn-context checkpoint schema for r3`
- fix commit message example: `fix: address r3-2 review findings`

Your job is done only when Packet `R3-2` is committed and a fresh review subagent reports it review-clean.
