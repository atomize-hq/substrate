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
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"` (strict packs only).

### Create
- None

### Edit
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `tests/installers/install_state_smoke.sh`

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
- Change: Successful Linux installs persist host platform detection + pkg-manager selection into `$SUBSTRATE_HOME/install_state.json`.
  - Direct impact:
    - After successful Linux installs, `install_state.json` becomes a reliable “what did we detect?” record (file is no longer absent on the success path due solely to “no host-state events”).
    - New stable keys exist under `host_state.platform.*` for later consumer UX (doctor/retry/support) without requiring immediate CLI surface changes.
  - Cascading impact:
    - Update installer smoke coverage to assert the new keys and their absence semantics (Linux-only; `/etc/os-release` missing/unreadable must not fail install; keys may be absent).
    - Ensure persisted `pkg_manager.selected` + `pkg_manager.source` are derived from the installer’s authoritative detection pipeline (do not duplicate detection logic in the persistence step).
  - Contradiction risks:
    - `--no-world` semantics drift:
      - A) Persist platform metadata even when `--no-world` is set.
      - B) Skip persistence when `--no-world` is set.
      - Selected: A (locked in intake; consistent with ADR-0032 “successful Linux install” language; current scripts must remove the `world_enabled` gate for metadata writes).
    - “File must exist” vs “best-effort; don’t fail install”:
      - A) Make `python3` a hard prerequisite so the file-write guarantee is enforceable.
      - B) Keep `python3` optional and implement a non-python write path so “file exists after successful install” does not depend on `python3` availability.
      - Selected: B (aligns with current installer prereqs: `jq` is required on Linux; adding `python3` as a hard prereq is a contract change and increases failure modes).

### Config / env vars / paths
- Change: Extend `install_state.json` (schema_version=1) with additive platform keys under `host_state.platform.*`.
  - Direct impact:
    - On Linux, `install_state.json` contains additional, non-sensitive host platform metadata (`/etc/os-release` `ID`/`ID_LIKE`) and the selected package manager identifier + selection source.
    - No new config keys, CLI flags, or env vars are introduced by this persistence slice.
  - Cascading impact:
    - Persistence must respect the existing `$SUBSTRATE_HOME` contract (default `~/.substrate`; empty treated as unset) without redefining it (`docs/reference/env/contract.md` owns this surface).
    - The update path must be idempotent and must not unintentionally discard unrelated JSON keys when updating `host_state.platform.*` (explicit merge/overwrite model required).
  - Contradiction risks:
    - Prefix/home divergence: installers write under `<prefix>/install_state.json`, but `scripts/substrate/uninstall-substrate.sh` reads `${HOME}/.substrate/install_state.json` (no `SUBSTRATE_HOME` support). Non-default install prefixes may therefore have “persisted metadata exists but uninstall cleanup can’t find it” behavior unless explicitly scoped/accepted.

### Policy / isolation / security posture
- Change: Read `/etc/os-release` (best-effort) and persist only a minimal allowlist of host platform keys.
  - Direct impact:
    - No new privilege escalation: `/etc/os-release` is read-only; the only write is under `$SUBSTRATE_HOME`.
    - Persisted values are non-sensitive by design (no hostnames; no env dumps; no machine identifiers).
  - Cascading impact:
    - `/etc/os-release` parsing MUST NOT execute arbitrary code (do not `source` the file); parsing/canonicalization must align with the upstream detection contract (`best-effort-distro-package-manager`).
    - Logging must avoid accidentally emitting full `/etc/os-release` contents; warnings should mention only high-level failure (“unreadable”) and path.
  - Contradiction risks:
    - Current container smoke (`tests/installers/pkg_manager_container_smoke.sh`) uses `source /etc/os-release` (test-only), which is incompatible with the safe-parsing posture; implementation must not copy this pattern.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` parsing rules + canonicalization
    - `pkg_manager.source` enum semantics (`flag|env|os_release|path_probe`)
  - Conflict: yes
  - Resolution (explicit):
    - Sequence: `detecting_badger` must land first (or at minimum its `contract.md` must be authoritative) so this feature can persist the already-decided `pkg_manager.selected` + `pkg_manager.source` outputs without re-implementing detection.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `--no-world` semantics on dev installs
  - Conflict: yes
  - Resolution (explicit):
    - Treat both ADRs as consistent: `--no-world` means “skip provisioning/systemd”, not “skip all metadata and staging”.
    - Ensure metadata persistence and world-agent staging changes do not fight over the `WORLD_ENABLED` gate (metadata persistence must not be gated on world being enabled).

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: no
  - Resolution (explicit):
    - Coordinate changes in a single edit to `dev-install-substrate.sh`; ensure helper-staging logic remains intact while extending metadata persistence.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - Installer responsibilities and `$SUBSTRATE_HOME` semantics
  - Conflict: no
  - Resolution (explicit):
    - This feature introduces no new env var surfaces and must rely on the existing `SUBSTRATE_HOME` contract; do not add new exports or precedence rules while persisting metadata.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` parsing + canonicalization rules
    - `pkg_manager.selected` identifier set + `pkg_manager.source` semantics
  - Conflict: yes
  - Resolution (explicit):
    - Enforce a non-overlap boundary:
      - `best-effort-distro-package-manager` defines detection + selection outputs and explicitly MUST NOT persist `install_state.json`.
      - `persist-detected-linux-distro-pkg-manager` persists those outputs (and must not redefine detection rules).

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001: metadata persistence location (`install_state.json` vs `host_platform.json`) — confirm A is locked and document any compat constraints.
  - DR-0002: field naming + nesting under `host_state.platform.*` — lock the exact JSON path set and optionality.
  - DR-0003: `host_state.platform.pkg_manager.source` enum set + semantics — lock the finite set and the mapping rules to detection inputs.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` — explicitly pin `--no-world` persistence semantics and the “success-path write” boundary.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` — define canonicalization rules (especially `os_release.id_like`) and merge/overwrite posture for pre-existing/corrupt JSON.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` — add a sequencing gate on `best-effort-distro-package-manager` being authoritative for detection outputs used by persistence.

