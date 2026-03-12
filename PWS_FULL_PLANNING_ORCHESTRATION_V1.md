# Pack Planning Workstreams (PWS) + Full-Planning Orchestration (v1)

Status: Implemented through Step 5.5 convergence foundation  
Last updated: 2026-03-07

## Why this doc exists

Pre-planning now emits a `pre-planning/workstream_triage.md` artifact that proposes **pack-internal planning workstreams** (PWS). We want to:

1) Make those PWS IDs stable and machine-readable, so we can automate *full planning* in parallel where safe.
2) Keep the same safety model as pre-planning: strict output allowlists + logs-only drafts.
3) Avoid colliding with the repo’s existing umbrella **Workstreams** (`WS-YYYYMM-...`) concept.

This orchestration layer sits where the older workflow had a single “Planning agent → PACK created” step (see `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md`).

## Terminology (avoid collisions)

- **PWS (Planning Workstream)**: pack-internal planning stream, used to parallelize *full planning* work within a single pack.
- **Workstream (umbrella)**: initiative grouping multiple ADRs/packs/work items; ID format `WS-YYYYMM-initiative_slug` (see `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` and `WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md`).
- **Slice**: execution unit in a pack (for example `PREFIX0`, `PREFIX1`), typically mapped to triads.

## Decisions (locked-in)

### D1 — PWS IDs are stable and git/path-safe

PWS IDs use **no `:`** and must be stable once pre-planning is “done” for a pack.

- Format: `<SLICE_PREFIX>-PWS-<slug>`
- Examples:
  - `PFX-PWS-contract`
  - `PFX-PWS-tests_ci`
  - `PFX-PWS-slice_spec_pfx0`
- Regex (recommended): `^[A-Z][A-Z0-9]*-PWS-[a-z0-9_]+$`

#### Source of truth for `<SLICE_PREFIX>`

The prefix MUST come from the pack’s `pre-planning/minimal_spec_draft.md` “Draft slice skeleton” section (for example `Slice prefix (draft): PFX`).

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
  "pws_index_version": 2,
  "slice_prefix": "PFX",
  "accepted_slice_order": ["PFX0", "PFX1"],
  "draft_slice_order": ["PFX0"],
  "pws": [
    {
      "id": "PFX-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [],
      "owns": ["contract.md", "decision_register.md"]
    },
    {
      "id": "PFX-PWS-slice_spec_pfx0",
      "role": "implementation",
      "depends_on": ["PFX-PWS-contract"],
      "assumes": [],
      "owns": ["slices/PFX0/PFX0-spec.md"]
    },
    {
      "id": "PFX-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": ["PFX-PWS-contract"],
      "assumes": [],
      "owns": ["tasks.json", "session_log.md", "kickoff_prompts/", "slices/PFX0/kickoff_prompts/", "slices/PFX1/kickoff_prompts/"]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->
````

Notes:
- The JSON block is the canonical input for orchestration (not the prose headings).
- `accepted_slice_order` is the authoritative post-triage slice order for convergence and full planning.
- `draft_slice_order` is optional and advisory; it mirrors the unchanged draft skeleton when useful for diagnostics.
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

Important:
- `alignment_report.md` is a generated routing/index artifact, not a second slice-authority surface.
- Slice-order authority remains limited to `PM_PWS_INDEX.accepted_slice_order` plus the converged slice-bearing pre-planning docs.

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

### Step 3.5 — Align Step 3 `tasks_checkpoints` with the triad system (execution-ready packs)

This landed as a contract hardening step around `tasks_checkpoints`.

What is now true:
- the pre-planning triage prompt requires triad-critical `owns` for `*-PWS-tasks_checkpoints`,
- `validate_pws_index.py` enforces those required `owns`,
- and the runner continues to keep tracked writes strictly allowlist-driven.

The important rule is:
- `*-PWS-tasks_checkpoints` must own the execution-triad scaffolding it is expected to author,
- especially `session_log.md`, `kickoff_prompts/`, and per-slice kickoff prompt directories.

This closes the class of failures where a `tasks_checkpoints` session could make `tasks.json` look mechanically acceptable while still being unable to author the prompt/report surfaces required for execution.

#### Quick reference: schema v4 cross-platform checkpoint-boundary model (what `validate_tasks_json.py` expects)

When `tasks.json` has:
- `meta.schema_version >= 4`
- `meta.cross_platform = true`

Then (in addition to the normal task schema rules) the cross-platform integration model is:

- `meta.checkpoint_boundaries` is **required** and must list the **last slice id** in each checkpoint group.
  - `validate_ci_checkpoint_plan.py` additionally requires this list to match the checkpoint boundaries in `ci_checkpoint_plan.md` exactly.
- For **every** slice `X`:
  - Always define: `X-code`, `X-test`, and `X-integ`.
    - `X-integ` is the only task used by `validate_slice_specs.py` for AC traceability (`ac_ids` must match the slice spec).
  - If `X` is **not** a checkpoint boundary:
    - Do **not** define any `X-integ-core` or `X-integ-<platform>` tasks.
    - `X-code.integration_task` and `X-test.integration_task` must both be `X-integ`.
  - If `X` **is** a checkpoint boundary:
    - Define: `X-integ-core`, `X-integ-<platform>` for every CI parity platform, and `X-integ` as the final aggregator.
    - Wiring rules (hard requirements):
      - `X-code.integration_task = "X-integ-core"` and `X-test.integration_task = "X-integ-core"`
      - `X-integ-core.depends_on` includes `X-code` and `X-test`
      - Each `X-integ-<platform>.depends_on` includes `X-integ-core` and sets `platform="<platform>"`
      - `X-integ.depends_on` includes `X-integ-core` and all `X-integ-<platform>` tasks
    - Automation merge rules (when `meta.schema_version >= 3` and `meta.automation.enabled=true`):
      - `X-integ-core.merge_to_orchestration = false`
      - `X-integ-<platform>.merge_to_orchestration = false`
      - `X-integ.merge_to_orchestration = true`

Authoritative references:
- `docs/project_management/system/scripts/planning/validate_tasks_json.py` (`_validate_platform_integ_model`)
- `docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py`
- `docs/project_management/system/scripts/planning/new_feature.sh` (canonical scaffolding)

#### Validator gotchas (common failure modes)

- `validate_slice_specs.py` enforces a hard limit of **1..8** AC bullets in each slice spec.
  - If a slice spec has more than 8 ACs, `tasks_checkpoints` cannot produce a consistent `ac_ids` set; fix/split the slice before “final” task wiring.
- `validate_tasks_json.py` requires `kickoff_prompt` files to exist on disk and live under:
  - `<FEATURE_DIR>/kickoff_prompts/`, or
  - `<FEATURE_DIR>/slices/<SLICE_ID>/kickoff_prompts/`
  …so the `PM_PWS_INDEX` `owns` for `tasks_checkpoints` must include these directories as prefix paths.
- Planning lint requires the exact sentinel line in every kickoff prompt:
  - `Do not edit planning docs inside the worktree.`
  - Templates should include it; if not, use `docs/project_management/system/scripts/planning/ensure_kickoff_prompt_sentinel.py`.
- `validate_ci_checkpoint_plan.py` derives slice ordering from `*-integ` tasks.
  - If `tasks_checkpoints` hasn’t created the `*-integ` tasks yet, checkpoint plan validation cannot run (and should fail).

#### Pack-local inputs `tasks_checkpoints` should always read

These are the pack artifacts that drive correct task graph + prompt wiring:
- `<FEATURE_DIR>/pre-planning/workstream_triage.md` (including `PM_PWS_INDEX`)
- `<FEATURE_DIR>/pre-planning/minimal_spec_draft.md` (slice skeleton + prefix source of truth)
- `<FEATURE_DIR>/pre-planning/spec_manifest.md`
- `<FEATURE_DIR>/pre-planning/impact_map.md`
- `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md` (when cross-platform automation is intended)
- `<FEATURE_DIR>/pre-planning/alignment_report.md` (gates/risks that must route into tasks/checkpoints)
- For each slice: `<FEATURE_DIR>/slices/<SLICE_ID>/<SLICE_ID>-spec.md` (AC IDs + references)

#### Required contract changes (start at pre-planning)

Update `docs/project_management/system/prompts/planning/workstream_triage_agent.md` so the generated `PM_PWS_INDEX` for `<SLICE_PREFIX>-PWS-tasks_checkpoints` includes **triad-critical owns**.

Minimum recommended `owns` additions for `*-PWS-tasks_checkpoints` (pack-relative; use trailing `/` for prefix ownership):
- `session_log.md`
- `kickoff_prompts/` (feature/ops kickoff prompts: `F0-exec-preflight`, `CP*-ci-checkpoint`, `FZ-feature-cleanup`, etc.)
- For each slice in the accepted/draft skeleton: `slices/<SLICE_ID>/kickoff_prompts/`

Additional recommended `owns` (depending on feature posture):
- `execution_preflight_report.md` (when `meta.execution_gates=true` or when you always want preflight gating)
- For each slice: `slices/<SLICE_ID>/<SLICE_ID>-closeout_report.md` (when `meta.execution_gates=true`)

Notes:
- Keep `tasks.json` as a single-writer owned only by `*-PWS-tasks_checkpoints` (already enforced by `validate_pws_index.py`).
- Avoid `owns` overlap: do **not** give any other PWS ownership of kickoff prompt paths if `tasks_checkpoints` owns them.

#### Required prompt changes (`tasks_checkpoints` role)

Update `docs/project_management/system/prompts/planning/pws_tasks_checkpoints_agent.md` to be explicitly triad-aware:
- Generate an execution-ready `tasks.json` (do not “make validation pass” by disabling automation/cross-platform when the pack intends to use triads).
- Generate all kickoff prompt files referenced by `tasks.json.kickoff_prompt` using:
  - `docs/project_management/system/templates/kickoff/*`
  - Canonical locations:
    - Slice tasks: `slices/<SLICE_ID>/kickoff_prompts/<task-id>.md`
    - Feature/ops tasks: `kickoff_prompts/<task-id>.md`
- Populate `ac_ids` for `<SLICE_ID>-code`, `<SLICE_ID>-test`, and `<SLICE_ID>-integ` by extracting `AC-<SLICE_ID>-NN` entries from the slice spec’s `## Acceptance criteria` section.
  - Do **not** add `ac_ids` to `*-integ-core` or `*-integ-<platform>` tasks; only `<SLICE_ID>-integ` is used for AC traceability (see `validate_slice_specs.py` and existing packs).
- Include the kickoff prompt sentinel required by lint: `Do not edit planning docs inside the worktree.` (templates should already do this; lint will fail if missing).
- If allowlisting still blocks required tracked outputs, do **not** downgrade schemas.
  - Instead: emit `allowlist_request.json` + `draft.patch` under `<PACK>/logs/pws/<PWS_ID>/`.

Implementation reference for the canonical task graph + prompt rendering:
- `docs/project_management/system/scripts/planning/new_feature.sh` (authoritative scaffolder)

#### Required Step 3 runner hardening (definition of “success”)

Update `docs/project_management/system/scripts/planning/run_pws_agent.sh` so that for `role=tasks_checkpoints`:
- After `validate_tasks_json.py` passes, also run:
  - `validate_slice_specs.py --feature-dir "<FEATURE_DIR>"`
  - `validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"` (when a checkpoint plan exists / when `meta.cross_platform=true`)
- Optionally (strongly recommended): run `make planning-lint FEATURE_DIR="<FEATURE_DIR>"` as a final “execution-ready” gate once it’s stable/fast enough.

The validator is intentionally narrow here:
- it does not try to infer higher-level execution semantics from pack-specific content,
- it only enforces the ownership preconditions needed for safe triad authoring later in full planning.

### Step 4 — Add a sequential full-planning orchestrator

- Add `make pm-full-planning-orchestrate FEATURE_DIR=...`:
  - runs `<PREFIX>-PWS-contract` first,
  - runs remaining runnable PWS sequentially (MVP),
  - runs `<PREFIX>-PWS-tasks_checkpoints` last,
  - refreshes `pre-planning/alignment_report.md` immediately before the pre-task coherence gate for `<PREFIX>-PWS-tasks_checkpoints`,
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

### Step 5.5 — Pre-full-planning convergence (landed)

The landed Step 5.5 work is a dedicated convergence stage between pre-planning and full planning.

The problem it solves is generic:
- triage can adopt a post-draft slice inventory/order,
- some pre-planning artifacts can still describe the old draft slice model,
- and full planning should not start task wiring from those contradictory inputs.

#### Landed contract

The landed authority and convergence rules are:

1) `PM_PWS_INDEX` v2 makes the post-triage slice order explicit.
- `accepted_slice_order` is the authoritative post-triage slice order.
- `draft_slice_order` is optional and advisory.
- v1 remains readable for migration.

2) Pre-full-planning coherence is now a distinct validation phase.
- `validate_slice_inventory_coherence.py --phase pre_full_planning`
- This phase compares `workstream_triage.accepted_slice_order` against:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/ci_checkpoint_plan.md`
- `minimal_spec_draft.md` remains informational when triage adopts a different accepted order.

3) Convergence is bounded and narrowly scoped.
- `pre_full_planning_convergence.py` classifies the pack as:
  - `pass`
  - `needs_remediation`
  - `hard_fail`
- Only safe pre-planning slice inventory/order drift is auto-remediable.
- Non-slice semantic contradictions remain hard failures.

4) The remediation agent is intentionally constrained.
- The agent may edit only:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/ci_checkpoint_plan.md`
- It must not edit:
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/workstream_triage.md`
  - `tasks.json`

5) `alignment_report.md` remains generated, not agent-authored.
- `pre_full_planning_converge.sh` regenerates the tracked `pre-planning/alignment_report.md` after successful convergence.
- The report is not directly remediated by the agent.
- During full planning, the orchestrator refreshes `pre-planning/alignment_report.md` again immediately before `<PREFIX>-PWS-tasks_checkpoints`.
- The report may mention only a subset of accepted slices; it is used for routing follow-ups, not for exact slice-set authority.

#### Landed entrypoints

The landed orchestration entrypoints are:

- `make pm-pre-full-planning-converge FEATURE_DIR=...`
  - runs only the new convergence stage
- `make pm-full-planning-orchestrate FEATURE_DIR=...`
  - now runs convergence before requiring/reading the tracked alignment report and before computing the PWS execution plan
- `make pm-planning-pipeline FEATURE_DIR=...`
  - runs pre-planning research, then convergence, then full planning

There is also an optional `RUN_PIPELINE=1` path on `pm-pre-planning-from-adr` to launch the full chain after scaffold.

#### Landed implementation pieces

The concrete pieces that landed are:

- `docs/project_management/system/scripts/planning/pre_full_planning_convergence.py`
  - emits deterministic JSON classification for convergence
- `docs/project_management/system/scripts/planning/pre_full_planning_converge.sh`
  - runs validate -> optional reconcile -> regenerate alignment report -> revalidate
- `docs/project_management/system/prompts/planning/pre_planning_slice_reconcile_agent.md`
  - tightly constrained remediation prompt
- `docs/project_management/system/scripts/planning/full_planning_orchestrate.sh`
  - invokes convergence first
- `docs/project_management/system/scripts/planning/planning_pipeline_orchestrate.sh`
  - optional top-level orchestration chain

#### What Step 5.5 does not do

This landed scope is intentionally narrow.

It does not:
- auto-edit `minimal_spec_draft.md`,
- auto-edit `workstream_triage.md`,
- auto-edit `tasks.json`,
- or treat general semantic contradictions as safe auto-remediations.

Those remain outside the convergence loop and should fail deterministically when encountered.

### Step 5.6 — Post-full-planning execution convergence

After the last PWS finishes, full planning now runs a second bounded convergence gate before orchestration success is reported.

#### Landed contract

1) The gate is execution-readiness oriented, not slice-authority oriented.
- It runs after full planning completes.
- It is the final blocker before `full_planning_orchestrate.sh` reports success.

2) The gate uses the existing mechanical validators plus one new touch-set coherence check.
- Baseline dry run:
  - `validate_tasks_json.py`
  - `validate_slice_inventory_coherence.py --phase execution_ready`
  - `validate_slice_specs.py`
  - `validate_ci_checkpoint_plan.py` when applicable
  - `validate_impact_map.py`
  - `make planning-lint FEATURE_DIR=...`
- New gap-filler:
  - `validate_execution_touchset_coherence.py`
  - This checks explicit repo-relative, non-pack implementation-facing paths referenced by late-pack outputs against `impact_map.md`.

3) Classification matches the pre-full model.
- `post_full_planning_convergence.py` classifies the pack as:
  - `pass`
  - `needs_remediation`
  - `hard_fail`

4) Safe remediation scope is fixed and narrow.
- The reconcile agent may edit only:
  - `pre-planning/impact_map.md`
  - `plan.md`
  - `tasks.json`
  - kickoff prompts referenced by `tasks.json`
  - `manual_testing_playbook.md`
  - `execution_preflight_report.md`
  - existing per-slice closeout reports
- It must not edit:
  - ADRs
  - `contract.md`
  - `decision_register.md`
  - slice specs
  - pre-planning slice-authority docs (`workstream_triage.md`, `minimal_spec_draft.md`, `spec_manifest.md`, `ci_checkpoint_plan.md`)

5) `alignment_report.md` remains generated here too.
- `post_full_planning_converge.sh` regenerates `pre-planning/alignment_report.md` after successful convergence.
- The report is still generated, not agent-authored.

#### Landed entrypoints

- `make pm-post-full-planning-converge FEATURE_DIR=...`
  - runs only the post-full execution-readiness gate
- `make pm-full-planning-orchestrate FEATURE_DIR=...`
  - now runs post-full convergence after the PWS loop and before reporting success
- `make pm-planning-pipeline FEATURE_DIR=...`
  - picks up post-full convergence transitively through full planning; it does not add a second top-level post-full step

### Step 6 — Add safe parallelism (worktrees)

- Only after Step 4/5 is stable:
  - run disjoint PWS concurrently using git worktrees/branches,
  - route shared-file work to an explicit integration/apply step,
  - preserve single-writer invariants (`tasks.json`, often `plan.md`).

## Open questions (explicitly not decided yet)

- Exact “integration apply” mechanics:
  - manual operator step vs orchestrator-assisted patch apply vs a dedicated integration PWS.
- Exact locations/names of prompts for each `role` (contract, slice_spec, docs_validation, tasks_checkpoints, etc.).
