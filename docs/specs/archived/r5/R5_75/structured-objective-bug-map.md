# Structured Objective Bug Map

Status: read-only diagnosis artifact created on 2026-06-19 after validating that the older `R5.75-1` hard gate predates the broader structured-objective Phase 1 work and after running manual smoke on previously unseen native sessions.

Purpose: capture the current best explanation for why the live structured-objective implementation does not yet behave as intended by the planning/design docs, even though the locked acceptance wall is green.

Scope: this is a diagnosis map, not a fix plan. It is intended to support follow-on debugging, packetization, and future implementation prompts.

> **Currency note (updated 2026-06-20, post-fix).** The body below is the original 2026-06-19
> diagnosis snapshot, preserved as the deep root-cause reference. The **authority for current status**
> is the `R5.75-1 → Remaining Bug Ledger` in `docs/specs/r5/R5_75/MAP.md`. Where the snapshot below
> still describes a defect as unfixed, cross-check the ledger. Current status as of 2026-06-20 (after
> the Issue 1/2/3 anchoring fix landed):
>
> | Issue | Current status |
> |-------|----------------|
> | 1 — extraction scope too broad | **Landed** (R5.75-1 anchoring fix) |
> | 2 — pooled field assembly | **Landed** (R5.75-1 anchoring fix) |
> | 3 — goal-role too permissive (boilerplate gets `Goal` spans) | **Landed** (R5.75-1 anchoring fix) |
> | 4 — intent misclassification | Landed (`SO-2.3D`) — now unconditional (1/2/3 landed) |
> | 5 — target prefers broad repo/dir paths | OPEN — non-blocking follow-on |
> | 6 — success/deliverable over-upgrade | Landed (`SO-2.3D`) — now unconditional (1/2/3 landed) |
> | 7 — legacy narrowing overwrite + downstream migration | Scheduled as bounded packet `R5.75-6` (gated before `R6`); deeper consumer migration a later phase |
> | 8 — acceptance corpus blind spots | **Landed** (`orchestration-evaluate-ask-anchor` + `checkpoints.rs` regression) |
>
> The 2026-06-20 gate first **held** `R5.75-1` promotion (`019eb47f` anchored its goal to boilerplate:
> implement intent + cargo-ladder `success_conditions`). The Issue 1/2/3 anchoring fix then landed in
> `crates/agent-drift-analyzer/src/context/objective.rs` — goal selection, evidence spans, and field
> assembly are now grounded to the selected goal's mission surface; pasted-boilerplate rows (negative
> `objective_score`) are excluded from goal candidacy; the `Goal` role is source-gated to user/goal
> surfaces; and a structural goal is synthesized for the top user prompt when its phrasing misses the
> keyword heuristics. The gate was re-run **green**, so `R5.75-1` is promoted to `R5.75-2`. Issues 4/6
> (`SO-2.3D`) now hold unconditionally because the goal anchor they depend on is correct.

---

## Executive Summary

The current mismatch is **not one bug**. It is a stack of interacting issues:

1. **Expectation drift / downstream-surface confusion**
   - Phase 1 landed the additive structured sidecar and acceptance wall.
   - It did **not** fully migrate downstream checkpoint/task-frame/progress consumers.
   - Some current observations that "the checkpoint objective is still lossy" are real, but some are also caused by reading deferred downstream surfaces as if they had already been migrated.

2. **Extraction scope is too broad**
   - The structured extractor currently runs across the session's full compact-row surface instead of an "active objective" slice.
   - This lets unrelated scaffolding, earlier asks, later asks, policy text, and review-closeout details enter the same decomposition pool as the real mission.

3. **Structured field assembly is pooled across multiple candidate rows**
   - Even when the selected top goal clause is mostly right, `success_conditions`, `deliverables`, `verification_commands`, and `evidence_spans` are assembled from the full decomposed clause pool.
   - That causes semantic pollution from unrelated rows.

4. **Heuristics are too permissive and substring-driven**
   - Intent classification, goal-role labeling, deliverable detection, verification detection, and target extraction rely on broad substring matching.
   - This allows instruction/policy/process text to look like semantic task state.

5. **Legacy compatibility narrowing is still able to grab the wrong imperative line**
   - The stopgap narrowing logic can still overwrite the effective exported objective with an imperative bullet from a later section (for example an optional reviewer nit) instead of the actual user ask.

6. **The acceptance corpus is too narrow for real orchestration-shaped sessions**
   - The locked corpus is strong for WDAP, instruction-surface controls, concise `/goal`, review/no-code, planning/docs, and research controls.
   - It does not yet cover many of the actual real-session shapes that are failing.

The single highest-level diagnosis is:

> The live implementation still treats a broad **session text soup** as the extraction surface, while the design assumes a grounded, mission-centered objective surface.

---

## Important Expectation Boundary

Before debugging the code, keep the intended Phase 1 boundary straight.

### Intended Phase 1 contract

See:
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md`
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

Key points:
- additive `StructuredObjective` sidecar on `ObjectiveSummary`
- deterministic extraction
- evidence-grounded fields
- compatibility text rendered from structured state when safe
- semantic `comparison_key`
- committed `objective_acceptance` wall

### Explicitly deferred after Phase 1

The plan/tasks docs explicitly defer:
- `TaskFrame` coexistence / semantic migration
- `context/working_set.rs` migration
- `checkpoint/mod.rs` semantic predicate migration
- `checkpoint/progress.rs` comparability migration

Primary file pointers:
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/context/working_set.rs`

Takeaway:
- If the expectation is "all downstream checkpoint/task-frame/progress surfaces should already be fully structured-objective-native," that expectation is ahead of the stated Phase 1 migration boundary.
- That said, the broader smoke failures below are still real implementation problems, not just expectation drift.

---

## Live Smoke Evidence Used For This Diagnosis

### Previously unseen sessions used for manual smoke

All four session IDs below were checked with `rg` in this repo and were not already referenced in repo docs/tests:

- `019edd98-8a79-7b53-a90b-94cd0d32329b`
- `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
- `019edd91-07c6-7de0-a1cf-9828687d5cff`
- `019eddaf-711d-7a93-924a-fc683f37cfd1`

### Stored smoke artifacts

For each session:
- compactor bundle: `target/structured-objective-smoke/<session-id>/compactor/`
- analyzer bundle: `target/structured-objective-smoke/<session-id>/analyzer/`

Important files:
- `target/structured-objective-smoke/<session-id>/compactor/rows.compact.jsonl`
- `target/structured-objective-smoke/<session-id>/analyzer/checkpoints.jsonl`
- `target/structured-objective-smoke/<session-id>/analyzer/summary.md`

### What the smoke showed

#### Session `019edd98-8a79-7b53-a90b-94cd0d32329b`
Session ask shape:
- review Packet 3.6 only, findings-first, no fixing

Observed behavior:
- rendered top objective text looked mostly plausible
- `primary_intent` resolved to **Implement** instead of Review
- target collapsed to repo directory
- `success_conditions` and `deliverables` were polluted with unrelated policy/skill/memory/plugin text

#### Session `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`
Session ask shape:
- validate Packet 3.6 landed, then confirm/deny readiness for Packet 4

Observed behavior:
- checkpoint/task-frame objective narrowed to an optional reviewer nit:
  - `add extra task-local grep checks for transition routing / outcome/final-marker meaning`
- structured compatibility text collapsed to:
  - `Deferred/potential future work only: docs/research/cycle-future-work.md`

This is a strong repro for the "wrong imperative line wins" failure.

#### Session `019edd91-07c6-7de0-a1cf-9828687d5cff`
Session ask shape:
- land only Set 1 Packet 1.2B

Observed behavior:
- objective text stayed near the right ask
- verification commands were recovered reasonably
- target remained unknown
- `success_conditions` / `deliverables` still picked up prompt scaffolding

#### Session `019eddaf-711d-7a93-924a-fc683f37cfd1`
Session ask shape:
- land only Set 1 Packet 1.3

Observed behavior:
- objective text stayed near the right ask
- verification commands were recovered
- target degraded into a broad conceptual topic
- structured side fields still absorbed unrelated scaffolding

---

## Root Cause Map

## Issue 1: Objective extraction is scoped to the full compact session instead of the active objective surface

### Why this matters

The design assumes mission-centered, evidence-grounded extraction. The live implementation instead extracts from the entire session compact-row set.

### Code path

- `crates/agent-drift-analyzer/src/context/mod.rs`
  - `assemble_context(session)`
  - `extract_objective(&session.compact_rows)`

### Why this is risky

This means the extractor sees, in one pool:
- kickoff ask
- later follow-up asks
- AGENTS / developer / system scaffolding
- memory instructions
- plugin / tool capability blocks
- review closeout detail
- optional nits
- output formatting instructions
- subagent status text

The design docs explicitly wanted:
- mission/scope outranking subordinate checklist items
- tool-choice instructions treated as execution metadata
- weak evidence staying unknown

The current scope makes those goals much harder because unrelated rows are admitted before field assembly even starts.

### Primary evidence

- review-only and validation-followup sessions polluted by unrelated rows
- session `019eddaa-e8b2-74b2-9f45-e4ce17aaab55` shows later-detail bullets competing with the actual ask

### Research pointers

Start here:
- `crates/agent-drift-analyzer/src/context/mod.rs`
- `crates/agent-drift-analyzer/src/context/objective.rs`

Questions:
- Should extraction be anchored to the highest-confidence user/goal row first?
- Should only a local neighborhood around the chosen candidate row be decomposed?
- Should later user messages open a new objective scope instead of remaining in the same extraction pool?

---

## Issue 2: Structured field assembly pools clauses from all candidate rows

### Why this matters

Even when the right goal clause is found, the rest of the semantic frame is assembled from the whole decomposition pool.

### Code path

- `crates/agent-drift-analyzer/src/context/objective.rs`
  - `collect_directive_row_candidates(...)`
  - `decompose_objective_rows(...)`
  - `assemble_structured_objective(...)`
  - `success_conditions_from_decomposition(...)`
  - `deliverables_from_decomposition(...)`
  - `verification_commands_from_decomposition(...)`

### Current behavior

The extractor:
- chooses up to `MAX_OBJECTIVE_CANDIDATE_ROWS`
- decomposes all selected rows into sections/clauses
- then builds structured fields from the **entire clause set**

This is the main reason unrelated policy/process/instruction text can show up as:
- `success_conditions`
- `deliverables`
- `verification_commands`
- `evidence_spans`

### Concrete failure signal

The helper inspection on session `019edd98-8a79-7b53-a90b-94cd0d32329b` showed `Goal` evidence spans sourced from:
- real user `/goal` rows
- but also unrelated system-instruction lines like skill catalog text and memory-policy instructions

That should not happen if the semantic frame were anchored to a selected mission row and local grounded clauses.

### Research pointers

Primary files:
- `crates/agent-drift-analyzer/src/context/objective.rs`

Key functions to inspect:
- `collect_directive_row_candidates`
- `decompose_objective_rows`
- `selected_goal_clause`
- `assemble_structured_objective`
- `success_conditions_from_decomposition`
- `deliverables_from_decomposition`
- `verification_commands_from_decomposition`

Key question:
- After selecting the winning goal clause, should all downstream field assembly be constrained to:
  - the same row,
  - the same section,
  - nearby rows in the same turn,
  - or another explicitly defined active-objective slice?

---

## Issue 3: Goal-role detection is too permissive

### Why this matters

The design rules require:
- checklist items not to become goal when broader mission exists
- boilerplate/instruction lines only to become goal when the user explicitly targets that surface

The current role heuristics are broad enough that many non-goal lines still get `Goal` candidates.

### Code path

- `crates/agent-drift-analyzer/src/context/objective.rs`
  - `role_candidates_for_clause(...)`
  - `looks_like_goal_text(...)`
  - `looks_like_constraint_text(...)`
  - `looks_like_deliverable_text(...)`
  - `looks_like_context_text(...)`
  - `looks_like_verification_text(...)`
  - `looks_like_boilerplate_text(...)`

### Why this is failing

`looks_like_goal_text(...)` treats a clause as goal-like if it contains terms such as:
- `review`
- `analyze`
- `ensure`
- `validate`
- `keep`
- `stop`
- `perform`

That is broad enough to turn many process/instruction sentences into goal candidates.

### Concrete failure signal

In the goal-span inspection for `019edd98-8a79-7b53-a90b-94cd0d32329b`, `Goal` spans included unrelated system instruction lines. That is a direct sign that clause-role labeling is admitting too many false-positive goals.

### Research pointers

Inspect:
- `role_candidates_for_clause`
- `looks_like_goal_text`
- section-label handling in `split_objective_sections` / `classify_section_label`

Questions:
- Should goal candidacy require stronger positional/contextual evidence?
- Should system/developer rows be unable to contribute `Goal` unless they are explicit preserved-target cases?
- Should `Goal` be restricted to a smaller set of section kinds and source kinds after a user prompt exists?

---

## Issue 4: Intent classification is substring-driven and misclassifies review prompts as implementation

### Why this matters

The planning docs explicitly wanted review/no-code and planning/docs prompts to remain semantically correct.

### Code path

- `crates/agent-drift-analyzer/src/context/objective.rs`
  - `intent_for_text(...)`

### Current failure pattern

The first check in `intent_for_text(...)` treats any text containing:
- `implement`
- `add`
- `update`
- `wire`
- `land`
- `build`

as `ObjectiveIntent::Implement`.

That means review prompts containing phrases like:
- `already-landed Packet 3.6 implementation`

can be misclassified as implementation just because the text includes the noun `implementation` or a neighboring implementation word.

### Concrete failure signal

Session `019edd98-8a79-7b53-a90b-94cd0d32329b` was a review prompt, but `primary_intent` became `Implement`.

### Research pointers

Inspect:
- `intent_for_text(...)`

Questions:
- Should intent be derived from the selected goal clause plus section/source metadata rather than a raw substring search?
- Should noun forms like `implementation` be treated differently from imperative verbs like `implement`?
- Should review/analyze/determine language outrank implementation nouns when the sentence is findings-first or review-only?

---

## Issue 5: Target extraction is too eager to choose broad repo/directory paths and too weak on packet/doc review targets

### Why this matters

The design expected grounded target extraction. The live behavior often either:
- collapses to repo root / broad directory target, or
- leaves a too-broad conceptual target, or
- leaves target unknown.

### Code path

- `crates/agent-drift-analyzer/src/context/objective.rs`
  - `explicit_target_for_clause(...)`
  - `explicit_target_anchor_for_text(...)`
  - `extract_inline_paths(...)`
  - `extract_named_artifacts(...)`
  - `extract_workspace_refs(...)`
  - `extract_work_item_identifier(...)`
  - `extract_named_doc_target(...)`
  - `extract_named_test_or_verifier_target(...)`
  - `extract_named_conceptual_target(...)`

### Failure pattern

If a goal clause contains a repo path early, target extraction often locks onto that broad path even if the actual task target is more specific, such as:
- a particular packet
- a spec/plan/tasks bundle
- a review surface
- a design doc family

### Concrete failure signals

- `019edd98-8a79-7b53-a90b-94cd0d32329b` collapsed to the cycle repo directory
- `019eddaf-711d-7a93-924a-fc683f37cfd1` degraded to a broad conceptual topic
- `019edd91-07c6-7de0-a1cf-9828687d5cff` left target unknown

### Research pointers

Questions:
- Should packet/doc identifiers outrank broad repo path mentions when both are present?
- Should review-against-doc-family prompts derive multi-artifact targets instead of a single path fallback?
- Should broad repo roots be suppressed when more discriminative doc or packet anchors appear in the same clause?

---

## Issue 6: `success_conditions` and `deliverables` are currently over-upgrading weak evidence into fake structure

### Why this matters

This is a direct violation of the design/evaluation guidance.

The design docs explicitly require:
- tool-choice instructions treated as execution metadata
- weakly supported fields staying unknown
- fields without evidence staying absent/unknown

### Code path

- `crates/agent-drift-analyzer/src/context/objective.rs`
  - `success_conditions_from_decomposition(...)`
  - `deliverables_from_decomposition(...)`
  - `looks_like_deliverable_text(...)`
  - `looks_like_verification_text(...)`
  - `unknowns_for_objective(...)`

### Why this is failing

#### `success_conditions_from_decomposition(...)`
Any clause becomes a success condition if:
- it has top role `Verification`, or
- it contains `green`, or
- it contains `success`

That is too broad for real prompts and allows procedural or policy text to become semantic proof goals.

#### `deliverables_from_decomposition(...)`
Any clause becomes a deliverable if:
- it is in a deliverables section, or
- `looks_like_deliverable_text(...)` matches broad phrases like:
  - `return`
  - `provide`
  - `final response`
  - `tests run`
  - `recommended commit message`

That captures output-format/process text much too aggressively.

#### `unknowns_for_objective(...)`
The only explicit unknowns currently tracked are:
- `primary_goal`
- `target`

There is no symmetric honesty mechanism for:
- weak success conditions
- weak deliverables
- weak constraints
- weak verification

So weak evidence tends to be over-upgraded into populated fields instead of remaining unknown.

### Concrete failure signal

The unseen-session smoke produced huge polluted `success_conditions` and `deliverables` lists sourced from skill catalogs, memory instructions, safety text, and output-format instructions.

### Research pointers

Questions:
- Should success conditions only come from explicitly grounded verification/proof sections tied to the active goal?
- Should output-format instructions be split from semantic deliverables?
- Should additional unknown fields exist for deliverables / success conditions / constraints / verification when evidence is weak?

---

## Issue 7: Legacy narrowing / compatibility logic still overwrites good structure with the wrong imperative line

### Why this matters

Even if the sidecar were mostly correct, the exported effective objective seen in checkpoint/task-frame surfaces can still be replaced by the older narrowing logic.

### Code path

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `checkpoint_analyses(...)`
  - `narrowed_objective_summary(...)`
  - `objective_candidate(...)`
  - `objective_sort_key(...)`
  - `normalized_objective_text(...)`
  - `extract_concrete_imperative_objective_line(...)`
  - `extract_labeled_concrete_objective_text(...)`
  - `extract_embedded_goal_line(...)`

### Key behavior

During checkpoint analysis:
- `assemble_context(...)` first extracts the structured objective
- then `narrowed_objective_summary(...)` runs
- if structured state exists, the checkpoint logic still overlays compatibility display from the narrowed objective

That means the older stopgap can still replace the effective rendered objective text.

### Concrete failure signal

Session `019eddaa-e8b2-74b2-9f45-e4ce17aaab55` narrowed to:
- `add extra task-local grep checks for transition routing / outcome/final-marker meaning`

from an optional reviewer-nits bullet.

The real ask was lower in the same prompt:
- validate Packet 3.6 landed correctly/completely
- then confirm/deny readiness for Packet 4 spec/plan/tasks

### Why this happens

`normalized_objective_text(...)` will accept the first imperative line that "looks concrete," and `objective_line_looks_concrete(...)` uses a broad prefix list including:
- `add`
- `review`
- `determine`
- `plan`
- `land`
- `validate`
- `run`
- `perform`
- `only`

So an imperative bullet inside a later detail section can still win.

### Research pointers

Questions:
- Should narrowing be disabled entirely once a structured objective with grounded goal span exists?
- Should the checkpoint layer preserve the structured objective's chosen goal span rather than re-normalizing the raw row text?
- Should imperative bullets under "optional nits" / "not taken" / review-closeout prose be explicitly demoted?

---

## Issue 8: The acceptance harness does not yet represent the live orchestration session shapes that are failing

### Why this matters

A green acceptance wall is currently insufficient evidence that the extractor works on unseen real sessions.

### Current committed corpus

See:
- `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**`

Current locked corpus families include:
- WDAP linux/macOS kickoff
- preserved instruction-surface controls
- concise `/goal`
- review/no-code control
- planning/docs control
- research/docs control

### Missing live shapes

The current corpus does **not** obviously cover:
- multi-turn orchestration sessions with long AGENTS/developer scaffolding preceding the real ask
- validate-what-landed followups with optional reviewer nits above the real ask
- real review-closeout prompts with multiple packet/doc references and findings-first semantics
- sessions where target should be packet/doc-family, not repo-root or conceptual fallback
- sessions where semantic field assembly must stay local to the active goal row instead of pooled from other candidate rows

### Research pointers

Potential new locked-acceptance families:
- unseen-orchestration-review-packet-closeout
- unseen-validate-then-readiness-followup
- unseen-packet-implementation-subagent-prompt
- active-goal-locality-vs-session-pool
- no-optional-nit-promotion

---

## File Pointer Map For Further Research

## Design / authority docs

- `docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md`
- `docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md`
- `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md`
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md`
- `docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md`

## Primary live code

### Objective extraction and assembly
- `crates/agent-drift-analyzer/src/context/objective.rs`

Key functions:
- `extract_objective`
- `collect_directive_row_candidates`
- `split_objective_sections`
- `split_section_into_clauses`
- `role_candidates_for_clause`
- `select_compatibility_text`
- `compatibility_text_from_structured`
- `assemble_structured_objective`
- `selected_goal_clause`
- `success_conditions_from_decomposition`
- `deliverables_from_decomposition`
- `verification_commands_from_decomposition`
- `intent_for_text`
- `explicit_target_anchor_for_text`
- `extract_inline_paths`
- `extract_named_artifacts`
- `extract_workspace_refs`
- `extract_work_item_identifier`
- `extract_named_doc_target`
- `extract_named_conceptual_target`
- `looks_like_goal_text`
- `looks_like_constraint_text`
- `looks_like_deliverable_text`
- `looks_like_context_text`
- `looks_like_verification_text`
- `looks_like_boilerplate_text`

### Context assembly
- `crates/agent-drift-analyzer/src/context/mod.rs`

Key functions:
- `assemble_context`
- `row_text_is_focusable`

### Checkpoint narrowing / legacy stopgap
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`

Key functions:
- `checkpoint_analyses`
- `narrowed_objective_summary`
- `objective_candidate`
- `objective_sort_key`
- `normalized_objective_text`
- `extract_labeled_concrete_objective_text`
- `extract_embedded_goal_line`
- `extract_inline_concrete_objective_clause`
- `extract_concrete_imperative_objective_line`
- `boilerplate_class`
- `row_is_pure_boilerplate`

### Downstream task-frame bridge
- `crates/agent-drift-analyzer/src/inference/mod.rs`

Key functions:
- `infer_task_frame`
- `infer_confidence`

### Schema
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`

Inspect these types closely:
- `StructuredObjective`
- `ObjectiveEvidenceSpan`
- `ObjectiveUnknown`
- `ObjectiveTarget`
- `SuccessCondition`
- `RequestedDeliverable`
- `TaskFrame`

## Tests / fixture contract

- `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**`
- `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**`

---

## Suggested Reproduction Commands

### Acceptance wall

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

### Full analyzer suite

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

### Re-run a previously unseen session smoke

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="019eddaa-e8b2-74b2-9f45-e4ce17aaab55"
export SMOKE_ROOT="target/structured-objective-smoke/$SESSION_ID"
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

sed -n '1,12p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

### Inspect the real source rows around the failure

```bash
python3 - <<'PY'
import json
p='target/structured-objective-smoke/019eddaa-e8b2-74b2-9f45-e4ce17aaab55/compactor/rows.compact.jsonl'
for line in open(p):
    obj=json.loads(line)
    if obj.get('kind') == 'user_message' and obj.get('event_index') == 5:
        print(obj['text'])
        break
PY
```

### Important current debugging limitation

`analyzer/checkpoints.jsonl` only shows the checkpoint/task-frame export surface.
It does **not** expose the full internal `ContextPack.objective.structured` payload directly.

For deeper diagnosis, use an ad hoc helper or temporary debug binary that calls:
- `agent_drift_analyzer::analyze_bundle(...)`

and prints:
- `session.context.objective.text`
- `session.context.objective.comparison_key`
- `session.context.objective.structured`
- goal-role evidence spans
- selected target
- success conditions
- deliverables
- unknowns

The need for that helper is itself a diagnostic friction point.

---

## Recommended Research Order

If future work starts from this bug map, the recommended order is:

1. **Confirm extraction locality problem**
   - prove how many candidate rows and clauses are entering a failing session
   - compare chosen goal clause vs full field assembly pool

2. **Confirm field-pollution boundaries**
   - instrument which exact rows feed:
     - `success_conditions`
     - `deliverables`
     - `verification_commands`
     - `target`

3. **Audit the narrowing overlay**
   - confirm whether checkpoint display should still apply `narrowed_objective_summary(...)` once a structured goal span already exists

4. **Harden intent and target classification**
   - especially review vs implement, and packet/doc-family vs repo-root fallback

5. **Expand locked acceptance with orchestration-shaped real cases**
   - add failing unseen-session shapes to the corpus in minimized form

---

## Likely Packetizable Bug Families

This diagnosis naturally decomposes into separate future seams:

1. **Extraction-scope / active-objective-locality bug**
2. **Pooled field-assembly pollution bug**
3. **Intent classifier bug**
4. **Target grounding specificity bug**
5. **Legacy narrowing overwrite bug**
6. **Acceptance corpus blind-spot / real-session fixture expansion bug**

Those should likely be treated as distinct packets rather than one large cleanup.

---

## Final Diagnosis Statement

The live structured-objective implementation is currently closest to:

```text
best-effort structured interpretation over a broad session text pool
```

but the planning/design docs intended something closer to:

```text
grounded semantic extraction from the active mission surface, with conservative unknowns and compatibility projection layered on top
```

That gap between **session-wide pooled heuristics** and **active-goal-grounded structure** is the core reason unseen real sessions are still failing.
