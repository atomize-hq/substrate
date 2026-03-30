---
seam_id: SEAM-2
seam_slug: install-state-writer-reliability
type: platform
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - any SEAM-1 closeout changes field names path semantics or copy-through rules after THR-01 revalidation
    - adjacent packs refactor hosted or dev installer scripts before this seam lands
    - dry-run or invalid-file fallback semantics change before seam exit publishes C-03 and C-04
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: threaded-seams/seam-2-install-state-writer-reliability/slice-3-seam-exit-gate.md
  status: pending
open_remediations:
  - REM-003
---

# SEAM-2 - Install-state writer reliability

- **Goal / value**:
  - Make successful Linux installs reliably create or update one canonical `install_state.json` file with the exact no-write branches, atomic replacement flow, and warning-only failure posture carried by the source pack.
- **Scope**
  - In:
    - Successful-Linux write matrix for hosted install, hosted `--no-world`, dev install, and dev `--no-world`
    - Hosted `--dry-run` and non-Linux no-write branches
    - Create/update behavior when the canonical file is absent
    - Idempotent rewrite behavior when the canonical file is present
    - Warning-only fallback for unreadable JSON, invalid JSON, or non-`1` schema content
    - Same-directory temp-file rendering and single replace step
    - Preservation of prior canonical content when temp-file write or replace fails
  - Out:
    - Redefining the platform metadata payload itself
    - Redefining package-manager selection semantics or vocabularies
    - Extending behavior to macOS or Windows beyond parity expectations
    - Smoke harness and documentation hardening
    - Uninstaller cleanup alignment
- **Primary interfaces**
  - Inputs:
    - `C-01` persisted platform schema contract
    - `C-02` canonical path and authority-boundary contract
    - Current installer branch structure in hosted and dev install scripts
  - Outputs:
    - `C-03` successful-Linux producer matrix
    - `C-04` reliability and warning-only replace contract
    - Closeout-backed writer truth for `SEAM-3`
- **Key invariants / rules**:
  - A successful Linux producer flow must not depend on incidental group or linger events to create the file
  - Hosted `--dry-run` must create neither the canonical file nor its temp file nor a metadata-only parent directory
  - Non-Linux runs do not gain new `host_state.platform.*` writes from this scope
  - Rewrites must happen through a same-directory temp file plus a single replace step
  - In-place truncation is not allowed
  - Metadata read or write failure must degrade to warning-only behavior and preserve installer success
- **Dependencies**
  - Direct blockers:
    - `THR-01` must remain current; this seam should not decompose against stale field or path assumptions
    - Shared-file sequencing with adjacent installer-focused packs must be explicit because both installer scripts are conflict surfaces
  - Transitive blockers:
    - Any ADR or pack that changes effective-prefix semantics or installer helper structure in the same files
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - Future metadata readers and operator guidance that expect a file after successful Linux installs
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - Possible shared helper logic for JSON read, merge, temp-file render, and replace
- **Verification**:
  - Because this seam **consumes** upstream contracts, verification may depend on accepted upstream contract evidence once `SEAM-1` lands.
  - The first seam-local review should try to falsify:
    - whether any successful Linux branch still exits without the canonical file
    - whether dry-run or non-Linux branches still accidentally create state
    - whether temp-file replace semantics can still truncate or partially update the canonical file
    - whether invalid JSON or write failures still leak into non-zero installer exit behavior
  - A passing pre-exec posture should leave `SEAM-3` able to prove behavior with smoke coverage instead of compensating for ambiguous writer semantics.
- **Risks / unknowns**:
  - Risk:
    - Shared installer files already participate in adjacent ADRs and packs, which can cause merge and sequencing conflicts.
  - De-risk plan:
    - Carry explicit revalidation triggers in `THR-01` and require seam-local review to re-open the touch set if shared-file edits moved.
  - Risk:
    - Invalid-file fallback and cleanup semantics can differ subtly between hosted and dev installers.
  - De-risk plan:
    - Keep one explicit producer matrix and require parity across both scripts in seam-local planning.
  - Risk:
    - The out-of-scope uninstaller path mismatch can create confusion about canonical path semantics.
  - De-risk plan:
    - Track it explicitly as `REM-003` rather than broadening this seam.
- **Rollout / safety**:
  - This seam should remain fail-open for metadata persistence and should not change success exit codes.
  - It should preserve prior good state whenever a temp-file write or replace fails.
  - It should not hide unfinished net-new behavior inside any eventual seam-exit slice; runtime delivery must be complete before closeout accounting begins.
- **Downstream decomposition context**:
  - This seam is now `active` because `SEAM-1` closeout published `C-01` and `C-02`, and `THR-01` is revalidated for execution.
  - The most important inbound thread is `THR-01`; `THR-02` becomes the outbound handoff to conformance.
  - The refreshed seam-local review focuses on branch matrix completeness, temp-file placement, invalid-file fallback, and shared-file sequencing risk in the current hosted and dev installer scripts.
  - Source-plan lineage: primarily `PDLDPM1`, with the old pack's accepted write matrix and atomicity rules preserved as seam-level requirements.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-03`
    - `C-04`
  - Threads likely to advance:
    - `THR-02` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the successful Linux install flow will show canonical file creation after no-event success
    - the failure-posture flow will need to reflect landed warning messages and cleanup behavior
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
