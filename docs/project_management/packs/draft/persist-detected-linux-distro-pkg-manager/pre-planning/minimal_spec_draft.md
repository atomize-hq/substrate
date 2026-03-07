**Pre-Planning Only:** This draft exists only to align pre-planning outputs for `persist-detected-linux-distro-pkg-manager`. Full planning deletes or retires this file.

## Scope + authority
- This draft defines only cross-cutting defaults, precedence, invariants, and the draft slice baseline for this pack.
- This draft does not define slice-specific behavior, detailed JSON schemas, implementation tasks, test procedures, or decision-register selections.
- Authoritative inputs for this draft:
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Cross-pack authority inherited without redefinition:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` owns Linux distro detection, selected package-manager vocabulary, and `pkg_manager.source` vocabulary.
- Persisted metadata surface in scope:
  - `install_state.json` under the effective Substrate home remains the only persisted metadata file touched by this pack.

## Defaults + precedence
- Metadata path rule:
  1. The installer's effective install prefix resolves the on-disk metadata path.
  2. The default prefix is `~/.substrate`.
  3. The operator-facing form of that same location is `$SUBSTRATE_HOME/install_state.json`.
  4. This pack adds no new feature-local config file or env-var override for that path.
- Linux package-manager selection precedence is inherited unchanged from `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`:
  1. `--pkg-manager`
  2. `PKG_MANAGER`
  3. `/etc/os-release` or `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
  4. Deterministic `PATH` probe
- Source-of-truth surfaces:
  - `pre-planning/spec_manifest.md` owns the required-doc set and canonical slice IDs `PDLDPM0`, `PDLDPM1`, and `PDLDPM2`.
  - `pre-planning/impact_map.md` owns the pack touch set and shared-file conflict boundaries.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` owns detection semantics, supported manager spellings, and `pkg_manager.source` values.
  - Full-planning `contract.md` owns the final installer-facing contract for this pack.
  - Full-planning `install-state-schema-spec.md` owns JSON field paths, types, absence semantics, and merge rules.

## Failure posture + invariants
- Metadata persistence is fail-open inside the success path: metadata read failure or metadata write failure does not convert an otherwise successful Linux install into failure.
- Explicit override validation and package-manager selection failure remain fail-closed under `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`. This draft does not change those exit paths.
- Dry-run does not write install metadata.
- The new `host_state.platform.*` persistence contract is Linux-only. macOS and Windows gain no new platform metadata writes from this pack.
- Writes stay under the effective Substrate home only.
- Persisted values are limited to:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- No secrets, hostnames, environment dumps, policy data, or new trace/log fields enter the persisted contract.

## Exit-code posture
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This pack does not require new exit codes.
- Full planning keeps installer success and failure meanings aligned to the existing taxonomy and the inherited detection-contract behavior.

## Cross-cutting seams / constraints
- Naming and path invariants:
  - Use `schema_version` as the field name. Do not reintroduce `Schema version`.
  - Use one equivalence rule between the effective `--prefix` path and `$SUBSTRATE_HOME/install_state.json`.
  - Keep `install_state.json` as the only persisted metadata file touched by this pack.
- Field-list invariants:
  - Preserve `schema_version = 1`.
  - Preserve existing `host_state.group` and `host_state.linger` data.
  - Add only `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.
- Ownership invariants:
  - Persist manager and source strings verbatim from `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
  - Do not duplicate supported-manager spellings, `pkg_manager.source` values, or os-release parsing rules in slice specs for this pack.
- Ordering invariants:
  - Detection selects values first.
  - Persistence stores emitted values on successful Linux install and update paths.
  - Smoke coverage locks the behavior after persistence semantics are pinned.
- Multi-surface alignment from `pre-planning/impact_map.md`:
  - `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` share one metadata-contract baseline.
  - `tests/installers/install_state_smoke.sh` and `docs/INSTALLATION.md` must align to the same field names, path wording, and write-trigger rules.
  - Hosted uninstaller HOME-vs-prefix path alignment stays outside this pack's selected touch set unless full planning expands scope deliberately.

## Follow-ups for full planning
- Fix ADR-0032 feature-directory drift and related-doc path drift from `docs/project_management/packs/draft/stashing-ferret/` to `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- Pin the exact write and no-write matrix for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run` in `decision_register.md` and `slices/PDLDPM1/PDLDPM1-spec.md`.
- Pin the exact temp-file and replace rule for `install_state.json` updates in `slices/PDLDPM1/PDLDPM1-spec.md`.
- Reconcile operator wording in `docs/INSTALLATION.md` for `schema_version`, effective metadata path, and shared hosted/dev installer scope.
- Decide whether hosted uninstaller HOME-vs-prefix path alignment stays follow-up-only or moves into a separate pack.
- Pin the missing-`/etc/os-release` smoke assertions and additive-compatibility assertions in `slices/PDLDPM2/PDLDPM2-spec.md`.

## Draft slice skeleton (pre-planning only)
draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `PDLDPM`

### `PDLDPM0`
- `slice_id`: `PDLDPM0`
- `name`: Persist Linux platform metadata
- `intent`: Stabilize the persisted `host_state.platform.*` field set and the rule that package-manager values come from the detection contract without local re-derivation.
- `likely touch surfaces`: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

### `PDLDPM1`
- `slice_id`: `PDLDPM1`
- `name`: Make install-state writes reliable
- `intent`: Stabilize the successful Linux write triggers, the dry-run no-write rule, and the idempotent update contract for `install_state.json`.
- `likely touch surfaces`: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`

### `PDLDPM2`
- `slice_id`: `PDLDPM2`
- `name`: Lock smoke coverage and operator wording
- `intent`: Stabilize validation evidence and operator documentation for persisted metadata without expanding installer behavior beyond this pack.
- `likely touch surfaces`: `tests/installers/install_state_smoke.sh`, `docs/INSTALLATION.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

Downstream note: CI-checkpoint uses this slice list first when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`. Mechanical validation stays off until slice tasks exist in `tasks.json`.

Downstream note: Workstream triage records any recommended edits in `pre-planning/workstream_triage.md`. That step does not edit this file.
