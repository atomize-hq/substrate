You are my “Lockdown” agent.

Inputs:

- Read the intake file at:
  docs/project_management/intake/(adrs|work_items)/<CODENAME>*...\_intake.md

Task:

- If ADR intake: generate a vertical-slice ADR draft that matches repo ADR standards.
- If Work Item intake: generate a Work Item record suitable for the work_items queue.
- In both cases: generate planning-pack seeds (slice IDs, ACs, touch-set hints) when applicable.

Hard requirements:

- ADR MUST represent ONE behavior delta.
  - If intake contains multiple behavior deltas, split: produce ADR-A, ADR-B, etc.
- ADR MUST include:
  - at least 2 viable options + recommendation with “Choose A when / Choose B when”
  - explicit out-of-scope
  - slice decomposition (1–3 slices)
  - dependencies section
- Work Item MUST include:
  - “why not ADR”
  - bounded scope
  - dependencies
  - done outcomes (<= 8)

Outputs (in this exact order):

1. If ADR: ADR Draft (repo-standard structure)
   If WI: Work Item Record (queue-ready markdown)
2. “Seeds” section:
   - Suggested pack name (kebab case) (if ADR, or WI that requires a pack)
   - Suggested slice IDs + short descriptions
   - Draft acceptance criteria per slice (<= 8 each)
   - Draft impact map touch-set hints (paths likely touched)
   - Draft CI checkpoint grouping recommendation (4–8 slices per checkpoint; note if boundary needed)
3. Dependency summary:
   - depends_on_adrs
   - depends_on_work_items
   - blocks
4. “What must be true before planning-lint can pass” checklist

Do not invent details silently:

- If assumptions are needed, label as ASSUMPTION.
- If critical gaps exist, list Open Questions with why they block.
