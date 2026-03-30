# Persist detected Linux distro + pkg manager - seam extraction

Source: `persist-detected-linux-distro-pkg-manager.zip`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It starts one level above seam-local decomposition, and promotion may add seam-local planning for the active seam.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

## Restated scope

Transform the deep-researched planning pack for Linux distro and package-manager metadata persistence into a governance-ready seam pack without carrying forward the old pack's slice files as authoritative execution units.

The extracted scope remains the same as the source pack:

- persist Linux distro identity and selected package-manager metadata into `install_state.json`
- keep the upstream detection contract authoritative for vocabulary and selection semantics
- keep metadata failures warning-only so they do not flip an otherwise successful install into failure
- keep behavior changes Linux-only while preserving compile and test parity expectations for macOS and Windows
- preserve `schema_version = 1`, existing `host_state.group`, existing `host_state.linger`, and unknown keys during rewrites

## Extraction assumptions

- The source pack's accepted slice order `PDLDPM0 -> PDLDPM1 -> PDLDPM2` is the best available critical-path signal, so it is used to infer the execution horizon in this seam pack.
- The source pack's planning workstreams are preserved as internal reasoning inputs, but they are rolled up into higher-level seams because extractor v2.3 must stay above slice decomposition.
- `CP1` remains an end-of-feature evidence checkpoint, but it is treated here as part of the conformance seam rather than as a standalone seam.
- `governance/seam-1-closeout.md` now records landed SEAM-1 reality.
- Remaining closeout documents in `governance/` stay scaffolds until their owning seams land.

Execution horizon:

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

## Source-plan lineage

The old deep-researched pack is preserved as lineage, not as the output structure:

- `PDLDPM0`, `contract.md`, `decision_register.md`, and `install-state-schema-spec.md` roll up into `SEAM-1`
- `PDLDPM1` rolls up into `SEAM-2`
- `PDLDPM2`, `docs/INSTALLATION.md` reconciliation, smoke coverage, `tasks.json`, `plan.md`, and checkpoint evidence roll up into `SEAM-3`

The result stays compliant with the seam extractor skill by removing authoritative slice files while retaining the depth, risks, cross-pack authority boundaries, and validation posture from the source pack.
