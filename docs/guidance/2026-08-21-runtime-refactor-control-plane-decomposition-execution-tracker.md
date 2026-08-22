# Runtime-refactor control-plane decomposition execution tracker

- Date created: 2026-08-21
- Status: **D0 baseline complete; D1 and later source-document decomposition slices remain blocked pending explicit later authorization**
- Canonical roadmap record: [ChatGPT Pro roadmap](2026-08-21-runtime-refactor-control-plane-decomposition-chatgpt-pro-roadmap.md)
- Provenance correction: [ChatGPT Pro corrigendum](2026-08-21-runtime-refactor-bundle-provenance-correction-chatgpt-pro.md)
- Reviewed input artifact: `/Users/spensermcconnell/Downloads/substrate-runtime-refactor-control-plane-6ab6032.zip`
- ZIP SHA-256: `34d7c3c4f40867af7d71963a310c809eaf84e87ec77bcdd108b3ec133b77454e`

> This tracker is an execution aid, not control-plane authority.  It does not select
> runtime work, alter packet/gate status, or authorize source-document edits.  Each
> slice requires a fresh admission against the then-live repository state and explicit
> user approval before a patch is applied.

## Provenance posture

The 148 included `repo/llm-last-mile/runtime-refactor/` files in the reviewed ZIP
were verified byte-identical to commit
`6ab6032fe6f0f917be68982adba416d2ded707e3`.  The ZIP's
`BUNDLE_PROVENANCE.md` declares that commit, the source root, and the source branch
`origin/feat/a1-3-public-adoption-doc-correction`.  The manifest value
`head_sha=7fdacfd12590baec3c258e909b65caadee4212f8` is retained as observed,
unclassified, non-authoritative metadata.  The ZIP and its contents are unchanged.

## Operating rules

1. Preserve text, authority, history, links, anchors, and path-frozen records before
   any later minimization.  The roadmap is a decomposition plan, not minimization
   authority.
2. Start every slice with a fresh GPT Pro session and a bounded input.  For a proposed
   change, use the ChatGPT Pro change-exchange workflow to obtain one text-only unified
   diff; locally preflight it, and apply only after action-time user confirmation.
3. Use a fresh, independently scoped GPT Pro review after a slice is landed.  A review
   is evidence, not authority to start the successor.
4. Keep one canonical owner per concept.  Root `00`--`05` retain compatibility anchors
   until D12; `06-review-finding-inventory.md` and `review-control/` remain path-stable
   where the roadmap says they do.
5. Do not modify the ZIP, its manifest, `BUNDLE_PROVENANCE.md`, runtime behavior,
   platform lanes, packets, gates, or scheduling merely to execute this tracker.

## Slice queue

| ID | Roadmap slice | Status | Entry / completion boundary | GPT Pro work item |
|---|---|---|---|---|
| D0 | Freeze read-only baseline | **Complete** | Bind live commit and tree; record file hashes, links/anchors, heading tree, authority/supersession/status map, frozen subjects, and selected source span. Complete before D1. | Create the bounded baseline/admission plan; do not edit source docs. |
| D1 | Semantic-status pilot and minimum migration ledger | **Blocked pending explicit later authorization** | Only the roadmap's three Markdown paths; preserve exact semantic-status span and legacy anchor; prove hash/link/authority transfer and rollback. | Produce one bounded D1 patch proposal and an independent preservation review. |
| D2 | Stable foundations | **Blocked by D1** | One self-contained foundation section per landing; exclude scheduling prose and preserve one owner per definition. | One separate bounded exchange per chosen foundation section. |
| D3 | Review and remediation governance | **Blocked by D1** | Move only the specified governance material; keep `06` and `review-control/` unchanged. | One bounded governance extraction and review. |
| D4 | Current-decision consolidation | **Blocked by D2 and D3** | One atomic projection/index correction; all current decision, held-packet, macOS-lane, and deferred-work projections agree. | One atomic change exchange; no partial patch acceptance. |
| D5 | Completed R2-4 vertical pilot | **Blocked by D4** | Account for its architecture, seam, slice, gate, and evidence material while keeping review artifacts byte-stable. | One vertical-family exchange and review. |
| D6 | Packet and task families | **Blocked by D5** | One complete packet or inseparable packet family per landing, reverse active chronology; preserve exclusions and predecessor relations. | One exchange per activated family. |
| D7 | Shared target architecture and invariants | **Blocked by D6** | Preserve invariant numbering, authority map, and packet/history separation. | One invariant or inseparable invariant-family exchange. |
| D8 | Seam families and inventories | **Blocked by D7** | Preserve every table field, semantic status, A0's location, and non-promotions. | One seam-family exchange. |
| D9 | Slices, tracks, and tasks | **Blocked by D8** | Preserve dependency/status distinctions; current schedule remains a decision projection. | One slice/task-family exchange. |
| D10 | Contracts and gates | **Blocked by D9** | Preserve named versions, ordered fields, code blocks, negative requirements, exceptions, and old anchors. | One contract/gate unit or inseparable version-family exchange. |
| D11 | Evidence and regression records | **Blocked by D10** | Preserve chronology, limitations, issue IDs, opaque external links, and receipt boundary. | One evidence-family exchange. |
| D12 | Root compatibility-index cutover | **Blocked by D11** | Only after the extraction ledger covers every substantive `00`--`05` span and all canonical destinations/legacy anchors resolve. | One final all-index cutover exchange and review. |

## Parallelism policy

**No concurrent source-document landing is pre-authorized.**  The roadmap gives an
ordered dependency chain and many slices update shared compatibility indexes,
authority projections, and the extraction ledger.  Parallel commits would make
canonical ownership and rollback ambiguous.

Safe parallel work is limited to read-only preparation:

- independent GPT Pro source maps, bounded-patch proposals, or adversarial reviews
  for later **unstarted** slices;
- per-section candidate inventory for D2; and
- per-packet-family inventory for D6.

Those outputs remain advisory and must be discarded or regenerated if an earlier
landed slice changes their frozen baseline.  A future slice may be considered for
parallel landing only after a new, explicit admission proves disjoint source spans,
destination paths, link/index edits, extraction-ledger rows, authority projections,
and rollback units.  Until then, merge and validate one slice at a time.

## Per-slice closeout record

When a slice is activated, append a row below rather than changing its queued status
without evidence:

| Slice | Bound commit/tree | GPT Pro chat / bundle | Local verifier results | Review result | Commit | Push | Notes |
|---|---|---|---|---|---|---|---|
| D0 | `60f34a7064d21a6755308e94abf564af6a554a6e` / `31020fdd8b5311521a2a35d0c10a39c99a2843d1` | Chat: https://chatgpt.com/c/6a890462-cf44-83ea-9faf-bbcb0891e21e ; ZIP `34d7c3c4f40867af7d71963a310c809eaf84e87ec77bcdd108b3ec133b77454e` | Inventory/headings/links/D1 span and `6ab6032fe6f0f917be68982adba416d2ded707e3` comparison revalidated; guidance links clean; `git diff --check`; GitNexus staged docs-only check | **KEEP** (advisory only; no D1 authorization) | this landing commit | `origin/feat/runtime-refactor-decomposition-d0-baseline` | Changed paths limited to the D0 guidance artifacts plus this tracker; source root and ZIP unchanged; post-landing clean-state verified locally; D1 remains blocked pending explicit later authorization. |
