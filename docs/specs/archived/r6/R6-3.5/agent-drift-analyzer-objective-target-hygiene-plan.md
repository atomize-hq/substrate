# R6-3.5 PLAN — Objective Target Hygiene For Semantic Goal Drift

Companion to `agent-drift-analyzer-objective-target-hygiene-spec.md`. Execution is TDD
(RED → GREEN → regression → build → commit), one commit per task, in dependency order.

## Discovery (done, 2026-07-04)

Design was grounded in two deterministic literatures and reviewed read-only by Codex CLI before
implementation:
- Log-template variable abstraction (Drain/LogPai "Preprocessing is All You Need" arXiv 2412.05254,
  Drain3, LogPPT arXiv 2302.07435) → Stage-A variable masks.
- Schema-guided dialogue typed slots (SGD arXiv 1909.05855, FastSGT arXiv 2008.12335) → Stage-B
  typed-anchor grammar allowlist.
- Codex review verdict: design sound; widen Stage B (roots/rootless-files/symbols/crate-names/
  Windows paths), keep `Weak` non-scoring, share ONE taxonomy across extraction+scorer, parse links
  before whitespace split, hard-gate delegation only for `Opaque` parents, keep constraint terms
  out of the junk filter, test all three sides symmetrically. All folded into the SPEC.

## Task sequence

1. **Shared taxonomy core** — `TargetAnchorQuality`, `is_variable_noise_token`,
   `stable_target_anchor_kind`, `anchor_quality` (grammar-first) + `is_stable_goal_term` (normalized
   form). Pure functions, unit-tested per class. No caller rewired yet (foundation).
2. **Path/link extraction hardening** — route `extract_inline_paths` / `looks_like_repo_path_token`
   through the path grammars; add markdown-link / backtick / quoted-path pre-pass before whitespace
   split. GraphQL log → no path target; `Cargo.lock`/`.github/…`/`v2.3.1/notes.md`/link targets
   preserved.
3. **Named-anchor hardening** — replace permissive `looks_like_explicit_named_target` with typed
   grammars (symbol/crate-name/rootless-file/work-item/workspace/instruction-surface). Reject
   `GPT-5.4` & prose enumerations; preserve `foo::bar`, bare `agent-drift-analyzer`, `README`.
4. **Comparison-key defense-in-depth** — `comparison_key_segments_for_target` and the display
   fallback take Stable specifics only.
5. **Scorer stable-term backstop** — `is_stable_goal_term` in `push_goal_term`; `eligible_current_goal`
   requires ≥1 stable target-derived term; constraint terms exempt; symmetric on all sides.
6. **Delegation guardrail** — `Opaque` `DelegatingParent` hard limited-evidence unless both goals
   carry a stable anchor; `Partial`/`Mixed` rely on the backstop. 2–3 regressions.
7. **Batch-scan delegation stratification** — `scripts/dev/drift-batch-scan/tabulate.py` + README:
   break metrics by delegation category.
8. **Docs** — FINDINGS before/after + delegation caveat + source map; MAP next-action; root
   `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` status note.
9. **Batch re-run** — regenerate the 110-session batch from `~/.codex/sessions`, tabulate stratified
   before/after, record in FINDINGS.
10. **Codex code review + verification** — `cargo test -p agent-drift-analyzer`, workspace build,
    `cargo fmt --check`, `cargo clippy`; address findings.

## Verification wall (every code task)

```
cargo test -p agent-drift-analyzer <focused filter> -- --nocapture   # RED then GREEN
cargo test -p agent-drift-analyzer                                    # regression
cargo build -p agent-drift-analyzer                                   # compile
```
Full-close: `cargo fmt --all -- --check`, `cargo clippy -p agent-drift-analyzer -- -D warnings`.

## Risk

Impact analysis (gitnexus, 97a0-substrate): `extract_inline_paths` (1 caller),
`looks_like_explicit_named_target` (3), `comparison_key_from_structured` (0 indexed),
`eligible_current_goal` (2) — all **LOW** risk, module-local, 0 execution processes affected.
Dominant residual risk is **false negatives** (legit targets rejected); mitigated by the review's
widenings and the edge-case fixture set.
