# Execution Preflight Gate Report — policy_and_config_precedence

Date (UTC): 2026-01-02T01:31:01Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/policy_and_config_precedence/`

## Recommendation

RECOMMENDATION: **ACCEPT**

## Inputs Reviewed

- [x] ADR accepted and still matches intent
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Cross-platform plan is explicit (`tasks.json` meta: platforms + WSL mode if needed)
- [x] `manual_testing_playbook.md` exists when required and is runnable
- [x] Smoke scripts exist where required and map to the manual playbook

## Cross-Platform Coverage (if applicable)

- Declared platforms (from `tasks.json` meta): `linux, macos, windows`
- WSL required: `no`

## Smoke ↔ Manual Parity Check

Smoke scripts must mimic the manual playbook by running the same commands/workflows and validating exit codes + key output.

- Linux smoke: `docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1`

Notes:
- Smoke scripts cover the manual playbook’s two required behaviors:
  - Workspace config overrides `SUBSTRATE_*` exports in a workspace (asserts `.world.caged==false` when `SUBSTRATE_CAGED=1`).
  - Workspace-scoped `substrate config show --json` exits `2` when no workspace exists.
- Dependency behavior is explicit:
  - Missing prerequisites (e.g., `substrate` / `jq`) exit `3` (smoke/playbook dependency-unavailable convention).
  - Assertion failures exit `1` with a concrete message (Linux/macOS include the observed value for fast debugging).

## CI Dispatch Readiness (if applicable)

- [x] Dispatch commands in integration tasks are correct and runnable
- [x] Required self-hosted runners exist and are labeled correctly

Verified runner inventory (GitHub Actions):
- Linux: `linux-manjaro-runner`
- macOS: `macOS-runner`
- Windows: `windows11-runner`
- WSL: `linux-wsl` (present, but not required for this feature)

Run ids/URLs (if executed during preflight):
- Linux:
- macOS:
- Windows:
- WSL:

## Required Fixes Before Starting PCP0 (if any)

- None.
