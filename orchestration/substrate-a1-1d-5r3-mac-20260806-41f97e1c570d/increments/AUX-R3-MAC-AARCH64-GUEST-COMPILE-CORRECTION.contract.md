# AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION

## Objective

Publish the smallest source correction that makes the exact R3 MAC aarch64 Lima guest executor
compile under `aarch64-unknown-linux-gnu`. The independently verified failure is four Rust `E0308`
errors because `CString::as_ptr()` is `*const u8` for this target while the local Linux FFI
declarations hard-code `*const i8`. Use the target-correct C character type; do not change runtime
behavior, authority, selectors, transport, lifecycle, pairing, or platform scope.

## Exact authority

- Publication: remote normal fast-forward to
  `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`.
- Expected base: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` /
  `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`.
- Required ancestor: `270f6e55e1a94b7e2f9b2667e605980d2e50579c`.
- Amendment: `0038-r3-mac-aarch64-guest-compile-correction`.
- User override: every implementation/review subagent must use `gpt-5.6-terra` at Extra High.
- Next increment: `EVIDENCE:R3-MAC-IMP-01`; do not dispatch or run it.

## Exact path and symbol fence

Allowed implementation/test paths:

1. `src/bin/substrate-lifecycle-linux.rs`
2. `tests/mac/aarch64_guest_compile_r3.sh`

Allowed review metadata paths:

3. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-authority-security.md`
4. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-lifecycle-convergence.md`
5. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-allowlist-evidence.md`
6. `llm-last-mile/runtime-refactor/review-control/r3-mac-aarch64-guest-compile-correction-review-cycle-record.json`

Within the Rust binary, edit only the Linux `extern "C"` path pointer types for `open`, `openat`,
and `linkat`, plus imports strictly necessary to use the target-correct C character type. Do not
edit the four callers unless fresh proof shows the declaration correction alone is insufficient.
No other product symbol or path is authorized.

## Pre-edit analysis

Attempt repository-defined GitNexus impact analysis before editing each existing symbol. Do not run
`npx gitnexus analyze` when it would rewrite tracked `AGENTS.md` or `CLAUDE.md`; if the exact task
worktree index is unavailable or stale, record the degraded result and perform complete manual
caller/callee, FFI ABI, cfg, and execution-path analysis. Warn and stop on any genuine HIGH/CRITICAL
impact outside this exact guest-executor closure.

## Implementation and deterministic proof

Use TDD: first add/run the focused regression so the exact base reproduces the aarch64 Linux target
compile failure, then make the smallest correction and prove green. The installed platform inputs
are read-only prerequisites:

- Rust toolchain `1.89.0-aarch64-apple-darwin` with target `aarch64-unknown-linux-gnu`;
- Zig `/opt/homebrew/opt/zig/bin/zig`;
- external linker wrapper
  `/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP/d514db6915877692c799086ce6c96daad6bcd0417d87a1bdf9ba44c2c578da53/linker/aarch64-linux-gnu-zig-cc`.

Required checks:

1. `cargo check --locked --offline --target aarch64-unknown-linux-gnu --bin substrate-lifecycle-linux`.
2. Exact locked offline `cargo build` for that target/binary using an external `CARGO_TARGET_DIR`
   and the explicit external linker wrapper.
3. Verify the built output is Linux ELF64 AArch64 and record its digest.
4. Run the focused existing R6 guest-entrypoint tests that are valid on this macOS host.
5. Run `cargo fmt --all -- --check`, the new regression script, and `git diff --check`.
6. Prove Linux selected runtime bodies other than the ABI-correct declaration are byte-identical;
   prove all Windows/WSL and macOS host product paths are byte-identical to base.
7. Enforce the exact six-path allowlist and scan changed bytes for secrets.

The external build output and logs must stay outside every repository checkout. Do not edit Cargo
manifests, lockfiles, repository Cargo configuration, or generated context files. Do not install,
run native lifecycle actions, or mutate the protected checkout.

## Causal cascading review

Freeze the product/test subject fingerprint and run the repository-defined causal review sequence:
one fresh discovery review or same-subject three-lens burst, then one different fresh closure review.
At most two supplemental cycles are allowed only for P1/P2 findings directly caused or unmasked by
the immediately preceding remediation under unchanged authority. Remediate valid P1/P2 only.
Record P3/P4 in the repository inventory only if required by current repo rules; they do not create
remediation cycles. Validate the review record with `validate_review_cycle.py`. Terminal CLEAN and
zero open P1/P2 are mandatory.

Review must explicitly confirm:

- the FFI types match the target's `c_char` ABI on aarch64 Linux;
- no runtime action, authority, input, selector, Linux-host behavior, Windows behavior, or macOS-host
  behavior changed;
- the focused test would have caught the exact four base failures;
- the exact path/symbol fence and external-build boundary hold.

## Publication and receipt

Run required change detection; use exact manual fallback if GitNexus cannot bind this task worktree.
Reverify the live remote still equals the expected base. Publish exactly one conventional commit by
normal fast-forward push. Verify remote equality, 0/0 divergence, clean task worktree/index, exact
changed paths, final subject fingerprint, and validated review record digest.

Return one `codex.top-level-task-receipt.v1` with status `LANDED_CLEAN`, exact task identity, base,
landed commit/tree/ref, changed files, subject fingerprint, review disposition, checks, change
detection, containment proof, and `next_increment: EVIDENCE:R3-MAC-IMP-01`. Do not run evidence,
MAC-CLOSEOUT, or any successor.
