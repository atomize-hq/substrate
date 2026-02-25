# Planning Pre-Planning Research Wrapper (Standard)

This standard describes the **Pre-Planning Research** orchestration workflow and the intended contracts between:
- the scripted orchestrator (automation),
- the focused planning agents (headless Codex runs), and
- the wrapper/orchestrator agent (operator).

Canonical wrapper prompt:
- `docs/project_management/system/prompts/planning/pre_planning_wrapper.md`

Canonical automation entrypoint:
- Start from ADR-only (no pack yet):
  - `make pm-pre-planning-from-adr ADR="docs/project_management/adrs/draft/ADR-000X-<kebab-title>.md" [FEATURE="<feature>"] [BUCKET=draft]`
- Start from an existing pack:
  - `make pm-pre-planning-research FEATURE_DIR="docs/project_management/packs/<bucket>/<feature>"`

---

## Goal

Produce the minimal, high-signal pre-planning artifacts needed to reduce thrash before full planning:

Tracked (canonical; pack root):
- `spec_manifest.md` (deterministic spec selection + ownership map)
- `impact_map.md` (touch set + cascading implications + cross-queue scan)
- `minimal_spec_draft.md` (**Pre‑Planning Only** alignment backbone; must be deleted/retired during full planning)
- `ci_checkpoint_plan.md` (cross-platform automation packs only; provisional until slice boundaries stabilize)

Untracked (logs-only):
- workstream triage draft (orchestrator-facing; not canonical)

Non-goals:
- Completing slice specs or passing `make planning-lint` on a newly scaffolded pack.
  - New packs scaffold Slice Spec v2 placeholders, and v2 lint forbids placeholders until full planning fills them.

---

## Preconditions

1) One of:
   - ADR-only start (recommended for pre-planning): run:
     - `make pm-pre-planning-from-adr ADR="docs/project_management/adrs/draft/ADR-000X-<kebab-title>.md" [FEATURE="<feature>"] [BUCKET=draft]`
     - This scaffolds a minimal pack (creates/patches `tasks.json` with `meta.adr_paths`) and auto-commits it before launching pre-planning.
   - Existing Planning Pack under `docs/project_management/packs/<bucket>/<feature>/`:
     - Recommended full scaffold: `make planning-new-feature FEATURE=<feature> ...`

2) The pack’s `tasks.json` can resolve ADR inputs for focused agents.
   - Strict packs (`meta.slice_spec_version >= 2`) must set at least one of:
     - `meta.adr_refs` (preferred), or
     - `meta.adr_paths` (use when refs are ambiguous).

3) Orchestration runs from a clean orchestration checkout.
   - `git status --porcelain=v1` must be empty before starting the orchestrator.

---

## Stable step logs (sentinels + streaming)

Pre-planning research uses stable, pack-local (gitignored) step log dirs under `<FEATURE_DIR>/logs/`:
- `spec-manifest/`
- `impact-map/`
- `min-spec-draft/`
- `CI-checkpoint/`
- `workstream-triage/`

Each step dir contains stable orchestration artifacts:
- `stderr.log` (streams while Codex runs; created/truncated at start)
- `codex.pid` (exists while Codex runs; removed at exit)
- `handoff.md` (mid-run “start downstream now” signal)
- `last_message.md` (stable completion sentinel; exists **only when the step successfully completes**)

Rationale:
- `docs/project_management/packs/**/logs/` is gitignored, so these artifacts do not affect canonical docs or quality gates.
- Stable sentinels allow downstream steps to self-gate without relying on “latest” pointers.

---

## Orchestration pattern (staggered overlap + sentinel-gated writes)

Pre-planning research is a 5-step chain:
1) `spec_manifest`
2) `impact_map`
3) `min_spec_draft`
4) `ci_checkpoint`
5) `workstream_triage`

Overlap model:
- The orchestrator starts steps in a staggered, overlapped manner (triggered by upstream `handoff.md`).
- Downstream agents may start early, but must:
  - write to their own step `logs/` only while upstream is still running, and
  - delay tracked (canonical) writes until upstream has completed successfully (`last_message.md` exists) and the pack is clean.

Default poll interval for sentinel gating: 60s.

Hard rules:
- Stop-on-failure: if any step fails, do not proceed to downstream steps.
- Avoid unbounded recursion: reruns are explicit and bounded via `START_AT=<step>`.

---

## Reruns (archive instead of delete)

When rerunning from a mid-chain step, existing stable step log dirs are not deleted. They are archived by renaming:
- `<FEATURE_DIR>/logs/<step>/` → `<FEATURE_DIR>/logs/<step>_run_N/` (per-step sequential numbering)

This prevents stale stable sentinels from being reused and preserves audit history.

---

## Validation guidance

During pre-planning research, prefer focused validation:
- `make planning-validate FEATURE_DIR="<FEATURE_DIR>"`

If `ci_checkpoint_plan.md` is applicable and touched:
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "<FEATURE_DIR>"`

Avoid assuming `make planning-lint` is green during pre-planning.

---

## Work Lift evidence (optional)

If the Touch Set is materially filled in (strict packs), the wrapper may compute pack-derived lift:
- `make pm-lift-pack PACK="<FEATURE_DIR>"`
- `make pm-lift-pack PACK="<FEATURE_DIR>" EMIT_JSON=1`

Store outputs under `<FEATURE_DIR>/logs/` to keep them as evidence without impacting canonical docs.
