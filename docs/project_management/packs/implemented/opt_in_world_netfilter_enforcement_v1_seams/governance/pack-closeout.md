# Pack Closeout - Opt-in World Netfilter Enforcement

- **Pack status**: landed and complete
- **Remaining open seams**: none
- **Open remediations still blocking pack closeout**: none
- **Threads still not closed**: none; `THR-01` through `THR-05` have landed closeout evidence and terminal revalidation in `SEAM-5`
- **Downstream stale triggers still requiring attention**: none inside this pack; future drift against the recorded seam stale triggers reopens follow-on work rather than leaving this pack active
- **Evidence summary**:
  - `SEAM-1` closeout records landed Snapshot V3 `net_allowed` canonicalization plus host-to-world routing handoff in `governance/seam-1-closeout.md`
  - `SEAM-2` closeout records fail-closed runtime enforcement, deny-all DNS behavior, and attach-or-fail execution invariants in `governance/seam-2-closeout.md`
  - `SEAM-3` closeout records the host opt-in config gate, override/export parity, and operator docs in `governance/seam-3-closeout.md`
  - `SEAM-4` closeout records the published doctor `netfilter_status` contract and shell/shim passthrough coverage in `governance/seam-4-closeout.md`
  - `SEAM-5` closeout records terminal conformance evidence across schema tests, host routing tests, doctor passthrough tests, privileged verification guidance, and macOS Lima smoke guidance in `governance/seam-5-closeout.md`

## Closeout Summary

This pack is complete. The planned host gate, snapshot routing, runtime enforcement, doctor observability, and conformance surfaces are all represented by landed code and seam-level closeout evidence.

The pack-level basis is:

- `C-01` through `C-07` are published by their owner seams and consumed downstream without remaining carry
- `THR-01` through `THR-05` were revalidated at the terminal `SEAM-5` boundary
- `governance/remediation-log.md` contains no open remediations

## Terminal Outcome

The repository now has one consistent three-way gate story for outbound egress enforcement:

- `world.net.filter` controls whether the host may request enforcement
- policy `net_allowed` controls the allow-all, deny-all, or restrictive posture
- `WORLD_NETFILTER_ENABLE=1` controls whether the world backend may honor a requested run

When those gates align for a restrictive posture, the world backend enforces outbound egress or fails closed; doctor and smoke surfaces expose the requested/enabled/failure state needed to debug drift.
