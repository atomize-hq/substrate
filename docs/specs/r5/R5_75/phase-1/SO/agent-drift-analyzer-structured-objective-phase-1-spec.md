# Spec: Agent Drift Analyzer Structured Objective Phase 1

Status: draft spec created on 2026-06-14 from the structured-objective design stack:
`DESIGN-r5-structured-objective-architecture.md` is the semantic authority,
`DESIGN-r5-structured-objective-evaluation-and-annotation.md` is the acceptance authority,
`DESIGN-r5-structured-objective-migration-and-integration.md` is the landing-order authority,
and `DESIGN-r5-structured-objective-classifier-taxonomy.md` is future-facing only for this phase.

## Assumptions I'm Making

1. Phase 1 means the deterministic, dependency-free sidecar defined by the architecture and
   migration docs: additive `StructuredObjective` state on `ObjectiveSummary`, grounded evidence
   spans, compatibility rendering as a view, and a committed objective-acceptance wall.
2. The current active repo queue still includes `R5.75`; this spec defines the structured-objective
   implementation authority but does not silently rewrite the current root landing-order document or
   claim that the new family is already promoted ahead of the live `R5.75` work.
3. The immediate implementation seam stays analyzer-local by default:
   `crates/agent-drift-analyzer/src/context/objective.rs`, `src/context/mod.rs`,
   `src/checkpoint/schema.rs`, and new analyzer-local tests/fixtures. Downstream consumers such as
   `context/working_set.rs`, `checkpoint/mod.rs`, and `checkpoint/progress.rs` remain follow-on
   work unless a strictly additive compile bridge is required.
4. Weak evidence should stay unknown. Phase 1 succeeds by being honest and grounded, not by filling
   every field.
5. Classifier work is explicitly out of scope for Phase 1 even if the future taxonomy is already
   documented.

If any of these assumptions drift, update this spec before implementation.

## Objective

Replace the analyzer's current “one chosen objective string becomes task truth” boundary with a
phase-1 structured-objective sidecar that is deterministic, evidence-grounded, compatibility-safe,
and acceptance-tested against the WDAP failure family plus neighboring prompt shapes.

Primary users:

1. maintainers reading analyzer output who need the real task meaning instead of one lossy string,
2. future downstream analyzer consumers that need a safer internal comparison bridge than raw
   `objective.text`,
3. reviewers validating that checklist lines, boilerplate, and tooling instructions no longer win
   over the real mission,
4. later structured-objective follow-on work (TaskFrame coexistence, working-set migration,
   checkpoint predicate migration) that must stand on a stable phase-1 semantic base.

Phase 1 succeeds when:

1. `ObjectiveSummary` exposes an additive structured sidecar plus deterministic comparison key while
   preserving the legacy display string,
2. the sidecar captures phase-1 semantic fields with evidence spans and unknowns,
3. WDAP-style prompts stop promoting subordinate checklist lines to canonical task truth,
4. preserved boilerplate-target prompts remain intact,
5. concise `/goal` prompts, review/no-code prompts, and planning/docs prompts remain semantically
   correct,
6. the new acceptance wall scores structure, grounding, forbidden promotions, compatibility, and
   unknown-field correctness rather than exact pretty-string equality alone.

## Phase-1 Semantic Contract

Phase 1 must make all of the following explicit when evidence supports them:

- `objective_class`
- `primary_intent`
- `target`
- `constraints`
- `success_conditions`
- `deliverables`
- `verification_commands`
- `evidence_spans`
- `confidence`
- `unknowns`

Compatibility rules:

- `ObjectiveSummary.text` remains the human-readable display string,
- `ObjectiveSummary.comparison_key` becomes the deterministic internal comparison bridge,
- `ObjectiveSummary.structured` is additive and optional,
- the display string is rendered *from* structured state when safe, not treated as semantic
  authority that the sidecar merely decorates.

Current SO-G4 guardrail: the live `comparison_key` is still provisional while it mirrors display
text. Until SO-3.2 lands and derives the key from structured state, no downstream migration may
treat the current value as the approved semantic bridge.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Current live seams:
  - `crates/agent-drift-analyzer/src/context/objective.rs`
  - `crates/agent-drift-analyzer/src/context/mod.rs`
  - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
  - `crates/agent-drift-analyzer/src/context/working_set.rs`
  - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `crates/agent-drift-analyzer/src/inference/mod.rs`
- Current regression surfaces:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/support/mod.rs`
- New phase-1 acceptance surface to add:
  - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/**`

No analyzer-time classifier, LLM call, or new runtime dependency belongs in Phase 1.

## Commands

Formatting and lint gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Focused analyzer regressions while landing Phase 1:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

Full analyzer validation:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional follow-on spot-check once a later packet starts migrating downstream consumers:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Useful source inspection while implementing:

```bash
rg -n "ObjectiveSummary|StructuredObjective|comparison_key|objective_key|normalized_objective_text" \
  crates/agent-drift-analyzer/src \
  crates/agent-drift-analyzer/tests
```

## Project Structure

```text
docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
  Semantic authority for fields, evidence, precedence, and compatibility rules.

docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
  Acceptance authority for corpus families, fixture shape, metrics, and promotion gates.

docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
  Landing-order authority for additive sidecar first, then downstream migration.

docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
  This phase-1 implementation authority.

docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
  Technical landing order for the phase-1 sidecar and acceptance wall.

docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
  Ordered implementation ledger for phase-1 work.

crates/agent-drift-analyzer/src/context/objective.rs
  Current objective extraction seam; phase-1 section/clause decomposition, structured assembly,
  compatibility rendering, and evidence grounding should land here or in narrowly scoped helpers
  owned by this module.

crates/agent-drift-analyzer/src/context/mod.rs
  Context-pack assembly and re-exports; additive structured-objective wiring should stay analyzer-local here.

crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Existing shared schema home for `Confidence`, `TaskFrame`, and related checkpoint types; phase-1
  structured-objective DTOs and evidence-span enums should live here unless a tighter shared module
  is introduced.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Focused deterministic regressions for objective precedence, decomposition, grounding, and
  compatibility fallback behavior.

crates/agent-drift-analyzer/tests/objective_acceptance.rs
  New committed objective-acceptance harness covering structure, grounding, forbidden promotions,
  compatibility rendering, and unknown-field correctness.

crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/**
  Fixture families defined by the evaluation authority.
```

## Code Style

Prefer explicit staged helpers and evidence-bearing DTOs over longer normalization ladders. Unknowns
are a first-class honest outcome.

```rust
let summary = ObjectiveSummary {
    text: compatibility_text,
    comparison_key,
    structured: Some(structured_objective),
    verification_commands,
    evidence,
};
```

Conventions for this phase:

- Separate structural decomposition from semantic assembly.
- Every nontrivial structured field must cite evidence spans.
- Use human-readable field names and explicit enums; do not hide meaning inside opaque strings.
- Prefer conservative omission/unknown over fabricated target, constraint, or deliverable fields.
- Treat tool-choice instructions as execution metadata unless they change semantic task meaning.
- Do not extend `normalized_objective_text(...)` into the long-term semantic authority.

## Testing Strategy

Phase 1 uses four validation layers.

1. **Focused checkpoint regressions**
   - extend `crates/agent-drift-analyzer/tests/checkpoints.rs`
   - prove mission/scope clauses outrank subordinate checklist lines
   - prove deliberate boilerplate-target prompts remain preserved
   - prove concise `/goal` prompts and no-code/review prompts stay semantically correct

2. **Objective acceptance harness**
   - add `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
   - add committed fixtures under `tests/fixtures/objective_acceptance/**`
   - score field exactness, role accuracy, grounding accuracy, compatibility rendering,
     forbidden-promotion rate, and unknown-field correctness

3. **Full analyzer regression run**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - proves the additive sidecar does not destabilize the existing analyzer contract

4. **Follow-on downstream checks only when needed**
   - `progress_acceptance` or later smoke checks belong to the next migration phases, not this
     phase-1 sidecar landing by default

## Boundaries

- **Always do:**
  - verify any prior packet tasks named as prerequisites are already landed in live repo state and tests before editing; if one is missing, stop and report it instead of compensating inside the later packet
  - keep Phase 1 deterministic and dependency-free
  - preserve evidence spans for every nontrivial structured field
  - keep the sidecar additive and optional
  - preserve the compatibility display string during migration
  - encode ambiguity as `unknowns` rather than guesses

- **Ask first:**
  - migrating `TaskFrame`, `working_set`, `checkpoint/mod.rs`, or `checkpoint/progress.rs`
  - widening public checkpoint/export schema contracts
  - adding a new crate dependency or analyzer-time model runtime
  - changing the active root landing-order authority outside these docs

- **Never do:**
  - make classifier or LLM output the semantic authority in Phase 1
  - let compatibility text remain the hidden truth source for new logic
  - treat the provisional `comparison_key` stopgap as downstream-ready while it still mirrors the
    display string
  - populate structured fields without supporting evidence
  - treat checklist or boilerplate text as the goal when a broader mission span exists
  - remove the legacy display string before downstream migration is complete

## Success Criteria

Phase 1 is done only when all of the following are true:

1. `ObjectiveSummary` includes `comparison_key` and `structured: Option<StructuredObjective>`.
2. The structured sidecar exposes phase-1 semantic fields, evidence spans, confidence, and
   unknowns consistent with the architecture authority.
3. Objective extraction is section-aware and clause-aware enough to stop the WDAP checklist
   promotion family.
4. `tests/objective_acceptance.rs` exists and enforces the evaluation authority's fixture contract.
5. The locked acceptance set includes WDAP linux/macOS seeds plus deliberate boilerplate-target,
   concise-goal, review/no-code, and planning/docs coverage.
6. Compatibility rendering is validated as acceptable output, not by one exact pretty string.
7. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`,
   `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`, and
   `cargo test -p agent-drift-analyzer -- --nocapture` are green.
8. No downstream consumer assumes the sidecar is always present.

## Open Questions

1. Should the phase-1 evidence span carry only char offsets initially, or should row/section/clause
   identifiers be treated as sufficient until a later follow-on needs finer offsets?
2. Should the compatibility renderer live in `context/objective.rs`, or should it be isolated in a
   dedicated helper once `TaskFrame` migration starts?
3. Is a narrow analyzer-local schema home inside `checkpoint/schema.rs` sufficient, or will a small
   shared `structured_objective` module become clearer once Phase 2 begins?
