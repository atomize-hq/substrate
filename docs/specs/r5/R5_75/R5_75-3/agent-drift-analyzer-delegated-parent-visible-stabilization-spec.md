# Spec: Agent Drift Analyzer Delegated Parent-Visible Stabilization (R5.75-3)

Status: draft spec created on 2026-06-22 after `R5.75-2` was promoted in
`docs/specs/r5/R5_75/MAP.md`. This spec is the implementation authority for the active `R5.75-3`
packet.

Authority order for this packet:
`docs/specs/r5/R5_75/MAP.md` (the `R5.75-3` packet) owns landing order, named smoke sessions, and the
promotion gate; this SPEC/PLAN/TASKS family owns the implementation contract; the live analyzer
(`crates/agent-drift-analyzer/src/checkpoint/progress.rs`) plus the current delegated corpus
(`crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`) are the ground truth for
current behavior.

## Assumptions I'm Making

1. **`R5.75-3` is the active seam** because `R5.75-2` is now promoted in the live
   `docs/specs/r5/R5_75/MAP.md` routing note.
2. **This packet stays analyzer-local to delegated parent-visible stabilization in `progress.rs`.**
   It does not widen into compactor changes, public schema changes, or downstream migration onto the
   structured `comparison_key`; comparability/reset logic stays on the legacy
   `task_frame.objective` surface in this packet.
3. **Planning/spec/handoff artifact edits are valid parent-visible synthesis evidence when delegation
   markers are otherwise strong.** The packet should stop discarding parent-visible orchestration solely
   because the parent refined a planning artifact.
4. **At least one native delegated real rollout is promoted into committed `progress_acceptance`
   coverage in this packet.** The default first committed real delegated case is
   `019eb970-3543-7ab1-a5d6-2a62c00c7185`, because the MAP already names it as the positive proof that
   the parent-visible path still works. The more unstable native repros
   `019eb907-95c4-73e1-843e-e337d1e93cb9` and `019eb917-9531-74e0-897d-ad8d362138ec` remain named smoke
   witnesses even if only one native case is promoted into the committed corpus.
5. **The adapted delegated case stays smoke-only in `R5.75-3`.**
   `da59436e63915185` is required manual smoke for this packet, but the broader adapted fixture-family
   formalization remains reserved for `R5.75-5`.
6. **Per-session acceptance must be explicit, but the exact enum/confidence expectations for the
   unstable repros are characterization-driven.** `019eb970-3543-7ab1-a5d6-2a62c00c7185` is pre-locked
   as the positive proof case; `019eb907-95c4-73e1-843e-e337d1e93cb9`,
   `019eb917-9531-74e0-897d-ad8d362138ec`, and `da59436e63915185` must stay in a conservative
   parent-visible lane with limiting child-visibility evidence visible, and Task `R5.75-3.1` locks the
   exact status/confidence/evidence expectations before implementation closes.
7. **One narrow `parent_visible_comparability_fingerprint(...)` tweak is allowed inside this packet if
   needed, but only conditionally.** First land the narrower planning/spec/handoff stabilization; only
   if the remaining failure is specifically over-broad parent-visible comparability resets on the named
   repros may this packet apply one additive comparability tweak in `progress.rs`. Generalized
   fingerprint redesign, public contract changes, or structured-state comparability migration remain out
   of scope.
8. **This docs pass is triplet-only.** The user asked for SPEC/PLAN/TASKS documents in this pass, so
   packet-prompts are intentionally deferred; if prompts are requested later, derive them from the
   finalized tasks ledger rather than improvising packet order from memory.

If any of these assumptions drift, update this spec before implementation.

## Objective

Stabilize delegated `session_progress` so strong parent orchestration/synthesis evidence continues to
surface as `ParentVisibleOrchestration` even when the parent edits planning artifacts or child
visibility is partial/opaque, without overclaiming child execution progress and without widening into
the later `R5.75-6` structured-objective consumer migration.

Primary users:

1. maintainers reading delegated-parent checkpoints and `summary.md` output to understand real
   orchestration progress,
2. the bounded `progress_acceptance` semantic corpus, which currently excludes live delegated cases
   because the analyzer does not yet surface them deterministically enough,
3. later `R5.75-4` and `R5.75-6` packets, which need stable delegated-parent classification before
   anti-flap and objective-faithfulness follow-ons land.

`R5.75-3` succeeds when:

1. parent-visible orchestration is no longer discarded solely because the parent edited a
   plan/spec/handoff artifact;
2. limited child visibility remains visible as limiting evidence instead of being hidden by a fallback
   to generic planning noise;
3. `019eb970-3543-7ab1-a5d6-2a62c00c7185` remains a stable positive proof that the parent-visible path
   still works and becomes the first committed native delegated real-rollout fixture in
   `progress_acceptance`;
4. `019eb907-95c4-73e1-843e-e337d1e93cb9` and `019eb917-9531-74e0-897d-ad8d362138ec` stay in a
   conservative parent-visible lane rather than collapsing into generic planning-only noise; their
   exact status/confidence expectations are locked in the characterization task and then held in manual
   smoke;
5. adapted delegated session `da59436e63915185` holds the same packet-owned parent-visible stability
   bar in manual smoke, while any zero-verifier/anti-flap concerns beyond that boundary remain owned by
   `R5.75-4`;
6. if the named repros still fail only because comparability resets are too broad, one narrow
   comparability tweak may land in this packet on the legacy objective surface; no broader fingerprint
   redesign or structured-state comparability migration is smuggled in;
7. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`,
   `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`, and
   `cargo test -p agent-drift-analyzer -- --nocapture` are all green.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Live code seams for this packet:
  - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
  - one or more delegated case directories under
    `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/`
- Live docs seam: this SPEC/PLAN/TASKS family plus the `R5.75-3` packet in
  `docs/specs/r5/R5_75/MAP.md`

No new crate dependency, no public schema bump, no compactor change, no replay redesign, and no
structured-objective consumer migration belong in this packet.

## Commands

Focused delegated-progress automated gates:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Full analyzer wall for closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Useful source/test inspection while implementing:

```bash
rg -n "parent_visible|delegated|comparability|plan_artifact_edits|child_visibility" \
  crates/agent-drift-analyzer/src/checkpoint/progress.rs \
  crates/agent-drift-analyzer/tests/checkpoints.rs \
  crates/agent-drift-analyzer/tests/progress_acceptance.rs
```

Native smoke harness for the three named delegated-parent sessions:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="<native-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-3/$SESSION_ID"
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

Adapted smoke harness for the named delegated witness:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="da59436e63915185"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-3/$SESSION_ID"
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

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Landing-order authority: the R5.75-3 packet (problem, required change, smoke sessions, promotion gate).

docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md
docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md
docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/checkpoint/progress.rs
  Live delegated-parent classification, limiting-evidence handling, and parent-visible comparability logic.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Fast synthetic/bundle-shaped delegated-parent regressions and comparability-reset checks.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Bounded semantic wall for session_progress; currently excludes delegated real-rollout fixtures that this packet
  is expected to stabilize enough to admit.

crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
  Corpus contract, included/excluded case rationale, and delegated-case guardrails.

target/r5_75-smoke/R5.75-3/<session-id>/
  Manual smoke outputs for the named native and adapted delegated sessions.
```

## Code Style

Prefer explicit, evidence-grounded delegation predicates over broad archetype fallbacks. A planning
artifact edit should only suppress the parent-visible path when it truly means "plain planning with no
delegation synthesis," not whenever a delegated parent happens to touch a spec file.

```rust
let plan_edits = plan_artifact_edits(analysis);
let orchestration_attempts = parent_visible_orchestration_attempts(analysis);
let synthesis_attempts = parent_visible_synthesis_attempts(analysis);

if archetype_label == SessionArchetypeLabel::Planning
    && !plan_edits.is_empty()
    && orchestration_attempts.is_empty()
    && synthesis_attempts.is_empty()
{
    return None;
}
```

Conventions for this packet:

- Keep child-visibility limits explicit and conservative; opaque/partial child work must not become a
  fabricated positive child-progress claim.
- Treat parent edits to plan/spec/handoff artifacts as potential parent-visible synthesis, not as an
  automatic reason to collapse into planning noise.
- Keep comparability tweaks additive and legacy-surface-bound (`task_frame.objective`,
  delegated objective surface, working set); do not silently switch to structured `comparison_key`.
- Preserve the bounded corpus style in `progress_acceptance`: exact files, exact case directories, no
  reads from `target/` or `~/.codex` at test time.

## Testing Strategy

This packet uses four validation layers:

1. **Characterization of named delegated repros**
   - run the named native/adapted smoke harnesses before finalizing implementation closeout
   - record exact lane/status/confidence/evidence expectations for:
     - `019eb907-95c4-73e1-843e-e337d1e93cb9`
     - `019eb917-9531-74e0-897d-ad8d362138ec`
     - `019eb970-3543-7ab1-a5d6-2a62c00c7185`
     - `da59436e63915185`
   - use that characterization to define explicit acceptance in the tasks ledger rather than leaving
     the packet at a vague "looks conservative enough" bar
2. **Fast delegated-parent regressions**
   - extend `crates/agent-drift-analyzer/tests/checkpoints.rs`
   - prove that planning/spec/handoff edits no longer erase parent-visible classification when
     delegation evidence is otherwise strong
   - prove that limited child visibility remains visible as limiting evidence
   - prove that any conditional comparability reset tweak remains narrow
3. **Committed `progress_acceptance` coverage**
   - promote at least one native delegated real case into
     `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
   - keep delegated cases guardrail-only in `R5`: they may prove
     `parent_visible_orchestration` with limited confidence, but must not claim positive opaque-child
     progress before `R7`
   - default first committed real delegated case: `019eb970-3543-7ab1-a5d6-2a62c00c7185`
4. **Automated wall + manual smoke**
   - automated:
     - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
     - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
     - `cargo test -p agent-drift-analyzer -- --nocapture`
   - manual:
     - native sessions
       `019eb907-95c4-73e1-843e-e337d1e93cb9`,
       `019eb917-9531-74e0-897d-ad8d362138ec`,
       `019eb970-3543-7ab1-a5d6-2a62c00c7185`
     - adapted session `da59436e63915185`
   - inspect `summary.md` and the first `checkpoints.jsonl` rows, not just command exit status

## Boundaries

- Always:
  - verify `R5.75-2` is still landed in live repo truth before implementation; if the prerequisite is
    missing, stop and report it instead of compensating inside this packet
  - keep the packet local to delegated parent-visible stabilization in `progress.rs` plus matching
    checkpoint/progress-acceptance regressions
  - keep child-visibility limitations visible as limiting/counter evidence
  - keep comparability/reset logic on the legacy objective surface in this packet
  - use named native sessions as the primary behavior authority
  - inspect `summary.md` and early `checkpoints.jsonl` rows during smoke review
- Ask first:
  - adding more than the bounded delegated real-case expansion needed for this packet
  - converting `da59436e63915185` into a committed adapted fixture family before `R5.75-5`
  - public schema changes, compactor changes, or structured-state comparability migration
  - a packet-prompt artifact derived from this triplet (deferred in this docs pass by user scope)
- Never:
  - overclaim positive opaque-child progress
  - silently widen into generalized parent-visible fingerprint redesign
  - replace the legacy `task_frame.objective` comparability surface with structured `comparison_key`
    inside this packet
  - remove the synthetic delegated guardrail proof without replacing it with an equally explicit
    delegated guardrail contract
  - close the packet from green tests alone without the named smoke review

## Success Criteria

1. The spec covers the packet's objective, commands, structure, code style, testing strategy, and
   boundaries.
2. `progress.rs` no longer discards parent-visible orchestration solely because the parent touched a
   plan/spec/handoff artifact.
3. `019eb970-3543-7ab1-a5d6-2a62c00c7185` is admitted into committed `progress_acceptance` coverage as
   the first native delegated real-rollout proof.
4. `019eb907-95c4-73e1-843e-e337d1e93cb9`,
   `019eb917-9531-74e0-897d-ad8d362138ec`, and `da59436e63915185` have explicit packet-local
   acceptance expectations locked by characterization and then upheld in smoke.
5. Any comparability follow-on remains narrow, additive, and conditional; if a broader redesign is
   required, the packet stops and opens a new issue instead of widening silently.
6. The three analyzer validation commands are green and the named native/adapted smoke sessions all
   hold a conservative but stable parent-visible interpretation.

## Open Questions

No blocking open questions remain for drafting this packet. The only intentionally unresolved item is
implementation-time characterization detail:

- if the exact status/confidence/evidence expectations observed for
  `019eb907-95c4-73e1-843e-e337d1e93cb9`,
  `019eb917-9531-74e0-897d-ad8d362138ec`, or `da59436e63915185`
  differ from the assumptions above, update this spec before implementation rather than silently
  changing the packet boundary.
