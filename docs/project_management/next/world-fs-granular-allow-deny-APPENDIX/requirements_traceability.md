# world-fs-granular-allow-deny-appendix — requirements traceability

This file maps stable requirement IDs to specs, tasks, and validation evidence.

## Requirements

### R-AX-001 — Policy schema V3 keys
- Statement: Appendix A keys are implemented and legacy keys hard error.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md`
- Tasks: `WFGADAX0-code`, `WFGADAX0-test`, `WFGADAX0-integ`
- Validation:
  - Smoke: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh`

### R-AX-002 — Routing fail-closed semantics
- Statement: `world_fs.fail_closed.routing=true` enforces no host fallback and maps failures to exit `3` or `4`.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX1-spec.md`
- Tasks: `WFGADAX1-*`
- Validation:
  - Smoke: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh`

### R-AX-003 — Policy-level caging requirement
- Statement: `world_fs.caged_required=true` enforces deterministic caging and rejects uncaged mode (exit `2`).
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX2-spec.md`
- Tasks: `WFGADAX2-*`
- Validation:
  - Manual playbook: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`

### R-AX-004 — REPL exit transparency + `repl.exit_cwd`
- Statement: exit note prints when world cwd differs, and `repl.exit_cwd` selects the target.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX3-spec.md`
- Tasks: `WFGADAX3-*`
- Validation:
  - Manual playbook: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`

