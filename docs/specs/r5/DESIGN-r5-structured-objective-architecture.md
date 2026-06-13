# Design: R5 Structured Objective Architecture

Status: draft design authority created on 2026-06-13 from the objective-architecture dossier,
live repo inspection, and the cited primary research/model sources.

## Why This Doc Exists

The analyzer still treats one extracted string as canonical task truth. The confirmed WDAP kickoff
failure shows why that boundary is wrong for long structured prompts: a subordinate checklist line
(`Run this task on a linux machine.`) can beat the actual mission in `## Scope`
(`Ensure the slice is green for linux...`).

This doc defines the design replacement for that abstraction. It is not a packet plan. It is the
canonical architecture note for how objective handling should evolve before additional heuristic or
model work continues.

## Confirmed Failure Shape

The dossier and live code inspection establish all of the following:

1. the kickoff user row is usually the right row to inspect,
2. the main failure is inside row interpretation and normalization,
3. the current ladder can over-promote a subordinate imperative line,
4. the repo still leaks that single chosen string into truth-artifact sourcing, task-frame
   construction, checkpoint heuristics, and progress comparison.

That means the problem is not merely "add one more imperative prefix." It is that one lossy string
is doing too much semantic work.

## Current Repo Reality

The current codebase still assumes one canonical objective string in several places:

1. `crates/agent-drift-analyzer/src/context/objective.rs`
   - `ObjectiveSummary` is currently `{ text, verification_commands, evidence }`.
2. `crates/agent-drift-analyzer/src/context/working_set.rs`
   - path attribution still checks `objective.text.contains(&path)`.
3. `crates/agent-drift-analyzer/src/inference/mod.rs`
   - `TaskFrame.objective` is inferred from `context.objective.text`.
4. `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
   - several predicates still inspect objective text directly.
5. `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
   - comparability and continuity logic still normalize and compare objective strings.
6. the regression wall still contains many exact-string assertions for task truth.

So a wrong string is not just cosmetic; it can distort downstream reasoning.

## Research-Derived Design Principles

The papers used in the dossier converge on several concrete patterns that are directly useful here.

### Principle 1: Decompose first, then solve

`Decomposed Prompting` argues that complex tasks should be decomposed into simpler sub-tasks that
can be solved by modular components, and those components can later be replaced by better prompts,
models, or symbolic functions. Design implication: objective handling should not be one monolithic
"pick one line" function. It should be a staged pipeline with swappable stages.

### Principle 2: Treat extraction as on-demand structured output

`Instruct and Extract` frames extraction as following user instructions and returning structured
outputs, with user-specified or contextually inferred headers. Design implication: the objective
system should emit structured fields (goal, constraints, deliverables, verification, etc.), not
just a sentence summary.

### Principle 3: Separate structuring from conceptualizing

`Universal Information Extraction as Unified Semantic Matching` explicitly decouples IE into
structuring and conceptualizing, jointly encoding schema and input text and decoding target
structures on demand. Design implication: the analyzer should separate:

- finding the structural units of the prompt,
- understanding what those units mean,
- assembling the final objective frame.

### Principle 4: Keep extraction grounded to source spans

`LMDX` emphasizes that extraction without grounding/localization is insufficient; predictions need a
grounding mechanism back to the source. Design implication: every structured objective field should
carry evidence spans to concrete rows/clauses instead of being a free-floating inferred string.

### Principle 5: Represent user requests as intent plus slots/state

Joint intent/slot work and schema-guided dialogue research support treating task meaning as intent
plus fielded state rather than one sentence. Design implication: the analyzer should preserve at
least intent, target, constraints, success conditions, deliverables, and unknowns as separate
fields.

### Principle 6: Use natural-language schema descriptions, not opaque internal names

`SGD` and `Description-Driven Task-Oriented Dialog Modeling` both emphasize predictions over a
dynamic set of intents/slots described in natural language rather than relying on brittle internal
ontology names. Design implication: any structured-objective schema introduced here should have
human-readable field descriptions and should not depend on magic enum names alone.

### Principle 7: A flat intent-slot pair is too weak for compositional asks

`MTOP` motivates moving beyond the simplest intent+slot framing when queries become compositional.
Design implication: the analyzer needs more than `intent + target`; it needs explicit constraint,
verification, and deliverable channels because repo tasks are often nested and compositional.

### Principle 8: Low-data classifiers should optimize data efficiency, not just raw size

`SetFit` shows a viable low-data pattern: contrastive fine-tuning on a small number of text pairs,
then training a classifier head on embeddings. Design implication: if classifier support is added,
it should be optional, cheap, and evaluated as a low-data baseline, not treated as a prerequisite.

## Locked Architecture Decisions

The dossier already resolved several questions. This doc adopts them as design authority.

1. **Architecture problem, not heuristic-only bug.**
2. **Structured sidecar before full replacement.**
3. **Tool-choice metadata stays outside semantic constraint fields.**
4. **Head A / Head B / Head C remain separate stages.**
5. **MiniLM is the first small-model prototype if classifier work starts.**

## Proposed Core Abstraction

The analyzer should move from:

```text
one chosen row -> one normalized objective string -> task truth
```

to:

```text
directive rows
  -> section and clause decomposition
  -> role labeling
  -> intent and target inference
  -> structured objective frame
  -> compatibility rendering for legacy consumers
```

## Proposed Phase-1 StructuredObjective

Recommended phase-1 sidecar:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredObjective {
    pub objective_class: ObjectiveClass,
    pub primary_intent: ObjectiveIntent,
    pub target: Option<ObjectiveTarget>,
    pub constraints: Vec<ObjectiveConstraint>,
    pub success_conditions: Vec<SuccessCondition>,
    pub deliverables: Vec<RequestedDeliverable>,
    pub verification_commands: Vec<String>,
    pub evidence_spans: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
    pub unknowns: Vec<ObjectiveUnknown>,
}
```

Recommended supporting types:

```rust
pub enum ObjectiveClass {
    TaskStatement,
    NotTaskStatement,
}

pub enum ObjectiveIntent {
    Implement,
    Debug,
    Review,
    Research,
    Plan,
    Validate,
    Docs,
    OtherTask,
}

pub enum ObjectiveRole {
    Goal,
    Constraint,
    Verification,
    Context,
    OtherRole,
}

pub struct ObjectiveEvidenceSpan {
    pub row: RowRef,
    pub role: ObjectiveRole,
    pub excerpt: String,
    pub confidence: Confidence,
}
```

## Minimum Semantic Coverage

A valid phase-1 sidecar must make all of the following explicit:

1. whether the text is actually task-bearing,
2. what primary task the user is asking for,
3. what the task is about,
4. what semantically binding constraints exist,
5. what proof or success conditions are requested,
6. what deliverable is expected,
7. what exact evidence supports those claims,
8. what remains ambiguous or unknown.

If a design cannot answer those separately, it is still too close to the current one-string model.

## Proposed Processing Pipeline

### Stage 0: Directive candidate selection

Input rows should still be filtered to likely objective-bearing directive rows. The existing
`R5.75-1` objective-condensation work remains useful here, but only as a front-end filter.

### Stage 1: Section decomposition

Long prompts should be segmented into meaningful sections such as:

- scope / mission
- checklist / operational steps
- verification
- boundaries / constraints
- context / motivation
- metadata / boilerplate

This follows the decomposition logic supported by `Decomposed Prompting` and the schema-first
paradigm from the IE papers.

### Stage 2: Clause decomposition

Within sections, the analyzer should identify sentence or clause candidates. This is the unit used
for semantic role assignment. Whole-row labeling is too coarse for WDAP-like prompts.

### Stage 3: Clause role labeling

Each clause should be labeled as one of:

- `goal`
- `constraint`
- `verification`
- `context`
- `other_role`

This should be clause- or sentence-level, not whole-utterance-level.

### Stage 4: Intent and target inference

After role labeling, infer:

- primary task intent,
- main target,
- optional secondary target signals for later analysis.

This is where the intent+slot/state literature contributes most directly.

### Stage 5: Structured frame assembly

Assemble the `StructuredObjective` using only grounded spans. No field should be populated without a
supporting clause or an explicit inference rule.

### Stage 6: Compatibility rendering

Render the legacy objective string from the structured frame only after the structured frame exists.
The string becomes a compatibility view, not the source of truth.

## Grounding Rules

Following the grounding/localization lesson from `LMDX`, the new system should obey these rules:

1. every nontrivial field must cite one or more source spans,
2. fields without good evidence should stay unknown,
3. the same clause may support multiple fields, but each usage should be explicit,
4. display text and reasoning evidence must stay linked.

## What Counts As Semantic State

Include in semantic task state:

- requested task intent,
- task target,
- scope limits,
- no-code / docs-only / packet-only restrictions,
- proof and verification requests,
- requested deliverable type.

Do not include in semantic task state:

- `use_hf_cli`-style tool-choice instructions,
- shell/program preferences that do not change task meaning,
- transient orchestration metadata,
- implementation convenience hints that belong to execution policy rather than semantics.

## Compatibility Model

Phase 1 should preserve current consumers by adding a sidecar rather than deleting current fields.

Recommended incremental shape:

```rust
pub struct ObjectiveSummary {
    pub text: String,
    pub structured: Option<StructuredObjective>,
    pub verification_commands: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
}

pub struct TaskFrame {
    pub objective: String,
    pub structured_objective: Option<StructuredObjective>,
    // existing fields remain during migration
}
```

## Why Not A Single Flat Classifier

A one-head design that predicts task gate + primary intent + clause role together would couple three
different granularities:

- utterance/task presence,
- request-level intent,
- clause-level semantic role.

The dossier already rejected that collapse, and the description-driven / schema-guided research
supports keeping them separate.

## Why Not Just Keep Improving `normalized_objective_text(...)`

That path can reduce some failures but does not fix the underlying mismatch:

- it still serializes one string as truth,
- it still mixes structural selection and semantic interpretation,
- it still provides weak auditability,
- it still leaves downstream modules dependent on a lossy representation.

## Non-Goals

This architecture doc does not commit the repo to:

- immediate model training,
- immediate checkpoint schema versioning,
- rewriting checkpoint boundary logic in the same slice,
- collapsing execution metadata into semantic state,
- replacing all string consumers in one packet.

## Open Design Questions

1. what is the smallest useful `ObjectiveTarget` representation for phase 1,
2. whether compatibility rendering should remain one string or split into display vs comparison
   strings internally,
3. whether clause segmentation should be purely heuristic in phase 1 or reserve an optional learned
   helper immediately,
4. which downstream string consumers should migrate first after the sidecar lands.

## Research Source Map

Primary sources used for this design:

- Decomposed Prompting: [arXiv 2210.02406](https://arxiv.org/abs/2210.02406)
- Instruct and Extract: [arXiv 2310.16040](https://arxiv.org/abs/2310.16040)
- Universal Information Extraction as Unified Semantic Matching: [arXiv 2301.03282](https://arxiv.org/abs/2301.03282)
- LMDX: [arXiv 2309.10952](https://arxiv.org/abs/2309.10952)
- BERT for Joint Intent Classification and Slot Filling: [arXiv 1902.10909](https://arxiv.org/abs/1902.10909)
- Schema-Guided Dialogue Dataset: [arXiv 1909.05855](https://arxiv.org/abs/1909.05855)
- Description-Driven Task-Oriented Dialog Modeling: [arXiv 2201.08904](https://arxiv.org/abs/2201.08904)
- MTOP: [arXiv 2008.09335](https://arxiv.org/abs/2008.09335)
- SetFit: [arXiv 2209.11055](https://arxiv.org/abs/2209.11055)
- MiniLM model card: [Hugging Face](https://huggingface.co/microsoft/MiniLM-L12-H384-uncased)
- DeBERTa-v3-small model card: [Hugging Face](https://huggingface.co/microsoft/deberta-v3-small)

Primary repo seams referenced:

- `crates/agent-drift-analyzer/src/context/objective.rs`
- `crates/agent-drift-analyzer/src/context/working_set.rs`
- `crates/agent-drift-analyzer/src/context/mod.rs`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `.codex/handoffs/2026-06-13-objective-architecture-decision-dossier.md`
- `.codex/handoffs/2026-06-13-081405-r5-75-objective-failure.md`
