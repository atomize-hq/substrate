# R3 MAC limactl HOME provenance correction — authority and security review

- Packet: `AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION`
- Base: `1126b907df6e38043e9071da222f6c6a377b341c` / `3a29fa323a8b9099d894c11d98cc085ecf4807c5`
- Discovery reviewer: fresh independent read-only `gpt-5.6-terra` Extra High
- Discovery subject: `sha256:0b1c7f989cfd3dbb7b1864262413c6df01df10bf7affbb60c716ce08aa27bcc8`

## Discovery finding

- `P1-R3-MAC-LIMACTL-SUBJECT-FINGERPRINT-UNBOUND`: the initially supplied
  `sha256:d27e1ad3403881f57c11602a87256297bf2813d5ad202b45ebc5ccf0d3491c4a`
  had no declared canonical serialization and did not bind the reviewed two-file bytes.
  The remediation declares the canonical form: lexically sort LF-terminated lines of
  `<SHA-256 content digest><two spaces><repository-relative path>` for the product/test
  subject, then SHA-256 those manifest bytes. The discovery manifest binds the installer
  digest `09b4a559ac8a9decebfce06a8ff73c2e6c7fcc9dcc9dc8054346ae232ec67109`
  and the initial fixture digest
  `e4a977196cd825e45e43a214ab7ee7c066ceaa5335a5584806ff425ef57fd301`.

## Confirmed security result

The source retains the absolute no-follow Lima image lookup and root-owned immutable path walk,
fixed privileged PATH, `env -i`, and all image identity, CDHash, requirement, and provenance
joins. Before the version probe it now requires `/var/empty` to be a non-symlink directory
owned by UID 0 without group/other write authority. The probe supplies only
`PATH=/usr/bin:/bin:/usr/sbin:/sbin` and `HOME=/var/empty`; it cannot forward a caller or
ambient HOME. Any validation or probe failure remains nonzero under `sh -ceu` and the caller's
`|| fatal`.

The read-only host reproduction observed Lima 2.1.1 exit 2 with
`panic: $HOME is not defined` under the old scrubbed environment, then exit 0 with
`HOME=/var/empty`. No installer, privileged publisher, Lima lifecycle, Keychain, launchd,
or evidence operation was run.

## Closure

A different fresh independent read-only `gpt-5.6-terra` Extra High reviewer bound
`sha256:e74ede1d8c3a795591c5c3b3f7567441c8add1e4dfa035979e5dbd0e31946a7b`
using the declared SHA-256 content-manifest form and found **CLEAN**: P1/P2/P3/P4 are all zero.
It independently confirmed that the fixed image and fail-closed provenance joins remain unchanged
and that the only added probe environment value is the root-controlled non-user HOME.
