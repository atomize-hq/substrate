# Azure Foundry Provider Transport - seam extraction

Source: `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway`, `docs/foundation/*.md`, `docs/adr/*.md`, and current gateway Azure transport anchors under `gateway/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for the remaining Azure Foundry runtime-provider work. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-2`
- Next seam: `null`

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- no additional seam is queued in the forward window until new follow-on work is extracted
- the active seam terminates in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

Practical operator question this pack is intended to answer:

- what exact work remains so an operator with real Azure Foundry credentials can configure this gateway, route think/planner traffic to `Kimi-K2-Thinking`, route default/execution traffic to `Kimi-K2.5`, and complete a live smoke run through the landed Anthropic-compatible gateway surface
