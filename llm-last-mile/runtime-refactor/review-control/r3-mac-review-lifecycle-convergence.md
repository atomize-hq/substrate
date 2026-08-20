# R3 MAC lifecycle and convergence discovery review

Discovery subject fingerprint:
`sha256:1097874180b86a3331c7d14c06d2b803cfd545273576e691b8eb1f1ec5bb4bca`

A fresh, read-only `gpt-5.6-terra` reviewer using Extra High reasoning assessed the MAC lifecycle
and recovery boundary before remediation.

## Required convergence shape

1. Typed `MacLimaBackend` traffic may activate only a mapped SSH-UDS attempt. Compatibility
   `auto_select`, VSock, TCP, ambient socket selection, and `new_with_mapping` remain frozen.
2. The forwarding attempt records selected socket, child, and known-host before-state. Timeout,
   Drop, retry, and handle loss may retire only the exact created identity and must restore a
   recorded A-local known-host entry.
3. The warm and stop scripts become transport-only carrier/PM request wrappers. The executor,
   not a shell default, owns exact Lima stage, guest artifact, group/home/unit/socket, and restore
   actions. Missing or mismatched build evidence performs no guest mutation.
4. The guest lifecycle path is model/fixture proof only. No invocation may provision a VM, build,
   code-sign, install, bootstrap, or exercise a native publisher.

Baseline parser/format and `git diff --check` were clean. `cargo test -p world-mac-lima` passed
48 tests; `tests/mac/prefix_mapping_r2_3.sh` and `tests/mac/lima_doctor_fixture.sh` passed.
The unrelated baseline `cargo test -p shell managed_lifecycle --lib` failed to compile with 56
pre-existing errors in `crates/shell/src/repl/async_repl.rs` (including missing
`write_test_world_stream_frame`, `finish_test_world_chunked_stream`, and
`write_test_world_http_json`). The baseline installer fixture also failed before this packet's
subject changed because `install-substrate.sh` attempted `exec: {context_fd}: not found`.
These failures are recorded as baseline debt and are not repaired by this MAC fence.

Protocol discovery verdict: `FINDINGS`.

## Closure-1 delta review and remediation

Fresh closure subject:
`sha256:f4db3ee786ba2c0e8ace7bda5a1064705b90afa488ce1d8eac089126c786dc65`.

`MAC-R3-P1-04` found that early SSH exit/timeout paths could leave a created socket or
A-local known-host mutation and could let a socket replacement suppress known-host reconciliation.
The remediation adds one failed-attempt cleanup routine that kills/waits before exact socket
reconciliation, captures socket device/inode before unlink, and restores known-host state
independently. Existing known-host files are identity-checked before restore; absent entries are
removed only when a regular file was created by the attempt. A replacement or symlink is preserved
and reported rather than followed/unlinked.

`MAC-R3-P2-01` required executable rather than source-text-only negatives. Embedded forwarding
tests now run a fake SSH that writes both a socket and known-host mutation before exit, then prove
cleanup and retry; they also prove replacement and symlink preservation. The MAC lifecycle fixture
uses only a copied fixed-path mock and proves missing, malformed mapping, and malformed evidence
stop before executor dispatch.

Protocol closure-1 verdict: `FINDINGS`; the supplemental causal review must re-evaluate these
changes and the complete exact cleanup/timeout/Drop surface.

## Supplemental-causal-1 review

Fresh subject:
`sha256:1cabe3430ef0a28a88ef560cca26cfc4d934c1107ca2512d66f22c126583fa31`.

The reviewer found that early-exit cleanup captured a socket identity only at cleanup time, which
could mistake a replacement for an owned socket. An initially absent known-hosts pathname likewise
had no created-file identity. The required bounded follow-up is to capture each owned identity
while the child is observed live and to preserve an unobserved/replaced path. It must add direct
non-native race tests while retaining exact Drop, timeout, and retry behavior for owned state.

The reviewer also found the fixed `run-publisher` listener has no request handler and the shell
client is a direct stdio process relay. The follow-up must be a fixed-name XPC request/response
and peer-token binding, with no alternate transport or standalone mutation operation.


## Supplemental-causal-2 final review — bounded stop

Fresh final subject:
`sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924`.

- **P1-MAC-FINAL-003 — stage-one create/start remains ambient direct mutation.**
  `lima-warm.sh` receives `SUBSTRATE_LIMA_VM_NAME`, rejects only names other than `substrate`,
  then directly starts or creates that VM before the mapping is observed. The imported
  `LimaStageOneAuthorizationV1` is not validated.
- **P1-MAC-FINAL-004 — a pre-spawn SSH unlink race remains.** The mapped forwarding path checks
  absence before it launches SSH with `StreamLocalBindUnlink=yes`; a foreign socket or symlink
  created between those operations can be removed before the live-child ownership capture.
- **P2-MAC-FINAL-001 — lifecycle fixture does not exercise XPC semantics.** It substitutes a
  fixed path with a stdio mock and source-greps behavior, so it cannot validate listener peer
  enforcement, real audit binding, or response attestation.
- **P2-MAC-FINAL-002 — forwarding tests omit the pre-spawn unlink race.** No test inserts a
  foreign pathname between the absence check and SSH spawn.

All returned P1/P2 findings are blocking. A repair would require a third supplemental causal
cycle, which the frozen review protocol prohibits, and may require authority beyond the current
canonical/Stage-1 validation fence. The terminal record is therefore a bounded stop rather than
another remediation pass. No P3/P4 finding was admitted.

Protocol supplemental-causal-2 verdict: `FINDINGS`; terminal control disposition:
`bounded_stop` / `budget_exhausted`.
