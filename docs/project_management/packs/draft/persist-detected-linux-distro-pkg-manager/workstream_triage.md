# persist-detected-linux-distro-pkg-manager — workstream triage

Goal: propose parallelizable workstreams + sequencing gates for full planning/execution.

## Evidence (inputs + completion sentinels)

Canonical artifacts relied on:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` (`meta.slice_spec_version=2` strict pack)

Stable sentinels:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/CI-checkpoint/last_message.md`

Work Lift evidence (advisory):
- Pack-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_pack.{txt,json}`
- Intake/ADR-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_intake.{txt,json}`

## Work Lift summary (advisory; prioritize triggers + confidence)

- Intake/ADR lift: `lift_score=9`, `estimated_slices=1`, `confidence=high`
- Pack lift (strict; impact-map touch set): `lift_score=6`, `estimated_slices=1`, `confidence=low` (many missing-input flags)
- Takeaway: small touch set (3 edits), but **contract-sensitive**:
  - shared persistence boundary (`$SUBSTRATE_HOME/install_state.json`, schema_version=1), and
  - dependency seam on upstream detection outputs (`/etc/os-release` + pkg-manager selection).

## Proposed workstreams

### WS-DEP — Upstream detection outputs contract (dependency gate)

Goal:
- Ensure `/etc/os-release` parsing + pkg-manager selection are **authoritatively defined** and exposed as outputs this pack can persist, without re-deriving detection logic here.

Owns (surfaces / contracts):
- Dependency contract (expected): `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Cross-ADR coordination: ADR-0031 “detecting_badger” (impact-map overlap)

Depends on:
- None (external prerequisite for this pack’s C0/C1)

Full-planning slices/triads to create:
- None in this pack; this is a hard gate for WS-SPEC + WS-INSTALLERS.

### WS-SPEC — Persistence contract + schema + decision register

Goal:
- Lock the operator contract + on-disk schema extension for Linux-only persistence into `$SUBSTRATE_HOME/install_state.json` (additive; `schema_version` remains `1`).

Owns (surfaces / files to create during planning; per `spec_manifest.md`):
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` (DR-0001/2/3)
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` (must include explicit dependency gate on WS-DEP)
- Slice specs: `slices/C0/C0-spec.md`, `slices/C1/C1-spec.md`, `slices/C2/C2-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` population + CP1 wiring (per `ci_checkpoint_plan.md` follow-ups)

Depends on:
- WS-DEP (must reference the authoritative detection outputs contract; do not define parsing/selection locally)

Full-planning slices/triads to create:
- C0 spec (persist platform fields) + C0 triad tasks
- C1 spec (success-path presence guarantee + idempotency + merge model) + C1 triad tasks
- C2 spec (smoke assertions + negative cases) + C2 triad tasks
- CP1 checkpoint task wiring (`meta.checkpoint_boundaries=["C2"]`, `CP1-ci-checkpoint`)

### WS-INSTALLERS — Implementation (prod + dev installers)

Goal:
- Implement best-effort, idempotent persistence of:
  - `host_state.platform.os_release.{id,id_like}`
  - `host_state.platform.pkg_manager.{selected,source}`
  into `$SUBSTRATE_HOME/install_state.json` on **successful Linux installs**, and ensure the file exists post-success even when:
  - `--no-world` is set, and/or
  - no group/linger “host-state events” occurred.

Owns (surfaces / touch set from `impact_map.md`):
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`

Depends on:
- WS-SPEC (DR-0001/2/3 locked; canonicalization; enum set; merge/corrupt-file posture; success-path boundary)
- WS-DEP (persist upstream detection outputs; do not duplicate detection logic)

Full-planning slices/triads to create:
- C0 triad (write platform fields; best-effort `/etc/os-release`; MUST NOT `source` the file)
- C1 triad (ensure `install_state.json` presence guarantee; updates additive; preserves unrelated keys)

### WS-TEST+CI — Smoke assertions + checkpoint execution

Goal:
- Extend installer smoke coverage to assert the new persisted keys + Linux-only semantics and run the single checkpoint CP1.

Owns (surfaces / touch set from `impact_map.md`):
- `tests/installers/install_state_smoke.sh`
- Planning-time wiring: `tasks.json` CP1 task + boundaries (per `ci_checkpoint_plan.md`)

Depends on:
- WS-INSTALLERS (behavior implemented)
- WS-SPEC (assertions must match contract + schema)

Full-planning slices/triads to create:
- C2 triad:
  - Assert `install_state.json` exists after successful Linux install.
  - Assert `host_state.platform.*` keys when `/etc/os-release` is readable.
  - Add negative/skip-safe coverage for missing/unreadable `/etc/os-release` (install must still succeed; fields may be absent, but file must exist).
  - Add an explicit `--no-world` success-path scenario for persistence.
- CP1-ci-checkpoint task (single checkpoint for C0–C2; per `ci_checkpoint_plan.md`)

## Sequencing + gates (hard constraints)

1) **WS-DEP gate:** upstream detection outputs must be authoritative (or explicitly pinned to current installer behavior) before implementing persistence (C0/C1).
2) **WS-SPEC gate:** DR-0001/2/3 + schema/canonicalization + merge model must land before C0/C1 code to prevent churn on a shared on-disk boundary.
3) Execute C0 + C1 (installers), then C2 (smoke assertions).
4) Execute CP1 after C2 (per `ci_checkpoint_plan.md`): compile parity + feature smoke + quick CI across `linux/macos/windows`.

## Risks + unknowns (to resolve during full planning)

- `pkg_manager.source` enum set + mapping rules are underspecified (DR-0003).
- Canonicalization rules for persisted `os_release.id_like` (“raw string”) must be made deterministic.
- Merge/overwrite posture for pre-existing or corrupted `install_state.json` must be explicit (including preservation of unrelated keys).
- “Successful install” boundary for the required write must be pinned (contract-level).
- Ensure persistence is not gated on world enablement (`--no-world`) and does not introduce a new required runtime dependency solely for persistence (e.g., do not make `python3` required).
- Known seam: uninstall reads `${HOME}/.substrate/install_state.json` without `SUBSTRATE_HOME` support (non-default prefixes may have “metadata exists but cleanup can’t find it” behavior unless explicitly accepted or addressed elsewhere).

## Follow-ups

- In `plan.md`, add an explicit dependency gate referencing the authoritative upstream detection contract doc path(s) and the exact outputs this pack persists.
- Decide (and pin in schema + contract) whether persisted `pkg_manager.selected` refers to the installer’s prereq installer manager (`apt-get|dnf|yum|pacman|zypper`) vs a broader abstraction; ensure it matches WS-DEP’s contract to avoid drift.
