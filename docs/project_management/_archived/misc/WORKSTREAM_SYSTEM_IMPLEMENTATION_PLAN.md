# Workstreams + Work Items (v1) — Implementation Plan

Created (UTC): 2026-02-24  
Status: Draft (plan only; no implementation in this document)

This plan describes how to implement the **Workstream** / **Work Item** registries and the **two-pass workstream lifecycle** described in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (D1), using the already-shipped Work Lift v1 tooling (`pm_lift.py`, Make targets, and contracts).

---

## 0) Context / current state

### What exists today

- **Work Lift v1 is implemented** and pinned by published artifacts:
  - Lift Vector schema: `docs/project_management/system/schemas/work_lift_vector.schema.json` (CONTRACT-1)
  - Lift model config: `docs/project_management/system/schemas/work_lift_model.v1.json` (CONTRACT-2)
  - `pm_lift --emit-json` output contract: `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md` (CONTRACT-3)
  - `validate_impact_map --emit-json` output contract: `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md` (CONTRACT-4)
  - Make entry points: `make pm-lift-intake`, `make pm-lift-pack`, `make pm-lift-diff`, `make pm-lift-strict`
- Discovery prompts under `docs/project_management/system/prompts/discovery/` already instruct computing lift via `make pm-lift-*`.
- Planning Pack `tasks.json` validation already supports optional workstream/work-item linking fields (format validation in all packs, stricter existence checks in strict packs):
  - Script: `docs/project_management/system/scripts/planning/validate_tasks_json.py`
  - Field: `tasks.json meta.workstream_id` (optional)
  - Field: `tasks.json meta.work_item_refs` (optional)

### What is missing

- The canonical registries referenced by standards and strict-pack validation defaults do **not** exist on disk yet:
  - Expected: `docs/project_management/workstreams/` and `docs/project_management/work_items/`
  - Present today: only `docs/project_management/intake/work_items/` and (currently) `docs/project_management/intake/workstreams/`
- There is **no scaffolding or Make targets** for workstream/work-item records (creation, linking, refinement updates).
- “Workstream Triage” and “Workstream Refinement” (D1) are not implemented as a workflow or tooling.

---

## 1) Decisions (already made)

These decisions are treated as requirements for implementation.

### Canonical registry locations

- Workstreams live at: `docs/project_management/workstreams/`
- Work items live at: `docs/project_management/work_items/`
- Intake remains intake-only:
  - Keep: `docs/project_management/intake/work_items/` (draft intake forms)
  - Remove: `docs/project_management/intake/workstreams/` (workstreams are canonical records, not intake-only artifacts)

### ID formats (snake_case initiative slugs)

- Workstream IDs: `WS-YYYYMM-initiative_slug` where `initiative_slug` is `snake_case`.
- Work item IDs: `WI-YYYYMM-work_item_slug` where `work_item_slug` is `snake_case`.

### One record file per ID (strict packs)

Strict packs (`tasks.json.meta.slice_spec_version >= 2`) must be able to resolve:

- exactly **one** on-disk workstream record per `meta.workstream_id`
- exactly **one** on-disk work item record per ID in `meta.work_item_refs` / dependency lists

This requires validator enforcement (see §4).

### Workstream meaning

A workstream is an **umbrella initiative** grouping **multiple** related artifacts:

- multiple Work Items and/or
- multiple Planning Packs (and therefore multiple ADRs/slices)

Workstream IDs and titles must be **initiative-themed** and **must not reuse** a single contained WI/ADR/pack codename/title.

### Optional linkage (for now)

- `tasks.json meta.workstream_id` remains optional (may be `null`).

### Workstream lifecycle timing (Option A)

- Discovery prompts do **not** create workstream records.
- Intake + lockdown produce:
  - ADR/WI intake files (draft inputs), and
  - ADR drafts (decision-ready, queued/accepted for planning).
- A dedicated **Pass A Workstream Triage** session happens **after ADR lockdown** and **immediately before planning kickoff**.
  - Pass A is where we select (or propose) the umbrella workstream and capture coarse lift evidence.
- **Pass B Workstream Refinement** happens **after planning-lint is green**.
  - Pass B is where we confirm/lock the assignment (if provisional) and replace coarse evidence with pack-derived evidence.

---

## 2) Minimal data model (records)

### Workstream record (one per `WS-*`)

Canonical path (example):
- `docs/project_management/workstreams/WS-202602-world_backend_unification-umbrella.md`

Required sections:

1. **Header**
   - `# WS-YYYYMM-initiative_slug — <title>`
2. **Metadata**
   - Status: `discovery | active | paused | completed | superseded`
   - Owner(s)
   - Created (UTC)
   - Last updated (UTC)
3. **Vision / Problem / Non-goals**
4. **Included work (authoritative index)**
   - Packs (paths): `docs/project_management/packs/<bucket>/<feature>/`
   - Work items (IDs): `WI-YYYYMM-...`
   - ADR refs (optional): `ADR-0000...`
5. **Pass A — Workstream Triage (coarse, ADR/intake-first)**
   - Lift evidence per included intake/ADR candidate:
     - command(s) run (`make pm-lift-intake FILE=...`)
     - summarized outputs: `lift_score`, `estimated_slices`, `confidence`, `missing_inputs`, `triggers`
   - Risks / split signals (interpret triggers first; see D8/D9)
6. **Pass B — Workstream Refinement (pack-derived)**
   - For each included pack (only after planning-lint is green):
     - command(s) run (`make pm-lift-pack PACK=...`, optional `PM_LIFT_ADVISORY=1 make planning-lint ...`)
     - summarized outputs + any strict-mode notes (if applicable)
     - execution primitives: total slices, checkpoint groupings (from pack docs; not inferred)
7. **Sequencing / dependency sketch**
8. **Milestones and exit criteria**

### Work item record (one per `WI-*`)

Canonical path (example):
- `docs/project_management/work_items/WI-202602-telemetry_hygiene-redaction_pass.md`

Required sections:

1. **Header**
   - `# WI-YYYYMM-work_item_slug — <title>`
2. **Metadata**
   - Status: `queued | in_progress | blocked | completed | canceled`
   - Owner(s)
   - Created (UTC)
   - Last updated (UTC)
3. **Why this is a Work Item (not an ADR)**
4. **Scope + Done means (observable outcomes)**
5. **Links**
   - Workstream: `WS-YYYYMM-...` (optional but recommended if a workstream exists)
   - Packs (paths, optional)
   - ADR refs (optional)
6. **Lift Summary (time-free)**
   - Include Lift Vector v1 + computed outputs when possible.
   - Source of lift vector:
     - v1: keep the intake file as the authoring surface (`docs/project_management/intake/work_items/...`) and copy the **computed outputs** into the canonical WI record (and optionally link back to the intake file).

---

## 3) Required workflow (D1)

### Pass A — Workstream Triage (post-lockdown, pre-planning kickoff)

Goal: select an umbrella workstream (existing or new) for the ADR and record coarse, time-free signals **before** Planning Pack creation.

Workflow:
1. Identify candidate workstreams that fit (if any):
   - Scan `docs/project_management/workstreams/` for active/in-flight initiatives (exclude `completed` / `superseded`).
   - Propose up to ~3 candidate workstreams that the ADR could belong to.
   - For each candidate, include explicit fit criteria (surface overlap, sequencing overlap, goal alignment) and any disqualifiers.
2. Choose a recommendation:
   - Either select an existing `WS-YYYYMM-initiative_slug`, or propose a new WSID + title (initiative-themed; not equal to any single contained pack/WI/ADR name).
   - Provide “Choose A when / Choose B when …” reasoning if multiple candidates are viable.
3. Create/update the canonical workstream record under `docs/project_management/workstreams/`:
   - If the recommended WSID does not exist yet: create the record (one file per WSID).
   - Add references:
     - ADR ref and/or ADR intake path(s)
     - any already-known work items (IDs when they exist; otherwise placeholders that will be promoted later)
4. Compute lift per intake/ADR candidate and paste the computed outputs:
   - `make pm-lift-intake FILE=...` (optional: `EMIT_JSON=1`)
   - Summarize: `lift_score`, `estimated_slices`, `confidence`, `missing_inputs`, `triggers`
5. Declare whether the assignment is **provisional** or **locked**:
   - If provisional: list explicit conditions that would cause reassignment/splitting during Pass B (pack-derived evidence).
   - If locked: state the reason it is stable enough to use as the umbrella grouping.

### Pass B — Workstream Refinement (after planning-lint is green)

Goal: confirm/lock the workstream assignment (if provisional) and replace coarse estimates with pack-derived, execution-shaped counts.

Workflow:
1. For each pack in the workstream:
   - record pack path + current bucket
   - record pack-derived lift outputs:
     - `make pm-lift-pack PACK=...`
     - optional: `PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR=...`
2. Record execution primitives from the pack (do not invent):
   - slice count
   - checkpoint boundaries/groups (if cross-platform)
3. If the pack’s `tasks.json meta.workstream_id` is unset (`null`):
   - set it to the selected WSID only once the assignment is locked (exactly one matching workstream record must exist for strict packs).
4. Update sequencing/dependency sketch at the workstream level.

---

## 4) Tooling + validation plan (implementation work)

This section enumerates the repo changes required to make the workflows above low-friction and mechanically correct.

### 4.1 Registry directories + READMEs

Add:
- `docs/project_management/workstreams/README.md` (purpose, ID rules, file naming, required sections)
- `docs/project_management/work_items/README.md` (same)

Remove:
- `docs/project_management/intake/workstreams/` (migrate any existing non-README content if it exists)

### 4.2 Templates (docs-only artifacts used by scaffolding)

Add templates under the PM system root (exact location is an implementation choice; keep them discoverable and canonical), e.g.:
- `docs/project_management/system/templates/registry/workstream.md.tmpl`
- `docs/project_management/system/templates/registry/work_item.md.tmpl`

### 4.3 Make targets + scripts (scaffolding + updates)

Add Make targets (names are proposals; keep them consistent with existing `pm-*` targets):

- `make pm-workstream-new WSID=WS-YYYYMM-initiative_slug TITLE="..."`
  - Creates a single workstream record file using the template.
- `make pm-work-item-new WIID=WI-YYYYMM-work_item_slug TITLE="..."`
  - Creates a single work item record file using the template.

Optional follow-ons (high leverage for adoption; still docs-first and deterministic):

- `make pm-workstream-triage WSID=... INTAKE=...`
  - Runs `pm-lift-intake` on referenced intakes and appends a “Pass A” entry.
- `make pm-workstream-refine WSID=... PACK=...`
  - Runs `pm-lift-pack` for the pack and appends a “Pass B” entry.

### 4.4 Strict-pack validator enforcement: “exactly one match”

Update `docs/project_management/system/scripts/planning/validate_tasks_json.py`:

- For strict packs, when resolving:
  - `meta.workstream_id`
  - `meta.work_item_refs`
  - `depends_on.workstreams` / `blocks.workstreams`
  - `depends_on.work_items` / `blocks.work_items`
- Require **exactly one** matching file per ID:
  - error if no matches
  - error if more than one match (include the list of matches in the error)

This implements the “one file per ID” requirement for strict packs while still allowing descriptive filenames via the existing `ID-*.md` pattern.

---

## 5) Documentation updates (system)

Update system docs/prompts so humans/agents create the right artifacts in the right place:

- Discovery prompts:
  - Remove workstream record creation from discovery prompts (Option A).
  - The discovery layer may reference workstreams (e.g., “optional workstream link”), but must not require creating any workstream file.
  - Workstreams are created/selected during **Pass A Workstream Triage** (post-lockdown, pre-planning kickoff).
- Add new system prompts (one session each):
  - `docs/project_management/system/prompts/planning/workstream_triage.md` (Pass A: propose up to ~3 candidate workstreams + recommendation; create/update the selected workstream record; record `pm-lift-intake` evidence)
  - `docs/project_management/system/prompts/planning/workstream_refinement.md` (Pass B: update the workstream record with pack-derived evidence; set `tasks.json meta.workstream_id` once locked)
- Ensure the PM system entrypoints correctly describe registries:
  - `docs/project_management/system/USER_GUIDE.md`
  - `docs/project_management/system/standards/planning/PLANNING_README.md`
  - `docs/project_management/system/schemas/tasks.schema.json` descriptions (if needed)
  - `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md` (add Pass A triage and Pass B refinement to the diagram)

---

## 6) Rollout stages (safe, incremental)

Stage 1 (enable scaffolding + registries; optional usage):
- Create `docs/project_management/workstreams/` and `docs/project_management/work_items/`.
- Update prompts/docs to point at canonical registries.
- Add scaffolding targets/scripts.
- Keep `meta.workstream_id` optional.

Stage 2 (adoption in strict packs):
- For any strict pack that sets `meta.workstream_id` / `meta.work_item_refs`, require the canonical record exists and is uniquely resolvable.

Stage 3 (workstream refinement helpers):
- Add `pm-workstream-triage` / `pm-workstream-refine` helpers (optional) once the base registry is stable.

---

## 7) Definition of done (for the implementation)

Implementation is considered complete when:

- The canonical registries exist on disk:
  - `docs/project_management/workstreams/`
  - `docs/project_management/work_items/`
- Workstreams no longer live under `docs/project_management/intake/`.
- There is a deterministic scaffolding path (templates + Make targets) to create:
  - one workstream record by WSID
  - one work item record by WIID
- Strict-pack validation enforces “exactly one match” per referenced WSID/WIID.
- The discovery prompts produce correctly-located artifacts and correctly instruct the Work Lift commands for evidence capture.
