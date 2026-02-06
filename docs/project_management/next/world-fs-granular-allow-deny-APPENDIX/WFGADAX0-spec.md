# WFGADAX0 — Policy schema V3 rename + validation (Spec)

## Goal
- Implement Appendix A policy patch schema V3 with deterministic defaults and hard errors.

## Acceptance criteria
- `substrate policy set` accepts Appendix A keys and rejects replaced keys (exit `2`).
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/linux-smoke.sh` passes for the schema cases.

## Non-goals
- No routing fail-closed behavior wiring (owned by WFGADAX1).

