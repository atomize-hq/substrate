**WARNING: This document is Pre-Planning Only and will be deleted or retired during full planning.**

# add-non-apt-system-package-provisioning-support — minimal spec draft

## Scope + authority

This draft defines only pack-level defaults, precedence, shared invariants, and cross-slice alignment points for `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`.

This draft does not define slice-specific behavior, detailed schemas, command payloads, test matrices, implementation tasks, or execution sequencing beyond the draft slice skeleton below.

Pre-planning authority is split as follows:
- Intent and baseline contract direction: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Required-doc set and canonical slice IDs: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- Touch surfaces, overlap decisions, and cross-pack reconciliation set: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`

Full-planning authority must hand off from this draft into:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`

## Defaults + precedence

This feature introduces no new config key and no new environment variable.

Precedence for the feature surface is:
1. Explicit CLI command and flags determine the operating mode.
   - `substrate world enable --provision-deps [--dry-run] [--verbose]` is the only operator-facing provisioning entrypoint.
   - `substrate world deps current sync` and `substrate world deps current install` remain runtime commands and do not become provisioning commands.
2. Existing world-deps inventory content plus effective enabled-set resolution determine the package requirements in scope for the chosen command path.
3. Internal request-profile plumbing is not an operator override surface.
   - `SUBSTRATE_WORLD_REQUEST_PROFILE` remains an implementation detail and must not be documented as required operator input for this feature.

Source-of-truth file ownership defaults are:
- Shared manager-aware CLI/runtime semantics: this pack’s `contract.md`
- Shared APT-only historical wording: `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` must defer on shared CLI/runtime semantics once full planning reconciles the docs
- Canonical slice IDs and slice count: `pre-planning/spec_manifest.md`

## Failure posture + invariants

The feature posture is fail-closed.

The following invariants apply across all slice specs:
- Runtime `substrate world deps current sync|install` never invokes `apt` or `pacman`.
- Unsupported backend, unsupported world, or manager-mismatch paths fail instead of falling back to another package manager or to host mutation.
- Host-native Linux OS mutation remains forbidden.
- Package-manager selection is derived in-world; host PATH, host installer detection, and host package-manager state are not routing inputs.
- `--dry-run` performs no mutation.
- This feature does not add a new protocol, request field, telemetry field, config key, or environment variable.

Security and redaction posture:
- No new redaction exception is introduced by this draft.
- Operator-facing output may identify the world package manager, package requirement names, exit code, and remediation command.
- Logs and errors must continue to reuse the existing redaction posture for secrets, credentials, and unrelated host-private values.

## Exit-code posture

Exit-code taxonomy reference:
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

This work does not require new exit codes.

The current pack-level mapping remains:
- `0`: success or contract-defined no-op
- `2`: invalid inventory or schema shape
- `3`: world backend unavailable
- `4`: unsupported prerequisite, unsupported provisioning path, or manager mismatch
- `5`: safety or policy violation

## Cross-cutting seams / constraints

- This pack owns the shared manager-aware operator contract for `substrate world enable --provision-deps` and the runtime no-system-package-mutation posture. Overlapping pack docs must defer on shared semantics after reconciliation.
- No separate protocol spec, env-vars spec, telemetry spec, filesystem-semantics spec, or compatibility spec is selected for this feature. Slice specs must not invent those surfaces.
- The canonical slice prefix is `NASP`. Canonical slice spec paths stay under `slices/NASP0/`, `slices/NASP1/`, and `slices/NASP2/`.
- Cross-doc terminology must stay stable:
  - `system-package items` means inventory items whose `install.method` is `apt` or `pacman`
  - `provisioning-time only` means `substrate world enable --provision-deps`, not runtime `world deps current ...`
- Mixed-manager enabled-set behavior is unresolved at pre-planning time. Slice specs must not assume fallback, partial success, or silent skipping before the decision register pins the exact rule.
- `docs/WORLD.md` and `docs/CONFIGURATION.md` remain additive-only references for existing request-profile plumbing. This feature does not define a new wire contract.

## Follow-ups for full planning

- Choose one exact probe tie-break rule between `/etc/os-release` and package-manager presence checks, then record it in `decision_register.md` and `slices/NASP0/NASP0-spec.md`.
- Choose one exact mixed-manager enabled-set rule, then record it in `contract.md`, `slices/NASP1/NASP1-spec.md`, and `slices/NASP2/NASP2-spec.md`.
- Choose one exact Windows posture for the WSL backend, then record it in `platform-parity-spec.md` and `smoke/windows-smoke.ps1`.
- Choose one exact pacman command-construction and idempotency rule, then record it in `decision_register.md` and `slices/NASP1/NASP1-spec.md`.
- Decide whether pacman-backed runnable-wrapper or present-semantics behavior is in scope for v1, then record the answer in `decision_register.md`, `world-deps-pacman-schema-spec.md`, and `contract.md`.
- Reconcile ADR and upstream contract links so that one authoritative manager-aware truth remains across `ADR-0033`, this pack’s `contract.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`, and `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`.

## Draft slice skeleton (pre-planning only)

draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `NASP`

This skeleton reuses the canonical slice IDs already selected in `pre-planning/spec_manifest.md`.

| slice_id | name | intent | likely touch surfaces |
| --- | --- | --- | --- |
| `NASP0` | Probe world manager and support gate | Stabilize the in-world OS/package-manager probe and the supported-vs-unsupported provisioning gate without defining pacman command details. | `crates/shell/src/builtins/world_enable/runner.rs`, `crates/shell/src/builtins/world_enable/runner/helper_script.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, `crates/world-agent/src/service.rs`, `slices/NASP0/NASP0-spec.md`, `platform-parity-spec.md` |
| `NASP1` | Add pacman schema and provisioning path | Stabilize the `install.method=pacman` contract, effective requirement derivation, and provisioning-time pacman execution path. | `crates/shell/src/builtins/world_deps/inventory.rs`, `crates/shell/src/builtins/world_deps/surfaces.rs`, `crates/shell/src/builtins/world_enable/runner/log_ops.rs`, `world-deps-pacman-schema-spec.md`, `contract.md`, `slices/NASP1/NASP1-spec.md` |
| `NASP2` | Lock runtime fail-early and reconcile docs | Stabilize the runtime no-system-package-mutation rule, operator remediation wording, and validation/doc reconciliation surfaces. | `crates/shell/src/execution/cli.rs`, `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`, `crates/shell/tests/world_deps_apt_install_wdp5.rs`, `docs/reference/world/deps/README.md`, `docs/internals/world/deps.md`, `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`, `slices/NASP2/NASP2-spec.md` |

Downstream note:
- CI-checkpoint should prefer this slice list when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`; do not validate mechanically until slice tasks exist in `tasks.json`.
- Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md`; it must not edit this file.
