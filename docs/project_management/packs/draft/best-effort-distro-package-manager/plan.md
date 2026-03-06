# best-effort-distro-package-manager — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- Orchestration branch: `feat/best-effort-distro-package-manager`
- Spec ownership map: `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`

## Goal
- Deliver a deterministic, supportable Linux host installer pkg-manager selection pipeline aligned to `contract.md`, with a hermetic detection harness and explicit exit-code semantics.

## Guardrails (non-negotiable)
- Specs under this feature directory are the single source of truth.
- Linux-only behavior: this feature introduces no behavior change on macOS or Windows.
- Exit-code remapping is scoped to the Linux installer pkg-manager decision path (`0/2/3/4` as defined in `contract.md`).
- The required decision one-liner is an exact string and is emitted without prefixes.

## Work Lift (advisory)

Pack-derived Work Lift is meaningful only for strict packs (`tasks.json.meta.slice_spec_version >= 2`).

Compute from the pack:

```bash
make pm-lift-pack PACK="docs/project_management/packs/draft/best-effort-distro-package-manager"
make pm-lift-pack PACK="docs/project_management/packs/draft/best-effort-distro-package-manager" EMIT_JSON=1
```

## Slices (sequencing)

Sequencing is fixed for this pack:
- `BEDPM0` → `BEDPM1` → `BEDPM2`

### BEDPM0 — Detect distro + emit decision one-liner

Primary deliverables:
- `/etc/os-release` best-effort read + safe parsing contract behavior in `scripts/substrate/install-substrate.sh`.
- Stable stderr decision one-liner (exact string; exactly once; emitted before prereq installation begins).

Required validation commands:

```bash
bash tests/installers/pkg_manager_detection_test.sh
```

### BEDPM1 — Deterministic pkg-manager selection + failure posture

Primary deliverables:
- Precedence pipeline: `--pkg-manager` → `PKG_MANAGER` → os-release mapping → deterministic PATH probe.
- Fail-closed semantics for explicit overrides and exit-code mapping for pkg-manager decision failures (`2/3/4`).

Required validation commands:

```bash
bash tests/installers/pkg_manager_detection_test.sh
```

### BEDPM2 — Hermetic detection harness

Primary deliverables:
- Implement `tests/installers/pkg_manager_detection_test.sh` to assert the contract deterministically without containers and without host mutation.

Required validation commands:

```bash
bash tests/installers/pkg_manager_detection_test.sh
```

## Manual validation

Run the playbook cases:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

## Optional local validation (not CI gating)

Container sanity check (requires Docker):

```bash
make installers-container-smoke
```

## CI parity (cross-platform)

Compile-parity gate (requires branch ref):

```bash
make ci-compile-parity CI_WORKFLOW_REF="feat/best-effort-distro-package-manager"
```
