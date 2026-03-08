# world-deps-apt-provisioning — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`

External authoritative inputs (this feature MUST NOT redefine these surfaces):
- Exit code taxonomy:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- World-deps inventory/enabled contract surfaces, including the `install.method=apt` discriminator and `install.apt[]` schema:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- World-agent endpoint baselines (`/v1/execute`, `/v1/execute/stream`, `/v1/stream`) and world-deps CLI surface list (baseline only; this feature defines the behavioral delta):
  - `docs/WORLD.md`
- Environment variable registry (baseline only):
  - `docs/CONFIGURATION.md` (includes `SUBSTRATE_WORLD_REQUEST_PROFILE`)

## Slice IDs (canonical)

ADR-0030 uses placeholder slice IDs (`C0`, `C1`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `WDAP` (derived from “World Deps APT Provisioning”)
- `WDAP0` — provisioning surface for APT requirements (ADR-0030 C0)
- `WDAP1` — runtime fail-early + remediation for APT items (ADR-0030 C1)

## Required spec documents (authoritative)

Every backticked token in this section is an existing document or directory path. Non-path identifiers, commands, AC IDs, and slice labels are described in prose only.

Spec templates used while authoring this pack:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
  - Owns required-doc selection, the authority map, follow-ups, and the canonical slice set for this feature.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`
  - Owns the touch set, cascading implications, cross-queue conflicts, and exact operator-doc update targets.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
  - Owns checkpoint grouping, accepted checkpoint boundaries, and CI gate cadence.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
  - Owns the execution runbook, accepted slice ordering, and required validation evidence.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
  - Owns task IDs, dependency graph, automation metadata, and checkpoint boundary wiring.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`
  - Owns the append-only planning and execution log.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/quality_gate_report.md`
  - Owns the planning quality gate outcome recorded before execution triads begin.

### Feature contract and decision docs (required)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
  - Owns DR-0001, DR-0002, and DR-0003 plus the impacted surfaces each decision constrains.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - Owns the operator-facing CLI, remediation, exit-code, platform, and safety contract for this feature.

### Validation artifacts (required)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
  - Owns deterministic manual validation setup, commands, expected output, and exit-code expectations.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
  - Owns Linux smoke validation for provisioning posture and runtime fail-early behavior.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
  - Owns macOS smoke validation for provisioning posture and runtime fail-early behavior.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`
  - Owns Windows smoke validation for provisioning posture and runtime fail-early behavior.

### Slice specs (required)

Accepted full-planning slice order is WDAP0 then WDAP1.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
  - Owns provisioning-time APT requirement derivation, guest-only execution posture, and helper and installer ordering invariants.
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
  - Owns runtime fail-early behavior, remediation invariants, and operator-doc and upstream contract reconciliation targets.

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0030 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | flags, defaults, success/no-op semantics, exit codes, remediation invariants, examples |
| Provisioning requirement derivation (APT packages from effective enabled world-deps set) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | enabled-set boundary; `install.method=apt` filter; de-dup rules; ordering; version conflict policy (linked to DR-0001) |
| Provisioning APT invocation contract (idempotency, mutation boundary, non-interactive invocation) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | exact command(s) + flags; idempotency definition; error→exit mapping; unsupported-backend posture |
| Provisioning execution isolation model (provisioning-time mutation without weakening hardened runtime) | `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` | A/B options; chosen selection; constraints; pointers to the constrained spec sections that implement the selection |
| Provisioning `--dry-run` and `--verbose` contract | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | exact “no mutation” definition; minimum guaranteed output content; stream requirements; stability posture |
| Runtime invariant: `substrate world deps current sync|install` MUST NOT invoke APT/dpkg | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | prohibited side effects; operator-visible rationale; linkage to provisioning command |
| Runtime fail-early operational semantics (scope rules + side-effect prohibitions) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` | exact trigger definition; enabled-set vs explicit-args handling; partial-apply rules (must be singular) |
| Runtime remediation message invariants (exact command string + unsupported-backend messaging) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | required exact command string `substrate world enable --provision-deps`; required “no host OS mutation” messaging on Linux host-native; stream requirements |
| Exit code meanings (`0/2/3/4/5`) for provisioning + runtime fail-early | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | mapping to taxonomy; per-command mapping; reserved meaning for exit `5` in this feature |
| Platform/backends support matrix (Linux host-native vs macOS Lima vs Windows WSL) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | supported/unsupported rules; guarantees; explicit unsupported messaging |
| Env var `SUBSTRATE_WORLD_REQUEST_PROFILE` (advanced/testing) | `docs/CONFIGURATION.md` | name, meaning, default, and explicit statement that it is not the operator-facing provisioning workflow for ADR-0030 |
| World-deps inventory schema inputs used by this feature (`install.method=apt`, `install.apt[]`, enabled-set resolution) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | schema and constraints; meaning of `install.apt[].version`; inventory/enabled merge order |
| World-agent execute protocol baselines (`/v1/execute`, request `profile` field) | `docs/WORLD.md` | request/response shapes; transport notes; any explicit “profile” semantics required by ADR-0030 |
| Manual validation | `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md` | deterministic preconditions, exact commands, expected key output lines, expected exit codes |
| Smoke validation | `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/*` | automated validation commands per platform; pass/fail expectations aligned to manual playbook |
| Slice acceptance (WDAP0) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | per-slice scope + acceptance criteria IDs for provisioning-time APT installs |
| Slice acceptance (WDAP1) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` | per-slice scope + acceptance criteria IDs for runtime fail-early + remediation |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Error model (exit codes, error-message invariants where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/safety invariants (no host OS mutation; hardened runtime preserved)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)

## Follow-ups

1. APT requirement derivation is underspecified
   - Issue: ADR-0030 requires deriving required APT packages from the effective enabled world-deps set, but does not define the deterministic conflict policy when multiple enabled items require the same APT package name with different `version` pins.
   - Required fix: in `decision_register.md` (DR-0001) and `slices/WDAP0/WDAP0-spec.md`, define exactly one deterministic policy, including:
     - de-dup rules and stable ordering rules
     - conflict behavior and exit-code mapping when version pins disagree

2. Provisioning execution isolation model is implied but not pinned
   - Issue: ADR-0030 requires provisioning execution “without weakening hardened runtime execution” and mentions a distinct request profile or guard rails, but does not define the exact execution posture and how it is prevented from mutating the Linux host OS.
   - Required fix: in `decision_register.md` (DR-0003) and `slices/WDAP0/WDAP0-spec.md`, define one deterministic isolation model, including:
     - the exact request `profile` value(s) used for provisioning-time execution (if any)
     - the explicit guard rail that blocks host-native Linux from provisioning-time APT mutation
     - the explicit relationship (or non-relationship) to `SUBSTRATE_WORLD_REQUEST_PROFILE`

3. `world enable --provision-deps` operational scope is underspecified
   - Issue: ADR-0030 defines `substrate world enable --provision-deps` but does not specify whether it also performs (or requires) the baseline world-backend provisioning steps of `substrate world enable`.
   - Required fix: in `contract.md` and `slices/WDAP0/WDAP0-spec.md`, define one deterministic ordering and idempotency posture for:
     - “world backend enable” steps, and
     - “APT provisioning” steps,
     including `--dry-run` behavior.

4. Runtime fail-early “scope” is underspecified for `deps current install`
   - Issue: ADR-0030 states the runtime short-circuit triggers when the “effective enabled set contains `install.method=apt` items”, but the existing CLI includes `substrate world deps current install <ITEM...>` which can target explicit items.
   - Required fix: in `contract.md` and `slices/WDAP1/WDAP1-spec.md`, define exactly one scope rule (enabled-set vs explicit args vs union) and require tests to enforce it.

5. Runtime `--dry-run`/`--verbose` behavior under fail-early is underspecified
   - Issue: ADR-0030 defines runtime fail-early behavior but does not specify whether `--dry-run` is permitted to emit the derived plan (including APT items) without failing, or whether it still exits `4`.
   - Required fix: in `contract.md` and `slices/WDAP1/WDAP1-spec.md`, define one deterministic rule for `--dry-run` and `--verbose` under the fail-early posture (including minimum guaranteed output elements and stream requirements).

6. Windows provisioning posture is ambiguous
   - Issue: ADR-0030 asserts Windows WSL support as an assumption, but `substrate world enable` is currently Windows-unsupported in implementation.
   - Required fix: in `contract.md` and smoke/playbook artifacts, define exactly one Windows posture for this feature:
     - either provisioning is supported as part of this pack, or
     - provisioning is explicitly unsupported on Windows (exit `4`) with deterministic manual guidance.

7. Operator-doc update targets are not enumerated by exact path/headings
   - Issue: ADR-0030 requires adding operator-facing explanation that APT-backed world-deps are provisioning-time, but does not enumerate the exact doc file(s)/heading(s) that must change.
   - Required fix: in `pre-planning/impact_map.md` and `slices/WDAP1/WDAP1-spec.md`, list the exact doc paths and headings, and require those docs to link to `contract.md` rather than restating the contract.

8. Cross-document contract ownership conflicts must be reconciled
   - Issue: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` and `docs/internals/world/deps.md` currently describe runtime `world deps current sync|install` applying APT (“world image installs first (apt)”), which conflicts with ADR-0030 (“runtime MUST NOT invoke APT/dpkg”).
   - Required fix: during planning/implementation, reconcile the world-deps contract so there is exactly one authoritative truth for runtime behavior. The fix MUST either:
     - update the upstream world-deps contract docs to incorporate the provisioning-time-only APT contract, or
     - explicitly defer to this feature’s `contract.md` for all system-package runtime/provisioning rules.
