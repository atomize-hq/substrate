# Planning Pre-Planning Research Wrapper (Standard)

This standard describes the **Pre-Planning Research** orchestration workflow and the intended contracts between:
- the orchestration wrapper agent, and
- the focused single-output planning agents (`spec_manifest`, `impact_map`).

Canonical prompt:
- `docs/project_management/system/prompts/planning/pre_planning_wrapper.md`

---

## Goal

Produce (or rapidly converge on) the initial, high-signal Planning Pack artifacts needed to reduce thrash before full planning:

- `spec_manifest.md` (deterministic spec selection + ownership map)
- `impact_map.md` (touch set + cascading implications + cross-queue scan)
- `ci_checkpoint_plan.md` (cross-platform automation packs only; provisional until slice boundaries stabilize)

Non-goals:
- Completing slice specs or passing `make planning-lint` on a newly scaffolded pack.
  - New packs scaffold Slice Spec v2 placeholders, and v2 lint forbids placeholders until full planning fills them.

---

## Preconditions

1) A Planning Pack exists under `docs/project_management/packs/<bucket>/<feature>/`.
   - Recommended scaffold: `make planning-new-feature FEATURE=<feature> ...`

2) The pack’s `tasks.json` can resolve ADR inputs for focused agents.
   - Strict packs (`meta.slice_spec_version >= 2`) must set at least one of:
     - `meta.adr_refs` (preferred), or
     - `meta.adr_paths` (use when refs are ambiguous).

3) The orchestrator keeps the feature directory clean between focused agent runs.
   - The focused agents are executed via `make pm-run-planning-agent ...` which enforces a single-output rule.
   - Commit the output of each focused agent before running the next.

---

## Artifacts and ownership

### Canonical artifacts (tracked; pack root)

- `spec_manifest.md` is authoritative for:
  - which spec documents must exist under the pack, and
  - which contract surfaces each spec document owns.

- `impact_map.md` is authoritative for:
  - the Touch Set allowlist (strict packs),
  - cascading implications, and
  - cross-queue overlap/conflict resolution notes.

- `ci_checkpoint_plan.md` is authoritative for CI cadence (schema v4+):
  - Checkpoint grouping, gates per checkpoint, and rationale.
  - `tasks.json meta.checkpoint_boundaries` must match checkpoint group endings.

### Scratch artifacts (untracked; logs)

The wrapper may produce scratch outputs for caching and synthesis under:
- `<FEATURE_DIR>/logs/**`

Rationale:
- `docs/project_management/packs/**/logs/` is gitignored.
- Scratch notes must not be mistaken for canonical contract statements and should not be reviewed as part of quality gate.

---

## Orchestration pattern (staggered overlap)

The wrapper should model “overlap” deterministically as **bounded iterations**, not time-based concurrency:

1) Run `spec_manifest` once.
2) Run `impact_map` once (consuming `spec_manifest.md`).
3) If (and only if) `impact_map` reveals ownership gaps or missing required docs:
   - re-run `spec_manifest` once, then re-run `impact_map` once.

Hard rule:
- Avoid unbounded recursion. If ADR intent is insufficient, record follow-ups in the canonical artifacts and stop.

---

## Validation guidance

During pre-planning research, prefer focused validation:

- `make planning-validate FEATURE_DIR="<FEATURE_DIR>"`
  - checks `tasks.json` shape and (for strict packs) reference formats/existence rules.

Avoid assuming `make planning-lint` is green during pre-planning.

---

## Work Lift evidence (optional)

If the Touch Set is materially filled in (strict packs), the wrapper may compute pack-derived lift:
- `make pm-lift-pack PACK="<FEATURE_DIR>"`
- `make pm-lift-pack PACK="<FEATURE_DIR>" EMIT_JSON=1`

Store outputs under `<FEATURE_DIR>/logs/` to keep them as evidence without impacting canonical docs.

