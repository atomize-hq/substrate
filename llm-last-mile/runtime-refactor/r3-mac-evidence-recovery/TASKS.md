# TASKS — AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN

## Planning-task checklist

- [ ] **T1 — Verify the planning start and inputs.**
  - Pass: target remote/base/tree, clean planning worktree, skill-suite digest, donor base/tree,
    tracked diff hash, and all donor status entries equal the values in `SPEC.md`.
  - Verify: `git rev-parse`, `git ls-remote`, `git status --short`, donor `git diff --binary | shasum`.
  - Files: no repository mutation.
- [ ] **T2 — Freeze the three planning artifacts and current-status appendices.**
  - Pass: `SPEC.md`, `PLAN.md`, and this file cover the selected outcome, full matrix, decisions,
    exact packet fences, dependencies, proof, rollback, and closed statuses.
  - Verify: Markdown/reference/allowlist/secret checks and manual comparison with amendment 0008.
  - Files: only the three recovery files and the five named R3 current-status documents.
- [ ] **T3 — Freeze the planning subject.**
  - Pass: the authoritative subject is exactly the eight lexical paths named below, excluding all
    four review-control files; the pre-review hash is computed by
    `for p in <eight literal paths>; do shasum -a 256 -- "$p"; done | LC_ALL=C sort | shasum -a 256`.
  - Verify: record the literal ordered path list, command, and output in every review-cycle entry;
    a report that observes a different algorithm/hash is a finding, never a clean attestation.
  - Files: exact review set only after the subject is frozen.
- [ ] **T4 — Run fresh discovery burst.**
  - Pass: fresh read-only authority/security, lifecycle/convergence, and allowlist/evidence
    reviewers receive the frozen subject, gates, and non-goals but no author reasoning/conclusion.
  - Verify: all reports cite the same subject fingerprint and their findings are classified.
  - Files: three review reports and `r3-mac-evidence-recovery-planning-review-cycle-record.json`.
- [ ] **T5 — Remediate every valid P1/P2 in one documentation pass.**
  - Pass: every valid blocker is repaired inside the 12-path planning fence; P3/P4 are deduplicated
    into `06` only if valid and unfixed.
  - Verify: review validator accepts `--next-cycle closure`; changed subject fingerprint is new.
  - Files: planning subject; optional `06` only for a valid P3/P4.
- [ ] **T6 — Fresh closure and bounded supplemental cycles.**
  - Pass: a different closure reviewer ends clean, or each supplemental cycle proves direct causality
    and stays within the two-cycle budget. Any other P1/P2, widened risk, or exhausted budget stops.
  - Verify: validate the JSON record after every cycle and before dispatching the next one.
  - Files: exact review set and bounded subject deltas.
- [ ] **T7 — Publish only a clean planning packet.**
  - Pass: zero unresolved valid P1/P2, and this packet's stricter zero unresolved P3/P4; exact
    changed-path allowlist; secret/diff/change-detection gates; remote still equals base.
  - Verify: one normal fast-forward docs/review commit, remote equals landed commit, clean tree.
  - Files: the 12-path allowed union only.

## Future recovery execution checklist

Each item is individually pass/fail verifiable and requires its own later authority; none is
started by this planning commit.

- [ ] **R1: recreate only the Bash descriptor fix.**
  - Acceptance: default Bash 3.2 leaves a pre-open caller FD usable and leaves no bootstrap temp
    file; context argv/env remain canonical.
  - Verify: `tests/installers/dev_install_bash32_fd_regression.sh` on macOS and focused script diff.
  - Files: exactly the R1 allowlist in `PLAN.md`.
- [ ] **R2: recreate only macOS cfg/compile admissions.**
  - Acceptance: the specified existing cfg leaves compile on macOS, installer build flags include
    exactly named lifecycle binaries, Linux host behavior is unchanged, and Windows is untouched.
  - Verify: `tests/mac/dev_install_compile_surface_r3.sh`, target cargo checks, Linux cfg diff review.
  - Files: exactly the R2 allowlist.
- [ ] **R3: bind project identity outside evidence artifact control.**
  - Acceptance: the validator requires `--expected-product-project-id`, rejects absence/mismatch,
    and validates the historical Linux artifact only with the historical expected value.
  - Verify: `python3 -m unittest scripts.ci.test_validate_r3_native_evidence` plus direct negatives.
  - Files: exactly the R3 allowlist.
- [ ] **R4: implement the minimum protected MAC foundation.**
  - Acceptance: canonical Keychain/P-256/Stage-1/ticket record joins reject malformed/replay/expired
    and wrong peer/requirement inputs before effect; no stdio session type or action executes.
  - Verify: focused Rust/MAC target checks and source-shape negatives; no native state action.
  - Files: exactly the R4 allowlist.
- [ ] **R5: land the trusted mapped-submit bridge.**
  - Acceptance: Stage-1 authorizes only an absent `mac.lima.instance/Create` before PM finalization;
    every post-PM pair uses the canonical request and direct publisher bootstrap needs a retained
    terminal/channel `PublisherBootstrapAuthorizationV1`. No raw `lima-action`, direct helper,
    caller XPC operation, or caller ticket/record path remains.
  - Verify: focused client/control/executor/script tests, closed role/action enumeration, direct
    bootstrap/missing-auth/stage1-reuse negatives, and descriptor preservation matrix.
  - Files: exactly the R5 allowlist.
- [ ] **R6: land the dual PM-bound pairing sessions.**
  - Acceptance: data and operator-TTY sessions have the same immutable PM/source/artifact binding
    and distinct session identities; the retained host terminal alone displays full confirmation
    values, the guest TTY obtains them manually, and generation-CAS/replay/retry/lifetime transitions
    follow the durable table exactly.
  - Verify: focused typed transition, host-terminal/data-derived confirmation negatives, no-stdio-
    alias/no-third-session, wrong-binding, each kill/reconnect/expiry/replay table row, and Linux-
    host-unreachable tests.
  - Files: exactly the R6 allowlist.
- [ ] **Recovery review/receipt.**
  - Acceptance: all source packets form the explicitly authorized recovery subject, review cycles
    are clean, all source gates pass, one remote implementation commit/receipt is recorded, and its
    only evidence successor is `EVIDENCE:R3-MAC-IMP-01`.
  - Verify: commit ancestry, remote equality, fingerprint, review validator, exact file union.
- [ ] **Official native evidence (separate task).**
  - Acceptance: supported macOS proves both sessions, protected bootstrap, exact actions and full
    restoration against the published remote-equal source; no static test is substituted.
  - Verify: native artifact validator, evidence receipt validator, artifact/receipt digest join, and
    independently observed baseline parity.

## Required implementation review lenses

For R1–R6 and their combined recovery receipt, freeze the exact source subject and use all three
fresh read-only lenses: authority/security (carrier, Keychain, XPC, ticket, confirmation, secrets),
lifecycle/convergence (Stage-1 order, CAS, retry/replay, descriptor preservation, rollback), and
allowlist/evidence (paths, symbols, platform bounds, test/native separation). The closure reviewer
must be a different fresh reviewer. `CLEAN` is terminal; only P1/P2 caused or unmasked by the
immediately previous remediation may use either supplemental cycle.

## Terminal status map

| Observation | Required result |
|---|---|
| base/tree/remote moves | `BASE_DRIFT` |
| protected donor/checkout or allowlist contradiction | `BLOCKED_CONTRADICTION` |
| a new transport/session/selector/principal or unlisted source is needed | `BLOCKED_SCOPE_EXPANSION` |
| valid unresolved P1/P2, invalid causal record, or this packet's P3/P4 publication wall | `BLOCKED_REVIEW` |
| required native macOS/privilege/two-session capability is unavailable | `BLOCKED_PLATFORM_HANDOFF_REQUIRED` |
| native action/restoration/evidence validation fails | `BLOCKED_NATIVE_EVIDENCE` |
| all required recovery source gates are remote-equal and clean | `LANDED_CLEAN` then separately authorize evidence |

## Skill application record

| Loaded repository-local workflow | Constraint applied here |
|---|---|
| `.agents/skills/using-agent-skills/SKILL.md` | selected the required phase workflows and recorded their paths. |
| `.agents/skills/orchestrate-top-level-tasks/SKILL.md` | preserved identity, exact remote base, one receipt, and no successor dispatch. |
| `.agents/skills/spec-driven-development/SKILL.md` | froze objective, success criteria, commands/boundaries, and assumptions before planning. |
| `.agents/skills/planning-and-task-breakdown/SKILL.md` | produced small ordered packets, dependencies, checkpoints, and explicit verification. |
| `.agents/skills/incremental-implementation/SKILL.md` | constrained future work to independently reversible, proof-gated slices rather than donor bulk. |
| `.agents/skills/documentation-and-adrs/SKILL.md` | recorded the why/alternatives for data/TTY and mapped-submit decisions. |
| `.agents/skills/code-review-and-quality/SKILL.md` | requires independent five-axis reviews, evidence-backed findings, and closure before publication. |
| `.agents/skills/security-and-hardening/SKILL.md` | applied threat boundaries/least privilege to Keychain, XPC, ticket, TTY, and confirmation flows. |

## Authoritative planning-subject path list

Before the discovery burst and after each consolidated remediation, the planning subject is exactly:

1. `llm-last-mile/runtime-refactor/00-README.md`
2. `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
3. `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
4. `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
5. `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
6. `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/PLAN.md`
7. `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md`
8. `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/TASKS.md`

The exact review set is excluded from the subject fingerprint so that review bookkeeping cannot
self-change the reviewed bytes. Any added source, test, manifest, script, review report, or control
path is a fence failure unless the packet contract explicitly names it.
