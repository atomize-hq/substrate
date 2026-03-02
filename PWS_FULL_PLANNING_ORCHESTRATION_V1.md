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

Recommended format: YAML.

Example:
```yaml
pws_index:
  - id: WDAP-PWS-contract
    role: contract
    depends_on: []
    owns:
      - contract.md
      - decision_register.md
  - id: WDAP-PWS-world_agent_profile
    role: implementation
    depends_on: [WDAP-PWS-contract]
    owns:
      - slices/WDAP2/WDAP2-spec.md
  - id: WDAP-PWS-tasks_checkpoints
    role: tasks_checkpoints
    depends_on: [WDAP-PWS-contract]
    owns:
      - tasks.json
      - plan.md
```

Notes:
- `depends_on` MUST encode **hard dependencies only**.
- Non-blocking ordering preferences go in an optional `assumes:` list (not used to schedule).
- `owns` is repo-relative *within the pack root* (e.g., `contract.md`, `slices/...`, `tasks.json`).

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

## Implementation outline (MVP)

1) Enforce `pws_index` block emission in triage prompt + optional mechanical validator.
2) Add a PWS runner:
   - `make pm-run-pws FEATURE_DIR=... PWS_ID=...`
   - Strict allowlist derived from `pws_index[*].owns`
3) Add a PWS orchestrator:
   - Builds a runnable set from `pws_index` + `depends_on`
   - Enforces concurrency rules (`owns` disjoint, single-writer constraints)
   - Incorporates `pre-planning/alignment_report.md` as required gate input
4) Add allowlist expansion handling:
   - Detect `allowlist_request.yaml` + draft artifacts
   - Pause for operator decision (approve/deny/apply)

## Open questions (explicitly not decided yet)

- Exact “integration apply” mechanics:
  - manual operator step vs orchestrator-assisted patch apply vs a dedicated integration PWS.
- Exact locations/names of prompts for each `role` (contract, slice_spec, docs_validation, tasks_checkpoints, etc.).

