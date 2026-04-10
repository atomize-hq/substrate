# persist-macos-host-os-install-state — spec manifest (pre-planning)

This file enumerates every contract, schema, path, compatibility, and validation surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- Reference-only upstream and touched implementation/docs:
  - `scripts/substrate/install-substrate.sh`
  - `docs/INSTALLATION.md`
  - `tests/mac/installer_parity_fixture.sh`
  - `tests/installers/install_state_smoke.sh`

## Slice IDs (canonical)

Slice IDs follow `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`.

- Slice prefix: `PMHOIS`
- `PMHOIS0` — lock the persisted macOS metadata contract and open decisions
- `PMHOIS1` — implement hosted macOS writer behavior and filesystem semantics
- `PMHOIS2` — lock validation coverage and operator-facing conformance evidence

Canonical slice spec paths:
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS0/PMHOIS0-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`

## Required spec documents (authoritative)

This ADR requires one user-facing contract doc, one schema doc, one filesystem-semantics doc, one platform-parity doc, one compatibility doc, one decision register, one impact map, one execution plan, and three canonical slice specs.

No separate protocol, policy, telemetry, or env-vars spec is selected.
- This ADR introduces no host-agent protocol, HTTP, WebSocket, named-pipe, or IPC delta.
- This ADR introduces no policy broker rule or approval-cache delta.
- This ADR introduces no new trace field, no new log field, and no redaction-rule delta.
- This ADR introduces no new environment variable. Existing installer environment variables remain external context, not a feature-local contract change.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` (this file)
  - Owns:
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the canonical slice IDs and slice spec paths
    - the follow-ups required before quality gate
  - Must define:
    - one surface-complete coverage matrix with exactly one owner per surface
    - the explicit list of unselected doc classes for this ADR
    - the explicit statement that this pack defines hosted macOS installer behavior only

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
  - Owns:
    - the exact create/edit touch set for this feature
    - the cascading implications and cross-pack conflicts
  - Must define:
    - the exact touched implementation paths under `scripts/substrate/`
    - the exact touched validation paths under `tests/`
    - the exact touched operator-doc path under `docs/INSTALLATION.md`
    - the explicit statement that `scripts/substrate/dev-install-substrate.sh` stays out of scope unless the ADR is amended

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/plan.md`
  - Owns:
    - slice execution order
    - required validation commands and evidence collection
  - Must define:
    - the slice order `PMHOIS0` → `PMHOIS1` → `PMHOIS2`
    - the exact command set for automated validation once the decision register selects the primary macOS harness
    - the exact manual validation flow for `scripts/substrate/install-substrate.sh` on macOS with the normal path and the `--no-world` path

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
    - the user-facing installer contract for this feature
    - canonical file identity and path semantics
    - CLI and exit-code deltas
    - future-consumer read semantics
  - Must define:
    - the exact scope boundary:
      - hosted macOS installer path only: `scripts/substrate/install-substrate.sh`
      - normal install and `--no-world` are in scope
      - `scripts/substrate/dev-install-substrate.sh` is out of scope for this ADR
    - the exact CLI delta:
      - this ADR introduces no new commands
      - this ADR introduces no new flags
      - this ADR introduces no new environment variable
    - the exact canonical metadata path rule:
      - canonical file name: `install_state.json`
      - canonical path: `<effective_prefix>/install_state.json`
      - default path on macOS: `~/.substrate/install_state.json`
    - the exact producer rule:
      - a successful hosted macOS install creates or updates the canonical file even when no Linux-only host-state events exist
      - hosted `--dry-run` is a no-write branch
    - the exact exit-code posture:
      - metadata collection and persistence failures do not change the exit status of an otherwise successful install
      - this ADR introduces no new nonzero exit-code meaning
    - the exact future-consumer read rule:
      - later user-facing guidance prefers persisted `host_state.os.*` when present
      - later user-facing guidance tolerates missing values and falls back to runtime detection
    - the exact no-telemetry delta statement:
      - this ADR adds no trace field and no log field

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - Owns:
    - the `install_state.json` serialized schema touched by this ADR
    - the exact `host_state.os.*` field contract
    - top-level timestamp and version invariants for files created or rewritten by this feature
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
    - the exact value normalization rule for command output strings
    - the exact presence and invalid-state rules for `host_state.os` and its fields after the decision register resolves partial-emission behavior
    - authoritative JSON examples for:
      - a fresh successful macOS install
      - an upgraded existing file with preserved legacy content
      - the selected partial-emission outcome

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - Owns:
    - the on-disk write/update semantics for `install_state.json`
    - temp-file and replace behavior
    - read/parse/write failure handling at the file-operation layer
  - Must define:
    - the exact temp-file path: `<effective_prefix>/install_state.json.tmp`
    - the exact atomicity rule:
      - render complete JSON to the temp file first
      - replace the canonical file in one same-directory replace step
      - no in-place truncation
    - the exact directory-creation rule for the canonical file parent directory
    - the exact parse-failure rule:
      - emit warning-only diagnostics
      - seed a fresh `schema_version = 1` base document for the rewrite path
    - the exact write-failure and replace-failure rule:
      - emit warning-only diagnostics
      - leave the prior canonical file unchanged when the replace step fails
      - remove the temp file when cleanup succeeds

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - Owns:
    - per-platform guarantees and non-guarantees for this feature
    - platform validation evidence boundaries
  - Must define:
    - macOS guarantees:
      - successful hosted macOS installs persist the selected `host_state.os.*` contract
      - `--no-world` remains eligible for the same metadata persistence behavior
    - Linux guarantees:
      - this ADR introduces no new `host_state.os.*` writes
      - existing Linux `host_state.platform.*` behavior remains external to this pack
    - Windows guarantees:
      - this ADR introduces no new `host_state.os.*` writes
      - this pack does not define a Windows persisted-host-state schema
    - the exact parity evidence required for each behavior platform declared in `tasks.json`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - Owns:
    - additive-only compatibility policy for the extended file
    - preservation rules for legacy content and unknown keys
    - rollout and no-migration boundaries
  - Must define:
    - the exact version policy:
      - `schema_version` remains `1`
      - this ADR does not introduce a schema-version bump
    - the exact additive-only rule:
      - no existing key is renamed
      - no existing key is removed
    - the exact preservation rules for existing content:
      - preserve unknown keys
      - preserve existing `host_state.group` content
      - preserve existing `host_state.linger` content
      - preserve existing `host_state.platform` content
    - the exact rollout boundary:
      - no migration or backfill occurs outside successful hosted macOS installs

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`
  - Owns:
    - A/B decisions that the ADR leaves open and that must be closed before quality gate
  - Must define:
    - one decision for fresh-file scaffolding:
      - option A: a fresh macOS file writes only the fields required for `host_state.os` plus the existing top-level metadata fields
      - option B: a fresh macOS file also seeds empty legacy `host_state.group` and `host_state.linger` containers
    - one decision for partial-emission behavior:
      - option A: write `host_state.os` with every successfully collected field and omit only failed fields
      - option B: skip the entire `host_state.os` block when any required source command fails
    - one decision for the primary automated validation vehicle:
      - option A: extend `tests/mac/installer_parity_fixture.sh`
      - option B: add macOS coverage through another named harness path recorded in `plan.md`

### Slice specs (required)

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS0/PMHOIS0-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for locking the contract, schema, compatibility rules, and decision outcomes
  - Must define:
    - acceptance criteria for the exact field set, file path, compatibility rules, and chosen A/B decisions
    - the explicit linkage boundary to `contract.md`, `install-state-schema-spec.md`, `filesystem-semantics-spec.md`, `platform-parity-spec.md`, `compatibility-spec.md`, and `decision_register.md`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for the hosted installer implementation
  - Must define:
    - the exact success branches that write or update the file
    - the exact no-write branches
    - the exact acceptance criteria for atomic replace, warning-only degradation, and compatibility preservation

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`
  - Owns:
    - the slice scope and acceptance criteria for validation and operator-facing conformance evidence
  - Must define:
    - the exact automated assertions required by the selected validation harness
    - the exact manual validation evidence for hosted macOS install and hosted macOS `--no-world`
    - the exact operator-doc conformance checks for `docs/INSTALLATION.md`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Required-doc set and canonical slice paths | `pre-planning/spec_manifest.md` | exact filenames, unselected doc classes, slice IDs |
| Exact touched implementation and doc paths | `pre-planning/impact_map.md` | scripts, tests, docs, and out-of-scope touched neighbors |
| Slice order and validation command list | `plan.md` | execution order, automated commands, manual evidence |
| Triad task graph and task IDs | `tasks.json` | `PMHOIS0/1/2` task IDs, dependencies, references |
| Installer scope boundary | `contract.md` | hosted installer only, `--no-world` in scope, dev installer out of scope |
| Canonical metadata file name and path | `contract.md` | `install_state.json`, `<effective_prefix>/install_state.json`, default path |
| Installer flag delta | `contract.md` | no new commands, no new flags, `--dry-run` no-write branch |
| Exit-code posture | `contract.md` | exit code taxonomy mapping and warning-only success preservation |
| Future-consumer read semantics | `contract.md` | persisted-value preference, missing-value tolerance, runtime fallback |
| Telemetry delta | `contract.md` | no new trace field and no new log field |
| Top-level file shape | `install-state-schema-spec.md` | `schema_version`, `created_at`, `updated_at`, `host_state` |
| `host_state.os.family` | `install-state-schema-spec.md` | type, literal value `macos`, presence rule |
| `host_state.os.product_version` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| `host_state.os.build_version` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| `host_state.os.arch` | `install-state-schema-spec.md` | type, source command, normalization, absence rule |
| Fresh-file scaffolding for legacy `host_state.group` / `host_state.linger` containers | `decision_register.md` | one A/B selection and downstream doc links |
| Partial `host_state.os` emission when a source command fails | `decision_register.md` | one A/B selection and downstream doc links |
| Temp-file path and same-directory replace rule | `filesystem-semantics-spec.md` | temp path, replace ordering, no in-place truncation |
| Parse-failure rebuild behavior | `filesystem-semantics-spec.md` | warning-only diagnostic, fresh base document rule |
| Write-failure and replace-failure degradation | `filesystem-semantics-spec.md` | warning-only behavior, prior-file preservation, temp cleanup |
| macOS changed behavior | `platform-parity-spec.md` | successful-write guarantee and validation evidence |
| Linux non-delta guarantee | `platform-parity-spec.md` | no new `host_state.os.*` writes |
| Windows non-delta guarantee | `platform-parity-spec.md` | no new `host_state.os.*` writes and no Windows schema claim |
| Additive-only extension policy | `compatibility-spec.md` | no rename, no removal, no schema-version bump |
| Legacy-content preservation | `compatibility-spec.md` | preserve existing `host_state.group`, `host_state.linger`, and `host_state.platform` content |
| Unknown-key preservation | `compatibility-spec.md` | preserve unknown keys during rewrites |
| No migration/backfill boundary | `compatibility-spec.md` | writes occur only on successful hosted macOS installs |
| Primary automated validation vehicle | `decision_register.md` | one A/B selection linked from `plan.md` and `PMHOIS2-spec.md` |

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
- Define top-level timestamp behavior.
- Define invalid states and JSON examples.
- Consume the selected A/B outcomes from `decision_register.md`.

### `filesystem-semantics-spec.md`
- Define temp-file naming and placement.
- Define ordering of render, replace, and cleanup.
- Define parse-failure and write-failure behavior with warning-only outcomes.

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

1. Resolve fresh-file scaffolding for macOS. ADR-0039 defines `host_state.os.*` and preservation rules for legacy content but does not state whether a newly created macOS file also seeds empty `host_state.group` and `host_state.linger` containers.
2. Resolve partial-emission behavior for `host_state.os`. ADR-0039 allows missing or partial values and does not lock whether the writer omits only failed fields or skips the entire block when one source command fails.
3. Resolve the primary automated validation vehicle. ADR-0039 requires smoke coverage and manual validation but does not select the canonical macOS harness path.
4. Keep Windows alignment rationale-only in this pack. No repo artifact surfaced a Windows `host_state.os.*` contract, so `platform-parity-spec.md` must define only a Windows non-delta guarantee.
