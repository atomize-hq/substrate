---
pack_id: opt_in_world_netfilter_enforcement
pack_version: v1
pack_status: extracted
source_ref: user_scope_writeup_2026-03-25
execution_horizon:
  active_seam: SEAM-3
  next_seam: SEAM-1
---

# Scope Brief - Opt-in World Netfilter Enforcement (Config Gate + Snapshot V3 Plumbing)

- **Goal**: Add opt-in, fail-closed outbound egress enforcement at the world boundary (Linux + macOS via Lima guest) using nftables/netfilter, driven by policy `net_allowed` and gated by host config + world-agent service env.
- **Why now**: Current behavior can represent restrictive `net_allowed` but does not reliably enforce deny-all/allowlist at runtime; this creates “policy says deny-all but ping works” risk and undermines safety guarantees for isolated execution.
- **Primary user(s) + JTBD**:
  - Operator: “Enable real outbound egress restrictions for worlds on machines that support nftables, without breaking existing workspaces by default.”
  - Policy author: “Express allowed outbound domains in policy and have it enforced at execution time.”
- **In-scope**:
  - Config-side opt-in lever `world.net.filter: bool` (default `false`) and related overrides/exports.
  - Policy Snapshot V3 plumbing for `net_allowed` (canonicalized list) from host to world-agent.
  - world-agent uses snapshot `net_allowed` and routes to world backend with `WorldSpec.isolate_network` and `allowed_domains`.
  - `crates/world` netfilter enforcement becomes real + fail-closed when requested (deny-all denies DNS; resolution failures are errors; execution attaches to cgroup or fails).
  - Observability: doctor output includes a netfilter status block sufficient to debug “requested vs enabled vs failed”.
- **Out-of-scope**:
  - True wildcard domain enforcement beyond `"*"` (reject other wildcards when enforcement is enabled).
  - Inbound filtering, per-port rules, URL/path matching, or protocol-aware policy.
  - Windows/WSL enforcement (explicitly non-goal for this pack).
  - Reworking the policy broker’s semantics for `net_allowed` (policy remains authoritative for *what* is allowed; config decides *whether enforcement is permitted*).
- **Success criteria**:
  - When all gates align (`world.net.filter=true`, `WORLD_NETFILTER_ENABLE=1`, and policy `net_allowed` not `["*"]`), outbound egress is enforced and fail-closed if enforcement cannot be applied.
  - Back-compat preserved: default `world.net.filter=false` means existing restrictive policies do not unexpectedly break networking.
  - Operator-visible diagnostics exist to explain whether enforcement is requested, enabled, and what failed.
- **Constraints**:
  - Fail-closed posture for requested filtering: no warn-and-run-unfiltered.
  - No in-guest broker state: world-agent enforcement is driven entirely by Snapshot V3 + request spec.
  - cgroup-scoped rules must apply to every spawned process; any non-attached path must fail when isolation is requested.
- **External systems / dependencies**:
  - nftables in the world environment (Linux host world, Lima guest world).
  - systemd unit environment / installer plumbing for `WORLD_NETFILTER_ENABLE=1` in world-agent service env.
  - DNS resolution availability during setup (fail-closed if required names cannot resolve).
- **Known unknowns / risks**:
  - Exact domain normalization rules (casefolding, IDNA/punycode behavior) required to avoid false allows/denies.
  - macOS Lima guest operational ergonomics (where to set `WORLD_NETFILTER_ENABLE` and how to surface errors).
  - Ensuring all execution paths attach to the cgroup under all modes (PTY, non-PTY, “direct exec”, helper binaries).
- **Assumptions**:
  - `net_allowed: []` means deny-all egress (including DNS) when enforcement is active.
  - `net_allowed: ["*"]` means allow-all egress (no filtering requested).
  - Non-`"*"` wildcards (e.g. `"*.example.com"`) are rejected with a clear diagnostic when enforcement is enabled.
