# R3 MAC limactl principal-execution correction — authority and security review

- Packet: `AUX-R3-MAC-LIMACTL-PRINCIPAL-EXECUTION-CORRECTION`
- Base: `645f786e4f30f2ce8104d0da2718fa51b427dddd` / `33cdd01a2be7bb844d70f1d6a186d969f688e39a`
- Supported model: hardened same-user Lima; no privilege-boundary claim against the exact owning user
- Discovery subject: `sha256:da3084d6ff3749fe233404b216f79b0e3ff54e6f98fe2c65f139c612c03a063c`
- First-closure subject: `sha256:d0189f6fc6e4190aca943033c055ca54fc4ba2e94a33f087832f7ba7ae36bcc1`
- Terminal subject: `sha256:de7e47a4445345653f6506a28cb51242f304a2e39faff21bede048748cbc894f`
- Reviewers: fresh independent read-only `gpt-5.6-sol` Extra High agents (closest supported runtime to the repository-requested `gpt-5.4` Extra High)

## Causal findings and remediation

Discovery found two authority/security P2s: the fresh per-spawn FD reopen did not compare its
physical identity with the retained descriptor, and a child nonzero plus post-command validation
failure lost the causal nonzero. The bounded fix exact-joined the new descriptor to the retained
`dev`/`ino`, added identical-byte inode-substitution denial, and introduced an explicit
preserving-first outcome join.

The first closure then found the same dual-failure ordering gap in the retained R6 streaming
guard. The second fix added an R6-specific exit-status/post-validation join while keeping the
guard reaped and empty before validation. Its regression proves a nonzero status is rendered
before the appended validation failure.

## Terminal security result

The terminal reviewer independently recomputed the terminal fingerprint and returned **CLEAN**:
P0/P1/P2/P3 are all zero. It confirmed exact account/UID/primary-GID/canonical-home joins,
UID/GID 0 denial, `setgroups -> setgid -> setuid`, exact scrubbed environment, fixed measured
`/usr/local/bin/limactl`, role-bound read-only FD3/FD4, root ownership/mode/nlink/size/digest and
physical-identity joins, fresh retry offsets, unrelated-FD CLOEXEC, timeout/nonzero/disconnect
cleanup, preserving-first behavior, and intended-user Lima-state ownership.

The no-follow pre/post checks for `_config/default.yaml` and `override.yaml` are recorded only as
deterministic drift detection for the supported same-user path. They do not claim a race-free
security boundary against that owning user. Group/other-writable home, Lima-control, or `_config`
directories fail closed so another local principal cannot supply that drift.

Existing XPC, designated-requirement, signature/SPKI, Keychain, protected-state/CAS, FD3,
observation, receipt, retry, timeout, and retirement controls were not weakened.
