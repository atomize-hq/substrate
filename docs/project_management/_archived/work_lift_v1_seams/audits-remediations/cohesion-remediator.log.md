# Cohesion Remediation Log (Work Lift v1 seam docs)

Date: 2026-02-23

Input report: `cohesion-audit.report.json`

Scope: `work_lift_v1_seams/**` plus `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (as referenced authority).

## Triage

- **CH-0001 (major, canonicalization_needed)** → Bucket **B/F** (cross-doc canonicalization + navigation continuity)
- **CH-0002 (major, scope_disconnect)** → Bucket **D** (seam continuity repair)
- **CH-0003 (minor, canonicalization_needed)** → Bucket **B/E** (canonical contract references + traceability)
- **CH-0004 (minor, terminology_drift)** → Bucket **E** (terminology + canonical field mapping)

## Remediations

### CH-0001 — Fixed

- **Cohesion break**: Entry points lead to seam briefs/threading, but do not point readers to the threaded decompositions that contain the executable slice/task breakdowns.
- **Remediation pattern**: Canonicalize navigation + add explicit execution-plan entry point.
- **Evidence used**:
  - `work_lift_v1_seams/README.md:8` (entrypoint list lacked `threaded-seams/`) and new reading order at `work_lift_v1_seams/README.md:13`.  
  - `work_lift_v1_seams/seam_map.md:40` (seam briefs listed) and new execution-plan pointer at `work_lift_v1_seams/seam_map.md:48`.
- **Doc changes**:
  - Added a canonical “Reading order” section and an explicit “Execution plan” pointer in `work_lift_v1_seams/README.md`.
  - Added a new index doc at `work_lift_v1_seams/threaded-seams/README.md` that explains structure and links to per-seam `seam.md`.
  - Added an explicit “Threaded decompositions (execution plan)” section to `work_lift_v1_seams/seam_map.md`.
- **Result**: A reader can traverse scope → seam map → threading → threaded decompositions without needing to discover `threaded-seams/` by browsing.

### CH-0002 — Fixed

- **Cohesion break**: SEAM-2 brief verification referenced a “golden” example “in the rubric”, but the threaded SEAM-2 decomposition explicitly keeps rubric edits in SEAM-1 and produces goldens as a SEAM-2 deliverable.
- **Remediation pattern**: Align brief-level verification to threaded deliverables; make ownership explicit.
- **Evidence used**:
  - SEAM-2 brief verification: `work_lift_v1_seams/seam-2-lift-model-config-v1.md:34`.
  - Threaded SEAM-2 S2 scope/ownership: `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-2-goldens-and-conformance.md:1` (goldens produced by SEAM-2; rubric edits explicitly out).
- **Doc changes**:
  - Updated SEAM-2 brief verification to reference the threaded S2 “golden cases” deliverable and clarify that SEAM-1 owns rubric edits: `work_lift_v1_seams/seam-2-lift-model-config-v1.md`.
- **Result**: Goldens have a single seam owner (SEAM-2) and a single referenced deliverable; the rubric is no longer implied as the canonical golden source for SEAM-2.

### CH-0003 — Fixed

- **Cohesion break**: Some seam briefs referenced contract artifacts by basename, while `threading.md` is the canonical contract registry with contract IDs + paths.
- **Remediation pattern**: Canonicalize contract references using `CONTRACT-*` IDs (ID first, path second).
- **Evidence used**:
  - Canonical contract registry: `work_lift_v1_seams/threading.md:7` (CONTRACT-1/2 definitions with canonical paths).
  - Brief-level basename usage: `work_lift_v1_seams/seam-3-pm-lift-core-engine.md:8`.
- **Doc changes**:
  - Updated SEAM-3 scope/inputs to reference `CONTRACT-1:work_lift_vector_block_v1` and `CONTRACT-2:work_lift_model_v1` with canonical paths: `work_lift_v1_seams/seam-3-pm-lift-core-engine.md`.
  - Added an explicit note in the directory entrypoint that `threading.md` is the canonical contract registry: `work_lift_v1_seams/README.md`.
- **Result**: A reader can map “schema/config” mentions in seam briefs directly to the contract registry without ambiguity.

### CH-0004 — Fixed

- **Cohesion break**: Strict-mode invariant list included “confirm actual field name” despite an existing canonical field definition in the cited decision log.
- **Remediation pattern**: Remove uncertainty language; link invariants to canonical field path and source-of-truth.
- **Evidence used**:
  - Candidate invariant with uncertainty: `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md:47`.
  - Canonical field definition: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md:147` (Lift Vector v1 fields; `contract.behavior_deltas`).
- **Doc changes**:
  - Replaced the ambiguous invariant with an explicit `CONTRACT-3`-aligned field path and a direct pointer to D6: `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md`.
- **Result**: Strict-mode invariants now reference a concrete, already-defined field name and location, eliminating unnecessary discovery.

## Decisions introduced

None required (all changes grounded in existing docs).
