# Authority/security discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R1

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
| `P2-R1-AUTH-001` | P2 | remediated | The fake binary initially proved only that FD 9 was open; an installer child could have replaced it with another open descriptor while the parent retained its original sentinel. The fixture now compares `os.fstat(9)` with `os.stat` of the caller sentinel before recording success, without reading or advancing that descriptor. |
| `P2-R1-ISO-001` | P2 | remediated | The initial fixture inherited ambient `SUBSTRATE_*` projections, so a developer shell with a different selected home could fail before the intended regression path. The installer now runs through `env -i` with only the temporary fixture inputs and fake-tool variables. |

The selected subject has no caller-supplied shell interpolation in the Python bootstrap input,
does not log the carrier, and confines the fixture to its temporary root.
