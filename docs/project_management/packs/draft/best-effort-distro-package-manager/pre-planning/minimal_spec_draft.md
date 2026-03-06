**PRE‑PLANNING ONLY — this document is not execution-ready and will be deleted or retired during full planning.**

# best-effort-distro-package-manager — minimal spec draft (alignment backbone)

Authority inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Spec manifest: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`

## Scope + authority

This draft defines only:
- Cross-cutting defaults, precedence, and invariants that all slice specs and `contract.md` align on.
- Shared vocabulary (allowed values, stable enum values, stable one-liner shape).
- Follow-ups that must be resolved during full planning.

This draft does not define:
- Slice-specific behavior, detailed schemas, or implementation tasks.
- Exact prose for non-normative logs/errors (except where the ADR requires exact text).

## Defaults + precedence

### Platform scope
- Linux: contract changes apply.
- macOS/Windows: no behavior change for this feature.

### Source-of-truth paths (behavior)
- Installer entrypoint: `scripts/substrate/install-substrate.sh`
- Distro detection input (best-effort read): `/etc/os-release`
- Hermetic test harness entrypoint (created by this feature): `tests/installers/pkg_manager_detection_test.sh`

### Config surfaces
- Persistent config files: none.
- This feature introduces no new config file surface.

### Precedence pipeline (total order)
1) `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` (source=`flag`)
2) `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>` (source=`env`)
3) `/etc/os-release` mapping using `ID` and `ID_LIKE` (source=`os_release`)
4) PATH probe fallback (source=`path_probe`)

## Failure posture + invariants

### Failure posture
- Explicit overrides (`--pkg-manager`, `PKG_MANAGER`) are validated and fail-closed:
  - Invalid override value → exit `2`.
  - Forced manager missing from `PATH` → exit `3` (no fallback).
- Autodetection is best-effort until selection is impossible:
  - Unreadable/missing `/etc/os-release` does not block selection; `<id>`/`<id_like>` render as `<unknown>` and selection continues via later steps.
  - If no supported manager can be selected, exit `4` and print actionable guidance (override usage plus a manual prereq list).

### Safety + redaction invariants
- `/etc/os-release` is parsed as plain text; implementation does not `source`/eval/execute file content.
- Distro detection performs no network calls.
- Outputs introduced by this feature do not print the full environment or full PATH.

### Output exactness invariant
- The required decision one-liner is an exact string and is emitted without prefixes or formatting wrappers.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work introduces no new exit code numbers.
- Exit codes used by this feature (per ADR-0031):
  - `0`: success / no-op by contract
  - `2`: invalid CLI usage or invalid override value
  - `3`: required dependency unavailable (forced package-manager binary missing from `PATH`)
  - `4`: not supported / missing prerequisites (no supported package manager can be selected)

## Cross-cutting seams / constraints

### Locked vocabularies (MUST match across contract/specs/tests)
- Supported package-manager set: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- Decision one-liner `source` enum: `flag|env|os_release|path_probe`.

### Required operator-facing decision one-liner (exact; stderr; exactly once)
Before installing prerequisites on Linux, the installer emits exactly one line to stderr:
- `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
- If `/etc/os-release` is missing or unreadable: `<id>` and `<id_like>` render as `<unknown>`.

### `/etc/os-release` family mapping (intent from ADR)
- Debian/Ubuntu family → `apt-get`
- Fedora/RHEL family → prefer `dnf`, fallback to `yum` when applicable
- Arch family → `pacman`
- SUSE family → `zypper`

### Deterministic PATH probe ambiguity handling
- When multiple supported managers are found in `PATH`, selection uses a fixed precedence order and emits a warning that:
  - lists the other detected supported managers, and
  - states the override mechanism (`--pkg-manager …` / `PKG_MANAGER=…`).
- The fixed precedence order is a single shared contract surface (one definition reused by implementation and tests).

### Documentation constraint (Linux-only)
- `docs/INSTALLATION.md` documents `--pkg-manager` and labels it Linux-only.

### Cross-pack non-overlap boundary (queued conflicts)
- This pack owns detection/selection/parsing and the `source` semantics for host installer pkg-manager selection.
- This pack does not persist host metadata (no `install_state.json`).
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` persists outputs and treats this pack’s `contract.md` as authoritative.

## Follow-ups for full planning

- Reconcile ADR “Related Docs” link drift (`detecting-badger/*` vs `best-effort-distro-package-manager/*`).
- Reconcile slice inventory drift:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` lists `BEDPM0`/`BEDPM1`/`BEDPM2`.
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md` Touch Set lists only `slices/BEDPM0/BEDPM0-spec.md`.
- DR-0001: Decide `/etc/os-release` parsing + canonicalization rules (quotes/whitespace, duplicate keys, case-sensitivity, `ID_LIKE` tokenization).
- DR-0002: Decide PATH probe fixed precedence order and required warning content elements.
- DR-0003: Decide the hermetic test seam for supplying fake os-release input without weakening production safety posture.
- Pin the exact prerequisite command set that remediation guidance lists for the exit-`4` case.
- Decide behavior when the `/etc/os-release` mapping matches but the mapped manager binary is missing from `PATH` (outside the Fedora `dnf`→`yum` fallback).
- Pin when the decision one-liner is emitted on the “no prereqs needed” path and when `install-substrate.sh` is sourced by `scripts/substrate/world-enable.sh`.
- Explicitly scope exit-code meanings as Linux-only vs global to avoid accidental macOS/Windows behavior changes (notably the current Windows exit `2` path).
- Reconcile hermetic-test slice ownership between `pre-planning/spec_manifest.md` and `pre-planning/impact_map.md` follow-ups.

## Draft slice skeleton (pre-planning only)

Disclaimer: “draft; may split/merge; do not wire `tasks.json` yet.”

Slice prefix (draft): `BEDPM`

- slice_id: `BEDPM0`
  - name: Detect distro and emit decision one-liner
  - intent: Stabilize `/etc/os-release` best-effort read posture and the required stderr decision one-liner shape and timing.
  - likely touch surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (DR-0001)
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

- slice_id: `BEDPM1`
  - name: Select package manager with deterministic precedence
  - intent: Stabilize the precedence pipeline (`--pkg-manager`/`PKG_MANAGER`/mapping/PATH probe), failure posture, and exit-code mapping for pkg-manager decision failures.
  - likely touch surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (DR-0002)
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`

- slice_id: `BEDPM2`
  - name: Add hermetic detection tests
  - intent: Stabilize a hermetic test harness that asserts precedence, one-liner content, warning/error content elements, and exit codes using a stubbed `PATH` and fake os-release input with no host mutation.
  - likely touch surfaces:
    - `tests/installers/pkg_manager_detection_test.sh`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (DR-0003)
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`

Downstream notes:
- CI-checkpoint prefers this slice list when populating the machine-readable slice list in `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage records any proposed edits to this slice skeleton as recommendations in `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md` (this file remains unchanged).
