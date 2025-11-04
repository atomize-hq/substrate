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
  - Require status checks from the CI workflow described in §3.
  - Require at least one approving review.
  - Disallow force pushes.
- Lock the GitHub Actions “production” environment so only the release workflow
  (running on tags from `main`) can deploy. This prevents accidental release
  from feature or testing branches.
- Document the merge discipline: no direct commits to `main`; the promotion
  workflow (see §3.4) is the only mechanism to move code from `testing`.

## 3. Workflow Inventory

### 3.1 Continuous Integration (`ci-testing.yml`)
- **Triggers**: `pull_request` targeting `testing` or `main`, `push` to those
  branches.
- **Jobs**:
  1. **Lint & Unit (matrix)**: Runs on `ubuntu-24.04`, `windows-2022`,
     `macos-14`. Steps include `cargo fmt`, `cargo clippy`, `cargo test
     --workspace`, and platform-specific smoke commands:
     - Linux: `substrate --shim-deploy --dry-run`
     - macOS: ensure Lima scripts pass `shellcheck`
     - Windows: run PowerShell installers with `-WhatIf`
  2. **Cross Targets**: Uses `houseabsolute/actions-rust-cross` to compile
     `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-musl` binaries.
     Outputs stored as build artifacts for debugging.
  3. **Documentation & Packaging**: Runs `cargo doc --no-deps`, `cargo dist
     check`, `shfmt -d scripts/**/*`, and ensures `README` quickstarts render
     (e.g., `mdbook test` if applicable).
- **Caching**: Use `actions/cache` with a key combining runner OS and the
  `hashFiles` of `Cargo.lock` to speed up builds. `target/` should be restored
  per toolchain triple to support reuse across jobs.

### 3.2 Nightly Validation (`nightly.yml`)
- **Trigger**: `schedule` (e.g., `0 2 * * *`) and manual `workflow_dispatch`.
- **Scope**: Runs the same jobs as the CI workflow plus extended integration
  suites (world agent integration tests, replay fixtures).
- **Change Detection**: The workflow should skip all heavy jobs if nothing has
  changed since the last successful nightly:
  1. Fetch the latest commit on `testing`.
  2. Download the previous nightly state artifact (`nightly-state.json`)
     containing the last processed commit SHA.
  3. If `HEAD` matches the stored SHA, exit early with a log message.
  4. After successful completion, upload a new `nightly-state.json` with the
     current SHA.
- **Reporting**: Failures open an actionable issue (via `peter-evans/create-issue-from-file`)
  summarising failing jobs and linking to logs.

### 3.3 Release Automation (`release.yml`)
- **Triggers**: `push` tags matching `v*.*.*` (including betas), manual
  `workflow_dispatch` for dry runs or hotfixes.
- **Jobs** (mirrors `cargo-dist`’s recommended plan/build/upload flow):
  1. **Plan** (Ubuntu):
     - Runs `cargo dist plan --tag ${{ github.ref_name }} --output
       dist-manifest.json`.
     - Uploads the manifest and release notes template as artifacts.
  2. **Build Matrix**:
     - Targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
       `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.
     - Each job installs the MSRV toolchain, restores caches, runs `cargo dist
       build --plan dist-manifest.json`, performs platform smoke checks, and
       uploads the produced bundles and `SHA256SUMS`.
  3. **Publish** (Ubuntu):
     - Downloads plan + artifacts.
     - Executes `cargo dist upload --plan dist-manifest.json` to draft/update
       the GitHub Release, attach assets, and render release notes from
       `CHANGELOG.md`.
     - Marks releases containing `-beta`, `-rc`, or `nightly-` as prereleases.
  4. **Crate Publishing** (optional, gated by manual approval):
     - Sequentially `cargo publish` crates in the required dependency order,
       respecting crates.io propagation delays with `sleep`.
- **Artifacts**: Ensure `dist/cargo-dist.toml` includes additional files
  (quickstarts, installer scripts, docs) under `additional-artifacts`.

### 3.4 Promotion Workflow (`promote.yml`)
- **Trigger**: `workflow_dispatch` from maintainers.
- **Steps**:
  1. Checkout `testing` and `main`.
  2. Verify CI status for the latest `testing` commit is green.
  3. Fast-forward `main` to match `testing`.
  4. Optionally create a Git tag (`vMAJOR.MINOR.PATCH[-betaN]`).
  5. Emit a summary for release notes preparation.

## 4. Tooling and Key Actions

- **`cargo-dist`** (installed as `cargo dist`): orchestrates plan/build/upload,
  and supports custom templates. Reference: <https://axodotdev.github.io/cargo-dist/>.
- **`houseabsolute/actions-rust-cross`**: builds non-native Linux targets using
  preconfigured cross toolchains.
- **`actions/github-script`**: used for nightly change detection and branch
  promotion guard rails.
- **`peter-evans/create-issue-from-file`**: opens issues when nightly runs fail.
- **`softprops/action-gh-release`** (optional): only needed if we bypass
  `cargo dist upload`.
- **Shell quality gates**: `shellcheck`, `shfmt` via `ludeeus/action-shellcheck`
  and `shfmt` Docker image.

## 5. Implementation Roadmap

1. **Installer Fixes** (current branch):
   - Address path quoting in Windows scripts.
   - Replace hard-coded `sudo` invocations with the existing `SUDO_CMD` logic.
   - Refresh documentation prerequisites.
2. **Retire Legacy CI**:
   - Remove `.github/workflows/ci-feature.yml`.
   - Add `ci-testing.yml` with matrix jobs and updated cache strategy.
3. **Nightly Workflow**:
   - Create `nightly-state` artifact management.
   - Integrate extended test suites and failure issue reporting.
4. **Adopt `cargo-dist`**:
   - Run `cargo dist init`.
   - Customise `dist/cargo-dist.toml`, release template, and artifact mapping.
   - Commit the generated configuration and lock `dist/` files.
5. **Release Workflow Overhaul**:
   - Regenerate `.github/workflows/release.yml` to the plan/build/upload model.
   - Ensure crate publishing is optional and requires manual approval.
6. **Promotion Automation**:
   - Introduce `promote.yml` workflow that fast-forwards `main`.
   - Document how maintainers trigger promotions and tagging.
7. **Branch Protections & Docs**:
   - Configure GitHub rulesets/environments to enforce the branch model.
   - Update `docs/DEVELOPMENT.md`, `docs/CONFIGURATION.md`, and this plan with
     the final workflow names and commands.
8. **Dry Run & Verification**:
   - Tag a non-production release (e.g., `v0.0.0-ci-smoke`) to test the pipeline.
   - Verify GitHub Release assets, checksums, and notes.
   - Review nightly skip logic by running the workflow twice without code
     changes and ensuring the second run exits early.

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
  - `.github/workflows/ci.yml`
  - `.github/workflows/release.yml`
  - `docs/INSTALLATION.md` / `docs/WORLD.md` (for provisioning scripts)

With this plan implemented, every change will be validated automatically, every
nightly run will execute only when required, and every Git tag will produce a
fully packaged, multi-platform release without manual intervention.
