# Integration Map — World OverlayFS Enumeration Reliability (ADR-0004)

Inputs:
- World selection (CLI/env/config), policy-derived “requires world”, platform capability probes.

Derived state:
- Selected world filesystem strategy (kernel overlayfs vs fuse-overlayfs).
- Whether world is required vs optional.

Actions:
- Create stable overlay mount topology in a private mount namespace.
- Run command with project bind enforcement.
- Validate directory enumeration; retry via fallback strategy when unhealthy.

Outputs:
- Correct directory listing semantics inside the world view.
- Trace/doctor observability of strategy selection and fallback reasons.

