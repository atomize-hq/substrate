# world-deps-apt-provisioning — workstream triage

## Inputs + evidence
- Canonical inputs used: `spec_manifest.md`, `impact_map.md`, `minimal_spec_draft.md`, `tasks.json`, `ci_checkpoint_plan.md`
- Lift (pack-derived; strict pack `meta.slice_spec_version: 2`):
  - `Lift Score (v1): 81`, `Estimated slices: 7`, `Confidence: low`
  - Triggers: `split_required:estimated_slices>3`, `likely_split:touch_files_sum>12`, `likely_split:lift_score>24`
  - Source: `logs/workstream-triage/pm_lift_pack.txt` / `logs/workstream-triage/pm_lift_pack.json`
- Lift (discovery; ADR intake):
  - `Lift Score (v1): 24`, `Estimated slices: 2`, `Confidence: low`
  - Source: `logs/workstream-triage/pm_lift_intake.txt` / `logs/workstream-triage/pm_lift_intake.json`
- Stable completion sentinels:
  - `logs/spec-manifest/last_message.md`
  - `logs/impact-map/last_message.md`
  - `logs/min-spec-draft/last_message.md`
  - `logs/CI-checkpoint/last_message.md`

## Proposed workstreams (for full planning)

### WDAP-PWS-contract — Contract + decision register (operator invariants first)
- Goal: make the operator-facing contract deterministic so slice specs/tests/docs can be authored without guesswork.
- Owned surfaces:
  - Planning docs: `contract.md`, `decision_register.md` (DR-0001/2/3), `plan.md`, `tasks.json` (slice/triad wiring)
  - UX invariants referenced by code/docs: exit-code tables (0/3/4/5), remediation wording (must include exact `substrate world enable --provision-deps`), backend support matrix + “no host OS mutation” wording
  - Overlap to coordinate: `crates/shell/src/builtins/health.rs` (ADR-0036)
- Dependencies: none (start here); unblocks WDAP-PWS-runtime_fail_early/WDAP-PWS-provisioning_wiring/WDAP-PWS-docs_validation.
- Slices/triads to plan:
  - Contract+decisions triad that produces the authoritative tables and DR outcomes that every other slice must reference.

### WDAP-PWS-runtime_fail_early — Runtime fail-early for APT items (`world deps current sync|install`)
- Goal: enforce “no APT/dpkg at runtime” with deterministic exit code + remediation for APT-backed items that are in-scope.
- Owned surfaces (code/test targets per `impact_map.md`):
  - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - Tests: `crates/shell/tests/world_deps_apt_install_wdp5.rs` (update/repurpose per new contract)
- Dependencies:
  - Requires WDAP-PWS-contract to lock the “in-scope” rule + remediation text + exit-code mapping.
- Slices/triads to plan:
  - `WDAP1` (or equivalent if renamed): code+tests+doc-snippets triad for runtime short-circuit behavior.

### WDAP-PWS-provisioning_wiring — Provisioning workflow in `substrate world enable --provision-deps` (shell-side)
- Goal: specify and implement the shell-facing provisioning UX: flag parsing, package-derivation plumbing, dry-run/verbose semantics, and dispatch gating by backend capability.
- Owned surfaces (code/test targets per `impact_map.md`):
  - CLI + dispatch: `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - World enable runner/logging: `crates/shell/src/builtins/world_enable/runner.rs`, `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
  - Bootstrapping: `crates/shell/src/execution/home_bootstrap.rs`
  - Tests: `crates/shell/tests/world_enable.rs`
- Dependencies:
  - Requires WDAP-PWS-contract for flag semantics + exit-code mapping + output invariants.
  - Requires WDAP-PWS-world_agent_profile (or an explicit temporary seam) for the provisioning execution profile model.
- Slices/triads to plan:
  - `WDAP0` (provisioning UX + derivation + dry-run determinism) and any split slices recommended below.

### WDAP-PWS-world_agent_profile — World-agent provisioning profile isolation + guard rails
- Goal: define and implement a safe provisioning-time execution profile (APT/dpkg allowed only when operator-invoked) without widening hardened runtime surfaces.
- Owned surfaces (per `impact_map.md`):
  - `crates/world-agent/src/service.rs`
  - Any required profile selection / request guard rails implied by ADR-0030 + DR-0003
- Dependencies:
  - Requires WDAP-PWS-contract to decide DR-0003 (profile selection + guard rails).
  - Must coordinate with `docs/project_management/packs/active/world_process_exec_tracing_parity/` (explicit conflict in `spec_manifest.md` / `impact_map.md`).
- Slices/triads to plan:
  - A dedicated slice for provisioning profile isolation (recommended as its own seam due to security + cross-pack churn).

### WDAP-PWS-docs_validation — Docs + installer + validation artifacts (keep operator guidance coherent)
- Goal: propagate the contract into operator docs, installer scripts, and validation artifacts with minimal merge risk.
- Owned surfaces (per `impact_map.md`):
  - Installer scripts: `scripts/substrate/world-enable.sh`, `scripts/substrate/install-substrate.sh`
  - Docs: `docs/reference/world/deps/README.md`, `docs/internals/world/deps.md`, `docs/WORLD.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/COMMANDS.md`, `docs/cross-platform/wsl_world_troubleshooting.md`
  - Cross-pack alignment: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - Validation artifacts to create: `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`
- Dependencies:
  - Requires WDAP-PWS-contract wording and WDAP-PWS-runtime_fail_early/WDAP-PWS-provisioning_wiring behavior semantics to be stable.
  - Script edits must be narrowly scoped to reduce conflict with `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
- Slices/triads to plan:
  - A docs+validation slice (or two slices if needed): operator docs updates + smoke/manual playbook + CI checkpoint wiring once `tasks.json` triads exist.

## Sequencing + gates (full planning)
- Gate G1 (contract determinism): land WDAP-PWS-contract outputs (DR-0001/2/3 + `contract.md` tables) before finalizing any slice specs.
- Gate G2 (runtime scope rule): decide the single “APT-backed items in-scope” rule for `deps current sync|install` before finalizing WDAP-PWS-runtime_fail_early (`WDAP1`).
- Gate G3 (provisioning isolation model): finalize provisioning execution profile selection + guard rails (WDAP-PWS-world_agent_profile) before finalizing WDAP-PWS-provisioning_wiring (`WDAP0`) dispatch semantics.
- Gate G4 (planning wiring): populate `tasks.json` with slice triads + deps; update `ci_checkpoint_plan.md` boundaries if the slice skeleton changes; ensure planning-lint would be satisfiable.

## Slice skeleton recommendations (required)
Starting point (from `minimal_spec_draft.md`): `WDAP0` (provisioning) + `WDAP1` (runtime fail-early).

Given impact-map-derived lift (est. **7** slices; split triggers), recommend expanding the skeleton to isolate high-churn/security seams:
- SPLIT `WDAP0` into:
  - KEEP `WDAP0`: shell UX + derivation + `--dry-run`/`--verbose` determinism for `world enable --provision-deps`
  - ADD `WDAP2`: world-agent provisioning profile isolation + guard rails (security-sensitive; coordinates with exec-tracing parity pack)
  - ADD `WDAP3`: installer/script wiring + backend support gating + “no host OS mutation” unsupported posture (Linux host-native + Windows posture clarity)
- KEEP `WDAP1` as the runtime fail-early slice, but require its spec to pin the “in-scope” rule and the exact remediation invariants.
- ADD `WDAP4`: docs alignment + cross-pack contract coherence (`world-deps-packages-bundles-contract/contract.md` + operator reference docs)
- ADD `WDAP5`: validation artifacts (manual playbook + per-platform smoke) + CI checkpoint wiring once `tasks.json` triads exist

If full planning chooses to keep only `WDAP0..WDAP1`, explicitly justify the merges and ensure each resulting slice has one dominant seam (avoid a “god slice” spanning world-agent hardening + shell UX + docs + scripts).

## Risks + unknowns (carry into full planning)
- Runtime “in-scope” ambiguity: enabled set vs explicit args vs union (must be singular + testable).
- Provisioning determinism: idempotency definition, APT invocation contract, ordering/dedup rules, retries/timeouts, and `--verbose` stream/stability.
- Backend matrix uncertainty: Windows+WSL support for `world enable` is assumed; must define supported vs unsupported contract + remediation.
- Security-sensitive seam: provisioning profile must not widen hardened runtime; keep guard rails explicit and coordinate `crates/world-agent/src/service.rs` changes with the exec-tracing parity pack.
- Cross-pack contract drift: reconcile and update `world-deps-packages-bundles-contract/contract.md` semantics that imply runtime APT behavior.

## Follow-ups
- If adopting `WDAP2..WDAP5`, update `spec_manifest.md`, `impact_map.md`, and `ci_checkpoint_plan.md` to include the additional slice specs and any new validation gates.
- Add explicit coordination note for ADR-0036 overlap (`crates/shell/src/builtins/health.rs`) so diagnostics never suggest contradictory “next steps”.
