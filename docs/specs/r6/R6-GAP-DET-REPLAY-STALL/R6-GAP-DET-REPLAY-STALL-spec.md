# R6-GAP-DET-REPLAY-STALL — Concurrent Replay Output Attribution

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / TASK `.1` ACCEPTANCE RECEIPT
`d788f45c9` REVIEW-CLEAN / TASK `.2` COMPLETE / TASK `.2A` DECISION REQUIRED / TASK `.3`
INCOMPLETE** within `R6-REPLAY`. Witness commit `60cde3dd7` preserves the trusted `CTX-R6-02`
behavioral red. Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` +
`9edf564d3` and Task `.1` decision receipt `d788f45c9` each received fresh independent built-in
`default` `REVIEW CLEAN`. The preserved-red reconfirmation in Task `.2` is complete. An uncommitted,
unproven `attempt.rs` pairing candidate makes its new unit test green but exposes progress and
recovery reds outside the prior decision's scope. Decision
`R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01` now gates any further implementation. `CTX-R6-06`, the
family wall, `R6-CLOSE`, and R7/R8 remain blocked.

## Objective And Preserved Witness

Close only the analyzer's concurrent tool-output attribution seam without changing the trusted
true-stall contract, compactor output, progress semantics, or `dead_end_thrash` scoring.

The selected annotated real rollout `019eb311-c7ce-7f50-ae13-b51a5b5461c3` is a depth-1 built-in
`default` subagent found by the expanded 2026-06-01 through 2026-07-13 screen. The screen covered all
`3,423` rollouts (`1,112` user, `2,298` subagent, `13` unspecified), including `1,611`
verifier-bearing sessions and `225` sessions with at least two failed verifier calls without requiring
identical commands. Of those, `161` were subagent candidates and `189` had non-identical failed
commands; `224` analyzed successfully, `0` were malformed, and this rollout was the sole exact match.
The known `019f3a4f-89c4-73a2-9bc0-2589b7430485` analysis was stopped after two minutes because its
existing result already adjudicated it as `Regressing` and later `Advancing`.

The fixture preserves these call-ID-linked facts:

- failed target verifier calls `420 -> 423` and `474 -> 477`, both exit `101` with the same failing
  assertion;
- concurrent sibling verifier calls `421 -> 425` and `475 -> 479`, both exit `0`;
- later clean target verifier `492 -> 495`, concurrent with clean sibling `493 -> 496`;
- checkpoint `5` is `TroubleshootingFrontier / Stalled / Medium`, exposes
  `FailureSignatureRepeated`, and produces flagged `dead_end_thrash Active / 30 / High`.

At committed baseline, the preserved test is red because the analyzer attributes repeated-failure
evidence to successful sibling calls `421` and `475` instead of truthful failed calls `420` and `474`.
The statement that the input contract, stall classification, and score disposition were otherwise
correct applies only to that committed pre-candidate baseline. It is not a forward claim: truthful
pairing exposes a separate progress-selection red described below.

## Owning Seam And Locked Fix

`pair_output_rows` in `crates/agent-drift-analyzer/src/checkpoint/attempt.rs` currently pairs by
position and stops at the next `ToolCall`. With concurrent calls, that leaves the first call without
its output and lets the second call consume both outputs. The compactor already preserves correct
`call_id` values in each call/output `dedupe_identity`; no compactor edit is authorized.

The fix is locked to this behavior:

1. when the command row has a `call_id`, scan the concurrent rows and pair only output rows carrying
   the same `call_id`;
2. do not use positional, error, or unknown-row fallback when a command `call_id` exists;
3. when no command `call_id` exists, retain the current legacy adjacency behavior, including
   `ToolOutput`, `Error`, and immediately adjacent output-shaped `Unknown` handling; and
4. add one interleaving unit test named `checkpoints_pair_concurrent_tool_outputs_by_call_id` that
   proves each concurrent call receives only its own outcome.

Default production scope is only `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`. Do not edit
progress construction, scoring, fixture rows, compactor normalization, public schemas, replay
presentation, R7, or R8. If this boundary cannot make the exact preserved witness green, stop under
the escalation contract rather than widening it.

## Mandatory Pre-Edit Decision Gate

Task `.1` recorded GitNexus evidence reports `pair_output_rows` as LOW risk (`1` direct caller, `17`
indexed impacts, `0` affected processes, `1` module), while its `build_command_attempts`
caller/context is HIGH (`16` direct indexed callers/tests). Repository rules require warning before
proceeding when impact is HIGH.

After these docs are committed and fresh-review-clean, refresh both upstream impacts and issue a
structured `DECISION REQUIRED` report with ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`. That gate
was satisfied on 2026-07-14 when the operator explicitly selected Option A, accepting the bounded
one-file change and proof wall below. This acceptance does not authorize scope outside
`attempt.rs`; if refreshed impact becomes CRITICAL or changes the owning boundary, recommend
stopping instead.

## Acceptance And Exact Proof

The implementation is acceptable only when:

- the interleaving unit test proves call-ID-exclusive output pairing;
- exact `CTX-R6-02` is green with evidence on target events `420` and `474`, not siblings `421` and
  `475`;
- checkpoint `5` remains `TroubleshootingFrontier / Stalled / Medium` with
  `FailureSignatureRepeated` and no direct frontier-advance signals;
- its score remains flagged `Active / 30 / High`;
- the progress corpus, checkpoint family, and full analyzer remain green; and
- the clean counter-evidence remains target `492 -> 495` and sibling `493 -> 496`.

Run in order:

```bash
cargo test -p agent-drift-analyzer checkpoints_pair_concurrent_tool_outputs_by_call_id -- --nocapture
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance progress_acceptance_cases_match_expected_progress_contract -- --nocapture
cargo test -p agent-drift-analyzer checkpoint -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-session-compactor --test normalization normalization_maps_rollout_events_into_provenance_preserving_rows -- --exact --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer --all-targets
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
git diff --check
```

The compactor normalization command is a read-only optional cross-check of the already-correct
call-ID contract; no compactor change is allowed.

## Post-Pairing Stop State And Decision Gate

Task `.2` reconfirmed the exact committed-baseline red with exit `101` at
`acceptance_fixtures.rs:463`, on the event-`420` truthful-evidence assertion. The preserved log is
`/tmp/r6-replay-stall-pre-edit-red.log`.

The Task `.3` candidate remains uncommitted and unproven. Its patch SHA-256 is
`030d3d3e97640ba8fd4cf71f29f886b2273ec6e653aefa622aacaeb7057fdae7`, with a preservation copy at
`/tmp/r6-pairing-fix-secondary-red.patch`. It establishes truthful diagnostic pairing:

- `420 -> 423` is `Failed`, exit `101`; `421 -> 425` is `Clean`, exit `0`;
- `474 -> 477` is `Failed`, exit `101`; `475 -> 479` is `Clean`, exit `0`; and
- `checkpoints_pair_concurrent_tool_outputs_by_call_id` passes `1 / 1`.

That is not sufficient acceptance. Exact `CTX-R6-02` exits `101` at
`acceptance_fixtures.rs:455`, where actual progress is `InsufficientEvidence` instead of the locked
`Stalled`; the log is `/tmp/r6-ctx-r6-02-green.log`. The test stops there before evidence and score
assertions. Candidate walls are red: analyzer library `144` pass / `13` fail; checkpoints integration
`109` pass / `24` fail; `dead_end_thrash` `15` pass / `3` fail; and progress corpus `2` pass / `1`
fail. Synthetic helpers also carry unmatched call IDs.

Diagnostics show `assess_troubleshooting_progress` selects clean sibling `475`, while comparability
excludes the failed target lane, producing `InsufficientEvidence`. Its GitNexus upstream impact is LOW
(`4` direct, `12` total, `0` processes, `2` modules). Shared comparability helpers must not be changed
under the current authority. Separately, frozen `CTX-R6-06` rollout
`019e894a-86c9-71e3-b57b-e3d3285f0988` becomes `HistoricalOnly` instead of locked `Recovered`
because truthful pairing exposes expected-negative check `831 -> 837`, exit `1`. Likely owner
`recovery_state` has HIGH upstream impact (`1` direct, `30` impacted, `3` processes, `2` modules).

The prior `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` decision authorized only the locked
`attempt.rs` change and its wall. It did not authorize progress, recovery, or test-helper scope. Do
not create a nested or successor packet. Same-packet internal gate `.2A` asks:

```text
DECISION REQUIRED
ID: R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01
PHASE/PACKET: R6-REPLAY / R6-GAP-DET-REPLAY-STALL
QUESTION: Preserve the locked Stalled/Active CTX-R6-02 and frozen Recovered CTX-R6-06 contracts and authorize a review-clean same-packet amendment for the smallest call-ID cutover scope beyond attempt.rs, including accepted HIGH recovery_state impact?
REPO EVIDENCE: The uncommitted pairing candidate truthfully links 420->423, 421->425, 474->477, and 475->479, but exact CTX-R6-02 becomes InsufficientEvidence and frozen CTX-R6-06 becomes HistoricalOnly. assess_troubleshooting_progress impact is LOW; likely recovery_state impact is HIGH.
WHY AUTHORITY CANNOT DECIDE: The accepted prior decision authorizes only attempt.rs; progress, recovery, synthetic-helper changes, and HIGH recovery_state impact are outside that boundary.
OPTIONS:
A. Preserve both contracts and authorize a docs-first, fresh-review-clean same-packet amendment for the smallest call-ID cutover scope beyond attempt.rs, including accepted HIGH recovery_state impact, followed by refreshed impact, focused tests, bounded fix, full wall, atomic commits, and fresh review.
B. Reject expansion, discard the uncommitted candidate, and leave CTX-R6-02, CTX-R6-06, the family wall, R6-CLOSE, and R7/R8 blocked.
RECOMMENDATION: A. The candidate proves pairing truth but not the two locked replay contracts; a docs-first same-packet amendment keeps the semantic decision explicit and bounded.
SAFE WORK ALREADY COMPLETED: Task .0 and Task .1 are review-clean; Task .2 is complete; the candidate and every secondary red are preserved but uncommitted.
BLOCKED SCOPE ONLY: Further Rust/test-helper edits, candidate commit, Task .4 proof/commit, CTX-R6-06, family wall, packet transition, R6-CLOSE, and R7/R8.
REPLY FORMAT: DECISION R6-REPLAY-STALL-POST-PAIRING-PROGRESS-01: A|B|explicit alternative
```

## Commit, Review, And Exit

Before every commit, stage only intended files, run
`npx gitnexus detect-changes --scope staged -r 97a0-substrate`, `git diff --cached --check`, and
inspect `git diff --cached`. Commit each batch atomically and dispatch a fresh built-in `default`
reviewer. Apply findings in a new commit and repeat with a fresh reviewer until clean.

This packet exits only after the docs gate, both required operator decisions, an authorized bounded
production fix, exact proof, proof receipt, and narrow authority transition are each committed and
fresh-review-clean. The current candidate is not a commit, proof receipt, or review-clean result.
The transition completes `CTX-R6-02`, clears the active packet, and returns control to still-active
`R6-REPLAY` with `CTX-R6-06` and its family wall next. It must not activate `R6-CLOSE` or start R7/R8.
