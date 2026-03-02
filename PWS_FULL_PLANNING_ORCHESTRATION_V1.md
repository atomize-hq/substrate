# Pack Planning Workstreams (PWS) + Full-Planning Orchestration (v1)

Status: Draft (decisions captured; implementation pending)  
Last updated: 2026-03-02

## Why this doc exists

Pre-planning now emits a `pre-planning/workstream_triage.md` artifact that proposes **pack-internal planning workstreams** (PWS). We want to:

1) Make those PWS IDs stable and machine-readable, so we can automate *full planning* in parallel where safe.
2) Keep the same safety model as pre-planning: strict output allowlists + logs-only drafts.
3) Avoid colliding with the repo’s existing umbrella **Workstreams** (`WS-YYYYMM-...`) concept.

This orchestration layer sits where the older workflow had a single “Planning agent → PACK created” step (see `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md`).

## Terminology (avoid collisions)

- **PWS (Planning Workstream)**: pack-internal planning stream, used to parallelize *full planning* work within a single pack.
- **Workstream (umbrella)**: initiative grouping multiple ADRs/packs/work items; ID format `WS-YYYYMM-initiative_slug` (see `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` and `WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md`).
- **Slice**: execution unit in a pack (e.g., `WDAP0`, `BEDPM2`, `DIWAS1`), typically mapped to triads.

## Decisions (locked-in)

### D1 — PWS IDs are stable and git/path-safe

PWS IDs use **no `:`** and must be stable once pre-planning is “done” for a pack.

- Format: `<SLICE_PREFIX>-PWS-<slug>`
- Examples:
  - `WDAP-PWS-contract`
  - `BEDPM-PWS-tests_ci`
  - `DIWAS-PWS-slice_spec_diwas0`
- Regex (recommended): `^[A-Z][A-Z0-9]*-PWS-[a-z0-9_]+$`

#### Source of truth for `<SLICE_PREFIX>`

The prefix MUST come from the pack’s `pre-planning/minimal_spec_draft.md` “Draft slice skeleton” section (e.g., “Slice prefix (draft): WDAP”).

Rule:
- Treat `<SLICE_PREFIX>` as **stable once pre-planning is done**; if it should change, record it as a gate/risk and do not rename existing PWS IDs mid-flight.

### D2 — Minimum required PWS

Every pack’s `pre-planning/workstream_triage.md` MUST include at least:

- `<PREFIX>-PWS-contract` (deterministic “first” gate)
- `<PREFIX>-PWS-tasks_checkpoints` (deterministic “last-ish” gate; single-writer for `tasks.json`)

All other PWS are dynamic and pack-specific.

### D3 — Workstream triage must emit a machine-readable PWS index

To avoid regex-parsing prose forever, `pre-planning/workstream_triage.md` MUST include a small machine-readable block enumerating PWS nodes, dependencies, and owned outputs.

Recommended format: embedded JSON (dependency-free) with deterministic markers.

Example:
````md
<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "WDAP",
  "pws": [
    {
      "id": "WDAP-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [],
      "owns": ["contract.md", "decision_register.md"]
    },
    {
      "id": "WDAP-PWS-world_agent_profile",
      "role": "implementation",
      "depends_on": ["WDAP-PWS-contract"],
      "assumes": [],
      "owns": ["slices/WDAP2/WDAP2-spec.md"]
    },
    {
      "id": "WDAP-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": ["WDAP-PWS-contract"],
      "assumes": [],
      "owns": ["tasks.json", "plan.md"]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->
````

Notes:
- The JSON block is the canonical input for orchestration (not the prose headings).
- `depends_on` MUST encode **hard dependencies only**.
- Non-blocking ordering preferences go in an optional `assumes:` list (not used to schedule).
- `owns` is repo-relative *within the pack root* (e.g., `contract.md`, `slices/...`, `tasks.json`).
- `owns` MUST be an exclusive set across PWS if we want safe parallel execution; `tasks.json` MUST be owned only by `<PREFIX>-PWS-tasks_checkpoints`.

### D4 — Default DAG shape is “star-ish”, not a chain

The system should *encourage* a “contract-first → parallel cluster → tasks/checkpoints” topology, but it must remain dynamic:

- `*-PWS-contract` is the deterministic first gate.
- Everything else may run concurrently if:
  - hard deps are satisfied, AND
  - `owns` sets are disjoint.
- `*-PWS-tasks_checkpoints` runs late and is the **single writer** for `tasks.json` (and usually `plan.md`).

### D5 — `pre-planning/alignment_report.md` is a first-class orchestration input

Full planning orchestration should treat `pre-planning/alignment_report.md` as a canonical “do not drop” index for:
- cross-pack misalignments (hard gates)
- Decision Register requirements
- CI/checkpoint wiring gaps
- risks/unknowns and other follow-ups

The orchestrator uses it to:
- seed required work into the appropriate PWS (especially `*-PWS-contract` and `*-PWS-tasks_checkpoints`)
- detect cross-pack conflicts that should be handled by an integration step (not buried inside one pack’s PWS)

## Execution model (safety + concurrency)

### Per-PWS safety model (same as pre-planning)

For a PWS run:
- Drafts/scratch MUST be written only to: `<PACK>/logs/pws/<PWS_ID>/**` (gitignored).
- Tracked writes MUST be restricted to that PWS’s declared `owns` allowlist.
- Any attempt to write tracked files outside `owns` is a hard error.

### Concurrency rules

PWS can run concurrently only if:
- their hard deps are satisfied, AND
- their `owns` sets are disjoint (tracked output isolation), AND
- they do not touch single-writer files (`tasks.json`, typically `plan.md`) except via `<PREFIX>-PWS-tasks_checkpoints`.

If isolation is unclear, run sequentially by default.

## Allowlist expansion + escalation policy (operator-controlled)

Problem:
- While running a PWS, we may discover we must edit an additional tracked file not in `owns`.

Policy:
1) The PWS agent MUST NOT silently expand scope.
2) Instead, it writes **logs only**:
   - `<PACK>/logs/pws/<PWS_ID>/allowlist_request.yaml` (requested paths + reason)
   - A proposed change as either:
     - `<PACK>/logs/pws/<PWS_ID>/draft.patch`, and/or
     - `<PACK>/logs/pws/<PWS_ID>/draft/<path>` (full draft file)

Operator decision (via the orchestrator):
- **Approve allowlist expansion**:
  - Expand that PWS’s tracked allowlist.
  - If the requested path is outside the `pre-planning/impact_map.md` touch set, treat it as scope drift:
    - update the touch set first, then re-run `make pm-lift-pack PACK="<PACK>"`.
- **Deny allowlist expansion, but accept the change**:
  - The PWS remains strict (no extra tracked writes).
  - The orchestrator is responsible for applying the draft to the tracked file(s) in a dedicated “integration apply” step (including reconciliation/merge with other changes).
- **Deny the change**:
  - Keep the draft in logs only; no tracked edits occur.

## Incremental implementation plan (MVP → parallelism)

### Step 0 — Lock the interface (triage output contract)

- Update the triage agent contract so every `pre-planning/workstream_triage.md` contains:
  - required PWS IDs (`<PREFIX>-PWS-contract`, `<PREFIX>-PWS-tasks_checkpoints`), and
  - an embedded JSON `PM_PWS_INDEX` block (see D3).

### Step 1 — Add a mechanical validator (non-invasive)

- Add `validate_pws_index.py` (or equivalent) that:
  - extracts and parses the `PM_PWS_INDEX` JSON block,
  - validates ID formats, required PWS presence, and basic schema,
  - checks that `owns` paths are pack-relative,
  - checks `tasks.json` is owned by exactly one PWS (`<PREFIX>-PWS-tasks_checkpoints`).
- Hook into `make planning-lint FEATURE_DIR=...` as a non-blocking advisory first (then promote to required when stable).

### Step 2 — Add a scheduler dry-run

- Add `make pm-pws-plan FEATURE_DIR=...` that prints:
  - a topo-ordered plan (by hard deps),
  - “parallel layers” (runnable sets) subject to `owns` disjointness and single-writer rules.

### Step 3 — Add a single-PWS runner (strict allowlists)

- Add `make pm-run-pws FEATURE_DIR=... PWS_ID=...`:
  - creates `<PACK>/logs/pws/<PWS_ID>/...` (drafts only),
  - enforces tracked-write allowlist = `pws_index[*].owns` for the selected PWS,
  - executes a role-specific prompt (start with `contract` and `tasks_checkpoints`; keep others generic initially).

### Step 4 — Add a sequential full-planning orchestrator

- Add `make pm-full-planning-orchestrate FEATURE_DIR=...`:
  - runs `<PREFIX>-PWS-contract` first,
  - runs remaining runnable PWS sequentially (MVP),
  - runs `<PREFIX>-PWS-tasks_checkpoints` last,
  - treats `pre-planning/alignment_report.md` as required input and routes:
    - “Gates / hard decisions” + “Decision Register required” → `<PREFIX>-PWS-contract`
    - “CI/checkpoint wiring gaps” → `<PREFIX>-PWS-tasks_checkpoints`

### Step 5 — Add operator-controlled allowlist expansion + integration-apply

- Standardize logs-only artifacts when a PWS needs to edit a tracked file outside `owns`:
  - `<PACK>/logs/pws/<PWS_ID>/allowlist_request.json` (requested paths + reason)
  - `<PACK>/logs/pws/<PWS_ID>/draft.patch` and/or `<PACK>/logs/pws/<PWS_ID>/draft/<path>`
- Orchestrator pauses for operator decision:
  - approve allowlist expansion (and optionally update touch set + re-run `pm-lift-pack`),
  - deny expansion but accept the change via “integration apply” step,
  - deny the change entirely (keep draft in logs only).

### Step 6 — Add safe parallelism (worktrees)

- Only after Step 4/5 is stable:
  - run disjoint PWS concurrently using git worktrees/branches,
  - route shared-file work to an explicit integration/apply step,
  - preserve single-writer invariants (`tasks.json`, often `plan.md`).

## Open questions (explicitly not decided yet)

- Exact “integration apply” mechanics:
  - manual operator step vs orchestrator-assisted patch apply vs a dedicated integration PWS.
- Exact locations/names of prompts for each `role` (contract, slice_spec, docs_validation, tasks_checkpoints, etc.).
