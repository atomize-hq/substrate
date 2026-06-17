# Spec: Agent Drift Analyzer Session Progress R5.75-1

Status: draft spec created on 2026-06-12 from `docs/specs/r5/R5_75/MAP.md` after `R5.75-0`
landed and promoted this packet to the active pre-`R6` implementation gate.

## Assumptions I'm Making

1. `R5.75-1` is the next active packet because `R5.75-0` already reconciled the stale `R5.5`
   authority docs and cleared the promotion gate in `docs/specs/r5/R5_75/MAP.md`.
2. The bug is now narrower than the older `R5.5-2` objective extraction work: live code already
   filters many boilerplate classes, but `normalized_objective_text(...)` and candidate ordering in
   `crates/agent-drift-analyzer/src/checkpoint/mod.rs` can still let giant pasted user prompts win
   as the stored checkpoint objective.
3. This packet should stay analyzer-local to `checkpoint/mod.rs` and
   `tests/checkpoints.rs`; it should not widen into compactor changes, replay/schema changes, or
   the adapted fixture-family work reserved for `R5.75-5`.
4. Native `.codex` sessions remain the primary authority for expected objective behavior, while the
   adapted `05a56cc51632982b` case is a smoke-only robustness witness for this packet.
5. The packet must preserve user-requested boilerplate targets when the real task is to analyze,
   compare, explain, or edit `AGENTS.md`, `<skill>`, `Available skills`, tooling scaffolds, or
   similar instruction surfaces.

If any of these assumptions drift, update this spec before implementation.

## Objective

Make first-checkpoint objective selection condense giant pasted user prompts down to the shortest
concrete task ask, without regressing the preserved cases where the instruction/boilerplate surface
itself is the true requested target.

Primary users:

1. maintainers reading checkpoint output to understand the real task frontier quickly,
2. downstream drift/scoring consumers that rely on the checkpoint objective as the task anchor,
3. reviewers validating that later progress comparisons stay attached to the true ask rather than
   to pasted skill/profile scaffolding,
4. future `R5.75-*` packets, which need stable objective anchoring before later fail-open and
   delegated-parent fixes land.

`R5.75-1` succeeds when:

1. long user prompts condense to the smallest concrete actionable ask when that ask is present,
2. `/goal`, short imperative asks, steer pivots, and clear workspace/action-target phrases outrank
   longer same-turn scaffolding bodies,
3. the current same-priority length bias no longer favors a longer pasted body over a shorter
   concrete target,
4. user requests to inspect or edit boilerplate surfaces remain preserved instead of being
   incorrectly shortened away,
5. the three named native smokes and the one named adapted smoke all show the concrete task in the
   first checkpoint objective.

## Tech Stack

- Language: Rust 2021
- Crate under change: `agent-drift-analyzer`
- Primary implementation seam: checkpoint objective selection in
  `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- Primary regression surface: `crates/agent-drift-analyzer/tests/checkpoints.rs`
- Manual smoke tooling:
  - `agent-session-compactor`
  - `agent-drift-analyzer`
  - `agent-drift-sentinel`
- Packet authority:
  - `docs/specs/r5/R5_75/MAP.md`
  - this spec
  - the companion plan/tasks docs for `R5.75-1`

No new crate dependency, checkpoint schema bump, or fixture-family expansion is expected here.

## Commands

Focused objective-selection regression suite:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full analyzer validation:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional downstream compatibility sentinels only if checkpoint output shape or replay/operator
compatibility changes unexpectedly:

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Native smoke harness for the named primary sessions:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="<native-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/r5_75-1/$SESSION_ID"
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

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Adapted smoke harness for the required secondary witness:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="05a56cc51632982b"
export SMOKE_ROOT="target/r5_75-smoke/r5_75-1/$SESSION_ID"
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

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Relevant source/test inspection while implementing:

```bash
rg -n "narrowed_objective_summary|normalized_objective_text|objective_candidate|boilerplate" \
  crates/agent-drift-analyzer/src/checkpoint/mod.rs \
  crates/agent-drift-analyzer/tests/checkpoints.rs
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Packet-family authority and promotion gate for R5.75.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md
  This packet's objective/verification authority.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md
  Technical implementation plan for the packet.

docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md
  Ordered implementation checklist for this packet only.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Live objective-candidate ranking, boilerplate classification, and objective normalization logic.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Regression surface for synthetic objective-selection and checkpoint-boundary cases.

target/manual-r55-validation/
  Primary native evidence bundle for manual smoke validation.

target/ranga-validation/codex-home/
target/ranga-validation/runs/
  Secondary adapted-external smoke inputs used only for robustness confirmation in this packet.
```

## Code Style

Prefer small, explicit ranking helpers over ad hoc string hacks or “longer text is better”
tie-breaks. The desired style is deterministic, reviewable, and conservative about preservation.

```rust
fn objective_sort_key(candidate: &ObjectiveCandidate<'_>) -> (u8, u8, u8, std::cmp::Reverse<usize>) {
    (
        candidate.pivot_priority,
        candidate.priority,
        candidate.role_priority,
        std::cmp::Reverse(candidate.text.len()),
    )
}
```

Conventions for this packet:

- Prefer explicit extraction/condensation rules tied to recognized objective surfaces.
- Shorter concrete asks should beat longer same-priority pasted bodies once the true target has
  been identified.
- Preserve full text when shortening would destroy a deliberate boilerplate-target request.
- Keep helper names grounded in objective semantics (`condense`, `target`, `preserve`,
  `candidate`) rather than generic “clean up text” wording.

## Testing Strategy

This packet uses four validation layers:

1. **Focused checkpoint regressions**
   - extend `crates/agent-drift-analyzer/tests/checkpoints.rs`
   - cover giant pasted user prompts whose true ask appears later in the body
   - cover shorter action-target phrases beating longer same-priority scaffolding
   - cover preserved boilerplate-target requests so the packet does not regress the `R5.5-2`
     protection wall
2. **Full analyzer regression run**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - proves the new heuristic does not destabilize other checkpoint behavior
3. **Native manual smoke**
   - sessions:
     - `019eb430-6f9a-7a03-9a63-cb451b654795`
     - `019eb47f-0118-7e90-8291-30a1fb93769e`
     - `019eb98e-3c16-7ba0-92f9-0085654b470c`
   - confirm the first checkpoint objective resolves to the concrete task, not the pasted
     skill/profile/instruction body
4. **Adapted manual smoke**
   - session: `05a56cc51632982b`
   - confirm the adapted export also condenses to the workspace action request without becoming a
     committed fixture family yet

Sentinel tests are not part of the default gate unless the checkpoint output contract changes in a
way that plausibly affects downstream presentation.

## Boundaries

- Always:
  - verify any prior packet tasks named as prerequisites are already landed in live repo state and tests before editing; if one is missing, stop and report it instead of compensating inside the later packet
  - keep the packet scoped to checkpoint objective condensation / target extraction
  - preserve deliberate boilerplate-target requests as first-class objectives
  - use native-session behavior as the primary authority
  - inspect `summary.md` and early `checkpoints.jsonl` rows during smoke review, not just exit
    codes
- Ask first:
  - widening into compactor parsing or replay/schema changes
  - adding committed adapted fixture families before `R5.75-5`
  - changing broader phase-boundary segmentation beyond what the objective fix strictly requires
- Never:
  - reintroduce a tie-break that prefers longer same-priority prompt bodies
  - silently drop a real user request to analyze/edit boilerplate itself
  - declare `R5.75-1` done from green tests alone without the named smoke review

## Hard Gate For R5.75-1

Do not close `R5.75-1` unless all of the following are true:

1. the stored first-checkpoint objective resolves to the concrete task ask, not to the pasted
   skill/profile/instruction body, for native sessions
   `019eb430-6f9a-7a03-9a63-cb451b654795`,
   `019eb47f-0118-7e90-8291-30a1fb93769e`,
   `019eb98e-3c16-7ba0-92f9-0085654b470c`, and adapted session `05a56cc51632982b`,
2. preserved boilerplate-target requests still remain intact when the real task is to inspect,
   compare, explain, analyze, or edit that boilerplate surface itself,
3. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green, and
4. `cargo test -p agent-drift-analyzer -- --nocapture` is green.

Green tests alone are not sufficient. Manual smoke review of `summary.md` and the first
`checkpoints.jsonl` rows is mandatory for packet closeout.

## Success Criteria

1. `crates/agent-drift-analyzer/src/checkpoint/mod.rs` no longer prefers a longer same-priority
   pasted objective body over a shorter concrete task ask.
2. `crates/agent-drift-analyzer/tests/checkpoints.rs` contains regression coverage for:
   - giant pasted user prompts condensing to the true ask,
   - the adapted-style workspace action request,
   - preserved instruction/skill/tooling boilerplate targets.
3. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green.
4. `cargo test -p agent-drift-analyzer -- --nocapture` is green.
5. Manual smoke for native sessions
   `019eb430-6f9a-7a03-9a63-cb451b654795`,
   `019eb47f-0118-7e90-8291-30a1fb93769e`,
   `019eb98e-3c16-7ba0-92f9-0085654b470c`,
   plus adapted session `05a56cc51632982b`, all show the concrete task ask in the first
   checkpoint objective.

## Open Questions

None blocking for packet start.

Resolved by current authority:

1. The adapted `05a56cc51632982b` case remains smoke-only in `R5.75-1`; committed adapted fixture
   family work is deferred to `R5.75-5`.
2. Boilerplate-target preservation remains in scope here because the map explicitly calls out
   preserving those real targets as part of this packet’s correctness definition.
