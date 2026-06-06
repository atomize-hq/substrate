# Spec: Agent Drift Analyzer Outcome Evidence R1 Packet Family

## Assumptions I'm Making

1. Live repo truth is the authority: analyzer checkpoint state `v0.6A` and sentinel cutover
   `v0.6B` are already landed, so this is the next follow-on family rather than a reopening of
   checkpoint-state ownership.
2. The work still belongs to one semantic family, but the original three-packet split proved too
   narrow once bounded replay exposed a second analyzer seam in repeated verification-loop
   semantics.
3. The compactor bundle contract remains `v0.2` and does not grow new fields first. The analyzer
   must classify outcome evidence from the current `CompactionRow` surface (`kind`, `text`,
   stable row refs, timestamps) rather than requiring an upstream schema change.
4. `CompactionKind::Error` remains explicit failure evidence, while generic
   `CompactionKind::ToolOutput` becomes neutral by default unless the row carries a trusted,
   deterministic failure signal from the existing normalized text.
5. Ambiguous `ToolOutput` rows must bias neutral, not failure. False-positive thrash is a more
   damaging error here than temporarily undercounting a failure loop.
6. External research is advisory, not authoritative, but it points in the same direction:
   trajectory diagnostics are more honest when they classify a typed intermediate evidence surface
   first and keep failure attribution separate from later progress/archetype reasoning.
7. Real-session proof should still use a small handful of completed sessions rather than a single
   spot-check session, but that proof should be its own packet rather than bundled into the first
   code-change landing.
8. Those proof sessions must exclude subagent/delegation usage because worker-agent orchestration
   can materially change trajectory shape and would confound the current heuristics.
9. Current rollout JSONL appears to expose subagent usage directly through `response_item` entries
   such as `spawn_agent`, `wait_agent`, and `close_agent` under `multi_agent_v1`, so proof-session
   screening should be possible without adding a fully supported analyzer seam first.

## Objective

Implementation target on `2026-06-06`:

- split the typed outcome-evidence follow-on into five focused packets
- land the analyzer-owned outcome-evidence seam before the replay-proof work
- preserve honest `dead_end_thrash` recovery semantics after the repeated-failure cutover
- use bounded replay to prove the live family status honestly, even if that proof exposes another
  analyzer-owned seam
- land the repeated verification-loop fix before the final bounded replay re-proof

Primary users:

- the engineer reading analyzer checkpoints or sentinel output and expecting successful tool output
  plus `task_complete` to clear or downgrade stale thrash honestly
- the maintainer changing analyzer heuristics and needing one narrow module to own row-to-outcome
  classification before drift scoring consumes the result

Success means:

- `R1A` lands the analyzer-owned outcome-evidence seam plus repeated-failure cutover
- `R1B` lands the honest recovery/export semantics and focused regressions needed to trust the
  narrower failure surface
- `R1C` lands the bounded non-subagent replay diagnosis and honest status correction when the
  original replay proof still fails
- `R1D` lands the analyzer-owned repeated verification-loop and recovery fix exposed by `R1C`
- `R1E` lands the final bounded non-subagent replay re-proof and continuity-note refresh without
  widening into broader analyzer or sentinel work

## Packet Family Boundary

This spec defines the `R1` packet family, split into `R1A`, `R1B`, `R1C`, `R1D`, and `R1E`.

### Packet R1A: Outcome-Evidence Seam And Repeated-Failure Cutover

In scope:

- lock the packet-family boundary in repo docs
- add one analyzer-owned typed outcome-evidence seam over current `CompactionRow` input
- narrow repeated-failure loop construction to explicit failure evidence
- leave repeated verification loop collection unchanged
- add focused regression coverage for classification and repeated-failure grouping

Out of scope:

- recovery/export semantic revalidation that requires broader fixture work
- bounded replay proof over real sessions
- continuity-note refresh beyond packet-family routing language needed to reflect the split

### Packet R1B: Recovery, Export, And Honest Downgrade Semantics

In scope:

- update `dead_end_thrash` and recovery semantics to consume the narrowed repeated-failure surface
- prove successful-output-only tails clear or downgrade honestly
- preserve historical-only and recovered transitions
- verify analyzer export/checkpoint behavior stays deterministic after the evidence cutover
- add focused regressions for success tails and bookkeeping output

Out of scope:

- real-session corpus screening
- compactor -> analyzer -> sentinel replay proof
- continuity-note refresh driven only by the real-session proof set

### Packet R1C: Bounded Replay Diagnosis And Honest Status Correction

In scope:

- screen a small handful of completed proof sessions to exclude delegated/subagent runs
- add or use only the smallest acceptance-only screening helper if rollout-text screening proves
  unreliable
- run the bounded compactor -> analyzer -> sentinel replay flow on the screened session set
- compare the control corpus and representative sticky replay against the real rollout tails
- record the honest family state if the representative sticky replay still fails after `R1A`/`R1B`
  and name the remaining analyzer-owned seam precisely enough to scope the follow-on packet

Out of scope:

- landing the repeated verification-loop semantics fix itself
- changing sentinel scheduler policy, operator presentation, or live runtime behavior
- adding a general-purpose subagent semantics or scoring seam beyond the minimal proof-session
  screening needed to exclude delegated runs

### Packet R1D: Repeated Verification-Loop Semantics And Honest Recovery Fix

In scope:

- update analyzer-owned verification-loop semantics after `R1C` proved the remaining sticky
  `dead_end_thrash` false positive
- keep repeated verification evidence visible as historical context while preventing it from
  forcing active thrash by itself on completed in-scope success tails
- tighten `recovery_state(...)`, `repetition_slice(...)`, and `score_dead_end_thrash(...)` so
  repeated successful verification only stays active when the current interval is still out of
  scope or still carries repeated explicit failure evidence
- narrow `verification_like` classification if needed, but only with deterministic analyzer-local
  rules over the existing `CompactionRow` surface
- add focused regressions for repeated successful verification loops, out-of-scope verification,
  and honest recovered/historical transitions

Out of scope:

- changing compactor bundle schema or adding richer upstream typed command/outcome structure first
- re-proving the real-session corpus
- changing sentinel runtime or operator-surface logic

### Packet R1E: Bounded Replay Re-Proof And Continuity Refresh

In scope:

- rerun the bounded compactor -> analyzer -> sentinel replay flow on the screened non-subagent
  proof set after `R1D`
- prove the representative sticky session now clears or downgrades honestly instead of ending
  falsely active from repeated verification loops
- refresh continuity notes if the final proof corpus, verifier shape, or packet-routing language
  changed materially during `R1D`
- keep proof claims labeled as bounded replay proof rather than live proof

Out of scope:

- new analyzer semantics beyond what `R1D` already landed
- changing sentinel scheduler policy, operator presentation, or live runtime behavior
- widening into richer turn-context, archetype, or progress packets

## Family-Wide Out Of Scope

- changing compactor bundle schema or normalization contracts
- adding per-checkpoint turn-context fields
- adding `session_archetype` or `session_progress`
- retuning other drift classes unless a shared helper must move mechanically with this family
- broadening into replay/live operator docs beyond what is needed to describe the bounded replay
  diagnosis and re-proof
- changing sentinel scheduler, runtime, or presentation logic

## Tech Stack

- Language: Rust 2021
- Target crate:
  - `crates/agent-drift-analyzer`
- Existing upstream contract:
  - compactor bundle `v0.2`
- Existing downstream checkpoint contract:
  - analyzer checkpoint schema `v0.3`
- Existing scorer surface:
  - `CheckpointAnalysis.repetition.repeated_failure_loops`
  - `CheckpointAnalysis.repetition.repeated_verification_loops`

Dependency posture:

- no compactor schema change is required first
- no sentinel code change is required first
- no new crate or external dependency is required

## Commands

### Packet R1A validation

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Packet R1B validation

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Packet R1C bounded replay diagnosis

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

Run that flow for a small handful of completed sessions, not just one. Each proof session must be
screened first to confirm the rollout does not contain subagent/delegation markers such as
`spawn_agent`, `wait_agent`, or `close_agent` under `multi_agent_v1`.

That proof is bounded replay evidence, not live proof. It is honest only if the source rollouts are
known completed non-subagent sessions whose tails can be compared against the cited analyzer
evidence.

### Packet R1D validation

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Packet R1E bounded replay re-proof

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

Run that flow for the final screened non-subagent proof set after `R1D` lands. The re-proof is
successful only if the representative sticky session and the clean control corpus both avoid a
false final `dead_end_thrash=active` result.

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Owns checkpoint analysis, repetition slicing, and recovery-state inputs. `R1A` should add the
  typed outcome-evidence seam here or in a closely related analyzer-local helper module. `R1D`
  should land the repeated verification-loop and recovery adjustments here as well.

crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs
  Owns thrash scoring. `R1B` should confirm it consumes explicit failure evidence rather than raw
  row-kind groupings, and `R1D` should keep repeated verification history from forcing active
  thrash on completed in-scope success tails.

crates/agent-drift-analyzer/src/context/working_set.rs
  Owns `collect_command_observations(...)` and `verification_like` tagging. `R1D` may narrow that
  tagging if family-only command classification proves too coarse for honest verification-loop
  recovery.

crates/agent-drift-analyzer/tests/dead_end_thrash.rs
  Primary regression coverage for repeated failure, successful-output recovery, and historical-only
  transitions across `R1A`, `R1B`, and `R1D`.

crates/agent-drift-analyzer/tests/export_bundle.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
  Coverage for checkpoint export and end-to-end analyzer behavior that should stay honest after the
  new evidence seam lands.

crates/agent-drift-analyzer/tests/support/mod.rs
  Shared fixture helpers for building bounded analyzer bundles and real-session-style test inputs.

/Users/spensermcconnell/.codex/sessions/
  Source rollout corpus for `R1C` replay diagnosis and `R1E` bounded replay re-proof, plus
  screening whether a candidate proof session used subagents.

target/hybrid-drift-evals/
  Local bounded-replay evidence artifacts used during `R1C` diagnosis and `R1E` re-proof. These
  are proof artifacts, not committed code by themselves.

HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md
  Repo-root planning authority that declares the `R1` packet family the next landing step and
  keeps later turn/archetype/progress packets separate.

docs/specs/hybrid-drift-sentinel-implementation-order.md
  Packet continuity authority for the older analyzer/sentinel sequence. Continuity notes may need
  to point at this `R1` family once implementation planning is finalized.
```

## Code Style

Make outcome evidence explicit in analyzer types instead of encoding failure semantics through
`CompactionKind::ToolOutput` alone.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutcomeEvidenceKind {
    Failure,
    Success,
    Neutral,
    Bookkeeping,
    Milestone,
}

fn classify_outcome_evidence(row: &CompactionRow) -> OutcomeEvidenceKind {
    match row.kind {
        CompactionKind::Error => OutcomeEvidenceKind::Failure,
        CompactionKind::ToolOutput => classify_tool_output_text(&row.text),
        _ => OutcomeEvidenceKind::Neutral,
    }
}
```

Conventions:

- keep the new evidence seam analyzer-owned and deterministic
- prefer one explicit classifier helper over duplicating row-kind checks across scorers
- use conservative defaults: unknown or mixed `ToolOutput` content stays neutral
- treat progress-like signals and failure-like signals as separate modules even if they are derived
  from the same rows later
- avoid stringly control flow in scorers once a typed evidence helper exists

## Testing Strategy

### Packet R1A

1. Outcome-classification unit coverage
   - verify `CompactionKind::Error` always becomes failure evidence
   - verify generic `ToolOutput` defaults neutral when it lacks a trusted failure signal
   - verify success markers such as `Exit code: 0` remain non-failure
2. Repetition-slice coverage
   - verify repeated-failure loops only include rows classified as explicit failure evidence
   - verify repeated successful tool output no longer forms a repeated-failure loop

### Packet R1B

3. `dead_end_thrash` regression coverage
   - verify real repeated failures still keep `dead_end_thrash` active
   - verify successful-output-only tails clear or downgrade `dead_end_thrash`
   - verify historical evidence remains available after honest recovery
4. Analyzer export/integration coverage
   - verify analyzer checkpoints and summary output remain deterministic after the evidence cutover
   - verify checkpoint export still reflects honest `dead_end_thrash` state on focused fixtures

### Packet R1C

5. Bounded real-session-style acceptance
  - screen a small handful of completed proof sessions and exclude any session with subagent /
     delegation markers already visible in the rollout
  - if the current rollout text proves too weak to screen subagent usage reliably, add only the
     smallest classifier needed to label `subagent_observed` for proof selection; do not broaden
     into a general supported subagent seam
  - run the current compactor -> analyzer -> sentinel replay flow on that non-subagent session set
  - if the representative sticky session still ends active, capture that result honestly and name
    the remaining analyzer-owned seam precisely enough to route the follow-on packet

### Packet R1D

6. Repeated verification-loop regression coverage
   - verify repeated successful verification commands such as `cargo fmt`, `cargo clippy`, and
     `cargo test` do not by themselves keep completed in-scope success tails active
   - verify repeated verification still stays active when the current interval is out of scope
   - verify repeated explicit failure evidence still keeps `dead_end_thrash` active
   - verify historical and recovered transitions remain deterministic after the verifier-loop fix

### Packet R1E

7. Bounded replay re-proof
   - rerun the screened non-subagent proof corpus after `R1D`
   - verify the representative sticky replay no longer ends falsely active from repeated
     verification loops
   - verify clean control replays stay cleared
   - refresh continuity notes if the final verifier list or packet routing changed materially

## Boundaries

- Always:
  - treat `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` as the authority that the `R1`
    packet family is next and `R1A` is the first landing
  - keep failure attribution separate from turn-context, archetype, and progress semantics
  - classify ambiguous `ToolOutput` rows as neutral
  - update analyzer regressions in the same packet as the code change they defend
- screen real-session proof candidates for subagent usage before treating them as `R1C` or `R1E`
  evidence
- Ask first:
  - changing compactor schema or upstream normalization to expose richer tool-output structure
  - widening analyzer checkpoint schema beyond the already-landed `v0.3`
  - changing sentinel code if analyzer-only changes prove insufficient
  - changing non-`dead_end_thrash` drift semantics beyond mechanical helper reuse
- Never:
  - reopen checkpoint-state ownership as if `v0.6` were still the missing packet
  - implement session archetype, turn context, or progress modeling in this family
  - treat generic `ToolOutput == failure` as an acceptable fallback
  - claim bounded replay acceptance is equivalent to true live proof

## Success Criteria

1. `R1A` lands an analyzer-local outcome-evidence seam and repeated-failure cutover that no longer
   treats generic `CompactionKind::ToolOutput` rows as failure by default.
2. `R1B` proves the narrower repeated-failure surface clears or downgrades honestly when only
   neutral/successful tool output remains, without yet retuning repeated verification loops.
3. `R1C` proves the bounded replay status honestly by showing whether the representative sticky
   non-subagent session still fails after `R1A`/`R1B`, and names the remaining analyzer-owned seam
   precisely if it does.
4. `R1D` lands the repeated verification-loop semantics fix so completed in-scope success tails no
   longer stay active merely because verification commands repeated while converging.
5. `R1E` proves on a small handful of screened non-subagent sessions that completed rollout tails
   no longer end falsely active from repeated verification loops.
6. The family does not broaden into turn-context, session-archetype, or progress semantics; those
   remain separate planned work after `R1E`.

## Open Questions

1. Trusted failure signal boundary: what is the smallest honest analyzer-local rule for promoting a
   `ToolOutput` row from neutral to failure without new upstream structure?
   - Assumed default for this family: start conservative, using explicit `Error` rows as failure
     and only a very small deterministic subset of `ToolOutput` text when the failure signal is
     unambiguous.
2. Evidence taxonomy shape: should bookkeeping and milestone output be distinct enum variants, or
   can they stay grouped as non-failure `Neutral` in `R1A`/`R1B` as long as future packets can
   refine the classifier without breaking callers?
   - Assumed default for this family: allow an internal richer enum if it simplifies later packets,
     but only `failure` versus `non-failure` needs to affect `dead_end_thrash` in this family.
3. Acceptance fixture source: should the bounded real-session-style regression use a compact
   purpose-built analyzer fixture, or a checked-in artifact derived from one reproduced session?
   - Assumed default for this family: prefer the smallest fixture that still preserves the false-
     positive tail shape and row kinds seen in the reproduced sessions.
4. Subagent screening seam: is rollout-text screening for `multi_agent_v1` activity sufficient for
   excluding delegated sessions from the proof set, or does `R1C`/`R1E` need a tiny explicit
   `subagent_observed` classifier purely for acceptance selection?
   - Assumed default for this family: use rollout-text screening first and add only the smallest
     acceptance-only classifier if that screening proves unreliable during implementation.
5. Verification command taxonomy: should `verification_like` remain a family-level classifier
   (`cargo`, `pnpm`, `npm`) during `R1D`, or does honest replay behavior require a slightly
   narrower analyzer-local rule over known verification subcommands?
   - Assumed default for this family: keep the smallest deterministic analyzer-local rule that
     clears the reproduced false positive without adding new compactor structure first.
