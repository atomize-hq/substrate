# Planning Spec Determination Standard

This standard inserts a mandatory step between:
- drafting or refining an ADR,
- and performing downstream FSE planning or decomposition.

Goal:
- Given an ADR, deterministically derive the exact document set required to make the work plannable with zero ambiguity about contracts, protocols, schemas, invariants, platform scope, and downstream ownership.

This step exists to prevent a common failure mode:
- the ADR describes intent, but the pack never pins down which documents must own each contract surface, leaving implied behavior that later drifts.

## When to run this step

Run this immediately after an ADR is drafted, before downstream planning or decomposition begins.

Recommended ordering:
1. Draft or refine the ADR.
2. Produce `spec_manifest.md`.
3. Produce `impact_map.md`.
4. Iterate the ADR until it no longer implies undocumented behavior.
5. Use the resulting pre-planning artifacts as the basis for downstream FSE planning or decomposition.

## Required output

Create:
- `docs/project_management/packs/<bucket>/<feature>/pre-planning/spec_manifest.md`

This file is authoritative for:
- which documents must exist for the feature,
- which contract surfaces each document owns,
- which downstream docs remain to be authored after pre-planning.

Scaffolding:
- use `docs/project_management/system/fse/templates/planning_pack/spec_manifest.md.tmpl`

## Rules

1) `spec_manifest.md` must leave no implicit surfaces.
   - Every externally visible surface touched by the ADR must appear in the coverage matrix and be assigned to one authoritative doc.

2) Each surface has exactly one authoritative doc.
   - Other docs may link to that truth, but must not redefine it.

3) Absence semantics are mandatory.
   - If a field, flag, env var, path, or mode may be absent, behavior when absent must be explicit and testable.

4) Determinism beats flexibility.
   - If the ADR appears to permit multiple behaviors, collapse it into one deterministic contract or record a crisp follow-up.

5) Pre-planning does not create execution wiring.
   - Do not require `plan.md`, `tasks.json`, kickoff prompts, or execution ownership artifacts as mandatory outputs of this step.
   - Do not require pre-full-planning convergence, post-full-planning reconcile, or other legacy full-planning follow-up artifacts as mandatory outputs of this step.

## Document catalog

`spec_manifest.md` is produced by selecting from this catalog. Selection must follow the ADR, not personal preference.

Always create in the pre-planning lane:
- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/workstream_triage.md`
- `pre-planning/alignment_report.md`

Create `pre-planning/ci_checkpoint_plan.md` when:
- the work has cross-platform behavior,
- the verification cadence is expensive,
- or the feature introduces obvious risk seams that warrant explicit checkpoint intent.

Do not add legacy convergence or reconcile surfaces to the pre-planning catalog.
If those surfaces still exist elsewhere in the repo, treat them as compatibility scaffolding outside the supported pre-planning contract.

Create downstream topic-focused specs when the ADR introduces or changes any of the following:

1) User-facing contract surface
   - create `contract.md`
   - must define commands, flags, defaults, exit-code meanings, path semantics, and precedence

2) Wire or API protocol surface
   - create `<topic>-protocol-spec.md`
   - must define request and response shapes, error model, ordering, retries, and versioning

3) Data schema or file format surface
   - create `<topic>-schema-spec.md`
   - must define full schema, constraints, canonicalization, defaults, and compatibility posture

4) Environment-variable surface
   - create `env-vars-spec.md` or fold into `contract.md` when the scope is small
   - must define name, type, allowed values, default, precedence, and redaction implications

5) Policy surface
   - create `policy-spec.md`
   - must define evaluation inputs, precedence, default posture, and audit expectations

6) Telemetry or observability surface
   - create `telemetry-spec.md`
   - must define field names, types, redaction rules, and consumer impact

7) Filesystem semantics
   - create `filesystem-semantics-spec.md`
   - must define path invariants, failure modes, resolution rules, and platform differences

8) Platform parity or divergence
   - create `platform-parity-spec.md`
   - must define guarantees, allowed divergence, and required validation evidence

9) Compatibility or rollout surface
   - create `compatibility-spec.md`
   - must define compatibility policy, end condition, and validation evidence

10) Downstream decomposition candidates
   - when the ADR clearly implies multiple seams, record draft seam or slice-candidate docs in the manifest
   - these are candidate downstream docs, not execution tasks

## Required structure for `spec_manifest.md`

Use:
- `docs/project_management/system/fse/templates/planning_pack/spec_manifest.md.tmpl`

`spec_manifest.md` must include:
1. ADR inputs and feature directory
2. Concrete list of exact docs to create under the feature directory
3. Coverage matrix mapping each surface to its authoritative doc
4. Determinism checklist for each selected doc
5. Clear distinction between:
   - docs authored during pre-planning,
   - docs deferred to downstream FSE planning or decomposition

## Prompt

Canonical prompt:
- `docs/project_management/system/fse/prompts/planning/spec_manifest_agent.md`
