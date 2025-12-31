# Execution Preflight Gate Standard (Feature-Level Start Gate)

Goal:
- Prevent “execution drift” and weak smoke coverage by running a **feature-level start gate** before any triad work begins.

This gate is **execution-time** (not a planning quality gate):
- Planning creates the Planning Pack + smoke/manual plans.
- Execution preflight confirms the pack is runnable and the smoke/manual plans are actually adequate.

## When it runs
- After the Planning Pack has a quality gate report with `RECOMMENDATION: ACCEPT`.
- Before starting the first slice (before `C0-code` / `C0-test`).

## Required artifacts
For features that opt in (`tasks.json` meta: `execution_gates: true`):
- `docs/project_management/next/<feature>/execution_preflight_report.md`
- Task in `tasks.json`: `F0-exec-preflight` (type `ops`), with its kickoff prompt.

## What it checks (no ambiguity)

### 0) Slices are sized for reliable execution
- Each slice should represent one behavior delta (avoid “grab bag” slices).
- If slices are large or cross-cutting, require a split before execution begins.
  - See: `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md` → “Context Budget & Triad Sizing”.

### 1) Inputs are coherent
- ADR accepted and still matches the intended work.
- Planning Pack is complete and internally consistent:
  - `plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts.
  - `integration_map.md` and `manual_testing_playbook.md` exist when required by the planning standard.

### 2) Cross-platform implications are explicitly covered
- `tasks.json` declares platform requirements:
  - `meta.platforms_required: ["linux","macos","windows"]` (subset allowed).
  - If WSL coverage is required: `meta.wsl_required: true` and `meta.wsl_task_mode: "bundled"|"separate"`.
- Platform-fix integration structure is present if using schema v2 parity model:
  - `X-integ-core`, `X-integ-<platform>`, `X-integ` per slice.

### 3) Smoke scripts are not “toy” checks
Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature:
- Smoke scripts execute real commands/workflows and validate exit codes and observable output.
- They mirror the `manual_testing_playbook.md` intent:
  - If the manual playbook says “run command X and verify output contains Y”, the smoke script should run X and assert Y.
  - If the manual playbook describes multi-step workflows (world/agent provisioning, shim lifecycle, etc.), smoke should exercise the minimal viable subset.

### 4) CI dispatch path is runnable
- Cross-platform dispatch commands listed in the integration tasks are valid and runnable:
  - `make feature-smoke ...`
- Any required self-hosted runners exist and are labeled correctly.

## Output and rules
- Fill `execution_preflight_report.md` with:
  - Recommendation (`ACCEPT` or `REVISE`).
  - Any required fixes before starting C0.
  - If `REVISE`: do not start triads until the issues are fixed and the preflight is re-run.
