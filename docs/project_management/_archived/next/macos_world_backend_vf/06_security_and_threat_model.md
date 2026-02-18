# Security review and threat model (VF backend)
## Security goals
1. **Strong isolation from host OS** by default.
2. **Deterministic policy enforcement** for:
   - commands
   - filesystem visibility and writeability
   - network egress (at least coarse on/off)
3. **Least privilege** on the host as much as possible.
## Primary isolation boundary
The VM boundary is the primary boundary. This is stronger than most “native sandbox” approaches and more comparable to the Linux VM approach we already use on macOS.
## Threat model assumptions
### Adversary model
- Attacker controls arbitrary code executed inside the world (guest).
- Attacker attempts to:
  - read files not intended to be visible
  - exfiltrate data via network
  - escape to host
  - tamper with policy enforcement
### Trust assumptions
- Substrate host-side manager is trusted.
- Guest OS is untrusted (in the sense it may be compromised).
- Policy mounts and vsock are part of the trusted computing base on the host.
## Key attack surfaces
### 1) Guest → host escape via virtualization vulnerabilities
- This is a baseline risk in any VM approach.
- Mitigation: keep macOS up to date; avoid unnecessary device exposure; prefer NAT; avoid bridged unless needed.
### 2) Policy bypass via direct guest access
If a user can open an uncontrolled shell or remote login to the VM, they may bypass agent-level command policy.
- Mitigation:
  - do not expose SSH
  - provide only agent-mediated execution paths
  - lock down guest to auto-login into agent shell where possible
### 3) Filesystem exfiltration via shared directories
If we share host directories into the guest, the guest can read whatever is shared.
- Mitigation:
  - share only policy-constructed directories
  - implement discover-only by construction (no contents to read)
  - keep secrets out of RW shares unless required
### 4) Network egress bypass
If the guest has a NIC, any process in the guest can attempt network connections.
- Mitigation:
  - attach/no-attach NIC enforcement (Phase 0)
  - optional host-level filtering (Phase 2) when feasible
## Host entitlements and signing
Using Virtualization.framework requires code signing with the `com.apple.security.virtualization` entitlement.
If we support bridged networking, `com.apple.vm.networking` may be required.
If we distribute Substrate as a notarized macOS app/binary, we must use hardened runtime and declare entitlements appropriately.
## Security acceptance criteria
- [ ] VF backend uses minimal necessary Virtio devices:
  - block, vsock, virtiofs, (optional) net
- [ ] Default VM has **no bridged networking**
- [ ] Default VM shares **no host directories** except policy mounts
- [ ] Discover-only paths do not include file contents
- [ ] A compromised guest cannot read host paths outside policy mounts
- [ ] A compromised guest cannot reach the network when NIC is not attached
- [ ] All entitlements and signing requirements are documented


## References

- vftool contributing docs (virtualization entitlement + signing): https://thedocumentation.org/vftool/contributing/
- Code-Hex/vz README (virtualization entitlement + networking entitlement): https://github.com/Code-Hex/vz
- Xcode hardened runtime overview: https://help.apple.com/xcode/mac/current/en.lproj/devf87a2ac8f.html
