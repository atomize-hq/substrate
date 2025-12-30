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
  - `X-integ-linux` (platform-fix): runs Linux smoke; if it fails, applies fixes on a Linux machine/worktree and re-runs smoke.
  - `X-integ-macos` (platform-fix): same for macOS.
  - `X-integ-windows` (platform-fix): same for Windows.
  - optional: `X-integ-wsl` (platform-fix): same for WSL (Linux-in-WSL).
  - `X-integ` (final aggregator): merges any platform-fix branches, re-runs `make integ-checks` on the primary dev platform, and confirms cross-platform smoke is green.

This preserves parallelism and makes it explicit which validations happened where.

Only include platform-specific integration tasks for platforms that are actually required by the feature’s platform guarantees (as defined in ADR/spec/contract). If a feature is Linux-only, do not add macOS/Windows integration tasks.

## Planning Pack requirement (schema v2)

If a planning pack opts into the platform-fix model, encode it in `tasks.json`:
- `meta.schema_version: 2`
- `meta.platforms_required: ["linux","macos","windows"]` (plus `"wsl"` if required)

The mechanical tasks validator enforces the required task shape when `meta.schema_version >= 2` and `meta.platforms_required` is present:
- `scripts/planning/validate_tasks_json.py`

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

## Trigger mechanism (repeatable)

GitHub Actions must run on a ref that exists on the remote. The repeatable pattern is:
1) Create a throwaway remote branch from the integration worktree commit (the code under test)
2) Dispatch the workflow from a stable workflow ref, checking out the throwaway branch (`checkout_ref`)
3) Wait for success/failure
4) Merge if green
5) Delete the throwaway branch

Helper script (requires `gh` auth):
- `scripts/ci/dispatch_feature_smoke.sh`
  - Defaults to dispatching from a stable workflow ref (`feat/policy_and_config`) and using `runner_kind=self-hosted`; override with `--workflow-ref` / `--runner-kind` if needed.

## Platform-fix note (important)

GitHub Actions workflows in this repo are **validation mechanisms**. They run smoke scripts, but they do not commit fixes back to the repository.

Platform-fix work happens on the corresponding platform machine:
- create a platform-fix branch + worktree,
- apply the fix,
- commit,
- re-run smoke via `scripts/ci/dispatch_feature_smoke.sh`,
- fast-forward merge back to the orchestration branch.

## WSL testing note

This repo treats WSL as “Linux-in-WSL” and runs it on a dedicated self-hosted runner labeled `[self-hosted, Linux, wsl]`.
Do not assume WSL is available on GitHub-hosted Windows runners, and do not couple WSL coverage to the Windows runner.

## Public repo safety note (self-hosted)

Self-hosted runners must never be exposed to untrusted code. For this repo:
- Keep self-hosted jobs behind `workflow_dispatch` only (no `pull_request` / fork triggers).
- Restrict who can trigger workflows (write access required) and who can administer runners.
