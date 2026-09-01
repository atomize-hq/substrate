**Kind:** slice row
**Stable ID:** `e1-restricted-world-fs-narrowing`
**Canonical for:** extracted E1 slice row plus its terminal implementation closure
**Status:** terminally complete
**Authority scope:** exact extracted source table header and row plus the terminal closure below; no E2 admission, dispatch, implementation, or completion authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 220
**Supersedes:** canonical ownership of the extracted `E1 — Restricted world_fs narrowing` row
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E1 — Restricted world_fs narrowing

> **Authority boundary:** This file owns the extracted E1 Track E row and its terminal closure. It preserves the exact row text below and records completed E1 authority and proof without admitting, dispatching, implementing, or completing E2.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E1 — Restricted world_fs narrowing** | Accept a request-scoped restricted `PolicyPatch.world_fs`, validate path-containment monotonicity, and materialize a canonical narrowed snapshot. | `02` EffectivePolicyResolver + NarrowingPatch rows; `04` narrowing contract/rules; steering policy matrix capability section | WorldCommandExecutionBroker; receipt acceptance; agent inventory overlay logic | `crates/broker`; `execution/{policy_model,policy_snapshot,agent_inventory}.rs`; transport API policy types; resolver tests | No new filesystem policy model; no broadening dimensions; no receipt/manifests yet. | Gate=false rejects; gate=true accepts only narrowing; file-under-directory containment works; escapes/symlinks/broadening fail closed. | `RG-POLICY-01`, `RG-POLICY-02` |

## Terminal closure

The completed implementation is bound to:

- commit `6194788d45267d91b4428a42e24c02dfcaae3c1e`, with sole parent
  `bbd5c76626ccb65240a3759ce2b8645764d957b2`;
- tree `6be705b4e957c071c05ec3a97fc973c05fa1f302`;
- subject `feat: add restricted world filesystem narrowing`;
- reviewed implementation fingerprint
  `sha256:35b77c26e04b55b9f355268a5f28be1ff1d7cfeb9aef6b6751894cda95ab3dac`; and
- final independent gpt-5.4 Extra High implementation verdict `CLEAN`.

E1 adds the strict V1 request carrier and a broker-owned additive
`EffectivePolicyResolver` entry point for `parent AND restricted world_fs patch`. The shell compares
the carrier with independently resolved request, session, caller, backend, world, subject, exact
parent reference/revision, canonical parent snapshot identity, and the separately resolved
narrowing gate before invoking the broker. The borrowed parent is unchanged. The result is
materialized only through the existing `PolicySnapshotV3` schema-3 canonicalization and hash path;
no durable schema, migration, receipt, manifest, HSA, lifecycle, or ambient-resolver behavior
changes.

The broker enforces every V1 boolean/rank/list row, rejects non-`world_fs` material and unsupported
path/glob syntax, and proves directory-to-child-file or exact-file containment relative to the
authoritative root. Inventory now consumes that same containment authority rather than an
equality-only comparison or process CWD. Linux Landlock distinguishes regular-file and directory
rule masks, intersects them with ABI-supported rights, never passes `READ_DIR` for an exact regular
file, and uses `openat2(RESOLVE_NO_SYMLINKS)` plus trusted `fstat` classification so final and
ancestor symlinks fail closed without granting a parent directory.

The binding Linux ABI-7 proof used the real `WorldService` and candidate-produced narrowed
snapshot: the named exact file succeeded, sibling and outside files failed, ancestor symlink
escapes failed, a nonexistent lexical exact-file target was rechecked at enforcement time, the
parent snapshot/hash stayed unchanged, and repeated resolution produced identical canonical
schema-3 bytes and hash. Focused Landlock coverage separately proved final-component symlink
rejection. Focused transport (2), broker (8), policy-snapshot (6), complete inventory module (20),
and Landlock unit/integration (5 + 7) tests passed, as did formatting, diff checks,
focused checks, required binary builds, and Linux world doctor. The workspace-wide clippy command
remains non-green at the same inherited warning/error set reproduced against untouched exact
baseline `bbd5c76626ccb65240a3759ce2b8645764d957b2`. The workspace test's first substantive compile
failure is the same inherited macOS lifecycle boundary reproduced there; after that failure, the
candidate run also encountered secondary ENOSPC linker failures that were not differentially
reproduced and are recorded only as an environmental limitation. None of these failures is E1
proof or an E1 waiver.

The E1-scoped carrier/resolver clause of `RG-POLICY-01` and the E1-scoped
containment/enforcement clause of `RG-POLICY-02` are complete. Their later receipt/manifest,
per-operation mediation, and host-visibility/synchronization clauses remain with E2, D2, and E4;
the ledger-wide gates are not globally closed. `RG-UAA-02` and `RG-POLICY-03` remain open under
their existing owners, `RG-BASE-03` remains open under C2, and `RG-DIFF-01` is preserved. E1 is
terminally complete. E2 is only eligible for fresh admission and explicit dispatch; this closure
does not admit, dispatch, implement, or complete it.
