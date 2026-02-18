# Policy model details
This document defines how we map Substrate’s existing policy concepts to a VF-based macOS backend.
## Command policy (allow/deny)
### Principle
Command enforcement happens at the **world-agent command dispatch boundary**, not by trying to sandbox arbitrary processes at the OS kernel level.
### Implementation notes
- Commands are executed only via the agent (e.g., agent provides the shell, or is the entrypoint service).
- The agent maintains:
  - allowlist/denylist rules
  - optional argument patterns
  - per-command resource budgets (timeouts)
- If a user bypasses the agent and logs directly into the VM, command policy is not guaranteed.
  - Mitigation: do not expose SSH by default; expose only agent-controlled channels (vsock console).
## Filesystem policy (read / write / discover)
### Definitions
- **Read:** file contents can be read.
- **Write:** file contents can be modified / created.
- **Discover:** path existence + names (and possibly metadata like size) are visible, but *contents are not accessible*.
### Key decision: implement by construction
Instead of trying to enforce “discover-only” against a real file tree at runtime (which is hard to do robustly across OSes), we build a **shadow tree** for discover-only.
#### Host-side staging structure (per world)
- `share_ro/` – contains content visible to the world read-only
- `share_rw/` – contains content visible read-write
- `share_discover/` – contains:
  - directory names
  - placeholder empty files (or synthetic metadata files)
  - no sensitive contents
#### Guest mount points
- `/mnt/substrate/ro`
- `/mnt/substrate/rw`
- `/mnt/substrate/discover`
#### Mapping a host path into the guest
For a host path `~/project/secrets.env`:
- If policy grants **RO**:
  - the file is copied (or bind-linked) into `share_ro/project/secrets.env`
  - world sees `/mnt/substrate/ro/project/secrets.env` (read-only)
- If policy grants **RW**:
  - file (or directory) is linked into `share_rw/...` (and must support write semantics)
- If policy grants **DISCOVER**:
  - create placeholder `share_discover/project/secrets.env` (empty file)
  - world sees its presence but cannot read its contents (because contents are absent)
### Trade-offs vs Linux
- Linux worlds can sometimes rely on kernel-level path restrictions (e.g., Landlock) to enforce “exists but not readable” on real files.
- VF backend chooses an implementation that is:
  - explicit
  - cross-guest compatible
  - testable
…but may differ from Linux in how “discover-only” is represented.
## Network egress policy
### Definitions
- **No egress:** guest cannot reach external network.
- **Restricted egress:** guest can reach only allowlisted destinations (domain/IP/port).
- **Unrestricted egress:** guest can reach the internet.
### Phase 0 (must ship)
- Implement **No egress** by not attaching any network device to the VM.
- Implement **Unrestricted egress** by attaching a NAT network device.
  - Bridged networking is behind a feature flag and may require an additional entitlement.
### Phase 1 (should ship)
- Provide an agent-managed proxy/gateway path:
  - the world agent provides a “network service” for common tools (git, curl, package managers)
  - enables restricted egress for common flows without needing host-level packet filters
### Phase 2 (stretch)
- Host-enforced packet filtering:
  - apply PF anchor rules to the VM interface (or to a VM-local bridge) to enforce allowlists
  - likely requires a privileged helper and careful UX
### What we are missing vs Linux (explicit)
Compared to Linux hosts, we will *not* have a direct equivalent of:
- network namespaces / iptables per namespace
- Landlock network-related restrictions (not applicable)
- easy per-process egress isolation enforced by kernel namespaces
Instead, we rely on:
- VM device attachment control
- optional host packet filtering (harder on macOS distribution-wise)
- application-level mediation in the agent for many workflows


## References

- Code-Hex/vz README (bridged networking entitlement): https://github.com/Code-Hex/vz
- UTM issue discussion of shared vs bridged networking (anecdotal): https://github.com/utmapp/UTM/issues/6568
