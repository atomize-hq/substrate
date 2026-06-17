/goal Land Packet `R5.75-0` from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-0` only, assuming the
`R5.75` map docs are already present and no later `R5.75-1+` packet has started.

Packet authority:

- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Packet `R5.75-0` scope only:

- add landed-status reconciliation language to the `R5.5` plan
- reconcile the `R5.5` task ledger so landed work is not still presented as open implementation
  debt
- update root landing-order authority to name `R5.75` as the active pre-`R6` family
- run a cross-doc authority audit after the edits
- run baseline validation and optional control smoke before promotion

Out of scope:

- any analyzer-semantic code changes
- any compactor/sentinel/scorer work
- any new fixture-family landings
- `R5.75-1+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. The implementation subagent should stay docs-only unless the packet authority itself proves a
   touched root-order doc also needs synchronization.
4. The orchestration agent should not implement the packet locally unless the subagent path is
   unavailable; the default path is delegated implementation.
5. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
6. Run `gitnexus_detect_changes()` before every commit, including docs-only commits.
7. Commit the landed Packet `R5.75-0` implementation before any review subagent is dispatched.
8. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
9. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
10. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its
    prompt must start with `/goal ` and must explicitly instruct the subagent to use the
    `$incremental-implementation` skill.
11. Commit every fix batch before sending a fresh review subagent back through the review loop.
12. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
13. Do not start Packet `R5.75-1`.

Required verification wall for Packet `R5.75-0`:

```bash
rg -n "R5\\.5|R5\\.75|R6" \
  docs/specs/r5/R5_75/MAP.md \
  docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md \
  docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md \
  HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md

cargo test -p agent-drift-analyzer -- --nocapture
```

Optional control smoke for Packet `R5.75-0`:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="019eb430-6f9a-7a03-9a63-cb451b654795"
export SMOKE_ROOT="target/r5_75-smoke/r5_75-0/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5.75-0 only from `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the R5.75 map docs already exist and no later packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.75-0`:
- add landed-status reconciliation language to the `R5.5` plan
- reconcile the `R5.5` task ledger so landed work is not still presented as open implementation debt
- update root landing-order authority to name `R5.75` as the active pre-`R6` family
- run a cross-doc authority audit after the edits
- run baseline validation and optional control smoke before promotion

Read first:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Files to inspect before editing:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md` only if it still names the active next family

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly within Packet `R5.75-0`
- keep the packet docs-only
- preserve historical R5.5 facts while reconciling landed vs remaining scope
- keep `R6` closed and route remaining work into `R5.75`
- run the required authority-audit command
- run `cargo test -p agent-drift-analyzer -- --nocapture`
- if you also touch `docs/specs/hybrid-drift-sentinel-implementation-order.md`, explain why the sync was necessary
- run `gitnexus_detect_changes()` before handing back for commit

Return with: changed files, verification run, whether optional control smoke was run or intentionally skipped, any open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5.75-0 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-0` from:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Focus:
- correctness of the landed-vs-remaining reconciliation language
- correctness of the `R5.75` vs `R6` routing
- consistency across all touched authority docs
- packet-scope adherence
- verification story quality, including whether optional smoke was honestly recorded as run or skipped

Review the docs first, then the verification story.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5.75-0 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5.75-0 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/R5_75/MAP.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-plan.md`
- `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `AGENTS.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.75-0` issues
- keep the work docs-only
- do not broaden into `R5.75-1+`
- rerun the smallest sufficient verification subset needed to restore confidence
- rerun `cargo test -p agent-drift-analyzer -- --nocapture` if the touched authority docs could affect the claimed packet state
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification rerun, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `docs: reconcile r5.5 status for r5.75 packet 0`
- fix commit message example: `fix: address r5.75-0 review findings`

Your job is done only when Packet `R5.75-0` is committed and a fresh review subagent reports it
review-clean.
