# Lifecycle/convergence discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R1

## Review boundary

Fresh read-only discovery review of the two-path implementation subject bound to
`sha256:9c7de54632556f383a5a6da073d0d5900e05b6f050a7adac69067bd79f90f2d5`.
The fingerprint is the SHA-256 of the lexically sorted `shasum -a 256 -- <path>`
manifest for only `scripts/substrate/dev-install-substrate.sh` and
`tests/installers/dev_install_bash32_fd_regression.sh`. The reviewer did not mutate,
test, install, stage, or publish.

## Findings classified against the R1 contract

| ID | Priority | Disposition | Evidence and consolidated correction |
|---|---|---|---|
| `P2-R1-ISO-001` | P2 | duplicate of authority/security finding; remediated once | Ambient canonical bootstrap state could prevent the isolated fixture from reaching its FD assertions. The consolidated `env -i` fixture invocation removes that inherited state. |
| `P3-R1-LIFECYCLE-001` | P3 | fixed inside the authorized test fence | The resolver intentionally rejects root principals, so a root invocation previously failed before exercising the regression. The test now explicitly skips on UID 0 rather than misreporting the policy rejection as an FD failure. |

The descriptor read group has one process-substitution input, succeeds only after all six
NUL-delimited fields are read, and treats a short/error output as the existing fatal path.
