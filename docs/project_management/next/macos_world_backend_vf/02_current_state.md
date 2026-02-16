# Current state (macOS + Substrate)
## What we have today
### macOS world backend
- macOS support is achieved via a **Linux world** running in a **Lima VM**.
- The guest Linux image can be hardened with Landlock (and other Linux primitives).
- Filesystem isolation is implemented using a separate filesystem/mount model (VM + guest FS), and then further constrained with Linux security controls.
### Linux host backend (reference)
On Linux hosts, Substrate achieves strong isolation through:
- Separate filesystem/mounting model per world (and often mount namespaces)
- Additional kernel-enforced restrictions (e.g., Landlock as an extra hardening layer)
- Tight control over what paths are visible and writable
This “Linux reference model” is what we want to approximate for macOS.
## Gaps observed on macOS today
1. **No macOS tooling in-world.** The world is Linux; it cannot run Xcode, macOS SDK tools, codesign, etc.
2. **Lima dependency surface area.** VM lifecycle, networking, and sharing semantics are mediated by Lima, not Substrate directly.
3. **Policy parity is constrained by guest OS.** If the guest is Linux, we can use Linux primitives; if the user wants macOS tools, those primitives no longer apply.
## Constraints
- macOS does not provide Linux-style primitives (mount namespaces, Landlock, etc.).
- To get a “Linux-like” isolation model, we either:
  - use a VM boundary (strong, simple, portable in concept), or
  - use native sandboxing with elevated system permissions (complex distribution and weaker/fragile enforcement for our use case)
