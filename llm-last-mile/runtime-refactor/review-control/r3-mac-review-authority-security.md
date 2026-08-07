# A1.1d-5R3-MAC fresh MAC review epoch

This is the nonce-bound attempt-4 review epoch under
`substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` authority amendment
`0003-fresh-mac-review-epoch.json`. The prior four-cycle bounded-stop record is immutable
predecessor evidence; its byte digest is `sha256:bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`. This fresh record does not append a fifth
historical cycle. No native evidence, publisher installation, code-signing, Keychain action,
launchd action, Lima action, or successor dispatch is included.

## Discovery review

Fresh independent read-only review model/reasoning: `gpt-5.6-terra` / Extra High.
Discovery subject: `sha256:8da3cf1e8cb7d043d86ada4e63a08a5c5df6283e45dd904cf9b72c10810f60d2`. The manifest is the expected base followed by all 16 exact
non-review subject paths, mode or `NEW`, and no-filter blob ID; review-control files are excluded.

The reviewer returned the following P1/P2 findings; no P3/P4 result was admitted.

- **P1-MAC-A4-001 — privileged carrier commitment was not recomputed.** The original binary
  only checked that the canonical carrier's commitment field looked like a SHA-256 string. A
  direct fixed-XPC `lima-action` request could bypass the shell recomputation before state/receipt
  mutation.
- **P1-MAC-A4-002 — Drop cleanup retained pathname TOCTOU mutations.** Socket/known-host
  identity observations were followed by pathname unlink/write operations. A replacement after the
  check could have been mutated.
- **P2-MAC-A4-001 — bootstrap accepted an unattested response.** The MAC client decoded a
  bootstrap response without requiring `audit_token_bound: true` and the exact service/requirement
  attestation.

The bounded remediation recomputes the carrier hash in the fixed XPC handler, writes a preexisting
known-host entry only through a verified no-follow descriptor, preserves a leftover socket or
created known-host path rather than performing a post-check path unlink, and attests bootstrap
responses. Closure review must re-evaluate only this delta.


## Closure-1 review

A different fresh read-only `gpt-5.6-terra` / Extra High reviewer assessed closure subject
`sha256:c1f700f483ee59529c41b2f797815b903ab48cd72fffe22ee6f252ab70230765`. It confirmed the
carrier recomputation and bootstrap attestation corrections, but returned one P1 directly unmasked
by the Drop remediation.

- **P1-MAC-A4-003 — known-host snapshot bytes and identity were not coherent.** The pre-state
  reader observed an identity, read bytes by pathname, then independently observed an identity
  again. A replacement between those steps could bind predecessor bytes to a later foreign inode
  and cause a descriptor-verified restore to alter that foreign file.

The one authorized supplemental causal remediation changes capture to one no-follow file
descriptor that supplies both bytes and device/inode. Closure is therefore `FINDINGS`, not clean.


## Supplemental-causal-1 review

A third fresh read-only `gpt-5.6-terra` / Extra High reviewer assessed subject
`sha256:5eed302e680a58332b210f2b4066ba8c8f3b13dba96cc20c22507dbc62e1e002`. It returned no P1
but one direct P2 consequence.

- **P2-MAC-A4-004 — FIFO snapshot availability denial.** Opening a pre-existing FIFO read-only
  before file-type rejection can block waiting for a writer. This is directly caused by moving the
  capture coherence check to a descriptor.

The final allowed causal remediation adds non-blocking open flags before file-type inspection and
an executable FIFO-negative. No new authority, selector, platform, or evidence work is admitted.


## Supplemental-causal-2 final review

A fourth fresh read-only `gpt-5.6-terra` / Extra High reviewer assessed final subject
`sha256:2f504d6b21503396af7b48276ec238de88f43f22dc4ac2c824131dce1252502c` and returned
**CLEAN**. It confirmed non-blocking no-follow FIFO rejection, coherent known-host capture and
restore, XPC carrier rehash/peer audit binding, bootstrap attestation, and typed SSH-only
activation. No P1/P2/P3/P4, native evidence, selector, or file-fence escape was returned.

This completes the fresh review epoch. The result proves only the non-native implementation
packet; it does not claim native execution, evidence, promotion, or successor dispatch.
