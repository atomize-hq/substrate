# R3 MAC aarch64 guest compile correction — allowlist and evidence review

- Packet: `AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION`
- Discovery subject: `sha256:69253bdee615875b83322aefe11bf163a0517aaf55fdcab21b148c1099088db2`
- Base: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` / `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`
- Review mode: fresh independent read-only discovery lens, `gpt-5.6-terra` Extra High

## Discovery finding

`P1-R3-MAC-AARCH64-BASE-REPRO`: the first regression-script version ran `expect-base-failure` against the current checkout, so the red path could not remain reproducible after the correction. This is a blocking proof defect, not a product behavior defect.

## Remediation

The script now archives the exact bound base `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` into an external temporary directory before running the locked offline aarch64 check in `expect-base-failure` mode. Its green mode remains scoped to the current candidate. Both build targets/logs and the archived source remain outside every repository checkout; no lifecycle action occurs. The remediated subject is `sha256:352cb9bb6961425847c8709f89b14adc633496aaacebe0040b0224878b9cef38`.

The report of five raw pointer arguments is not a separate blocker: the frozen compiler output contains exactly four `E0308` diagnostics because both bad `linkat` arguments are diagnosed by a single call error. ELF output is separately covered by the contract-required exact `cargo build` and `file` check, so it is not a regression-script P2.

## Closure

The different fresh closure reviewer returned **CLEAN** with zero P1/P2/P3/P4 for `sha256:352cb9bb6961425847c8709f89b14adc633496aaacebe0040b0224878b9cef38`. It independently confirmed the two-path product/test fence, exact locked/offline target command, external temporary source/build/log boundary, and the separation between the regression check and the required ELF build proof.
