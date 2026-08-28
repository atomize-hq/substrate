**Kind:** gate
**Status:** canonical closed selection gate
**Canonical for:** `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`

## `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` contract (2026-08-20; closed selection record)

This was the global documentation/control-plane rebind-and-selection gate. It superseded the former
use of `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` as the global product-work predecessor, while leaving
the macOS lane contract above unchanged in scope. It is now closed only as the selection of the
historical [A1.3 Linux-first implementation packet](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md),
later narrowed by the held
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and finally corrected so the active implementation authority is
[`linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).

### Admission

The reentry task must bind the live product branch, ancestry, worktree/index, current source and
control-pack state, the specific historical evidence it relies on, and Linux-first adoption posture.
It must name exactly one subsequent runtime-refactor packet with its owner, path and symbol fences,
dependencies, acceptance criteria, verification, and macOS/Windows exclusions.

### Allowed completion claim

The gate may close only as a documentation/control-plane rebind that selects that one later packet.
It may not claim implementation, Linux proof, cross-platform completion, or promotion of a runtime
seam.

### Prohibited ownership and actions

This gate owns no runtime, installer, Lima, macOS, Windows, archive, Keychain, Attempt 4, or native
operation. It does not reopen Linux R3, make macOS parity a predecessor, or dispatch the selected
packet without separate authority.

The complete global scheduling decision and closed selection record are
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). The
separate, active A1.3-P1 implementation authority is
[`linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md),
and the held A1.3-P0 and A1.3 packets remain
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md)
and
[`linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md`](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md).

**Source provenance:** extracted from [`04-contracts-and-gates.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record`](../04-contracts-and-gates.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record), baseline lines 10317–10354; the seven repository-relative Markdown targets under `linux-first-runtime-resumption/` were rebased by one parent directory to preserve their original root-level targets after relocation under `gates/`
**Baseline span SHA-256:** `f2f4a855f7a850bcdc990e5089801085d51e02ae60fd56db7b45a2d9c5bafaef`
