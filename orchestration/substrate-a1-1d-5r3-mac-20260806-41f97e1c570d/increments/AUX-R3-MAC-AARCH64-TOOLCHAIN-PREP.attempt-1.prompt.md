AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP — START AUTHORITY

Bind this bounded auxiliary platform-preparation continuation:

- task: `019fdf7f-c7d6-7112-9cda-a05db972da7f` on `local`
- worktree: `/Users/spensermcconnell/.codex/worktrees/fa11/substrate`
- orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- auxiliary packet: `AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP`
- dispatch nonce: `d514db6915877692c799086ce6c96daad6bcd0417d87a1bdf9ba44c2c578da53`
- authority amendment: `0037-r3-mac-aarch64-guest-build-toolchain-prep`
- source commit/tree: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` / `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`

This is platform preparation, not official evidence attempt 5. First reverify the exact clean task
worktree, remote equality, protected-checkout cleanliness, host sudo, and unchanged root-owned
`/etc/sudoers.d/substrate-r3-mac-evidence` sentinel.

Install the official `aarch64-unknown-linux-gnu` Rust standard-library target for the active Rust
1.89.0 toolchain. Install one compatible aarch64 Linux GNU link capability. Prefer an already
available compatible cross GCC; otherwise install Homebrew Zig and create a fixed external wrapper
that invokes Zig for an aarch64 Linux GNU target. Do not add or edit repository Cargo configuration,
manifests, source, tests, or tracked files. Put wrappers, logs, Cargo target output, and proof artifacts
under a task-external nonce-scoped platform-prep directory.

Prove readiness by running the exact source's locked offline
`cargo build --target aarch64-unknown-linux-gnu --bin substrate-lifecycle-linux` with an external
`CARGO_TARGET_DIR` and explicit external linker configuration. Verify the result is Linux ELF64
AArch64, record the active Rust toolchain/target/linker identities and digests, and reverify all
repository checkouts plus the sudoers sentinel are unchanged. The installed Rust target and linker
may remain because they are prerequisites for the immediate official evidence retry.

Do not run dev-install, code-signing, publisher bootstrap, pairing, lifecycle, intent, retirement,
restoration actions, or official `EVIDENCE:R3-MAC-IMP-01`. Do not mutate Lima, Keychain, launchd,
known-hosts, product prefixes, or the sudoers sentinel. Do not commit, push, archive, or begin any
successor.

Return one structured `codex.auxiliary-platform-prep-receipt.v1` with installed components, exact
commands/results, external artifact paths/digests, checkout/sentinel parity, and either `READY` or an
exact blocker. Send it to meta as your final tool action and make no tool call after the send.
