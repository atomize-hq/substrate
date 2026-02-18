# Initiative 1 — Slice Specs v2 (Rev A): AC IDs, Smaller Slices, Validator with Rollout Gating, and updated CI checkpoint sizing

**Status:** Draft (implementation-ready, rev A)

**Applies to:**
- **New Planning Packs:** v2 enabled by default (scaffolded)
- **Existing Planning Packs:** opt-in per-pack; no breaking changes until opted in

**Primary outcome:** Fewer “almost right but missing something” implementations by forcing (a) truly small vertical slices, (b) explicitly addressable acceptance criteria IDs, and (c) mechanical validation that slice specs are complete and wired into `tasks.json`.

---

## Changelog vs Rev0 (what this revision fixes)

This revision incorporates review feedback:

1. **Rollout gating / back-compat**: adds `meta.slice_spec_version` opt-in and makes the validator **non-breaking** for existing packs unless explicitly enabled.
2. **Docs drift**: expands “Files impacted” for checkpoint sizing changes to include `PLANNING_README.md` and `PLANNING_QUALITY_GATE_PROMPT.md` (both mention min=2/max=4 today).
3. **Standards/example drift**: updates required edits to `TASK_TRIADS_AND_FEATURE_SETUP.md` so its `tasks.json` example matches AC ID traceability.
4. **Traceability semantics clarified**: explicitly states whether `ac_ids` is a **contract mapping** (recommended) vs a per-task “subset ownership” mapping (not required for v2).
5. **Validator tolerance**: clarifies parsing rules for `-` vs `*` bullets, wrapped bullets, code fences, and Windows path separators; also clarifies discovery scope so we don’t accidentally validate non-slice specs.

---

## 0. Executive summary

This initiative tightens the unit of work from “slice spec with some bullets” to a **slice spec with explicitly addressable acceptance criteria (AC IDs)**, plus a **validator that enforces slice size, structure, and traceability**.

Key changes:

1. **Slice spec format v2** (required sections + “single behavior delta” section).
2. **Acceptance Criteria IDs**: `AC-<SLICE_ID>-NN` (e.g., `AC-WS0-01`).
3. **Traceability contract (machine-checkable)**:
   - Slice specs contain AC IDs.
   - `tasks.json` triad tasks contain an `ac_ids` list that maps to the slice spec’s AC IDs.
4. **Validator**: `docs/project_management/system/scripts/planning/validate_slice_specs.py` runs in planning lint and blocks invalid v2 slice specs (but only when opted in).
5. **Checkpoint boundary sizing update** (because slices become smaller): update default checkpoint group bounds from **2–4** to **4–8** triads (details below).

---

## 1. Goals

### 1.1 Goals (must achieve)
- **Hard-enforce small slices** so that:
  - parallel code/test alignment improves, and
  - follow-up “fix slices” are less common.
- **Make every acceptance criterion addressable** (AC IDs), so tests and implementation can map to a concrete checklist.
- **Make slice-spec completeness mechanical**, not subjective:
  - no “None yet” scaffolds,
  - no missing sections,
  - no oversized AC lists.
- **Improve traceability** across:
  - `slice spec` → `tasks.json` → `kickoff prompts` → `tests`.

### 1.2 Non-goals (explicitly out of scope for this initiative)
- Directory layout refactor (handled in Initiative 3).
- Impact map enforcement (handled in Initiative 2).
- “Supervised parallelism / live steering” (future initiative).
- ADR / Decision Register narrowing rules (future initiative).

---

## 2. Rollout gating and back-compat (critical)

### 2.1 Why gating is required

Planning lint (`docs/project_management/system/scripts/planning/lint.*`) is intentionally strict and is used by the quality gate. If we add a new validator unconditionally, **existing packs** that contain legacy slice spec scaffolds (e.g., `None yet.`) will fail immediately.

So: **slice spec v2 enforcement must be explicitly opted-in per Planning Pack**.

### 2.2 Opt-in mechanism (recommended)

Add a new meta field in `tasks.json`:

```json
{
  "meta": {
    "slice_spec_version": 2
  }
}
```

Rules:
- If `meta.slice_spec_version` is missing: treat as **v1** (legacy) and **do not enforce v2 rules**.
- If `meta.slice_spec_version >= 2`: enforce **v2 rules strictly** and fail lint on violations.

### 2.3 Validator behavior by version

`validate_slice_specs.py` MUST implement the following modes:

- **v1 mode (default / legacy)**:  
  - Do **not** enforce v2 headers, AC ID formats, or traceability.
  - Only verify that slice specs referenced by triad tasks *exist* (and are readable).
  - Exit **0** even if slice spec content is old-style (this avoids breaking existing packs).

- **v2 mode (opted-in)**:
  - Enforce all v2 invariants described in sections 3 and 4.
  - Exit **non-zero** with actionable errors on any violation.

Optional (future-proofing): add a CLI override flag:
- `--force-v2` to enforce v2 regardless of meta (useful later when you want global enforcement).

---

## 3. New standards and hard rules (v2 only)

### 3.1 Slice spec format v2 (required)

Every v2 slice spec MUST contain the following sections (exact headers required):

1. `## Behavior delta (single)`
2. `## Scope`
3. `## Behavior (authoritative)`
4. `## Acceptance criteria`
5. `## Out of scope`

> Note: v1 slice specs may use `## Behavior` instead of `## Behavior (authoritative)`. That is allowed only when `meta.slice_spec_version < 2`.

---

### 3.2 “Single behavior delta” rule (hard-enforced)

In `## Behavior delta (single)`, the spec MUST contain **exactly one** each:

- `- Existing: …`
- `- New: …`
- `- Why: …`

If you need a second “Existing/New/Why”, you need a second slice.

---

### 3.3 AC ID schema (hard-enforced)

In `## Acceptance criteria`, each **top-level** bullet MUST start with an AC ID:

- `- AC-<SLICE_ID>-NN: <criterion>`
- or `* AC-<SLICE_ID>-NN: <criterion>` (both `-` and `*` are accepted)

Rules:
- `<SLICE_ID>` MUST match the slice id (e.g., `WS0`, `WDP3`, `OR1`).
- `NN` is two digits, starting at `01`, no gaps recommended.
- IDs MUST be unique within the spec.
- Maximum AC count: **8** (hard limit). Recommended: **3–6**.

---

### 3.4 Placeholder rules (hard-enforced)

The following are forbidden anywhere in required sections of a v2 slice spec:
- `None yet.`
- `TBD`, `TODO`, `WIP`, `TBA`
- `[[FILL]]` (v2 template placeholder; see section 4.1)

---

### 3.5 Out-of-scope is mandatory (hard-enforced)

`## Out of scope` MUST NOT be empty and MUST list at least one explicit exclusion.

If the slice truly has no meaningful exclusion (rare), it must explicitly say:

- `- None (this slice is intentionally comprehensive for this delta).`

---

### 3.6 CI checkpoint boundary sizing update (baked into this initiative)

Because slices are becoming smaller, checkpoint grouping must expand so checkpoints don’t become too frequent.

**New defaults (recommended and enforced by templates + standards):**
- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

Exceptions:
- If total slice count `< min_triads_per_checkpoint`, a single checkpoint may cover the entire feature.
- If a slice group is high-risk (protocol/schema/FS semantics/platform guards/policy enforcement), smaller groups are allowed with explicit rationale in `ci_checkpoint_plan.md`.

---

## 4. Concrete file changes (implementation plan)

### 4.1 Add a v2 slice spec template

**Add file:**
- `docs/project_management/standards/templates/slice_spec.v2.md.tmpl`

Template contents (example):

```md
# {{SLICE_ID}}-spec — [[FILL]] short descriptive title

## Behavior delta (single)
- Existing: [[FILL]] describe current behavior
- New: [[FILL]] describe new behavior
- Why: [[FILL]] reason / benefit / problem solved

## Scope
- [[FILL]] minimal surfaces required

## Behavior (authoritative)
### [[FILL]] subheading
- [[FILL]] deterministic rules (exit codes, error text constraints, file paths, etc.)

## Acceptance criteria
- AC-{{SLICE_ID}}-01: [[FILL]] criterion
- AC-{{SLICE_ID}}-02: [[FILL]] criterion
- AC-{{SLICE_ID}}-03: [[FILL]] criterion

## Out of scope
- [[FILL]] explicit exclusions
```

Rationale:
- Uses `[[FILL]]` so the validator can **mechanically** block incomplete scaffolds once v2 is enabled.
- Does not rely on the repo-wide `TBD` hard-ban scan (though that also exists).

---

### 4.2 Update planning feature scaffolding to use the template

**Modify:**
- `docs/project_management/system/scripts/planning/new_feature.sh`
- `docs/project_management/system/scripts/planning/new_feature.ps1`

Changes:
1. Replace the heredoc that writes `${SLICE_ID}-spec.md` with a `render` call:

Bash (`new_feature.sh`):

```bash
render "${TEMPLATES_DIR}/slice_spec.v2.md.tmpl" "${FEATURE_DIR}/${SLICE_ID}-spec.md"
```

PowerShell (`new_feature.ps1`):
- Add the equivalent `Render-Template` call for `slice_spec.v2.md.tmpl`.

2. In the tasks.json meta scaffold, set:

```json
"slice_spec_version": 2
```

So all new packs adopt v2 by default.

> If you want a softer rollout for new packs, you can set it to `1` in scaffolding and require upgrading to `2` during execution preflight. But the recommended stance is: **new packs are v2 immediately**, and lint fails until the spec is filled.

---

### 4.3 Add a slice spec validator (with gating)

**Add file:**
- `docs/project_management/system/scripts/planning/validate_slice_specs.py`

#### 4.3.1 Validator inputs
- `--feature-dir <path>`: the Planning Pack directory (currently `docs/project_management/_archived/next/<feature>`).

Optional:
- `--force-v2` (future): enforce v2 regardless of meta.

#### 4.3.2 Slice discovery rules (to avoid validating non-slice specs)

The validator MUST derive slice IDs from tasks of type `code` and `test`:

- A slice ID is the prefix of task ids:
  - `<SLICE>-code` (type=`code`)
  - `<SLICE>-test` (type=`test`)

This avoids accidentally validating unrelated `*-spec.md` files (e.g., policy specs) that are not slice triads.

#### 4.3.3 Locating each slice spec file

For each discovered slice id `X`, locate a spec file via:

1. Search the triad tasks’ `references[]` for a path whose basename is one of:
   - `X-spec.md` (current layout)
   - `X-spec.md` under a slice directory (future layout; Initiative 3): `slices/X/X-spec.md`

2. If not found in references, fall back to:
   - `<feature-dir>/X-spec.md`

**Path normalization requirements:**
- Handle both `/` and `\` in `references` (Windows-friendly).
- Convert to OS paths before `os.path.exists()` checks.

#### 4.3.4 Required checks (v2 mode only)

For each slice spec:

1. **Required headers exist** (exact strings):
   - `## Behavior delta (single)`
   - `## Scope`
   - `## Behavior (authoritative)`
   - `## Acceptance criteria`
   - `## Out of scope`

2. **No forbidden placeholders** appear in required sections:
   - `None yet.`, `TBD`, `TODO`, `WIP`, `TBA`, `[[FILL]]`

3. **Behavior delta shape**:
   - Exactly one `Existing:`, one `New:`, one `Why:` bullet.
   - Accept `-` or `*` bullets with any indentation.

4. **Acceptance criteria parsing**:
   - Count ACs by matching bullet starters only (wrapped lines are allowed).
   - Ignore code-fenced blocks when scanning for headers/ACs (basic fence detection: lines starting with ``` toggle “in fence”).
   - Each top-level AC bullet must match:
     - `^\s*[-*]\s+AC-<SLICE_ID>-\d\d:\s+.+`
   - AC count: `1..8`
   - No duplicate AC IDs

5. **Out-of-scope is non-empty**:
   - At least one bullet entry.

Optional (recommend strict):
- `## Behavior (authoritative)` must include at least one `###` subsection.

#### 4.3.5 Traceability checks (v2 mode only)

**Recommended semantics (keep it simple and strong):**
- `ac_ids` on triad tasks is a **contract mapping**, not “ownership”.  
  In this workflow, all three tasks are accountable to the full slice contract:
  - `X-code` implements the behaviors
  - `X-test` enforces them
  - `X-integ` reconciles + verifies + merges

Therefore:

- For `X-code`, `X-test`, and the integration aggregator `X-integ`:
  - `ac_ids` MUST exist
  - `ac_ids` MUST match **exactly** the set of AC IDs found in `X-spec.md`

Notes:
- For cross-platform packs, there may also be:
  - `X-integ-core`, `X-integ-<platform>` tasks  
  These may optionally include `ac_ids`, but the validator only *requires* the triad tasks listed above.

> If you later decide you truly need “subset ownership”, add an optional field like `ac_ids_focus` (subset) and keep `ac_ids` as the full contract. That preserves mechanical validation without weakening traceability.

---

### 4.4 Update tasks schema documentation and examples (to avoid drift)

Even though `additionalProperties: true` allows `ac_ids` today, we want standards and examples to match reality.

**Modify:**
- `docs/project_management/standards/tasks.schema.json`

Add an optional property on task objects:

```json
"ac_ids": {
  "type": "array",
  "items": { "type": "string" },
  "description": "Acceptance criteria IDs for the slice contract (e.g., AC-WS0-01). For v2 slice packs, triad tasks should include the full set from the slice spec."
}
```

**Modify:**
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

Update the example entry to include `ac_ids`, and update acceptance criteria wording:

Example:

```json
"ac_ids": ["AC-<SLICE_ID>-01", "AC-<SLICE_ID>-02", "AC-<SLICE_ID>-03"],
"acceptance_criteria": [
  "Implements the behaviors required by ac_ids (see <SLICE_ID>-spec.md)"
]
```

This keeps `acceptance_criteria[]` human-readable while the validator keys off `ac_ids`.

---

### 4.5 Add validator to planning lint (safe because gated)

**Modify:**
- `docs/project_management/system/scripts/planning/lint.sh`
- `docs/project_management/system/scripts/planning/lint.ps1`

Add after `validate_spec_manifest.py`:

```bash
echo "-- slice spec invariants (gated by meta.slice_spec_version)"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "${FEATURE_DIR}"
```

Because the validator is gated, this will not break existing packs that do not opt in.

---

### 4.6 Update CI checkpoint defaults everywhere they are documented

**Modify:**
- `docs/project_management/standards/templates/ci_checkpoint_plan.md.tmpl`
  - `"min_triads_per_checkpoint": 2` → `4`
  - `"max_triads_per_checkpoint": 4` → `8`

**Modify:**
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
  - Update defaults and rationale.

**Modify (docs drift fix):**
- `docs/project_management/standards/PLANNING_README.md`
  - Update the line that states “default checkpoint size bounds: min=2 triads, max=4 triads …”.

**Modify (docs drift fix):**
- `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
  - Update the line that states “default group size bounds: min=2 triads, max=4 triads …”.

---

## 5. Migration strategy

### 5.1 For new Planning Packs (immediate)
- `new_feature.*` scaffolds v2 slice spec template and sets `meta.slice_spec_version: 2`.
- Planning lint will fail until `[[FILL]]` placeholders are removed and v2 invariants are satisfied.

### 5.2 For existing Planning Packs (incremental, opt-in)
- Existing packs remain valid and unaffected until you opt in.
- To migrate a pack:
  1. Set `meta.slice_spec_version: 2` in `tasks.json`
  2. Update each slice spec to v2 headers + AC IDs + remove placeholders
  3. Add `ac_ids` to each triad task and align them to the slice spec

Recommended: migrate “active” packs first; leave historical packs untouched.

---

## 6. Test plan (how to verify implementation)

### 6.1 Verify gating behavior
- Pick an existing pack that does **not** set `meta.slice_spec_version`.
- Run:
  - `docs/project_management/system/scripts/planning/lint.sh --feature-dir <pack>`
- Expected:
  - `validate_slice_specs.py` reports SKIP or runs in v1 mode and exits 0.

### 6.2 Verify v2 enforcement behavior
- Create a new pack with `make planning-new-feature ...` after scaffolding is updated.
- Run lint.
- Expected:
  - Fails until you remove all `[[FILL]]` tokens and satisfy the format rules.

### 6.3 “Bad spec” test cases (validator must catch)
Create a scratch v2 pack and intentionally introduce:
- Missing `## Behavior delta (single)`
- Two `- Existing:` lines
- AC bullets without IDs
- 9+ AC items
- Duplicate AC IDs
- Missing/empty out-of-scope
- `ac_ids` missing or mismatched in tasks.json

Validator must fail with actionable errors.

---

## 7. Acceptance criteria for this initiative

- New features created via `docs/project_management/system/scripts/planning/new_feature.*` include a v2 slice spec scaffold (template-based).
- The v2 scaffold includes `[[FILL]]` placeholders and cannot pass lint until filled.
- `docs/project_management/system/scripts/planning/validate_slice_specs.py` exists, is gated by `meta.slice_spec_version`, and is invoked by planning lint.
- For v2 packs, slice specs must have AC IDs and ≤ 8 AC items; triad tasks must include `ac_ids` matching the spec.
- Checkpoint sizing defaults updated to **4–8** in:
  - `ci_checkpoint_plan.md.tmpl`
  - `PLANNING_CI_CHECKPOINT_STANDARD.md`
  - `PLANNING_README.md`
  - `PLANNING_QUALITY_GATE_PROMPT.md`
- At least one existing Planning Pack is migrated end-to-end as a proof point.
