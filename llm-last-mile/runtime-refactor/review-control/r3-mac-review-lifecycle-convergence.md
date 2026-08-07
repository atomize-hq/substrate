# A1.1d-5R3-MAC fresh MAC review epoch

This is the nonce-bound attempt-4 review epoch under
`substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` authority amendment
`0003-fresh-mac-review-epoch.json`. The prior four-cycle bounded-stop record is immutable
predecessor evidence; its byte digest is `sha256:bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`. This fresh record does not append a fifth
historical cycle. No native evidence, publisher installation, code-signing, Keychain action,
launchd action, Lima action, or successor dispatch is included.

## Discovery review

Discovery subject: `sha256:8da3cf1e8cb7d043d86ada4e63a08a5c5df6283e45dd904cf9b72c10810f60d2`. The review treated the private/cfg/Drop forwarding surface as a
manual HIGH-risk surface even though the graph risk was low. The typed mapped constructor remains
the only product path changed: it has `StreamLocalBindUnlink=no`, owns no VSock/TCP/selector
activation, and retains child kill/wait before cleanup.

`P1-MAC-A4-002` required the remediation to stop relying on an identity-check-then-path-mutate
sequence. The delta therefore lets the SSH child own normal listener teardown; when an exact
socket remains after reap, the host fails closed and preserves it for explicit reconciliation.
For a preexisting A-local known-host entry, restoration opens the verified regular-file identity
with no-follow semantics and writes through that descriptor. An initially absent path created by
the child is likewise preserved rather than unlinked by pathname. The existing early-exit/retry
fixture models the child performing its own orderly listener cleanup.

The discovery review admitted no P3/P4 and no alternate transport, selector, mapping, or native
proof work.


## Closure-1 review

The independent reviewer found `P1-MAC-A4-003` at the capture boundary, not a new selector or
transport domain. The causal repair is strictly local: capture existing A-local known-host bytes
and its physical identity from one opened no-follow descriptor; if absent/replaced, preserve rather
than adopt it. This is the first of at most two fresh-epoch supplemental causal cycles.


## Supplemental-causal-1 review

`P2-MAC-A4-004` is a bounded availability defect in the descriptor snapshot only. The repair uses
non-blocking no-follow open before `fstat`, so a FIFO is rejected as non-regular without waiting
for a writer. The new focused unit test creates a FIFO and requires prompt rejection. This is the
second and final fresh-epoch supplemental causal cycle; its next fresh review must end clean.


## Supplemental-causal-2 final review

Final fresh review returned **CLEAN** for subject
`sha256:2f504d6b21503396af7b48276ec238de88f43f22dc4ac2c824131dce1252502c`. The FIFO negative
passes without a writer, mapped pre-spawn replacement remains fail-closed, and normal child-owned
listener cleanup supports retry without host-side pathname unlink. No further review cycle is
authorized or needed.
