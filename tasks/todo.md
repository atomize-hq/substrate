# Historical completed tasks: A1.1c

> **Bookkeeping status:** Every item below is preserved completed A1.1c history, not a current task
> queue. R2-3 is complete. R2-4 is the next authorized packet but is not started. Its remaining
> start gates are inactive prerequisites only: a clean/current indexed checkout, the operator-
> approved dedicated Linux window and restoration baseline, and explicit future packet authority.
> No executable R2-4 task is opened by this file.

## Task 1: Durable store schema and validation

- [x] Define initialization, key registry, root, certificate, namespace,
  intent/journal, object-index, and storage-state records.
- [x] Reject unknown fields, wrong versions, repeated-key mismatches, invalid
  active-key/certificate state, and inconsistent object lifecycle variants.
- [x] Verify with focused canonical round-trip and negative tests.
- Files: `store_schema.rs`, `mod.rs`.

## Task 2: Closed grammar and key envelope

- [x] Parse/generate only exact `as_`, `ak_`, `ao_`, nonce, and four temp-name
  grammars.
- [x] Encode/decode the exact immutable commitment-key envelope with no trailing
  bytes and redacted errors.
- [x] Verify byte vectors, malformed lengths/tags/IDs, and round trips.
- Files: `store_format.rs`.

## Task 3: Locked layout and bootstrap classification

- [x] Create/open and durably sync the fixed directory skeleton and lock file.
- [x] Reconcile recognized temps; reject malformed or unsafe temp entries.
- [x] Safely enumerate both legacy authority collections and implement exact
  five-state precedence.
- [x] Verify fresh, legacy, malformed, unsafe, and replacement cases.
- Files: `store.rs`, `store/platform/{layout,legacy}.rs`, plus focused tests.

## Task 4: Fresh and pending initialization recovery

- [x] Publish marker, key, revision-1 root/certificate, and marker deletion in
  exact durable order.
- [x] Resume every pending crash window without replacing persisted identity.
- [x] Verify competing initializers converge on one store/key/root.
- Files: `store.rs` plus focused tests.

## Task 5: Existing root, key, and object verification

- [x] Strictly validate existing root identity, registry, key files, object
  routing tree, reachable refs, and leftover matching marker cleanup.
- [x] Retain safely named unindexed objects; reject malformed/unsafe entries.
- [x] Verify missing or mismatched reachable data is corruption.
- Files: `store_schema.rs`, `store/platform/{layout,reachability}.rs`, plus focused tests.

## Task 6: Typed publication, exact adoption preparation, and key lifecycle

- [x] Publish typed immutable objects through temp/no-replace/fsync ordering.
- [x] Join existing objects only after exact byte/ref verification.
- [x] Prepare retained orphans only with exact parent/store/intent/run context;
  leave the semantic parent/root commit to the later transaction owner.
- [x] Reconcile unregistered, verification-only, and retired key-file states.
- Files: `store/platform/{object_persistence,key_lifecycle}.rs`, `store_format.rs`, plus focused tests.

## Task 7: Packet wall and review

- [x] Complete marker/key/root/object/temp crash matrices.
- [x] Run targeted tests with nonzero execution and zero ignored.
- [x] Run formatting, full shell Clippy, diff check, and GitNexus detection.
- [x] Obtain a fresh read-only A1.1c reviewer and resolve valid findings before
  committing.
- [x] Confirm A1.1d has not started and the diff remains independently
  reviewable.
