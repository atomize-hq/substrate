# persist-detected-linux-distro-pkg-manager — plan

## Scope

- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- Orchestration branch: `feat/persist-detected-linux-distro-pkg-manager`
- Spec ownership map: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Workstream triage authority for slice inventory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md`

## Goal

- Deliver one Linux install-state persistence contract across the production and dev installers, preserve additive compatibility for `install_state.json`, and lock the Linux validation bundle that keeps the contract observable.

## Guardrails (non-negotiable)

- Specs under this feature directory are the single source of truth for ADR-0032.
- Linux is the only behavior-delta platform; macOS and Windows remain explicit no-delta platforms for `host_state.platform.*`.
- `best-effort-distro-package-manager` remains authoritative for package-manager selection, `pkg_manager.source`, and `/etc/os-release` normalization.
- `schema_version` remains `1`, and compatible `host_state.group` plus `host_state.linger` data stays preserved.
- Metadata persistence stays under `<resolved SUBSTRATE_HOME>/install_state.json` only and remains fail-open.

## Slices (sequencing)

Workstream triage split the original draft validation seam into a fourth slice so dev-installer parity does not hide inside the production-installer write guarantee. Execution order is fixed:

- `PDLDPM0` → `PDLDPM1` → `PDLDPM3` → `PDLDPM2`

### PDLDPM0 — Persist additive Linux platform metadata

Primary deliverables:

- Capture `host_state.platform.os_release.id`
- Capture `host_state.platform.os_release.id_like`
- Capture `host_state.platform.pkg_manager.selected`
- Capture `host_state.platform.pkg_manager.source`
- Omit unavailable values without `null` or sentinel substitutes

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
```

### PDLDPM1 — Guarantee production installer install-state writes

Primary deliverables:

- `scripts/substrate/install-substrate.sh` always leaves `<resolved SUBSTRATE_HOME>/install_state.json` present after a successful non-dry-run Linux install
- compatible existing files preserve `created_at`, unknown keys, `host_state.group`, and `host_state.linger`
- corrupt and wrong-schema files are replaced with a fresh `schema_version: 1` document
- `--no-world` stays inside the same write contract and dry-run remains no-write

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
bash tests/installers/install_smoke.sh --scenario prod
bash tests/installers/install_smoke.sh --scenario prod-no-world
```

### PDLDPM3 — Keep the dev installer on the same install-state contract

Primary deliverables:

- `scripts/substrate/dev-install-substrate.sh` writes the same Linux install-state contract as the production installer
- dev-installer dry-run remains no-write
- dev-installer `--no-world` remains inside the same metadata contract

Required validation commands:

```bash
bash tests/installers/install_smoke.sh --scenario dev
```

### PDLDPM2 — Verify the persisted metadata contract

Primary deliverables:

- lock the Linux smoke ownership split between `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh`
- prove the fresh-file path, compatible-merge path, corrupt-file replacement path, and unreadable-`/etc/os-release` omission path
- record the explicit no-delta evidence expected for macOS and Windows

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
bash tests/installers/install_smoke.sh --scenario prod
bash tests/installers/install_smoke.sh --scenario prod-no-world
bash tests/installers/install_smoke.sh --scenario dev
```

## Linux validation ownership

- `tests/installers/install_state_smoke.sh` owns production-installer metadata assertions for:
  - fresh-file creation
  - compatible-file preservation
  - corrupt or wrong-schema replacement
  - unreadable `/etc/os-release` omission behavior
- `tests/installers/install_smoke.sh` owns shared installer-flow assertions for:
  - production installer success path
  - production installer `--no-world`
  - dev-installer parity for the same install-state contract

## CI parity (cross-platform)

Run the cross-platform parity gate on the orchestration branch:

```bash
make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"
```

## Operator-doc updates required for closeout

- `docs/INSTALLATION.md`
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

## External dependency sequencing

- Land or adopt the `best-effort-distro-package-manager` contract before implementation closes this pack so persisted manager names, `pkg_manager.source`, and `/etc/os-release` normalization come from one authority.
