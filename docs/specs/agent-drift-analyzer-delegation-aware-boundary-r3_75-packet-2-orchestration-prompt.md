/goal Land Packet `R3.75-2` from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3.75-2` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R3.75-2` scope only:

- add analyzer-local delegation types and conservative marker harvesting
- keep delegation state internal to checkpoint analysis
- reuse visible `multi_agent_v1`, `spawn_agent`, `wait_agent`, and `close_agent` markers
- reuse conservative context/command signals
- preserve checkpoint schema `v0.4`

Out of scope:

- public checkpoint export of `DelegationContext`
- sentinel replay/live changes
- analyzer reporting/presentation work beyond what is strictly needed for internal wiring
- `R3.75-3` derivation/presentation work except where required for internal contract plumbing
- `R3.75-4` regression coverage
- `R4+` semantics
- unrelated refactors

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3.75-2` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R3.75-3`.

Required implementation verification wall for Packet `R3.75-2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting gate if Rust files change:

```bash
cargo fmt --all -- --check
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3.75-2 only from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3.75-2`:
- add analyzer-local delegation types and conservative marker harvesting
- keep delegation state internal to checkpoint analysis
- preserve checkpoint schema `v0.4`
- keep the first landing parent-rollout-local even when child work lives in a separate rollout file

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/context/mod.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3.75-2`
- do not serialize delegation state into exported checkpoint DTOs
- do not bump checkpoint schema
- do not broaden into sentinel or `R4+` semantics
- run the required verification wall
- if Rust files changed, run `cargo fmt --all -- --check`
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3.75-2 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3.75-2` from:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Focus:
- correctness of the analyzer-local delegation contract
- correctness of marker harvesting from visible rollout/context surfaces
- correctness of the no-schema-bump boundary
- architecture fit with checkpoint-analysis internals
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3.75-2 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3.75-2 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3.75-2` issues
- do not broaden into export, sentinel, or `R3.75-3+` work
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `R3.75-2` checks needed to restore confidence
- if Rust files changed, run `cargo fmt --all -- --check`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add analyzer-local delegation boundary for r3.75`
- fix commit message example: `fix: address r3.75-2 review findings`

Your job is not done when code lands. Your job is done only when Packet `R3.75-2` is committed and a fresh review subagent reports it review-clean.
