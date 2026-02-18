# Open questions
## Apple Virtualization.framework constraints
1. What are the practical limits of virtiofs permissions/mapping for our use case?
2. Can we reliably identify the VM’s host-side network interface to apply PF rules (if we go Phase 2 egress)?
3. What is the best long-term approach for packaging a VF backend:
   - as a signed CLI binary
   - as a notarized app bundle with a helper
4. What does “macOS guest provisioning” look like operationally for Substrate users?
   - Do we require an IPSW download?
   - How do we cache images and manage updates?
5. Do we need snapshots, clones, or full copies for per-world disks to achieve acceptable performance?
## Policy semantics
1. Is “discover-only” best represented as placeholders, or do we need metadata fidelity (size/hash/mtime)?
2. Do we need to support “write-only” (rare) or other edge cases?
## UX / product
1. How do users choose between VF-Linux and VF-macOS worlds?
2. What is the default for network in VF-macOS?
3. How do we surface “missing entitlements / not signed” errors cleanly?
## Security
1. Are we comfortable relying on agent-level command policy, or do we need deeper guest sandboxing?
2. Do we need host privileged helper for egress? If yes, what is the minimal scope?
