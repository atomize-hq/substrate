# R4 allowlist and evidence review

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R4`
Subject base: `b12cb6c6e2dc326b4150291ef30676171df34f58`

## Exact subject

The frozen implementation subject has exactly these six production/test paths:

- `Cargo.toml`
- `Cargo.lock`
- `crates/common/src/lib.rs`
- `crates/common/src/managed_artifact.rs`
- `src/bin/substrate-lifecycle-macos.rs`
- `tests/mac/lifecycle_r3.sh`

The manifest adds only root `libc = "0.2"` and workspace `sha2`; the lockfile adds only the
corresponding `substrate` dependency edges. No dependency version, feature, checksum, source, or
other package edge changes. The implementation adds no `lima-stdio-v1` channel/frame/phase type,
generic issue wrapper, caller-selected operation, mapped action, guest command, endpoint, or
unrelated FFI family.

## Causal review lineage

Discovery at
`sha256:5dfc6dffba611f8a4d445880aa4247f0e47eefaa1aa240709a5057781badf390` identified the
control-image allowlist deficiency that was remediated by the protected identifier/CDHash
requirement and direct audited-message code validation. Closure-1 at
`sha256:dba6d99703255dfca2dbc366c4832ece91df85ea7c6dfca91c69285666855b39` found the remaining
PID-path TOCTOU and led to the direct `SecCodeCreateWithXPCMessage` remediation.

Supplemental-causal-1 at
`sha256:248074b7d76c0525a76eca98b958c04189d9b6d248f489ad7b56d02733ca6e44` found
`P2-R4-SYMBOL-FENCE-MAC-SECURITY-KEY-FFI`: the candidate had touched the unrelated public
`mac_security_key_ffi_v1` and removed its standalone declarations. That baseline function and
its Security/CoreFoundation declarations were restored exactly. The restoration exposed the
native link error `ld: library 'xpc' not found`; the final immediate XPC FFI linkage is therefore
`System`, which is the direct required linkage for the existing XPC calls and passes native arm64
and x86_64 compilation. This direct FFI repair stayed inside the named R4 XPC closure.

Final supplemental closure at
`sha256:efd2f5b317b53cb79e4738f46f2633d0403e6d182f11339cc36ab708690ecc52` is **CLEAN for
P1/P2**. It recorded only nonblocking `P3-R4-CLOSURE-RESPONSE-METADATA`, already retained in the
authority review; no global finding inventory entry is required.

## Proof and hygiene

`cargo fmt --all -- --check`, `git diff --check`, source-shape negatives, focused common and
inline macOS tests, both requested macOS target compile checks, exact path/public-symbol/manifest/
lockfile closure, and changed-byte secret scan passed. Warnings in broader workspace dependencies
were pre-existing and outside this subject. Neither native lifecycle execution nor Keychain/XPC/
Lima mutation was invoked.
