# make-doctor-health-output-explain-why - seam extraction

Source: `make-doctor-health-output-explain-why.zip :: docs/project_management/packs/draft/make-doctor-health-output-explain-why/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It is intentionally one level above seam-local decomposition.

It preserves the source pack's deep research and exact contract detail while normalizing the output to feature-seam-extractor-v2-3 posture. Current control-plane artifacts also record the post-landed resequencing that lets `SEAM-2` advance while `SEAM-1` waits on manual parity proof closeout.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-2`
- Previous active seam with remaining closeout blocker: `SEAM-1` (`REM-001`; manual macOS/Windows doctor parity proof only)
- Next seam: none

Source-pack mapping:

- `DHO0` -> `SEAM-1` doctor text disable attribution
- `DHO1` -> `SEAM-2` JSON + health disable attribution

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- `SEAM-1` no longer blocks `SEAM-2` horizon advancement because `governance/seam-1-closeout.md` already publishes landed `C-01`/`C-02`; if later native proof changes that truth, `SEAM-2` must revalidate before landing
- the resequenced active seam must still terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs
