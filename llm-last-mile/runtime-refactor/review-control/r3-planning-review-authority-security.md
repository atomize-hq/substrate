# R3 planning authority and security review

Terminal subject fingerprint:
`sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`

The subject is the ordered lexical content of `00-README.md` through
`05-debug-regression-ledger.md`, recomputed as `sha256sum` of the six files followed by
`sha256sum` of that ordered output. A fresh read-only gpt-5.4 Extra High reviewer using standard
speed inspected the terminal bytes; it authored no subject byte.

The causal review protocol calls a cycle `CLEAN` when it has zero unresolved P1/P2 findings.
P3/P4 do not trigger a causal cycle. Separately, this planning increment forbids publication with
any unresolved P1-P4 finding. The terminal subject passes both rules; the protocol rule is not
being restated as the stricter publication rule.

## Recorded causal lineage

The discovery subject was
`sha256:da97c6ca51b27a61fb1e0f64050030a46f84b38329136ffc337f263a0fe0576d`.
Its authority findings were `R3-AS-005` (P1), because retirement authority was not precommitted
into the generation-one publisher lineage, and the raw reviewer's first `R3-AS-006` (P1), because
an empty guest lacked a non-circular trust source for the dynamic host publisher key. The closed
record uses `R3-AS-006-DISCOVERY` as a transparent occurrence alias for that first raw ID because
the v1 validator requires finding IDs to be globally unique; this does not rename or rewrite the
raw finding. The discovery cycle also carried `R3-CPA-002`, `R3-CPA-004`, `R3-CPA-003`,
`R3-LC-DISCOVERY-001`, and `R3-LC-DISCOVERY-002` at their recorded priorities.

The closure subject was
`sha256:e2b91149587e8d23dab2455de443e56383506d75bac041ca1402ecee71e43835`.
It closed the discovery set but returned `R3-AS-006` (P1): retained child-channel continuity did
not authenticate the protected host publisher and the pairing ticket lacked the canonical SPKI
bytes needed to verify its signature. It also returned `R3-AS-007` (P1): retirement authorization
and receipt wording reintroduced future/self digest cycles. Those two findings, and only those
two, triggered the changed-subject `supplemental_causal` cycle.

## Supplemental causal evidence

The terminal subject closes `R3-AS-006` by binding canonical P-256 SPKI DER, its complete SHA-256
fingerprint, and the signed current-anchor record into the one-use pairing ticket. The operator
transfers the complete fingerprint, challenge, and literal only between independently opened host
and guest controlling TTYs. The guest hashes and exact-matches the SPKI before verifying the
anchor and fixed P1363 low-S signature. Retained `limactl shell` or `wsl -d` continuity carries no
authentication authority; substituted SPKI/encoding/key/child/channel and high-S cases reject.

The terminal subject closes `R3-AS-007` with a null-slot bootstrap core that precommits the
retirement key, external-store physical identity, authorization digest, and complete reservation
set without final-bootstrap self-reference. Signed retirement and action receipts contain no
self, artifact, future index/head/anchor, or resulting CAS digest. Receipt bytes and parents are
fsynced before external hashing; the separately precommitted harness acknowledgement is fsynced
and hashed before teardown. Managed publication proceeds receipt, external hash, receipt-index
CAS, head CAS, then anchor CAS, with golden order and kill/retry negatives.

The same causal inspection closed later draft hazards before the terminal review: unused
reservation proof binds only prior protected state and expected next revision before proof and
acknowledgement hashing; both `Reserved` and `TicketIssued` branches use exact prior-state CAS and
zero guest effects. Linux, Lima, WSL, and host/guest publisher socket or endpoint activation now
has singular ownership. A marked service-state role precommits its fixed non-requestable endpoint
in one protected transition and receipt; every managed world socket source omits service-to-socket
propagation. Candidate rollback remains current-attempt, descriptor/handle-bound, no-follow,
exact-identity, empty, and nonrecursive. Manifest deletion authority remains independently located,
monotonic, exact-role/action bound, preservation-first, and non-adopting.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
