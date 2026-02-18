# Env Var Contract — World FS Appendix (Authoritative)

This document is authoritative for Appendix A + B of:
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It defines env var interfaces for exported state and execution routing behavior.

## 1) Exported state env vars (output-only)

### 1.1 Routing fail-closed state
- `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING=1|0`

Rules:
- This env var is derived from the effective policy.
- It is output-only and MUST NOT be consumed as an override input.
- It MUST be set consistently for:
  - the host shell process,
  - shimmed subprocesses,
  - world-agent requests.

### 1.2 Deletions (hard requirement)
- `SUBSTRATE_WORLD_REQUIRE_WORLD` MUST be deleted.

## 2) Override inputs (non-overlapping)
- Override inputs remain `SUBSTRATE_OVERRIDE_*` per:
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`

