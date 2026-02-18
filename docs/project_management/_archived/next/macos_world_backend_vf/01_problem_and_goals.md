# Problem statement and goals
## Problem statement
Substrate’s current macOS support runs a Linux world inside a Lima VM. This solves isolation and lets us reuse our Linux-based policy enforcement, but it does **not** support workflows that require macOS-native toolchains inside the world (Xcode, codesign/notarization workflows, Swift, etc.). In practice, this splits users into:
- Users happy with a Linux toolchain on macOS (current Lima path is fine)
- Users who need macOS toolchains *and* want Substrate isolation (no first-class solution today)
Additionally, macOS support depends on Lima’s implementation details, which increases our maintenance surface area and makes it harder to reason about long-term support.
## Goals (must)
1. **Provide a first-class macOS backend** that does not require Lima as the core backend runtime.
2. **Enable macOS guest worlds on Apple Silicon** so that macOS tooling can run inside an isolated world.
3. Preserve Substrate’s policy model:
   - Command allow/deny
   - Filesystem: read / write / discover semantics
   - Network egress control
4. Provide an incremental migration path (feature flags + fallback to Lima) to avoid breaking existing users.
5. Make the solution testable in CI and locally with deterministic acceptance tests.
## Non-goals (explicit)
- Supporting macOS guest worlds on Intel Macs (Apple Silicon only is acceptable per scope).
- Achieving perfect parity with Linux kernel-level sandboxing primitives (e.g., Landlock) *when running a macOS guest*. The VM boundary becomes the primary isolation primitive.
- Building a system extension / kernel extension as the mainline solution.
- Replacing the Linux world model on Linux hosts.
## Success metrics
- ✅ A user can create and run a VF-Linux world on macOS with parity to the existing Lima flow.
- ✅ A user can create and run a VF-macOS world on Apple Silicon and run common macOS toolchain commands.
- ✅ Policy enforcement works:
  - Commands are denied when configured to be denied.
  - Files outside allowed policy mounts are not visible.
  - Read/write/discover semantics behave as specified.
- ✅ Clear operational docs (installation, signing requirements, troubleshooting).
- ✅ In-field telemetry (or at minimum structured logging) can attribute failures to: provisioning, VM boot, sharing, agent comms, policy mounts, networking.
