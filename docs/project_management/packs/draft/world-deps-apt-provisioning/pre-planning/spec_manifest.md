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

Each entry is feature-local (must live under `docs/project_management/packs/draft/world-deps-apt-provisioning/`) and is authoritative only for the surfaces listed.

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md`
  - Owns (authoritative):
    - required-doc selection for this feature directory
    - surface → authoritative-doc ownership map (coverage matrix)
    - follow-ups required to remove ADR ambiguity before quality gate
    - canonical slice ID set for this feature (no `C0/C1/...`)
  - Must define (deterministic items):
    - an explicit “no implied surfaces” posture for any surface category not used by ADR-0030
    - the slice ID mapping from ADR-0030 (`C0`/`C1`) to `WDAP0`/`WDAP1`
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`
  - Owns (authoritative):
    - touch set + cascading implications + cross-queue conflicts for slices `WDAP0` and `WDAP1`
  - Must define (deterministic items):
    - explicit create/edit touch allowlists by path for `WDAP0-{code,test,integ}` and `WDAP1-{code,test,integ}`
    - explicit list of operator-doc update targets (by exact path) required by ADR-0030, including the “link-to-contract.md; do not restate contract tables” rule
    - cross-pack dependency/conflict notes, including:
      - `world-deps-packages-bundles-contract` (inventory schema and existing `install.method=apt` semantics), and
      - `add-non-apt-system-package-provisioning-support` (explicit downstream dependency on `WDAP0` APT provisioning contract)
  - Links to (non-authoritative):
    - all feature-local docs listed in this section

- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - checkpoint grouping + CI gates for this pack (schema v4 cross-platform automation packs)
  - Must define (deterministic items):
    - checkpoint groups and which slice(s) end each checkpoint group (MUST include `WDAP0` and `WDAP1`)
    - alignment rule: `tasks.json` MUST define `meta.checkpoint_boundaries` and it MUST match this plan
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
  - Owns (authoritative):
    - execution runbook + sequencing overview (including required validation evidence)
  - Must define (deterministic items):
    - orchestration branch name (MUST match `tasks.json` `meta.automation.orchestration_branch`)
    - canonical locations for pre-planning artifacts for this pack:
      - `pre-planning/spec_manifest.md`
      - `pre-planning/impact_map.md`
      - `pre-planning/ci_checkpoint_plan.md`
    - slice ordering (single explicit order): `WDAP0` then `WDAP1`
    - required validation commands per ADR-0030:
      - unit/integration test targets (by exact `cargo test ...` command or `make` target)
      - manual playbook completion criteria (by exact path)
      - smoke script run requirements (by exact path) and required platforms (Linux/macOS/Windows)
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - slice specs under `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json` (already exists)
  - Owns (authoritative):
    - task IDs, dependency graph, and automation metadata (branch/worktree/prompt paths)
  - Must define (deterministic items):
    - `meta.checkpoint_boundaries` (required for schema v4 cross-platform automation packs)
    - triad tasks for both slices:
      - `WDAP0-code`, `WDAP0-test`, `WDAP0-integ`
      - `WDAP1-code`, `WDAP1-test`, `WDAP1-integ`
    - each task’s acceptance criteria MUST reference `AC-WDAP0-*` or `AC-WDAP1-*` IDs from the corresponding slice spec
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`
  - Owns (authoritative):
    - append-only planning/execution log for this pack
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - “every task start/end must be logged with timestamp + task id” rule
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/quality_gate_report.md`
  - Owns (authoritative):
    - planning quality gate outcome for starting triads
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - rule: execution triads MUST NOT start unless `RECOMMENDATION: ACCEPT` is present
  - Links to (non-authoritative):
    - every required artifact referenced by `RECOMMENDATION`

### Feature contract + decisions (required by ADR-0030)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
  - Owns (authoritative):
    - DR-0001 — package-list conflict policy for APT requirement derivation (duplicate names; version pins)
    - DR-0002 — provisioned-state tracking posture (probe-only vs persisted state)
    - DR-0003 — provisioning execution isolation model (hardened runtime preserved; provisioning-time mutation boundary)
  - Must define (deterministic items):
    - exactly two options (A/B) per DR and exactly one selection per DR
    - the exact surfaces impacted by each DR (which spec docs must change)
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - Owns (authoritative):
    - operator-facing contract introduced/changed by ADR-0030 for:
      - provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]`
      - runtime invariant: `substrate world deps current sync|install` MUST NOT invoke APT/dpkg
    - exit-code meanings used by this feature (taxonomy-aligned; ADR-0030 subset: `0/3/4/5`, plus taxonomy defaults for all other codes)
    - platform/backends support matrix for provisioning and runtime remediation:
      - Linux host-native (unsupported; no host OS mutation)
      - macOS Lima guest (supported)
      - Windows WSL guest (explicitly scoped per Follow-ups)
    - remediation message invariants (including the required exact command string `substrate world enable --provision-deps`)
    - protected paths / OS-mutation invariants required by ADR-0030 (hardened runtime remains fail-closed; provisioning is explicit)
    - explicit statement: this feature introduces no new config keys and no new environment variables
  - Must define (deterministic items):
    - provisioning success/no-op semantics:
      - behavior when the effective enabled world-deps set contains zero `install.method=apt` items
    - provisioning failure semantics:
      - backend unavailable / cannot connect to world-agent (exit `3`)
      - provisioning unsupported on this backend (exit `4`) with required “no host OS mutation” messaging on Linux host-native
      - hardening conflict / fail-closed safety violation (exit `5`; reserved meaning for provisioning misconfiguration)
    - runtime fail-early trigger definition (high level; operational details live in slice specs):
      - how `current sync` and `current install` determine whether APT-backed items are in scope
    - `--dry-run` contract for provisioning:
      - prints the derived APT package list and intended actions
      - performs no mutation
    - minimum guaranteed `--verbose` additions and the required stream(s) (`stdout` vs `stderr`)
  - Links to (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/CONFIGURATION.md` (`SUBSTRATE_WORLD_REQUEST_PROFILE`; baseline only)
    - `docs/WORLD.md` (world-deps command syntax baseline; baseline only)

### Validation artifacts (required by ADR-0030)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
  - Owns (authoritative):
    - human validation workflow for ADR-0030 (commands + expected key output + exit codes)
  - Must define (deterministic items):
    - guest provisioning success path (macOS Lima; Windows WSL per Follow-ups)
    - runtime remediation behavior for `world deps current sync` and `world deps current install`
    - Linux host-native unsupported provisioning behavior (explicit “no host OS mutation” posture)
    - required preconditions and how to establish them (including how to ensure at least one enabled dep uses `install.method=apt`)
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - slice specs under `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/`

- Smoke scripts (feature-local; cross-platform):
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`
  - Own (authoritative):
    - automated validation steps per platform and pass/fail expectations aligned to `manual_testing_playbook.md`
  - Must define (deterministic items):
    - exact commands executed and exact assertions performed for:
      - provisioning supported vs unsupported paths (per platform/backend)
      - runtime fail-early remediation behavior
    - exit-code expectations for the smoke scripts themselves
  - Link to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
  - Owns (authoritative):
    - vertical slice behavior + acceptance criteria for ADR-0030 C0 (provisioning-time APT)
  - Must define (deterministic items):
    - acceptance criteria (AC-WDAP0-*) that prove:
      - the APT requirement set is derived from the effective enabled world-deps set and filtered by `install.method=apt`
      - the derived package list is deterministic (de-dup + stable ordering) and includes explicit version pins when present
      - provisioning executes APT installs only on supported guest backends and never mutates the Linux host OS
      - provisioning uses an execution posture that can mutate guest OS packages without weakening hardened runtime execution (selected by DR-0003)
      - `--dry-run` produces deterministic output and performs no mutation
    - the exact APT invocation contract:
      - command(s), flags, and environment required for non-interactive installs
      - idempotency definition (what “already installed” means)
      - error → exit-code mapping aligned to `contract.md`
  - Links to (non-authoritative):
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
  - Owns (authoritative):
    - vertical slice behavior + acceptance criteria for ADR-0030 C1 (runtime fail-early + remediation for APT items)
  - Must define (deterministic items):
    - acceptance criteria (AC-WDAP1-*) that prove:
      - `substrate world deps current sync` MUST NOT invoke APT/dpkg and fails early when APT-backed items are in scope
      - `substrate world deps current install` MUST NOT invoke APT/dpkg and fails early when APT-backed items are in scope
      - the failure is actionable:
        - includes the exact remediation command `substrate world enable --provision-deps`
        - includes manual guidance on backends where provisioning is unsupported and explicitly states “no host OS mutation”
      - exit code is `4` for the fail-early posture (aligned to `contract.md`)
    - the exact scope rule for determining “APT-backed items are in scope” for both commands (enabled-set vs explicit args)
    - operator-doc update requirements (by exact path/headings) for the “APT is provisioning-time” contract (listed in `impact_map.md`)
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`

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
