# Proposed solution overview
## Summary
Implement a new Substrate world backend for macOS, **VF backend**, that uses Apple Virtualization.framework as the VM orchestrator.
This backend supports two guest OS flavors:
- **VF-Linux:** a Linux guest world similar to the existing Lima backend, but orchestrated directly.
- **VF-macOS:** a macOS guest world (Apple Silicon only) enabling macOS toolchains inside a world.
## Why Virtualization.framework
Virtualization.framework provides a first-party API surface for creating and running VMs with Virtio devices (storage, networking, sockets, filesystem sharing). Open-source tooling and bindings confirm key requirements:
- A process must have the `com.apple.security.virtualization` entitlement to use the API.
- Bridged networking requires `com.apple.vm.networking` entitlement.
## Compatibility / support matrix
| Host | VF-Linux | VF-macOS |
|------|----------|----------|
| macOS Apple Silicon | ✅ Supported | ✅ Supported (scope) |
| macOS Intel | ✅ Likely supported | ❌ Out of scope |
| Linux / Windows hosts | N/A | N/A |
> Note: VF backend is macOS-only. Linux and Windows hosts keep their existing backends.
## High-level components
### Host-side (macOS)
- `vf_runtime` (new): creates VM, attaches devices, manages lifecycle, streams logs
- `policy_mount_builder` (new): builds shared directories for read/write/discover
- `world_image_manager` (new): manages base images, snapshots/clones, cleanup
- `net_policy_manager` (new): attaches network device or not; later may implement stronger filtering
### Guest-side (Linux or macOS)
- `world-agent` (existing concept): entrypoint process; command dispatch + policy enforcement hooks
- optional `bootstrapping` scripts to mount virtiofs shares at well-known mount points
## Key design idea: policy mounts instead of host namespaces
In a VM, we get strong isolation from the host without needing host mount namespaces. We still need fine-grained visibility control, which we implement by:
1. Creating a **per-world staging directory** on the host.
2. Populating it based on Substrate policy as:
   - real copies/symlinks (for read/write)
   - placeholders (for discover-only)
3. Sharing that staging directory into the guest via virtiofs.
This makes policy enforcement explicit and testable.


## References

- Code-Hex/vz README (entitlements + networking entitlement): https://github.com/Code-Hex/vz
- vftool contributing docs (entitlement + signing): https://thedocumentation.org/vftool/contributing/
