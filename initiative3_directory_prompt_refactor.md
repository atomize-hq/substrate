# Initiative 3 — Directory & Prompt Architecture Refactor: System Isolation, Prompt Library, Planning Pack Restructure (slices as directories), ADR migration, and updated CI checkpoint sizing

**Status:** Draft (implementation-ready)  
**Primary outcome:** Remove bloat and path chaos by separating:
- the **Project Management System** (standards/templates/prompts/scripts), and
- the **Project Management Artifacts** (ADRs + Planning Packs),

…and by making each slice a directory (so slice docs don’t explode a single folder).

This initiative also introduces a single-purpose planning prompt library and dispatcher so you can run “spec manifest agent” / “impact map agent” / etc independently.

---

## 0. Executive summary

Today, the project management system and the project artifacts are interleaved:

- System:
  - `docs/project_management/standards/` (standards + templates + prompts mixed)
  - `scripts/planning/` and `scripts/triad/` (mixed into repo-wide scripts)

- Artifacts:
  - ADRs are split across:
    - `docs/project_management/adrs/*` (good start), and
    - legacy ADR files under `docs/project_management/next/…` (messy)
  - Planning Packs live under `docs/project_management/next/` and `future/` (mixed contents, large folders, and kickoff prompts in a single flat directory).

This initiative makes the structure intentional and portable:

1) **System root**: a single directory that can be copied to other repos.
2) **Artifact roots**: ADRs and Planning Packs organized by status buckets.
3) **Slice directories**: each slice gets its own folder containing:
   - spec, closeout, and kickoff prompts for that slice.
4) **Prompt library**: prompts live in a dedicated prompt directory; standards docs link to prompts instead of embedding them.
5) **Planning agent dispatcher**: one command to run a single-purpose planning agent (spec manifest only, impact map only, etc).
6) **Checkpoint sizing update is maintained**: defaults stay at **4–8** triads per checkpoint (Initiative 1), and all templates/standards in the new structure preserve it.

---

## 0.1 Compatibility contract (bake in immediately)

This initiative moves directories that are currently relied on by enforcement and triad orchestration. To avoid destabilizing the system mid-flight, **define and preserve a compatibility surface before moving any directories**.

**Recommended approach:** preserve old entrypoints with thin wrappers and add path resolution so callers can use either old or new pack roots during the transition.

### Compatibility surface (guarantees during the refactor)
- **Stable script entrypoints remain callable** (even if implementation moves):
  - `scripts/planning/*` (including `scripts/planning/validate_impact_map.py`)
  - `scripts/triad/*` (including orchestration worktree discovery logic used by `task_*`)
  - (If you introduce `pm-run-planning-agent`, provide a stable wrapper entrypoint as well.)
- **Planning Pack locations are accepted in both forms during migration**:
  - legacy: `docs/project_management/next/<feature>`
  - new: `docs/project_management/packs/<bucket>/<feature>`
  - Implement a shared “feature dir resolver” used by:
    - planning lint + all planning validators (`--feature-dir` consumers), and
    - triad scripts that read `feature_dir` from `.taskmeta.json`.
- **In-flight worktrees must not break when packs move**:
  - `.taskmeta.json` stores `feature_dir` (repo-relative) and triad registries store a `feature_dir` as well.
  - Before moving `docs/project_management/next/<feature>` to `packs/...`, either:
    - require **zero active worktrees** for that feature, or
    - ship a mechanical migration that updates `.taskmeta.json` + the triad registry entries to the new pack path.

If you skip this compatibility contract, the refactor becomes “all callers + all docs + all prompts in one change”, which is riskier and harder to review.

---

## 1. Goals

### 1.1 Goals (must achieve)
- A cleaner layout:
  - predictable locations,
  - fewer “where is the thing” hunts,
  - less chance an agent reads the wrong doc.
- Make the system portable:
  - copy one directory into another repo and it works with minimal configuration.
- Reduce “prompt bloat”:
  - standards are standards (normative rules),
  - prompts are prompts (copy/paste runnable instructions),
  - templates are templates.
- Make Planning Packs scalable:
  - slice directory per slice,
  - kickoff prompts local to the slice,
  - reduce feature root clutter.
- Complete ADR migration:
  - legacy ADRs moved into `adrs/implemented|superseded|queued|draft`.

### 1.2 Non-goals
- Changing triad execution model to supervised live-steering (future).
- Redesigning ADR content rules (future).
- Refactoring feature content semantics (this is structural/migrational).

---

## 2. Proposed target directory layout

### 2.1 Top-level layout under docs/project_management

**Target structure:**

```
docs/project_management/
  system/                      # the reusable “system”
    README.md
    standards/                 # normative rules (no copy/paste mega prompts)
      planning/
      execution/
      adr/
      ci/
      triad/
    templates/                 # .tmpl files used by scaffolding
      planning_pack/
      kickoff/
      spec/
    prompts/                   # runnable prompts (copy/paste or codex input)
      planning/
      execution/
      triad_wrappers/
      quality_gate/
    agent_guides/              # role-specific reading
      testing/
      integration/
      coding/
    schemas/
      tasks.schema.json
      (future: slice_spec.schema.json, impact_map.schema.json)
    scripts/                   # scripts live with the system, not repo-wide
      planning/
      triad/
      lib/                     # shared python helpers (optional)
  adrs/
    draft/
    queued/
    implemented/
    superseded/
    README.md
  packs/                       # Planning Packs (replaces next/future)
    draft/
    queued/
    active/
    implemented/
    superseded/
    README.md
    sequencing.json            # moved from next/
```

**Why this layout works**
- `system/` is portable and self-contained.
- `adrs/` and `packs/` are artifacts and can be large; they stay outside the system.
- Standards and prompts are separate to prevent “prompt = standard” drift.

---

### 2.2 Planning Pack layout (feature directory)

Each feature under `packs/<bucket>/<feature>/` has:

```
packs/active/<feature>/
  plan.md
  tasks.json
  session_log.md
  contract.md
  spec_manifest.md
  impact_map.md
  decision_register.md
  ci_checkpoint_plan.md          # only when cross-platform automation enabled
  quality_gate_report.md
  execution_preflight_report.md  # when execution gates enabled
  smoke/                         # if present today (unchanged)
  kickoff_prompts/               # feature-level tasks only: F0 / CP* / FZ
    F0-exec-preflight.md
    CP1-ci-checkpoint.md
    ...
    FZ-feature-cleanup.md
  slices/
    <SLICE_ID>/
      <SLICE_ID>-spec.md
      <SLICE_ID>-closeout_report.md
      kickoff_prompts/
        <SLICE_ID>-code.md
        <SLICE_ID>-test.md
        <SLICE_ID>-integ.md
        <SLICE_ID>-integ-core.md      # for checkpoint boundary slices
        <SLICE_ID>-integ-linux.md
        <SLICE_ID>-integ-macos.md
        <SLICE_ID>-integ-windows.md
  logs/
    <SLICE_ID>/
      code/
      test/
      integ/
```

Notes:
- Feature-level kickoff prompts remain at `<feature>/kickoff_prompts/` because they are not slice-specific.
- Slice-specific kickoff prompts move under `<feature>/slices/<SLICE_ID>/kickoff_prompts/`.
- Slice docs (`spec`, `closeout`) live with the slice.
- Logs remain under feature dir and can remain slice-bucketed.

---

## 3. Prompt library (single-purpose planning agents)

### 3.1 Prompt taxonomy

Create dedicated prompt files so each planning artifact can be authored by a focused agent:

**Planning prompts (one output each):**
- `prompts/planning/spec_manifest_agent.md`
- `prompts/planning/impact_map_agent.md`
- `prompts/planning/ci_checkpoint_plan_agent.md`
- `prompts/planning/tasks_json_agent.md`
- `prompts/planning/kickoff_prompt_generator_agent.md`
- `prompts/planning/quality_gate_reviewer.md` (moves existing quality gate prompt here)
- `prompts/planning/quality_gate_remediation.md` (moves remediation prompt here)

**Execution prompts:**
- `prompts/triad_wrappers/triad_wrapper.md`
- `prompts/triad_wrappers/triad_integration_wrapper.md`
- `prompts/triad_wrappers/triad_unified_wrapper_checkpoint_aware.md`

**Agent guides (role reading, not prompts):**
- `agent_guides/testing/TESTING_AGENT.md`
- `agent_guides/testing/TUI_TESTING_GUIDE.md`
- etc.

Standards should link to these prompt files rather than embedding giant prompt templates.

---

### 3.2 Single-output rule (hard)

Each planning agent prompt MUST:
- Read inputs (feature dir, ADR, standards)
- Produce exactly **one** artifact (one file)
- Not “do the whole planning pack”.

If the agent discovers it needs additional docs, it must write a “Follow-ups” section in the artifact rather than editing other files.

---

### 3.3 Planning agent dispatcher (script)

**Add script (implementation):**
- `docs/project_management/system/scripts/planning/run_planning_agent.sh`
- (and optional `run_planning_agent.ps1`)

**Expose via Make (recommended):**
- Add a `make` target named `pm-run-planning-agent` (or `planning-run-agent` if you prefer the existing `planning-*` naming).

Suggested Make usage:

```bash
make pm-run-planning-agent \
  FEATURE_DIR=docs/project_management/packs/active/<feature> \
  AGENT=spec_manifest \
  [CODEX_PROFILE=<p>] [CODEX_MODEL=<m>] [CODEX_JSONL=1]
```

Direct script usage (optional, for non-Make contexts):

```bash
docs/project_management/system/scripts/planning/run_planning_agent.sh \
  --feature-dir docs/project_management/packs/active/<feature> \
  --agent spec_manifest \
  [--codex-profile <p>] [--codex-model <m>] [--codex-jsonl]
```

Behavior:
- Select prompt file based on `--agent`.
- Pass the prompt to `codex exec` using the same safe patterns you already use in triad runners:
  - `codex exec --dangerously-bypass-approvals-and-sandbox --cd <repo_root> --output-last-message <log> -`
- Provide prompt inputs via environment substitution:
  - FEATURE_DIR
  - ADR_PATH (optional)
  - SYSTEM_ROOT (where standards/templates live)
- Write output file deterministically:
  - either by instructing the agent to edit the file directly (preferred), OR
  - by capturing `--output-last-message` and applying it via a small “apply patch” helper.

**Recommendation:** instruct the agent to edit the artifact file directly. This keeps behavior consistent with how codex operates in your repo today.

---

## 4. System isolation: moving scripts into the system

### 4.1 Move scripts

Move:

- `scripts/planning/*` → `docs/project_management/system/scripts/planning/*`
- `scripts/triad/*` → `docs/project_management/system/scripts/triad/*`

Add a thin compatibility shim (temporary) in old locations to avoid breaking muscle memory:

- Keep `scripts/planning/new_feature.sh` as:
  - a wrapper that calls the new path, e.g.:

```bash
#!/usr/bin/env bash
set -euo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"
exec "${repo_root}/docs/project_management/system/scripts/planning/new_feature.sh" "$@"
```

Same for `scripts/triad/*`.

This lets you migrate gradually without breaking existing instructions.

---

### 4.2 Parameterize the roots (portability)

Introduce environment variables with defaults:

- `PM_ROOT` default: `docs/project_management`
- `PM_SYSTEM_ROOT` default: `${PM_ROOT}/system`
- `PM_PACKS_ROOT` default: `${PM_ROOT}/packs` (and default bucket: `active`)
- `PM_ADRS_ROOT` default: `${PM_ROOT}/adrs`

Update scripts to use these instead of hardcoding:
- `docs/project_management/next`
- `docs/project_management/standards/templates`
- etc.

Examples of scripts that currently hardcode:
- `new_feature.sh` / `new_feature.ps1`
- `lint.sh` / `lint.ps1` (references `docs/project_management/next/sequencing.json`)
- triad runners (`task_start*`, `task_finish`, `feature_cleanup`, `codex_pidfiles`, `dispatch_feature_smoke`) that:
  - hardcode `docs/project_management/next` in invariants (e.g., “no planning docs edits inside worktree”), and/or
  - print help text that implies `next/<feature>` is the only supported pack layout
- `ensure_kickoff_prompt_sentinel.py` default root
- any regex in validators that references `docs/project_management/(next|adrs/...)`

---

## 5. ADR migration plan

### 5.1 Target ADR policy
- The canonical home for ADRs is `docs/project_management/adrs/<bucket>/`.
- Legacy ADR locations become deprecated (read-only compatibility during transition).

### 5.2 Migration steps
1) Create any missing bucket dirs (`draft/queued/implemented/superseded` already exist).
2) Move legacy ADRs:
   - legacy ADRs under `docs/project_management/next/**` → `docs/project_management/adrs/<bucket>/` (`implemented/` or `superseded/` as applicable)
   - `docs/project_management/next/<feature>/ADR-000*-*.md` → appropriate bucket
3) Update references:
   - Use a mechanical search/replace in Planning Packs:
     - legacy ADR paths under `docs/project_management/next/**` → canonical ADR registry paths under `docs/project_management/adrs/<bucket>/`
4) Update `docs/project_management/adrs/README.md`:
   - Remove “legacy locations still supported” after migration is complete, OR keep but mark deprecated.

---

## 6. Planning Pack migration plan (next/future → packs/*)

### 6.1 Target policy
- `docs/project_management/packs/active/` replaces `next/`
- `docs/project_management/packs/queued/` replaces “ready but not executing”
- `docs/project_management/packs/draft/` for incomplete packs
- `docs/project_management/packs/implemented/` for completed packs
- `docs/project_management/packs/superseded/` for replaced packs (rare)

### 6.2 Migration steps (mechanical, in phases)

**Phase A — introduce packs/ without moving anything**
1) Create `docs/project_management/packs/*` buckets.
2) Copy `docs/project_management/next/sequencing.json` → `docs/project_management/packs/sequencing.json`.
3) Update planning lint scripts to reference the new `sequencing.json` location via `${PM_ROOT}`.

**Phase B — move active packs**
1) Move directories:
   - `docs/project_management/next/<feature>` → `docs/project_management/packs/active/<feature>`
2) Update any scripts and docs that default to `docs/project_management/next` to default to `packs/active`.

**Phase C — move future packs**
- `docs/project_management/future/*` → `docs/project_management/packs/draft/*` or `queued/*` depending on readiness.

**Phase D — update cross-references**
- Tasks and references contain many absolute repo-relative paths pointing to the old locations (e.g., `docs/project_management/next/world-sync/...`).
- Perform a mechanical rewrite:
  - prefix replace: `docs/project_management/next/` → `docs/project_management/packs/active/`
- Apply across:
  - `tasks.json` (kickoff_prompt + references)
  - `plan.md`, `spec_manifest.md`, `impact_map.md` (if they contain explicit paths)
  - kickoff prompts text

> Do this rewrite as a single mechanical commit.

---

## 7. Slice directories migration (kickoff prompts + spec/closeout per slice)

### 7.1 Target policy
- Each slice gets a directory under `slices/<SLICE_ID>/`.
- Slice kickoff prompts move into that directory.

### 7.2 Mechanical migration steps for one feature
For feature `<feature_dir>`:

1) Create `slices/<SLICE_ID>/kickoff_prompts/` for each slice id.
2) Move slice docs:
   - `<feature_dir>/<SLICE_ID>-spec.md` → `<feature_dir>/slices/<SLICE_ID>/<SLICE_ID>-spec.md`
   - `<feature_dir>/<SLICE_ID>-closeout_report.md` → `<feature_dir>/slices/<SLICE_ID>/<SLICE_ID>-closeout_report.md`
3) Move kickoff prompts:
   - `<feature_dir>/kickoff_prompts/<SLICE_ID>-code.md` → `<feature_dir>/slices/<SLICE_ID>/kickoff_prompts/<SLICE_ID>-code.md`
   - same for test/integ and integ-* variants
4) Keep feature-level prompts in `<feature_dir>/kickoff_prompts/`:
   - `F0-exec-preflight.md`
   - `CP*-ci-checkpoint.md`
   - `FZ-feature-cleanup.md`

5) Update `tasks.json`:
   - For slice tasks:
     - `kickoff_prompt` path updated to new slice directory path
     - references updated (spec path, closeout path)
   - For feature-level tasks:
     - unchanged

6) Update planning validators that assume kickoff prompts live only at feature root:
   - `validate_tasks_json.py` currently enforces kickoff prompts under `<feature_dir>/kickoff_prompts`.
   - Update to allow:
     - `<feature_dir>/kickoff_prompts/**.md` (feature-level)
     - `<feature_dir>/slices/<SLICE_ID>/kickoff_prompts/**.md` (slice-level)

7) Update planning lint sentinel scan:
   - It currently checks only `<feature_dir>/kickoff_prompts` and `-maxdepth 1`.
   - Update to scan all `kickoff_prompts` directories recursively under the feature dir.

---

## 8. Standards/templates/prompts migration

### 8.1 Move templates out of standards
Move:
- `docs/project_management/standards/templates/*` → `docs/project_management/system/templates/*`

Update:
- `new_feature.*` scripts to point at the new templates root.

### 8.2 Move prompts out of standards
Move prompt-like docs into `system/prompts/…`:
- `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
- `docs/project_management/standards/PLANNING_QUALITY_GATE_REMEDIATION_PROMPT.md`
- `docs/project_management/standards/TRIAD_WRAPPER_PROMPT.md`
- `docs/project_management/standards/TRIAD_INTEGRATION_WRAPPER_PROMPT.md`
- `docs/project_management/standards/TRIAD_WRAPPER_PROMPT_UNIFIED.md`

Replace them in standards with short link-only stubs, e.g.:

```md
This standard uses prompt: `docs/project_management/system/prompts/quality_gate/quality_gate_reviewer.md`
```

This reduces duplication and keeps prompts versioned separately.

---

## 9. CI checkpoint sizing (must be preserved in the refactor)

As part of Initiative 1, the default checkpoint sizing is:

- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

In this refactor:
- Ensure the *template* moved to `system/templates/planning_pack/ci_checkpoint_plan.md.tmpl` still contains 4–8 defaults.
- Ensure the *standard* moved under `system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md` still documents 4–8 defaults.

---

## 10. Script and validator updates required by path changes

This section is the “breakage checklist” — what must be updated once paths move.

### 10.1 new_feature scripts
- `new_feature.sh|ps1` must:
  - create new feature under `packs/<bucket>/` (default active)
  - use new templates root
  - create slice directory structure (`slices/<SLICE_ID>/...`)
  - create feature-level kickoff prompts dir
  - create slice-level kickoff prompts dir

### 10.2 lint scripts
- `lint.sh|ps1` must:
  - require correct feature files at new locations
  - reference `sequencing.json` at `docs/project_management/packs/sequencing.json` (or from `${PM_ROOT}`)
  - scan kickoff prompts recursively
  - call validators from new script paths

### 10.3 validate_tasks_json.py
Update these assumptions:
- kickoff prompts can be in either:
  - `<feature_dir>/kickoff_prompts/`
  - `<feature_dir>/slices/<slice>/kickoff_prompts/`
- feature dir is no longer `docs/project_management/next/<feature>` (but validator already receives `--feature-dir`, so just remove any explicit “next” logic in regex or messages).

### 10.4 ensure_kickoff_prompt_sentinel.py
- Default root should become `docs/project_management/packs` (or `${PM_ROOT}/packs`).
- The script already does `rglob("kickoff_prompts")`, so this mostly means updating the default arg.

### 10.5 ADR executive summary checker
- `check_adr_exec_summary.py` should accept ADR paths under the new ADR root.
- Any regex that searches for ADR references should include `docs/project_management/adrs/...` only (after legacy is removed).

---

## 11. Implementation sequencing (recommended)

Because this refactor is large, do it in phases that keep the repo buildable.

### Phase 1 — Introduce new directories + compatibility shims (no moves yet)
- Add `docs/project_management/system/` structure.
- Copy (not move) templates/prompts/standards into new structure.
- Add wrapper scripts in old `scripts/*` that call the new location.

### Phase 2 — Move scripts (system isolation)
- Move scripts into `system/scripts`.
- Keep wrappers in old location.

### Phase 3 — Move ADRs (one mechanical commit)
- Move legacy ADR files into `adrs/implemented|superseded`.
- Update references.

### Phase 4 — Move Planning Packs (next → packs/active)
- Move directories.
- Mechanical rewrite of path references.

### Phase 5 — Slice directories migration (per feature, or mechanical batch)
- Introduce `slices/<SLICE_ID>/...`.
- Update tasks.json + kickoff prompts.
- Update validators to support both locations temporarily, then remove old support.

### Phase 6 — Remove legacy compatibility
- Remove wrappers.
- Remove “legacy locations supported” notes.

---

## 12. Acceptance criteria for this initiative

- The project management system is self-contained under `docs/project_management/system/` including scripts.
- ADRs exist only under `docs/project_management/adrs/<bucket>/` (no legacy ADRs in packs/).
- Planning Packs exist under `docs/project_management/packs/<bucket>/`.
- Each Planning Pack stores slice docs and slice kickoff prompts under `slices/<SLICE_ID>/`.
- Standards do not embed giant prompts; prompts live in `system/prompts/`.
- A single-purpose planning agent dispatcher exists and can generate at least:
  - `spec_manifest.md` and
  - `impact_map.md`
  independently.
- CI checkpoint templates and standards preserve default sizing **4–8**.
