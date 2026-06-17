/goal Land Packet `R2` from `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R2` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R2` scope only:

- lock the analyzer-local acceptance-fixture contract in repo docs
- add analyzer test support for checked-in acceptance cases
- freeze the screened `R1E` acceptance corpus into committed analyzer fixtures
- add the analyzer acceptance wall for final `dead_end_thrash` posture
- lock fixture maintenance and exclusion rules near the acceptance seam

Out of scope:

- new analyzer semantics
- sentinel runtime, input, scheduler, or operator-surface changes
- widening the acceptance corpus beyond the screened `R1E` sessions
- new bounded replay or live-proof claims
- `R3` turn-context work
- `R4` session-archetype work
- `R5` progress work
- unrelated cleanup or refactors

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R2` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not broaden beyond Packet `R2`.

Required implementation verification wall for Packet `R2`:

```bash
cargo build -p agent-drift-analyzer
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting gate if Rust files change:

```bash
cargo fmt --all -- --check
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R2 only from `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R2`:
- lock the analyzer-local acceptance-fixture contract in repo docs
- add analyzer test support for checked-in acceptance cases
- freeze the screened `R1E` acceptance corpus into committed analyzer fixtures
- add the analyzer acceptance wall for final `dead_end_thrash` posture
- lock fixture maintenance and exclusion rules near the acceptance seam

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Live files and directories to inspect before editing:
- `crates/agent-drift-analyzer/tests/support/mod.rs`
- `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`
- `crates/agent-drift-sentinel/tests/fixtures/`
- `target/hybrid-drift-evals/`

Acceptance corpus to freeze:
- cleared controls:
  - `019e93fa-60d4-73d1-9092-014130b60e14`
  - `019e940c-a91b-7fe0-a967-b0bdd595b581`
  - `019e943c-668e-7a03-992b-6a98cf3055da`
- representative recovered sticky case:
  - `019e894a-86c9-71e3-b57b-e3d3285f0988`

Sessions that must remain excluded:
- delegated:
  - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
  - `019e9406-6736-79a2-946b-8a603e557422`
- non-success-tail:
  - `019e9401-9d69-7190-a43e-9ee3be08b369`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R2`
- keep the acceptance seam analyzer-local; do not route checks through sentinel
- committed tests must not read from `target/` or `~/.codex` at test time
- do not change analyzer semantics or widen the proof corpus
- prefer reduced committed fixtures, but preserve enough row identity to reproduce the landed analyzer result honestly
- run the required verification wall
- if Rust files changed, run `cargo fmt --all -- --check`
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R2 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R2` from:
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md`

Focus:
- correctness of the checked-in acceptance fixture contract
- correctness of the included and excluded corpus boundaries
- correctness of analyzer-local test helpers and acceptance assertions
- test adequacy for final `dead_end_thrash` posture
- architecture fit with analyzer-owned acceptance rather than sentinel-owned proof
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R2 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R2 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R2` issues
- do not broaden into new analyzer semantics, sentinel changes, or `R3+` work
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `R2` checks needed to restore confidence
- if Rust files changed, run `cargo fmt --all -- --check`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add analyzer acceptance fixtures for r2`
- fix commit message example: `fix: address packet r2 review findings`

Your job is not done when code lands. Your job is done only when Packet `R2` is committed and a fresh review subagent reports it review-clean.
