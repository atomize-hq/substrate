# persist-detected-linux-distro-pkg-manager — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- Orchestration branch: `feat/persist-detected-linux-distro-pkg-manager`
- Canonical pre-planning surfaces:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`

## Goal
- Persist Linux distro identity and selected package-manager metadata into `install_state.json` without changing the upstream detection contract or turning metadata failures into installer failures.

## Guardrails
- Specs under this feature directory are the single source of truth for execution.
- Linux distro detection semantics, selected-manager spellings, and `pkg_manager.source` vocabulary remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- Behavior changes remain Linux-only. macOS and Windows participate only in compile and test parity for this pack.
- `install_state.json` remains `schema_version = 1` with additive `host_state.platform.*` fields only.
- `--dry-run` remains a no-write branch and metadata read or write failures remain warning-only.
- No manual validation playbook is required for this pack. The installer smoke harness output and checkpoint CI evidence are the required validation artifacts.

## Slice Order
- `PDLDPM0` → `PDLDPM1` → `PDLDPM2`

### PDLDPM0 — Persist Linux platform metadata

Primary deliverables:
- Persist `host_state.platform.os_release.id`
- Persist `host_state.platform.os_release.id_like`
- Persist `host_state.platform.pkg_manager.selected`
- Persist `host_state.platform.pkg_manager.source`
- Preserve existing `host_state.group`, `host_state.linger`, and unknown keys during rewrite

Required validation command:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
```

### PDLDPM1 — Make install-state writes reliable

Primary deliverables:
- Write or update `<effective_prefix>/install_state.json` on successful Linux hosted and dev installs
- Keep hosted `--dry-run` and non-Linux runs as no-write branches
- Use same-directory temp-file replacement for successful writes
- Preserve successful installer exit status when metadata reads or writes fail

Required validation command:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
```

### PDLDPM2 — Lock smoke coverage and operator wording

Primary deliverables:
- Extend `tests/installers/install_state_smoke.sh` to cover no-event success, persisted platform fields, missing `/etc/os-release`, and additive compatibility
- Reconcile `docs/INSTALLATION.md` with the canonical metadata path, `schema_version = 1`, and the shared hosted-plus-dev producer contract
- Produce checkpoint-ready evidence for Linux behavior smoke and cross-platform parity

Required validation command:

```bash
bash tests/installers/install_state_smoke.sh --scenario metadata
```

## CI Checkpoint
- `CP1` runs after `PDLDPM2-integ-core`.
- `CP1` validates compile parity across `linux`, `macos`, and `windows`, plus Linux behavior smoke for the checkpoint SHA.

Required checkpoint commands:

```bash
CHECKOUT_SHA="$(git rev-parse persist-detected-linux-distro-pkg-manager-pdldpm2-integ-core)"
make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"
make ci-testing CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_MODE=quick CI_CHECKOUT_REF="$CHECKOUT_SHA"
make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" PLATFORM=behavior SMOKE_SLICE_ID="PDLDPM2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0
```

## Validation Evidence
- Slice execution must record the `tests/installers/install_state_smoke.sh --scenario metadata` output used for the slice.
- `CP1-ci-checkpoint` must record compile parity, CI Testing quick, and feature-smoke run ids and URLs in `session_log.md`.
- Feature cleanup runs only after `PDLDPM2-integ` and `CP1-ci-checkpoint` are both complete.
