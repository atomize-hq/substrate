# best-effort-distro-package-manager — workstream triage

Goal: propose parallelizable **planning workstreams (PWS)** + sequencing gates for full planning (pack execution is out of scope here).

## Evidence (inputs + completion sentinels)

Canonical artifacts relied on:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md` (slice prefix: `BEDPM`; draft slices: `BEDPM0..BEDPM2`)
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` (`meta.slice_spec_version=2` strict pack; `meta.cross_platform=true`)

Stable sentinels (step completion markers):
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/CI-checkpoint/last_message.md`

Work Lift evidence (advisory-first; strict-pack eligible):
- Pack-derived: `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_pack.{txt,json}`
- Intake/ADR-derived: `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_intake.{txt,json}` (ADR-0031)

## Work Lift summary (advisory; prioritize triggers + confidence)

- Intake/ADR lift (ADR-0031): `lift_score=16`, `estimated_slices=2`, `confidence=high` (no triggers; `risk.security_sensitive=true`)
- Pack lift (strict; impact-map touch set): `lift_score=28`, `estimated_slices=3`, `confidence=low` (includes `likely_split:lift_score>24` plus missing-input flags)
  - Derived touch set counts (`derived.impact_map_touch_counts`): create=`6`, edit=`5`, delete=`0`, deprecate=`0`

Interpretation:
- Keep one pack.
- Keep the 3-slice skeleton and treat `likely_split:lift_score>24` as a “keep slices tight” signal, not a requirement to add more slices.
- Treat as **contract-sensitive**: user-facing installer semantics (one-liner exactness, deterministic precedence, exit-code scoping, cross-pack seams).

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "BEDPM",
  "pws": [
    {
      "id": "BEDPM-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0031 remains authoritative for one-liner shape, precedence pipeline, and exit-code meanings"
      ],
      "owns": ["contract.md", "decision_register.md"]
    },
    {
      "id": "BEDPM-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": ["BEDPM-PWS-contract"],
      "assumes": [
        "Pack remains Linux-only behavior; macOS/Windows remain no-op behavior surfaces"
      ],
      "owns": ["spec_manifest.md", "impact_map.md", "plan.md", "manual_testing_playbook.md"]
    },
    {
      "id": "BEDPM-PWS-slice_spec_bedpm0",
      "role": "slice_spec",
      "depends_on": ["BEDPM-PWS-contract"],
      "assumes": [],
      "owns": ["slices/BEDPM0/BEDPM0-spec.md"]
    },
    {
      "id": "BEDPM-PWS-slice_spec_bedpm1",
      "role": "slice_spec",
      "depends_on": ["BEDPM-PWS-contract"],
      "assumes": [],
      "owns": ["slices/BEDPM1/BEDPM1-spec.md"]
    },
    {
      "id": "BEDPM-PWS-slice_spec_bedpm2",
      "role": "slice_spec",
      "depends_on": ["BEDPM-PWS-contract"],
      "assumes": [
        "Hermetic harness must not require containers and must not mutate the host"
      ],
      "owns": ["slices/BEDPM2/BEDPM2-spec.md"]
    },
    {
      "id": "BEDPM-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "BEDPM-PWS-docs_validation",
        "BEDPM-PWS-slice_spec_bedpm0",
        "BEDPM-PWS-slice_spec_bedpm1",
        "BEDPM-PWS-slice_spec_bedpm2"
      ],
      "assumes": [
        "CI checkpoint plan remains single-checkpoint (CP1) with boundary at the end of BEDPM2"
      ],
      "owns": [
        "tasks.json",
        "session_log.md",
        "kickoff_prompts/",
        "slices/BEDPM0/kickoff_prompts/",
        "slices/BEDPM1/kickoff_prompts/",
        "slices/BEDPM2/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams (PWS)

### BEDPM-PWS-contract — operator contract + decision register (hard gate)

Goal:
- Lock the operator-visible contract for Linux installer pkg-manager selection:
  - overrides (`--pkg-manager`, `PKG_MANAGER`) + validation + fail-closed rules,
  - precedence pipeline + deterministic PATH ambiguity handling,
  - stderr decision one-liner (exact string; exactly once; emission timing),
  - exit-code scoping (`0|2|3|4`) and whether meanings are Linux-only vs global.

Owned surfaces (full planning outputs; pack-root tracked files):
- `contract.md`
- `decision_register.md` (DR-0001..DR-0003)

Hard dependencies:
- None (upstream gate)

Proposed slices/triads to create during full planning:
- Doc gate: resolve DR-0001 (`/etc/os-release` parsing/canonicalization), DR-0002 (PATH ambiguity order + warning content), DR-0003 (hermetic os-release seam).

### BEDPM-PWS-docs_validation — pack doc set + coherence (promotion + alignment)

Goal:
- Promote pre-planning artifacts into execution-ready pack-root docs and keep them coherent with the contract:
  - `spec_manifest.md` ownership matrix,
  - `impact_map.md` touch set (strict-format compliance, cross-queue scan),
  - `plan.md` with one explicit slice ordering + validation commands,
  - `manual_testing_playbook.md` with deterministic Linux cases + expected outputs.
- Ensure the plan explicitly captures required non-pack touches already in the Touch Set:
  - `docs/INSTALLATION.md`, `docs/project_management/packs/sequencing.json`, and ADR “Related Docs” link corrections (wire as tasks via `tasks.json`).

Owned surfaces (full planning outputs; pack-root tracked files):
- `spec_manifest.md`
- `impact_map.md`
- `plan.md`
- `manual_testing_playbook.md`

Hard dependencies:
- BEDPM-PWS-contract

Proposed slices/triads to create during full planning:
- None (doc workstream; enables implementation slices without owning them)

### BEDPM-PWS-slice_spec_bedpm0 — slice spec: detect distro + report decision one-liner

Goal:
- Author `BEDPM0` acceptance criteria for:
  - best-effort `/etc/os-release` read + safe parsing posture (never `source`),
  - `<unknown>` rendering rules,
  - decision one-liner emission timing and exactly-once invariant.

Owned surfaces (full planning outputs; pack-root tracked files):
- `slices/BEDPM0/BEDPM0-spec.md`

Hard dependencies:
- BEDPM-PWS-contract

Proposed slices/triads to create during full planning:
- `BEDPM0` triad (code/test/integ) as defined by the spec + `contract.md`.

### BEDPM-PWS-slice_spec_bedpm1 — slice spec: deterministic manager selection + precedence + failures

Goal:
- Author `BEDPM1` acceptance criteria for:
  - override precedence (`--pkg-manager` over `PKG_MANAGER` over autodetect),
  - mapping rules + “mapping matched but binary missing” behavior,
  - deterministic PATH probe precedence + required warning elements,
  - exit-code mapping for override/selection failures (`2|3|4`).

Owned surfaces (full planning outputs; pack-root tracked files):
- `slices/BEDPM1/BEDPM1-spec.md`

Hard dependencies:
- BEDPM-PWS-contract

Proposed slices/triads to create during full planning:
- `BEDPM1` triad (code/test/integ) as defined by the spec + `contract.md`.

### BEDPM-PWS-slice_spec_bedpm2 — slice spec: hermetic detection tests + checkpoint seam

Goal:
- Author `BEDPM2` acceptance criteria for a hermetic test harness that asserts:
  - precedence + selected manager,
  - decision one-liner text/fields + `source` enum,
  - exit codes for invalid override / forced manager missing / no supported manager.

Owned surfaces (full planning outputs; pack-root tracked files):
- `slices/BEDPM2/BEDPM2-spec.md`

Hard dependencies:
- BEDPM-PWS-contract

Proposed slices/triads to create during full planning:
- `BEDPM2` triad (code/test/integ), intended to enable the single checkpoint boundary in `ci_checkpoint_plan.md`.

### BEDPM-PWS-tasks_checkpoints — tasks.json + checkpoint wiring + prompts (single writer)

Goal:
- Populate `tasks.json` (single-writer) and align it with:
  - the accepted slice skeleton (`BEDPM0..BEDPM2`),
  - the checkpoint plan (CP1) and required boundaries,
  - kickoff prompts + session log scaffolding needed by automation.

Owned surfaces (full planning outputs; pack-root tracked files):
- `tasks.json` (exclusive writer)
- `session_log.md`
- `kickoff_prompts/` (directory prefix ownership)
- `slices/BEDPM0/kickoff_prompts/` (directory prefix ownership)
- `slices/BEDPM1/kickoff_prompts/` (directory prefix ownership)
- `slices/BEDPM2/kickoff_prompts/` (directory prefix ownership)

Hard dependencies:
- BEDPM-PWS-docs_validation
- BEDPM-PWS-slice_spec_bedpm0
- BEDPM-PWS-slice_spec_bedpm1
- BEDPM-PWS-slice_spec_bedpm2

Proposed slices/triads to create during full planning:
- Wire execution tasks for `BEDPM0`, `BEDPM1`, `BEDPM2` (one triad per slice).
- Wire CP1 checkpoint task(s) per `ci_checkpoint_plan.md` and validate mechanically once tasks exist.

## Sequencing + gates (hard constraints)

1) **Contract gate:** `contract.md` + DR-0001/2/3 must land before slice specs are treated as stable (prevents churn on parsing, precedence, one-liner timing, and test seams).
2) **Doc + slice fan-out:** once contract is pinned, author pack-root docs and slice specs in parallel.
3) **Tasks/checkpoints last:** update `tasks.json` only after the doc + slice spec surfaces exist, to avoid churn and to keep one canonical writer.

## Risks + unknowns (to resolve during full planning)

- `/etc/os-release` parsing corner cases (quotes/whitespace, duplicates, comments, case-sensitivity) must be deterministic (DR-0001).
- PATH ambiguity policy: fixed precedence order + complete warning content elements must be pinned and testable (DR-0002).
- “Mapping matched but binary missing” semantics (outside Fedora `dnf`→`yum` fallback) must be explicit (fallback vs fail; exit code + message guidance).
- “Exactly once” decision one-liner timing must consider:
  - “no prereqs needed” paths, and
  - scripts sourcing `install-substrate.sh` (e.g., `world-enable.sh`) without double-printing.
- Exit-code meanings must be explicitly scoped (Linux-only vs global) to avoid accidental macOS/Windows behavior changes.
- `tasks.json.meta.behavior_platforms_required` currently includes `linux|macos|windows`, but the contract is Linux-only behavior; planning must reconcile this (CI parity vs behavior assertions).
- Cross-pack seam: `persist-detected-linux-distro-pkg-manager` depends on this pack’s detection/selection outputs; vocabulary drift risk is high if `contract.md` is vague.
- Touch Set coherence risk: `impact_map.md` must accurately represent the final pack doc set and slice spec set (avoid drift between `spec_manifest.md`, slice skeleton, and touch entries).

## Slice skeleton recommendations (pre-planning; required)

Starting point:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md` defines draft slices `BEDPM0..BEDPM2` (prefix `BEDPM`).

Recommended changes:
- None (keep 3 slices; seams are clean: detect/report vs select/precedence vs hermetic tests).

## Follow-ups

- Populate `tasks.json` with slice triad tasks + CP1 wiring, then validate mechanically:
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`
- Reconcile ADR “Related Docs” links (`detecting-badger/*` vs `best-effort-distro-package-manager/*`) as part of full planning (Touch Set already includes editing the ADR).
