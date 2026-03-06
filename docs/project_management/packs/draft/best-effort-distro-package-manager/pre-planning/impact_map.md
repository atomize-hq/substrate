# best-effort-distro-package-manager — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Spec manifest:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/best-effort-distro-package-manager"` (strict packs only).

### Create
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- `tests/installers/pkg_manager_detection_test.sh`

### Edit
- `docs/INSTALLATION.md`
- `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`
- `docs/project_management/packs/sequencing.json`
- `scripts/substrate/install-substrate.sh`

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
- Change: Linux installer supports explicit override (`--pkg-manager`) and emits an operator-facing distro/pkg-manager decision one-liner before prerequisite installation begins.
  - Direct impact:
    - Operators can deterministically force the Linux prereq install path to use a specific manager, reducing brittle PATH-based selection.
    - Installer output includes a support-friendly one-liner that makes the detection inputs, selected manager, and selection source explicit.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` `--help` output must document `--pkg-manager` and `PKG_MANAGER`, including allowed values and precedence.
    - `docs/INSTALLATION.md` must add `--pkg-manager` to the Installer Options Reference and explicitly label it Linux-only.
    - The one-liner is required to be emitted exactly once; `ensure_linux_prereqs` currently performs multiple prereq checks, so implementation must gate emission to avoid duplicate one-liners.
  - Contradiction risks:
    - `scripts/substrate/world-enable.sh` sources `install-substrate.sh` and reuses prereq helpers; if the one-liner is emitted from shared helpers, `world-enable` UX changes unintentionally unless explicitly scoped.
    - Output exactness: the ADR requires an exact one-liner string, so the implementation cannot reuse `log()` if it prefixes or formats the line.

- Change: Pkg-manager decision failures use taxonomy exit codes (`2/3/4`) and include actionable remediation guidance.
  - Direct impact:
    - Invalid override values fail with exit `2`; forced manager missing from PATH fails with exit `3`; inability to select any supported manager fails with exit `4`.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` currently exits `1` for most failures via `fatal`; implementing `2/3/4` requires refactoring error handling so only pkg-manager-related failures map to `2/3/4` (without reclassifying unrelated install failures).
    - `tests/installers/pkg_manager_detection_test.sh` must assert exit codes and required remediation content elements for the failure cases.
  - Contradiction risks:
    - `scripts/substrate/install-substrate.sh` currently exits `2` on the Windows code path (“PowerShell flow not yet implemented”). To preserve “no behavior change on Windows”, `contract.md` must scope the `2/3/4` meanings to the Linux pkg-manager decision path (or explicitly accept a separate Windows change).

### Config / env vars / paths
- Change: Add env override `PKG_MANAGER` as an explicit contract input with deterministic precedence under `--pkg-manager`.
  - Direct impact:
    - Operators can use `PKG_MANAGER` as an override while `--pkg-manager` remains the highest-precedence override.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` must not clobber the environment value at startup (it currently initializes `PKG_MANAGER=""`) and must validate allowed values before proceeding.
    - Precedence + validation become stable contract surfaces that must match across one-liner `source` values, warnings/errors, and tests.
  - Contradiction risks:
    - `PKG_MANAGER` is a generic env var name; hosts may already have it set for unrelated tooling. After this change, an accidental value can cause a new hard failure (exit `2`) unless the contract explicitly constrains when it is read and strongly encourages the flag for explicit override.

- Change: Default selection uses best-effort `/etc/os-release` parsing + distro-family mapping, with deterministic fallback to PATH probing.
  - Direct impact:
    - Default manager selection matches common operator expectations when `/etc/os-release` is available.
  - Cascading impact:
    - Specs must pin safe parsing posture (no `source`/eval), `<unknown>` rendering, and mapping rules exactly as in ADR-0031.
    - Specs must pin “mapping matched but binary missing” behavior (fallback vs fail) and the resulting `source` value and exit code.
  - Contradiction risks:
    - In-world provisioning packs also probe `/etc/os-release` in different contexts; if canonicalization and “unknown” handling drift, operator guidance can become contradictory across host installer vs in-world probes.

### Policy / isolation / security posture
- Change: Detection remains local, non-executing, and best-effort while overrides are validated and fail-closed.
  - Direct impact:
    - Distro detection reads only `/etc/os-release` and performs no writes or network calls for detection.
    - Explicit overrides validate and fail-closed (no silent fallback).
  - Cascading impact:
    - `contract.md` must explicitly separate “detection” (local reads only) from the rest of the installer, which still performs network downloads for release artifacts.
    - Hermetic tests must ensure the os-release parser never executes file content and that override validation never falls back silently.
  - Contradiction risks:
    - A hermetic-test hook that allows arbitrary os-release paths via environment variables can weaken the production safety posture unless tightly constrained and test-only (Decision Register DR-0003).

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - shared detected fields: os-release `ID`/`ID_LIKE`, selected manager, and `source`
  - Conflict: yes
  - Resolution (explicit):
    - Authority boundary: this pack owns detection/selection semantics and the installer one-liner; ADR-0032 owns persistence into `install_state.json` and must treat this pack’s `contract.md` as authoritative for parsing/selection/`source` vocabulary.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared file; provisioning pack touch surface)
  - Conflict: yes
  - Resolution (explicit):
    - Keep pkg-manager detection/selection logic isolated and stable within `install-substrate.sh`; provisioning-related edits must not refactor the detection pipeline owned by ADR-0031.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing and distro-family vocabulary (in-world OS probe)
    - manager identifier vocabulary (`apt-get`, `dnf`, `yum`, `pacman`, `zypper`)
  - Conflict: no
  - Resolution (explicit):
    - Align canonicalization + “unknown” rendering rules across host installer and in-world probes while keeping “host pkg-manager” distinct from “world OS pkg-manager” in operator text.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` as a staged helper dependency (sourced by `scripts/substrate/world-enable.sh`)
  - Conflict: no
  - Resolution (explicit):
    - Preserve the “safe to source” posture (`main` only runs when executed) and avoid interface changes that would break helper staging/discovery flows.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` sourcing `install-substrate.sh` and calling prereq helpers
  - Conflict: no
  - Resolution (explicit):
    - Ensure the new one-liner + pkg-manager decision behavior is gated so it does not introduce unexpected side effects when installer helpers are reused by world-enable workflows.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - shared output fields and semantics for os-release + pkg-manager selection
  - Conflict: yes
  - Resolution (explicit):
    - Authority boundary:
      - `best-effort-distro-package-manager` owns detection/selection/parsing/`source` semantics and MUST NOT persist host metadata.
      - `persist-detected-linux-distro-pkg-manager` persists those outputs (Linux-only) and MUST NOT redefine how they are derived.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared file)
  - Conflict: yes
  - Resolution (explicit):
    - Keep ADR-0030 edits to `install-substrate.sh` limited to provisioning workflow integration/messaging and avoid modifying the pkg-manager detection pipeline owned by ADR-0031.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared file via installer/world-enable coupling)
    - `/etc/os-release` parsing vocabulary and manager identifiers
  - Conflict: yes
  - Resolution (explicit):
    - Keep host installer pkg-manager selection distinct from in-world provisioning manager selection while aligning parsing/canonicalization rules to avoid contradictory remediation.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - helper staging/discovery flows that stage `install-substrate.sh` under `$SUBSTRATE_HOME/scripts/substrate/…`
  - Conflict: no
  - Resolution (explicit):
    - Ensure helper staging continues to stage the current `install-substrate.sh` and that sourcing expectations remain valid after this pack’s changes.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` sourcing `install-substrate.sh` for provisioning
  - Conflict: no
  - Resolution (explicit):
    - Keep detection changes constrained so downstream “enable later” improvements can rebase cleanly without changing pkg-manager detection semantics.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Pin `/etc/os-release` parsing + matching rules (normalization; duplicate keys; case-sensitivity; `ID_LIKE` tokenization).
  - DR-0002 — Pin deterministic PATH-probe precedence order and multi-manager ambiguity policy (warn vs fail; required warning content elements).
  - DR-0003 — Pin hermetic-test os-release injection seam (fake input without weakening production safety posture).
- Spec updates required:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — pin mapping failure semantics (“mapped manager missing” posture), PATH-probe precedence order, and one-liner emission timing (including the “no prereqs needed” case and reuse via `scripts/substrate/world-enable.sh`).
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — pin remediation guidance content elements and the exact prerequisite command list included in the “no supported manager” guidance.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` — define the hermetic harness contract and assert precedence, one-liner content, warning/error elements, and exit codes.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` — include the exact command(s) to run the hermetic detection test and explicitly label the container smoke as optional (if it is not CI gating).
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` — populate the `BEDPM0` triad tasks and reference the slice spec + contract explicitly.
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` — reconcile Related Docs link drift (paths under `docs/project_management/packs/draft/detecting-badger/` vs `docs/project_management/packs/draft/best-effort-distro-package-manager/`) so downstream planning artifacts are discoverable.
