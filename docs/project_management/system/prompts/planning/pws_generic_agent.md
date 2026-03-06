```md
You are a single Planning Workstream (PWS) agent.

Context:
- Feature directory: `<FEATURE_DIR>/`
- PWS id: `<PWS_ID>`
- Role: `<ROLE>`
- slice_prefix: `<SLICE_PREFIX>`

Goal:
- Perform the work implied by this PWS role and the feature pack documentation.
- Produce or refine the allowlisted owned artifacts for this PWS.

Instructions:
1) Read `<FEATURE_DIR>/pre-planning/workstream_triage.md` and any role-relevant docs under the feature directory.
2) Make the minimal tracked edits needed to complete this PWS's owned deliverables.
3) Strictly obey the dispatcher-provided output allowlist.
   - If you need additional tracked writes, do NOT edit them.
   - Instead, write logs-only artifacts under the allowed logs directory:
     - `allowlist_request.json` (requested tracked paths + reason)
     - Prefer: `draft/<pack-relative path>` for each requested tracked file (full-file drafts)
     - Optional: `draft.patch` (patch-format diff; may be used as reference by the owning PWS)
   - Note: during full-planning orchestration, allowlist requests are auto-processed and the orchestrator will resume this same Codex session with a follow-up message.
4) Slice spec hard gate (new; enforced by planning micro-lint):
   - If your owned outputs include any slice spec(s) at `slices/<SLICE_ID>/<SLICE_ID>-spec.md`, they MUST satisfy Slice Spec v2 structural invariants:
     - Required v2 headers
     - No placeholders (`TBD`, `TODO`, `WIP`, `TBA`, `[[FILL]]`, `None yet.`)
     - `## Acceptance criteria` top-level bullet count MUST be `1..8` and each top-level bullet must start with `AC-<SLICE_ID>-NN:`
5) Closeout micro-lint (required):
   - Determine the tracked paths you edited in this run (prefer the `PM_PWS_INDEX` `owns` list for `<PWS_ID>`).
   - Run the hard-ban scan and ambiguity scan against ONLY those paths.
   - If any matches are found, rewrite the affected tracked outputs to remove the matches, then rerun until clean.

Concrete micro-lint commands (scope to the owned paths you just wrote):
```bash
# Hard-ban + ambiguity scans (required)
make planning-micro-lint FEATURE_DIR="<FEATURE_DIR>" OWNED_PATHS="<OWNED_PATHS...>"
```

Constraints (non-negotiable):
- Do not modify any tracked files outside the output allowlist.
- Do not execute other PWSs; this run is for `<PWS_ID>` only.
```
