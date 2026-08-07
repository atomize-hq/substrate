# Allowlist/evidence discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R1

## Review boundary

Fresh read-only discovery review of the two-path implementation subject bound to
`sha256:9c7de54632556f383a5a6da073d0d5900e05b6f050a7adac69067bd79f90f2d5`.
The fingerprint is the SHA-256 of the lexically sorted `shasum -a 256 -- <path>`
manifest for only `scripts/substrate/dev-install-substrate.sh` and
`tests/installers/dev_install_bash32_fd_regression.sh`. The reviewer did not mutate,
test, install, stage, or publish.

## Finding classification

The review questioned whether invoking the installer fixture could expand R1 into R2/R3 lifecycle
work. That is a contract misread, not a valid finding: R1 explicitly requires an isolated installer
regression that may operate only inside a temporary fixture prefix. This fixture invokes the copied
installer with `--no-world` and `--no-shims`, substitutes a local fake Cargo/Substrate binary, keeps
every write under its temporary root, asserts only caller-FD identity, canonical bootstrap argv and
environment, and empty fixture `TMPDIR`, then removes the fixture. It adds no lifecycle binary build
or copy assertion and executes no host installation, Keychain, XPC, Lima, pairing, native evidence,
Windows, or Linux-runtime action.

No valid P1/P2/P3/P4 finding remained for this lens. The changed-byte secret/private-data scan was
clean, and the production hunk is confined to `resolve_install_bootstrap_context`.

## Fresh full-subject closure

A different fresh read-only reviewer bound the remediated two-path subject to
`sha256:5b7e23174e01586b21ea6407d92b17fde992443779aba3505a1157417b6c22ec`
and reviewed authority/security, lifecycle/convergence, and allowlist/evidence without author
reasoning or previous conclusions. Verdict: **CLEAN** — no actionable P1, P2, P3, or P4 finding.
The closure reviewer performed no mutation, test, installation, Keychain, XPC, Lima, or publication
operation.
