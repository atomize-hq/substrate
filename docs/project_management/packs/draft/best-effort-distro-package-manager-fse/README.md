# Best-Effort Distro Package Manager - Seam Extraction

Source: `docs/project_management/packs/draft/best-effort-distro-package-manager/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for ADR-0031 (Detecting Badger). It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-01` (Distro Detection and Mapping)
- Next seam: `SEAM-02` (Override Precedence and Fallback)

Policy:

- Only the active seam is eligible for authoritative downstream sub-slices by default
- The next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- Active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- Future seams remain seam briefs
- `SEAM-03` and `SEAM-04` (future seams) stay at seam-brief depth only and will receive provisional deeper planning when promoted
