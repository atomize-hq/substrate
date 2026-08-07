META IDENTITY BINDING AND R1 START AUTHORITY

Bind this exact identity and begin only the bounded R1 increment now:

- task_thread_id: 019fdccb-4ac3-7232-bc2e-4204b7b777ae
- task_host_id: local
- exact_task_worktree: /Users/spensermcconnell/.codex/worktrees/8fd3/substrate
- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: 3db435048a170d726093482541effdb41b1f930bd2cd6d627048b285eacc15ab
- complete_skill_suite_source: /Users/spensermcconnell/.codex/worktrees/eb49/substrate/.agents/skills
- complete_skill_suite_target: /Users/spensermcconnell/.codex/worktrees/8fd3/substrate/.agents/skills
- complete_skill_suite_count: 25 skills / 54 files
- complete_skill_suite_sha256: e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282
- skill_suite_verified: true
- expected_base_commit: d4bd729de8d344dfc567aaa62203838f68c2c895
- expected_base_tree: 3d5005d38c83b68c0a9813c10cbe6cb11996aed0
- live_remote_equal: true
- start_authorized: true
- authorized_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R1
- R2_through_R6_authorized: false
- native_or_host_mutation_authorized: false

Load `.agents/skills/using-agent-skills/SKILL.md` first, then invoke every applicable hydrated
repository-local skill named by the contract. Use `gpt-5.6-terra` with Extra High (`xhigh`) for
every subagent and reviewer. Preserve the donor, all protected worktrees, and this task worktree on
any block. Complete R1 and its causal review only; do not dispatch or begin R2.

The full rendered increment prompt follows.

Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for AUX-R3-MAC-EVIDENCE-RECOVERY-R1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: 3db435048a170d726093482541effdb41b1f930bd2cd6d627048b285eacc15ab
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R1
- packet_id: AUX-R3-MAC-EVIDENCE-RECOVERY-R1
- next_increment: AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-R2

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

/Users/spensermcconnell/.codex/worktrees/8fd3/substrate

Never mutate these protected checkouts:

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- /Users/spensermcconnell/.codex/worktrees/eb49/substrate
- /Users/spensermcconnell/.codex/worktrees/60cc/substrate
- /Users/spensermcconnell/.codex/worktrees/e2c8/substrate
- /Users/spensermcconnell/.codex/worktrees/abc3/substrate
- /Users/spensermcconnell/.codex/worktrees/a943/substrate
- /Users/spensermcconnell/.codex/worktrees/2fdb/substrate
- /Users/spensermcconnell/.codex/worktrees/46c9/substrate

Canonical starting state:

- publication mode: declared by the increment contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: d4bd729de8d344dfc567aaa62203838f68c2c895
- expected base tree: 3d5005d38c83b68c0a9813c10cbe6cb11996aed0
- required ancestor: d4bd729de8d344dfc567aaa62203838f68c2c895

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

# AUX-R3-MAC-EVIDENCE-RECOVERY-R1 contract

## Selected outcome

Implement and remotely publish **R1 only** from the landed recovery plan at `d4bd729de8d344dfc567aaa62203838f68c2c895`: make
`scripts/substrate/dev-install-substrate.sh` read its canonical install-bootstrap context under
macOS `/bin/bash` 3.2 without allocating, overwriting, or closing a caller-owned file descriptor,
and prove that behavior with the isolated installer regression. Do not begin or partially implement
R2, R3, R4, R5, R6, host preparation, installation, or native evidence.

## Authoritative source truth

Read from the exact expected base before work:

- `AGENTS.md`;
- `.agents/skills/using-agent-skills/SKILL.md` and every skill it selects;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/PLAN.md`;
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/TASKS.md`;
- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`;
- `llm-last-mile/runtime-refactor/06-review-finding-inventory.md`;
- `llm-last-mile/runtime-refactor/review-control/README.md`;
- `llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`.

Use the repository-local skills only. Load `using-agent-skills` first and apply at minimum
`source-driven-development`, `test-driven-development`, `incremental-implementation`,
`debugging-and-error-recovery`, `code-review-and-quality`, and `git-workflow-and-versioning`.
Record the loaded paths and how each constrained R1.

## Exact donor boundary

The preserved donor is read-only evidence:

- worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`;
- donor base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` /
  `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`;
- tracked binary-diff SHA-256:
  `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`;
- R1 untracked test SHA-256:
  `2de2da832cbb38c1722d2075ffcef781d7bb9d23a76318cfd6d3b54fee0b0664`.

Re-hash and inventory the donor before relying on it. Never mutate, stage, restore, checkout,
reset, clean, commit, archive, or execute tests in the donor. Manually recreate only the R1 lines
in the fresh task worktree. A matching path does not authorize any other donor line. If the donor
identity or hashes differ, return `BLOCKED_CONTRADICTION` without edits.

## Exact implementation boundary

### Completion claim

The installer reads its canonical bootstrap context under `/bin/bash` 3.2 without allocating or
closing a caller descriptor; context argv/environment remain canonical and no bootstrap temporary
file remains.

### Production allowlist

- `scripts/substrate/dev-install-substrate.sh`: only
  `resolve_install_bootstrap_context`, limited to the Bash-3.2-compatible descriptor lifecycle.

### Test allowlist

- `tests/installers/dev_install_bash32_fd_regression.sh`: only caller-owned FD survival,
  canonical context argv/environment, and no-bootstrap-temp assertions using a temporary fixture
  prefix.

### Review artifact allowlist

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-cycle-record.json`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-authority-security.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-lifecycle-convergence.md`;
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r1-review-allowlist-evidence.md`.

The exact final changed-path set is the two implementation/test paths plus those four review
artifacts. `06-review-finding-inventory.md` may change only if a valid unfixed P3/P4 is discovered;
otherwise it must remain byte-identical. No other path may change.

### Frozen and prohibited surface

All `tests/mac/*`, lifecycle binaries, manifests, lockfiles, common pairing contracts, shell
lifecycle clients, Keychain/XPC/Lima code, R3 control documents, `.agents`, orchestration state,
Windows paths, ordinary Linux host paths, evidence artifacts, and every R2-R6 path/symbol are
frozen. No install, Keychain, XPC, Lima, publisher, code-signing, pairing, native evidence, or host
mutation may run. No lifecycle binary build/copy, selector, transport, principal, or pairing change
is authorized.

## Impact and implementation workflow

Before editing `resolve_install_bootstrap_context`, run the repository-required GitNexus upstream
impact analysis and record direct callers, affected processes, and risk. If GitNexus is degraded,
refresh only as allowed by the repository and supplement with manual shell caller analysis. Any
HIGH/CRITICAL result or required unlisted symbol/path is a closed `BLOCKED_SCOPE_EXPANSION` unless
it is solely the already-authorized R1 Bash descriptor surface and can be justified inside this
exact fence. Do not edit first and justify later.

Use test-first or regression-first development. One editing subagent may own the two allowed
implementation paths; all reviewers are fresh and read-only. Every subagent and reviewer must use
`gpt-5.6-terra` with Extra High (`xhigh`) reasoning under the explicit user override. No reviewer
may review work it authored.

## Required deterministic proof

Run and record at minimum:

1. `/bin/bash tests/installers/dev_install_bash32_fd_regression.sh` on macOS;
2. `/bin/bash -n scripts/substrate/dev-install-substrate.sh`;
3. `git diff --check`;
4. exact path and symbol allowlist verification;
5. focused changed-byte secret/private-data scan;
6. a manual diff proving no build/copy, Windows, Linux-runtime, lifecycle, or native surface changed;
7. `gitnexus_detect_changes()` before publication and manual review of any degraded/unknown result.

The regression may operate only inside its temporary fixture prefix and must restore/remove its
fixture. Do not perform a real install.

## Causal cascading review

Freeze the exact full implementation subject and fingerprint after deterministic checks. Run one
fresh discovery review (or same-fingerprint burst) using all three read-only lenses:

1. authority/security: descriptor ownership, argv/environment integrity, injection, secrets;
2. lifecycle/convergence: FD lifetime, cleanup, error/interrupt behavior, temporary-file absence;
3. allowlist/evidence: exact path/symbol fence, Bash 3.2 execution, proof sufficiency, platform
   non-expansion.

Classify against the selected R1 outcome. Remediate valid P1/P2 only in one consolidated pass.
Record valid P3/P4 in `06-review-finding-inventory.md` without creating remediation cycles. Validate
`--next-cycle closure`, then use a different fresh read-only reviewer for closure on the remediated
subject/delta. Permit at most two supplemental causal cycles only for P1/P2 directly caused or
unmasked by the immediately preceding remediation under unchanged authority and risk. `CLEAN` is
terminal. Unresolved P1/P2, unrelated findings, invalid review records, or exhausted budget return
`BLOCKED_REVIEW` or `BLOCKED_SCOPE_EXPANSION` and prohibit publication.

Validate the review-cycle JSON with
`python3 llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py <record>` after every
cycle and before publication. Preserve all pre-existing R3 review records byte-for-byte.

## Publication and terminal contract

Publication mode is `remote`. Immediately before publication, require the live remote target still
equals `d4bd729de8d344dfc567aaa62203838f68c2c895` and the exact tree is `3d5005d38c83b68c0a9813c10cbe6cb11996aed0`. If and only if every gate and terminal review are
CLEAN, create exactly one normal commit and fast-forward push it to `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`. Never merge,
rebase, reset, clean, cherry-pick, amend, rewrite, or force-push. Verify the live remote equals the
new commit, ahead/behind is 0/0, and the task worktree/index are clean.

Return `LANDED_CLEAN` with `next_increment` exactly `AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-R2`. A generated successor is not start
authority. On any block, preserve the task and worktree byte-for-byte, return the exact blocked
receipt with a complete handoff, and do not request archival. Do not dispatch R2 or any successor.

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
complete continuation/handoff prompt.

The send_message_to_thread call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-R2. The meta
orchestrator owns independent verification and subsequent dispatch.
