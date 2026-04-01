# world-disabled-reason-attribution - seam extraction

Source: `world-disabled-reason-attribution.zip::world-disabled-reason-attribution/`

This pack re-expresses the deep-researched ADR-0038 planning pack as a governance-ready seam pack for `feature-seam-extractor-v2-3`.
It preserves the source pack's contract detail, telemetry shape, redaction rules, platform parity expectations, and execution ordering; promoted seams may now carry seam-local decomposition while future seams remain seam briefs.

Restated scope and assumptions:

- Add effective world-disable attribution to replay operator output and replay telemetry using the same winning-layer semantics already locked by the source pack for ADR-0037 and ADR-0038.
- Keep replay-local opt-out fragments stable: `--no-world flag`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, and `--flip-world`.
- Keep replay selection precedence, backend selection, timeout behavior, and exit codes unchanged.
- Keep redaction strict: no absolute config paths and no raw env values outside fixed allowlisted tokens.
- Treat the source pack's `WDRA0 -> WDRA1 -> WDRA2` chain as the best available critical-path signal for seam extraction.

Start here:

- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- `governance/remediation-log.md`

Execution horizon:

- Active seam: `none`
- Next seam: `none`
- Previous active seam: `SEAM-3` (its closeout now records the terminal lock-in evidence for `C-02`, `C-03`, and `C-04`)

Horizon inference:

- `SEAM-3` has now left the forward planning window because its seam-exit gate passed and `governance/seam-3-closeout.md` records the terminal parity lock-in evidence.
- There is no remaining `next` seam inside this pack after `SEAM-3`; the pack is now in closeout posture rather than another promotion handoff.
- `SEAM-1` and `SEAM-2` remain historical upstream seams whose closeouts provide the authoritative basis for the landed final seam.

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- active and next seams must eventually terminate in a dedicated final `seam-exit-gate` slice once seam-local planning begins
- future seams remain seam briefs

Source-pack crosswalk:

- `slices/WDRA0/WDRA0-spec.md`, `decision_register.md` DR-0001, and the shared contract/redaction rules map chiefly to `SEAM-1`
- `slices/WDRA1/WDRA1-spec.md`, `contract.md`, `telemetry-spec.md`, and decision register DR-0002/DR-0003 map chiefly to `SEAM-2`
- `slices/WDRA2/WDRA2-spec.md`, `manual_testing_playbook.md`, `platform-parity-spec.md`, and `smoke/` map chiefly to `SEAM-3`
