---
pack_id: persist-detected-linux-distro-pkg-manager-seam-pack
pack_version: v1
pack_status: extracted
source_ref: persist-detected-linux-distro-pkg-manager.zip
execution_horizon:
  active_seam: SEAM-2
  next_seam: SEAM-3
---

# Scope Brief - Persist detected Linux distro + pkg manager

- **Goal**:
  - Persist Linux distro identity and selected package-manager metadata into `install_state.json` without changing the upstream detection contract and without converting metadata failures into installer failures.
- **Why now**:
  - The source pack identified a contract gap: successful Linux installs can currently exit without a canonical metadata file when no `host_state.group` or `host_state.linger` event occurs.
  - Future guidance consumers need one stable persisted metadata surface after successful Linux installs.
  - The source pack also found operator-doc and authority-path drift that should be carried into governance as explicit remediations rather than left implicit.
- **Primary user(s) + JTBD**:
  - Installer operators and support maintainers who need a stable post-install metadata file.
  - Future command or guidance consumers that prefer persisted host metadata when it exists and fall back to runtime detection when it does not.
  - Maintainers who need one cross-platform planning surface where Linux behavior is explicit but macOS and Windows remain parity-only for compile and test expectations.
- **In-scope**:
  - Additive `install_state.json` persistence for `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`
  - Canonical path semantics for `<effective_prefix>/install_state.json` and the operator-facing alias `$SUBSTRATE_HOME/install_state.json`
  - Shared producer behavior across hosted install, hosted `--no-world`, dev install, and dev `--no-world`
  - Hosted `--dry-run` and non-Linux no-write branches
  - Same-directory temp-file replacement and warning-only metadata failure posture
  - Smoke-harness coverage for no-event success, persisted platform fields, missing os-release degradation, and additive compatibility
  - `docs/INSTALLATION.md` wording reconciliation and checkpoint-ready evidence surfaces
  - Governance tracking for ADR feature-directory drift and related follow-ups discovered by the source pack
- **Out-of-scope**:
  - Package-manager detection precedence, manager spellings, and `pkg_manager.source` vocabulary
  - World-deps provisioning behavior
  - Uninstaller HOME-vs-prefix cleanup alignment, except as an explicit tracked follow-up
  - New macOS or Windows platform-metadata writes
  - New CLI flags, commands, environment variables, log fields, or trace fields
  - A manual validation playbook; the smoke harness and checkpoint evidence remain the authoritative validation surfaces
- **Success criteria**:
  - Successful Linux producer flows create or update one canonical `install_state.json` file even when no new group or linger event exists.
  - The persisted file remains on `schema_version = 1` and preserves existing `host_state.group`, `host_state.linger`, and unknown keys.
  - The persisted `pkg_manager.selected` and `pkg_manager.source` values are copied verbatim from the upstream detection contract.
  - Missing or unreadable os-release data degrades to literal `unknown`-sentinel persistence plus warning-only behavior rather than install failure.
  - Hosted `--dry-run` and non-Linux flows remain no-write for this feature.
  - Smoke coverage and operator wording prove the contract and do not drift from the implementation.
- **Constraints**:
  - Upstream authority for distro and package-manager detection stays with `best-effort-distro-package-manager`.
  - Linux-only behavior change; macOS and Windows stay compile and test parity participants only.
  - `install_state.json` remains the only persisted metadata file touched by this scope.
  - Same-directory temp-file replacement is required; in-place truncation is not allowed.
  - The extracted pack must keep exactly one `active` seam and one `next` seam by default.
  - Seam-local `review.md`, `seam.md`, and `slice-*.md` artifacts may exist only for the active seam and only after promotion refreshes them against landed upstream reality.
- **External systems / dependencies**:
  - Upstream detection contract: `best-effort-distro-package-manager`
  - Hosted installer script: `scripts/substrate/install-substrate.sh`
  - Dev installer script: `scripts/substrate/dev-install-substrate.sh`
  - Installer smoke harness: `tests/installers/install_state_smoke.sh`
  - Operator doc: `docs/INSTALLATION.md`
  - CI parity and quick-testing surfaces used by `CP1`
  - Adjacent ADRs and packs that share installer or documentation files
- **Known unknowns / risks**:
  - Shared-file edits across installer scripts and `docs/INSTALLATION.md` may collide with adjacent packs if sequencing is not explicit.
  - Hosted uninstaller cleanup semantics still trail the selected producer path rule and remain outside this scope.
  - Invalid JSON, unreadable file, or non-`1` schema fallback behavior is contractually defined but still requires careful seam-local review before execution.
  - The extractor is inferring seam order from the source slice order; if upstream priorities changed since the deep-researched pack was produced, seam-local revalidation must confirm the horizon.
- **Assumptions**:
  - `PDLDPM0 -> PDLDPM1 -> PDLDPM2` remains the intended execution order and is therefore used to infer `SEAM-2` as active and `SEAM-3` as next after `SEAM-1` closeout.
  - `CP1` remains the single checkpoint after the conformance/evidence seam unless downstream planning intentionally changes the horizon.
  - Source-pack decisions DR-0001 through DR-0005 remain the best available authority until superseded by newer accepted inputs.
  - `governance/seam-1-closeout.md` is now landed evidence; remaining governance closeouts stay scaffolds until their owning seams land.
