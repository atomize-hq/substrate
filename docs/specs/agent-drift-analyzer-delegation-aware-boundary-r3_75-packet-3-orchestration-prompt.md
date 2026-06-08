/goal Land Packet `R3.75-3` from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3.75-3` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R3.75-3` scope only:

- derive checkpoint-local delegation topology and child-work visibility
- populate supporting and counter-evidence deterministically
- add compact analyzer-owned delegation inspection if needed for packet proof
- keep the first landing descriptive only

Out of scope:

- checkpoint schema export of `DelegationContext`
- sentinel replay/live changes
- delegated-session progress or drift semantics
- `R3.75-4` delegated regression coverage, except minimal fixture plumbing strictly required to complete packet `R3.75-3`
- `R4+` semantics
- unrelated cleanup

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3.75-3` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R3.75-4`.

Required implementation verification wall for Packet `R3.75-3`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Optional broader wall before handoff if confidence is shaky:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting gate if Rust files change:

```bash
cargo fmt --all -- --check
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3.75-3 only from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3.75-3`:
- derive checkpoint-local delegation topology and child-work visibility
- populate deterministic supporting and counter-evidence
- add compact analyzer-owned delegation inspection only if needed for packet proof
- keep the packet descriptive only

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/export_bundle.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3.75-3`
- ordinary non-delegated sessions should remain `single_agent` with `child_work_visibility = none`
- delegated parent-orchestration cases should degrade conservatively to non-single-agent, `partial`, or `opaque`
- do not invent child certainty when the child rollout is separate and not joined
- do not add `R4`, `R5`, `R6`, or `R7` semantics
- run the required verification wall
- if Rust files changed, run `cargo fmt --all -- --check`
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3.75-3 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3.75-3` from:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Focus:
- correctness of delegation topology and child-visibility derivation
- correctness of conservative degradation for separate child rollout opacity
- correctness and readability of supporting/counter-evidence output
- architecture fit with analyzer-owned reporting only
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3.75-3 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3.75-3 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3.75-3` issues
- do not broaden into sentinel changes, delegated regression corpus work, or `R4+` semantics
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `R3.75-3` checks needed to restore confidence
- if Rust files changed, run `cargo fmt --all -- --check`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: derive delegation visibility for r3.75`
- fix commit message example: `fix: address r3.75-3 review findings`

Your job is not done when code lands. Your job is done only when Packet `R3.75-3` is committed and a fresh review subagent reports it review-clean.
