# SEAM-5 — Advisory-first workflow integration (+ strict-mode onramp) (threaded decomposition)

## Seam Brief (Restated)

- **Seam ID**: SEAM-5
- **Name**: Work Lift in the planning workflow
- **Goal / user value**: Make lift computation a standard advisory step in intake/triage and Planning Pack refinement, without breaking legacy packs or prematurely enforcing uncalibrated rules.
- **Type**: platform
- **Scope**
  - In:
    - Document “how to run” `pm_lift` for:
      - intake/ADR markdown (`from-intake`),
      - Planning Pack (`from-impact-map`),
      - post-implementation diff (`from-git-diff`, calibration).
    - Add lightweight integration points:
      - Makefile target(s) for running lift (advisory),
      - optional lint/report output in existing planning scripts (non-fatal by default).
    - Define a strict-mode onramp plan:
      - gating keyed off `tasks.json meta.slice_spec_version >= 2`,
      - initial candidate invariants to enforce after calibration.
  - Out:
    - Turning split triggers into hard errors immediately.
    - Requiring lift blocks for all legacy packs.
- **Key invariants / rules**
  - Advisory-first: lift computation MUST inform, not block, until explicitly promoted via an explicit strict-mode opt-in.
  - Legacy compatibility: strict requirements MUST be gated and opt-in.
  - Avoid “single-number overfitting”: emphasize triggers + missing inputs + confidence over score precision.
- **Touch surface**
  - `Makefile`
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md` (new)
  - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md` (new; opt-in)
  - `docs/project_management/system/standards/planning/PLANNING_README.md` (edit: add links)
  - `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md` (edit: add links)
  - `docs/project_management/system/scripts/planning/lint.{sh,ps1}` (optional non-fatal hook only; avoid changing `pm_lift.py` behavior here)
  - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (reference only; avoid duplicating canonical rules)
- **Verification**
  - Happy path walkthrough:
    - compute lift for an intake/ADR markdown file,
    - compute lift for a Planning Pack,
    - confirm outputs are readable and actionable.
  - Failure path walkthrough:
    - invalid Lift Vector block produces actionable error with location hints.
  - Confirm no legacy scripts change behavior unless explicitly invoked.
- **Threading constraints**
  - Upstream blockers:
    - SEAM-3 (`CONTRACT-3:pm_lift_emit_json_v1`) for stable result output semantics (`--emit-json`) used by any wrappers/hooks.
    - SEAM-4 (`CONTRACT-4:impact_map_emit_json_v1`) for deterministic pack-derived lift inputs.
  - Contracts produced (owned):
    - None (this seam integrates and documents; it does not define new shared contracts).
  - Contracts consumed:
    - `CONTRACT-1:work_lift_vector_block_v1` (Lift Vector markers + schema, for intake/ADR usage docs)
    - `CONTRACT-2:work_lift_model_v1` (model config versioning, referenced by workflow docs)
    - `CONTRACT-3:pm_lift_emit_json_v1` (workflow integration and any wrapper checks)
    - `CONTRACT-4:impact_map_emit_json_v1` (Planning Pack `impact_map.md` path, referenced by workflow docs)

## Slice index

- `S1` → `slice-1-advisory-workflow-docs-and-make-targets.md`: publish “how to run” and add stable Makefile entry points (advisory-only)
- `S2` → `slice-2-advisory-report-hook-and-lint-integration.md`: add an optional, non-fatal report/hook surface that consumes `--emit-json` (no enforcement)
- `S3` → `slice-3-strict-mode-onramp-plan.md`: define the strict-mode promotion plan and gating rules (without enabling by default)

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - None.
- **Contracts consumed**:
  - `CONTRACT-3:pm_lift_emit_json_v1` (SEAM-3)
    - Consumed by: S2 (report/hook parsing and invariant checks in strict gating *only when opted-in*).
  - `CONTRACT-4:impact_map_emit_json_v1` (SEAM-4)
    - Consumed by: S1 (Planning Pack “from-impact-map” workflow guidance) and S2 (optional hook runs in packs).
  - `CONTRACT-1:work_lift_vector_block_v1` (SEAM-1) and `CONTRACT-2:work_lift_model_v1` (SEAM-2)
    - Consumed by: S1 (intake/ADR authoring guidance, “what lift means” doc links).
- **Dependency edges honored**:
  - `SEAM-3 blocks SEAM-5`: S2/S3 depend on a stable `--emit-json` contract and exit-code expectations.
  - `SEAM-4 blocks SEAM-5`: S1/S2 depend on deterministic pack-derived inputs for Planning Pack workflows.
- **Parallelization notes**:
  - What can proceed now:
    - S1 docs + Makefile scaffolding can land early (advisory-only, clear “requires pm_lift v1” notes).
    - S3 can land as a plan/spec (no enforcement enabled).
  - What must coordinate:
    - Any wrapper/hook that parses JSON must track `CONTRACT-3` (additive keys only; never depend on debug-only fields unless documented as stable).
