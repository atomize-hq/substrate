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

## Runner toolchain contract (self-hosted)

Self-hosted runners must have a working Rust toolchain bootstrap without emitting ignorable `error:` lines.

Contract:
- Provision **rustup** for the runner user and ensure the runner service PATH includes `~/.cargo/bin` (so `rustup` is discoverable in non-interactive shells).
- Avoid “mixed” installs (system Rust + rustup) when possible; if system Rust exists under `/usr/bin`, ensure rustup-managed tools take precedence in PATH.

Common setup (runner installed under `/opt/actions-runner`):
- The runner service sets PATH from `/opt/actions-runner/.path` (see `/opt/actions-runner/runsvc.sh`).
- Ensure `/opt/actions-runner/.path` begins with `/home/github-runner/.cargo/bin` (or the runner user’s equivalent).
- Restart the runner service after changes.

## Recommended task structuring

For cross-platform work, split integration into:
- **Option A (validation-only):** one integration task runs the smoke workflow for the feature’s **behavior platforms** (P3-008) and records the run id/URL in `session_log.md`.
  - Prefer a single dispatch with `platform=behavior` (the workflow reads `tasks.json` and runs only required behavior platforms).
- **Option B (platform-fix when needed):** split integration into core + platform-fix tasks:
  - `X-integ-core` (core integration): merges code+tests and gets primary-platform green.
  - `X-integ-linux` (platform-fix): if Linux is a behavior platform, runs Linux smoke (and bundled WSL smoke if required); otherwise treats Linux as CI parity-only.
  - `X-integ-macos` (platform-fix): if macOS is a behavior platform, runs macOS smoke; otherwise treats macOS as CI parity-only.
  - `X-integ-windows` (platform-fix): if Windows is a behavior platform, runs Windows smoke; otherwise treats Windows as CI parity-only.
  - optional: `X-integ-wsl` (platform-fix): only create when WSL divergence is likely/expected and you want independent ownership/retry loops.
  - `X-integ` (final aggregator): merges any platform-fix branches, re-runs `make integ-checks` on the primary dev platform, and confirms behavioral smoke + CI parity gates are green.

This preserves parallelism and makes it explicit which validations happened where.

Platform task inclusion rules (P3-008):
- Include platform-fix integration tasks for **CI parity platforms** (`meta.ci_parity_platforms_required` / legacy `meta.platforms_required`), because CI failures on those platforms are blocking.
- Smoke scripts are required only for **behavior platforms** (`meta.behavior_platforms_required`); do not force smoke scripts for CI parity-only platforms.

## Slice-scoped smoke (recommended)

When a Planning Pack is executed slice-by-slice, feature smoke should be able to run at earlier slices without requiring later-slice functionality.

Mechanism:
- When dispatching smoke for a given slice, pass `SMOKE_SLICE_ID="<slice>"` to `make feature-smoke`.
- The workflow exports this as `SUBSTRATE_SMOKE_SLICE_ID` for smoke scripts.
- Smoke scripts should use `SUBSTRATE_SMOKE_SLICE_ID` to run only the checks that are expected to be present at that slice (and treat an empty value as “full feature smoke”).

## CI Audit + Evidence Ledger (recommended; reduces redundant multi-OS runs)

Problem:
- Cross-platform validation can become redundant when CI is re-dispatched multiple times against effectively the same code (or docs/planning-only changes).
- Dispatchers often create throwaway branches, which can make it ambiguous (from GitHub metadata alone) what exact commit SHA was “the code under test”.

Solution:
- Use the advisory CI audit tool before dispatch, and record evidence after dispatch into a local ledger:
  - Audit: `scripts/ci-audit/ci_audit.sh`
  - Record: `scripts/ci-audit/ci_audit_record.sh`

Ledger location (recommended; survives `cargo clean`; do not commit):
- `$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl`

Policy:
- Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke. The audit should show `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.

Workflow (operator/integration tasks):
1) Before dispatching Feature Smoke or CI Testing, run the audit:
   - CI Testing: `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "<orch-branch>" --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl"`
   - Feature Smoke: `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "<orch-branch>" --feature-dir "$FEATURE_DIR" --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl"`
2) If `RECOMMEND=skip`, do not dispatch that gate; record the audit output + last-green run id/URL in `session_log.md`.
3) If you dispatch, record evidence after the run completes:
   - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "<orch-branch>" --run-id "<run-id>" --tested-sha "<sha>" --feature-dir "$FEATURE_DIR"`
   - Use `<run-id>` from `RUN_ID=...` and `<sha>` from `HEAD=...` (both are printed by the dispatchers).

## Smoke script hygiene (recommended)

To avoid “CI-only” failures caused by script brittleness:
- Use a unique scratch dir per run (Linux/macOS: `mktemp -d`; Windows: `New-Item -ItemType Directory` under `$env:TEMP`) and ensure cleanup is safe.
- Avoid relying on raw JSON string formatting; when comparing JSON, normalize (e.g., `jq -cS` on Unix; `ConvertFrom-Json | ConvertTo-Json -Compress` on PowerShell).
- Avoid PowerShell string interpolation edge cases (`"${var}"` when adjacent to `:` or other tokens).

## Planning Pack requirement (schema v2)

Cross-platform Planning Packs are required to use the platform-fix model. Encode it in `tasks.json`:
- `meta.cross_platform: true`
- `meta.schema_version: 2` (or `3` when automation is enabled)
- Declare both scopes (P3-008):
  - `meta.behavior_platforms_required: [...]` (platforms with behavior guarantees; smoke scripts required here)
  - `meta.ci_parity_platforms_required: ["linux","macos","windows"]` (platforms that must be green in CI parity gates; platform-fix tasks required here)
  - Legacy compatibility: `meta.platforms_required` is accepted as an alias for `meta.ci_parity_platforms_required`.
- If WSL coverage is required:
  - `meta.wsl_required: true`
  - `meta.wsl_task_mode: "bundled" | "separate"`
    - `"bundled"` (default/recommended): run WSL as part of `X-integ-linux` by dispatching with `--run-wsl`
    - `"separate"`: add `X-integ-wsl` as its own platform-fix task

The mechanical tasks validator enforces the required task shape when `meta.cross_platform=true`:
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
2) Dispatch the workflow from the orchestration/task ref (not `main`/`testing`), checking out the throwaway branch (`checkout_ref`)
3) Wait for success/failure
4) Merge if green
5) Delete the throwaway branch

Helper script (requires `gh` auth):
- `make feature-smoke`
  - Defaults to using `RUNNER_KIND=self-hosted` and dispatching from the current branch ref.
  - For triad execution, dispatch from the orchestration/task ref (never from `main`/`testing`). The workflow checks out the throwaway `checkout_ref` branch, so the code under test is still the integration `HEAD`.
  - When workflow files change, land workflow-file-only changes on `main` to register them before relying on dispatch from your feature refs.

Smoke result interpretation (important):
- The smoke dispatcher always prints machine-parseable fields including `DISPATCH_OK`, `RUN_ID`, `RUN_URL`, `CONCLUSION`, and `SMOKE_FAILED_PLATFORMS`.
- If the smoke workflow concludes failure, the underlying script exits non-zero. When invoked via `make feature-smoke`, GNU make typically exits with code **2** on any recipe failure; treat `DISPATCH_OK=1` + `RUN_URL` as “dispatch succeeded” and use `RUN_URL` to inspect logs (do not rerun just to obtain a run id).
- If the failure is due to self-hosted runner provisioning (e.g., missing/permission-denied `/run/substrate.sock`), the dispatcher will set `RUNNER_MISPROVISIONED=1` and provide `RUNNER_MISPROVISIONED_REASON`; treat this as “fix runner, then retry” (do not thrash reruns).

Operational default (avoid remote temp-branch buildup):
- Keep cleanup enabled (`CLEANUP=1` / `--cleanup`) so temp branches are deleted automatically.
- If a dispatch is interrupted before cleanup, manually delete `tmp/feature-smoke/...` or `tmp/ci-testing/...` branches on the remote to avoid buildup.

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
  - Use `--mode compile-parity` for fast cross-platform parity (fmt/check/clippy only).
  - Use `--mode quick` for automation selection (skip docs/cross-build).
  - Use `--mode full` (default) as the final CI gate before merging to `testing`.
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
- do not merge to the orchestration branch (the final aggregator integration task merges platform-fix branches, re-runs smoke, and is the only merge-back point to orchestration).

## WSL testing note

This repo treats WSL as “Linux-in-WSL” and runs it on a dedicated self-hosted runner labeled `[self-hosted, Linux, wsl]`.
Do not assume WSL is available on GitHub-hosted Windows runners, and do not couple WSL coverage to the Windows runner.

## Public repo safety note (self-hosted)

Self-hosted runners must never be exposed to untrusted code. For this repo:
- Keep self-hosted jobs behind `workflow_dispatch` only (no `pull_request` / fork triggers).
- Restrict who can trigger workflows (write access required) and who can administer runners.
