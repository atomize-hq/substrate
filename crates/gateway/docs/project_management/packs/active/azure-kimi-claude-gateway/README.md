# Azure Kimi Claude Gateway - seam extraction

Source: `IMPORTANT_SUBSTRATE_ALIGNMENT.md`, `docs/adr/0001-0007`, `/Users/spensermcconnell/__Active_Code/openClaw/.codex/handoffs/2026-03-27-144003-azure-kimi-claude-adapter.md`, `/Users/spensermcconnell/__Active_Code/openClaw/.codex/handoffs/2026-03-27-141151-ccr-kimi-routing-debug.md`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: none remaining in this pack
- Next seam: none remaining in this pack

Policy:

- no seam remains in the active forward window for this pack
- no later seam remains in this pack, so the forward window is closed until new pack work is introduced
- active seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- `SEAM-5` has now landed, published `THR-05`, and closed the pack's boundary-conformance horizon on closeout-backed `C-05` and `C-06` truth

Extractor posture:

- the ADRs and Substrate memo constrain the pack but do not map one-to-one to seams
- `SEAM-1` proved the actual `claude-code-mux` foundation and published the handoff `THR-01`
- `SEAM-2` has now landed and published `THR-02`, so `SEAM-3` is the active seam on closeout-backed `C-02` truth
- `SEAM-3` has now landed, published `THR-03`, and recorded closeout-backed `C-03` truth for downstream seams
- `SEAM-4` has now landed and published `THR-04`, so internal policy is closed out and the boundary seam can freeze on current upstream truth
- `SEAM-5` has now landed, published `THR-05`, and recorded closeout-backed external-boundary truth for later Substrate integration work
