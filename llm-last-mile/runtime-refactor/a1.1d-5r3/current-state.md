**Kind:** current-state projection
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** archived R3 planning/current-state projection bundle
**Status:** canonical
**Authority scope:** exact extracted 00 current-state source body only
**Source span:** [`00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling`](../00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling) lines 1023–1114
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 archived planning/current-state projection

## A1.1d-5R3-PLAN authoritative planning status (archived for active scheduling)

`A1.1d-5R3-PLAN` froze an implementation-ready decomposition for lifecycle cleanup and
convergence. It is archived engineering evidence and its former active-scheduling replacement is
now superseded by
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). It is
not implementation or native evidence. Planning was complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
terminal planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
`R3` implementation is `PARKED_BY_USER`, no R3 implementation task has been dispatched from this
plan, and the index below is preserved intent rather than the current next implementation line. The
exact architecture is in
[01-target-architecture.md](../01-target-architecture.md), row ownership in
[02-seam-crosswalk.md](../02-seam-crosswalk.md), dispatch-ready packet contracts in
[03-phase-slice-map.md](../03-phase-slice-map.md), normative gates in
[04-contracts-and-gates.md](../04-contracts-and-gates.md), and planned proof in
[05-debug-regression-ledger.md](../05-debug-regression-ledger.md).

The preserved historical implementation index was:

1. `A1.1d-5R3-HOME` — descriptor-bound synchronous private-home candidate rollback;
2. `A1.1d-5R3-MANIFEST` — non-destructive managed-artifact contract and durable manifest core;
3. `A1.1d-5R3-LINUX` — publish Linux privileged/system restoration implementation;
4. `EVIDENCE:R3-LINUX-IMP-01` then `A1.1d-5R3-LINUX-CLOSEOUT` — prove the exact published Linux
   checkpoint natively, then land evidence/control bytes only;
5. `A1.1d-5R3-MAC` — publish macOS/Lima and PM-bound SSH-UDS lifecycle implementation;
6. `EVIDENCE:R3-MAC-IMP-01` then `A1.1d-5R3-MAC-CLOSEOUT` — prove and close the exact published
   macOS checkpoint;
7. `A1.1d-5R3-WIN` — publish Windows prefix/shared/WSL/forwarder lifecycle implementation;
8. `EVIDENCE:R3-WIN-IMP-01` then `A1.1d-5R3-WIN-CLOSEOUT` — prove and close the exact published
   Windows checkpoint;
9. `A1.1d-5R3-UNIX` — Unix prefix/shim/payload/profile convergence, executor distribution, and
   integration of the already-landed platform providers;
10. `EVIDENCE:R3-NATIVE-LINUX-01`, `EVIDENCE:R3-NATIVE-MAC-01`, and
    `EVIDENCE:R3-NATIVE-WIN-01` — independent native proofs of the same published UNIX checkpoint;
    and
11. `A1.1d-5R3-CLOSEOUT` — cross-platform evidence ingestion and gate closeout with no product
    behavior change.

After a later revalidated resume, `HOME` and `MANIFEST` may be separately authorized from this
index; every destructive packet depends on `MANIFEST`. `LINUX` and `MAC` precede `UNIX` so the
three historically bundled rows PI-012, PI-026, and PI-095 have one row owner and disjoint
provider fences rather than shared mutation ownership. `WIN` remains mutation-disjoint but follows
MAC closeout in the publication chain. Each implementation is published before its native evidence
task, so every evidence receipt binds an exact remote-equal commit/tree/ref.
Each platform closeout depends on its clean evidence receipt, and final `CLOSEOUT` depends on all
three final native evidence tasks at the same published UNIX checkpoint. No task is pre-created by
this document.

The index is deliberately fail-closed around two discovered source facts. Windows PM already
requires an exact registered distro machine ID, so R3 activates only an existing PM-bound WSL
instance and forbids import, install-tree deletion, and unregister. The macOS packet tombstones the
current in-guest build/DNS/toolchain fallback and requires exact native-evidence-built artifacts;
its group, membership,
private-home, unit/service, layout, known-hosts, staging, socket, and instance effects are separate
manifest roles. Publisher bootstrap is a direct-interactive, OS-attested, component-durable
transition into fixed protected service paths; product uninstall retains its anchor. Disposable
native bootstrap proof hashes the null-retirement-slot bootstrap core, then precommits the harness
key, external-store descriptor identity, and exact core-bound test-retirement authorization digest
into the final bootstrap and generation-one anchor before any publisher component exists or can be
torn down. Retirement receipt bytes are externally hashed only after fsync and authorized by a
separate signed harness acknowledgement; neither record contains its own or a future digest.
Lima/WSL guest bootstrap uses a protected-host-publisher-signed one-use pairing ticket;
the ticket carries canonical signer SPKI bytes, and the operator must pin their full hash/challenge
from the host terminal at the independent guest TTY before the guest verifies the fixed P-256
signature or atomically links the root-only external intent. That intent durably confines the
guest seed/nonce before the publisher directory exists; the final inactive key is materialized
only after the host transcript. Completed state records exact-join, while an effect-visible/
identity-record-not-durable component-creation gap is an explicit preserving stop rather than
path/byte adoption. Evidence-only guest retirement is separately null-slot-precommitted in the
host-signed ticket, copied through both pairing records and generation one, externally receipts
the exhaustive guest DAG, removes guest residue before the retained host pairing record, and only
then permits the exhaustive host retirement and parity receipt. The retained
`limactl shell`/`wsl -d` channel carries only the post-pinning hello/transcript and is not authority.
Provider evidence tasks natively build both host and isolated Linux
guest executors from the exact remote-equal checkpoint under `ExecutorBuildEvidenceV1`, clean the
build scope before baseline, and supply those exact hashes to bootstrap. Evidence receipts validate
source and artifact digest; the separately
validated evidence artifact, not the skill receipt schema, binds each gated successor.

The planning subject is exactly these six Markdown files. Review control is recorded in
[r3-planning-review-cycle-record.json](../review-control/r3-planning-review-cycle-record.json) and
the three linked R3 planning review reports. `06-review-finding-inventory.md` is unchanged because
repository truth contains no current R3-owned P3/P4 item. Publication of this plan cannot
authorize any product edit, native lifecycle action, provisioning, or successor dispatch.

Explicit exclusions remain passive health/world-deps remediation, authenticated Codex execution,
retained workers/tasks, authoritative-session repair, orchestrator packet-3 lifecycle/routing,
gateway adoption beyond exact managed cleanup, direct-member architecture, unrelated
runtime-refactor work, policy/capability redesign, a new shared state root, or reinterpretation of
the bounded `0640`/`0650` cache observation. R1/R2 evidence is immutable.
**Source provenance:** extracted from [`../00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling`](../00-README.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling), baseline lines 1023–1114
**Relocation note:** repository-relative Markdown targets were rebased as needed to preserve their original repository destinations after relocation.
**Baseline span SHA-256:** `4a994a7352e5f9d6b66cdfd05320119ac972a55951d71510b8d188bc516f200e`
