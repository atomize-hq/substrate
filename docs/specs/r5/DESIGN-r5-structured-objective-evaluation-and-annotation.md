# Design: R5 Structured Objective Evaluation And Annotation

Status: draft design note created on 2026-06-13 from the objective-architecture dossier, the live
WDAP failure evidence, and the cited research sources.

## Why This Doc Exists

The structured-objective design is not useful unless the repo can tell whether it is actually more
honest than the current string-first system. The dossier explicitly called for a small gold
benchmark built from real prompts. This doc defines that benchmark and annotation protocol.

## Design Goal

Create a durable evaluation wall for objective understanding that:

1. captures the WDAP kickoff failure family,
2. captures neighboring failure families where long prompt structure matters,
3. scores both extraction quality and grounding quality,
4. stays small enough to maintain by hand,
5. can evaluate heuristic-only and classifier-assisted implementations with the same protocol.

## Research Patterns This Design Borrows

### Structured output over ad hoc summary

`Instruct and Extract` treats extraction as returning structured outputs aligned to user requests.
Design implication: evaluation should score field extraction, not only whether a final summary
string looks plausible.

### Schema-driven generalization

`SGD` and `D3ST` both emphasize dynamic schemas described in natural language rather than one fixed
internal ontology. Design implication: the benchmark should be defined by field descriptions and
annotation rules, not by fragile implementation-specific enum names.

### Grounding/localization

`LMDX` highlights that extraction without localization/grounding is not enough. Design implication:
each annotated field in the benchmark should point back to row/section/clause evidence.

### Compositional semantics

`MTOP` shows the limits of a simple intent+slot paradigm for compositional requests. Design
implication: benchmark cases must include nested and compositional asks, not just short single-line
imperatives.

## Corpus Families

The first benchmark should cover a small number of prompt families with high semantic value.

### Family A: WDAP kickoff prompts

These are the primary motivating failures.

Required properties:

- one long user kickoff row,
- `## Scope` or equivalent mission section,
- subordinate checklist or operational section later in the same row,
- known failure where a subordinate line beats the mission.

Primary seed artifacts:

- `/Users/spensermcconnell/.codex/sessions/2026/03/10/rollout-2026-03-10T20-40-30-019cda56-8bbf-7ed2-9de3-2db5f65e24d7.jsonl`
- `/Users/spensermcconnell/.codex/sessions/2026/03/10/rollout-2026-03-10T20-41-00-019cda57-012e-7073-a736-6eda6f2ba615.jsonl`

Add the named `R5.75` linux/macOS smoke adaptations as locked benchmark seeds first. Adapted
external prompt families are secondary stretch coverage, not the primary acceptance anchor.

### Family B: Deliberate boilerplate-target prompts

These are preserved-target cases where the user is actually asking to analyze or edit the
instruction surface itself.

Required properties:

- `AGENTS.md`, `<skill>`, `Available skills`, or equivalent instruction payload is present,
- the real task is about that surface,
- shortening must not erase the real target.

### Family C: Short explicit objective prompts

These are control cases.

Required properties:

- concise `/goal` or equivalent task line,
- little or no competing scaffolding,
- the structured system should preserve the obvious answer without overcomplication.

### Family D: Replan / steer pivots

These cases prove that objective state can change across turns.

Required properties:

- later steer or user prompt changes the active task,
- benchmark labels should show what fields persist vs what fields reset.

### Family E: Review / closeout / no-code prompts

These cases ensure that the structured frame captures semantics such as:

- review-only,
- validate-only,
- no-code-change expectations,
- proof or verification emphasis.

### Family F: Planning / research / docs prompts

These cases ensure the taxonomy can distinguish non-implementation task intent while still
extracting target, constraints, and deliverables.

## Annotation Unit

The benchmark should annotate at four linked levels:

1. **row**
   - which rows are objective-bearing candidates,
2. **section**
   - scope, checklist, verification, context, boilerplate, etc.,
3. **clause / sentence**
   - the unit for role labeling,
4. **frame**
   - the final structured objective expected from the prompt.

This layered annotation is necessary because the failure is often not row selection alone.

## Annotation Schema

### Clause-level labels

Each clause or sentence should be labeled with:

- `goal`
- `constraint`
- `verification`
- `context`
- `other_role`

### Request-level labels

Each prompt should also have:

- `objective_class`
- `primary_intent`
- `target`
- `deliverables`
- `success_conditions`
- `semantic_constraints`
- `verification_commands`
- `unknowns`

### Grounding labels

Each extracted field should carry:

- supporting row ref,
- supporting source kind,
- supporting section kind,
- supporting clause span,
- annotator confidence.

## Canonical Annotation Rules

1. annotate the smallest span that supports the field,
2. do not promote checklist steps to `goal` when a broader mission span exists,
3. treat tool-choice instructions as execution metadata, not semantic constraints,
4. mark fields unknown when evidence is weak rather than guessing,
5. allow a clause to support more than one field only when both uses are explicit and justified,
6. record forbidden role promotions explicitly, not only forbidden final objective strings.

A phase-1 extractor passes when weakly supported fields stay unknown with evidence explaining why.
It fails when it fabricates target, constraint, deliverable, or verification fields from weak spans.

## Expected Output Shape

A benchmark manifest should be easy for both humans and tests to inspect.

Recommended repo shape:

```text
crates/agent-drift-analyzer/tests/objective_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/<case-id>/raw.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/<case-id>/expected.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/<case-id>/raw.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/<case-id>/expected.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/<case-id>/raw.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/<case-id>/expected.json
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/<case-id>/notes.md
```

Recommended `expected.json` shape:

```json
{
  "case_id": "wdap_linux_scope_vs_checklist_01",
  "family": "wdap_kickoff",
  "source_rollout_id": "019cda56-8bbf-7ed2-9de3-2db5f65e24d7",
  "objective_class": "task_statement",
  "primary_intent": "validate",
  "target": {
    "display": "ensure the slice is green for linux",
    "kind": "repo_slice"
  },
  "semantic_constraints": [
    {
      "display": "linux-specific scope",
      "constraint_kind": "platform_boundary"
    }
  ],
  "deliverables": [],
  "success_conditions": [
    {
      "display": "slice is green for linux"
    }
  ],
  "verification_commands": [],
  "role_spans": [
    {
      "source_kind": "user_prompt",
      "section_kind": "mission",
      "role": "goal",
      "excerpt": "Ensure the slice is green for linux..."
    }
  ],
  "field_evidence": {
    "target": ["source:user_prompt/row:0/section:mission/clause:0"],
    "success_conditions": ["source:user_prompt/row:0/section:mission/clause:0"]
  },
  "forbidden_objective_strings": [
    "Run this task on a linux machine."
  ],
  "forbidden_role_promotions": [
    {
      "excerpt": "Run this task on a linux machine.",
      "from_source_kind": "user_prompt",
      "from_section_kind": "checklist",
      "forbidden_role": "goal"
    }
  ],
  "compatibility_rendering": {
    "acceptable_any_of": [
      "Ensure the slice is green for linux.",
      "Validate the slice is green for linux."
    ],
    "comparison_key": "validate|repo_slice|linux|green"
  },
  "unknowns": [],
  "notes": "checklist line is subordinate, not canonical mission"
}
```

Compatibility rendering may be acceptable even when structured extraction is richer than the old
string-first output. Tests should not overfit to one exact pretty string when the structured frame
is clearly correct.

## Data Split Strategy

The first benchmark does not need a large ML-style split. It needs a reviewable design split.

Recommended partition:

- **design-set**: a handful of cases used to refine the schema and rules,
- **locked acceptance set**: cases not edited during rule tuning,
- **stretch external**: optional external/adapted cases used for robustness only.

When/if classifier work begins later, these can become train/dev/test splits.

## Metrics

### Required metrics

1. **field exactness**
   - was the correct structured field populated,
2. **role accuracy**
   - were goal/constraint/verification/context spans labeled correctly,
3. **grounding accuracy**
   - did the extracted field cite the right supporting span,
4. **compatibility rendering accuracy**
   - does the legacy objective string remain acceptable for current consumers,
5. **forbidden-promotion rate**
   - how often does a subordinate checklist or boilerplate line incorrectly become canonical task
     truth,
6. **unknown-field correctness**
   - were weakly supported fields left unknown instead of guessed.

### Acceptance summary table

Each scorer run should emit at least:

- `cases_total`
- `field_exact_pass`
- `grounding_pass`
- `forbidden_promotion_failures`
- `compatibility_rendering_failures`
- `unknown_field_correctness`

### Optional metrics

- confidence calibration,
- per-family error rate,
- human review time per case.

## Acceptance Gates

A future implementation should not be considered successful unless it passes all of the following:

1. WDAP linux and macOS cases stop promoting the subordinate checklist line,
2. preserved boilerplate-target cases remain intact,
3. concise `/goal` cases do not regress,
4. review/no-code cases preserve their semantic boundaries,
5. extracted fields are grounded to evidence spans,
6. unknowns are used instead of fabricated values when the prompt is genuinely ambiguous.

## Non-Goals

This design does not define:

- the final Rust schema implementation,
- the final classifier training pipeline,
- the full downstream migration order,
- the final pretty-string rendering policy beyond acceptance compatibility.

## Source Map

Primary local authority used for this design:

- `.codex/handoffs/2026-06-13-objective-architecture-decision-dossier.md`
- `.codex/handoffs/2026-06-13-081405-r5-75-objective-failure.md`
- `/Users/spensermcconnell/.codex/sessions/2026/03/10/rollout-2026-03-10T20-40-30-019cda56-8bbf-7ed2-9de3-2db5f65e24d7.jsonl`
- `/Users/spensermcconnell/.codex/sessions/2026/03/10/rollout-2026-03-10T20-41-00-019cda57-012e-7073-a736-6eda6f2ba615.jsonl`
