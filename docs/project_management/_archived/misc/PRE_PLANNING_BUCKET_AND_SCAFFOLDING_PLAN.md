# Pre‑Planning Bootstrap: `PM_DEFAULT_PACK_BUCKET` + ADR→Pack Scaffolding

## Context / Problem

Pre‑planning research is intended to start from an **ADR draft**, before a Planning Pack directory exists.

Current reality in this repo:
- The pre‑planning orchestrator (`make pm-pre-planning-research …`) requires an existing `FEATURE_DIR` with `tasks.json`.
- The canonical pack scaffolder (`make planning-new-feature …` → `docs/project_management/system/scripts/planning/new_feature.sh`) currently hardcodes creation under `docs/project_management/packs/active/<feature>/`.
- `pm_paths.py` already defines `PM_DEFAULT_PACK_BUCKET` and returns `pm_default_pack_bucket` in `print-roots`, but most tooling does not consume it yet.

We want pre‑planning to default to **draft packs** (e.g. `docs/project_management/packs/draft/<feature>/`) until the work is aligned and ready to promote.

## Goals

1) **Bucket correctness in pre‑planning**
- Pre‑planning should default to `docs/project_management/packs/draft/<feature>/`.
- The default bucket must be controlled by `PM_DEFAULT_PACK_BUCKET` (with a sane default for pre‑planning, likely `draft`).
- Tooling should allow explicit overrides (CLI flag / Make var) without requiring edits.

2) **ADR→Pack bootstrap**
- Add a deterministic “bootstrap” step that:
  - accepts an ADR draft path,
  - derives or accepts a feature slug,
  - scaffolds a minimal Planning Pack directory (pre‑planning scaffolding),
  - records ADR linkage into `tasks.json` (`meta.adr_paths` and/or `meta.adr_refs`),
  - then runs the existing pre‑planning chain on that new pack.

3) **Keep existing full-planning flows working**
- Existing `planning-new-feature` (full pack scaffolding) must remain usable.
- Any bucket changes must be backwards compatible or gated behind new commands/flags.

## Non‑Goals

- Implementing the “research backplane” intake pipeline.
- Refactoring the entire PM system to eliminate every `packs/active` mention in all historical docs in one pass.
- Changing core planning semantics beyond the bucket + bootstrap behavior.

---

## Inventory (what exists today)

### Roots / env
- `PM_DEFAULT_PACK_BUCKET` is defined in:
  - `docs/project_management/system/scripts/planning/pm_paths.py` (`print-roots` returns `pm_default_pack_bucket`)
- Documented in:
  - `docs/project_management/system/README.md`

### Scaffolding
- Full planning pack scaffolder (hardcoded to active):
  - `docs/project_management/system/scripts/planning/new_feature.sh`
  - `docs/project_management/system/scripts/planning/new_feature.ps1`
  - `make planning-new-feature` and `make planning-new-feature-ps`

### Pre‑planning orchestration (assumes pack exists)
- Orchestrator:
  - `docs/project_management/system/scripts/planning/pre_planning_research_orchestrate.sh`
  - `make pm-pre-planning-research FEATURE_DIR=...`
- Focused agent runner:
  - `docs/project_management/system/scripts/planning/run_planning_agent.sh`

---

## Proposed design

### A) Propagate `PM_DEFAULT_PACK_BUCKET` through pre‑planning

**Principle:** bucket selection should be deterministic and centralized, not copy‑pasted.

Plan:
1) Extend the pack scaffolding tooling to support a bucket parameter:
   - Either a new `--bucket <name>` flag, or a default bucket resolved via `pm_paths.py print-roots` (`pm_default_pack_bucket`).
2) For pre‑planning entrypoints, set/assume:
   - `PM_DEFAULT_PACK_BUCKET=draft` (either in a Make target wrapper, or in the new bootstrap command).
3) Update pre‑planning prompts/standards/examples to reference `packs/draft` (not `packs/active`) where appropriate.

Notes:
- `Makefile` guards already accept any `docs/project_management/packs/<bucket>/<feature>` path for `FEATURE_DIR`; the main incompatibility is in scaffolding + examples + ADR template language.

### B) Add an ADR→Pack bootstrap scaffold (pre‑planning scaffolding)

We need a script that creates the *minimum viable pack* for pre‑planning, not the full planning pack.

**Minimal pre‑planning scaffolding (recommended):**
- Create directory:
  - `docs/project_management/packs/<bucket>/<feature>/` where `<bucket>` defaults via `PM_DEFAULT_PACK_BUCKET` (expected `draft` for pre‑planning).
- Create/seed tracked files required by the pre‑planning chain:
  - `tasks.json` (must include top-level `tasks: []` and `meta.adr_paths` or `meta.adr_refs`)
  - `spec_manifest.md` (template scaffold)
  - `impact_map.md` (template scaffold)
  - (optional) `ci_checkpoint_plan.md` only if pre‑planning is explicitly running in cross‑platform automation mode
- Do **not** scaffold slice specs/triads unless explicitly requested (that is full planning scope).

**Feature slug derivation (deterministic):**
- Prefer an explicit `FEATURE=<slug>` input.
- If deriving from ADR filename:
  - `ADR-0123-foo-bar.md` → `foo-bar`
- If deriving from ADR content, only use fields that are already required and stable (avoid heuristic parsing of prose).

**ADR linkage in tasks.json:**
- Write `meta.adr_paths` (repo-relative paths) by default for draft ADRs, because refs can be ambiguous early.
- Optionally also set `meta.adr_refs` when the ADR id is unambiguous and matches registry naming.

### C) Integrate bootstrap into an existing entrypoint

We should not require operators to remember a two-command ceremony (“scaffold, then orchestrate”).

Preferred integration:
- Add a new Make target (or script) that becomes the canonical pre‑planning entrypoint:
  - `make pm-pre-planning-from-adr ADR="docs/project_management/adrs/draft/ADR-XXXX-....md" FEATURE="<slug>" [BUCKET=draft]`
- Behavior:
  1) resolve bucket (default `PM_DEFAULT_PACK_BUCKET`),
  2) compute `FEATURE_DIR` under that bucket,
  3) if `FEATURE_DIR` does not exist → scaffold minimal pre‑planning pack,
  4) then run `make pm-pre-planning-research FEATURE_DIR="..."`

Alternate (acceptable) integration:
- Extend `pm-pre-planning-research` to accept `ADR=...` and auto-scaffold if `FEATURE_DIR` is missing.
  - Downside: changes behavior/contract of an existing command; prefer a new wrapper target.

---

## Work plan (tracked tasks)

### 1) Decide bucket policy (pre‑planning vs full planning)
- [ ] Confirm: pre‑planning default bucket = `draft`
- [ ] Define “promotion to active” trigger and mechanism (separate command/script; out of this immediate bootstrap scope but must be documented)

### 2) Add bucket support to scaffolding
Files likely to change:
- [ ] `docs/project_management/system/scripts/planning/new_feature.sh` (remove `active/` hardcode; accept `--bucket` or read `pm_default_pack_bucket`)
- [ ] `docs/project_management/system/scripts/planning/new_feature.ps1` (same)
- [ ] `Makefile` targets `planning-new-feature` / `planning-new-feature-ps` (stop hardcoding validate path under `packs/active`)

Acceptance:
- `make planning-new-feature FEATURE=foo` still works (defaults may remain `active` for that command if desired).
- `PM_DEFAULT_PACK_BUCKET=draft make planning-new-feature FEATURE=foo` produces `packs/draft/foo/` (if that is the chosen compatibility model).

### 3) Create pre‑planning scaffolder script (minimal pack)
Add:
- [ ] `docs/project_management/system/scripts/planning/scaffold_pre_planning_pack.sh` (name TBD)
  - Inputs: `--adr <path>`, `--feature <slug>`, `--bucket <name optional>`
  - Outputs: minimal pack dir + seeded templates + `tasks.json` with ADR linkage

Potential template sources:
- `docs/project_management/system/templates/planning_pack/spec_manifest.md.tmpl`
- `docs/project_management/system/templates/planning_pack/impact_map.md.tmpl`
- (new) a minimal `tasks.json` template for pre‑planning (or generate JSON directly in script)

### 4) Wire bootstrap into canonical entrypoint
Add:
- [ ] Make target: `pm-pre-planning-from-adr`
- [ ] Or a wrapper script that the prompt references

Update operator-facing docs/prompts (once the command exists):
- [ ] `docs/project_management/system/prompts/planning/pre_planning_wrapper.md` should reference `pm-pre-planning-from-adr` as the starting point when no `FEATURE_DIR` exists.

### 5) Update standards / ADR template language (bucket-aware)
Likely updates:
- [ ] `docs/project_management/system/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md` (examples should default to `packs/draft`)
- [ ] `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
  - Draft ADRs may point at `packs/draft/<feature>/`
  - Accepted ADRs must point at `packs/active/<feature>/` (current contract)

---

## Acceptance tests (smoke)

1) Bootstrap + pre‑planning from ADR:
- `make pm-pre-planning-from-adr ADR="docs/project_management/adrs/draft/ADR-XXXX-foo.md" FEATURE="foo"`
  - Creates `docs/project_management/packs/draft/foo/` (or bucket override)
  - Seeds `tasks.json` with ADR linkage
  - Runs the pre‑planning orchestration chain and produces stable per-step logs/sentinels

2) Bucket override:
- `PM_DEFAULT_PACK_BUCKET=draft …` vs `PM_DEFAULT_PACK_BUCKET=active …` selects output location deterministically.

3) Backwards compatibility:
- Existing `make planning-new-feature FEATURE=bar` continues to produce the expected location (as defined by the chosen policy).

---

## Open questions / decisions to make

- Should pre‑planning `tasks.json` be **minimal** (no triad tasks) or **full scaffold** (slice specs + triads) from day one?
- Should pre‑planning always create `ci_checkpoint_plan.md`, or only when cross‑platform automation is explicitly enabled?
- Do we want a dedicated “promote draft pack to active” command (and what invariants must hold before promotion)?

