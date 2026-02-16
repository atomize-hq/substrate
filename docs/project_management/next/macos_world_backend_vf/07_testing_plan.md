# Testing plan
## Test strategy overview
We need confidence in:
- VM provisioning and boot reliability
- Agent control plane (vsock) stability
- Policy mount correctness (read/write/discover)
- Network attachment behavior
- Regression parity with existing Lima backend (VF-Linux)
## Test layers
### 1) Unit tests (host-side)
- Policy compilation → mount manifest
- Shadow tree builder:
  - discover-only creates placeholders, not contents
  - RW/RO rules map correctly
- VM configuration builder:
  - required devices present
  - optional devices gated by flags (network, bridged, etc.)
- Entitlement detection logic (if we add runtime checks)
### 2) Integration tests (local)
#### VF-Linux
- Boot a minimal Linux guest
- Verify agent handshake via vsock
- Verify virtiofs mount inside guest
- Run commands and validate:
  - allowed commands succeed
  - denied commands are blocked
- Verify filesystem:
  - a file in RO share is readable but not writable
  - a file in RW share is writable
  - a discover-only placeholder exists but content is empty / missing
#### VF-macOS (Apple Silicon only)
- Boot macOS guest
- Verify agent handshake
- Verify mounting virtiofs works (macOS uses `mount_virtiofs`)
- Smoke test: run `xcodebuild -version` (or similar toolchain check)
### 3) Security tests
- Attempt to read host paths outside mounts (should fail)
- Attempt to reach network when NIC is detached (should fail)
- Attempt to write to RO mounts (should fail)
- Attempt to read discover-only content (should be absent)
### 4) Performance tests
- Cold boot time, warm boot time
- Mount refresh time when policy updates
- Command roundtrip latency over vsock
- Large repo sync behavior when using staging copies
## CI considerations
- VF-macOS tests may not run in CI unless CI runners are Apple Silicon with entitlement/signing support.
- We should design tests to run:
  - locally for developers
  - optionally on dedicated macOS Apple Silicon runners (self-hosted) for gated merges


## References

- Shared directories mount commands (macOS guest uses `mount_virtiofs`): https://github-wiki-see.page/m/Code-Hex/vz/wiki/Shared-Directories
