# Design: R5 Structured Objective Classifier Taxonomy

Status: draft design note created on 2026-06-13 from the objective-architecture dossier and the
cited classifier / dialogue-state research sources.

## Why This Doc Exists

The dossier identified a likely future classifier front-end, but also explicitly warned against
collapsing several prediction problems into one head. This doc defines the classifier taxonomy,
model matrix, and evaluation philosophy for that future work.

This is a design note only. It does not commit the repo to classifier implementation now.

## Core Decision

The classifier front-end, if added, should be a **cascaded helper** for the structured objective
pipeline, not the sole owner of objective semantics.

Its job is to improve routing and role labeling quality under low-data constraints. It is not a
replacement for grounding, evidence spans, or the structured sidecar.

## Guardrail Invariants

1. a classifier cannot populate ungrounded structured fields,
2. classifier confidence cannot exceed the confidence of the supporting evidence spans,
3. classifier outputs are hints to the assembler, not direct `StructuredObjective` DTO fields,
4. the final assembler remains evidence-grounded and deterministic about what becomes canonical
   structured state.

## Research Patterns This Design Borrows

### Intent and slot/state remain foundational

`BERT for Joint Intent Classification and Slot Filling` reinforces the usefulness of intent plus
fielded state as a semantic frame. We borrow that framing, but not the assumption that the entire
problem is a single flat sentence-level task.

### Dynamic schema descriptions improve transfer

`SGD` and `D3ST` both show that using natural-language schema descriptions improves generalization
and zero-shot transfer to unseen services or tasks. We borrow that directly: every label family in
this design should be documented in natural language, not only in code enums.

### Simple intent-slot labeling is insufficient for compositional asks

`MTOP` motivates keeping richer structure for compositional requests. Repo prompts often combine
mission, constraints, verification, and deliverables; therefore the classifier stack cannot stop at
"one intent + one slot list".

### Low-data sentence-embedding approaches are legitimate baselines

`SetFit` provides a prompt-free, data-efficient baseline using contrastive fine-tuning of sentence
transformers followed by a classification head. We borrow that as an explicit low-data benchmark,
not as a mandatory production path.

## Cascaded Head Design

### Head A: Task-statement gate

Purpose:

- decide whether a candidate row/section/clause is task-bearing or not.

Recommended labels:

- `task_statement`
- `not_task_statement`

Granularity:

- row, section, or clause candidate depending on stage input.

Why separate it:

- it is a gating problem,
- it has different class balance and evaluation than intent classification,
- a good gate can reduce noise for later heads.

### Head B: Primary task intent

Purpose:

- classify the main request-level intent after gating.

Recommended labels:

- `implement`
- `debug`
- `review`
- `research`
- `plan`
- `validate`
- `docs`
- `other_task`

Granularity:

- request or prompt level.

Why separate it:

- it is a prompt-level semantic routing problem,
- it should not be forced to operate at clause granularity,
- it should be interpretable independently from role extraction.

### Head C: Clause / section role labeling

Purpose:

- assign semantic role to clause- or sentence-level units.

Recommended labels:

- `goal`
- `constraint`
- `verification`
- `context`
- `other_role`

Granularity:

- clause or sentence, not whole utterance.

Why separate it:

- it works at a different unit than Head B,
- it directly feeds structured field assembly,
- it must support multiple labeled spans inside one prompt.

## Why The Heads Must Stay Separate

A single joint head would mix:

- binary gating,
- request-level intent,
- clause-level semantic role.

That collapse is operationally awkward, harder to annotate cleanly, and directly contradicts the
dossier's design decision. The better design is a cascade with explicit interfaces between stages.

## Interface Contract Between Heads

Recommended flow:

```text
candidate text units
  -> Head A keeps likely task-bearing units
  -> Head B classifies prompt-level primary intent
  -> Head C labels clause/sentence roles
  -> rule-based or learned assembler builds StructuredObjective
```

Head outputs should be advisory signals that help the final assembler, not direct replacements for
source-grounded extraction.

## Model Matrix

### Primary recommendation: MiniLM-L12-H384

Why first:

- relatively small footprint,
- strong NLU tradeoff for size,
- model card shows a 33M-parameter model with competitive GLUE/SQuAD dev results,
- a practical first prototype for local experimentation.

Recommended use:

- Head A baseline,
- Head B primary baseline,
- optional Head C baseline if clause-level data volume is sufficient.

### Second recommendation: DeBERTa-v3-small

Why second:

- stronger small-model follow-up candidate,
- model card suggests better upside than ultra-tiny models when nuance matters,
- reasonable comparator once MiniLM establishes the low-cost baseline.

Recommended use:

- stronger follow-up baseline for Head B,
- likely better candidate than MiniLM for harder Head C role classification if annotation volume
  becomes adequate.

### Low-data baseline: SetFit + MiniLM (or another sentence transformer body)

Why include it:

- low-data-friendly,
- prompt-free,
- cheap and fast to benchmark,
- useful for Heads A and B when labeled data is initially scarce.

Recommended use:

- benchmark against fine-tuned MiniLM for small labeled sets,
- especially useful before enough clause-level examples exist for heavier experiments.

### Optional Prototype 0: Tiny binary gate

Allowed models:

- `task-classifier-mini-v3`
- `bert-tiny`

Purpose:

- a cheap front-of-pipeline binary filter,
- weak-labeling helper,
- ablation experiment.

Important constraint:

- Prototype 0 is optional and distinct from the broader model recommendation.
- Tiny binary models must not become the default main semantic-routing model.

## Training Philosophy

1. build the annotation benchmark first,
2. evaluate simple heuristic baselines first,
3. train Head A and Head B before treating Head C as mandatory,
4. keep the classifier advisory until the grounding/evidence pipeline is trustworthy,
5. prefer data-efficient baselines before larger experiments.

## Minimum Data Thresholds

Do not begin model experiments until at least these floor conditions are met:

- Head B prompt-level intent training: at least **50 manually labeled request-level cases**,
- Head C clause-role training: at least **200 manually labeled clause-level spans**,
- every model experiment: a locked heuristic baseline from the objective acceptance wall already
  exists.

These are minimum entry conditions, not claims that the dataset is sufficient for production.

## Evaluation Philosophy

Each head should be evaluated on its own task:

- Head A: precision/recall/F1 on task-bearing detection,
- Head B: intent accuracy and per-class confusion,
- Head C: span/role accuracy at clause level,
- end-to-end: impact on final structured-objective accuracy and forbidden-promotion rate.

A classifier that improves its local score but worsens end-to-end structured extraction should not be
considered a success.

Every experiment must compare against:

1. a deterministic heuristic baseline,
2. the current structured-objective acceptance suite,
3. forbidden-promotion error rate.

## Data Requirements By Head

### Head A

Smallest data need. Can start with manually labeled row/section examples.

### Head B

Moderate data need. Should start once the evaluation corpus has stable prompt-level intent labels.

### Head C

Highest annotation burden. Should wait until the clause-level benchmark protocol is locked.

## Non-Goals

This design does not do any of the following yet:

- choose final training hyperparameters,
- add model dependencies to the repo,
- define the exact serialized model interface,
- replace heuristic decomposition entirely,
- allow classifier predictions to bypass evidence-grounded assembly.

## Source Map

Primary sources used for this design:

- BERT for Joint Intent Classification and Slot Filling: [arXiv 1902.10909](https://arxiv.org/abs/1902.10909)
- Schema-Guided Dialogue Dataset: [arXiv 1909.05855](https://arxiv.org/abs/1909.05855)
- Description-Driven Task-Oriented Dialog Modeling: [arXiv 2201.08904](https://arxiv.org/abs/2201.08904)
- MTOP: [arXiv 2008.09335](https://arxiv.org/abs/2008.09335)
- SetFit: [arXiv 2209.11055](https://arxiv.org/abs/2209.11055)
- MiniLM model card: [Hugging Face](https://huggingface.co/microsoft/MiniLM-L12-H384-uncased)
- DeBERTa-v3-small model card: [Hugging Face](https://huggingface.co/microsoft/deberta-v3-small)
