# ADR: macOS World backend via Apple Virtualization.framework (Apple Silicon)
- **Date:** 2026-02-13
- **Status:** Proposed
- **Decision type:** Architecture / Platform support
- **Owners:** Substrate Runtime team
- **Related:** Planning Pack: `planning_pack_2026-02-13_macos_world_backend_vf/README.md`

## Current Curated Draft ADR

- Current curated draft ADR: `docs/adr/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Context
Substrate currently supports macOS by running a Linux “world” inside a Lima VM. This provides strong isolation (VM boundary) and allows us to reuse the Linux security model (mount namespaces + optional Landlock hardening, etc.), but it has two major limitations:
1. **macOS tooling cannot run inside the Linux world.** Users who need native macOS toolchains (Xcode, codesign/notarytool workflows, SwiftPM/Xcodebuild, etc.) cannot run them inside the existing Linux VM world.
2. **macOS support depends on Lima and a Linux guest stack.** This increases operational complexity and adds another dependency surface area on macOS.
At the same time, Apple provides **Virtualization.framework** APIs that can create and manage virtual machines and expose Virtio devices (block, network, sockets, filesystem sharing, etc.). Third‑party reference implementations confirm this framework is usable with the required entitlements and code signing flow.
We want a macOS-first backend that:
- Preserves *isolation strength comparable to our Linux setup* (at least VM-level isolation).
- Enables **macOS guest worlds** (Apple Silicon only) for macOS tooling.
- Preserves Substrate’s policy model: **command denies**, **read/write/discover** filesystem visibility, and **network egress controls**.

## Executive Summary (Operator)

ADR_BODY_SHA256: cda04e55d60fd283d44c13d7e21a0935b6b236047adfcaa90441360d653edd60

- Existing: On macOS, Substrate worlds run inside a Linux VM via Lima; macOS-native toolchains cannot run in-world and macOS support depends on the Lima stack.
- New: Add a macOS world backend based on Apple Virtualization.framework (VF), supporting VF-Linux and VF-macOS (Apple Silicon), with VF preferred on Apple Silicon and Lima retained as a fallback during rollout.
- Why: Enables macOS toolchains inside Substrate worlds while reducing operational complexity and keeping isolation VM-backed.

## Decision
Implement a new macOS backend based on **Apple Virtualization.framework** (“VF backend”) and make it the preferred backend on Apple Silicon. This backend supports two world “OS flavors”:
1. **Linux guest world (VF-Linux):** Equivalent to today’s Lima approach, but using Virtualization.framework directly (reduces dependency on Lima).
2. **macOS guest world (VF-macOS):** A macOS VM-based world (Apple Silicon only) for macOS toolchains.
The VF backend will be implemented as an internal world-runtime provider (pluggable backend) selected at runtime:
- Default on Apple Silicon: **VF backend**
- Fallback: existing Lima backend (initially kept as a compatibility fallback / escape hatch)
## High-level architecture
### Trust boundaries
**Host (macOS)**
- Substrate manager / orchestrator
- Policy compiler (filesystem + network + command policies)
- VM lifecycle manager
- Host-side “policy mounts” staging directories
**Guest (VM: Linux or macOS)**
- World agent (entrypoint; enforces command-level policy + mediates tool access)
- Guest filesystem (ephemeral root disk)
- Optional in-guest firewall layer
### VM integration devices
We will use Virtualization.framework Virtio devices where possible:
- **Virtio block storage** for VM root disk + optional additional data disks.
- **Virtio socket (vsock)** for host⇄guest control plane.
- **Virtio filesystem sharing (virtiofs)** for controlled host directory exposure into the VM. Shared directories are mounted by the user or initialization scripts inside the guest (macOS: `mount_virtiofs`, Linux: `mount -t virtiofs`).
- **Network attachment** via NAT (“shared network”) by default; bridged networking only when explicitly enabled due to additional entitlements and higher risk.
## Security and policy model
### Command denies
Command denies remain enforced at the Substrate “world agent” layer (the command dispatch boundary), independent of OS flavor.
### Filesystem policy: read / write / discover
We will implement filesystem policy by **constructing policy-controlled shared directories on the host** and exposing only those via virtiofs.
This aligns with the model:
- VM boundary provides coarse isolation.
- Policy-controlled mounts provide fine-grained visibility.
Mechanisms:
- **Read-only:** mount a host share as read-only in the guest.
- **Read-write:** mount a host share as read-write in the guest.
- **Discover-only:** expose *names/paths* but not content by mounting a “shadow tree” that contains:
  - directories + placeholder files (optionally with metadata stubs),
  - no sensitive contents.
  (Discover-only is therefore implemented as *presence without content*, not as “real file but unreadable”.)
### Network egress controls
We will support a staged implementation:
- **Phase 0:** allow/deny network by attaching / not attaching the VM NIC.
- **Phase 1:** add a Substrate-managed egress gateway/proxy path for common tools (git, curl, package managers) via world agent integration.
- **Phase 2:** host-enforced egress filtering (PF anchor rules on the VM interface, or a privileged helper) as the “strong” option where feasible.
## Platform / distribution constraints
Using Virtualization.framework requires:
- **Code signing** with the `com.apple.security.virtualization` entitlement.
- **Hardened Runtime + declared entitlements** for notarization workflows.
- **Additional entitlement** for bridged networking when used (`com.apple.vm.networking`).
- macOS guest virtualization is supported on **Apple Silicon** (scope accepted for this ADR).
## Alternatives considered
1. **Keep Lima-only**
   - Pros: mature, already works.
   - Cons: no macOS tooling in-world; extra dependency and integration complexity.
2. **“True native” macOS sandbox (no VM)**
   - Pros: no VM.
   - Cons: replicating Linux-grade isolation (mount namespaces + kernel-enforced path access + reliable egress control) would likely require deeper macOS security primitives, privileged helpers, or extensions with substantial distribution friction. Security model is harder to make robust and consistent.
3. **Hypervisor.framework directly**
   - Pros: lower-level control.
   - Cons: more work; Virtualization.framework already provides high-level device support.
## Consequences
### Positive
- First-class macOS backend with Apple-supported virtualization primitives.
- Enables macOS toolchains inside Substrate worlds (VF-macOS).
- Potential reduction in operational issues vs Lima by consolidating VM orchestration in one stack.
### Negative / tradeoffs
- Requires careful handling of macOS entitlements and distribution (codesign + notarization).
- Some policy controls (especially network egress) may require privileged host integration for strong guarantees.
- Apple Silicon only for VF-macOS.
## Rollout plan (summary)
- Ship VF-Linux behind a feature flag; keep Lima default.
- Stabilize file sharing and control plane (vsock).
- Add VF-macOS (Apple Silicon only) as an opt-in world flavor.
- Promote VF backend to default on Apple Silicon after telemetry + soak.
## References

- vftool contributing docs (entitlements + signing): https://thedocumentation.org/vftool/contributing/
- Code-Hex/vz README (Virtualization.framework overview + entitlements + networking entitlement): https://github.com/Code-Hex/vz
- Shared directories notes + mount commands (virtiofs): https://github-wiki-see.page/m/Code-Hex/vz/wiki/Shared-Directories
- Go package docs noting virtualization entitlement requirement: https://pkg.go.dev/github.com/aoxn/vz
- Xcode Help: Enable hardened runtime (notarization + entitlements): https://help.apple.com/xcode/mac/current/en.lproj/devf87a2ac8f.html
- UTM issue discussing NAT (“shared network”) vs bridged behavior (anecdotal): https://github.com/utmapp/UTM/issues/6568
