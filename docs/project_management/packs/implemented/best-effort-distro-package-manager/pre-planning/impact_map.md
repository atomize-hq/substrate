# best-effort-distro-package-manager — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/implemented/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Spec manifest:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/implemented/best-effort-distro-package-manager"` (strict packs only).

### Create
- `tests/installers/pkg_manager_detection_smoke.sh`

### Edit
- `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/plan.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/manual_testing_playbook.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/session_log.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/quality_gate_report.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/tasks.json`
- `docs/project_management/packs/sequencing.json`
- `scripts/ci-audit/ci_audit.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/install.sh`
- `docs/INSTALLATION.md`
- `docs/reference/env/contract.md`
- `tests/installers/pkg_manager_container_smoke.sh`

### Deprecate
- None

### Delete
- None

## Implementation surface note

Implementation stays inside installer shell scripts, installer docs, and planning artifacts.

No implementation change is required under:
- `crates/`
- `src/`
- `crates/world/`
- `crates/world-mac-lima/`
- `crates/world-windows-wsl/`
- `crates/shim/`
- `crates/shell/`
- `crates/world-agent/`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/world-enable.sh`
- `tests/installers/install_smoke.sh`
- `tests/installers/install_state_smoke.sh`
- `tests/installers/pkg_manager_container_smoke.sh`
- `tests/mac/installer_parity_fixture.sh`

Shared-file guard:
- `scripts/substrate/world-enable.sh` sources `scripts/substrate/install-substrate.sh`.
- ADR-0031 therefore must keep all new behavior behind hosted-installer argument parsing and prerequisite-install functions.
- No new source-time side effects, shared-helper renames, or world-enable-specific behavior enter this feature.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Linux hosted installs become explicit about distro/package-manager selection and gain a deterministic override surface.
  - Direct impact:
    - `scripts/substrate/install-substrate.sh` prints one stable stderr line before prerequisite installation: distro `ID`, `ID_LIKE`, selected package manager, and selection source.
    - Operators can force selection with `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`.
    - `PKG_MANAGER` remains supported as a lower-precedence legacy override.
  - Cascading impact:
    - `print_usage()` in `scripts/substrate/install-substrate.sh` must document `--pkg-manager`.
    - `docs/INSTALLATION.md` must document the new flag in the Linux install flow, the offline-wrapper flow, and the installer options table.
    - `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, and `tests/installers/pkg_manager_detection_smoke.sh` must all prove the same precedence chain: flag -> env -> os-release -> PATH probe.
  - Contradiction risks:
    - The current installer chooses the first supported manager in `PATH` with no visible source attribution.
    - The ADR requires a fixed PATH fallback order but does not name that order.
    - `scripts/substrate/install.sh` currently collapses upstream non-zero exits to `1`, which would hide the new `2` / `3` / `4` contract from the hosted and offline wrapper entrypoints.
  - Resolution options (A/B):
    - Option A: keep the new exit-code taxonomy scoped to direct `install-substrate.sh` runs and leave `install.sh` at exit `1`; choose a new PATH fallback order later.
    - Option B: preserve the current fallback manager order already encoded in `scripts/substrate/install-substrate.sh` (`apt-get -> dnf -> yum -> pacman -> zypper`) and update `scripts/substrate/install.sh` to pass through the upstream installer exit status.
    - Selected: Option B.

- Change: detection failures become contract-bearing instead of generic fatal errors.
  - Direct impact:
    - Invalid `--pkg-manager` or invalid `PKG_MANAGER` exits `2`.
    - Forced manager missing from `PATH` exits `3`.
    - No supported manager selected exits `4`.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` must stop routing these branches through the generic `fatal`/exit `1` path.
    - `docs/INSTALLATION.md` and `manual_testing_playbook.md` must align remediation text with the new failure classes.
    - `smoke/linux-smoke.sh` must verify that the wrapper path (`scripts/substrate/install.sh`) preserves the same failure class as the direct installer path.
  - Contradiction risks:
    - The current quick-install workflow enters through `scripts/substrate/install.sh`, not `install-substrate.sh`.
    - Without wrapper pass-through, the operator-facing one-liner would advertise a failure taxonomy it does not actually preserve.

### Config / env vars / paths
- Change: installer env-var semantics become part of the explicit contract instead of remaining partially implicit.
  - Direct impact:
    - `docs/reference/env/contract.md` must add `PKG_MANAGER` as a legacy installer override with explicit precedence and allowed values.
    - The hermetic test hook must be either fully absent from production or fully documented as one Linux-only installer env var.
  - Cascading impact:
    - `contract.md` must define the exact test-hook rule used by `BEDPM3`.
    - `spec_manifest.md` must pin the same rule so downstream packs stop treating the hook as undecided.
    - `tests/installers/pkg_manager_detection_smoke.sh` must use the selected mechanism and no alternative mechanism.
  - Contradiction risks:
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md` already assumes `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
    - Leaving the fake-os-release hook undecided would force downstream packs to either duplicate an unstable assumption or block on this pack.
  - Resolution options (A/B):
    - Option A: keep fake os-release input test-only and remove downstream references to any stable installer hook.
    - Option B: introduce one Linux-only installer env var, `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, document it in `docs/reference/env/contract.md`, and make it the only alternate os-release input contract.
    - Selected: Option B.

- Change: this feature remains hosted-installer-only even though the selected hook is installer-local.
  - Direct impact:
    - `scripts/substrate/dev-install-substrate.sh` does not adopt distro/package-manager detection or the new override surface under ADR-0031.
    - macOS and Windows operator behavior does not change.
  - Cascading impact:
    - `tasks.json` must keep the schema v4 cross-platform automation model while separating Linux behavior smoke from CI parity-only validation on macOS and Windows.
    - `docs/INSTALLATION.md` must scope the new behavior to Linux hosted installs and keep macOS/Windows text unchanged.
  - Contradiction risks:
    - A task model that does not separate Linux behavior smoke from macOS and Windows CI parity would overstate no-change-platform behavior requirements.
  - Resolution options (A/B):
    - Option A: narrow the task graph to Linux-only and drop cross-platform CI parity from the automation pack.
    - Option B: keep `meta.cross_platform=true`, require Linux behavior smoke, require Linux/macOS/Windows CI parity, and use the schema v4 boundary-only checkpoint model.
    - Selected: Option B.

### Policy / isolation / security posture
- Change: `/etc/os-release` parsing becomes security-relevant installer behavior.
  - Direct impact:
    - Production detection must read `ID` and `ID_LIKE` without `source`-executing the file.
    - Missing or unreadable os-release data degrades to `<unknown>` without network calls and without skipping explicit failure behavior for unsupported/forced manager selection.
  - Cascading impact:
    - `contract.md` must define the safe parsing rule, the literal `<unknown>` sentinel, and the `pkg_manager.source` vocabulary.
    - `tests/installers/pkg_manager_detection_smoke.sh` must cover both readable and unreadable os-release inputs.
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` must inherit `<unknown>` and `pkg_manager.source` verbatim instead of re-deriving values.
  - Contradiction risks:
    - `tests/installers/pkg_manager_container_smoke.sh` currently uses `source /etc/os-release`; that pattern must remain explicitly test-only and must not be copied into the installer implementation.
    - Any refactor that changes source-time behavior in `scripts/substrate/install-substrate.sh` risks breaking `scripts/substrate/world-enable.sh`, which sources the same file.

### Validation / operator-doc update targets
- Change: the pack needs one hermetic repo test and one feature-local Linux smoke wrapper, not two competing implementation authorities.
  - Direct impact:
    - `tests/installers/pkg_manager_detection_smoke.sh` becomes the exact repo test path for stubbed PATH/os-release validation.
    - `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh` becomes the feature-local wrapper that calls the same assertions and captures planning evidence.
  - Cascading impact:
    - `BEDPM3-spec.md` must make the repo test authoritative and the feature-local smoke script a thin orchestrating wrapper.
    - `docs/INSTALLATION.md` and `manual_testing_playbook.md` must reference the same visible stderr line and override/remediation examples as the repo test.
  - Contradiction risks:
    - ADR-0031 says no new platform smoke scripts are required beyond the hermetic test, while the spec manifest requires `smoke/linux-smoke.sh` as part of the pack artifact set.
  - Resolution options (A/B):
    - Option A: treat `smoke/linux-smoke.sh` as a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`, with no second contract.
    - Option B: drop the feature-local smoke script and rely only on the repo test.
    - Selected: Option A.

Operator-doc update targets:
- `docs/INSTALLATION.md`
  - Update the Linux quick-install narrative, offline install paragraph, and installer options table so they describe `--pkg-manager`, `PKG_MANAGER`, the stable detection line, and wrapper exit-code pass-through.
- `docs/reference/env/contract.md`
  - Add `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` with exact precedence, Linux-only scope, and absence semantics.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - `pkg_manager.selected`
    - `pkg_manager.source`
    - `<unknown>` os-release sentinel behavior
    - `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: let ADR-0032 invent its own fake-os-release hook and local vocabulary.
    - Option B: ADR-0031 owns detection, emitted vocabulary, `<unknown>` semantics, and the alternate os-release hook; ADR-0032 persists the emitted values only.
    - Selected: Option B.
    - Sequencing boundary: ADR-0031 must land before, or in the same planning wave as, ADR-0032 so persistence does not lock in a conflicting contract.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - package-manager wording shown to operators
  - Conflict: yes
  - Resolution (explicit):
    - Option A: let host prerequisite detection and in-world provisioning share one generic “package manager” operator narrative.
    - Option B: keep host prerequisite detection in ADR-0031 and keep `substrate world enable --provision-deps` semantics in ADR-0030.
    - Selected: Option B.
    - Shared-file rule: ADR-0030 must not redefine host PATH probing, distro mapping, or `pkg_manager.source`.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `pacman` vocabulary
    - `/etc/os-release` family names
    - operator wording around package-manager support
  - Conflict: no
  - Resolution (explicit):
    - Keep ADR-0031 scoped to host prerequisite installation.
    - Keep ADR-0033 scoped to in-world provisioning-time system-package support.
    - Use the same manager spellings without merging the host and guest contracts.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - installer/helper exit-code and messaging posture
  - Conflict: yes
  - Resolution (explicit):
    - Option A: refactor shared helpers in `scripts/substrate/install-substrate.sh` while adding distro detection.
    - Option B: confine ADR-0031 edits to argument parsing, detection, reporting, and prerequisite-install branches so world-enable and dev-install follow-on work rebase cleanly.
    - Selected: Option B.

- ADR: `docs/project_management/adrs/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - environment-variable documentation posture
    - installer path/config precedence language
  - Conflict: no
  - Resolution (explicit):
    - `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` remain installer-local env surfaces only.
    - ADR-0031 does not add config-file keys, `SUBSTRATE_HOME` semantics, or `substrate world enable` flags.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - `pkg_manager.selected`
    - `pkg_manager.source`
    - `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
  - Conflict: yes
  - Resolution (explicit):
    - Authority boundary:
      - `best-effort-distro-package-manager` owns detection, reporting, selected-manager spellings, source vocabulary, and alternate os-release input.
      - `persist-detected-linux-distro-pkg-manager` owns persistence into `install_state.json`.
    - Selected path: keep the downstream pack aligned to the exact env-var and vocabulary contract chosen here; do not duplicate detection logic there.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - operator package-manager wording
  - Conflict: yes
  - Resolution (explicit):
    - Option A: allow the world-deps pack to edit host-installer package-manager messaging.
    - Option B: keep host prerequisite detection in this pack and keep `world enable --provision-deps` messaging in the world-deps pack.
    - Selected: Option B.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `pacman` spelling
    - `/etc/os-release` family labels
    - operator package-manager wording
  - Conflict: no
  - Resolution (explicit):
    - Keep the host-installer meaning of `pacman` here.
    - Keep the guest-world provisioning meaning of `pacman` in that pack.
    - Do not reuse host PATH probing or `PKG_MANAGER` semantics for in-world provisioning.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
  - Conflict: no
  - Resolution (explicit):
    - Follow the same boundary already recorded in that pack: ADR-0031 changes stay in hosted-installer detection code and do not alter the sourced helper behavior relied on by `world-enable.sh`.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - helper-script staging/reuse assumptions
  - Conflict: no
  - Resolution (explicit):
    - Preserve source-time invariants in `scripts/substrate/install-substrate.sh`.
    - Keep ADR-0031 hosted-installer-only and do not expand into helper-discovery behavior.

### Relevant Archived Packs
- Archived Pack: `docs/project_management/_archived/world_deps_selection_layer/`
  - Overlap surfaces:
    - package-manager guidance
    - host mutation boundary language
  - Conflict: no
  - Resolution (explicit):
    - Retain the archived boundary that runtime world-deps flows do not mutate host packages.
    - ADR-0031 remains an explicit installer-time host prerequisite workflow and does not reopen archived host-mutation debates for runtime surfaces.

## Follow-ups (explicit)

- Decision Register entries required:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` — record the selected fallback PATH order and the selected wrapper exit-code pass-through rule.
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` — record the selected alternate os-release input contract: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` — record that `smoke/linux-smoke.sh` is a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`.

- Spec and planning updates required:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md` — pin `tests/installers/pkg_manager_detection_smoke.sh` as the exact hermetic repo test path and align the selected `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract.
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/tasks.json` — keep the schema v4 cross-platform task graph aligned to Linux behavior smoke, Linux/macOS/Windows CI parity, and the `BEDPM3` checkpoint boundary.
  - `docs/project_management/packs/sequencing.json` — add the feature entry and sequence ADR-0031 ahead of `persist-detected-linux-distro-pkg-manager`.

- Ownership-gap reminders:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` — remove any remaining ambiguity by inheriting `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, `<unknown>`, and `pkg_manager.source` exactly from this pack once ADR-0031 planning artifacts exist.
