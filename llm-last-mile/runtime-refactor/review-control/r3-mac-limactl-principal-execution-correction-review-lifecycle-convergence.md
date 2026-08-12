# R3 MAC limactl principal-execution correction — lifecycle and convergence review

- Packet: `AUX-R3-MAC-LIMACTL-PRINCIPAL-EXECUTION-CORRECTION`
- Discovery: `sha256:da3084d6ff3749fe233404b216f79b0e3ff54e6f98fe2c65f139c612c03a063c` — findings
- First closure: `sha256:d0189f6fc6e4190aca943033c055ca54fc4ba2e94a33f087832f7ba7ae36bcc1` — one R6 finding
- Terminal closure: `sha256:de7e47a4445345653f6506a28cb51242f304a2e39faff21bede048748cbc894f` — CLEAN

## Discovery convergence findings

The three independent discovery lenses accepted six in-scope P2s:

1. fresh child input did not exact-join the retained physical identity;
2. captured-child nonzero plus post-validation failure did not preserve the first failure;
3. home/Lima-control/overlay directories did not reject group/other write authority;
4. post-command overlay rejection lacked behavioral proof;
5. the scrubbed-environment proof did not reject unexpected keys; and
6. inherited-input cleanup coverage did not exercise nonzero, timeout, and disconnect paths.

The first fix round resolved all six with exact identity comparison, explicit outcome joining,
directory-mode checks, child-created `default.yaml` and `override.yaml` rejection, full environment
enumeration, and inherited FD3/FD4 failure-path tests. It also proves identical-byte replacement
denial, read-to-EOF retry behavior, and no unrelated descriptor inheritance.

The first closure found one remaining P2: R6 `finish` reaped the child but could mask a nonzero
status with a later state-validation error. The second fix added the R6-specific preserving-first
join and focused regression without changing the stream protocol or caller behavior.

## Terminal convergence result

A fresh terminal reviewer traced Stage-1, post-PM mutation/observation, and R6 streaming callsites
to the one principal runner and returned **CLEAN**, with P0/P1/P2/P3 all zero. Start, list, stop,
delete, shell, and copy remain closed executor-owned plans. Captured and streaming children are
reaped on success, nonzero, timeout, poll/disconnect, and retry boundaries; post-command identity,
ownership, and owner-overlay checks run before an observation is accepted.

GitNexus was degraded for this file: its analyzer skipped the oversized
`src/bin/substrate-lifecycle-macos.rs`, impact queries returned `Target not found` with risk
`UNKNOWN`, and FTS remained missing after forced analysis. Manual caller/callee, executable,
credential, FD, privilege, timeout, observation, and retry tracing found no bypassing `limactl`
spawn and no affected Windows/WSL or ordinary Linux path.
