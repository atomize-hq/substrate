**PRE-PLANNING ONLY: Alignment backbone draft. Delete or retire this document during full planning.**

# world-deps-apt-provisioning — minimal spec draft (pre-planning)

## Scope + authority

This document defines cross-cutting defaults, precedence, and invariants for:
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`

Authority sources for this draft:
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Pre-planning manifest: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
- Pre-planning impact map: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`

This document does not define:
- Per-slice acceptance criteria.
- Detailed schemas or algorithms (APT requirement derivation, ordering, conflict policy).
- Implementation tasks or `tasks.json` wiring.

## Defaults + precedence

Single-source-of-truth posture (planning-time):
- Operator-facing contract text for this feature lives in this pack’s `contract.md` (created during planning).
- Operator-doc updates link to `contract.md` and do not restate contract tables.

Upstream authoritative inputs (this feature does not restate their schemas/precedence):
- World-deps inventory/enabled semantics (`install.method=apt`, `install.apt[]`, merge/precedence): `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- World-agent protocol baselines + request `profile` field: `docs/WORLD.md`
- Env var registry (incl. `SUBSTRATE_WORLD_REQUEST_PROFILE`): `docs/CONFIGURATION.md`

Input precedence in this feature:
- CLI flags are the operator surface for ADR-0030 behavior.
- This feature introduces no new config keys and does not change existing world-deps config precedence rules.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` remains an advanced/testing escape hatch; provisioning-time behavior must not rely on operators setting it manually and must not create a hidden host-mutation path on Linux host-native.

## Failure posture + invariants

Fail-closed runtime invariants (ADR-0030 + impact map):
- Runtime `substrate world deps current sync` never invokes APT/dpkg.
- Runtime `substrate world deps current install` never invokes APT/dpkg.
- When APT-backed items are in scope for runtime `sync|install`, the command exits non-zero and emits remediation that includes the exact command `substrate world enable --provision-deps`.

Host mutation invariants (ADR-0030 + impact map):
- Linux host-native: provisioning-time APT is unsupported by default; Substrate does not mutate the host OS.
- Guest backends: provisioning-time OS mutation is permitted only via the explicit provisioning surface (`substrate world enable --provision-deps`).

Security/redaction posture (cross-cutting):
- New behavior must remain observable (logs/trace) and must not silently fall back to host mutation.
- Any new user-visible output invariants and any new log/trace fields are specified in `contract.md` and per-slice specs during planning.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- New exit codes: none.
- Feature-local exit mappings remain taxonomy-aligned and are owned by `contract.md` (created during planning).

## Cross-cutting seams / constraints

Slice ID invariants (from `pre-planning/spec_manifest.md`):
- Slice prefix (draft): `WDAP`
- Slice IDs: `WDAP0`, `WDAP1`

Contract seam invariants (ADR-0030):
- Provisioning is an explicit, separate surface: `substrate world enable --provision-deps [--dry-run] [--verbose]`.
- v1 provisioning derives APT requirements from the effective enabled world-deps set (no explicit item list in v1).
- Runtime world-deps operations do not perform APT/system package mutation.

Operator-doc update targets (from `pre-planning/impact_map.md`):
- `docs/reference/world/deps/README.md` (headings: `## APT packages (current limitation in hardened worlds)`, `## Commands you will use`)
- `docs/internals/world/deps.md` (headings: `## High-level flow`, `## APT installs vs hardening`)
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (headings: `#### substrate world deps current install <item_name...>`, `#### substrate world deps current sync [--dry-run] ...`)
- `docs/WORLD.md` (heading: `## 5) Agent API (over UDS)` → `POST /v1/execute` request body fields, if provisioning relies on request `profile`)
- `docs/CONFIGURATION.md` (heading: `SUBSTRATE_WORLD_REQUEST_PROFILE` row; state it is not the operator-facing provisioning workflow)
- `docs/COMMANDS.md` (heading: `### world Subcommand` row for `substrate world enable` flags)

Cross-queue constraints (from `pre-planning/impact_map.md`):
- ADR-0030 lands first; ADR-0033 extends it without redefining the base APT contract.
- The shared `--provision-deps` UX has exactly one authoritative contract doc; manager-aware extensions edit/defers to that single authority.
- Provisioning guidance is enabled-mode only; disabled-mode diagnostics own the “disabled/skip probes” behavior and exclude provisioning guidance.
- Host installer package-manager detection/selection is out of scope for this pack and remains owned by ADR-0031/ADR-0032 packs.
- The legacy `substrate world deps provision` surface is not reintroduced without an explicit alias + compatibility policy.

## Follow-ups for full planning

1) Pin deterministic APT requirement derivation conflict policy (version pins, de-dup, ordering) via `decision_register.md` + `WDAP0-spec.md`.
2) Pin provisioning-time execution isolation model and host-mutation guard rails, including request `profile` value(s) and relationship to `SUBSTRATE_WORLD_REQUEST_PROFILE`, via `decision_register.md` + `WDAP0-spec.md`.
3) Pin `world enable --provision-deps` operational scope and ordering relative to baseline `world enable` behavior via `contract.md` + `WDAP0-spec.md`.
4) Pin runtime fail-early scope rule for `deps current install <ITEM...>` via `contract.md` + `WDAP1-spec.md`.
5) Pin runtime `--dry-run` and `--verbose` behavior under the fail-early posture via `contract.md` + `WDAP1-spec.md`.
6) Pin Windows posture for this feature (supported vs unsupported) via `contract.md` + playbooks/smoke.
7) Reconcile upstream contract/doc contradictions that currently imply runtime APT mutation (single authoritative truth).

## Draft slice skeleton (pre-planning only)

Disclaimer: draft; split/merge permitted during full planning; do not wire `tasks.json` yet.

Slice prefix (draft): WDAP

Downstream notes:
- CI checkpoint planning seeds the machine-readable slice list from this skeleton; defer mechanical validation until slice tasks exist in `tasks.json`.
- Workstream triage is permitted to propose edits to this skeleton as recommendations in `pre-planning/workstream_triage.md`; do not edit this file.

### WDAP0
- slice_id: WDAP0
- name: Add provisioning-time APT surface
- intent: Define the provisioning workflow and guard rails for installing APT/system packages required by the effective enabled world-deps set.
- likely touch surfaces:
  - `crates/shell/src/builtins/world_enable/`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/world-agent/src/service.rs`
  - `scripts/substrate/world-enable.sh`
  - `scripts/linux/world-provision.sh`
  - `scripts/mac/lima-warm.sh`
  - `scripts/windows/wsl-warm.ps1`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`

### WDAP1
- slice_id: WDAP1
- name: Fail early at runtime for APT items
- intent: Enforce the runtime invariant that `world deps current sync|install` never runs APT/dpkg and instead emits actionable remediation for APT-backed items.
- likely touch surfaces:
  - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - `crates/shell/src/builtins/world_deps/errors.rs`
  - `crates/shell/tests/world_deps_apt_install_wdp5.rs`
  - `docs/reference/world/deps/README.md`
  - `docs/internals/world/deps.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

