# R5 allowlist and evidence review — final supplemental causal stop

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5`  
Base: `05d655fa1458a276f179d12057cbda772cc51eb6`  
Reviewed implementation/test fingerprint: `sha256:d1f4e28e5bd3ca3ccb66e981397698e527403a647ce2103021debdae5a67007d`

## Subject containment

The reviewed changed source/test subject is exactly the authorized fifteen paths:

1. `crates/common/src/lib.rs`
2. `crates/common/src/managed_artifact.rs`
3. `crates/shell/src/execution/managed_lifecycle.rs`
4. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`
5. `crates/shell/src/execution/mod.rs`
6. `crates/shell/src/lib.rs`
7. `crates/shell/tests/managed_lifecycle_v1.rs`
8. `scripts/mac/lima-lifecycle.sh`
9. `scripts/mac/lima-stop.sh`
10. `scripts/mac/lima-warm.sh`
11. `scripts/substrate/dev-install-substrate.sh`
12. `src/bin/substrate-lifecycle-control.rs`
13. `src/bin/substrate-lifecycle-macos.rs`
14. `tests/installers/dev_install_bash32_fd_regression.sh`
15. `tests/mac/lifecycle_r3.sh`

Review metadata is limited to this report, the authority/security report, the lifecycle/convergence
report, and the review-cycle record. No `06` update is needed because the final reviewer found no
P3/P4.

Manual fallback was used for change detection and caller/symbol review because GitNexus has no
index for this assigned worktree: `substrate-current` is absent and its available indexes point to
other worktrees. No `npx gitnexus analyze` was run. The manual diff/path fence, shell caller
analysis, Rust decoder/XPC/receipt/descriptor review, and static MAC-only containment check found
no changed Linux/Windows target path. Changed-byte credential-pattern scan and `git diff --check`
passed.

## Deterministic proof executed before final review

- `cargo test -p substrate-common --lib --no-fail-fast` — 80 passed.
- `cargo test --manifest-path crates/shell/Cargo.toml --test managed_lifecycle_v1 --no-fail-fast`
  — 4 passed.
- `bash tests/installers/dev_install_bash32_fd_regression.sh` — passed.
- `bash tests/mac/lifecycle_r3.sh` — passed.
- `cargo check --bin substrate-lifecycle-macos --bin substrate-lifecycle-control` — passed.
- `cargo check --target x86_64-apple-darwin --bin substrate-lifecycle-macos --bin substrate-lifecycle-control`
  — passed.
- `cargo fmt --all -- --check` and `git diff --check` — passed.

Those deterministic checks do not override the final independent P1 findings. No native service,
Lima, XPC, Keychain, socket, known-host, install, code-sign, or evidence action was run.

## Verdict

**NOT CLEAN — bounded stop / budget exhausted.** The allowlist is contained, but contained code
still has the three authoritative post-PM P1 defects recorded by the final reviewer.
