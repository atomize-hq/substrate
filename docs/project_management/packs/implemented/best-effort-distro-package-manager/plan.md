# best-effort-distro-package-manager — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- Orchestration branch: `feat/best-effort-distro-package-manager`
- Canonical pre-planning surfaces:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`

## Goal
- Ship the ADR-0031 installer contract as four bounded triads that preserve Linux-only behavior change, keep operator docs aligned, and end with one checkpoint-backed validation seam.

## Guardrails
- Specs under this feature directory are the single source of truth for execution.
- Planning-pack docs change only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Behavior change remains Linux-only. macOS and Windows participate in CI parity only.
- `BEDPM3` is the single checkpoint boundary. No earlier slice dispatches cross-platform CI.
- Each task stays within the single-slice context budget defined by `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`.

## Slice Order
- `BEDPM0` — Lock distro detection, mapping, and decision-line reporting.
  - Primary surfaces: `scripts/substrate/install-substrate.sh`, `contract.md`, `decision_register.md`
- `BEDPM1` — Lock override precedence, fallback order, and failure classes.
  - Primary surfaces: `scripts/substrate/install-substrate.sh`, `contract.md`, `decision_register.md`
- `BEDPM2` — Preserve wrapper exits and propagate the operator contract.
  - Primary surfaces: `scripts/substrate/install.sh`, `docs/INSTALLATION.md`, `docs/reference/env/contract.md`
- `BEDPM3` — Hermetic validation, thin smoke alignment, and manual evidence.
  - Primary surfaces: `tests/installers/pkg_manager_detection_smoke.sh`, `smoke/linux-smoke.sh`, `manual_testing_playbook.md`

## Validation Commands
- Hermetic harness:

```bash
bash tests/installers/pkg_manager_detection_smoke.sh
```

- Feature-local Linux smoke wrapper:

```bash
bash docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh
```

## CI Checkpoint
- `CP1` runs after `BEDPM3-integ-core`.
- `CP1` validates:
  - compile parity across `linux`, `macos`, and `windows`
  - CI Testing quick across `linux`, `macos`, and `windows`
  - Linux behavior smoke for slice `BEDPM3`

Checkpoint commands:

```bash
CHECKOUT_SHA="$(git rev-parse best-effort-distro-package-manager-bedpm3-integ-core)"
make ci-compile-parity CI_WORKFLOW_REF="feat/best-effort-distro-package-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"
make ci-testing CI_WORKFLOW_REF="feat/best-effort-distro-package-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_MODE=quick CI_CHECKOUT_REF="$CHECKOUT_SHA"
make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" PLATFORM=behavior SMOKE_SLICE_ID="BEDPM3" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/best-effort-distro-package-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0
```

## Execution Notes
- `BEDPM0`, `BEDPM1`, and `BEDPM2` use the normal schema v4 triad shape: `code`, `test`, `integ`.
- `BEDPM3` uses the checkpoint-boundary schema v4 shape: `code`, `test`, `integ-core`, `integ-linux`, `integ-macos`, `integ-windows`, `integ`.
- `FZ-feature-cleanup` runs after `BEDPM3-integ`.
- `pre-planning/spec_manifest.md` still needs one tracked follow-up to mirror the schema v4 task model. That allowlist-blocked reconciliation is logged under `logs/pws/BEDPM-PWS-tasks_checkpoints/`.
