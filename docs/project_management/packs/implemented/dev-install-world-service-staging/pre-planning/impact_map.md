# dev-install-world-service-staging — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/dev-install-world-service-staging/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
- Spec manifest:
  - `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/dev-install-world-service-staging/"` (strict packs only).

### Create
- `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/decision_register.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/manual_testing_playbook.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/plan.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/quality_gate_report.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/session_log.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/DIWAS0-spec.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/kickoff_prompts/DIWAS0-code.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/kickoff_prompts/DIWAS0-integ.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS0/kickoff_prompts/DIWAS0-test.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/DIWAS1-spec.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/kickoff_prompts/DIWAS1-code.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/kickoff_prompts/DIWAS1-integ.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/slices/DIWAS1/kickoff_prompts/DIWAS1-test.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh`

### Edit
- `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
- `docs/project_management/packs/draft/dev-install-world-service-staging/tasks.json`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/tests/world_enable.rs`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/world-enable.sh`
- `tests/installers/install_smoke.sh`

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
- Change: Linux `scripts/substrate/dev-install-substrate.sh --no-world` becomes “enable later ready” by still building/staging `world-service` (no provisioning).
  - Direct impact:
    - The “install with `--no-world`, enable later” dev workflow stops requiring a manual `cargo build -p world-service ...` step.
    - Dev install runtime increases on Linux when `--no-world` is used (adds a `world-service` build step).
  - Cascading impact:
    - Dev-installer help/output must reflect the refined meaning of `--no-world` (skip provisioning, but not necessarily skip build/staging).
    - Validation must explicitly assert the staged artifact exists at the contract-defined path(s) after dev install with `--no-world`.
  - Contradiction risks:
    - `--no-world` semantics: other docs/features may treat it as “skip all world-related work”; this ADR requires a deterministic contract (Decision Register DR-0002).
    - Profile confusion: `dev-install --profile debug` vs `substrate world enable --profile release` may produce logs/defaults that look inconsistent unless contract text pins the intended mapping (Decision Register DR-0003).

- Change: `substrate world enable` fails early on Linux when the staged `world-service` artifact is missing, with a single actionable remediation.
  - Direct impact:
    - Operators get a deterministic “missing staged binary” failure instead of late systemd/socket/readiness probe failures that read like general breakage.
  - Cascading impact:
    - The remediation text must be consistent across:
      - CLI stderr output,
      - helper logs (`<home>/logs/world-enable-*.log`),
      - `manual_testing_playbook.md`,
      - `smoke/linux-smoke.sh`,
      - tests (assert the minimum remediation string).
    - `--dry-run` semantics must be pinned for this failure mode: whether dry-run performs the preflight, what it prints, and what exit code it returns.
  - Contradiction risks:
    - Default CLI output currently suppresses helper stdout/stderr unless `--verbose`; if missing-artifact remediation is implemented only inside the helper script, it may be hidden by default. This must be made coherent by contract + tests (DR-0001: preflight locus).

### Config / env vars / paths
- Change: Make the staged artifact path set under the inferred dev “version dir” (`<repo>/target/`) a contract-bearing surface for enable.
  - Direct impact:
    - `substrate world enable` can treat dev installs and production installs uniformly by checking the same “release root” layout.
  - Cascading impact:
    - The contract must pin:
      - the authoritative staged path set (`bin/world-service`, `bin/linux/world-service`),
      - whether BOTH are required or ONE is sufficient (ADR currently asserts both),
      - the overwrite/idempotency policy when the staged path(s) already exist (DR-0004).
    - Tests must assert the chosen “path accept set” and overwrite policy deterministically.
  - Contradiction risks:
    - `cargo clean` removes `<repo>/target/bin/...` (dev version dir). This ADR does not solve that brittleness; it must not over-promise robustness unless coordinated with ADR-0034 (helper discovery stability) or a follow-up “bundle parity” ADR.

- Change: Keep config semantics stable: `world.enabled` remains `false` after dev-install `--no-world` and flips to `true` only after successful enable verification.
  - Direct impact:
    - Existing world-disabled behavior remains (no silent provisioning; no implicit enable).
  - Cascading impact:
    - `substrate world enable --home <path>` must remain the only state root (ADR-0003); success paths must write to `<home>/config.yaml` and `<home>/env.sh` only after verification.
  - Contradiction risks:
    - If the enable path starts doing any “build” work (Option B in ADR-0035), it can violate the “provisioning-only” expectation and may require additional contract guards. ADR-0035 selected staging during dev-install, but the contract must still explicitly forbid running `cargo build` under `sudo`.

### Policy / isolation / security posture
- Change: Fail-closed preflight prevents privileged side effects when the staged `world-service` artifact is missing.
  - Direct impact:
    - Avoids host mutation/provisioning attempts when the binary artifact root cause is missing.
  - Cascading impact:
    - Preflight must run before any privileged/systemd-related steps in the enable path (and before any helper-driven prerequisite installation).
    - Exit-code mapping must be taxonomy-aligned and consistent across CLI, smoke, and playbooks (missing executable is typically exit `3`).
  - Contradiction risks:
    - Other in-flight work grows `substrate world enable` provisioning responsibilities (ADR-0030/ADR-0033). This feature’s preflight must remain orthogonal and must not create a policy/OS-mutation surface beyond missing-artifact checks.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `substrate world enable --home` semantics + `$SUBSTRATE_HOME` consistency
    - env-script ownership (`env.sh` vs `manager_env.sh`)
    - meaning/precedence of `--no-world` and `world.enabled`
  - Conflict: no (this ADR does not change precedence rules), but must remain aligned
  - Resolution (explicit):
    - Ensure enable preflight + remediation do not introduce new env vars, do not reintroduce `--prefix`, and do not write world exports into `manager_env.sh`.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `substrate world enable` “enable later” workflow reliability
  - Conflict: no (adjacent sharp-edge fixes), but shared dev-install surface + shared “target/ as version dir” coupling
  - Resolution (explicit):
    - Keep contracts non-overlapping: ADR-0034 owns helper discovery robustness across `cargo clean`; ADR-0035 owns staging/proving the `world-service` artifact exists for enable provisioning.
    - Sequence to reduce merge conflict risk: land ADR-0034 dev-installer refactors first (helper staging), then land ADR-0035 staging changes (world-service build/stage) as a smaller follow-on edit.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `substrate world enable` CLI + helper invocation
    - `scripts/substrate/world-enable.sh` (shared helper script)
  - Conflict: no (different intent), but shared surface + interface drift risk
  - Resolution (explicit):
    - Keep missing-`world-service` preflight independent of any `--provision-deps` flag work so provisioning UX can evolve without breaking the “artifact missing” remediation contract.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `substrate world enable` flag surface growth
    - `scripts/substrate/world-enable.sh` + `crates/shell/src/builtins/world_enable/`
  - Conflict: no, but shared helper script surface
  - Resolution (explicit):
    - Ensure helper interface changes remain backwards compatible within the same version dir, and ensure dev-install staging continues to provide the helper(s) used by world enable (directly or via symlink) without version skew.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `crates/shell/src/builtins/world_enable/runner/paths.rs`
    - `crates/shell/tests/world_enable.rs`
  - Conflict: yes (shared files; both change “enable later” reliability)
  - Resolution (explicit):
    - Sequence boundary: land helper discovery/staging first, then land world-service staging. Share/align overwrite + ownership guard decisions to avoid divergent dev-install contracts.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/tests/world_enable.rs`
  - Conflict: no (shared entrypoint), but merge-conflict risk is high
  - Resolution (explicit):
    - Keep the missing-artifact preflight logic isolated (single helper function + targeted tests) so it is easy to rebase across `world enable` flag surface changes.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/tests/world_enable.rs`
  - Conflict: no (different intent), but shared helper script surface
  - Resolution (explicit):
    - Same as `world-deps-apt-provisioning`: keep missing-artifact preflight small and orthogonal so manager-aware provisioning work can proceed without semantic drift.

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (sourced by `scripts/substrate/world-enable.sh`)
  - Conflict: no (different section of the script), but shared exit-code and messaging posture risk
  - Resolution (explicit):
    - Avoid refactors in shared installer helpers when implementing this feature; constrain changes to world-enable preflight and dev-installer staging so pkg-manager detection/exit-code work can land independently.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Decide where missing-`world-service` preflight is implemented (Rust runner vs helper script vs installer helper), given default helper output suppression and the “fail before privileged steps” invariant.
  - DR-0002 — Define dev meaning of `scripts/substrate/dev-install-substrate.sh --no-world` (skip provisioning only vs skip all world-related build outputs).
  - DR-0003 — Define profile mapping for staging `world-service` (`release` only vs match `dev-install --profile`), and reconcile any UX/defaults drift.
  - DR-0004 — Define overwrite/idempotency rules for staged `bin/(linux/)world-service` paths under `<repo>/target/bin/`.
- Spec updates required (if any):
  - `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md` — reconcile internal option label inconsistency and update Related Docs paths to the canonical pre-planning locations used by this pack (`pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`).
  - `docs/project_management/packs/draft/dev-install-world-service-staging/tasks.json` — reconcile `behavior_platforms_required` with Linux-only behavior change (either narrow to Linux or add deterministic “no change/unsupported” validations for macOS/Windows); add `meta.checkpoint_boundaries`; populate triad tasks + kickoff prompt paths.
  - `docs/project_management/packs/draft/dev-install-world-service-staging/contract.md` — pin exit code for missing staged binary (taxonomy-aligned) + minimum remediation string; pin whether both staged paths are required and the `--dry-run` behavior for the missing-artifact path.
