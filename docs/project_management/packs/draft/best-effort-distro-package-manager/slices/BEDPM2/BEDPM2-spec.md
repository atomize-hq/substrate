# BEDPM2-spec — Hermetic installer pkg-manager detection harness

## Behavior delta (single)
- Existing: pkg-manager selection validation relies on `tests/installers/pkg_manager_container_smoke.sh`, which requires Docker/Podman and covers only a small subset of the contract.
- New: add a hermetic, container-free harness (`tests/installers/pkg_manager_detection_test.sh`) that runs the real installer in `--dry-run` mode with a controlled `PATH` and fake os-release input, asserting precedence, mapping, warnings, decision one-liner exactness, and exit-code behavior per `contract.md`.
- Why: make the Linux installer pkg-manager selection contract continuously testable in CI without containers and without mutating the host.

## Scope
- Create `tests/installers/pkg_manager_detection_test.sh` as the canonical validation entrypoint for this pack.
- The harness runs `scripts/substrate/install-substrate.sh` only in `--dry-run` mode and always passes:
  - `--no-world` and `--no-shims` to avoid world/shim side effects,
  - `--version <tag>` to avoid GitHub API calls to resolve the latest release tag,
  - `--prefix <temp>` with `HOME=<temp>` to bound all filesystem effects to a temp directory.
- The harness controls selection inputs by:
  - setting `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to fixture paths under a temp directory, and
  - constructing a deterministic `PATH` that contains only stub package-manager binaries plus a sandbox of required utility commands.
- The harness asserts decision one-liner and warning/error content on stderr and asserts exit codes for failure cases.

## Inputs (authoritative)
- Operator contract (selection pipeline, warning/error content elements, decision one-liner template, exit codes): `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Decisions DR-0001/2/3 (normalization, PATH-probe policy, os-release seam): `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- BEDPM0/BEDPM1 slice specs (selection semantics + warning/error expectations): `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- Feature intent + locked vocabularies: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Behavior (authoritative)

### Platform behavior
- On non-Linux platforms, the harness MUST print a single-line skip reason to stderr and exit `0`.
- On Linux platforms, the harness MUST execute the test matrix below and exit non-zero on the first failure.

### Harness invariants
- The harness MUST run without containers and MUST NOT invoke `docker` or `podman`.
- The harness MUST NOT require root.
- The harness MUST NOT write outside a temp work root created under `${TMPDIR:-/tmp}`.
- Every installer invocation MUST:
  - set `HOME` to a temp directory under the work root,
  - pass `--prefix` under the work root,
  - pass `--dry-run --no-world --no-shims`,
  - pass `--version <tag>` (any fixed tag string is acceptable as long as it prevents network calls to resolve “latest”),
  - redirect stdout to `/dev/null`,
  - capture stderr for assertions.

### Controlled `PATH` model
- For every installer invocation, the harness MUST set `PATH` to exactly:
  - `${PM_STUB_DIR}:${UTIL_BIN_DIR}`
- `${PM_STUB_DIR}` contains case-specific stub executables named from the supported set: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- `${UTIL_BIN_DIR}` contains the required non-package-manager utilities as either symlinks or copies:
  - `bash`, `cat`, `chmod`, `cp`, `cut`, `date`, `dirname`, `getent`, `grep`, `head`, `id`, `install`, `jq`, `ln`, `mkdir`, `mktemp`, `mv`, `rm`, `sed`, `sudo`, `tar`, `tr`, `uname`,
  - plus exactly one of `sha256sum` or `shasum`.
- The harness MUST NOT include the host PATH in `PATH` for installer invocations.

### os-release fixtures
- For cases that require os-release inputs, the harness MUST write fixture files under the work root and set `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to the fixture path.
- For “missing/unreadable os-release” cases, the harness MUST set `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to a non-existent path under the work root.

### Assertions
- For success cases (exit `0`), stderr MUST contain exactly one decision one-liner matching the exact template from `contract.md`.
- For failure cases (exit `2|3|4`), stderr MUST NOT contain the decision one-liner.
- For warning cases (mapping fallback; PATH ambiguity), stderr MUST include the required contract content elements and MUST NOT contain a `PATH=` dump.

## Acceptance criteria
- AC-BEDPM2-01: `bash tests/installers/pkg_manager_detection_test.sh` runs on Linux without requiring Docker/Podman, exits `0`, and performs no host mutation by running the installer only with `--dry-run --no-world --no-shims` and a temp-scoped `HOME`/`--prefix`.
- AC-BEDPM2-02: Flag override precedence: with a controlled PATH containing `yum`, an os-release fixture that maps to `apt-get`, and `PKG_MANAGER=dnf`, invoking the installer with `--pkg-manager yum` selects `yum` and emits exactly one decision one-liner with `(source: flag)`.
- AC-BEDPM2-03: Env override precedence: with a controlled PATH containing `pacman` and an os-release fixture that maps to `apt-get`, invoking the installer with `PKG_MANAGER=pacman` (and no `--pkg-manager`) selects `pacman` and emits exactly one decision one-liner with `(source: env)`.
- AC-BEDPM2-04: os-release mapping selection: with an os-release fixture `ID=ubuntu` and `ID_LIKE=debian` and a controlled PATH containing `apt-get`, the installer selects `apt-get` and emits a decision one-liner with `(source: os_release)`; with an os-release fixture `ID=fedora` and a controlled PATH containing `dnf`, the installer selects `dnf` and emits a decision one-liner with `(source: os_release)`.
- AC-BEDPM2-05: Mapping binary missing fallback warning: with an os-release fixture that maps to `apt-get`, a controlled PATH lacking `apt-get` but containing `dnf`, the installer emits the mapping-missing warning content elements (mentions `apt-get`, states it is not in PATH, states that PATH probe fallback is being used, includes `--pkg-manager` and `PKG_MANAGER` guidance), selects `dnf` by PATH probe, and emits a decision one-liner with `(source: path_probe)` without printing `PATH=...`.
- AC-BEDPM2-06: PATH ambiguity warning: with no overrides, a missing/unreadable os-release input, and a controlled PATH containing both `yum` and `dnf`, the installer selects `dnf` by fixed precedence, emits a warning that includes the chosen manager and the other manager name plus override guidance, and does not print `PATH=...`.
- AC-BEDPM2-07: Invalid override value: invoking the installer with `--pkg-manager not-a-manager` exits `2`, prints stderr that includes the invalid value and override guidance (`--pkg-manager`, `PKG_MANAGER`), and does not emit the decision one-liner.
- AC-BEDPM2-08: Forced manager missing + no supported manager: (a) invoking with `--pkg-manager zypper` when the controlled PATH contains no `zypper` exits `3` and prints required stderr content elements without emitting the decision one-liner; (b) invoking with no overrides, a missing/unreadable os-release input, and zero supported managers in the controlled PATH exits `4` and prints stderr that includes override guidance and includes `curl`, `tar`, and `jq` without emitting the decision one-liner.

## Out of scope
- Changing installer selection logic or contract surfaces (owned by BEDPM0/BEDPM1 and `contract.md`).
- Container-based smoke validation (`tests/installers/pkg_manager_container_smoke.sh`) beyond keeping it as an optional local check.
