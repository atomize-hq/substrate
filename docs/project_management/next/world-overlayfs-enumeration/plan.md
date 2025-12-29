# Plan — World OverlayFS Directory Enumeration Reliability (ADR-0004)

Objective: ensure Linux world overlay mounts support correct directory enumeration and introduce a controlled fallback strategy for overlayfs health failures.

Execution shape (triads):
- WO0: Diagnose + implement stable overlay mount topology + strategy health check + fallback plumbing.
- WO0-integ: Add integration test + smoke coverage for enumeration correctness and fallback observability.

