/goal Land Packet `R3-1` from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3-1` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R3-1` scope only:

- lock the `R3` packet boundary in repo docs
- lock the explicit `v0.4` checkpoint-contract decision in repo docs
- lock the deliberate differences from AgentLens and DST-inspired patterns
- keep `R4`, `R5`, and `R6` explicitly out of scope

Out of scope:

- any Rust code changes
- analyzer or sentinel schema implementation
- archetype, progress, or scorer semantics
- unrelated doc cleanup

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. After the implementation subagent finishes, review its actual diff and doc-verification results yourself before proceeding.
4. Commit the landed Packet `R3-1` implementation before any review subagent is dispatched.
5. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
6. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
7. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
8. Commit every fix batch before sending a fresh review subagent back through the review loop.
9. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
10. Run `gitnexus_detect_changes()` before every commit.
11. Do not start Packet `R3-2`.

Required implementation verification wall for Packet `R3-1`:

- doc review against:
  - `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
  - `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
  - `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
  - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
  - `docs/specs/hybrid-drift-sentinel-implementation-order.md`

Implementation subagent prompt to send:

```text
/goal Implement Packet R3-1 only from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3-1`:
- lock the `R3` packet boundary in repo docs
- lock the explicit `v0.4` checkpoint-contract decision in repo docs
- lock the deliberate differences from AgentLens and DST-inspired patterns
- keep `R4`, `R5`, and `R6` explicitly out of scope

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Execution rules:
- stay strictly within Packet `R3-1`
- preserve the narrow `R3` interpretation: checkpoint-local turn state only
- do not start code implementation or broaden into `R3-2+`
- return with: changed files, doc checks performed, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3-1 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3-1` from:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Focus:
- correctness of the `R3` packet boundary
- correctness of the `v0.4` schema-positioning decision
- correctness of the AgentLens/DST interpretation boundaries
- strict packet-scope adherence

Review the docs first.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3-1 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3-1 doc verification.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3-1` issues
- do not broaden into code changes or Packet `R3-2+`
- return with: exact fixes made, checks rerun, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `docs: lock turn-context r3 packet boundary`
- fix commit message example: `docs: address r3-1 review findings`

Your job is not done when docs land. Your job is done only when Packet `R3-1` is committed and a fresh review subagent reports it review-clean.
