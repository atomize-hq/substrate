### S1 — Advisory workflow docs + Makefile entry points

- **User/system value**: Make lift computation easy to run and consistently interpreted during triage and Planning Pack refinement, without changing default behavior of existing workflows.
- **Scope (in/out)**:
  - In:
    - Document “where lift fits” in the planning workflow (advisory step).
    - Document how to run `pm_lift` for:
      - intake/ADR markdown (`from-intake`),
      - Planning Pack (`from-impact-map`),
      - git diff calibration (`from-git-diff`).
    - Add stable Makefile targets that call into the planning scripts (advisory only).
  - Out:
    - Hard enforcement gates.
    - Requiring lift blocks for legacy packs.
- **Acceptance criteria**:
  - A new/updated planning standard doc exists that answers:
    - what lift is (and is not),
    - how to run it for each context,
    - how to interpret score vs triggers vs confidence vs missing inputs.
  - Makefile targets:
    - are discoverable (`make help` or documented section),
    - are opt-in (not run by default in `make lint` unless explicitly requested),
    - pass through script arguments cleanly (path to intake/pack, `--emit-json`, etc).
  - The doc references (but does not duplicate) canonical policy decisions in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`.
- **Dependencies**:
  - Consumes: `CONTRACT-3:pm_lift_emit_json_v1` (documented output expectations), `CONTRACT-4:impact_map_emit_json_v1` (pack-derived path)
  - References: `CONTRACT-1` (Lift Vector block) and `CONTRACT-2` (model config versioning)
- **Verification**:
  - Run commands in the doc against:
    - one intake/ADR markdown file with a valid lift block,
    - one Planning Pack containing `impact_map.md`,
    - one small git diff example.
  - Confirm:
    - commands succeed in advisory mode,
    - output is readable and highlights triggers + missing inputs.

#### S1.T1 — Write “Work Lift in the planning workflow” doc (advisory-first)

- **Outcome**: A single place to learn how to use lift during triage/planning.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_v1_seams/scope_brief.md`
    - `work_lift_v1_seams/threading.md` (contracts + dependency edges)
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (reference-only)
  - Outputs (pinned):
    - Add `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`.
    - Edit `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md` to link to the advisory doc.
    - Edit `docs/project_management/system/standards/planning/PLANNING_README.md` to link to the advisory doc.
  - **Implementation notes**:
    - Include three “how to run” recipes:
      - intake/ADR: `python3 docs/project_management/system/scripts/planning/pm_lift.py from-intake --intake <path> [--emit-json]`
      - Planning Pack: `python3 docs/project_management/system/scripts/planning/pm_lift.py from-impact-map --feature-dir <pack_dir> [--emit-json]`
      - calibration: `python3 docs/project_management/system/scripts/planning/pm_lift.py from-git-diff --git-range <base>..<head> [--emit-json]`
    - Include a short interpretation guide:
      - score is advisory,
      - triggers are the primary split signal,
      - confidence/missing-inputs explain uncertainty.
- **Acceptance criteria**:
  - A reader can run lift in <5 minutes without spelunking scripts.

Checklist:
- Implement:
  - Add/Update doc and add cross-links from existing planning docs.
  - Use advisory-first language; explicitly call out strict gating is deferred.
- Validate:
  - Confirm links resolve; avoid duplicating policy text.
- Cleanup:
  - Keep examples small; prefer “commands + expected fields” over full JSON dumps.

#### S1.T2 — Add Makefile targets for advisory lift runs

- **Outcome**: Quick, consistent entry points for common lift runs.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/scripts/planning/pm_lift.py`
  - Outputs:
    - `Makefile` targets, e.g.:
      - `pm-lift-intake` (requires `PATH`/python; takes `FILE=...`)
      - `pm-lift-pack` (takes `PACK=...`)
      - `pm-lift-diff` (takes `BASE=...` and `HEAD=...`)
- **Implementation notes**:
  - Default to human-readable output; allow `EMIT_JSON=1` to pass `--emit-json`.
  - Keep targets “do nothing by default” unless required vars are set (print usage and exit 2).
- **Acceptance criteria**:
  - Targets run without modifying any other planning behavior.

Checklist:
- Implement:
  - Add targets and document usage in the new doc (S1.T1).
- Validate:
  - Run each target once (happy path + missing var usage message).
- Cleanup:
  - Keep target names stable and consistent with existing Makefile conventions.

#### S1.T3 — Add a short “happy path” walkthrough section (docs-only)

- **Outcome**: A minimal tutorial that proves the workflow is coherent.
- **Inputs/outputs**:
  - Inputs:
    - Planning docs from S1.T1
  - Outputs:
    - A short “walkthrough” section with:
      - prerequisites,
      - 3 commands (intake, pack, diff),
      - “what to look for” bullets.
- **Acceptance criteria**:
  - A new contributor can reproduce the lift workflow reliably.

Checklist:
- Implement:
  - Keep it short and operational.
- Validate:
  - Confirm it doesn’t assume strict-mode defaults.
- Cleanup:
  - Avoid embedding large outputs (link to `--emit-json` field list instead).
