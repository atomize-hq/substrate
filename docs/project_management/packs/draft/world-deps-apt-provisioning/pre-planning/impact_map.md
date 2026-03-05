# world-deps-apt-provisioning — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Spec manifest:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/world-deps-apt-provisioning"` (strict packs only).

### Create
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/execution/platform/mod.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
- `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/errors.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/world-agent/src/service.rs`
- `crates/shell/tests/world_enable.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/linux/world-provision.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/substrate-world-agent.service`
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- `scripts/windows/wsl-warm.ps1`
- `scripts/wsl/provision.sh`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- `docs/WORLD.md`
- `docs/CONFIGURATION.md`
- `docs/COMMANDS.md`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Add provisioning-time system package flow: `substrate world enable --provision-deps [--dry-run] [--verbose]` (guest backends only).
  - Direct impact:
    - Operators have a single explicit entry point to install/ensure APT system packages required by the effective enabled world-deps set on supported guest backends.
    - `--dry-run` becomes a first-class “no mutation” preview that prints the derived APT requirement list deterministically.
  - Cascading impact:
    - `world enable` must define the ordering and idempotency of “world backend enable” vs “APT provisioning” steps (and how `--dry-run` applies).
    - Provisioning must be gated by backend posture so Linux host-native never mutates the host OS; unsupported paths must exit `4` with explicit “no host OS mutation” messaging.
    - `scripts/substrate/world-enable.sh` and installer helper flows that currently call `substrate world deps current sync` post-provision must remain coherent with the new system-packages posture.
  - Contradiction risks:
    - `SUBSTRATE_WORLD_REQUEST_PROFILE` exists as an advanced/testing escape hatch; provisioning-time behavior must not rely on operators setting this manually, and must not create a hidden host-mutation path on Linux host-native.
    - Windows posture is currently ambiguous because `substrate world enable` is Windows-unsupported today; the feature must define a deterministic Windows behavior (supported vs explicitly unsupported with manual guidance).

- Change: Runtime invariant: `substrate world deps current sync|install` MUST NOT invoke APT/dpkg.
  - Direct impact:
    - Runtime “apply deps” surfaces stop attempting OS mutation; encountering APT-backed items yields an actionable remediation that includes the exact command `substrate world enable --provision-deps`.
    - Exit code `4` becomes the stable “unmet prerequisites/unsupported for runtime” signal for APT-backed items.
  - Cascading impact:
    - `deps current sync`/`install` must implement a deterministic “APT in scope” rule (enabled-set vs explicit args) and a deterministic `--dry-run` posture under fail-early.
    - The remediation text must branch by backend capability (supported provisioning vs unsupported) and must explicitly state “no host OS mutation” on Linux host-native.
    - Existing APT install coverage in `crates/shell/tests/world_deps_apt_install_wdp5.rs` must be repurposed so it asserts “no runtime apt” and checks remediation text + exit code.
  - Contradiction risks:
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` currently requires runtime `deps current install` to “apply world image installs first (apt)”, which directly contradicts this ADR; contract ownership must be reconciled (single authoritative truth).
    - `docs/internals/world/deps.md` and operator docs currently describe runtime APT behavior and its failure mode; these must be updated to avoid implying runtime APT is still attempted.

### Config / env vars / paths
- Change: No new config keys; provisioning derives requirements from the existing effective enabled world-deps view.
  - Direct impact:
    - Operators do not need to adopt new config keys to use provisioning-time APT; the enabled-set remains the input.
  - Cascading impact:
    - Docs must clearly distinguish the operator-facing workflow (`substrate world enable --provision-deps`) from the advanced/testing env var (`SUBSTRATE_WORLD_REQUEST_PROFILE`) and must define their relationship explicitly.
    - Any update to `docs/WORLD.md` request schema must remain additive and must not imply TLS/remote transport (UDS/forwarded socket contract remains).
  - Contradiction risks:
    - Multiple docs currently reference legacy knobs and/or drifted flags (`--prefix` vs `--home`) for `world enable`; documentation updates must avoid increasing drift while adding `--provision-deps`.

### Policy / isolation / security posture
- Change: Provisioning-time APT requires an execution posture that permits guest OS mutation without weakening hardened runtime execution (decision DR-0003).
  - Direct impact:
    - Provisioning can succeed for apt/dpkg even when hardened runtime execution paths are effectively read-only.
  - Cascading impact:
    - World-agent execution must introduce explicit guard rails: provisioning behavior is available only for the explicit provisioning workflow and never for arbitrary runtime commands on Linux host-native.
    - Provisioning-time execution must be observable (logs/trace) and must not silently fall back to host mutation.
    - Platform-specific provisioning scripts and service hardening must remain coherent with the chosen posture (systemd unit sandboxing vs in-world isolation; no “runtime hardening regression”).
  - Contradiction risks:
    - Active work in `docs/project_management/packs/active/world_process_exec_tracing_parity/` touches `crates/world-agent/src/service.rs` and systemd capability posture; provisioning-profile changes must not break tracing/process-capture invariants.

### Operator-doc update targets (exact paths + headings)

These docs MUST be updated to reflect “APT is provisioning-time” and MUST link to `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` rather than restating tables:
- `docs/reference/world/deps/README.md` (update: `## APT packages (current limitation in hardened worlds)` and `## Commands you will use`)
- `docs/internals/world/deps.md` (update: `## High-level flow` and `## APT installs vs hardening`)
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (update: `#### substrate world deps current install <item_name...>` and `#### substrate world deps current sync [--dry-run] ...`)
- `docs/WORLD.md` (update: `## 5) Agent API (over UDS)` → `POST /v1/execute` body fields if the provisioning posture relies on request `profile`)
- `docs/CONFIGURATION.md` (update: `SUBSTRATE_WORLD_REQUEST_PROFILE` row to explicitly state it is not the operator-facing provisioning workflow)
- `docs/COMMANDS.md` (update: `### world Subcommand` row for `substrate world enable` flags)

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - shared provisioning entrypoint `substrate world enable --provision-deps` and its `--dry-run`/`--verbose` semantics
    - shared runtime “no OS package manager at runtime” invariant for `world deps current sync|install`
    - shared guard rails around request `profile` / `SUBSTRATE_WORLD_REQUEST_PROFILE`
  - Conflict: yes
  - Resolution (explicit):
    - Sequencing boundary: ADR-0030 must land first (defines the APT baseline + shared provisioning entrypoint); ADR-0033 extends it for pacman/Arch-family worlds without redefining the base APT contract.
    - Contract ownership boundary: exactly one authoritative contract doc must own the shared `--provision-deps` UX; if ADR-0033 makes the shared surface manager-aware, it must do so by editing/deferring to the single authority rather than duplicating contract text across packs.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - Linux “no host OS mutation” posture and future Linux guest provisioning support
    - operator messaging for Linux provisioning support boundaries
  - Conflict: no
  - Resolution (explicit):
    - Keep Linux host-native provisioning unsupported (exit `4`, explicit manual guidance) while ensuring the backend-capability gate can later permit provisioning on a guest-rootfs backend without changing the operator-facing entrypoint or exit-code meanings.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` and `scripts/substrate/install-substrate.sh` staging under `$SUBSTRATE_HOME/scripts/substrate/…`
    - `crates/shell/src/builtins/world_enable/runner/paths.rs` helper discovery contract (dev installs)
  - Conflict: yes
  - Resolution (explicit):
    - Treat helper staging/discovery as orthogonal to provisioning semantics; ensure any new helper-script flags or helper/script coupling required by `--provision-deps` are reflected in what dev-install stages (and covered by dev-install tests).

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` and `scripts/substrate/world-enable.sh` “enable later” workflows
    - `crates/shell/src/builtins/world_enable/` error/remediation shape
  - Conflict: yes
  - Resolution (explicit):
    - Keep missing-artifact “enable later” fixes and world-deps provisioning semantics separable: provisioning logic must not introduce new late failures that obscure the “world-agent missing” remediation path.

- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
  - Overlap surfaces:
    - `substrate health` / `substrate shim doctor` operator guidance and “next steps” messaging for world-deps
  - Conflict: yes
  - Resolution (explicit):
    - Diagnostics ADRs own disabled/skip behavior and “why disabled” attribution; provisioning ADRs own enabled-mode remediation and must be compatible with the disabled short-circuit (no provisioning guidance when world is disabled).

- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs` execution semantics and guard rails (security-sensitive)
  - Conflict: yes
  - Resolution (explicit):
    - Coordinate provisioning-profile changes with process-exec tracing work so provisioning executions remain traceable and do not break the world-agent request/response invariants assumed by tracing parity.

- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared script) and installer messaging about package managers
  - Conflict: yes
  - Resolution (explicit):
    - Non-overlap boundary: ADR-0031 owns host installer package-manager selection; ADR-0030 owns in-world provisioning-time system packages and must not add host PATH-based manager selection semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` and `docs/INSTALLATION.md` (if installer output/persistence needs updates for provisioning workflows)
  - Conflict: yes
  - Resolution (explicit):
    - Keep host metadata persistence (install_state) isolated to its pack; if ADR-0030 edits installer messaging, do not refactor detection/persistence code paths owned by ADR-0031/ADR-0032 packs.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `substrate world enable` “home” semantics (`--home`, `SUBSTRATE_HOME`) and legacy knob avoidance
    - config key naming and exit-code taxonomy alignment
  - Conflict: no
  - Resolution (explicit):
    - Ensure `--provision-deps` does not introduce new config/env precedence rules and does not reintroduce removed legacy knobs; keep exit-code meanings aligned to taxonomy.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - shared provisioning entrypoint (`substrate world enable --provision-deps`)
    - shared runtime fail-early/remediation posture for system-package methods
    - `crates/world-agent/src/service.rs` provisioning execution isolation seam
    - shared doc update targets and shared scripts (`scripts/substrate/world-enable.sh`, `scripts/substrate/install-substrate.sh`)
  - Conflict: yes
  - Resolution (explicit):
    - Dependency: this pack (`world-deps-apt-provisioning`) is the baseline for APT invocation and provisioning posture; the non-APT pack must not redefine APT semantics and must reference WDAP0/WDAP1 specs for APT-specific behavior.
    - Single-authority rule: the shared `--provision-deps` UX must have exactly one authoritative contract doc; if the non-APT pack needs to make it manager-aware, it must do so by editing/deferring to that single authority.

- Planning Pack: `docs/project_management/packs/draft/world-disabled-diagnostics/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` and operator guidance surfaces for world-deps when world is disabled
  - Conflict: yes
  - Resolution (explicit):
    - World-disabled packs own the “disabled/skip probes” behavior; provisioning packs own enabled-mode remediation and must not print provisioning guidance in disabled mode.

- Planning Pack: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` operator text and JSON evolution
  - Conflict: yes
  - Resolution (explicit):
    - Keep provisioning text changes compatible with attribution fields and avoid JSON field collisions; sequence disabled-status work first, then attribution, then provisioning guidance refinements.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/world_enable/runner/paths.rs`
    - `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/dev-uninstall-substrate.sh`
    - staged helper scripts under `$SUBSTRATE_HOME/scripts/substrate/…`
  - Conflict: yes
  - Resolution (explicit):
    - Ensure helper discovery/staging changes and `--provision-deps` semantics do not drift: any helper interface change required by provisioning must be reflected in the staged artifacts and tests for dev-install flows.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` and `scripts/substrate/world-enable.sh` “enable later” workflows
  - Conflict: yes
  - Resolution (explicit):
    - Avoid compounding unrelated changes in the same shared scripts; keep `--provision-deps` integration minimal and compatible with “enable later” artifact staging and remediations.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs`
    - systemd unit hardening/capabilities for world-agent (Linux + macOS guest)
  - Conflict: yes
  - Resolution (explicit):
    - Coordinate provisioning posture changes (profile/guard rails/unit sandbox choices) with tracing parity so new provisioning executions remain observable and do not break capability assumptions (e.g., added capabilities, unit templates).

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared file)
  - Conflict: yes
  - Resolution (explicit):
    - Keep ADR-0030 changes to `install-substrate.sh` limited to world-deps provisioning workflow integration/messaging; do not modify host package-manager detection or selection logic owned by ADR-0031.

- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (shared file)
  - Conflict: yes
  - Resolution (explicit):
    - Keep host metadata persistence and schema evolution isolated to its pack; provisioning pack changes must not refactor install_state persistence paths.

- Planning Pack (archived reference): `docs/project_management/_archived/world_deps_selection_layer/`
  - Overlap surfaces:
    - legacy “world deps provision” surface and its contract expectations
  - Conflict: yes
  - Resolution (explicit):
    - Do not reintroduce `substrate world deps provision` unless explicitly documented as an alias with a compatibility policy; ADR-0030’s entrypoint is `substrate world enable --provision-deps`.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — APT requirement derivation (de-dup + ordering + version-pin conflict policy)
  - DR-0002 — Provisioned-state tracking (probe-only vs state file) and its impact on runtime fail-early/no-op behavior
  - DR-0003 — Provisioning execution isolation model (request `profile` value(s), guard rails, and explicit relationship to `SUBSTRATE_WORLD_REQUEST_PROFILE`)

- Spec updates required (if any):
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md` — update if the Touch Set or operator-doc targets expand beyond the surfaces listed here (single-authority enforcement).
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` — resolve spec-manifest follow-ups #3–#6 (ordering of enable vs provision; runtime scope rules; dry-run semantics; Windows posture).
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` — pin deterministic derivation + APT invocation + backend capability gate and produce testable acceptance criteria.
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` — pin deterministic fail-early triggers + remediation invariants + dry-run behavior and produce testable acceptance criteria.
