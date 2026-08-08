# R6 Correction-01 authority and security review — CLEAN

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6-CORRECTION-01`
- Final candidate subject: `sha256:6333a3f395b164ad484f31ad64786cb81a3ca47a0394be16bc6ee904cfdb89ce`
- Base: `a9626bc10a1737416c1d8f7f48630850118f3e6a` / `3f7cdf20125a847a3dfaa472d3797759b5cf6737`
- Authority: amendment `0036-r6-direct-tty-proof-retry-correction` (`sha256:0efd2a9be850672fb3c909806ccc54bdac3bd5b6f3bbf34383f8feec3aeae27d`)
- Immutable R6 discovery lineage: `r3-mac-evidence-recovery-r6-review-cycle-record.json` (`sha256:12f9659045852167a8d0a6f89471f88b20237a691474c881f110cdde3d1f4485`)
- Result: **CLEAN.** No R6 P1/P2 remains.

## Authority closure

The control-only direct operator capability is signed against the retained ticket, measured again before launch, and invokes one fixed `limactl shell --tty=true` command with independently duplicated `/dev/tty` descriptors. The direct-only operator protocol tag is private to `substrate-lifecycle-control`; it does not decode through the shell public mapped/XPC request type. The XPC client and listener carry only the typed data-session form; no XPC FD, terminal relay, generic operation, caller-selected transport, or public terminal API remains.

The guest operator receives only a signed canonical binding envelope. It opens the controlling guest TTY, suppresses and restores echo through `R6TerminalEchoGuardV1` including `Drop`, and atomically stores one fixed-path no-follow `GuestPublisherPairingOperatorProofV1`. The proof contains only binding/session metadata and a non-reusable commitment. The separate data session requires the independently loaded proof before intent publication and records its canonical digest through the host-record generation CAS. Ticket/data values cannot establish confirmation authority by themselves.

The exact original ten blocking inputs were remediated: direct TTY replaces the pipe relay; proof gates data intent; immutable binding generation is distinct from mutable current CAS generation; pre-intent loss closes and fresh retry remints identities; consumed-marker retry is exact/idempotent; expiry is uniformly `now >= expires`; terminal-FD/public-terminal APIs and confirmation projection are absent; and focused executable proof covers the former proof gaps.

## Correction discovery / closure findings

The correction discovery identified and remediated `P1-R6-CORR-EXPIRED-ACTIVE-REPLACEMENT`, `P1-R6-CORR-OPERATOR-FAILURE-NOT-DURABLE`, and `P2-R6-CORR-PRIVATE-OPERATOR-TAG-ABSENT`. A first closure then identified `P2-R6-CORR-EXECUTABLE-RECOVERY-PROOF`; focused executable expiry/fresh-attempt and fixed-failure tests were added. A causal supplemental review identified `P1-R6-CORR-FRESH-ATTEMPT-EXPIRY`, `P1-R6-CORR-PROGRESSED-EXPIRY-PRESERVATION`, and `P2-R6-CORR-THIRD-SESSION-PROOF`; the final review confirmed all are closed:

- a R6 ticket derives `min(Stage-1 expiry, now + fixed R6 lifetime)`, allowing a fresh pre-intent attempt while never extending Stage-1 authority;
- any ordinary expired active issue returns `expired_preserved` without mutation; only an exact proof-less `sessions_opened` full-record retry may allocate replacement authority;
- decoder rejection proves that a third stored session cannot enter the closed host-record contract; and
- the operator failure transition is an executable, signed, one-way durable `pre_intent_closed` CAS transition.

## Analysis and containment

GitNexus has no index for this exact `1bd8` worktree (the available indexes target other worktrees), so graph output was inconclusive and `npx gitnexus analyze` was not run. Manual caller/callee, decoder/XPC, descriptor/no-follow, termios/Drop, proof, CAS/generation, replay/retry, expiry, and macOS cfg analysis was completed. The final fresh closure reviewer found no in-scope P1/P2/P3 and no outside-fence P1/P2.

No Keychain, XPC service, Lima, code-signing, installation, pairing, or native evidence action was executed.
