# Feature: Slice Spec v2 (gated) + AC ID traceability + checkpoint defaults 4–8

## Goal
Make slice specs mechanically complete and traceable (AC IDs + `ac_ids` wiring) without breaking legacy Planning Packs, and update CI checkpoint defaults everywhere from 2–4 → 4–8.

## Primary user
Developer / planning-pack author + quality gate reviewer — wants planning lint to catch “almost right but missing something” *before* execution.

## Problem / why now
- Legacy slice spec scaffolds (`None yet.`) + subjective review lets gaps slip through.
- No mechanical mapping from spec acceptance criteria to tasks.
- Smaller slices imply checkpoint grouping should be larger; docs/templates currently drift.

## In scope
- Gated `docs/project_management/system/scripts/planning/validate_slice_specs.py` (v1 existence-only; v2 strict).
- v2 slice spec template + scaffolder updates (`new_feature.sh|ps1`).
- `docs/project_management/system/scripts/planning/lint.sh|ps1` wiring.
- Standards/templates/docs drift fixes (checkpoint defaults, tasks schema + examples).
- Proof-point migration of 1 existing pack to v2.

## Out of scope
- Impact map enforcement at task finish (Initiative 2).
- Directory/prompt refactor / slice directories migration (Initiative 3).
- Changing triad execution model.

## Success criteria
- Existing packs with no `meta.slice_spec_version` still pass planning lint (no new failures).
- New packs default to v2 and fail lint until slice spec `[[FILL]]` removed and `ac_ids` match AC IDs.
- Checkpoint defaults are 4–8 in templates + all docs that mention bounds.
- At least one active pack migrated end-to-end and passes `make planning-lint`.

## Constraints
- Must retain legacy compatibility by default.
- Must keep paths on current system layout for now (Initiative 3 will move later).

## Unknowns / risks
- Decide whether “Behavior (authoritative) must include at least one `###`” is strict or warning.
- Pick which existing Planning Pack is the proof-point migration target.

---

## Vertical Slices

## Slice S1: Gated slice-spec validator + lint hook
**User value**: Quality gate can mechanically validate v2 packs while legacy packs remain unaffected.

**Scope**
- In: implement `validate_slice_specs.py` (v1/v2 modes), wire into planning lint.
- Out: changing any existing slice spec content in packs (migration is S4).

**Acceptance criteria**
- [ ] If `meta.slice_spec_version` is missing, validator exits 0 after verifying slice spec files exist for discovered slices.
- [ ] If `meta.slice_spec_version >= 2`, validator fails on missing headers, placeholders, AC-ID format/count/dup, missing out-of-scope bullets, and missing/mismatched `ac_ids`.
- [ ] `docs/project_management/system/scripts/planning/lint.sh` and `lint.ps1` call the validator safely (no break for legacy).

**Dependencies**
- Blocks on: none
- Unblocks: scaffolder v2 defaults (S2), v2 migrations (S4)

**Verification**
- Automated: run planning lint against (a) a legacy pack (no meta flag) and (b) a v2 pack.
- Manual/demo: show a failing error message includes spec path + actionable fix.

**Rollout**
- Backward compatibility: default v1 mode does not enforce content rules.

### Task S1.T1: Implement `validate_slice_specs.py` (gated v1/v2)
**Outcome**: New validator exists and enforces RevA rules only when opted-in.

**Inputs/outputs**
- Inputs: feature dir contains `tasks.json`; slice IDs derivable from `<SLICE>-code`/`<SLICE>-test`.
- Outputs: `docs/project_management/system/scripts/planning/validate_slice_specs.py`

**Acceptance criteria**
- [ ] Implements v1 mode (existence/readability only).
- [ ] Implements v2 mode with header/placeholder/delta/AC parsing + `ac_ids` traceability.
- [ ] Normalizes Windows path separators in `references[]`.
- [ ] Ignores code fences when scanning.

**Implementation notes**
- Where: `docs/project_management/system/scripts/planning/`
- Keep discovery scoped to slice triads (code/test), not “any `*-spec.md`”.

**Test notes**
- Add at least one “bad spec” fixture per failure class OR a minimal self-test mode; run via `python3 ...`.

**Risk / rollback**
- Risk: false positives on markdown parsing edge cases.
- Rollback: remove lint hook (S1.T2) while keeping script for manual use.

**Checklist**
- [ ] Parse `tasks.json`, read `meta.slice_spec_version`
- [ ] Derive slice IDs from code/test tasks
- [ ] Locate spec path via `references[]` then fallback `<feature>/<SLICE>-spec.md`
- [ ] Implement v1 and v2 checks + clear failure messages
- [ ] Add minimal fixtures or scripted test cases
- [ ] Verify on one legacy pack + one scratch v2 pack

### Task S1.T2: Wire validator into planning lint scripts
**Outcome**: Planning lint runs slice-spec validation (gated).

**Inputs/outputs**
- Inputs: validator script path
- Outputs: updated `docs/project_management/system/scripts/planning/lint.sh`, `docs/project_management/system/scripts/planning/lint.ps1`

**Acceptance criteria**
- [ ] Lint prints a clear section header for slice spec validation.
- [ ] Legacy packs still pass this step.

**Checklist**
- [ ] Add call after `validate_spec_manifest.py`
- [ ] Ensure consistent messaging across bash/ps1
- [ ] Run `docs/project_management/system/scripts/planning/lint.sh --feature-dir <legacy-pack>` locally

---

## Slice S2: v2 slice spec template + scaffolder emits v2-ready metadata
**User value**: New Planning Packs start in v2 format immediately, and the required wiring (`slice_spec_version`, initial `ac_ids`) is present.

**Scope**
- In: add `slice_spec.v2.md.tmpl`, update `new_feature.sh|ps1` to render it and set `meta.slice_spec_version: 2` + seed `ac_ids`.
- Out: any directory refactor (Initiative 3).

**Acceptance criteria**
- [ ] `make planning-new-feature ...` creates `<SLICE>-spec.md` from the v2 template (with `[[FILL]]` placeholders).
- [ ] `tasks.json` meta includes `"slice_spec_version": 2`.
- [ ] Triad tasks include `ac_ids` seeded to match the scaffolded spec AC IDs (e.g., `01..03`).

**Dependencies**
- S1 blocks S2 (validator expectations should be stable before scaffolder emits v2 by default).

**Verification**
- Create a fresh pack and confirm lint fails for placeholders + passes once filled and `ac_ids` aligned.

### Task S2.T1: Add v2 slice spec template
**Outcome**: Canonical v2 template exists.

**Inputs/outputs**
- Outputs: `docs/project_management/standards/templates/slice_spec.v2.md.tmpl`

**Acceptance criteria**
- [ ] Contains required v2 headers and `[[FILL]]` markers.
- [ ] AC bullets use `AC-{{SLICE_ID}}-NN:` format.

**Checklist**
- [ ] Create template file
- [ ] Ensure it matches validator header strings exactly
- [ ] Keep minimal; no giant prompts

### Task S2.T2: Update `new_feature.sh` and `new_feature.ps1` to render v2 template + seed meta/`ac_ids`
**Outcome**: New pack scaffolding is v2-by-default and mechanically consistent.

**Inputs/outputs**
- Inputs: template path, tasks.json scaffolding logic
- Outputs: updated `docs/project_management/system/scripts/planning/new_feature.sh`, `docs/project_management/system/scripts/planning/new_feature.ps1`

**Acceptance criteria**
- [ ] Replaces heredoc slice spec with render call.
- [ ] Adds `meta.slice_spec_version: 2`.
- [ ] Adds `ac_ids` arrays to `X-code`, `X-test`, `X-integ` tasks matching scaffold AC IDs.

**Checklist**
- [ ] Swap heredoc → template render
- [ ] Update tasks.json generator to include `slice_spec_version`
- [ ] Add `ac_ids` to relevant tasks
- [ ] Generate a pack; confirm file presence and JSON validity

---

## Slice S3: Drift cleanup — docs, standards, schema, checkpoint defaults
**User value**: One coherent story across standards/templates/examples; no “2–4” remnants.

**Scope**
- In: update checkpoint defaults (template + standard + docs), update tasks schema + triads doc example for `ac_ids` / `slice_spec_version`.
- Out: broader standards re-org (Initiative 3).

**Acceptance criteria**
- [ ] `ci_checkpoint_plan.md.tmpl` defaults are 4–8.
- [ ] `PLANNING_CI_CHECKPOINT_STANDARD.md` documents 4–8 defaults.
- [ ] `PLANNING_README.md` and `PLANNING_QUALITY_GATE_PROMPT.md` no longer mention min=2/max=4.
- [ ] `tasks.schema.json` documents `ac_ids` and `meta.slice_spec_version`.
- [ ] `TASK_TRIADS_AND_FEATURE_SETUP.md` example shows `ac_ids` + updated acceptance_criteria wording.

**Dependencies**
- Can run in parallel with S1/S2, but final consistency check happens in WS-INT.

**Verification**
- Grep for “min=2” / “max=4” and ensure no relevant hits remain.

### Task S3.T1: Update checkpoint defaults everywhere
**Outcome**: 4–8 is canonical across template + standards + docs.

**Inputs/outputs**
- Outputs:
  - `docs/project_management/standards/templates/ci_checkpoint_plan.md.tmpl`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
  - `docs/project_management/standards/PLANNING_README.md`
  - `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`

**Checklist**
- [ ] Update JSON defaults in template (2/4 → 4/8)
- [ ] Update standard defaults text + rationale
- [ ] Update docs lines that cite min/max bounds
- [ ] `rg` confirm no stale bound statements remain in those files

### Task S3.T2: Update schema + example docs for `ac_ids` and `slice_spec_version`
**Outcome**: Tooling and examples reflect v2 traceability.

**Inputs/outputs**
- Outputs:
  - `docs/project_management/standards/tasks.schema.json` (already has `slice_spec_version`; add `ac_ids` if not yet)
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md` example update

**Checklist**
- [ ] Add `ac_ids` property to task schema items (optional field)
- [ ] Ensure `meta.slice_spec_version` is documented
- [ ] Update example JSON snippet + explanatory text
- [ ] Ensure example stays consistent with validators (human-readable `acceptance_criteria`, machine-readable `ac_ids`)

---

## Slice S4: Proof-point migration of one existing Planning Pack to v2
**User value**: Demonstrates end-to-end viability; catches edge cases before wider rollout.

**Scope**
- In: pick one active pack, migrate slice specs to v2 format, add `slice_spec_version: 2` and `ac_ids`, run lint.
- Out: bulk migration.

**Acceptance criteria**
- [ ] Selected pack passes `make planning-lint` after migration.
- [ ] Migration commit is mechanical and scoped (no semantic changes beyond format + mapping).

**Dependencies**
- S1 + S2 + S3 should be complete first.

**Verification**
- Run `make planning-lint FEATURE_DIR="<pack>"` and record output in pack’s `session_log.md` or PR description.

### Task S4.T1 (Spike): Select proof-point pack + enumerate slices
**Outcome**: A concrete target and migration checklist.

**Outputs**
- Pack path chosen
- Slice IDs list
- Inventory of existing slice spec formats + AC counts

**Checklist**
- [ ] Pick a currently active/queued pack with 1–3 slices
- [ ] List slice IDs from `tasks.json`
- [ ] Confirm slice spec file locations referenced by tasks

### Task S4.T2: Migrate selected pack to v2 (spec headers + AC IDs + tasks `ac_ids`)
**Outcome**: Pack is v2 compliant and lint-clean.

**Inputs/outputs**
- Inputs: pack directory from S4.T1
- Outputs: updated pack docs (`*-spec.md`) + `tasks.json` updates

**Checklist**
- [ ] Set `meta.slice_spec_version: 2`
- [ ] Update each `<SLICE>-spec.md` to required v2 headers + delta section
- [ ] Replace acceptance bullets with `AC-<SLICE>-NN:` format (≤8)
- [ ] Ensure Out-of-scope non-empty
- [ ] Add/align `ac_ids` arrays on `X-code`, `X-test`, `X-integ`
- [ ] Run `make planning-lint FEATURE_DIR="<pack>"` and fix failures

---

## Dependency graph (text)
- S1 blocks S2 (scaffolder must target validator contract)
- S1 blocks S4 (migration depends on validator rules)
- S2 blocks S4 (migration uses v2 scaffolding conventions + `ac_ids` semantics)
- S3 is parallelizable but WS-INT gates final “no drift” consistency
- S4 depends on S1 + S2 + S3

## Risks / unknowns
- Markdown parsing edge cases (nested bullets, wrapped lines, fenced code): mitigate via fixtures + clear “only top-level AC bullets count”.
- Strictness choice for `###` subsections: decide early; if strict causes friction, downgrade to warning but keep placeholders/AC wiring strict.

## Milestones
- M1: Validator exists + lint wired (legacy safe)
- M2: Scaffolder emits v2-by-default (template + meta + seeded `ac_ids`)
- M3: Checkpoint defaults 4–8 are consistent across template/standards/docs
- M4: One existing pack migrated and lint-clean

## Workstreams

### WS-SCRIPTS: Planning validators + lint + scaffolder
**Touch surface**
- `docs/project_management/system/scripts/planning/*`

### WS-DOCS: Standards/templates drift cleanup
**Touch surface**
- `docs/project_management/standards/*`

### WS-MIGRATE: Proof-point pack migration
**Touch surface**
- One selected `docs/project_management/_archived/next/<feature>/...`

### WS-INT: Integration
**Depends on**
- WS-SCRIPTS
- WS-DOCS
- WS-MIGRATE

**Touch surface**
- Cross-cutting verification: run `make planning-lint` on (a) legacy pack, (b) new v2 pack, (c) migrated pack; final grep for stale 2–4 references.

