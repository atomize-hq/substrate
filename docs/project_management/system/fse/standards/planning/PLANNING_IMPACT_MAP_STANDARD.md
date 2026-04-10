# Planning Impact Map Standard

This standard inserts a mandatory step between:
- producing `spec_manifest.md`,
- and downstream FSE planning or decomposition.

Goal:
- Produce a deterministic, cross-cutting impact map that:
  - enumerates exact file and module surfaces to touch,
  - identifies cascading behavioral and UX implications,
  - checks alignment and conflict with queued or unimplemented ADRs and packs.

## When to run this step

Run this immediately after `spec_manifest.md` exists:
1. ADR drafted or refined
2. `spec_manifest.md` created
3. `impact_map.md` created
4. ADR iterated or accepted
5. downstream FSE planning or decomposition begins

## Required output

Create:
- `docs/project_management/packs/<bucket>/<feature>/pre-planning/impact_map.md`

Scaffolding:
- use `docs/project_management/system/fse/templates/planning_pack/impact_map.md.tmpl`

## Rules

1) Touch set is explicit.
   - Every expected file creation, edit, deprecation, and removal must be listed.
   - Use exact repo-relative paths by default.
   - Directory-prefix entries are fallback-only and must be tightened later when exact files become defensible.

2) Cascading implications are explicit.
   - For every user-facing or operator-facing change, state:
     - direct impact,
     - second-order impact,
     - contradiction risks.

3) Cross-queue alignment is required.
   - Identify relevant ADRs and planning packs that are not implemented yet and document:
     - overlap,
     - conflict or non-conflict,
     - explicit resolution.

4) No implied work.
   - If the ADR implies a change that does not appear in the touch set or implications sections, the map is incomplete.

5) The impact map is for planning truth, not task wiring.
   - Do not define task IDs, kickoff ownership, or execution graph behavior here.
   - Do not route alignment through pre-full-planning convergence, post-full-planning reconcile, or other legacy full-planning follow-up surfaces.

## Inputs

The impact map is derived from:
- ADR(s)
- `spec_manifest.md`
- the repo’s current code and docs
- queued or unimplemented ADRs and planning packs

## ADR locations for cross-queue discovery

Preferred ADR location:
- `docs/project_management/adrs/`
  - `draft/`
  - `queued/`
  - `implemented/`
  - `superseded/`

Legacy ADR locations may still exist:
- `docs/project_management/adrs/<bucket>/ADR-000X-*.md`

## Required structure for `impact_map.md`

Use:
- `docs/project_management/system/fse/templates/planning_pack/impact_map.md.tmpl`

The impact map must include:
1. Inputs
2. Touch set with exact paths
3. Cascading implications and contradiction risks
4. Cross-queue scan and conflict resolutions
5. Concrete follow-ups

## Touch-set validation

FSE pre-planning uses strict touch-set expectations by default:
- Required subsections: `### Create`, `### Edit`, `### Deprecate`, `### Delete`
- Each subsection content is either:
  - exactly `- None`, or
  - one or more top-level bullets
- Each bullet should contain exactly one backticked repo-relative path token.
- Placeholder tokens such as `<path>`, `TBD`, `TODO`, and `WIP` are forbidden.
- Paths must use POSIX `/` separators, with no `..`, no globs, and no absolute paths.
- `### Edit`, `### Deprecate`, and `### Delete` may reference only paths that already exist at authoring time.
- Non-existent paths belong under `### Create`.

Directory-prefix guidance:
- A directory-prefix entry must end with `/`.
- The directory must already exist.
- The map must record a follow-up to tighten it to exact files later.

## Downstream consumers

The impact map is consumed by:
- downstream workstream-pressure estimation and candidate restructuring analysis,
- downstream FSE planning and decomposition,
- human reviewers checking scope completeness.

Legacy execution-time task-wiring semantics are not part of this contract.
Legacy convergence or reconcile surfaces, if present, are compatibility scaffolding only and are outside the supported pre-planning lane.

## Prompt

Canonical prompt:
- `docs/project_management/system/fse/prompts/planning/impact_map_agent.md`
