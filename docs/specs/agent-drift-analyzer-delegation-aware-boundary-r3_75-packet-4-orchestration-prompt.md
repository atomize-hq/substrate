/goal Land Packet `R3.75-4` from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3.75-4` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R3.75-4` scope only:

- add bounded delegated regression coverage without reopening the `R2` acceptance corpus
- prove conservative classification for the two known delegated sessions
- keep delegated sessions explicitly excluded from the non-subagent success-tail corpus
- preserve the already-landed `R2` corpus outcomes

Out of scope:

- new checkpoint schema fields
- sentinel replay/live changes
- widening delegated support beyond the two bounded cases
- `R7`-style parent/child stitching semantics
- `R4+` semantics
- unrelated cleanup

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3.75-4` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not broaden beyond Packet `R3.75-4`.

Required implementation verification wall for Packet `R3.75-4`:

```bash
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Optional broader wall before handoff if fixture churn is larger than expected:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting gate if Rust files change:

```bash
cargo fmt --all -- --check
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3.75-4 only from `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3.75-4`:
- add bounded delegated regression coverage without reopening the `R2` acceptance corpus
- prove conservative classification for:
  - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
  - `019e9406-6736-79a2-946b-8a603e557422`
- preserve the existing `R2` acceptance corpus outcomes and exclusion boundary

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
- `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`
- `crates/agent-drift-analyzer/tests/fixtures/acceptance/README.md`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3.75-4`
- do not turn the bounded delegated regressions into full delegated-session support
- keep the `R2` success-tail corpus unchanged and explicitly non-subagent
- treat separate child rollout files as the reason these delegated cases remain conservative, not as permission to stitch richer semantics here
- run the required verification wall
- if Rust files changed, run `cargo fmt --all -- --check`
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3.75-4 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3.75-4` from:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Focus:
- correctness of the bounded delegated regression scope
- correctness of the two delegated-case assertions
- correctness of the unchanged `R2` acceptance-corpus boundary
- test adequacy for conservative non-single-agent classification
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3.75-4 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3.75-4 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3.75-4` issues
- do not broaden into richer delegated semantics, sentinel changes, or `R4+` work
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `R3.75-4` checks needed to restore confidence
- if Rust files changed, run `cargo fmt --all -- --check`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `test: add bounded delegated regressions for r3.75`
- fix commit message example: `fix: address r3.75-4 review findings`

Your job is not done when code lands. Your job is done only when Packet `R3.75-4` is committed and a fresh review subagent reports it review-clean.
