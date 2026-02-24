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

**Meaning of `last_message.md`**
- Exists only when the step has completed successfully enough for downstream to proceed.
- It must not be left over from a prior run at the stable location; stale is prevented by archiving (next section).

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

### 2.1 Add a new orchestrator script
Create:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh`

Inputs (env vars or flags; pick one style and standardize):
- `FEATURE_DIR` (required)
- `START_AT` (optional; default `spec-manifest`)
  - allowed: `spec-manifest | impact-map | min-spec-draft | CI-checkpoint | workstream-triage`
- `POLL_S` (optional; default `2` seconds)

Behavior:
1) **Preflight**
   - Ensure run from repo root.
   - Ensure orchestration checkout is clean: `git status --porcelain=v1` must be empty.
   - Ensure `FEATURE_DIR` exists and contains `tasks.json`.
2) **Archive log dirs for the rerun range**
   - Determine step chain: `spec-manifest → impact-map → min-spec-draft → CI-checkpoint → workstream-triage`
   - Archive (rename whole dir) for `START_AT` and all downstream steps only.
   - Do not touch upstream step log dirs.
3) **Launch the chain with staggered overlap**
   - Start the `spec_manifest` runner (foreground or background; recommended: background so orchestration can watch for handoffs).
   - Watch for `<FEATURE_DIR>/logs/spec-manifest/handoff.md` to appear; when it does, start `impact_map` runner.
   - Similarly:
     - `impact-map/handoff.md` → start `min_spec_draft`
     - `min-spec-draft/handoff.md` → start `ci_checkpoint`
     - `CI-checkpoint/handoff.md` → start `workstream_triage`
   - Fallback rule: if an upstream runner exits before writing `handoff.md`, treat that as “handoff implied” and start the next step immediately (unless upstream failed).
4) **Commit strategy**
   - After each step runner exits successfully, the orchestrator commits its tracked outputs (if any) before downstream is allowed to finalize writes:
     - spec_manifest: `git add <FEATURE_DIR>/spec_manifest.md && git commit -m "docs: pre-planning spec manifest"`
     - impact_map: `git add <FEATURE_DIR>/impact_map.md && git commit -m "docs: pre-planning impact map"`
     - min_spec_draft: `git add <FEATURE_DIR>/minimal_spec_draft.md && git commit -m "docs: pre-planning minimal spec draft"`
     - ci_checkpoint: `git add <FEATURE_DIR>/ci_checkpoint_plan.md <FEATURE_DIR>/tasks.json && git commit -m "docs: pre-planning CI checkpoint plan"` (only include `tasks.json` if changed)
     - workstream_triage: no commit
   - If a step made no tracked changes, skip committing for that step (but still produce its stable sentinel).
5) **Failure handling**
   - If any step runner exits non‑zero or violates output allowlists:
     - Stop orchestration immediately.
     - Terminate any already-launched downstream runners (cleanly if possible).
     - Do **not** create downstream stable `last_message.md` sentinels.
6) **Orchestrator run log**
   - Always write a run‑scoped orchestration summary under:
     - `<FEATURE_DIR>/logs/pre_planning_wrapper/<UTC_TS>/summary.md`
   - Include:
     - which steps ran/skipped
     - commit SHAs per step
     - stable sentinel paths
     - “next actions / follow-ups” extracted from agent last_message files

### 2.2 Add a Make target
Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Makefile`

Add:
- `pm-pre-planning-research` target that shells out to `pre_planning_research_orchestrate.sh`

Usage:
- `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/active/<feature>" [START_AT=impact-map]`

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
     - `prompt.md`, `stdout.txt`/`events.jsonl`, `stderr.txt`
     - `last_message.run.md` (Codex `--output-last-message` target)
4) **Stable sentinel creation rule**
   - Codex writes to `last_message.run.md` only.
   - The runner copies `last_message.run.md` → `<FEATURE_DIR>/logs/<step>/last_message.md` only after:
     - output allowlist passes, and
     - (for tracked-output steps) git diff within `<FEATURE_DIR>` only contains allowed files.
5) **Output allowlist enforcement**
   - Replace “exactly one changed file” with:
     - For normal steps: `changed_files ⊆ allowed_outputs` and `changed_files` may be empty.
     - For logs‑only (`workstream_triage`): require `changed_files` empty.
6) **Prompts**
   - Add prompt file selection for the new agents:
     - `min_spec_draft` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md`
     - `ci_checkpoint` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md`
     - `workstream_triage` → `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md`

---

## 4) Prompt changes (agents overlap + gating + handoffs)

### 4.1 Update existing prompts
Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/spec_manifest_agent.md`

Add requirements:
- Emit mid‑run handoff marker (logs only; not tracked):
  - Write/overwrite: `<FEATURE_DIR>/logs/spec-manifest/handoff.md`
  - Write it when:
    - surfaces are enumerated, required doc set chosen, and ownership matrix is drafted enough for impact-map to start discovery.

Update:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/impact_map_agent.md`

Change behavior to support overlap:
- Phase A (start immediately; logs only):
  - Do repo scans, queued ADR scan, pack scan, and draft an initial touch set in a scratch file under:
    - `<FEATURE_DIR>/logs/impact-map/scratch.md`
  - Emit `<FEATURE_DIR>/logs/impact-map/handoff.md` once the scratch touch set and main implication buckets exist.
- Phase B (canonical write gate):
  - Poll until BOTH are true:
    - `<FEATURE_DIR>/logs/spec-manifest/last_message.md` exists
    - `git status --porcelain=v1 -- "<FEATURE_DIR>"` is empty
  - Then write tracked output:
    - `<FEATURE_DIR>/impact_map.md` (only tracked file)

### 4.2 Add new prompts (prompt‑only; no preamble)
Add:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md`
  - Output (tracked): `<FEATURE_DIR>/minimal_spec_draft.md` only
  - Overlap behavior:
    - Start early; do scratch notes under `<FEATURE_DIR>/logs/min-spec-draft/`
    - Emit `<FEATURE_DIR>/logs/min-spec-draft/handoff.md` once you have a coherent outline
    - Poll until:
      - `<FEATURE_DIR>/logs/impact-map/last_message.md` exists, and
      - feature dir clean
    - Then write `minimal_spec_draft.md`
  - Content contract:
    - explicitly labeled **Pre‑Planning Only** and must be deleted/retired in full planning
    - cross‑cutting defaults/precedence/invariants/failure posture only (alignment backbone)

Add:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md`
  - Conditional behavior:
    - If pack is not cross‑platform automation (detect via `tasks.json`), write “not applicable” into logs and still emit `handoff.md` (logs only), and exit without tracked changes.
  - If applicable:
    - Start early; scratch under `<FEATURE_DIR>/logs/CI-checkpoint/`
    - Emit `<FEATURE_DIR>/logs/CI-checkpoint/handoff.md` once you’ve drafted checkpoint group boundaries and gates.
    - Poll until:
      - `<FEATURE_DIR>/logs/min-spec-draft/last_message.md` exists, and
      - feature dir clean
    - Then update:
      - `<FEATURE_DIR>/ci_checkpoint_plan.md`
      - `<FEATURE_DIR>/tasks.json` only if required to satisfy the checkpoint wiring rules
    - Must pass:
      - `python3 /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"`

Add:
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md`
  - Logs‑only output:
    - Draft: `<FEATURE_DIR>/logs/workstream-triage/workstream_triage_draft.md`
    - Emit `<FEATURE_DIR>/logs/workstream-triage/handoff.md` when the draft has a usable initial structure (optional, since this is the end of the chain).
  - Gate finalization (still logs-only):
    - Poll until:
      - `<FEATURE_DIR>/logs/CI-checkpoint/last_message.md` exists (or CI checkpoint step declared non-applicable by its logs), and
      - feature dir clean
  - Draft must include:
    - Proposed workstreams (parallelizable clusters) + rationale
    - Sequencing/gating constraints
    - Explicit “unknowns/follow-ups” to resolve in full planning
    - References/links to the stable step `last_message.md` files (so future agents can ingest filtered summaries)

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
