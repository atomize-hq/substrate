# Planning Impact Map Standard (ADR → Impact Map)

This standard inserts a mandatory step between:
- producing `spec_manifest.md`, and
- running the full Planning Pack authoring workflow in `docs/project_management/standards/PLANNING_README.md`.

This step replaces the legacy `integration_map.md`.

Goal:
- Produce a deterministic, cross-cutting **impact map** that:
  - enumerates the exact file/module surfaces that will be touched (create/edit/deprecate/delete),
  - identifies cascading behavioral/UX implications and contradiction risks across the product,
  - checks alignment/conflicts with queued/unimplemented ADRs and Planning Packs.

---

## When to run this step

Run this immediately after `spec_manifest.md` exists for the feature:
1) ADR drafted
2) `spec_manifest.md` created (ADR → required specs)
3) `impact_map.md` created (ADR + spec_manifest → global impacts)
4) ADR iterated/accepted
5) Planning Pack produced (plan/tasks/specs/kickoffs/etc.)

---

## Required output

Create:
- `docs/project_management/next/<feature>/impact_map.md`

Scaffolding:
- `make planning-new-feature FEATURE=<feature>` creates `impact_map.md` from `docs/project_management/standards/templates/impact_map.md.tmpl`.

---

## Rules (non-negotiable)

1) **Touch set is explicit**
   - Every expected file creation/edit/deprecation/removal must be listed (best-effort, but exhaustive for contract-bearing files).
   - Use concrete paths (repo-relative) and name the responsible component/crate.

2) **Cascading implications are explicit**
   - For every user-facing change (CLI/config/exit codes/paths/provisioning flows), state:
     - direct impact,
     - second-order impact (what else must change to keep the experience coherent),
     - “contradiction risks” (where existing semantics would conflict).

3) **Cross-queue alignment is required**
   - You must identify relevant ADRs and Planning Packs that are not implemented yet and document:
     - whether they overlap the same surfaces,
     - whether they conflict,
     - how the conflict is resolved (Decision Register, sequencing, or explicit non-overlap boundary).

4) **No implied work**
   - If the ADR implies a change that is not represented in the touch set and implications sections, the impact map is incomplete.

---

## Inputs

The impact map is derived from:
- ADR(s) (draft or accepted)
- `spec_manifest.md` (authoritative spec selection + ownership map)
- The repo’s current behavior and contracts (code + docs)
- Queued/unimplemented ADRs and Planning Packs

---

## Where ADRs live (for cross-queue discovery)

Preferred ADR location (new):
- `docs/project_management/adrs/`
  - `draft/` (not accepted)
  - `queued/` (accepted or ready-for-planning; not implemented)
  - `implemented/` (landed/merged)
  - `superseded/` (obsolete; replaced by another ADR)

Legacy ADR locations (still supported):
- `docs/project_management/next/ADR-000X-*.md` (cross-cutting)
- `docs/project_management/next/<feature>/ADR-000X-*.md` (feature-local)

---

## Required structure for `impact_map.md`

Use the template:
- `docs/project_management/standards/templates/impact_map.md.tmpl`

The impact map must include:
1) Inputs (ADR paths + spec_manifest path)
2) Touch set (create/edit/deprecate/delete) with exact paths
3) Cascading implications (behavior/UX) and contradiction risks
4) Cross-queue scan (other ADRs + other Planning Packs) and conflict resolutions
5) Concrete follow-ups (Decision Register entries and/or required spec updates)

---

## Prompt (copy/paste)

Use this prompt to generate `impact_map.md`.

```md
You are the Impact Map agent for <FEATURE>.

Goal:
- Read the ADR(s) and `spec_manifest.md` for <FEATURE>.
- Produce `docs/project_management/next/<feature>/impact_map.md` that is exhaustive about:
  - what files/surfaces will be touched (create/edit/deprecate/delete),
  - cascading behavioral/UX implications,
  - contradictions and conflicts with queued/unimplemented work.

Constraints (non-negotiable):
- Do not write production code.
- Do not invent new scope; derive impacts from the ADR + required specs.
- No implied work: every change the ADR implies must appear in the touch set and implication sections.
- If you find multiple viable resolutions to a conflict, record a Decision Register entry (A/B) and select one.

Required reading:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`

Inputs:
- ADR(s): <list exact paths>
- Feature directory: `docs/project_management/next/<feature>/`
- `spec_manifest.md`: `docs/project_management/next/<feature>/spec_manifest.md`

Discovery requirements (must do):
1) Repo-wide touch discovery:
   - Identify the exact crates/modules/scripts/config/docs that must change to implement the ADR.
   - List paths under `crates/`, `src/`, `scripts/`, `docs/`, and any platform backends (`crates/world*`, `crates/shim`, `crates/shell`, `crates/world-agent`) that are implicated.
2) Cascading UX/behavior analysis:
   - Compare the proposed ADR contract against existing operator workflows to catch contradictions.
   - Call out any UX disjoint (different commands behaving inconsistently, mismatched defaults, conflicting env var semantics).
3) Cross-queue scan:
   - Scan queued/unimplemented ADRs under `docs/project_management/adrs/{draft,queued}/` (and legacy ADR locations).
   - Scan queued Planning Packs under `docs/project_management/next/*/` and `docs/project_management/_archived/*/` for overlapping surfaces.
   - Document overlaps/conflicts and how they are resolved (sequencing boundary, Decision Register, or explicit non-overlap).

Output requirements:
- Write/overwrite: `docs/project_management/next/<feature>/impact_map.md` using the template.
- The touch set must have concrete repo-relative file paths (no vague “update some files”).
- If you discover a missing surface or ownership gap, update `spec_manifest.md` before proceeding.
```

