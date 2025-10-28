# Release Pipeline Implementation Plan

This plan defines how we evolve Substrate's build and distribution workflow so
that cutting a beta or stable release automatically produces polished GitHub
release pages with per-platform artifacts, checksums, and changelog summaries.

## Objectives

- Produce reproducible binaries for Linux, macOS, and Windows from a single tag.
- Package artifacts in OS-friendly formats (tarballs, zips, AppImage/deb/rpm).
- Generate checksums and publish them alongside binaries.
- Draft GitHub release notes referencing the changelog and quickstart docs.
- Support nightly/pre-release channels while keeping stable releases simple.
- Minimise operator intervention: tagging (or scheduled nightly) should be the
  only manual step.

## Tooling Choices

1. **cargo-dist**
   - Purpose-built for Rust projects; handles matrix builds, packaging, release
     note templating, and asset uploads.
   - Supports AppImage, deb, rpm, msi/exe, dmg/zip, and tar bundles.
   - Generates GitHub Actions workflows automatically (`cargo dist init`).
2. **GitHub Actions**
   - Host the release workflow; leverage matrix runners for Linux/macOS/Windows.
   - Secure secrets (code signing, if added later) via Actions secrets.
3. **gh CLI (optional)**
   - For local dry runs or manual hotfix uploads.
4. **Artifacts bucket (future)**
   - Optional CDN or object storage if we want mirrors beyond GitHub.

## High-Level Flow

1. Developer updates `CHANGELOG.md` and bumps versions.
2. Developer runs `cargo dist check` locally to validate manifest configuration.
3. Tag is created (`v0.2.0-beta.2`, `v0.2.0`, etc.).
4. GitHub Actions workflow (`Release`) triggers on the tag push.
5. Workflow executes `cargo dist build` with matrix targets to produce binaries.
6. `cargo dist upload` attaches resulting assets and checksums to the release.
7. Release notes template pulls from changelog and quickstart doc links.
8. Nightly workflow reuses the same steps on a schedule, tagging pre-release
   builds (`nightly-YYYYMMDD`), and marks them as pre-release.

## Detailed Tasks

### 1. Project Preparation

- [ ] Add `cargo-dist` as a dev dependency (no build impact):
  ```bash
  cargo install cargo-dist
  cargo dist init
  ```
- [ ] Review the generated `dist/` directory and customise:
  - `dist/cargo-dist.toml`: configure desired installers (tar.gz, AppImage, rpm,
    deb, zip/exe/dmg), rename packages (`substrate`, `substrate-shim`, etc.).
  - Define matrix triples explicitly (linux-x86_64, linux-aarch64, macos-arm64,
    macos-x86_64, windows-x86_64).
  - Specify artifact paths for quickstart/docs to include in archives.
- [ ] Commit the `dist/` configuration and update `.gitignore` to keep only
  tracked templates (omit generated artifacts).

### 2. Release Workflow Design

Create `.github/workflows/release.yml` with triggers:
```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
```
Key jobs:

1. **Plan Job (linux)**
   - Uses `cargo dist plan` to create a build matrix and release metadata.
   - Uploads the plan as an artifact consumed by platform jobs.

2. **Build Jobs (matrix)**
   - Runners:
     - `ubuntu-22.04` (x86_64)
     - `ubuntu-22.04` with cross for `aarch64-unknown-linux-gnu` (or use
       `cross` containers).
     - `macos-14` (arm64)
     - `macos-13` (x86_64)
     - `windows-2022`
   - Steps:
     1. Checkout with submodules disabled.
     2. Install Rust toolchain pinned to MSRV/ stable.
     3. Restore caches (`~/.cargo/registry`, `target` keyed by `dist-plan`).
     4. Run `cargo dist build --plan ${{ steps.plan.outputs.path }}` (per docs).
     5. Upload build artifacts (binaries + packaging outputs) for aggregation.

3. **Publish Job (linux)**
   - Needs `WRITE` permissions to Releases.
   - Downloads plan and artifacts.
   - Runs `cargo dist upload` to:
     - Create (or update) the GitHub Release for the tag.
     - Attach all assets and generated `SHA256SUMS`.
     - Apply release notes via template (pulling from `CHANGELOG.md`).
   - Mark release as pre-release if tag contains `beta`, `rc`, or `nightly`.

### 3. Nightly Workflow

- `.github/workflows/nightly.yml` with `schedule` trigger (e.g., 02:00 UTC).
- Steps mirror `release.yml` but:
  - Tag format `nightly-${{ github.run_id }}` or `nightly-$(date)`.
  - Mark release as pre-release and optionally delete old nightlies beyond N
    days.
  - Optionally push an update to a `nightly` branch for tracking.

### 4. Documentation & Quickstart Bundles

- Update `dist/cargo-dist.toml` `artifact` sections to include additional files:
  - `release/common/docs/QUICKSTART.md`
  - `release/common/docs/README.md`
  - Platform-specific scripts (macOS Lima, Windows WSL, Linux world provisioning).
- `cargo dist` supports `additional-artifacts` to copy these into packages.
- Maintain quickstart docs under `docs/releases/` and reference them in release
  notes and README.

### 5. Release Notes Template

- `cargo-dist` allows a `release-template.md`. Create one under `dist/` that
  includes:
  ```markdown
  {{ changelog }}
  ## Downloads
  - [Linux x86_64](#) … (links auto-filled by cargo-dist)
  ```
- Ensure `CHANGELOG.md` uses the `[version] - YYYY-MM-DD` sections expected.
- For betas, mention soak-testing expectations and link to evidence logs.

### 6. Verification Steps

- Dry run locally:
  ```bash
  cargo dist build
  cargo dist manifest --tag v0.2.0-beta.1
  ```
- Trigger workflow on a throwaway tag in a fork to confirm actions succeed.
- Validate archive contents (scripts, docs, binaries, checksums).
- Confirm GitHub release page layout matches expectations.

### 7. Rollout Checklist

1. Merge `dist/` configs and workflows into `main`.
2. Tag `v0.2.0-beta.2` (or next beta) and observe automated release.
3. Publish communication documenting nightly and beta release process.
4. Clean up previous manual release instructions to avoid confusion.

### 8. Future Enhancements

- **Code Signing**: integrate macOS notarization and Windows signing if we
  acquire certificates (store keys in GitHub Secrets).
- **Package Repos**: push `.deb`/`.rpm` to apt/yum repositories using
  `cloudsmith` or `packagecloud` actions.
- **Telemetry**: instrument installer success via optional opt-in analytics.
- **Release Dashboard**: create a docs page or script summarising latest beta
  and nightly builds with direct links.

## Ownership

- Release engineering: @spenser (initial setup), rotate to DevOps once stable.
- Documentation: keep `docs/releases/*` current with each version.
- CI maintenance: ensure runner updates or toolchain changes are addressed.

---
With this plan in place, tagging a release will mirror the polished asset table
and download experience from projects like WezTerm while retaining our beta
soak-testing workflow.
