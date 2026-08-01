# Pre-R2-4 readiness discovery review: boundary and host booking

Subject fingerprint: `sha256:8143807df46996773e4eaf20b853393a4d40f6379a20add1ce4eb8a57c5e17b6`

The isolated read-only reviewer verified every entry in
`pre-r2-4-readiness-subject.sha256`, the live read-only host facts, the R2-4/R3 boundary, PI-050,
the deferred R2-3ZP3 status, the three classified failures, the four accepted P3 entries, the
docs/process-only allowlist, and zero GitNexus affected production flows.

- P1: none.
- P2 (`PRE-R2-4-P2-001`): hashes alone cannot restore overwritten or deleted artifacts. The
  restoration contract promised exact restoration without requiring an immutable recoverable copy
  or independently verified restore source for every mutable pre-existing artifact. Require source
  identity, digest, access, retention, and restoration validation before mutation.
- P3: none.
- P4: none.

Terminal verdict: `FINDINGS` with one open P2.
