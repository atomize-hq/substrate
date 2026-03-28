Dispatcher context (do not remove):
- Resolved feature dir: `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/`
- Resolved ADR paths:
  - `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/ADR-0001-fixture.md`
- Orchestration mode: `pre_planning_research_orchestrate.sh` overlap run (do not ask the operator to commit/stash/clean; if a Phase B gate is blocked by upstream uncommitted outputs, keep polling — orchestration will commit allowlisted outputs)

Output allowlist (non-negotiable):
- Tracked outputs: (none; wrapper/runner promotes staged candidates)
- Staged tracked-output candidates (write only these under logs):
  - `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/staged/pre-planning/impact_map.md`
- Direct writes to canonical tracked paths are forbidden.
- Logs allowed (untracked only): `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/`
- Do not edit any tracked files directly. If you find follow-ups, record them inside the relevant staged/log output under a "Follow-ups" section.

---

You are the Impact Map agent for impact_map_invalid_staged_orchestrated.

Goal:
- Read the ADR(s) and `spec_manifest.md` for impact_map_invalid_staged_orchestrated.
- Produce `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/pre-planning/impact_map.md` that is exhaustive about:
  - what files/surfaces will be touched (create/edit/deprecate/delete),
  - cascading behavioral/UX implications,
  - contradictions and conflicts with queued/unimplemented work.

Constraints (non-negotiable):
- Do not write production code.
- Do not invent new scope; derive impacts from the ADR + required specs.
- No implied work: every change the ADR implies must appear in the touch set and implication sections.
- If you find multiple viable resolutions to a conflict, record the A/B options and the selected option inside `impact_map.md` (do not edit `decision_register.md` in this single-output run).
- Do not call `update_plan` or include tool-meta commentary in your output; do the work.

Required reading:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/USER_GUIDE.md` (strict pack `task_finish` reads the orchestration worktree copy of `impact_map.md`)

Inputs:
- ADR(s): <list exact paths>
- Feature directory: `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/`
- `spec_manifest.md`: `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/pre-planning/spec_manifest.md`

Discovery requirements (must do):
1) Repo-wide touch discovery:
   - Identify the exact crates/modules/scripts/config/docs that must change to implement the ADR.
   - List paths under `crates/`, `src/`, `scripts/`, `docs/`, and any platform backends (`crates/world*`, `crates/shim`, `crates/shell`, `crates/world-agent`) that are implicated.
2) Cascading UX/behavior analysis:
   - Compare the proposed ADR contract against existing operator workflows to catch contradictions.
   - Call out any UX disjoint (different commands behaving inconsistently, mismatched defaults, conflicting env var semantics).
3) Cross-queue scan:
   - Scan queued/unimplemented ADRs under `docs/project_management/adrs/{draft,queued}/` (and legacy ADR locations).
   - Scan Planning Packs under `docs/project_management/packs/{active,queued,draft}/*/` and `docs/project_management/_archived/*/` for overlapping surfaces.
   - Document overlaps/conflicts and how they are resolved (sequencing boundary, Decision Register, or explicit non-overlap).

Output requirements:
0) Allowed writes:
   - Tracked (canonical): none. Do not write tracked files directly.
   - Staged candidate (logs-only; promoted later by runner/wrapper): write/overwrite only `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/staged/pre-planning/impact_map.md`.
   - Logs (untracked; scratch + orchestration handoff): you may write under `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/**` only.
   - Do not edit ADRs or any other tracked files directly.
1) Overlap execution model (required):
   - Phase A (start immediately; logs only):
     - Perform discovery and draft an initial touch set + implication buckets.
     - Write/overwrite scratch notes at: `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/scratch.md`
       - Target: create `scratch.md` within the first 5 minutes with:
         - an initial Touch Set (directory prefixes are acceptable in Phase A), and
         - the main implication buckets.
     - If present, read upstream handoff notes as an input:
       - `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/spec-manifest/handoff.md`
   - Emit an orchestration handoff signal once Phase A is usable:
     - Write/overwrite: `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/handoff.md`
     - Write it once `scratch.md` contains:
       - a concrete preliminary Touch Set (paths or directory prefixes), and
       - the main implication buckets.
      - Target: emit this handoff within the first 10 minutes (do not wait for cross-queue scan completion).
2) Phase B (staged candidate write gate; required):
   - Before writing `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/staged/pre-planning/impact_map.md`, poll until BOTH are true:
     - `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/spec-manifest/last_message.md` exists, and
     - `git status --porcelain=v1 -- ".tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated"` is empty.
   - Default poll interval: `sleep 60` between checks.
   - If the dispatcher context indicates an orchestration overlap run, **do not** ask the operator to commit/stash/clean upstream outputs; treat a dirty `git status` as transient and keep polling until the gate clears.
3) Then write/overwrite `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/logs/impact-map/staged/pre-planning/impact_map.md` using the template.
   - The Touch Set must be authored for triad execution compatibility, not just planning syntax.
   - The Touch Set must default to exact repo-relative file paths (no vague “update some files”).
   - Strict Touch Set existence rule (non-negotiable):
     - If a path is listed under `### Edit`, `### Deprecate`, or `### Delete`, it MUST exist in the repo at authoring time.
     - If it does not exist, it MUST be moved to `### Create` (if it will be created) or corrected/removed (if it was a guessed path).
     - If you cannot determine the exact file yet, directory-prefix entries are the fallback only:
       - the directory MUST already exist,
       - the token MUST end with `/`,
       - and you MUST record a Follow-up to tighten it to exact file paths later.
4) If you discover a missing surface or ownership gap, record follow-ups inside `impact_map.md` under a “Follow-ups” section (not in ADRs).
5) Closeout validation:
   - Do not write `.tmp_pm_script_tests/run_planning_agent/impact_map_invalid_staged_orchestrated/pre-planning/impact_map.md` directly.
   - The planning runner / wrapper validates the staged candidate before success; direct runs promote it only after that validation passes.
