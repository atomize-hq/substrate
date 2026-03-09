# dev-install-world-agent-staging — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`

## Slice IDs (canonical)

ADR-0035 uses placeholder slice IDs (`C0`, `C1`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `DIWAS` (derived from “Dev Install World Agent Staging”)
- `DIWAS0` — enable preflight: missing `world-agent` (ADR “C0”)
- `DIWAS1` — dev-install `--no-world` stages `world-agent` (ADR “C1”)

## Required spec documents (authoritative)

Each entry lists:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns (authoritative):
    - the required-doc set for this feature directory
    - the coverage matrix (surface → authoritative doc)
    - follow-ups required to remove ambiguity before quality gate
  - Must define:
    - an explicit “no implied surfaces” statement for every surface category that is *not* used by this feature (protocol, telemetry, etc.)
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
    - planning + triad standards referenced in this manifest

- `docs/project_management/packs/draft/dev-install-world-agent-staging/plan.md` — execution runbook + sequencing overview
  - Owns (authoritative):
    - slice ordering (`DIWAS0` then `DIWAS1`)
    - task execution guardrails (triad workflow, worktree boundaries)
  - Must define:
    - the orchestration branch (MUST match `tasks.json` `meta.automation.orchestration_branch`)
    - the full slice list (`DIWAS0`, `DIWAS1`) and each slice’s objective (single sentence each)
    - the validation evidence required at the end of each slice (manual + smoke references)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json`
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json` — triad task graph + acceptance criteria
  - Owns (authoritative):
    - task IDs, dependency graph, and automation metadata (branch/worktree/prompt paths)
  - Must define:
    - `meta.checkpoint_boundaries` (required for schema v4 cross-platform automation packs): an array of slice IDs that are the last slice in each checkpoint group in `ci_checkpoint_plan.md`
    - triad tasks for each slice:
      - `DIWAS0-code`, `DIWAS0-test`, `DIWAS0-integ`
      - `DIWAS1-code`, `DIWAS1-test`, `DIWAS1-integ`
    - each task’s `acceptance` list that references the slice spec AC IDs (no freeform acceptance)
    - kickoff prompt paths for every task (MUST exist on disk)
  - Links (non-authoritative):
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/session_log.md` — planning + execution audit log
  - Owns (authoritative):
    - the append-only record of planning/execution events for this pack
  - Must define:
    - the file MUST be initialized from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - every task start/end MUST be recorded with timestamp + task id
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/impact_map.md` — touch set + cascading implications + cross-queue conflicts
  - Owns (authoritative):
    - the explicit touch set (create/edit/deprecate/delete) for this feature
    - the explicit list of cascading implications and contradiction risks
  - Must define:
    - the touch set MUST enumerate every repo file expected to be created/edited/deprecated/deleted by this feature
    - if this feature edits any of the following anticipated targets, the touch set MUST include them:
      - `scripts/substrate/dev-install-substrate.sh`
      - `scripts/substrate/world-enable.sh`
      - `scripts/substrate/install-substrate.sh`
      - `crates/shell/src/builtins/world_enable/**`
    - the touch set MUST include every new/edited test file introduced for this behavior (Linux-only)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/ci_checkpoint_plan.md` — CI cadence plan (schema v4 cross-platform automation packs)
  - Owns (authoritative):
    - checkpoint grouping + CI gates for this pack
  - Must define:
    - machine-readable plan per `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`
    - `tasks.json` `meta.checkpoint_boundaries` MUST exist and MUST match the checkpoint group boundaries defined by this plan
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/impact_map.md`
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/quality_gate_report.md` — planning quality gate outcome
  - Owns (authoritative):
    - third-party reviewer recommendation gate for starting execution triads
  - Must define:
    - the report MUST be authored from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - execution triads MUST NOT start unless `RECOMMENDATION: ACCEPT` is present
  - Links (non-authoritative):
    - all required artifacts in this section

### Feature contract + decisions (required by ADR-0035)

- `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md` — A/B decisions with explicit selection
  - Owns (authoritative):
    - the decisions enumerated by ADR-0035 as decision register entries:
      - DR-0001 (enable preflight implementation locus)
      - DR-0002 (dev meaning of `--no-world`)
      - DR-0003 (profile mapping for staging `world-agent`)
      - DR-0004 (overwrite/idempotency policy for staged `world-agent`)
  - Must define:
    - each DR MUST be exactly two options (A/B) with a single explicit selection
    - each DR MUST include the exact surfaces impacted (CLI/config/paths/exit codes)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` — user-facing contract surface (single source of truth)
  - Owns (authoritative):
    - CLI and script contracts (names, flags, defaults, ordering, examples)
    - config file paths + precedence + schema constraints
    - exit code meanings (taxonomy-aligned; no feature overrides unless explicitly declared)
    - filesystem/path semantics for the staged `world-agent` artifact
    - platform guarantees (Linux/macOS/Windows)
    - safety invariants relevant to enable/provisioning
  - Must define (minimum; no implicit behavior):
    - CLI/scripts in-scope (contract-owned):
      - `scripts/substrate/dev-install-substrate.sh` (Linux):
        - `--prefix <path>`
        - `--profile <debug|release>`
        - `--no-world`
      - `scripts/substrate/world-enable.sh` (Linux):
        - `--home <path>`
        - `--profile <name>`
        - `--dry-run`
        - `--verbose`
        - `--force`
        - `--no-sync-deps`
      - `scripts/substrate/install-substrate.sh` (Linux):
        - `--no-world` (enable-later posture; no regression contract)
      - `substrate world enable` (Linux):
        - `--home <path>`
        - `--profile <name>`
        - `--dry-run`
        - `--verbose`
        - `--force`
        - `--timeout <seconds>`
    - For each command/script above:
      - required inputs and precedence order (flags vs env vars vs defaults)
      - default values (including the default profile used for staging/provisioning)
      - absence semantics (missing config file, missing `world.enabled`, missing staged binary, missing helper script)
      - success/no-op semantics
      - failure semantics: exit code (taxonomy) + minimum remediation text constraints
    - Config:
      - file path resolution and precedence order for `$SUBSTRATE_HOME/config.yaml` (MUST include how `--home` interacts with `$SUBSTRATE_HOME` and the default `~/.substrate`)
      - schema for `world.enabled` (boolean) including behavior when absent
      - invariant: `world.enabled` MUST be set to `true` only after successful provisioning verification
      - invariant: `--dry-run` MUST NOT write config or provision systemd
    - Environment variables (in-scope):
      - `SUBSTRATE_HOME`
      - `SUBSTRATE_WORLD_ENABLE_SCRIPT`
      - `SUBSTRATE_WORLD_SOCKET`
      - For each: type, default when unset, and precedence vs CLI flags
    - Filesystem/path semantics:
      - canonical “dev version dir” root used by the enable helper for dev installs (`<repo>/target/` per ADR-0035)
      - helper script discovery rules for `substrate world enable`:
        - default search paths (version dir then home dir)
        - override behavior for `SUBSTRATE_WORLD_ENABLE_SCRIPT`
      - provisioning log path for `substrate world enable` (location + filename pattern)
      - canonical staged artifact path set under the version dir:
        - `bin/world-agent`
        - `bin/linux/world-agent`
      - deterministic rule for whether BOTH staged paths are required, or whether ONE is sufficient (ADR-0035 currently asserts both; this MUST be made deterministic)
      - overwrite/idempotency policy for already-present staged binaries (decision-owned by DR-0004; contract MUST reflect the selection)
    - Exit codes:
      - taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - deterministic mapping for missing staged `world-agent` during enable preflight (exit code + remediation)
      - deterministic mapping for unsupported platforms (Windows) for `substrate world enable`
    - Safety posture:
      - invariant: enable MUST fail before systemd/socket/readiness work when the staged `world-agent` artifact is missing
      - invariant: `cargo build` MUST NOT be executed under `sudo` as part of enable (ADR-0035 fail-closed rule)
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md`
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS0/DIWAS0-spec.md` — enable preflight when `world-agent` is missing (Linux)
  - Owns (authoritative):
    - the DIWAS0 slice scope + acceptance criteria (implementation-level), without redefining the `contract.md` CLI/config/exit-code contract
  - Must define:
    - a single “behavior delta” (Existing/New/Why) scoped to preflight + missing-artifact remediation
    - acceptance criteria (AC-DIWAS0-*) that are directly checkable via:
      - a Linux-only test (unit or integration) that exercises the preflight error path deterministically
      - manual playbook steps that demonstrate the remediation path
      - at least one acceptance criterion that proves the “fail before provisioning/systemd/socket/readiness work” invariant as defined by `contract.md`
    - a “contract link rule”: this slice spec MUST link to `contract.md` and MUST NOT redefine CLI/config/exit-code tables
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS1/DIWAS1-spec.md` — dev-install `--no-world` stages `world-agent` (Linux)
  - Owns (authoritative):
    - the DIWAS1 behavior delta and acceptance criteria (implementation-level)
  - Must define:
    - a single “behavior delta” (Existing/New/Why) scoped to dev-install staging
    - acceptance criteria (AC-DIWAS1-*) that are directly checkable via:
      - a Linux-only integration test that runs `scripts/substrate/dev-install-substrate.sh --no-world --profile <profile>` and asserts the staged binary exists and is executable at the contract-defined path
    - a “contract link rule”: this slice spec MUST link to `contract.md` and MUST NOT redefine CLI/config/exit-code tables
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md`

### Validation artifacts (required by ADR-0035)

- `docs/project_management/packs/draft/dev-install-world-agent-staging/manual_testing_playbook.md` — manual validation cases (authoritative)
  - Owns (authoritative):
    - deterministic manual validation steps for Linux that exercise:
      - dev-install `--no-world` staging
      - `substrate world enable --dry-run`
      - `substrate world enable` (privileged) and the `world.enabled` flip on success
  - Must define:
    - prerequisites (Linux host, required tooling, and privilege requirements)
    - exact commands to run (no placeholders) and expected exit codes per `contract.md`
    - exact filesystem assertions (paths and executable bit) per `contract.md`
    - exact config assertions (`world.enabled` value in `$SUBSTRATE_HOME/config.yaml`) per `contract.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`
    - slice specs under `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/`

- `docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh` — automated Linux smoke validation
  - Owns (authoritative):
    - the smoke assertions and their pass/fail exit code contract
  - Must define:
    - deterministic checks that validate the DIWAS0/DIWAS1 contract surfaces without human intervention
    - explicit skip/abort rules (if privilege is required, the script MUST state the required invocation mode and MUST fail with a deterministic exit code when not satisfied)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0035 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI/script contract (commands + flags + defaults) | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | command list; flag list; defaults; precedence; success/no-op; examples |
| Missing-`world-agent` preflight behavior | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | path(s) checked; ordering (“before provisioning”); exit code; remediation text constraints |
| Dev-install `--no-world` staging behavior | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | staged path set; profile mapping; overwrite policy (via DR-0004); “no provisioning” invariant |
| Config file paths + precedence | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | `$SUBSTRATE_HOME/config.yaml` resolution; `--home` interaction; absence semantics |
| Config schema (`world.enabled`) | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | type; defaults; meaning of `true/false`; “flip only after successful provisioning” invariant |
| Exit codes | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | taxonomy mapping; codes for missing artifact; codes for unsupported platform; codes for usage/config errors |
| Environment variables (`SUBSTRATE_HOME`, `SUBSTRATE_WORLD_ENABLE_SCRIPT`, `SUBSTRATE_WORLD_SOCKET`) | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | names; types; precedence vs CLI flags; defaults; socket-path normalization rules (if any) |
| Platform guarantees | `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md` | Linux supported; macOS no-change stance; Windows unsupported stance + required error/exit code |
| Policy surface | `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify policy broker schemas, policy decisions, or enforcement mode |
| Decision points | `docs/project_management/packs/draft/dev-install-world-agent-staging/decision_register.md` | DR-0001..DR-0004 options/selection; impacted surfaces |
| Slice definitions + AC IDs | `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS*/DIWAS*-spec.md` | behavior delta per slice; acceptance criteria IDs and checks |
| Manual validation | `docs/project_management/packs/draft/dev-install-world-agent-staging/manual_testing_playbook.md` | exact commands; expected exit codes; expected file/config assertions |
| Smoke validation (Linux) | `docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh` | automated commands; expected pass/fail; required privileges |
| Touch set + cascading implications | `docs/project_management/packs/draft/dev-install-world-agent-staging/impact_map.md` | explicit touched files; contradiction risks (dev vs prod installs) |
| CI cadence checkpoints | `docs/project_management/packs/draft/dev-install-world-agent-staging/ci_checkpoint_plan.md` | checkpoint groups; gates; rationale; tasks.json alignment |
| Task graph (automation) | `docs/project_management/packs/draft/dev-install-world-agent-staging/tasks.json` | tasks; dependencies; kickoff prompt paths; branch/worktree metadata |
| Rollout / backwards compatibility | `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` | this feature MUST NOT introduce migrations, backwards compatibility policy, or deprecation requirements |
| New environment variables | `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` | this feature MUST NOT introduce new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` environment variables |
| Protocol surface | `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify any host↔agent wire/API protocol surface |
| Telemetry/log schema fields | `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify structured log schema fields or trace span fields |

## Follow-ups (required to remove ambiguity)

- ADR-0035 options are internally inconsistent:
  - “Options” section labels Option A/B/C differently than the “Decision Summary” A/B labels.
  - ADR MUST be corrected so option labels and the selected option are consistent.
- ADR-0035 declares “no exit code overrides” but does not pin the exit code for “missing staged `world-agent`” early failure.
  - `contract.md` MUST specify the exact exit code (taxonomy-aligned) and the minimum remediation text.
- `scripts/substrate/install-substrate.sh` and `scripts/substrate/world-enable.sh` currently use `exit 1` for `fatal`.
  - `contract.md` MUST state whether `substrate world enable` maps helper-script failures onto taxonomy exit codes, or whether helper exit codes are treated as opaque and surfaced as exit code `1`.
- ADR-0035 asserts both `bin/world-agent` and `bin/linux/world-agent` should exist after dev-install.
  - `decision_register.md` MUST decide whether BOTH are required or ONE is sufficient, and `contract.md` MUST reflect the decision.
- `--dry-run` semantics for the missing-artifact path are not explicit.
  - `contract.md` MUST state whether `--dry-run` performs the preflight check and what exit code/output occurs when the staged artifact is missing.
- ADR-0035 does not address helper-script discovery overrides (`SUBSTRATE_WORLD_ENABLE_SCRIPT`) or socket overrides (`SUBSTRATE_WORLD_SOCKET`).
  - `contract.md` MUST define how these environment variables interact with helper discovery, preflight checks, and health verification.
- `tasks.json` currently declares `behavior_platforms_required` includes `macos` and `windows`, while ADR-0035 scopes behavior deltas and smoke validation to Linux only.
  - The pack MUST choose exactly one:
    - (A) update `tasks.json` platform requirements to match Linux-only behavior validation, or
    - (B) add macOS/Windows validation artifacts that deterministically assert the intended “no change” / “unsupported” behavior.
- `tasks.json` is schema v4 with `meta.cross_platform=true` but does not currently define `meta.checkpoint_boundaries`.
  - `tasks.json` MUST add `meta.checkpoint_boundaries` and it MUST match `ci_checkpoint_plan.md` checkpoint group endings.
- ADR-0035 “Related Docs” currently references `docs/project_management/packs/draft/dev-install-world-agent-staging/spec_manifest.md`, but this run’s canonical output is `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`.
  - The pack MUST choose one canonical spec manifest path and update ADR-0035 “Related Docs” links to match.
