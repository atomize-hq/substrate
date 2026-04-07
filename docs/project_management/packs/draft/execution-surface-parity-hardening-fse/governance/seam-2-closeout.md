---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-interactive-terminal-loss-resilience/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - S00 e75320a9
    - S1 a15c9287
    - runtime-fix e6636e20
    - S2 70bfece5
    - S3 89c24f09
  required_threads:
    - THR-02
  stale_triggers:
    - any change to `crates/shell/src/repl/async_repl.rs` that alters prompt-worker shutdown, forced stdin invalidation, abnormal terminal-loss classification, or REPL exit handling
    - any Reedline or crossterm behavior change that alters how revoked/disconnected controlling TTYs surface during async prompt reads
    - any wording change in `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, or `docs/USAGE.md` that drifts from the landed `0` versus `1` exit split or the best-effort diagnostic posture
    - any change to `crates/shell/tests/repl_tty_disconnect_macos.rs` or its CI/environment assumptions that stops proving the Reedline path under macOS revoke
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Interactive terminal-loss resilience

Execution landed and the producer contract is now published.

## Seam-exit gate record

- **Source artifact**: `S99` at `docs/project_management/packs/draft/execution-surface-parity-hardening-fse/threaded-seams/seam-2-interactive-terminal-loss-resilience/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `S00` `e75320a9`
  - `S1` `a15c9287`
  - runtime fix `e6636e20`
  - `S2` `70bfece5`
  - `S3` `89c24f09`
  - runtime surface:
    - `crates/shell/src/repl/async_repl.rs`
  - regression-proof surface:
    - `crates/shell/tests/repl_tty_disconnect_macos.rs`
  - publication surfaces:
    - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
    - `docs/reference/env/contract.md`
    - `docs/USAGE.md`
  - validation commands:
    - `cargo fmt --all -- --check`
    - `cargo test -p shell --lib classify_prompt_worker_error -- --nocapture`
    - `cargo test -p shell --lib shutdown_disposition_tracks_termination_cause -- --nocapture`
    - `cargo test -p shell --test repl_tty_disconnect_macos -- --nocapture --test-threads=1`
- **Contracts published or changed**:
  - `C-03`: the abnormal interactive-terminal-loss contract now lives in `crates/shell/src/repl/async_repl.rs`, is proven by `crates/shell/tests/repl_tty_disconnect_macos.rs`, and is published for operators in ADR-0016, the env contract, and the usage guide
- **Threads published / advanced**:
  - `THR-02`: `identified` -> `published`
- **Review-surface delta**:
  - async REPL now distinguishes concrete terminal-loss errors from generic prompt failures and keeps the cursor-timeout fallback separate
  - the revoke path now forces the blocked prompt read off stdin so the REPL exits promptly with code `1` instead of hanging behind the Reedline worker
  - the macOS revoke harness now proves the Reedline path, bounded exit, exit code `1`, and bounded diagnostic posture without leaving the child unreaped on timeout paths
  - authoritative exit-semantics docs now align on normal interactive exit `0`, abnormal terminal loss `1`, and a best-effort bounded diagnostic without claiming broader proof than the landed macOS harness
- **Planned-vs-landed delta**:
  - the seam landed one corrective runtime follow-up (`e6636e20`) after the first S1 pass so the revoke proof could force the blocked prompt read to unwind instead of relying on post-failure classification alone
  - the published operator contract stayed intentionally narrow: the diagnostic remains best-effort, and the closeout cites the macOS revoke harness as the highest-confidence proof instead of inventing blanket cross-platform guarantees
- **Downstream stale triggers raised**:
  - any change to `crates/shell/src/repl/async_repl.rs` that alters prompt-worker shutdown, forced stdin invalidation, abnormal terminal-loss classification, or REPL exit handling
  - any Reedline or crossterm behavior change that alters how revoked/disconnected controlling TTYs surface during async prompt reads
  - any wording change in `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`, `docs/reference/env/contract.md`, or `docs/USAGE.md` that drifts from the landed `0` versus `1` exit split or the best-effort diagnostic posture
  - any change to `crates/shell/tests/repl_tty_disconnect_macos.rs` or its CI/environment assumptions that stops proving the Reedline path under macOS revoke
- **Remediation disposition**: none
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
