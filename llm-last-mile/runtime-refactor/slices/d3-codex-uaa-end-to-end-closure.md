**Kind:** slice row
**Stable ID:** `d3-codex-uaa-end-to-end-closure`
**Canonical for:** extracted D3 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 214
**Supersedes:** canonical ownership of the extracted `D3 — Codex/UAA end-to-end closure` row
**Superseded by:** none
**Projection consumers:** [`track-d-uaa-execution-envelope-and-side-effect-mediation.md`](track-d-uaa-execution-envelope-and-side-effect-mediation.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# D3 — Codex/UAA end-to-end closure

> **Authority boundary:** This file owns only the extracted D3 Track D row. It preserves the exact row text below, keeps D1/D2/E3 prerequisites outside the row scope, and does not dispatch D3.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **D3 — Codex/UAA end-to-end closure** | Prove world Codex/UAA placement, adoption of the existing secure gateway credential bootstrap, and side-effect mediation, including non-zero turn closeout and diagnostics. | `05` credential carrier/adoption, UAA caging, external-sandbox, and non-zero rows; `04` secret handoff + supervisor terminal rules; Codex mapping design | Supervisor; broker; envelope; config projection; in-world gateway | targeted Codex/UAA runtime integration tests and smoke harnesses; only defects exposed within D1/D2/E3 boundaries | No broad provider feature expansion; no FD-carrier rewrite without a separately proven defect; no advisory-only success; no copied-credential compatibility evidence counted as proof; no suppression of non-zero exits. | Real worker turn points Codex at the managed gateway, joins its one-time credential consumption to the exact envelope, creates no copied secret file or inherited child FD, proves caged/uncaged policy contrast and no bypassing edit/tool channel, records durable failure on non-zero, and retains session authority; traces contain handoff state but no secret material. Final `RG-OBS-01` integration closure additionally requires the already-landed B0/B1/B2.1/B3.1/C1/B4, E2/E3, and D2 clause evidence to join in one trace proof. | `RG-UAA-03`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-WORKER-EXIT-01`, final integration clause of `RG-OBS-01` |
