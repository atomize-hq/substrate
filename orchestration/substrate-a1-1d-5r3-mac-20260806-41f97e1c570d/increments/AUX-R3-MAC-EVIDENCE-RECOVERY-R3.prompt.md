Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for AUX-R3-MAC-EVIDENCE-RECOVERY-R3. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-a1-1d-5r3-mac-20260806-41f97e1c570d
- dispatch_nonce: 436e79e59e5a06bdafa67f5b066c95934838835e4d127f57f4d8ec8ebadad3f3
- meta_thread_id: 019fd8c3-b12c-7e73-919f-898600ab0f64
- meta_host_id: local
- increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R3
- packet_id: AUX-R3-MAC-EVIDENCE-RECOVERY-R3
- next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R4

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

/Users/spensermcconnell/.codex/worktrees/9afb/substrate

Never mutate these protected checkouts:

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- /Users/spensermcconnell/.codex/worktrees/eb49/substrate
- /Users/spensermcconnell/.codex/worktrees/60cc/substrate
- /Users/spensermcconnell/.codex/worktrees/bacb/substrate
- /Users/spensermcconnell/.codex/worktrees/05f9/substrate
- /Users/spensermcconnell/.codex/worktrees/1958/substrate
- /Users/spensermcconnell/.codex/worktrees/8fd3/substrate
- /Users/spensermcconnell/.codex/worktrees/e2c8/substrate
- /Users/spensermcconnell/.codex/worktrees/abc3/substrate
- /Users/spensermcconnell/.codex/worktrees/a943/substrate
- /Users/spensermcconnell/.codex/worktrees/2fdb/substrate
- /Users/spensermcconnell/.codex/worktrees/46c9/substrate

Canonical starting state:

- publication mode: declared by the increment contract
- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: fc150213826b8b423f042b4591d6b8b1cb3ace20
- expected base tree: 02217ad2dbd106ae5befc60c8de862b6f3864647
- required ancestor: 270f6e55e1a94b7e2f9b2667e605980d2e50579c

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

# Increment contract — AUX-R3-MAC-EVIDENCE-RECOVERY-R3

## Objective

Implement only the trusted native-evidence project binding defined by the landed recovery plan.
The validator must accept an artifact only when its `product_project_id` equals a required,
dispatch-supplied expected project ID. The historical Linux artifact remains valid only when the
caller explicitly supplies its historical expected project ID.

## Exact source and successor

- Expected base: `fc150213826b8b423f042b4591d6b8b1cb3ace20`
- Expected tree: `02217ad2dbd106ae5befc60c8de862b6f3864647`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Publication: one normal fast-forward commit after terminal review `CLEAN`
- Sole successor: `AUX-R3-MAC-EVIDENCE-RECOVERY-R4`

## Exact path and symbol fence

Production:

- `scripts/ci/validate_r3_native_evidence.py`: only `validate_artifact`, `parse_args`, `main`, and
  the minimum direct argument plumbing necessary to make `--expected-product-project-id` required
  and compare it fail-closed with the artifact field.

Tests:

- `scripts/ci/test_validate_r3_native_evidence.py`: only exact valid, missing, mismatch, and
  historical-Linux-static-fixture cases for the caller-supplied expected project ID.

Append-only current-control documentation:

- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`

Append only the recovery-current authoritative invocation:

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`

The historical Linux check supplies `2ccb802f-301c-4af4-9bd5-51d22808f0a2`. Do not rewrite frozen
architecture, prior Linux evidence, or earlier review records.

Review metadata only:

- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-authority-security.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-lifecycle-convergence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-allowlist-evidence.md`
- `llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r3-review-cycle-record.json`

No other path or symbol is authorized.

## Donor boundary

The preserved donor `/Users/spensermcconnell/.codex/worktrees/60cc/substrate` is read-only evidence.
Inspect it if useful, but manually recreate only the narrow R3 behavior in the fresh task worktree.
Never copy the donor diff wholesale, mutate it, reset it, clean it, check it out, commit it, or use
it as a publication base.

## Mandatory skills and analysis

Load `.agents/skills/using-agent-skills/SKILL.md` first, then invoke the relevant repository-local
skills including `orchestrate-top-level-tasks`, `source-driven-development`,
`test-driven-development`, `incremental-implementation`, `code-review-and-quality`,
`security-and-hardening`, and `git-workflow-and-versioning`. Before editing any existing symbol,
run GitNexus impact analysis when the existing index can answer it and supplement with exact manual
call-site/CLI analysis. Do not run `npx gitnexus analyze`; its tracked metadata injection is
forbidden. Treat degraded/UNKNOWN GitNexus output as inconclusive, not as zero impact.

Every subagent and reviewer must use `gpt-5.6-terra` at Extra High reasoning under the explicit
user override. Editing ownership must be disjoint where practical. Reviewers are fresh, read-only,
and independent of the implementation they review.

## TDD and deterministic proof

Prove the old fixed-project behavior fails the new caller-bound contract before the implementation
change, then make the narrow tests pass. Required proof includes:

1. `python3 -m unittest -v scripts.ci.test_validate_r3_native_evidence`.
2. Direct missing `--expected-product-project-id` rejection.
3. Direct mismatched expected project ID rejection.
4. Historical Linux artifact validation only with its explicitly supplied historical project ID.
5. A check that every recovery-current authoritative validator command includes the required flag.
6. Python syntax/compile, exact path/symbol allowlist, changed-byte secret scan, and `git diff --check`.

This is non-native validator work. Do not install, build platform targets, run lifecycle actions,
create official evidence artifacts, mutate Keychain/XPC/Lima/system state, or perform Linux/Windows
runtime work. The historical Linux fixture is read-only predecessor compatibility, not Linux work.

## Review and publication

Freeze the exact five-path product/test/documentation subject, excluding review metadata. Run the
repository-defined causal cascading review process: one discovery review or same-subject
authority/security, lifecycle/convergence, and allowlist/evidence burst; remediate valid P1/P2;
then one different fresh closure review. At most two supplemental causal cycles are allowed only
for a P1/P2 directly caused or unmasked by the immediately preceding remediation under unchanged
authority and risk. `CLEAN` ends the loop. Valid P3/P4 are recorded in
`06-review-finding-inventory.md` and do not create remediation/review cycles; because that inventory
path is outside this packet, stop for meta adjudication if an entry is required.

Validate the review-cycle record with
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. Run change detection with
the existing index if available and exact manual fallback otherwise. Before publication, require
the live target still equals the expected base, zero open P1/P2, exact nine-path final inventory,
and clean checks. Publish exactly one normal fast-forward commit. Never force-push, merge, rebase,
reset, or clean.

## Stops

Return `BASE_DRIFT` for remote/base/tree movement. Return `BLOCKED_CONTRADICTION` for identity,
donor, or exact-fence contradictions. Return `BLOCKED_SCOPE_EXPANSION` if implementation needs an
unlisted path/symbol, native action, new authority carrier, or Linux/Windows runtime work. Return
`BLOCKED_REVIEW` for unresolved P1/P2 or invalid review lineage. Preserve the task and worktree on
every stop; never request archival.

On success return `codex.top-level-task-receipt.v1` with `LANDED_CLEAN` and
`next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R4`. Sending the receipt to meta thread
`019fd8c3-b12c-7e73-919f-898600ab0f64` on host `local` must be the final tool action.

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

Do not generate the next increment prompt and do not begin AUX-R3-MAC-EVIDENCE-RECOVERY-R4. The meta
orchestrator owns independent verification and subsequent dispatch.
