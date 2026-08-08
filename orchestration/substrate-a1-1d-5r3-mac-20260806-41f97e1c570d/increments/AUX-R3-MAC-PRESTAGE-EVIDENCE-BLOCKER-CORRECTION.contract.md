# AUX-R3-MAC-PRESTAGE-EVIDENCE-BLOCKER-CORRECTION

## Classification

This is one bounded MAC-only source correction discovered by official
`EVIDENCE:R3-MAC-IMP-01` attempt 9. It is not R7, does not reopen R1-R6, and does not authorize a
new pairing, transport, endpoint, selector, principal, scheduler, or lifecycle architecture.

Publication mode is `remote` to
`origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap` from exact base
`a06b3fd26ceee4ece7e5bd1481981755eafcedb5` / tree
`f4c396d0e28a43405a9a2b1d49a450d6df545c84`.

## Objective

Make the canonical macOS installer executable through the already-landed R5/R6 direct-bootstrap,
Stage-1, and closed post-PM path by producing and safely retaining the exact existing AArch64
Linux artifacts before Stage-1. Correct the unsupported `mac.lima.guest-binary(world)` role rather
than inventing a binary. Preserve all authority and platform boundaries.

## Required behavior

1. Reverify exact Cargo metadata and all consumers before changing the artifact set. The closed
   retained set must be exactly:
   - `mac.lima.publisher-executor` from existing binary `substrate-lifecycle-linux`;
   - `mac.lima.guest-binary(substrate-world-service)` from existing binary `world-service`;
   - `mac.lima.guest-binary(substrate-gateway)` from existing binary `substrate-gateway`;
   - `mac.lima.guest-binary(substrate)` from existing binary `substrate`.
2. Remove only `mac.lima.guest-binary(world)` and `/usr/local/bin/world` from the R3 MAC managed
   artifact role table, receipt planning, executor admission, tests, and current control-pack row.
   Do not change the `world` crate, the `substrate world` CLI, or unrelated world behavior.
3. The canonical macOS installer must own one fixed locked/offline
   `aarch64-unknown-linux-gnu` build of those four existing binaries. It must use the already
   authorized exact-source build-root semantics, the prepared fixed Zig linker, and no caller-
   supplied artifact role/path selector.
4. Before direct bootstrap, publish the four fixed outputs into the recognized
   `${PREFIX}/bin/linux` retained area as regular, no-follow, mode-0755 files with exact role,
   source commit/tree/ref, Cargo.lock, toolchain/build command, physical identity, and SHA-256
   joins. Publication must be all-or-nothing and retry-convergent. Unknown or mismatched prior
   files fail closed; no post-failure partial bundle may be treated as authoritative.
5. Use the already-landed R5/R6 manifest and install-provenance model. A schema adjustment is
   allowed only when exact impact analysis proves it is the minimum way to bind the four fixed
   artifacts. Do not add a new authority family or generic bundle ingestion API.
6. After retained artifact and root install provenance publication, the installer invokes the
   already-landed no-argument/direct publisher-bootstrap path. Pass its signed Stage-1 result only
   through the existing fixed control/wrapper boundary. Do not expose raw authorization in logs,
   caller-selected files, ambient environment, or a new reusable carrier.
7. `lima-warm` and `lima-lifecycle` may be changed only to consume that existing fixed result and
   execute the closed signed `stage_one_create` plus the already-returned closed
   `post_pm_requests_v1` in canonical response order. Do not add a request selector, action, role,
   generic scheduler, alternate transport, or pairing behavior.
8. If the landed direct-bootstrap/Stage-1 interface cannot satisfy item 6 without a new Keychain
   locator, transport, principal, or authority type, stop with `BLOCKED_CONTRADICTION`. That is a
   legitimate evidence blocker; do not implement the expansive planning proposal by implication.

## Authorized implementation fence

Production/control paths:

- `scripts/substrate/dev-install-substrate.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-lifecycle.sh`
- `src/bin/substrate-lifecycle-control.rs`
- `src/bin/substrate-lifecycle-macos.rs`
- `crates/common/src/managed_artifact.rs`
- `crates/common/src/lib.rs`
- `crates/shell/src/execution/managed_lifecycle.rs`
- `crates/shell/src/execution/managed_lifecycle/macos_client.rs`
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`

Test paths:

- `tests/mac/prestage_artifact_route_r3.sh`
- `tests/mac/aarch64_guest_compile_r3.sh`
- `tests/mac/dev_install_compile_surface_r3.sh`
- `tests/mac/lifecycle_r3.sh`
- `tests/installers/dev_install_bash32_fd_regression.sh`
- `crates/shell/tests/managed_lifecycle_v1.rs`

Review metadata paths:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-prestage-evidence-blocker-correction-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-prestage-evidence-blocker-correction-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-prestage-evidence-blocker-correction-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-prestage-evidence-blocker-correction-review-cycle-record.json`

The allowlist is a maximum fence, not a requirement to edit every path. Any additional product,
test, manifest, lockfile, or control-pack path requires a blocked receipt and explicit correction
authority.

## Frozen surfaces

- all Cargo manifests and lockfiles;
- all Windows, WSL, and ordinary Linux-host source and behavior;
- all R1-R6 review records and completed finding dispositions;
- pairing semantics and direct-TTY proof;
- Keychain account/state shape except an independently proven in-fence use of the already-landed
  R5 record;
- XPC endpoint, designated requirement, control principal, fixed Lima instance, IH/PM selection,
  and evidence validator semantics;
- any new binary target, renamed/aliased binary, generic artifact input, generic action broker, or
  manual prefix staging route.

## TDD and deterministic proof

Before production edits, add focused RED proof on the exact base for:

- absence of a canonical four-artifact pre-Stage-1 build/stage route;
- the unsupported `mac.lima.guest-binary(world)` role;
- installer failure to connect direct bootstrap to the fixed Stage-1 wrapper;
- partial publication and retry/unknown-prior-state behavior;
- any raw-authorization leakage or caller-selected role/path input.

GREEN proof must include:

- exact Cargo metadata proving only the four named binaries;
- locked/offline AArch64 check and build of all four outputs with the prepared Zig wrapper and an
  external target directory, followed by ELF64 AArch64, mode, identity, and SHA-256 inspection;
- focused prestage route, Bash 3.2 installer, compile-surface, lifecycle, and managed-lifecycle
  tests;
- `cargo fmt --all -- --check` and `git diff --check`;
- exact path/symbol fence, changed-byte credential scan, Windows/WSL byte identity, and manual
  ordinary-Linux-host containment if GitNexus remains unavailable.

## Impact, review, and publication

Run required GitNexus upstream impact before every existing symbol edit. If the exact worktree is
not indexed, record the degraded result and perform exact manual caller/callee, cfg, manifest,
receipt, retry, and platform analysis. Stop and warn on HIGH or CRITICAL impact.

Freeze the product/test subject and run fresh independent authority/security,
lifecycle/convergence, and allowlist/evidence discovery reviews, followed by a different fresh
closure reviewer. Use repository-required reviewer model/reasoning. Validate the causal review
record and require no open P1/P2. Record valid P3/P4 without expanding this correction.

Immediately before publication, require the live remote to equal the exact base. Publish exactly
one normal fast-forward commit, verify the live remote and ancestry, and finish with a clean
worktree/index. Return `LANDED_CLEAN` with next increment `EVIDENCE:R3-MAC-IMP-01`. On any real
authority, architecture, platform, or review blocker, preserve the worktree and return the exact
blocked receipt. Do not dispatch evidence or closeout.
