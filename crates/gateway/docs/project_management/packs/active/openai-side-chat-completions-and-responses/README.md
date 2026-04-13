# OpenAI-Side Chat Completions and Responses - seam extraction

Source: `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`

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
- landed seams remain authoritative basis outside the forward window
- `SEAM-3` has now landed, published `THR-13`, and recorded closeout-backed `C-13` truth for future OpenAI-side maintenance work
