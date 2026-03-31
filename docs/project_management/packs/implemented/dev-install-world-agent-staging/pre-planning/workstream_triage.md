# dev-install-world-agent-staging — workstream triage (pre-planning)

## Canonical inputs used

- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json`
- `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`

## Evidence links (stable sentinels)

- `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/CI-checkpoint/last_message.md`

## Lift evidence (v1; advisory)

- Intake/ADR lift: `lift_score=10`, `estimated_slices=1`, `confidence=low` (missing inputs). Evidence: `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/workstream-triage/pm_lift_intake.txt`.
- Pack lift (impact-map-derived): `lift_score=66`, `estimated_slices=6`, `confidence=low`; triggers include `split_required:estimated_slices>3`, `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`. Evidence: `docs/project_management/packs/draft/dev-install-world-agent-staging/logs/workstream-triage/pm_lift_pack.txt`.

Triage interpretation:
- Treat this as a **multi-workstream** planning effort with high merge-conflict risk on shared installer/enable surfaces.
- Full planning must explicitly reconcile the lift "split" triggers with the 2-slice skeleton (either justify the 2-slice shape or split the ADR into multiple items).

## Proposed workstreams (full planning)

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "DIWAS",
  "accepted_slice_order": ["DIWAS0", "DIWAS1"],
  "pws": [
    {
      "id": "DIWAS-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "Contract decisions in decision_register.md remain the single source of truth for contract.md wording changes."
      ],
      "owns": [
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "DIWAS-PWS-slice_spec_diwas0",
      "role": "slice_spec",
      "depends_on": [
        "DIWAS-PWS-contract"
      ],
      "assumes": [
        "DIWAS0 spec references contract.md/decision_register.md and does not re-define CLI/config/exit-code tables."
      ],
      "owns": [
        "slices/DIWAS0/DIWAS0-spec.md"
      ]
    },
    {
      "id": "DIWAS-PWS-slice_spec_diwas1",
      "role": "slice_spec",
      "depends_on": [
        "DIWAS-PWS-contract"
      ],
      "assumes": [
        "DIWAS1 spec references contract.md/decision_register.md and stays scoped to Linux dev-install staging."
      ],
      "owns": [
        "slices/DIWAS1/DIWAS1-spec.md"
      ]
    },
    {
      "id": "DIWAS-PWS-docs_validation",
      "role": "tests_ci",
      "depends_on": [
        "DIWAS-PWS-contract",
        "DIWAS-PWS-slice_spec_diwas0",
        "DIWAS-PWS-slice_spec_diwas1"
      ],
      "assumes": [
        "Linux smoke and the manual playbook align to slice acceptance criteria and the contract (paths, exit codes, ordering)."
      ],
      "owns": [
        "manual_testing_playbook.md",
        "platform-parity-spec.md",
        "smoke/"
      ]
    },
    {
      "id": "DIWAS-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "DIWAS-PWS-contract",
        "DIWAS-PWS-slice_spec_diwas0",
        "DIWAS-PWS-slice_spec_diwas1",
        "DIWAS-PWS-docs_validation"
      ],
      "assumes": [
        "tasks.json schema_version=4 cross_platform=true remains the automation model.",
        "Checkpoint boundaries remain meta.checkpoint_boundaries=[\"DIWAS1\"]."
      ],
      "owns": [
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "pre-planning/ci_checkpoint_plan.md",
        "pre-planning/spec_manifest.md",
        "pre-planning/impact_map.md",
        "pre-planning/minimal_spec_draft.md",
        "pre-planning/workstream_triage.md",
        "kickoff_prompts/",
        "slices/DIWAS0/kickoff_prompts/",
        "slices/DIWAS1/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

### DIWAS-PWS-contract — Contract + Decision Register (pin seams before slice specs)

Goal:
- Make the DIWAS behavior deterministic by pinning **paths, exit codes, `--dry-run` semantics, and overwrite/profile policy** before slice specs and tasks are authored.

Owns (authoritative docs):
- `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md`

Owns (seams to decide; must land first):
- Missing-artifact exit code + minimum remediation text (taxonomy-aligned).
- Staged `world-agent` path sufficiency rule (accept either; search order pinned).
- `substrate world enable --dry-run` behavior when staged artifact is missing (exit code + messaging + “no privileged actions” invariant).
- DR-0001..DR-0005 (implementation locus; meaning of `--no-world`; profile mapping; overwrite/idempotency; staged path sufficiency).

Dependencies:
- None.

Planned slices/triads (creates requirements for):
- `DIWAS0-{code,test,integ}`
- `DIWAS1-{code,test,integ}`

---

### DIWAS-PWS-slice_spec_diwas0 — DIWAS0 slice spec (enable preflight + remediation)

Goal:
- Specify DIWAS0’s acceptance criteria and contracts for “fail fast when staged `world-agent` is missing” (before any privileged steps).

Owns:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS0/DIWAS0-spec.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS0/kickoff_prompts/DIWAS0-{code,test,integ}.md`

Owned implementation surfaces (per `pre-planning/impact_map.md`):
- `scripts/substrate/world-enable.sh`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/tests/world_enable.rs`

Dependencies:
- DIWAS-PWS-contract (must have pinned exit code + remediation + `--dry-run` semantics + staged-path rule).

Planned slices/triads:
- `DIWAS0-{code,test,integ}`

---

### DIWAS-PWS-slice_spec_diwas1 — DIWAS1 slice spec (dev-install `--no-world` stages `world-agent`)

Goal:
- Specify DIWAS1’s acceptance criteria and contracts for Linux dev-install staging so “install with `--no-world`, enable later” is execution-ready.

Owns:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS1/DIWAS1-spec.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS1/kickoff_prompts/DIWAS1-{code,test,integ}.md`

Owned implementation surfaces (per `pre-planning/impact_map.md`):
- `scripts/substrate/dev-install-substrate.sh`
- `tests/installers/install_smoke.sh`

Dependencies:
- DIWAS-PWS-contract (must have pinned staged-path rule + profile mapping + overwrite policy).
- DIWAS-PWS-slice_spec_diwas0 is upstream by slice ordering (`DIWAS0` then `DIWAS1`), but the slice specs can be authored in parallel once DIWAS-PWS-contract decisions are pinned.

Planned slices/triads:
- `DIWAS1-{code,test,integ}`

---

### DIWAS-PWS-docs_validation — Validation artifacts (smoke + manual playbook + CI-proof)

Goal:
- Ensure validation is deterministic and maps to the 2-slice spine and CI checkpoint plan.

Owns:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/manual_testing_playbook.md`

Dependencies:
- DIWAS-PWS-contract (exit codes + path rules + dry-run semantics).
- DIWAS-PWS-slice_spec_diwas0/DIWAS-PWS-slice_spec_diwas1 (slice AC must exist to reference from smoke/playbook).

Planned slices/triads:
- Treat as acceptance surface for `DIWAS0-integ` and `DIWAS1-integ` (smoke + playbook assertions must map to AC IDs).

---

### DIWAS-PWS-tasks_checkpoints — Automation + checkpoints (tasks/plan/session log/quality gate)

Goal:
- Wire the planning pack into executable triads and checkpoint cadence (cross-platform compile parity + Linux behavior smoke).

Owns:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/plan.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/session_log.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/quality_gate_report.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json` (populate `tasks`; keep `meta.checkpoint_boundaries=["DIWAS1"]` aligned to checkpoint plan)

Dependencies:
- DIWAS-PWS-slice_spec_diwas0/DIWAS-PWS-slice_spec_diwas1 (slice specs define AC IDs and kickoff scope).
- DIWAS-PWS-docs_validation (smoke script existence/contract informs `feature_smoke` gate wiring).

Planned slices/triads:
- Add triad tasks:
  - `DIWAS0-{code,test,integ}`
  - `DIWAS1-{code,test,integ}`
- Add checkpoint task(s) per `pre-planning/ci_checkpoint_plan.md` (e.g., `CP1-ci-checkpoint`) and wire dependencies.

## Sequencing + gates (hard constraints)

1) **Gate A — Pin seams (DIWAS-PWS-contract)**
   - Decisions in `decision_register.md` + the external contract in `contract.md` must land before slice specs are finalized.

2) **Gate B — Slice specs (DIWAS-PWS-slice_spec_diwas0/DIWAS-PWS-slice_spec_diwas1)**
   - Land DIWAS0 spec before DIWAS1 spec is considered final (slice ordering), but authoring can proceed in parallel once Gate A is done.

3) **Gate C — Tasks + checkpoint wiring (DIWAS-PWS-tasks_checkpoints)**
   - Populate `tasks.json` tasks + kickoff prompts + checkpoint tasks; ensure `meta.behavior_platforms_required=["linux"]` and `meta.ci_parity_platforms_required=["linux","macos","windows"]` remain aligned to validation artifacts.

4) **Gate D — Validation readiness (DIWAS-PWS-docs_validation)**
   - Linux smoke + manual playbook must be mechanically runnable and referenced by slice acceptance criteria.

## Risks + unknowns (must be resolved in full planning)

- High-churn seams (must be decided early; otherwise slice specs churn):
  - Staged artifact path set (`bin/world-agent` vs `bin/linux/world-agent`, BOTH vs ONE).
  - Missing-artifact exit code + remediation text contract.
  - `--dry-run` semantics for missing staged artifact.
  - Profile mapping + overwrite/idempotency.
- Cross-queue conflict risk (per `pre-planning/impact_map.md`):
  - Shared surfaces with other queued work (notably `scripts/substrate/world-enable.sh`, `crates/shell/src/builtins/world_enable/runner.rs`, and dev-install helper discovery work) increase rebase/merge risk; keep changes orthogonal and small.
- Lift split trigger:
  - Pack lift flags `estimated_slices>3`; planning must either justify 2 slices explicitly or split ADR scope into multiple items.

## Slice skeleton recommendations (required)

Starting point (from `pre-planning/minimal_spec_draft.md`):
- `DIWAS0` (enable preflight)
- `DIWAS1` (dev-install staging)

Recommendation:
- **No change** to slice IDs/skeleton at pre-planning.
- If strict lift posture later requires addressing `estimated_slices>3`, prefer splitting ADR scope into multiple ADRs/packs rather than introducing new `DIWAS*` slice IDs (the minimal spec draft explicitly pins `DIWAS0`/`DIWAS1` as stable).

## Follow-ups

- Path canonicalization:
  - `pre-planning/ci_checkpoint_plan.md` is the canonical checkpoint plan location for this pack (aligned with other strict packs).
- Touch-set alignment: `pre-planning/minimal_spec_draft.md` lists `scripts/substrate/install-substrate.sh` under DIWAS1 likely touch surfaces, but `pre-planning/impact_map.md` does not currently list it; confirm whether it is actually in-scope and update the touch set or minimal spec accordingly.
- ADR hygiene (do not do in this workstream): ADR-0035 has internal option-label inconsistency and stale Related Docs links; reconcile during full planning per spec-manifest follow-ups.
