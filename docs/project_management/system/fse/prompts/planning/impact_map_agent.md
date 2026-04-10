```md
You are the Impact Map agent for <FEATURE>.

Goal:
- Read the ADR(s) and `spec_manifest.md` for <FEATURE>.
- Produce `<FEATURE_DIR>/pre-planning/impact_map.md` that is exhaustive about:
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
- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/USER_GUIDE.md` (strict pack `task_finish` reads the orchestration worktree copy of `impact_map.md`)

Inputs:
- ADR(s): <list exact paths>
- Feature directory: `<FEATURE_DIR>/`
- `spec_manifest.md`: `<FEATURE_DIR>/pre-planning/spec_manifest.md`

Runner-injected phase directive (authoritative when present):
<!-- PM_PHASE_DIRECTIVE:BEGIN -->
- Default if no runner-injected directive is present: `single` mode.
- `single` mode:
  - Complete the full prompt in one run.
  - Produce the required Phase A log artifacts first, then produce the staged candidate in the same run.
  - Do not wait for `last_message.md`, canonical tracked files to appear, or git cleanliness. If a canonical upstream artifact is unavailable, use the ADR(s) plus any available handoff/scratch artifacts, record the gap as a follow-up, and proceed.
- `phase_a` mode:
  - Produce only the Phase A logs/scratch/handoff artifacts listed below, then stop.
  - Do not write staged candidates.
- `phase_b` mode:
  - Assume upstream authoritative inputs are ready.
  - Re-read the canonical tracked inputs listed in this prompt before writing the staged candidate.
  - Write the staged candidate immediately.
  - Do not wait for `last_message.md`, canonical tracked files to appear, or git cleanliness.
<!-- PM_PHASE_DIRECTIVE:END -->

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
   - Staged candidate (logs-only; promoted later by runner/wrapper): write/overwrite only `<FEATURE_DIR>/logs/impact-map/staged/pre-planning/impact_map.md`.
   - Logs (untracked; scratch + orchestration handoff): you may write under `<FEATURE_DIR>/logs/impact-map/**` only.
   - Do not edit ADRs or any other tracked files directly.
1) Overlap execution model (required):
   - Phase A (start immediately; logs only):
     - Perform discovery and draft an initial touch set + implication buckets.
     - Write/overwrite scratch notes at: `<FEATURE_DIR>/logs/impact-map/scratch.md`
       - Target: create `scratch.md` within the first 5 minutes with:
         - an initial Touch Set (directory prefixes are acceptable in Phase A), and
         - the main implication buckets.
     - If present, read upstream handoff notes as an input:
       - `<FEATURE_DIR>/logs/spec-manifest/handoff.md`
   - Emit an orchestration handoff signal once Phase A is usable:
     - Write/overwrite: `<FEATURE_DIR>/logs/impact-map/handoff.md`
     - Write it once `scratch.md` contains:
       - a concrete preliminary Touch Set (paths or directory prefixes), and
       - the main implication buckets.
      - Target: emit this handoff within the first 10 minutes (do not wait for cross-queue scan completion).
2) Phase B (staged candidate write; required):
   - Re-read `<FEATURE_DIR>/pre-planning/spec_manifest.md` when it is available canonically; otherwise use the best available upstream handoff/scratch artifacts and record the gap in Follow-ups.
3) Then write/overwrite `<FEATURE_DIR>/logs/impact-map/staged/pre-planning/impact_map.md` using the template.
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
5) Closeout micro-lint (required for `single` and `phase_b` runs):
   - After writing the staged candidate, run the hard-ban scan and ambiguity scan against the staged file before ending the run:
     - `make planning-micro-lint FEATURE_DIR="<FEATURE_DIR>" OWNED_PATHS="logs/impact-map/staged/pre-planning/impact_map.md"`
   - If the scan fails, fix the staged candidate and rerun the command until it passes.
5) Closeout validation:
   - Do not write `<FEATURE_DIR>/pre-planning/impact_map.md` directly.
   - The planning runner / wrapper validates the staged candidate before success; direct runs promote it only after that validation passes.
```
