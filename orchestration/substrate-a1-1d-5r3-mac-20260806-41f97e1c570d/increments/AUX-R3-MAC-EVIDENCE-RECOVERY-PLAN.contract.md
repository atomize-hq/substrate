# AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN contract

## Selected outcome

Produce and remotely publish one review-clean, implementation-ready planning packet that defines
how to recover only the source corrections required to make a fresh official
`EVIDENCE:R3-MAC-IMP-01` attempt possible. The packet must eliminate the ambiguity that caused the
preserved source-correction task to expand to 8,821 inserted lines. It must leave zero architecture,
authority, file-fence, task-sequencing, verification, or review-budget decision to the later
implementation task.

This is a planning/documentation/review increment. It implements no product behavior, runs no
install or native lifecycle action, and does not dispatch its successor.

## Required context and source truth

Read current repository truth at the exact base, including `AGENTS.md` and the complete
`llm-last-mile/runtime-refactor/00-README.md` through `06-review-finding-inventory.md`,
`review-control/README.md`, `review-control/review-cycle-record.example.json`,
`review-control/validate_review_cycle.py`, and the existing R3 planning/MAC review records.

Load `.agents/skills/using-agent-skills/SKILL.md` first. It must route this work to, at minimum:

1. `spec-driven-development` for the exact selected outcome, requirements, boundaries, and
   acceptance criteria;
2. `planning-and-task-breakdown` for the ordered recovery packets, dependencies, proof gates, and
   stop states;
3. `incremental-implementation` for bounded multi-file documentation work;
4. `documentation-and-adrs` for durable decision rationale;
5. `code-review-and-quality` for every independent review;
6. `security-and-hardening` for Keychain/XPC/ticket/TTY/authority decisions.

Record the loaded skill paths and how each applicable workflow constrained the subject. The
historical `/spec` user-approval pause does not apply here: complete SPEC, PLAN, and TASKS in this
one bounded task. Do not implement their contents.

Treat these external paths as read-only evidence inputs:

- preserved donor worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`;
- donor base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` /
  `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`;
- donor tracked diff SHA-256:
  `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`;
- amendment 0008 and all AUX source-correction receipts under the bound orchestration state root;
- independent audit
  `orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/audits/AUX-R3-MAC-SOURCE-CORRECTION.scope-ledger-independent-audit.md`.

Never mutate, stage, clean, reset, restore, checkout, commit, archive, or otherwise alter the donor
worktree. Re-hash it before relying on it. If its bytes differ, return `BLOCKED_CONTRADICTION`.

## Runtime-refactor packet template

Use the existing packet contract shape in `03-phase-slice-map.md`, specifically the
`A1.1d-5R3-MAC` section, rather than inventing a new workflow. Every planned implementation packet
must state:

- completion claim, exact predecessor, exact successor, and authority stop;
- owned findings/PI rows/contracts and explicitly preserved completed proof;
- exact production path and symbol allowlist;
- exact test/fixture allowlist and run-only frozen paths;
- documentation/control surface;
- required GitNexus/manual impact, focused checks, native/non-native proof, allowlist, secret, and
  change-detection gates;
- platform/privilege/publication posture;
- explicit non-goals and hard stops.

Apply `04-contracts-and-gates.md` as the normative template for skill invocation, selected-outcome
freezing, priority classification, review independence, subject fingerprinting, the bounded causal
review budget, P3/P4 inventory, and machine-valid review records.

## Required planning artifacts

Create exactly:

- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/PLAN.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/TASKS.md`.

The three files must together include:

1. verified current state and root-cause analysis;
2. a hunk-level salvage/discard matrix for every donor path and materially changed symbol;
3. explicit retention of the proven Bash 3.2 FD, macOS shell-compile, and expected-project-ID
   corrections without dragging in pairing-prototype hunks;
4. an explicit decision and contract for distinct logical data and operator-TTY sessions joined to
   the same PM-bound SSH transport, including identity, authentication, lifetime, retry, replay,
   transcript, and confirmation semantics—or a smaller compliant design proved from current
   control-pack authority;
5. an explicit trusted mapped-lifecycle submit/bootstrap bridge replacing raw `lima-action`, with
   exact authority carrier, ordering, admission, and fail-closed behavior;
6. a minimum necessary Keychain/XPC/P-256/ticket/Stage-1 surface, with every retained donor hunk
   justified and every unapproved `lima-stdio-v1` exclusivity assumption removed;
7. no Windows behavior changes and only demonstrably compatibility-preserving Linux adaptations;
8. ordered, independently landable recovery packets small enough for causal review, with exact
   commit/publication/evidence successors;
9. acceptance criteria and proof matrices sufficient for installation/readiness and the later
   official native evidence task;
10. rollback/abandonment rules, hard scope ceilings, and terminal status mapping;
11. a dependency graph and a TODO checklist whose items are individually pass/fail verifiable;
12. exact instructions for recovering from the donor by audited hunk selection into a fresh
    exact-base worktree, never by publishing the donor wholesale.

Integrate the planning authority with minimal append-only/current-status changes to:

- `llm-last-mile/runtime-refactor/00-README.md`;
- `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`.

`01-target-architecture.md` is frozen. `06-review-finding-inventory.md` may change only to add or
deduplicate valid unfixed P3/P4 findings. Do not rewrite completed Linux or landed MAC history.

## Exact review set and causal cascading process

Create exactly:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-planning-review-cycle-record.json`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-planning-review-authority-security.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-planning-review-lifecycle-convergence.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-planning-review-allowlist-evidence.md`.

Run the repository-defined causal cascading process exactly:

1. freeze the full planning subject and fingerprint after deterministic checks;
2. run one fresh complete-subject discovery review or same-fingerprint burst with disjoint
   authority/security, lifecycle/transport/convergence, and allowlist/evidence/recovery lenses;
3. validate and classify findings against the selected outcome; remediate every valid P1/P2 in one
   consolidated documentation pass; record but do not automatically remediate P3/P4;
4. validate `--next-cycle closure`, then use a different fresh read-only closure reviewer on the
   remediated subject/delta;
5. allow at most two supplemental causal cycles only for P1/P2 directly caused or unmasked by the
   immediately preceding remediation, with unchanged authority/risk and explicit causal evidence;
6. `CLEAN` is terminal; unresolved P1/P2, unrelated scope expansion, or exhausted budget returns a
   bounded blocked receipt and prohibits publication;
7. validate the JSON record with
   `python3 llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py <record>` after
   every cycle and before each next cycle.

Review subagents are read-only, fresh, and receive no author reasoning or success-asserting
summary. All task subagents use `gpt-5.6-terra` with Extra High reasoning under the existing user
override. Do not use the planning author as its own independent reviewer.

## File allowlist

Only the three new recovery planning files, the five named current control-pack files, optional
P3/P4-only `06-review-finding-inventory.md`, and the four exact review-set files may change.
Product source, tests, manifests, scripts, lockfiles, `.agents`, orchestration state, and every
other path are forbidden.

## Verification and publication

Before review and again before publication:

- verify exact base/tree/ref, clean index/worktree, and unchanged protected checkout;
- re-hash and inventory the donor read-only;
- run Markdown/link/reference checks available in the repository;
- run `git diff --check`;
- prove the exact path allowlist;
- scan the changed bytes for secrets and accidental machine-private data;
- validate every review-cycle transition and the terminal record;
- run `gitnexus_detect_changes()` and manually review documentation/control-pack reachability;
- prove zero unresolved valid P1/P2 and inventory every valid unfixed P3/P4.

Publication mode is `remote`. If and only if the final record is `CLEAN`, create exactly one
docs/review commit and normally fast-forward push it to
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Re-read the live remote
immediately before publication and require it still equals the expected base. Never merge,
rebase, reset, cherry-pick, amend, rewrite, force-push, or publish an orchestration branch.

On success, return `LANDED_CLEAN` with next increment exactly
`AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION`. On any block, preserve the
worktree and return the exact non-advancing status. Do not dispatch implementation, installation,
native evidence, MAC closeout, WIN, UNIX, or any other successor.
