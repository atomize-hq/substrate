# Platform Integration & CI Standard

Goal:
- Enable parallel, reproducible integration validation across Linux/macOS/Windows (and optionally WSL) without blocking local development.

## Model

When a feature has platform smoke scripts under:
- `docs/project_management/next/<feature>/smoke/`

…treat platform validation as integration work that may run in different execution environments:
- Prefer GitHub Actions + self-hosted runners for cross-platform parity (Linux/macOS/Windows).
- WSL is treated as a distinct environment and runs on a dedicated self-hosted **Linux-in-WSL** runner.
- GitHub-hosted runners are allowed as a fallback, but are not the default for this repo’s “full fidelity” platform validation.

## Runner label contract (self-hosted)

To avoid accidental cross-environment pickup (e.g., a WSL runner taking a native Linux job), self-hosted runner selection is label-based:
- Native Linux runner: `[self-hosted, Linux, linux-host]`
- Linux-in-WSL runner: `[self-hosted, Linux, wsl]`
- macOS runner: `[self-hosted, macOS]`
- Windows runner: `[self-hosted, Windows]`

If you add a new runner, ensure it has a unique, positive label for its environment (no negative matching exists).

## Runner preflight (recommended)

Before relying on self-hosted smoke runs, validate runner availability and labels:
- `scripts/ci/check_self_hosted_runners.sh`

## Recommended task structuring

For cross-platform work, split integration into:
- **Option A (validation-only):** one integration task runs the smoke workflow with `platform=all` (and optionally `run_wsl=true`) and records the run id/URL in `session_log.md`.
- **Option B (platform-fix when needed):** split integration into core + platform-fix tasks:
  - `X-integ-core` (core integration): merges code+tests and gets primary-platform green.
  - `X-integ-linux` (platform-fix): runs Linux smoke; if WSL is required, also runs WSL smoke; if it fails, applies fixes on a Linux machine/worktree and re-runs smoke.
  - `X-integ-macos` (platform-fix): same for macOS.
  - `X-integ-windows` (platform-fix): same for Windows.
  - optional: `X-integ-wsl` (platform-fix): only create when WSL divergence is likely/expected and you want independent ownership/retry loops.
  - `X-integ` (final aggregator): merges any platform-fix branches, re-runs `make integ-checks` on the primary dev platform, and confirms cross-platform smoke is green.

This preserves parallelism and makes it explicit which validations happened where.

Only include platform-specific integration tasks for platforms that are actually required by the feature’s platform guarantees (as defined in ADR/spec/contract). If a feature is Linux-only, do not add macOS/Windows integration tasks.

## Planning Pack requirement (schema v2)

If a planning pack opts into the platform-fix model, encode it in `tasks.json`:
- `meta.schema_version: 2`
- `meta.platforms_required: ["linux","macos","windows"]`
- If WSL coverage is required:
  - `meta.wsl_required: true`
  - `meta.wsl_task_mode: "bundled" | "separate"`
    - `"bundled"` (default/recommended): run WSL as part of `X-integ-linux` by dispatching with `--run-wsl`
    - `"separate"`: add `X-integ-wsl` as its own platform-fix task

The mechanical tasks validator enforces the required task shape when `meta.schema_version >= 2` and `meta.platforms_required` is present:
- `make planning-validate FEATURE_DIR="docs/project_management/next/<feature>"`

## WSL task rubric (bundled vs separate)

Default: **bundled** (`meta.wsl_required=true`, `meta.wsl_task_mode="bundled"`)
- `X-integ-linux` runs `--platform linux --run-wsl` and owns Linux + WSL fixes unless/until WSL needs independent iteration.

Create a separate `X-integ-wsl` task only if one or more are true:
- The ADR/contract includes WSL-specific guarantees that differ from native Linux.
- The change touches WSL-specific surfaces (heuristic examples):
  - `crates/world-windows-wsl/**`
  - `scripts/windows/**`
  - files/sections explicitly about WSL bridging, path translation, or mount semantics
- WSL is historically flaky for the area being changed and you want clean independent ownership/retry loops.

## `tasks.json` (recommended fields)

For integration tasks, add optional fields:
- `platform`: `linux|macos|windows|wsl`
- `runner`: `local|github-actions|manual`
- `workflow`: a GitHub Actions workflow name/file (e.g., `.github/workflows/feature-smoke.yml`)

These fields are optional for now, but recommended for cross-platform planning packs.

## GitHub Actions workflow

This repo provides a reusable workflow:
- `.github/workflows/feature-smoke.yml`

It runs the feature-local smoke script for the selected platform(s), on either:
- GitHub-hosted runners (`runner_kind=github-hosted`), or
- your self-hosted runners (`runner_kind=self-hosted`).

WSL support:
- Bundled: dispatch `--platform linux --run-wsl` (runs Linux + WSL jobs).
- Separate: dispatch `--platform wsl` (runs only the WSL job).

## Trigger mechanism (repeatable)

GitHub Actions must run on a ref that exists on the remote. The repeatable pattern is:
1) Create a throwaway remote branch from the integration worktree commit (the code under test)
2) Dispatch the workflow from a stable workflow ref, checking out the throwaway branch (`checkout_ref`)
3) Wait for success/failure
4) Merge if green
5) Delete the throwaway branch

Helper script (requires `gh` auth):
- `make feature-smoke`
  - Defaults to dispatching from a stable workflow ref (`feat/policy_and_config`) and using `RUNNER_KIND=self-hosted`.
  - For triad execution, set `WORKFLOW_REF` to the orchestration branch (typically `feat/<feature>`) so the workflow definition matches the feature branch.

Smoke result interpretation (important):
- The smoke dispatcher always prints machine-parseable fields including `DISPATCH_OK`, `RUN_ID`, `RUN_URL`, `CONCLUSION`, and `SMOKE_FAILED_PLATFORMS`.
- If the smoke workflow concludes failure, the underlying script exits non-zero. When invoked via `make feature-smoke`, GNU make typically exits with code **2** on any recipe failure; treat `DISPATCH_OK=1` + `RUN_URL` as “dispatch succeeded” and use `RUN_URL` to inspect logs (do not rerun just to obtain a run id).
- If the failure is due to self-hosted runner provisioning (e.g., missing/permission-denied `/run/substrate.sock`), the dispatcher will set `RUNNER_MISPROVISIONED=1` and provide `RUNNER_MISPROVISIONED_REASON`; treat this as “fix runner, then retry” (do not thrash reruns).

### Dispatcher timeouts (do not tune unless needed)

The dispatchers are hardened to avoid “silent hang” failure modes (e.g., runner outages, `gh` API flakiness, or a stuck remote-branch cleanup).

Defaults are intentionally generous for normal operation:
- Overall wait for a workflow run to complete is **2 hours**.

You should not need to change these, but if you must (infra incident, runner backlog), both dispatchers accept env var knobs:

- Feature Smoke (`make feature-smoke` → `scripts/ci/dispatch_feature_smoke.sh`)
  - `FEATURE_SMOKE_WATCH_TIMEOUT_SECS` (default `7200`)
  - `FEATURE_SMOKE_WATCH_INTERVAL_SECS` (default `15`)
  - `FEATURE_SMOKE_GH_TIMEOUT_SECS` (default `120`) — per `gh` call
  - `FEATURE_SMOKE_GIT_PUSH_TIMEOUT_SECS` (default `300`) — push + cleanup
  - `FEATURE_SMOKE_RUN_LOOKUP_TIMEOUT_SECS` (default `120`) — wait for run id to appear

- CI Testing (`scripts/ci/dispatch_ci_testing.sh`)
  - `CI_TESTING_WATCH_TIMEOUT_SECS` (default `7200`)
  - `CI_TESTING_WATCH_INTERVAL_SECS` (default `15`)
  - `CI_TESTING_GH_TIMEOUT_SECS` (default `120`) — per `gh` call
  - `CI_TESTING_GIT_PUSH_TIMEOUT_SECS` (default `300`) — push + cleanup
  - `CI_TESTING_RUN_LOOKUP_TIMEOUT_SECS` (default `120`) — wait for run id to appear

## Platform-fix note (important)

GitHub Actions workflows in this repo are **validation mechanisms**. They run smoke scripts, but they do not commit fixes back to the repository.

Platform-fix work happens on the corresponding platform machine:
- create a platform-fix branch + worktree,
- apply the fix,
- commit,
- re-run smoke via `make feature-smoke`,
- do not merge to the orchestration branch (the final aggregator integration task merges platform-fix branches, re-runs smoke, and is the only FF-merge back).

## WSL testing note

This repo treats WSL as “Linux-in-WSL” and runs it on a dedicated self-hosted runner labeled `[self-hosted, Linux, wsl]`.
Do not assume WSL is available on GitHub-hosted Windows runners, and do not couple WSL coverage to the Windows runner.

## Public repo safety note (self-hosted)

Self-hosted runners must never be exposed to untrusted code. For this repo:
- Keep self-hosted jobs behind `workflow_dispatch` only (no `pull_request` / fork triggers).
- Restrict who can trigger workflows (write access required) and who can administer runners.
