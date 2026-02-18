# Initiative 1 — Slice Specs v2: AC IDs, Smaller Slices, and a Slice Spec Validator (plus updated CI checkpoint sizing)

**Status:** Draft (implementation-ready)  
**Applies to:** All new Planning Packs; existing packs migrated opportunistically  
**Primary outcome:** Fewer “almost right but missing something” implementations by forcing (a) truly small vertical slices, (b) explicit acceptance criteria IDs, and (c) mechanical validation that the slice spec is complete and wired into `tasks.json`.

---

## 0. Executive summary

This initiative tightens the *unit of work* from “slice spec with some bullets” to a **slice spec with explicitly addressable acceptance criteria (AC IDs)**, plus a **validator that enforces slice size, structure, and traceability**.

Key changes:

1. **Slice spec format v2** (required sections + “single behavior delta” section).
2. **Acceptance Criteria IDs**: `AC-<SLICE_ID>-NN` (e.g., `AC-WS0-01`).
3. **Traceability contract**:
   - Slice specs contain AC IDs.
   - `tasks.json` tasks for that slice reference those AC IDs (either directly or by “covers all AC-*”).
4. **Validator**: `docs/project_management/system/scripts/planning/validate_slice_specs.py` runs in planning lint and blocks incomplete/oversized slice specs.
5. **Checkpoint boundary sizing update** (because slices become smaller): update default `min/max triads per checkpoint` from **2–4** to **4–8** (details below). This is baked into templates + standards.

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

## 2. Definitions

- **Slice**: the smallest vertical feature increment, executed as a triad (code/test/integ) and governed by one slice spec.
- **Triad**: `<SLICE>-code`, `<SLICE>-test`, `<SLICE>-integ`.
- **AC (Acceptance Criterion)**: a testable statement of required behavior.
- **AC ID**: a stable identifier for an AC in a slice spec.

---

## 3. New standards and hard rules

### 3.1 Slice spec format v2 (required)

Every slice spec MUST contain the following sections (exact headers required):

1. `## Behavior delta (single)`  
2. `## Scope`  
3. `## Behavior (authoritative)`  
4. `## Acceptance criteria`  
5. `## Out of scope`

**Rationale:** When “Behavior delta” is missing, specs tend to include multiple deltas that should be split. When Acceptance Criteria don’t have IDs, drift goes undetected.

---

### 3.2 “Single behavior delta” rule (hard-enforced)

In `## Behavior delta (single)`, the spec MUST contain exactly:

- `- Existing: …`
- `- New: …`
- `- Why: …`

This is a forcing function. If you need a second “Existing/New/Why”, you need a second slice.

---

### 3.3 AC ID schema (hard-enforced)

In `## Acceptance criteria`, each bullet MUST start with an AC ID:

- `- AC-<SLICE_ID>-NN: <criterion>`

Rules:
- `<SLICE_ID>` MUST match the slice id (e.g., `WS0`, `WDP3`, `OR1`).
- `NN` is two digits, starting at `01`, no gaps recommended.
- IDs MUST be unique within the spec.
- Maximum AC count: **8** (hard limit). Recommended: **3–6**.

> Why max 8? If a slice needs >8 acceptance criteria, it is almost always multiple deltas or multiple subsystems. Smaller slices are the primary lever for parallel agent correctness.

---

### 3.4 Scope and out-of-scope rules (hard-enforced)

- `## Scope` must include the minimal surfaces required to satisfy ACs.
- `## Out of scope` MUST NOT be empty and MUST list at least one clear exclusion (unless the slice truly has none; then explicitly say: `- None (this slice is intentionally comprehensive for this delta).`).

Additionally:
- Placeholder scaffolds such as `None yet.` are forbidden in any required section.

---

### 3.5 CI checkpoint boundary sizing update (baked into this initiative)

Because slices are becoming *smaller*, the default checkpoint grouping must expand so checkpoints don’t become too frequent.

**New defaults (recommended and enforced by templates):**
- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

Exceptions (unchanged conceptually):
- If total slice count `< min_triads_per_checkpoint`, a single checkpoint may cover the entire feature.
- If a slice group is high-risk (protocol/schema/FS semantics/platform guards/policy enforcement), smaller groups are allowed with explicit rationale in `ci_checkpoint_plan.md`.

**Files impacted:**
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`

---

## 4. Concrete file changes (implementation plan)

### 4.1 Add a slice spec template

**Add file:**
- `docs/project_management/system/templates/planning_pack/slice_spec.v2.md.tmpl`

Template contents (example):

```md
# {{SLICE_ID}}-spec — <short descriptive title>

## Behavior delta (single)
- Existing: <what happens today, as observed by the user/system>
- New: <what will happen after this slice lands>
- Why: <reason / benefit / problem solved>

## Scope
- <one or two bullets; keep tight>

## Behavior (authoritative)
### <subheading 1>
- <deterministic rules; include exit codes, error text constraints, file paths, etc.>

### <subheading 2>
- ...

## Acceptance criteria
- AC-{{SLICE_ID}}-01: <criterion>
- AC-{{SLICE_ID}}-02: <criterion>
- AC-{{SLICE_ID}}-03: <criterion>

## Out of scope
- <explicit exclusions>
```

Notes:
- Keep the template minimal; do not embed large prompts here (Initiative 3 moves prompts out).

---

### 4.2 Update planning feature scaffolding to use the template

**Modify:**
- `docs/project_management/system/scripts/planning/new_feature.sh`
- `docs/project_management/system/scripts/planning/new_feature.ps1`

Change:
- Replace the hardcoded heredoc that writes `<SLICE_ID>-spec.md` with a `render` call to the new template.

Bash (`new_feature.sh`):
- Add:

```bash
render "${TEMPLATES_DIR}/slice_spec.md.tmpl" "${FEATURE_DIR}/${SLICE_ID}-spec.md"
```

- Remove the heredoc that writes:

```md
## Scope
- None yet.
...
```

PowerShell (`new_feature.ps1`):
- Add a `Render-Template` call equivalent to your existing template rendering logic.
- Ensure `{{SLICE_ID}}` is substituted.

---

### 4.3 Add a slice spec validator

**Add file:**
- `docs/project_management/system/scripts/planning/validate_slice_specs.py`

#### 4.3.1 Validator inputs
- `--feature-dir <path>`: the Planning Pack directory (currently `docs/project_management/_archived/next/<feature>`; later Initiative 3 changes the root, but this validator must be path-agnostic).

#### 4.3.2 Validator discovery rules (compatible with current + future layouts)
The validator MUST find slice specs in either of these ways:

- **Primary:** Parse `tasks.json` and discover slice ids from triad tasks, then locate their spec via `references`:
  - For each slice id `X`, locate an existing file in `task.references` that ends with either:
    - `X-spec.md` (current layout), or
    - `slices/X/X-spec.md` (future layout), or
    - `slices/X/spec.md` (optional future variant)
- **Fallback:** Search under `feature-dir` for files matching `*-spec.md` where the basename prefix matches a slice id derived from tasks.

Rationale: keeps validator stable across directory refactors.

#### 4.3.3 Required checks (strict)
For each slice spec:
1. Must contain required headers:
   - `## Behavior delta (single)`
   - `## Scope`
   - `## Behavior (authoritative)`
   - `## Acceptance criteria`
   - `## Out of scope`
2. Must not contain placeholders:
   - `None yet.`
   - `TBD`, `TODO`, `WIP`, `TBA`
3. `Behavior delta (single)` must contain exactly one each:
   - `- Existing:`
   - `- New:`
   - `- Why:`
4. Acceptance criteria format:
   - Every AC bullet begins with `AC-<SLICE_ID>-NN:`
   - `NN` must be `01..99` (2 digits)
   - No duplicate AC IDs
   - **AC count <= 8**
5. Optional but recommended (warn or strict—choose strict for new packs):
   - `## Behavior (authoritative)` should include at least one `###` subsection (prevents “wall of text”).

#### 4.3.4 Traceability checks (strict)
The validator must ensure the slice spec’s AC IDs are wired into `tasks.json`:

Option A (recommended): Add a new optional field `ac_ids` to tasks (see 4.4). Then:
- For `X-code`, `X-test`, `X-integ` tasks:
  - `ac_ids` must exist
  - `ac_ids` must match exactly the set of AC IDs in the spec

Option B (minimal changes): If you do not want a new field:
- Require that each of `X-code`, `X-test`, `X-integ` tasks has `acceptance_criteria[]` entries that mention the AC IDs (either enumerated or via `AC-X-*`).
- This is less machine-checkable; Option A is strongly preferred.

---

### 4.4 Update tasks schema documentation (optional but recommended)

Even though your JSON schema allows extra fields, formally documenting AC mapping helps tools and humans.

**Modify:**
- `docs/project_management/standards/tasks.schema.json`

Add an optional property to task items:

```json
"ac_ids": {
  "type": "array",
  "items": { "type": "string" },
  "description": "Acceptance criteria IDs this task is responsible for (e.g., AC-WS0-01)."
}
```

---

### 4.5 Add validator to planning lint

**Modify:**
- `docs/project_management/system/scripts/planning/lint.sh`
- `docs/project_management/system/scripts/planning/lint.ps1`

Add a new step after `validate_spec_manifest.py` (ordering suggestion):

```bash
echo "-- slice spec invariants"
python3 docs/project_management/system/scripts/planning/validate_slice_specs.py --feature-dir "${FEATURE_DIR}"
```

PowerShell equivalent.

---

### 4.6 Update CI checkpoint default sizing (templates + standard)

**Modify:**
- `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`
  - Change:
    - `"min_triads_per_checkpoint": 2` → `4`
    - `"max_triads_per_checkpoint": 4` → `8`

**Modify:**
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
  - Update the default values in the doc text.
  - Add a short rationale: “Slice Specs v2 reduces slice size, so checkpoint grouping expands.”

No changes are required in:
- `docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py`  
Because it reads the defaults from the file and enforces them.

---

## 5. Migration strategy

### 5.1 For new Planning Packs (immediate)
- New feature scaffolding generates v2 slice spec skeleton.
- Planning lint will fail unless:
  - all required sections exist,
  - AC IDs are present,
  - AC count ≤ 8,
  - placeholders removed.

### 5.2 For existing Planning Packs (incremental)
Choose one of these strategies:

**Strategy A (recommended): migrate “active” packs only**
- Update packs currently being executed or queued.
- Leave historical packs unchanged unless edited again.

**Strategy B: bulk migrate**
- Write a one-off script (optional) to:
  - locate `*-spec.md`,
  - convert acceptance bullets to `AC-<SLICE>-NN:` format,
  - update tasks.json to include `ac_ids`.

> If you bulk migrate, do it in a single mechanical commit with no semantic edits.

---

## 6. Test plan (how to verify implementation)

### 6.1 Local checks (must pass)
For a sample feature dir (e.g., `docs/project_management/_archived/next/world-sync`):

- Run:
  - `docs/project_management/system/scripts/planning/lint.sh --feature-dir <feature_dir>`
- Confirm failures until:
  - slice specs updated with AC IDs,
  - `ac_ids` added (if implementing Option A),
  - placeholders removed.

### 6.2 “Bad spec” test cases (validator must catch)
Create a scratch feature and intentionally introduce:
- Missing `## Behavior delta (single)`
- Two `- Existing:` lines
- AC bullets without IDs
- 9+ AC items
- Duplicate AC IDs

Validator must fail with actionable messages.

---

## 7. Acceptance criteria for this initiative

- New features created via `docs/project_management/system/scripts/planning/new_feature.*` include a v2 slice spec scaffold (no “None yet” placeholders).
- `docs/project_management/system/scripts/planning/validate_slice_specs.py` exists, is strict, and is invoked by planning lint.
- Slice specs must have AC IDs and ≤ 8 AC items, otherwise lint fails.
- `ci_checkpoint_plan.md.tmpl` defaults updated to **4–8** and `PLANNING_CI_CHECKPOINT_STANDARD.md` reflects it.
- At least one existing Planning Pack is migrated end-to-end as a proof point.
