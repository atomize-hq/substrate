# Planning Pack: macOS World backend via Apple Virtualization.framework (VF backend)
- **Date:** 2026-02-13
- **Status:** Draft
- **Owner:** Substrate Runtime team
- **Related ADR:** `ADR-2026-02-13-macos-world-backend-virtualization-framework.md`
## One-paragraph summary
Replace (or substantially reduce dependence on) Lima for macOS by introducing a new Substrate world backend powered by **Apple Virtualization.framework**. This backend supports (1) a Linux guest world for parity with today’s Lima-based Linux worlds, and (2) a macOS guest world (Apple Silicon only) to enable macOS-native toolchains within an isolated world. The policy model (command denies, read/write/discover filesystem controls, and network egress controls) is preserved by combining:
- a **VM boundary** for strong isolation, and
- **host-constructed policy mounts** (shared into the guest via virtiofs) plus **world-agent mediation** for finer-grained controls.
## Why this exists (the “why now”)
- Users need macOS tooling in-world (Xcode, codesign, SwiftPM) while keeping strong isolation.
- We want to reduce reliance on Lima-specific behavior and standardize macOS support on Apple’s virtualization primitives.
- We want a clear, auditable permission model comparable to Linux worlds (even if implemented differently).
## Deliverables
- VF backend runtime (host-side) with VM lifecycle, storage, file sharing, and control-plane comms.
- VF-Linux world flavor (parity with existing macOS+Linux backend).
- VF-macOS world flavor (Apple Silicon only).
- Policy mount builder for read/write/discover.
- Egress controls at least at “attach/no-attach NIC” level, plus a roadmap to stronger filtering.
- Docs + developer tooling for codesigning entitlements required by Virtualization.framework.
## Document index
1. `01_problem_and_goals.md`
2. `02_current_state.md`
3. `03_solution_overview.md`
4. `04_architecture_and_flows.md`
5. `05_policy_model.md`
6. `06_security_and_threat_model.md`
7. `07_testing_plan.md`
8. `08_rollout_plan.md`
9. `09_work_breakdown.md`
10. `10_open_questions.md`
