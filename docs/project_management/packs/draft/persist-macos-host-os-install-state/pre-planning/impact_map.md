# persist-macos-host-os-install-state — impact map

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- Spec manifest:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created, edited, deprecated, or removed. Use repo-relative paths.

FSE pre-planning touch-set rules:
- each entry is a top-level bullet containing exactly one backticked path token
- an empty section is exactly `- None`
- the touch set must include at least one non-None entry total across all sections
- exact file paths are the default expectation
- directory-prefix entries are fallback-only and must end with `/`

### Create
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/threaded-seams/seam-1-install-state-surface-lock.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/threaded-seams/seam-2-macos-validation-and-doc-alignment.md`

### Edit
- `scripts/substrate/install-substrate.sh`
- `tests/installers/install_state_smoke.sh`
- `tests/mac/installer_parity_fixture.sh`
- `docs/INSTALLATION.md`

### Deprecate
- None

### Delete
- None

## Implementation surface note

- This ADR stays inside installer scripts, validation harnesses, operator docs, and planning-pack docs.
- No `src/`, `crates/`, `crates/world/`, `crates/world-mac-lima/`, `crates/world-agent/`, `crates/shim/`, or `crates/shell/` path enters the owned touch set.
- `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/uninstall-substrate.sh`, and `scripts/substrate/dev-uninstall-substrate.sh` are explicit no-change review surfaces for this ADR.
- Current repo evidence shows no runtime reader for `host_state.os.*`; this ADR ends at producer behavior, validation, and operator documentation.

## Cascading implications (behavior and UX)

For each externally visible change, list:
- direct impact,
- cascading impact,
- contradiction risks.

### CLI / UX
- Change: successful macOS hosted installs, including hosted `--no-world`, create or update `<effective_prefix>/install_state.json` with `host_state.os.*`; `scripts/substrate/dev-install-substrate.sh` stays outside this ADR.
  - Direct impact:
    - Operators see `install_state.json` after successful macOS hosted installs at the same effective-prefix path already used on Linux.
    - Support flows gain stable macOS family, product version, build version, and architecture values without re-probing the host.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` has to remove the Linux-only producer boundary inside `write_host_state_metadata()` for macOS hosted success branches and collect `sw_vers` plus `uname -m` into the existing JSON writer.
    - `tests/mac/installer_parity_fixture.sh` has to cover default hosted install, hosted `--no-world`, seeded-file rewrite, and warning-only capture failures under a macOS host stub.
    - `docs/INSTALLATION.md` has to split Linux `host_state.platform.*` language from macOS `host_state.os.*` language and keep cleanup guidance Linux-only.
  - Contradiction risks:
    - `docs/INSTALLATION.md` currently states macOS does not write `install_state.json`.
    - Pulling `scripts/substrate/dev-install-substrate.sh` into scope reopens the landed dev-install `--no-world` and runtime-bundle contracts owned by other packs.
  - Resolution:
    - Option A: hosted installer only.
    - Option B: hosted installer plus dev installer.
    - Selected: Option A.

### Config / env vars / paths
- Change: `install_state.json` stays `schema_version = 1` and gains additive `host_state.os.family`, `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch`.
  - Direct impact:
    - One file continues to carry cross-platform installer metadata instead of splitting macOS facts into a second file.
    - Existing Linux `host_state.platform.*`, `host_state.group`, `host_state.linger`, and unknown keys survive macOS rewrites.
  - Cascading impact:
    - `install-state-schema-spec.md` has to lock exact field types, literal `family = "macos"`, canonical stored values, and leaf absence rules.
    - `filesystem-semantics-spec.md` has to lock `<effective_prefix>/install_state.json`, same-directory `.tmp`, atomic replace, cleanup after failed temp writes, and hosted dry-run no-write behavior.
    - `tests/installers/install_state_smoke.sh` has to keep shared replace, rebuild, and preservation regressions aligned with the additive contract while `tests/mac/installer_parity_fixture.sh` proves the macOS runtime branch.
  - Contradiction risks:
    - Rebuilding `host_state` from scratch drops pre-existing Linux metadata and violates the implemented Linux contract.
    - Moving the new data into `config.yaml` or a second file conflicts with ADR-0039 Option A.
    - Doc text that equates “macOS writes `install_state.json`” with “macOS uninstall consumes cleanup-state metadata” creates a false operator contract.
  - Resolution:
    - Option A: extend the existing `install_state.json` surface.
    - Option B: create a second macOS metadata file or reuse `config.yaml`.
    - Selected: Option A.

### Policy / isolation / security posture
- Change: metadata capture remains best-effort, warning-only, and producer-side; cleanup-state, world provisioning, and backend behavior remain unchanged.
  - Direct impact:
    - A successful macOS hosted install stays successful when OS-detail capture or metadata persistence degrades.
    - `--cleanup-state` remains Linux-only even when macOS hosted installs start leaving `install_state.json` behind.
  - Cascading impact:
    - `compatibility-spec.md` has to define unreadable-file rebuild rules, unsupported-schema rebuild rules, unknown-key preservation, and future-consumer tolerance for missing or partial `host_state.os.*`.
    - `manual_testing_playbook.md` has to cover hosted macOS and hosted macOS `--no-world` plus warning-only degraded paths.
    - `docs/INSTALLATION.md` has to state plainly that macOS writes host OS metadata but does not gain Linux group or linger cleanup automation.
  - Contradiction risks:
    - Treating macOS metadata as a provisioning prerequisite changes ADR scope and collides with the existing Lima setup contract.
    - Extending uninstall scripts or cleanup-state behavior in this ADR collides with existing Linux-only cleanup semantics and archived parity docs.
  - Resolution:
    - Option A: producer-only metadata capture with no uninstall behavior change.
    - Option B: fold cleanup-state or dev-install runtime-bundle changes into this pack.
    - Selected: Option A.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps or conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
    - `install_state.json`
  - Conflict: yes
  - Resolution:
    - Option A: reuse the implemented Linux additive contract and extend the same file under `host_state.os`.
    - Option B: redefine the file as macOS-owned metadata or bump the schema.
    - Selected: Option A.
    - Linux `host_state.platform.*` semantics remain owned by `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `$SUBSTRATE_HOME/scripts/mac/`
    - macOS dev-install runtime-bundle staging
  - Conflict: no
  - Resolution:
    - Option A: keep dev-install outside ADR-0039.
    - Option B: expand ADR-0039 into macOS dev-install metadata parity.
    - Selected: Option A.
    - The impact-map touch set excludes `scripts/substrate/dev-install-substrate.sh` so helper staging and copied Linux guest binaries remain under ADR-0034 ownership.

- ADR: `docs/project_management/adrs/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`
  - Overlap surfaces:
    - `sw_vers`
    - `uname -m`
    - operator diagnostics for macOS backend issues
  - Conflict: no
  - Resolution:
    - ADR-0039 records host facts only.
    - Lima provisioning, Virtualization.framework checks, entitlements, and guest bootstrap behavior stay outside this touch set.

### Relevant Planning Packs
- Planning Pack: `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `install_state.json`
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution:
    - Option A: keep one additive cross-platform metadata file and preserve the Linux-owned schema and compatibility rules.
    - Option B: fork the metadata model for macOS.
    - Selected: Option A.
    - This pack owns `host_state.os.*` plus macOS success-branch semantics; the implemented Linux pack keeps ownership of `host_state.platform.*` and the existing reliable-write contract.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: no
  - Resolution:
    - Keep macOS metadata writes scoped to producer behavior and operator docs.
    - Keep `substrate world enable --provision-deps` remediation text and provisioning behavior under the world-deps pack.

- Planning Pack: `docs/project_management/packs/implemented/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `$SUBSTRATE_HOME/scripts/substrate/`
    - `$SUBSTRATE_HOME/scripts/mac/`
  - Conflict: no
  - Resolution:
    - The selected hosted-only scope leaves `scripts/substrate/dev-install-substrate.sh` untouched.
    - Dev-install helper staging, protected-path cleanup, and macOS runtime-bundle assets remain under the implemented helper-discovery pack.

- Planning Pack: `docs/project_management/_archived/p0-platform-stability-macOS-parity/`
  - Overlap surfaces:
    - `tests/mac/installer_parity_fixture.sh`
    - macOS installer metadata and cleanup-state language
  - Conflict: yes
  - Resolution:
    - Treat the archived pack as evidence only.
    - Current planning truth is ADR-0039 plus the implemented Linux install-state contract.
    - macOS hosted installs gain `host_state.os.*`; cleanup-state remains Linux-only.

- Planning Pack: `docs/project_management/packs/active/`
  - Overlap surfaces:
    - None
  - Conflict: no
  - Resolution:
    - Current repo scan found no active pack that adds another overlapping `install_state.json` producer or macOS installer metadata contract.

- Planning Pack: `docs/project_management/packs/queued/`
  - Overlap surfaces:
    - None
  - Conflict: no
  - Resolution:
    - Current repo scan found no queued pack that adds another overlapping `install_state.json` producer or macOS installer metadata contract.

## Follow-ups

- Decision or clarification follow-ups:
  - Any runtime consumer that reads `host_state.os.*` needs a separate ADR or planning pack; this ADR stops at producer behavior, validation, and operator docs.
- Spec updates required:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` — pin exact stored values and exact leaf absence semantics when `sw_vers -productVersion`, `sw_vers -buildVersion`, or `uname -m` fails during an otherwise successful hosted install.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` — state that hosted install and hosted `--no-world` are in scope, and `scripts/substrate/dev-install-substrate.sh` remains out of scope and unchanged.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md` — add explicit no-change assertions for `scripts/substrate/uninstall-substrate.sh` and `scripts/substrate/dev-uninstall-substrate.sh` plus the Linux-only cleanup-state guidance on mac hosts.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` — pin the temp-file cleanup rule and the hosted dry-run no-write rule for macOS branches.
