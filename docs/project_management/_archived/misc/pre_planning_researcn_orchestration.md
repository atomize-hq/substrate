## Pre‑Planning Research Orchestration (Staggered + Sentinel‑Gated + Log‑Archived)

### Summary
Build a **scripted orchestrator** (plus a Make target) that runs the pre‑planning research chain with **overlapped/staggered agent starts**, where downstream agents can start early but must **only write to pack `logs/`** until the upstream step’s **stable `last_message.md` sentinel** exists (and the pack is clean).  
Instead of deleting stale sentinels/logs, the orchestrator **archives** existing step log dirs by `mv` to `*_run_N` directories.

This produces/updates these tracked pre‑planning artifacts (minimal set):
- `spec_manifest.md`
- `impact_map.md`
- `minimal_spec_draft.md` (pack‑level alignment backbone; explicitly pre‑planning only)
- `ci_checkpoint_plan.md` (+ sometimes `tasks.json`, when cross‑platform automation requires wiring fixes)
- plus a logs‑only **workstream triage draft** (no tracked output)

---

## 1) Canonical paths + semantics

### 1.1 Stable step log dirs (pack‑local, gitignored)
All under `<FEATURE_DIR>/logs/` (example `<FEATURE_DIR>` is a Planning Pack dir like `docs/project_management/packs/active/<feature>`):
- `<FEATURE_DIR>/logs/spec-manifest/`
- `<FEATURE_DIR>/logs/impact-map/`
- `<FEATURE_DIR>/logs/min-spec-draft/`
- `<FEATURE_DIR>/logs/CI-checkpoint/`
- `<FEATURE_DIR>/logs/workstream-triage/`

### 1.2 Stable sentinels (no `*.latest.md`)
Each step dir contains:
- `last_message.md` (stable completion sentinel used for polling/gating)
- `handoff.md` (mid‑run “start the next agent now” signal)
- `stderr.log` (stable Codex stderr stream for this step; created/truncated at step start so it can be tailed while the agent runs)
- `codex.pid` (stable PID file while Codex is running; remove when Codex exits; triad-format: if present at launch time, wait for the existing Codex PID when it is still a live Codex process; otherwise remove as stale)

**Meaning of `last_message.md`**
- Exists only when the step has **successfully completed** (so downstream gating by file existence is safe).
- It must not be left over from a prior run at the stable location; stale is prevented by archiving (next section).
- If Codex fails to write `--output-last-message` (interruption/crash), the runner must still write a **run-scoped** placeholder summary (triad-style) so operators have a deterministic breadcrumb:
  - Write/overwrite: `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/last_message.run.md` (generated stub with exit code + pointers to stdout/stderr)
  - Do **not** copy that stub into the stable `<FEATURE_DIR>/logs/<step>/last_message.md` unless the step is considered successful.

### 1.3 Archive‑instead‑of‑delete contract (per step; most resilient)
When a step is being rerun, if `<FEATURE_DIR>/logs/<step>/` exists, the orchestrator archives it by renaming the entire directory:

- `mv "<FEATURE_DIR>/logs/<step>" "<FEATURE_DIR>/logs/<step>_run_N"`

Where `N` is computed **per step**:
- If no `<step>_run_<number>` dirs exist → `N=1`
- Else → `N = max(existing numbers) + 1`

Then recreate the fresh working dir:
- `mkdir -p "<FEATURE_DIR>/logs/<step>"`

This per‑step numbering is the most resilient when you rerun only some steps.

### 1.4 Tracked outputs per agent (allowlist)
These are the only tracked files each agent step is allowed to change:

- **spec_manifest** → `<FEATURE_DIR>/spec_manifest.md`
- **impact_map** → `<FEATURE_DIR>/impact_map.md`
- **min_spec_draft** → `<FEATURE_DIR>/minimal_spec_draft.md`
- **ci_checkpoint** → `<FEATURE_DIR>/ci_checkpoint_plan.md` and (only if needed) `<FEATURE_DIR>/tasks.json`
- **workstream_triage** → no tracked outputs (logs only)

---

## 2) Orchestrator (script + Make target)

This layer is explicitly split into:
- **Automation (scripts)**: deterministic mechanics (archiving, dispatching focused Codex runs, wait/monitor, commit, summary).
- **Focused planning agents (Codex instances)**: the actual LLM “agent work” for each step, launched headlessly by the automation via `run_planning_agent.sh`.
- **Orchestrator agent (operator)**: decisions + interventions when inputs are missing/ambiguous or when runs fail; does not manually “run” the chain step-by-step.

### 2.1 Automated via scripts (canonical)
Add:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Makefile` target: `pm-pre-planning-research`

Entry point (canonical):
- `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/active/<feature>" [START_AT=impact-map] [POLL_S=60]`

Script responsibilities (must be fully automated):
1) **Mechanical preflight (fail-fast; no “fixing” here)**
   - Require repo root + clean orchestration checkout (`git status --porcelain=v1` empty).
   - Require `FEATURE_DIR` exists and contains `tasks.json`.
2) **Prepare log dirs**
   - For `START_AT` and downstream steps only:
     - Archive existing `<FEATURE_DIR>/logs/<step>/` → `<FEATURE_DIR>/logs/<step>_run_N/` (rename whole dir).
     - Recreate fresh `<FEATURE_DIR>/logs/<step>/`.
3) **Launch + monitor the chain (staggered overlap)**
   - Launch each step’s **focused planning agent** by calling the planning runner (`run_planning_agent.sh`) in a background process.
     - Note: `run_planning_agent.sh` is a *runner/dispatcher script* that starts a Codex instance with the step prompt; it is automation glue, not the “orchestrator agent”.
   - For each step:
     - Ensure `<FEATURE_DIR>/logs/<step>/stderr.log` is created/truncated at start and streams while running.
     - Ensure `<FEATURE_DIR>/logs/<step>/codex.pid` is written while Codex runs and removed at exit.
   - Trigger downstream starts:
     - Start next step when upstream writes `<FEATURE_DIR>/logs/<upstream>/handoff.md`.
     - Fallback: if upstream exits successfully before emitting `handoff.md`, treat that as implied handoff and start downstream.
     - Never start downstream if upstream fails.
4) **Commit tracked artifacts (deterministic)**
   - After each step completes successfully, commit only the step’s allowlisted tracked outputs (skip commit if no tracked changes).
   - Commit messages are standardized (pre‑planning; per step) to keep runs easy to audit.
5) **Stop-on-failure**
   - If any step fails (non‑zero) or violates output allowlists:
     - Stop orchestration immediately.
     - Attempt to terminate already-launched downstream runners.
     - Do not produce downstream stable `last_message.md` sentinels.
6) **Write orchestrator summary**
   - Always write: `<FEATURE_DIR>/logs/pre_planning_wrapper/<UTC_TS>/summary.md`
   - Include: steps started/completed, exit codes, commit SHAs, stable sentinel paths, and pointers to per-step logs.

### 2.2 Orchestrator agent responsibilities (operator; not scripted)
The orchestrator agent is responsible for decisions and repo edits that require judgment.

Before running the script:
- Choose `FEATURE_DIR` and (if needed) `START_AT`.
- Ensure required inputs exist (especially ADR refs/paths in `tasks.json` for strict packs); if missing, add them and commit *before* running.
- Ensure the orchestration checkout is clean (commit/stash unrelated changes).

During a run (if monitoring is desired):
- Tail `<FEATURE_DIR>/logs/<step>/stderr.log` to observe progress without increasing polling frequency.
- If the script fails, inspect:
  - `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/last_message.run.md`
  - `<FEATURE_DIR>/logs/<step>/stderr.log`
  - `<FEATURE_DIR>/logs/pre_planning_wrapper/<UTC_TS>/summary.md`

After a run:
- Review the committed tracked artifacts (`spec_manifest.md`, `impact_map.md`, `minimal_spec_draft.md`, and if applicable `ci_checkpoint_plan.md` / `tasks.json`).
- Decide whether follow-ups require rerunning from a specific step (`START_AT=...`) vs deferring to full planning.
- Confirm `minimal_spec_draft.md` is clearly labeled pre‑planning only (and plan its deletion/retirement during full planning).

---

## 3) Update the planning runner to support stable step dirs + new agents

Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/run_planning_agent.sh`

Changes (decision‑complete):
1) **Add new `--agent` values**
   - existing: `spec_manifest`, `impact_map`
   - add:
     - `min_spec_draft`
     - `ci_checkpoint`
     - `workstream_triage`
2) **Map agent → stable step dir name**
   - `spec_manifest` → `spec-manifest`
   - `impact_map` → `impact-map`
   - `min_spec_draft` → `min-spec-draft`
   - `ci_checkpoint` → `CI-checkpoint`
   - `workstream_triage` → `workstream-triage`
3) **Write artifacts under stable step dir**
   - Run artifacts go under:
     - `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/`
   - Keep:
     - `prompt.md`, `stdout.txt`/`events.jsonl`
     - `last_message.run.md` (Codex `--output-last-message` target)
   - Also maintain stable “live” artifacts at the step root (triad-format):
     - `<FEATURE_DIR>/logs/<step>/stderr.log` (Codex stderr; streams while running)
     - `<FEATURE_DIR>/logs/<step>/codex.pid` (Codex PID while running)
   - Implementation requirement (to match triad behavior):
     - Launch Codex in the background so `codex.pid` can be written immediately.
     - Redirect stderr to `<FEATURE_DIR>/logs/<step>/stderr.log` (truncate at start), so it streams while running.
     - Remove `<FEATURE_DIR>/logs/<step>/codex.pid` after Codex exits.
4) **Stable sentinel creation rule**
   - Codex writes to `last_message.run.md` only.
   - The runner copies `last_message.run.md` → `<FEATURE_DIR>/logs/<step>/last_message.md` only when the step is **successful**:
     - Codex exit code is `0`, and
     - output allowlist passes, and
     - (for tracked-output steps) git diff within `<FEATURE_DIR>` only contains allowed files.
5) **Output allowlist enforcement**
   - Replace “exactly one changed file” with:
     - For normal steps: `changed_files ⊆ allowed_outputs` and `changed_files` may be empty.
     - For logs‑only (`workstream_triage`): require `changed_files` empty.
6) **Prompt prelude must match allowlists (fix current mismatch)**
   - Today the runner injects:
     - “Only write/overwrite: `<output>`”
     - “Do not edit any other files”
   - That injected prelude must be updated so it does **not** contradict the orchestration model:
     - Allow *untracked* writes under `<FEATURE_DIR>/logs/<step>/**` (for `handoff.md`, scratch, etc.).
     - Keep a strict tracked-file allowlist for canonical outputs.
7) **Prompts**
   - Add prompt file selection for the new agents:
     - `min_spec_draft` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md`
     - `ci_checkpoint` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md`
     - `workstream_triage` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md`

---

## 4) Prompt changes (agents overlap + gating + handoffs)

### 4.0 Prompt inventory audit (as-is)
This section is grounded in the current repo state under:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/`

Present today:
- Focused planning agents (used by `run_planning_agent.sh`):
  - `spec_manifest` → `spec_manifest_agent.md`
  - `impact_map` → `impact_map_agent.md`
  - `min_spec_draft` → `min_spec_draft_agent.md`
  - `ci_checkpoint` → `ci_checkpoint_agent.md`
  - `workstream_triage` → `workstream_triage_agent.md`
- Wrapper prompt (operator/orchestrator agent instructions):
  - `pre_planning_wrapper.md`
- Full planning kickoff prompt (not used by this pre-planning orchestration):
  - `planning_kickoff_prompt.md`

Implementation notes (current):
- Focused agent prompts are prompt-only payloads (` ```md ... ``` `).
- The runner prelude must allow logs writes under the stable step directory, and enforce tracked-file allowlists.
- The wrapper prompt is script-driven (calls `make pm-pre-planning-research`).

Prompt format contract (runner-dependent):
- Each focused prompt file should be exactly one fenced payload block starting with ```` ```md ```` because the runner extracts the first ` ```md` block as stdin to Codex.
  - The runner ignores any text outside the first ` ```md` block; for clarity and consistency, avoid putting anything outside that fence.
- Prompts must contain prompt text only. Any rationale/guide belongs in standards under `docs/project_management/system/standards/**`.

### 4.1 Existing prompts (required behaviors)
Note: this prompt set depends on the runner prelude allowing logs writes under the stable step log directory (Section 3.6).

Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/spec_manifest_agent.md`

Add requirements:
- Update the output constraints to match the orchestration model:
  - Keep the tracked-file single-output contract: only write/overwrite `<FEATURE_DIR>/spec_manifest.md`.
  - Allow logs-only side effects under: `<FEATURE_DIR>/logs/spec-manifest/**` (scratch + `handoff.md`).
  - Explicitly forbid writing anywhere else (including other tracked pack files).
- Emit mid‑run handoff marker (logs only; not tracked):
  - Write/overwrite: `<FEATURE_DIR>/logs/spec-manifest/handoff.md`
  - Write it when:
    - surfaces are enumerated, required doc set chosen, and ownership matrix is drafted enough for impact-map to start discovery.

Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/impact_map_agent.md`

Change behavior to support overlap:
- Update the output constraints to match the orchestration model:
  - Keep the tracked-file single-output contract: only write/overwrite `<FEATURE_DIR>/impact_map.md`.
  - Allow logs-only side effects under: `<FEATURE_DIR>/logs/impact-map/**` (scratch + `handoff.md`).
  - Explicitly forbid writing anywhere else (including other tracked pack files).
- Phase A (start immediately; logs only):
  - Do repo scans, queued ADR scan, pack scan, and draft an initial touch set in a scratch file under:
    - `<FEATURE_DIR>/logs/impact-map/scratch.md`
  - Emit `<FEATURE_DIR>/logs/impact-map/handoff.md` once the scratch touch set and main implication buckets exist.
- Phase B (canonical write gate):
  - Poll until BOTH are true:
    - `<FEATURE_DIR>/logs/spec-manifest/last_message.md` exists
    - `git status --porcelain=v1 -- "<FEATURE_DIR>"` is empty
    - Default poll interval: `sleep 60` between checks.
  - Then write tracked output:
    - `<FEATURE_DIR>/impact_map.md` (only tracked file)

### 4.2 Additional prompts (prompt‑only; no preamble)
Ensure these prompts exist and enforce the overlap/gating behavior:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md`
  - Output (tracked): `<FEATURE_DIR>/minimal_spec_draft.md` only
  - Overlap behavior:
    - Start early; do scratch notes under `<FEATURE_DIR>/logs/min-spec-draft/`
    - Emit `<FEATURE_DIR>/logs/min-spec-draft/handoff.md` once you have a coherent outline
    - Poll until:
      - `<FEATURE_DIR>/logs/impact-map/last_message.md` exists, and
      - feature dir clean
    - Default poll interval: `sleep 60` between checks.
    - Then write `minimal_spec_draft.md`
  - Content contract:
    - explicitly labeled **Pre‑Planning Only** and must be deleted/retired in full planning
    - cross‑cutting defaults/precedence/invariants/failure posture only (alignment backbone)
  - Required reading (minimum; cite these explicitly in the prompt):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md` (surface inventory discipline)
    - `<FEATURE_DIR>/spec_manifest.md`
    - `<FEATURE_DIR>/impact_map.md`

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md`
  - Conditional behavior:
    - If pack is not cross‑platform automation (detect via `tasks.json`), write “not applicable” into logs and still emit `handoff.md` (logs only), and exit without tracked changes.
  - If applicable:
    - Start early; scratch under `<FEATURE_DIR>/logs/CI-checkpoint/`
    - Emit `<FEATURE_DIR>/logs/CI-checkpoint/handoff.md` once you’ve drafted checkpoint group boundaries and gates.
    - Poll until:
      - `<FEATURE_DIR>/logs/min-spec-draft/last_message.md` exists, and
      - feature dir clean
    - Default poll interval: `sleep 60` between checks.
    - Then update:
      - `<FEATURE_DIR>/ci_checkpoint_plan.md`
      - `<FEATURE_DIR>/tasks.json` only if required to satisfy the checkpoint wiring rules
    - Must pass:
      - `python3 /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"`
  - Required reading (minimum; cite these explicitly in the prompt):
    - `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`
    - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md` (tasks.json wiring expectations)
    - `<FEATURE_DIR>/impact_map.md`
    - `<FEATURE_DIR>/tasks.json`
    - `<FEATURE_DIR>/ci_checkpoint_plan.md` (if it already exists)

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md`
  - Logs‑only output:
    - Draft: `<FEATURE_DIR>/logs/workstream-triage/workstream_triage_draft.md`
    - Emit `<FEATURE_DIR>/logs/workstream-triage/handoff.md` when the draft has a usable initial structure (optional, since this is the end of the chain).
  - Gate finalization (still logs-only):
    - Poll until:
      - `<FEATURE_DIR>/logs/CI-checkpoint/last_message.md` exists (or CI checkpoint step declared non-applicable by its logs), and
      - feature dir clean
    - Default poll interval: `sleep 60` between checks.
  - Draft must include:
    - Proposed workstreams (parallelizable clusters) + rationale
    - Sequencing/gating constraints
    - Explicit “unknowns/follow-ups” to resolve in full planning
    - References/links to the stable step `last_message.md` files (so future agents can ingest filtered summaries)
  - Required reading (minimum; cite these explicitly in the prompt):
    - `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (repo root; canonical semantics)
    - `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
    - `<FEATURE_DIR>/spec_manifest.md`
    - `<FEATURE_DIR>/impact_map.md`
    - `<FEATURE_DIR>/minimal_spec_draft.md`
    - `<FEATURE_DIR>/ci_checkpoint_plan.md` (if applicable)

---

## 5) Standards + wrapper prompt updates

### 5.1 Update the pre‑planning wrapper standard
Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md`

Must include:
- The full 5‑step chain (spec-manifest → impact-map → min-spec-draft → CI-checkpoint → workstream-triage)
- The stable sentinel paths and `handoff.md` trigger behavior
- The archive‑instead‑of‑delete `_run_N` policy (per step)
- Minimal spec draft lifecycle: explicitly pre‑planning only; delete/retire during full planning

### 5.2 Update the wrapper prompt to point to the canonical Make target
Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/pre_planning_wrapper.md`

Change it from “manual steps” to:
- Run: `make pm-pre-planning-research FEATURE_DIR="<...>" [START_AT=...]`
- Then review:
  - commits
  - stable sentinels in step dirs
  - workstream triage draft in logs
- Keep prompt text only; no model/profile references; no `test -d ...` style assertions.

---

## 6) Validation / acceptance tests (manual + deterministic)
Run on at least one real pack:
1) Full run:
   - `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/active/<feature>"`
   - Verify:
     - stable `last_message.md` exists in each step dir
     - while a step is running: `<FEATURE_DIR>/logs/<step>/codex.pid` exists and `<FEATURE_DIR>/logs/<step>/stderr.log` streams
     - after a step completes: `<FEATURE_DIR>/logs/<step>/codex.pid` is removed
     - tracked outputs updated/committed where applicable
     - workstream triage draft exists (logs-only)
2) Partial rerun from mid‑chain:
   - `make pm-pre-planning-research FEATURE_DIR="..." START_AT=impact-map`
   - Verify:
     - `logs/impact-map` and downstream step dirs were archived to `*_run_N` and recreated
     - `logs/spec-manifest` was NOT moved
     - downstream gating did not proceed using archived/stale sentinels
3) Failure mode:
   - Force an agent to violate allowlist (temporarily, in a dev test) and confirm:
     - runner exits non‑zero
     - orchestrator stops and does not generate downstream stable sentinels

---

## Assumptions / defaults
- Log dir names are exactly: `spec-manifest`, `impact-map`, `min-spec-draft`, `CI-checkpoint`, `workstream-triage` (case‑sensitive).
- Archive numbering is **per step** (`<step>_run_N`) to maximize resilience for partial reruns.
- Orchestrator commits are the canonical way to keep `<FEATURE_DIR>` clean between canonical writes.
- `minimal_spec_draft.md` is tracked and committed, but explicitly pre‑planning only and must be deleted/retired during full planning.
