# R3 MAC aarch64 guest compile correction — authority and security review

- Packet: `AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION`
- Discovery subject: `sha256:69253bdee615875b83322aefe11bf163a0517aaf55fdcab21b148c1099088db2`
- Base: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` / `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`
- Review mode: fresh independent read-only discovery lens, `gpt-5.6-terra` Extra High

## Discovery result

The Linux executor changes only the `extern "C"` pathname declarations for `open`, `openat`, and the two pathname arguments of `linkat`, from fixed signed bytes to `core::ffi::c_char`. Every caller continues to pass a NUL-terminated `CString::as_ptr()` without a cast. The change neither adds an input, selector, authority, lifecycle action, transport, or host-platform path nor changes the fixed no-follow and `O_TMPFILE` runtime bodies.

The authority/security reviewer initially questioned whether five pointer arguments required five `E0308` diagnostics. The frozen-base compile transcript disproved that concern: rustc emits one `E0308` for the two invalid `linkat` arguments, so the exact base has four `E0308` diagnostics as required. That observation is invalid and is not a finding.

No valid authority/security P1 or P2 was found. The discovery burst's only valid blocker is recorded by the allowlist/evidence lens.

## Closure

A different fresh read-only `gpt-5.6-terra` Extra High closure reviewer, isolated from these discovery conclusions, reviewed `sha256:352cb9bb6961425847c8709f89b14adc633496aaacebe0040b0224878b9cef38` and returned **CLEAN**: no P1/P2/P3/P4 finding. It independently confirmed that the target-correct declarations preserve the authority boundary and that the script has no secret, lifecycle, or native-action path.
