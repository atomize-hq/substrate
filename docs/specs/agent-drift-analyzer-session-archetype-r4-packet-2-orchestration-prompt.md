/goal Land Packet `R4-2` from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R4-2` only, assuming Packet
`R4-1` is already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `docs/specs/DESIGN-r4-kickoff-prior-signals-and-guardrails.md`
- `AGENTS.md`

Packet `R4-2` scope only:

- derive deterministic checkpoint-local `session_archetype` during checkpoint analysis
- introduce or lock the lower deterministic intent-evidence seam used by archetype aggregation
- consume landed delegation topology / child-visibility state to cap confidence or add
  counter-evidence
- add low-hanging command-role and file-role interpretation so broad command families do not become
  direct label proxies
- populate deterministic `supporting_evidence` and `counter_evidence`

Out of scope:

- analyzer summary rendering
- regression-fixture expansion beyond the derivation coverage needed to prove `R4-2`
- sentinel replay/live compatibility
- replay/live operator presentation
- `R5+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact
   analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
5. Commit the landed Packet `R4-2` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt
   must start with `/goal ` and must explicitly instruct the subagent to use the
   `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R4-3`.

Required implementation verification wall for Packet `R4-2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R4-2 only from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet R4-1 is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R4-2`:
- derive deterministic checkpoint-local `session_archetype` during checkpoint analysis
- introduce or lock the lower deterministic intent-evidence seam used by archetype aggregation
- consume landed delegation topology / child-visibility state to cap confidence or add counter-evidence
- add low-hanging command-role and file-role interpretation so broad command families do not become direct label proxies
- populate deterministic `supporting_evidence` and `counter_evidence`

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `docs/specs/DESIGN-r4-kickoff-prior-signals-and-guardrails.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R4-2`
- keep the first landing behavior-first; kickoff priors must stay deferred, disabled, or weak-only behind a narrow internal gate
- use the landed delegation boundary as an input seam; do not reopen delegation detection itself
- ambiguous cases must degrade confidence instead of inventing a fifth archetype
- do not implement analyzer summary rendering, sentinel compatibility, replay/live presentation, or `R4-3+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R4-2 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R4-2` from:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`

Focus:
- correctness of checkpoint-local archetype derivation
- correctness and conservatism of delegation-aware confidence capping
- correctness of command-role and file-role interpretation
- evidence and counter-evidence quality
- strict packet-scope adherence and refusal to introduce `R5` semantics

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R4-2 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R4-2 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R4-2` issues
- do not broaden into Packet `R4-3+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: derive session archetype evidence for r4`
- fix commit message example: `fix: address r4-2 review findings`

Your job is done only when Packet `R4-2` is committed and a fresh review subagent reports it
review-clean.
