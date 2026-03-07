# persist-detected-linux-distro-pkg-manager — workstream triage

Goal: propose pack-internal planning workstreams (PWS) and hard sequencing gates for full planning.

## Evidence

- Canonical artifacts relied on:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `tasks.json`
- Stable sentinels:
  - `logs/spec-manifest/last_message.md`
  - `logs/impact-map/last_message.md`
  - `logs/min-spec-draft/last_message.md`
  - `logs/CI-checkpoint/last_message.md`
- Slice prefix from `pre-planning/minimal_spec_draft.md`:
  - `PDLDPM`
- Draft slice skeleton from `pre-planning/minimal_spec_draft.md`:
  - `PDLDPM0`, `PDLDPM1`, `PDLDPM2`
- Work Lift evidence:
  - Intake/ADR: `logs/workstream-triage/pm_lift_intake.{txt,json}`
  - Pack-derived: `logs/workstream-triage/pm_lift_pack.{txt,json}`

## Work Lift summary

- Intake/ADR lift (`ADR-0032`):
  - `lift_score=9`
  - `estimated_slices=1`
  - `confidence=high`
- Pack lift (strict-pack eligible because `tasks.json.meta.slice_spec_version=2`):
  - `lift_score=41`
  - `estimated_slices=4`
  - `confidence=low`
  - triggers:
    - `split_required:estimated_slices>3`
    - `likely_split:lift_score>24`
    - `likely_split:touch_files_sum>12`
  - derived touch counts:
    - create=`9`
    - edit=`7`
    - delete=`0`
    - deprecate=`0`

Interpretation:
- Keep one planning pack.
- Add one slice boundary during full planning.
- Main churn seam is dual-installer parity:
  - `pre-planning/impact_map.md` selects both `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
  - the dev-installer path overlaps other draft packs, so full planning needs a separate slice boundary instead of burying it inside the production installer write-guarantee slice.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "PDLDPM",
  "pws": [
    {
      "id": "PDLDPM-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0032 remains authoritative for Linux-only behavior delta and no-new-exit-code posture",
        "best-effort-distro-package-manager remains authoritative for selected manager names and pkg_manager.source semantics"
      ],
      "owns": [
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "PDLDPM-PWS-contract"
      ],
      "assumes": [
        "schema_version remains 1 and additive-only evolution stays in force"
      ],
      "owns": [
        "install-state-schema-spec.md",
        "compatibility-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-implementation_seams",
      "role": "implementation",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory"
      ],
      "assumes": [
        "Shared-script edits stay metadata-scoped and do not absorb unrelated staging or provisioning churn"
      ],
      "owns": [
        "slices/PDLDPM0/PDLDPM0-spec.md",
        "slices/PDLDPM1/PDLDPM1-spec.md",
        "slices/PDLDPM3/PDLDPM3-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory",
        "PDLDPM-PWS-implementation_seams"
      ],
      "assumes": [
        "Behavior smoke remains Linux-only while CI parity remains linux/macos/windows"
      ],
      "owns": [
        "platform-parity-spec.md",
        "plan.md",
        "slices/PDLDPM2/PDLDPM2-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "PDLDPM-PWS-docs_validation"
      ],
      "assumes": [
        "Single checkpoint remains viable after the recommended slice split because the feature is still one coherent install-state contract"
      ],
      "owns": [
        "tasks.json",
        "session_log.md",
        "kickoff_prompts/",
        "slices/PDLDPM0/kickoff_prompts/",
        "slices/PDLDPM1/kickoff_prompts/",
        "slices/PDLDPM2/kickoff_prompts/",
        "slices/PDLDPM3/kickoff_prompts/",
        "pre-planning/alignment_report.md",
        "pre-planning/ci_checkpoint_plan.md",
        "pre-planning/impact_map.md",
        "pre-planning/minimal_spec_draft.md",
        "pre-planning/spec_manifest.md",
        "pre-planning/workstream_triage.md"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams

### PDLDPM-PWS-contract — operator contract + DR selections

- Goal:
  - lock the Linux-only persistence contract before slice specs fan out
  - resolve dry-run, incomplete-input, and installer-entrypoint-scope decisions
- Owned surfaces:
  - `contract.md`
  - `decision_register.md`
- Dependencies:
  - none
- Proposed slices/triads to create during full planning:
  - doc gate only; this workstream does not own slice specs

### PDLDPM-PWS-schema_inventory — additive schema + compatibility rules

- Goal:
  - pin the additive `host_state.platform.*` shape, omission rules, corrupt/wrong-schema handling, and preservation of existing `host_state.group` / `host_state.linger`
- Owned surfaces:
  - `install-state-schema-spec.md`
  - `compatibility-spec.md`
- Dependencies:
  - `PDLDPM-PWS-contract`
- Proposed slices/triads to create during full planning:
  - none directly; this PWS supplies the schema/compat truth used by implementation slices

### PDLDPM-PWS-implementation_seams — schema capture + installer parity slice specs

- Goal:
  - author slice specs for:
    - `PDLDPM0` additive platform metadata capture
    - `PDLDPM1` production installer successful-install write guarantee
    - `PDLDPM3` dev-installer parity for the same install-state contract
- Owned surfaces:
  - `slices/PDLDPM0/PDLDPM0-spec.md`
  - `slices/PDLDPM1/PDLDPM1-spec.md`
  - `slices/PDLDPM3/PDLDPM3-spec.md`
- Dependencies:
  - `PDLDPM-PWS-contract`
  - `PDLDPM-PWS-schema_inventory`
- Proposed slices/triads to create during full planning:
  - `PDLDPM0-{code,test,integ}`
  - `PDLDPM1-{code,test,integ}`
  - `PDLDPM3-{code,test,integ}`

### PDLDPM-PWS-docs_validation — platform parity + validation slice

- Goal:
  - author the validation-facing slice spec and the pack-level plan after the implementation seams are stable
  - keep Linux behavior-smoke expectations and macOS/Windows no-delta notes coherent with the tracked CI checkpoint plan
- Owned surfaces:
  - `platform-parity-spec.md`
  - `plan.md`
  - `slices/PDLDPM2/PDLDPM2-spec.md`
- Dependencies:
  - `PDLDPM-PWS-contract`
  - `PDLDPM-PWS-schema_inventory`
  - `PDLDPM-PWS-implementation_seams`
- Proposed slices/triads to create during full planning:
  - `PDLDPM2-{code,test,integ}`

### PDLDPM-PWS-tasks_checkpoints — single writer for automation wiring

- Goal:
  - populate `tasks.json`, checkpoint wiring, session log scaffolding, and per-slice kickoff prompts after slice boundaries are stable
- Owned surfaces:
  - `tasks.json`
  - `session_log.md`
  - `kickoff_prompts/`
  - `slices/PDLDPM0/kickoff_prompts/`
  - `slices/PDLDPM1/kickoff_prompts/`
  - `slices/PDLDPM2/kickoff_prompts/`
  - `slices/PDLDPM3/kickoff_prompts/`
- Dependencies:
  - `PDLDPM-PWS-docs_validation`
- Proposed slices/triads to create during full planning:
  - wire `PDLDPM0`, `PDLDPM1`, `PDLDPM2`, `PDLDPM3`
  - add `CP1-ci-checkpoint` after the validation slice

## Sequencing + gates

1. Gate A: `PDLDPM-PWS-contract`
   - must land first because DR choices change scope, failure posture, and shared-script boundaries
2. Gate B: `PDLDPM-PWS-schema_inventory`
   - must land before slice specs so field names, omission rules, and compat posture are stable
3. Gate C: `PDLDPM-PWS-implementation_seams`
   - author `PDLDPM0`, `PDLDPM1`, and recommended `PDLDPM3`
   - keep the production and dev installer seams separate to reduce merge churn with other packs touching `dev-install-substrate.sh`
4. Gate D: `PDLDPM-PWS-docs_validation`
   - author `PDLDPM2`, `platform-parity-spec.md`, and `plan.md` after implementation seams are fixed
5. Gate E: `PDLDPM-PWS-tasks_checkpoints`
   - single final writer for `tasks.json`, prompts, and checkpoint wiring

CI checkpoint implications:
- Current `pre-planning/ci_checkpoint_plan.md` defines one checkpoint, `CP1`, ending at `PDLDPM2`.
- If the recommended slice split is accepted:
  - keep one checkpoint
  - update the checkpoint slice list to `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, `PDLDPM2`
  - keep the checkpoint boundary at `PDLDPM2`
  - set `meta.checkpoint_boundaries` to `["PDLDPM2"]` when slice tasks exist

## Risks + unknowns

- High-churn seam:
  - `scripts/substrate/dev-install-substrate.sh` overlaps `stabilize-dev-install-helper-discovery` and `dev-install-world-agent-staging`; isolating dev-installer parity into its own slice reduces merge risk
- High-churn seam:
  - `pkg_manager.selected` and `pkg_manager.source` vocabulary is dependency-owned by `best-effort-distro-package-manager`; this pack must link to it, not restate it
- Unknown:
  - exact dry-run persistence rule still needs DR resolution
- Unknown:
  - exact corrupt-file and wrong-schema handling still needs compatibility-spec text
- Unknown:
  - whether `tests/installers/install_smoke.sh` needs behavior assertions beyond dev-installer parity coverage, or remains parity-only support for the shared contract
- Follow-up:
  - `pre-planning/impact_map.md` includes `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` in the edit set, but dispatcher constraints prohibit ADR edits in this pass; full planning must reconcile whether that remains a tracked touch or an external follow-on

## Slice skeleton recommendations

Starting point:
- `pre-planning/minimal_spec_draft.md` defines `PDLDPM0`, `PDLDPM1`, `PDLDPM2`

Recommended edits:
- `KEEP PDLDPM0`
  - boundary: additive `host_state.platform.*` persistence contract and detector-output reuse
- `SPLIT PDLDPM1`
  - revised `PDLDPM1`: successful-install write guarantee and merge semantics in `scripts/substrate/install-substrate.sh`
  - `ADD PDLDPM3`: dev-installer parity for the same install-state contract in `scripts/substrate/dev-install-substrate.sh` plus the matching `tests/installers/install_smoke.sh` seam
- `KEEP PDLDPM2`
  - boundary: end-to-end validation slice that proves Linux smoke behavior and records no-delta expectations for macOS/Windows

Recommended order:
- `PDLDPM0` → `PDLDPM1` → `PDLDPM3` → `PDLDPM2`

Why:
- Pack-derived lift estimates `4` slices and emits `split_required:estimated_slices>3`
- `impact_map.md` confirms a real high-churn boundary between production installer writes and dev-installer parity work
- keeping validation last preserves one coherent checkpoint seam

## Evidence links

- Sentinels:
  - `logs/spec-manifest/last_message.md`
  - `logs/impact-map/last_message.md`
  - `logs/min-spec-draft/last_message.md`
  - `logs/CI-checkpoint/last_message.md`
- Canonical artifacts:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `tasks.json`
- Lift outputs:
  - `logs/workstream-triage/pm_lift_intake.txt`
  - `logs/workstream-triage/pm_lift_intake.json`
  - `logs/workstream-triage/pm_lift_pack.txt`
  - `logs/workstream-triage/pm_lift_pack.json`

## Follow-ups

- When full planning accepts or rejects `PDLDPM3`, update `pre-planning/ci_checkpoint_plan.md` and `tasks.json` together.
- If full planning rejects the split, treat dev-installer parity as the highest-risk sub-seam inside `PDLDPM1` and avoid mixing unrelated helper-staging changes into that slice.
