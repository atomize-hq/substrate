# Platform Integration & CI Standard

Goal:
- Enable parallel, reproducible integration validation across Linux/macOS/Windows (and optionally WSL) without blocking local development.

## Model

When a feature has platform smoke scripts under:
- `docs/project_management/next/<feature>/smoke/`

…treat platform validation as integration work that may run in different execution environments:
- Linux: typically run locally
- macOS/Windows: typically run on GitHub Actions runners
- WSL: treat as a distinct platform (best-effort on hosted runners; reliably on self-hosted runners)

## Recommended task structuring

For cross-platform work, split integration into:
- `X-integ-linux` (runner: local)
- `X-integ-macos` (runner: github-actions)
- `X-integ-windows` (runner: github-actions)
- optional: `X-integ-wsl` (runner: github-actions or self-hosted)
- `X-integ` (final aggregator): depends on all platform integ tasks, performs merge + `make integ-checks` on the primary dev platform, and records the combined result in `session_log.md`.

This preserves parallelism and makes it explicit which validations happened where.

Only include platform-specific integration tasks for platforms that are actually required by the feature’s platform guarantees (as defined in ADR/spec/contract). If a feature is Linux-only, do not add macOS/Windows integration tasks.

## `tasks.json` (recommended fields)

For integration tasks, add optional fields:
- `platform`: `linux|macos|windows|wsl`
- `runner`: `local|github-actions|manual`
- `workflow`: a GitHub Actions workflow name/file (e.g., `.github/workflows/feature-smoke.yml`)

These fields are optional for now, but recommended for cross-platform planning packs.

## GitHub Actions workflow

This repo provides a reusable workflow:
- `.github/workflows/feature-smoke.yml`

It runs the feature-local smoke script for the selected platform(s).

## Trigger mechanism (repeatable)

GitHub Actions must run on a ref that exists on the remote, so the repeatable pattern is:
1) Create a throwaway remote branch from the integration worktree commit
2) Dispatch the workflow against that branch
3) Wait for success/failure
4) Merge if green
5) Delete the throwaway branch

Helper script (requires `gh` auth):
- `scripts/ci/dispatch_feature_smoke.sh`

## WSL testing note

WSL testing on GitHub-hosted Windows runners is not guaranteed to be available/configured across all environments.
For reliable WSL coverage, prefer:
- a self-hosted Windows runner with WSL pre-provisioned, or
- a dedicated CI environment where WSL is known-good.

The workflow includes a WSL job stub that is intended for self-hosted runners.
