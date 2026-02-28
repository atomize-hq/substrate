# stabilize-dev-install-helper-discovery — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`.

Each entry includes:
- path
- what surfaces it owns (authoritative)
- what it links to (non-authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns:
    - The required-doc set for this feature.
    - The surface inventory and surface→doc ownership mapping.
  - Links to:
    - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
    - `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md`
  - Owns:
    - Touch set inventory for this feature (scripts + crates + docs), including cross-platform implications.
  - Links to:
    - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md`
  - Owns:
    - Execution sequencing for the slices in this feature (what ships first, gates, and required validation evidence).
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json`
  - Owns:
    - Triad task graph (code/test/integ) for each slice, including acceptance criteria references.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`
  - Owns (single source of truth for user-facing contract wording for this feature):
    - CLI contract subset for `substrate world enable` that this feature relies on/changes:
      - helper discovery order
      - fail-closed behavior when helper cannot be found
    - Dev-install helper staging contract under `$SUBSTRATE_HOME/scripts/substrate/…`:
      - exact paths created/updated
      - overwrite policy when destination paths already exist
      - staging mechanism contract (copy vs symlink) as user-visible behavior (not implementation detail)
    - Dev-uninstall cleanup contract for the staged helpers:
      - ownership guard (what qualifies as “managed by dev-install”)
      - protected-path invariants (what must never be deleted)
    - Exit codes for feature-relevant failure modes (taxonomy reference; no overrides unless explicitly declared).
    - Platform guarantees for this feature (Linux/macOS supported; Windows behavior is explicitly “unsupported/no-change”).
  - Links to:
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`
  - Owns:
    - DR-0001: helper staging mechanism decision (exactly two options; one selected).
    - DR-0002: uninstall ownership guard decision (exactly two options; one selected).
    - DR-0003: overwrite policy decision when `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` already exists (exactly two options; one selected).
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` (contract is authoritative; decision register records the A/B decisions and the selection)

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md` — Slice SDIHD0 (“stage helpers under `$SUBSTRATE_HOME`”)
  - Owns:
    - Slice SDIHD0 scope, acceptance criteria, and required evidence for “dev-install stages helpers under `$SUBSTRATE_HOME/scripts/substrate/…`”.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` (authoritative contract)
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md` — Slice SDIHD1 (“uninstall cleanup for staged helpers”)
  - Owns:
    - Slice SDIHD1 scope, acceptance criteria, and required evidence for “dev-uninstall removes only dev-managed staged helpers under `$SUBSTRATE_HOME/scripts/substrate/…`”.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` (authoritative contract)
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/manual_testing_playbook.md`
  - Owns:
    - Deterministic manual validation steps and expected observable outcomes for this feature.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/linux-smoke.sh`
  - Owns:
    - Deterministic Linux smoke validation steps for this feature.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/macos-smoke.sh`
  - Owns:
    - Deterministic macOS smoke validation steps for this feature.
  - Links to:
    - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`

## Coverage matrix (surface → authoritative doc)

Every surface that the ADR touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `substrate world enable` helper discovery order | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | exact ordered list of candidates: (1) `<inferred version dir>/scripts/substrate/world-enable.sh`, then (2) `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` |
| Dev-install binary link: `$SUBSTRATE_HOME/bin/substrate -> <repo>/target/<profile>/substrate` (existing dev behavior) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | explicitly state “no change”; define the observable invariant that the dev install keeps `$SUBSTRATE_HOME/bin/substrate` pointing at the repo build output |
| `<inferred version dir>` semantics for dev installs | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | how the “inferred version dir” is derived when `$SUBSTRATE_HOME/bin/substrate` is a symlink into `<repo>/target/<profile>/substrate` (including that the inferred version dir is `<repo>/target/`) |
| Fail-closed posture when helper not found | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | error condition, exit code category (taxonomy), and deterministic user remediation instruction |
| Helper script path: `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | required existence post dev-install; required file type (regular file or symlink) and executable bit; update/overwrite rules |
| Helper script path: `$SUBSTRATE_HOME/scripts/substrate/install-substrate.sh` | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | required existence post dev-install; required file type and executable bit; update/overwrite rules |
| Helper script path: `<repo>/target/scripts/substrate/world-enable.sh` (existing dev helper bridge) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | explicitly state “no change” and that this path may be absent after `cargo clean` (and that the `$SUBSTRATE_HOME` fallback must still work) |
| Helper script path: `<repo>/target/scripts/substrate/install-substrate.sh` (existing dev helper bridge dependency) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | explicitly state “no change” and that this path may be absent after `cargo clean` |
| `scripts/substrate/dev-install-substrate.sh` helper staging behavior | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | exact list of helpers staged under `$SUBSTRATE_HOME/scripts/substrate/`; staging mechanism outcome (copy vs symlink); idempotency; failure modes + exit codes |
| `scripts/substrate/dev-uninstall-substrate.sh` staged-helper cleanup behavior | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | exact list of helpers eligible for removal; ownership guard algorithm; “never delete user-managed” invariant; failure modes + exit codes |
| Overwrite policy for `$SUBSTRATE_HOME/scripts/substrate/*` | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | deterministic rule for “destination exists” (including the user-visible behavior when the file exists but is not dev-managed) |
| Protected paths / invariants | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | what paths may be created/modified/removed by dev-install/dev-uninstall; explicit prohibition on deleting non-dev-managed files under `$SUBSTRATE_HOME/scripts/substrate/…` |
| Environment variable: `SUBSTRATE_HOME` (as used by this feature) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | type (path), default when unset, and precedence relative to `dev-install-substrate.sh --prefix` / `dev-uninstall-substrate.sh --prefix` (if both exist) |
| Exit code taxonomy (no overrides) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | explicit reference to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` and mapping for each feature-relevant failure mode |
| Config files/paths/precedence/schema | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | explicitly state “no changes in this ADR/feature” (so future edits don’t silently drift into scope) |
| Platform guarantees (Linux/macOS supported; Windows unsupported) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` | exact platform behavior: supported vs unsupported, and how unsupported is detected/communicated (including exit code category) |
| Decision: DR-0001 helper staging mechanism | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` | exactly two options (A/B), one selection, and a link to the corresponding contract section in `contract.md` |
| Decision: DR-0002 uninstall ownership guard | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` | exactly two options (A/B), one selection, and a link to the corresponding contract section in `contract.md` |
| Decision: DR-0003 overwrite policy | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` | exactly two options (A/B), one selection, and a link to the corresponding contract section in `contract.md` |
| Slice SDIHD0 acceptance criteria | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md` | concrete acceptance criteria + evidence commands for staging helpers under `$SUBSTRATE_HOME/scripts/substrate/…` and surviving `cargo clean` |
| Slice SDIHD1 acceptance criteria | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md` | concrete acceptance criteria + evidence commands for uninstall cleanup safety (managed-only removal) |
| Manual validation steps | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/manual_testing_playbook.md` | exact commands, required observations, and expected exit codes for each step |
| Smoke validation (Linux) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/linux-smoke.sh` | exact commands + assertions (non-interactive) for Linux |
| Smoke validation (macOS) | `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/macos-smoke.sh` | exact commands + assertions (non-interactive) for macOS |

## Determinism checklist (per document; must be satisfied before quality gate)

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`
  - Must define the exact helper discovery order for `substrate world enable` and the exact failure behavior when helper is missing in all candidate locations.
  - Must define the exact `$SUBSTRATE_HOME/scripts/substrate/…` layout required post dev-install, including:
    - directory and file paths
    - file types (copy vs symlink) as a user-visible contract
    - executable-bit requirements for the staged helper scripts
  - Must explicitly state config behavior is unchanged by this feature (files, paths, precedence, schema).
  - Must define the exact overwrite policy when staging destinations already exist (including the behavior when the destination exists but is not dev-managed).
  - Must define the exact dev-uninstall eligibility set (which files may be removed) and the exact ownership-guard algorithm used to decide “managed by dev-install”.
  - Must define exit code mapping for each feature-relevant failure mode using `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (no overrides unless explicitly declared).
  - Must define platform support/unsupported behavior for Linux/macOS/Windows for this feature.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md`
  - Must include DR-0001, DR-0002, DR-0003 as enumerated in the ADR.
  - Each DR must have exactly two options (A/B) and exactly one selected option.
  - Each selected option must link to the relevant normative behavior section in `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md`.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD0/SDIHD0-spec.md`
  - Must scope the slice to “stage helpers under `$SUBSTRATE_HOME/scripts/substrate/…` during dev install”.
  - Must declare acceptance criteria that are runnable and OS-scoped (Linux/macOS) and that prove:
    - after dev-install, `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` exists
    - after `cargo clean` (removing `<repo>/target/scripts/…`), `substrate world enable` still locates the helper via `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`
  - Must link to `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` for authoritative contract wording.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/SDIHD1/SDIHD1-spec.md`
  - Must scope the slice to “dev-uninstall cleanup for staged helpers under `$SUBSTRATE_HOME/scripts/substrate/…`”.
  - Must declare acceptance criteria that prove:
    - dev-uninstall removes helpers only when they are dev-managed
    - dev-uninstall does not delete user-managed files under `$SUBSTRATE_HOME/scripts/substrate/…`
    - failure posture is deterministic when deletion is refused (exit code + message category per taxonomy)
  - Must link to `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` for authoritative contract wording.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md`
  - Must define slice sequencing (SDIHD0 then SDIHD1) and the exact validation evidence required before merging each slice (tests, manual playbook, smoke scripts).
  - Must state the exact files expected to change (touch set summary) and the non-goals from the ADR as explicit constraints.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json`
  - Must define the full triad task graph for SDIHD0 and SDIHD1 using the canonical slice IDs (`SDIHD0`, `SDIHD1`) and automation metadata.
  - Must ensure each task’s acceptance criteria points at the relevant slice spec and does not embed conflicting contract text.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md`
  - Must enumerate the touch set implied by the ADR (at minimum):
    - `scripts/substrate/dev-install-substrate.sh`
    - `scripts/substrate/dev-uninstall-substrate.sh`
    - `crates/shell/src/builtins/world_enable/runner/paths.rs` (reference-only; change is not required by ADR but behavior must be validated)
  - Must enumerate cross-platform implications for Linux/macOS/Windows as stated in the ADR.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/manual_testing_playbook.md`
  - Must include deterministic commands and expected observations for the ADR’s manual flow:
    - dev-install
    - `cargo clean` + rebuild
    - `substrate world enable --dry-run` helper resolution via `$SUBSTRATE_HOME/scripts/substrate/…`
    - dev-uninstall safe cleanup

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/linux-smoke.sh`
  - Must define a non-interactive smoke that exercises the helper staging + discovery behavior on Linux.

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/smoke/macos-smoke.sh`
  - Must define a non-interactive smoke that exercises the helper staging + discovery behavior on macOS.

## Follow-ups

- ADR path reconciliation: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md` references the feature directory as `docs/project_management/packs/draft/dev-install-helper-discovery/`, but this Planning Pack directory is `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`. One path must be selected and used consistently across the ADR and Planning Pack artifacts.
- Slice ID reconciliation: the ADR’s “Slice Decomposition” uses generic IDs (`C0`, `C1`). Planning artifacts for this feature must use feature-derived slice IDs per `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md` (this manifest selects `SDIHD0`, `SDIHD1`) and must include an explicit mapping from ADR slice names to these canonical IDs.
- Decision completeness: the ADR enumerates DR-0001..DR-0003 but does not include the A/B options or a selection; these must be authored in `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` before execution begins.
- Platform evidence mismatch risk: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json` currently requires Windows behavior evidence, but the ADR states `substrate world enable` is unsupported on Windows. Planning artifacts must define the deterministic Windows expectation for this feature (e.g., “no change; remains unsupported”) and how it is validated (or explicitly mark Windows validation as N/A for this feature).
