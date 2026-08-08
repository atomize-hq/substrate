# R6 authority and security review — bounded stop

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6`
- Review subject: the nine R6 production/test paths at `a9626bc10a1737416c1d8f7f48630850118f3e6a` plus the uncommitted candidate diff
- Subject fingerprint: `sha256:018791e14303b553d10286933aba6f911b41b3ad3247cf7bb3aa76092ff51462`
- Authority: `authority-amendments/0035-r6-closed-session-wiring-fence.json` (`sha256:5d540701bcd947d3c565539a03cb0395ddcc5d2eda7bcff64664c717a17c849b`)
- Result: **FINDINGS — bounded stop; no publication.**

## P1 findings

1. **`P1-R6-OPERATOR-TTY-PIPE-RELAY`** — The macOS operator child is created with piped stdin/stdout, receives the start frame on that pipe, then relays bytes read from the retained host terminal into that pipe (`src/bin/substrate-lifecycle-macos.rs:8292-8384`). The guest source documents the same host-terminal-to-PTY relay (`src/bin/substrate-lifecycle-linux.rs:522-530`). This violates the closed-session rule that confirmation is read only from the independent guest controlling TTY and never from a pipe or automation channel.
2. **`P1-R6-DATA-DERIVED-CONFIRMATION`** — The data child derives `confirmation_commitment` solely from the ticket/binding-derived nonce, host fingerprint, challenge, and literal, then publishes the intent (`src/bin/substrate-lifecycle-linux.rs:438-460`). The TTY path computes the same formula after input (`:702-742`), but its result is not causally required by intent publication. Ticket/data material can therefore supply the commitment, contrary to the host-only/operator-TTY authority contract.
3. **`P1-R6-CONSUME-RETRY-NONIDEMPOTENT`** — macOS explicitly accepts exact ticket-consumed recovery (`src/bin/substrate-lifecycle-macos.rs:8037-8063`), while Linux unconditionally rejects an existing consumption marker (`src/bin/substrate-lifecycle-linux.rs:2980-3004`). A loss after marker persistence cannot converge by byte-identical replay.

## P2 findings

1. **`P2-R6-UNAUTHORIZED-PUBLIC-TERMINAL-API`** — New public terminal-passing APIs in `crates/shell/src/execution/managed_lifecycle.rs:942` and `crates/shell/src/execution/managed_lifecycle/macos_client.rs:84,772` exceed Amendment 0035's named fixed relay/channel allowance, which permits immediate private helpers rather than a widened public terminal-FD interface.
2. **`P2-R6-EXPIRY-EQUALITY`** — Linux rejects only `now > expires` (`src/bin/substrate-lifecycle-linux.rs:887-891`) while canonical host validation rejects at `now >= expires`; equality can pass guest intent admission.
3. **`P2-R6-CONFIRMATION-PROJECTION`** — Operator guest output is copied wholesale to the retained host terminal (`src/bin/substrate-lifecycle-macos.rs:8398-8403`), while the guest confirmation read has no echo suppression (`src/bin/substrate-lifecycle-linux.rs:702-742`). The full values may be projected.

## Combined R1–R6 read-only audit

The combined range was `d4bd729de8d344dfc567aaa62203838f68c2c895..candidate`. Review used the committed R1–R5 review-cycle lineage at `review-control/r3-mac-evidence-recovery-r{1,2,3,4,5}-review-cycle-record.json`, the R5 Correction-01 cycle record, and the corresponding three-lens reports. The committed path attribution includes R1 installer recovery, R2 shell/lifecycle recovery, R3 evidence planning, R4 common/macOS authority, and R5 lifecycle bridge/correction paths. No independently confirmed new P1/P2 was found outside the Amendment 0035 R6 fence. All findings above are in-fence; therefore earlier frozen R1–R5 surfaces were not edited.

There is no R6 landing receipt: the required terminal meta receipt is held for this bounded stop. The R6 review-cycle record is the in-tree lineage artifact for this non-published candidate.
