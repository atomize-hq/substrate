# Historical completed spec: A1.1c Authority Store Bootstrap and Crash Reconciliation

> **Bookkeeping status:** This file is the preserved completed A1.1c specification; it is not the
> current implementation authority. R2-3 is complete. R2-4 is the next authorized packet but is
> not started. Its remaining start gates are inactive prerequisites only: a clean/current indexed
> checkout, the operator-approved dedicated Linux window and restoration baseline, and explicit
> future packet authority. This file creates no R2-4 implementation plan or task list.

## Objective

Implement the private A1.1c authority-store layer beneath
`host_session_authority`. The packet must deterministically classify and
recover fresh, pending, existing, legacy, and corrupt stores; publish and
verify the immutable initialization marker, commitment keys, root, and typed
objects; retain non-authoritative orphans; and prepare an orphan as an exact
adoption candidate only after complete byte/ref/parent-context verification.

Success means every specified marker/key/root/object/temp crash window
converges without path re-resolution, authority inference, or silent repair.

## Existing foundations

- `canonical_json.rs`: strict repository-owned canonical encoder/decoder.
- `hash.rs`: canonical SHA-256 and store HMAC commitment verification.
- `schema.rs`: A1 identity, ref, commitment, and named object schemas.
- `trusted_fs.rs`: opened-directory-relative, no-follow, same-filesystem file
  operations, atomic publication/replacement, durability barriers, and flock.
- `04-contracts-and-gates.md`: normative A1 persistence schema and crash rules.

## Public boundary

A1.1c remains `pub(crate)` and staged behind the existing module-level
dead-code allowance. It exposes no CLI, REPL, helper, StateStore, or production
transition operation. A1.1d owns centralized semantic preflight, writer
exclusion, and revision CAS; A1.1e owns the facade and exact resolution.

## Required behavior

1. Use the fixed `authority-v1` layout and exact owner-only modes.
2. Serialize classification and recovery on the no-follow-opened `root.lock`.
3. Recognize only the four closed temp filename grammars and delete recognized
   temps durably without promoting them.
4. Classify with exact precedence:
   `CorruptOrUnsupported`, `UnsupportedLegacyState`, `FreshAbsent`,
   `InitializationPending`, or `ValidExisting`.
5. Create fresh initialization marker, 256-bit key envelope, revision-1 root,
   and immutable greenfield certificate in the specified durable order.
6. Resume pending initialization using the persisted store ID, key ID,
   bootstrap identity, and timestamp; never mint replacements.
7. Strictly decode and validate existing root/key/object state, including map
   keys, active-key uniqueness, certificate identity, typed object paths, and
   parent-owned commitments.
8. Publish immutable typed objects with no-replace semantics and verify exact
   retries. Retain unindexed valid object files as non-authoritative orphans.
9. Adopt an orphan only when the caller supplies the exact complete ref,
   parent intent/run/store context, and bytes that verify the commitment.
10. Implement active/verification-only/retired key validation and deterministic
    crash cleanup without logging key or object bytes.
11. Fail closed on ambiguous absence, malformed names, unsafe types or modes,
    symlinks, replacement, legacy artifacts, missing reachable data, and
    unsupported platform/filesystem behavior.

## Testing strategy

- Unit tests for IDs, temp grammar, key-envelope bytes, root semantic
  validation, and typed object routing/commitments.
- Real-filesystem tests for fresh initialization, pending marker/key/root crash
  windows, temp cleanup, legacy classification, corrupt-state precedence,
  existing-store reopening, orphan retention, exact adoption, and key cleanup.
- Every targeted filter must execute at least one test with zero ignored.
- Run `cargo fmt --all -- --check`,
  `cargo clippy -p shell --all-targets -- -D warnings`, targeted A1.1c tests,
  `git diff --check`, and GitNexus change detection before review.

## Boundaries

- Always: operate from opened trusted handles; strictly decode canonical data;
  revalidate identities before publication; preserve durability ordering;
  redact sensitive values from errors.
- Ask first: dependency additions or changes outside the bounded authority
  module.
- Never: use ambient CWD or path-based descendant fallbacks; migrate or delete
  legacy authority; infer authority from temps/orphans; implement A1.1d CAS or
  production transition semantics; begin A1.1d.

## Assumptions

- Existing `rand`, `chrono`, `serde`, and `sha2` dependencies are sufficient.
- Production entropy uses `OsRng`; tests inject deterministic material.
- The complete durable schema is represented now, but A1.1c mutators remain
  private and do not issue or apply transitions.
- The original A1.1b physical-root binding is the bootstrap-home identity.

## Success criteria

- Fresh, pending, existing, legacy, and corrupt states are mutually exclusive
  and deterministically classified.
- Every initialization crash checkpoint recovers to the same root identity and
  initial key metadata.
- Recognized temps never become authority; malformed temps fail closed.
- Existing reachable key/object bytes verify exactly; missing or mismatched
  reachable data is corruption.
- Orphans remain retained and non-authoritative. A1.1c may publish or join only
  an exact verified adoption candidate; the later semantic transaction owner
  supplies the committed parent/root update.
- No production caller, dependency, A1.1d CAS behavior, or unrelated file is
  changed.

## Open questions

None. The normative contract fixes the layout, schemas, ordering, and non-goals.
