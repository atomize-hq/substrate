# R6-3.5 TASKS — Objective Target Hygiene For Semantic Goal Drift

Status: OPEN (kickoff 2026-07-04). One commit per task; each keeps R6-2/R6-3 acceptance green.

- [ ] **T1 — Shared taxonomy core.** Add `TargetAnchorQuality { Stable, Weak, Junk }`,
  `is_variable_noise_token`, `stable_target_anchor_kind`, `anchor_quality` (grammar-first),
  `is_stable_goal_term` (normalized-form guard) in `context/objective.rs`, exported `pub(crate)` for
  the scorer. Unit tests per variable-mask class and per typed grammar (Stable/Weak/Junk).
  Accept: every SPEC "Acceptance" token classifies as documented; no caller rewired yet.

- [ ] **T2 — Path/link extraction hardening.** Route `extract_inline_paths` /
  `looks_like_repo_path_token` through the path grammars; add a markdown-link/backtick/quoted-path
  pre-pass before whitespace splitting. Accept: GraphQL startup log yields no path target;
  `0.0.0.0:4000`/`/graphql` rejected; `Cargo.lock`, `.github/workflows/ci.yml`, `v2.3.1/notes.md`,
  `2024-report.md`, `docs/graphql/overview.md`, `[overview](docs/architecture_overview.md)`
  preserved; `/Users/.../Library/Application` not a stable target.

- [ ] **T3 — Named-anchor hardening.** Replace permissive `looks_like_explicit_named_target` accept
  rules with typed grammars (RustSymbolRef, CrateOrPackageRef, WellKnownRootlessFile,
  WorkItemIdentifier, WorkspaceRef, InstructionSurface). Accept: `GPT-5.4`, `linux/mac/windows`,
  `closeout/review-ready` rejected; `foo::bar`, bare `agent-drift-analyzer`, `README`, `R6-3.5`,
  `@workspace-ref` preserved.

- [ ] **T4 — Comparison-key defense-in-depth.** `comparison_key_segments_for_target` + display
  fallback emit Stable specifics only. Accept: a junk-target objective's `comparison_key` carries no
  junk segment; valid target's key unchanged.

- [ ] **T5 — Scorer stable-term backstop.** `is_stable_goal_term` guard in `push_goal_term`;
  `eligible_current_goal` requires ≥1 stable target-derived term; constraint terms exempt and
  un-filtered; symmetric on current/previous/anchor. Accept: junk-only current goal → not eligible →
  no fire; valid target change still fires; R6-2/R6-3 fixtures green.

- [ ] **T6 — Delegation guardrail.** `Opaque` `DelegatingParent` → limited-evidence `no_claim`
  unless both goals carry a stable anchor; `Partial`/`Mixed` unchanged. 2–3 regressions
  (parent-delegates-then-waits; parent-reviews-child-result; opaque-child-with-prose-task-names).

- [ ] **T7 — Batch-scan delegation stratification.** `scripts/dev/drift-batch-scan/tabulate.py` +
  README: report totals / fires / eligible / target-resolved / 100%-junk / disjoint-pairs broken by
  delegation category (single-agent, delegating-parent, child-visible/partial, opaque, unknown).

- [ ] **T8 — Docs.** FINDINGS "R6-3.5 Result" before/after + delegation-stratification caveat +
  source map; MAP next-action → R6-3.5 then R6-3.X.2/.X.3; root landing-order status note.

- [ ] **T9 — Batch re-run.** Regenerate the 110-session batch from `~/.codex/sessions`, tabulate
  stratified before/after, paste numbers into FINDINGS.

- [ ] **T10 — Codex review + verification.** `codex exec review` on the diff; address findings;
  `cargo test -p agent-drift-analyzer`, workspace build, `cargo fmt --all -- --check`,
  `cargo clippy -p agent-drift-analyzer -- -D warnings`.
