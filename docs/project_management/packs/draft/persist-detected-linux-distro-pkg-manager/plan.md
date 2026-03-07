# persist-detected-linux-distro-pkg-manager — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- Orchestration branch: `feat/persist-detected-linux-distro-pkg-manager`
- Spec manifest: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Platform parity: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`

## Goal
- Deliver one Linux-only persisted install-state contract for distro and package-manager metadata, keep `schema_version=1`, preserve the existing `host_state.group` and `host_state.linger` meaning, and add executable smoke evidence without introducing a macOS or Windows behavior delta.

## Guardrails (non-negotiable)
- Specs under this feature directory are the single source of truth for ADR-0032 planning.
- Slice order is fixed: `PDLDPM0` before `PDLDPM1`, `PDLDPM1` before `PDLDPM3`, and `PDLDPM3` before `PDLDPM2`.
- Linux is the only behavior-delta platform. macOS and Windows remain explicit no-delta platforms for this feature.
- The install-state contract remains one file at `<resolved SUBSTRATE_HOME>/install_state.json` with `schema_version=1`; this feature does not create a second metadata file.
- Validation ownership is split intentionally:
  - `tests/installers/install_state_smoke.sh` owns production-installer metadata, merge/reset, and cleanup compatibility assertions.
  - `tests/installers/install_smoke.sh` owns `--no-world` production assertions and dev-installer parity assertions.

## Slices (sequencing)

Sequencing is fixed for this pack:
- `PDLDPM0` -> `PDLDPM1` -> `PDLDPM3` -> `PDLDPM2`

### PDLDPM0 — Persist additive platform metadata

Primary deliverables:
- Add `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` to the `schema_version=1` install-state contract without redefining detection semantics.
- Lock omission semantics for unavailable `/etc/os-release` values.

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
```

### PDLDPM1 — Guarantee production-installer writes

Primary deliverables:
- Make successful non-dry-run Linux runs of `scripts/substrate/install-substrate.sh` create or update `<resolved SUBSTRATE_HOME>/install_state.json` even when no group or linger events occurred.
- Keep dry-run non-mutating and keep metadata failures warning-only.

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
bash tests/installers/install_smoke.sh --scenario prod-no-world
```

### PDLDPM3 — Keep dev-installer parity

Primary deliverables:
- Keep `scripts/substrate/dev-install-substrate.sh` on the same Linux install-state contract as the production installer.
- Keep one `install_state.json` meaning across both Unix installer entry points.

Required validation commands:

```bash
bash tests/installers/install_smoke.sh --scenario dev
```

### PDLDPM2 — Validate the persisted metadata contract

Primary deliverables:
- Extend Linux smoke coverage so the persisted platform payload, `schema_version=1` preservation, host-state preservation, unreadable `/etc/os-release` omission rule, production `--no-world` behavior, and dev-installer parity are observable in harnesses.
- Record the explicit macOS/Windows no-delta evidence required for closeout.

Required validation commands:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
bash tests/installers/install_state_smoke.sh --scenario cleanup
bash tests/installers/install_state_smoke.sh --scenario missing
bash tests/installers/install_smoke.sh --scenario prod-no-world
bash tests/installers/install_smoke.sh --scenario dev
```

## Operator-doc updates required for closeout

- `docs/INSTALLATION.md`
  - `### Installer Metadata & Cleanup` must state that successful non-dry-run Linux installs guarantee `<prefix>/install_state.json`, that the file remains `schema_version=1`, and that `host_state.platform.*` is a Linux-only additive subtree.
  - `### macOS (arm64)` must state that ADR-0032 introduces no new `host_state.platform.*` write contract on macOS.
  - `### Windows Host (PowerShell)` must state that ADR-0032 introduces no new `host_state.platform.*` write contract on Windows.

## CI parity (cross-platform)

Linux behavior smoke is required because Linux is the only behavior-delta platform. macOS and Windows remain parity-only platforms for this feature.

Required parity gate:

```bash
make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"
```
