**PRE‑PLANNING ONLY: This document is a cross-cutting alignment backbone draft. It MUST be deleted or explicitly retired during full planning.**

# persist-detected-linux-distro-pkg-manager — minimal spec draft

## Scope + authority

Authority inputs for this draft (only):
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`

This draft is allowed to define:
- Cross-cutting invariants and non-overlap boundaries that every slice spec MUST preserve.
- Cross-cutting precedence rules explicitly required by the ADR (notably: persisted metadata → runtime fallback).
- Cross-cutting safety posture (platform scope, protected paths, privacy allowlist/denylist posture).

This draft MUST NOT define:
- Slice-specific behavior details (atomicity mechanics, exact overwrite/merge rules, warnings text, task breakdown).
- The full `install_state.json` schema or examples (owned by `install-state-schema-spec.md`).
- Any new scope beyond ADR-0032 + the pack’s `spec_manifest.md` + `impact_map.md`.

## Defaults + precedence

### CLI/config/env precedence
- This feature introduces no new CLI commands, flags, config keys, or environment variables.
- `$SUBSTRATE_HOME` default is assumed to be `~/.substrate` (ADR-0032).
- `$SUBSTRATE_HOME` override mechanism(s) and their precedence for installer runs are owned by `contract.md` (see Follow-ups).

### Source-of-truth paths
- Persisted metadata file: `$SUBSTRATE_HOME/install_state.json`.
- Best-effort OS identity input: `/etc/os-release` (`ID`, `ID_LIKE`).

### Consumer read precedence (authoritative)
- Future consumers MUST prefer persisted metadata for guidance strings when it is present and readable.
- Future consumers MUST fall back to runtime detection when persisted metadata is missing or unreadable.

## Failure posture + invariants

### Failure posture
- Degrade: if platform metadata cannot be read or written, the installer MUST NOT hard-fail solely due to metadata persistence (ADR-0032).
- `/etc/os-release` missing/unreadable MUST NOT cause install failure; persistence behavior for missing inputs is pinned by the schema spec + slice spec.

### Platform invariants
- Linux-only behavior delta: Linux installer persists `host_state.platform.*` to `install_state.json`.
- macOS/Windows: no new writes of `host_state.platform.*` are introduced by this feature.

### Protected paths + privacy invariants
- Writes MUST be constrained to `$SUBSTRATE_HOME` (no writes outside that directory).
- Persisted allowlist is limited to:
  - `/etc/os-release` `ID` and `ID_LIKE` (or their absence representation), and
  - selected package manager identifier, plus its selection `source`.
- Persisted metadata MUST NOT include sensitive host details (e.g., hostnames, environment dumps).

### Parsing invariant
- `/etc/os-release` parsing MUST NOT execute the file (no `source /etc/os-release`).

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature introduces no new exit codes and no exit-code overrides (ADR-0032).

## Cross-cutting seams / constraints

### Persisted key set (stability boundary)
- `install_state.json` remains `schema_version=1`.
- The new persisted key set is exactly:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- The change is additive; readers MUST ignore unknown keys (backwards compatibility posture).

### Detection vs persistence (non-overlap boundary)
- Upstream detection/selection semantics (including `/etc/os-release` parsing rules and `pkg_manager.source` vocabulary) are owned by:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- This feature’s scope is persistence of upstream-derived outputs to `install_state.json` and MUST NOT redefine:
  - how distro/pkg-manager values are derived, or
  - the authoritative `pkg_manager.source` enum vocabulary.

### Touch set alignment
Slice specs and planning artifacts MUST remain consistent with the Impact Map touch set:
- `scripts/substrate/install-substrate.sh`
- `tests/installers/install_state_smoke.sh`
- `docs/INSTALLATION.md`

## Follow-ups for full planning

Each follow-up is required to remove ambiguity before planning can pass a quality gate.

1) Decision Register entries (from `impact_map.md`)
   - DR-0001 — Reconcile “`install_state.json` MUST exist after successful Linux install” with best-effort write posture (writer dependencies, fallback strategy, and whether operator-visible warning output is required).
   - DR-0002 — Pin `host_state.platform.pkg_manager.source` vocabulary for persistence by explicitly deferring to the upstream detection contract (or selecting a locally-owned enum set if deferral is rejected).
   - DR-0003 — Scope decision: whether `scripts/substrate/dev-install-substrate.sh` is in-scope for persisting `host_state.platform.*` (A: prod installer only; B: prod + dev installers).
   - DR-0004 — Overwrite policy on re-run: preserve existing `host_state.platform.*` vs overwrite with newly detected values (including behavior when inputs are missing on subsequent runs).

2) `$SUBSTRATE_HOME` resolution
   - Pin the exact override mechanisms (flag/env, if any) and their precedence for installer runs, and define behavior when the resolved prefix is unwritable.

3) “Successful install” definition for the file-exists guarantee
   - Define whether `--dry-run` is excluded from the “successful install” guarantee.
   - Define the required behavior for `--no-world` installs (impact map requires file presence in this success path).

4) Absence semantics (schema + persistence)
   - Pin the representation for missing/unreadable `/etc/os-release` inputs (omit vs null vs sentinel string) for:
     - `host_state.platform.os_release.id`
     - `host_state.platform.os_release.id_like`
   - Pin behavior when package-manager detection is missing/unavailable (including what `pkg_manager.selected` contains, and what `pkg_manager.source` reports).

5) Documentation alignment
   - Update requirements for `docs/INSTALLATION.md` to include the new persisted keys and the privacy posture (allowlist + sensitive-data denylist statement).

6) Naming/path drift cleanup
   - Reconcile ADR-0032 “feature directory” references (`stashing-ferret`) with this pack directory (`persist-detected-linux-distro-pkg-manager`).
   - Reconcile ADR-0032 dependency naming (`detecting_badger`) with the upstream pack path (`best-effort-distro-package-manager`).

## Draft slice skeleton (pre-planning only)

draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): PDL

- slice_id: PDL0
  name: Persist Linux platform metadata to `install_state.json`
  intent: Persist upstream-derived distro/pkg-manager detection outputs to `$SUBSTRATE_HOME/install_state.json` (`schema_version=1`) on Linux and extend installer smoke assertions to cover file presence + new keys.
  likely touch surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDL0/PDL0-spec.md`

Downstream notes:
- CI-checkpoint: prefer this slice list when populating the machine-readable slices list in `ci_checkpoint_plan.md` (no mechanical validation until slice tasks exist in `tasks.json`).
- Workstream triage is allowed to propose edits to this slice skeleton as recommendations in `workstream_triage.md` (must not edit this file).
