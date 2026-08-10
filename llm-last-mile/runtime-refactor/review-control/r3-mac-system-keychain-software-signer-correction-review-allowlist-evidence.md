# R3 macOS System-Keychain software signer correction — allowlist and evidence review

- Packet: `AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION`
- Discovery subject: `sha256:41befc0353221867731b20e078ce87d202f670752fbefc97d359e44ee2372625`
- Closure subject: `sha256:66bfcff9a7360a574ef96e05871ef9ed4827ce7ff936dfc280a3f565e5f3fe4f`
- Subject form: SHA-256 of the full-line `LC_ALL=C` sorted native
  `<content SHA-256><two spaces><repository-relative path><LF>` records; review metadata excluded

## Discovery and proof

The implementation subject contained exactly the four authorized runtime-refactor control docs,
`src/bin/substrate-lifecycle-macos.rs`, and `tests/mac/lifecycle_r3.sh`. No installer, common
contract, schema, Linux, Windows/WSL, Cargo, Intel/T2 support, product evidence, or closeout path
changed. The changed-byte secret and MAC-only containment scans passed.

The bound-base static red check failed first on private `kSecUseSystemKeychain`; the causal
retirement regression then failed before remediation on the missing shared lock. The remediated
static suite, Bash syntax, cargo format, diff check, full lifecycle binary tests (12), common tests,
managed lifecycle tests (6), and locked/offline `aarch64-apple-darwin` check pass. Repository-wide
clippy with `-D warnings` stops in two dead-code diagnostics in unchanged
`crates/world/src/session.rs`, whose bytes equal the bound base; this is an inherited ambient
failure, not a candidate regression.

The nonce-scoped product-independent proof receipt is
`/Users/spensermcconnell/.codex/evidence/system-keychain-software-signer-correction/c708627e-system-keychain-20260810T151614Z-7c81ad4e/receipt.md`
with SHA-256 `fef2ca75b601cf828653ecef584c4fbdb926386ba5f4e6b678dcff6ea03398ec`.
Its artifact manifest SHA-256 is
`1bc70bf1436427a07e577cdbcbc8a1430852c562d400eaffcba8d6efcc1a0032`; every listed artifact
revalidates, including exact deletion/final absence and launchd/plist/process restoration.

## Closure bounded stop

The closure fingerprint and external hashes revalidated. The runtime remediation is clean, but
`R3-MAC-SKC-P2-002` remains in the active, base-unchanged, out-of-fence
`llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md`: lines 31-32 and 131-136 require
non-exportable macOS signing. The existing regression scans only in-fence control docs. The
smallest required authority expansion is that one specification and its stale-claim regression;
therefore this cycle stops `BLOCKED_SCOPE_EXPANSION` without commit or push.

## Continuation-01 allowlist, evidence, and clean closure

The authorized recovery specification was added to the subject and only its two stale
non-exportability requirements were corrected. The lifecycle regression now scans that active
specification. The complete seven-path implementation subject is
`sha256:a52b409b7cb151353c167709d49e16a528d61acf1213b0e3586646451dbaad42`.

Affected static, Bash syntax, format, diff, exact 11-entry inventory, changed-byte secret,
MAC-only containment, and external-proof integrity gates pass; runtime source bytes remain
unchanged from the previously proved subject. A fresh independent supplemental causal reviewer and
a different fresh closure reviewer both returned CLEAN with zero P1/P2 findings. No product
evidence or closeout action ran.

GitNexus `detect_changes` was invoked before commit with the exact worktree, but the server did not
have `/Users/spensermcconnell/.codex/worktrees/ad9d/substrate` or repository name
`substrate-current` indexed; it returned the available-repository list instead. No protected index
was rebuilt. The required exact manual fallback mapped every Rust hunk to
`bootstrap_mac_publisher_from_authorized_fd3_v1`,
`mac_open_system_keychain_p256_spki_der_v1`,
`retire_mac_system_keychain_software_signer_v1`,
`validate_system_keychain_protected_state_key_binding_v1`,
`compare_and_swap_mac_publisher_protected_state_v1`,
`relay_mac_xpc_publisher_request_v1`, and the macOS-gated
`mac_system_keychain_ffi_v1` Security.framework boundary. Exact source search accounted for every
SPKI/protected-state caller and all `SecKeychain*`, `SecItem*`, and `SecKey*` declarations/calls;
the only non-macOS branches are explicit fail-closed `bail!` paths. The exact inventory proves no
Linux, Windows/WSL, installer, common-contract, Cargo, or product-evidence path changed.
