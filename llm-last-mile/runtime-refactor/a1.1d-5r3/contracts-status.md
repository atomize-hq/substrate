**Kind:** status appendix projection
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 implementation status appendix from 04
**Status:** canonical
**Authority scope:** exact extracted 04 status-append source body only
**Source span:** [`04-contracts-and-gates.md#r3-implementation-status-append`](../04-contracts-and-gates.md#r3-implementation-status-append) lines 10000–10043
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 implementation status appendix (04 projection)

## R3 implementation status append

This append preserves historical landed/status facts only. Its protected MAC, evidence, recovery,
retirement/finalizer, E03, and Windows gates are not current parity prerequisites and do not
authorize successor work.

### `A1.1d-5R3-MANIFEST`

`A1.1d-5R3-MANIFEST` now has a landed implementation for the non-destructive portions of
`R3-MANIFEST-01`: canonical manifest parsing, canonical bytes/digests, manifest/index/head/shared
claim publication helpers, receipt and pairing/ticket verification helpers, hidden direct-
interactive control entrypoints, and the separate native-evidence artifact validator required by
later evidence tasks and closeout. Its provider channels remain preserving `provider_unavailable`
stubs until LINUX/MAC/WIN replace them, and the closed terminal status mapping above remains
unchanged. Successor authority remains `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

### `A1.1d-5R3-LINUX`

`A1.1d-5R3-LINUX` now has a landed implementation for the Linux portions of the managed-system
lifecycle contract: the exact `substrate-lifecycle-linux` executor and attested sudo relay,
protected/disposable publisher prepared-transition and receipt publication paths, fixed
`substrate-lifecycle-publisher-v1` service/socket units, and the exact pre-state snapshot/restore
handoff used by `world-provision.sh`. The packet preserves the closed product wall for Unix
orchestrator bodies, native evidence, and other platforms, and leaves successor authority at
`EVIDENCE:R3-LINUX-IMP-01`.

### `A1.1d-5R3-LINUX-CLOSEOUT`

`A1.1d-5R3-LINUX-CLOSEOUT` now has a landed docs/evidence closeout for the published Linux
provider checkpoint. It materializes the validated external evidence artifact and the
`codex.top-level-evidence-receipt.v1` under `review-control`, revalidates the artifact via
`validate_r3_native_evidence.py` with exact evidence/source/gated-successor joins, revalidates the
receipt with the orchestration skill validator, and records the bounded closeout review set. The
packet changes no production or test bytes, performs no Linux repair or MAC dispatch, and for this
authoritative orchestration the terminal successor is `COMPLETE`.

### `A1.1d-5R3-MAC`

The MAC implementation request is closed over the selected prefix, install bootstrap carrier,
PlatformBootstrapMappingV1, and ExecutorBuildEvidenceV1. The implementation package models and
validates those joins with non-executing fixtures; native build, Keychain/XPC/Lima lifecycle
exercise, operator-TTY pairing, receipt collection, and restoration evidence are deferred without
substitution to `EVIDENCE:R3-MAC-IMP-01`.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#r3-implementation-status-append`](../04-contracts-and-gates.md#r3-implementation-status-append), baseline lines 10000–10043
**Baseline span SHA-256:** `805efe6d2cc257d479fd2ee0c206428d1e71f7b4316c396fdc11e503ac8c77b8`
