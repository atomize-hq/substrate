# R6 Correction-01 lifecycle and convergence review — CLEAN

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6-CORRECTION-01`
- Final candidate subject: `sha256:6333a3f395b164ad484f31ad64786cb81a3ca47a0394be16bc6ee904cfdb89ce`
- Combined read-only audit: `d4bd729de8d344dfc567aaa62203838f68c2c895..candidate`
- Result: **CLEAN.** All required transition rows converge inside the amended R6 fence.

## Durable lifecycle closure

The operator proof is the sole bridge from the independent guest controlling TTY to data-session intent admission. Before proof, direct operator loss records only the fixed `operator-tty-direct-child-failed` observation and transitions the exact generation-CAS record to `pre_intent_closed`. Replays or alternate records cannot replace that closure. A fresh pre-intent retry has a new ticket challenge, nonce, data/operator session IDs, and record; it never silently adopts a prior ticket, proof, or record.

After proof, current record generation is carried separately from immutable `binding.host_record_generation`. The accepted data session preserves the signed ticket, binding, proof digest, and durable frames through guest-root, Hello, transcript, and terminal consume states. The consumed marker returns only an exact prior terminal result; altered replay rejects without mutation. The expiry boundary is `now >= expires` for host and guest checks. Normal issue preserves every expired active record. Only the explicit exact full-record retry of an expired proof-less pre-intent record can allocate new authority, and its fixed lifetime is capped by still-valid Stage-1 authority.

## Executable proof

Focused executable checks passed for the fixed state-machine rows rather than source shape alone:

- common signed host-record CAS rejects stale/replayed/expired transitions, validates one-way operator pre-intent closure, distinct session identities, and decoder rejection of a third session;
- macOS tests exercise equality expiry, preserved-vs-exact-fresh admission, bounded fresh ticket expiry, new challenge/nonce identity, progressed-expiry preservation, and fixed failure plan rejection of caller-selected observations;
- Linux guest tests exercise pipe/EOF and non-controlling-TTY rejection, timeout, echo suppression/restoration on `Drop`, proof digest binding, proof-gated intent, equality expiry, exact consumed marker behavior, and closed guest entrypoint selector rejection;
- shell tests reject a public operator tag while retaining the private direct control tag and reject malformed data/control forms.

The authoritative checks were `cargo test -p substrate-common -- --nocapture`, `cargo test -p shell --test managed_lifecycle_v1 -- --nocapture`, `cargo test --bin substrate-lifecycle-macos -- --nocapture`, `RUSTFLAGS=-Awarnings cargo test --bin substrate-lifecycle-linux r6_ -- --nocapture`, both required cargo checks including `x86_64-apple-darwin`, and `bash tests/mac/lifecycle_r3.sh`. No native action was performed.

## Combined recovery audit

The 56-path range union was attributed to the committed R1-R5 recovery records/reports and the nine R6 product/test paths. No valid P1/P2 was found outside the R6 amendment fence, so frozen R1-R5 surfaces were not edited. The only new nonblocking observation is historical P4 trailing whitespace already committed at the base in the three R5 review Markdown files; it is recorded only and not remediated.
