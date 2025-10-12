# Phase 4.5 – Isolation Upgrade (Design Draft)

Status: Draft

Owner: Security/Replay Working Group

Last updated: 2025-09-10

## Problem Statement

Our current “world” isolation for replay uses best‑effort primitives in the host namespace (cgroups v2 and nftables per‑world tables). This is acceptable for privileged dev containers and gracefully degrades on constrained hosts, but it has limitations:

- Host‑wide network effects: per‑world nftables tables install hooks in the host stack during replay, even if briefly.
- Weak scoping: rules are not tied to a process boundary (netns/cgroup match), making isolation less predictable on busy systems.
- Partial namespace isolation: we do not consistently provide network isolation per replay by default.

We seek to elevate isolation to production‑grade, with well‑scoped, predictable effects while preserving current UX and graceful degradation.

## Goals

- Default per‑replay network namespace (netns) and apply network policy within that namespace.
- Scope nftables policy to the replayed process context (via netns and/or cgroup matching).
- Clear lifecycle: setup → execute → teardown; no lingering rules or namespaces.
- Preserve current contract: replay strategy lines unchanged; fs_diff intact; verbose warnings when features are unavailable.
- Maintain graceful degradation on non‑privileged hosts.

## Non‑Goals

- Full containerization (OCI) of replays.
- Cross‑platform parity for Windows/macOS in this phase (Windows handled in separate plan).
- Comprehensive SELinux/AppArmor policy authoring (document, do not implement here).

## Background and Current State

- Strategies: overlay → fuse-overlayfs → copy-diff → direct (unchanged).
- Cgroups v2: per‑world subtree creation/attach (best‑effort) implemented.
- Nftables: per‑world table with allow(loopback/established/DNS) + default LOG+drop; best‑effort teardown.
- Netns: not default; may be available as best‑effort extension (Phase B.4).

## Proposed Architecture

1) Network Namespace First

- Default: create a per‑replay named netns (e.g., `substrate-<world_id>`).
- Bring up loopback; optionally mount a minimal `/etc/resolv.conf` or rely on host’s default via /etc (read‑only).
- Apply nftables policy inside the netns only.
- Execute replayed command within the netns; teardown at end.

2) Nftables Scoping

- Preferred: rules exist only inside the per‑replay netns; never install host‑wide hooks by default.
- Optional enhancement: explore nftables cgroup v2 matching (nft “meta cgroup”) when running in host ns to further restrict policy to a process’ cgroup in environments lacking netns.
- Teardown: delete relevant tables/chains at exit; tolerate ENOENT/EBUSY.

3) Cgroups v2 Enhancements

- Ensure consistent attach of the replay child PID across all strategies (use spawn + pipes to capture PID).
- Consider cgroup‑based rate limits (pids/memory) as future switches (off by default in this phase).

4) Namespaces Roadmap

- Phase 4.5: Network namespace by default; PID namespace optional and off by default (documented).
- Future: Evaluate mount namespace hardening consistent with overlay/fuse usage.

## Privilege & Capability Detection

- Detect netns availability (unshare/capabilities, or iproute2 netns commands).
- Detect nft presence; report dmesg_restrict value.
- Degradation policy:
  - If netns unavailable → fall back to host‑ns policy with explicit warning, or fully skip nft with clear WARN.
  - Cgroups attach failures never abort replay; warn only.

## Observability & Logging

- Maintain existing `[replay] world strategy: …` lines.
- Add `[replay] info: netns=<name>` when netns is active.
- WARN when falling back or when LOGs may be hidden (dmesg_restrict=1).
- Continue unified trace JSONL; capture degraded components where helpful.

## Compatibility Matrix (high level)

| Env | Netns | Nftables | Cgroup v2 | Expected Behavior |
|-----|-------|----------|-----------|-------------------|
| Privileged dev container (Podman) | Yes | Yes | Yes | Netns + nft scoped, cgroup attach OK, teardown clean |
| Linux host (non‑root) | Likely no | Maybe | Yes/No | Fallback with WARNs; replay succeeds, no host‑wide rules |

## Validation Plan

1) Podman Container (privileged)
- Verify netns used (no host rules visible): `nft list ruleset` empty in host ns; rules present in netns.
- Replay a blocking curl span; confirm dmesg LOG lines (when dmesg_restrict=0).
- Confirm cgroup.procs contains the replay child PID for all strategies.

2) Manjaro Host (non‑root)
- Expect WARNs and fallback; replay succeeds; fs_diff intact.

## Risks & Mitigations

- Kernel variance: Some hosts may lack required sysctls or cap bits → degrade with warnings, don’t abort.
- Rule leaks: Ensure teardown in Drop + extra best‑effort cleanup on process abort; add GC tool if needed.
- Complexity: Keep host‑ns code paths intact as fallback; isolate netns code behind clear guards.

## Milestones & Deliverables

M1: Netns default path
- Implement named netns lifecycle; run command within; teardown.
- Update nft module to operate inside netns.

M2: PID fidelity across strategies
- Ensure spawn + attach PID uniformly; preserve output semantics.

M3: Docs & validation
- Update COMPLETE_FIXES_PHASE4_PRE45.md and DEV_PODMAN_LINUX_TESTING.md with netns defaults and checks.
- Add a small GC/diagnostic note for netns/rules if teardown fails.

M4: Optional enhancements
- Investigate nft cgroup matching as a host‑ns fallback for finer scoping.

## Acceptance Criteria

- Replay remains stable; strategy lines and fs_diff unaffected.
- In privileged envs: netns used by default; no host rules visible; LOG lines observable (when permitted).
- On constrained hosts: explicit WARNs; replay still succeeds without host‑wide side effects.
- No lingering netns or rules after replay completes under normal conditions.

