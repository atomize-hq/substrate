---
seam_id: SEAM-2
seam_slug: interactive-terminal-loss-resilience
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-interactive-terminal-loss-resilience.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-02
  stale_triggers:
    - `crates/shell/src/repl/async_repl.rs` changes prompt-worker shutdown, REPL exit handling, or async REPL error classification after decomposition
    - Reedline or crossterm changes alter what controlling-TTY revoke/disconnect looks like during `read_line()`
    - authoritative REPL exit wording changes in `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, or `docs/USAGE.md`
    - the macOS PTY revoke harness or CI assumptions change the Reedline path under test
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
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-2 - Interactive terminal-loss resilience

## Seam Brief (Restated)

- **Goal / value**: make abnormal controlling-TTY loss during the async REPL look explicitly broken instead of silently successful by unwinding the prompt worker, emitting one bounded diagnostic, and exiting with code `1` without leaving an orphaned busy-spin process behind.
- **Type**: capability
- **Scope**
  - In:
    - make owned contract `C-03` concrete enough for implementation: abnormal terminal-loss classification, prompt-worker unwind, singular stderr posture, exit code `1`, and child-reap expectations
    - harden `crates/shell/src/repl/async_repl.rs` so Reedline-backed input can terminate promptly when the controlling TTY is revoked or disconnected after startup
    - add a macOS-targeted regression proof that exercises the Reedline path specifically and proves bounded cleanup on failure
    - update the narrow authoritative doc set that describes interactive REPL exit semantics once runtime behavior is evidenced
  - Out:
    - replay-routing or tracing-validation contract publication owned by `SEAM-1`
    - backend transport, world-agent, shim, or general prompt-backend rewrites
    - downstream cross-surface lock-in and drift-guard work owned by `SEAM-3`
- **Touch surface**: `crates/shell/src/repl/async_repl.rs`, optionally `crates/shell/src/repl/editor.rs`, existing PTY-based REPL harnesses in `crates/shell/tests/repl_world_first_routing_v1.rs` and `crates/shell/tests/repl_world_first_rendering_v1.rs`, an adjacent or new macOS-targeted revoke regression test under `crates/shell/tests/`, `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, and `docs/USAGE.md`.
- **Verification**:
  - `C-03` must become concrete enough that execution can implement one explicit abnormal-terminal-loss contract before closeout publication.
  - Seam-local proof must cover prompt-worker unwind out of the Reedline path, one bounded stderr diagnostic, process exit code `1`, and bounded child cleanup on revoke/disconnect failure paths.
  - Accepted publication of the final operator contract belongs to `S99` and `../../governance/seam-2-closeout.md`, not as a precondition for producer-seam execution.
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`; the `SEAM-1` closeout-backed execution-language publication has been revalidated for this seam
  - Required threads: `THR-02`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers: current async REPL worker behavior in `crates/shell/src/repl/async_repl.rs` and the repo's PTY harness patterns
  - Downstream blocked seams: `SEAM-3`
  - Contracts produced: `C-03`
  - Contracts consumed: the landed `SEAM-1` closeout-backed execution-language baseline plus the current ADR and exit-contract wording that define this seam's abnormal-terminal-loss contract surface

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `SEAM-2` is the producer seam for `C-03` and `THR-02`; downstream conformance and documentation work cannot promote on inferred TTY-loss behavior or incomplete harness evidence.
- **Expected contracts to publish**: `C-03`
- **Expected threads to publish / advance**: `THR-02` from `identified` to `published`
- **Likely downstream stale triggers**: changes to prompt-worker shutdown mechanics, abnormal terminal-loss classification, exit-code taxonomy wording, Reedline/crossterm revoke behavior, or macOS harness cleanup assumptions
- **Expected closeout evidence**: landed async REPL runtime changes, landed macOS-targeted revoke/disconnect regression proof, updated authoritative exit-semantics docs, and explicit `THR-02` publication accounting

## Slice index

- `S00` -> `slice-00-abnormal-terminal-loss-contract-definition.md`
- `S1` -> `slice-1-prompt-worker-unwind-and-exit-cause.md`
- `S2` -> `slice-2-macos-revoke-regression-proof.md`
- `S3` -> `slice-3-exit-semantics-publication-surfaces.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
