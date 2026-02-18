# Security — World FS Appendix (Authoritative)

This document is authoritative for Appendix A + B of:
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## 1) Routing fail-closed is a safety boundary
- When `world_fs.fail_closed.routing=true`, Substrate MUST not execute host fallback.
- Routing failures MUST map to exit codes per Appendix B.

## 2) Deny enforcement posture semantics
- `deny_enforcement=strict`: deny rules are a hard boundary; strict prerequisites failures are fatal.
- `deny_enforcement=prefer_strict`: strict is selected when available; otherwise Substrate continues without failing.
- `deny_enforcement=weak`: denies are applied but are not a hard boundary.

## 3) Caging requirement is a policy boundary
- `world_fs.caged_required=true` defines a deterministic scope boundary rooted at `cage_root`.
- Escaping the boundary MUST be prevented for interactive REPL sessions.

