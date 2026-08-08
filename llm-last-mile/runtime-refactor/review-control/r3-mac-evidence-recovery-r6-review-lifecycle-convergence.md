# R6 lifecycle and convergence review — bounded stop

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6`
- Subject fingerprint: `sha256:018791e14303b553d10286933aba6f911b41b3ad3247cf7bb3aa76092ff51462`
- Result: **FINDINGS — no causal remediation is authorized; no publication.**

## P1 findings

1. **`P1-R6-OPERATOR-TTY-PIPE-RELAY`** — The purported operator session reads the retained host terminal and writes it into the child stdin pipe (`src/bin/substrate-lifecycle-macos.rs:8318-8384`), instead of the guest controlling TTY being the sole confirmation reader. Linux confirms the relay (`src/bin/substrate-lifecycle-linux.rs:528-530`).
2. **`P1-R6-REJOIN-GENERATION-MISMATCH`** — Ticket issue returns the record's current generation, which control passes as the immutable binding generation (`src/bin/substrate-lifecycle-control.rs:136-183,267-285`). Session admission instead requires that request field to equal the original binding generation (`crates/shell/src/execution/managed_lifecycle.rs:728-746`; `src/bin/substrate-lifecycle-macos.rs:7848-7867`). Once a CAS transition has advanced the record, durable rejoin cannot pass admission.
3. **`P1-R6-PREINTENT-REUSE`** — `guest_state_root_prepared` is durably written before the data child starts (`src/bin/substrate-lifecycle-macos.rs:7374-7389`) and admitted as a rejoinable active record (`:7683-7729`). The control path then skips a fresh operator session for a non-`sessions_opened` record (`src/bin/substrate-lifecycle-control.rs:141-175`). This silently adopts a ticket/nonce/record after pre-intent loss, contrary to retry-table row 1.
4. **`P1-R6-CONSUME-RETRY-NONIDEMPOTENT`** — See the authority review: the marker rejection at `src/bin/substrate-lifecycle-linux.rs:2988-3003` contradicts macOS's exact consumed replay branch.
5. **`P1-R6-EXPIRY-MUTATION`** — An expired active record returns `Ok(false)` from rejoin validation (`src/bin/substrate-lifecycle-macos.rs:7691-7700`), which falls through to fresh ticket/record creation (`:7224-7248`). The required expiry row preserves closed evidence with no resume or mutation.

## P2 proof finding

**`P2-R6-PROOF-GAPS`** — The focused additions are primarily source-shape/common-record assertions. They do not execute the required negatives for a missing/non-controlling guest TTY, third session, confirmation through a pipe, full pre-intent loss handling, durable rejoin, consume retry, expiry preservation, stale CAS, or Drop/kill boundaries. `tests/mac/lifecycle_r3.sh` positively requires the prohibited relay string.

## Combined R1–R6 convergence result

The read-only range `d4bd729de8d344dfc567aaa62203838f68c2c895..candidate` was attributed against the committed R1–R5 review records/reports and the current R6 subject. No new valid P1/P2 outside the amended R6 paths was found. The recovery chain cannot be marked clean because the above R6 state-machine rows diverge; frozen earlier packets remain untouched.
