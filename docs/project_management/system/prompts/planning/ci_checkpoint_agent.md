```md
You are the CI Checkpoint Planning agent for <FEATURE>.

Goal:
- Produce a **pre-planning first pass** CI checkpoint plan at:
  - `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md`
- Ensure `<FEATURE_DIR>/tasks.json` reflects the required baseline for this pack:
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
- `<FEATURE_DIR>/tasks.json`
- `<FEATURE_DIR>/pre-planning/impact_map.md`
- `<FEATURE_DIR>/pre-planning/spec_manifest.md`
- `<FEATURE_DIR>/pre-planning/minimal_spec_draft.md`
- `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md` (if it already exists)

Allowed writes:
- Tracked (canonical): none. Do not write tracked files directly.
- Staged candidates (logs-only; promoted later by runner/wrapper): you may write/overwrite only:
  - `<FEATURE_DIR>/logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md`
  - `<FEATURE_DIR>/logs/CI-checkpoint/staged/tasks.json` (only if required to satisfy schema-v4 cross-platform planning requirements)
- Logs (untracked; scratch + orchestration handoff): you may write under `<FEATURE_DIR>/logs/CI-checkpoint/**` only.
- Do not edit any other tracked files directly.

Runner-injected phase directive (authoritative when present):
<!-- PM_PHASE_DIRECTIVE:BEGIN -->
- Default if no runner-injected directive is present: `single` mode.
- `single` mode:
  - Complete the full prompt in one run.
  - Produce the required Phase A log artifacts first, then produce the staged candidate(s) in the same run.
  - Do not wait for `last_message.md`, canonical tracked files to appear, or git cleanliness. If a canonical upstream artifact is unavailable, use the best available canonical/log inputs, record the gap as a follow-up, and proceed.
- `phase_a` mode:
  - Produce only the Phase A logs/scratch/handoff artifacts listed below, then stop.
  - Do not write staged candidates.
- `phase_b` mode:
  - Assume upstream authoritative inputs are ready.
  - Re-read the canonical tracked inputs listed in this prompt before writing the staged candidate(s).
  - Write the staged candidate(s) immediately.
  - Do not wait for `last_message.md`, canonical tracked files to appear, or git cleanliness.
<!-- PM_PHASE_DIRECTIVE:END -->

Preflight (required; do first):
1) Read `<FEATURE_DIR>/tasks.json` and ensure the pack baseline is set for pre-planning:
   - `meta.schema_version = 4`
   - `meta.automation.enabled = true` and `meta.automation.orchestration_branch` is a non-empty string
   - `meta.cross_platform = true`
2) Ensure platform scope fields are present (dynamic per pack; be conservative when uncertain):
   - `meta.ci_parity_platforms_required` (default: `["linux","macos","windows"]`)
   - `meta.behavior_platforms_required` (default: same as ci_parity)
   - If `spec_manifest.md` / `minimal_spec_draft.md` explicitly scopes the behavior delta to a subset of platforms (e.g., Linux-only behavior change), set `meta.behavior_platforms_required` to that subset while keeping `meta.ci_parity_platforms_required` unchanged unless explicitly justified.
3) If any of the above is missing or wrong, prepare a staged candidate for `<FEATURE_DIR>/tasks.json` containing only those field changes.
   - Overlap note: in orchestration overlap runs, Phase A is logs-only; record required `tasks.json` edits in scratch during Phase A and write the staged candidate only in `phase_b` (or `single`) mode.

Overlap execution model (required):
- Phase A (start immediately; logs only):
  - Draft checkpoint grouping and gates as scratch:
    - `<FEATURE_DIR>/logs/CI-checkpoint/scratch.md`
  - Emit an orchestration handoff signal once you have a usable checkpoint outline:
    - Write/overwrite: `<FEATURE_DIR>/logs/CI-checkpoint/handoff.md`
    - Timing target (required):
      - Emit the initial `handoff.md` within the first 5 minutes of the run (do not wait for `impact_map.md` / `minimal_spec_draft.md` to be “perfect”).
      - If canonical inputs are not ready yet, base the handoff on upstream handoff/scratch artifacts (e.g., `logs/spec-manifest/handoff.md`, `logs/impact-map/handoff.md`, `logs/min-spec-draft/handoff.md`) and clearly label assumptions as `DRAFT`.
      - If you later change checkpoint grouping, gates, or slice IDs, overwrite `handoff.md` and label it `UPDATED` at the top.
    - Include:
      - proposed checkpoint groups (slice ranges),
      - proposed checkpoint task ids (e.g., `CP1-ci-checkpoint`),
      - the gates to run at each checkpoint (compile parity / smoke / CI testing).
- Phase B (staged candidate write; required):
  - Re-read `<FEATURE_DIR>/pre-planning/impact_map.md`, `<FEATURE_DIR>/pre-planning/spec_manifest.md`, and `<FEATURE_DIR>/pre-planning/minimal_spec_draft.md` when they are available canonically; otherwise use the best available upstream handoff/scratch artifacts and record the gap in Follow-ups.
  - Reconcile the checkpoint boundaries and any staged `tasks.json` edits against those authoritative inputs before writing the staged candidates.

Tracked output requirements (pre-planning first pass; required):
1) Write/overwrite `<FEATURE_DIR>/logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md` using the template:
   - `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`
2) Slice-awareness rule:
   - If `<FEATURE_DIR>/tasks.json` already defines slice integration tasks (`*-integ`), then:
     - Make the machine-readable JSON slices list match the real slice ids and group them contiguously.
     - For schema v4 cross-platform packs, update `meta.checkpoint_boundaries` to match the checkpoint boundaries.
     - Validate mechanically (must pass):
       - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"`
   - If `<FEATURE_DIR>/tasks.json` does NOT yet define slices, then:
     - Still write a useful first-pass plan:
       - decide which gates to run at checkpoints (compile parity / feature smoke / CI testing mode),
       - decide whether feature-smoke is required at every checkpoint or only at “risk seams” based on `impact_map.md`,
       - prefer the slice ids from `<FEATURE_DIR>/pre-planning/minimal_spec_draft.md` `## Draft slice skeleton (pre-planning only)` when populating the machine-readable JSON `checkpoints[].slices` list,
         - treat these as **draft** slice ids (may split/merge),
         - do not claim mechanical validity yet (until `tasks.json` slice tasks exist),
       - if the draft slice skeleton is missing, use placeholder slice ids only as placeholders (make that explicit in the rationale),
       - record follow-ups for full planning to replace placeholders with real slice ids + wiring.
     - Do NOT run `validate_ci_checkpoint_plan.py` (it will fail without real slice tasks).

Follow-ups:
- If the pack lacks enough information to choose code-grounded boundaries, record follow-ups in:
  - `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md` under a “Follow-ups” section, and
  - `<FEATURE_DIR>/logs/CI-checkpoint/scratch.md` (evidence and rationale).

Follow-up checklist for making this plan mechanically valid (required when slices are created):
- Ensure slice ids in `tasks.json` match the draft slice skeleton (or update both to the accepted ids).
- Replace any remaining placeholder slice ids in the plan’s machine-readable JSON with real slice ids.
- Set `tasks.json` `meta.checkpoint_boundaries` to match checkpoint boundaries.
- Add the `CP1-ci-checkpoint` task with correct dependencies (per this plan).
- Then run (must pass):
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"`

Closeout validation:
- Do not write `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md` or `<FEATURE_DIR>/tasks.json` directly.
- The planning runner / wrapper will promote the staged candidate(s) into the canonical tracked path(s) and run any required validation after promotion.
```
