Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZR1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: a49da6fa0a78421e40a221b5cea5f3290cdb154ceb3ba44faa331588c3e1f75a
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZR1
- packet_id: A1.1d-5R2-3ZP1
- next_increment: EVIDENCE:R2-3Z

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the fresh read-only Linux Codex worktree provisioned from the remote-tracking product ref

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- /home/spenser/.codex/worktrees/e847/substrate-r2-3
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 3a6490ff391453535168491b3f8d26adf9e20938
- expected base tree: fca816eaed26407ea89d352c7a087a201391bd49
- required ancestor: 0f1e147fb735791b44a65099a65167cbdc1803af

Before editing, fetch/query the live remote and verify the exact base, tree, ancestry, cleanliness,
0 ahead/0 behind, and current GitNexus index. If they differ, send BASE_DRIFT or
BLOCKED_CONTRADICTION. Do not reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

CURRENT AUTHORIZED DIAGNOSTIC: A1.1d-5R2-3ZP1

This is a fresh, bounded, read-only-first investigation supporting blocked increment `R2-3ZR1`.
It is not a product increment, does not advance the authoritative sequence, and has no source
editing or publication authority.

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Identify why `scripts/ci/canonical_shell_wall_runner.py` rejects the accepted R2-3 product source
with exit 65 `invocation_authority_invalid`, and produce the exact minimum test-harness-only repair
allowlist and proof contract needed to run an authenticated canonical shell wall on an ordinary
reviewed descendant commit.

Do not implement the repair. Do not run native evidence. Do not modify or archive the preserved
R2-3ZR1 WIP.

BOUND SOURCE

- commit: `3a6490ff391453535168491b3f8d26adf9e20938`
- tree: `fca816eaed26407ea89d352c7a087a201391bd49`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`

Before analysis, verify exact base/tree/ref/ancestor, clean checkout, 0 ahead/0 behind, and a
current GitNexus index for the exact checkout. If GitNexus changes only generated count blocks in
`AGENTS.md` and `CLAUDE.md`, restore only those blocks to the verified starting HEAD and record
the before/after counts. Any other diff is a hard stop.

PRESERVED WIP

The prior R2-3ZR1 worktree is:

`/home/spenser/.codex/worktrees/e847/substrate-r2-3`

It has exactly three unstaged paths:

- `crates/shell/src/execution/host_inbox_materialization.rs`
- `crates/world-mac-lima/examples/mac_backend_smoke.rs`
- `crates/world-mac-lima/src/lib.rs`

Treat that checkout as protected and read-only. Do not run Cargo, formatters, analyzers, tests, or
any command there that can create or change files. Do not reset, clean, restore, stage, commit,
archive, hand off, or remove its task/worktree.

ACCEPTED INPUT FACTS

The validated R2-3ZC1 classification established:

- the `1221 passed / 101 failed` wall is provenance-ineligible;
- the exact canonical baseline remains `1309 / 1264 / 45 / 0`;
- the exact 45-name and normalized-signature hashes remain
  `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
  `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbb50312a03f0042cdfac3`;
- a fresh run through `scripts/ci/canonical_shell_wall_runner.py` rejected both accepted-source
  and historical integration expected-head trials with exit 65
  `invocation_authority_invalid`; and
- the current product HEAD does not itself carry the reviewed P1 invocation-authority trailers,
  even though the runner bytes and their authority commit exist in its ancestry.

The classifier artifact is:

`/home/spenser/.codex/visualizations/2026/07/30/019fb35f-b602-72d1-b597-72e78063b60d/r2-3zc1-shell-wall-classification.json`

with SHA-256
`dabc8e7c65c26c4a383afdd644a36973374b0e41c98501858735ad678ea951a2`.

REQUIRED READ-ONLY ANALYSIS

Inspect only the minimum relevant sources and history:

- `scripts/ci/canonical_shell_wall_runner.py`
- `scripts/ci/test_canonical_shell_wall_runner.py`
- the canonical invocation/trailer contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- the RP3/RP4 runner and baseline records in `00-README.md`,
  `02-seam-crosswalk.md`, and `05-debug-regression-ledger.md`
- the exact commits/blobs/trailers that introduced and reviewed the runner
- the R2-3ZC1 classification artifact and receipt

Determine:

1. the exact function/site and exact predicate producing exit 65 for current HEAD;
2. every authority input used by that predicate: runner blob, expected-head, commit trailers,
   argv templates, source/object identity, Git metadata, toolchain, roots, process identity, and
   any other bound value;
3. whether the gate accidentally requires authority trailers on the current descendant HEAD
   instead of verifying the reviewed runner authority commit/blob in the descendant ancestry;
4. whether there is already a supported invocation for ordinary descendants that R2-3ZC1 missed;
5. whether the failure is implementation drift, invocation misuse, stale documentation, or a
   deliberate single-commit-only restriction;
6. the minimum safe model for descendant execution that preserves all original authentication
   properties;
7. the exact files, symbols, and tests a repair would require;
8. GitNexus upstream impact for every proposed edited symbol, including direct callers, affected
   processes, modules, and risk;
9. the minimum RED and GREEN proof matrix, including self-tests, negative substitutions,
   descendant commits, wrong blobs/trailers/templates, dirty Git state, and the final canonical
   shell wall; and
10. whether the repair belongs in R2-3ZR1, a separate prerequisite increment, or only the
    orchestration/control plane.

You may run diagnostic, self-test, and supported runner invocations only in the fresh task
checkout with all evidence, cache, target, and temporary roots outside the checkout. Do not run a
full canonical wall unless the existing unmodified runner accepts a documented supported
invocation. Stop before any source edit.

SECURITY INVARIANTS

A proposed repair must not weaken:

- exact source commit/tree/ref/ancestor binding;
- reviewed runner blob identity;
- immutable argv-template and substituted-argv verification;
- held Git/object/config/index authority;
- private-root ownership/type/mode/identity/ACL checks;
- repository/toolchain/cache read-only seeds and isolated writable runtime;
- parent/child peer credentials, pidfds, subreaper/reaping, and control/evidence descriptors;
- cleanup authority and proof of no live descendants;
- evidence hashes, eligibility status, or canonical failure-name/signature comparison; or
- fail-closed handling of dirty, substituted, unsigned, unrelated, or history-rewritten inputs.

The repair must not make arbitrary descendants self-authorizing merely because they contain the
runner file. It must anchor descendant eligibility in a reviewed immutable authority object or
equivalent causal proof.

PROHIBITED ACTIONS

- No source, test, fixture, script, control-document, generated-file, or manifest edits.
- No staging, commit, push, branch, merge, rebase, reset, cleanup, or publication.
- No shell/product repair.
- No native macOS or Windows evidence.
- No weakening or bypassing the runner gate.
- No claim that R2-3ZR1, R2-3Z, or the parent is complete.
- No archive or mutation of the preserved WIP task.

DELIVERABLE

Write an external JSON artifact containing:

- source/task identity and verified no-change state;
- the exact causal chain to `invocation_authority_invalid`;
- authoritative commits, blobs, trailers, templates, and predicates;
- supported-invocation determination;
- security-preserving descendant authority model;
- exact candidate repair file/symbol/test allowlist;
- GitNexus blast radius;
- RED/GREEN proof matrix;
- ownership recommendation; and
- exact authority required from the user.

SHA-256 the artifact. Send a `codex.top-level-task-receipt.v1` receipt to the meta task using:

- `increment: "R2-3ZR1"`
- `packet_id: "A1.1d-5R2-3ZP1"`
- blocked status appropriate to the result
- the bound task identity and nonce
- artifact path/digest
- exact findings and proposed allowlist
- confirmation of no changes/publication
- `next_increment: "EVIDENCE:R2-3Z"`

The receipt send must be the final tool action. After it succeeds, make no further tool calls.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run `gitnexus_detect_changes()` and inspect every affected flow.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

When authorized by the increment contract, create one atomic commit and perform a normal
fast-forward push of `HEAD` to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live remote equals the
landed commit, refresh GitNexus, restore analyzer-only generated count changes if necessary, and
finish clean.

Send a `codex.top-level-task-receipt.v1` message to the meta task. For success, use
`LANDED_CLEAN` and include your bound increment-task thread/host IDs, expected base, landed
commit/tree, changed paths, subject fingerprint, validated review record and digest, finding
disposition, checks, GitNexus result, clean status, and next increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The `send_message_to_thread` call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin EVIDENCE:R2-3Z. The meta
orchestrator owns independent verification and subsequent dispatch.
