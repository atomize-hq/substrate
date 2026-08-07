META IDENTITY BINDING AND START AUTHORITY

Bind this exact identity and begin the bounded planning increment now:

- task_thread_id: 019fdc95-705b-7cd0-92a3-8980c09c8193
- task_host_id: local
- exact_task_worktree: /Users/spensermcconnell/.codex/worktrees/e2c8/substrate
- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: c1f964d03d31a82ad5bf8b15a61b1a90cb618e20199b83b5674c6552a38a1798
- complete_skill_suite_source: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.agents/skills
- complete_skill_suite_target: /Users/spensermcconnell/.codex/worktrees/e2c8/substrate/.agents/skills
- complete_skill_suite_count: 25 skills / 54 files
- complete_skill_suite_sha256: e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282
- skill_suite_verified: true
- expected_base_commit: d8a65fc8890dd37584aaeac2984c906188e5f06e
- expected_base_tree: d8f25cc993f264f0c67fe3e00180eb393e41b2e3
- live_remote_equal: true
- start_authorized: true
- product_implementation_authorized: false

Load `.agents/skills/using-agent-skills/SKILL.md` first, then invoke every applicable repo-local
phase skill named by the contract. The SPEC workflow does not pause for user approval in this
orchestration: complete SPEC, PLAN, TASKS, and the bounded causal review sequence in this one
docs-only run. Do not implement the plan or dispatch its successor.

The full rendered increment prompt follows.

Use the repository-local $orchestrate-top-level-tasks skill and every skill required by the
increment contract. Never use or install a global copy. This initial prompt is self-contained;
before identity binding, the meta orchestrator must hydrate and verify the complete repository-
local suite at `.agents/skills` in the assigned worktree. If the bound follow-up arrives while that
local suite is absent, incomplete, or its persisted digest does not match, stop with
BLOCKED_CONTRADICTION before any work.

ROLE

You are the fresh top-level increment orchestrator for AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: c1f964d03d31a82ad5bf8b15a61b1a90cb618e20199b83b5674c6552a38a1798
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- increment: AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN
- packet_id: AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN
- next_increment: AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

/Users/spensermcconnell/.codex/worktrees/e2c8/substrate

Never mutate these protected checkouts:

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- /Users/spensermcconnell/.codex/worktrees/eb49/substrate
- /Users/spensermcconnell/.codex/worktrees/60cc/substrate
- /Users/spensermcconnell/.codex/worktrees/abc3/substrate
- /Users/spensermcconnell/.codex/worktrees/a943/substrate
- /Users/spensermcconnell/.codex/worktrees/2fdb/substrate
- /Users/spensermcconnell/.codex/worktrees/46c9/substrate

Canonical starting state:

- publication mode: declared by the increment contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: d8a65fc8890dd37584aaeac2984c906188e5f06e
- expected base tree: d8f25cc993f264f0c67fe3e00180eb393e41b2e3
- required ancestor: d8a65fc8890dd37584aaeac2984c906188e5f06e

Before editing, query the publication-mode authority: the live remote for `remote`, or the exact
local branch ref and worktree for `local`. Verify the exact base, tree, ancestry, cleanliness, and
any required index state. If they differ, send BASE_DRIFT or BLOCKED_CONTRADICTION. Do not
reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

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

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run required change detection.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

For publication mode `remote`, when authorized by the increment contract, create one atomic commit
and perform a normal fast-forward push of HEAD to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live
remote equals the landed commit, refresh required indexes, and finish clean.

For publication mode `local`, create exactly the contract-authorized two-commit chain:
`expected base -> reviewed work commit -> mechanical control commit`. Keep the isolated branch and
worktree clean. Do not merge or push. Record the local branch ref, worktree, both commits/trees,
both exact changed-file inventories, their sorted union, and an unchanged remote observation.

Send a codex.top-level-task-receipt.v1 message to the meta task. For remote success, use
LANDED_CLEAN. For local success, use CLOSED_CLEAN. Include your bound increment-task thread/host
IDs, expected base, exact publication identity, changed paths, subject fingerprint, validated
review record and digest, finding disposition, checks, change detection, clean status, and next
increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt. Preserve the task-assigned worktree exactly; do not clean,
reset, discard, or request archival of a blocked task.

The send_message_to_thread call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION. The meta
orchestrator owns independent verification and subsequent dispatch.
