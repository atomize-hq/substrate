# Tasks: Agent Drift Analyzer Outcome Evidence R1 Packet Family

This task list implements:

- `docs/specs/agent-drift-analyzer-outcome-evidence-r1-spec.md`
- `docs/specs/agent-drift-analyzer-outcome-evidence-r1-plan.md`

## Task List

## Packet R1A: Lock The Family Boundary And Land The Analyzer Outcome-Evidence Seam

- [x] Task: Lock the `R1A` / `R1B` / `R1C` / `R1D` / `R1E` packet-family boundary in repo docs
  - Acceptance: the docs explicitly define:
    - `R1A` as docs lock plus analyzer-only outcome-evidence classification and repeated-failure
      cutover
    - `R1B` as recovery/export semantic proof on top of the landed narrower repeated-failure
      surface
    - `R1C` as bounded non-subagent replay diagnosis plus honest status correction when the first
      proof still fails
    - `R1D` as the analyzer-owned repeated verification-loop semantics fix
    - `R1E` as the final bounded replay re-proof plus continuity-note refresh
    - repeated successful `ToolOutput` as non-failure by default
    - exclusion of delegated / subagent sessions from the `R1C` and `R1E` proof corpus
    - only a tiny acceptance-selection classifier such as `subagent_observed` if rollout-text
      screening proves unreliable
    - turn context, archetype, progress, broader scorer redesign, and sentinel cleanup as later
      `R*` packets rather than hidden `R1` scope
  - Verify: doc review against the current `R1` spec, `R1` plan, and
    `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-outcome-evidence-r1-spec.md`
    - `docs/specs/agent-drift-analyzer-outcome-evidence-r1-plan.md`
    - `docs/specs/agent-drift-analyzer-outcome-evidence-r1-tasks.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`

- [x] Task: Add one analyzer-owned outcome-evidence seam and route repeated-failure grouping through it
  - Acceptance: analyzer code has one explicit classifier seam that:
    - treats `CompactionKind::Error` as explicit failure evidence
    - treats generic `CompactionKind::ToolOutput` as neutral by default
    - promotes `ToolOutput` to failure only for a very small deterministic subset when the failure
      signal is unambiguous
    - replaces raw `is_failure_row(...)` logic in repeated-failure grouping
    - leaves repeated verification loop collection unchanged
    - does not require any compactor schema or normalization change first
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`

Packet `R1A` exit condition:

- repo docs define the `R1A` / `R1B` / `R1C` / `R1D` / `R1E` split clearly
- analyzer owns one explicit outcome-evidence seam instead of raw `ToolOutput == failure`
- repeated-failure history excludes successful tool output by default
- repeated verification loop collection is unchanged

## Packet R1B: Preserve Honest Recovery And Export Semantics For The Narrower Failure Surface

- [x] Task: Preserve honest recovery and `dead_end_thrash` state semantics after the evidence cutover
  - Acceptance: analyzer semantics prove that:
    - repeated successful output no longer manufactures repeated-failure loops
    - clean verification intervals are not blocked by neutral bookkeeping or success-only output
      when the only remaining issue was the old repeated-failure surface
    - `dead_end_thrash` remains active on real repeated explicit failures
    - `dead_end_thrash` still downgrades to `recovered` and later `historical_only` when recovery
      is honest
    - repeated verification-loop collection and scoring remain unchanged and are not silently
      broadened inside `R1B`
    - exported checkpoint behavior remains deterministic after the narrower failure surface lands
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [x] Task: Add focused regressions for success tails and bookkeeping output
  - Acceptance: regression coverage explicitly proves that:
    - `Exit code: 0` tool output stays non-failure
    - plan-update / bookkeeping text stays non-failure
    - generic function-call output stays non-failure unless the failure signal is explicit
    - export/checkpoint assertions stay deterministic after the cutover
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

Packet `R1B` exit condition:

- `dead_end_thrash` stays active on real explicit failures and no longer treats neutral tool
  output as repeated failure evidence
- clean verification intervals are not blocked by neutral bookkeeping output from the repeated-
  failure surface alone
- export/checkpoint behavior remains deterministic after the cutover
- repeated verification-loop semantics remain unchanged and explicitly deferred to `R1D`

## Packet R1C: Bounded Replay Diagnosis And Honest Status Correction

- [x] Task: Screen a small proof corpus and exclude delegated / subagent sessions
  - Acceptance: proof-session selection proves that:
    - a small handful of completed sessions were screened before use
    - sessions with delegation/subagent markers were excluded
    - current rollout markers such as `spawn_agent`, `wait_agent`, and `close_agent` are used
      first
    - if current rollout text is too weak for reliable screening, only the smallest acceptance-only
      `subagent_observed` helper is added and it is not broadened into a general supported
      subagent seam
  - Result:
    - rollout-text screening was sufficient; no `subagent_observed` helper was added
    - excluded delegated sessions:
      - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
      - `019e9406-6736-79a2-946b-8a603e557422`
    - both excluded rollouts contained `multi_agent_v1` `spawn_agent` / `wait_agent` /
      `close_agent` markers
  - Verify:
    - inspection of the selected rollout JSONL files
    - targeted test coverage if a tiny acceptance-only helper is added
  - Files:
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `/Users/spensermcconnell/.codex/sessions/`

- [x] Task: Run the first bounded replay proof honestly and route the remaining seam
  - Acceptance: bounded replay work shows that:
    - the compactor -> analyzer -> sentinel replay path runs on the final screened session set
    - the clean control corpus and the representative sticky replay are both compared against their
      source rollout tails
    - proof claims stay clearly labeled as bounded replay proof rather than true live proof
    - if the representative sticky replay still ends active, the remaining analyzer-owned seam is
      named precisely enough to scope `R1D` / `R1E`
  - Result:
    - rerun control corpus that still ends with final `dead_end_thrash.state=cleared` and
      `raw_score=0`:
      - `019e93fa-60d4-73d1-9092-014130b60e14`
      - `019e940c-a91b-7fe0-a967-b0bdd595b581`
      - `019e943c-668e-7a03-992b-6a98cf3055da`
    - representative non-subagent sticky live session still fails the bounded honesty claim on
      rerun:
      - `019e894a-86c9-71e3-b57b-e3d3285f0988`
      - rerun output directory:
        `target/hybrid-drift-evals/019e894a-86c9-71e3-b57b-e3d3285f0988-r1c-refresh/`
      - final checkpoint still ends `dead_end_thrash.state=active` with `raw_score=100`
    - screened but excluded from the final success-tail proof corpus:
      - `019e9401-9d69-7190-a43e-9ee3be08b369`
    - that excluded rollout is non-subagent, but it contains `Exit code: 1` tool-output rows, so
      it is not a successful-output-only tail and does not belong in the bounded success-tail proof
    - current status:
      - corpus screening is still valid
      - clean non-subagent control replays still stay clear
      - the representative sticky non-subagent replay still ends active under the current analyzer
        replay
      - the remaining live blocker is repeated verification-loop semantics plus recovery, not the
        old generic `ToolOutput == failure` rule
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - for each screened proof session:
      - `cargo run -p agent-session-compactor -- --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --output-dir "target/hybrid-drift-evals/$SESSION_ID/compactor"`
      - `cargo run -p agent-drift-analyzer -- --input-dir "target/hybrid-drift-evals/$SESSION_ID/compactor" --output-dir "target/hybrid-drift-evals/$SESSION_ID/analyzer"`
      - `cargo run -p agent-drift-sentinel -- --checkpoint-dir "target/hybrid-drift-evals/$SESSION_ID/analyzer"`
  - Files:
    - `.codex/handoffs/2026-06-06-080214-r1c-verification-loop-root-cause.md`
    - `target/hybrid-drift-evals/`

Packet `R1C` exit condition:

- bounded replay covers a small handful of completed non-subagent sessions
- the family status is corrected honestly even if the representative sticky replay still fails
- any subagent screening added for proof selection stays minimal and does not become a broader
  supported analyzer seam
- the remaining analyzer-owned seam is precise enough to scope `R1D` and `R1E`

## Packet R1D: Land The Repeated Verification-Loop Semantics Fix

- [ ] Task: Update analyzer semantics so repeated successful verification does not stay active by itself
  - Acceptance: analyzer semantics prove that:
    - repeated verification evidence remains visible as historical context
    - completed in-scope success tails do not stay `dead_end_thrash=active` solely because
      verification commands repeated while converging
    - out-of-scope verification still blocks recovery
    - repeated explicit failure evidence still keeps `dead_end_thrash` active
    - the fix stays analyzer-local and does not require a compactor schema change first
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/src/context/working_set.rs`
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`

- [ ] Task: Add focused regressions for repeated verification success tails and out-of-scope verifier loops
  - Acceptance: regression coverage explicitly proves that:
    - repeated successful `cargo fmt`, `cargo clippy`, and `cargo test`-style verification does
      not by itself end a completed in-scope success tail as active thrash
    - repeated verification still stays active when the current interval is out of scope
    - recovered and historical-only transitions remain deterministic after the fix
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`

Packet `R1D` exit condition:

- repeated successful verification loops no longer keep completed in-scope success tails active by
  themselves
- out-of-scope verification and repeated explicit failures still flag honestly
- analyzer export/checkpoint behavior remains deterministic after the verifier-loop fix

## Packet R1E: Re-Prove The Family On The Screened Non-Subagent Replay Corpus

- [ ] Task: Rerun the bounded replay proof after `R1D` and refresh continuity notes
  - Acceptance: bounded replay re-proof shows that:
    - the compactor -> analyzer -> sentinel replay path runs on the final screened session set
    - the representative sticky non-subagent replay no longer ends falsely active from repeated
      verification loops
    - the clean control corpus still ends cleared
    - proof claims stay clearly labeled as bounded replay proof rather than true live proof
    - continuity notes are refreshed if the final proof corpus, verifier shape, or packet-routing
      language changed materially during `R1D`
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - for each screened proof session:
      - `cargo run -p agent-session-compactor -- --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --output-dir "target/hybrid-drift-evals/$SESSION_ID/compactor"`
      - `cargo run -p agent-drift-analyzer -- --input-dir "target/hybrid-drift-evals/$SESSION_ID/compactor" --output-dir "target/hybrid-drift-evals/$SESSION_ID/analyzer"`
      - `cargo run -p agent-drift-sentinel -- --checkpoint-dir "target/hybrid-drift-evals/$SESSION_ID/analyzer"`
  - Files:
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `target/hybrid-drift-evals/`

Packet `R1E` exit condition:

- bounded replay covers a small handful of completed non-subagent sessions after `R1D`
- the representative sticky replay and the clean controls both avoid a false final
  `dead_end_thrash=active` result
- any subagent screening added for proof selection stays minimal and does not become a broader
  supported analyzer seam
- continuity notes reflect the final packet family and bounded-proof story honestly

## Family Exit Condition

- `R1A` landed the analyzer outcome-evidence seam and repeated-failure cutover
- `R1B` landed the honest recovery/export semantics for the narrower repeated-failure surface
- `R1C` landed the honest bounded replay diagnosis and routed the remaining analyzer seam
- `R1D` remains open until repeated verification-loop semantics are fixed on focused analyzer
  regressions
- `R1E` remains open until the screened bounded replay re-proof is honest on a representative
  non-subagent recovery tail as well as the clean control corpus
- the family stayed inside analyzer outcome evidence plus `dead_end_thrash` semantics and did not
  widen into turn context, archetype, progress, or sentinel cleanup
