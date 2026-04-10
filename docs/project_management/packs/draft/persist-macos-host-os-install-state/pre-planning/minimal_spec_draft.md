**Warning: Pre-Planning Only. This document is pre-planning only and will be superseded by downstream FSE planning or decomposition artifacts.**

# persist-macos-host-os-install-state minimal spec draft

## Scope and authority

This draft defines the pack-level alignment backbone for ADR-0039 and the related pre-planning inputs. It exists to freeze the cross-cutting defaults, precedence rules, invariants, seam boundaries, and follow-ups that downstream planning needs before topic-specific specs and decomposition docs are authored.

This draft is allowed to define:
- cross-cutting defaults that every downstream doc must inherit
- precedence and source-of-truth boundaries for the metadata path, schema ownership, failure posture, and compatibility posture
- seam boundaries between the new macOS `host_state.os.*` subtree and the preexisting Linux and cleanup metadata surfaces
- unresolved follow-ups that downstream FSE planning must close explicitly

This draft does not define:
- execution tasks
- kickoff prompts
- ownership of runtime worktrees
- detailed implementation sequencing
- code-level acceptance checklists

Authoritative inputs for this draft:
- `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`

External authorities that remain in force without redefinition here:
- `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/contract.md`
- `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

## Defaults and precedence

Feature-local defaults:
- This feature introduces no new CLI command, no new CLI flag, no new config key, and no new environment variable.
- The existing installer mechanism that resolves `effective_prefix` remains authoritative upstream of this feature.
- After `effective_prefix` is resolved, the canonical metadata path is exactly `<effective_prefix>/install_state.json`.
- The default-prefix alias for operator-facing docs is `~/.substrate/install_state.json`.
- This feature does not introduce a second metadata file.

Feature-local precedence:
1. Existing installer CLI/config/env rules resolve `effective_prefix`.
2. The canonical metadata file path is derived from that resolved prefix as `<effective_prefix>/install_state.json`.
3. Future consumers prefer persisted `host_state.os.*` values when those values are present and readable.
4. Future consumers fall back to runtime detection when persisted values are absent, partial, malformed, or unreadable.

Source-of-truth ownership for downstream docs:
- `contract.md` owns the user-visible path rule, warning posture, future-consumer precedence, and exit-code posture.
- `install-state-schema-spec.md` owns `schema_version = 1`, the `host_state.os.*` field list, and merge-preservation rules for existing subtrees.
- `filesystem-semantics-spec.md` owns the same-directory temp-file rule, replace ordering, parse-failure recovery, and failed-write recovery.
- `platform-parity-spec.md` owns the macOS producer matrix plus Linux and Windows no-change guarantees.
- `compatibility-spec.md` owns additive-only compatibility, unknown-key preservation, and reader tolerance of the new subtree.
- The implemented Linux pack remains the sole owner of `host_state.platform.*` semantics and field definitions.

## Failure posture and invariants

Failure posture:
- Metadata collection, metadata read, metadata parse, temp-file write, and replace failures remain warning-only when the install itself succeeds.
- This feature stays fail-open for metadata-side errors and preserves the existing install success result.
- This feature does not introduce a new fatal branch for `sw_vers -productVersion`, `sw_vers -buildVersion`, `uname -m`, metadata parse, or metadata write failures.

Cross-cutting invariants:
- `install_state.json` remains the single canonical installer metadata file for this feature.
- `schema_version` remains integer `1`.
- macOS data lands under `host_state.os.*`.
- Linux `host_state.platform.*` remains unchanged and stays under the existing implemented Linux authority.
- Existing cleanup fields `host_state.group` and `host_state.linger` remain intact across rewrites.
- Unknown top-level keys and unknown `host_state` sibling keys remain intact across rewrites.
- Writes use a same-directory temp file and a replace step after a complete JSON document exists at the temp path.
- In-place truncation of the canonical file remains banned.
- Parse failure seeds a fresh `schema_version = 1` document and emits a warning.
- Temp-file write failure or replace failure preserves the prior canonical file when one exists, emits a warning, and removes the temp file when removal succeeds.

Security and redaction posture:
- Persisted macOS data is limited to `host_state.os.family`, `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch`.
- This feature does not persist hostnames, serial numbers, or broad host inventories.
- This feature introduces no new structured telemetry field and no new policy input surface.
- Warning diagnostics stay aligned with the existing redaction posture and do not widen the persisted field set.

## Exit-code posture

- Exit-code taxonomy authority: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work does not appear to require new exit codes.
- This work does not justify an exit-code override based on the ADR, spec manifest, or impact map.
- A downstream `contract.md` must restate that metadata-side failures remain warning-only on an otherwise successful install and must keep the taxonomy aligned with the canonical standard.

## Cross-cutting seams and constraints

Pack-wide alignment constraints:
- Single-file rule: `install_state.json` remains the only canonical metadata file touched by this feature.
- Path rule: the file path remains `<effective_prefix>/install_state.json` with the default alias `~/.substrate/install_state.json`.
- Schema rule: the new macOS subtree is `host_state.os.*`; Linux `host_state.platform.*` is preserved without rename or repurpose.
- Compatibility rule: the feature is additive-only and preserves unknown keys plus the existing Linux and cleanup subtrees.
- Cleanup boundary: uninstall readers remain Linux-only for cleanup actions; macOS `host_state.os.*` is diagnostic-only data.
- Validation boundary: macOS producer assertions belong in the macOS parity fixture, while shared additive-merge and no-change assertions belong in the cross-installer install-state harness.

Planning baselines carried forward from `impact_map.md`:
- Producer-scope baseline: macOS producer coverage includes `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- Partial-capture baseline: write `host_state.os.family = "macos"` and add every successfully collected leaf under `host_state.os.*`.
- Validation baseline: `tests/mac/installer_parity_fixture.sh` is the primary macOS producer harness and `tests/installers/install_state_smoke.sh` is the secondary shared merge and no-change harness.

These baselines remain draft pack guidance until downstream specs and `decision_register.md` restate them in their final authorities.

## Follow-ups for downstream FSE planning and decomposition

1. Ratify the producer-scope baseline in `decision_register.md` and align `contract.md`, `filesystem-semantics-spec.md`, and `platform-parity-spec.md` to the same installer-entrypoint set.
2. Ratify the partial-capture baseline in `decision_register.md` and encode the exact leaf-presence contract in `install-state-schema-spec.md`, including the object-presence rule for `host_state.os`.
3. Ratify the validation split in `decision_register.md` and map each assertion to `tests/mac/installer_parity_fixture.sh`, `tests/installers/install_state_smoke.sh`, and `manual_testing_playbook.md`.
4. Reconcile ADR-0039 validation wording with the selected producer scope so the ADR and downstream specs point at the same installer matrix.
5. Reconcile `docs/INSTALLATION.md` so operator docs state that macOS writes diagnostic-only `host_state.os.*` metadata while Windows remains on the no-write side for this feature.
6. Restate the existing upstream `effective_prefix` resolution rule in `contract.md` so downstream docs do not drift on CLI/config/env precedence even though this feature adds no new precedence inputs.

## Draft seam and slice-candidate skeleton (pre-planning only)

draft; may split/merge during downstream FSE planning or decomposition

Candidate prefix (draft): PMHOS

### Candidate 1

- `candidate_id`: `PMHOS-01`
- `name`: `persist-host-state-os-schema-and-merge`
- `intent`: Freeze the `host_state.os.*` field contract, keep `schema_version = 1`, and preserve the Linux and cleanup subtrees plus unknown keys during rewrites.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `tests/installers/install_state_smoke.sh`

### Candidate 2

- `candidate_id`: `PMHOS-02`
- `name`: `macos-writer-flow-and-warning-only-degradation`
- `intent`: Freeze the macOS writer entrypoint set, the same-directory temp-file and replace sequence, and the warning-only recovery paths for collection, parse, and write failures.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`

### Candidate 3

- `candidate_id`: `PMHOS-03`
- `name`: `validation-and-doc-reconciliation`
- `intent`: Freeze the automated and manual evidence map, update operator docs, and verify the Linux and Windows no-change boundaries plus the Linux-only cleanup boundary.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`
  - `docs/INSTALLATION.md`
  - `tests/mac/installer_parity_fixture.sh`
  - `tests/installers/install_state_smoke.sh`

Downstream note:
- `ci_checkpoint_plan.md` may use this candidate list when proposing checkpoint groups.
- `workstream_triage.md` may recommend edits to this skeleton, but it does not own this file.
