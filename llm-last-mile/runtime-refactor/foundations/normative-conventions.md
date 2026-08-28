**Kind:** foundation
**Status:** canonical
**Canonical for:** normative conventions

## Normative conventions

- All V1 records are durable, schema-versioned, and reject unknown fields and identity/binding
  ambiguity.
- IDs and refs are opaque. Model-visible callers may receive task/worker handles but may not
  construct internal participant, resume, lease, policy-ref, or UAA-session truth.
- `root_revision`, `authority_revision`, and intent/receipt/manifest revisions increase
  monotonically under the exact atomic persistence rules owned by their contracts.
- Every A1 timestamp is `TimestampV1`; every A1 structured commitment is over the named immutable
  `CanonicalJsonV1` input, never a presentation record.
- A field marked `ref` is a complete `AuthorityObjectRefV1`. Its parent record owns the expected
  kind, schema version, and commitment and verifies all three before use.

**Source provenance:** extracted byte-for-byte from [`04-contracts-and-gates.md#normative-conventions`](../04-contracts-and-gates.md#normative-conventions), baseline lines 3–15
**Baseline span SHA-256:** `b8fd666cab13fa53ce90e5aefee49f8103aea286ff7186b5d235a838c958024c`
