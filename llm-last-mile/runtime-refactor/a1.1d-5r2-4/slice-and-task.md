**Kind:** slice/task
**Stable ID:** `A1.1d-5R2-4`
**Canonical for:** A1.1d-5R2-4 slice and task description
**Status:** canonical
**Authority scope:** exact extracted R2-4 source body only
**Source span:** [`03-phase-slice-map.md#a11d-5r2-4--r2-integration-and-closeout`](../03-phase-slice-map.md#a11d-5r2-4--r2-integration-and-closeout), D5 pre-extraction lines 1340–1382
**Supersedes:** canonical ownership of that source body; the source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** the source compatibility anchor and [`README.md`](README.md)
**Source span SHA-256:** `7fa6a147b3fe76e68e9d3fae6196ebb92a59dc77614a6b6b3a608ff7f4513305`

###### A1.1d-5R2-4 — R2 integration and closeout

| Packet field | Frozen requirement |
|---|---|
| Goal | Close the complete propagation matrix: default/custom A without outer overrides, ambient B conflict, repeat install propagation, install/uninstall context symmetry, generated and doctor correctness, and Linux product proof; hand cleanup/idempotency to R3. |
| Prerequisites | Final review-clean R2-1, R2-2, and R2-3 commits; all inventory rows implemented or explicitly R3-owned; clean tree and current GitNexus. |
| Must read | All six control-pack files; all three subpacket closeouts; PI-001–PI-118; every R2 proof gate and A1.1d-5I mapping. |
| Sibling context | This is a join/checkpoint only. A production defect returns to its owning subpacket. R3 remains next and owns all cleanup/convergence. |
| Exact production-file allowlist | **None.** Production fixes, generated checked-in artifact changes, installer changes, and platform changes are forbidden in R2-4. |
| Exact evidence/test allowlist | The existing six `llm-last-mile/runtime-refactor/*.md` control-pack files; the test files/runners already authorized by R2-1/R2-2/R2-3; no new production file. Test edits only close matrix coverage and must cite a missing proof row. |
| Explicit non-goals | Any production fix; deletion/rollback/manifest/convergence; R3 or `RG-INSTALL-01` closure; A1.1d/A1 closeout; B1/B2.1 joint closeout; B3.1 unblock; world capability/policy/supervisor/receipt changes; seam promotion. |
| Exit gate | Every R2-owned PI row has code+proof; supported Linux product install works at default and custom A with no outer override and with conflicting B; repeat install preserves commitment; install/uninstall select identical context; generated/doctor projections identify A; dedicated supported Linux full-world/Codex service proof is recorded; macOS mapping stops before forwarding and records the R3 activation prerequisite; R3 handoff lists its exclusive actions. |
| Regression gates | R2-UDEV-01, R2-UREL-01, R2-SHIM-01, R2-GEN-01, R2-RUNTIME-01, R2-LINUX-01, R2-DIAG-01 and applicable platform mapping gates; full format/Clippy/workspace/installer/world/doctor wall; GitNexus detection; `git diff --check`; unchanged B1/B2.1 and world/policy differentials. |
| GitNexus posture | Documentation/tests-only expected. Any affected production symbol or unexpected execution flow fails the packet and returns it to its owner. |
| Independent reviews | Fresh final authority/security, lifecycle/R2-versus-R3, and cross-platform/allowlist/regression reviews, all CLEAN. |
| Platform evidence | Linux native product proof is mandatory before R3. Native macOS/Windows assignments are reported truthfully as complete or still pending; static evidence never substitutes. |
| Stop conditions | An R2 row lacks an owner/proof; production repair is attempted; cleanup leaked into R2; Linux product host is unavailable; a native claim lacks native evidence; a seam/root architecture must change. |
| Next packet | A1.1d-5R3 — Partial-install cleanup and idempotency. |

**Terminal host status:** the historical conditional assignment in the
[booking and restoration contract](../review-control/r2-4-linux-host-booking.md) remains preserved as
pre-start authority. The bounded Linux proof and exact restoration later completed on
`spenser-linux` from source `316ee5c6cf12c060388c9d9376e0a79537f2094a` / tree
`1eae07018caef023b2f27ef22892825b140e9a4d`; the product correction that removed its PATH-bypass
and public-wrapper limitations landed as `d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` / tree
`b2d68905580d35d7d63aea8f36f274f723c26733`. The
[R2-4 closeout evidence record](../review-control/r2-4-closeout-evidence.md) binds the immutable
artifacts and the later `NARROW_R2_4` authority: the historical full-world/Codex phrase is adopted
only as a bounded supported-Linux world-backed command and Codex-runtime reachability record,
without authenticated execution or architectural promotion. No production-file allowlist is
created by this closeout.

R2 transports the exact context that R3 will later use, but R3 exclusively owns deleting partial
candidates; current-attempt rollback; removal of managed gateway/helper/unit/socket artifacts;
managed-artifact ownership manifests used for deletion; removal of recursive/wildcard deletion;
uninstall convergence; restoration of installer-created group, membership, ACL, and linger state;
crash-window cleanup; uninstall-to-reinstall convergence; and preservation of unrelated or
pre-existing artifacts. No R2 exit gate may count any of those actions as complete.
The inventory additionally assigns macOS/WSL stop actions that touch shared platform state
(PI-051/PI-053), recursive dev-shim fallback removal (PI-064), shell-profile snippet removal
(PI-074), every action in PI-092–PI-103, and forwarding teardown/timeout PI-113–PI-114 to R3; R2
may transport their future exact target only. R3 may activate the PM-bound Lima SSH-UDS target only
after PI-101/PI-113/PI-114 land together; activation cannot reselect any R2 context or mapping field.
