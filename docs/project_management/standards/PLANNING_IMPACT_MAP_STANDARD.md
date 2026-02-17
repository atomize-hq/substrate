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
- `make planning-new-feature FEATURE=<feature>` creates `impact_map.md` from `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`.

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
- `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`

The impact map must include:
1) Inputs (ADR paths + spec_manifest path)
2) Touch set (create/edit/deprecate/delete) with exact paths
3) Cascading implications (behavior/UX) and contradiction risks
4) Cross-queue scan (other ADRs + other Planning Packs) and conflict resolutions
5) Concrete follow-ups (Decision Register entries and/or required spec updates)

---

## Touch set validation (gated strict)

Planning-time validation of the Touch Set is gated by the Planning Pack version:
- Read `<feature_dir>/tasks.json`
- If `meta.slice_spec_version >= 2`: STRICT validation
- Else: LEGACY (warn-only; never blocks)

STRICT Touch Set rules (for `## Touch set (explicit)` only):
- Required subsections: `### Create`, `### Edit`, `### Deprecate`, `### Delete`
- Each subsection content is either:
  - exactly `- None`, OR
  - one or more top-level bullets (no indentation)
- Each bullet must contain **exactly one** backticked repo-relative path token (`` `...` ``).
- Placeholder tokens are forbidden (e.g., `<path>`, `TBD`, `TODO`, `WIP`, `None yet.`).
- Path rules: POSIX `/` separators, no `..` segments, no globs, no backslashes, no absolute/`~`/drive-letter paths; leading `./` is allowed but normalized away; directory allow entries must end with `/`.
- Strict Touch Set must include at least one non-None entry total across all sections.

Downstream consumers:
- Execution-time enforcement (triad `task_finish`) consumes the validator’s `--emit-json` output (see Initiative 2 S2).

---

## Prompt (copy/paste)

Use this prompt to generate `impact_map.md`.

Canonical prompt: docs/project_management/system/prompts/planning/impact_map_agent.md

