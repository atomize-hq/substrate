# Release Pipeline Implementation Plan (Updated 2025-11-04)

Substrate’s release process is moving from manual artifact builds toward a fully
automated GitHub Actions pipeline. This document describes the target branch
model, workflow inventory, tooling, and rollout steps required to deliver
repeatable builds, comprehensive validation, and painless promotions.

## 1. Goals and Non-Goals

### Goals
- Keep `main` continuously releasable while allowing rapid iteration on feature
  branches.
- Automate all validation (linting, testing, integration smoke tests) for the
  `testing` and `main` branches.
- Produce cross-platform release artifacts (Linux, macOS, Windows) directly
  from Git tags with checksums and release notes.
- Provide nightly soak coverage that only runs when new code has landed.
- Document a procedure that any maintainer (or agent) can follow end-to-end.

### Non-Goals (for now)
- Publishing to external package managers (brew tap, winget, snap, etc.).
- Code signing or notarization.
- Replacing GitHub Releases as the distribution channel.

## 2. Branch and Environment Model

| Branch      | Purpose                               | Allowed Sources                           |
|-------------|----------------------------------------|-------------------------------------------|
| `feature/*` | Personal or team feature work          | Developers push directly                  |
| `testing`   | Integration & soak testing             | PRs from feature branches only            |
| `main`      | Production-ready / release branch      | Fast-forward from `testing` via protected PR |

- Protect `testing` and `main`:
  - Require status checks from the CI workflow described in §3 (ruleset enforced).
  - Require at least one approving review.
  - Disallow force pushes.
- Lock the GitHub Actions “production” environment so only the release workflow
  (running on tags from `main`) can deploy. This prevents accidental release
  from feature or testing branches.
- Document the merge discipline: no direct commits to `main`; the promotion
  workflow (see §3.4) is the only mechanism to move code from `testing`.

## 3. Workflow Inventory

### 3.1 Continuous Integration (`ci-testing.yml`)
- **Triggers**: `pull_request` targeting `testing`, `push` to `testing`. The
  `main` branch is exercised via the promotion workflow and release tags.
- **Jobs**:
  1. **Lint & Unit (matrix)**: Runs on `ubuntu-24.04`, `windows-2022`,
     `macos-14`. Steps include `cargo fmt`, `cargo clippy`, `cargo test
     --workspace`, and platform-specific smoke commands:
     - Linux: `substrate --shim-deploy --dry-run`
     - macOS: ensure Lima scripts pass `shellcheck`
     - Windows: run PowerShell installers with `-WhatIf`
     - Non-musl targets also run `cargo rustc -p substrate-telemetry --profile
       dist --crate-type=rlib,cdylib` so the cdylib is exercised without
       breaking musl builds.
 2. **Documentation & Packaging**: Runs `cargo doc --no-deps` and basic shell
     linting. Comprehensive `dist` packaging validation is exercised in the
     nightly workflow (see §3.2) so PR builds stay fast while keeping the plan
     up to date. Installer scripts are being rewritten to fetch the per-app
     archives that `cargo-dist` produces and then assemble the familiar
     `bin/`+`scripts/` layout at install time (replacing the legacy "fat"
     bundles).
 3. **Cross Builds**: Uses `cross` to compile `aarch64-unknown-linux-gnu` and
    `x86_64-unknown-linux-musl` artifacts with the `dist` profile. The job
    installs target-specific prerequisites (e.g., `libseccomp-dev` and
    `libseccomp-dev:arm64`) before invoking `cross build`/`cross rustc`, and
    only the GNU target emits the telemetry cdylib.
- **Caching**: Use `actions/cache` with a key combining runner OS and the
  `hashFiles` of `Cargo.lock` to speed up builds. `target/` should be restored
  per toolchain triple to support reuse across jobs.

### 3.2 Nightly Validation (`nightly.yml`)
- **Trigger**: `workflow_dispatch` and `repository_dispatch` events. Scheduled execution is delegated to `.github/workflows/nightly-scheduler.yml`, which runs on the default branch and emits a dispatch targeting `refs/heads/testing`.
- **Scope**: Runs the same jobs as the CI workflow plus extended integration
  suites (world agent integration tests, replay fixtures). The workflow reuses
  `ci-testing.yml` through `workflow_call`, then executes a Linux-only
  "extended tests" job that runs nightly-only coverage: `cargo test --workspace
  --all-targets -- --ignored`, targeted `replay`/`world-agent` suites, and a
  `dist plan --ci github --output-format=json` sanity check. This validates
  packaging manifests without producing release artifacts; the actual build and
  hosting steps remain exclusive to the release workflow in §3.3.
- **Change Detection**: A `preflight` job skips heavy work when the latest
  `testing` commit matches the SHA stored in the persisted `nightly-state.json`
  artifact. Artifacts are retained for 90 days so the workflow avoids duplicate
  executions.
- **Reporting**: Failures continue to open actionable issues (via
  `peter-evans/create-issue-from-file`) with `nightly`/`ci-failure` labels and
  links back to the run.

### 3.3 Release Automation (`release.yml`)
- **Triggers**: `push` tags that match `v*.*.*` (the workflow also accepts bare
  semver tags, but we currently rely on the `v` prefix promoted by
  `promote.yml`).
- **What ships today** (validated on release run
  [19153854458](https://github.com/atomize-hq/substrate/actions/runs/19153854458)):
  - Only the `substrate` binary is built for the five supported targets
    (`aarch64/x86_64` for macOS+Linux and `x86_64-pc-windows-msvc`).
  - Assets uploaded: `substrate-<target>.tar.xz` (or `.zip` on Windows),
    `sha256.sum`, `dist-manifest.json`, and GitHub’s auto-generated source
    archives. No installers, docs, or helper scripts are attached.
- **Jobs** (generated by `dist init`):
  1. **Plan** (Ubuntu) runs `dist host --steps=create --tag=<tag>` to emit the
     manifest and matrix.
  2. **Build Local Artifacts** executes `dist build --artifacts=local` on the
     platform runners. Linux runners install `libseccomp-dev` and the
     `:arm64` variant before invoking `dist` so the world crate links.
  3. **Build Global Artifacts** aggregates checksum files and copies manifests.
  4. **Host** uploads the artifacts to the GitHub release (prerelease flag is
     driven by the tag suffix).
- **Gap**: the legacy installer depended on several additional payloads that
  are absent from the automated release:
  - Linux `world-agent` binary (used by Lima on macOS and WSL on Windows).
  - `substrate-forwarder` + helper scripts (`start-forwarder.ps1`,
    `lima-warm.sh`, `wsl-warm.ps1`, etc.).
  - `substrate-support.tar.gz` (docs + scripts referenced by the installer).
  As long as the release workflow only publishes `substrate`, the new
  `install-substrate.{sh,ps1}` cannot reproduce the one-command UX.

### 3.4 Promotion Workflow (`promote.yml`)
- **Trigger**: `workflow_dispatch` from maintainers.
- **Steps**:
  1. Checkout `testing` and `main` (full history, recursive submodules).
  2. Run `tools/version-bump` to update workspace manifests to `next_version`
     and push the bump back to `testing` (skipped when `dry_run=true`).
  3. Wait for the latest `ci-testing.yml` run on the bumped commit and post a
     commit status (`Promote to Main`).
  4. Fast-forward `main` to match `testing`, then push (or dry-run skip).
  5. Optionally tag `v<next_version>` after the fast-forward.
  6. Emit a markdown summary (CI link, bump commit, fast-forward SHAs, tag).
- **Branch protections**: now rely on the `Promote to Main` status rather than
  direct PR checks, so we can temporarily drop rulesets while iterating on the
  installer without rewriting the workflow.

### 3.5 Legacy Installer Expectations
The previous manual release (commit
[`e350bdb`](https://github.com/atomize-hq/substrate/tree/e350bdb1355fc5a7daa742b9a0524457a53b1025))
structured artifacts under `release/<version>/` and the install scripts pulled
directly from that directory. Those scripts remain the canonical reference for
the desired UX and must be studied before changing the automated pipeline:

- `scripts/substrate/install-substrate.sh` / `uninstall-substrate.sh`
- macOS Lima helpers (`scripts/mac/lima/*.sh`, `scripts/mac/lima/substrate.yaml`,
  `scripts/mac/substrate-world-agent.service`, `lima-warm.sh`, `lima-stop.sh`,
  `lima-doctor.sh`, `smoke.sh`)
- Windows PowerShell install/uninstall scripts plus helper modules
  (`pipe-status.ps1`, `start-forwarder.ps1`, `wsl-*.ps1`) and the WSL provisioner
  `scripts/wsl/provision.sh`

Key functional requirements captured by those scripts:

1. **World Agent Packaging** – macOS (Lima) and Windows (WSL) bootstrap a Linux
   guest and drop a `world-agent` binary inside, running it as
   `substrate-world-agent.service`. The binary must be published alongside the
   host `substrate` executable.
2. **Forwarder & Support Scripts** – Windows requires the TCP pipe forwarder and
   helper scripts to maintain connectivity; both macOS and Windows rely on a
   support bundle (`substrate-support.tar.gz`) that includes docs, doctor/smoke
   tests, and templated services.
3. **Single Command UX** –
   `curl -fsSL .../install-substrate.sh | bash` (Linux/macOS) and a single
   PowerShell command (Windows) must provision shims, Lima/WSL, the world agent,
   forwarder, and support scripts without manual steps.

Until release automation produces the same payload, the new installers will
continue to fail when they cannot find those files in the GitHub release.

### 3.6 Linux World Hardening
- **Status**: Completed (gated in `crates/world`).
- **Behavior**:
  - On glibc targets, `libseccomp` is linked and the baseline filter is applied.
  - On musl targets, the `world` crate skips the seccomp baseline (with a warning), keeping cross builds green while acknowledging reduced isolation.
- **Next steps**: Keep the warning in place until a musl-compatible seccomp path is available; document the reduced guarantees in platform guides.

## 4. Tooling and Key Actions

- **`dist`** CLI: orchestrates plan/build/upload, and supports custom
  templates. Reference: <https://axodotdev.github.io/cargo-dist/>.
- **`houseabsolute/actions-rust-cross`**: builds non-native Linux targets using
  preconfigured cross toolchains.
- **`Cross.toml` pre-build hooks**: install platform libraries (e.g.,
  `libseccomp-dev` and multi-arch variants) in the cross containers so linker
  dependencies resolve for GNU targets while musl remains self-contained.
- **`actions/github-script`**: used for nightly change detection and branch
  promotion guard rails.
- **`peter-evans/create-issue-from-file`**: opens issues when nightly runs fail.
- **`softprops/action-gh-release`** (optional): only needed if we bypass
  `cargo dist upload`.
- **Shell quality gates**: `shellcheck`, `shfmt` via `ludeeus/action-shellcheck`
  and `shfmt` Docker image.

## 5. Implementation Roadmap (revised)

1. **Installer & Packaging Audit (in progress)**
   - Deep-dive the legacy scripts listed in §3.5 and document every artifact,
     dependency, and service they provision (macOS Lima, Windows WSL, Linux
     systemd). Deliverables:
     - Updated installer acceptance criteria in `docs/RELEASE_PIPELINE_PLAN.md`.
     - A dependency matrix mapping which binaries/scripts must ship per OS.
     - Recommendations for how cargo-dist/release.yml should emit those files
       (e.g., multiple `dist` apps vs. custom packaging step).

2. **Release Workflow: Multi-artifact Packaging**
   - Extend `dist-workspace.toml` / release.yml so the automated release
     publishes:
     - Host binaries (`substrate` for all targets).
     - Guest components (`world-agent` Linux build, `substrate-forwarder`).
     - Support bundle (docs, doctor/smoke scripts, PowerShell/Lima helpers).
   - Ensure SHA files cover every artifact; attach them to the GitHub release.
   - Keep branch protections disabled during the refactor, then re-enable once
     the workflow is green end-to-end.

3. **Installer Re-alignment & Validation**
   - Update `install-substrate.sh`/`.ps1` (plus uninstall scripts) to pull the
     new artifacts from GitHub Releases instead of the old `release/` folder.
   - Reintroduce cross-platform smoke tests (Linux host, Lima, WSL) to verify
     the one-command experience.
   - Document the flow in `docs/DEVELOPMENT.md` / `docs/CONFIGURATION.md` and
     capture verification steps in `docs/RELEASE_PIPELINE_PLAN.tasks.json`.

## 6. Maintenance & Future Enhancements

- Regularly upgrade the Rust toolchain in workflows (aligned with MSRV policy).
- Monitor `cargo-dist` updates; regenerate config when new installer options or
  bug fixes land.
- Consider integrating code signing once certificates are available.
- Future distribution targets:
  - Homebrew tap automation (inspired by WezTerm’s workflow generator).
  - Winget/Flathub manifest PRs.
  - Container images (e.g., GitHub Container Registry).
- Build a release dashboard (GitHub Pages or docs) summarising latest nightly,
  testing state, and production release for quick at-a-glance health checks.

## 7. References

- `cargo-dist` documentation: <https://axodotdev.github.io/cargo-dist/>
- WezTerm CI/CD strategy (inspiration for multi-platform releases):
  DeepWiki summary – <https://deepwiki.com/search/how-does-the-repository-automa_9e7f6238-7e17-40b1-accb-4a32820dfe5e>
- Existing workflows in this repository:
  - `.github/workflows/ci-testing.yml`
  - `.github/workflows/release.yml`
  - `docs/INSTALLATION.md` / `docs/WORLD.md` (for provisioning scripts)

With this plan implemented, every change will be validated automatically, every
nightly run will execute only when required, and every Git tag will produce a
fully packaged, multi-platform release without manual intervention.
