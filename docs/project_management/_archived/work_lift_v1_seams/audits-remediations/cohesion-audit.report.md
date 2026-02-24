# Cohesion Audit Report

## Meta
- Generated at: 2026-02-23T01:20:23Z
- Files audited: 28
- Scan used: yes (`cohesion-audit.scan.json`)

## Summary
- Total issues: 4
- By severity: blocker=0, critical=0, major=2, minor=2
- High-confidence cohesion breaks: 4

## Issue index
| ID | Severity | Confidence | Type | Subject | Files |
|---|---|---|---|---|---|
| CH-0001 | major | high | canonicalization_needed | Threaded seam decompositions are not discoverable from the folder entrypoints | work_lift_v1_seams/README.md; work_lift_v1_seams/seam_map.md |
| CH-0002 | major | high | scope_disconnect | SEAM-2 verification mentions goldens “in the rubric” while threaded SEAM-2 scope explicitly forbids editing the rubric | work_lift_v1_seams/seam-2-lift-model-config-v1.md; work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md |
| CH-0003 | minor | high | canonicalization_needed | Some seam briefs reference contract artifacts by basename rather than the canonical CONTRACT paths used elsewhere | work_lift_v1_seams/seam-3-pm-lift-core-engine.md; work_lift_v1_seams/threading.md |
| CH-0004 | minor | high | terminology_drift | Strict-mode invariant list includes a “confirm field name” note even though the canonical field name is already defined in the cited decision log | work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md; WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md |

## Issues

### CH-0001 — Threaded seam decompositions are not discoverable from the folder entrypoints
- Severity: major
- Confidence: high
- Type: canonicalization_needed
- Subject: Threaded seam decompositions are not discoverable from the folder entrypoints
- Locations:
  - `work_lift_v1_seams/README.md:5-10` (primary) — `- Start here: \`scope_brief.md\`` / `- Seam overview: \`seam_map.md\`` / `- Threading ...: \`threading.md\`` (no pointer to `threaded-seams/`).
  - `work_lift_v1_seams/seam_map.md:40-46` (reference) — `## Seam briefs` lists only `seam-*.md` files (no pointer to `threaded-seams/` seam.md + slice docs).
- What breaks: A reader following the documented entrypoints can reach seam briefs and threading, but has no explicit navigation to the detailed threaded decompositions (seam.md + slice-*.md) that contain the executable slice/task breakdowns. This creates a continuity gap where the plan appears to stop at seam briefs unless the reader already knows to inspect `threaded-seams/`.
- Missing links:
  - An explicit pointer from the folder entrypoints (README + seam_map) to the threaded decompositions directory.
  - A clear statement of which level is canonical for execution (seam briefs vs threaded seam slice docs).
- Required to be cohesive:
  - Add a single canonical navigation section that links: `scope_brief` → `seam_map` → `threading` → `threaded-seams/` (and briefly explains what is found in `threaded-seams/`).
  - State the intended reading order and what artifact is considered the “execution plan” for implementers (the slice files).
- Suggested evidence order: docs → codebase → git history → external → decision

### CH-0002 — SEAM-2 verification mentions goldens “in the rubric” while threaded SEAM-2 scope explicitly forbids editing the rubric
- Severity: major
- Confidence: high
- Type: scope_disconnect
- Subject: SEAM-2 verification mentions goldens “in the rubric” while threaded SEAM-2 scope explicitly forbids editing the rubric
- Locations:
  - `work_lift_v1_seams/seam-2-lift-model-config-v1.md:34-36` (primary) — `A “golden” example in the rubric ...`.
  - `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md:9-12` (dependent) — `Editing \`WORK_LIFT_RUBRIC.md\` ... — this seam provides goldens separately in S2.`
- What breaks: SEAM-2’s high-level verification implies that scoring goldens are embedded in the human rubric, but the threaded decomposition explicitly excludes rubric edits and instead places goldens in SEAM-2 S2 deliverables. This disconnect makes it unclear where the canonical golden examples live and which seam owns them, increasing the risk of duplicate or drifting golden sources.
- Missing links:
  - A single canonical location/format for SEAM-2 goldens (doc file or JSON) referenced consistently from both the seam brief and threaded slices.
  - Explicit cross-reference from SEAM-2 brief verification to the threaded S2 goldens deliverable (instead of the rubric).
- Required to be cohesive:
  - Align SEAM-2 verification language so it references the threaded S2 golden deliverable rather than “in the rubric”, or explicitly state the rubric is updated by SEAM-1 and only references/links to goldens.
  - Ensure the seam ownership of the goldens is explicit in one place and referenced everywhere else.
- Suggested evidence order: docs → codebase → git history → external → decision

### CH-0003 — Some seam briefs reference contract artifacts by basename rather than the canonical CONTRACT paths used elsewhere
- Severity: minor
- Confidence: high
- Type: canonicalization_needed
- Subject: Some seam briefs reference contract artifacts by basename rather than the canonical CONTRACT paths used elsewhere
- Locations:
  - `work_lift_v1_seams/seam-3-pm-lift-core-engine.md:8-10` (primary) — references `work_lift_model.v1.json` / `work_lift_vector.schema.json` without the canonical path used elsewhere.
  - `work_lift_v1_seams/threading.md:12-25` (definition) — CONTRACT definitions include `docs/project_management/system/schemas/work_lift_*.json` paths.
- What breaks: Readers relying on the seam briefs may not know which exact files are referenced, especially given multiple similarly named artifacts (schema vs model config) and the explicit contract registry elsewhere. This increases friction for implementers and weakens traceability from seam briefs to CONTRACT definitions.
- Missing links:
  - Seam briefs consistently referencing CONTRACT IDs and/or canonical paths when they mention contract artifacts.
  - A consistent convention (CONTRACT ID first, file path second) for all seam briefs.
- Required to be cohesive:
  - Update seam briefs to reference artifacts as CONTRACT IDs (e.g., CONTRACT-1/2) and include the canonical path when needed, matching `threading.md`.
  - Ensure the folder entrypoints explain that CONTRACT IDs in `threading.md` are the canonical reference for paths and consumers.
- Suggested evidence order: docs → codebase → git history → external → decision

### CH-0004 — Strict-mode invariant list includes a “confirm field name” note even though the canonical field name is already defined in the cited decision log
- Severity: minor
- Confidence: high
- Type: terminology_drift
- Subject: Strict-mode invariant list includes a “confirm field name” note even though the canonical field name is already defined in the cited decision log
- Locations:
  - `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md:47-50` (primary) — `contract.behavior_deltas == 1` followed by “confirm actual field name”.
  - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:147-152` (definition) — canonical Lift Vector includes `contract.behavior_deltas` (nested under `contract`).
- What breaks: The strict-mode plan introduces uncertainty about a key invariant’s field name even though the canonical field name is already specified in the decision log that these docs cite as source-of-truth. This weakens the glossary/entity map and makes readers second-guess whether the invariant can be implemented without additional discovery.
- Missing links:
  - A direct pointer from the strict-mode invariant list to the canonical Lift Vector field definition (D6) for the invariant fields used in strict checks.
  - Removal of uncertainty language where the canonical contract already specifies the name.
- Required to be cohesive:
  - Replace “confirm actual field name” with a direct reference to the canonical field name in D6 (and/or CONTRACT-1 schema once published).
  - If the invariant is intended to be derived rather than sourced from the vector, explicitly state where it lives (`vector` vs `derived`) in CONTRACT-3 terms.
- Suggested evidence order: docs → codebase → git history → external → decision

## Audited files
- work_lift_v1_seams/README.md
- work_lift_v1_seams/scope_brief.md
- work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md
- work_lift_v1_seams/seam-2-lift-model-config-v1.md
- work_lift_v1_seams/seam-3-pm-lift-core-engine.md
- work_lift_v1_seams/seam-4-pack-derived-lift-inputs.md
- work_lift_v1_seams/seam-5-advisory-workflow-integration.md
- work_lift_v1_seams/seam_map.md
- work_lift_v1_seams/threading.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/seam.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-2-human-rubric-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/seam.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-2-goldens-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/seam.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-2-config-backed-scoring-and-validation.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-3-goldens-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/seam.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-2-prefix-expansion-and-derived-counts.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/seam.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-2-advisory-report-hook-and-lint-integration.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md
- WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md
- .codex/handoffs/2026-02-22-175724-workstream-triage-lift.md
