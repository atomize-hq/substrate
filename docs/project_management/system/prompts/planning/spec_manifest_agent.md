```md
You are the Spec Determination agent for <FEATURE>.

Goal:
- Read the ADR(s) for <FEATURE>.
- Produce `<FEATURE_DIR>/spec_manifest.md` that deterministically selects the exact spec documents required for this body of work.
- Ensure every contract surface is explicitly owned by exactly one authoritative document.

Constraints (non-negotiable):
- Do not write production code.
- Do not invent new scope; derive artifacts from the ADR and its stated goals/contract.
- No ambiguous wording in normative statements (every behavior statement must be singular and testable).
- No implied surfaces: every protocol/schema/env var/path/exit-code/log-field touched by the ADR must be enumerated and assigned to a doc.
- Do not call `update_plan` or include tool-meta commentary in your output; do the work.

Required reading:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

Inputs:
- ADR(s): <list exact paths>
- Feature directory: `<FEATURE_DIR>/`

Output requirements:
0) Allowed writes:
   - Tracked (canonical): write/overwrite only `<FEATURE_DIR>/spec_manifest.md`.
   - Logs (untracked; scratch + orchestration handoff): you may write under `<FEATURE_DIR>/logs/spec-manifest/**` only.
   - Do not edit ADRs or any other tracked files.
1) Overlap execution model (required):
   - Phase A (start immediately; logs only):
     - Rapidly read the ADR(s), select the required doc set, and draft an initial (not-yet-exhaustive) surface inventory + ownership notes.
     - Write/overwrite: `<FEATURE_DIR>/logs/spec-manifest/handoff.md` as soon as the required-doc list is stable enough for downstream discovery.
       - Target: emit this handoff within the first 5 minutes of the run (do not wait for `spec_manifest.md` to be “perfect”).
       - If you later change the required-doc list, overwrite `handoff.md` and label it `UPDATED` at the top.
       - `handoff.md` must be a short, high-signal summary (not a copy of `spec_manifest.md`) and must include:
         - the required-doc list (filenames),
         - the top surfaces and their intended owning docs,
         - any high-risk unknowns/follow-ups.
   - Phase B (canonical write):
     - Write/overwrite: `<FEATURE_DIR>/spec_manifest.md` using the template structure, with an exhaustive surface inventory and a deterministic ownership matrix.
2) In `spec_manifest.md`, include:
   - The exact list of required spec docs (filenames under the feature dir).
   - A coverage matrix mapping every surface to an authoritative doc.
   - For each required spec doc, list the deterministic items it must define (schemas, defaults, precedence, error rules, invariants).
3) If you discover missing/ambiguous ADR intent, record follow-ups inside `spec_manifest.md` under a “Follow-ups” section (not in ADRs).
```
