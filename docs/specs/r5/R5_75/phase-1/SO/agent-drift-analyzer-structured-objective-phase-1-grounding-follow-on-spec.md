# Spec: Agent Drift Analyzer Structured Objective Phase 1 Grounding Follow-On

Status: draft spec created on 2026-06-16 after verifying the live crate state, the current
structured-objective design stack, and the narrow harness-fix landing; reconciled on 2026-06-17
after the grounding follow-on family landed through `SO-G6`. This follow-on keeps the harness fix
intact and now stands as the bounded grounding-family spec of record.

Authority order for this follow-on:
`DESIGN-r5-structured-objective-architecture.md` owns semantic shape,
`DESIGN-r5-structured-objective-evaluation-and-annotation.md` owns acceptance expectations,
`DESIGN-r5-structured-objective-migration-and-integration.md` owns landing order,
and the existing phase-1 SPEC/PLAN/TASKS remain historical implementation context that now require
reconciliation against landed reality.

## Assumptions I'm Making

1. The checkpoint harness repair that routed `analyze_custom_rows()` through
   `BundleFixture::from_compact_rows(...)` is correct and should stand; this follow-on is not a
   rollback or review dispute about that packet.
2. The immediate structured-objective gap is grounding precision, not broader downstream migration:
   `section_index` / `clause_index` should return first, while `TaskFrame`, `working_set`,
   `checkpoint/mod.rs`, and `checkpoint/progress.rs` remain out of scope.
3. `ObjectiveEvidenceSpan.start_char` / `end_char` may remain `None` for now; additive
   section/clause identifiers are the honest bridge until real character offsets are implemented.
4. The current phase-1 task ledger is stale against landed crate reality and should be reconciled
   before future packet agents use it as implementation truth.
5. `ObjectiveSummary.comparison_key` is still provisional while it equals display text; no
   downstream consumer should rely on it as the semantic comparison authority until a dedicated
   derivation packet lands.

If any of these assumptions drift, update this spec before implementation.

## Objective

Land a bounded structured-objective follow-on that restores additive section/clause evidence
indices, tightens deterministic checkpoint coverage around grounding and classifier ambiguity, and
reconciles the structured-objective planning docs so future agents inherit honest packet boundaries.

Primary users:

1. maintainers reviewing `StructuredObjective` output who need durable grounding beyond excerpt-only
   evidence,
2. future objective-acceptance harness work that must assert which section/clause actually supported
   a field,
3. packet agents resuming the structured-objective family who need a truthful ledger of what has
   already landed versus what remains,
4. downstream migration work that must not treat the current `comparison_key == text` stopgap as a
   finished semantic bridge.

This follow-on succeeds when:

1. `ObjectiveEvidenceSpan` again exposes additive optional `section_index` and `clause_index`
   fields and the live decomposition path populates them where clause-level evidence exists,
2. the public sidecar remains backward-compatible: start/end char offsets stay optional, older
   artifacts remain loadable, and no production input contract is weakened,
3. focused checkpoint regressions cover duplicate-ish excerpts and adversarial headings such as
   `Task constraints`, `Verification task`, `Output request`, `Questions to ask`, `Implementation steps`,
   and `What I need`,
4. the structured-objective phase-1 plan/tasks docs are reconciled so landed schema, sidecar,
   decomposition, preliminary role labeling, and field assembly are not still presented as wholly
   pending work,
5. the docs make explicit that `comparison_key` is still a stopgap and that downstream reliance is
   blocked until a dedicated derivation packet lands,
6. the next objective-acceptance harness packet is explicitly sequenced after grounding restoration
   instead of being left as vague future intent.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Live code seams for this follow-on:
  - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
  - `crates/agent-drift-analyzer/src/context/objective.rs`
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
- Live docs seams for this follow-on:
  - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md`
  - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md`
  - `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md`
  - this follow-on SPEC/PLAN/TASKS family

No classifier runtime, no new dependency, no `TaskFrame` migration, and no downstream checkpoint
predicate migration belong in this packet family.

## Commands

Focused grounding and checkpoint regressions:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full analyzer wall for packet closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional formatting/lint gates if the packet expands beyond narrowly targeted code and docs:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Useful source inspection while implementing:

```bash
rg -n "ObjectiveEvidenceSpan|section_index|clause_index|comparison_key|structured" \
  crates/agent-drift-analyzer/src \
  crates/agent-drift-analyzer/tests \
  docs/specs/r5
```

## Project Structure

```text
docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
  Semantic authority for the sidecar, evidence spans, clause decomposition, and compatibility rules.

docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
  Acceptance authority for grounding expectations, forbidden promotion, and future acceptance-fixture shape.

docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
  Landing-order authority for additive sidecar first, then downstream migration.

docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
  Existing phase-1 implementation authority set; this follow-on must reconcile their stale backlog claims.

docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-plan.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
  New bounded follow-on authority for grounding restoration, docs reconciliation, and next-packet sequencing.

crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Shared schema home for `ObjectiveEvidenceSpan`; additive grounding identifiers belong here.

crates/agent-drift-analyzer/src/context/objective.rs
  Clause decomposition and evidence-span assembly seam; this is where section/clause indices should be populated.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Focused regression surface for grounding, ambiguous duplicate excerpts, and adversarial header classification.
```

## Code Style

Prefer additive, explicit grounding metadata over inferred hidden meaning. If the extractor knows a
supporting section/clause identity, expose it directly and keep weaker localization honest.

```rust
ObjectiveEvidenceSpan {
    row: clause.row_ref.clone(),
    source_kind: clause.source_kind,
    section_kind: clause.section_kind,
    role: role.role,
    excerpt: clause.text.clone(),
    section_index: Some(clause.section_index),
    clause_index: Some(clause.clause_index),
    start_char: None,
    end_char: None,
    confidence: role.confidence,
}
```

Conventions for this follow-on:

- Add schema fields additively with serde defaults; do not break older artifacts.
- Preserve the harness-fix seam; do not widen into `src/input.rs`.
- Use focused regression cases that prove why the new grounding identifiers matter.
- Keep `comparison_key` honesty explicit in docs; do not quietly treat a display-string echo as a
  finished semantic key.
- Reconcile stale ledgers directly instead of expecting future agents to infer landed state from git history.

## Testing Strategy

This follow-on uses four validation layers:

1. **Focused grounding regressions**
   - extend `crates/agent-drift-analyzer/tests/checkpoints.rs`
   - prove goal and verification spans carry `section_index` / `clause_index`
   - prove duplicate-ish scope/checklist wording resolves to the correct supporting span

2. **Adversarial header-classification controls**
   - keep mission extraction from overmatching headings like `Task constraints`, `Verification task`,
     `Output request`, `Questions to ask`, and `Implementation steps`
   - preserve `What I need` as a real mission-like heading

3. **Docs reconciliation proof**
   - manually review the updated phase-1 plan/tasks docs against landed crate state
   - ensure grounded follow-on packets are explicit and that already-landed work is not still shown
     as untouched backlog

4. **Full analyzer regression**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - keep the wider analyzer wall honest after the additive grounding change

## Boundaries

- **Always:**
  - preserve the landed harness fix and centralized fixture repair,
  - keep `ObjectiveEvidenceSpan` changes additive and backward-compatible,
  - run the focused checkpoint wall before calling any packet complete,
  - reconcile stale docs in the same packet family instead of leaving them misleading.

- **Ask first:**
  - migrating any downstream consumer to `comparison_key` or richer structured state,
  - replacing `start_char` / `end_char` placeholders with real offsets if that expands scope beyond
    the grounding identifiers,
  - regenerating packet-prompt artifacts if doc reconciliation changes the live packet order enough
    to invalidate those prompts.

- **Never:**
  - weaken the analyzer input contract in `src/input.rs` to accommodate test or grounding work,
  - reintroduce raw display text as hidden semantic truth under a new name,
  - mark `comparison_key` as production-ready for downstream reliance while it still mirrors the
    pretty display string,
  - silently leave the phase-1 ledger stale once this follow-on is approved to proceed.

## Success Criteria

1. `ObjectiveEvidenceSpan` exposes optional `section_index` and `clause_index`, and the live
   decomposition path populates them for clause-backed evidence spans.
2. Checkpoint regressions cover both:
   - section/clause population for goal + verification spans, and
   - ambiguous duplicate-ish wording where excerpt-only grounding would be insufficient.
3. Adversarial heading tests prove generic mission-word matching does not swallow more specific
   section labels while `What I need` still resolves correctly.
4. The phase-1 plan/tasks docs accurately distinguish:
   - what has already landed,
   - what remains for grounding hardening,
   - that `comparison_key` derivation is still unfinished,
   - that the objective-acceptance harness is the next substantive acceptance packet.
5. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` and
   `cargo test -p agent-drift-analyzer -- --nocapture` are green for the final implementation
   packet that follows these docs.

## Open Questions

1. Should the eventual character-offset implementation reuse the same packet family after
   `section_index` / `clause_index`, or should it be a later dedicated localization follow-on?
2. Should the existing phase-1 packet-prompt artifact be regenerated immediately after the plan/task
   ledger is reconciled, or is a docs-only warning sufficient until implementation packets are approved?
3. Resolved by SO-G5: continue the post-grounding sequence as `SO-2.3B-refine` first, then
   `SO-3.1` / `SO-3.2`, then the future objective-acceptance harness as `SO-4.1` / `SO-4.2` under
   the reconciled phase-1 docs rather than skipping directly to the harness boundary.
