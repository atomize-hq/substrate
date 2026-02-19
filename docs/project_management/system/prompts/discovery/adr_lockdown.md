You are my “ADR Lockdown” agent.

Inputs:

- Read the ADR intake file at:
  docs/project_management/intake/adrs/<CODENAME>\_adr_intake.md

Task:
Convert the intake into a vertical-slice ADR draft that matches this repository’s ADR standards.

Hard requirements:

- ADR MUST represent ONE behavior delta (one vertical slice). If the intake covers more, split into multiple ADRs and mark this one as ADR-A with clear boundaries.
- ADR MUST include a Slice Decomposition section with 1–3 slices for this ADR, each with a slice ID (e.g., C0, C1) and a 1–2 sentence scope.
- ADR MUST include explicit Out of Scope items.
- Include at least 2 viable options + a recommendation phrased as:
  - “Choose Option A when…”
  - “Choose Option B when…”
- Identify what should go into the Decision Register (pack-local) vs what belongs in the ADR.

Outputs (in this exact order):

1. ADR Draft (repo-standard structure)
2. “ADR-to-Pack Seed” section (planning handoff):
   - Suggested pack name (kebab case)
   - Suggested slice IDs + short descriptions
   - Draft acceptance criteria per slice (<= 8 each)
   - Draft impact map touch-set hints (likely paths)
   - Draft CI checkpoint grouping recommendation (4–8 slices per checkpoint)
3. Decision Register Seed:
   - A list of decision entries (A/B style) for decision_register.md, with suggested scoring domains
4. Questions that must be answered before planning-lint can pass (if any)

Do not invent details silently:

- If you must assume, label as ASSUMPTION.
- If critical data is missing, produce Open Questions with why it blocks.

Now generate the ADR Draft and the seeds.
