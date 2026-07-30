Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZR1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: f48c1e95710b3f98aee943e2fe1ea85b21f7f5e8204ad9126c0126d72238674f
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZR1
- packet_id: A1.1d-5R2-3ZR1R1
- next_increment: EVIDENCE:R2-3Z

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
- /home/spenser/.codex/worktrees/e847/substrate-r2-3
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZR1R1

This is the fresh continuation of the already-authorized `R2-3ZR1` repair/reconciliation increment
after prerequisite `R2-3ZP2` landed. It owns `R2-3ZR1` only.

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Complete the bounded R2-3ZR1 repair on the new descendant-authority base:

1. remove the remaining contextless macOS `MacLimaBackend` constructor surface;
2. migrate its smoke example and colocated tests to explicit typed carrier/mapping input;
3. land the already-authorized trusted-root hardening for the focused host-inbox test helper if
   fresh causal verification still supports it;
4. reconcile the already-authorized M4/F/R/S1 amendments, ZWR1, ZP2, and the R2-3 closeout history;
5. prove no new canonical shell-wall failures beyond the frozen 45-failure baseline; and
6. publish a clean source checkpoint suitable for fresh macOS and Windows evidence.

This is not authority to repair the canonical 45 baseline, broaden shell test work, activate
platform lifecycle, or begin native evidence.

BOUND SOURCE

- commit: `ec05167bd7bcb7be874af48fe33d6edb71492516`
- tree: `b0d399cfb09655d5ff61d47ec68408e52ba5cfec`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`
- reviewed descendant-authority commit: `ec05167bd7bcb7be874af48fe33d6edb71492516`

Before any edit, verify the exact base/tree/ref/ancestor, clean checkout, 0 ahead/0 behind, current
GitNexus index for the exact checkout, and exact P1 authority trailers on the base. If GitNexus
changes only generated count blocks in `AGENTS.md` and `CLAUDE.md`, restore only those blocks to
the verified starting HEAD and record the before/after counts. Any other pre-existing diff is a
hard stop.

PRESERVED WIP AND RECOVERY CONTRACT

The prior blocked worktree is:

`/home/spenser/.codex/worktrees/e847/substrate-r2-3`

It remains protected, read-only, and unarchived at:

- commit `3a6490ff391453535168491b3f8d26adf9e20938`
- tree `fca816eaed26407ea89d352c7a087a201391bd49`
- exactly three unstaged paths:
  - `crates/shell/src/execution/host_inbox_materialization.rs`
  - `crates/world-mac-lima/examples/mac_backend_smoke.rs`
  - `crates/world-mac-lima/src/lib.rs`

The exact binary diff of those three paths at dispatch has SHA-256:

`1caf269482b8baa81b02b86dc8a0c9135ac67969dbd44fce1631234be9826c9b`

You may perform read-only `git status`, `git rev-parse`, and `git diff --binary` operations against
that checkout solely to verify and capture the exact patch. Do not run Cargo, formatters,
analyzers, tests, or commands there that create or change files. Do not reset, clean, restore,
stage, commit, rebase, merge, cherry-pick, hand off, archive, or remove it.

After verifying the exact old-base identity, three-path status, and patch SHA-256, apply the patch
to the fresh task checkout at the new base. Resolve any conflict by re-deriving the intended
authorized change from source, never by changing prerequisite ZP2 runner/governance semantics.
Record whether the patch applied byte-for-byte or required bounded re-derivation.

EXACT PRODUCT/HARNESS ALLOWLIST

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/examples/mac_backend_smoke.rs`
- `crates/shell/src/execution/host_inbox_materialization.rs`

Inside `world-mac-lima/src/lib.rs`, changes are limited to:

- removing public contextless `MacLimaBackend::new`;
- removing public contextless `MacLimaBackend::new_with_vm_name`;
- removing `Default` only because it delegates to the removed ambient constructor;
- mechanically migrating colocated tests/private helpers to `new_with_mapping`; and
- minimum import/private cleanup made unreachable by those removals.

`MacLimaBackend::new_with_mapping`, typed validation, transport, forwarding, session lifecycle,
drop/cleanup, and every R3-owned body must remain behaviorally and byte stable except unavoidable
line movement.

Inside `mac_backend_smoke.rs`, changes are limited to:

- explicit CLI input for one encoded `InstallBootstrapContextCarrierV1`, one encoded
  `PlatformBootstrapMappingV1`, and one absolute project directory;
- shared typed decode/validation; and
- construction only through `MacLimaBackend::new_with_mapping`.

The example must not derive authority from environment, CWD, repository paths, defaults, Lima
availability, or ambient host state.

Inside `host_inbox_materialization.rs`, changes are limited to the private `#[cfg(test)]`
`tests::with_store` helper:

- select `XDG_RUNTIME_DIR` or `HOME/.cache` as an existing trusted parent;
- create the parent if necessary;
- use `tempfile::tempdir_in`;
- set Unix mode `0700`; and
- install that explicit root through the existing test guard.

No production body, StateStore preflight, authority-root validation, error behavior, or other test
helper may change. Retain this change only if the exact focused pre-edit failure and post-edit
success prove its causal purpose. Otherwise omit it and explain why.

CONTROL-PACK RECONCILIATION ALLOWLIST

- `llm-last-mile/runtime-refactor/00-README.md`
- `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `llm-last-mile/runtime-refactor/06-review-finding-inventory.md`

Record only verified history:

- the explicitly authorized M4 `scripts/mac/lima-doctor.sh` expansion;
- the explicitly authorized F `world_ops.rs` expansion;
- the explicitly authorized R `state.rs`, `auto_sync.rs`, `cli.rs`, and
  `world-backend-factory` expansions;
- the S1 `world-backend-factory` final-cutover handoff;
- ZWR1 commit `3a6490ff391453535168491b3f8d26adf9e20938` and its exact repair boundary;
- ZP2 commit `ec05167bd7bcb7be874af48fe33d6edb71492516`, its five-file authority repair,
  exact P1 trailers, descendant proof, and frozen canonical wall; and
- R2-3ZR1's exact completion and non-claims.

Do not rewrite prior findings, retroactively broaden semantic claims, close R2-3Z, or alter R3
ownership.

FROZEN/PROHIBITED FILES

Do not edit:

- `scripts/ci/canonical_shell_wall_runner.py`
- `scripts/ci/test_canonical_shell_wall_runner.py`
- any manifest or `Cargo.lock`
- any other Rust, shell, PowerShell, installer, platform, replay, shim, trace, or generated file.

Analyzer-only `AGENTS.md`/`CLAUDE.md` churn must be restored and excluded.

SOURCE CLOSURE AND IMPACT

Before editing any existing symbol:

1. run file-qualified upstream GitNexus impact and report direct callers, affected processes,
   modules, and risk;
2. warn before every HIGH/CRITICAL edit;
3. prove every source caller of `MacLimaBackend::new` and `new_with_vm_name` is inside the exact
   allowlist or already typed;
4. prove no macOS/Windows contextless platform-factory consumer remains;
5. freeze/hash `new_with_mapping`, typed mapping validation, transport/forwarding lifecycle,
   PI-077/PI-078 guards, trace writer/rotation/retention, and accepted factory/shim/replay bodies;
6. verify the ZP2 runner/test blobs are unchanged from the authority commit; and
7. build the exact historical commit/path ledger used for reconciliation.

Any out-of-allowlist caller or required production change is a scope-expansion stop.

CANONICAL WALL AUTHORITY

The canonical expected result is not zero failures. It is exactly:

- `1309 discovered`
- `1264 passed`
- `45 failed`
- `0 ignored`
- failure-name SHA-256:
  `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`
- normalized-signature SHA-256:
  `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`

Those 45 failures are the accepted baseline: 23 in
`execution::orchestrator_world_dispatch` and 22 in `repl::async_repl`. Do not edit, disable,
filter, or claim to repair them.

Run the final authenticated canonical wall from a disposable full non-worktree checkout at the
local final commit, using ZP2's reviewed authority commit as the authenticated ancestor. Linked
worktree support remains out of scope. A count/name/signature mismatch is a blocked result.

REQUIRED PROOF

Run at minimum:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p world-mac-lima`;
- `cargo test --locked -p world-mac-lima -- --nocapture`;
- `cargo clippy --locked -p world-mac-lima --all-targets -- -D warnings`;
- an external copied-source/cfg supplement harness that executes the macOS-gated constructor
  tests on Linux without behavior substitution;
- `cargo check --locked -p shell`;
- focused `cargo test --locked -p shell host_inbox_materialization -- --nocapture`;
- exact source closure and frozen-body/blob hashes;
- proof that manifests and `Cargo.lock` are unchanged;
- `git diff --check` and `git diff --cached --check`;
- exact allowlist checks; and
- `gitnexus_detect_changes()` with every affected flow inspected.

Attempt Apple-target static compilation honestly. Report absent SDK/toolchain as an environment
limitation, never as native evidence.

After a local final commit and before publication:

- create a disposable full non-worktree checkout;
- verify the ZP2 authority commit is in ancestry;
- verify runner/self-test blob continuity;
- run canonical self-test and the canonical shell wall;
- require the exact frozen result above.

REVIEW

Build a deterministic staged subject manifest and SHA-256 fingerprint. Use fresh independent
read-only GPT-5.4 Extra High reviewers on Standard/default speed covering:

1. constructor removal, typed mapping security, and no lifecycle activation;
2. trusted-root test-helper causality, production behavior freeze, and canonical baseline honesty;
3. historical reconciliation, exact allowlist, ZP2 authority continuity, and R2/R3 ownership.

Validate the review record. Publication requires CLEAN, P1=0, P2=0, and complete P3/P4
disposition. Any valid finding must be remediated only inside the allowlist and followed by a new
fingerprint/closure review.

PUBLICATION

Before commit and again before push, require the live target still equals
`ec05167bd7bcb7be874af48fe33d6edb71492516` with tree
`b0d399cfb09655d5ff61d47ec68408e52ba5cfec`.

Create one Conventional Commit and run the final authenticated proof from its disposable full
checkout. If proof fails after commit, do not amend, reset, or publish; return a blocked receipt.
If proof is clean, normal fast-forward push only. Verify remote commit/tree, clean checkout, 0/0,
and refreshed GitNexus.

COMPLETION BOUNDARY

Success completes only R2-3ZR1:

- no contextless macOS backend constructors;
- explicit typed smoke/example construction;
- bounded focused test-helper hardening if causally retained;
- accurate control-pack reconciliation; and
- a canonical-wall-clean source checkpoint ready for fresh native evidence.

It does not complete R2-3Z or the parent, claim native evidence, activate forwarding/provisioning,
or complete any R3 row.

TERMINAL RECEIPT

Send `codex.top-level-task-receipt.v1` with:

- `increment: "R2-3ZR1"`
- `packet_id: "A1.1d-5R2-3ZR1R1"`
- `next_increment: "EVIDENCE:R2-3Z"`

For success include landed commit/tree/ref, exact changed paths, WIP recovery disposition, subject
fingerprint, frozen-body/blob proof, canonical wall counts/hashes, validated review/digest,
finding disposition, reconciliation summary, GitNexus result, final clean/0-0 state, and explicit
non-claims.

The receipt send to the meta task must be the final tool action. Do not dispatch evidence or begin
R2-3Z.

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
