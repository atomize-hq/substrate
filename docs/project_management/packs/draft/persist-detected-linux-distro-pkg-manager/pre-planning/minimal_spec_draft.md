**Pre-Planning Only: this draft defines an alignment backbone for full planning and must be deleted or retired during full planning.**

# persist-detected-linux-distro-pkg-manager minimal spec draft

## Scope + authority

This draft defines only cross-cutting defaults, precedence reuse, and invariants for ADR-0032 in `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`.

This draft is allowed to define:
- the single persistence target for this feature
- cross-cutting failure posture
- platform and compatibility invariants
- slice IDs and a draft slice skeleton for downstream pre-planning work

This draft must not define:
- slice-specific acceptance behavior
- detailed JSON schema rules beyond the fixed field list and nesting already implied by ADR-0032
- implementation tasks, task wiring, or command sequences

## Defaults + precedence

- Single persistence target: `<resolved SUBSTRATE_HOME>/install_state.json`
- `SUBSTRATE_HOME` meaning and default path resolution remain externally authoritative in `docs/reference/env/contract.md`.
- This feature adds no new CLI flags, config files, or env vars.
- This feature adds no second detection or precedence stack. Persisted `host_state.platform.pkg_manager.*` records the final output of the existing Linux installer detection contract owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- Detection precedence reused by persistence: CLI override, then env override, then `/etc/os-release` driven detection, then PATH probe fallback.
- No feature-local config tier participates in package-manager selection for this draft.

## Failure posture + invariants

- Failure posture: fail-open for metadata persistence. A metadata read, merge, or write failure must not create a new non-zero exit on an otherwise successful install.
- Platform invariant: Linux is the only behavior-delta platform. macOS and Windows remain explicit no-delta platforms.
- Path invariant: metadata writes stay under `<resolved SUBSTRATE_HOME>` only.
- File invariant: this feature extends the existing `install_state.json` surface and does not create a second metadata file.
- Compatibility invariant: `schema_version` remains `1`.
- Preservation invariant: writes preserve existing `host_state.group` and `host_state.linger` content.
- Data-minimization invariant: persisted platform metadata is limited to `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.
- Omission invariant: unavailable `/etc/os-release` values are omitted from JSON rather than replaced with sentinel strings.
- Read invariant for later consumers: prefer persisted metadata when available; fall back to runtime detection when missing or unreadable.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- New exit codes required by this work: no

## Cross-cutting seams / constraints

- One install-state contract spans both `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- One persisted field list spans all downstream docs/specs: `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, `host_state.platform.pkg_manager.source`.
- `pkg_manager.selected` value names, `pkg_manager.source` enum semantics, and the detection algorithm remain externally authoritative in `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
- Planning docs under this feature must use canonical slice IDs `PDLDPM0`, `PDLDPM1`, and `PDLDPM2`.
- Non-authoritative docs that mention `install_state.json` must align to the Linux-only file-presence guarantee and the additive `schema_version=1` contract.

## Follow-ups for full planning

- Define the exact dry-run rule in `contract.md`: write install-state metadata during dry-run or skip it entirely.
- Define the exact handling rule in `compatibility-spec.md` for an existing `install_state.json` that is corrupt or on the wrong schema version.
- Confirm the authoritative dependency state for `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` and `decision_register.md` before finalizing stored `pkg_manager.*` semantics.
- Retire the ADR feature-dir mismatch by updating `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` references from `draft/stashing-ferret/` to `draft/persist-detected-linux-distro-pkg-manager/`.
- Pin the exact Linux validation ownership split between `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh` in `plan.md` and `PDLDPM2-spec.md`.

## Draft slice skeleton (pre-planning only)

Draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `PDLDPM`

CI-checkpoint uses this slice list as the default input when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`. Workstream triage records any recommended edits in `pre-planning/workstream_triage.md` and must not edit this file.

- `slice_id`: `PDLDPM0`
  - `name`: Persist platform metadata
  - `intent`: Stabilize the additive `host_state.platform.*` persistence seam. Reuse the existing detector output without redefining detection semantics.
  - `likely touch surfaces`: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `install-state-schema-spec.md`, `contract.md`, `slices/PDLDPM0/PDLDPM0-spec.md`

- `slice_id`: `PDLDPM1`
  - `name`: Guarantee install-state writes
  - `intent`: Stabilize the successful-Linux-install file-presence rule, `schema_version=1` preservation, and host-state merge invariants for installs with no event payload.
  - `likely touch surfaces`: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `compatibility-spec.md`, `contract.md`, `slices/PDLDPM1/PDLDPM1-spec.md`

- `slice_id`: `PDLDPM2`
  - `name`: Verify persisted metadata contract
  - `intent`: Stabilize Linux smoke coverage for file creation, additive platform keys, and explicit no-delta expectations for non-Linux platforms.
  - `likely touch surfaces`: `tests/installers/install_state_smoke.sh`, `tests/installers/install_smoke.sh`, `plan.md`, `platform-parity-spec.md`, `slices/PDLDPM2/PDLDPM2-spec.md`
