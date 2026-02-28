**PRE‑PLANNING ONLY: This document is a cross-cutting alignment backbone draft. It MUST be deleted or explicitly retired during full planning.**

# add-non-apt-system-package-provisioning-support — minimal spec draft

## Scope + authority

Authority inputs for this draft (only):
- `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`

This draft is allowed to define:
- Cross-cutting invariants and non-overlap boundaries that every slice spec MUST preserve.
- Cross-cutting defaults/precedence explicitly required by ADR-0033 (and reaffirmed by `spec_manifest.md` + `impact_map.md`).
- Cross-cutting failure posture, safety posture, and exit-code posture (taxonomy subset; no overrides).

This draft MUST NOT define:
- Slice-specific behavior details (exact probe algorithm, exact pacman flags, exact error strings, test task breakdown).
- Any schema tables (owned by `world-deps-packages-bundles-contract/contract.md` and this feature’s `contract.md` once created).
- Any new scope beyond ADR-0033 + this pack’s `spec_manifest.md` + `impact_map.md`.

## Defaults + precedence

### CLI/config/env precedence
- ADR-0033 introduces no new environment variables and no new config keys; config precedence rules remain unchanged.
- Provisioning input sources are:
  1) CLI flags on `substrate world enable --provision-deps`: `--dry-run`, `--verbose`.
  2) The effective enabled world-deps set and inventory resolution (authoritative in `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`; unchanged by this pack).
- Precedence order (authoritative):
  1) CLI flags (`--dry-run`, `--verbose`)
  2) Effective enabled world-deps view (existing; unchanged)

### Provisioning flag invariants (cross-cutting)
- `--dry-run` prints the derived requirement set(s) and intended actions and performs no mutation (ADR-0033).
- Minimum guaranteed `--verbose` output elements and their stream requirements are owned by this pack’s `contract.md` (follow-up).

### Source-of-truth paths / inputs
- In-world OS identity input: `/etc/os-release` (`ID`, `ID_LIKE`) (read-only; probe-only).
- In-world manager presence checks (read-only; probe-only): presence of the OS manager executable(s) inside the world (e.g., `command -v pacman`).
- Effective enabled world-deps inputs (existing; unchanged): `$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`, and inventory directories per the world-deps contract.

## Failure posture + invariants

### Failure posture (cross-cutting)
- Default posture is fail-closed for system-package execution:
  - Runtime `substrate world deps current sync|install` MUST NOT execute OS package managers (`apt` or `pacman`).
  - Provisioning `substrate world enable --provision-deps` MUST fail deterministically when provisioning is unsupported or when enabled system-package items do not match the detected world OS manager.

### Safety invariants (cross-cutting)
- Host OS mutation invariant:
  - Linux host-native backend MUST NOT mutate the host OS; provisioning is unsupported by default on Linux host-native.
- Manager-selection invariant:
  - Package-manager selection MUST be derived from an in-world probe only and MUST NOT consult host PATH.
- Hardened runtime invariant:
  - Provisioning support MUST NOT weaken hardened runtime execution constraints; any “provisioning execution isolation” behavior is owned by DR-0005 and `docs/WORLD.md` notes if required.

### Redaction posture (high level)
- ADR-0033 introduces no new secret-bearing inputs (no new env vars; no new config keys).
- This feature defines no new redaction rules; operator-visible output introduced by ADR-0033 includes derived manager identity, derived requirement sets, and remediation text.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work introduces no new exit codes and no exit-code overrides.
- ADR-0033 uses the taxonomy subset `0/2/3/4/5` for provisioning/runtime flows.

## Cross-cutting seams / constraints

### Contract single-source-of-truth rule
- Operator-facing contract wording is owned by this pack’s `contract.md` (per `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`).
- Operator docs listed in `impact_map.md` MUST link to this pack’s `contract.md` and MUST NOT restate contract tables.

### Cross-pack contract ownership (shared provisioning surface)
- Selected (from `impact_map.md`): this pack’s `contract.md` is authoritative for:
  - the shared provisioning UX (`substrate world enable --provision-deps` manager-aware behavior), and
  - the runtime system-package prohibition + remediation invariants.
- The APT pack (`docs/project_management/packs/draft/world-deps-apt-provisioning/`) remains authoritative for APT-only invocation details and must defer for shared contract wording.

### Schema + enabled-resolution ownership boundary
- Inventory schema and enabled-set resolution are owned by:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- This pack MUST NOT introduce a second inventory/merge authority; it must drive required updates by reconciling that contract to:
  - add `install.method=pacman` + `install.pacman`, and
  - remove the “runtime apt install” contract conflict by deferring runtime system-package behavior to provisioning-time semantics.

### Operator-doc update targets (must match `impact_map.md`)
- Updates are required at the exact paths/headings enumerated in `impact_map.md` under “Operator-doc update targets”.
- Exact targets (from `impact_map.md`; must link to `contract.md` and avoid restating contract tables):
  - `docs/reference/world/deps/README.md`:
    - “Commands you will use”
    - “APT packages (current limitation in hardened worlds)” (becomes manager-aware and reflects provisioning-time system packages)
  - `docs/internals/world/deps.md`:
    - “High-level flow” (remove runtime “apt first” description; describe provisioning-time system packages)
    - “APT installs vs hardening” (update to provisioning-time; add pacman notes)
  - `docs/USAGE.md`:
    - “Health” example output and next-step guidance
  - `docs/INSTALLATION.md`:
    - “Installer Options Reference” (`--sync-deps` semantics)
    - “Troubleshooting Highlights” (“World deps not present (Linux/macOS)” guidance)
  - `docs/COMMANDS.md`:
    - `substrate world enable` (`--provision-deps` semantics)
    - `substrate world deps current sync` and `substrate world deps current install` (runtime prohibition + remediation semantics)
  - `docs/cross-platform/wsl_world_troubleshooting.md`:
    - “World deps quick check” next-step guidance
  - `docs/WORLD.md`:
    - World-deps command list + any provisioning execution isolation notes (if introduced by DR-0005 selection)

### Touch set alignment (pack boundary)
- Slice specs and planning artifacts MUST remain consistent with the `impact_map.md` touch set; out-of-touch-set edits are out of scope for this pack.

## Follow-ups for full planning

Each follow-up is required to remove ambiguity before planning can pass a quality gate.

1) Reconcile feature-directory drift
   - Make ADR-0033 reference exactly one planning-pack directory (this pack) to prevent dual-authority docs.

2) DR-0002 — Pin a deterministic in-world probe algorithm
   - Define exact Arch-family vs Debian/Ubuntu-family classification rules, precedence between `/etc/os-release` and manager presence checks, and behavior on disagreement/missing inputs.

3) DR-0003 — Pin the deterministic pacman invocation contract
   - Define exact non-interactive flags, idempotency definition, ordering/de-dup rules across enabled items, and error→exit mapping aligned to `contract.md`.

4) DR-0004 — Pin the mismatch policy
   - Define behavior for mixed `install.method=apt` + `install.method=pacman` enabled sets and whether any partial provisioning is permitted; pin remediation content.

5) DR-0005 — Pin the provisioning execution isolation model
   - Select request-profile vs guard-rails posture, and define whether any `docs/WORLD.md` protocol notes are required.

6) Pin `--verbose` output invariants
   - Define minimum guaranteed verbose elements and stream requirements for provisioning/runtime remediation paths.

7) Reconcile cross-document contract conflict (runtime apt install)
   - Update `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` so runtime `deps current sync|install` no longer implies OS manager execution and does not contradict ADR-0033.

## Draft slice skeleton (pre-planning only)

draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): ANS

- slice_id: ANS0
  name: Probe world OS family and manager
  intent: Stabilize the in-world probe contract (inputs + precedence + failure posture) that yields a single detected manager/family result for provisioning-time gating.
  likely touch surfaces:
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS0/ANS0-spec.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
    - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
    - `crates/shell/src/builtins/world_enable/runner.rs`

- slice_id: ANS1
  name: Provision `install.method=pacman` system packages
  intent: Stabilize inventory validation + deterministic requirement derivation and pacman execution semantics for provisioning-time system packages on Arch-family worlds.
  likely touch surfaces:
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `crates/shell/src/builtins/world_deps/inventory.rs`
    - `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
    - `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`

- slice_id: ANS2
  name: Runtime fail-early + operator-doc updates
  intent: Stabilize runtime prohibition/remediation behavior for system-package methods and apply deterministic operator-doc updates that link to `contract.md`.
  likely touch surfaces:
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS2/ANS2-spec.md`
    - `docs/reference/world/deps/README.md`
    - `docs/internals/world/deps.md`
    - `docs/COMMANDS.md`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Downstream notes:
- CI-checkpoint: prefer this slice list when populating the machine-readable slices list in `ci_checkpoint_plan.md` (no mechanical validation until slice tasks exist in `tasks.json`).
- Workstream triage is allowed to propose edits to this slice skeleton as recommendations in `workstream_triage.md` (must not edit this file).
