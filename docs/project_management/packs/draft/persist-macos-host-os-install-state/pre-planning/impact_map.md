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

FSE pre-planning touch-set rules:
- each entry is a top-level bullet containing exactly one backticked path token
- an empty section is exactly `- None`
- the touch set includes at least one non-None entry total across all sections
- exact file paths are the default expectation
- directory-prefix entries are fallback-only and must end with `/`

### Create
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`

### Edit
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`
- `tests/installers/install_state_smoke.sh`
- `tests/mac/installer_parity_fixture.sh`
- `docs/INSTALLATION.md`

### Deprecate
- None

### Delete
- None

## Implementation surface note

- No `crates/` path is required by ADR-0039.
- No `src/` path is required by ADR-0039.
- No `crates/world*`, `crates/shim`, `crates/shell`, or `crates/world-agent` path is required by ADR-0039.
- `scripts/substrate/uninstall-substrate.sh` and `scripts/substrate/dev-uninstall-substrate.sh` are implicated reader surfaces and validation boundaries, but ADR-0039 does not require direct edits there because both readers already consume `host_state.group` and `host_state.linger` only and ignore unrelated `host_state` siblings.

## Cascading implications (behavior and UX)

### CLI / UX
- Change: Successful macOS installs become `install_state.json` producers instead of non-producers.
  - Direct impact:
    - Hosted macOS installs create or update `<effective_prefix>/install_state.json`.
    - Hosted macOS `--no-world` installs create or update the same file.
    - Operator guidance shifts from “macOS does not write install state” to “macOS writes diagnostic-only `host_state.os.*` metadata.”
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` must stop returning early from `write_host_state_metadata()` on macOS and must collect `sw_vers -productVersion`, `sw_vers -buildVersion`, and `uname -m` in the same warning-only writer flow.
    - `docs/INSTALLATION.md` must split Linux `host_state.platform.*` wording from macOS `host_state.os.*` wording while keeping Windows on the no-write side.
    - Manual validation must inspect the file after macOS hosted install and macOS hosted `--no-world`.
  - Contradiction risks:
    - The current installer metadata section in `docs/INSTALLATION.md` states that macOS and Windows do not write the file.
    - The current hosted installer writer exits on every non-Linux branch, so the ADR contract and the implementation diverge today.

- Change: Producer scope must cover both installer entrypoints that already own the shared install-state writer pattern.
  - Direct impact:
    - Hosted install and dev install present one macOS producer contract instead of two different platform stories.
  - Cascading impact:
    - `scripts/substrate/dev-install-substrate.sh` needs the same macOS collection and additive-write behavior as the hosted installer.
    - `contract.md`, `filesystem-semantics-spec.md`, and `platform-parity-spec.md` must define the same producer scope.
  - Contradiction risks:
    - A hosted-only scope leaves `scripts/substrate/dev-install-substrate.sh` on a Linux-only writer contract while the repo still treats both installers as install-state producers on Linux.
  - Resolution:
    - Option A: hosted installer only.
    - Option B: hosted installer plus dev installer.
    - Selected: Option B.

### Config / env vars / paths
- Change: `install_state.json` remains the single canonical metadata file and gains an additive macOS block under `host_state.os.*`.
  - Direct impact:
    - `schema_version` stays integer `1`.
    - `host_state.group`, `host_state.linger`, Linux `host_state.platform.*`, and unknown keys remain intact across rewrites.
    - No second metadata file appears beside `install_state.json`.
  - Cascading impact:
    - `install-state-schema-spec.md` must lock the exact field set for `host_state.os.family`, `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch`.
    - `compatibility-spec.md` must lock unknown-key preservation and reader tolerance of the new subtree.
    - `docs/INSTALLATION.md` must keep the canonical path wording aligned with `<effective_prefix>/install_state.json` and the default alias `~/.substrate/install_state.json`.
  - Contradiction risks:
    - Reusing `host_state.platform.*` for macOS would collide with the implemented Linux schema owner.
    - Writing a second file such as `host_os.json` would violate the ADR decision summary and split operator guidance across two metadata surfaces.

- Change: Partial-capture serialization needs one deterministic rule for best-effort command failures.
  - Direct impact:
    - Future consumers can rely on one stable block-presence rule instead of inferring intent from ad hoc omissions.
  - Cascading impact:
    - The writer implementation, schema examples, and validation harnesses must all encode the same missing-leaf behavior.
    - Support and diagnostics docs can state one fallback rule for absent leaves.
  - Contradiction risks:
    - An all-or-nothing block conflicts with ADR-0039 read semantics that tolerate missing or partial values.
    - A block that omits `host_state.os.family` weakens the “this file came from macOS install” signal.
  - Resolution:
    - Option A: always write `host_state.os.family = "macos"` and write every collected leaf that succeeds.
    - Option B: require all collectors to succeed before writing `host_state.os`.
    - Selected: Option A.

### Policy / isolation / security posture
- Change: The writer keeps the existing best-effort, warning-only, atomic-replace posture.
  - Direct impact:
    - Collection failures, parse failures, temp-file write failures, and replace failures do not turn an otherwise successful install into failure.
    - The file still uses same-directory temp-file plus atomic replace instead of in-place truncation.
  - Cascading impact:
    - `filesystem-semantics-spec.md` must define one recovery path for malformed existing JSON and one recovery path for failed temp-file write or replace.
    - The installer scripts must share one warning prefix and cleanup path for temp-file removal.
    - Validation must cover malformed JSON recovery and failed command collection without asserting new exit-code branches.
  - Contradiction risks:
    - Any fatal error path tied to `sw_vers`, `uname -m`, or metadata writes would violate the ADR goals.
    - Any direct file truncation path would diverge from the Linux persistence posture already shipped for `install_state.json`.

- Change: Cleanup readers stay Linux-only even after macOS starts writing `host_state.os.*`.
  - Direct impact:
    - macOS metadata remains diagnostic-only and does not create new cleanup actions.
    - Hosted and dev uninstallers continue to act on `host_state.group` and `host_state.linger` only.
  - Cascading impact:
    - `platform-parity-spec.md` and `manual_testing_playbook.md` must include a no-change reader boundary for cleanup.
    - Validation must keep a Linux uninstall assertion that ignores the macOS subtree.
  - Contradiction risks:
    - Treating `host_state.os.*` as cleanup input would invent new uninstall behavior outside ADR scope.

### Validation / support flows
- Change: Automated evidence needs one primary macOS harness plus one shared writer-compatibility harness.
  - Direct impact:
    - macOS producer assertions land in a harness that already executes macOS install and uninstall flows.
    - Shared additive-merge and Linux no-change assertions stay in the cross-installer install-state harness.
  - Cascading impact:
    - `tests/mac/installer_parity_fixture.sh` must assert macOS `install_state.json` creation and `host_state.os.*` content for hosted install flows.
    - `tests/installers/install_state_smoke.sh` must assert that Linux rewrites preserve a preexisting `host_state.os.*` block and keep uninstall cleanup behavior unchanged.
    - `platform-parity-spec.md` must map the two harnesses to their exact evidence roles.
  - Contradiction risks:
    - `tests/installers/install_state_smoke.sh` currently exits early on non-Linux hosts, so it cannot stand alone as macOS proof without a larger harness redesign.
    - Leaving `tests/mac/installer_parity_fixture.sh` untouched would leave the pack without an automated macOS producer assertion.
  - Resolution:
    - Option A: extend `tests/installers/install_state_smoke.sh` into the primary macOS producer harness.
    - Option B: use `tests/mac/installer_parity_fixture.sh` as the primary macOS producer harness and extend `tests/installers/install_state_smoke.sh` for shared merge and no-change coverage.
    - Selected: Option B.

## Cross-queue scan (ADRs + Planning Packs)

Active-pack scan found no overlap in `docs/project_management/packs/active/`.
Queued-pack scan found no overlapping pack directory under `docs/project_management/packs/queued/`.

### Relevant ADRs
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/dev-install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
    - `install_state.json`
  - Conflict: yes
  - Resolution:
    - ADR-0032 remains the authority for Linux `host_state.platform.*`, canonical path wording, additive merge posture, and same-directory temp-file replacement.
    - ADR-0039 adds macOS data under `host_state.os.*` only.
    - ADR-0039 does not rename, remove, or repurpose Linux field paths.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution:
    - ADR-0034 owns dev-install helper and runtime-bundle staging under `SUBSTRATE_HOME`.
    - ADR-0039 owns install-state collection and persistence under the same script.
    - Edits in `scripts/substrate/dev-install-substrate.sh` must stay isolated to metadata collection and shared-writer logic.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution:
    - ADR-0035 owns late-enable world-agent staging and preflight behavior.
    - ADR-0039 owns macOS install-state writes.
    - ADR-0039 does not change `--no-world` staging, world-agent paths, or enable-later remediation.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: no
  - Resolution:
    - ADR-0030 owns provisioning-time package installation and `world enable --provision-deps` guidance.
    - ADR-0039 owns persisted host OS metadata.
    - Shared-file edits must keep those boundaries separate.

- ADR: `docs/project_management/adrs/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - macOS operator guidance
  - Conflict: no
  - Resolution:
    - The virtualization ADR owns backend selection, Lima fallback, and provisioning prerequisites.
    - ADR-0039 owns the durable host OS record written after successful install.
    - No Lima, VF, or backend-selection behavior enters this touch set.

### Relevant Planning Packs
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: no
  - Resolution:
    - The world-deps pack owns provisioning-time package installation and related operator guidance.
    - This pack owns host metadata persistence only.
    - Shared-file edits must preserve `world enable --provision-deps` wording and must not move that responsibility into install-state docs.

### Relevant Archived Packs
- Planning Pack: `docs/project_management/_archived/p0-platform-stability/`
  - Overlap surfaces:
    - `install_state.json`
    - installer cleanup metadata
    - uninstall cleanup guidance
  - Conflict: no
  - Resolution:
    - This archived pack is evidence for the existing `host_state.group` and `host_state.linger` cleanup contract.
    - ADR-0039 extends the same file with diagnostic-only macOS fields and does not reopen cleanup behavior.

- Planning Pack: `docs/project_management/_archived/p0-platform-stability-macOS-parity/`
  - Overlap surfaces:
    - `tests/mac/installer_parity_fixture.sh`
    - macOS installer parity guidance
  - Conflict: no
  - Resolution:
    - This archived pack is evidence that the parity fixture already owns macOS install and uninstall flow assertions.
    - ADR-0039 uses that harness for new macOS metadata assertions instead of inventing a new feature-local smoke script.

## Follow-ups

- Decision or clarification follow-ups:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md` — lock DR-0001 to hosted installer plus dev installer, DR-0002 to `host_state.os.family` plus available leaves, and DR-0003 to `tests/mac/installer_parity_fixture.sh` primary plus `tests/installers/install_state_smoke.sh` secondary.
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md` — reconcile the hosted-only validation wording with the selected shared producer scope before promotion.

- Spec updates required:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` — define the no-new-CLI contract, canonical path wording, warning-only diagnostics, and future-consumer read precedence.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` — define the `host_state.os.*` field set, field-level absence semantics, and merge preservation of Linux and cleanup subtrees.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` — define the same-directory temp-file path, atomic replace sequence, parse-failure recovery, and failed-write cleanup posture.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` — define the macOS producer matrix, Linux and Windows no-change guarantees, the uninstaller no-change boundary, and the exact automated evidence map.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` — define additive-only compatibility, unknown-key preservation, and reader tolerance of `host_state.os.*`.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md` — cover hosted macOS install, hosted macOS `--no-world`, dev-install macOS producer coverage, malformed-file recovery, and Linux cleanup no-change verification.
