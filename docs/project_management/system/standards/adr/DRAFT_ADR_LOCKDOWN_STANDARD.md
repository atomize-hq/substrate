# Draft ADR Lockdown Standard

This standard defines the **minimum, scoped** rules for producing a **Draft ADR** from an ADR intake during the “Lockdown” phase.

Scope boundary:

- This standard is intentionally limited to authoring the **ADR document** itself.
- It does **not** instruct the agent to create a Planning Pack, run pre-planning, generate spec manifests, generate impact maps, or run other planning automation.

## Key rule: “Draft ADR” is a full ADR

A **Draft ADR** must be a **complete ADR document** using the normal ADR structure and required sections.  
“Draft” only means:

- `Status: Draft` (not `Accepted`), and
- the feature directory path references the **draft pack bucket** (`docs/project_management/packs/draft/<feature>/`) until the ADR is accepted.

Do not omit required ADR sections because the ADR is a draft.

## Draft vs Accepted pack bucket rule (non-negotiable)

- Draft ADRs should reference: `docs/project_management/packs/draft/<feature>/`
- Accepted ADRs must reference: `docs/project_management/packs/active/<feature>/`

## Lockdown requirements (ADR drafts only)

An ADR Draft produced during lockdown must:

- Represent **exactly one** user-visible behavior delta.
  - If intake implies multiple behavior deltas, split into multiple ADR drafts (ADR-A, ADR-B, …).
- Include **at least two** viable options (A/B) with explicit tradeoffs and risks.
- Include a recommendation with an explicit chooser rubric:
  - “Choose A when … / Choose B when …”
- Include explicit non-goals / out-of-scope constraints.
- Include a slice decomposition of **1–3 slices** (high level; not a full task graph).
- Include explicit dependencies and blockers.

## Rigor bar (what “good” looks like)

The ADR draft must be written as a **deterministic contract**, not a brainstorm.

Minimum rigor expectations:
- Every contract surface implied by the intake must appear in `## User Contract (Authoritative)` with explicit defaults and precedence (no “etc.”).
- If the ADR changes CLI/config/paths/exit codes, the ADR must enumerate the exact names/paths/defaults and the failure posture.
- If the ADR introduces ambiguity that blocks acceptance, record it explicitly as open questions/follow-ups (with why it blocks).
- The A/B options must be meaningfully different, with explicit tradeoffs and a chooser rubric (“Choose A when / Choose B when”).

## Required ADR format (Draft)

Start from the canonical ADR template:

- `docs/project_management/system/templates/adr/ADR_TEMPLATE.md`

Do not omit template sections or invent a “draft-only” ADR shape. Draft ADRs must preserve the full template structure.

Draft-only adjustments you must make to the template:
- Set `Status: Draft`.
- Set the feature directory path(s) to `docs/project_management/packs/draft/<feature>/` (not `active`).
- In `Related Docs (links only)`, use `docs/project_management/packs/draft/<feature>/...` paths (even if the files do not exist yet).
- Do not create any Planning Pack files as part of lockdown; `Related Docs` must be links only.
- In the Executive Summary, `ADR_BODY_SHA256` may remain a placeholder; do not run extra workflow steps unless explicitly instructed elsewhere.

## Normative strength rules

- Avoid ambiguous normative language in contract statements (`should`, `could`, `might`, `maybe`).
- If something is uncertain, record it explicitly as an open question or follow-up, and explain why it blocks acceptance (if it does).
