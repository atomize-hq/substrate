**Warning: Pre-Planning Only. This draft is a temporary alignment backbone and downstream FSE planning or decomposition will supersede it.**

# persist-macos-host-os-install-state — minimal spec draft

## Scope and authority

This draft defines:
- cross-cutting defaults for the installer metadata write target
- precedence and ownership boundaries for the touched contract surfaces
- pack-wide invariants for warning-only degradation, additive compatibility, and protected data exclusions
- seam boundaries and draft slice candidates for downstream FSE planning
- concrete follow-ups that block downstream contract lock-in

This draft does not define:
- execution tasks
- kickoff prompts
- ownership of runtime worktrees
- detailed implementation sequencing
- implementation-level acceptance checklists

Final authoritative ownership lands in downstream docs:
- `contract.md` owns CLI surface, effective-prefix wording, and exit-code wording.
- `install-state-schema-spec.md` owns the serialized `install_state.json` schema touched by this feature.
- `filesystem-semantics-spec.md` owns on-disk write, temp-file, replace, and dry-run rules.
- `platform-parity-spec.md` owns hosted macOS scope, Linux and Windows no-change guarantees, and validation topology.
- `compatibility-spec.md` owns additive-only compatibility, unknown-key preservation, and recovery rules for unreadable or unsupported existing files.
- `manual_testing_playbook.md` owns deterministic operator validation steps after the contract locks.

## Defaults and precedence

- Canonical metadata file: `<effective_prefix>/install_state.json`
- Canonical temp path: `<effective_prefix>/install_state.json.tmp`
- Precedence order for the feature-owned write target:
  1. existing installer `--prefix` CLI input, when present
  2. existing installer default effective prefix `~/.substrate`
- Feature-local config keys: none
- Feature-local env vars: none
- Command surface delta: none
- Flag surface delta: none beyond reuse of the existing installer `--prefix` behavior
- Source-of-truth files and paths:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` will become the final conflict resolver for path wording and exit-code wording.
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` will define `schema_version`, `created_at`, `updated_at`, and `host_state.os.*`.
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` remains authoritative for Linux `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` semantics.
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/contract.md` remains authoritative for the existing Linux installer metadata contract that this feature preserves.

## Failure posture and invariants

- Failure posture: fail-open for installer metadata collection and persistence on an otherwise successful hosted macOS install. Warning paths do not convert installer success into installer failure.
- Scope posture: hosted macOS install and hosted macOS `--no-world` are in scope. `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/uninstall-substrate.sh`, and `scripts/substrate/dev-uninstall-substrate.sh` stay outside the feature-owned behavior change.
- Security posture: persist only the allowlisted macOS host facts required by ADR-0039 under `host_state.os.*`.
- Persisted allowlist for the new macOS block:
  - `host_state.os.family`
  - `host_state.os.product_version`
  - `host_state.os.build_version`
  - `host_state.os.arch`
- Explicit exclusions:
  - hostnames
  - serial numbers
  - broad `system_profiler` output
- Compatibility invariant: preserve unknown keys and preserve any pre-existing `host_state.group`, `host_state.linger`, and Linux `host_state.platform.*` content during macOS rewrites.
- Write invariant: use a same-directory temp-file write plus atomic replace flow. Do not truncate the canonical file in place.
- Recovery invariant: unreadable prior files and unsupported schema versions rebuild from a fresh `schema_version = 1` document after a warning path.
- Platform invariant: Linux and Windows externally visible behavior remains unchanged in this pack.

## Exit-code posture

- Exit-code taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- New exit codes required by ADR-0039: no
- Successful hosted macOS installs that degrade during host OS collection or metadata persistence still exit through the success path.
- Downstream `contract.md` must restate that this feature adds no exit-code override and stays inside the shared taxonomy.

## Cross-cutting seams and constraints

### Normalized vocabulary

- `effective_prefix`: the installer-resolved root used to derive the metadata write target.
- `install_state.json`: the canonical installer metadata file at `<effective_prefix>/install_state.json`.
- `host_state.os`: the additive macOS host OS object introduced by this feature.
- `schema_version`: top-level integer that remains `1`.
- `created_at`: top-level timestamp field owned by the schema spec.
- `updated_at`: top-level timestamp field owned by the schema spec.

### Pack-wide alignment rules

- Downstream docs must not redefine Linux `host_state.platform.*` semantics inside this feature pack.
- Downstream docs must keep the feature on the hosted-installer path and must not expand into dev-install runtime-bundle behavior.
- Operator-facing docs must separate macOS host OS persistence from Linux cleanup-state language so the resulting contract does not imply new macOS uninstall behavior.
- Validation docs must align on the existing harness split:
  - `tests/installers/install_state_smoke.sh` for shared file-shape, rewrite, and preservation assertions
  - `tests/mac/installer_parity_fixture.sh` for hosted macOS branch coverage and degraded-path coverage
- Filesystem ordering must align on one write flow across the spec set:
  1. resolve `<effective_prefix>`
  2. read existing `install_state.json` when present
  3. recover to a fresh `schema_version = 1` document after unreadable or unsupported prior content
  4. merge preserved values and the new `host_state.os.*` block
  5. write `<effective_prefix>/install_state.json.tmp`
  6. atomically replace `<effective_prefix>/install_state.json`
  7. emit warnings and perform temp-file cleanup on failure branches

## Downstream spec skeletons

- `contract.md`
  - unchanged command and flag surface
  - no new env vars
  - effective-prefix path rule
  - success-path exit-code posture for warning-only metadata degradation

- `install-state-schema-spec.md`
  - top-level file fields and timestamp ownership
  - `host_state.os.*` field list, types, and stored values
  - leaf absence semantics for partial capture failure
  - preserved existing metadata blocks and prohibited data families

- `filesystem-semantics-spec.md`
  - canonical path and temp path
  - parent-directory and dry-run behavior
  - same-directory temp write and atomic replace
  - warning-only read, write, replace, and cleanup branches

- `platform-parity-spec.md`
  - hosted macOS install matrix
  - hosted macOS `--no-world` matrix
  - Linux and Windows no-change guarantees
  - dev-install exclusion and validation evidence boundaries

- `compatibility-spec.md`
  - additive-only schema posture
  - unknown-key preservation
  - preserve-as-read behavior for existing Linux metadata
  - rebuild rules for unreadable files and unsupported schema versions

- `manual_testing_playbook.md`
  - hosted macOS validation run
  - hosted macOS `--no-world` validation run
  - warning-path validation
  - no-change assertions for uninstall and Linux-only cleanup-state behavior

## Follow-ups for downstream FSE planning and decomposition

- Lock the exact absence semantics for `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch` when their source commands fail independently during an otherwise successful hosted macOS install.
- Lock the exact `created_at` and `updated_at` rewrite behavior for first-write versus subsequent-write paths inside `schema_version = 1`.
- Lock the exact warning text and operator-facing evidence expected when an unreadable existing file or unsupported schema version triggers a rebuild path.
- Lock the exact dry-run rule for parent-directory creation, temp-file creation, and canonical-file creation on macOS hosted branches.
- Lock the exact automated validation ownership split between `tests/installers/install_state_smoke.sh` and `tests/mac/installer_parity_fixture.sh` so each assertion family has one owner.
- Lock the exact operator-doc edits in `docs/INSTALLATION.md` that separate macOS host OS persistence from Linux cleanup-state guidance.
- Record explicit no-change assertions for `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/uninstall-substrate.sh`, and `scripts/substrate/dev-uninstall-substrate.sh` in downstream validation docs.

## Draft seam and slice-candidate skeleton (pre-planning only)

draft; may split/merge during downstream FSE planning or decomposition

Candidate prefix (draft): `PMHOS`

Baseline candidate count from `spec_manifest.md`: `2`

### Candidate 1

- `candidate_id`: `PMHOS-S1`
- `name`: `seam-1-install-state-surface-lock`
- `intent`: lock the authoritative contract, schema, filesystem, and compatibility surfaces for additive macOS persistence in the existing `install_state.json`
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` as linked evidence only

### Candidate 2

- `candidate_id`: `PMHOS-S2`
- `name`: `seam-2-macos-validation-and-doc-alignment`
- `intent`: lock platform parity wording, automated and manual validation topology, and operator-doc reconciliation for hosted macOS install-state writes without expanding into dev-install or uninstall behavior changes
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
  - `tests/installers/install_state_smoke.sh`
  - `tests/mac/installer_parity_fixture.sh`
  - `docs/INSTALLATION.md`

Downstream note:
- `ci_checkpoint_plan.md` may use this candidate list when proposing checkpoint groups.
- `workstream_triage.md` may recommend edits to this skeleton, but it does not own this file.
