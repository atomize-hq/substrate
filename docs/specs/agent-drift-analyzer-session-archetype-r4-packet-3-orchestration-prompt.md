/goal Land Packet `R4-3` from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R4-3` only, assuming Packets
`R4-1` and `R4-2` are already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`
- `docs/specs/DESIGN-r4-validation-and-labeling-protocol.md`
- `AGENTS.md`

Packet `R4-3` scope only:

- extend analyzer summary output with compact archetype inspection
- add analyzer regression coverage for the four initial archetype shapes plus the required
  ambiguous, transition, delegated, PR-response, proof-closeout, and failing-verification cases
- lock the fixture-manifest authority for the first R4 labeling matrix

Out of scope:

- checkpoint schema/export contract changes beyond what `R4-1` already landed
- new archetype-derivation semantics beyond the minimum needed to satisfy the regression walls
- sentinel replay/live compatibility
- replay/live operator presentation
- `R4-4+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact
   analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
5. Commit the landed Packet `R4-3` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt
   must start with `/goal ` and must explicitly instruct the subagent to use the
   `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R4-4`.

Required implementation verification wall for Packet `R4-3`:

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R4-3 only from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R4-1 and R4-2 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R4-3`:
- extend analyzer summary output with compact archetype inspection
- add analyzer regression coverage for the four initial archetype shapes plus the required ambiguous, transition, delegated, PR-response, proof-closeout, and failing-verification cases
- lock the fixture-manifest authority for the first R4 labeling matrix

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`
- `docs/specs/DESIGN-r4-validation-and-labeling-protocol.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-analyzer/tests/export_bundle.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R4-3`
- keep summary rendering compact and operator-readable
- use the fixture manifest as the reviewable authority for expected labels, confidence ceilings/floors, decisive evidence, and counter-evidence
- do not expand into sentinel compatibility, replay/live presentation, or `R4-4+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R4-3 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R4-3` from:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`

Focus:
- correctness and readability of summary rendering
- adequacy of regression coverage for the required R4 label matrix
- determinism and packet-scope discipline of the new tests
- whether the fixture-manifest authority is actually aligned with the tests

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R4-3 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R4-3 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R4-3` issues
- do not broaden into Packet `R4-4+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add r4 archetype summary and regression walls`
- fix commit message example: `fix: address r4-3 review findings`

Your job is done only when Packet `R4-3` is committed and a fresh review subagent reports it
review-clean.
