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
- **Triggers**: `pull_request` targeting `testing` (commits land via PRs), plus
  `workflow_call`/`workflow_dispatch` so nightly/promote flows can reuse the
  pipeline without double-running on direct pushes. The `main` branch inherits
  coverage through promotion + release tags.
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
- **Trigger**: `workflow_dispatch` from maintainers (testing branch is gated by
  PRs; promotion is the only path into `main`).
- **Steps**:
  1. Checkout `testing`/`main` (full history, recursive submodules) and determine
     the next release version. If the operator provides `next_version`, use it;
     otherwise parse the latest `v*` tag and auto-increment the patch component.
  2. Run `tools/version-bump` with the computed version and push the commit back
     to `testing` (skipped when `dry_run=true`).
  3. Validate that at least one `ci-testing.yml` workflow finished successfully
     in the last 30 minutes (any commit on `testing` counts); record its URL in
     the summary. This avoids waiting for a brand-new run while still enforcing
     “CI was green recently.”
  4. Fast-forward `main` to match `testing`, then push (or skip when dry-running).
  5. Always create/push tag `v<next_version>` and immediately dispatch
     `release.yml` against that tag so publishing starts without manual pushes.
  6. Emit a markdown summary (CI link, bump commit, fast-forward SHAs, tag, and
     release trigger result) for audit trails.
- **Branch protections**: rely on the `Promote to Main` commit status, allowing
  us to toggle repo rulesets during installer work without rewriting pipelines.

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

#### 3.5.1 Artifact & Dependency Matrix

The last manual cut (`release/0.2.0-beta/` in commit
[`e350bdb`](https://github.com/atomize-hq/substrate/tree/e350bdb1355fc5a7daa742b9a0524457a53b1025))
packaged a complete `bin/ + docs/ + scripts/` tree per host platform. Reinstating
the one-command installers requires every artifact in the table below:

| Artifact | Produced from | Targets / format | Consumed by | Notes |
| --- | --- | --- | --- | --- |
| `substrate` (host CLI) | `cargo build -p substrate` | `aarch64/x86_64` for Linux & macOS (`.tar.gz`), `x86_64-pc-windows-msvc` (`.zip`) | All hosts | Provides the CLI, shim deployer, and `world doctor`; copied to `~/.substrate/bin` and used by every verification command. |
| `substrate-shim` (PATH interceptor) | `cargo build -p substrate` (binary target `substrate-shim`) | Same as host CLI targets | All hosts | The shim is required for PATH interception and fast policy lookups; it must ship next to `substrate` so `install-substrate.{sh,ps1}` can deploy/update shims without rebuilding. |
| `world-agent` (guest daemon) | `cargo build -p world-agent` | Linux glibc: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (`.tar.gz`) | Linux systemd service, macOS Lima VM, Windows WSL | Linux hosts install it directly; macOS/Windows copy the same binary into their guest (`bin/linux/world-agent`) before enabling `substrate-world-agent.service`. |
| `substrate-forwarder` (named-pipe bridge) | `cargo build -p substrate-forwarder` | `x86_64-pc-windows-msvc` (`.zip`) | Windows host | Launched by `scripts/windows/wsl-warm.ps1` / `start-forwarder.ps1` to proxy requests from the host CLI into WSL via named pipe or optional TCP bridge. |
| `host-proxy` | `cargo build -p host-proxy` | All host triples (`.tar.gz` / `.zip`) | Linux/macOS/Windows hosts | Runs alongside `substrate` when agent APIs need an HTTP(S) proxy; PowerShell uninstall explicitly removes it, so releases must continue to ship the binary. |
| `substrate-support.tar.gz` / `.zip` | `dist/scripts/collect-supporting-artifacts.sh` | Platform-agnostic | All installers | Bundles `docs/{INSTALLATION,CONFIGURATION,WORLD}.md` plus `scripts/linux`, `scripts/mac`, `scripts/windows`, `scripts/wsl`, and `scripts/substrate` so every host has the Lima/WSL helpers, systemd units, doctor/smoke tests, and uninstallers. |

`release.yml` will continue to build every row above at tag time, but the
release page will stay clean: a follow-on “bundle” job will combine the
per-crate outputs into **one archive per host target** (matching the legacy
`release/<target>` layout) and upload only those bundles plus the support
tarball/zip. All other tarballs remain as workflow artifacts for debugging, so
users see exactly five public downloads instead of dozens.

#### 3.5.2 Guest World Provisioning Flow (Legacy Reference)

- **macOS (Lima)** – `install-substrate.sh` shells into `scripts/mac/lima-warm.sh`
  (which templatises `scripts/mac/lima/substrate.yaml` via `envsubst`) to boot a
  `substrate` Lima VM with Ubuntu 24.04, nftables, dnsmasq, and systemd enabled.
  After the VM is running, the installer copies the Linux `world-agent` binary
  into `/usr/local/bin/substrate-world-agent`, installs the unit from
  `scripts/mac/substrate-world-agent.service`, and starts it via `limactl shell
  substrate sudo systemctl enable --now substrate-world-agent`. Health is
  verified with `scripts/mac/lima-doctor.sh` plus `scripts/mac/smoke.sh`.
- **Windows (WSL)** – `scripts/windows/install-substrate.ps1` unwraps the support
  bundle so it can run `scripts/windows/wsl-warm.ps1`. That helper downloads a
  Ubuntu Noble WSL image when necessary, runs `scripts/wsl/provision.sh` inside
  the distribution to install nftables/seccomp/systemd units, places the Linux
  `world-agent` under `/usr/local/bin/substrate-world-agent`, and restarts the
  service. It then ensures `substrate-forwarder.exe` is available, launches it
  with `--pipe \\.\pipe\substrate-agent`, and validates connectivity using
  `scripts/windows/pipe-status.ps1`, `wsl-doctor.ps1`, and `wsl-smoke.ps1`.
- **Linux hosts** – `install-substrate.sh` installs `world-agent` directly under
  `/usr/local/bin`, writes `/etc/systemd/system/substrate-world-agent.service`
  (with hardened `ReadWritePaths`, seccomp prerequisites, and
  `SUBSTRATE_AGENT_TCP_PORT=61337`), and reloads/enables the unit before running
  `substrate world doctor --json`.
- **Support bundle checks** – Regardless of platform, installers rely on the
  support archive for uninstallers, `lima-stop.sh`, `windows/start-forwarder.ps1`,
  and smoke suites, so missing scripts break the acceptance checks even if the
  binaries exist.

#### 3.5.3 Automated Release Acceptance Criteria

1. GitHub Releases must attach **only** the per-target aggregate bundles
   (`substrate-v<ver>-<target>.{tar.gz,zip}`) plus `substrate-support.{tar.gz,zip}`
   and matching checksum files, keeping the release UI uncluttered.
2. Each bundle has to unpack into the legacy layout (`bin/`, `bin/linux/`,
   `docs/`, `scripts/`) that already contains every artifact from §3.5.1 so the
   installers perform zero additional downloads.
3. `substrate-support.tar.gz` and `substrate-support.zip` must be published for
   every tag, contain the full `docs/` set plus `scripts/{linux,mac,windows,wsl,substrate}`,
   and remain byte-for-byte reproducible so checksum verification works.
4. `SHA256SUMS` must include entries for both the aggregate bundles and the
   support bundle; installers will refuse to run if verification fails.
5. `dist-manifest.json` needs to list the per-crate outputs and the aggregate
   bundles so the release job fails early if `cargo-dist` (or the bundler) skips
   a component.
6. A release is considered complete only if `install-substrate.sh` (Linux/macOS)
   and `install-substrate.ps1` (Windows) can consume the aggregated bundles,
   deploy shims, provision Lima/WSL, and run the bundled doctor/smoke scripts
   without manual intervention.

#### 3.5.4 Current Automation Status (2025-11-08)

- **Bundle assembly live** – `release.yml` now copies every per-crate dist
  artifact plus `substrate-support.{tar.gz,zip}` into a single archive per host
  target. These bundles already include `substrate`, `substrate-shim`,
  `host-proxy`, `world-agent`, and (on Windows) `substrate-forwarder`, so the
  installers only need one download per platform plus the support archive.
- **Installer UX surfaced in release notes** – the GitHub Release body links to
  the shell/PowerShell one-liners and uninstall commands so operators can copy
  them directly. The cargo-dist quickstart tables are suppressed to avoid
  dangling links to non-existent artifacts.
- **Known gap: source tarball** – Previously `git archive` omitted nested
  submodules (e.g., the legacy `third_party/reedline` fork), so the source tarball lacked our
  REPL fork. As of v0.2.12 we consume Reedline directly from crates.io, so the
  source tarball now contains all required Rust sources without additional
  steps.

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

### 5.1 Multi-artifact Release Blueprint

- **Model the binaries as first-class dist apps** – Add
  `[package.metadata.dist]` blocks for `substrate`, `world-agent`,
  `substrate-forwarder`, and `host-proxy`, enabling them inside
  `dist-workspace.toml`. World agent only needs glibc Linux targets (x86_64 and
  aarch64); `substrate-forwarder` is Windows-only; `host-proxy` should match the
  host target matrix so uninstall scripts stay accurate.
- **Add a bundler job per tag** – After `cargo dist build` finishes, a new
  workflow step downloads the per-crate archives plus `substrate-support.*`,
  recreates the legacy layout for each host triple, repackages it as
  `substrate-v<ver>-<target>.{tar.gz,zip}`, and generates checksums. Only these
  bundles (and the support archive) are uploaded to the GitHub Release, while the
  raw per-crate artifacts stay in the workflow for debugging.
- **Keep installers custom, but point them at the bundles** – Leave `installers
  = []` in the dist config so we continue shipping the hand-written shell/PowerShell
  flows. `install-substrate.{sh,ps1}` just download the single bundle for their
  host target, unfold it, then proceed with Lima/WSL provisioning.
- **Retain and harden the support bundle step** – The existing
  `dist.artifacts.extra` entry that runs
  `dist/scripts/collect-supporting-artifacts.sh` remains the canonical way to
  produce `substrate-support.{tar.gz,zip}`; the bundler job consumes those files
  when building each host archive.
- **Release notes driven from template** – `release.yml` now feeds
  `dist/release-template.md` through `envsubst` so GitHub releases always show
  the curl|bash and PowerShell installers plus a table of OS/arch bundles and
  checksum links (no more `null` body when cargo-dist omits announcement text).
- **Pre-publish verification** – Developers and CI should run
  `cargo dist plan --ci github --output-format=json` (and ideally
  `cargo dist build --artifacts=local`) before tagging to confirm every crate and
  bundle is scheduled. Publish should be blocked if `SHA256SUMS` lacks a row for
  any required bundle/support artifact. For now the `source.tar.gz` output does
  not include git submodules (legacy note from when `third_party/reedline` was a
  submodule); this gap was removed in v0.2.12 when we switched to the upstream
  Reedline crate.

### 5.4 Source Tarball Completeness (new)

- **Current behavior** – `git archive` now captures all Rust sources because we
  no longer rely on submodules for Reedline (the old `third_party/reedline` fork
  was removed in v0.2.12).
- **Required change** – None. Leave this section for historical context; if new
  submodules are added in the future, re-open the investigation to ensure the
  source tarball stays complete.
- **Acceptance** – Document in release notes that the source tarball is
  self-contained for v0.2.12+.

### 5.2 Installer Re-alignment Outline

- **Linux/macOS shell** – Keep `install-substrate.sh` as the single entry point,
  but simplify it to download exactly one bundle (based on host triple), verify
  its checksum, and then proceed with the existing steps (link `bin/`, stage
  `bin/linux/world-agent`, unpack `scripts/`, run Lima provisioning + doctor +
  smoke). No additional per-crate downloads are needed post-bundle.
- **Windows PowerShell** – Update `install-substrate.ps1` to download the
  Windows bundle, verify its checksum (local or remote), stage `bin/` +
  `bin\linux/`, then run `wsl-warm.ps1`, `wsl-doctor.ps1`, `pipe-status.ps1`,
  and `wsl-smoke.ps1` so the one-command UX fails fast when WSL services or the
  forwarder misbehave.
- **Verification & docs** – Add a release checklist that records the doctor and
  smoke outputs per platform, document the new artifact names/URLs in
  `docs/INSTALLATION.md` and `docs/CONFIGURATION.md`, and keep
  `docs/RELEASE_PIPELINE_PLAN.tasks.json` updated with the validation evidence.

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
