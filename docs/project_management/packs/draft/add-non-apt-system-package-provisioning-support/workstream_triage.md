# add-non-apt-system-package-provisioning-support — workstream triage

Goal: propose parallelizable workstreams + sequencing gates for full planning/execution.

## Evidence (inputs + completion sentinels)

Canonical artifacts relied on:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/minimal_spec_draft.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` (`meta.slice_spec_version=2` strict pack; `meta.cross_platform=true`)

Stable sentinels:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/CI-checkpoint/last_message.md`

Work Lift evidence (advisory):
- Pack-derived: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_pack.{txt,json}`
- Intake/ADR-derived: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_intake.{txt,json}`

## Work Lift summary (advisory; prioritize triggers + confidence)

- Intake/ADR lift: `lift_score=27`, `estimated_slices=3`, `confidence=low`
- Pack lift (strict; impact-map touch set): `lift_score=67`, `estimated_slices=6`, `confidence=low`
  - Trigger highlights: `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`, `split_required:estimated_slices>3`
  - Derived touch set counts: `create=11`, `edit=15` (no deletes/deprecations)

Takeaway:
- This is “large enough to split” by lift triggers, but the touch set is still concentrated (primarily `crates/shell` + docs/contracts), so plan for 5 workstreams with hard gates on seams that would otherwise churn (schema, probe, mismatch policy, request-profile guard rails).

## Proposed workstreams

### WS-CONTRACT — Shared CLI contract + decision register (hard gate)

Goal:
- Lock the operator-visible contract for:
  - provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]`,
  - manager mismatch + mixed enabled-set policy (`apt` + `pacman`),
  - runtime `substrate world deps current sync|install` fail-early behavior for system-package methods,
  - exit-code mapping + remediation message invariants,
  - platform support matrix (Linux host-native vs macOS Lima vs Windows WSL).

Owns (surfaces / files to create during planning; per `spec_manifest.md`):
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` (DR-0001..DR-0005)
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md` (must encode gates + validation commands)

Depends on:
- None (this is the upstream gate for the rest of the pack)

Full-planning slices/triads to create:
- Doc-first: DR-0001..DR-0005 resolved with explicit consequences on C0/C1/C2 specs and cross-pack updates.
- Plan gate: explicitly pin “Option A” cross-pack ownership (this pack’s `contract.md` is authoritative for shared CLI/exit-code/remediation across `apt|pacman`).

### WS-SCHEMA — Inventory schema + cross-pack boundary updates (dependency seam)

Goal:
- Extend the world-deps inventory schema to support pacman deterministically and ensure cross-pack docs do not duplicate/contradict the shared CLI contract.

Owns (surfaces / touch set from `impact_map.md`):
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (add `install.method=pacman` + `install.pacman[]`; align runtime fail-early contract references)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` (defer shared CLI contract wording to this pack’s `contract.md`)
- `crates/shell/src/builtins/world_deps/inventory.rs` (schema validation alignment)
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`

Depends on:
- WS-CONTRACT (DR-0001 inventory schema approach; DR-0005 contract-ownership boundary)

Full-planning slices/triads to create:
- Schema-validation triad for `install.method=pacman` and required `install.pacman[]` shape (and invalid-shape exit code `2` mapping).

### WS-PROBE — In-world probe + manager classification (C0)

Goal:
- Define and implement the in-world probe algorithm (no host PATH dependence) and derived manager classification used by provisioning-time flows.

Owns (surfaces / touch set):
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md`
- `crates/shell/src/execution/platform/mod.rs` (probe wiring; backend constraints)
- `crates/shell/src/builtins/world_enable/runner.rs` (probe invocation plumbing)
- Tests: `crates/shell/tests/world_enable.rs`

Depends on:
- WS-CONTRACT (DR-0002 probe strategy + derived enum vocabulary; mismatch-policy constraints)

Full-planning slices/triads to create:
- C0 triad: `/etc/os-release` parsing + canonicalization, manager presence checks inside the world, and deterministic derived-enum mapping (incl. Arch-family classification rules).

### WS-PROVISION — Provisioning protocol + pacman invocation semantics (C1)

Goal:
- Specify and implement the pacman provisioning path for supported guest worlds, including guard rails that prevent provisioning semantics from leaking into hardened runtime.

Owns (surfaces / touch set):
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs` (provisioning execution routing)
- `crates/shell/src/execution/cli.rs` (flag plumbing; stdout/stderr contract surfaces)
- `crates/shell/src/builtins/world_enable/runner.rs` (request-profile selection; dry-run behavior; provisioning execution)

Depends on:
- WS-CONTRACT (DR-0003 pacman invocation; DR-0004 mismatch policy; platform support matrix)
- WS-SCHEMA (effective enabled set → `install.pacman[]` derivation inputs)
- WS-PROBE (manager classification outputs)

Full-planning slices/triads to create:
- C1 triad: requirement derivation from the effective enabled set (bundle expansion, de-dup, ordering), deterministic `--dry-run` output, and exact non-interactive pacman command contract + idempotency posture.
- Provisioning protocol spec: request profile value, timeouts/budgets, and error model boundaries (`3` vs `4`) aligned to `EXIT_CODE_TAXONOMY.md`.

### WS-RUNTIME+QA — Runtime fail-early + docs + validation harness (C2 + checkpoint)

Goal:
- Make runtime surfaces fail early when system-package methods exist in scope, and ship coherent operator docs + tests + checkpoint wiring across linux/macos/windows.

Owns (surfaces / touch set):
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`

Depends on:
- WS-CONTRACT (exit-code + remediation invariants; “no host mutation” wording)
- WS-SCHEMA (what “system-package method exists in scope” means)
- WS-PROVISION (provisioning remediation must be accurate and manager-aware)

Full-planning slices/triads to create:
- C2 triad: pin and enforce runtime fail-early scope (effective enabled set vs explicit args), update docs, and add deterministic tests for short-circuit + remediation.
- CP1 checkpoint wiring (per `ci_checkpoint_plan.md`): populate `tasks.json` triads + checkpoint task(s) and set `meta.checkpoint_boundaries`.

## Sequencing + gates (hard constraints)

1) **WS-CONTRACT gate:** DR-0001..DR-0005 + `contract.md` must land before C0/C1/C2 slice specs are treated as stable (prevents mismatch-policy and boundary churn).
2) **WS-SCHEMA gate:** inventory schema + contract ownership boundary must be pinned before implementing or testing pacman requirement derivation.
3) **WS-PROBE gate:** manager classification must be deterministic before provisioning behavior (`--provision-deps`) can be finalized.
4) **WS-PROVISION gate:** provisioning request profile semantics + pacman command contract must be pinned before any “supported backend” claim is made in docs/tests.
5) Execute C2 (runtime fail-early + docs/tests), then run CP1 checkpoint (compile parity + feature smoke + CI testing) once `tasks.json` is populated.

Cross-queue merge-risk gates (from `impact_map.md`):
- Coordinate changes in `crates/shell/src/builtins/world_enable/` with ADR-0034/ADR-0035 work to minimize conflicts.
- Reconcile ADR-0009 legacy surface (`substrate world deps provision`) with the chosen entrypoint (`substrate world enable --provision-deps`) during contract finalization.

## Risks + unknowns (to resolve during full planning)

- Mixed enabled sets (`install.method=apt` + `install.method=pacman`): deterministic mismatch policy (fail-closed vs partial provision).
- Runtime “fail early” scope for `deps current install <ITEM...>`: effective enabled set vs explicit args only.
- WSL support posture: ADR-0033 marks assumptions; full planning must lock as supported/unsupported with deterministic messaging.
- Provisioning request profile seam: profile value, guard rails, and whether any global env knobs exist (must not weaken hardened runtime).
- Probe parsing/canonicalization drift risk vs host installer detection/persistence work (ADR-0031/ADR-0032): align vocabulary without conflating contracts.
- Exit-code mapping boundaries (`3` world backend unavailable vs `4` unsupported/mismatch) must be testable and stable.

## Follow-ups

- `ci_checkpoint_plan.md` is intentionally not mechanically valid yet: it requires slice `*-integ` tasks + checkpoint wiring in `tasks.json` (see its Follow-ups).
- After pack acceptance, fix ADR-0033 link drift to reference this feature directory (recorded in `spec_manifest.md` Follow-ups; do not patch ADRs from triage).
