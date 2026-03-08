# add-non-apt-system-package-provisioning-support — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Spec manifest:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"` (strict packs only).

### Create
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/session_log.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/quality_gate_report.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP0/NASPP0-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP1/NASPP1-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP2/NASPP2-spec.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
- `docs/project_management/packs/sequencing.json`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`
- `docs/reference/world/deps/README.md`
- `docs/reference/world/deps/authoring_packages.md`
- `docs/internals/world/deps.md`
- `docs/WORLD.md`
- `docs/CONFIGURATION.md`
- `docs/COMMANDS.md`
- `docs/USAGE.md`
- `docs/INSTALLATION.md`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
- `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/install-substrate.sh`
- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/errors.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/world-agent/src/service.rs`
- `crates/shell/tests/world_enable.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `crates/shell/tests/world_deps_inventory_views.rs`
- `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- `crates/shell/tests/world_deps_present_semantics_wdh1.rs`

### Deprecate
- None

### Delete
- None

## Implementation surface note

- Direct implementation work is concentrated in `crates/shell/`, `crates/world-agent/`, shared installer/helper scripts, and operator/internal docs.
- No direct code change is implied by ADR-0033 itself under:
  - `crates/world/`
  - `crates/world-mac-lima/`
  - `crates/world-windows-wsl/`
  - `crates/shim/`
- Those backends remain dependency surfaces only. Arch-capable guest images, Linux guest-rootfs enablement, or Windows `substrate world enable` support stay in adjacent tracks rather than expanding this feature’s scope.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: `substrate world enable --provision-deps [--dry-run] [--verbose]` becomes manager-aware for world OS system packages.
  - Direct impact:
    - Supported Debian or Ubuntu worlds continue to provision via APT.
    - Supported Arch-family worlds provision via `pacman`.
    - Unsupported or unknown worlds fail with exit `4`, manager-aware remediation, and explicit no-host-mutation guidance.
  - Cascading impact:
    - The command can no longer be documented as APT-only anywhere in the repo.
    - `scripts/substrate/world-enable.sh` currently runs `substrate world deps current sync` after provisioning; the provisioning path must explicitly suppress that runtime sync or it will immediately conflict with the new runtime fail-early posture for `install.method=apt|pacman`.
    - `--dry-run` and `--verbose` must print the detected-manager inputs and the derived requirement sets without creating a second operator workflow or requiring manual env-var overrides.
    - Current operator guidance in `docs/COMMANDS.md`, `docs/USAGE.md`, and `docs/INSTALLATION.md` still describes `substrate world enable` without `--provision-deps` and still contains `--prefix` drift; the new flag work must fix that drift instead of extending it.
  - Contradiction risks:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` and `WDAP0`/`WDAP1` already describe the same `--provision-deps` surface as APT-only.
    - `docs/project_management/_archived/world_deps_selection_layer/S2-spec-system-packages-provisioning.md` still models the older `substrate world deps provision` namespace.
  - Resolution options (A/B):
    - Option A: this pack’s `contract.md` becomes the single post-ADR-0033 authority for manager-aware system-package provisioning; the ADR-0030 planning-pack docs become APT-baseline or defer-only docs.
    - Option B: the ADR-0030 planning-pack docs remain the single authority and this pack documents only pacman-specific deltas.
    - Selected: Option A.

- Change: runtime `substrate world deps current sync`, `sync --all`, and `install <ITEM...>` remain fail-closed for all system-package methods, not just APT.
  - Direct impact:
    - Runtime `world deps` never invokes `apt`, `apt-get`, or `pacman`.
    - Any unsatisfied in-scope `install.method=apt` or `install.method=pacman` requirement exits `4` and points to the exact remediation command `substrate world enable --provision-deps`.
  - Cascading impact:
    - Existing runtime plan output, applied-status expectations, and wrapper-generation paths must remain deterministic when system-package-backed packages are already satisfied and only user-space/script work continues.
    - `scripts/substrate/install-substrate.sh --sync-deps` can no longer treat `substrate world deps current sync` as a universal “finish provisioning” step when enabled system-package items are present; its warning/remediation text must become provisioning-aware.
    - Mixed enabled `apt` and `pacman` requirement sets must have one consistent outcome for `--dry-run`, `--verbose`, non-dry-run provisioning, `sync`, and `install <ITEM...>`.
  - Contradiction risks:
    - Current code in `crates/shell/src/builtins/world_deps/surfaces.rs` still computes runtime APT install plans and emits `apt-get` execution.
    - `docs/reference/world/deps/README.md` and `docs/internals/world/deps.md` still frame APT as a hardened-world limitation rather than a provisioning-time-only system-package rule.
  - Resolution options (A/B):
    - Option A: fail the whole command when any enabled system-package method mismatches the detected world OS manager; `--dry-run` still prints the derived requirement sets and the mismatch cause.
    - Option B: provision or accept the matching manager subset and ignore or defer the mismatched subset.
    - Selected: Option A.

### Config / env vars / paths
- Change: the world-deps package schema grows an additive `install.method=pacman` branch with `install.pacman[]`, but the feature introduces no new config key and no new environment variable.
  - Direct impact:
    - Inventory authors can express Arch-family world OS packages explicitly instead of encoding pacman behavior indirectly through scripts or manual guidance.
    - Operators continue using the existing enabled-set/config merge model; there is no new config namespace for package-manager selection.
  - Cascading impact:
    - `crates/shell/src/builtins/world_deps/inventory.rs`, list/show outputs, and schema-validation tests must all treat `pacman` as a first-class install method alongside the existing `apt|script|manual` contract.
    - `docs/reference/world/deps/authoring_packages.md` must stop implying that script installs are the only authorable alternative to APT for system packages.
    - The pack must reconcile its feature-local schema spec with the implemented world-deps contract and ADR-0011 so the post-merge repo has one authoritative system-package schema.
  - Contradiction risks:
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` and `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` currently list `install.method=apt|script|manual`.
    - Host installer ADRs (`ADR-0031`, `ADR-0032`) also use package-manager vocabulary and `/etc/os-release`, which can be confused with the world OS manager vocabulary if the docs do not keep host-vs-world boundaries explicit.
  - Resolution options (A/B):
    - Option A: the feature-local `world-deps-system-package-schema-spec.md` is the planning authority for the new `pacman` branch, and landing the feature also updates the implemented world-deps contract and ADR-0011 so there is one post-merge authority.
    - Option B: skip feature-local schema authority and edit only the implemented world-deps contract and ADR-0011.
    - Selected: Option A.

### Policy / isolation / security posture
- Change: provisioning reuses the existing `world-deps-provision` request profile and performs all package-manager detection inside the world; runtime world execution stays hardened.
  - Direct impact:
    - Pacman provisioning does not create a second privileged path or a second request-profile name.
    - Host PATH detection, host install-state metadata, and host package-manager selection logic remain out of scope for world OS provisioning.
  - Cascading impact:
    - `crates/world-agent/src/service.rs` and the world-ops request builders must become manager-agnostic while preserving the existing `world-deps-provision` guard rails.
    - `/etc/os-release` and package-manager-presence probes must run inside the world and must fail closed when the signals disagree or the manager is unknown.
    - Windows posture must remain explicit and testable instead of leaving the current ADR assumption unresolved.
  - Contradiction risks:
    - Current world-agent commentary still describes `world-deps-provision` as an APT/dpkg-specific escape hatch.
    - Reusing host detection or persisted host package-manager data from ADR-0031/0032 would leak host facts into a guest-only provisioning decision.
  - Resolution options (A/B):
    - Option A: add a second profile or rely on `SUBSTRATE_WORLD_REQUEST_PROFILE` overrides for pacman provisioning.
    - Option B: keep `world-deps-provision` as the only provisioning profile, ignore operator overrides for provisioning commands, and require the in-world probe signals to agree or fail closed.
    - Selected: Option B.

- Change: Windows posture must stop being an ADR assumption and become a concrete feature boundary.
  - Direct impact:
    - Operators get a stable exit code and remediation path on Windows instead of ambiguous “if or when” wording.
  - Cascading impact:
    - The feature-local contract, manual playbook, and Windows smoke must all validate the same Windows rule.
    - `docs/INSTALLATION.md`, `docs/USAGE.md`, and shared world-enable docs must not imply that ADR-0033 adds Windows `substrate world enable` support.
  - Contradiction risks:
    - `crates/shell/src/builtins/world_enable/runner.rs` still rejects Windows today.
    - ADR-0033 currently phrases Windows behavior as an assumption rather than a final contract.
  - Resolution options (A/B):
    - Option A: treat Windows as supported inside WSL for this feature and expand scope into Windows `world enable` support.
    - Option B: keep Windows unsupported for this feature, return exit `4`, and preserve guest-only provisioning as a future dependency.
    - Selected: Option B.

### Operator-doc update targets (exact paths + headings)
- `docs/reference/world/deps/README.md`
  - `## Commands you will use`
  - `## APT packages (current limitation in hardened worlds)`
- `docs/reference/world/deps/authoring_packages.md`
  - `## Schema sketch (version 1)`
  - `## Package file layout`
- `docs/internals/world/deps.md`
  - `## High-level flow`
  - `## APT installs vs hardening`
- `docs/COMMANDS.md`
  - `### world Subcommand`
- `docs/USAGE.md`
  - `### World Commands`
- `docs/INSTALLATION.md`
  - `### Installer Metadata & Cleanup`
  - `## Installer Options Reference`
  - the Linux `substrate world enable` / `--sync-deps` narrative in `## Quick Install (Release Bundles)`
- `docs/WORLD.md`
  - `### World Dependencies (world deps)`
  - `## 8) Environment Variables`
- `docs/CONFIGURATION.md`
  - the `SUBSTRATE_WORLD_REQUEST_PROFILE` row

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `substrate world enable --provision-deps`
    - runtime fail-early semantics for system-package-backed world-deps items
    - `world-deps-provision` request-profile guard rails
  - Conflict: yes
  - Resolution (explicit):
    - Keep ADR-0030 as the APT baseline and make this pack’s `contract.md` the single manager-aware authority for the shared CLI surface.
    - Edit the ADR-0030 planning-pack contract/spec docs so they defer to the manager-aware contract instead of competing with it.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - Linux no-host-mutation posture
    - future Linux guest support for system-package provisioning
  - Conflict: no
  - Resolution (explicit):
    - Keep Linux host-native unsupported in this feature.
    - Treat any Linux Arch-capable guest image or guest-rootfs support as a dependency owned by ADR-0009 or later backend-image work, not by ADR-0033.

- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing
    - package-manager vocabulary (`apt-get`, `pacman`, `dnf`, `yum`, `zypper`)
    - shared installer docs and scripts
  - Conflict: yes
  - Resolution (explicit):
    - Option A: reuse the host installer detection contract end-to-end for world OS probing.
    - Option B: reuse only the manager spellings and keep world detection in-world and guest-scoped.
    - Selected: Option B.

- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - persisted manager vocabulary
    - `docs/INSTALLATION.md`
    - host vs world package-manager explanations
  - Conflict: no
  - Resolution (explicit):
    - Keep persisted install-state metadata host-scoped only.
    - Do not let world provisioning consume or derive from host install-state package-manager fields.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - helper-script staging and discovery for `substrate world enable`
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
  - Conflict: no
  - Resolution (explicit):
    - Keep helper discovery/staging orthogonal to manager-aware provisioning behavior.
    - Any new helper flags or helper/runner interface changes introduced here must match the staged helper pair that ADR-0034 installs.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `substrate world enable` shared runner/helper files
    - missing-artifact preflight before provisioning
  - Conflict: no
  - Resolution (explicit):
    - Preserve the small preflight boundary from ADR-0035 and layer manager-aware provisioning on top of it.
    - Do not make pacman support depend on dev-install staging behavior.

- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs`
    - `/v1/execute` request/response behavior
    - provisioning profile visibility in trace/diagnostic flows
  - Conflict: yes
  - Resolution (explicit):
    - Keep the provisioning-profile change additive.
    - Do not rename or fork the `world-deps-provision` profile; tracing work must continue to observe provisioning executions under the existing profile identity.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
    - `crates/shell/src/builtins/world_enable/`
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/world-agent/src/service.rs`
  - Conflict: yes
  - Resolution (explicit):
    - This pack owns the manager-aware post-ADR-0033 contract.
    - The APT pack remains the APT-baseline planning history and must be edited so it does not remain a competing authority for the shared CLI surface.

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
    - package-manager vocabulary
  - Conflict: no
  - Resolution (explicit):
    - Keep host prerequisite detection separate from world OS provisioning.
    - Shared-file edits must stay scoped to provisioning messaging and not redefine host detection or wrapper exit-code behavior.

- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `docs/INSTALLATION.md`
    - package-manager vocabulary
    - `/etc/os-release` identity terms
  - Conflict: no
  - Resolution (explicit):
    - Align spellings only.
    - Keep persisted host metadata and in-world probe results as separate concepts.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/tests/world_enable.rs`
  - Conflict: no
  - Resolution (explicit):
    - Keep pacman provisioning changes orthogonal to missing-artifact staging and preflight logic.
    - Shared-file edits must preserve the “artifact missing” remediation contract.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
    - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Sequence helper-discovery/staging changes ahead of, or alongside, provisioning-flag wiring so the helper pair that gets staged remains interface-compatible with the new `--provision-deps` behavior.

- Planning Pack: `docs/project_management/packs/draft/world-disabled-diagnostics/`
  - Overlap surfaces:
    - remediation messaging when the world is disabled
    - downstream operator guidance that points at `substrate world enable`
  - Conflict: yes
  - Resolution (explicit):
    - Disabled-mode diagnostics own the “do not suggest provisioning when the world is disabled” boundary.
    - ADR-0033 owns enabled-mode remediation for system-package-backed items only.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs`
    - `/v1/execute` request-profile handling
  - Conflict: yes
  - Resolution (explicit):
    - Keep provisioning-profile behavior additive and trace-visible.
    - Do not introduce a second provisioning profile or alternate transport contract.

### Relevant Archived Packs
- Archived Pack: `docs/project_management/_archived/world_deps_selection_layer/`
  - Overlap surfaces:
    - explicit system-package provisioning workflow
    - Linux no-host-mutation posture
    - operator remediation for runtime system-package failure
  - Conflict: yes
  - Resolution (explicit):
    - Option A: revive `substrate world deps provision` as the modern command surface.
    - Option B: keep the archived command namespace retired and consolidate on `substrate world enable --provision-deps`.
    - Selected: Option B.

## Follow-ups (explicit)

- Decision Register entries required:
  - `DR-0001 — exact in-world probe precedence and derived-manager vocabulary` — pin the `/etc/os-release` + manager-presence algorithm, including the fail-closed disagreement rule selected here.
  - `DR-0002 — pacman requirement normalization and idempotency` — pin duplicate handling, stable ordering, and the no-op detector for already-satisfied pacman requirements.
  - `DR-0003 — pacman command construction and privilege model` — pin the exact invocation contract, non-interactive flags, and error classification under `world-deps-provision`.
  - `DR-0004 — mixed `apt` + `pacman` set reporting` — pin the exact stdout/stderr split for `--dry-run`, `--verbose`, runtime `sync`, and runtime `install <ITEM...>` under the fail-whole-command rule selected here.

- Spec updates required (if any):
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` — record the selected contract-authority rule, selected schema-authority rule, and selected Windows unsupported posture so the manifest no longer carries those as open follow-ups.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` — encode the selected manager-aware authority boundary, Windows exit-`4` posture, fail-whole-command mismatch rule, and exact remediation text requirements.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md` — encode pacman normalization, forbidden-field rules, and the additive-extension rule relative to the implemented world-deps contract.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP0/NASPP0-spec.md` — enumerate the exact in-world probe inputs, precedence, and fail-closed disagreement behavior.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP1/NASPP1-spec.md` — enumerate pacman command construction, no-op detection, and the shared `world-deps-provision` request-profile usage.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP2/NASPP2-spec.md` — enumerate the exact doc headings and runtime remediation assertions that must be reconciled.
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` — replace the Windows assumption with the selected unsupported posture and link the final authority boundaries chosen in this impact map.
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` — convert the pack from competing shared-contract authority to an APT-baseline/defer-only role.
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` — defer shared manager-aware CLI semantics to the new contract while preserving APT-specific baseline requirements.
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` — defer shared runtime system-package semantics to the new contract while preserving APT-specific baseline requirements.
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` — absorb the landed `pacman` branch so the repo has one post-merge schema authority for system-package methods.
  - `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` — keep the approved ADR in parity with the landed schema authority once `install.method=pacman` is added.
