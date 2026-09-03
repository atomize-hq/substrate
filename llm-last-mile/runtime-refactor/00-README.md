# Runtime Refactor Control Pack

**Kind:** entrypoint
**Stable ID:** `runtime-refactor-control-pack-entrypoint`
**Status:** canonical stable human entrypoint
**Canonical for:** runtime-refactor selective-load entrypoint and final root compatibility map

> **Entry-point boundary:** Start here to choose the minimum relevant root, navigation index, and canonical-owner set. Indexes, projections, and compatibility roots grant no implementation, gate, promotion, remediation, dispatch, or successor authority; follow each linked canonical owner for exact scope and status.

**Legacy title/status context:** before D12 this root was labeled “canonical control pack for future runtime-refactor slices”; D12 retains the title and legacy anchors for compatibility, not that former broad authority claim
**Entrypoint scope:** locating and selectively loading planning, contract, sequencing, and proof-gate owners; this root does not own their substance, authorize action, or serve as implementation history
**Source directive:** [`../../substrate-runtime-refactor-directive-revised.md`](../../substrate-runtime-refactor-directive-revised.md)
**Repo-truth snapshot:** 2026-08-02 at
`4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e` / tree
`8ed5dc7a354b731016a103b68091864b6a09223a`; re-check live code before every slice.
**Current scheduling projection:** [`index/current.md`](index/current.md) is the visibly non-authoritative current-state projection for terminally complete A1.3-P1, A1.4/A1, A2, A3, Track A, E1, and E2; the completed documentation-only but specified/unadmitted E2-RM authority correction; the controlling documentation-only, unadmitted E3 specification; separate future-admission B2.2/E3 implementation candidates, with B2.2 blocked pending `E2-RM` completion and fresh re-admission; held A1.3-P0/A1.3 records; the separate macOS lane; and deferred Windows work. Canonical decisions, packets, and gates remain at their linked path-stable owners.
**Navigation indexes:** [`index/README.md`](index/README.md) by stable title/owner, [`index/by-id.md`](index/by-id.md), [`index/by-kind.md`](index/by-kind.md), and [`index/by-packet.md`](index/by-packet.md).
**Historical scheduling state (superseded for active scheduling on 2026-08-19):**
Canonical historical content: [`history/cross-cutting-control-pack-checkpoints.md#historical-scheduling-state-superseded-for-active-scheduling-on-2026-08-19`](history/cross-cutting-control-pack-checkpoints.md#historical-scheduling-state-superseded-for-active-scheduling-on-2026-08-19). Use [`index/current.md`](index/current.md) and its linked owners—not this historical record—for active, held, lane-local, and deferred state.


## Canonical repo location

This pack's canonical location is:

```text
llm-last-mile/runtime-refactor/
```

The `../../...` links to repo-root directives and debug memos assume that placement. If this directory moves, update every affected relative link in the same PR.

## Purpose

This directory compresses the runtime-refactor directive into a selective-load control surface. It exists to prevent a recurring failure mode:

> An artifact in the tree is not evidence that its architecture seam has landed.

A seam is landed only when **all four** conditions are true:

1. the correct authority boundary owns the decision;
2. the real production call path routes through that boundary;
3. the intended policy is enforced at that boundary; and
4. smoke/e2e/regression proof exercises that exact path.

Unit tests, type names, persisted rows, helper functions, process liveness, socket reachability, and successful launch are useful evidence. None is sufficient by itself.

## Control-pack map

| File | Load when | Content or navigation role |
|---|---|---|
| [`01-target-architecture.md`](01-target-architecture.md) | resolving a legacy architecture heading or anchor | typed non-authoritative compatibility index; follow its canonical links |
| [`02-seam-crosswalk.md`](02-seam-crosswalk.md) | resolving a legacy seam heading, anchor, or table projection | typed non-authoritative compatibility index; A0 remains location-locked |
| [`03-phase-slice-map.md`](03-phase-slice-map.md) | resolving a legacy track, slice, or task heading | typed non-authoritative compatibility index; follow canonical slice/task owners |
| [`04-contracts-and-gates.md`](04-contracts-and-gates.md) | resolving a legacy contract or gate heading | typed non-authoritative compatibility index; follow canonical contract/gate owners |
| [`contracts/development-review-and-remediation-contract.md`](contracts/development-review-and-remediation-contract.md) | implementing, reviewing, remediating, or closing a packet | development review and remediation contract |
| [`gates/authority-required-macos-dev-parity.md`](gates/authority-required-macos-dev-parity.md) | evaluating or activating the macOS developer-parity lane | canonical `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` gate |
| [`gates/authority-required-runtime-refactor-reentry.md`](gates/authority-required-runtime-refactor-reentry.md) | interpreting the closed global reentry selection | canonical `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` gate |
| [`05-debug-regression-ledger.md`](05-debug-regression-ledger.md) | resolving a legacy evidence, issue, regression, or smoke heading | typed non-authoritative compatibility index; follow canonical evidence owners |
| [`index/README.md`](index/README.md) | resolving stable titles and canonical owners | title-and-owner navigation only |
| [`index/current.md`](index/current.md) | checking the current global packet, held packets, lane-local gate, or deferred work | visibly non-authoritative current-state projection |
| [`index/by-id.md`](index/by-id.md) | resolving stable IDs, packet/task IDs, seam names, contract versions, gate IDs, issue IDs, or review IDs | identifier navigation only |
| [`index/by-kind.md`](index/by-kind.md) | browsing every canonical destination by document kind | exhaustive document-kind navigation only |
| [`index/by-packet.md`](index/by-packet.md) | assembling a packet/family selective-load set | packet and packet-family navigation only |
| [`history/cross-cutting-control-pack-checkpoints.md`](history/cross-cutting-control-pack-checkpoints.md) | reading indivisible cross-packet root-00 checkpoint chronology | canonical historical record only; never current scheduling authority |
| [`history/authenticated-runtime-projections.md`](history/authenticated-runtime-projections.md) | reading the deferred R2-2E/F0a/F0b architecture projection block | canonical historical packet/architecture record only |
| [`06-review-finding-inventory.md`](06-review-finding-inventory.md) | classifying, retaining, deduplicating, or resolving non-blocking review findings | the single `P3`/`P4` review and process-debt inventory; never a `P1`/`P2` waiver |
| [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md) | classifying a seam or reviewing a promotion claim | semantic status labels and the promotion rule |
| [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md) | interpreting authority terms or reviewing a control-plane boundary | authority vocabulary |
| [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md) | reading or updating the control pack | reading and update rules |
| [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md) | assembling implementation or review context for a slice | per-slice context assembly protocol |
| [`foundations/normative-conventions.md`](foundations/normative-conventions.md) | interpreting record, identity, revision, timestamp, commitment, or ref conventions | normative conventions |
| [`review-control/`](review-control/) | opening, closing, or extending a review cycle | small standard-library cycle record, validator, example, and focused tests |

Do not load the full historical design/debug stack by default. Start with the applicable crosswalk
row, slice row, contract section, regression row, and review contract. Follow only the named
must-read links. Slice A0's authority-leak inventory remains inside `02-seam-crosswalk.md`; `06` is
only the cross-slice non-blocking review-finding inventory and does not absorb A0 authority truth.

## Per-slice context assembly protocol

Canonical content is maintained in [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md).

## Current gateway carrier correction

Keep this split explicit in every D1, D3, or E3 context capsule:

- The managed in-world gateway auth carrier is a landed positive primitive: `world-service` creates a validated `GatewayAuthBundleV1` pipe handoff, launches `substrate-gateway` with `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, scrubs raw secret env vars, and the gateway consumes and validates the bundle once.
- Direct world Codex/member execution still uses the isolated seed-home compatibility bridge. It is not yet consistently pointed at the managed gateway with a per-worker, Substrate-owned `CODEX_HOME`/`config.toml` projection derived from accepted policy and logical config.
- Therefore, do not rebuild or describe the secure-FD carrier as missing. Preserve it under `RG-CONFIG-03`. The unresolved adoption/projection seam is `RG-CONFIG-04`, and the complete world-Codex path remains below `ContractCorrectAndProven` until production-path smoke/e2e closes that gate.

R2-2E did not change that split. The managed gateway secure-FD path is landed, regression-proven,
and unchanged by R2-2E. Direct-member Codex/UAA gateway adoption remains unresolved transitional
compatibility, is non-promotable, and stays owned by E3/D1/D3. `RG-CONFIG-02`, `RG-CONFIG-04`,
`RG-UAA-02`, and `RG-UAA-03` remain open.

The controlling E3 documentation authority is now
[`slices/e3-agent-config-projection-and-gateway-adoption.md`](slices/e3-agent-config-projection-and-gateway-adoption.md),
with [`AgentConfigProjectionRecordV1`](contracts/agent-config-projection-v1.md) and
[`Managed gateway adoption V1`](contracts/managed-gateway-adoption-v1.md). It preserves the landed
carrier, corrects the absent local `crates/codex`, specifies strict V2 before D1's later V3, and
marks no gate green. E3 is not admitted, dispatched, or implemented.

## Semantic status labels

Canonical authority for these labels and their promotion rule is now in [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md).

## Authority vocabulary

Canonical authority for this vocabulary is now in [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md).

## Reading and update rules

Canonical authority for these rules is now in [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md).

## Current control conclusion

The canonical B1/B2.1 control conclusion moved to [`b1-b2-1/current-state.md#current-b1b21-control-conclusion`](b1-b2-1/current-state.md#current-b1b21-control-conclusion).

The canonical A1.2a/A1.2a-WB/A1.2a-S completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12aa12a-wba12a-s-completion-projection`](a1-2-earlier-histories/current-state.md#a12aa12a-wba12a-s-completion-projection).

The canonical B1/B2.1 completion projection moved to [`b1-b2-1/current-state.md#b1b21-completion-projection`](b1-b2-1/current-state.md#b1b21-completion-projection).

The canonical mixed B3.1/C1 predecessor and A1.2b completion projection moved to
[`a1-2-earlier-histories/current-state.md#a12b-mixed-predecessor-completion-projection`](a1-2-earlier-histories/current-state.md#a12b-mixed-predecessor-completion-projection).

The cross-cutting historical checkpoint body moved to [`history/cross-cutting-control-pack-checkpoints.md#current-control-conclusion`](history/cross-cutting-control-pack-checkpoints.md#current-control-conclusion). This root section and the existing family-local pointers are compatibility routes only; current state remains in [`index/current.md`](index/current.md) and its linked owners.

The canonical B1/B2.1 retained-target and dispatch-prerequisite projection moved to [`b1-b2-1/current-state.md#retained-target-and-dispatch-prerequisite-projection`](b1-b2-1/current-state.md#retained-target-and-dispatch-prerequisite-projection).


## A1.1d-5R2-2F0-HC shell-harness closure audit and environment correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction).

## A1.1d-5R2-2F0 historical differential authority correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-historical-differential-authority-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f0-historical-differential-authority-correction).

## F0/F0a/F0b/F0-HC canonical closeout

Canonical content: [`a1.1d-5r2-2f/current-state.md#f0f0af0bf0-hc-canonical-closeout`](a1.1d-5r2-2f/current-state.md#f0f0af0bf0-hc-canonical-closeout).

## A1.1d-5R2-2F historical readiness-boundary correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-historical-readiness-boundary-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-historical-readiness-boundary-correction).

## A1.1d-5R2-2F5-PD canonical correction

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f5-pd-canonical-correction`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f5-pd-canonical-correction).

## A1.1d-5R2-2F canonical closeout

Canonical content: [`a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-canonical-closeout`](a1.1d-5r2-2f/current-state.md#a11d-5r2-2f-canonical-closeout).
## A1.1d-5R2-2 renewed closeout publication authority (historical pre-RP4 checkpoint)

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-renewed-closeout-publication-authority-historical-pre-rp4-checkpoint`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-renewed-closeout-publication-authority-historical-pre-rp4-checkpoint).

## Canonical broad-wall invocation authority correction

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#canonical-broad-wall-invocation-authority-correction`](a1.1d-5r2-2-renewed-closeout/current-state.md#canonical-broad-wall-invocation-authority-correction).

## A1.1d-5R2-2 closeout-remediation planning authority (historical RP0 checkpoint)

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-closeout-remediation-planning-authority-historical-rp0-checkpoint`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-closeout-remediation-planning-authority-historical-rp0-checkpoint).

## A1.1d-5R2-2 RP3/RP4/RP5 closeout status

Canonical content: [`a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-rp3rp4rp5-closeout-status`](a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-rp3rp4rp5-closeout-status).
## A1.1d-5R2-3 closeout status

Canonical content: [`a1.1d-5r2-3/current-state.md#a11d-5r2-3-closeout-status`](a1.1d-5r2-3/current-state.md#a11d-5r2-3-closeout-status).
## A1.1d-5R3-PLAN authoritative planning status (archived for active scheduling)

Canonical content: [`a1.1d-5r3/current-state.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling`](a1.1d-5r3/current-state.md#a11d-5r3-plan-authoritative-planning-status-archived-for-active-scheduling).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/status-append.md#r3-implementation-status-append`](a1.1d-5r3/status-append.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-manifest`](a1.1d-5r3/status-append.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-linux`](a1.1d-5r3/status-append.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-linux-closeout`](a1.1d-5r3/status-append.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/status-append.md#a11d-5r3-mac`](a1.1d-5r3/status-append.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/current-state.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-status-2026-08-07`](r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current authority correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-current-authority-correction-2026-08-07`](r3-mac-evidence-recovery/current-state.md#aux-r3-mac-evidence-recovery-plan-current-authority-correction-2026-08-07).
## macOS developer-parity lane (2026-08-19; scoped)

Canonical content: [`macos-dev-parity/current-state.md#macos-developer-parity-lane-2026-08-19-scoped`](macos-dev-parity/current-state.md#macos-developer-parity-lane-2026-08-19-scoped).

## Linux-first runtime-refactor scheduling decision (2026-08-20; controlling)

> **Projection status:** non-authoritative current-state projection. Canonical owners are linked from [`index/current.md`](index/current.md); this legacy section grants no authority.

[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md)
supersedes the macOS decision's former global blocking order. The reentry gate has closed as a
live-source historical selection of
[A1.3](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), later narrowed by the held
[A1.3-P0](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md), and now
corrected so the active Linux-first implementation packet is
[A1.3-P1](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
The older A1.3 and A1.3-P0 records remain held as historical fences only. macOS parity may
proceed in its separate lane and Windows remains deferred. Neither lane's closure dispatches the
other's successor.
