# best-effort-distro-package-manager — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Spec manifest:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/best-effort-distro-package-manager"` (strict packs only).

### Create
- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
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
    - Operators can deterministically force the Linux prereq install path to use a specific manager.
    - Installer output includes a support-friendly one-liner that makes detection inputs, selected manager, and selection source explicit.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` `--help` output documents `--pkg-manager` and `PKG_MANAGER`, including allowed values and precedence.
    - `docs/INSTALLATION.md` adds `--pkg-manager` to the Installer Options Reference and labels it Linux-only.
    - The one-liner is emitted exactly once; implementation gates emission to avoid duplicates when prereq checks loop.
  - Contradiction risks:
    - `scripts/substrate/world-enable.sh` sources `install-substrate.sh` and reuses prereq helpers; emitting the one-liner from shared helpers changes world-enable UX unless scoped.
    - Output exactness: the contract requires an exact one-liner string without prefixes, so implementation cannot reuse helpers that prefix output.

- Change: Pkg-manager decision failures use taxonomy exit codes (`2/3/4`) and include remediation guidance.
  - Direct impact:
    - Invalid override values fail with exit `2`.
    - Forced manager missing from `PATH` fails with exit `3` and does not fall back.
    - Inability to select any supported manager fails with exit `4` and prints a manual prerequisite command list.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` currently exits `1` for most failures via `fatal`; implementing `2/3/4` requires scoped error handling so only pkg-manager decision failures map to `2/3/4`.
    - `tests/installers/pkg_manager_detection_test.sh` asserts exit codes and required remediation content elements for failure cases.
  - Contradiction risks:
    - Windows code paths currently use exit `2` for PowerShell flow gaps; contract scopes `2/3/4` meanings to Linux installer flows to avoid Windows behavior reclassification.

### Config / env vars / paths
- Change: Add deterministic env-var seam for hermetic tests: `SUBSTRATE_INSTALL_OS_RELEASE_PATH=<path>`.
  - Direct impact:
    - Tests can supply a fake os-release file without containers and without mutating the host OS.
  - Cascading impact:
    - The contract pins safety invariants: read as plain text; no `source`/eval/execute.
    - `tests/installers/pkg_manager_detection_test.sh` uses the seam to validate mapping and `<unknown>` rendering deterministically.
  - Contradiction risks:
    - Installer helper discovery flows that `source` the script must not become dependent on the seam being set.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0030-summoning-honeybadger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
  - Conflict: no
  - Resolution (explicit):
    - Keep installer UX changes scoped to the pkg-manager decision path; do not broaden unrelated installer failure taxonomy.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` sourcing `install-substrate.sh` and calling prereq helpers
  - Conflict: no
  - Resolution (explicit):
    - Gate the decision one-liner so it does not introduce side effects when installer helpers are reused by world-enable workflows.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - shared output fields and semantics for os-release + pkg-manager selection
  - Conflict: yes
  - Resolution (explicit):
    - Authority boundary:
      - `best-effort-distro-package-manager` owns detection/selection/parsing/`source` semantics and MUST NOT persist host metadata.
      - `persist-detected-linux-distro-pkg-manager` persists outputs and MUST treat this pack’s `contract.md` as authoritative.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Keep provisioning workflow changes separate from host installer pkg-manager detection semantics.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` parsing vocabulary and manager identifiers
  - Conflict: yes
  - Resolution (explicit):
    - Align parsing/canonicalization rules while keeping host installer selection distinct from in-world provisioning selection.

## Follow-ups (explicit)

- Spec updates required:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` — define acceptance criteria for os-release parsing + one-liner exactness and timing.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md` — define acceptance criteria for selection algorithm, warnings, and failure posture.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md` — define the hermetic harness contract and assertions.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` — include the exact command(s) to run the hermetic detection test and label container smoke as optional (not CI gating).
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` — populate slice triad tasks and reference the slice specs + contract.
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` — reconcile Related Docs link drift so planning artifacts are discoverable.
