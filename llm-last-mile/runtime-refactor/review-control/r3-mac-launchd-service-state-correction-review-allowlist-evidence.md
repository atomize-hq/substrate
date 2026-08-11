# AUX-R3 macOS launchd service-state correction: allowlist/evidence review

Packet: `AUX-R3-MAC-LAUNCHD-SERVICE-STATE-CORRECTION`

## Independent discovery review

- Reviewer: fresh read-only `gpt-5.4`, reasoning effort `xhigh`
- Subject fingerprint: `sha256:3ed4e3c1ff6cd03af99d2e11230bd775bb52ebec594ce5b6d27f1445f12b7764`
- Result: `FINDINGS` (`P1=0`, `P2=2`)

| Finding | Priority | Disposition |
| --- | --- | --- |
| `AUX-R3-MAC-LAUNCHD-004` impossible phase/revision receipt chains were admissible | P2 | Remediated with exact phase/revision/cursor/predecessor validation and positive/negative canonical-record tests. |
| `AUX-R3-MAC-LAUNCHD-005` the native proof facts were not present in a tracked artifact | P2 | Remediated by this tracked proof record, bound to the final non-review subject fingerprint and the disposable proof summary digest below. |

## Bounded Apple Silicon native proof

- Host: local Apple Silicon (`arm64`) macOS.
- Non-review subject fingerprint: `sha256:b9dee2fe0beb155838a4a9267ed914090629c6aafa73d675f67cc863c182e5ce`.
- Canonical ephemeral summary SHA-256: `211e6c308673b9aceedf34c0cc8b3f05fc40f387eb7904da1f7aded1396728fb`.
- Baseline: fixed helper, plist, provenance, process, and system-domain service absent; `launchctl print system/com.substrate.lifecycle.publisher.v1` exited 113; Lima list empty; exact task Keychain identities absent.
- Publication: the canonical installer ran with `--no-world --no-shims`, published and verified the fixed helper/plist/provenance plus the exact four AArch64 retained artifacts, and performed no raw shell launchd mutation.
- Stage-1: direct FD3 bootstrap completed after the literal controlling-terminal confirmation. Scope `019ff1c0-3455-7d91-8b23-32e5a4f914bc`; signed authorization digest `96e79a815b447ce922b8275f96c1460000a446add449a04f2fcc074d8aecd1c4`.
- Registration: closed install returned protected record `535fa89a4315f6b97dc65653f8f5925590cf4447d68853cd9348ba845b3384da`; definite system-domain registration followed exact `bootstrap`; no `kickstart` occurred.
- Demand activation/admission: the first real mapped-lifecycle XPC call demand-launched PID 53812 and reached the expected stale protected-state rejection rather than any connection-invalid/interrupted, peer-requirement, or malformed-frame classification. A separately ad-hoc re-signed negative control (`CDHash=0ed48ba216c3b3627ec36a88569b07b4aeb17558`) was rejected by launchd/XPC with exact OSStatus `-67050`; the positive service remained healthy.
- Idempotency: a second closed install returned the identical installed record digest.
- Re-demand: normal TERM of PID 53812 was followed by demand activation as PID 66219. Forced KILL of PID 66219 was followed by demand activation as PID 67568.
- Retirement: closed retirement returned protected record `0f7795b85a9c77f913e796f3c3d438fc220a1f4b29a700688c28c0e5b6d00296`; fixed bootout preceded deletion; final launchd observation exited 113 and all three fixed files were absent.
- Cleanup: the exact six task-owned generic-password items, exact P-256 signing key (through the source's exact retirement function under a disposable native-only test invocation), exact six CAS lock files, disposable prefix (including the negative client), and helper process were removed. The temporary cleanup test source bytes were restored exactly before fingerprint revalidation. Lima remained empty. No new crash report was produced. One pre-existing 2026-08-10 control crash report was preserved unchanged.

The proof did not create Lima, run pairing, run product smoke, dispatch official evidence, retire unrelated state, or run MAC-CLOSEOUT.

## Allowlist and frozen semantics

The non-review changed paths are confined to the declared production/test/contract fence. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`, the launchd plist, Linux/Windows/world/Lima/pairing sources, XPC service/schema/reply classifier, audit/code-signing admission policy, and fallback behavior are unchanged. The executor-side Darwin FFI correction only fixes the native audit-token ABI and Blocks/XPC lifetime required for the unchanged admission path to run without a crash.

## Closure

A different fresh read-only `gpt-5.4` reviewer at reasoning effort `xhigh` reviewed the final
non-review subject fingerprint
`sha256:b9dee2fe0beb155838a4a9267ed914090629c6aafa73d675f67cc863c182e5ce`
and the remediated native proof, then returned terminal `CLEAN` with `P1=0`, `P2=0`, and no P3/P4
findings.
