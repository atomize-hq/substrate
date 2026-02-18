# Work breakdown and milestones
## Milestone 0: Research & spike (1–2 weeks)
- [ ] Spike: minimal VF-Linux boot (no sharing, no agent) using Virtualization.framework wrappers
- [ ] Spike: vsock host↔guest comms
- [ ] Spike: virtiofs share mount in Linux guest
- [ ] Spike: VF-macOS boot feasibility on Apple Silicon (basic)
## Milestone 1: VF backend skeleton (2–4 weeks)
- [ ] Backend selector plumbing (vf vs lima)
- [ ] VM lifecycle implementation (create/start/stop/destroy)
- [ ] Disk management (base image + per-world overlay)
- [ ] Guest agent handshake and command execution over vsock
- [ ] Structured logging / diagnostics
## Milestone 2: Policy mounts (2–3 weeks)
- [ ] Policy manifest format for read/write/discover
- [ ] Host-side policy mount builder:
  - [ ] RW mount tree
  - [ ] RO mount tree
  - [ ] DISCOVER shadow tree + placeholders
- [ ] Guest mount automation (boot scripts)
- [ ] Acceptance tests for filesystem policy
## Milestone 3: Networking (1–3 weeks)
- [ ] NIC attach/detach policy
- [ ] NAT networking for “network enabled”
- [ ] Optional bridged networking behind flag + entitlement docs
- [ ] Agent-level proxy integration (optional) for restricted egress MVP
## Milestone 4: VF-Linux parity + default rollout (2–4 weeks)
- [ ] parity bug fixes vs Lima
- [ ] performance tuning (mount refresh, large repos)
- [ ] defaults, UX, docs
## Milestone 5: VF-macOS (Apple Silicon) (3–6 weeks)
- [ ] Provisioning flow (base macOS VM image)
- [ ] Guest agent packaging for macOS
- [ ] Toolchain smoke tests
- [ ] Documented compatibility constraints
## Milestone 6: Hardening (ongoing)
- [ ] Improved egress enforcement (host PF / helper) if required
- [ ] Security review + penetration testing
- [ ] Observability improvements
