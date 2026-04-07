# Execution Surface Parity Hardening - seam extraction

Source work items:

- `docs/project_management/intake/work_items/taming_tapir_work_item_intake.md`
- `docs/project_management/intake/work_items/taming_tapir_fact_finding.md`
- `docs/project_management/intake/work_items/aligning_otter_work_item_intake.md`
- `docs/project_management/intake/work_items/untangling_lemur_work_item_intake.md`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for a combined execution-surface hardening initiative. It intentionally stays one level above seam-local decomposition.

Scope restatement:

- normalize operator-visible execution semantics across replay routing, command-mode tracing validation, and interactive REPL terminal-loss handling
- keep the work host/shell-local unless implementation evidence later proves a deeper backend change is required
- avoid new public CLI or config surface by default

Planning assumptions used for extraction:

- primary seam axis is inferred as integration-first because the biggest risks are ambiguous contracts and cross-surface drift, not one isolated runtime module
- the execution horizon is inferred rather than user-specified, with exactly one `active` seam and one `next` seam
- the tracing clarification work may terminate in docs or decision-record updates instead of immediate runtime changes, but it still owns a publishable contract surface for downstream planning

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `none`
- Next seam: `none`
- Previous active seam: `SEAM-3` (its closeout now records the terminal cross-surface revalidation evidence for `THR-01` and `THR-02`)

Horizon inference:

- `SEAM-3` has now left the forward planning window because its seam-exit gate passed and `governance/seam-3-closeout.md` records the final conformance evidence.
- There is no remaining `next` seam inside this pack after `SEAM-3`; the pack is now in closeout posture rather than another promotion handoff.
- `SEAM-1` and `SEAM-2` remain historical upstream seams whose closeouts provide the authoritative basis for the landed final seam.

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- there is no remaining `next` seam in this pack after `SEAM-3`; any further work should come from a new pack or a reopened basis change
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contracts may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future seams remain seam briefs
