# Plan: Agent Drift Analyzer Outcome Evidence R1 Packet Family

## Scope

This plan implements
`docs/specs/agent-drift-analyzer-outcome-evidence-r1-spec.md`.

The old single-packet `R1` shape was first split into three packets, and is now expanded to six
after bounded replay exposed a second analyzer seam:

- `R1A`: docs lock, analyzer outcome-evidence seam, and repeated-failure cutover
- `R1B`: recovery/export semantics plus focused regressions for honest downgrade behavior
- `R1C`: bounded replay diagnosis plus honest status correction when the proof still fails
- `R1D`: analyzer repeated verification-loop semantics and honest recovery fix
- `R1E`: bounded replay re-proof over screened non-subagent sessions plus continuity-note refresh
- `R1F`: sentinel trigger/presentation cleanup so replay output labels do not overstate analyzer
  repeated-failure activity after `R1E`

The family goal remains the same: make `dead_end_thrash` honest on completed successful tails
without widening into broader analyzer or sentinel redesign. Live repo truth now shows that the
family has two distinct analyzer seams:

- the landed outcome-evidence cutover that stopped generic `ToolOutput` from counting as failure
- the still-open repeated verification-loop semantics that can keep completed success tails falsely
  active

This family should:

- add one analyzer-owned outcome-evidence classification seam over `CompactionRow`
- narrow repeated-failure loop construction to explicit failure evidence
- use bounded replay to verify the family status honestly, even when that proof exposes a remaining
  analyzer seam
- fix repeated verification-loop recovery semantics without first changing compactor schema
- keep `dead_end_thrash` active on real repeated failures and out-of-scope verification while
  allowing honest downgrade on completed in-scope success tails
- add focused analyzer regressions before the final real-session-style replay re-proof
- exclude subagent/delegated sessions from that proof corpus unless a minimal screening classifier
  is first added to identify them reliably

This family should not:

- change compactor bundle schema or upstream normalization
- change sentinel scheduler, runtime, or presentation logic before `R1F`, or beyond the narrow
  trigger/presentation cleanup owned by `R1F`
- add turn-context, `session_archetype`, or `session_progress`
- redesign drift scoring beyond the minimal `dead_end_thrash` input cutover
- broaden into a general learned or hybrid monitor

## Why This Family Comes Next

The live stack already has analyzer-owned `DriftState`, sentinel `v0.3` checkpoint consumption,
and the landed `R1A`/`R1B` outcome-evidence cutover. But bounded replay proved semantic honesty is
still wrong because repeated verification loops remain a direct `dead_end_thrash` input:

- `collect_command_observations(...)` still marks `cargo`, `pnpm`, and `npm` families as
  `verification_like`
- `repetition_slice(...)` still builds repeated verification history from those commands
- `recovery_state(...)` still blocks recovery when the current interval touches repeated
  verification loops
- `score_dead_end_thrash(...)` still lets repeated verification loops force active thrash by
  themselves

That means the highest-leverage next work is no longer a new `ToolOutput` cutover. It is the
verification-loop semantics fix exposed by the bounded replay diagnosis. The new `R1D`/`R1E` split
exists to keep that fix and its replay re-proof reviewable and checkpoint-green.

## Packet Strategy

### Packet R1A: Outcome-Evidence Seam And Repeated-Failure Cutover

Deliver first:

- lock the `R1A`/`R1B`/`R1C`/`R1D`/`R1E` family split in repo docs
- define one analyzer-local outcome-evidence type or helper
- classify `CompactionKind::Error` as explicit failure
- classify generic `CompactionKind::ToolOutput` as neutral by default
- allow only a very small deterministic subset of `ToolOutput` text to count as failure if the
  signal is unambiguous
- replace `is_failure_row(...)`-driven repeated-failure grouping with classifier-driven grouping
- preserve row refs and evidence reasons so current scorer/export paths remain stable where honest

Why first:

- every downstream semantic fix depends on replacing raw row-kind checks with one typed seam

### Packet R1B: Recovery, Export, And Honest Downgrade Semantics

Deliver second:

- confirm `recovery_state(...)` now sees a clean interval when only successful output remains
- keep `dead_end_thrash` scoring logic as narrow as possible, changing only what is necessary to
  consume the cleaner repeated-failure surface
- preserve historical-only and recovered semantics after honest recovery
- verify analyzer checkpoints and exports remain deterministic after the narrower failure surface
- add focused regressions for `Exit code: 0`, bookkeeping output, success tails, and recovery

Why second:

- the family should fix the false positive by improving inputs and then proving the recovery
  semantics, not by jumping straight to replay proof while core analyzer behavior is still moving

### Packet R1C: Bounded Replay Diagnosis And Honest Status Correction

Deliver third:

- screen candidate proof sessions to exclude delegated/subagent runs
- run bounded replay across a small handful of completed non-subagent sessions that match the
  reproduced false-positive tail shape
- compare the clean controls and representative sticky replay against their rollout tails
- if the representative sticky replay still ends active, capture that state honestly and route the
  family to the precise remaining analyzer seam
- add only the smallest acceptance-only helper such as `subagent_observed` if rollout-text
  screening proves unreliable
- refresh continuity notes only enough to correct the packet route and current family status

Why third:

- the first replay proof is where the family proves whether the analyzer semantics are actually
  sufficient; if they are not, the honest move is to stop and split the remaining seam instead of
  overclaiming completion

### Packet R1D: Repeated Verification-Loop Semantics And Honest Recovery Fix

Deliver fourth:

- adjust `collect_command_observations(...)`, `recovery_state(...)`, and
  `score_dead_end_thrash(...)` so repeated verification stays historical unless the current
  interval is still out of scope or still carries repeated explicit failure evidence
- keep repeated verification evidence visible without letting it force active thrash by itself on
  completed in-scope success tails
- narrow `verification_like` tagging only if the smallest deterministic analyzer-local rule is
  needed to clear the reproduced false positive
- add focused regressions for repeated successful verification loops, out-of-scope verification,
  and recovered/historical transitions

Why fourth:

- `R1C` already proved the remaining seam is analyzer-owned, so the next honest step is a narrow
  analyzer fix, not more replay paperwork

### Packet R1E: Bounded Replay Re-Proof And Continuity Refresh

Deliver fifth:

- rerun the final screened non-subagent replay corpus after `R1D`
- prove the representative sticky session now clears or downgrades honestly instead of ending
  active from repeated verification loops
- confirm clean controls remain cleared
- refresh continuity notes if the final proof corpus or verifier surface changed materially

Why fifth:

- the family is only complete once the real-session-style bounded replay proof passes on top of
  the landed verification-loop fix

### Packet R1F: Sentinel Trigger And Presentation Honesty Cleanup

Deliver sixth:

- update the sentinel replay/live presentation seam so scheduler trigger labels cannot be read as
  analyzer drift-class outcomes
- preserve analyzer checkpoint state and checkpoint posture as the operator-facing truth source
- add focused replay/live parity tests for recovered and historical-only checkpoints reached via a
  scheduler fast path

Why sixth:

- `R1E` can prove the analyzer fix honestly today, but it still leaves one downstream labeling seam
  that is easy to misread in replay output; that cleanup belongs after the proof packet, not
  inside it

## Sequencing

Sequential work:

1. land `R1A` docs plus classifier contract and repeated-failure cutover
2. verify `R1A` with focused analyzer tests and stop there
3. land `R1B` recovery/export semantics for the narrower repeated-failure surface
4. verify `R1B` with focused analyzer tests and stop there
5. screen a small proof corpus for non-subagent sessions using current rollout markers
6. land `R1C` bounded replay diagnosis and honest status correction
7. land `R1D` repeated verification-loop semantics and focused regressions
8. verify `R1D` with focused analyzer tests and stop there
9. land `R1E` bounded replay re-proof and any continuity-note refresh
10. run the analyzer wall and then the final bounded replay proof set
11. land `R1F` sentinel trigger/presentation cleanup and focused sentinel parity tests

Parallel-safe work after `R1A` lands:

- focused `R1B` and later `R1D` regression drafting in `dead_end_thrash.rs` and
  `export_bundle.rs`
- screening candidate real sessions for subagent markers for later `R1C`/`R1E`
- drafting continuity-note updates while replay diagnosis or re-proof is executing
- drafting the narrow `R1F` operator-surface wording while `R1E` proof artifacts are being
  finalized

Not parallel-safe:

- classifier design and repeated-failure cutover
- recovery semantic changes before `R1A` lands
- replay-proof claims before `R1D` verification-loop behavior is locked
- sentinel trigger/presentation cleanup before `R1E` locks the bounded replay proof story

## Major Risks And Mitigations

### Risk 1: Broad String Matching Recreates False Positives In A Different Form

Mitigation:

- keep `ToolOutput` neutral by default
- whitelist only a tiny set of unambiguous failure signals if needed
- add explicit regression coverage for `Exit code: 0`, plan updates, and generic function-call
  output staying non-failure

### Risk 2: The Fix Moves Too Much Logic Into The Scorer Instead Of The Analyzer Seam

Mitigation:

- make classification and repeated-failure grouping the primary `R1A` change
- keep `R1B` scorer changes minimal and input-focused
- reject designs that special-case completion tails inside `dead_end_thrash` without fixing the
  evidence surface first

### Risk 3: Real Failure Signals Embedded In Tool Output Become Invisible

Mitigation:

- start with `Error` rows as the firm failure base
- if a `ToolOutput` subset must count as failure, constrain it to deterministic, test-backed cases
- keep open the later option for richer compactor structure if current text-only evidence proves
  insufficient

### Risk 4: The Verification-Loop Fix Hides Real Thrash By Overcorrecting

Mitigation:

- keep repeated explicit failures and out-of-scope verification as active thrash inputs
- preserve repeated verification evidence as historical context even when it stops forcing `Active`
- add regressions that prove out-of-scope verification and explicit repeated failure still flag

### Risk 5: The Replay Proof Still Dominates Session Size

Mitigation:

- keep `R1C` and `R1E` separate from the analyzer-semantic packets
- treat replay re-proof as the first ejectable step if `R1D` uncovers deeper analyzer work
- avoid mixing proof-corpus plumbing into `R1A`, `R1B`, or `R1D` unless it is the smallest route
  to a passing focused regression

### Risk 6: Proof Sessions Are Contaminated By Subagent Delegation

Mitigation:

- screen candidate sessions for `multi_agent_v1` activity such as `spawn_agent`, `wait_agent`, and
  `close_agent` before treating them as `R1C` or `R1E` proof
- treat delegated sessions as out of corpus for `R1C` and `R1E`
- if text screening proves unreliable, add only a tiny acceptance-selection classifier such as
  `subagent_observed`; do not broaden into a general supported subagent seam

### Risk 7: Scope Drifts Into R2-R7 Work

Mitigation:

- treat `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` as the packet authority
- refuse to add turn context, archetype, progress, or sentinel cleanup in this family
- if implementation pressure reveals one of those needs, stop and reclassify it under its owning
  `R*` packet rather than silently broadening the `R1` family

### Risk 8: Replay Output Still Sounds Wrong Even After The Analyzer Fix Is Correct

Mitigation:

- keep `R1E` proof claims rooted in analyzer checkpoint state and posture, not scheduler trigger
  names
- isolate the downstream replay/live labeling cleanup into `R1F`
- add focused sentinel tests proving scheduler triggers and analyzer-active classes are rendered as
  distinct concepts after `R1F`

## Verification Wall

### Packet R1A

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Packet R1B

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Packet R1C

```bash
export SESSION_ID="<completed-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export OUTPUT_DIR="target/hybrid-drift-evals/$SESSION_ID"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$OUTPUT_DIR/compactor"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$OUTPUT_DIR/compactor" \
  --output-dir "$OUTPUT_DIR/analyzer"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$OUTPUT_DIR/analyzer"
```

Run that flow for a small handful of completed proof sessions. Before including any session in the
proof set, inspect its rollout JSONL for delegation/subagent markers such as `spawn_agent`,
`wait_agent`, and `close_agent` under `multi_agent_v1`. Sessions with those markers are out of
corpus for `R1C`.

The replay proof is successful only if the final analyzer/sentinel state is compared against each
known completed non-subagent rollout tail and no longer ends with active `dead_end_thrash` for a
successful-output-only tail.

### Packet R1D

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Packet R1E

```bash
export SESSION_ID="<completed-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export OUTPUT_DIR="target/hybrid-drift-evals/$SESSION_ID"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$OUTPUT_DIR/compactor"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$OUTPUT_DIR/compactor" \
  --output-dir "$OUTPUT_DIR/analyzer"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$OUTPUT_DIR/analyzer"
```

## Verification Checkpoints

### Checkpoint VR1-A: `R1A` Classifier Contract Is Locked

Must be true:

- one analyzer-local outcome-evidence seam exists
- `Error` rows count as failure
- ambiguous `ToolOutput` rows count as neutral
- repeated-failure loops no longer depend on generic `ToolOutput == failure`

Verify:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Checkpoint VR1-B: `R1B` Recovery And Export Semantics Stay Honest

Must be true:

- repeated-failure loops no longer include successful tool output
- clean verification intervals are not blocked by neutral bookkeeping output
- historical thrash evidence still survives honest recovery
- export/checkpoint output stays deterministic after the evidence cutover

Verify:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Checkpoint VR1-C: `R1C` Bounded Replay Status Is Honest

Must be true:

- a small handful of bounded real-session-style tails were replayed and compared against their
  rollout tails
- if the representative sticky replay still ends active, the remaining analyzer-owned seam is
  named precisely and routed to the next packet instead of being hand-waved away
- each proof session has been screened to exclude subagent/delegation usage, or a tiny explicit
  screening classifier exists to mark that usage for proof selection
- the proof remains analyzer-rooted rather than relying on a sentinel-only presentation tweak
- the family boundary still excludes `R2`-`R7` work

Verify:

- run the bounded replay proof commands above for each screened session
- inspect the final checkpoint output against the source rollout tail

### Checkpoint VR1-D: `R1D` Verification-Loop Semantics Are Honest

Must be true:

- repeated successful verification loops do not by themselves keep completed in-scope success
  tails active
- out-of-scope verification still blocks recovery
- repeated explicit failures still keep `dead_end_thrash` active
- recovered and historical-only transitions remain deterministic after the fix

Verify:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Checkpoint VR1-E: `R1E` Final Bounded Honesty Proof Passes

Must be true:

- the representative sticky non-subagent replay no longer ends falsely active from repeated
  verification loops
- clean controls remain cleared
- each proof session has been screened to exclude subagent/delegation usage, or a tiny explicit
  screening classifier exists to mark that usage for proof selection
- continuity notes describe the final packet route and bounded-proof result honestly

Verify:

- run the bounded replay proof commands above for each final screened session
- inspect the final checkpoint output against the source rollout tail

## Review Notes

The main review question for this plan is whether `R1D` stays comfortably bounded once `R1C`
pinpoints the remaining seam. If the verification-loop fix starts dragging in richer compactor
structure or broader analyzer redesign work, the honest move is to stop and re-scope instead of
silently re-merging `R1D` and `R1E` into one oversized session. A second review question now
exists after `R1E`: keep the sentinel replay/live trigger/presentation cleanup in `R1F` narrow
enough that it does not reopen analyzer semantics or broaden into a larger sentinel redesign.
