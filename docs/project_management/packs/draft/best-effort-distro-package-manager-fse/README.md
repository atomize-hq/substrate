# Best-Effort Distro Package Manager - Seam Extraction

Source: `docs/project_management/packs/draft/best-effort-distro-package-manager/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for ADR-0031. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-04` - fallback probe failure taxonomy
- Next seam: `SEAM-05` - wrapper doc propagation

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

Extraction assumptions:

- the source planning pack is treated as approved input for extraction, including the previously human-reviewed planning-gate blocker
- this extraction expands the original four slice-oriented units into seven seam briefs because the source pack contains more independent contracts than the slice count alone exposes
- downstream seam planning must preserve the source pack's single checkpoint boundary at `BEDPM3` semantics even though the conformance work is split across `SEAM-06` and `SEAM-07`
- macOS-hosted installs that route through the Lima-backed Linux backend are treated as required behavior-coverage surfaces for validation and checkpoint evidence, even though the package-manager decision logic itself remains Linux-scoped
