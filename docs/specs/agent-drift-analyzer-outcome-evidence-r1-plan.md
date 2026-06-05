# Plan: Agent Drift Analyzer Outcome Evidence R1 Packet Family

## Scope

This plan implements
`docs/specs/agent-drift-analyzer-outcome-evidence-r1-spec.md`.

The old single-packet `R1` shape is intentionally split into three packets:

- `R1A`: docs lock, analyzer outcome-evidence seam, and repeated-failure cutover
- `R1B`: recovery/export semantics plus focused regressions for honest downgrade behavior
- `R1C`: bounded replay proof over screened non-subagent sessions plus continuity-note refresh

The family goal remains the same: correct the analyzer evidence surface that still treats generic
`ToolOutput` as failure evidence and thereby keeps `dead_end_thrash` sticky through successful
session tails. The split exists to keep each landing unit reviewable and checkpoint-green.

This family should:

- add one analyzer-owned outcome-evidence classification seam over `CompactionRow`
- narrow repeated-failure loop construction to explicit failure evidence
- keep repeated verification loops unchanged
- keep `dead_end_thrash` active on real repeated failures while allowing honest downgrade on
  successful-output-only tails
- add bounded analyzer regressions first, then a small handful of real-session-style acceptance
  runs after the analyzer semantics are already landed
- exclude subagent/delegated sessions from that proof corpus unless a minimal screening classifier
  is first added to identify them reliably

This family should not:

- change compactor bundle schema or upstream normalization
- change sentinel scheduler, runtime, or presentation logic
- add turn-context, `session_archetype`, or `session_progress`
- redesign drift scoring beyond the minimal `dead_end_thrash` input cutover
- broaden into a general learned or hybrid monitor

## Why This Family Comes Next

The live stack already has analyzer-owned `DriftState` and sentinel `v0.3` checkpoint consumption,
but semantic honesty is still wrong because the analyzer builds repeated-failure history from a
coarse row-kind rule:

- `repetition_slice(...)` still filters archival rows through `is_failure_row(...)`
- `is_failure_row(...)` still treats `CompactionKind::ToolOutput` as failure
- `dead_end_thrash` consumes those repeated-failure loops directly

That means the highest-leverage next work is still the evidence-surface correction. The only
change here is packet sizing: the real-session proof is valuable, but it should not be bundled
into the first code-change landing if that makes the packet too large to execute cleanly.

## Packet Strategy

### Packet R1A: Outcome-Evidence Seam And Repeated-Failure Cutover

Deliver first:

- lock the `R1A`/`R1B`/`R1C` family split in repo docs
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

### Packet R1C: Bounded Replay Proof And Continuity Refresh

Deliver third:

- screen candidate proof sessions to exclude delegated/subagent runs
- add bounded replay acceptance across a small handful of completed non-subagent sessions that
  match the reproduced false-positive tail shape
- add only the smallest acceptance-only helper such as `subagent_observed` if rollout-text
  screening proves unreliable
- refresh continuity notes only if the final proof surface or verifier list changes materially

Why third:

- the family is only complete once it proves semantic honesty on real-session-style tails, but that
  proof is a better fit after the code semantics are already stable

## Sequencing

Sequential work:

1. land `R1A` docs plus classifier contract and repeated-failure cutover
2. verify `R1A` with focused analyzer tests and stop there
3. land `R1B` recovery/export semantics and focused regressions
4. verify `R1B` with focused analyzer tests and stop there
5. screen a small proof corpus for non-subagent sessions using current rollout markers
6. land `R1C` bounded replay proof and any continuity-note refresh
7. run the analyzer wall and then the bounded replay proof set

Parallel-safe work after `R1A` lands:

- focused `R1B` regression drafting in `dead_end_thrash.rs` and `export_bundle.rs`
- screening candidate real sessions for subagent markers for later `R1C`
- drafting continuity-note updates while replay acceptance is executing

Not parallel-safe:

- classifier design and repeated-failure cutover
- recovery semantic changes before `R1A` lands
- replay-proof claims before `R1B` semantic behavior is locked

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

### Risk 4: The Replay Proof Still Dominates Session Size

Mitigation:

- keep `R1C` separate from the analyzer-semantic packets
- treat replay proof as the first ejectable step if `R1B` uncovers deeper analyzer work
- avoid mixing proof-corpus plumbing into `R1A` or `R1B` unless it is the smallest route to a
  passing focused regression

### Risk 5: Proof Sessions Are Contaminated By Subagent Delegation

Mitigation:

- screen candidate sessions for `multi_agent_v1` activity such as `spawn_agent`, `wait_agent`, and
  `close_agent` before treating them as `R1C` proof
- treat delegated sessions as out of corpus for `R1C`
- if text screening proves unreliable, add only a tiny acceptance-selection classifier such as
  `subagent_observed`; do not broaden into a general supported subagent seam

### Risk 6: Scope Drifts Into R2-R7 Work

Mitigation:

- treat `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` as the packet authority
- refuse to add turn context, archetype, progress, or sentinel cleanup in this family
- if implementation pressure reveals one of those needs, stop and reclassify it under its owning
  `R*` packet rather than silently broadening the `R1` family

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

### Checkpoint VR1-C: `R1C` Bounded Honesty Proof Passes

Must be true:

- a small handful of bounded real-session-style tails no longer end in active `dead_end_thrash`
- each proof session has been screened to exclude subagent/delegation usage, or a tiny explicit
  screening classifier exists to mark that usage for proof selection
- the proof remains analyzer-rooted rather than relying on a sentinel-only presentation tweak
- the family boundary still excludes `R2`-`R7` work

Verify:

- run the bounded replay proof commands above for each screened session
- inspect the final checkpoint output against the source rollout tail

## Review Notes

The main review question for this plan is whether `R1B` stays comfortably bounded once `R1A` lands.
If recovery/export verification starts dragging in proof-corpus or broader analyzer redesign work,
the honest move is to stop and re-scope instead of silently re-merging `R1B` and `R1C` into one
oversized session.
