# persist-detected-linux-distro-pkg-manager — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Spec manifest:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/"` (strict packs only).

### Create
- None

### Edit
- `scripts/substrate/install-substrate.sh`
- `tests/installers/install_state_smoke.sh`
- `docs/INSTALLATION.md`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: After a successful Linux install, persist platform detection into `$SUBSTRATE_HOME/install_state.json` (`schema_version=1`) and ensure the file is present even when `--no-world` is used.
  - Direct impact:
    - Operators (and later tooling) can read a stable record of what the installer detected/selected (`/etc/os-release` + package-manager selection) without re-running detection.
    - Removes the “successful install but no install_state.json exists” diagnostic ambiguity (notably `--no-world` installs).
  - Cascading impact:
    - Installer must define “successful install” precisely for this guarantee (exclude `--dry-run`; clarify behavior if prefix is unwritable).
    - Installer smoke coverage must add assertions for the new keys and for the `--no-world` success case (file present; platform keys present when inputs are available).
    - `docs/INSTALLATION.md` must be updated to list the new keys and their privacy posture.
  - Contradiction risks:
    - “Must exist after successful install” vs “best-effort persistence; do not hard-fail solely due to metadata write” is currently non-deterministic because the existing writer requires `python3` and the current write path is skipped for `--no-world`; a Decision Register entry is required to make this contract testable and non-contradictory.
    - Upstream detection work (ADR-0031) defines `pkg_manager.source` values and `/etc/os-release` parsing rules; if this feature re-parses inputs independently, persisted values can drift from the installer’s printed selection/reporting.

### Config / env vars / paths
- Change: Extend `install_state.json` with additive, Linux-only `host_state.platform.*` fields (schema stays `schema_version=1`).
  - Direct impact:
    - New optional keys become available for future guidance surfaces:
      - `host_state.platform.os_release.id`
      - `host_state.platform.os_release.id_like`
      - `host_state.platform.pkg_manager.selected`
      - `host_state.platform.pkg_manager.source`
  - Cascading impact:
    - Absence semantics must be explicit and consistent across installer output + persisted values:
      - `/etc/os-release` missing/unreadable must not fail install; persisted `os_release.*` representation must be pinned (omit vs `<unknown>` vs null).
      - `pkg_manager.source` allowed vocabulary must be pinned (ideally by deferring to the upstream detection contract).
    - Writes should remain atomic and idempotent (write-then-rename; stable permissions; no writes outside `$SUBSTRATE_HOME`).
    - Existing uninstall flows must remain compatible: readers must ignore unknown keys and must continue to accept `schema_version=1`.
  - Contradiction risks:
    - Multiple `/etc/os-release` parsing implementations (host installer detection vs persisted metadata vs in-world probes) can disagree on normalization/canonicalization and “unknown” rendering, leading to confusing support guidance and inconsistent diagnostics.

### Policy / isolation / security posture
- Change: Persist additional host facts to disk (`install_state.json`) while keeping strict privacy boundaries.
  - Direct impact:
    - Installs store more host metadata on Linux, which can be inspected by operators and support.
  - Cascading impact:
    - The schema/contract must explicitly denylist sensitive host details and ensure logs do not echo unexpected `/etc/os-release` contents.
    - The feature must remain Linux-only and must not write these platform keys on macOS/Windows.
    - Parsing posture must remain safe (no `source /etc/os-release`).
  - Contradiction risks:
    - Using unsafe parsing approaches (shell execution of `/etc/os-release`) would be a security regression; the parsing posture must be explicitly aligned with ADR-0031’s “no source / no shell execution” invariant.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` parsing posture and `pkg_manager.source` vocabulary
  - Conflict: yes
  - Resolution (explicit):
    - Sequencing boundary: ADR-0031 (detection + selection + reporting) must land first and remain authoritative for how `distro_id`/`id_like` and `pkg_manager.*` values are derived.
    - This feature MUST persist the upstream-derived outputs and MUST NOT redefine the detection/selection pipeline or `source` vocabulary.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared helper script; sourced by `scripts/substrate/world-enable.sh`)
    - `docs/INSTALLATION.md` (installer/provisioning documentation touchpoint)
  - Conflict: no
  - Resolution (explicit):
    - Keep host-metadata persistence changes isolated to the `install_state.json` writer path; avoid refactors that would complicate provisioning-time changes landing in the same script.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh` (helper staging), potential overlap if this feature is expanded to dev-install metadata persistence
  - Conflict: no (unless dev-install is brought into scope)
  - Resolution (explicit):
    - Default scope stays `scripts/substrate/install-substrate.sh` only; if dev-install is later included, sequence after ADR-0034 to reduce merge conflicts in `dev-install-substrate.sh`.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh` (artifact staging), potential overlap if this feature is expanded to dev-install metadata persistence
  - Conflict: no (unless dev-install is brought into scope)
  - Resolution (explicit):
    - Same as ADR-0034: keep dev-install out of scope for this feature unless explicitly decided; otherwise coordinate sequencing to avoid compounding edits to the dev installer.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - Installers as writers of `$SUBSTRATE_HOME/env.sh` and related global config invariants
  - Conflict: no
  - Resolution (explicit):
    - This feature introduces no new env vars and does not change env-script generation semantics; limit changes to `install_state.json` persistence only.

- ADR: `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing vocabulary (in-world OS identity) and `<unknown>` rendering expectations
  - Conflict: no
  - Resolution (explicit):
    - Keep host vs in-world OS identity distinct but align canonicalization/unknown rendering rules to avoid contradictory “OS identity” strings across diagnostics and persisted metadata.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - shared output fields and semantics: `distro_id`, `id_like`, `pkg_manager.selected`, `pkg_manager.source`
  - Conflict: yes
  - Resolution (explicit):
    - Enforce strict non-overlap boundary:
      - `best-effort-distro-package-manager` owns detection/selection/parsing semantics and MUST NOT persist host metadata.
      - `persist-detected-linux-distro-pkg-manager` persists those outputs to `install_state.json` and MUST NOT redefine how they are derived.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: no
  - Resolution (explicit):
    - Coordinate sequencing and keep edits narrow to reduce merge conflict risk; avoid touching the pkg-manager detection pipeline in this feature (owned by ADR-0031 pack).

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `/etc/os-release` parsing and canonicalization vocabulary
    - manager identifier vocabulary (`apt-get`, `pacman`, etc.)
  - Conflict: no
  - Resolution (explicit):
    - Align canonicalization rules and identifier vocabulary so persisted host metadata, installer output, and in-world probes do not diverge.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Reconcile “install_state.json must exist after successful Linux install” with best-effort write posture (writer dependencies, fallback strategy, required warning output).
  - DR-0002 — Define `pkg_manager.source` authoritative vocabulary for persistence (explicitly defer to ADR-0031 contract vs define locally).
  - DR-0003 — Scope: whether `scripts/substrate/dev-install-substrate.sh` is in-scope for persisting the new `host_state.platform.*` keys (A: prod installer only; B: prod + dev installers).
  - DR-0004 — Overwrite policy on re-run: preserve existing `host_state.platform.*` vs overwrite with newly detected values (and what happens when inputs are missing on subsequent runs).
- Spec updates required (if any):
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md` — update “Installer entrypoint in scope” row if dev-install is selected as in-scope; add explicit dependency link to the upstream detection pack path if required.
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` — reconcile feature-dir link drift (`stashing-ferret` vs `persist-detected-linux-distro-pkg-manager`) and dependency naming drift (`detecting_badger` vs the actual upstream pack directory).

