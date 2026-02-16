# Planning Quality Gate Report — macos_world_backend_vf

## Metadata
- Feature directory: `docs/project_management/next/macos_world_backend_vf/`
- Reviewed commit: `a8ed4d9082f88bb349e0a084624e595b236b601b`
- Reviewer: `Codex (third-party planning quality gate)`
- Date (UTC): `2026-02-16`
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/macos_world_backend_vf"

# Mechanical lint (required)
make planning-lint FEATURE_DIR="$FEATURE_DIR"
# exit=2
# == Planning lint: docs/project_management/next/macos_world_backend_vf ==
# Missing required path: docs/project_management/next/macos_world_backend_vf/plan.md

# tasks.json validation (required by lint checklist)
make planning-validate FEATURE_DIR="$FEATURE_DIR"
# exit=2
# docs/project_management/next/macos_world_backend_vf/tasks.json: missing

# JSON validity (template minimum)
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
# exit=2 (missing file)
jq -e . docs/project_management/next/sequencing.json >/dev/null
# exit=0

# ADR drift check (lint runner requirement when ADRs present)
make adr-check ADR="$FEATURE_DIR/ADR-2026-02-13-macos-world-backend-virtualization-framework.md"
# exit=2
# missing `## Executive Summary (Operator)` section

# Sequencing presence (required by lint checklist)
rg -n "macos_world_backend_vf" docs/project_management/next/sequencing.json
# exit=1 (no matches)

# Rubric scans (required by planning standard)
rg -n --hidden --glob '!**/.git/**' '\b(TBD|TODO|WIP|TBA)\b|open question|\betc\.|and so on' "$FEATURE_DIR"
# exit=0 (matches found; FAIL)
rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' '\b(should|could|might|maybe|optionally|likely)\b' "$FEATURE_DIR"
# exit=0 (matches found; FAIL)
```

## Required Inputs Read End-to-End (checklist)
Mark `YES` only if read end-to-end.

- ADR(s): `YES` (`docs/project_management/next/macos_world_backend_vf/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`)
- `spec_manifest.md`: `NO` (missing)
- `plan.md`: `NO` (missing)
- `tasks.json`: `NO` (missing)
- `session_log.md`: `NO` (missing)
- All specs in scope: `NO` (no slice specs present)
- `decision_register.md` (if present/required): `NO` (missing)
- `impact_map.md` (if present/required): `NO` (missing)
- `manual_testing_playbook.md` (if present/required): `NO` (missing)
- Feature smoke scripts under `smoke/` (if required): `NO` (missing)
- `docs/project_management/next/sequencing.json`: `NO` (validated + searched; not read end-to-end)
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/macos_world_backend_vf/05_policy_model.md:58` (“should ship”)
  - `docs/project_management/next/macos_world_backend_vf/05_policy_model.md:65` (“likely requires”)
  - `docs/project_management/next/macos_world_backend_vf/07_testing_plan.md:37-40` (“should fail”)
  - `docs/project_management/next/macos_world_backend_vf/01_problem_and_goals.md:3` (“etc.”)
- Notes: The planning rubric bans ambiguity words in behavior/contracts and hard-bans `etc.` / similar placeholders.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence:
  - Missing `docs/project_management/next/macos_world_backend_vf/decision_register.md`
  - ADR lists 3 alternatives and does not follow the “exactly 2 viable options” decision standard.
- Notes: No Decision Register entries mapping decisions to execution tasks exist.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `FAIL`
- Evidence:
  - Proposed command `substrate world create...` in `docs/project_management/next/macos_world_backend_vf/04_architecture_and_flows.md:41`, but current CLI has no `world create` subcommand (`crates/shell/src/execution/cli.rs:522-533`).
  - Proposed env var `SUBSTRATE_WORLD_BACKEND` in `docs/project_management/next/macos_world_backend_vf/08_rollout_plan.md:9`, but it does not exist in code (repo search returns only this doc).
- Notes: User/operator contracts must match existing CLI/env taxonomy or explicitly propose and specify the new contract surfaces via `spec_manifest.md` + specs.

### 4) Sequencing and dependency alignment
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/sequencing.json` has no entries for `macos_world_backend_vf` (search returned no matches).
  - `tasks.json` is missing, so there is no dependency model to align.
- Notes: The sequencing spine is required for readiness.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - Missing: `manual_testing_playbook.md`, `smoke/*`, and `tasks.json` acceptance criteria.
- Notes: No runnable acceptance criteria, exit code expectations, or smoke gates are defined.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `FAIL`
- Evidence:
  - Missing `tasks.json` (no `meta.schema_version`, no `meta.cross_platform`, no integration task model).

### 6) Triad interoperability (execution workflow)
- Result: `FAIL`
- Evidence:
  - Missing `plan.md`, `tasks.json`, `session_log.md`, `kickoff_prompts/`, and slice specs.

## Findings (must be exhaustive)

### Finding 001 — Planning Pack is missing required execution artifacts
- Status: `DEFECT`
- Evidence:
  - `make planning-lint` fails with missing `docs/project_management/next/macos_world_backend_vf/plan.md`.
  - `docs/project_management/next/macos_world_backend_vf/` contains only narrative docs + ADR (no `plan.md`, `spec_manifest.md`, `tasks.json`, `session_log.md`, `kickoff_prompts/`, slice specs).
- Impact: The plan cannot be executed under the repo’s triad workflow; there is no authoritative contract surface, no task DAG, and no audit log scaffolding.
- Fix required (exact): Add the full Planning Pack skeleton per `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` and make `make planning-lint FEATURE_DIR="$FEATURE_DIR"` pass.
- If DEFECT: Alternative (one viable): Treat this directory as a research memo only and move the ADR into `docs/project_management/adrs/draft/`, then create a new proper Planning Pack directory when execution planning begins.

### Finding 002 — Mechanical lint fails (non-negotiable gate)
- Status: `DEFECT`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/macos_world_backend_vf"` exits non-zero (missing required files).
- Impact: Per the lint checklist, any mechanical failure forces `FLAG FOR HUMAN REVIEW`.
- Fix required (exact): Make `make planning-lint FEATURE_DIR="$FEATURE_DIR"` exit `0`.
- If DEFECT: Alternative (one viable): None (this is a hard gate).

### Finding 003 — ADR fails `make adr-check` (missing Executive Summary section)
- Status: `DEFECT`
- Evidence: `make adr-check ADR=docs/project_management/next/macos_world_backend_vf/ADR-2026-02-13-macos-world-backend-virtualization-framework.md` reports missing `## Executive Summary (Operator)` section.
- Impact: ADR drift guard is part of the mechanical quality bar; operator-facing contract is missing.
- Fix required (exact): Add `## Executive Summary (Operator)` to the ADR in the required format so `make adr-check` passes.
- If DEFECT: Alternative (one viable): Move the ADR to the ADR system location and regenerate it from the ADR template (ensures required sections exist).

### Finding 004 — Proposed CLI and env-var contract does not match current Substrate
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/macos_world_backend_vf/04_architecture_and_flows.md:41` references `substrate world create...`; current world CLI actions are `Doctor|Enable|Deps|Cleanup|Verify` (`crates/shell/src/execution/cli.rs:522-533`).
  - `docs/project_management/next/macos_world_backend_vf/08_rollout_plan.md:9` references `SUBSTRATE_WORLD_BACKEND`, which is absent in code (repo search finds only this doc).
- Impact: Operator workflows and tooling cannot be implemented or tested deterministically if they reference non-existent surfaces without a spec + manifest.
- Fix required (exact): Either (A) update the planning docs to match existing surfaces (`substrate world enable/doctor/verify`, existing config/env knobs), or (B) define the new surfaces in `spec_manifest.md` and slice specs (including exit codes and docs updates).
- If DEFECT: Alternative (one viable): Reframe rollout to use existing `SUBSTRATE_WORLD_SOCKET` override + macOS backend detection as the selection mechanism until a dedicated backend selector is specified and implemented.

### Finding 005 — Backend selection is not runtime-pluggable today
- Status: `DEFECT`
- Evidence:
  - `crates/world-backend-factory/src/lib.rs:11-15` hard-selects `world_mac_lima::MacLimaBackend` on macOS at compile time.
  - `crates/shell/src/execution/platform_world/mod.rs:66-111` constructs `MacLimaBackend` unconditionally on macOS.
- Impact: The ADR’s stated “selected at runtime; Apple Silicon default VF fallback Lima” is not currently realizable without significant refactors to selection/config plumbing.
- Fix required (exact): Record an explicit decision (A/B) for backend selection wiring (compile-time vs runtime), then specify the contract + implementation plan in `spec_manifest.md` and tasks.
- If DEFECT: Alternative (one viable): Start with VF as a separate experimental binary/backend (opt-in via existing socket/env override surfaces) to avoid refactoring selection plumbing in the first milestone.

### Finding 006 — VF-macOS guest world is incompatible with current world-agent architecture
- Status: `DEFECT`
- Evidence:
  - `docs/WORLD.md:11-23` defines “world” as a reusable **Linux** execution context; on macOS it runs inside a Lima VM.
  - `crates/world-agent/src/service.rs:384-387` explicitly bails on non-Linux: “World agent only supported on Linux inside VMs”.
- Impact: A macOS guest world cannot run the existing `world-agent` as-is, so the plan’s “VF-macOS world flavor” requires a new agent implementation and policy enforcement model (major scope and security surface).
- Fix required (exact): Convert this into an explicit decision with two viable options and select one:
  - Option A: Scope VF backend to VF-Linux only (preserve current Linux world-agent semantics).
  - Option B: Add a macOS-compatible agent + enforcement model (new spec surfaces and security review required).
- If DEFECT: Alternative (one viable): Defer VF-macOS entirely and ship only VF-Linux parity work first; treat “macOS tooling in-world” as a separate follow-on track with its own ADR/spec_manifest/tasks.

### Finding 007 — Filesystem “discover-only” semantics proposed differ from current enforcement semantics
- Status: `DEFECT`
- Evidence:
  - Proposed placeholder shadow-tree model in `docs/project_management/next/macos_world_backend_vf/05_policy_model.md:19-48`.
  - Current discover semantics are enforced via allowlists consumed by the Landlock exec path (`crates/world-agent/src/internal_exec.rs:22-61`).
- Impact: Placeholder files change behavior from “exists but unreadable” to “exists and readable-but-empty”, which can break tools and undermines “preserve policy model” claims.
- Fix required (exact): Define the authoritative discover/read/write semantics in a spec and map them to the actual enforcement implementation for VF-Linux; for VF-macOS, explicitly document the semantic divergence (if any) and its security implications.
- If DEFECT: Alternative (one viable): For VF-Linux, reuse the existing Linux world-agent overlay/pivot + Landlock enforcement model instead of inventing a host-side staging filesystem.

### Finding 008 — Entitlements/signing/distribution is acknowledged but not decided
- Status: `DEFECT`
- Evidence:
  - ADR states VF requires code signing + entitlements (`docs/project_management/next/macos_world_backend_vf/ADR-2026-02-13-macos-world-backend-virtualization-framework.md:60-65`).
  - Work breakdown does not include an explicit decision and contract for how Substrate binaries are signed/notarized and how dev/CI will run VF flows.
- Impact: Without a concrete packaging/signed-helper decision, VF execution may be impossible for most users (and impossible in CI), blocking implementation.
- Fix required (exact): Add a Decision Register entry with exactly two viable packaging options (e.g., signed helper vs signed main binary) and define the resulting build/test/distribution contract.
- If DEFECT: Alternative (one viable): Use a separate signed helper binary/app bundle responsible only for VF VM lifecycle (minimal entitlement surface), controlled by the unsigned main CLI via a local socket/XPC.

## Decision: ACCEPT or FLAG

### If FLAG FOR HUMAN REVIEW
- Summary: The feature directory is not an implementation-ready Planning Pack (missing required artifacts) and fails mandatory mechanical gates. Several core feasibility claims (runtime backend selection, VF-macOS guest agent support, policy semantics) are inconsistent with the current Substrate architecture.
- Required human decisions (explicit):
  - Whether VF scope is VF-Linux-only (parity) vs VF-macOS (new agent/enforcement model).
  - Packaging/signed-helper strategy for Virtualization.framework entitlements.
  - Whether the plan should reuse current Linux world-agent enforcement vs invent host-side policy mount staging.
- Blockers to execution:
  - `make planning-lint` fails.
  - `make adr-check` fails.
  - No `spec_manifest.md` / `tasks.json` / `plan.md` / `session_log.md` / `kickoff_prompts/` / slice specs.

