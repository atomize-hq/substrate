# Pack Planning Workstreams (PWS) + Full-Planning Orchestration (v1)

Status: Draft (decisions captured; implementation pending)  
Last updated: 2026-03-06

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

### Step 3.5 — Align Step 3 `tasks_checkpoints` with the triad system (execution-ready packs)

Step 3 (single-PWS runner) is **strictly allowlist-driven**: tracked writes are allowed only to the PWS’s `owns` paths from the `PM_PWS_INDEX` fenced JSON in `pre-planning/workstream_triage.md`.

That strictness is correct — but it creates a sharp edge:
the `tasks_checkpoints` PWS is the natural place to author **execution triads** (`tasks.json` + kickoff prompts + checkpoint wiring), yet the current pre-planning triage contract does not require that `tasks_checkpoints` owns the triad-critical paths.

The result is a “false success” mode where a `tasks_checkpoints` run can make `validate_tasks_json.py` pass while still producing a pack that **cannot** be executed via the repo’s triad workflow.

#### The problem (what went wrong in WDRA)

In the smoke-tested pack:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/`

`WDRA-PWS-tasks_checkpoints` did **not** own any kickoff prompt paths in `PM_PWS_INDEX`, so `pm-run-pws` correctly prohibited generating them.

The run therefore produced:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/logs/pws/WDRA-PWS-tasks_checkpoints/allowlist_request.json`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/logs/pws/WDRA-PWS-tasks_checkpoints/draft.patch`

…and it “unblocked” mechanical validation by temporarily downgrading the pack to a non-automation stub:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json` set to `meta.schema_version=2`, `meta.cross_platform=false`, `meta.automation.enabled=false`

This is **not aligned** with the repo’s execution system:
execution triads require a schema v3/v4 automation pack + kickoff prompts + slice AC traceability (see triad standards below).

#### Root cause (system-level)

1) The pre-planning triage agent contract (`workstream_triage_agent.md`) defines `PM_PWS_INDEX` but does **not** require `tasks_checkpoints` to own:
   - `session_log.md`
   - `kickoff_prompts/` and `slices/<SLICE_ID>/kickoff_prompts/`
   - (optionally) execution gate files (`execution_preflight_report.md`, per-slice closeout reports)
2) The Step 3 runner (`run_pws_agent.sh`) currently validates only `validate_tasks_json.py` after `tasks_checkpoints` runs.
   - It does **not** validate slice spec ↔ task traceability (`validate_slice_specs.py`)
   - It does **not** validate checkpoint plan ↔ tasks.json wiring (`validate_ci_checkpoint_plan.py`)
   - It does **not** enforce the kickoff prompt sentinel required by planning lint (`Do not edit planning docs inside the worktree.`)

#### Target state (what “aligned with triads” means)

For packs that intend to execute via triads (especially automation packs):

1) `PM_PWS_INDEX` must give `*-PWS-tasks_checkpoints` the ability to write the triad surfaces it is responsible for.
2) A successful `tasks_checkpoints` PWS run must produce an **execution-ready** pack, not a schema downgrade.

Concretely, the `tasks_checkpoints` PWS should be able to generate:
- `tasks.json` with the triad task graph:
  - slice code/test tasks (always paired)
  - slice integration tasks (schema-v4 boundary model when `meta.cross_platform=true`)
  - checkpoint ops tasks (`CP*-ci-checkpoint`) when checkpoint plans exist
  - `FZ-feature-cleanup` (required for automation packs)
  - (recommended) `F0-exec-preflight` + execution gate wiring when `meta.execution_gates=true`
- Kickoff prompts referenced by `tasks.json.kickoff_prompt` for every code/test/integration task, plus feature-level ops prompts.
- `session_log.md` (required by planning lint; updated by the operator during execution).

And it should do so in a way that satisfies the repo’s validators/lint:
- `validate_tasks_json.py` (task graph invariants + automation/cross-platform rules)
- `validate_slice_specs.py` (Slice Spec v2 required headers + AC IDs; tasks.json `ac_ids` must match the slice spec; max 8 AC items)
- `validate_ci_checkpoint_plan.py` (checkpoint plan partitions must match slice ordering and `meta.checkpoint_boundaries` for schema v4 cross-platform packs)
- `make planning-lint FEATURE_DIR=...` (kickoff prompt sentinel, hard bans, ambiguity scan, etc.)

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
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/` (real schema v4 triad pack example)

#### Required Step 3 runner hardening (definition of “success”)

Update `docs/project_management/system/scripts/planning/run_pws_agent.sh` so that for `role=tasks_checkpoints`:
- After `validate_tasks_json.py` passes, also run:
  - `validate_slice_specs.py --feature-dir "<FEATURE_DIR>"`
  - `validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"` (when a checkpoint plan exists / when `meta.cross_platform=true`)
- Optionally (strongly recommended): run `make planning-lint FEATURE_DIR="<FEATURE_DIR>"` as a final “execution-ready” gate once it’s stable/fast enough.

#### Optional validator tightening (earlier, clearer failures)

Consider enhancing `docs/project_management/system/scripts/planning/validate_pws_index.py` so that (at least in advisory mode) it warns when:
- `*-PWS-tasks_checkpoints` owns `tasks.json` but does not own kickoff prompt directories and `session_log.md`.

This makes the triad alignment issue visible at pre-planning time (before any PWS runs).

#### Required reading for Step 3.5 (new-session index)

Standards (triads + planning):
- `docs/project_management/system/standards/planning/PLANNING_README.md`
- `docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md`
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Pre-planning interface + orchestration:
- `docs/project_management/system/prompts/planning/workstream_triage_agent.md` (source of `PM_PWS_INDEX`)
- `docs/project_management/system/scripts/planning/validate_pws_index.py`
- `docs/project_management/system/scripts/planning/pm_pws_index_extract.py`
- `docs/project_management/system/scripts/planning/pm_pws_plan.py`
- `docs/project_management/system/scripts/planning/run_pws_agent.sh`

Task graph + slice/cp validators:
- `docs/project_management/system/scripts/planning/validate_tasks_json.py`
- `docs/project_management/system/scripts/planning/validate_slice_specs.py`
- `docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py`
- `docs/project_management/system/scripts/planning/lint.sh` (what the quality gate runs)

Templates / reference implementations:
- `docs/project_management/system/templates/kickoff/`
- `docs/project_management/system/scripts/planning/new_feature.sh`
- `docs/project_management/system/scripts/planning/scaffold_pre_planning_pack.sh` (pre-planning scaffold defaults for `tasks.json` meta)
- `docs/project_management/system/scripts/planning/ensure_kickoff_prompt_sentinel.py`
- `docs/project_management/system/schemas/tasks.schema.json`

Live examples:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/` (WDRA; contains the allowlist failure + draft patch)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/` (schema v4 triad-aligned example pack)

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

### Step 5.5 — Harden the no-pause auto-heal path (BEDPM postmortem; current direction)

Step 5 remains useful as historical/reference context for the older operator-controlled expansion model.

The current direction is stricter:
- the full-planning orchestrator should **not** pause for operator input,
- it should automatically resolve safe allowlist issues,
- it should resume the same Codex session,
- and it should finish with a pack that is not only mechanically green, but also **semantically aligned** with the accepted slice plan.

This step exists because the first end-to-end Step 4 run that completed successfully exposed a new failure mode:
the orchestrator can now “auto-heal” long enough to get a green exit, but still allow a **false-green semantic collapse** if the validator stack is too weak.

#### BEDPM failure specimen (what actually happened)

Observed pack:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/`

Observed run:
- `make pm-full-planning-orchestrate FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager"`

What happened:
1) `BEDPM-PWS-contract`, `BEDPM-PWS-docs_validation`, and the three slice-spec PWSes all completed cleanly.
2) `BEDPM-PWS-tasks_checkpoints` attempt 0 produced the **correct** execution-ready 3-slice triad graph:
   - `BEDPM0`
   - `BEDPM1`
   - `BEDPM2` as the checkpoint-boundary slice
3) The runner then failed on:
   - `validate_ci_checkpoint_plan.py`
   - because `pre-planning/ci_checkpoint_plan.md` still modeled only `["BEDPM0"]`.
4) The PWS agent wrote a good allowlist request + draft patch for the out-of-allowlist fix.
5) The orchestrator did **not** process the request, because the request schema did not match what the orchestrator parsed.
6) The orchestrator then resumed the same session multiple times with only a generic “runner failed; fix within allowlist” message.
7) After repeated retries, the session eventually “solved” the mismatch by collapsing `tasks.json` back to the single-slice `BEDPM0` model so the stale checkpoint plan would pass.
8) The run exited green — but the resulting pack no longer matched the accepted 3-slice planning surfaces.

This is the exact class of problem Step 5.5 must eliminate.

#### Root causes (separate the system defect from the pack defect)

##### A) Allowlist request contract mismatch (system defect)

The no-pause auto-heal loop currently assumes a specific request schema, but the prompts do not pin it precisely enough.

Observed in BEDPM:
- The agent wrote `allowlist_request.json` with:
  - `requested_paths`
  - `reason`
  - additional diagnostic context
- The orchestrator only parsed:
  - `requested_tracked_paths`

Result:
- the request was logically correct,
- the orchestrator treated it as empty,
- auto-heal never granted or routed the needed write,
- the same blocked session kept getting resumed with no real state change.

##### B) Split-brain slice inventory in pre-planning artifacts (pack defect)

BEDPM entered full planning with contradictory slice inventory across canonical inputs:
- `pre-planning/minimal_spec_draft.md` modeled `BEDPM0`, `BEDPM1`, `BEDPM2`
- `pre-planning/workstream_triage.md` modeled 3 slice-spec PWSes and assumed the checkpoint boundary was at the end of `BEDPM2`
- `plan.md` modeled `BEDPM0 -> BEDPM1 -> BEDPM2`
- existing slice specs existed for all three slices

But at the same time:
- `pre-planning/spec_manifest.md` still described a single-slice `BEDPM0` plan
- `pre-planning/impact_map.md` still referenced only `slices/BEDPM0/BEDPM0-spec.md`
- `pre-planning/alignment_report.md` still described `tasks.json` as a `BEDPM0` triad
- `pre-planning/ci_checkpoint_plan.md` still declared `["BEDPM0"]`

So the pack was already internally inconsistent before `tasks_checkpoints` started.

##### C) Validator stack allowed a false-green by shrinking the slice set (system defect)

The final green run was not achieved by repairing the stale checkpoint plan.

It was achieved by changing `tasks.json` until the validators no longer saw `BEDPM1` and `BEDPM2` as required.

Key reason:
- `validate_slice_specs.py` derives the slice set from `tasks.json` itself (via `*-code` / `*-test` / `*-integ` tasks),
- so once `BEDPM1` and `BEDPM2` disappear from `tasks.json`, their slice specs effectively become invisible to that validator,
- and `validate_ci_checkpoint_plan.py` then happily accepts the now-single-slice model.

Result:
- the run can exit green,
- while `minimal_spec_draft.md`, `workstream_triage.md`, `plan.md`, and existing slice specs still describe a 3-slice pack.

That is a correctness failure, not just a UX issue.

##### D) Resume feedback was too generic to converge safely (system defect)

The resume messages during the retry loop were too weak:
- they did not inline the exact failing validator message,
- they did not say the allowlist request was malformed / unparseable,
- they pointed the agent at `stderr.log`, which may be empty or unhelpful for this failure class.

That makes the resumed session rely on memory and inference instead of being told the exact machine failure.

##### E) Ownership / authority mismatch around `pre-planning/ci_checkpoint_plan.md`

`validate_ci_checkpoint_plan.py` validates the authoritative checkpoint plan file under:
- `<FEATURE_DIR>/pre-planning/ci_checkpoint_plan.md`

But in BEDPM’s `PM_PWS_INDEX`, no full-planning PWS owned that tracked file.

That means a compliant `tasks_checkpoints` agent could diagnose the real fix, but it could not land it without allowlist expansion or routing.

In the no-pause model, this is acceptable only if the orchestrator can **reliably** auto-grant or auto-route.
When that auto-heal path fails, the run has no safe way to converge.

#### Step 5.5 target state

The full-planning orchestrator is only “done” when all of the following are true:

1) It can automatically process allowlist requests without operator intervention.
2) It can distinguish:
   - malformed allowlist requests,
   - legitimate requests for unowned paths,
   - requests that should route to another already-run owning PWS.
3) It resumes the same session with the exact machine failure context.
4) It cannot obtain a green run by silently dropping accepted slices from `tasks.json`.
5) It enforces coherence across the accepted slice inventory, checkpoint plan, slice specs, and task graph.

#### Required Step 5.5 hardening work

##### 1) Canonicalize `allowlist_request.json` (non-negotiable)

Define one exact machine contract for:
- `<PACK>/logs/pws/<PWS_ID>/allowlist_request.json`

Minimum required fields:
- `pws_id`
- `requested_tracked_paths` (array; canonical field name)
- `reason`

Recommended fields:
- `required_updates`
- `recommended_follow_on_paths`
- `consistency_evidence`
- `blocked_validation`

Implementation requirements:
- prompts must tell agents the exact field names,
- the orchestrator must parse the canonical name,
- and, during migration, it should also accept legacy aliases like `requested_paths` so a well-reasoned request is not discarded due to field drift.
- The canonical parser should preserve optional diagnostic keys (for example `required_updates`, `recommended_follow_on_paths`, `consistency_evidence`, `blocked_validation`) without requiring them.

If the request file exists but is malformed, the orchestrator should:
- preserve it,
- emit a deterministic error summary,
- and resume the same session with an explicit “malformed allowlist request” message that names the missing/incorrect keys.

##### 2) Add an accepted-slice coverage validator (non-negotiable)

The validator stack must stop using `tasks.json` as the only source of slice truth.

We need a hard gate that compares the accepted slice inventory across:
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/workstream_triage.md`
- `plan.md`
- `slices/<SLICE_ID>/<SLICE_ID>-spec.md`
- `pre-planning/ci_checkpoint_plan.md`
- `tasks.json`
- and, when present, `pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`, and `pre-planning/alignment_report.md`

Authority nuance:
- `pre-planning/minimal_spec_draft.md` is the draft starting skeleton and default CI-checkpoint input.
- `pre-planning/workstream_triage.md` may explicitly recommend `ADD` / `SPLIT` / `MERGE` / `RENAME` edits without mutating `minimal_spec_draft.md`.
- Once full-planning surfaces (`workstream_triage`, `plan.md`, slice specs, checkpoint plan) adopt that revised skeleton, the validator must treat the triage-adopted slice set as the accepted inventory and then fail any stale planning surfaces that still describe the old draft set.

Minimum required invariants:
- every accepted slice must have a slice spec,
- every accepted slice must appear in `tasks.json`,
- every accepted slice must appear in the checkpoint plan in deterministic contiguous order,
- `tasks.json` must not silently drop accepted slices,
- slice ids referenced by PWS topology, slice specs, and checkpoint plan must agree.

Implement this as a dedicated validator:
- `docs/project_management/system/scripts/planning/validate_slice_inventory_coherence.py`
- It should support:
  - `pre_tasks_checkpoints` phase (used before `*-PWS-tasks_checkpoints`)
  - `execution_ready` phase (used after `tasks_checkpoints` and by planning lint)

The end result must be:
- green is impossible if `BEDPM1` / `BEDPM2` still exist in accepted planning docs but disappear from `tasks.json`.

##### 3) Add an earlier coherence gate before `tasks_checkpoints`

The BEDPM run should have failed **before** `tasks_checkpoints` attempt 0 finished, because the pack-local inputs were already contradictory.

Add a hard preflight/coherence gate for full planning that checks:
- slice inventory agreement across pre-planning artifacts,
- checkpoint-plan slice list agreement with the accepted slice skeleton,
- no “single-slice vs multi-slice” drift across canonical inputs.

This gate should run before the late `tasks_checkpoints` phase so the pack enters task wiring with one coherent slice model.

##### 4) Strengthen `tasks_checkpoints` prompt to forbid semantic collapse

Current prompt guidance already says:
- do not disable automation / cross-platform just to get green

That needs to be tightened further.

Explicitly forbid:
- collapsing the accepted slice set,
- merging away accepted slices,
- deleting slice tasks for accepted slices,
- or re-authoring the task graph to match stale upstream docs,

unless the accepted slice inventory itself has been changed by the proper owning planning surface.

In other words:
- `tasks_checkpoints` may wire tasks for the accepted slice set,
- but it must not rewrite the accepted slice plan to satisfy a validator.

##### 5) Make resume messages machine-specific and stateful

For runner / validator failures, the orchestrator should resume the same session with:
- the exact failing validator name,
- the exact failing error line,
- the relevant tracked file(s),
- whether an allowlist request was detected / granted / rejected / malformed,
- and what happened in any routed owner PWS runs.

Do **not** send only a generic:
- “runner failed; inspect logs”

The session should be resumed with enough concrete machine state that it can take the correct next action immediately.

##### 6) Resolve authority/ownership for checkpoint-plan repairs

The system must make one of these true:
- `pre-planning/ci_checkpoint_plan.md` is owned by a full-planning PWS when it remains authoritative during task wiring, or
- the no-pause orchestrator must reliably auto-grant unowned checkpoint-plan edits to the blocked requestor, or
- checkpoint-plan authority must be promoted to a full-planning-owned surface before `tasks_checkpoints` runs.

What is **not** acceptable is the BEDPM state:
- authoritative validator input,
- no owning PWS,
- and an auto-heal loop that can fail to honor a correct request.

##### 7) Upgrade the definition of orchestrator success

For the no-pause model, “success” now means:
- runner gates pass,
- micro-lint passes,
- allowlist requests are fully resolved or absent,
- accepted slice inventory is still intact,
- no accepted slice specs are orphaned,
- checkpoint plan, `tasks.json`, and slice specs agree on slice ordering and boundaries,
- and the session did not achieve green by shrinking the declared work.

#### BEDPM as the regression test for Step 5.5

Use:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/`

as the canonical regression specimen for this step.

Step 5.5 is complete only when a rerun of BEDPM produces one of two acceptable outcomes:

1) **Correct repair path**:
   - the 3-slice model is preserved,
   - the checkpoint plan is auto-healed / granted / routed correctly,
   - `tasks.json`, checkpoint plan, and slice specs all agree,
   - and the run exits green.

2) **Correct hard failure path**:
   - the run stops with a deterministic coherence error **before** any false-green collapse is possible,
   - and the failing reason points at the contradictory planning surfaces directly.

It must **not** be possible to get a green run by collapsing the 3-slice pack to `BEDPM0` only while the rest of the planning pack still declares `BEDPM1` and `BEDPM2`.

#### Required reading for Step 5.5

Core orchestration / prompts:
- `PWS_FULL_PLANNING_ORCHESTRATION_V1.md` (this document)
- `docs/project_management/system/scripts/planning/full_planning_orchestrate.sh`
- `docs/project_management/system/scripts/planning/run_pws_agent.sh`
- `docs/project_management/system/prompts/planning/pws_generic_agent.md`
- `docs/project_management/system/prompts/planning/pws_tasks_checkpoints_agent.md`

Validators:
- `docs/project_management/system/scripts/planning/validate_tasks_json.py`
- `docs/project_management/system/scripts/planning/validate_slice_specs.py`
- `docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py`
- `docs/project_management/system/scripts/planning/validate_pws_index.py`

BEDPM failure artifacts:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/full_planning_orchestrator/20260306-022442/summary.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/pws/BEDPM-PWS-tasks_checkpoints/allowlist_request.json`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/pws/BEDPM-PWS-tasks_checkpoints/draft.patch`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/pws/BEDPM-PWS-tasks_checkpoints/runs/20260306T031452Z/last_message.run.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/pws/BEDPM-PWS-tasks_checkpoints/runs/20260306T034732Z/last_message.run.md`

BEDPM pack inputs that drifted:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

### Step 6 — Add safe parallelism (worktrees)

- Only after Step 4/5 is stable:
  - run disjoint PWS concurrently using git worktrees/branches,
  - route shared-file work to an explicit integration/apply step,
  - preserve single-writer invariants (`tasks.json`, often `plan.md`).

## Open questions (explicitly not decided yet)

- Exact “integration apply” mechanics:
  - manual operator step vs orchestrator-assisted patch apply vs a dedicated integration PWS.
- Exact locations/names of prompts for each `role` (contract, slice_spec, docs_validation, tasks_checkpoints, etc.).
