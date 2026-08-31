**Kind:** slice row
**Stable ID:** `a2-host-execution-episode-demotion`
**Canonical for:** extracted A2 slice row plus its terminal closure
**Status:** terminally complete
**Authority scope:** exact extracted source table header and row plus the terminal closure below; no successor admission, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 131
**Supersedes:** canonical ownership of the extracted `A2 — HostExecutionEpisode demotion` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A2 — HostExecutionEpisode demotion

> **Authority boundary:** This file owns the extracted A2 Track A row and its terminal closure. It preserves the exact row text below; the closure records completed authority and proof without authorizing A3 or any other successor.

The exact HSA-owned public `run_stop` intent, bound active-owner delivery, durable terminal
closeout, and exact retry were pulled forward from A2 solely to satisfy the A1 final wall and
completed in A1.4. A2 did not repeat or alter that adoption.

A bounded HSA prerequisite at direct parent
`57b1719ee4ec92c4b960312a7081a4edbbafcf0e` supplies a private typed fork-successor allocation operation. It
authenticates an exact source authority, proves the target namespace absent, and atomically creates
only the target session, participant lineage, binding, and `ParkedResumable` authority, with exact
join and crash/retry semantics. The target's direct birth proof is the allocation record; it does
not fabricate a target Start, process, PID, helper, readiness, endpoint, timeout, or prompt event.
The A2 implementation consumes that prerequisite only for the existing public fork path; it does
not expose a new allocation surface or grant the prerequisite independent successor authority.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A2 — HostExecutionEpisode demotion** | Represent REPL/helper/toolbox/recovered processes as episodes whose PID/socket/readiness data are observations only. | `01` invariants 1–2; `02` SurfaceAdapter row; `04` HostExecutionEpisodeV1; `DESIGN-internal-toolbox-transport-and-session-binding.md` | HostSessionAuthority; InternalToolboxTransport; RouterAttachTrigger | `agent_runtime/control.rs`; `execution/agents_cmd.rs`; `repl/async_repl.rs`; episode-focused tests | No deletion of `__owner-helper`; no transport protocol rewrite; no auto-attach redesign. | Killing/orphaning an episode leaves durable session truth intact; stale episode updates are revision-rejected; transport classification is explicit. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01` |

## Terminal closure

The completed implementation is bound to:

- commit `1d7176a25b7662368e6e1fb3f65b3521e6cab78b`, with sole parent
  `57b1719ee4ec92c4b960312a7081a4edbbafcf0e`;
- tree `e71f731c279a8d65e1111245fc3d31940edd3afd`;
- subject `feat: demote host execution episodes from authority`;
- reviewed fingerprint
  `sha256:f35f07f203a22ac458747ea1abc9b9a3a22fe4705956826b384a3c9b9cb4c392`; and
- final independent implementation verdict `CLEAN`.

The Linux proof records the complete six-kind and four-status `HostExecutionEpisodeV1` tables;
revision-bound, idempotent, process-local observations; PID-zero, competing-process, conflicting-
terminal, stale-revision, and substituted-binding rejection; exact HSA-backed active Turn and
toolbox resolution without legacy target StateStore authority; and public fork consumption of the
landed HSA `ParkedResumable` allocation. The installed-witness S1 path passed Start, retained
worker and receipt preservation, reattach, helper/process/endpoint loss with byte-identical HSA
truth, exact retry without stale mutation, and terminal Stop. The focused episode wall passed
`3/3`; the final public-control inventory passed all six candidate/shared positive proofs. Its 35
remaining failures have the same exact names and normalized installed-witness or REPL-banner
signatures as the 35 failures reproduced against the exact parent (`6/41` candidate versus `4/39`
parent), so none is counted as A2 proof or classified as an A2 regression.
All candidate-specific review findings were resolved before the recorded `CLEAN` verdict.

The frozen implementation boundary remains exact: the HostSessionAuthority tree is
`c56d2ad21c6122bfe8436d380f44c570a39ec762`; `auto_attach.rs` is
`8383f41000986f24aabcd0177b035a32aba3255a`; `orchestration_session.rs` is
`23fd3bb4da6ee0796e4f2b9dabd5faa3c1adc2c0`; `session.rs` is
`3242cd51990d1660bb2e8a8f85136be97439351e`; and `state_store.rs` is
`bb2f2df84fe5e23488bf134678b1bc4932302647`. The public `run_stop` body remains the
A1.4-owned body at
`sha256:31ea1349d9bb079867d56b76dd41561b2c25819a033658bba5648e9f2fc15834`; this is
preserved A1 work, not a new A2 claim.

The `SurfaceAdapter / HostExecutionEpisode` seam's A2 scope satisfies the four-part landing rule.
The A2-scoped clauses of `RG-AUTH-01` and `RG-AUTH-02` are green, `RG-CLOSE-01` remains green,
and `RG-BASE-01` remains green through A2. Ledger-wide work assigned to A3 and other named owners
remains unresolved. A2 is terminally complete and no longer active. The existing canonical
successor is A3, which awaits fresh admission and explicit dispatch; this closure does not admit,
dispatch, implement, or complete it. Track A, macOS, Windows, Track B, and Track C are not
completed by this closure.
