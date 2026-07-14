# Tasks: R6-GAP-DET-REPLAY-STALL

Status: **ACTIVE PACKET / TASK `.0` DOCS-GATE REVIEW-CLEAN / DECISION REQUIRED** within
`R6-REPLAY`. Trusted witness `60cde3dd7` preserves `CTX-R6-02` red. Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`.
The current gate is DECISION REQUIRED `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`; no Rust edit is authorized before explicit operator acceptance.

## Required Commit Gate

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Every commit receives a fresh built-in `default` review. Findings are fixed in a new bounded commit
and sent to another fresh reviewer until `REVIEW CLEAN`.

## Task Ledger

- [x] **R6-GAP-DET-REPLAY-STALL.0 — Commit and independently review the packet docs gate.**
  - Files: exactly this SPEC/PLAN/TASKS plus the already-authorized `expected.json` correction from
    clean target `493 -> 495` to truthful target `492 -> 495`, retaining sibling `493 -> 496`.
  - Verify: JSON parse; local Markdown links/status wording; required staged gate; `git diff --check`.
  - Review: fresh built-in `default`; bounded docs/annotation fix commits; fresh reviewer until clean.
  - Boundary: do not run the witness or edit Rust until this task is committed and review-clean.
  - Current receipt: full series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. Task `.0` is complete.

- [ ] **R6-GAP-DET-REPLAY-STALL.1 — Refresh impact and obtain the required operator decision.**
  - Prerequisite: Task 0 review-clean.
  - Refresh upstream impact for `pair_output_rows`; inspect `build_command_attempts` as its caller and
    packet-wide context.
  - Current evidence: `pair_output_rows` LOW (`1` direct caller, `17` indexed impacts, `0` processes,
    `1` module); `build_command_attempts` context HIGH (`16` direct indexed callers/tests).
  - Required output: structured `DECISION REQUIRED` ID
    `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`, recommending acceptance of the locked one-file fix plus
    full proof wall. Record explicit operator acceptance before any Rust edit.
  - Stop if refreshed risk is CRITICAL or the authorized boundary changes.

- [ ] **R6-GAP-DET-REPLAY-STALL.2 — Reconfirm the preserved behavioral red.**
  - Prerequisites: Tasks 0 and 1 complete.
  - Run exact `acceptance_fixtures_integrated_true_stall_stays_active`.
  - Expected: exit `101` at the event-`420` truthful-evidence assertion; checkpoint `5` is still
    `TroubleshootingFrontier / Stalled / Medium`, score `Active / 30 / High`, flagged, but repeated
    evidence is misattributed to sibling calls `421`/`475` rather than target calls `420`/`474`.
  - Unexpected green/different red: preserve output and stop for authority reconciliation.

- [ ] **R6-GAP-DET-REPLAY-STALL.3 — Land the smallest call-ID-aware pairing fix.**
  - File: only `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`.
  - Change: call-ID-exclusive scanning when a command ID exists; no positional fallback in that lane;
    unchanged legacy adjacency/Error/Unknown fallback without an ID.
  - Unit regression: `checkpoints_pair_concurrent_tool_outputs_by_call_id` proves interleaved calls
    receive only their matching outputs and outcomes.
  - Impact every additional existing symbol before editing it; do not edit compactor, progress,
    scoring, fixtures, schemas, or later-phase surfaces.

- [ ] **R6-GAP-DET-REPLAY-STALL.4 — Run exact proof, commit, and close fresh review.**
  - Verify in order:

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

  - The compactor normalization command is optional confirmation, not edit authority.
  - Acceptance: target evidence events `420`/`474`; siblings `421`/`475` excluded; checkpoint/score
    unchanged; clean target `492 -> 495`, sibling `493 -> 496`; all walls green.
  - Record exact results in this TASKS and the replay ledger, use the required commit gate, commit the
    bounded fix/proof receipt, and obtain fresh built-in `default` `REVIEW CLEAN`.

- [ ] **R6-GAP-DET-REPLAY-STALL.5 — Land and review the narrow packet transition.**
  - Prerequisite: Task 4 review-clean.
  - Mark `CTX-R6-02` and this packet complete, clear the active packet, keep `R6-REPLAY` active, and
    make `CTX-R6-06` then the family wall next.
  - Commit authority-only, run the staged gate, dispatch a fresh built-in `default` reviewer, and fix
    findings in new commits until clean.
  - Stop the packet after review-clean transition. Do not activate `R6-CLOSE` or start R7/R8.

## Preserved Witness Receipt

- Fixture: `019eb311-c7ce-7f50-ae13-b51a5b5461c3`, annotated real depth-1 built-in `default`
  subagent rollout.
- Commit: `60cde3dd7` (`test: preserve true-stall replay evidence red`).
- Raw target failures: `420 -> 423`, `474 -> 477`; successful siblings: `421 -> 425`, `475 -> 479`.
- Later clean target/sibling: `492 -> 495`, `493 -> 496`.
- Trusted disposition: checkpoint `5` `TroubleshootingFrontier / Stalled / Medium`; flagged
  `dead_end_thrash Active / 30 / High`.
- Current defect: evidence names the successful siblings because `pair_output_rows` uses positional
  pairing across concurrent calls.
