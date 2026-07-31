Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3ZM5. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 442b3f902a927a4ab09058d21fb92b7eb3163056133e0e72b39e5f27c06f5c44
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3ZM5
- packet_id: A1.1d-5R2-3ZM5
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
- /home/spenser/.codex/worktrees/3b8f/substrate-r2-3
- /home/spenser/.codex/worktrees/4712/substrate-r2-3
- /home/spenser/.codex/worktrees/acb8/substrate
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 610db8a9350c9b52496954f5c93232d885f439d9
- expected base tree: df4eb9869304a629a480dc47adbf79fd0ebf56b2
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3ZM5

This is the final source-changing R2-3 checkpoint before fresh native macOS and Windows evidence.
It owns only the remaining contextless macOS constructor surface and its typed pre-R3 smoke caller.

This top-level task and every subagent must run at Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning.

MISSION

Land one atomic macOS compatibility cleanup that:

1. removes public contextless `MacLimaBackend::new` and `new_with_vm_name` plus the `Default`
   implementation that delegates to the ambient constructor;
2. migrates only colocated tests/private helpers to explicit `new_with_mapping` construction;
3. converts `mac_backend_smoke` into an explicit typed pre-R3 contract validation example; and
4. updates its sole shell caller to project authenticated typed inputs without crossing any R3
   lifecycle, forwarding, readiness, endpoint, or execution gate.

BOUND SOURCE

- commit: `610db8a9350c9b52496954f5c93232d885f439d9`
- tree: `df4eb9869304a629a480dc47adbf79fd0ebf56b2`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`

Before editing, verify exact base/tree/ref/ancestor, clean task checkout, 0 ahead/0 behind, and the
current GitNexus index. Analyzer-only `AGENTS.md`/`CLAUDE.md` count churn must be restored to the
starting HEAD and excluded. Any other pre-existing diff is a hard stop.

EXACT TRACKED FILE ALLOWLIST

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/examples/mac_backend_smoke.rs`
- `scripts/mac/orchestration-smoke.sh`

No other Rust, shell, test, fixture, manifest, lockfile, runner, control-pack, orchestration, or
platform file may change.

PRESERVED PRIOR SUBJECT

The unpublished commit `199e1b1a5f3854934a08727f2846e028c065c5cb` in protected checkout
`/home/spenser/.codex/worktrees/3b8f/substrate-r2-3` contains an earlier reviewed implementation
of this three-file macOS subject mixed with host-inbox and control-record changes. It is read-only
historical evidence, not a patch to cherry-pick and not publication authority.

You may inspect that commit and checkout with read-only Git commands only. Do not run Cargo,
formatters, analyzers, tests, restoration, staging, commits, handoffs, or cleanup there. Re-derive
the authorized three-file subject on the current base, retaining only behavior supported by fresh
impact, source-closure, and proof. Do not copy its host-inbox or documentation changes.

EXACT RUST SCOPE

In `crates/world-mac-lima/src/lib.rs`, changes are limited to:

- removing public contextless `MacLimaBackend::new`;
- removing public contextless `MacLimaBackend::new_with_vm_name`;
- removing `Default for MacLimaBackend` because it delegates to the removed ambient constructor;
- adding the minimum private `#[cfg(test)]` typed-construction helper needed to replace those
  constructors in colocated tests/helpers;
- mechanically migrating every colocated contextless test caller to validated
  `new_with_mapping`; and
- minimum import/private cleanup made unreachable by those removals.

`MacLimaBackend::new_with_mapping`, typed mapping validation, `from_parts`, transport selection,
session lifecycle, VM lifecycle, forwarding, readiness, endpoint, execution, policy behavior,
drop/cleanup, and every R3-owned production body must remain behaviorally and textually stable
except unavoidable line movement. Do not introduce a renamed contextless constructor, fallback,
ambient factory, default mapping, or environment-derived compatibility path.

In `crates/world-mac-lima/examples/mac_backend_smoke.rs`, require exactly these explicit inputs:

- `--install-bootstrap-context-v1 <encoded carrier>`;
- `--platform-bootstrap-mapping-v1 <encoded mapping>`; and
- `--project-dir <absolute path>`.

Decode and validate with the shared `transport-api-types` types, construct only with
`MacLimaBackend::new_with_mapping`, report typed pre-R3 contract success, and exit. The example
must not call `ensure_session`, `exec`, VM/session/forwarding/readiness/endpoint APIs, or any other
R3-owned lifecycle surface. It must not derive authority from environment, CWD, repository paths,
defaults, Lima state, or ambient host state. Reject missing, duplicate, unknown, malformed, or
relative-path input.

EXACT SHELL-SCRIPT SCOPE

In `scripts/mac/orchestration-smoke.sh`, change only what is necessary to replace the no-argument
`mac_backend_smoke` invocation with explicit authenticated carrier/mapping/project projection and
to describe that leg truthfully as typed pre-R3 contract validation.

The projection must be derived from the already-routed, authenticated world-doctor/Lima facts and
the current account database, must fail closed on mismatch, and must not trust conflicting ambient
`HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, socket, VM-name, or forwarder values.
Prefer existing repository helpers and shared formats where practical; do not invent a second
product schema. Any temporary data must be outside the checkout, mode/cleanup safe, and removed by
an EXIT trap.

Preserve all existing warm/doctor commands and every other smoke test invocation. Do not activate,
repair, weaken, or bypass lifecycle/forwarding. `scripts/mac/smoke.sh`, `lima-warm.sh`, and
`lima-doctor.sh` are read-only and frozen.

IMPACT, SOURCE CLOSURE, AND FREEZE PROOF

Before editing every existing function or method, run file-qualified GitNexus upstream impact and
report direct callers, affected processes, modules, and risk. Warn before every HIGH/CRITICAL edit.

Fresh source closure on the bound base must prove:

1. every source caller of `MacLimaBackend::new` and `new_with_vm_name` is inside the exact
   three-file allowlist or is a colocated test migrated within `lib.rs`;
2. no production macOS or Windows contextless platform-factory consumer remains;
3. the sole shell caller of `mac_backend_smoke` is `scripts/mac/orchestration-smoke.sh`; and
4. no replacement ambient constructor or default appears after the patch.

Capture pre/post hashes or exact text comparisons for `new_with_mapping`, typed mapping
validation, `from_parts`, `ensure_vm_running`, `ensure_forwarding`, `ensure_agent_ready`,
`get_agent_endpoint`, `ensure_session`, `exec`, drop/cleanup, PI-077/PI-078 guard surfaces, and
other R3-owned bodies touched by source movement. Any semantic drift is a hard stop.

REQUIRED PROOF

Keep all logs, copied sources, and temporary harnesses outside the checkout.

1. `cargo fmt --all -- --check`.
2. `cargo check --locked -p world-mac-lima`.
3. `cargo test --locked -p world-mac-lima -- --nocapture`.
4. `cargo clippy --locked -p world-mac-lima --all-targets -- -D warnings`.
5. `cargo check --locked -p shell`.
6. Focused constructor/mapping tests, including malformed carrier/mapping, commitment mismatch,
   conflicting ambient values, explicit VM name/socket/control-root preservation, and proof that
   no lifecycle method is entered by the example.
7. A temporary external copied-source or cfg supplement that actually compiles and executes the
   macOS-gated typed-constructor/example parsing logic on Linux without changing behavior or
   repository files. Record the limitation that this is supplemental, not native evidence.
8. `bash -n scripts/mac/orchestration-smoke.sh` and focused script assertions proving the exact
   explicit argument projection and fail-closed conflicting-ambient behavior.
9. `bash tests/mac/prefix_mapping_r2_3.sh` as supplemental regression proof.
10. Attempt Apple-target static compilation honestly. An absent target/SDK/C toolchain is an
    environment limitation, never native evidence.
11. Do not rerun the full Linux shell-library wall. R2-3ZH1 already ran it on the exact current
    base `610db8a9350c9b52496954f5c93232d885f439d9` and established the accepted
    `1322 / 1274 / 48 / 0` result with failure-name SHA-256
    `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
    SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`.
    Two R2-3ZM5 reruns showed that executing the shared-state suite again produces unrelated,
    root/socket-sensitive fixture clusters even while the common accepted failures remain. Preserve
    both blocked logs as diagnostic evidence; do not bless their 100- or 76-failure inventories.
12. Replace that non-causal execution gate with all of the following:
    - run `cargo tree --locked -p shell --target x86_64-unknown-linux-gnu -e normal,build,dev` (or
      equivalent Cargo metadata proof) and show the modified macOS target code/example/script is
      absent from the Linux shell-library test executable dependency graph;
    - run `cargo test --locked -p shell --lib --no-run` successfully with private mode-0700 roots;
    - prove by cfg and exact-diff closure that `world-mac-lima` changes compile only for macOS and
      `scripts/mac/orchestration-smoke.sh` is not part of the Rust test executable; and
    - have independent review explicitly validate that inheriting the exact R2-3ZH1 wall is honest
      and that fresh native macOS evidence, not Linux shell fixture execution, owns behavior proof
      for this subject.
13. Prove manifests and `Cargo.lock` are unchanged; run `git diff --check`, exact three-file
    containment, and `gitnexus_detect_changes()` with every affected flow inspected.

Do not use or repair the deferred authenticated canonical-wall runner. Linux and cross-target
checks do not satisfy the fresh native evidence gate; the meta orchestrator dispatches that gate
only after this increment lands.

REVIEW AND PUBLICATION

Build a deterministic subject manifest and SHA-256 fingerprint. Use fresh independent read-only
GPT-5.4 Extra High reviewers at Standard/default speed covering:

1. constructor removal, typed mapping security, caller closure, and absence of ambient fallback;
2. pre-R3 example/script semantics, hostile-environment fail closure, and lifecycle/R3 byte freeze;
3. exact allowlist, inherited R2-3ZH1 wall honesty, Linux dependency-graph exclusion, proof
   limitations, and readiness for source-bound native evidence.

Validate the standard bounded review-cycle record. Publication requires terminal CLEAN, P1=0,
P2=0, complete P3/P4 disposition, all required proofs, and the live target still equal to the
expected base. Publish one normal fast-forward commit; never force-push. Verify remote commit/tree,
clean checkout, 0/0 divergence, and current landed GitNexus index.

COMPLETION BOUNDARY

Success completes only R2-3ZM5 and establishes the final source checkpoint for fresh native
evidence. It does not activate R3 lifecycle/forwarding/provisioning, claim native macOS/Windows
proof, edit the deferred canonical runner, repair the accepted 48 shell failures, execute R2-3Z,
complete broader runtime-refactor work, or complete any R3 row.

TERMINAL RECEIPT

Use `increment: "R2-3ZM5"`, `packet_id: "A1.1d-5R2-3ZM5"`, and
`next_increment: "EVIDENCE:R2-3Z"`. A LANDED_CLEAN receipt must include landed commit/tree/ref,
exact three changed paths, subject manifest/fingerprint, validated review record/digest, source
closure, frozen-body hashes, focused Rust/script proof, inherited R2-3ZH1 48-result wall reference,
Linux dependency-graph/no-run proof, both non-canonical wall diagnostic dispositions, cross-target
limitations, GitNexus result, remote 0/0, and clean checkout. For failure, send exact evidence,
required authority, and a complete handoff prompt. Receipt send is the final tool action.

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
