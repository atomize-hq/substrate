# Historical completed plan: A1.1c Authority Store Persistence

> **Bookkeeping status:** This plan is retained as completed A1.1c history and is not executable
> current work. R2-3 is complete. R2-4 is the next authorized packet but is not started. Its
> remaining start gates are inactive prerequisites only: a clean/current indexed checkout, the
> operator-approved dedicated Linux window and restoration baseline, and explicit future packet
> authority. No R2-4 implementation sequence is defined here.

## Architecture decisions

- Keep durable persistence schemas separate from the already-large named
  object schema module.
- Keep key-envelope and filename grammar code pure so byte-level tests are
  small and deterministic.
- Make the store orchestrator the only A1.1c consumer of trusted-fs handles.
- Inject initialization material in tests while production uses OS entropy and
  the canonical timestamp shape.
- Separate classification from mutation: classify under the lock, then run the
  one recovery path selected by that classification.
- Model crash tests at each durable publication boundary rather than mocking
  filesystem calls.

## Dependency graph

```text
durable store schema + validation
        │
        ├── closed IDs/temp grammar
        ├── key envelope codec
        │       │
        └───────┴── store layout + locked classifier
                        │
                        ├── fresh/pending initialization recovery
                        ├── existing root/key/object verification
                        └── typed object publication/adoption preparation
                                      │
                                      └── crash-matrix regression wall
```

## Ordered increments

1. Add exact durable store schemas and semantic validation.
2. Add closed identifier/temp grammar and key-envelope codec.
3. Add locked layout creation, temp reconciliation, and legacy classifier.
4. Add fresh initialization and pending crash recovery.
5. Add valid-existing root/key/object verification.
6. Add typed object publication, exact retry, retained orphan preparation, and key
   lifecycle recovery.
7. Complete crash matrices and run packet-wide gates.

Each increment starts with a failing focused test, makes the minimum code pass,
and reruns the affected focused wall before the next increment.

## Verification checkpoints

### Schema checkpoint

- Canonical round trips reject unknown fields and semantic mismatches.
- Key envelope and temp grammar vectors pass.
- Formatting and focused Clippy pass.

### Bootstrap checkpoint

- Fresh and pending recovery matrices pass on a real filesystem.
- Legacy and corrupt states never initialize.
- Recognized temp cleanup is durable; malformed temp state fails closed.

### Object/key checkpoint

- Typed routes and commitments verify.
- Exact retry can join only complete matching bytes/context and returns no
  semantic root mutation authority.
- Orphans remain retained and non-authoritative.
- Reachable missing/mismatched objects or keys fail closed.

### Complete checkpoint

- All A1.1c targeted tests execute and pass with zero ignored.
- Formatting, full shell Clippy, diff check, and change detection pass.
- A fresh read-only packet reviewer returns clean before commit.

## Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Ambiguous `ENOENT` or race | Accidental fresh-store creation | Use only trusted directory-relative lookup and revalidation under flock |
| Schema accepts inconsistent maps | Corrupt root treated as valid | Validate every repeated map key and cross-record invariant |
| Crash state inferred from a temp/orphan | Fabricated authority | Never promote temps; require exact parent/ref/context for adoption |
| Secret bytes escape through diagnostics | Credential disclosure | Use typed redacted errors and never format key/raw bytes |
| A1.1d scope leaks in | Unreviewable transaction layer | Keep mutators private and omit expected-revision/CAS facade APIs |
| Cross-platform behavior drifts | Linux/macOS mismatch | Reuse A1.1b primitives and keep platform code out of A1.1c |
