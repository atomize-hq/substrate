# persist-detected-linux-distro-pkg-manager — workstream triage

Goal: propose parallelizable workstreams + sequencing gates for full planning/execution.

## Evidence (inputs + completion sentinels)

Canonical artifacts relied on:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` (`meta.slice_spec_version=2` strict pack; `meta.cross_platform=true`; `meta.behavior_platforms_required=["linux"]`)

Stable sentinels:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/CI-checkpoint/last_message.md`

Work Lift evidence (advisory):
- Pack-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_pack.{txt,json}`
- Intake/ADR-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_intake.{txt,json}` (ADR-0032)

## Work Lift summary (advisory; prioritize triggers + confidence)

- Intake/ADR lift: `lift_score=9`, `estimated_slices=1`, `confidence=high` (no split triggers)
- Pack lift (strict; impact-map touch set): `lift_score=6`, `estimated_slices=1`, `confidence=low` (missing-input flags)
  - Derived touch set counts: `edit=3` (no creates/deletes/deprecations)

Takeaway:
- No split recommended; keep one pack and one primary slice (`PDL0`).
- Treat as **contract-sensitive** despite small touch set: installer write semantics + on-disk schema + “must exist” vs best-effort tension.

## Proposed workstreams

### PDL-PWS-contract — Operator contract + schema + decision register (hard gate)

Goal:
- Lock a deterministic, testable persistence contract for Linux-only platform metadata in `$SUBSTRATE_HOME/install_state.json` (`schema_version=1` unchanged), including explicit best-effort failure posture and consumer read precedence guidance.

Owns (surfaces / files to create during full planning; per `spec_manifest.md` + Touch Set):
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` (DR-0001..DR-0004; required by `impact_map.md`)
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` (encode gates + validation commands)
- `docs/INSTALLATION.md` (document new keys + privacy posture; allowlist/denylist statement)

Depends on:
- Upstream detection contract (owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/`) for authoritative semantics/vocabulary of:
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source` (do not redefine locally)

Full-planning slices/triads to create:
- Doc-first gate (before PDL0 acceptance is treated as stable): resolve the Decision Register items from `impact_map.md`:
  - DR-0001: “file must exist” guarantee vs best-effort persistence (writer dependencies, fallback strategy, required warning surface)
  - DR-0002: pin `pkg_manager.source` semantics by deferring to upstream detection contract
  - DR-0003: scope decision for `scripts/substrate/dev-install-substrate.sh` (keep out by default unless explicitly included)
  - DR-0004: overwrite policy on re-run (preserve vs overwrite; missing-input behavior)

### PDL-PWS-installer — Implement persistence in installer (PDL0-code)

Goal:
- Update the Linux installer to write/update `$SUBSTRATE_HOME/install_state.json` on successful installs (including `--no-world`) and persist `host_state.platform.*` without introducing new exit-code behavior.

Owns (surfaces / touch set from `impact_map.md`):
- `scripts/substrate/install-substrate.sh`

Depends on:
- PDL-PWS-contract (must pin write semantics, atomicity posture, absence semantics, overwrite policy, and best-effort behavior)
- Upstream detection semantics (avoid re-parsing `/etc/os-release` independently; persist upstream-derived outputs to prevent drift)

Full-planning slices/triads to create:
- `PDL0-code` triad scope:
  - Ensure `install_state.json` exists after successful Linux install even when no group/linger events occurred.
  - Persist the new keys exactly as pinned by schema/contract:
    - `host_state.platform.os_release.id`
    - `host_state.platform.os_release.id_like`
    - `host_state.platform.pkg_manager.selected`
    - `host_state.platform.pkg_manager.source`
  - Confirm macOS/Windows remain no-op for these fields (Linux-only behavior delta).

### PDL-PWS-tests_ci — Installer smoke assertions (PDL0-test)

Goal:
- Extend installer smoke coverage to make the new persistence contract observable and regression-proof.

Owns (surfaces / touch set from `impact_map.md`):
- `tests/installers/install_state_smoke.sh`

Depends on:
- PDL-PWS-contract (exact key set + absence semantics + “successful install” definition)
- PDL-PWS-installer (behavior must exist; test design can proceed once contract is pinned)

Full-planning slices/triads to create:
- `PDL0-test` triad scope:
  - Assert `install_state.json` exists after successful Linux install (including `--no-world` case).
  - Assert new keys are present when `/etc/os-release` is available.
  - Assert missing/unreadable `/etc/os-release` does not fail install and still records `pkg_manager.*` with an explicit fallback `source` (per upstream detection contract).

### PDL-PWS-tasks_checkpoints — Checkpoint wiring + validation (CP1)

Goal:
- Wire the slice triads and checkpoint task graph so CI cadence matches the authored checkpoint plan (`CP1` covering `PDL0`).

Owns:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/ci_checkpoint_plan.md` (authoritative cadence; already authored)
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` (add slice triads + checkpoint wiring once full planning creates tasks)

Depends on:
- Minimal slice skeleton (`minimal_spec_draft.md`): `PDL0`
- PDL-PWS-contract/PDL-PWS-installer/PDL-PWS-tests_ci (triad acceptance criteria must be stable to wire `*-integ` + CP1 deps)

Full-planning slices/triads to create:
- Add `PDL0-{code,test,integ-*}` tasks + `CP1-ci-checkpoint` ops task and kickoff prompt (per `ci_checkpoint_plan.md` Follow-ups).
- Run and keep passing (once wiring exists):
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`

## Sequencing + gates (hard constraints)

1) PDL-PWS-contract gate: resolve DR-0001..DR-0004 and lock schema + absence semantics before treating `PDL0` slice spec acceptance criteria as stable (prevents churn).
2) Implement `PDL0-code` (installer persistence), then land `PDL0-test` (smoke assertions).
3) Wire `PDL0-integ` + `CP1-ci-checkpoint` and run CP1 gates per `ci_checkpoint_plan.md` (compile parity + Linux feature smoke + quick CI testing).
4) Cross-pack seam guardrail: keep changes in `scripts/substrate/install-substrate.sh` narrowly scoped to persistence to reduce merge conflict risk with:
   - `docs/project_management/packs/draft/best-effort-distro-package-manager/` (detection/selection semantics), and
   - other installer-touching packs referenced in `impact_map.md`.

## Risks + unknowns (to resolve during full planning)

- DR-0001: reconcile “install_state.json must exist after successful Linux install” with “do not hard-fail solely due to metadata persistence” (writer deps like `python3`, fallback strategy, warning/diagnostics).
- `$SUBSTRATE_HOME` resolution + unwritable prefix behavior must be explicit (contract + plan).
- Absence semantics must be pinned and testable (omit vs null vs sentinel string) for `os_release.{id,id_like}`.
- Overwrite semantics on re-run (DR-0004) must be explicit to prevent silent drift across repeated installs.
- Cross-pack drift risk: persisted values must match the installer’s detection/selection outputs and `pkg_manager.source` vocabulary owned by the upstream detection pack.
- Scope drift risk: whether to extend persistence to `dev-install-substrate.sh` is explicitly a decision (default: keep out of scope unless selected).

## Slice skeleton recommendations (pre-planning; required)

Starting point:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/minimal_spec_draft.md` defines draft slice `PDL0`.

Recommended changes:
- None (keep one slice; seams are clean: contract/schema/decision pinning + installer persistence + smoke assertions within `PDL0`).

## Follow-ups

- Populate `tasks.json` with slice triads + CP1 wiring (per `ci_checkpoint_plan.md` Follow-ups), then validate mechanically:
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`
- ADR link drift follow-up (do not fix in this pack): ADR-0032 references `stashing-ferret` feature dir; reconcile to `persist-detected-linux-distro-pkg-manager` during planning.
- Dependency naming follow-up: ADR intake references `detecting_badger`; reconcile to the upstream pack path `best-effort-distro-package-manager` and encode sequencing in `docs/project_management/packs/sequencing.json`.
