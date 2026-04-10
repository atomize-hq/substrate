# persist-macos-host-os-install-state — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

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

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/persist-macos-host-os-install-state"` (strict packs only).

### Create
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/plan.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS0/PMHOIS0-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/tasks.json`
- `scripts/substrate/install-substrate.sh`
- `tests/mac/installer_parity_fixture.sh`
- `tests/installers/install_state_smoke.sh`
- `docs/INSTALLATION.md`

### Deprecate
- None

### Delete
- None

## Implementation surface note

Implementation stays inside the hosted installer script, installer validation harnesses, operator docs, and this feature’s planning docs.

No implementation change enters:
- `src/`
- `crates/`
- `crates/world/`
- `crates/world-mac-lima/`
- `crates/world-windows-wsl/`
- `crates/shim/`
- `crates/shell/`
- `crates/world-agent/`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `scripts/mac/smoke.sh`
- `docs/WORLD.md`

Shared-file guard:
- `scripts/substrate/install-substrate.sh` already owns the additive `install_state.json` writer, the temp-file replace flow, and the hosted macOS provisioning path.
- This feature extends that existing writer for hosted macOS only. It does not fork a second metadata file, a second writer helper, or a dev-installer copy of the new contract.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: successful hosted macOS installs create or update `<effective_prefix>/install_state.json`, including the hosted `--no-world` branch, while hosted `--dry-run` stays no-write.
  - Direct impact:
    - A successful macOS install leaves a stable metadata file at the effective prefix.
    - The file records `host_state.os.family`, `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch` when command collection succeeds.
    - The hosted installer log gains the same visible metadata-write confirmation path that Linux already uses.
  - Cascading impact:
    - `docs/INSTALLATION.md` must stop stating that macOS does not write `install_state.json`.
    - The operator docs must separate diagnostic metadata from Linux-only cleanup actions so the file’s presence on macOS does not imply `--cleanup-state` support.
    - `plan.md` and `tasks.json` must require validation for the normal hosted path and the hosted `--no-world` path on macOS.
  - Contradiction risks:
    - `scripts/substrate/install-substrate.sh` returns early from `write_host_state_metadata()` whenever `PLATFORM != linux`.
    - `docs/INSTALLATION.md` states that macOS and Windows do not write the file.
    - `tests/mac/installer_parity_fixture.sh` exercises macOS installer flows today without asserting metadata output.
  - Conflict resolution:
    - Option A: expand this feature to the dev installer and to macOS cleanup semantics so every installer surface uses one macOS contract immediately.
    - Option B: keep the ADR boundary exactly where ADR-0039 puts it: hosted macOS installer only, no macOS cleanup behavior change, docs and tests updated to make that boundary explicit.
    - Selected: Option B.

- Change: fresh macOS metadata files use the same top-level additive schema but do not seed empty Linux cleanup containers.
  - Direct impact:
    - A fresh macOS file carries `schema_version`, timestamps, `host_state`, and the `host_state.os.*` block without empty `host_state.group` or `host_state.linger` scaffolding.
    - Existing Linux cleanup containers remain intact when the installer updates a file that already contains them.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` must stop unconditional `group` and `linger` seeding for fresh macOS files while preserving those sections when they already exist.
    - `tests/installers/install_state_smoke.sh` must keep ownership of additive-merge assertions so the shared writer keeps existing Linux content and unknown keys intact.
    - `install-state-schema-spec.md` and `compatibility-spec.md` must lock the fresh-file and merge behavior in one place.
  - Contradiction risks:
    - The current Python writer always seeds `host_state.group` and `host_state.linger`.
    - That behavior broadens macOS `install_state.json` from “diagnostic metadata” into “diagnostic metadata plus empty Linux cleanup containers,” which blurs the cleanup contract.
  - Conflict resolution:
    - Option A: fresh macOS files write only the top-level metadata plus `host_state.os.*`, and existing Linux cleanup containers are preserved only when already present.
    - Option B: fresh macOS files also seed empty `host_state.group` and `host_state.linger` containers.
    - Selected: Option A.

### Config / env vars / paths
- Change: the canonical metadata path remains `<effective_prefix>/install_state.json`; the feature adds no new CLI flags, no new config keys, and no new environment variables.
  - Direct impact:
    - Operators keep one canonical file path at the effective install prefix.
    - The default macOS path remains `~/.substrate/install_state.json`.
    - Future consumer guidance has one persisted source of truth for “what macOS version/build/arch was this installed on?”.
  - Cascading impact:
    - `contract.md` must lock the exact path rule and the “future consumers prefer persisted values, then fall back to runtime detection” rule.
    - `tests/mac/installer_parity_fixture.sh` must own macOS execution-path assertions, including `sw_vers` and `uname -m` stubs and the hosted `--no-world` path.
    - `tests/installers/install_state_smoke.sh` must own shared JSON and filesystem semantics assertions so the writer contract stays aligned across Linux and macOS.
  - Contradiction risks:
    - The current macOS smoke story in `docs/WORLD.md` reuses the BEDPM Linux harness for hosted installer verification, which does not cover `host_state.os.*`.
    - The current `install_state_smoke.sh` harness is Linux-centric and does not encode the new macOS writer branch.
  - Conflict resolution:
    - Option A: use `tests/mac/installer_parity_fixture.sh` as the primary macOS execution harness and reuse `tests/installers/install_state_smoke.sh` for shared JSON assertions and atomic-write semantics.
    - Option B: treat `scripts/mac/smoke.sh` or the BEDPM wrapper as the primary authority for macOS installer validation.
    - Selected: Option A.

### Policy / isolation / security posture
- Change: host OS collection stays warning-only, additive, and low-sensitivity; it introduces no new trace field, no new log schema field, no new redaction rule, and no new policy behavior.
  - Direct impact:
    - Failure to read `sw_vers`, failure to read `uname -m`, parse failure on an existing file, temp-file write failure, and replace failure all leave an otherwise successful install successful.
    - The feature persists only OS family, version, build, and architecture. It does not persist hostname, serial number, or broad `system_profiler` output.
  - Cascading impact:
    - `filesystem-semantics-spec.md` must define the same-directory temp-file path, replace ordering, cleanup behavior, and prior-file preservation rule.
    - The validation harnesses must cover warning-only degradation branches without turning them into install failures.
    - `docs/INSTALLATION.md` must state that Linux-only cleanup commands remain Linux-only even when the metadata file exists on macOS.
  - Contradiction risks:
    - The current writer is Linux-only, so every macOS success branch today silently skips the metadata path.
    - ADR-0032 and its implemented pack already treat the same writer and file as the canonical additive install-state mechanism for Linux.
  - Conflict resolution:
    - Option A: add a separate macOS-only metadata writer or second file so Linux and macOS stop sharing the same install-state path.
    - Option B: keep one additive writer in `scripts/substrate/install-substrate.sh`, extend it with a macOS branch, and preserve existing Linux data and unknown keys.
    - Selected: Option B.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
    - `install_state.json`
    - additive merge semantics for `host_state.*`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: let ADR-0039 redefine the shared writer and file shape independently from the Linux install-state work.
    - Option B: treat ADR-0032 as the prior-art owner for Linux `host_state.platform.*`, timestamp preservation, unknown-key preservation, and temp-file replace semantics; ADR-0039 adds macOS `host_state.os.*` only.
    - Selected: Option B.
    - Sequencing boundary: the shared writer contract stays additive with `schema_version = 1`, so either pack can land first without forcing a file-format split.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: combine world-deps provisioning edits and macOS install-state edits inside one refactor of the hosted installer path.
    - Option B: keep ADR-0039 inside the install-state writer, macOS source-command collection, validation harnesses, and install-state docs while ADR-0030 owns world-deps provisioning branches and remediation text.
    - Selected: Option B.
    - Shared-file rule: edits for ADR-0039 stay out of `world deps current sync` and `--provision-deps` logic.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - installer workflow messaging around `--no-world`
    - future operator expectations for “install now, enable later”
  - Conflict: no
  - Resolution (explicit):
    - Explicit non-overlap boundary:
      - ADR-0035 owns dev-install `--no-world` artifact staging and later `world enable` readiness.
      - ADR-0039 owns hosted macOS metadata persistence after a successful install.
    - `scripts/substrate/dev-install-substrate.sh` stays out of this touch set unless ADR-0039 is amended.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: let the shared-file overlap drive one combined planning slice.
    - Option B: keep the overlap as sequencing-only, with ADR-0039 owning install-state metadata flow and the world-deps pack owning dependency provisioning flow.
    - Selected: Option B.

- Planning Pack: `docs/project_management/_archived/p0-platform-stability/`
  - Overlap surfaces:
    - historical `install_state.json` manual validation notes
    - archived installer/uninstaller touch lists
  - Conflict: no
  - Resolution (explicit):
    - Treat the archived pack as evidence only.
    - Do not copy its historical “inspect `install_state.json`” guidance into the canonical contract without restating the new macOS boundary in this pack’s specs and docs.

- Planning Pack: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
  - Overlap surfaces:
    - `host_state.os.*`
    - hosted macOS installer metadata
  - Conflict: no
  - Resolution (explicit):
    - No other active, queued, or draft pack claims `host_state.os.*` or the hosted macOS `sw_vers` / `uname -m` collection path.
    - This pack is the sole owner of that contract surface.

## Follow-ups (explicit)

- Decision Register entries required:
  - `DR-0001 — fresh macOS file scaffolding` — mirror the selected Option A: fresh macOS files write top-level metadata plus `host_state.os.*` and do not seed empty Linux cleanup containers.
  - `DR-0002 — partial `host_state.os` emission` — close the remaining choice between partial field emission and whole-block suppression when one source command fails, then align `contract.md`, `install-state-schema-spec.md`, and both validation harnesses.
  - `DR-0003 — primary macOS automated validation vehicle` — mirror the selected Option A: `tests/mac/installer_parity_fixture.sh` owns macOS execution-path assertions and `tests/installers/install_state_smoke.sh` owns shared JSON and filesystem assertions.

- Spec updates required:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` — lock the hosted-installer-only scope, the hosted `--no-world` write rule, the hosted `--dry-run` no-write rule, and the Linux-only cleanup boundary.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` — lock the exact `host_state.os.*` field set, fresh-file shape, merge behavior, and partial-emission rule.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` — lock temp-file naming, same-directory replace ordering, prior-file preservation, and warning-only failure semantics.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` — lock macOS hosted and hosted `--no-world` guarantees, Linux no-delta guarantees, and Windows no-delta guarantees.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` — lock preservation of unknown keys, `host_state.group`, `host_state.linger`, and `host_state.platform`.

- Tightening follow-ups:
  - `tests/mac/installer_parity_fixture.sh` needs exact scenario naming for metadata success, hosted `--no-world`, source-command degradation, and cleanup-guidance-on-mac-with-metadata-present.
  - `tests/installers/install_state_smoke.sh` needs a macOS branch or shared helper extraction that proves the macOS writer path without turning the file into a Linux-only schema authority.
  - `docs/INSTALLATION.md` needs one explicit sentence that `install_state.json` on macOS is diagnostic metadata, while `--cleanup-state` remains Linux-only behavior.
