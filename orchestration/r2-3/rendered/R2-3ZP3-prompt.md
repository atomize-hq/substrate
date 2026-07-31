Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZP3. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 745771f218d81aed279861b84b759bba3b5d85fbe4e8afea95e655e5b59653d9
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZP3
- packet_id: A1.1d-5R2-3ZP3
- next_increment: R2-3ZT1

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the fresh Linux Codex worktree provisioned from the remote-tracking product ref

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- /home/spenser/.codex/worktrees/3b8f/substrate-r2-3
- /home/spenser/.codex/worktrees/acb8/substrate
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: ec05167bd7bcb7be874af48fe33d6edb71492516
- expected base tree: b0d399cfb09655d5ff61d47ec68408e52ba5cfec
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZP3

This is a fresh proof-infrastructure prerequisite before `R2-3ZT1` and the final fresh
`R2-3ZR1` landing task. It owns only the canonical shell-wall runner's pre-status liveness,
containment, evidence-finalization, and teardown repair. It must not resume or mutate the preserved
R2-3ZR1 product WIP.

This top-level task and every subagent must run at Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Land one security-preserving canonical-runner infrastructure commit that converts the reproduced
`stage_a_liveness_failed` plus `mount_teardown_failed` pre-status failure into eligible finalized
provenance in authenticated serial and parallel runs. The known 46th shell-test failure is a
separate, later test-helper increment and must remain untouched here.

BOUND SOURCE

- commit: `ec05167bd7bcb7be874af48fe33d6edb71492516`
- tree: `b0d399cfb09655d5ff61d47ec68408e52ba5cfec`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`
- preserved donor worktree: `/home/spenser/.codex/worktrees/3b8f/substrate-r2-3`
- donor local commit/tree: `199e1b1a5f3854934a08727f2846e028c065c5cb` /
  `15e17d17fa1a9222488a695a0e4e360376e3556c`
- donor runner-only diff SHA-256:
  `de9a857e5d6341d2f2430ed15354355568fb45a10d34f294384fa905a8ce452e`
- prior receipt: `orchestration/r2-3/receipts/R2-3ZR1-attempt-8.json` on the meta branch
- prior proof root: `/home/spenser/r2-3zr1-runner-proof-final.l69l_x14`

Before editing, verify exact base/tree/ref/ancestor, clean task checkout, 0 ahead/0 behind, and the
current GitNexus index. Analyzer-only `AGENTS.md`/`CLAUDE.md` count churn must be restored to the
starting HEAD and excluded from publication. Any other pre-existing diff is a hard stop.

PRESERVED WIP BOUNDARY

The donor worktree is read-only evidence. Do not run commands there that can create or change
files. Do not restore, reset, clean, stage, commit, archive, remove, or hand it off. Read only the
two authorized file diffs and verify their combined diff hash before adopting them in the fresh
task checkout.

EXACT TRACKED FILE ALLOWLIST

- `scripts/ci/canonical_shell_wall_runner.py`
- `scripts/ci/test_canonical_shell_wall_runner.py`

No Rust, product test, governance, control-pack, platform, native-evidence, or other tracked file
may change.

AUTHORIZED DIAGNOSTIC SCOPE

Sanitized runner diagnostics may be written only into fresh external disposable proof roots.
Diagnostics may expose process state, authenticated protocol events, namespace identities,
pidfd/liveness transitions, mount identities, and teardown outcomes, but must not record secrets,
protected environment values, or weaken fail-closed behavior. Remove or disposition temporary
diagnostic instrumentation before publication unless it is a reviewed permanent non-sensitive
runner diagnostic covered by tests.

REQUIRED STARTING SUBJECT

Adopt the verified two-file donor patch, then diagnose and minimally repair the remaining failure:

- authenticated self-tests pass at 103/103;
- the prior `evidence_write_failed` signature is gone;
- all four base/head serial/parallel proofs stop before stage-A status with
  `stage_a_liveness_failed` plus `mount_teardown_failed`;
- `containment.jsonl` contains only `stage_a_started`;
- `stage_a_status_received=false`;
- no `summary.json` materializes; and
- the hidden backing directory is retained but empty.

Do not bypass liveness, containment, evidence-integrity, namespace, pidfd, mount, or teardown
checks. The fix must establish the missing authenticated transition and prove cleanup; it must not
relabel an incomplete run as eligible.

IMPACT AND SOURCE CLOSURE

Before editing each existing symbol, run GitNexus upstream impact. If Python symbol coverage is
missing, record that limitation and perform exact textual caller/state-machine closure. Warn on
HIGH/CRITICAL results before editing. Inspect every stage-A/stage-B status producer/consumer,
host recovery path, provenance merge/finalization path, and teardown verifier affected by the fix.

REQUIRED PROOF

1. `python3 -m py_compile` on both allowlisted files.
2. Focused RED/GREEN tests for the exact pre-status liveness and teardown failure.
3. Complete authenticated runner self-test suite, with no failures.
4. Disposable full non-worktree proof checkouts outside protected paths.
5. Authenticated authority-commit and ordinary-descendant runs in serial and parallel modes.
6. Every proof must finalize with `eligible=true` and materialize validated `provenance.json`,
   `summary.json`, `cargo.log`, `containment.jsonl`, `failure-names.txt`, and
   `normalized-signatures.txt`, with proven backing/mount/namespace teardown.
7. The current shell result may remain the independently classified known regression:
   `1322 discovered / 1276 passed / 46 failed / 0 ignored`, failure-name SHA-256
   `e46e0f4cc12e34f11007cfe992c0e2020400771b91e43ad8ddc7dbaabcb8a549`, and normalized-signature
   SHA-256 `415f1a0c37b3455ce2bf4b5a4d5e6a6e85d2f5c506e47d24692f69f83955fb0f`.
   This increment must not change that test set or bless it as the final R2-3 baseline; it proves
   only that the runner produces eligible, honest regression evidence.
8. `git diff --check`, exact two-file containment, and `gitnexus_detect_changes()` with all affected
   flows inspected or the tooling gap explicitly dispositioned.

AUTHORITY COMMIT AND PUBLICATION

Because runner bytes are authenticated, compute and include the exact five P1 trailers on the
final atomic commit:

- `P1-Bootstrap-Bytes`
- `P1-Bootstrap-SHA256`
- `P1-Host-Argv-Template-SHA256`
- `P1-Stage-A-Argv-Template-SHA256`
- `P1-Stage-B-Argv-Template-SHA256`

That commit becomes the reviewed runner authority commit. Prove the authority commit itself and an
ordinary disposable descendant with identical runner/test blobs. If post-commit proof fails, do
not amend, reset, or publish; send a blocked receipt preserving the local commit. Publish only by
one normal fast-forward push after all proofs and review pass and the live target still equals the
expected base.

REVIEW CONTRACT

Use fresh read-only GPT-5.4 Extra High reviewers at Standard/default speed covering:

1. stage-A/stage-B liveness, pidfd, namespace, mount, and teardown correctness;
2. evidence/provenance authentication, finalization, and anti-bypass properties; and
3. exact allowlist, P1 authority trailers, proof honesty, donor-WIP preservation, and non-expansion
   into the known retained-worker test failure.

Validate the review record. Publication requires terminal CLEAN, P1=0, P2=0, and complete P3/P4
disposition.

COMPLETION BOUNDARY

Success completes only `R2-3ZP3`, the proof-runner infrastructure prerequisite. It does not repair
the retained-worker test helper, does not land R2-3ZR1 product WIP, does not run native evidence,
does not execute R2-3Z, and does not complete the parent packet.

TERMINAL RECEIPT

Use `increment: "R2-3ZP3"`, `packet_id: "A1.1d-5R2-3ZP3"`, and
`next_increment: "R2-3ZT1"`. A LANDED_CLEAN receipt must include the landed commit/tree/ref, exact
two changed paths, subject fingerprint, five P1 trailers, eligible authority/descendant serial and
parallel proof records and hashes, validated review record/digest, GitNexus result, remote 0/0,
and clean task checkout. For failure, send exact evidence, required authority, and a complete
handoff prompt. Receipt send is the final tool action.

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

Do not generate the next increment prompt and do not begin R2-3ZT1. The meta
orchestrator owns independent verification and subsequent dispatch.
