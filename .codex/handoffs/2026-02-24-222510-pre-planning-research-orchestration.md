# Handoff: Pre-planning research orchestration (prompt audit + plumbing implementation)

## Session Metadata
- Created: 2026-02-24 22:25:10
- Project: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- Branch: ops/work_lift_v1_seams
- Session duration: ~2h

### Recent Commits (for context)
  - 404cc6e2 Align planning logs with stable last
  - b1dcbb15 Add missing planning wrapper steps
  - 3d302a3c Review Work Lift doc gaps
  - c47238d3 Review docs for Work Lift updates
  - 6d591f06 Add advisory lift docs and Makefile

## Handoff Chain

- **Continues from**: [2026-02-24-101830-workstream-system-planning.md](./2026-02-24-101830-workstream-system-planning.md)
  - Previous title: Workstream system planning (Pass A triage + Pass B refinement)
- **Supersedes**: None

> Review the previous handoff for full context before filling this one.

## Current State Summary

Audited the pre-planning research orchestration plan vs the *real* prompt + script inventory, then updated the docs/prompts and implemented the missing plumbing so the workflow is actually runnable: stable per-step log dirs with `stderr.log` + `codex.pid`, stable `last_message.md` sentinels promoted only on successful step completion, staggered overlap triggered by upstream `handoff.md`, and archive-instead-of-delete for reruns (`*_run_N`). Current repo state has uncommitted changes (docs/scripts/Makefile) plus new untracked prompt/script files that should be committed (excluding `.codex/` artifacts) before running the orchestrator on a real pack.

## Codebase Understanding

### Architecture Overview

Pre-planning research is now implemented as a scripted orchestration layer that launches focused Codex runs (“agents”) via a runner:
- `make pm-pre-planning-research …` → `docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh`
- Orchestrator script launches the 5-step chain in the pack:
  `spec_manifest` → `impact_map` → `min_spec_draft` → `ci_checkpoint` → `workstream_triage`
- Each step is executed by `docs/project_management/system/scripts/planning/run_planning_agent.sh`, which:
  - injects a dispatcher prelude (ADR paths + output allowlist + logs allowance),
  - runs `codex exec` headless,
  - writes run artifacts under `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/`,
  - maintains stable live artifacts at `<FEATURE_DIR>/logs/<step>/stderr.log` + `<FEATURE_DIR>/logs/<step>/codex.pid`,
  - promotes `<FEATURE_DIR>/logs/<step>/last_message.md` only when the step exits `0` and output allowlist checks pass.

The “overlap” is prompt-driven: downstream agents can start early but are instructed to do logs-only work until upstream stable `last_message.md` exists and the feature dir is clean.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh | Scripted orchestrator for the 5-step chain | Canonical entrypoint for staggered overlap + commits + summary |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/run_planning_agent.sh | Runs a focused planning agent via Codex | Implements stable step logs, allowlists, and sentinel promotion |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Makefile | Planning automation targets | Adds `pm-pre-planning-research` + expands `pm-run-planning-agent` |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/spec_manifest_agent.md | Focused agent prompt | Now allows logs writes + emits `handoff.md` |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/impact_map_agent.md | Focused agent prompt | Overlap phases + sentinel-gated tracked write |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md | Focused agent prompt | New; produces `minimal_spec_draft.md` (pre-planning only) |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md | Focused agent prompt | New; writes `ci_checkpoint_plan.md` + optional `tasks.json` |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md | Focused agent prompt | New; logs-only workstream triage draft |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md | Standard for the workflow | Updated to 5-step chain + stable step logs + rerun archiving |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/pre_planning_researcn_orchestration.md | Working spec/plan | Reality-grounded doc of how orchestration is meant to work |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/triad/task_start.sh | Triad automation reference | Source of the `stderr.log` + `codex.pid` pattern copied into planning runner |

### Key Patterns Discovered

- **Prompt payload convention**: focused planning prompts are a single fenced ` ```md ` block; the runner extracts the first ` ```md ` fence payload as Codex stdin.
- **Stable sentinel model (triad-style)**:
  - Stable per-step dirs: `<FEATURE_DIR>/logs/<step>/`
  - Stable artifacts: `stderr.log`, `codex.pid`, `handoff.md`, `last_message.md`
  - Run-scoped artifacts: `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/*`
- **No `last_message.latest.md`**: stable paths only; reruns archive step dirs to `*_run_N` to avoid stale sentinels.
- **Default low-frequency polling**: `POLL_S=60` (both prompts and orchestrator) to reduce tight loops.

## Work Completed

### Tasks Finished

- [x] Audited prompt inventory vs `pre_planning_researcn_orchestration.md` and removed “fantasy prompt” references.
- [x] Updated existing focused prompts to allow logs-only overlap and emit `handoff.md`.
- [x] Added missing focused prompts: minimal spec draft, CI checkpoint, workstream triage.
- [x] Updated `run_planning_agent.sh` to implement stable step logs (`stderr.log`, `codex.pid`) + stable sentinel promotion rules + multi-output allowlist.
- [x] Added `pre_planning_research_orchestrate.sh` and Make target `pm-pre-planning-research`.
- [x] Updated wrapper standard to reflect the real 5-step staggered/sentinel-gated model.
- [x] Ran `shellcheck` + `bash -n` for the new/modified scripts.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/pre_planning_wrapper.md | Rewrote wrapper prompt to be script-driven (`make pm-pre-planning-research`) instead of manual sequential steps | Keep wrapper “prompt-only” and aligned with canonical automation |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md | Expanded standard to full 5-step chain, stable step logs/sentinels, rerun archiving, canonical Make entrypoint | Make the standard match the intended orchestrated reality |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Makefile | Added `pm-pre-planning-research` target; expanded `pm-run-planning-agent` supported agents; added `START_AT` + `POLL_S` vars | Wire orchestration into canonical tooling |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/pre_planning_researcn_orchestration.md | Reality-grounded the plan doc and documented runner/prompt mismatches + required fixes | Keep orchestration plan grounded and executable |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/impact_map_agent.md | Added overlap phases, logs allowance, `handoff.md`, sentinel-gated tracked write | Enable staggered overlap chain without premature tracked writes |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/spec_manifest_agent.md | Allowed logs writes and required `handoff.md` emission | Allow downstream to start discovery early |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/run_planning_agent.sh | Added new agents; stable step log dirs + `stderr.log`/`codex.pid`; multi-output allowlist; stable `last_message.md` promotion on success; removed old `logs/planning_agents` layout | Make runner support sentinel-gated orchestration model |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh | New orchestrator script: archives step logs to `*_run_N`, launches staggered overlap chain, commits allowlisted outputs, writes wrapper summary | Canonical automation implementation |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/min_spec_draft_agent.md | New focused agent prompt | Add “Minimal Spec Draft” step (tracked `minimal_spec_draft.md`, pre-planning only) |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/ci_checkpoint_agent.md | New focused agent prompt | Add CI checkpoint planning step with applicability gating |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/prompts/planning/workstream_triage_agent.md | New focused agent prompt | Add logs-only workstream triage drafting step |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Stable per-step sentinels only (`last_message.md`, no `*.latest.md`) | Stable sentinel vs `last_message.latest.md` pointer | Avoid stale “latest” reads in downstream steps; match triad sentinel model |
| Archive logs on rerun (`*_run_N`) instead of deleting | Delete vs mv/rename | Preserves audit trail and prevents stale stable sentinels from short-circuiting polls |
| Default polling interval `60s` | 2s vs 60s | Reduce busy polling; prefer tailing `stderr.log` for real-time progress |
| Runner-injected prelude must allow logs writes | “only write X” vs “tracked allowlist + logs allowed” | Orchestration requires `handoff.md` + scratch logs while preserving tracked allowlists |
| Minimal spec draft is tracked but explicitly pre-planning-only | Logs-only vs tracked | Needs to function as alignment backbone; must be clearly marked for deletion/retirement in full planning |
| Remove model/profile lines and unknown shell assertions from prompts | Include `--profile/--model` and “ensure FEATURE_DIR exists” | Prompts must be prompt-only; model/profile is operator tooling; keep consistency with existing repo prompt conventions |

## Pending Work

### Immediate Next Steps

1. Review diffs and commit the changes (exclude `.codex/`):
   - `git add Makefile pre_planning_researcn_orchestration.md docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md docs/project_management/system/scripts/planning/run_planning_agent.sh docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh docs/project_management/system/prompts/planning/*.md`
   - `git status --porcelain=v1` should then be clean (except ignored logs).
2. Run a real end-to-end smoke of the orchestrator on a representative pack:
   - `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/active/<feature>"`
3. If the run reveals prompt compliance gaps (agents writing tracked outputs too early), decide whether to add enforcement in the runner (beyond prompt-only gating).

### Blockers/Open Questions

- [ ] Decide whether we need runner-level enforcement that downstream steps cannot change tracked outputs until upstream sentinel exists (currently prompt-driven).
- [ ] Confirm whether `CI-checkpoint` directory naming should remain `CI-checkpoint` (mixed case) or be normalized (would require updating prompts/scripts consistently).

### Deferred Items

- Research backplane integration for intake → pre-planning (explicitly out of scope for this pass).
- Any additional standards refactoring beyond aligning `PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md` to the implemented model.

## Context for Resuming Agent

### Important Context

- The core mismatch that was fixed: `run_planning_agent.sh` previously injected “Do not edit any other files”, which made the overlap/logs/handoff model impossible. The runner prelude now allows logs writes under `<FEATURE_DIR>/logs/<step>/` and enforces a tracked-file allowlist.
- Canonical stable step dirs under `<FEATURE_DIR>/logs/`:
  - `spec-manifest/`, `impact-map/`, `min-spec-draft/`, `CI-checkpoint/`, `workstream-triage/`
  - Stable files: `stderr.log`, `codex.pid`, `handoff.md`, `last_message.md`
  - Run artifacts: `<FEATURE_DIR>/logs/<step>/runs/<RUN_TS>/…` including `last_message.run.md`
- `last_message.md` is a *success sentinel only*:
  - Promoted only when step exit code is `0` and allowlists pass.
  - If Codex doesn’t emit `--output-last-message`, the runner writes a run-scoped stub to `last_message.run.md` but does not promote it unless the step is otherwise successful.
- Orchestrator:
  - Requires clean orchestration checkout (`git status` empty).
  - Archives step log dirs for `START_AT` and downstream to `*_run_N` and recreates fresh dirs.
  - Starts downstream steps when upstream emits `handoff.md` (or upstream exits successfully without it).
  - Commits allowlisted tracked outputs per step; `workstream-triage` is logs-only.

### Assumptions Made

- `docs/project_management/packs/**/logs/` is gitignored, so per-step logs do not appear as untracked files to the runner allowlist checks.
- Starting downstream steps early is safe because prompts enforce “logs-only until upstream sentinel exists” (not runner-enforced yet).
- `ps` is available on host platforms where the planning runner runs (used for `codex.pid` safety checks).

### Potential Gotchas

- Orchestrator will refuse to run if *any* repo changes are present. Commit/stash first.
- `START_AT` is the **step dir name** (`impact-map`, not `impact_map`).
- Step dir naming is case-sensitive in prompts/scripts (`CI-checkpoint`).
- Runner fails if an agent writes any unignored untracked file under `<FEATURE_DIR>` (intended guardrail).

## Environment State

### Tools/Services Used

- `git`, `bash`, `jq`, `python3`
- `codex exec` (headless; runner-controlled)
- `shellcheck` (for scripts)

### Active Processes

None.

### Environment Variables

- `CODEX_PROFILE`
- `CODEX_MODEL`
- `CODEX_JSONL`
- `PM_SYSTEM_ROOT`

## Related Resources

- Orchestration spec/plan: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/pre_planning_researcn_orchestration.md`
- Wrapper standard: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md`
- Canonical run command: `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/active/<feature>"`
- Orchestrator script: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh`
- Planning runner: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/run_planning_agent.sh`

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
