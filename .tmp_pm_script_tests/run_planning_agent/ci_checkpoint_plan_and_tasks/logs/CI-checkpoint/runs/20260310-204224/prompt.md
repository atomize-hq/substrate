Dispatcher context (do not remove):
- Resolved feature dir: `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/`
- Resolved ADR paths:
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/ADR-0001-fixture.md`

Output allowlist (non-negotiable):
- Tracked outputs: (none; wrapper/runner promotes staged candidates)
- Staged tracked-output candidates (write only these under logs):
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md`
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/staged/tasks.json`
- Direct writes to canonical tracked paths are forbidden.
- Logs allowed (untracked only): `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/`
- Do not edit any tracked files directly. If you find follow-ups, record them inside the relevant staged/log output under a "Follow-ups" section.

---

You are the CI Checkpoint Planning agent for ci_checkpoint_plan_and_tasks.

Goal:
- Produce a **pre-planning first pass** CI checkpoint plan at:
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/ci_checkpoint_plan.md`
- Ensure `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` reflects the required baseline for this pack:
  - schema v4 (automation + cross-platform),
  - dynamic platform scope fields (set conservatively when uncertain).

Constraints (non-negotiable):
- Do not write production code.
- Do not edit ADR files.
- Do not invent new scope; derive checkpoint boundaries from `impact_map.md`, `spec_manifest.md`, and existing plan intent.
- Do not call `update_plan` or include tool-meta commentary in your output; do the work.

Required reading:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`
- `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json`
- `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/impact_map.md`
- `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/spec_manifest.md`
- `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/minimal_spec_draft.md`
- `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/ci_checkpoint_plan.md` (if it already exists)

Allowed writes:
- Tracked (canonical): none. Do not write tracked files directly.
- Staged candidates (logs-only; promoted later by runner/wrapper): you may write/overwrite only:
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md`
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/staged/tasks.json` (only if required to satisfy schema-v4 cross-platform planning requirements)
- Logs (untracked; scratch + orchestration handoff): you may write under `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/**` only.
- Do not edit any other tracked files directly.

Preflight (required; do first):
1) Read `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` and ensure the pack baseline is set for pre-planning:
   - `meta.schema_version = 4`
   - `meta.automation.enabled = true` and `meta.automation.orchestration_branch` is a non-empty string
   - `meta.cross_platform = true`
2) Ensure platform scope fields are present (dynamic per pack; be conservative when uncertain):
   - `meta.ci_parity_platforms_required` (default: `["linux","macos","windows"]`)
   - `meta.behavior_platforms_required` (default: same as ci_parity)
   - If `spec_manifest.md` / `minimal_spec_draft.md` explicitly scopes the behavior delta to a subset of platforms (e.g., Linux-only behavior change), set `meta.behavior_platforms_required` to that subset while keeping `meta.ci_parity_platforms_required` unchanged unless explicitly justified.
3) If any of the above is missing or wrong, prepare a staged candidate for `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` containing only those field changes.
   - Overlap note: in orchestration overlap runs, Phase A is logs-only; if the Phase B gate has not cleared yet, record required `tasks.json` edits in scratch and write the staged candidate only after the Phase B gate clears.

Overlap execution model (required):
- Phase A (start immediately; logs only):
  - Draft checkpoint grouping and gates as scratch:
    - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/scratch.md`
  - Emit an orchestration handoff signal once you have a usable checkpoint outline:
    - Write/overwrite: `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/handoff.md`
    - Timing target (required):
      - Emit the initial `handoff.md` within the first 5 minutes of the run (do not wait for `impact_map.md` / `minimal_spec_draft.md` to be “perfect”).
      - If canonical inputs are not ready yet, base the handoff on upstream handoff/scratch artifacts (e.g., `logs/spec-manifest/handoff.md`, `logs/impact-map/handoff.md`, `logs/min-spec-draft/handoff.md`) and clearly label assumptions as `DRAFT`.
      - If you later change checkpoint grouping, gates, or slice IDs, overwrite `handoff.md` and label it `UPDATED` at the top.
    - Include:
      - proposed checkpoint groups (slice ranges),
      - proposed checkpoint task ids (e.g., `CP1-ci-checkpoint`),
      - the gates to run at each checkpoint (compile parity / smoke / CI testing).
- Phase B (staged candidate write gate; required):
  - Before writing staged candidates, poll until BOTH are true:
    - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/min-spec-draft/last_message.md` exists, and
    - `git status --porcelain=v1 -- ".tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks"` is empty.
  - Default poll interval: `sleep 60` between checks.
  - If the dispatcher context indicates an orchestration overlap run, **do not** ask the operator to commit/stash/clean upstream outputs; treat a dirty `git status` as transient and keep polling until the gate clears.
  - After the gate clears, re-read `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/impact_map.md` (not just upstream handoff/scratch artifacts) before writing the staged candidates.

Tracked output requirements (pre-planning first pass; required):
1) Write/overwrite `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md` using the template:
   - `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`
2) Slice-awareness rule:
   - If `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` already defines slice integration tasks (`*-integ`), then:
     - Make the machine-readable JSON slices list match the real slice ids and group them contiguously.
     - For schema v4 cross-platform packs, update `meta.checkpoint_boundaries` to match the checkpoint boundaries.
     - Validate mechanically (must pass):
       - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir ".tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks"`
   - If `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` does NOT yet define slices, then:
     - Still write a useful first-pass plan:
       - decide which gates to run at checkpoints (compile parity / feature smoke / CI testing mode),
       - decide whether feature-smoke is required at every checkpoint or only at “risk seams” based on `impact_map.md`,
       - prefer the slice ids from `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/minimal_spec_draft.md` `## Draft slice skeleton (pre-planning only)` when populating the machine-readable JSON `checkpoints[].slices` list,
         - treat these as **draft** slice ids (may split/merge),
         - do not claim mechanical validity yet (until `tasks.json` slice tasks exist),
       - if the draft slice skeleton is missing, use placeholder slice ids only as placeholders (make that explicit in the rationale),
       - record follow-ups for full planning to replace placeholders with real slice ids + wiring.
     - Do NOT run `validate_ci_checkpoint_plan.py` (it will fail without real slice tasks).

Follow-ups:
- If the pack lacks enough information to choose code-grounded boundaries, record follow-ups in:
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/ci_checkpoint_plan.md` under a “Follow-ups” section, and
  - `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/logs/CI-checkpoint/scratch.md` (evidence and rationale).

Follow-up checklist for making this plan mechanically valid (required when slices are created):
- Ensure slice ids in `tasks.json` match the draft slice skeleton (or update both to the accepted ids).
- Replace any remaining placeholder slice ids in the plan’s machine-readable JSON with real slice ids.
- Set `tasks.json` `meta.checkpoint_boundaries` to match checkpoint boundaries.
- Add the `CP1-ci-checkpoint` task with correct dependencies (per this plan).
- Then run (must pass):
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir ".tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks"`

Closeout validation:
- Do not write `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/pre-planning/ci_checkpoint_plan.md` or `.tmp_pm_script_tests/run_planning_agent/ci_checkpoint_plan_and_tasks/tasks.json` directly.
- The planning runner / wrapper will promote the staged candidate(s) into the canonical tracked path(s) and run any required validation after promotion.
