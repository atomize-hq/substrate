# R6-GAP-DET-REPLAY-STALL — Concurrent Replay Output Attribution

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / DECISION REQUIRED** within
`R6-REPLAY`. Witness commit `60cde3dd7` preserves the trusted `CTX-R6-02` behavioral red. Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`.
The current gate is DECISION REQUIRED `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`; no Rust edit is authorized before explicit operator acceptance.

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

The preserved test is red because the analyzer attributes repeated-failure evidence to the successful
sibling calls `421` and `475` instead of the truthful failed calls `420` and `474`. The input contract,
stall classification, and score disposition are otherwise correct.

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

Recorded GitNexus evidence reports `pair_output_rows` as LOW risk (`1` direct caller, `17` indexed
impacts, `0` affected processes, `1` module), while its `build_command_attempts` caller/context is
HIGH (`16` direct indexed callers/tests). Repository rules require warning before proceeding when
impact is HIGH.

After these docs are committed and fresh-review-clean, refresh both upstream impacts and issue a
structured `DECISION REQUIRED` report with ID `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`. Do not edit
Rust until the operator explicitly accepts the bounded one-file change with the proof wall below.
If refreshed impact is CRITICAL or changes the owning boundary, recommend stopping instead.

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

## Commit, Review, And Exit

Before every commit, stage only intended files, run
`npx gitnexus detect-changes --scope staged -r 97a0-substrate`, `git diff --cached --check`, and
inspect `git diff --cached`. Commit each batch atomically and dispatch a fresh built-in `default`
reviewer. Apply findings in a new commit and repeat with a fresh reviewer until clean.

This packet exits only after the docs gate, operator impact acceptance, one-file production fix,
exact proof, proof receipt, and narrow authority transition are each committed and fresh-review-clean.
The transition completes `CTX-R6-02`, clears the active packet, and returns control to still-active
`R6-REPLAY` with `CTX-R6-06` and its family wall next. It must not activate `R6-CLOSE` or start R7/R8.
