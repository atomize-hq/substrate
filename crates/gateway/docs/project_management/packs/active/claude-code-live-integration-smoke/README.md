# Claude Code Live Integration Smoke - seam extraction

Source: `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway`, `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport`, `docs/foundation/*.md`, `docs/adr/0001-0007`, and current Claude Code/gateway operator anchors under `gateway/`

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
- active seams must terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- landed seams remain authoritative basis outside the forward window

Practical operator question this pack is intended to answer:

- what exact work remains so a real operator can configure Claude Code and this gateway, run live Azure-hosted `Kimi-K2-Thinking` and `Kimi-K2.5` sessions, verify routing and tool-loop behavior, and classify failures without reverse-engineering the repo

Extractor posture:

- this is a follow-on pack that consumes closeout-backed truth from the gateway and Azure transport packs rather than reopening those architecture decisions
- `C-07` and `C-08` remain upstream basis; this pack extends them from gateway-backed `/v1/messages` verification into the real Claude Code operator path
- `SEAM-3` has now landed, published `THR-10`, and recorded closeout-backed troubleshooting-boundary truth for downstream support work
- the bootstrap and live-smoke seams are now landed basis rather than forward-window work
- the landed bootstrap seam remains authoritative basis for setup order, evidence hooks, and boundary language
