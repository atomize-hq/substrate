# persist-macos-host-os-install-state — spec manifest

This file enumerates every contract, schema, filesystem, compatibility, platform, and validation surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- Reference-only implementation and validation surfaces reviewed during spec determination:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `tests/mac/installer_parity_fixture.sh`
  - `tests/installers/install_state_smoke.sh`
  - `docs/INSTALLATION.md`

## Slice IDs (canonical)

Slice IDs follow `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`.

- Slice prefix: `PMHOIS`
- `PMHOIS0` — freeze the hosted macOS installer contract, schema, compatibility rules, and open decisions
- `PMHOIS1` — implement hosted macOS `install_state.json` write/update behavior and file-operation semantics
- `PMHOIS2` — lock automated validation, manual evidence, and operator-doc conformance

Canonical slice spec paths:
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS0/PMHOIS0-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`

## Required spec documents (authoritative)

This ADR requires one feature contract doc, one stable file-schema doc, one filesystem-semantics doc, one platform-parity doc, one compatibility doc, one decision register, one impact map, one execution plan, and three canonical slice specs.

No separate protocol, policy, telemetry, or env-vars spec is selected.
- This ADR introduces no host↔agent RPC, HTTP, WebSocket, named-pipe, IPC, or shim↔shell wire contract delta.
- This ADR introduces no policy broker rule, approval-cache, or enforcement-mode delta.
- This ADR introduces no new trace field, log field, metric field, or redaction rule.
- This ADR introduces no new environment variable and no changed precedence rule for an existing environment variable.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
  - Owns:
    - the exact required-doc set for this feature
    - the surface-to-doc ownership map
    - the canonical slice IDs and canonical slice-spec paths
    - the follow-ups that block quality-gate acceptance
  - Must define:
    - one surface-complete coverage matrix with exactly one owner per surface
    - the explicit list of unselected doc classes for this ADR
    - the explicit scope statement that this pack defines hosted macOS installer behavior only

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
  - Owns:
    - the exact create/edit touch set for this feature
    - the cascading implications and cross-pack conflicts
  - Must define:
    - the exact implementation path changes under `scripts/substrate/install-substrate.sh`
    - the exact automated-validation path changes under `tests/mac/installer_parity_fixture.sh` and any additional selected harness path
    - the exact operator-doc path change under `docs/INSTALLATION.md`
    - the explicit no-expansion statement that `scripts/substrate/dev-install-substrate.sh` stays outside this feature unless ADR-0039 is amended

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/plan.md`
  - Owns:
    - slice execution order
    - required automated-validation commands
    - required manual validation evidence
  - Must define:
    - the slice order `PMHOIS0` → `PMHOIS1` → `PMHOIS2`
    - the exact command set for the selected macOS automated harness and any Linux guardrail checks that confirm no Linux regression
    - the exact manual validation flow for hosted macOS install with the normal branch and the `--no-world` branch

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/tasks.json`
  - Owns:
    - the triad task graph and automation metadata for this pack
  - Must define:
    - triad task IDs and dependencies for `PMHOIS0`, `PMHOIS1`, and `PMHOIS2`
    - references to the canonical slice spec paths under `slices/PMHOIS*/`
    - acceptance-criteria traceability to the slice specs

### Feature contract, schema, compatibility, parity, and decisions (required)

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - Owns:
    - the hosted macOS installer user-facing contract for this feature
    - canonical metadata file identity and path semantics
    - CLI and exit-code deltas
    - future-consumer read semantics
  - Must define:
    - the exact scope boundary:
      - in scope: hosted installer path in `scripts/substrate/install-substrate.sh`
      - in scope: successful macOS install with world provisioning enabled
      - in scope: successful macOS install with `--no-world`
      - in scope: `--dry-run` no-write semantics because the same command path owns metadata persistence
      - out of scope: `scripts/substrate/dev-install-substrate.sh`
    - the exact CLI delta:
      - this ADR introduces no new command
      - this ADR introduces no new flag
      - this ADR introduces no new environment variable
    - the exact canonical metadata path rule:
      - canonical file name: `install_state.json`
      - canonical path: `<effective_prefix>/install_state.json`
      - default macOS path: `~/.substrate/install_state.json`
    - the exact producer rule:
      - a successful hosted macOS install creates or updates the canonical file even when no Linux-only host-state events exist
      - a successful hosted macOS install with `--no-world` follows the same metadata write contract
      - hosted `--dry-run` performs no metadata write
    - the exact exit-code posture:
      - metadata collection, parse, write, and replace failures never change the exit status of an otherwise successful install
      - this ADR introduces no new nonzero exit-code meaning
    - the exact future-consumer read rule:
      - later user-facing guidance prefers persisted `host_state.os.*` when present
      - later user-facing guidance tolerates missing values and falls back to runtime detection
    - the exact no-telemetry statement:
      - this ADR adds no trace field and no log field

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - Owns:
    - the `install_state.json` serialized schema touched by this ADR
    - the exact `host_state.os.*` field contract
    - the top-level metadata invariants for files created or rewritten by this feature
  - Must define:
    - the exact top-level fields and types:
      - `schema_version` as integer `1`
      - `created_at` as a persisted UTC timestamp string
      - `updated_at` as a persisted UTC timestamp string
      - `host_state` as the root object for persisted host metadata
    - the exact macOS block and field contract:
      - `host_state.os`
      - `host_state.os.family`
      - `host_state.os.product_version`
      - `host_state.os.build_version`
      - `host_state.os.arch`
    - the exact source-command mapping:
      - `host_state.os.family` is the literal string `macos`
      - `host_state.os.product_version` comes from `sw_vers -productVersion`
      - `host_state.os.build_version` comes from `sw_vers -buildVersion`
      - `host_state.os.arch` comes from `uname -m`
    - the exact normalization rule for command output strings
    - the exact presence and invalid-state rules for `host_state.os` and each child field after the decision register selects partial-emission behavior
    - authoritative JSON examples for:
      - a fresh successful hosted macOS install
      - an upgraded existing file with preserved legacy content
      - the selected partial-emission outcome

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - Owns:
    - the on-disk write/update semantics for `install_state.json`
    - temp-file naming, replace ordering, file mode, and cleanup behavior
    - parse/write/replace failure handling at the file-operation layer
  - Must define:
    - the exact temp-file path: `<effective_prefix>/install_state.json.tmp`
    - the exact write ordering rule:
      - render complete JSON to the temp file first
      - replace the canonical file in one same-directory `mv` step
      - no in-place truncation
    - the exact directory-creation rule for the canonical file parent directory
    - the exact file-mode rule:
      - successful writes apply mode `0644` to the canonical file
    - the exact parse-failure rule:
      - emit warning-only diagnostics
      - seed a fresh `schema_version = 1` base document for the rewrite path
    - the exact unsupported-schema rule:
      - emit warning-only diagnostics
      - rebuild from a fresh `schema_version = 1` base document
    - the exact write-failure and replace-failure rule:
      - emit warning-only diagnostics
      - leave the prior canonical file unchanged when the replace step fails
      - remove the temp file when cleanup succeeds
    - the exact dry-run rule:
      - emit the dry-run skip diagnostic
      - create no metadata file and no temp file

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - Owns:
    - per-platform guarantees and non-guarantees for this feature
    - platform validation evidence boundaries
  - Must define:
    - macOS guarantees:
      - successful hosted macOS installs persist the selected `host_state.os.*` contract
      - successful hosted macOS installs with `--no-world` persist the same `host_state.os.*` contract
      - hosted macOS `--dry-run` performs no metadata write
    - Linux guarantees:
      - this ADR introduces no new `host_state.os.*` writes
      - existing Linux `host_state.platform.*` behavior remains external to this pack
    - Windows guarantees:
      - this ADR introduces no new `host_state.os.*` writes
      - this pack defines no Windows persisted-host-state schema
    - the exact validation evidence required for each declared behavior platform in `tasks.json`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - Owns:
    - the additive-only compatibility policy for the extended file
    - preservation rules for legacy content and unknown keys
    - rollout and no-migration boundaries
  - Must define:
    - the exact version policy:
      - `schema_version` remains `1`
      - this ADR introduces no schema-version bump
    - the exact additive-only rule:
      - no existing key is renamed
      - no existing key is removed
    - the exact preservation rules for existing content:
      - preserve unknown top-level keys
      - preserve unknown nested keys under `host_state`
      - preserve existing `host_state.group` content
      - preserve existing `host_state.linger` content
      - preserve existing `host_state.platform` content
    - the exact rollout boundary:
      - no migration or backfill occurs outside successful hosted macOS installs

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`
  - Owns:
    - A/B decisions that ADR-0039 leaves open and that must close before quality gate
  - Must define:
    - one decision for fresh-file scaffolding:
      - option A: a fresh macOS file writes only the top-level metadata fields plus `host_state.os`
      - option B: a fresh macOS file also seeds empty legacy `host_state.group` and `host_state.linger` containers
    - one decision for partial-emission behavior:
      - option A: write `host_state.os` with every successfully collected field and omit only failed fields
      - option B: skip the entire `host_state.os` block when any required source command fails
    - one decision for the primary automated validation vehicle:
      - option A: extend `tests/mac/installer_parity_fixture.sh`
      - option B: define a different named macOS harness path and record it in `plan.md`

### Slice specs (required)

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS0/PMHOIS0-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for locking the contract, schema, compatibility rules, and decision outcomes
  - Must define:
    - acceptance criteria for the exact field set, file path, compatibility rules, and selected A/B decisions
    - the explicit linkage boundary to `contract.md`, `install-state-schema-spec.md`, `filesystem-semantics-spec.md`, `platform-parity-spec.md`, `compatibility-spec.md`, and `decision_register.md`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for hosted-installer implementation
  - Must define:
    - the exact success branches that write or update the file
    - the exact no-write branches
    - the exact acceptance criteria for temp-file replace, warning-only degradation, file mode `0644`, and compatibility preservation

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for validation and operator-facing conformance evidence
  - Must define:
    - the exact automated assertions required by the selected macOS validation harness
    - the exact manual validation evidence for hosted macOS install and hosted macOS `--no-world`
    - the exact `docs/INSTALLATION.md` conformance checks that replace the current statement that macOS does not write `install_state.json`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Required-doc set and canonical slice paths | `pre-planning/spec_manifest.md` | exact filenames, unselected doc classes, slice IDs |
| Exact touched implementation, test, and docs paths | `pre-planning/impact_map.md` | installer path, selected harness path, `docs/INSTALLATION.md`, out-of-scope neighbor paths |
| Slice order and validation commands | `plan.md` | execution order, automated commands, manual evidence |
| Triad task graph and task IDs | `tasks.json` | `PMHOIS0/1/2` task IDs, dependencies, references |
| Hosted-installer scope boundary | `contract.md` | hosted installer only, `--no-world` in scope, `--dry-run` no-write, dev installer out of scope |
| Canonical metadata file name and path | `contract.md` | `install_state.json`, `<effective_prefix>/install_state.json`, default macOS path |
| CLI and environment-variable delta | `contract.md` | no new command, no new flag, no new env var |
| Exit-code posture | `contract.md` | taxonomy reference and warning-only success preservation |
| Future-consumer read semantics | `contract.md` | persisted-value preference, missing-value tolerance, runtime fallback |
| No-telemetry delta | `contract.md` | no new trace field and no new log field |
| Top-level file shape | `install-state-schema-spec.md` | `schema_version`, `created_at`, `updated_at`, `host_state` |
| `host_state.os.family` | `install-state-schema-spec.md` | type, literal value `macos`, presence rule |
| `host_state.os.product_version` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| `host_state.os.build_version` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| `host_state.os.arch` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| Fresh-file scaffolding for legacy `host_state.group` and `host_state.linger` containers | `decision_register.md` | one A/B selection and downstream doc links |
| Partial `host_state.os` emission when a source command fails | `decision_register.md` | one A/B selection and downstream doc links |
| Primary automated validation vehicle | `decision_register.md` | one A/B selection linked from `plan.md` and `PMHOIS2-spec.md` |
| Temp-file path and same-directory replace rule | `filesystem-semantics-spec.md` | temp path, replace ordering, no in-place truncation |
| Parent-directory creation rule | `filesystem-semantics-spec.md` | exact directory-creation behavior before writing |
| File-mode rule | `filesystem-semantics-spec.md` | mode `0644` after successful replace |
| Parse-failure rebuild behavior | `filesystem-semantics-spec.md` | warning-only diagnostic, fresh base-document rule |
| Unsupported-schema rebuild behavior | `filesystem-semantics-spec.md` | warning-only diagnostic, schema-version mismatch reset |
| Write-failure and replace-failure degradation | `filesystem-semantics-spec.md` | warning-only behavior, prior-file preservation, temp cleanup |
| Dry-run no-write behavior | `filesystem-semantics-spec.md` | no file write, no temp file, skip diagnostic |
| macOS changed behavior | `platform-parity-spec.md` | successful-write guarantee and validation evidence |
| Linux non-delta guarantee | `platform-parity-spec.md` | no new `host_state.os.*` writes |
| Windows non-delta guarantee | `platform-parity-spec.md` | no new `host_state.os.*` writes and no Windows schema claim |
| Additive-only extension policy | `compatibility-spec.md` | no rename, no removal, no schema-version bump |
| Legacy-content preservation | `compatibility-spec.md` | preserve existing `host_state.group`, `host_state.linger`, and `host_state.platform` content |
| Unknown-key preservation | `compatibility-spec.md` | preserve unknown top-level and nested keys during rewrites |
| No migration/backfill boundary | `compatibility-spec.md` | writes occur only on successful hosted macOS installs |

## Determinism checklist (must be satisfied before quality gate)

### `pre-planning/spec_manifest.md`
- Enumerate every touched surface in the coverage matrix.
- Assign exactly one owner per surface.
- Record every unresolved ADR gap in `Follow-ups`.

### `pre-planning/impact_map.md`
- Name every touched implementation file.
- Name every touched validation file.
- Name every touched operator-facing doc file.
- Record every adjacent out-of-scope path that is at risk of accidental expansion.

### `plan.md`
- State the slice order exactly once.
- State the automated validation commands exactly once.
- State the manual validation evidence exactly once.

### `tasks.json`
- Use `PMHOIS0`, `PMHOIS1`, and `PMHOIS2` consistently across task IDs, prompts, and references.
- Attach acceptance criteria to canonical slice spec paths only.
- Keep dependency order aligned to `plan.md`.

### `contract.md`
- Define all path semantics and branch semantics.
- Define the exit-code posture using the canonical taxonomy.
- Define the no-new-env-var and no-new-telemetry delta explicitly.

### `install-state-schema-spec.md`
- Define every serialized field path, type, and source mapping.
- Define the top-level timestamp behavior.
- Define invalid states and JSON examples.
- Consume the selected A/B outcomes from `decision_register.md`.

### `filesystem-semantics-spec.md`
- Define temp-file naming and placement.
- Define ordering of render, replace, cleanup, and file mode.
- Define parse, schema-mismatch, write, and replace failure behavior with warning-only outcomes.
- Define the dry-run no-write path.

### `platform-parity-spec.md`
- Define macOS guarantees.
- Define Linux and Windows non-delta guarantees.
- Define the parity evidence required for each declared behavior platform.

### `compatibility-spec.md`
- Define additive-only semantics.
- Define preservation rules for legacy fields and unknown keys.
- Define the no-migration boundary explicitly.

### `decision_register.md`
- Present each unresolved point as exactly two options.
- Select one option for each decision.
- Link each selected option back to the downstream owning spec docs.

### `slices/PMHOIS0/PMHOIS0-spec.md`
- Freeze the contract and decision baseline with acceptance criteria.
- Point downstream implementation at the authoritative docs without restating tables.

### `slices/PMHOIS1/PMHOIS1-spec.md`
- Freeze the exact hosted-installer write behavior.
- Freeze the exact filesystem behavior acceptance checks.

### `slices/PMHOIS2/PMHOIS2-spec.md`
- Freeze the exact validation assertions and manual evidence.
- Freeze the exact `docs/INSTALLATION.md` conformance check.

## Follow-ups

1. Select fresh-file scaffolding behavior for new macOS metadata files. ADR-0039 defines `host_state.os.*` and compatibility preservation but does not select whether a brand-new macOS file also seeds empty legacy `host_state.group` and `host_state.linger` containers.
2. Select partial-emission behavior for `host_state.os`. ADR-0039 names best-effort collection and future-consumer tolerance but does not select whether the writer omits only failed child fields or skips the entire `host_state.os` block when any required source command fails.
3. Select the primary macOS automated validation harness. ADR-0039 requires smoke coverage and manual validation but does not select the canonical harness path for the new assertions.
