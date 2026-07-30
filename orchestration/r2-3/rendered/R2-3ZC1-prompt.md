Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZR1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 6346ed3b1bb04ac225c1e32ca94ce0d3fdf1c859f1b4b5eb2d2a993f3cfc9f5b
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZR1
- packet_id: A1.1d-5R2-3ZC1
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

CURRENT AUTHORIZED DIAGNOSTIC: A1.1d-5R2-3ZC1

This is a fresh, read-only classification task supporting blocked increment `R2-3ZR1`. It is not
a product increment, does not advance the authoritative sequence, and has no mutation or
publication authority.

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- gitnexus-debugging
- gitnexus-exploring
- code-review-and-quality

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

PURPOSE

Classify the shell-library wall reported by `R2-3ZR1-attempt-3` as `1221 passed / 101 failed`
without changing product, tests, fixtures, control documents, generated files, or the preserved
three-file WIP.

The result must answer:

1. which failures are members of the frozen, provenance-valid 45-failure baseline;
2. which failures arise only because the prior wall did not satisfy the canonical private-root
   invocation contract;
3. which failures are genuine regressions caused by accepted R2-3 transitions;
4. which failures are unrelated repository drift or pre-existing failures; and
5. the exact minimal file/test allowlist for each bounded repair packet that is actually needed.

SOURCE AND CHECKOUT BOUNDARIES

Use a fresh task-assigned worktree at:

- commit `3a6490ff391453535168491b3f8d26adf9e20938`;
- tree `fca816eaed26407ea89d352c7a087a201391bd49`;
- target `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`; and
- required ancestor `0f1e147fb735791b44a65099a65167cbdc1803af`.

Before analysis, verify exact base/tree/ref/ancestor, clean checkout, 0 ahead/0 behind, and a
current GitNexus index for the exact worktree. GitNexus may update only generated count blocks in
`AGENTS.md` and `CLAUDE.md`; if and only if the diff is confined to those count blocks, restore
those two files to the verified starting HEAD and record the before/after counts. Any other diff
is a hard stop.

The preserved WIP lives at:

`/home/spenser/.codex/worktrees/e847/substrate-r2-3`

It contains exactly these unstaged paths:

- `crates/shell/src/execution/host_inbox_materialization.rs`
- `crates/world-mac-lima/examples/mac_backend_smoke.rs`
- `crates/world-mac-lima/src/lib.rs`

That WIP worktree and its owning task must remain untouched and unarchived. You may inspect its
existing diff and recorded receipts read-only, but do not run Cargo, formatting, analyzers, or any
command there that could create or modify files. Do not reset, clean, restore, stage, commit,
archive, or remove it.

AUTHORITATIVE BASELINE FACTS

The expected broad shell wall was not zero failures.

The frozen provenance-valid canonical baseline is:

- `1309 discovered / 1264 passed / 45 failed / 0 ignored`;
- failure-name SHA-256
  `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`;
- normalized-signature SHA-256
  `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.

RP3 and RP4 preserved the same historical 45-failure membership; those failures were not
subsequently resolved or converted into an expected-zero wall. Current test discovery may be
higher because later increments added tests, so totals alone are never authoritative.

The historical omitted-root wall produced `1309 / 1205 / 104 / 0`; its 59 additional trusted-root
rejections were explicitly classified as an invocation mismatch, not product failures or baseline
additions. Therefore the `1221 / 101` result must not be interpreted as 56 new regressions merely
by subtracting 45.

Read only the relevant baseline/provenance sections:

- `llm-last-mile/runtime-refactor/00-README.md` around RP3/RP4;
- `02-seam-crosswalk.md` section “Broad-wall invocation provenance crosswalk”;
- `04-contracts-and-gates.md` section “canonical shell-library broad-wall invocation contract”;
- `05-debug-regression-ledger.md` RP3/RP4 closeout rows; and
- `orchestration/r2-3/receipts/R2-3ZR1-attempt-2.json` and
  `R2-3ZR1-attempt-3.json`.

CLASSIFICATION METHOD

1. Establish whether the `1221 / 101` command satisfied the canonical private-root invocation
   contract. Treat incomplete provenance as ineligible evidence.
2. Recover the exact failure names and normalized signatures from existing eligible artifacts,
   logs, or a fresh canonical read-only wall. Do not infer membership from totals.
3. If a fresh wall is necessary, run it only through the repository's canonical authenticated
   runner with a fresh validated private root and external evidence/output directories. Do not
   edit tracked files or real product state. Preserve command, root, toolchain, counts, names,
   signatures, hashes, and exit status.
4. Reproduce representative failures one focused causal group at a time. Prefer the smallest
   command that discriminates root cause.
5. Compare every observed failure name against the frozen 45-name set and compare signatures
   where available.
6. For each non-baseline failure, trace the first accepted commit/contract transition that could
   cause it. Classify it as:
   - `CANONICAL_BASELINE_45`;
   - `INVOCATION_PROVENANCE_MISMATCH`;
   - `R2_3_OWNED_REGRESSION`;
   - `UNRELATED_BASELINE_OR_REPOSITORY_DRIFT`;
   - `WIP_ALREADY_REPAIRED`; or
   - `UNRESOLVED_NEEDS_MORE_EVIDENCE`.
7. Group failures by one causal repair, not merely by file or error string.
8. For every proposed repair group, provide:
   - exact test names;
   - exact files and symbols/helpers;
   - pre-edit GitNexus risk/caller/process information;
   - accepted R2-3 transition or unrelated owner;
   - minimum proof commands;
   - explicit non-goals; and
   - whether the current three-file WIP should be preserved unchanged, narrowed, or incorporated.

Known representative failures from the blocked receipt include:

- `builtins/shim_doctor/report.rs` and `builtins/world_deps/mod.rs` expectation drift;
- trusted-root/legacy-authority preflight failures in `agent_runtime/auto_attach.rs`,
  `agent_runtime/tool_invocation_contract.rs`, `agents_cmd.rs`,
  `agent_runtime/retained_worker_runtime.rs`, and
  `execution/orchestrator_world_dispatch.rs`; and
- missing-live-parent behavior in `repl/async_repl.rs`.

Do not assume these share one cause.

PROHIBITED ACTIONS

- No tracked or untracked file edits in the fresh checkout or preserved WIP.
- No staging, commits, pushes, branches, merges, rebases, resets, cleanup, or publication.
- No control-pack reconciliation.
- No product lifecycle, provisioning, forwarding, VM, WSL, socket, process, PID, or artifact
  action.
- No test deletion, ignoring, filtering-around, expectation weakening, or source repair.
- No claim that the expected wall is zero failures.
- No authorization of any repair packet; classification only.
- Do not begin native evidence or `R2-3Z`.

DELIVERABLE

Write an external JSON classification artifact containing:

- source and task identity;
- exact provenance determination for the 101-failure wall;
- canonical 45 baseline facts and membership comparison;
- every causal group, classification, evidence, and confidence;
- exact candidate bounded repair packets/file/test allowlists;
- unresolved questions and required authority; and
- confirmation that both checkouts remain unchanged.

SHA-256 the artifact. Then send a `codex.top-level-task-receipt.v1` receipt to the meta task using:

- `increment: "R2-3ZR1"` (the blocked authoritative sequence increment);
- `packet_id: "A1.1d-5R2-3ZC1"`;
- `status: "BLOCKED_SCOPE_EXPANSION"` unless the evidence instead proves no repair is required;
- the bound task thread/host and dispatch nonce;
- the artifact path/digest;
- exact checks and classification summary;
- exact next authority decision; and
- `next_increment: "EVIDENCE:R2-3Z"`.

The receipt must state that no product increment landed and the remote remains unchanged. The
`send_message_to_thread` call must be the final tool action. After it succeeds, make no more tool
calls.

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
