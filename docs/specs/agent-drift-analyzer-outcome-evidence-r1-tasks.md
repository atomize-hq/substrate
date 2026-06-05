# Tasks: Agent Drift Analyzer Outcome Evidence R1 Packet Family

This task list implements:

- `docs/specs/agent-drift-analyzer-outcome-evidence-r1-spec.md`
- `docs/specs/agent-drift-analyzer-outcome-evidence-r1-plan.md`

## Task List

## Packet R1A: Lock The Family Boundary And Land The Analyzer Outcome-Evidence Seam

- [x] Task: Lock the `R1A` / `R1B` / `R1C` packet-family boundary in repo docs
  - Acceptance: the docs explicitly define:
    - `R1A` as docs lock plus analyzer-only outcome-evidence classification and repeated-failure
      cutover
    - `R1B` as recovery/export semantic proof on top of the landed narrower evidence surface
    - `R1C` as bounded non-subagent replay proof plus continuity-note refresh
    - repeated successful `ToolOutput` as non-failure by default
    - exclusion of delegated / subagent sessions from the `R1C` proof corpus
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

- repo docs define the `R1A` / `R1B` / `R1C` split clearly
- analyzer owns one explicit outcome-evidence seam instead of raw `ToolOutput == failure`
- repeated-failure history excludes successful tool output by default
- repeated verification loop collection is unchanged

## Packet R1B: Preserve Honest Recovery And Export Semantics

- [x] Task: Preserve honest recovery and `dead_end_thrash` state semantics after the evidence cutover
  - Acceptance: analyzer semantics prove that:
    - repeated successful output no longer manufactures repeated-failure loops
    - clean verification intervals are not blocked by neutral bookkeeping or success-only output
    - `dead_end_thrash` remains active on real repeated explicit failures
    - `dead_end_thrash` still downgrades to `recovered` and later `historical_only` when recovery
      is honest
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

- `dead_end_thrash` stays active on real explicit failures but clears or downgrades on
  successful-output-only tails
- clean verification intervals are not blocked by neutral bookkeeping output
- export/checkpoint behavior remains deterministic after the cutover

## Packet R1C: Prove The Family On A Screened Non-Subagent Replay Corpus

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

- [x] Task: Prove `R1C` on a small handful of completed non-subagent sessions and refresh continuity notes
  - Acceptance: bounded proof shows that:
    - the compactor -> analyzer -> sentinel replay path runs on the final screened session set
    - final checkpoints no longer end in active `dead_end_thrash` for successful-output-only tails
    - proof claims stay clearly labeled as bounded replay proof rather than true live proof
    - continuity notes are refreshed if the final proof corpus, verifier shape, or packet-routing
      language changed materially during `R1A` / `R1B`
  - Result:
    - final success-tail proof corpus:
      - `019e93fa-60d4-73d1-9092-014130b60e14`
      - `019e940c-a91b-7fe0-a967-b0bdd595b581`
      - `019e943c-668e-7a03-992b-6a98cf3055da`
    - each replay ended with final `dead_end_thrash.state=cleared` and `raw_score=0`
    - screened but excluded from the final success-tail proof corpus:
      - `019e9401-9d69-7190-a43e-9ee3be08b369`
    - that excluded rollout is non-subagent, but it contains `Exit code: 1` tool-output rows, so
      it is not a successful-output-only tail and does not belong in the bounded success-tail proof
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

Packet `R1C` exit condition:

- bounded proof covers a small handful of completed non-subagent sessions
- final checkpoints no longer stay falsely active on successful-output-only tails
- any subagent screening added for proof selection stays minimal and does not become a broader
  supported analyzer seam
- continuity notes reflect the final packet family and bounded-proof story honestly

## Family Exit Condition

- `R1A` landed the analyzer outcome-evidence seam and repeated-failure cutover
- `R1B` landed the honest recovery/export semantics and focused regressions
- `R1C` landed the screened bounded replay proof and continuity-note refresh
- the family stayed inside analyzer outcome evidence plus `dead_end_thrash` semantics and did not
  widen into turn context, archetype, progress, or sentinel cleanup
