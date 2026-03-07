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
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"` (strict packs only).

### Create
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `tests/installers/install_state_smoke.sh`
- `docs/INSTALLATION.md`

### Deprecate
- None

### Delete
- None

## Implementation Surface Note

Implementation surface note:
- ADR-0032 stays inside installer scripts, installer smoke coverage, and operator/planning docs.
- No `crates/`, `src/`, `crates/world*`, `crates/shim`, `crates/shell`, or `crates/world-agent` path enters this touch set.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Successful Linux installs create or update `install_state.json` even when no group or linger event exists.
  - Direct impact:
    - Operators get one stable post-install metadata file after successful Linux installs instead of an event-only file.
    - Hosted install and dev install use one visible contract for install-state persistence.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` need one write-trigger rule across default install, `--no-world`, and dry-run.
    - `tests/installers/install_state_smoke.sh` needs explicit no-event assertions in addition to the current group/linger assertions.
    - `docs/INSTALLATION.md` needs one statement for install-state creation rules across both installers.
  - Contradiction risks:
    - Hosted-only scope leaves dev-install on an older metadata contract while docs still present both installers as metadata producers.
    - Event-only writes keep the file absent after a successful Linux install with no group or linger changes.
  - Conflict resolution:
    - Option A: hosted installer only.
    - Option B: hosted installer and dev installer share one metadata contract.
    - Selected: Option B.

### Config / env vars / paths
- Change: `install_state.json` remains `schema_version = 1` and gains additive `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` fields.
  - Direct impact:
    - Future consumers can read persisted distro and selected-manager metadata without losing existing group/linger cleanup data.
    - Path semantics stay anchored to the effective install prefix; operator docs can refer to the same location as `$SUBSTRATE_HOME/install_state.json`.
  - Cascading impact:
    - `install-state-schema-spec.md` must lock JSON nesting, types, absence semantics, and merge rules with existing `host_state.group` and `host_state.linger`.
    - `docs/INSTALLATION.md` needs field-name reconciliation from `Schema version = 1` to `schema_version = 1`.
    - `tests/installers/install_state_smoke.sh` needs additive-compatibility assertions rather than a schema bump assertion.
  - Contradiction risks:
    - ADR text uses `$SUBSTRATE_HOME`, installer code uses the effective `--prefix`, docs use `<prefix>`, hosted uninstaller reads `HOME/.substrate`, and dev uninstaller reads `<prefix>`.
  - Conflict resolution:
    - Option A: keep `scripts/substrate/uninstall-substrate.sh` outside this touch set and record the prefix mismatch as a follow-up because ADR-0032 changes producer behavior, not cleanup-reader behavior.
    - Option B: expand this feature to align the hosted uninstaller read path now.
    - Selected: Option A.

### Policy / isolation / security posture
- Change: Metadata persistence stays Linux-only, best-effort, and producer-side; it introduces no new CLI flags, env vars, policy rules, trace fields, or backend contracts.
  - Direct impact:
    - Missing `/etc/os-release` or metadata write failure does not turn an otherwise successful Linux install into failure.
    - macOS and Windows do not gain new `host_state.platform.*` writes.
  - Cascading impact:
    - `pkg_manager.selected` and `pkg_manager.source` must come from the detection contract, not local re-derivation.
    - `tests/installers/install_state_smoke.sh` needs a missing-`/etc/os-release` branch and a no-event branch.
  - Contradiction risks:
    - Local duplication of manager or `source` vocabulary breaks parity with installer output and future persisted metadata consumers.
  - Conflict resolution:
    - Option A: duplicate detection vocabulary inside this pack.
    - Option B: use `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` as the single authority and persist emitted strings verbatim.
    - Selected: Option B.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - `pkg_manager.selected` and `pkg_manager.source` vocabulary
  - Conflict: yes
  - Resolution (explicit):
    - Option A: duplicate detection semantics and vocabulary inside this pack.
    - Option B: detection semantics stay authoritative in `best-effort-distro-package-manager`, and this pack persists the emitted strings only.
    - Selected: Option B.
    - Sequencing boundary: land ADR-0031 first or concurrently; ADR-0032 does not redefine selection semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Non-overlap boundary:
      - ADR-0030 owns world-deps provisioning behavior and operator guidance for `world enable --provision-deps`.
      - ADR-0032 owns install-state producers and install-state operator docs.
    - Shared-file rule: ADR-0030 must not refactor install-state write helpers or schema wording owned by this pack.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `docs/INSTALLATION.md`
    - manager identifier vocabulary (`apt-get`, `pacman`, `zypper`, `dnf`, `yum`)
    - `/etc/os-release` identity terms
  - Conflict: no
  - Resolution (explicit):
    - Keep host installer metadata distinct from in-world provisioning metadata.
    - Use the same manager spellings while keeping “host pkg-manager” and “world OS pkg-manager” as separate operator concepts.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Sequence helper-staging refactors before, or alongside, dev-install metadata persistence.
    - Keep edits orthogonal inside `scripts/substrate/dev-install-substrate.sh` so helper discovery and metadata writes rebase cleanly.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Keep world-agent staging and missing-artifact remediation separate from install-state write semantics.
    - Dev-install metadata writes must not change the world-enable artifact contract owned by ADR-0035.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `$SUBSTRATE_HOME` naming
    - installer-owned files under the effective Substrate home
    - operator wording for `--home` and `--prefix` derived paths
  - Conflict: no
  - Resolution (explicit):
    - Keep path semantics aligned to the effective Substrate home and add no new env vars.
    - `contract.md` must define one equivalence rule between the effective install prefix and the operator-facing `$SUBSTRATE_HOME`.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - manager-selection and `source` vocabulary
  - Conflict: yes
  - Resolution (explicit):
    - Authority boundary:
      - `best-effort-distro-package-manager` owns detection, selected-manager vocabulary, and `pkg_manager.source`.
      - `persist-detected-linux-distro-pkg-manager` owns persistence into `install_state.json` and the reliable-write contract.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Keep install-state persistence isolated to this pack.
    - Keep provisioning-flow edits isolated to the world-deps pack.
    - Shared-file edits need sequencing discipline because the semantic boundaries are separate but the files are the same.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - shared package-manager vocabulary
  - Conflict: no
  - Resolution (explicit):
    - Align vocabulary only.
    - Keep host install-state persistence and in-world provisioning as separate surfaces.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - This pack’s dev-install metadata changes must remain orthogonal to helper staging/discovery changes.
    - Shared-file edits need sequencing because both packs touch the same script for different reasons.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Keep install-state write semantics separate from world-agent build/stage semantics.
    - Shared-file edits need sequencing because both packs touch the same dev-install script.

### Relevant Archived Packs
- Archived Pack: `docs/project_management/_archived/p0-platform-stability/`
  - Overlap surfaces:
    - `install_state.json`
    - installer metadata and cleanup expectations
  - Conflict: no
  - Resolution (explicit):
    - Extend the existing `install_state.json` surface.
    - Do not introduce a second metadata file for platform detection.

## Follow-ups (explicit)

- Decision Register entries required:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` — DR-0001: persistence location contract (`install_state.json` vs separate file)
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` — DR-0002: field naming and nesting under `host_state.platform.*`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` — DR-0003: vocabulary ownership (`best-effort-distro-package-manager` contract vs local duplication)
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` — DR-0004: write-trigger scope across hosted install, hosted `--no-world`, dev install, and dev `--no-world`

- Spec updates required:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` — pin installer-scope selection, Linux-only guarantees, prefix-to-`$SUBSTRATE_HOME` equivalence, and no-world/dry-run/write-failure rules
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` — pin `schema_version` field name, `host_state.platform.*` schema, merge rules with `host_state.group` and `host_state.linger`, and JSON examples
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md` — pin exact write/no-write branches and the temp-file replacement rule
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md` — pin smoke assertions for no-event writes, missing `/etc/os-release`, and additive compatibility
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` — fix feature-directory drift and related-doc path drift
  - `docs/INSTALLATION.md` — reconcile installer-scope language, `schema_version` field name, and effective metadata path wording
  - `scripts/substrate/uninstall-substrate.sh` — review HOME-vs-prefix path handling as a separate follow-up outside the selected touch set
