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
6. **Phase 1 must be deterministic and dependency-free.** No classifier or LLM is required for the
   first structured-objective slice.
7. **Compatibility rendering is downstream-only.** The legacy string is a view over structured
   state, not the authority that structured fields merely decorate.

## R5.75 Stopgap Status

`R5.75-1` objective condensation remains useful only as a stopgap compatibility filter and
front-end candidate reducer.

It is **not** the target architecture and must **not** be extended into another growing ladder of
imperative-prefix or boilerplate-substring heuristics. The repo should not continue investing in
`normalized_objective_text(...)` as if it were the durable abstraction boundary.

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

pub struct ObjectiveTarget {
    pub display: String,
    pub kind: ObjectiveTargetKind,
    pub paths: Vec<String>,
    pub symbols: Vec<String>,
    pub named_artifacts: Vec<String>,
    pub workspace_refs: Vec<String>,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

pub enum ObjectiveTargetKind {
    RepoSlice,
    CrateOrPackage,
    FileOrDirectory,
    SpecOrDesignDoc,
    TestOrVerifier,
    SkillOrInstructionSurface,
    ExternalArtifact,
    ConceptualTopic,
    UnknownTarget,
}

pub struct ObjectiveConstraint {
    pub display: String,
    pub constraint_kind: ObjectiveConstraintKind,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

pub enum ObjectiveConstraintKind {
    ScopeBoundary,
    NoCode,
    DocsOnly,
    ReviewOnly,
    ValidateOnly,
    PlatformBoundary,
    DeliverableFormat,
    OtherConstraint,
}

pub struct SuccessCondition {
    pub display: String,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

pub struct RequestedDeliverable {
    pub display: String,
    pub deliverable_kind: RequestedDeliverableKind,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
    pub confidence: Confidence,
}

pub enum RequestedDeliverableKind {
    CodeChange,
    DesignDoc,
    Plan,
    Review,
    ValidationReport,
    ResearchSummary,
    OtherDeliverable,
}

pub enum ObjectiveSourceKind {
    ThreadGoal,
    UserPrompt,
    AssistantContext,
    SystemInstruction,
    ToolOutput,
    UnknownSource,
}

pub enum ObjectiveSectionKind {
    Scope,
    Mission,
    Checklist,
    Verification,
    Constraints,
    Deliverables,
    Context,
    Boilerplate,
    ToolingInstructions,
    UnknownSection,
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
    pub source_kind: ObjectiveSourceKind,
    pub section_kind: ObjectiveSectionKind,
    pub role: ObjectiveRole,
    pub excerpt: String,
    pub start_char: Option<usize>,
    pub end_char: Option<usize>,
    pub confidence: Confidence,
}

pub struct ObjectiveUnknown {
    pub field_name: String,
    pub reason: String,
    pub evidence: Vec<ObjectiveEvidenceSpan>,
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
- deliverables
- context / motivation
- metadata / boilerplate
- tooling instructions

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
supporting clause or an explicit deterministic inference rule.

### Stage 6: Compatibility rendering

Render legacy compatibility fields only after the structured frame exists. Compatibility text is a
projection of structured state, not the source of truth.

## Frame Assembly Precedence

The staged pipeline also needs explicit conflict-resolution rules.

### Goal precedence

1. explicit `/goal` or accepted thread goal,
2. mission / scope section goal,
3. direct imperative user ask near the beginning or end of a prompt,
4. checklist item only if no broader mission / scope goal exists,
5. boilerplate or instruction line only if the user explicitly asks to analyze or edit that surface.

### Verification precedence

1. explicit verification section or command request,
2. success condition phrased as proof requirement,
3. tool-choice or environment instruction only as execution metadata, never as verification by
   default.

### Constraint precedence

1. scope limits, no-code/docs-only, platform-specific task boundary,
2. deliverable format requirements,
3. tool/environment preference only if it changes semantic task meaning.

These precedence rules are required so the WDAP class of failure is visible as a section-hierarchy
mistake rather than hidden inside generic text normalization.

## Grounding Rules

Following the grounding/localization lesson from `LMDX`, the new system should obey these rules:

1. every nontrivial field must cite one or more source spans,
2. fields without good evidence should stay unknown,
3. the same clause may support multiple fields, but each usage should be explicit,
4. display text and reasoning evidence must stay linked,
5. source kind and section kind must be preserved so scope-vs-checklist and user-vs-boilerplate conflicts are auditable.

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
    pub comparison_key: String,
    pub structured: Option<StructuredObjective>,
    pub verification_commands: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
}

pub struct TaskFrame {
    pub objective: String,
    pub objective_key: String,
    pub structured_objective: Option<StructuredObjective>,
    // existing fields remain during migration
}
```

Rules:

- `ObjectiveSummary.text` is the human-readable compatibility rendering.
- `ObjectiveSummary.comparison_key` is the deterministic semantic comparison key derived from the
  structured frame.
- `TaskFrame.objective` remains the public display string during migration.
- Task-frame and progress comparability should migrate to `comparison_key` first, then to richer
  structured fields.

## Unknowns Are Success, Not Failure

A phase-1 extractor passes when it leaves weakly supported fields unknown and preserves the evidence
showing why.

It fails when it fabricates target, constraint, deliverable, or verification fields from weak spans
just to look complete.

## Determinism And Phase-1 Scope

Phase 1 should be deterministic, dependency-free, and auditable.

That means:

1. no classifier is required to produce the first sidecar,
2. no LLM call belongs in analyzer-time extraction,
3. the first implementation should succeed or fail by evidence-grounded deterministic rules,
4. classifier experiments are blocked on the objective acceptance wall defined in the evaluation
   design.

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
- it still leaves downstream modules dependent on a lossy representation,
- it invites more prompt-shape-specific heuristic ladders instead of a typed architecture.

That section is a hard design boundary, not merely a preference.

## Non-Goals

This architecture doc does not commit the repo to:

- immediate model training,
- immediate checkpoint schema versioning,
- rewriting checkpoint boundary logic in the same slice,
- collapsing execution metadata into semantic state,
- replacing all string consumers in one packet,
- using a classifier or LLM in the analyzer before the objective acceptance wall exists.

## Open Design Questions

The major phase-1 shape questions are now intentionally narrowed. Remaining open questions are:

1. which deterministic clause-segmentation helper is simplest without overfitting,
2. how aggressively `comparison_key` should normalize synonymous phrasing in early migration,
3. which downstream predicate family after `working_set` should migrate first once coexistence is
   stable.

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
