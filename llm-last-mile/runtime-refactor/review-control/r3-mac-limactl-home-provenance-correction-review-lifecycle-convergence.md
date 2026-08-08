# R3 MAC limactl HOME provenance correction — lifecycle and convergence review

- Packet: `AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION`
- Discovery reviewer: fresh independent read-only `gpt-5.6-terra` Extra High
- Discovery subject: `sha256:0b1c7f989cfd3dbb7b1864262413c6df01df10bf7affbb60c716ce08aa27bcc8`

## Discovery finding

- `P3-R3-MAC-LIMACTL-HOME-GUARD-STATIC-FALLTHROUGH`: the first fixture checked that the
  `/var/empty` guard tokens occurred before the probe, but did not require the two explicit
  fail-closed `exit 1` clauses. The current source did fail closed, but the regression would have
  accepted a later source edit that merely logged an unsafe HOME state and continued.

The fixture remediation now bounds validation to
`publish_mac_publisher_install_provenance_v1` and requires both exact failure clauses before the
probe. It also continues to execute only the extracted scrubbed probe against a temporary fake
`limactl`; it never invokes the installer, `sudo`, Lima, or a lifecycle effect. This P3 is
remediated in the closure subject and does not require an inventory entry.

## Confirmed lifecycle result

The only production call remains in the existing `IS_MAC=1` managed-copy branch. The correction
does not alter lifecycle action authority, transport, selectors, managed-copy ordering, pairing,
or any Windows, WSL, or ordinary Linux body. The root publication remains absent-or-exact and fails
before publication when the HOME control or fixed-version probe cannot complete.

## Closure

The different fresh closure reviewer bound
`sha256:e74ede1d8c3a795591c5c3b3f7567441c8add1e4dfa035979e5dbd0e31946a7b`
and returned **CLEAN**: P1/P2/P3/P4 are zero. It specifically confirmed that the now
function-bounded fixture requires both guard `exit 1` clauses and that no lifecycle or native
path is exercised by the RED/GREEN test.
