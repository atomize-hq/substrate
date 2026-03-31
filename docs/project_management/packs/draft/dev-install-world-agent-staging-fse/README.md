# dev-install-world-agent-staging - seam extraction

Source: `dev-install-world-agent-staging.zip`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

This extraction is workflow-first and infers the horizon from the source pack’s accepted slice order and checkpoint shape:

1. stabilize the standard version-dir preflight and deterministic missing-artifact failure,
2. then stage `world-agent` during Linux dev-install `--no-world` using that shared path rule,
3. then lock the full feature with cross-platform proof, checkpoint evidence, and drift guards.

The source pack’s deep planning detail is preserved here as seam-level contracts, threads, verification paths, stale triggers, overlap revalidation rules, and governance scaffolds rather than as slice-level triads.
