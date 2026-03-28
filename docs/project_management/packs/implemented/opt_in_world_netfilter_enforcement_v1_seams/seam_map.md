# Seam Map - Opt-in World Netfilter Enforcement

This seam map enumerates cohesive seams for delivering opt-in, fail-closed outbound egress enforcement at the world boundary.

## Seams

- `SEAM-3` (active, capability): Config opt-in `world.net.filter` + CLI set/reset + env override/export + docs.
- `SEAM-1` (next, integration): Snapshot V3 `net_allowed` contract + host→world-agent plumbing (remove in-guest broker state).
- `SEAM-2` (future, platform/risk): `crates/world` netfilter enforcement is fail-closed and applies to every executed process (cgroup attach invariants).
- `SEAM-4` (future, integration/observability): Doctor/diagnostics surface “requested vs enabled vs failed” netfilter status.
- `SEAM-5` (future, conformance): Verification surfaces (unit tests, privileged integration tests, macOS Lima smoke playbook).

## Non-goals (explicit)

- Implementing wildcard support beyond `"*"`.
- Adding per-port or per-protocol allow/deny (policy remains host/domain allowlist only).
- Windows/WSL netfilter support.
